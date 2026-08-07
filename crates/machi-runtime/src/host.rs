//! Session host for nested agent runs (dynamic multi-agent delegation).

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use async_trait::async_trait;
use futures::future::try_join_all;
use machi_agent::{Agent, AgentBuilder};
use machi_llm::LlmSampler;
use machi_tools::SharedTool;
use machi_tools::registry::CapabilityMode;
use machi_types::{AgentId, ErrorCode, MachiError, Usage};
use serde_json::Value;
use tokio_util::sync::CancellationToken;
use tracing::{Instrument, info_span};

use crate::state::VecConversationState;
use crate::turn::{TurnInput, TurnOptions, TurnRuntime};

/// Options for spawning a nested agent.
#[derive(Debug, Clone)]
pub struct SpawnOpts {
    /// User prompt for the child.
    pub prompt: String,
    /// Optional label for logs / result correlation.
    pub label: Option<String>,
    /// Override model.
    pub model: Option<String>,
    /// Capability mode for child tools.
    pub capability_mode: CapabilityMode,
    /// Max steps for child turn.
    pub max_steps: Option<usize>,
    /// Cancel token for this child.
    pub cancel: CancellationToken,
}

impl SpawnOpts {
    /// Prompt-only spawn with a fresh cancel token.
    #[must_use]
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            label: None,
            model: None,
            capability_mode: CapabilityMode::Full,
            max_steps: None,
            cancel: CancellationToken::new(),
        }
    }

    /// Set label.
    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set capability mode.
    #[must_use]
    pub const fn with_capability(mut self, mode: CapabilityMode) -> Self {
        self.capability_mode = mode;
        self
    }

    /// Set cancel token (often a child of a parent token).
    #[must_use]
    pub fn with_cancel(mut self, cancel: CancellationToken) -> Self {
        self.cancel = cancel;
        self
    }

    /// Set max steps.
    #[must_use]
    pub const fn with_max_steps(mut self, max_steps: usize) -> Self {
        self.max_steps = Some(max_steps);
        self
    }
}

/// Result of a nested agent run.
#[derive(Debug, Clone)]
pub struct AgentRunResult {
    /// Child agent id.
    pub agent_id: AgentId,
    /// Optional label echoed from [`SpawnOpts`].
    pub label: Option<String>,
    /// Whether the run completed without runtime error.
    pub success: bool,
    /// Model text or structured payload.
    pub output: Value,
    /// Cancelled flag.
    pub cancelled: bool,
    /// Usage.
    pub usage: Usage,
    /// Wall duration ms.
    pub duration_ms: u64,
    /// Steps.
    pub steps: usize,
}

/// Host capable of spawning nested agents.
#[async_trait]
pub trait SessionHost: Send + Sync {
    /// Spawn and run a nested agent to completion.
    async fn spawn_agent(&self, opts: SpawnOpts) -> Result<AgentRunResult, MachiError>;

    /// Spawn many nested agents concurrently (order of results matches input).
    async fn spawn_agents(&self, opts: Vec<SpawnOpts>) -> Result<Vec<AgentRunResult>, MachiError> {
        try_join_all(opts.into_iter().map(|o| self.spawn_agent(o))).await
    }
}

/// In-process host: nested [`TurnRuntime`] with shared sampler, tool pool, and budget.
pub struct InProcessHost {
    sampler: Arc<dyn LlmSampler>,
    tools: Vec<SharedTool>,
    base_instructions: String,
    runtime: TurnRuntime,
    /// Absolute cap on nested agent spawns (`None` = unlimited).
    agent_budget: Option<u64>,
    spent: AtomicU64,
}

impl std::fmt::Debug for InProcessHost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InProcessHost")
            .field("tools", &self.tools.len())
            .field("base_instructions_len", &self.base_instructions.len())
            .field("runtime", &self.runtime)
            .field("agent_budget", &self.agent_budget)
            .field("spent", &self.spent.load(Ordering::Relaxed))
            .finish_non_exhaustive()
    }
}

impl InProcessHost {
    /// Create a host with unlimited agent budget.
    #[must_use]
    pub fn new(sampler: Arc<dyn LlmSampler>, tools: Vec<SharedTool>) -> Self {
        Self {
            sampler,
            tools,
            base_instructions: "You are a focused sub-agent. Complete the task.".into(),
            runtime: TurnRuntime::new(),
            agent_budget: None,
            spent: AtomicU64::new(0),
        }
    }

    /// Absolute agent-call budget for this host (every successful admission counts 1).
    #[must_use]
    pub const fn with_agent_budget(mut self, budget: u64) -> Self {
        self.agent_budget = Some(budget);
        self
    }

    /// Override child system instructions.
    #[must_use]
    pub fn with_instructions(mut self, instructions: impl Into<String>) -> Self {
        self.base_instructions = instructions.into();
        self
    }

    /// Agents admitted so far (including in-flight after reservation).
    #[must_use]
    pub fn agents_spent(&self) -> u64 {
        self.spent.load(Ordering::Relaxed)
    }

    /// Remaining budget when capped.
    #[must_use]
    pub fn agents_remaining(&self) -> Option<u64> {
        self.agent_budget
            .map(|b| b.saturating_sub(self.spent.load(Ordering::Relaxed)))
    }

    fn reserve_slot(&self) -> Result<(), MachiError> {
        let Some(budget) = self.agent_budget else {
            self.spent.fetch_add(1, Ordering::Relaxed);
            return Ok(());
        };
        self.reserve_against_budget(budget)
    }

