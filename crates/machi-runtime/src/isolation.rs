//! Isolation backend for nested agent runs (default: in-process, no sandbox).
//!
//! Worktree / OS sandbox backends are optional future adapters implementing
//! the same trait — the kernel does not ship a worktree pool by default.

use std::path::PathBuf;

use async_trait::async_trait;
use machi_types::{ErrorCode, MachiError};

use crate::host::SpawnOpts;

/// Environment prepared for a single nested spawn.
#[derive(Debug, Clone, Default)]
pub struct IsolationEnv {
    /// Working directory for tools (`TurnOptions::cwd`).
    pub cwd: Option<PathBuf>,
    /// Optional isolation label for logs / metrics.
    pub label: Option<String>,
}

/// Prepares (and later tears down) an execution environment for a child agent.
///
/// Maturity: **core** (port). Default adapter is [`InProcessIsolation`];
/// worktree / OS sandbox are product adapters, not kernel defaults.
#[async_trait]
pub trait IsolationBackend: Send + Sync {
    /// Stable backend id (`in_process`, `worktree`, …).
    fn name(&self) -> &'static str;

    /// Allocate environment for this spawn.
    ///
    /// # Errors
    ///
    /// Backend-specific failures map to [`ErrorCode::HostIsolation`].
    async fn prepare(&self, opts: &SpawnOpts) -> Result<IsolationEnv, MachiError>;

    /// Release resources after the child turn finishes.
    ///
    /// # Errors
    ///
    /// Backend-specific failures.
    async fn cleanup(&self, env: &IsolationEnv) -> Result<(), MachiError>;
}

/// Default isolation: same process and filesystem as the parent host.
///
/// Does not create worktrees or OS sandboxes. Requests for product-level
/// isolation should inject a different backend (or fail at the host boundary).
#[derive(Debug, Default, Clone, Copy)]
pub struct InProcessIsolation;

#[async_trait]
impl IsolationBackend for InProcessIsolation {
    fn name(&self) -> &'static str {
        "in_process"
    }

    async fn prepare(&self, opts: &SpawnOpts) -> Result<IsolationEnv, MachiError> {
        // Nested agents share the process; cwd remains host-controlled via TurnOptions.
        Ok(IsolationEnv {
            cwd: None,
            label: opts.label.clone(),
        })
    }

    async fn cleanup(&self, _env: &IsolationEnv) -> Result<(), MachiError> {
        Ok(())
    }
}

/// Map isolation failures to a typed host error.
#[must_use]
pub fn isolation_error(backend: &str, message: impl Into<String>) -> MachiError {
    MachiError::new(
        ErrorCode::HostIsolation,
        format!("isolation backend '{backend}': {}", message.into()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::SpawnOpts;

    #[tokio::test]
    async fn in_process_prepare_cleanup() {
        let backend = InProcessIsolation;
        assert_eq!(backend.name(), "in_process");
        let env = backend
            .prepare(&SpawnOpts::new("hi").with_label("child"))
            .await
            .expect("prepare");
        assert_eq!(env.label.as_deref(), Some("child"));
        assert!(env.cwd.is_none());
        backend.cleanup(&env).await.expect("cleanup");
    }
}
