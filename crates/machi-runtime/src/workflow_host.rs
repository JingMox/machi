//! Adapter: `machi-workflow` host channel → [`SessionHost`] nested runs.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use machi_obs::{NoopMetrics, SharedMetrics, record_workflow_agents, record_workflow_run};
use machi_tools::registry::CapabilityMode;
use machi_workflow::{
    AgentOpts, AgentResult, BudgetState, HostError, WorkflowHostRequest, WorkflowOutcome,
    WorkflowRunParams, run_workflow,
};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{Instrument, info_span};

use crate::host::{SessionHost, SpawnOpts};
use crate::side_effects::WorkflowSideEffects;

/// Run a workflow script whose `agent` / `parallel` calls resolve through `host`.
///
/// Blocks the calling async task on a worker thread for the Rhai engine while
/// servicing host requests on the current runtime.
///
/// # Errors
///
/// Propagates channel / join failures as [`HostError::Failed`]. Workflow
/// terminal outcomes are returned as [`WorkflowOutcome`] (including
/// `Failed` / `BudgetExceeded` variants) rather than `Err`.
pub async fn run_workflow_on_host(
    host: Arc<dyn SessionHost>,
    params: WorkflowRunParams,
    agent_budget: Option<u64>,
) -> Result<WorkflowOutcome, HostError> {
    run_workflow_on_host_with_metrics(host, params, agent_budget, Arc::new(NoopMetrics)).await
}

/// Like [`run_workflow_on_host`] with an explicit metrics sink.
///
/// # Errors
///
/// Same as [`run_workflow_on_host`].
pub async fn run_workflow_on_host_with_metrics(
    host: Arc<dyn SessionHost>,
    params: WorkflowRunParams,
    agent_budget: Option<u64>,
    metrics: SharedMetrics,
) -> Result<WorkflowOutcome, HostError> {
    run_workflow_configured(
        host,
        params,
        agent_budget,
        metrics,
        WorkflowSideEffects::shared(),
    )
    .await
}

/// Full configuration: metrics + side-effect store (scratch / templates).
///
/// # Errors
///
/// Same as [`run_workflow_on_host`].
pub async fn run_workflow_configured(
    host: Arc<dyn SessionHost>,
    mut params: WorkflowRunParams,
    agent_budget: Option<u64>,
    metrics: SharedMetrics,
    effects: Arc<WorkflowSideEffects>,
) -> Result<WorkflowOutcome, HostError> {
    let (tx, mut rx) = mpsc::unbounded_channel::<WorkflowHostRequest>();
    let spent = Arc::new(AtomicU64::new(0));
    let reserved = Arc::new(AtomicU64::new(0));
    let cancel = params.cancel.clone();

    let budget = agent_budget;
    let spent_h = Arc::clone(&spent);
    let reserved_h = Arc::clone(&reserved);
    let cancel_h = cancel.clone();
    let host_svc = Arc::clone(&host);
    let effects_svc = Arc::clone(&effects);

    let service = tokio::spawn(async move {
        let mut inflight = Vec::new();
        while let Some(req) = rx.recv().await {
            if cancel_h.is_cancelled() {
                reply_cancelled(req);
                continue;
            }
            // SpawnAgent runs concurrent so parallel() fan-out is real concurrency.
            // Other requests are cheap and handled inline.
            match req {
                WorkflowHostRequest::SpawnAgent { opts, reply } => {
                    let host = Arc::clone(&host_svc);
                    let spent = Arc::clone(&spent_h);
                    let reserved = Arc::clone(&reserved_h);
                    let cancel = cancel_h.clone();
                    inflight.push(tokio::spawn(async move {
                        handle_spawn(host.as_ref(), opts, reply, &spent, &reserved, &cancel).await;
                    }));
                }
                other => {
                    dispatch_inline(
                        other,
                        budget,
                        &spent_h,
                        &reserved_h,
                        effects_svc.as_ref(),
                    );
                }
            }
        }
        for t in inflight {
            let _ = t.await;
        }
    });

    params.host_tx = tx;
    let outcome = tokio::task::spawn_blocking(move || run_workflow(params))
        .await
        .map_err(|e| HostError::Failed(format!("workflow join: {e}")))?;

    // Dropping the sender (inside run_workflow when it finishes) ends the service loop.
    let _ = service.await;

    let spent_n = spent.load(Ordering::Relaxed);
    record_workflow_agents(metrics.as_ref(), spent_n);
    record_workflow_run(metrics.as_ref(), outcome_label(&outcome));
    Ok(outcome)
}

fn outcome_label(outcome: &WorkflowOutcome) -> &'static str {
    match outcome {
        WorkflowOutcome::Completed { .. } => "completed",
        WorkflowOutcome::Paused { .. } => "paused",
        WorkflowOutcome::BudgetExceeded { .. } => "budget_exceeded",
        WorkflowOutcome::Cancelled => "cancelled",
        WorkflowOutcome::Failed { .. } => "failed",
        _ => "other",
    }
}

