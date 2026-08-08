//! Agent definitions and built instances.
//!
//! An [`Agent`] does not run a `ReAct` loop. [`machi_runtime::TurnRuntime`]
//! (or a product host) executes turns against an agent.

#![forbid(unsafe_code)]

pub mod builder;
pub mod definition;
pub mod discovery;
pub mod instance;

pub use builder::AgentBuilder;
pub use definition::{AgentDefinition, CompletionRequirement, Instructions, ToolPolicy};
pub use discovery::{
    PROJECT_AGENTS_DIR, by_name, by_name_in_dir, discover_in_dir, discover_project, load_file,
    parse_definition_markdown,
};
pub use instance::Agent;
