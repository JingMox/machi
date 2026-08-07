//! Tool registry with capability filtering.

use std::collections::HashMap;
use std::sync::Arc;

use crate::error::{ToolError, codes};
use crate::metadata::CapabilityFlag;
use crate::tool::{DynTool, SharedTool, ToolDefinition};

/// How nested/session capability mode filters tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum CapabilityMode {
    /// All registered tools.
    #[default]
    Full,
    /// Only tools admissible under read-only metadata rules.
    ReadOnly,
    /// Only tools that do not include execute/spawn (plan-friendly).
    Plan,
}

/// Thread-safe tool registry.
#[derive(Clone, Default)]
pub struct ToolRegistry {
    tools: Arc<HashMap<String, SharedTool>>,
}

impl std::fmt::Debug for ToolRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut names: Vec<_> = self.tools.keys().map(String::as_str).collect();
        names.sort_unstable();
        f.debug_struct("ToolRegistry")
            .field("tools", &names)
            .finish()
    }
}

impl ToolRegistry {
    /// Empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Build from a list of tools (last wins on name collision).
    #[must_use]
    pub fn from_tools(tools: Vec<SharedTool>) -> Self {
        let mut map = HashMap::new();
        for tool in tools {
            map.insert(tool.name().to_owned(), tool);
        }
        Self {
            tools: Arc::new(map),
        }
    }

    /// Lookup by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<SharedTool> {
        self.tools.get(name).cloned()
    }

    /// Require tool or error.
    ///
    /// # Errors
    ///
    /// Returns [`ToolError`] when the tool is missing.
    pub fn require(&self, name: &str) -> Result<SharedTool, ToolError> {
        self.get(name).ok_or_else(|| codes::not_found(name))
    }

    /// Definitions visible under a capability mode.
    #[must_use]
    pub fn definitions(&self, mode: CapabilityMode) -> Vec<ToolDefinition> {
        let mut defs: Vec<_> = self
            .tools
            .values()
            .filter(|t| self.allows(t.as_ref(), mode))
            .map(|t| t.definition())
            .collect();
        defs.sort_by(|a, b| a.name.cmp(&b.name));
        defs
    }

    /// Whether a tool is allowed under mode.
    #[must_use]
    pub fn allows(&self, tool: &dyn DynTool, mode: CapabilityMode) -> bool {
        let _ = self;
        let meta = tool.metadata();
        match mode {
            CapabilityMode::Full => true,
            CapabilityMode::ReadOnly => meta.allowed_in_read_only(),
            CapabilityMode::Plan => !meta
                .capabilities
                .iter()
                .any(|c| matches!(c, CapabilityFlag::Execute | CapabilityFlag::Spawn)),
        }
    }

    /// Number of tools.
    #[must_use]
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// True when empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }
}
