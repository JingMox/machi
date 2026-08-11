# Machi Master Roadmap

**Status:** living north star v3 — supersedes and replaces v2 entirely.
**Product:** embeddable Rust **multi-agent runtime library** (`machi` workspace), not a TUI / CLI / server product.
**References:** `3rdparty/grok-build`, `3rdparty/codex`, `3rdparty/pi`, `3rdparty/opencode` — extract **contracts and semantics**, never bulk-copy product code.
**Baseline:** ~25.7k LOC kernel, **612 tests green** (`--workspace --all-features`, fully offline), `cargo deny` / `fmt` clean, zero `todo!`/`unimplemented!`, W1–W5 exit verified.
**Updated:** 2026-08-11.

---

## 0. Charter (one sentence)

Build a **complete, embeddable, offline-testable multi-agent runtime library** that first-class supports **two multi-agent delegation models**, exposes every runtime effect through **one observable event surface**, enforces **fail-closed resource and security boundaries**, and reaches production-grade correctness on that vertical slice before growing any peripheral surface.

| Is | Is not |
| ---- | -------- |
| Reusable lib (ports & adapters) | TUI / pager / server / CLI product shell |
| Dual multi-agent (dynamic spawn + journaled workflow) | Feature-name checklist clone of any reference repo |
| Fail-closed budget / cancel / capability / journal / sandbox | Silent degrade / hollow public APIs / trust-based security |
| Live event surface for embedders | Function-return-only black box |
| Vertical slice depth first | Peripheral pile (marketplace, voice, self-update) |

### Dual-mode core (fixed)

