//! Concurrent tool dispatch with exclusivity rules.

use std::time::Duration;

use futures::future::join_all;
use machi_types::{ToolCall, ToolCallId};
use tokio::time::timeout;
use tracing::{Instrument, info_span};

use crate::context::ToolCallContext;
use crate::error::{ToolError, codes};
use crate::metadata::ConcurrencyMode;
use crate::registry::{CapabilityMode, ToolRegistry};
use crate::tool::{DynTool, SharedTool, ToolResult};

/// One tool call to execute.
#[derive(Debug, Clone)]
pub struct DispatchRequest {
    /// Model tool call.
    pub call: ToolCall,
}

/// Outcome for a single dispatched call.
#[derive(Debug, Clone)]
pub struct DispatchOutcome {
    /// Call id.
    pub id: ToolCallId,
    /// Tool name.
    pub name: String,
    /// Result or error mapped for the model.
    pub result: Result<ToolResult, ToolError>,
}

/// Scheduler for tool batches.
#[derive(Debug, Clone, Copy)]
pub struct ToolDispatch {
    /// Maximum concurrent non-exclusive tools.
    pub max_concurrency: usize,
    /// Capability filter applied before execution.
    pub capability_mode: CapabilityMode,
}

impl Default for ToolDispatch {
    fn default() -> Self {
        Self {
            max_concurrency: 32,
            capability_mode: CapabilityMode::Full,
        }
    }
}

impl ToolDispatch {
    /// Execute a batch preserving input order in the output vector.
    pub async fn execute_batch(
        &self,
        registry: &ToolRegistry,
        ctx: ToolCallContext,
        requests: Vec<DispatchRequest>,
    ) -> Vec<DispatchOutcome> {
        if requests.is_empty() {
            return Vec::new();
        }

        let mut outcomes: Vec<Option<DispatchOutcome>> =
            (0..requests.len()).map(|_| None).collect();
        let mut index = 0usize;

        while index < requests.len() {
            if ctx.is_cancelled() {
                fill_cancelled(&requests, &mut outcomes, index);
                break;
            }

            let Some(req) = requests.get(index) else {
                break;
            };

            match prepare_call(registry, self.capability_mode, req) {
                Prepare::Deny(out) | Prepare::Missing(out) => {
                    if let Some(slot) = outcomes.get_mut(index) {
                        *slot = Some(out);
                    }
                    index = index.saturating_add(1);
                }
                Prepare::Ready(tool) => {
                    let meta = tool.metadata();
                    if meta.concurrency == ConcurrencyMode::Exclusive {
                        let out = self.run_one(tool.as_ref(), ctx.clone(), req).await;
                        if let Some(slot) = outcomes.get_mut(index) {
                            *slot = Some(out);
                        }
                        index = index.saturating_add(1);
                        continue;
                    }

                    let window = collect_concurrent_window(
                        registry,
                        self.capability_mode,
                        &requests,
                        index,
                        self.max_concurrency.max(1),
                    );
                    let next = window.last().map_or(index + 1, |i| i.saturating_add(1));
                    let futs = window.into_iter().filter_map(|win_i| {
                        let win_req = requests.get(win_i)?.clone();
                        let win_tool = registry.require(&win_req.call.name).ok()?;
                        let win_ctx = ctx.clone();
                        Some(async move {
                            (
                                win_i,
                                self.run_one(win_tool.as_ref(), win_ctx, &win_req).await,
                            )
                        })
                    });
                    for (i, out) in join_all(futs).await {
                        if let Some(slot) = outcomes.get_mut(i) {
                            *slot = Some(out);
                        }
                    }
                    index = next;
                }
            }
        }

        finalize_outcomes(&requests, outcomes)
    }

