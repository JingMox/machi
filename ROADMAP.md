# Machi Master Roadmap

**Status:** living north star v2 — supersedes and replaces v1 entirely.
**Product:** embeddable Rust **multi-agent runtime library** (`machi` workspace), not a TUI / shell product.
**Reference:** `3rdparty/grok-build` — extract **contracts and semantics**, never bulk-copy product code.
**Baseline:** ~14k LOC kernel, 149 test attrs, dual-mode demo paths green, `machi-auto` removed.
**Updated:** 2026-08-09.

---

## 0. Charter (one sentence)

Build a **complete, embeddable, offline-testable multi-agent runtime library** that first-class supports **two multi-agent delegation models**, shares one turn/tools/host/obs/state kernel, and reaches production-grade correctness and thickness on that vertical slice before growing any peripheral surface.

| Is | Is not |
| ---- | -------- |
| Reusable lib (ports & adapters) | Grok TUI / pager / ACP product shell |
| Dual multi-agent (dynamic spawn + journaled workflow) | Feature-name checklist clone of Grok crates |
| Fail-closed budget / cancel / capability / journal | Silent degrade / hollow public APIs |
| Vertical slice depth first | Peripheral pile (MCP market, memory product, hooks early) |

### Dual-mode core (fixed)

```text
                    ┌──────────────────────────────────────┐
                    │         Shared kernel                 │
                    │  types · tools · turn · state · obs   │
                    │  SessionHost · budget · cancel        │
                    └───────────────┬──────────────────────┘
                                    │
              ┌─────────────────────┼─────────────────────┐
              ▼                                           ▼
   Mode A — Dynamic Delegation              Mode B — Journaled Workflow
   spawn_agents / SpawnAgentTool             Rhai agent()/parallel()
   runtime decides who & how parallel        deterministic script + host RPC
   child turns nest inside parent ReAct      engine never samples an LLM
```

**Hard invariants (permanent):**

1. A and B share **one** `SessionHost` execution path for nested agents.
2. `machi-workflow` **never** depends on `machi-llm` / HTTP (firewall test permanent).
3. Budget, cancel, capability, usage, metrics are **isomorphic** across A and B.
4. **Budget conservation law:** any resumable termination (budget exhausted, cancelled) must release reserved slots and journal nothing for the interrupted panel — resume never double-charges.
5. Zero panics on production paths; `unsafe_code` denied workspace-wide.

---

## 1. Honest baseline — v1 reset

v1 marked P1–P6 ✅. A source-level audit against `3rdparty/grok-build` (2026-08-09) found
**A-class correctness holes inside items previously declared done**. v2 resets the ledger:

| v1 claim | Verified reality |
| ---------- | ------------------ |
| P1.4 durable journal ✅ | `MAX_JOURNAL_BYTES` declared but never enforced; no torn-write repair; no `truncate_tail` / prune; no symlink rejection; hash not canonicalized |
| P1.3 budget isomorphism ✅ | Reserved agent slots are **never released** on BudgetExceeded/Cancelled → resume double-charges |
| P1.5 validate_script ✅ | Meta validation lacks length / kebab-case / phase-uniqueness checks; Rhai engine lacks expr-depth and string/array/map size limits |
| U3 turn thickness | No repeated-tool-call (stationarity) protection, no preflight context-overflow check, no mid-turn steering |
| U6 obs / llm | Sampler layer has no retry, no idle timeout, no breaker; usage lacks cache/reasoning tokens |

**Gap classes:**

- **A — correctness:** resume/journal/budget semantics that are wrong today (W1).
- **B — production thickness:** main-path resilience and completeness (W2–W5).
- **C — ecosystem ports:** hooks/memory/MCP/isolation adapters — port shape only, after W5.

---

## 2. Architecture (unchanged layer map)