```text
                    ┌──────────────────────────────────────┐
                    │         Shared kernel                 │
                    │  types · tools · turn · state · obs   │
                    │  SessionHost · budget · cancel        │
                    │  TurnEvent stream (W7)                │
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
2. `machi-workflow` **never** depends on `machi-llm` / `machi-mcp` / HTTP (firewall test permanent).
3. Budget, cancel, capability, usage, metrics, **events**, and **hooks** are **isomorphic** across A and B.
4. **Budget conservation law:** any resumable termination (budget exhausted, cancelled) must release reserved slots and journal nothing for the interrupted panel — resume never double-charges.
5. Zero panics on production paths; `unsafe_code` denied workspace-wide.
6. **Event completeness law (W7+):** every runtime effect (sample delta, tool progress, spawn lifecycle, compaction, pause) must be observable through the `TurnEvent` stream; no capability may ship with a private side-channel as its only observation path.
7. **Secure-by-default law (W8+):** tools that execute processes or mutate the filesystem require an explicit policy (sandbox / exec-policy / approval); trust mode is an explicit opt-out, never the silent default.
8. **Protocol containment law:** external protocol shapes (MCP, OAuth, provider wire formats) never leak into L0–L2 types; they live in leaf adapter crates behind ports.

---

## 1. Honest baseline — v2 ledger closure and v3 audit

### 1.1 v2 phase ledger (verified 2026-08-11, 612 tests green)

| Phase | v2 claim | Verified reality |
| ------- | ---------- | ------------------ |
| W1 journal/budget | ✅ 2026-08-09 | Confirmed: journal v2 (canonical hash, torn-write repair, `O_NOFOLLOW`, prune), budget release-on-resumable-termination, sentinel replay, `await_user` — engine + dual_modes tests green |
| W2 LLM supply | ✅ 2026-08-09 | Confirmed: retry classification matrix (`http_status_matrix.rs`), breaker state machine, idle timeout, `SampleEvent`/`Usage` growth |
| W3 turn/tools | ✅ 2026-08-09 | Confirmed: stationarity nudge@8/stop@16, preflight overflow, `ToolProgress::Partial` UTF-8 framing, lifecycle port, interjection |
| W4 state/compaction | **✅ verified 2026-08-11** | `w4_ledger_and_jsonl_restart` green; compaction fuzz (`select_matrix.rs`) green; ledger per-prompt/per-model live. **Deviation (decision #7):** 4.2 shipped as `SummarizingCompaction` with a sync `summarize` callback port instead of an `llm-compaction` feature — cleaner for the firewall; no LLM dependency in `machi-compaction`. |
| W5 agent/host | **✅ verified 2026-08-11** | `w5_builtin_types_spawnable`, `w5_fork_from_parent_handle_and_resume_store` green; discovery precedence tests in `machi-agent`; `SpawnAgentTool` allowlist live |
| W6 quality closure | **⏳ open** | Test target exceeded (612 ≥ 500); stress + benches (`turn_spawn`, `journal`) exist; `cargo deny` configured. **Remaining:** metric/span snapshot wired as a dedicated CI job, rustdoc maturity audit, hollow-API sweep, freeze tag. Closed as W6F below. |

### 1.2 v3 production-readiness audit (2026-08-11)

A source-level audit against all four reference systems found the kernel correct but
the **product charter unfulfilled** in five gap classes. v3 exists to close them.

| Class | Gap | Evidence |
| ------- | ----- | ---------- |
| **E — embeddability** | No live event surface: `TurnRuntime::run` returns only a final `TurnOutcome`; `SampleEvent` / `ToolProgress` are aggregated internally and discarded; `TurnLifecycleContributor` has only 4 coarse callbacks. An embedding host cannot render a running turn. | codex ships SQ/EQ as its core architecture; pi emits 14 `AgentEvent` kinds; opencode persists every state change as a subscribable event |
| **S — security** | `resolve_jailed` is lexical-only — a symlink inside the jail escapes it (`path_util.rs:48`); `ShellTool` documents "Does not sandbox syscalls" (`shell.rs:25`); no OS sandbox, no exec policy. | codex: Seatbelt/Landlock/Restricted-Token + `ExecPolicy`; grok: nono (Landlock/Seatbelt) |
| **X — ecosystem** | No MCP (all four references have it); providers = OpenAI-compat + Ollama only; no model catalog, no cost accounting, no auth port. | codex MCP client+server; grok `xai-grok-mcp`; pi 40+ providers with cost tracking |
| **V — validation** | All 612 tests run against `MockSampler`; zero live-provider conformance fixtures; retry matrix never exercised against a real 429 storm. | pi/opencode ship record-replay + eval harnesses |
| **D — durability UX** | Session log is linear: no tree/branch/fork, no snapshot/revert, no file-change attribution. | pi session tree; opencode git snapshot revert; grok hunk-tracker |

**Gap-class priority is the phase order: E → S → X → (V threaded through) → D.**

---

## 2. Architecture (layer map v3)

```text
L0  Pure values          machi-types, machi-protocol (+ TurnEvent, W7)
L1  Observability ports  machi-obs
L2  Tool contract        machi-tools (+ stream, dispatch, approval, capability, exec-policy port W8)
L3  Model port           machi-llm (Mock + OpenAI-compat + Ollama; + Anthropic / Responses W11)
L4  Agent definition     machi-agent (Definition ≠ Instance ≠ Builder, discovery)
L5  State / compaction   machi-state (+ session tree W12), machi-compaction
L6a Turn + SessionHost   machi-runtime (TurnRuntime, Session, Host, SpawnAgentTool, hooks W10)
L6b Workflow engine      machi-workflow (engine, journal, validate)  ── no LLM / MCP / HTTP
L6c Host adapter         runtime::workflow_host + side_effects
L7  Facade + examples    machi, examples, dual_modes / stress tests
── v3 leaf adapters (ports first, adapters feature-gated) ──
L8  machi-sandbox        SandboxPolicy + SeatbeltBackend / LandlockBackend    (W8)
L8  machi-mcp            ToolSource adapter over rmcp                         (W9)
L8  snapshot / worktree  SnapshotPort adapter, WorktreeIsolation backend      (W12)
```

**Dependency firewall (CI-enforced, permanent):**

```text
machi-workflow ↛ machi-llm / machi-mcp / HTTP
machi-types    ↛ HTTP / rmcp / tokio business logic
machi-protocol ↛ provider wire formats / rmcp
machi-mcp      ↛ machi-workflow / machi-runtime   (pure ToolSource leaf)
OAuth flows    ↛ kernel (credential snapshot port only)
```

---

## 3. Contract extraction — reading maps

Rule unchanged: extract **contracts and semantics**; never bulk-copy product code.
"Reference has X" is never sufficient; ask "does the embeddable dual-mode slice lack X?"

### 3.1 grok-build (`3rdparty/grok-build/crates/`) — remaining extractions

| Theme | Path | Consumed by |
| ------- | ------ | ------------- |
| Hooks: event envelope, gate kinds, matcher, fail-open, trust | `codegen/xai-grok-hooks/src/{event,config,dispatcher,discovery,matcher,runner,trust}.rs` | W10 |
| MCP: init state machine, tool-name validation, liveness, credential store | `codegen/xai-grok-mcp/src/{servers,credentials,liveness,oauth}.rs` | W9 |
| OS sandbox: profiles, network policy, deny rules | `codegen/xai-grok-sandbox/src/{profiles,network_policy,deny}.rs` | W8 |
| Fast worktree: CoW clone, sync, metadata GC | `codegen/xai-fast-worktree/src/{api,worktree,copy,sync,db}.rs` | W12 |
| Hunk attribution (deferred port) | `codegen/xai-hunk-tracker/src/{actor,events,types}.rs` | post-W12 |
| Memory: markdown storage + SQLite FTS5 index (deferred adapter) | `codegen/xai-grok-memory/src/{storage,index,chunker}.rs` | post-W12 |
| Subagent resolution precedence refinements | `codegen/xai-grok-subagent-resolution/src/{definition,overrides,resume}.rs` | W10/W12 |

### 3.2 codex (`3rdparty/codex/codex-rs/`)

| Theme | Path | Consumed by |
| ------- | ------ | ------------- |
| SQ/EQ protocol, submission loop, input queue | `core/src/session/{session.rs,handlers.rs,input_queue.rs}`, `core/src/codex_thread.rs` | W7 |
| Sandboxing: Seatbelt policy, Landlock/bwrap args | `sandboxing/src/{seatbelt.rs,landlock.rs}`, `sandboxing/src/*.sbpl` | W8 |
| Exec policy + approval requirement derivation | `core/src/exec_policy.rs`, `core/src/tools/sandboxing.rs` | W8 |
| Unified exec: persistent PTY sessions, output caps, yield-time | `core/src/unified_exec/{mod.rs,process_manager.rs}` | W8 |
| Rollout / resume / fork persistence | `rollout/`, `thread-store/`, `core/src/thread_manager.rs` | W12 |
| Responses API retry, stream reconnect | `core/src/responses_retry.rs`, `core/src/client.rs` | W11 |
| Error taxonomy breadth | `protocol/src/error.rs` | W8/W11 |
| Multi-agent v2 tools (spawn/wait/send/close) | `core/src/tools/handlers/multi_agents_v2/` | W7 event shapes |
| OTEL provider / trace context | `otel/src/{provider.rs,trace_context.rs}` | W11 obs |

### 3.3 pi (`3rdparty/pi/packages/`)

| Theme | Path | Consumed by |
| ------- | ------ | ------------- |
| Agent loop event taxonomy (14 kinds), double loop (steering/follow-up) | `agent/src/agent-loop.ts`, `agent/src/types.ts` | W7 |
| Tool-call interception (block / rewrite args / rewrite result) | `coding-agent/src/core/extensions/types.ts` (`tool_call`, `tool_result`) | W10 |
| Session tree: id/parentId entries, branch summaries, fork | `coding-agent/src/core/session-manager.ts` | W12 |
| Tool output truncation heuristics | `coding-agent/src/core/tools/truncate.ts` | W8 exec caps |
| Provider retry (retry-after-ms, jitter) | `ai/src/utils/provider-retry.ts` | W11 |
| Models registry + auth resolution + cost fields | `ai/src/models.ts` | W11 |
| Subprocess subagents (single / parallel / chain) | `coding-agent/examples/extensions/subagent/` | W12 isolation |

### 3.4 opencode (`3rdparty/opencode/packages/`)

| Theme | Path | Consumed by |
| ------- | ------ | ------------- |
| Durable event bus: publish/subscribe/replay/project | `core/src/event.ts` | W7 |
| Git snapshot: capture/diff/restore, revert stage/commit/clear | `core/src/snapshot.ts`, `core/src/session/revert.ts` | W12 |
| Compaction trigger math (context − max(output, buffer)) | `core/src/session/compaction.ts` | W11 catalog |
| Permission ruleset: action/resource/effect wildcards | `core/src/permission.ts` | W8/W10 |
| MCP config shape (local command / remote url, timeouts) | `core/src/config/mcp.ts` | W9 |
| Markdown agent definition format | agent config schema | W5 (done) / W10 |

### 3.5 Permanently excluded

TUI / pager / renderer stacks, ACP shell, voice, self-update, crash handler,
plugin marketplace install pipelines, Mixpanel/Sentry product telemetry,
Computer Hub, opencode's HTTP-server product shape (machi provides the event
stream; serving HTTP is the embedder's job), OAuth browser flows inside the
kernel, memory *product* (port + one adapter only, post-W12).

---

## 4. Phase plan W6F, W7–W12 (the only path to 2.0)

> **Rule:** do not open the next phase's new surface until the current phase exit
> criteria pass. Deepening within a phase is always allowed. Every phase exit leaves
> a working product. All changes remain breaking-without-compat per AGENTS.md.

### W6F — v1.0 freeze closure (bookkeeping, days not weeks)

| # | Work | Notes |
| --- | ------ | ------- |
| 6F.1 | Record W4/W5 exit in this file + `CHANGELOG.md` (done above, decision #7) | honesty ledger |
| 6F.2 | Metric/span catalogue snapshot as a dedicated CI job (not only a test attr) | golden files committed |
| 6F.3 | Bench baselines committed for `machi-runtime/turn_spawn`, `machi-workflow/journal`; CI regression threshold ±20 % | `cargo bench` harness exists |
| 6F.4 | Rustdoc maturity-tag audit: every public item tagged `core` / `optional` / `experimental`; delete hollow API instead of documenting it | grep-driven sweep |
| 6F.5 | Tag **v1.0 freeze**; from here the 2.0 line owns breaking changes | anchor for embedders |

**Exit criteria:** CI green including new snapshot/bench jobs; freeze tag pushed;
CHANGELOG freeze entry recorded.

### W7 — Embedding event surface (E-class, highest priority)

The charter says *embeddable*; without a live event surface that claim is false.
This phase is the kernel analogue of codex's SQ/EQ, pi's `AgentEvent`, and
opencode's event bus — as a **library port, not a server**.

**Events (`machi-protocol`):**

| # | Work | Design |
| --- | ------ | -------- |
| 7.1 | `TurnEvent` enum (`#[non_exhaustive]`, serde): `TurnStarted`, `StepStarted{step}`, `TextDelta`, `ReasoningDelta`, `ToolCallPlanned`, `ToolExecutionStart/Update/End`, `SpawnStarted/Finished{agent_id,label,depth}`, `CompactionApplied{strategy}`, `InterjectionApplied`, `StationarityNudge`, `TurnPaused`, `TurnFinished{outcome}`, `TurnAborted{reason}` | pure values; field names reuse the existing span catalogue (`machi.run_id`, `machi.step`, …); every event carries `run_id` + monotonic `seq` |
| 7.2 | Ordering contract documented as invariants: per-run `seq` strictly monotonic; every `*Start` has exactly one matching `*End/Finished/Aborted`; deltas never follow their terminal | property tests |

**Runtime (`machi-runtime`):**

| # | Work | Design |
| --- | ------ | -------- |
| 7.3 | `TurnOptions.event_tx: Option<mpsc::UnboundedSender<TurnEvent>>`; the turn loop emits at every effect point; sampling in stream mode forwards `TextDelta`/`ReasoningDelta` instead of silently aggregating; `ToolDispatch` forwards `ToolProgress` frames as `ToolExecutionUpdate` | `None` ⇒ zero-cost (no allocation on the hot path; bench-guarded) |
| 7.4 | `InProcessHost` propagates: child spawns emit `SpawnStarted/Finished` into the **parent's** event stream with `depth`; workflow adapter emits the same shapes for Mode B (`phase` → `StepStarted`-analogue) — invariant #3/#6 | isomorphism test in dual_modes |
| 7.5 | Follow-up queue (pi contract): `TurnOptions.followup_rx` drained when the model produces a final message — if non-empty, the turn continues instead of completing; modes `OneAtATime` / `All`; existing `interject_rx` becomes the steering half of the same surface | ordering test: steering before sample, follow-up after final |
| 7.6 | `SessionChannel` minimal SQ/EQ helper: `Op::{UserInput, Interrupt, Compact, Shutdown}` submission side + the `TurnEvent` receiver; a thin optional convenience over `Session`, not a server | codex `submission_loop` contract, kernel subset only |

**Exit criteria:** example host (`examples/live_events.rs`) renders a complete
turn — deltas, tool progress, nested spawn tree — from events alone; ordering
property tests (fuzz interleavings); Mode A/B spawn-event isomorphism test;
`event_tx: None` bench delta < 2 % vs v1.0 baseline.

### W8 — Enforced security boundary (S-class)

| # | Work | Design |
| --- | ------ | -------- |
| 8.1 | **Jail symlink fix (immediate):** `resolve_jailed` gains a filesystem check — canonicalize the deepest existing ancestor of the resolved path and require it to remain under the canonicalized jail root; reject otherwise. Lexical pass stays as the first (cheap, pure) gate | regression test: in-jail symlink → outside target must fail with `EscapesJail` |
| 8.2 | New crate `machi-sandbox`: `SandboxPolicy { fs: FsPolicy::{ReadOnly(paths), ReadWrite(paths)}, net: NetPolicy::{Denied, Allowed} }` + `SandboxBackend { fn wrap(&self, policy, cmd: Command) -> Result<Command> }`; adapters: `SeatbeltBackend` (feature `seatbelt`, macOS `sandbox-exec` with embedded `.sbpl` base policy) and `LandlockBackend` (feature `landlock`); `NoSandbox` is an explicit, log-visible choice | codex `sandboxing` contract; policies are data, backends are leaves |
| 8.3 | `ExecPolicy` port in `machi-tools`: normalized command prefix rules → `Allow / Deny / Ask`; unknown commands route to the existing `ApprovalGate`; assessment order fixed: exec-policy → sandbox wrap → approval | codex `exec_policy.rs` + opencode wildcard ruleset contracts |
| 8.4 | **Persistent exec sessions:** `ExecSessionTool` — create/reuse named sessions, write stdin, poll output with `yield_time_ms`, output caps with head/tail truncation metadata, idle GC, `InterruptBehavior` honored; PTY backend feature-gated | codex `unified_exec` contract; pi `truncate.ts` head/line/tail semantics |
| 8.5 | Secure-by-default rollout (invariant #7): `ShellTool` / `ExecSessionTool` require either a `SandboxBackend` or an explicit `TrustedExecution` marker in their constructor; the "intended for trusted hosts" disclaimer is deleted because it is no longer true | breaking change, no compat shim |

**Exit criteria:** symlink-escape and `..`-escape regression matrix; sandbox
denial → approval-escalation path test (macOS CI leg for seatbelt, Linux leg
for landlock); exec-session matrix (timeout / cancel / cap / reuse / GC);
`ShellTool::default()` no longer compiles without a policy decision.

### W9 — MCP client (X-class, ecosystem entry ticket)

| # | Work | Design |
| --- | ------ | -------- |
| 9.1 | New leaf crate `machi-mcp` (feature `mcp` on the facade) implementing the **existing** `ToolSource` port: one MCP server ⇒ one `ToolSource`; tools surface as `DynTool` with `server__tool` namespacing on collision | the port was built for this; zero kernel changes |
| 9.2 | Transport via the official `rmcp` SDK: stdio (local command) + streamable HTTP (remote url); config shape mirrors opencode (`command/cwd/env` vs `url/headers`, per-server timeouts); init state machine `NotStarted → Starting → Finished`; liveness probe; config diff (added/removed/retained) for hot reload | grok `servers.rs` contract |
| 9.3 | Tool-name validation `^[a-zA-Z_][a-zA-Z0-9_-]{0,63}$` (cross-provider LLM API requirement); invalid names rejected at registration, never silently renamed | grok contract |
| 9.4 | `ToolMetadata` mapping: MCP annotations (`readOnlyHint`, `destructiveHint`) → `ConcurrencyMode`/`Destructiveness`; absent annotations default to `Concurrent` + `Destructiveness::Irreversible` (fail-closed, approval consulted under the default policy) | safe defaults |
| 9.5 | `CredentialStore` port (keyed `"{server}:{url}"`, JSON file adapter); OAuth browser flows stay out of the kernel (invariant #8) | grok `credentials.rs` shape |

**Exit criteria:** scripted mock MCP server (in-repo, no network) covering
handshake / list / call / cancel / timeout / malformed-frame; dispatch
integration test (MCP tool under capability + approval + concurrency chains);
firewall tests extended (`machi-workflow ↛ machi-mcp`, `machi-mcp ↛ machi-runtime`).

### W10 — Hooks system (on top of the W3.5 lifecycle port + W7 events)

| # | Work | Design |
| --- | ------ | -------- |
| 10.1 | `HookEvent` envelope reusing the `TurnEvent` taxonomy plus session-scoped events: `session_start/end`, `pre_tool_use`, `post_tool_use`, `pre/post_compact`, `spawn_start/stop`, `turn_stop`; envelope carries session/agent/run ids + event payload | grok `event.rs` contract; **no parallel taxonomy** — hooks observe the same events embedders do |
| 10.2 | Gate semantics: `pre_tool_use` is the only blocking gate — it may `Deny{reason}` (reason surfaces as the tool result error, model-visible) or rewrite arguments; all other hooks are observe-only, **fail-open**, default timeout 5 s; `turn_stop` gate honors a long timeout (600 s) and may inject additional context or prevent completion (feeds the existing `GateChain`) | grok `GateKind` + pi `tool_call`/`tool_result` rewrite contracts |
| 10.3 | Handler ports: in-process `Hook` trait first (`Send + Sync`, async); `CommandHookRunner` (feature `hooks-command`): JSON envelope on stdin, decision JSON on stdout, `sh -c` execution wrapped by the W8 sandbox policy | HTTP handler deferred until a production call site exists |
| 10.4 | Discovery: `.machi/hooks/*.json` (project, cwd → repo root) + `~/.machi/hooks/` (user), same shadowing rules as agent discovery; project hooks require explicit trust (constructor opt-in), matching invariant #7 | grok `discovery.rs` + `trust.rs` |

**Exit criteria:** block / rewrite / timeout / fail-open four-way matrix; hooks
fire isomorphically for Mode A and Mode B subagents (dual_modes); a hook
failure can never break tool-call pairing (state invariant test); sandbox-wrapped
command handler test.

### W11 — Model supply & ledger 2.0 (X + V class)

| # | Work | Design |
| --- | ------ | -------- |
| 11.1 | `AnthropicSampler` (Messages API, feature `anthropic`) and `OpenAiResponsesSampler` (Responses API, feature `responses`); providers stay transport-only — the W2 decorator stack (`RetryingSampler`/`BreakerSampler`) wraps them unchanged | decision #4 holds |
| 11.2 | `ModelCatalog` port in `machi-llm`: `ModelInfo { context_window, max_output_tokens, pricing{input,output,cache_read,cache_write}, capabilities }`; built-in static table for shipped providers + injectable external source (models.dev JSON shape); `TurnOptions.context_window_tokens` auto-resolves from the catalog when unset | pi `models.ts` / opencode catalog contracts |
| 11.3 | Cost accounting: `UsageLedger` gains cost columns computed from catalog pricing (cache read/write priced separately, per-model and per-prompt breakdowns extend the existing ledger) | pi `Usage.cost` contract |
| 11.4 | `AuthProvider` port: credential snapshot (`api_key` from env / command output) + refresh hook; adapters stay leaf; OAuth flows remain excluded (invariant #8) | grok `xai-grok-auth` inversion seam |
| 11.5 | **Record/replay conformance harness (V-class):** HTTP fixtures recorded once against live providers, replayed offline in CI; one shared suite (sampling, streaming, tool calls, 429 storm, idle timeout, malformed SSE) that all three HTTP providers must pass; fixtures live in-repo, keys never do | closes the "mock-only" gap without networked CI |
| 11.6 | Compaction trigger math upgraded with catalog data: trigger when `estimate > context − max(max_output, buffer)` (opencode formula) as the default `TokenThreshold` derivation | `compaction.ts` contract |

**Exit criteria:** three HTTP providers green on the shared conformance suite;
cost assertions in dual_modes ledgers; catalog fallback behavior tested
(unknown model ⇒ preflight disabled with a typed warning, never a guess);
firewall intact (`machi-workflow ↛ machi-llm`).

### W12 — Session tree & reversibility (D-class)

| # | Work | Design |
| --- | ------ | -------- |
| 12.1 | Session format v3 (breaking, no migration): JSONL entries gain `id` / `parent_id` forming a tree; branch navigation, fork-to-new-session, branch summaries on leave; compaction entries record `first_kept_entry_id`; context rebuild = latest compaction summary + entries from `first_kept_entry_id` | pi `session-manager.ts` contract; reuses W1 torn-write repair |
| 12.2 | `SnapshotPort`: `capture() → SnapshotId`, `diff`, `restore`, turn-level revert with `stage / commit / clear` semantics; `GitSnapshot` adapter (feature `snapshot-git`) using git tree objects, no working-branch pollution | opencode `snapshot.ts` + `revert.ts` contracts |
| 12.3 | `WorktreeIsolation` adapter for the **existing** `IsolationBackend` port: `git worktree add --no-checkout` + optional dirty-file copy, cleanup + age-based GC; child agents get true filesystem isolation composable with W8 sandbox policies | grok `fast-worktree` minimal subset (no BTRFS/overlay until a call site demands it) |
| 12.4 | Deferred (ports only, adapters post-2.0): hunk attribution (`HunkEvent` shape from grok), `MemoryPort` SQLite+FTS5 adapter | ports exist; adapters wait for production call sites |

**Exit criteria:** fork → branch → revert round-trip test; session-v3 fuzz
(random tree operations, invariants: single root, acyclic, dense ids); 64
concurrent spawns under `WorktreeIsolation` stress; revert never leaves the
work tree in a mixed state (stage/commit/clear matrix).

### 2.0 freeze criteria

- Every phase W7–W12 exit re-verified honest green in one CI run.
- Test count is not a goal; **contract coverage** is: event ordering, security
  regression, MCP protocol, hook gates, provider conformance, session tree fuzz
  suites all present and meaningful.
- Docs = code; maturity tags current; hollow API deleted.
- `machi` facade compiles with `default`, `full`, and each feature standalone.
- Freeze recorded here + `CHANGELOG.md`; 2.0 tag pushed.

---

## 5. Quality gates (every phase exit)

```bash
cargo test --workspace --all-features
cargo +nightly clippy --workspace --fix --all-targets --all-features \
  --allow-dirty --allow-staged -- -D warnings
cargo fmt --all -- --check
cargo deny check
cargo bench -- --save-baseline phase-exit   # compare vs committed baseline, ±20 %
# firewalls: workflow ↛ llm/mcp/HTTP ; types/protocol ↛ wire formats ; mcp ↛ runtime
# snapshots: metric catalogue, span catalogue, TurnEvent serde shapes (W7+)
# security: jail escape regression suite (W8+)
# conformance: provider record/replay suite (W11+)
```

### PR discipline

1. One work package per PR when possible.
2. New public API requires: production call site + test + maturity tag.
3. "Reference repo has X" is never sufficient; ask "does the embeddable dual-mode slice lack X?"
4. Prefer deleting hollow API over documenting it. No BC shims (AGENTS.md).
5. Report progress only as **phase exit status**, never symbol counts or LOC.
6. New runtime effects must emit `TurnEvent`s in the same PR (invariant #6).
7. New process-executing surfaces must declare their policy story in the same PR (invariant #7).

### API maturity tags (rustdoc)

```rust
/// Maturity: core
/// Maturity: optional (feature = "…")
/// Maturity: experimental — may break without major bump until freeze
```

---

## 6. Feature matrix (v3 target)

| Feature | Contents | Phase |
| --------- | ---------- | ------- |
| default | runtime + workflow + mock | shipped |
| `openai` / `ollama` | HTTP samplers | shipped |
| `anthropic` / `responses` | Anthropic Messages / OpenAI Responses samplers | W11 |
| `toolkit` | cwd-jailed fs/shell tools (hardened W8) | shipped |
| `state` / `compaction` / `obs` | respective crates | shipped |
| `seatbelt` / `landlock` | OS sandbox backends | W8 |
| `mcp` | `machi-mcp` ToolSource adapter | W9 |
| `hooks-command` | command hook runner | W10 |
| `snapshot-git` | git snapshot adapter | W12 |
| `worktree` | worktree isolation backend | W12 |
| `full` | all of the above | rolling |

Semver: **v1.0 freeze at W6F exit** anchors embedders; the 2.0 line owns all
W7–W12 breaking changes, landing without compatibility layers per AGENTS.md.
The 2.0 freeze is declared only when §4's freeze criteria are recorded here and
in `CHANGELOG.md`.

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
| 7 | 2026-08-11 | W4/W5 exit verified green (612 tests). W4.2 deviation accepted: `SummarizingCompaction` ships a sync summarize-callback port instead of an `llm-compaction` feature — `machi-compaction` stays LLM-free. |
| 8 | 2026-08-11 | ROADMAP v3 replaces v2. References expanded to codex / pi / opencode alongside grok-build; extraction rule unchanged (contracts, never code). Execution queue is W6F → W7 → … → W12. |
| 9 | 2026-08-11 | **Embedding event surface (W7) is the top priority**: invariant #6 (event completeness) adopted; `TurnEvent` lives in `machi-protocol`; no capability ships without event observability. |
| 10 | 2026-08-11 | **Secure-by-default (W8)**: invariant #7 adopted; `resolve_jailed` symlink hole acknowledged and scheduled as W8.1; `ShellTool` trust mode becomes an explicit opt-out — breaking, no shim. |
| 11 | 2026-08-11 | MCP ships as a leaf `ToolSource` adapter crate (`machi-mcp`) over the official rmcp SDK; OAuth browser flows and marketplace surfaces stay permanently outside the kernel (invariant #8). |
| 12 | 2026-08-11 | Provider validation strategy: record/replay conformance fixtures in-repo (W11.5); networked tests never gate CI; live keys never enter the repository. |

**Authority:** this file > ad-hoc chat memory > README feature enthusiasm.
**Local drafts:** `docs/` is gitignored; commit architecture truth in `README.md` + this file + rustdoc.
