//! [`TurnRuntime`]: host-agnostic ReAct-style loop with stop gates and compaction.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use futures::StreamExt;
use machi_agent::Agent;
use machi_compaction::{CompactionStrategy, MaxMessages};
use machi_llm::{LlmSampler, SampleEvent, SampleRequest, SampleResponse, ToolChoice};
use machi_obs::{NoopMetrics, SharedMetrics, record_compaction, record_sample};
use machi_protocol::{
    MESSAGE_FRAME_TOKENS, PreflightOverflow, check_context_overflow, estimate_image_tokens,
    estimate_text_tokens,
};
use machi_tools::registry::CapabilityMode;
use machi_tools::{
    ApprovalGate, ApprovalPolicy, AutoApprove, DispatchRequest, ToolCallContext, ToolDispatch,
};
use machi_types::{
    AgentId, ContentPart, Deadline, ErrorCode, MachiError, Message, RunId, SessionId, Usage,
};
use serde_json::Value;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{Instrument, info_span};

use crate::gates::{GateChain, GateDecision};
use crate::lifecycle::{LifecycleFanout, TurnAbortReason, TurnLifecycleContributor};
use crate::schema::{
    STRUCTURED_OUTPUT_MAX_RETRIES, compile_schema, schema_retry_reminder,
    validate_structured_output,
};
use crate::state::ConversationState;
use crate::stationarity::{StationarityAction, StationarityTracker, nudge_message};

/// User-facing turn input.
#[derive(Debug, Clone)]
pub enum TurnInput {
    /// Plain text user message.
    Text(String),
    /// Pre-built message.
    Message(Message),
}

impl TurnInput {
    fn into_message(self) -> Message {
        match self {
            Self::Text(t) => Message::user(t),
            Self::Message(m) => m,
        }
    }
}

/// Options for a single turn.
#[derive(Clone)]
pub struct TurnOptions {
    /// Hard step ceiling (defaults to agent `max_steps`).
    pub max_steps: Option<usize>,
    /// Tool concurrency.
    pub max_tool_concurrency: usize,
    /// Capability mode for tools.
    pub capability_mode: CapabilityMode,
    /// Cancel token.
    pub cancel: CancellationToken,
    /// Deadline.
    pub deadline: Option<Deadline>,
    /// Session id for context.
    pub session_id: Option<SessionId>,
    /// Agent id for context.
    pub agent_id: Option<AgentId>,
    /// Working directory for path tools.
    pub cwd: Option<PathBuf>,
    /// Compaction strategy applied before each sample when present.
    pub compaction: Option<Arc<dyn CompactionStrategy>>,
    /// Approval gate for tools.
    pub approval: Arc<dyn ApprovalGate>,
    /// When to consult approval.
    pub approval_policy: ApprovalPolicy,
    /// Optional override stop-gate chain (default: from agent definition).
    pub stop_gates: Option<Arc<GateChain>>,
    /// Metrics sink (default no-op).
    pub metrics: SharedMetrics,
    /// Nesting depth when this turn is a host-spawned agent (`None` = top-level session).
    pub spawn_depth: Option<u32>,
    /// Optional max output tokens for the sampler.
    pub max_output_tokens: Option<u32>,
    /// Prefer [`LlmSampler::sample_stream`] and aggregate into a full response.
    pub use_stream: bool,
    /// Lifecycle contributors (W3.5).
    pub contributors: Arc<dyn TurnLifecycleContributor>,
    /// Optional mid-turn user interjections drained before each sample (W3.6).
    pub interject_rx: Option<Arc<tokio::sync::Mutex<mpsc::UnboundedReceiver<Message>>>>,
    /// Context window size for preflight overflow (tokens). `None` disables check.
    pub context_window_tokens: Option<u32>,
    /// Soft threshold ratio of context window (default 0.9).
    pub context_overflow_ratio: f32,
    /// When true, overflow after compaction is a hard error; else continue.
    pub fail_on_context_overflow: bool,
}

impl std::fmt::Debug for TurnOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TurnOptions")
            .field("max_steps", &self.max_steps)
            .field("max_tool_concurrency", &self.max_tool_concurrency)
            .field("capability_mode", &self.capability_mode)
            .field("cwd", &self.cwd)
            .field("has_compaction", &self.compaction.is_some())
            .field("approval_policy", &self.approval_policy)
            .field("spawn_depth", &self.spawn_depth)
            .field("max_output_tokens", &self.max_output_tokens)
            .field("use_stream", &self.use_stream)
            .field("context_window_tokens", &self.context_window_tokens)
            .field("context_overflow_ratio", &self.context_overflow_ratio)
            .field("fail_on_context_overflow", &self.fail_on_context_overflow)
            .finish_non_exhaustive()
    }
}

