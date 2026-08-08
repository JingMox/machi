//! In-memory registry of [`AgentDefinition`] for host / workflow resolution.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

use machi_types::{ErrorCode, MachiError};

use crate::definition::AgentDefinition;
use crate::discovery::{discover_in_dir, discover_project};

/// Shared agent definition catalogue for `agent_type` resolution.
///
/// Clone is cheap (`Arc` interior). Lookups are O(log n) on a `BTreeMap`.
#[derive(Debug, Clone, Default)]
pub struct AgentRegistry {
    inner: Arc<BTreeMap<String, AgentDefinition>>,
}

impl AgentRegistry {
    /// Empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Build from an iterator of definitions (later names overwrite earlier).
    #[must_use]
    pub fn from_definitions(defs: impl IntoIterator<Item = AgentDefinition>) -> Self {
        let mut map = BTreeMap::new();
        for def in defs {
            map.insert(def.name.clone(), def);
        }
        Self {
            inner: Arc::new(map),
        }
    }

    /// Insert or replace a definition; returns a new registry (copy-on-write).
    #[must_use]
    pub fn insert(&self, def: AgentDefinition) -> Self {
        let mut map = (*self.inner).clone();
        map.insert(def.name.clone(), def);
        Self {
            inner: Arc::new(map),
        }
    }

    /// Merge another registry (other wins on key conflict).
    #[must_use]
    pub fn merge(&self, other: &Self) -> Self {
        let mut map = (*self.inner).clone();
        for (k, v) in other.inner.iter() {
            map.insert(k.clone(), v.clone());
        }
        Self {
            inner: Arc::new(map),
        }
    }

    /// Number of registered agents.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Whether empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Lookup by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&AgentDefinition> {
        self.inner.get(name)
    }

    /// Lookup by name or error.
    ///
    /// # Errors
    ///
    /// [`ErrorCode::AgentNotFound`] when missing.
    pub fn require(&self, name: &str) -> Result<&AgentDefinition, MachiError> {
        self.get(name).ok_or_else(|| {
            MachiError::new(
                ErrorCode::AgentNotFound,
                format!("agent_type '{name}' not registered"),
            )
        })
    }

    /// Sorted names.
    #[must_use]
    pub fn names(&self) -> Vec<String> {
        self.inner.keys().cloned().collect()
    }

    /// Discover definitions under a directory and merge (discovered wins on conflict).
    ///
    /// # Errors
    ///
    /// Directory read / parse failures when `strict`.
    pub fn discover_dir(self, root: impl AsRef<Path>, strict: bool) -> Result<Self, MachiError> {
        let found = discover_in_dir(root, strict)?;
        Ok(self.merge(&Self::from_definitions(found)))
    }

    /// Discover `{cwd}/.machi/agents` when present.
    ///
    /// # Errors
    ///
    /// Propagates discovery failures.
    pub fn discover_project(self, cwd: impl AsRef<Path>) -> Result<Self, MachiError> {
        let found = discover_project(cwd)?;
        Ok(self.merge(&Self::from_definitions(found)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definition::{Instructions, ToolPolicy};
    use std::fs;
    use std::io::Write;
    use tempfile::tempdir;

    fn def(name: &str) -> AgentDefinition {
        AgentDefinition {
            name: name.into(),
            description: String::new(),
            instructions: Instructions::Static("hi".into()),
            model: "mock".into(),
            tools: ToolPolicy::InheritAll,
            output_schema: None,
            completion: None,
            max_steps: 4,
        }
    }

    #[test]
    fn insert_and_require() {
        let reg = AgentRegistry::new().insert(def("worker"));
        assert_eq!(reg.require("worker").expect("get").name, "worker");
        assert_eq!(
            reg.require("missing").expect_err("nf").code(),
            ErrorCode::AgentNotFound
        );
    }

    #[test]
    fn discover_merges() {
        let dir = tempdir().expect("tmp");
        let path = dir.path().join("helper.md");
        let mut f = fs::File::create(&path).expect("create");
        write!(f, "---\nname: helper\nmodel: m\n---\n\nHelp.\n").expect("write");
        let reg = AgentRegistry::new()
            .insert(def("base"))
            .discover_dir(dir.path(), true)
            .expect("disc");
        assert!(reg.get("base").is_some());
        assert!(reg.get("helper").is_some());
        assert_eq!(reg.len(), 2);
    }
}