async fn handle_spawn(
    host: &dyn SessionHost,
    opts: AgentOpts,
    reply: tokio::sync::oneshot::Sender<Result<AgentResult, HostError>>,
    spent: &AtomicU64,
    reserved: &AtomicU64,
    cancel: &CancellationToken,
) {
    let span = info_span!(
        "machi.workflow.host",
        machi.workflow.kind = "spawn_agent",
        machi.agent_label = opts.label.as_deref().unwrap_or(""),
    );
    let result = async {
        if cancel.is_cancelled() {
            return Err(HostError::Cancelled);
        }
        let spawn = to_spawn_opts(opts, cancel.child_token());
        match host.spawn_agent(spawn).await {
            Ok(run) => {
                spent.fetch_add(1, Ordering::Relaxed);
                let r = reserved.load(Ordering::Relaxed);
                reserved.fetch_sub(r.min(1), Ordering::Relaxed);
                let tokens = u64::from(run.usage.total_tokens);
                Ok(AgentResult {
                    agent_id: run.agent_id.to_string(),
                    success: run.success && !run.cancelled,
                    output: run.output,
                    cancelled: run.cancelled,
                    tokens_used: tokens,
                    duration_ms: run.duration_ms,
                })
            }
            Err(e) => {
                let r = reserved.load(Ordering::Relaxed);
                reserved.fetch_sub(r.min(1), Ordering::Relaxed);
                if e.code() == machi_types::ErrorCode::HostBudget {
                    Err(HostError::BudgetExceeded)
                } else if e.code() == machi_types::ErrorCode::HostCancelled {
                    Err(HostError::Cancelled)
                } else {
                    Err(HostError::Failed(e.to_string()))
                }
            }
        }
    }
    .instrument(span)
    .await;
    let _ = reply.send(result);
}

fn dispatch_inline(
    req: WorkflowHostRequest,
    budget: Option<u64>,
    spent: &AtomicU64,
    reserved: &AtomicU64,
    effects: &WorkflowSideEffects,
) {
    match req {
        WorkflowHostRequest::ReserveAgentCalls { count, reply } => {
            let result = reserve(budget, spent, reserved, count);
            let _ = reply.send(result);
        }
        WorkflowHostRequest::ReleaseAgentCalls { count, reply } => {
            let r = reserved.load(Ordering::Relaxed);
            reserved.fetch_sub(count.min(r), Ordering::Relaxed);
            let _ = reply.send(Ok(()));
        }
        WorkflowHostRequest::SpawnAgent { reply, .. } => {
            // Concurrent path is handled in the service loop.
            let _ = reply.send(Err(HostError::Failed(
                "internal: SpawnAgent must be handled concurrently".into(),
            )));
        }
        WorkflowHostRequest::BudgetQuery { reply } => {
            let s = spent.load(Ordering::Relaxed);
            let r = reserved.load(Ordering::Relaxed);
            let state = BudgetState {
                total: budget,
                spent: s,
                reserved: r,
                remaining: budget.map(|b| b.saturating_sub(s.saturating_add(r))),
            };
            let _ = reply.send(Ok(state));
        }
        WorkflowHostRequest::Phase { title, replayed } => {
            tracing::info!(target: "machi.workflow", %title, replayed, "phase");
        }
        WorkflowHostRequest::Log { message, replayed } => {
            tracing::info!(target: "machi.workflow", %message, replayed, "log");
        }
        WorkflowHostRequest::Telemetry {
            name,
            fields,
            replayed,
        } => {
            tracing::info!(target: "machi.workflow", %name, %fields, replayed, "telemetry");
        }
        WorkflowHostRequest::RenderTemplate { reply, name, vars } => {
            let _ = reply.send(effects.render_template(&name, &vars));
        }
        WorkflowHostRequest::WriteScratchFile {
            reply,
            name,
            content,
        } => {
            let _ = reply.send(effects.write_scratch(&name, content));
        }
        WorkflowHostRequest::ReadScratchFile { reply, name } => {
            let _ = reply.send(effects.read_scratch(&name));
        }
        WorkflowHostRequest::GitDiffSince { reply, commit } => {
            let _ = reply.send(effects.git_diff_since(&commit));
        }
    }
}