impl Default for TurnOptions {
    fn default() -> Self {
        Self {
            max_steps: None,
            max_tool_concurrency: 32,
            capability_mode: CapabilityMode::Full,
            cancel: CancellationToken::new(),
            deadline: None,
            session_id: None,
            agent_id: None,
            cwd: None,
            compaction: None,
            approval: Arc::new(AutoApprove),
            approval_policy: ApprovalPolicy::Destructive,
            stop_gates: None,
            metrics: Arc::new(NoopMetrics),
            spawn_depth: None,
            max_output_tokens: None,
            use_stream: false,
            contributors: Arc::new(LifecycleFanout::new()),
            interject_rx: None,
            context_window_tokens: None,
            context_overflow_ratio: 0.9,
            fail_on_context_overflow: true,
        }
    }
}

impl TurnOptions {
    /// Cap conversation length via [`MaxMessages`] strategy.
    ///
    /// # Errors
    ///
    /// Returns error when `max == 0`.
    pub fn with_max_messages(mut self, max: usize) -> Result<Self, MachiError> {
        self.compaction = Some(Arc::new(MaxMessages::new(max)?));
        Ok(self)
    }

    /// Install a compaction strategy.
    #[must_use]
    pub fn with_compaction(mut self, strategy: Arc<dyn CompactionStrategy>) -> Self {
        self.compaction = Some(strategy);
        self
    }

    /// Set cwd for tools.
    #[must_use]
    pub fn with_cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    /// Set capability mode.
    #[must_use]
    pub const fn with_capability(mut self, mode: CapabilityMode) -> Self {
        self.capability_mode = mode;
        self
    }

    /// Set max steps.
    #[must_use]
    pub const fn with_max_steps(mut self, max_steps: usize) -> Self {
        self.max_steps = Some(max_steps);
        self
    }

    /// Set cancel token.
    #[must_use]
    pub fn with_cancel(mut self, cancel: CancellationToken) -> Self {
        self.cancel = cancel;
        self
    }

    /// Set approval gate.
    #[must_use]
    pub fn with_approval(mut self, gate: Arc<dyn ApprovalGate>) -> Self {
        self.approval = gate;
        self
    }

    /// Set approval policy.
    #[must_use]
    pub const fn with_approval_policy(mut self, policy: ApprovalPolicy) -> Self {
        self.approval_policy = policy;
        self
    }

    /// Set metrics sink.
    #[must_use]
    pub fn with_metrics(mut self, metrics: SharedMetrics) -> Self {
        self.metrics = metrics;
        self
    }

    /// Set absolute deadline for the turn (sample + tools).
    #[must_use]
    pub const fn with_deadline(mut self, deadline: Deadline) -> Self {
        self.deadline = Some(deadline);
        self
    }

    /// Prefer streaming sample aggregation for this turn.
    #[must_use]
    pub const fn with_stream(mut self, use_stream: bool) -> Self {
        self.use_stream = use_stream;
        self
    }

    /// Override stop-gate chain.
    #[must_use]
    pub fn with_stop_gates(mut self, gates: Arc<GateChain>) -> Self {
        self.stop_gates = Some(gates);
        self
    }

    /// Install lifecycle contributors (W3.5).
    #[must_use]
    pub fn with_contributors(mut self, contributors: Arc<dyn TurnLifecycleContributor>) -> Self {
        self.contributors = contributors;
        self
    }

    /// Mid-turn interjection channel drained before each sample (W3.6).
    #[must_use]
    pub fn with_interject_rx(
        mut self,
        rx: Arc<tokio::sync::Mutex<mpsc::UnboundedReceiver<Message>>>,
    ) -> Self {
        self.interject_rx = Some(rx);
        self
    }

    /// Enable preflight context overflow checks (W3.2).
    #[must_use]
    pub const fn with_context_window(mut self, tokens: u32) -> Self {
        self.context_window_tokens = Some(tokens);
        self
    }
}

/// Successful or failed turn result.
#[derive(Debug, Clone)]
pub struct TurnOutcome {
    /// Run id.
    pub run_id: RunId,
    /// Final assistant text when completed normally.
    pub output_text: String,
    /// Optional structured JSON when schema mode produced parseable content.
    pub output_json: Option<Value>,
    /// Accumulated usage.
    pub usage: Usage,
    /// Steps consumed.
    pub steps: usize,
    /// Whether cancelled.
    pub cancelled: bool,
}

/// Stateless turn engine.
#[derive(Debug, Default, Clone, Copy)]
pub struct TurnRuntime;

