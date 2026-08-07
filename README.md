<!-- markdownlint-disable MD033 MD041 MD036 -->

# Machi

[![Crates.io](https://img.shields.io/crates/v/machi.svg)](https://crates.io/crates/machi)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](#license)

**Enterprise embeddable agent runtime kernel** for Rust (v1 clean break).

Layers: agent **definition** → **tools** → **turn runtime** → **session host**
(nested agents) → **journaled Rhai workflow** (no LLM dependency in the engine).

This line is **not** API-compatible with Machi ≤0.8 (`Runner`, etc.).

- [Kernel architecture](docs/architecture/kernel.md)
- [Production standards](docs/architecture/production.md)
- [Quality gates](docs/architecture/BASELINE.md)

### Single-agent turn

```rust
use std::sync::Arc;
use machi::{AgentBuilder, MockSampler, TurnInput, TurnOptions, TurnRuntime, VecConversationState};

# async fn demo() {
let sampler = Arc::new(MockSampler::new());
sampler.push_text("ok");
let agent = AgentBuilder::named("assistant").model("mock").build().unwrap();
let mut state = VecConversationState::new();
let out = TurnRuntime::new()
    .run(&agent, sampler.as_ref(), &mut state, TurnInput::Text("hi".into()), TurnOptions::default())
    .await
    .unwrap();
assert_eq!(out.output_text, "ok");
# }
```

### Multi-agent mode A — dynamic delegation

1. **Programmatic:** `InProcessHost::spawn_agents` (concurrent, budget/cancel).
2. **Model-driven:** parent `ReAct` loop calls the `spawn_agent` tool
   ([`SpawnAgentTool`]), which nests children through the same host.

```bash
cargo run -p machi --example delegate_multi
cargo run -p machi --example delegate_via_tool
```

### Multi-agent mode B — journaled workflow

Rhai script `agent` / concurrent `parallel` / `complete` → host channel →
nested runs. Resume replays the journal (no re-sample).

```bash
cargo run -p machi --example workflow_fanout
cargo run -p machi --example workflow_resume
```

`machi-workflow` has no dependency on LLM HTTP clients.

### Providers (feature-gated)

| Feature | Sampler |
|---------|---------|
| (default) | `MockSampler` offline |
| `openai` | `OpenAiCompatSampler` — Chat Completions HTTP |
| `ollama` | `OllamaSampler` — `/api/chat` HTTP |
| `full` | runtime + workflow + openai + ollama |

Wire helpers `build_chat_completions_body` / `parse_chat_completions_response`
are always available for testing without enabling HTTP.

Built-in non-spawn tool: `CalcTool` (arithmetic) for demos and integration tests.

### Quality

```bash
cargo test --workspace --all-features
cargo +nightly clippy --workspace --all-targets --all-features -- -D warnings
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project shall be dual-licensed as above, without any additional terms or conditions.

---

<div align="center">

A **[QuantX](https://qntx.fun)** open-source project.

<a href="https://qntx.fun"><img alt="QuantX" width="369" src="https://raw.githubusercontent.com/qntx/.github/main/profile/qntx.svg" /></a>

Code is law. We write both.

</div>
