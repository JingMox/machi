//! Tool contracts and concurrent dispatch for the Machi kernel.

#![forbid(unsafe_code)]

pub mod calc;
pub mod context;
pub mod dispatch;
pub mod error;
pub mod metadata;
pub mod registry;
pub mod tool;

pub use calc::CalcTool;
pub use context::ToolCallContext;
pub use dispatch::{DispatchRequest, ToolDispatch};
pub use error::ToolError;
pub use metadata::{
    CapabilityFlag, ConcurrencyMode, Destructiveness, InterruptBehavior, ToolMetadata,
};
pub use registry::{CapabilityMode, ToolRegistry};
pub use tool::{DynTool, SharedTool, ToolDefinition, ToolResult};