impl TurnRuntime {
    /// Create a runtime.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Run one turn to completion.
    ///
    /// # Errors
    ///
    /// Returns typed runtime/LLM/tool failures.
    pub async fn run(
        &self,
        agent: &Agent,
        sampler: &dyn LlmSampler,
        state: &mut dyn ConversationState,
        input: TurnInput,
        options: TurnOptions,
    ) -> Result<TurnOutcome, MachiError> {
        let run_id = RunId::generate();
        let span = info_span!(
            "machi.turn",
            machi.run_id = %run_id,
            machi.agent_name = agent.name(),
            machi.model = agent.model(),
        );

        async move {
            self.run_inner(agent, sampler, state, input, options, run_id)
                .await
        }
        .instrument(span)
        .await
    }

    #[allow(
        clippy::too_many_lines,
        clippy::excessive_nesting,
        reason = "turn loop owns cancel/preflight/stationarity/dispatch/lifecycle"
    )]
    async fn run_inner(
        &self,
        agent: &Agent,
        sampler: &dyn LlmSampler,
        state: &mut dyn ConversationState,
        input: TurnInput,
        options: TurnOptions,
        run_id: RunId,
    ) -> Result<TurnOutcome, MachiError> {
        let agent_max = agent.max_steps();
        let max_steps = options.max_steps.unwrap_or(agent_max);
        if max_steps == 0 {
            return Err(MachiError::new(
                ErrorCode::RuntimeMaxSteps,
                "max_steps must be >= 1",
            ));
        }

        if state.messages().is_empty() && !agent.system_prompt().is_empty() {
            state.append(Message::system(agent.system_prompt()));
        }
        state.append(input.into_message());

        let mut usage = Usage::zero();
        let mut steps = 0usize;
        let mut completion_retries_used = 0u32;
        let mut schema_retries_used = 0u32;
        let schema_validator = match agent.definition().output_schema.as_ref() {
            Some(schema) => Some(compile_schema(schema)?),
            None => None,
        };
        let stop_gates = options
            .stop_gates
            .clone()
            .unwrap_or_else(|| Arc::new(GateChain::from_agent(agent)));
        let dispatch = ToolDispatch::default()
            .with_max_concurrency(options.max_tool_concurrency)
            .with_capability(options.capability_mode)
            .with_approval(Arc::clone(&options.approval))
            .with_approval_policy(options.approval_policy)
            .with_metrics(Arc::clone(&options.metrics));

        options.contributors.on_turn_start(&run_id);
        let mut stationarity = StationarityTracker::new();

        loop {
            if options.cancel.is_cancelled() {
                options
                    .contributors
                    .on_turn_abort(&run_id, &TurnAbortReason::Cancelled);
                return Ok(empty_cancelled(run_id, usage, steps));
            }
            if deadline_expired(&options) {
                let err = MachiError::new(ErrorCode::RuntimeDeadline, "turn deadline expired");
                options
                    .contributors
                    .on_turn_abort(&run_id, &TurnAbortReason::from_error(&err));
                return Err(err);
            }
            if steps >= max_steps {
                let err = MachiError::new(
                    ErrorCode::RuntimeMaxSteps,
                    format!("exceeded max_steps ({max_steps})"),
                );
                options
                    .contributors
                    .on_turn_abort(&run_id, &TurnAbortReason::from_error(&err));
                return Err(err);
            }
            steps = steps.saturating_add(1);
            let step_u32 = u32::try_from(steps).unwrap_or(u32::MAX);

            drain_interjections(state, &options).await;

            maybe_compact(
                state,
                options.compaction.as_deref(),
                options.metrics.as_ref(),
            )?;

            if let Err(err) = preflight_context_check(state, &options) {
                options.contributors.on_turn_error(&run_id, &err);
                return Err(err);
            }

            let tools = agent.tools().definitions(options.capability_mode);
            let request = SampleRequest {
                model: agent.model().to_owned(),
                messages: state.messages().to_vec(),
                tools,
                tool_choice: ToolChoice::Auto,
                response_format: agent.definition().output_schema.clone(),
                max_output_tokens: options.max_output_tokens,
                temperature: None,
                cancel: options.cancel.clone(),
                deadline: options.deadline,
            };

            let sample_span = info_span!("machi.sample", machi.step = step_u32);
            let sample_started = Instant::now();
            let response = match sample_once(sampler, request, &options)
                .instrument(sample_span)
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    options.contributors.on_turn_error(&run_id, &e);
                    return map_sample_error(e, &options, run_id, usage, steps);
                }
            };
            let sample_ms = sample_started.elapsed().as_secs_f64() * 1000.0;
            record_sample(
                options.metrics.as_ref(),
                sample_ms,
                u64::from(response.usage.input_tokens),
                u64::from(response.usage.output_tokens),
            );
            usage += response.usage;

            let message = response.message;
            if message.tool_calls.is_empty() {
                stationarity.reset();
                let mut final_ctx = FinalCtx {
                    agent,
                    state,
                    message: &message,
                    schema_validator: schema_validator.as_ref(),
                    stop_gates: stop_gates.as_ref(),
                    completion_retries_used: &mut completion_retries_used,
                    schema_retries_used: &mut schema_retries_used,
                    run_id: run_id.clone(),
                    usage,
                    steps,
                };
                match handle_final_assistant(&mut final_ctx)? {
                    FinalStep::Done(outcome) => {
                        options.contributors.on_turn_done(&run_id, outcome.steps);
                        return Ok(outcome);
                    }
                    FinalStep::Continue => continue,
                }
            }

            match stationarity.observe_tool_batch(&message.tool_calls) {
                StationarityAction::Ok => {}
                StationarityAction::Nudge { reminder } => {
                    state.append(nudge_message(reminder));
                }
                StationarityAction::HardStop { error } => {
                    options
                        .contributors
                        .on_turn_abort(&run_id, &TurnAbortReason::from_error(&error));
                    return Err(error);
                }
            }

            dispatch_tools(agent, state, &options, &dispatch, message, step_u32).await;
        }
    }
}

