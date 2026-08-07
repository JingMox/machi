//! Rhai workflow engine with journaled host calls.

use std::sync::{Arc, Mutex};

use rhai::{Dynamic, Engine, EvalAltResult, Position};
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;

use crate::host::{AgentOpts, HostError, WorkflowHostRequest};
use crate::journal::{Journal, JournalError, request_hash};
use crate::run::{PauseKind, WorkflowOutcome};
use crate::{MAX_HOST_CALLS, MAX_PARALLEL};

/// Parameters for [`run_workflow`].
#[derive(Debug)]
pub struct WorkflowRunParams {
    /// Script source.
    pub script: String,
    /// Bound `args` global.
    pub args: serde_json::Value,
    /// Journal (may already contain entries for resume).
    pub journal: Journal,
    /// Host channel.
    pub host_tx: mpsc::UnboundedSender<WorkflowHostRequest>,
    /// Cancellation.
    pub cancel: CancellationToken,
    /// Max Rhai operations.
    pub max_ops: u64,
}

impl WorkflowRunParams {
    /// Default max ops.
    pub const DEFAULT_MAX_OPS: u64 = 100_000_000;
}

#[derive(Debug, Clone)]
enum ControlToken {
    Complete(serde_json::Value),
    Pause(PauseKind, String),
    Budget(String),
    Cancelled,
    Fatal(String),
}

struct Ctx {
    host_tx: mpsc::UnboundedSender<WorkflowHostRequest>,
    journal: Journal,
    seq: u64,
}

impl Ctx {
    fn next_seq(&mut self) -> Result<u64, Box<EvalAltResult>> {
        if self.seq >= MAX_HOST_CALLS {
            return Err(terminated(ControlToken::Fatal(
                "workflow exceeded max host calls".into(),
            )));
        }
        let seq = self.seq;
        self.seq += 1;
        Ok(seq)
    }
}

type ScriptResult<T> = Result<T, Box<EvalAltResult>>;

/// Run a workflow script to a terminal outcome.
#[must_use]
pub fn run_workflow(params: WorkflowRunParams) -> WorkflowOutcome {
    let WorkflowRunParams {
        script,
        args,
        journal,
        host_tx,
        cancel,
        max_ops,
    } = params;

    let ctx = Arc::new(Mutex::new(Ctx {
        host_tx,
        journal,
        seq: 0,
    }));

    let mut engine = Engine::new();
    engine.set_max_operations(max_ops);
    engine.set_max_call_levels(64);
    engine.set_module_resolver(rhai::module_resolvers::DummyModuleResolver::new());
    engine.disable_symbol("eval");
    engine.register_fn("timestamp", || -> ScriptResult<()> {
        Err(runtime_error(
            "timestamp() is unavailable: workflows must be deterministic",
        ))
    });
    engine.register_fn("sleep", |_s: i64| -> ScriptResult<()> {
        Err(runtime_error("sleep() is unavailable in workflow scripts"))
    });

    let cancel_flag = cancel.clone();
    engine.on_progress(move |_| {
        if cancel_flag.is_cancelled() {
            Some(Dynamic::from(ControlToken::Cancelled))
        } else {
            None
        }
    });

    register_fns(&mut engine, &ctx);

    let mut scope = rhai::Scope::new();
    let args_dyn = match rhai::serde::to_dynamic(&args) {
        Ok(d) => d,
        Err(e) => {
            return WorkflowOutcome::Failed {
                error: format!("invalid args: {e}"),
            };
        }
    };
    scope.push_dynamic("args", args_dyn);

    match engine.eval_with_scope::<Dynamic>(&mut scope, &script) {
        Ok(value) => WorkflowOutcome::Completed {
            result: dynamic_to_value(value),
        },
        Err(err) => outcome_from_error(*err),
    }
}

