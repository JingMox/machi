//! Turn runtime and session host for the Machi kernel.

#![forbid(unsafe_code)]

pub mod host;
pub mod metrics;
pub mod session;
pub mod spawn_tool;
pub mod state;
pub mod turn;
pub mod workflow_host;

pub use host::{AgentRunResult, InProcessHost, SessionHost, SpawnOpts};
pub use metrics::{MetricsSink, NoopMetrics, SharedMetrics};
pub use session::Session;
pub use spawn_tool::SpawnAgentTool;
pub use state::{ConversationState, VecConversationState};
pub use turn::{TurnInput, TurnOptions, TurnOutcome, TurnRuntime};
pub use workflow_host::run_workflow_on_host;