fn reserve(
    budget: Option<u64>,
    spent: &AtomicU64,
    reserved: &AtomicU64,
    count: u64,
) -> Result<(), HostError> {
    if let Some(max) = budget {
        loop {
            let s = spent.load(Ordering::Acquire);
            let r = reserved.load(Ordering::Acquire);
            if s.saturating_add(r).saturating_add(count) > max {
                return Err(HostError::AgentCallQuotaExceeded {
                    requested: s.saturating_add(r).saturating_add(count),
                    maximum: max,
                });
            }
            if reserved
                .compare_exchange(
                    r,
                    r.saturating_add(count),
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .is_ok()
            {
                return Ok(());
            }
        }
    }
    reserved.fetch_add(count, Ordering::Relaxed);
    Ok(())
}

fn to_spawn_opts(opts: AgentOpts, cancel: CancellationToken) -> SpawnOpts {
    let mut spawn = SpawnOpts::new(opts.prompt).with_cancel(cancel);
    if let Some(label) = opts.label {
        spawn = spawn.with_label(label);
    }
    if let Some(model) = opts.model {
        spawn.model = Some(model);
    }
    if let Some(mode) = opts.capability_mode.as_deref() {
        spawn.capability_mode = parse_capability(mode);
    }
    spawn
}

fn parse_capability(mode: &str) -> CapabilityMode {
    match mode {
        "read_only" | "read-only" | "readonly" => CapabilityMode::ReadOnly,
        "plan" => CapabilityMode::Plan,
        _ => CapabilityMode::Full,
    }
}

fn reply_cancelled(req: WorkflowHostRequest) {
    match req {
        WorkflowHostRequest::ReserveAgentCalls { reply, .. }
        | WorkflowHostRequest::ReleaseAgentCalls { reply, .. } => {
            let _ = reply.send(Err(HostError::Cancelled));
        }
        WorkflowHostRequest::SpawnAgent { reply, .. } => {
            let _ = reply.send(Err(HostError::Cancelled));
        }
        WorkflowHostRequest::BudgetQuery { reply } => {
            let _ = reply.send(Err(HostError::Cancelled));
        }
        WorkflowHostRequest::RenderTemplate { reply, .. }
        | WorkflowHostRequest::WriteScratchFile { reply, .. }
        | WorkflowHostRequest::ReadScratchFile { reply, .. }
        | WorkflowHostRequest::GitDiffSince { reply, .. } => {
            let _ = reply.send(Err(HostError::Cancelled));
        }
        WorkflowHostRequest::Phase { .. }
        | WorkflowHostRequest::Log { .. }
        | WorkflowHostRequest::Telemetry { .. } => {}
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use machi_llm::MockSampler;
    use machi_workflow::{Journal, WorkflowOutcome, WorkflowRunParams};
    use tokio_util::sync::CancellationToken;

    use super::*;
    use crate::host::InProcessHost;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn workflow_parallel_on_session_host() {
        let sampler = Arc::new(MockSampler::new());
        sampler.map_user_text("a", "from-a");
        sampler.map_user_text("b", "from-b");
        let host: Arc<dyn SessionHost> = Arc::new(InProcessHost::new(sampler, vec![]));
        let script = r#"
            let meta = #{ name: "fanout", description: "test" };
            phase("work");
            let rs = parallel([
                #{ prompt: "a", label: "wa" },
                #{ prompt: "b", label: "wb" },
            ]);
            complete(#{ results: rs });
        "#;
        let (tx, _rx) = mpsc::unbounded_channel();
        let outcome = run_workflow_on_host(
            host,
            WorkflowRunParams {
                script: script.into(),
                args: serde_json::json!({}),
                journal: Journal::new(None),
                host_tx: tx,
                cancel: CancellationToken::new(),
                max_ops: WorkflowRunParams::DEFAULT_MAX_OPS,
            },
            Some(16),
        )
        .await
        .expect("run");
        let WorkflowOutcome::Completed { result } = outcome else {
            unreachable!("expected completed outcome");
        };
        let arr = result
            .get("results")
            .and_then(|v| v.as_array())
            .expect("results array");
        assert_eq!(arr.len(), 2);
        assert_eq!(
            arr.first().and_then(|v| v.get("output")),
            Some(&serde_json::json!("from-a"))
        );
        assert_eq!(
            arr.get(1).and_then(|v| v.get("output")),
            Some(&serde_json::json!("from-b"))
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn workflow_budget_on_host() {
        let sampler = Arc::new(MockSampler::new());
        sampler.push_text("x");
        let host: Arc<dyn SessionHost> =
            Arc::new(InProcessHost::new(sampler, vec![]).with_agent_budget(0));
        // Budget 0 at adapter reserve layer
        let script = r#"
            let meta = #{ name: "b", description: "b" };
            agent("x");
            complete(1);
        "#;
        let (tx, _rx) = mpsc::unbounded_channel();
        let outcome = run_workflow_on_host(
            host,
            WorkflowRunParams {
                script: script.into(),
                args: serde_json::json!({}),
                journal: Journal::new(None),
                host_tx: tx,
                cancel: CancellationToken::new(),
                max_ops: WorkflowRunParams::DEFAULT_MAX_OPS,
            },
            Some(0),
        )
        .await
        .expect("run");
        assert!(
            matches!(outcome, WorkflowOutcome::BudgetExceeded { .. }),
            "{outcome:?}"
        );
    }
}