fn register_fns(engine: &mut Engine, ctx: &Arc<Mutex<Ctx>>) {
    let c = Arc::clone(ctx);
    engine.register_fn("agent", move |prompt: &str| -> ScriptResult<Dynamic> {
        spawn_agent(
            &c,
            AgentOpts {
                prompt: prompt.to_owned(),
                ..AgentOpts::default()
            },
        )
    });

    let c = Arc::clone(ctx);
    engine.register_fn(
        "agent",
        move |prompt: &str, opts: rhai::Map| -> ScriptResult<Dynamic> {
            let mut agent_opts = agent_opts_from_map(opts)?;
            if agent_opts.prompt.is_empty() {
                agent_opts.prompt = prompt.to_owned();
            }
            spawn_agent(&c, agent_opts)
        },
    );

    let c = Arc::clone(ctx);
    engine.register_fn(
        "parallel",
        move |items: rhai::Array| -> ScriptResult<rhai::Array> { spawn_agents_parallel(&c, items) },
    );

    let c = Arc::clone(ctx);
    engine.register_fn("phase", move |title: &str| {
        let (tx, replaying) = {
            let ctx = c.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            (ctx.host_tx.clone(), ctx.journal.covers(ctx.seq))
        };
        let _ = tx.send(WorkflowHostRequest::Phase {
            title: title.to_owned(),
            replayed: replaying,
        });
    });

    let c = Arc::clone(ctx);
    engine.register_fn("log", move |message: &str| {
        let (tx, replaying) = {
            let ctx = c.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            (ctx.host_tx.clone(), ctx.journal.covers(ctx.seq))
        };
        let _ = tx.send(WorkflowHostRequest::Log {
            message: message.to_owned(),
            replayed: replaying,
        });
    });

    engine.register_fn("complete", |value: Dynamic| -> ScriptResult<()> {
        Err(terminated(ControlToken::Complete(dynamic_to_value(value))))
    });

    engine.register_fn("pause", |kind: &str, message: &str| -> ScriptResult<()> {
        let kind = match kind {
            "user" => PauseKind::User,
            "back_off" | "backoff" => PauseKind::BackOff,
            "no_progress" => PauseKind::NoProgress,
            "verification" | "blocked" => PauseKind::Verification,
            "infra" => PauseKind::Infra,
            other => {
                return Err(runtime_error(format!("unknown pause kind: {other}")));
            }
        };
        Err(terminated(ControlToken::Pause(kind, message.to_owned())))
    });
}

enum ParallelSlot {
    Replayed(serde_json::Value),
    Pending {
        opts: AgentOpts,
        seq: u64,
        hash: String,
    },
    Live {
        seq: u64,
        hash: String,
        reply_rx: oneshot::Receiver<Result<crate::host::AgentResult, HostError>>,
    },
}

/// Fan-out: reserve live slots once, dispatch all host spawns, wait as a barrier.
fn spawn_agents_parallel(ctx: &Arc<Mutex<Ctx>>, items: rhai::Array) -> ScriptResult<rhai::Array> {
    if items.len() > MAX_PARALLEL {
        return Err(runtime_error(format!(
            "parallel() accepts at most {MAX_PARALLEL} items"
        )));
    }

    let mut prepared: Vec<(AgentOpts, String, u64)> = Vec::with_capacity(items.len());
    for item in items {
        let map = item
            .try_cast::<rhai::Map>()
            .ok_or_else(|| runtime_error("parallel() items must be maps"))?;
        let opts = agent_opts_from_map(map)?;
        let payload = serde_json::to_value(&opts)
            .map_err(|e| runtime_error(format!("invalid agent options: {e}")))?;
        let hash = request_hash("spawn_agent", &payload);
        let seq = {
            let mut g = ctx
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            g.next_seq()?
        };
        prepared.push((opts, hash, seq));
    }

    let mut pending: Vec<ParallelSlot> = Vec::with_capacity(prepared.len());
    let mut live_count = 0u64;
    for (opts, hash, seq) in prepared {
        let replayed = {
            let g = ctx
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            g.journal
                .replay(seq, "spawn_agent", &hash)
                .map_err(journal_fatal)?
        };
        if let Some(value) = replayed {
            pending.push(ParallelSlot::Replayed(value));
        } else {
            live_count = live_count.saturating_add(1);
            pending.push(ParallelSlot::Pending { opts, seq, hash });
        }
    }

    // Reserve before any live spawn so budget failure never races partial fan-out.
    reserve_n(ctx, live_count)?;

    let mut slots: Vec<ParallelSlot> = Vec::with_capacity(pending.len());
    for slot in pending {
        match slot {
            ParallelSlot::Replayed(v) => slots.push(ParallelSlot::Replayed(v)),
            ParallelSlot::Pending { opts, seq, hash } => {
                let (reply_tx, reply_rx) = oneshot::channel();
                {
                    let g = ctx
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    g.host_tx
                        .send(WorkflowHostRequest::SpawnAgent {
                            opts,
                            reply: reply_tx,
                        })
                        .map_err(|_| {
                            terminated(ControlToken::Fatal("workflow host channel closed".into()))
                        })?;
                }
                slots.push(ParallelSlot::Live {
                    seq,
                    hash,
                    reply_rx,
                });
            }
            ParallelSlot::Live { .. } => {
                return Err(terminated(ControlToken::Fatal(
                    "internal: live slot before dispatch".into(),
                )));
            }
        }
    }

    let mut results = rhai::Array::with_capacity(slots.len());
    for slot in slots {
        match slot {
            ParallelSlot::Replayed(value) => results.push(value_to_dynamic(&value)?),
            ParallelSlot::Pending { .. } => {
                return Err(terminated(ControlToken::Fatal(
                    "internal: pending slot after dispatch".into(),
                )));
            }
            ParallelSlot::Live {
                seq,
                hash,
                reply_rx,
            } => {
                let reply = reply_rx.blocking_recv().map_err(|_| {
                    terminated(ControlToken::Fatal("workflow host dropped reply".into()))
                })?;
                let value = map_spawn_reply(reply)?;
                {
                    let mut g = ctx
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    g.journal
                        .record(seq, "spawn_agent", hash, value.clone())
                        .map_err(journal_fatal)?;
                }
                results.push(value_to_dynamic(&value)?);
            }
        }
    }
    Ok(results)
}

