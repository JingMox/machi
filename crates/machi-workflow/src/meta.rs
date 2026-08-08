//! Workflow metadata extraction from scripts.

use serde::{Deserialize, Serialize};

/// Phase descriptor for UIs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhaseMeta {
    /// Title.
    pub title: String,
    /// Optional detail.
    #[serde(default)]
    pub detail: Option<String>,
}

/// Workflow catalog metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowMeta {
    /// Slug name.
    pub name: String,
    /// Description.
    pub description: String,
    /// Optional phases.
    #[serde(default)]
    pub phases: Vec<PhaseMeta>,
}

/// Meta extraction errors.
#[derive(Debug, thiserror::Error)]
pub enum MetaError {
    /// Script failed to parse/run for meta probe.
    #[error("meta extract failed: {0}")]
    Failed(String),
    /// Missing required field.
    #[error("meta missing field: {0}")]
    Missing(&'static str),
}

/// Extract `meta` map by evaluating the script with dummy host functions noop.
///
/// Scripts should start with `let meta = #{ name: "...", description: "..." };`.
///
/// # Errors
///
/// Returns [`MetaError`] on parse/eval/missing fields.
pub fn extract_meta(script: &str) -> Result<WorkflowMeta, MetaError> {
    let mut engine = rhai::Engine::new();
    engine.set_max_operations(100_000);
    engine.disable_symbol("eval");
    // Stub host fns so meta-only scripts that reference them later still compile.
    engine.register_fn("agent", |_p: &str| rhai::Map::new());
    engine.register_fn("phase", |_t: &str| {});
    engine.register_fn(
        "complete",
        |_v: rhai::Dynamic| -> Result<(), Box<rhai::EvalAltResult>> { Err("complete".into()) },
    );
    engine.register_fn(
        "pause",
        |_k: &str, _m: &str| -> Result<(), Box<rhai::EvalAltResult>> { Err("pause".into()) },
    );
    engine.register_fn("log", |_m: &str| {});
    engine.register_fn("telemetry", |_n: &str, _f: rhai::Dynamic| {});
    engine.register_fn("write_scratch", |_n: &str, _c: &str| String::new());
    engine.register_fn("read_scratch", |_n: &str| String::new());
    engine.register_fn("render_template", |_n: &str, _v: rhai::Dynamic| String::new());
    engine.register_fn("git_diff_since", |_c: &str| String::new());
    engine.register_fn("parallel", |_a: rhai::Array| rhai::Array::new());
    engine.register_fn("budget", rhai::Map::new);

    let mut scope = rhai::Scope::new();
    scope.push_dynamic("args", rhai::Dynamic::UNIT);
    // Probe evaluation may fail after `meta` is bound (e.g. complete()); ignore.
    drop(engine.eval_with_scope::<rhai::Dynamic>(&mut scope, script));

    let meta_map = scope
        .get_value::<rhai::Map>("meta")
        .ok_or(MetaError::Missing("meta"))?;
    let value = rhai::serde::from_dynamic::<serde_json::Value>(&meta_map.into())
        .map_err(|e| MetaError::Failed(e.to_string()))?;
    let meta: WorkflowMeta =
        serde_json::from_value(value).map_err(|e| MetaError::Failed(e.to_string()))?;
    if meta.name.trim().is_empty() {
        return Err(MetaError::Missing("meta.name"));
    }
    if meta.description.trim().is_empty() {
        return Err(MetaError::Missing("meta.description"));
    }
    Ok(meta)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_meta() {
        let script = r#"
            let meta = #{ name: "fanout", description: "test workflow", phases: [] };
            complete(#{});
        "#;
        let meta = extract_meta(script).expect("meta");
        assert_eq!(meta.name, "fanout");
    }
}
