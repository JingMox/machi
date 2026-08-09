<!-- markdownlint-disable MD033 MD041 MD036 -->

# Machi

[![Crates.io](https://img.shields.io/crates/v/machi.svg)](https://crates.io/crates/machi)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](#license)

**Embeddable multi-agent runtime kernel** for Rust (v1 clean break).

**Not** API-compatible with Machi ≤0.8. **Not** a full Grok product clone
(no TUI/shell); targets the agent/runtime/workflow kernel surface.

**Development north star:** [`ROADMAP.md`](./ROADMAP.md) — dual multi-agent
modes (dynamic spawn + journaled workflow), phase gates P0–P7, Ultimate DoD.
Do not expand peripherals ahead of the current phase exit criteria.

### Architecture (shipped crates)

```
types → protocol / obs
     → tools → toolkit (feature)
     → llm / agent / state / compaction
     → workflow (no LLM)
     → runtime (turn, session, host, workflow adapter)
     → machi facade
```

**Canonical vertical slice:** Session/handle → TurnRuntime → tools(+toolkit) →
approval/gates/compaction → metrics → spawn and/or journaled workflow
(scratch/template). Optional: `git_diff_since` (set git cwd), HTTP providers.

**Not shipped:** hooks crate, long-term memory crate, derive macros, OTEL SDK,
MCP. Do not treat README feature lists as production-complete systems.

**State model:** `VecConversationState` = turn buffer; `ChatStateHandle` =
session source of truth; checkpoints via `FilePersistence::for_session` /
`Session::open_checkpointed_turn`. Workflow metadata: `WorkflowRunStore`.
Optional summaries: `MemoryPort` (not a vector DB).

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

1. **Programmatic:** `InProcessHost::spawn_agents` (concurrent, budget/cancel,
   depth, concurrency caps).
2. **Model-driven:** parent `ReAct` loop calls the `spawn_agent` tool
   ([`SpawnAgentTool`]), which nests children through the same host
   (depth via `EXTRA_SPAWN_DEPTH`).

`InProcessHost` defaults: `max_spawn_depth=16`, `max_concurrent_children=64`.
`SpawnOpts` is isomorphic with workflow `AgentOpts` (including `agent_type`,
`output_schema`). Resolve types via `AgentRegistry`; optional
`ProjectPromptAssembler` (project `AGENTS.md`). `fork_messages` seeds a child
conversation; bare `fork_context` without messages / `resume_from` fail-closed.

```bash
cargo run -p machi --example delegate_multi
cargo run -p machi --example delegate_via_tool
cargo run -p machi --example session_checkpoint --features state
```

### Multi-agent mode B — journaled workflow

Rhai script `agent` / concurrent `parallel` / `complete` → host channel →
nested runs. Resume replays the journal (no re-sample).

```bash
cargo run -p machi --example workflow_fanout
cargo run -p machi --example workflow_resume
```

`machi-workflow` has no dependency on LLM HTTP clients.

### Providers / toolkit (feature-gated)

| Feature | Contents |
|---------|----------|
| (default) | `MockSampler` offline + runtime + workflow |
| `openai` | `OpenAiCompatSampler` — Chat Completions HTTP |
| `ollama` | `OllamaSampler` — `/api/chat` HTTP |
| `toolkit` | cwd-jailed `ReadFile` / `WriteFile` / `Grep` / `Shell` |
| `state` | `ChatStateHandle` actor + usage ledger + persistence ports |
| `compaction` | `MaxMessages` strategy crate (also used by runtime) |
| `obs` | `machi-obs` metrics catalogue + redaction helpers |
| `full` | runtime + workflow + toolkit + state + compaction + obs + providers |

Wire helpers `build_chat_completions_body` / `parse_chat_completions_response`
are always available for testing without enabling HTTP.

Built-in demo tool: `CalcTool` (arithmetic). Toolkit: `default_toolkit(jail)`.

Agents: load Markdown definitions via `parse_definition_markdown` /
`discover_project` (`.machi/agents/*.md`). Session: `run_turn_on_handle` syncs
to `ChatStateHandle`. LLM: `LlmSampler::sample_stream` (+ mock default).

Workflow core: `validate_script`, `write_scratch_file` / `read_scratch_file` /
`json_encode` / `budget` / `render_template` (journaled). Optional:
`git_diff_since` after `WorkflowSideEffects::set_git_cwd`. Metrics: inject
`SharedMetrics`; `RecordingMetrics` / `PrometheusRecorder` for capture/export.

### Vertical slice demos (offline)

```bash
# Toolkit write through TurnRuntime (mock)
cargo run -p machi --example repo_task --features "toolkit,obs"
# Plan → parallel → scratch report (mock)
cargo run -p machi --example workflow_plan --features "workflow,obs"
# Session file checkpoint resume
cargo run -p machi --example session_resume --features state
# Live Ollama workflow (optional)
cargo run -p machi --example workflow_ollama --features ollama
```

### Isolation & tool sources

- Host: `InProcessHost::with_isolation(Arc<dyn IsolationBackend>)` (default
  `InProcessIsolation` — same process/FS; product worktrees inject their own backend).
- Tools: `ToolSource` / `StaticToolSource` / `merge_tool_sources` (last source wins
  on name). MCP adapters implement the trait outside the kernel.

### Quality

```bash
cargo test --workspace --all-features
cargo +nightly clippy --workspace \
  --fix \
  --all-targets \
  --all-features \
  --allow-dirty \
  --allow-staged \
  -- -D warnings
cargo deny check
cargo bench -p machi-runtime --bench turn_spawn
cargo bench -p machi-workflow --bench journal
```

See [`CHANGELOG.md`](./CHANGELOG.md), [`SECURITY.md`](./SECURITY.md),
[`ROADMAP.md`](./ROADMAP.md) (Phase 7 freeze policy).

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