fn map_spawn_reply(
    reply: Result<crate::host::AgentResult, HostError>,
) -> ScriptResult<serde_json::Value> {
    match reply {
        Ok(result) => Ok(serde_json::to_value(result).unwrap_or(serde_json::Value::Null)),
        Err(HostError::BudgetExceeded | HostError::AgentCallQuotaExceeded { .. }) => {
            Err(terminated(ControlToken::Budget(
                "workflow agent budget exceeded".into(),
            )))
        }
        Err(HostError::Cancelled) => Err(terminated(ControlToken::Cancelled)),
        Err(HostError::Unsupported(msg) | HostError::Failed(msg)) => Err(runtime_error(msg)),
    }
}

fn spawn_agent(ctx: &Arc<Mutex<Ctx>>, opts: AgentOpts) -> ScriptResult<Dynamic> {
    let payload = serde_json::to_value(&opts)
        .map_err(|e| runtime_error(format!("invalid agent options: {e}")))?;
    let hash = request_hash("spawn_agent", &payload);
    let seq = {
        let mut g = ctx
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        g.next_seq()?
    };

    {
        let g = ctx
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(recorded) = g
            .journal
            .replay(seq, "spawn_agent", &hash)
            .map_err(journal_fatal)?
        {
            return value_to_dynamic(&recorded);
        }
    }

    reserve_one(ctx)?;

    let (reply_tx, reply_rx) = oneshot::channel();
    {
        let g = ctx
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        g.host_tx
            .send(WorkflowHostRequest::SpawnAgent {
                opts,
                reply: reply_tx,
            })
            .map_err(|_| terminated(ControlToken::Fatal("workflow host channel closed".into())))?;
    }

    let reply = reply_rx
        .blocking_recv()
        .map_err(|_| terminated(ControlToken::Fatal("workflow host dropped reply".into())))?;

    let value = map_spawn_reply(reply)?;

    {
        let mut g = ctx
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        g.journal
            .record(seq, "spawn_agent", hash, value.clone())
            .map_err(journal_fatal)?;
    }
    value_to_dynamic(&value)
}

fn reserve_one(ctx: &Arc<Mutex<Ctx>>) -> ScriptResult<()> {
    reserve_n(ctx, 1)
}