fn preflight_context_check(
    state: &dyn ConversationState,
    options: &TurnOptions,
) -> Result<(), MachiError> {
    let Some(window) = options.context_window_tokens else {
        return Ok(());
    };
    let estimated = estimate_conversation_tokens(state.messages());
    match check_context_overflow(estimated, window, options.context_overflow_ratio) {
        PreflightOverflow::Ok { .. } => Ok(()),
        PreflightOverflow::Overflow {
            estimated,
            limit,
            window,
        } if options.fail_on_context_overflow => Err(MachiError::new(
            ErrorCode::CompactionOverflow,
            format!(
                "context overflow: estimated {estimated} tokens exceeds limit {limit} (window {window})"
            ),
        )),
        PreflightOverflow::Overflow { .. } => Ok(()),
    }
}

async fn drain_interjections(state: &mut dyn ConversationState, options: &TurnOptions) {
    let Some(rx) = &options.interject_rx else {
        return;
    };
    let mut guard = rx.lock().await;
    while let Ok(msg) = guard.try_recv() {
        state.append(msg);
    }
}

/// Estimate tokens for a conversation using protocol heuristics.
#[must_use]
#[allow(
    clippy::excessive_nesting,
    reason = "message part/tool fold is a shallow match"
)]
pub fn estimate_conversation_tokens(messages: &[Message]) -> u32 {
    messages
        .iter()
        .fold(0u32, |acc, m| acc.saturating_add(estimate_one_message(m)))
}

fn estimate_one_message(m: &Message) -> u32 {
    let mut n = MESSAGE_FRAME_TOKENS;
    if m.parts.is_empty() {
        n = n.saturating_add(estimate_text_tokens(&m.text()));
    } else {
        for part in &m.parts {
            n = n.saturating_add(estimate_part(part));
        }
    }
    for call in &m.tool_calls {
        n = n.saturating_add(estimate_text_tokens(&call.name));
        n = n.saturating_add(estimate_text_tokens(&call.arguments.to_string()));
    }
    n
}

fn estimate_part(part: &ContentPart) -> u32 {
    match part {
        ContentPart::Text { text } => estimate_text_tokens(text),
        ContentPart::Image { .. } => estimate_image_tokens(),
        _ => 0,
    }
}

enum FinalStep {
    Done(TurnOutcome),
    Continue,
}

struct FinalCtx<'a> {
    agent: &'a Agent,
    state: &'a mut dyn ConversationState,
    message: &'a Message,
    schema_validator: Option<&'a jsonschema::Validator>,
    stop_gates: &'a GateChain,
    completion_retries_used: &'a mut u32,
    schema_retries_used: &'a mut u32,
    run_id: RunId,
    usage: Usage,
    steps: usize,
}

fn handle_final_assistant(ctx: &mut FinalCtx<'_>) -> Result<FinalStep, MachiError> {
    ctx.state.append(ctx.message.clone());

    if let Some(validator) = ctx.schema_validator {
        match validate_structured_output(validator, &ctx.message.text()) {
            Ok(value) => return apply_stop_gates(ctx, Some(value)),
            Err(err) => {
                if *ctx.schema_retries_used >= STRUCTURED_OUTPUT_MAX_RETRIES {
                    return Err(MachiError::new(
                        ErrorCode::RuntimeStructuredOutput,
                        format!(
                            "structured output invalid after {STRUCTURED_OUTPUT_MAX_RETRIES} retries: {err}"
                        ),
                    ));
                }
                *ctx.schema_retries_used = ctx.schema_retries_used.saturating_add(1);
                ctx.state.append(Message::user(schema_retry_reminder(&err)));
                return Ok(FinalStep::Continue);
            }
        }
    }

    apply_stop_gates(ctx, None)
}

