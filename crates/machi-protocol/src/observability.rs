//! Stable tracing span names and field keys for OpenTelemetry-friendly hosts.

/// Session lifetime span.
pub const SPAN_SESSION: &str = "machi.session";
/// Single turn span.
pub const SPAN_TURN: &str = "machi.turn";
/// One LLM sample call.
pub const SPAN_SAMPLE: &str = "machi.sample";
/// One tool execution.
pub const SPAN_TOOL: &str = "machi.tool";
/// Dispatched tool batch.
pub const SPAN_TOOL_BATCH: &str = "machi.tool.batch";
/// Nested agent spawn.
pub const SPAN_SPAWN: &str = "machi.spawn";
/// Workflow run.
pub const SPAN_WORKFLOW: &str = "machi.workflow";
/// One workflow host request.
pub const SPAN_WORKFLOW_HOST: &str = "machi.workflow.host";
/// Compaction pass.
pub const SPAN_COMPACT: &str = "machi.compact";

/// Canonical field names (string constants for hosts and tests).
pub mod field {
    /// Session id.
    pub const SESSION_ID: &str = "machi.session_id";
    /// Agent id.
    pub const AGENT_ID: &str = "machi.agent_id";
    /// Agent name.
    pub const AGENT_NAME: &str = "machi.agent_name";
    /// Run / turn id.
    pub const RUN_ID: &str = "machi.run_id";
    /// Step index within a turn.
    pub const STEP: &str = "machi.step";
    /// Tool name.
    pub const TOOL_NAME: &str = "machi.tool_name";
    /// Model id.
    pub const MODEL: &str = "machi.model";
    /// Input tokens.
    pub const USAGE_INPUT: &str = "machi.usage.input_tokens";
    /// Output tokens.
    pub const USAGE_OUTPUT: &str = "machi.usage.output_tokens";
    /// Workflow run id.
    pub const WORKFLOW_RUN_ID: &str = "machi.workflow.run_id";
    /// Workflow journal sequence.
    pub const WORKFLOW_SEQ: &str = "machi.workflow.seq";
}

/// All required span names (contract test surface).
#[must_use]
pub fn required_span_names() -> &'static [&'static str] {
    &[
        SPAN_SESSION,
        SPAN_TURN,
        SPAN_SAMPLE,
        SPAN_TOOL,
        SPAN_TOOL_BATCH,
        SPAN_SPAWN,
        SPAN_WORKFLOW,
        SPAN_WORKFLOW_HOST,
        SPAN_COMPACT,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn span_names_are_machi_prefixed_and_unique() {
        let names = required_span_names();
        assert_eq!(names.len(), 9, "expected nine spans");
        let mut seen = std::collections::BTreeSet::new();
        for name in names {
            assert!(
                name.starts_with("machi."),
                "span {name} must start with machi."
            );
            assert!(seen.insert(*name), "duplicate span {name}");
        }
    }

    #[test]
    fn field_keys_are_stable() {
        assert_eq!(field::SESSION_ID, "machi.session_id");
        assert_eq!(field::TOOL_NAME, "machi.tool_name");
        assert_eq!(field::WORKFLOW_SEQ, "machi.workflow.seq");
    }
}
