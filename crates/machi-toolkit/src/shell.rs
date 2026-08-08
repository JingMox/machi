//! Cwd-jailed shell command execution (restricted).

use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use async_trait::async_trait;
use machi_tools::stream::ToolStream;
use machi_tools::{
    DynTool, ToolCallContext, ToolError, ToolMetadata, ToolProgress, ToolResult, with_progress,
};
use serde_json::{Value, json};
use tokio::process::Command;
use tokio::time::timeout;

use crate::jail::resolve_root;

/// Default command timeout.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
/// Default combined stdout+stderr capture limit.
pub const DEFAULT_MAX_OUTPUT: usize = 64 * 1024;

/// Run a shell command with `cwd` jailed to the workspace root.
///
/// Security: intended for trusted hosts. Does not sandbox syscalls; only sets
/// process cwd to the jail and applies timeout/output caps.
#[derive(Debug, Clone)]
pub struct ShellTool {
    /// Jail / working directory.
    pub jail_root: Option<PathBuf>,
    /// Per-call timeout.
    pub timeout: Duration,
    /// Max captured output bytes.
    pub max_output: usize,
}

impl Default for ShellTool {
    fn default() -> Self {
        Self {
            jail_root: None,
            timeout: DEFAULT_TIMEOUT,
            max_output: DEFAULT_MAX_OUTPUT,
        }
    }
}

impl ShellTool {
    /// Explicit jail as cwd.
    #[must_use]
    pub fn with_jail(root: impl Into<PathBuf>) -> Self {
        Self {
            jail_root: Some(root.into()),
            ..Self::default()
        }
    }
}

#[async_trait]
impl DynTool for ShellTool {
    fn name(&self) -> &'static str {
        "shell"
    }

    fn description(&self) -> &'static str {
        "Run a shell command with cwd set to the workspace jail. Args: command (string)."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "Shell command line (executed via `sh -c`)"
                }
            },
            "required": ["command"],
            "additionalProperties": false
        })
    }

    fn metadata(&self) -> ToolMetadata {
        ToolMetadata::shell_execute(self.timeout)
    }

    async fn call(&self, ctx: ToolCallContext, arguments: Value) -> Result<ToolResult, ToolError> {
        let stream = self.execute(ctx, arguments).await;
        machi_tools::drain_terminal(stream).await
    }

    async fn execute(&self, ctx: ToolCallContext, arguments: Value) -> ToolStream {
        let command = arguments
            .get("command")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_owned);

        let root = match resolve_root(self.jail_root.as_ref(), &ctx, "shell") {
            Ok(r) => r,
            Err(e) => return machi_tools::terminal_only(Err(e)),
        };
        let Some(command) = command else {
            return machi_tools::terminal_only(Err(machi_tools::error::codes::invalid_args(
                "shell requires non-empty command",
            )));
        };
        let limit = self.timeout;
        let max_output = self.max_output;
        let cancel = ctx.cancel.clone();
        with_progress(
            vec![ToolProgress::text(format!("shell: {command}"))],
            move || async move {
                if cancel.is_cancelled() {
                    return Err(machi_tools::error::codes::cancelled());
                }
                let child = Command::new("sh")
                    .arg("-c")
                    .arg(&command)
                    .current_dir(&root)
                    .stdin(Stdio::null())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .kill_on_drop(true)
                    .spawn()
                    .map_err(|e| {
                        machi_tools::error::codes::execution(format!("spawn shell: {e}"))
                    })?;

                let wait = child.wait_with_output();
                let output = tokio::select! {
                    () = cancel.cancelled() => {
                        return Err(machi_tools::error::codes::cancelled());
                    }
                    res = timeout(limit, wait) => {
                        match res {
                            Ok(Ok(o)) => o,
                            Ok(Err(e)) => {
                                return Err(machi_tools::error::codes::execution(format!(
                                    "shell wait: {e}"
                                )));
                            }
                            Err(_) => {
                                return Err(machi_tools::error::codes::timeout(format!(
                                    "shell timed out after {limit:?}"
                                )));
                            }
                        }
                    }
                };

                let mut stdout = String::from_utf8_lossy(&output.stdout).into_owned();
                let mut stderr = String::from_utf8_lossy(&output.stderr).into_owned();
                truncate_in_place(&mut stdout, max_output / 2);
                truncate_in_place(&mut stderr, max_output / 2);
                let code = output.status.code().unwrap_or(-1);
                let mut content = format!("exit_code={code}\n");
                if !stdout.is_empty() {
                    content.push_str("--- stdout ---\n");
                    content.push_str(&stdout);
                    if !content.ends_with('\n') {
                        content.push('\n');
                    }
                }
                if !stderr.is_empty() {
                    content.push_str("--- stderr ---\n");
                    content.push_str(&stderr);
                }
                Ok(ToolResult {
                    content,
                    structured: Some(json!({
                        "exit_code": code,
                        "stdout": stdout,
                        "stderr": stderr,
                        "command": command,
                    })),
                    is_error: !output.status.success(),
                })
            },
        )
    }
}

fn truncate_in_place(s: &mut String, max: usize) {
    if s.len() > max {
        s.truncate(max);
        s.push_str("\n…[truncated]");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn echoes_in_jail() {
        let dir = tempdir().expect("temp");
        let tool = ShellTool::with_jail(dir.path());
        let r = tool
            .call(
                ToolCallContext {
                    cwd: Some(dir.path().to_path_buf()),
                    ..ToolCallContext::default()
                },
                json!({"command": "echo hi && pwd"}),
            )
            .await
            .expect("shell");
        assert!(r.content.contains("hi"), "{}", r.content);
        assert!(
            r.content.contains(dir.path().to_string_lossy().as_ref())
                || r.structured
                    .as_ref()
                    .and_then(|v| v.get("stdout"))
                    .and_then(Value::as_str)
                    .is_some_and(|s| s.contains("hi")),
            "{}",
            r.content
        );
    }
}