fn apply_stop_gates(
    ctx: &mut FinalCtx<'_>,
    output_json: Option<Value>,
) -> Result<FinalStep, MachiError> {
    match ctx
        .stop_gates
        .evaluate(ctx.agent, ctx.state, *ctx.completion_retries_used)
    {
        GateDecision::Complete => Ok(FinalStep::Done(TurnOutcome {
            run_id: ctx.run_id.clone(),
            output_text: ctx.message.text(),
            output_json: output_json.or_else(|| {
                ctx.agent
                    .definition()
                    .output_schema
                    .as_ref()
                    .and_then(|_| serde_json::from_str(&ctx.message.text()).ok())
            }),
            usage: ctx.usage,
            steps: ctx.steps,
            cancelled: false,
        })),
        GateDecision::Continue { reminder } => {
            *ctx.completion_retries_used = ctx.completion_retries_used.saturating_add(1);
            ctx.state.append(Message::user(reminder));
            Ok(FinalStep::Continue)
        }
    }
}

async fn dispatch_tools(
    agent: &Agent,
    state: &mut dyn ConversationState,
    options: &TurnOptions,
    dispatch: &ToolDispatch,
    message: Message,
    step_u32: u32,
) {
    state.append(message.clone());
    let mut extras = std::collections::HashMap::new();
    if let Some(depth) = options.spawn_depth {
        extras.insert(machi_tools::EXTRA_SPAWN_DEPTH.to_owned(), depth.to_string());
    }
    let ctx = ToolCallContext {
        cancel: options.cancel.clone(),
        deadline: options.deadline,
        cwd: options.cwd.clone(),
        session_id: options.session_id.clone(),
        agent_id: options.agent_id.clone(),
        extras: Arc::new(extras),
    };
    let requests: Vec<DispatchRequest> = message
        .tool_calls
        .into_iter()
        .map(|call| DispatchRequest { call })
        .collect();
    let batch_span = info_span!("machi.tool.batch", machi.step = step_u32);
    let outcomes = dispatch
        .execute_batch(agent.tools(), ctx, requests)
        .instrument(batch_span)
        .await;
    for out in outcomes {
        let content = match out.result {
            Ok(r) => r.content,
            Err(e) => format!("error: {e}"),
        };
        state.append(Message::tool_result(out.id, out.name, content));
    }
}

fn maybe_compact(
    state: &mut dyn ConversationState,
    strategy: Option<&dyn CompactionStrategy>,
    metrics: &dyn machi_obs::MetricsSink,
) -> Result<(), MachiError> {
    let Some(strategy) = strategy else {
        return Ok(());
    };
    let msgs = state.messages();
    let tokens = state.token_estimate();
    if !strategy.should_compact(msgs, tokens) {
        return Ok(());
    }
    let name = strategy.name();
    match strategy.compact(msgs.to_vec()) {
        Ok(outcome) => {
            if outcome.changed {
                state.replace(outcome.messages);
                record_compaction(metrics, name, "ok");
            }
            Ok(())
        }
        Err(e) => {
            record_compaction(metrics, name, "error");
            Err(e)
        }
    }
}

fn deadline_expired(options: &TurnOptions) -> bool {
    options.deadline.is_some_and(|d| d.is_expired())
}

async fn sample_once(
    sampler: &dyn LlmSampler,
    request: SampleRequest,
    options: &TurnOptions,
) -> Result<SampleResponse, MachiError> {
    if options.use_stream {
        let stream = sampler.sample_stream(request).await?;
        collect_sample_stream(stream).await
    } else {
        sampler.sample(request).await
    }
}

