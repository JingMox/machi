//! Structured errors and stable error codes.

use std::fmt;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

/// Machine-stable error code for control-plane handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ErrorCode {
    /// Invalid or empty identifier.
    TypesInvalidId,
    /// Message or payload failed validation.
    TypesValidation,
    /// Serialization failure.
    TypesSerde,
    /// Tool not found in registry.
    ToolNotFound,
    /// Tool arguments failed schema/parse.
    ToolInvalidArgs,
    /// Tool execution failed.
    ToolExecution,
    /// Tool timed out.
    ToolTimeout,
    /// Tool cancelled.
    ToolCancelled,
    /// Tool denied by policy/capability.
    ToolDenied,
    /// LLM transport or provider failure.
    LlmProvider,
    /// LLM request cancelled.
    LlmCancelled,
    /// LLM response invalid.
    LlmInvalidResponse,
    /// Agent definition invalid.
    AgentInvalidDefinition,
    /// Agent build failure.
    AgentBuild,
    /// Turn hit max steps.
    RuntimeMaxSteps,
    /// Turn cancelled.
    RuntimeCancelled,
    /// Runtime gate rejected the outcome.
    RuntimeGate,
    /// Host spawn failed.
    HostSpawn,
    /// Agent budget exhausted.
    HostBudget,
    /// Host capability unsupported.
    HostUnsupported,
    /// Host cancelled.
    HostCancelled,
    /// Workflow script compile/runtime failure.
    WorkflowScript,
    /// Journal divergence on resume.
    WorkflowDivergence,
    /// Journal I/O or integrity failure.
    WorkflowJournal,
    /// Workflow agent budget exceeded.
    WorkflowBudget,
    /// Workflow cancelled.
    WorkflowCancelled,
    /// Generic internal failure.
    Internal,
}

impl ErrorCode {
    /// Stable `snake_case` code string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TypesInvalidId => "types.invalid_id",
            Self::TypesValidation => "types.validation",
            Self::TypesSerde => "types.serde",
            Self::ToolNotFound => "tool.not_found",
            Self::ToolInvalidArgs => "tool.invalid_args",
            Self::ToolExecution => "tool.execution",
            Self::ToolTimeout => "tool.timeout",
            Self::ToolCancelled => "tool.cancelled",
            Self::ToolDenied => "tool.denied",
            Self::LlmProvider => "llm.provider",
            Self::LlmCancelled => "llm.cancelled",
            Self::LlmInvalidResponse => "llm.invalid_response",
            Self::AgentInvalidDefinition => "agent.invalid_definition",
            Self::AgentBuild => "agent.build",
            Self::RuntimeMaxSteps => "runtime.max_steps",
            Self::RuntimeCancelled => "runtime.cancelled",
            Self::RuntimeGate => "runtime.gate",
            Self::HostSpawn => "host.spawn",
            Self::HostBudget => "host.budget",
            Self::HostUnsupported => "host.unsupported",
            Self::HostCancelled => "host.cancelled",
            Self::WorkflowScript => "workflow.script",
            Self::WorkflowDivergence => "workflow.divergence",
            Self::WorkflowJournal => "workflow.journal",
            Self::WorkflowBudget => "workflow.budget",
            Self::WorkflowCancelled => "workflow.cancelled",
            Self::Internal => "internal",
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Whether an automatic retry may be appropriate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum RetryClass {
    /// Do not retry.
    #[default]
    Never,
    /// Safe to retry immediately.
    Immediate,
    /// Retry with backoff.
    Backoff,
    /// Refresh credentials then retry.
    AuthRefresh,
}

/// Kernel error with stable code, message, and optional source.
#[derive(Debug, Clone, thiserror::Error)]
pub struct MachiError {
    code: ErrorCode,
    message: String,
    retry: RetryClass,
    #[source]
    source: Option<Arc<dyn std::error::Error + Send + Sync>>,
}

impl MachiError {
    /// Create an error with code and message.
    #[must_use]
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            retry: RetryClass::Never,
            source: None,
        }
    }

    /// Attach retry classification.
    #[must_use]
    pub const fn with_retry(mut self, retry: RetryClass) -> Self {
        self.retry = retry;
        self
    }

    /// Attach a source error.
    #[must_use]
    pub fn with_source(mut self, source: impl std::error::Error + Send + Sync + 'static) -> Self {
        self.source = Some(Arc::new(source));
        self
    }

    /// Stable code.
    #[must_use]
    pub const fn code(&self) -> ErrorCode {
        self.code
    }

    /// Retry class.
    #[must_use]
    pub const fn retry_class(&self) -> RetryClass {
        self.retry
    }

    /// Human-readable message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for MachiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

/// Result alias using [`MachiError`].
pub type Result<T> = std::result::Result<T, MachiError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_includes_code() {
        let err = MachiError::new(ErrorCode::ToolTimeout, "exceeded 5s");
        assert!(err.to_string().contains("tool.timeout"), "{err}");
    }
}