fn reserve_n(ctx: &Arc<Mutex<Ctx>>, count: u64) -> ScriptResult<()> {
    if count == 0 {
        return Ok(());
    }
    let (reply_tx, reply_rx) = oneshot::channel();
    {
        let g = ctx
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        g.host_tx
            .send(WorkflowHostRequest::ReserveAgentCalls {
                count,
                reply: reply_tx,
            })
            .map_err(|_| terminated(ControlToken::Fatal("workflow host channel closed".into())))?;
    }
    match reply_rx
        .blocking_recv()
        .map_err(|_| terminated(ControlToken::Fatal("workflow host dropped reply".into())))?
    {
        Ok(()) => Ok(()),
        Err(HostError::BudgetExceeded | HostError::AgentCallQuotaExceeded { .. }) => {
            Err(terminated(ControlToken::Budget(
                "workflow agent budget exceeded".into(),
            )))
        }
        Err(HostError::Cancelled) => Err(terminated(ControlToken::Cancelled)),
        Err(HostError::Unsupported(msg) | HostError::Failed(msg)) => Err(runtime_error(msg)),
    }
}

fn agent_opts_from_map(map: rhai::Map) -> ScriptResult<AgentOpts> {
    let value = rhai::serde::from_dynamic::<serde_json::Value>(&Dynamic::from_map(map))
        .map_err(|e| runtime_error(format!("invalid options map: {e}")))?;
    serde_json::from_value(value).map_err(|e| runtime_error(format!("invalid agent options: {e}")))
}

fn dynamic_to_value(d: Dynamic) -> serde_json::Value {
    rhai::serde::from_dynamic(&d).unwrap_or(serde_json::Value::Null)
}

fn value_to_dynamic(v: &serde_json::Value) -> ScriptResult<Dynamic> {
    rhai::serde::to_dynamic(v).map_err(|e| runtime_error(format!("host result conversion: {e}")))
}

fn terminated(token: ControlToken) -> Box<EvalAltResult> {
    Box::new(EvalAltResult::ErrorTerminated(
        Dynamic::from(token),
        Position::NONE,
    ))
}

fn runtime_error(message: impl Into<String>) -> Box<EvalAltResult> {
    Box::new(EvalAltResult::ErrorRuntime(
        Dynamic::from(message.into()),
        Position::NONE,
    ))
}

fn journal_fatal(error: JournalError) -> Box<EvalAltResult> {
    terminated(ControlToken::Fatal(error.to_string()))
}

fn find_control_token(err: &EvalAltResult) -> Option<ControlToken> {
    match err {
        EvalAltResult::ErrorTerminated(token, _) => token.clone().try_cast::<ControlToken>(),
        EvalAltResult::ErrorInFunctionCall(_, _, inner, _) => find_control_token(inner),
        EvalAltResult::ErrorInModule(_, inner, _) => find_control_token(inner),
        _ => None,
    }
}