async fn collect_sample_stream(
    mut stream: machi_llm::SampleStream,
) -> Result<SampleResponse, MachiError> {
    let mut message: Option<Message> = None;
    let mut usage = Usage::zero();
    let mut stop_reason = None;
    let mut text_buf = String::new();

    while let Some(ev) = stream.next().await {
        match ev {
            SampleEvent::TextDelta { text } => text_buf.push_str(&text),
            SampleEvent::ToolCalls { message: m } => message = Some(m),
            SampleEvent::Usage(u) => usage += u,
            SampleEvent::Completed {
                message: m,
                stop_reason: reason,
            } => {
                message = Some(m);
                stop_reason = reason;
            }
            SampleEvent::Failed { message: msg } => {
                return Err(MachiError::new(ErrorCode::LlmInvalidResponse, msg));
            }
            _ => {}
        }
    }

    let message = match message {
        Some(m) => m,
        None if !text_buf.is_empty() => Message::assistant(text_buf),
        None => {
            return Err(MachiError::new(
                ErrorCode::LlmInvalidResponse,
                "sample stream ended without Completed",
            ));
        }
    };

    Ok(SampleResponse {
        message,
        usage,
        stop_reason,
    })
}

fn map_sample_error(
    e: MachiError,
    options: &TurnOptions,
    run_id: RunId,
    usage: Usage,
    steps: usize,
) -> Result<TurnOutcome, MachiError> {
    if deadline_expired(options) {
        return Err(
            MachiError::new(ErrorCode::RuntimeDeadline, e.message().to_owned()).with_source(e),
        );
    }
    if e.code() == ErrorCode::LlmCancelled || options.cancel.is_cancelled() {
        return Ok(TurnOutcome {
            run_id,
            output_text: String::new(),
            output_json: None,
            usage,
            steps,
            cancelled: true,
        });
    }
    Err(e)
}

