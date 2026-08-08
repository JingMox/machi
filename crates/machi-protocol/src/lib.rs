//! Wire-agnostic protocol types shared across tools, LLM adapters, and hosts.
//!
//! This crate sits above [`machi_types`] in the DAG and must not depend on
//! HTTP clients, provider SDKs, or runtime I/O.

#![forbid(unsafe_code)]

pub mod content;
pub mod observability;
pub mod tool_id;

pub use content::{ContentBlock, ImageBlock};
pub use observability::{
    SPAN_COMPACT, SPAN_SAMPLE, SPAN_SESSION, SPAN_SPAWN, SPAN_TOOL, SPAN_TOOL_BATCH, SPAN_TURN,
    SPAN_WORKFLOW, SPAN_WORKFLOW_HOST, field,
};
pub use tool_id::ToolId;