```text
L0  Pure values          machi-types, machi-protocol
L1  Observability ports  machi-obs
L2  Tool contract        machi-tools (+ stream, dispatch, approval, capability)
L3  Model port           machi-llm (Mock + OpenAI-compat + Ollama features)
L4  Agent definition     machi-agent (Definition ≠ Instance ≠ Builder)
L5  State / compaction   machi-state, machi-compaction
L6a Turn + SessionHost   machi-runtime (TurnRuntime, Session, Host, SpawnAgentTool)
L6b Workflow engine      machi-workflow (engine, journal, validate)  ── no LLM
L6c Host adapter         runtime::workflow_host + side_effects
L7  Facade + examples    machi, examples, dual_modes tests
── later (C-class) ──
L8  Hooks / memory / MCP / worktree-isolation adapters (ports first)
```

**Dependency firewall (CI-enforced, permanent):**

```text
machi-workflow ↛ machi-llm / HTTP
machi-types    ↛ HTTP / tokio business logic
machi-compaction --feature llm-compaction → machi-llm   (runtime side only;
                                             workflow never routes through it)
```

---

## 3. Contract extraction from grok-build

### 3.1 Goes into the kernel

1. **Journal full semantics** — canonical hash, host-error sentinel, torn-write repair, prune, size caps, `await_user` (xai-workflow `journal.rs`, `engine.rs`).
2. **Budget conservation** — release-on-resumable-termination, unjournaled interrupted panels (xai-workflow `engine.rs:480-655`).
3. **Sampler resilience** — retry classification table, exponential backoff + jitter, separate 429 budget honoring `Retry-After`, per-chunk idle timeout, circuit breaker — all as **decorators/ports**, providers stay dumb (xai-grok-sampler `retry.rs`, xai-circuit-breaker).
4. **Incremental tool streaming** — delta/truncated/gap frames, UTF-8-safe slicing, frame caps (xai-tool-runtime `streaming.rs`).
5. **Compaction invariant** — a split point never lands inside an `[assistant+tool_calls, tool…]` run (xai-grok-compaction `select.rs`).
6. **Complete usage ledger** — cache/reasoning tokens, per-model breakdown, per-prompt vs per-session (xai-chat-state `usage.rs`).
7. **Loop protection** — identical-tool-call stationarity (nudge → hard stop), preflight overflow (grok-shell `turn.rs`, xai-token-estimation).
8. **Lifecycle port** — `TurnLifecycleContributor` trait; the kernel-side mount point for future hooks (xai-agent-lifecycle).
9. **Multi-level agent discovery** — project → user → builtin with shadowing rules and builtin agent types (xai-grok-agent `discovery.rs`, `config.rs`).

### 3.2 Permanently excluded

TUI / pager / ACP shell, auth credential refresh, terminal backends, MCP marketplace,
memory product, mermaid/voice, shell-owned process trees, bulk source paste.
Hooks / memory / MCP remain **ports only** until W6 exits.

### 3.3 Reading map (open while implementing)

| Theme | Path under `3rdparty/grok-build/` |
| ------- | ----------------------------------- |
| Journal hardening, sentinel, await_user | `crates/codegen/xai-workflow/src/{engine,journal}.rs` |
| Budget release on cancel/budget tests | `xai-workflow/src/engine.rs` (tests ~1240–1472) |
| Meta / validate limits | `xai-workflow/src/{meta,validate}.rs` |
| Retry policy / backoff / idle timeout | `crates/codegen/xai-grok-sampler/src/{retry,client,request_task}.rs` |
| Circuit breaker | `crates/common/xai-circuit-breaker/src/` |
| Streaming deltas / frame caps | `crates/common/xai-tool-runtime/src/streaming.rs` |
| Tool error taxonomy | `crates/common/xai-tool-runtime/src/error.rs` |
| Stationarity / preflight overflow | `xai-grok-shell/src/session/acp_session_impl/turn.rs` |
| Token estimation constants | `crates/codegen/xai-token-estimation/src/lib.rs` |
| Compaction split selection | `crates/common/xai-grok-compaction/src/select.rs` |
| Usage ledger shape | `crates/codegen/xai-chat-state/src/usage.rs` |
| Agent discovery / builtins | `crates/codegen/xai-grok-agent/src/{discovery,config}.rs` |
| Lifecycle contributor trait | `crates/codegen/xai-agent-lifecycle/src/lib.rs` |