fn empty_cancelled(run_id: RunId, usage: Usage, steps: usize) -> TurnOutcome {
    TurnOutcome {
        run_id,
        output_text: String::new(),
        output_json: None,
        usage,
        steps,
        cancelled: true,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use machi_agent::AgentBuilder;
    use machi_llm::MockSampler;
    use machi_tools::{AlwaysDeny, DynTool, ToolMetadata, ToolResult};
    use machi_types::{Message, ToolCall, ToolCallId};
    use serde_json::{Value, json};

    use super::*;
    use crate::state::VecConversationState;

    struct EchoTool;

    #[async_trait]
    impl DynTool for EchoTool {
        fn name(&self) -> &'static str {
            "echo"
        }
        fn description(&self) -> &'static str {
            "echo"
        }
        fn parameters(&self) -> Value {
            json!({"type":"object","properties":{"text":{"type":"string"}},"required":["text"]})
        }
        fn metadata(&self) -> ToolMetadata {
            ToolMetadata::read_only()
        }
        async fn call(
            &self,
            _ctx: ToolCallContext,
            arguments: Value,
        ) -> Result<ToolResult, MachiError> {
            let text = arguments
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or_default();
            Ok(ToolResult::text(text))
        }
    }

    struct WriteStub;

    #[async_trait]
    impl DynTool for WriteStub {
        fn name(&self) -> &'static str {
            "write_stub"
        }
        fn description(&self) -> &'static str {
            "write"
        }
        fn parameters(&self) -> Value {
            json!({"type":"object","properties":{}})
        }
        fn metadata(&self) -> ToolMetadata {
            ToolMetadata::exclusive_write()
        }
        async fn call(
            &self,
            _ctx: ToolCallContext,
            _arguments: Value,
        ) -> Result<ToolResult, MachiError> {
            Ok(ToolResult::text("wrote"))
        }
    }

    struct SubmitTool;

    #[async_trait]
    impl DynTool for SubmitTool {
        fn name(&self) -> &'static str {
            "submit"
        }
        fn description(&self) -> &'static str {
            "submit"
        }
        fn parameters(&self) -> Value {
            json!({"type":"object","properties":{}})
        }
        fn metadata(&self) -> ToolMetadata {
            ToolMetadata::read_only()
        }
        async fn call(
            &self,
            _ctx: ToolCallContext,
            _arguments: Value,
        ) -> Result<ToolResult, MachiError> {
            Ok(ToolResult::text("submitted"))
        }
    }

    #[tokio::test]
    async fn tool_then_final() {
        let sampler = Arc::new(MockSampler::new());
        let id = ToolCallId::new("c1").expect("id");
        sampler.push_tools(Message::assistant_tools(vec![ToolCall {
            id,
            name: "echo".into(),
            arguments: json!({"text":"pong"}),
        }]));
        sampler.push_text("done");

        let agent = AgentBuilder::named("a")
            .model("mock")
            .tools(vec![Arc::new(EchoTool)])
            .build()
            .expect("agent");
        let mut state = VecConversationState::new();
        let out = TurnRuntime::new()
            .run(
                &agent,
                sampler.as_ref(),
                &mut state,
                TurnInput::Text("ping".into()),
                TurnOptions::default(),
            )
            .await
            .expect("turn");
        assert_eq!(out.output_text, "done");
        assert!(out.steps >= 2);
    }

    #[tokio::test]
    async fn max_steps() {
        let sampler = Arc::new(MockSampler::new());
        let id = ToolCallId::new("c1").expect("id");
        sampler.push_tools(Message::assistant_tools(vec![ToolCall {
            id: id.clone(),
            name: "echo".into(),
            arguments: json!({"text":"x"}),
        }]));
        sampler.push_tools(Message::assistant_tools(vec![ToolCall {
            id,
            name: "echo".into(),
            arguments: json!({"text":"y"}),
        }]));
        let agent = AgentBuilder::named("a")
            .model("mock")
            .tools(vec![Arc::new(EchoTool)])
            .max_steps(1)
            .build()
            .expect("agent");
        let mut state = VecConversationState::new();
        let err = TurnRuntime::new()
            .run(
                &agent,
                sampler.as_ref(),
                &mut state,
                TurnInput::Text("ping".into()),
                TurnOptions::default(),
            )
            .await
            .expect_err("max steps");
        assert_eq!(err.code(), ErrorCode::RuntimeMaxSteps);
    }

    #[tokio::test]
    async fn approval_denies_write_tool() {
        let sampler = Arc::new(MockSampler::new());
        let id = ToolCallId::new("w1").expect("id");
        sampler.push_tools(Message::assistant_tools(vec![ToolCall {
            id,
            name: "write_stub".into(),
            arguments: json!({}),
        }]));
        sampler.push_text("after-deny");
        let agent = AgentBuilder::named("a")
            .model("mock")
            .tools(vec![Arc::new(WriteStub)])
            .build()
            .expect("agent");
        let mut state = VecConversationState::new();
        let out = TurnRuntime::new()
            .run(
                &agent,
                sampler.as_ref(),
                &mut state,
                TurnInput::Text("write".into()),
                TurnOptions::default().with_approval(Arc::new(AlwaysDeny)),
            )
            .await
            .expect("turn continues after tool error");
        assert_eq!(out.output_text, "after-deny");
        let texts: String = state.messages().iter().map(Message::text).collect();
        assert!(
            texts.contains("approval denied") || texts.contains("error:"),
            "{texts}"
        );
    }

    #[tokio::test]
    async fn max_messages_compaction() {
        let sampler = Arc::new(MockSampler::new());
        sampler.push_text("ok");
        let agent = AgentBuilder::named("a")
            .model("mock")
            .build()
            .expect("agent");
        let mut state = VecConversationState::from_messages(vec![
            Message::system("sys"),
            Message::user("1"),
            Message::user("2"),
            Message::user("3"),
            Message::user("4"),
        ]);
        let opts = TurnOptions::default().with_max_messages(3).expect("max");
        let _ = TurnRuntime::new()
            .run(
                &agent,
                sampler.as_ref(),
                &mut state,
                TurnInput::Text("new".into()),
                opts,
            )
            .await
            .expect("turn");
        // system + compacted tail + new user + assistant at least
        assert!(state.messages().len() <= 6);
        assert_eq!(
            state.messages().first().map(Message::text).as_deref(),
            Some("sys")
        );
    }

    #[tokio::test]
    async fn cancel_before_sample() {
        let sampler = Arc::new(MockSampler::new());
        sampler.push_text("should-not-run");
        let agent = AgentBuilder::named("a")
            .model("mock")
            .build()
            .expect("agent");
        let mut state = VecConversationState::new();
        let cancel = CancellationToken::new();
        cancel.cancel();
        let out = TurnRuntime::new()
            .run(
                &agent,
                sampler.as_ref(),
                &mut state,
                TurnInput::Text("hi".into()),
                TurnOptions::default().with_cancel(cancel),
            )
            .await
            .expect("cancelled ok");
        assert!(out.cancelled);
        assert!(out.output_text.is_empty());
    }

    #[tokio::test]
    async fn deadline_before_sample() {
        use std::time::Duration;

        let sampler = Arc::new(MockSampler::new());
        sampler.push_text("late");
        let agent = AgentBuilder::named("a")
            .model("mock")
            .build()
            .expect("agent");
        let mut state = VecConversationState::new();
        let err = TurnRuntime::new()
            .run(
                &agent,
                sampler.as_ref(),
                &mut state,
                TurnInput::Text("hi".into()),
                TurnOptions::default().with_deadline(Deadline::after(Duration::ZERO)),
            )
            .await
            .expect_err("deadline");
        assert_eq!(err.code(), ErrorCode::RuntimeDeadline);
    }

    #[tokio::test]
    async fn structured_output_retry_then_ok() {
        let sampler = Arc::new(MockSampler::new());
        sampler.push_text("not-json");
        sampler.push_text(r#"{"ok":true}"#);
        let schema = json!({
            "type": "object",
            "properties": { "ok": { "type": "boolean" } },
            "required": ["ok"]
        });
        let agent = AgentBuilder::named("a")
            .model("mock")
            .output_schema(schema)
            .max_steps(8)
            .build()
            .expect("agent");
        let mut state = VecConversationState::new();
        let out = TurnRuntime::new()
            .run(
                &agent,
                sampler.as_ref(),
                &mut state,
                TurnInput::Text("give json".into()),
                TurnOptions::default(),
            )
            .await
            .expect("turn");
        assert_eq!(out.output_json, Some(json!({"ok": true})));
        assert!(out.steps >= 2);
    }

    #[tokio::test]
    async fn structured_output_exhausted() {
        let sampler = Arc::new(MockSampler::new());
        // initial + STRUCTURED_OUTPUT_MAX_RETRIES bad attempts
        for _ in 0..=STRUCTURED_OUTPUT_MAX_RETRIES {
            sampler.push_text("nope");
        }
        let schema = json!({
            "type": "object",
            "properties": { "ok": { "type": "boolean" } },
            "required": ["ok"]
        });
        let agent = AgentBuilder::named("a")
            .model("mock")
            .output_schema(schema)
            .max_steps(16)
            .build()
            .expect("agent");
        let mut state = VecConversationState::new();
        let err = TurnRuntime::new()
            .run(
                &agent,
                sampler.as_ref(),
                &mut state,
                TurnInput::Text("give json".into()),
                TurnOptions::default(),
            )
            .await
            .expect_err("schema");
        assert_eq!(err.code(), ErrorCode::RuntimeStructuredOutput);
    }

    #[tokio::test]
    async fn completion_gate_forces_retry() {
        use machi_agent::CompletionRequirement;

        let sampler = Arc::new(MockSampler::new());
        // first final without submit tool → gate continues; second final after tools
        sampler.push_text("thinking");
        let id = ToolCallId::new("s1").expect("id");
        sampler.push_tools(Message::assistant_tools(vec![ToolCall {
            id,
            name: "submit".into(),
            arguments: json!({}),
        }]));
        sampler.push_text("done");

        let agent = AgentBuilder::named("a")
            .model("mock")
            .tools(vec![Arc::new(SubmitTool)])
            .completion(CompletionRequirement {
                tool: "submit".into(),
                reminder: "please call submit".into(),
                max_retries: 3,
            })
            .max_steps(8)
            .build()
            .expect("agent");
        let mut state = VecConversationState::new();
        let out = TurnRuntime::new()
            .run(
                &agent,
                sampler.as_ref(),
                &mut state,
                TurnInput::Text("finish".into()),
                TurnOptions::default(),
            )
            .await
            .expect("turn");
        assert_eq!(out.output_text, "done");
        assert!(
            state
                .messages()
                .iter()
                .any(|m| m.text().contains("please call submit")),
            "expected completion reminder"
        );
    }

    #[tokio::test]
    async fn stream_sample_path() {
        let sampler = Arc::new(MockSampler::new());
        sampler.push_text("streamed-out");
        let agent = AgentBuilder::named("a")
            .model("mock")
            .build()
            .expect("agent");
        let mut state = VecConversationState::new();
        let out = TurnRuntime::new()
            .run(
                &agent,
                sampler.as_ref(),
                &mut state,
                TurnInput::Text("hi".into()),
                TurnOptions::default().with_stream(true),
            )
            .await
            .expect("turn");
        assert_eq!(out.output_text, "streamed-out");
    }

    #[tokio::test]
    async fn token_threshold_compaction() {
        use machi_compaction::TokenThreshold;

        let sampler = Arc::new(MockSampler::new());
        sampler.push_text("ok");
        let agent = AgentBuilder::named("a")
            .model("mock")
            .build()
            .expect("agent");
        let mut state = VecConversationState::from_messages(vec![
            Message::system("sys"),
            Message::user("aaaaaaaaaa"),
            Message::user("bbbbbbbbbb"),
            Message::user("cccccccccc"),
            Message::user("dddddddddd"),
        ]);
        // token_estimate is coarse; force trigger with low max_tokens
        let strategy = TokenThreshold::new(1, 3).expect("strategy");
        let opts = TurnOptions::default().with_compaction(Arc::new(strategy));
        let _ = TurnRuntime::new()
            .run(
                &agent,
                sampler.as_ref(),
                &mut state,
                TurnInput::Text("new".into()),
                opts,
            )
            .await
            .expect("turn");
        assert_eq!(
            state.messages().first().map(Message::text).as_deref(),
            Some("sys")
        );
    }
}
