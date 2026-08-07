//! Machi — enterprise embeddable agent runtime kernel.
//!
//! # Layers
//!
//! - **types** — messages, ids, usage, errors
//! - **tools** — tool trait, registry, concurrent dispatch
//! - **llm** — [`LlmSampler`](machi_llm::LlmSampler) + [`MockSampler`](machi_llm::MockSampler)
//! - **agent** — definition / builder / instance
//! - **runtime** — [`TurnRuntime`](machi_runtime::TurnRuntime), [`SessionHost`](machi_runtime::SessionHost)
//! - **workflow** — journaled Rhai orchestration (no LLM dependency)
//!
//! This is a **clean break** from Machi ≤0.8 (`Runner`, etc.). See
//! `docs/architecture/kernel.md`.

#![forbid(unsafe_code)]
// Feature-gated transitive deps are unused when compiling the facade lib alone.
#![allow(
    unused_crate_dependencies,
    reason = "facade re-exports optional workspace crates"
)]

#[cfg(feature = "runtime")]
pub use machi_agent as agent;
#[cfg(feature = "runtime")]
pub use machi_agent::{
    Agent, AgentBuilder, AgentDefinition, CompletionRequirement, Instructions, ToolPolicy,
};
#[cfg(feature = "runtime")]
pub use machi_llm as llm;
#[cfg(feature = "openai")]
pub use machi_llm::OpenAiCompatSampler;
#[cfg(feature = "runtime")]
pub use machi_llm::{
    LlmSampler, MockSampler, OpenAiCompatConfig, SampleRequest, SampleResponse, ToolChoice,
    build_chat_completions_body, parse_chat_completions_response,
};
#[cfg(feature = "ollama")]
pub use machi_llm::{OllamaConfig, OllamaSampler};
#[cfg(feature = "runtime")]
pub use machi_runtime as runtime;
#[cfg(all(feature = "runtime", feature = "workflow"))]
pub use machi_runtime::run_workflow_on_host;
#[cfg(feature = "runtime")]
pub use machi_runtime::{
    AgentRunResult, ConversationState, InProcessHost, MetricsSink, NoopMetrics, Session,
    SessionHost, SharedMetrics, SpawnAgentTool, SpawnOpts, TurnInput, TurnOptions, TurnOutcome,
    TurnRuntime, VecConversationState,
};
#[cfg(feature = "runtime")]
pub use machi_tools as tools;
#[cfg(feature = "runtime")]
pub use machi_tools::{
    CalcTool, CapabilityFlag, CapabilityMode, ConcurrencyMode, Destructiveness, DispatchRequest,
    DynTool, InterruptBehavior, SharedTool, ToolCallContext, ToolDefinition, ToolDispatch,
    ToolError, ToolMetadata, ToolRegistry, ToolResult,
};
pub use machi_types as types;
pub use machi_types::{
    AgentId, CompletionTokensDetails, ContentPart, Deadline, ErrorCode, ImageMime, MachiError,
    Message, PromptTokensDetails, Result, RetryClass, Role, RunId, SessionId, ToolCall, ToolCallId,
    Usage, WorkflowRunId,
};
#[cfg(feature = "workflow")]
pub use machi_workflow as workflow;
#[cfg(feature = "workflow")]
pub use machi_workflow::{
    AgentOpts, AgentResult as WorkflowAgentResult, BudgetState, DEFAULT_AGENT_BUDGET, HostError,
    Journal, JournalEntry, JournalError, MAX_AGENT_BUDGET, PauseKind, WorkflowHostRequest,
    WorkflowMeta, WorkflowOutcome, WorkflowRunParams, extract_meta, run_workflow,
};
