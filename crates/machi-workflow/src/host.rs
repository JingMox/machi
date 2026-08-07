//! Host request protocol (side-effect boundary).

use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

/// Options for `agent()` / `parallel()` host spawns.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentOpts {
    /// Prompt text.
    #[serde(default)]
    pub prompt: String,
    /// Optional label.
    #[serde(default)]
    pub label: Option<String>,
    /// Optional model override.
    #[serde(default)]
    pub model: Option<String>,
    /// Capability mode string (`full`, `read_only`, `plan`).
    #[serde(default)]
    pub capability_mode: Option<String>,
    /// Optional JSON schema for structured output.
    #[serde(default)]
    pub output_schema: Option<serde_json::Value>,
    /// Optional phase tag for UI.
    #[serde(default)]
    pub phase: Option<String>,
}

/// Result returned from a host agent spawn.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResult {
    /// Host-assigned agent id.
    pub agent_id: String,
    /// Success flag.
    pub success: bool,
    /// Output payload.
    pub output: serde_json::Value,
    /// Cancelled flag.
    pub cancelled: bool,
    /// Tokens used (best effort).
    pub tokens_used: u64,
    /// Duration ms.
    pub duration_ms: u64,
}

/// Budget snapshot.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BudgetState {
    /// Total budget when capped.
    pub total: Option<u64>,
    /// Spent slots.
    pub spent: u64,
    /// Reserved but not spent.
    pub reserved: u64,
    /// Remaining when capped.
    pub remaining: Option<u64>,
}

/// Host-side failures.
#[derive(Debug, Clone, thiserror::Error)]
pub enum HostError {
    /// Agent call quota exceeded.
    #[error("workflow agent-call quota exceeded: requested {requested}, maximum {maximum}")]
    AgentCallQuotaExceeded {
        /// Requested count.
        requested: u64,
        /// Maximum allowed.
        maximum: u64,
    },
    /// Budget exhausted.
    #[error("workflow token/agent budget exceeded")]
    BudgetExceeded,
    /// Cancelled.
    #[error("workflow cancelled")]
    Cancelled,
    /// Capability not supported by this host.
    #[error("unsupported in this context: {0}")]
    Unsupported(String),
    /// Generic host failure.
    #[error("host failure: {0}")]
    Failed(String),
}

/// Requests the pure engine sends to the host.
#[derive(Debug)]
pub enum WorkflowHostRequest {
    /// Reserve agent call slots.
    ReserveAgentCalls {
        /// Count to reserve.
        count: u64,
        /// Reply channel.
        reply: oneshot::Sender<Result<(), HostError>>,
    },
    /// Release unused reservations.
    ReleaseAgentCalls {
        /// Count to release.
        count: u64,
        /// Reply channel.
        reply: oneshot::Sender<Result<(), HostError>>,
    },
    /// Spawn a nested agent and wait for completion.
    SpawnAgent {
        /// Spawn options.
        opts: AgentOpts,
        /// Reply channel.
        reply: oneshot::Sender<Result<AgentResult, HostError>>,
    },
    /// Phase notification (non-journaled UI signal).
    Phase {
        /// Phase title.
        title: String,
        /// True when replaying.
        replayed: bool,
    },
    /// Log line.
    Log {
        /// Message.
        message: String,
        /// True when replaying.
        replayed: bool,
    },
    /// Budget query.
    BudgetQuery {
        /// Reply channel.
        reply: oneshot::Sender<Result<BudgetState, HostError>>,
    },
}

impl WorkflowHostRequest {
    /// Stable kind string for journaling.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::ReserveAgentCalls { .. } => "reserve_agent_calls",
            Self::ReleaseAgentCalls { .. } => "release_agent_calls",
            Self::SpawnAgent { .. } => "spawn_agent",
            Self::Phase { .. } => "phase",
            Self::Log { .. } => "log",
            Self::BudgetQuery { .. } => "budget",
        }
    }
}