    async fn run_one(
        &self,
        tool: &dyn DynTool,
        ctx: ToolCallContext,
        req: &DispatchRequest,
    ) -> DispatchOutcome {
        let span = info_span!(
            "machi.tool",
            machi.tool_name = tool.name(),
            machi.tool_call_id = %req.call.id,
        );
        let meta = tool.metadata();
        let fut = tool.call(ctx.clone(), req.call.arguments.clone());
        let result = async {
            if ctx.is_cancelled() {
                return Err(codes::cancelled());
            }
            let limit = meta
                .timeout
                .or_else(|| ctx.deadline.map(|d| d.remaining()).filter(|d| !d.is_zero()));
            match limit {
                Some(limit) => match timeout(limit.max(Duration::from_millis(1)), fut).await {
                    Ok(r) => r,
                    Err(_) => Err(codes::timeout(format!("tool '{}' timed out", tool.name()))),
                },
                None => fut.await,
            }
        }
        .instrument(span)
        .await;

        DispatchOutcome {
            id: req.call.id.clone(),
            name: req.call.name.clone(),
            result,
        }
    }
}

enum Prepare {
    Ready(SharedTool),
    Missing(DispatchOutcome),
    Deny(DispatchOutcome),
}

fn prepare_call(registry: &ToolRegistry, mode: CapabilityMode, req: &DispatchRequest) -> Prepare {
    match registry.require(&req.call.name) {
        Err(err) => Prepare::Missing(DispatchOutcome {
            id: req.call.id.clone(),
            name: req.call.name.clone(),
            result: Err(err),
        }),
        Ok(tool) if !registry.allows(tool.as_ref(), mode) => Prepare::Deny(DispatchOutcome {
            id: req.call.id.clone(),
            name: req.call.name.clone(),
            result: Err(codes::denied(format!(
                "tool '{}' denied by capability mode {mode:?}",
                req.call.name
            ))),
        }),
        Ok(tool) => Prepare::Ready(tool),
    }
}

fn collect_concurrent_window(
    registry: &ToolRegistry,
    mode: CapabilityMode,
    requests: &[DispatchRequest],
    start: usize,
    max: usize,
) -> Vec<usize> {
    let mut window = vec![start];
    let mut j = start.saturating_add(1);
    while j < requests.len() && window.len() < max {
        let Some(req) = requests.get(j) else {
            break;
        };
        let Ok(tool) = registry.require(&req.call.name) else {
            break;
        };
        if !registry.allows(tool.as_ref(), mode) {
            break;
        }
        if tool.metadata().concurrency == ConcurrencyMode::Exclusive {
            break;
        }
        window.push(j);
        j = j.saturating_add(1);
    }
    window
}

fn fill_cancelled(
    requests: &[DispatchRequest],
    outcomes: &mut [Option<DispatchOutcome>],
    from: usize,
) {
    for (i, req) in requests.iter().enumerate().skip(from) {
        if let Some(slot) = outcomes.get_mut(i)
            && slot.is_none()
        {
            *slot = Some(DispatchOutcome {
                id: req.call.id.clone(),
                name: req.call.name.clone(),
                result: Err(codes::cancelled()),
            });
        }
    }
}

