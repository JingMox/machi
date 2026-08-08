//! Per-call execution context.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use machi_types::{AgentId, Deadline, SessionId};
use tokio_util::sync::CancellationToken;

/// Extra key: nesting depth of the agent that owns this tool call
/// (`0` = first host-spawned level). Used by `spawn_agent` to fail-closed on depth.
pub const EXTRA_SPAWN_DEPTH: &str = "machi.spawn_depth";

/// Context passed into every tool invocation.
#[derive(Debug, Clone)]
pub struct ToolCallContext {
    /// Cancellation token for the call / turn.
    pub cancel: CancellationToken,
    /// Optional absolute deadline.
    pub deadline: Option<Deadline>,
    /// Working directory for relative paths.
    pub cwd: Option<PathBuf>,
    /// Session id when known.
    pub session_id: Option<SessionId>,
    /// Agent id when known.
    pub agent_id: Option<AgentId>,
    /// Host-defined extensions (stringly map for v1).
    pub extras: Arc<HashMap<String, String>>,
}

impl Default for ToolCallContext {
    fn default() -> Self {
        Self {
            cancel: CancellationToken::new(),
            deadline: None,
            cwd: None,
            session_id: None,
            agent_id: None,
            extras: Arc::new(HashMap::new()),
        }
    }
}

impl ToolCallContext {
    /// Builder: set cancel token.
    #[must_use]
    pub fn with_cancel(mut self, cancel: CancellationToken) -> Self {
        self.cancel = cancel;
        self
    }

    /// Builder: set deadline.
    #[must_use]
    pub fn with_deadline(mut self, deadline: Deadline) -> Self {
        self.deadline = Some(deadline);
        self
    }

    /// Builder: replace extras map.
    #[must_use]
    pub fn with_extras(mut self, extras: HashMap<String, String>) -> Self {
        self.extras = Arc::new(extras);
        self
    }

    /// Read nesting depth of the current agent (`None` = top-level session turn).
    #[must_use]
    pub fn spawn_depth(&self) -> Option<u32> {
        self.extras
            .get(EXTRA_SPAWN_DEPTH)
            .and_then(|s| s.parse().ok())
    }

    /// True when cancel requested or deadline expired.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancel.is_cancelled() || self.deadline.is_some_and(|d| d.is_expired())
    }
}
