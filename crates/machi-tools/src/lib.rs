//! Tool contracts, streaming protocol, and concurrent dispatch for Machi.

#![forbid(unsafe_code)]

pub mod approval;
pub mod calc;
pub mod context;
pub mod dispatch;
pub mod error;
pub mod metadata;
pub mod registry;
pub mod stream;
pub mod tool;

pub use approval::{AlwaysDeny, ApprovalDecision, ApprovalGate, AutoApprove, denied_error};
pub use calc::CalcTool;
pub use context::{EXTRA_SPAWN_DEPTH, ToolCallContext};
pub use dispatch::{ApprovalPolicy, DispatchOutcome, DispatchRequest, ToolDispatch};
pub use error::ToolError;
pub use metadata::{
    CapabilityFlag, ConcurrencyMode, Destructiveness, InterruptBehavior, ToolMetadata,
};
pub use registry::{CapabilityMode, ToolRegistry};
pub use stream::{
    ToolProgress, ToolStream, ToolStreamItem, drain_terminal, drain_with_progress, terminal_only,
    with_progress,
};
pub use tool::{DynTool, SharedTool, ToolDefinition, ToolResult};
