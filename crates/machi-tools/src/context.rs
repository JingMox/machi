//! Per-call execution context.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use machi_types::{AgentId, Deadline, SessionId};
use tokio_util::sync::CancellationToken;

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

    /// True when cancel requested or deadline expired.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancel.is_cancelled() || self.deadline.is_some_and(|d| d.is_expired())
    }
}
