//! Agent definitions and built instances.
//!
//! An [`Agent`] does not run a `ReAct` loop. [`machi_runtime::TurnRuntime`]
//! (or a product host) executes turns against an agent.

#![forbid(unsafe_code)]

pub mod builder;
pub mod definition;
pub mod instance;

pub use builder::AgentBuilder;
pub use definition::{AgentDefinition, CompletionRequirement, Instructions, ToolPolicy};
pub use instance::Agent;