fn finalize_outcomes(
    requests: &[DispatchRequest],
    outcomes: Vec<Option<DispatchOutcome>>,
) -> Vec<DispatchOutcome> {
    outcomes
        .into_iter()
        .enumerate()
        .map(|(i, o)| {
            o.unwrap_or_else(|| {
                let req = requests.get(i);
                DispatchOutcome {
                    id: req.map_or_else(ToolCallId::generate, |r| r.call.id.clone()),
                    name: req.map_or_else(|| "unknown".into(), |r| r.call.name.clone()),
                    result: Err(codes::execution("dispatch internal gap")),
                }
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use async_trait::async_trait;
    use machi_types::ErrorCode;
    use serde_json::json;
    use tokio::sync::Barrier;

    use super::*;
    use crate::metadata::{ConcurrencyMode, ToolMetadata};
    use crate::tool::DynTool;

    struct CountingTool {
        name: String,
        meta: ToolMetadata,
        active: Arc<AtomicUsize>,
        max_active: Arc<AtomicUsize>,
        barrier: Option<Arc<Barrier>>,
    }

    #[async_trait]
    impl DynTool for CountingTool {
        fn name(&self) -> &str {
            &self.name
        }
        fn description(&self) -> &str {
            "test"
        }
        fn parameters(&self) -> serde_json::Value {
            json!({"type":"object","properties":{}})
        }
        fn metadata(&self) -> ToolMetadata {
            self.meta.clone()
        }
        async fn call(
            &self,
            _ctx: ToolCallContext,
            _arguments: serde_json::Value,
        ) -> Result<ToolResult, ToolError> {
            let n = self.active.fetch_add(1, Ordering::SeqCst) + 1;
            self.max_active.fetch_max(n, Ordering::SeqCst);
            if let Some(b) = &self.barrier {
                b.wait().await;
            }
            self.active.fetch_sub(1, Ordering::SeqCst);
            Ok(ToolResult::text("ok"))
        }
    }

    fn call(name: &str, id: &str) -> DispatchRequest {
        DispatchRequest {
            call: ToolCall {
                id: ToolCallId::new(id).expect("id"),
                name: name.into(),
                arguments: json!({}),
            },
        }
    }

    #[tokio::test]
    async fn concurrent_readonly_overlap() {
        let active = Arc::new(AtomicUsize::new(0));
        let max_active = Arc::new(AtomicUsize::new(0));
        let barrier = Arc::new(Barrier::new(2));
        let t1 = Arc::new(CountingTool {
            name: "r1".into(),
            meta: ToolMetadata {
                concurrency: ConcurrencyMode::ReadOnly,
                ..ToolMetadata::read_only()
            },
            active: Arc::clone(&active),
            max_active: Arc::clone(&max_active),
            barrier: Some(Arc::clone(&barrier)),
        });
        let t2 = Arc::new(CountingTool {
            name: "r2".into(),
            meta: ToolMetadata {
                concurrency: ConcurrencyMode::ReadOnly,
                ..ToolMetadata::read_only()
            },
            active: Arc::clone(&active),
            max_active: Arc::clone(&max_active),
            barrier: Some(barrier),
        });
        let reg = ToolRegistry::from_tools(vec![t1, t2]);
        let outs = ToolDispatch::default()
            .execute_batch(
                &reg,
                ToolCallContext::default(),
                vec![call("r1", "c1"), call("r2", "c2")],
            )
            .await;
        assert_eq!(outs.len(), 2);
        assert!(outs.iter().all(|o| o.result.is_ok()));
        assert!(
            max_active.load(Ordering::SeqCst) >= 2,
            "expected overlap, max={}",
            max_active.load(Ordering::SeqCst)
        );
    }

    #[tokio::test]
    async fn exclusive_serial() {
        let active = Arc::new(AtomicUsize::new(0));
        let max_active = Arc::new(AtomicUsize::new(0));
        let t1 = Arc::new(CountingTool {
            name: "e1".into(),
            meta: ToolMetadata::exclusive_write(),
            active: Arc::clone(&active),
            max_active: Arc::clone(&max_active),
            barrier: None,
        });
        let t2 = Arc::new(CountingTool {
            name: "e2".into(),
            meta: ToolMetadata::exclusive_write(),
            active,
            max_active: Arc::clone(&max_active),
            barrier: None,
        });
        let reg = ToolRegistry::from_tools(vec![t1, t2]);
        let outs = ToolDispatch::default()
            .execute_batch(
                &reg,
                ToolCallContext::default(),
                vec![call("e1", "c1"), call("e2", "c2")],
            )
            .await;
        assert!(outs.iter().all(|o| o.result.is_ok()));
        assert_eq!(max_active.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn readonly_mode_denies_write() {
        let tool = Arc::new(CountingTool {
            name: "w".into(),
            meta: ToolMetadata::exclusive_write(),
            active: Arc::new(AtomicUsize::new(0)),
            max_active: Arc::new(AtomicUsize::new(0)),
            barrier: None,
        });
        let reg = ToolRegistry::from_tools(vec![tool]);
        let dispatch = ToolDispatch {
            capability_mode: CapabilityMode::ReadOnly,
            ..ToolDispatch::default()
        };
        let outs = dispatch
            .execute_batch(&reg, ToolCallContext::default(), vec![call("w", "c1")])
            .await;
        let err = outs
            .first()
            .expect("one outcome")
            .result
            .as_ref()
            .expect_err("denied");
        assert_eq!(err.code(), ErrorCode::ToolDenied);
    }
}
