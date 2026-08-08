//! Agent builder.

use std::sync::Arc;

use machi_tools::{SharedTool, ToolRegistry};
use machi_types::{ErrorCode, MachiError};

use crate::definition::{AgentDefinition, CompletionRequirement, Instructions, ToolPolicy};
use crate::instance::Agent;

/// Builds a validated [`Agent`].
#[derive(Default)]
pub struct AgentBuilder {
    definition: Option<AgentDefinition>,
    tools: Vec<SharedTool>,
}

impl std::fmt::Debug for AgentBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentBuilder")
            .field("definition", &self.definition)
            .field("tools", &self.tools.len())
            .finish()
    }
}

impl AgentBuilder {
    /// Empty builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Start from a definition.
    #[must_use]
    pub fn from_definition(definition: AgentDefinition) -> Self {
        Self {
            definition: Some(definition),
            tools: Vec::new(),
        }
    }

    /// Programmatic minimal definition.
    #[must_use]
    pub fn named(name: impl Into<String>) -> Self {
        Self {
            definition: Some(AgentDefinition {
                name: name.into(),
                description: String::new(),
                instructions: Instructions::Static(String::new()),
                model: "default".into(),
                tools: ToolPolicy::InheritAll,
                output_schema: None,
                completion: None,
                max_steps: 32,
            }),
            tools: Vec::new(),
        }
    }

    /// Set instructions.
    #[must_use]
    pub fn instructions(mut self, instructions: impl Into<Instructions>) -> Self {
        if let Some(def) = &mut self.definition {
            def.instructions = instructions.into();
        }
        self
    }

    /// Set model.
    #[must_use]
    pub fn model(mut self, model: impl Into<String>) -> Self {
        if let Some(def) = &mut self.definition {
            def.model = model.into();
        }
        self
    }

    /// Set description.
    #[must_use]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        if let Some(def) = &mut self.definition {
            def.description = description.into();
        }
        self
    }

    /// Set max steps.
    #[must_use]
    pub fn max_steps(mut self, max_steps: usize) -> Self {
        if let Some(def) = &mut self.definition {
            def.max_steps = max_steps;
        }
        self
    }

    /// Attach tools available to the agent (filtered by definition policy).
    #[must_use]
    pub fn tools(mut self, tools: Vec<SharedTool>) -> Self {
        self.tools = tools;
        self
    }

    /// Require a named tool call before the turn may complete.
    #[must_use]
    pub fn completion(mut self, requirement: CompletionRequirement) -> Self {
        if let Some(def) = &mut self.definition {
            def.completion = Some(requirement);
        }
        self
    }

    /// Require structured JSON output matching a JSON Schema object.
    #[must_use]
    pub fn output_schema(mut self, schema: serde_json::Value) -> Self {
        if let Some(def) = &mut self.definition {
            def.output_schema = Some(schema);
        }
        self
    }

    /// Build the agent instance.
    ///
    /// # Errors
    ///
    /// Returns validation or build errors.
    pub fn build(self) -> Result<Agent, MachiError> {
        let definition = self.definition.ok_or_else(|| {
            MachiError::new(ErrorCode::AgentBuild, "agent definition is required")
        })?;
        definition.validate()?;

        let filtered: Vec<SharedTool> = match &definition.tools {
            ToolPolicy::InheritAll => self.tools,
            ToolPolicy::Allowlist(allow) => self
                .tools
                .into_iter()
                .filter(|t| allow.iter().any(|n| n == t.name()))
                .collect(),
            ToolPolicy::Denylist(deny) => self
                .tools
                .into_iter()
                .filter(|t| !deny.iter().any(|n| n == t.name()))
                .collect(),
        };

        let system_prompt = definition.instructions.resolve();
        let tools = Arc::new(ToolRegistry::from_tools(filtered));
        Ok(Agent::new(definition, system_prompt, tools))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_name() {
        let err = AgentBuilder::named("  ").build().expect_err("empty");
        assert_eq!(err.code(), ErrorCode::AgentInvalidDefinition);
    }

    #[test]
    fn builds_minimal() {
        let agent = AgentBuilder::named("assistant")
            .instructions("You are helpful.")
            .model("mock")
            .build()
            .expect("build");
        assert_eq!(agent.name(), "assistant");
        assert_eq!(agent.system_prompt(), "You are helpful.");
    }
}
