# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
with the freeze policy in [`ROADMAP.md`](./ROADMAP.md) § 6.

## [Unreleased]

### Added

- **W3 turn/tools thickness:** stationarity gate (nudge@8 / hard-stop@16);
  preflight token estimate + overflow check; `ToolProgress::Partial` + UTF-8
  framing helpers; tool error codes (rate/concurrency/network/unavailable);
  `TurnLifecycleContributor` + `interject_rx`; `ToolMetadata::max_concurrency`.
- **W2 LLM supply layer:** `RetryPolicy` / `RetryingSampler` (429 budget,
  backoff+jitter, empty-response retry, stream idle timeout); `CircuitBreaker`
  / `BreakerSampler`; `SampleEvent` variants (`ReasoningDelta`, `ToolCallDelta`,
  `ResponseStarted`, `Retrying`); `Usage` cache/reasoning/api_duration fields;
  `ErrorCode::{LlmRateLimited,LlmIdleTimeout,LlmEmptyResponse,LlmTruncated}`.

### Changed

- **ROADMAP v2:** full rewrite around grok-build contract audit; W1–W6 phase
  plan replaces P0–P7; v1 "P1/P2 done" status voided where correctness holes
  were found (journal hardening, budget release on resumable termination).
- **W1 Journal format v2 (breaking, no migration):** version header
  `# machi-journal/2`, canonical key-sorted 16-byte request hashes,
  `MAX_JOURNAL_BYTES` on load/append, torn-write repair, symlink rejection
  (Unix `O_NOFOLLOW`), `prune_trailing_host_error`.
- **W1 Budget conservation:** `agent`/`parallel` on Cancelled/BudgetExceeded
  release reserved slots and journal nothing for the interrupted panel.
- **W1 Host-error sentinel:** `Failed`/`Unsupported` journaled as
  `{"__machi_host_error": msg}` and re-raised on replay.
- **W1 Rhai surface:** `await_user`, `fingerprint`, `print`/`debug` → log,
  `telemetry_event` (Map); Rhai expr/string/array/map limits; meta kebab-case
  and length validation; first statement must be `let meta = #{…}`.
- **W1 review fixes:** `parallel` Quota journals dense `null` + releases
  unused reservations; journal `line_starts` stack for repeated prune;
  torn-tail repair never exceeds `MAX_JOURNAL_BYTES`; dual_modes W1 cases
  (`await_user` resume, budget non-journal + resume).

### Removed

- `machi-auto` demo CLI crate; `crates/machi/examples` is the only demo surface.
- Rhai `telemetry(name, fields)` — use `telemetry_event(name, #{…})`.

### Added

- **N1 vertical slice:** Rhai `json_encode`, `write_scratch_file` / `read_scratch_file`,
  `budget`, bare `complete()`; examples `repo_task`, `workflow_plan`, `session_resume`,
  `workflow_ollama`.
- **Phase 6 ports:** `IsolationBackend` + `InProcessIsolation` (host `with_isolation`);
  `ToolSource` + `StaticToolSource` + `merge_tool_sources` / `merge_arc_sources`.
- **Phase 1–5 (prior):** dual-mode host depth/concurrency, turn state machine, obs golden,
  AgentRegistry / PromptAssembler / fork_messages, session checkpoint, WorkflowRunStore,
  MemoryPort.

### Security

- `cargo deny` license allow-list configured for workspace dependency set
  (see `deny.toml`). Advisories/bans/sources checks enabled.

## [1.0.0] — pre-freeze line

Workspace crates ship as `1.0.0` during consolidation. **Public API may still
break without a major bump until Phase 7 freeze is declared** in ROADMAP and
this file records a freeze tag. Prefer maturity tags in rustdoc
(`core` / `optional` / `experimental`).

[Unreleased]: https://github.com/qntx/machi/compare/main...HEAD