---

## 4. Phase plan W1–W6 (only path to 1.0)

> **Rule:** do not open the next phase's new surface until the current phase exit
> criteria pass. Deepening within a phase is always allowed. Every phase exit leaves
> a working product.

### W1 — Workflow/Journal correctness closure (A-class, highest priority) ✅ (2026-08-09)

Reopens v1 "P1 done" items. All changes are breaking; journal format v2 ships
**without migration** (decision log #1).

**Journal (`crates/machi-workflow/src/journal.rs`):**

| # | Work | Grok contract |
| --- | ------ | --------------- |
| 1.1 | `canonical_json` (recursive key sort) before hashing; truncate digest to 16 bytes hex | `journal.rs:332-362` |
| 1.2 | Enforce `MAX_JOURNAL_BYTES` on load **and** append (size check before read + TOCTOU re-check) | `journal.rs:255-294` |
| 1.3 | Torn-write repair: unterminated last line → valid JSON gets newline, invalid JSON truncates to previous line | `journal.rs:81-115` |
| 1.4 | `truncate_tail` + `prune_trailing_host_error` (recoverable host-error entries prunable for resume) | `journal.rs:227-252, 308-312` |
| 1.5 | Reject symlinked journal paths (Unix `O_NOFOLLOW`); version header line (format evolution anchor) | `journal.rs` |

**Engine (`crates/machi-workflow/src/engine.rs`):**

| # | Work | Semantics |
| --- | ------ | ----------- |
| 1.6 | **Budget release:** `parallel`/`agent` hitting BudgetExceeded/Cancelled → release reserved live slots, journal nothing for the panel; resume re-executes it | conservation law §0 |
| 1.7 | **Host-error sentinel:** `Failed`/`Unsupported` journaled as `{"__machi_host_error": msg}`; replay re-raises the same catchable error | `engine.rs:276-317` |
| 1.8 | `await_user(kind, msg)`: the only journaled pause — records then pauses first run, skipped on resume | `engine.rs:723-741` |
| 1.9 | `fingerprint(text)` pure fn; `print`/`debug` hooks map to `log`; rename `telemetry` → `telemetry_event` with `Map` arg | |
| 1.10 | `AgentCallQuotaExceeded` inside `parallel` returns catchable `null` instead of terminating | |
| 1.11 | Rhai limits: `set_max_expr_depths(128, 64)`, string 16 MiB, array/map 64 K | `engine.rs:119-127` |

**Validate / meta (`meta.rs`, `validate.rs`):**

- name ≤64 bytes kebab-case; description ≤1024; optional `when_to_use` ≤2048;
  phases ≤64 with unique titles ≤128; first statement must be `let meta = #{…}`.

**Exit criteria:** port the semantics of grok's engine test list (not the code):
budget-release-then-resume-no-double-charge ×4, sentinel replay, torn-write repair,
divergence, oversized fanout, await_user resume, host-call cap. `dual_modes` grows
matching A+B cases.

### W2 — LLM supply layer production hardening (`machi-llm`)

Decorator stack; the `LlmSampler` trait itself does not change shape:

```text
RetryingSampler(policy)          ← retry / backoff / jitter / Retry-After
  └─ BreakerSampler(breaker)     ← circuit breaker (feature-gated)
       └─ OpenAiCompatSampler / OllamaSampler / MockSampler
```

| # | Work | Contract |
| --- | ------ | ---------- |
| 2.1 | `RetryPolicy`: retry 429 + 5xx (except 525/526); 400/401/403/404/422 fatal; 429 separate budget (2) waiting full `Retry-After` capped 120 s; `x-should-retry` server hint wins; exp backoff 2 s → cap 30 s ± 20 % jitter; EmptyResponse retried | `retry.rs` |
| 2.2 | Per-chunk **idle timeout** (default 300 s) independent of deadline | `request_task.rs` |
| 2.3 | `SampleEvent` growth: `ReasoningDelta` (channel split), `ToolCallDelta` (incremental args), `ResponseStarted{cache_read, cache_creation}`, `Retrying{attempt, reason}` | `events.rs` |
| 2.4 | `Usage` growth (machi-types, breaking): `cache_read_tokens`, `cache_creation_tokens`, `reasoning_tokens`, `api_duration_ms` | `usage.rs` |
| 2.5 | Circuit breaker module: windowed error-rate state machine (Closed→Open→HalfOpen), `min_samples` / `error_rate_threshold` / `half_open_max_probes`, keyed per endpoint | xai-circuit-breaker |
| 2.6 | `ErrorCode` growth: `LlmRateLimited`, `LlmIdleTimeout`, `LlmEmptyResponse`, `LlmTruncated`; `RetryClass` aligned with 2.1 table | |

**Exit criteria:** retry classification table fully covered by tests against a scripted
mock transport (no network); breaker state-machine table tests; firewall still green.

### W3 — Turn & tools main-path thickness (`machi-runtime`, `machi-tools`)

| # | Work | Design |
| --- | ------ | -------- |
| 3.1 | **Stationarity gate:** track consecutive identical `(tool, args_hash)`; nudge reminder at 8, hard stop at 16 with dedicated `ErrorCode`; built-in `GateChain` member | grok `turn.rs:2724-2785` |
| 3.2 | **Preflight overflow:** token estimation in `machi-protocol` (bytes/4, image 765); before sample, `estimate > window × threshold` triggers compaction or typed error | xai-token-estimation |
| 3.3 | **Incremental tool streaming:** `ToolProgress::Partial{delta, total_bytes, truncated, gap}`; UTF-8-boundary-safe slicing; `max_delta_bytes` 16 KiB / `max_frame_bytes` 16 MiB | `streaming.rs:30-156` |
| 3.4 | Tool error taxonomy growth: `RateLimited`, `ConcurrencyLimit`, `NetworkError`, `ServiceUnavailable`; structured `details` (model-visible) + `source` chain (model-invisible) | `error.rs:17-124` |
| 3.5 | **Lifecycle port:** `TurnLifecycleContributor { on_turn_start/done/abort/error }` default-empty trait; `TurnOptions.contributors`; abort carries reason classification | xai-agent-lifecycle |
| 3.6 | **Interjection:** `TurnOptions.interject_rx: Option<mpsc::Receiver<Message>>`, drained before each sample — minimal steering kernel | |
| 3.7 | Per-tool numeric `max_concurrency: Option<usize>` metadata alongside the mode enum | |

**Exit criteria:** stationarity matrix (nudge/hard-stop/reset-on-different-call),
streaming delta property tests (UTF-8 fuzz, caps), lifecycle contributor called on all
four paths, interjection ordering test.

### W4 — State & compaction production hardening (`machi-state`, `machi-compaction`)

| # | Work | Design |
| --- | ------ | -------- |
| 4.1 | **Tool-pair invariant:** shared `select_compaction_range` snapping split points past tool-result runs; used by every strategy | `select.rs:60-193` |
| 4.2 | `SummarizingCompaction` via `LlmSampler` (feature `llm-compaction`); plus tool-result-pruning and image-stripping light strategies | intra/inter compaction |
| 4.3 | `ChatStateHandle` growth: `prompt_index` (turn boundaries), per-prompt + per-session ledgers, `record_compaction_at`, per-model usage breakdown | `state.rs:115-176` |
| 4.4 | **Incremental persistence port:** `ChatPersistence::persist_message` (append-style); `FilePersistence` JSONL events + periodic snapshot, reusing W1 torn-write repair | `persistence.rs` |

**Exit criteria:** compaction property test — random conversations fuzzed for split
points, invariant never violated; restart recovery from JSONL event log; ledger
breakdown assertions in dual_modes.

### W5 — Agent resolution & host completeness (`machi-agent`, `machi-runtime`)

| # | Work | Design |
| --- | ------ | -------- |
| 5.1 | Multi-level discovery: `.machi/agents/` (cwd walking to repo root) → `~/.machi/agents/` → builtin; **shadowing:** project may override builtin; user-level names colliding with builtin are skipped (visible == callable); per-agent toggle | `discovery.rs:122-200` |
| 5.2 | Builtin agent types: `general-purpose` / `explore` (read-only) / `plan` + orchestrator delegation prompt | `config.rs:97-142` |
| 5.3 | Host completion: `fork_context` sources from `ChatStateHandle` snapshot; `resume_from` effective via `WorkflowRunStore`; `SpawnAgentTool` allowed-`agent_types` allowlist; child spans link parent trace | |
| 5.4 | Definition-level `allowed_tools` filtering at resolution time (not only runtime capability) | |

**Exit criteria:** discovery precedence table tests, builtin types spawnable through
both modes, fork/resume round-trip in dual_modes.

### W6 — Quality closure → 1.0 freeze

- Tests 149 → **≥500 meaningful** cases, grown per contract: W1 journal/budget matrix,
  W2 retry table, W3 stream/stationarity matrix, W4 compaction fuzz.
- Stress: 64 concurrent spawns × depth 8; 64 MiB journal boundary; torn-write fuzz.
- Metric/span name snapshot CI; `cargo deny`; bench baselines
  (`machi-runtime/turn_spawn`, `machi-workflow/journal`).
- Docs = code; maturity tags on all public API; delete hollow API instead of documenting it.
- Freeze tag only after every phase exit re-verified honest green.

---

## 5. Quality gates (every phase exit)

```bash
cargo test --workspace --all-features
cargo +nightly clippy --workspace --fix --all-targets --all-features \
  --allow-dirty --allow-staged -- -D warnings
cargo fmt --all -- --check
cargo deny check
# firewall: workflow ↛ llm ; types ↛ HTTP
```

### PR discipline

1. One work package per PR when possible.
2. New public API requires: production call site + test + maturity tag.
3. "Grok has X" is never sufficient; ask "does the dual-mode vertical slice lack X?"
4. Prefer deleting hollow API over documenting it. No BC shims (AGENTS.md).
5. Report progress only as **phase exit status**, never symbol counts or LOC.

### API maturity tags (rustdoc)

```rust
/// Maturity: core
/// Maturity: optional (feature = "…")
/// Maturity: experimental — may break without major bump until freeze
```

---

## 6. Feature matrix (stable)

| Feature | Contents |
| --------- | ---------- |
| default | runtime + workflow + mock |
| `openai` / `ollama` | HTTP samplers |
| `toolkit` | cwd-jailed fs/shell tools |
| `state` / `compaction` / `obs` | respective crates |
| `llm-compaction` | `SummarizingCompaction` (W4; runtime side only) |
| `full` | all of the above |

Semver: workspace stays `1.0.0` during consolidation; **freeze is not active** until
W6 exit is recorded here and in `CHANGELOG.md`. Until then breaking changes land
without major bumps and without compatibility layers.

---

## 7. Decision log

| # | Date | Decision |
| --- | ------ | ---------- |
| 1 | 2026-08-09 | Journal format v2 (canonical 16-byte hash + version header) ships **without migration**; old journals are invalid. |
| 2 | 2026-08-09 | `machi-auto` demo CLI removed from the workspace; examples under `crates/machi/examples` are the only demo surface. |
| 3 | 2026-08-09 | ROADMAP v2 replaces v1; v1's P1/P2 "done" status is void where §1 lists holes. W1–W6 is the sole execution queue. |
| 4 | 2026-08-09 | Sampler resilience implemented as decorators (`RetryingSampler`/`BreakerSampler`); provider adapters stay transport-only. |
| 5 | 2026-08-09 | Hooks ship later **on top of** `TurnLifecycleContributor` (W3.5); no separate hooks crate before W6 exit. |
| 6 | 2026-08-09 | W1 exit green: journal v2, budget conservation, host-error sentinel, await_user, meta/Rhai limits. Next: W2. |

**Authority:** this file > ad-hoc chat memory > README feature enthusiasm.
**Local drafts:** `docs/` is gitignored; commit architecture truth in `README.md` + this file + rustdoc.
