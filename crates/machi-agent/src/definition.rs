//! Portable agent configuration.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Static or deferred instructions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum Instructions {
    /// Fixed system prompt body.
    Static(String),
}

impl Instructions {
    /// Resolve to a string.
    #[must_use]
    pub fn resolve(&self) -> String {
        match self {
            Self::Static(s) => s.clone(),
        }
    }
}

impl From<String> for Instructions {
    fn from(value: String) -> Self {
        Self::Static(value)
    }
}

impl From<&str> for Instructions {
    fn from(value: &str) -> Self {
        Self::Static(value.to_owned())
    }
}

/// Tool allow/deny policy on a definition.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ToolPolicy {
    /// Inherit all tools supplied at build time.
    #[default]
    InheritAll,
    /// Only these tool names.
    Allowlist(Vec<String>),
    /// All except these names.
    Denylist(Vec<String>),
}

/// Require a tool call before the turn may complete.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequirement {
    /// Canonical tool name that must be called.
    pub tool: String,
    /// Reminder injected when the model stops without calling it.
    pub reminder: String,
    /// Max forced re-samples.
    pub max_retries: u32,
}

/// Versionable agent definition (data only).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDefinition {
    /// Unique name (slug).
    pub name: String,
    /// Human description.
    pub description: String,
    /// Instructions / system prompt body.
    pub instructions: Instructions,
    /// Default model id.
    pub model: String,
    /// Tool policy.
    #[serde(default)]
    pub tools: ToolPolicy,
    /// Optional structured output schema (JSON Schema object).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<Value>,
    /// Optional completion gate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completion: Option<CompletionRequirement>,
    /// Default max steps for turns using this agent.
    #[serde(default = "default_max_steps")]
    pub max_steps: usize,
}

fn default_max_steps() -> usize {
    32
}

impl AgentDefinition {
    /// Validate required fields.
    ///
    /// # Errors
    ///
    /// Returns [`machi_types::MachiError`] when name/model empty or `max_steps` is zero.
    pub fn validate(&self) -> Result<(), machi_types::MachiError> {
        use machi_types::{ErrorCode, MachiError};
        if self.name.trim().is_empty() {
            return Err(MachiError::new(
                ErrorCode::AgentInvalidDefinition,
                "agent name must be non-empty",
            ));
        }
        if self.model.trim().is_empty() {
            return Err(MachiError::new(
                ErrorCode::AgentInvalidDefinition,
                "agent model must be non-empty",
            ));
        }
        if self.max_steps == 0 {
            return Err(MachiError::new(
                ErrorCode::AgentInvalidDefinition,
                "max_steps must be >= 1",
            ));
        }
        Ok(())
    }
}
