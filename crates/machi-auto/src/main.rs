//! Thin demo CLI: mock turn + dynamic multi-agent spawn.
#![allow(
    clippy::print_stdout,
    clippy::expect_used,
    unused_crate_dependencies,
    reason = "demo binary prints results and uses expect for setup"
)]

use std::sync::Arc;

use machi::{
    AgentBuilder, InProcessHost, MockSampler, SessionHost, SpawnOpts, TurnInput, TurnOptions,
    TurnRuntime, VecConversationState,
};

#[tokio::main]
async fn main() {
    // Single-agent turn
    let sampler = Arc::new(MockSampler::new());
    sampler.push_text("machi kernel ok");
    let agent = AgentBuilder::named("demo")
        .instructions("demo")
        .model("mock")
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
    println!("turn: {}", out.output_text);

    // Dynamic multi-agent
    let multi = Arc::new(MockSampler::new());
    multi.map_user_text("task-a", "A");
    multi.map_user_text("task-b", "B");
    let host = InProcessHost::new(multi, vec![]).with_agent_budget(4);
    let results = host
        .spawn_agents(vec![
            SpawnOpts::new("task-a").with_label("a"),
            SpawnOpts::new("task-b").with_label("b"),
        ])
        .await
        .expect("spawn");
    println!(
        "spawn: {} workers spent={}",
        results.len(),
        host.agents_spent()
    );
}
