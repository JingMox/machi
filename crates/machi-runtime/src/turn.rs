//! [`TurnRuntime`]: host-agnostic `ReAct` loop.

use machi_agent::Agent;
use machi_llm::{LlmSampler, SampleRequest, ToolChoice};
use machi_tools::registry::CapabilityMode;
use machi_tools::{DispatchRequest, ToolCallContext, ToolDispatch};
use machi_types::{AgentId, Deadline, ErrorCode, MachiError, Message, RunId, SessionId, Usage};
use serde_json::Value;
use tokio_util::sync::CancellationToken;
use tracing::{Instrument, info_span};

use crate::state::ConversationState;

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
#[derive(Debug, Clone)]
pub struct TurnOptions {
    /// Hard step ceiling.
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
        }
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
        let dispatch = ToolDispatch {
            max_concurrency: options.max_tool_concurrency,
            capability_mode: options.capability_mode,
        };

        loop {
            if cancelled(&options) {
                return Ok(TurnOutcome {
                    run_id,
                    output_text: String::new(),
                    output_json: None,
                    usage,
                    steps,
                    cancelled: true,
                });
            }
            if steps >= max_steps {
                return Err(MachiError::new(
                    ErrorCode::RuntimeMaxSteps,
                    format!("exceeded max_steps ({max_steps})"),
                ));
            }
            steps = steps.saturating_add(1);
            let step_u32 = u32::try_from(steps).unwrap_or(u32::MAX);

            let tools = agent.tools().definitions(options.capability_mode);
            let request = SampleRequest {
                model: agent.model().to_owned(),
                messages: state.messages().to_vec(),
                tools,
                tool_choice: ToolChoice::Auto,
                response_format: agent.definition().output_schema.clone(),
                max_output_tokens: None,
                temperature: None,
                cancel: options.cancel.clone(),
                deadline: options.deadline,
            };

            let sample_span = info_span!("machi.sample", machi.step = step_u32);
            let response = sampler.sample(request).instrument(sample_span).await?;
            usage += response.usage;

            let message = response.message;
            if message.tool_calls.is_empty() {
                return Ok(finalize_turn(agent, state, run_id, usage, steps, message));
            }

            state.append(message.clone());
            let ctx = ToolCallContext {
                cancel: options.cancel.clone(),
                deadline: options.deadline,
                cwd: None,
                session_id: options.session_id.clone(),
                agent_id: options.agent_id.clone(),
                extras: std::sync::Arc::new(std::collections::HashMap::new()),
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
    }
}

fn cancelled(options: &TurnOptions) -> bool {
    options.cancel.is_cancelled() || options.deadline.is_some_and(|d| d.is_expired())
}

fn finalize_turn(
    agent: &Agent,
    state: &mut dyn ConversationState,
    run_id: RunId,
    usage: Usage,
    steps: usize,
    message: Message,
) -> TurnOutcome {
    let text = message.text();
    state.append(message);
    let output_json = agent
        .definition()
        .output_schema
        .as_ref()
        .and_then(|_| serde_json::from_str(&text).ok());
    TurnOutcome {
        run_id,
        output_text: text,
        output_json,
        usage,
        steps,
        cancelled: false,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use machi_agent::AgentBuilder;
    use machi_llm::MockSampler;
    use machi_tools::{DynTool, ToolMetadata, ToolResult};
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
        assert_eq!(out.steps, 2);
        assert!(
            state
                .messages()
                .iter()
                .any(|m| m.role == machi_types::Role::Tool)
        );
    }

    #[tokio::test]
    async fn max_steps() {
        let sampler = Arc::new(MockSampler::new());
        for i in 0..5 {
            sampler.push_tools(Message::assistant_tools(vec![ToolCall {
                id: ToolCallId::new(format!("c{i}")).expect("id"),
                name: "echo".into(),
                arguments: json!({"text":"x"}),
            }]));
        }
        let agent = AgentBuilder::named("a")
            .model("mock")
            .max_steps(2)
            .tools(vec![Arc::new(EchoTool)])
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
}
