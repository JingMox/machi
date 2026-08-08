//! Discover agent definitions from Markdown files with YAML frontmatter.

use std::fs;
use std::path::{Path, PathBuf};

use machi_types::{ErrorCode, MachiError};

use crate::definition::{AgentDefinition, Instructions};

/// Default relative directory under a project root.
pub const PROJECT_AGENTS_DIR: &str = ".machi/agents";

/// Parse a definition file: YAML frontmatter between `---` fences, body = instructions.
///
/// Frontmatter keys: `name`, `description`, `model`, `max_steps` (optional).
/// Unknown keys are ignored (forward compatible).
///
/// # Errors
///
/// Returns parse / validation errors.
pub fn parse_definition_markdown(raw: &str) -> Result<AgentDefinition, MachiError> {
    let (front, body) = split_frontmatter(raw)?;
    let meta = parse_simple_yaml_map(&front)?;
    let name = meta
        .get("name")
        .cloned()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            MachiError::new(
                ErrorCode::AgentInvalidDefinition,
                "frontmatter missing name",
            )
        })?;
    let description = meta.get("description").cloned().unwrap_or_default();
    let model = meta
        .get("model")
        .cloned()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "default".into());
    let max_steps = meta
        .get("max_steps")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(32);
    let def = AgentDefinition {
        name,
        description,
        instructions: Instructions::Static(body.trim().to_owned()),
        model,
        tools: crate::definition::ToolPolicy::InheritAll,
        output_schema: None,
        completion: None,
        max_steps,
    };
    def.validate()?;
    Ok(def)
}

/// Load a single file.
///
/// # Errors
///
/// I/O or parse failures.
pub fn load_file(path: impl AsRef<Path>) -> Result<AgentDefinition, MachiError> {
    let path = path.as_ref();
    let raw = fs::read_to_string(path).map_err(|e| {
        MachiError::new(
            ErrorCode::AgentBuild,
            format!("read agent file {}: {e}", path.display()),
        )
    })?;
    parse_definition_markdown(&raw)
}

/// Discover `*.md` definitions under `root` (non-recursive).
///
/// # Errors
///
/// Directory read failures; individual bad files are skipped and collected as soft errors
/// only when `strict` is true (then first error fails).
pub fn discover_in_dir(
    root: impl AsRef<Path>,
    strict: bool,
) -> Result<Vec<AgentDefinition>, MachiError> {
    let root = root.as_ref();
    let rd = fs::read_dir(root).map_err(|e| {
        MachiError::new(
            ErrorCode::AgentBuild,
            format!("read agents dir {}: {e}", root.display()),
        )
    })?;
    let mut out = Vec::new();
    for entry in rd {
        let entry = entry
            .map_err(|e| MachiError::new(ErrorCode::AgentBuild, format!("read_dir entry: {e}")))?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        match load_file(&path) {
            Ok(def) => out.push(def),
            Err(e) if strict => return Err(e),
            Err(_) => {}
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

/// Discover under `{cwd}/.machi/agents` when the directory exists.
///
/// # Errors
///
/// Propagates discovery errors; missing directory yields empty list.
pub fn discover_project(cwd: impl AsRef<Path>) -> Result<Vec<AgentDefinition>, MachiError> {
    let dir = cwd.as_ref().join(PROJECT_AGENTS_DIR);
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    discover_in_dir(dir, false)
}

/// Find by name in a directory (file stem or frontmatter name).
///
/// # Errors
///
/// Not found or I/O.
pub fn by_name_in_dir(root: impl AsRef<Path>, name: &str) -> Result<AgentDefinition, MachiError> {
    let root = root.as_ref();
    let direct = root.join(format!("{name}.md"));
    if direct.is_file() {
        return load_file(direct);
    }
    for def in discover_in_dir(root, false)? {
        if def.name == name {
            return Ok(def);
        }
    }
    Err(MachiError::new(
        ErrorCode::AgentNotFound,
        format!("agent not found: {name}"),
    ))
}

/// Convenience: `{cwd}/.machi/agents` then optional extra roots.
///
/// # Errors
///
/// Not found after searching all roots.
pub fn by_name(name: &str, roots: &[PathBuf]) -> Result<AgentDefinition, MachiError> {
    for root in roots {
        let dir = if root.ends_with(PROJECT_AGENTS_DIR) {
            root.clone()
        } else {
            root.join(PROJECT_AGENTS_DIR)
        };
        if dir.is_dir()
            && let Ok(def) = by_name_in_dir(&dir, name)
        {
            return Ok(def);
        }
    }
    Err(MachiError::new(
        ErrorCode::AgentNotFound,
        format!("agent not found: {name}"),
    ))
}

fn split_frontmatter(raw: &str) -> Result<(String, String), MachiError> {
    let text = raw.trim_start_matches('\u{feff}');
    let Some(rest) = text.strip_prefix("---") else {
        return Err(MachiError::new(
            ErrorCode::AgentInvalidDefinition,
            "agent markdown must start with --- frontmatter",
        ));
    };
    let rest = rest.strip_prefix('\n').unwrap_or(rest);
    let Some((front, body)) = rest.split_once("\n---") else {
        return Err(MachiError::new(
            ErrorCode::AgentInvalidDefinition,
            "agent markdown missing closing --- frontmatter",
        ));
    };
    let body = body.strip_prefix('\n').unwrap_or(body);
    Ok((front.to_owned(), body.to_owned()))
}

/// Minimal `key: value` YAML map (string values only; no nested objects).
fn parse_simple_yaml_map(
    front: &str,
) -> Result<std::collections::BTreeMap<String, String>, MachiError> {
    let mut map = std::collections::BTreeMap::new();
    for line in front.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((k, v)) = line.split_once(':') else {
            continue;
        };
        let key = k.trim().to_owned();
        if key.is_empty() {
            continue;
        }
        let mut val = v.trim().to_owned();
        if (val.starts_with('"') && val.ends_with('"'))
            || (val.starts_with('\'') && val.ends_with('\''))
        {
            val = val[1..val.len().saturating_sub(1)].to_owned();
        }
        map.insert(key, val);
    }
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn parses_frontmatter() {
        let raw = "---\n\
name: reviewer\n\
description: Reviews code\n\
model: mock\n\
max_steps: 8\n\
---\n\
\n\
You review diffs carefully.\n";
        let def = parse_definition_markdown(raw).expect("parse");
        assert_eq!(def.name, "reviewer");
        assert_eq!(def.model, "mock");
        assert_eq!(def.max_steps, 8);
        assert!(def.instructions.resolve().contains("review diffs"));
    }

    #[test]
    fn discover_dir() {
        let dir = tempdir().expect("tmp");
        let path = dir.path().join("helper.md");
        let mut f = fs::File::create(&path).expect("create");
        write!(f, "---\nname: helper\nmodel: m\n---\n\nHelp.\n").expect("write");
        let defs = discover_in_dir(dir.path(), true).expect("discover");
        assert_eq!(defs.len(), 1);
        assert_eq!(defs.first().map(|d| d.name.as_str()), Some("helper"));
    }
}
