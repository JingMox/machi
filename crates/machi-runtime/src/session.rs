//! Multi-turn session helper over [`TurnRuntime`].

use std::time::Instant;

use machi_agent::Agent;
use machi_llm::LlmSampler;
use machi_types::MachiError;

use crate::metrics::{MetricsSink, NoopMetrics, record_turn};
use crate::state::ConversationState;
use crate::turn::{TurnInput, TurnOptions, TurnOutcome, TurnRuntime};

/// Thin multi-turn orchestrator (one agent, persistent conversation state).
#[derive(Debug, Clone, Copy)]
pub struct Session {
    runtime: TurnRuntime,
    turn_count: u64,
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

impl Session {
    /// Create a session.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            runtime: TurnRuntime::new(),
            turn_count: 0,
        }
    }

    /// Number of turns completed successfully (or cancelled) through this session.
    #[must_use]
    pub const fn turn_count(self) -> u64 {
        self.turn_count
    }

    /// Run one user turn, appending to `state`.
    ///
    /// # Errors
    ///
    /// Propagates [`TurnRuntime::run`] failures.
    pub async fn run_turn(
        &mut self,
        agent: &Agent,
        sampler: &dyn LlmSampler,
        state: &mut dyn ConversationState,
        input: TurnInput,
        options: TurnOptions,
    ) -> Result<TurnOutcome, MachiError> {
        self.run_turn_with_metrics(agent, sampler, state, input, options, &NoopMetrics)
            .await
    }

    /// Like [`Self::run_turn`] with metrics.
    ///
    /// # Errors
    ///
    /// Propagates turn failures.
    pub async fn run_turn_with_metrics(
        &mut self,
        agent: &Agent,
        sampler: &dyn LlmSampler,
        state: &mut dyn ConversationState,
        input: TurnInput,
        options: TurnOptions,
        metrics: &dyn MetricsSink,
    ) -> Result<TurnOutcome, MachiError> {
        let started = Instant::now();
        let outcome = self
            .runtime
            .run(agent, sampler, state, input, options)
            .await;
        let ms = started.elapsed().as_secs_f64() * 1000.0;
        match &outcome {
            Ok(o) => {
                self.turn_count = self.turn_count.saturating_add(1);
                let status = if o.cancelled { "cancelled" } else { "ok" };
                let steps = u64::try_from(o.steps).unwrap_or(u64::MAX);
                record_turn(metrics, status, steps, ms);
            }
            Err(_) => {
                record_turn(metrics, "error", 0, ms);
            }
        }
        outcome
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use machi_agent::AgentBuilder;
    use machi_llm::MockSampler;

    use super::*;
    use crate::state::VecConversationState;

    #[tokio::test]
    async fn multi_turn() {
        let sampler = Arc::new(MockSampler::new());
        sampler.push_text("one");
        sampler.push_text("two");
        let agent = AgentBuilder::named("a")
            .model("mock")
            .build()
            .expect("agent");
        let mut state = VecConversationState::new();
        let mut session = Session::new();
        let o1 = session
            .run_turn(
                &agent,
                sampler.as_ref(),
                &mut state,
                TurnInput::Text("hi".into()),
                TurnOptions::default(),
            )
            .await
            .expect("t1");
        let o2 = session
            .run_turn(
                &agent,
                sampler.as_ref(),
                &mut state,
                TurnInput::Text("again".into()),
                TurnOptions::default(),
            )
            .await
            .expect("t2");
        assert_eq!(o1.output_text, "one");
        assert_eq!(o2.output_text, "two");
        assert_eq!(session.turn_count(), 2);
        assert!(state.messages().len() >= 4);
    }
}