    fn reserve_against_budget(&self, budget: u64) -> Result<(), MachiError> {
        loop {
            let spent = self.spent.load(Ordering::Acquire);
            if spent >= budget {
                return Err(MachiError::new(
                    ErrorCode::HostBudget,
                    format!("agent budget exhausted: spent {spent}, maximum {budget}"),
                ));
            }
            if self
                .spent
                .compare_exchange(spent, spent + 1, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return Ok(());
            }
        }
    }

    fn build_child(&self, opts: &SpawnOpts) -> Result<Agent, MachiError> {
        let name = opts.label.clone().unwrap_or_else(|| "subagent".to_owned());
        let mut builder = AgentBuilder::named(name)
            .instructions(self.base_instructions.clone())
            .tools(self.tools.clone());
        if let Some(model) = &opts.model {
            builder = builder.model(model.clone());
        }
        if let Some(max_steps) = opts.max_steps {
            builder = builder.max_steps(max_steps);
        }
        builder.build()
    }

    async fn spawn_one(&self, opts: SpawnOpts) -> Result<AgentRunResult, MachiError> {
        let agent_id = AgentId::generate();
        let label = opts.label.clone();
        let span = info_span!(
            "machi.spawn",
            machi.agent_id = %agent_id,
            machi.agent_label = label.as_deref().unwrap_or(""),
            machi.capability = ?opts.capability_mode,
        );

        async move {
            if opts.cancel.is_cancelled() {
                return Err(MachiError::new(
                    ErrorCode::HostCancelled,
                    "spawn cancelled before start",
                ));
            }
            self.reserve_slot()?;

            let started = Instant::now();
            let agent = self.build_child(&opts)?;
            let mut state = VecConversationState::new();
            let turn_opts = TurnOptions {
                max_steps: opts.max_steps,
                capability_mode: opts.capability_mode,
                cancel: opts.cancel.clone(),
                agent_id: Some(agent_id.clone()),
                ..TurnOptions::default()
            };
            let outcome = self
                .runtime
                .run(
                    &agent,
                    self.sampler.as_ref(),
                    &mut state,
                    TurnInput::Text(opts.prompt),
                    turn_opts,
                )
                .await
                .map_err(|e| {
                    if matches!(
                        e.code(),
                        ErrorCode::RuntimeCancelled | ErrorCode::LlmCancelled
                    ) {
                        MachiError::new(ErrorCode::HostCancelled, e.message().to_owned())
                            .with_source(e)
                    } else {
                        MachiError::new(ErrorCode::HostSpawn, e.message().to_owned()).with_source(e)
                    }
                })?;

            let output = outcome
                .output_json
                .unwrap_or_else(|| Value::String(outcome.output_text.clone()));
            Ok(AgentRunResult {
                agent_id,
                label,
                success: !outcome.cancelled,
                output,
                cancelled: outcome.cancelled,
                usage: outcome.usage,
                duration_ms: u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
                steps: outcome.steps,
            })
        }
        .instrument(span)
        .await
    }
}

#[async_trait]
impl SessionHost for InProcessHost {
    async fn spawn_agent(&self, opts: SpawnOpts) -> Result<AgentRunResult, MachiError> {
        self.spawn_one(opts).await
    }

    async fn spawn_agents(&self, opts: Vec<SpawnOpts>) -> Result<Vec<AgentRunResult>, MachiError> {
        try_join_all(opts.into_iter().map(|o| self.spawn_one(o))).await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use machi_llm::MockSampler;
    use machi_types::ErrorCode;

    use super::*;

    #[tokio::test]
    async fn concurrent_two_workers() {
        let sampler = Arc::new(MockSampler::new());
        // Keyed by user prompt so concurrent spawns cannot race FIFO order.
        sampler.map_user_text("task a", "worker-a-result");
        sampler.map_user_text("task b", "worker-b-result");
        let host = InProcessHost::new(sampler, vec![]);
        let results = host
            .spawn_agents(vec![
                SpawnOpts::new("task a").with_label("alpha"),
                SpawnOpts::new("task b").with_label("beta"),
            ])
            .await
            .expect("spawn");
        assert_eq!(results.len(), 2);
        assert_eq!(
            results.first().and_then(|r| r.label.as_deref()),
            Some("alpha")
        );
        assert_eq!(
            results.get(1).and_then(|r| r.label.as_deref()),
            Some("beta")
        );
        assert_eq!(
            results.first().map(|r| &r.output),
            Some(&Value::String("worker-a-result".into()))
        );
        assert_eq!(
            results.get(1).map(|r| &r.output),
            Some(&Value::String("worker-b-result".into()))
        );
        assert_eq!(host.agents_spent(), 2);
    }

    #[tokio::test]
    async fn budget_exhausted() {
        let sampler = Arc::new(MockSampler::new());
        sampler.push_text("only-one");
        let host = InProcessHost::new(sampler, vec![]).with_agent_budget(1);
        host.spawn_agent(SpawnOpts::new("first"))
            .await
            .expect("first");
        let err = host
            .spawn_agent(SpawnOpts::new("second"))
            .await
            .expect_err("budget");
        assert_eq!(err.code(), ErrorCode::HostBudget);
    }

    #[tokio::test]
    async fn cancel_before_start() {
        let sampler = Arc::new(MockSampler::new());
        let host = InProcessHost::new(sampler, vec![]);
        let cancel = CancellationToken::new();
        cancel.cancel();
        let err = host
            .spawn_agent(SpawnOpts::new("x").with_cancel(cancel))
            .await
            .expect_err("cancel");
        assert_eq!(err.code(), ErrorCode::HostCancelled);
    }
}