fn outcome_from_error(err: EvalAltResult) -> WorkflowOutcome {
    if let Some(token) = find_control_token(&err) {
        return match token {
            ControlToken::Complete(result) => WorkflowOutcome::Completed { result },
            ControlToken::Pause(kind, message) => WorkflowOutcome::Paused { kind, message },
            ControlToken::Budget(message) => WorkflowOutcome::BudgetExceeded { message },
            ControlToken::Cancelled => WorkflowOutcome::Cancelled,
            ControlToken::Fatal(error) => WorkflowOutcome::Failed { error },
        };
    }
    WorkflowOutcome::Failed {
        error: err.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;
    use crate::host::{AgentResult, WorkflowHostRequest};

    fn spawn_host(
        budget: u64,
    ) -> (
        mpsc::UnboundedSender<WorkflowHostRequest>,
        tokio::task::JoinHandle<()>,
        Arc<AtomicU64>,
    ) {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let spent = Arc::new(AtomicU64::new(0));
        let spent2 = spent.clone();
        let handle = tokio::task::spawn(async move {
            let mut reserved = 0u64;
            while let Some(req) = rx.recv().await {
                match req {
                    WorkflowHostRequest::ReserveAgentCalls { count, reply } => {
                        if reserved + count + spent2.load(Ordering::SeqCst) > budget {
                            let _ = reply.send(Err(HostError::BudgetExceeded));
                        } else {
                            reserved += count;
                            let _ = reply.send(Ok(()));
                        }
                    }
                    WorkflowHostRequest::ReleaseAgentCalls { count, reply } => {
                        reserved = reserved.saturating_sub(count);
                        let _ = reply.send(Ok(()));
                    }
                    WorkflowHostRequest::SpawnAgent { opts, reply } => {
                        spent2.fetch_add(1, Ordering::SeqCst);
                        reserved = reserved.saturating_sub(1);
                        let _ = reply.send(Ok(AgentResult {
                            agent_id: format!("a-{}", spent2.load(Ordering::SeqCst)),
                            success: true,
                            output: serde_json::json!({"prompt": opts.prompt}),
                            cancelled: false,
                            tokens_used: 1,
                            duration_ms: 1,
                        }));
                    }
                    WorkflowHostRequest::Phase { .. } | WorkflowHostRequest::Log { .. } => {}
                    WorkflowHostRequest::BudgetQuery { reply } => {
                        let spent = spent2.load(Ordering::SeqCst);
                        let _ = reply.send(Ok(crate::host::BudgetState {
                            total: Some(budget),
                            spent,
                            reserved,
                            remaining: Some(budget.saturating_sub(spent + reserved)),
                        }));
                    }
                }
            }
        });
        (tx, handle, spent)
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn agent_and_complete() {
        let (tx, host, spent) = spawn_host(10);
        let script = r#"
            let meta = #{ name: "t", description: "t" };
            phase("go");
            let r = agent("hello");
            complete(r);
        "#;
        let outcome = tokio::task::spawn_blocking(move || {
            run_workflow(WorkflowRunParams {
                script: script.into(),
                args: serde_json::json!({}),
                journal: Journal::new(None),
                host_tx: tx,
                cancel: CancellationToken::new(),
                max_ops: WorkflowRunParams::DEFAULT_MAX_OPS,
            })
        })
        .await
        .expect("join");
        drop(host);
        assert!(
            matches!(outcome, WorkflowOutcome::Completed { .. }),
            "{outcome:?}"
        );
        assert_eq!(spent.load(Ordering::SeqCst), 1);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn journal_resume_skips_first_agent() {
        let dir = tempfile::tempdir().expect("tmp");
        let path = dir.path().join("j.jsonl");
        let (tx, host, spent) = spawn_host(10);
        let script = r#"
            let meta = #{ name: "t", description: "t" };
            let a = agent("one");
            let b = agent("two");
            complete(#{ a: a, b: b });
        "#;
        let outcome1 = {
            let tx = tx.clone();
            let path = path.clone();
            let script = script.to_owned();
            tokio::task::spawn_blocking(move || {
                run_workflow(WorkflowRunParams {
                    script,
                    args: serde_json::json!({}),
                    journal: Journal::new(Some(path)),
                    host_tx: tx,
                    cancel: CancellationToken::new(),
                    max_ops: WorkflowRunParams::DEFAULT_MAX_OPS,
                })
            })
            .await
            .expect("join")
        };
        assert!(matches!(outcome1, WorkflowOutcome::Completed { .. }));
        assert_eq!(spent.load(Ordering::SeqCst), 2);

        // Resume: both agent calls should replay from journal (no new spawns).
        spent.store(0, Ordering::SeqCst);
        let journal = Journal::load(path).expect("load");
        assert_eq!(journal.len(), 2);
        let outcome2 = tokio::task::spawn_blocking(move || {
            run_workflow(WorkflowRunParams {
                script: script.into(),
                args: serde_json::json!({}),
                journal,
                host_tx: tx,
                cancel: CancellationToken::new(),
                max_ops: WorkflowRunParams::DEFAULT_MAX_OPS,
            })
        })
        .await
        .expect("join");
        drop(host);
        assert!(matches!(outcome2, WorkflowOutcome::Completed { .. }));
        assert_eq!(
            spent.load(Ordering::SeqCst),
            0,
            "resume must not re-spawn agents"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn budget_exceeded() {
        let (tx, host, _) = spawn_host(0);
        let script = r#"
            let meta = #{ name: "t", description: "t" };
            agent("x");
            complete(1);
        "#;
        let outcome = tokio::task::spawn_blocking(move || {
            run_workflow(WorkflowRunParams {
                script: script.into(),
                args: serde_json::json!({}),
                journal: Journal::new(None),
                host_tx: tx,
                cancel: CancellationToken::new(),
                max_ops: WorkflowRunParams::DEFAULT_MAX_OPS,
            })
        })
        .await
        .expect("join");
        drop(host);
        assert!(
            matches!(outcome, WorkflowOutcome::BudgetExceeded { .. }),
            "{outcome:?}"
        );
    }
}
