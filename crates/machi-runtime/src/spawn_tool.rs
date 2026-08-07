//! Model-facing `spawn_agent` tool for dynamic multi-agent delegation.

use std::sync::Arc;

use async_trait::async_trait;
use machi_tools::registry::CapabilityMode;
use machi_tools::{DynTool, ToolCallContext, ToolMetadata, ToolResult};
use machi_types::{ErrorCode, MachiError};
use serde_json::{Value, json};
use tokio_util::sync::CancellationToken;

use crate::host::{SessionHost, SpawnOpts};

/// Tool that spawns a nested agent through a shared [`SessionHost`].
///
/// This is the **dynamic delegation** entry point for a parent agent’s `ReAct`
/// loop: the model calls `spawn_agent`, the tool blocks until the child finishes,
/// and the child’s output is returned as the tool result.
pub struct SpawnAgentTool {
    host: Arc<dyn SessionHost>,
    default_capability: CapabilityMode,
}

impl std::fmt::Debug for SpawnAgentTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpawnAgentTool")
            .field("default_capability", &self.default_capability)
            .finish_non_exhaustive()
    }
}

impl SpawnAgentTool {
    /// Bind to a host.
    #[must_use]
    pub fn new(host: Arc<dyn SessionHost>) -> Self {
        Self {
            host,
            default_capability: CapabilityMode::Full,
        }
    }

    /// Default capability mode for children when the model omits it.
    #[must_use]
    pub const fn with_default_capability(mut self, mode: CapabilityMode) -> Self {
        self.default_capability = mode;
        self
    }
}

#[async_trait]
impl DynTool for SpawnAgentTool {
    fn name(&self) -> &'static str {
        "spawn_agent"
    }

    fn description(&self) -> &'static str {
        "Spawn a nested agent to handle a subtask with its own context. \
         Provide a clear prompt; optionally set label and capability_mode \
         (full | read_only | plan)."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "prompt": {
                    "type": "string",
                    "description": "Task instructions for the nested agent"
                },
                "label": {
                    "type": "string",
                    "description": "Optional short label for logs and aggregation"
                },
                "capability_mode": {
                    "type": "string",
                    "enum": ["full", "read_only", "plan"],
                    "description": "Tool capability filter for the child agent"
                },
                "max_steps": {
                    "type": "integer",
                    "minimum": 1,
                    "description": "Optional max ReAct steps for the child turn"
                }
            },
            "required": ["prompt"],
            "additionalProperties": false
        })
    }

    fn metadata(&self) -> ToolMetadata {
        ToolMetadata::spawn()
    }

    async fn call(&self, ctx: ToolCallContext, arguments: Value) -> Result<ToolResult, MachiError> {
        if ctx.is_cancelled() {
            return Err(MachiError::new(
                ErrorCode::ToolCancelled,
                "spawn_agent cancelled",
            ));
        }

        let prompt = arguments
            .get("prompt")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                MachiError::new(ErrorCode::ToolInvalidArgs, "spawn_agent requires prompt")
            })?;

        let label = arguments
            .get("label")
            .and_then(Value::as_str)
            .map(str::to_owned);
        let capability_mode = arguments
            .get("capability_mode")
            .and_then(Value::as_str)
            .map_or(self.default_capability, parse_capability);
        let max_steps = arguments
            .get("max_steps")
            .and_then(Value::as_u64)
            .and_then(|n| usize::try_from(n).ok());

        // Child cancel is linked to the parent turn token when present.
        let child_cancel = if ctx.cancel.is_cancelled() {
            CancellationToken::new()
        } else {
            ctx.cancel.child_token()
        };

        let mut opts = SpawnOpts::new(prompt)
            .with_capability(capability_mode)
            .with_cancel(child_cancel);
        if let Some(label) = label {
            opts = opts.with_label(label);
        }
        if let Some(max_steps) = max_steps {
            opts = opts.with_max_steps(max_steps);
        }

        let run = self.host.spawn_agent(opts).await.map_err(|e| {
            MachiError::new(ErrorCode::ToolExecution, e.message().to_owned()).with_source(e)
        })?;

        let content = serde_json::to_string_pretty(&json!({
            "agent_id": run.agent_id.to_string(),
            "label": run.label,
            "success": run.success,
            "cancelled": run.cancelled,
            "output": run.output,
            "steps": run.steps,
            "duration_ms": run.duration_ms,
        }))
        .unwrap_or_else(|_| run.output.to_string());

        Ok(ToolResult {
            content,
            structured: Some(json!({
                "agent_id": run.agent_id.to_string(),
                "label": run.label,
                "success": run.success,
                "output": run.output,
            })),
            is_error: !run.success || run.cancelled,
        })
    }
}

fn parse_capability(mode: &str) -> CapabilityMode {
    match mode {
        "read_only" | "read-only" | "readonly" => CapabilityMode::ReadOnly,
        "plan" => CapabilityMode::Plan,
        _ => CapabilityMode::Full,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use machi_llm::MockSampler;
    use serde_json::json;

    use super::*;
    use crate::host::InProcessHost;

    #[tokio::test]
    async fn spawn_tool_runs_child() {
        let sampler = Arc::new(MockSampler::new());
        sampler.map_user_text("child task", "child-done");
        let host: Arc<dyn SessionHost> = Arc::new(InProcessHost::new(sampler, Vec::new()));
        let tool = SpawnAgentTool::new(host);
        let result = tool
            .call(
                ToolCallContext::default(),
                json!({"prompt": "child task", "label": "w1"}),
            )
            .await
            .expect("call");
        assert!(!result.is_error);
        let structured = result.structured.expect("structured");
        assert_eq!(
            structured.get("output").and_then(|v| v.as_str()),
            Some("child-done")
        );
        assert_eq!(structured.get("label").and_then(|v| v.as_str()), Some("w1"));
    }
}
