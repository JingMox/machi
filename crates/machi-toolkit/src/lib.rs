//! Reference toolkit tools (cwd-jailed filesystem and shell).
//!
//! Hosts register the tools they want; the kernel does not install them by default.

#![forbid(unsafe_code)]

pub mod glob_files;
pub mod grep;
pub mod jail;
pub mod path_util;
pub mod read_file;
pub mod shell;
pub mod write_file;

pub use glob_files::{GlobTool, glob_match};
pub use grep::GrepTool;
pub use path_util::{PathJailError, resolve_jailed};
pub use read_file::ReadFileTool;
pub use shell::ShellTool;
pub use write_file::WriteFileTool;

use std::path::PathBuf;
use std::sync::Arc;

use machi_tools::SharedTool;

/// Convenience bundle: read / write / grep / glob / shell with a shared jail root.
#[must_use]
pub fn default_toolkit(jail: impl Into<PathBuf>) -> Vec<SharedTool> {
    let root = jail.into();
    vec![
        Arc::new(ReadFileTool::with_jail(root.clone())),
        Arc::new(WriteFileTool::with_jail(root.clone())),
        Arc::new(GrepTool::with_jail(root.clone())),
        Arc::new(GlobTool::with_jail(root.clone())),
        Arc::new(ShellTool::with_jail(root)),
    ]
}
