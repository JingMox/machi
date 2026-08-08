//! Cwd-jail path resolution.

use std::path::{Component, Path, PathBuf};

/// Path jail failures.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PathJailError {
    /// Path escapes the configured root after normalization.
    #[error("path escapes jail root: {0}")]
    EscapesJail(String),
    /// Empty path.
    #[error("path must be non-empty")]
    Empty,
    /// Absolute path not under jail (when absolute inputs are rejected).
    #[error("absolute path not under jail: {0}")]
    AbsoluteOutside(String),
}

/// Resolve `user_path` under `jail_root`, rejecting `..` escapes.
///
/// Relative paths are joined to `jail_root`. Absolute paths are accepted only
/// when they normalize to a descendant of `jail_root`.
///
/// # Errors
///
/// Returns [`PathJailError`] when the path is empty or escapes the jail.
pub fn resolve_jailed(jail_root: &Path, user_path: &str) -> Result<PathBuf, PathJailError> {
    let user_path = user_path.trim();
    if user_path.is_empty() {
        return Err(PathJailError::Empty);
    }

    let candidate = if Path::new(user_path).is_absolute() {
        PathBuf::from(user_path)
    } else {
        jail_root.join(user_path)
    };

    let normalized = normalize_lexically(&candidate);
    let root = normalize_lexically(jail_root);

    if !normalized.starts_with(&root) {
        return Err(PathJailError::EscapesJail(user_path.to_owned()));
    }
    Ok(normalized)
}

/// Lexical normalization without filesystem access (no symlink resolution).
fn normalize_lexically(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for comp in path.components() {
        push_component(&mut out, comp);
    }
    out
}

fn push_component(out: &mut PathBuf, comp: Component<'_>) {
    match comp {
        Component::Prefix(p) => out.push(p.as_os_str()),
        Component::RootDir => out.push(comp.as_os_str()),
        Component::CurDir => {}
        Component::ParentDir => {
            if !out.pop() && out.as_os_str().is_empty() {
                out.push("..");
            }
        }
        Component::Normal(c) => out.push(c),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_ok() {
        let root = PathBuf::from("/workspace");
        let p = resolve_jailed(&root, "src/main.rs").expect("ok");
        assert_eq!(p, PathBuf::from("/workspace/src/main.rs"));
    }

    #[test]
    fn rejects_parent_escape() {
        let root = PathBuf::from("/workspace");
        let err = resolve_jailed(&root, "../etc/passwd").expect_err("escape");
        assert!(matches!(err, PathJailError::EscapesJail(_)));
    }

    #[test]
    fn rejects_nested_escape() {
        let root = PathBuf::from("/workspace");
        let err = resolve_jailed(&root, "a/../../outside").expect_err("escape");
        assert!(matches!(err, PathJailError::EscapesJail(_)));
    }
}
