# Machi Master Roadmap

**Status:** living north star — all implementation work is measured against this document.  
**Product:** embeddable Rust **multi-agent library** (`machi` workspace), not a TUI / shell product.  
**Reference:** `3rdparty/grok-build` — extract **contracts and layering**, never bulk-copy product code.  
**Baseline:** `ae98cca` (~13k LOC kernel, ~95 test attrs, dual-mode demo paths green).  
**Updated:** 2026-08-08.

---

## 0. Charter (one sentence)

Build a **complete, embeddable, offline-testable multi-agent runtime library** that first-class supports **two multi-agent delegation models**, shares one turn/tools/host/obs/state kernel, and can be thickened to production-grade completeness without horizontal feature thrash.

| Is | Is not |
|----|--------|
| Reusable lib (ports & adapters) | Grok TUI / pager / ACP product shell |
| Dual multi-agent (dynamic + journaled workflow) | “Feature name checklist” clone of Grok crates |
| Fail-closed budget / cancel / capability / journal | Silent degrade / half-live public APIs |
| Vertical slice depth first | Peripheral pile (MCP market, memory product, hooks early) |

---

## 1. Goals (must stay fixed)

### 1.1 Core product: two multi-agent modes

```
                    ┌──────────────────────────────────────┐
                    │         Shared kernel                 │
                    │  types · tools · turn · state · obs   │
                    │  SessionHost · budget · cancel        │
                    └───────────────┬──────────────────────┘
                                    │
              ┌─────────────────────┼─────────────────────┐
              ▼                                           ▼
   Mode A — Dynamic Delegation              Mode B — Journaled Workflow
   程序 spawn_agents / 模型 spawn_agent       Rhai agent()/parallel()
   运行时决定谁、并行谁                         确定性脚本 + host RPC
   子 turn 嵌套在父 ReAct 内                   引擎永不采样 LLM
   对应 Grok: SessionActor spawn              对应 Grok: xai-workflow
```

| Mode | Entry points | Nested execution | Resume model |
|------|--------------|------------------|--------------|
| **A Dynamic** | `SessionHost::spawn_agent(s)`, `SpawnAgentTool` | Same `TurnRuntime` | Chat persistence (handle/file), not journal |
| **B Workflow** | `run_workflow` / `run_workflow_on_host` | Host maps `SpawnAgent` → Mode A path | Journal seq+hash, no re-sample |

**Hard invariants:**

1. A and B share **one** `SessionHost` execution path for nested agents.  
2. `machi-workflow` **never** depends on `machi-llm` / HTTP (firewall test permanent).  
3. Budget, cancel, capability, usage, metrics are **isomorphic** across A and B.  
4. B may call A (workflow spawn → host → turn). A must not hard-require Rhai.

### 1.2 Lib shape

```
App / CLI / service
        │ depends on
        ▼
 machi facade (feature-gated)
        │ injects
        ▼
 Ports: LlmSampler · SessionHost · MetricsSink · ApprovalGate · ChatPersistence
        · IsolationBackend · ToolSource
```

- Default CI path: **MockSampler**, no network.  
- Semver: 1.x consolidation may break; public API tags: **core / optional / experimental**.  
- Zero panics on production paths; `forbid(unsafe_code)` workspace-wide.

### 1.3 Ultimate Definition of Done

| # | Criterion | Metric |
|---|-----------|--------|
| U1 | Mode A: program + tool spawn, depth limit, concurrency cap, budget, cancel, no leak | stress suite green |
| U2 | Mode B: full host RPC table (Unsupported explicit), validate, durable journal cross-process, parallel barrier | contract suite green |
| U3 | Turn: sample/stream, tool stream, stop gates, schema retry, compaction pluggable | state-machine table tests |
| U4 | Session: handle SoT + turn buffer narrative only; usage ledger; checkpoint | restart recovery tests |
| U5 | Tools: typed stream Progress→Terminal; toolkit sufficient for repo tasks; approval policies | dispatch matrix |
| U6 | Obs: stable metric/span names; redact; Prometheus export; OTEL optional feature | CI snapshot + export sample |
| U7 | Tests ≥500 meaningful; clippy nightly `-D warnings`; firewall always green | CI |
| U8 | Docs = code; no phantom crates; no hollow public APIs | rustdoc maturity tags |
| U9 | Kernel ~40–90k real LOC (density, not padding) | honest size |

**Honesty dial:** P1–P6 ✅ · N1 demos ✅ · P7 partial · main-path ~78–82% Ultimate · tests ~174/500.

### 1.4 Permanent non-goals

| Non-goal | Why |
|----------|-----|
| Grok TUI / full ACP product | Not lib responsibility |
| Default worktree pool / full OS sandbox | Isolation trait later; default InProcess |
| Plugin marketplace / encrypted prompts | Product dialect |
| Bulk paste of Grok sources | License + maintainability |
| Phantom crates documented as shipped | Credibility |

---

## 2. Bottom-up architecture (design order)

Build and harden layers **bottom → top**. Upper layers must not invent semantics lower layers do not own.

### Layer map

```
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
── later ──
L8  Hooks / derive / isolation / MCP adapters (only after L6–L7 thick)
```

### L0 — Pure types & protocol

**Owns:** `MachiError` + `ErrorCode` + `RetryClass`, ids (`AgentId`, `SessionId`, `RunId`), `Message`, `Usage`, `Deadline`, `ToolId`, content blocks, stable span name catalogue.

**Rules:**

- No HTTP, no Tokio business logic, no filesystem product semantics.  
- Errors are **typed**; call sites never string-match for control flow.  
- Protocol constants (metric names, span names) change only with CI snapshot updates.

**Exit when:** serde round-trips + error taxonomy complete for host/turn/tool/workflow codes used in L6.

### L1 — Observability ports

**Owns:** `MetricsSink`, `RecordingMetrics`, `PrometheusRecorder`, redaction helpers.

**Rules:**

- All hot paths accept injectable sink (default no-op).  
- Redact secrets in any logged headers/bodies (`x-api-key`, bearer, etc.).  
- Metric names stable; document in one table.

### L2 — Tools

**Owns:** `DynTool`, `ToolRegistry`, `CapabilityMode`, `ToolDispatch`, `ToolStreamItem` (`Progress*` → `Terminal`), `ApprovalGate`.

**Grok extract:** Progress→Terminal stream discipline; concurrent ReadOnly vs serial Exclusive; capability fail-closed.

**Rules:**

- Every tool is ordinary `DynTool` — including `SpawnAgentTool` (Mode A model path).  
- Dispatch owns concurrency windows; tools do not spawn free-floating task graphs.

### L3 — LLM port

**Owns:** `LlmSampler` (`sample`, optional `sample_stream`), `SampleRequest`/`Response`, `MockSampler`, provider adapters behind features.

**Rules:**

- Mock is the CI default and race-safe (prefer keyed maps over pure FIFO for concurrent spawns).  
- Providers never leak into workflow crate.

### L4 — Agent definition / instance

**Owns:** `AgentDefinition`, `Agent` (resolved instance), `AgentBuilder`, markdown discovery (`.machi/agents`).

**Grok extract:** Definition ≠ Instance; `agent_type` resolution; allowed subagent types (later).

**Rules:**

- Spawn paths resolve `agent_type` through the same registry as discovery.  
- Builder is the only public construction path for runtime agents.

### L5 — State & compaction

**Owns:**

| Type | Role |
|------|------|
| `VecConversationState` | **Turn buffer** only (ReAct local) |
| `ChatStateHandle` | **Session source of truth** (actor) |
| `FilePersistence` / `MemoryPersistence` | Checkpoint port |
| `CompactionStrategy` / `MaxMessages` | Pre-sample shrink |

**Canonical narrative (no dual SoT):**

```
Session handle (SoT)
   → sync into Vec buffer for turn
   → TurnRuntime mutates buffer
   → write-back + optional FilePersistence checkpoint
```

### L6a — Turn + SessionHost (Mode A kernel)

**TurnRuntime owns:**

```
input → (optional compact) → sample → tool_calls?
  → dispatch tools (approval + capability) → append results → loop
  → stop gates / structured output schema retries → TurnOutcome
```

**SessionHost owns:**

```
SpawnOpts → reserve budget → depth/concurrency checks → build child Agent
  → child TurnRuntime → AgentRunResult → metrics
```

**SpawnOpts (target complete fields):**

| Field | Phase | Notes |
|-------|-------|-------|
| `prompt`, `label`, `model` | done | |
| `capability_mode`, `max_steps`, `cancel` | done | |
| `agent_type` | P1/P4 | resolve definition |
| `output_schema` | P1/P2 | force structured child |
| `fork_context` | P4 | optional parent transcript inject |
| `resume_from` | P5 | nested run resume id |
| `max_output_tokens` | P2 | sample hint |
| `parent_depth` / host `max_spawn_depth` | **P1** | fail-closed |
| host `max_concurrent_children` | **P1** | semaphore |

**SpawnAgentTool:** model-facing DynTool; maps JSON args → SpawnOpts; blocks until child completes.

### L6b — Workflow engine (Mode B pure)

**Owns:** Rhai `agent` / `parallel` / `complete` / budget APIs, `Journal`, `validate_script`, meta extract.

**Does not own:** LLM, HTTP, UI, git product, process tree.

**Journal contract:**

```
entry = { seq, kind, req_hash, result, at_ms }
resume: if journal.covers(seq) → return recorded result (no host spawn)
divergence: req_hash mismatch → JournalError::Divergence fail-closed
durable: optional jsonl path; load + append atomic enough for crash resume tests
```

**WorkflowHostRequest table (parity with Grok host surface):**

| Kind | Journaled | Core host | Optional |
|------|-----------|-----------|----------|
| Reserve/Release agent calls | yes | yes | |
| SpawnAgent | yes | yes → SessionHost | |
| BudgetQuery | yes | yes | |
| Phase / Log / Telemetry | no (or soft) | log only | |
| RenderTemplate | yes | side_effects | |
| Scratch R/W | yes | side_effects | |
| GitDiffSince | yes | **optional** (set cwd) | default Unsupported if unset |

### L6c — Workflow host adapter

**Owns:** channel service loop, concurrent SpawnAgent tasks, budget reserve/release, map `AgentOpts` → `SpawnOpts`, side effects store.

**Critical mapping rule:** every field on `AgentOpts` that Mode A understands must pass through `to_spawn_opts` — **no silent drop** of `agent_type` / `output_schema` once SpawnOpts grows them.

### L7 — Facade

**Owns:** re-exports, feature flags, examples, dual_modes contract suite.

**Feature matrix (stable):**

| Feature | Contents |
|---------|----------|
| default | runtime + workflow + mock |
| `openai` / `ollama` | HTTP samplers |
| `toolkit` | jailed fs/shell tools |
| `state` / `compaction` / `obs` | respective crates |
| `full` | all of the above |

---

## 3. Engineering extract from Grok (contracts only)

### 3.1 Layering axiom (mandatory)

```
Definition  ≠  Instance  ≠  Runtime  ≠  Host
```

| Layer | Grok | Machi |
|-------|------|-------|
| Definition | agent markdown / types | `machi-agent` definition + discovery |
| Instance | resolved agent + tools | `Agent` + registry |
| Runtime | turn / sampler_turn / stop_gate | `TurnRuntime` + gates + schema |
| Host | shell session spawn + workflow host | `SessionHost` + `workflow_host` |

### 3.2 Reading map (open while implementing)

| Theme | Path under `3rdparty/grok-build/` |
|-------|-----------------------------------|
| Pure workflow engine | `crates/codegen/xai-workflow/src/{engine,host,journal,validate}.rs` |
| Host RPC table | `xai-workflow/src/host.rs` (AgentOpts + WorkflowHostRequest) |
| Dynamic spawn | `crates/codegen/xai-grok-shell/src/session/acp_session_impl/spawn.rs` |
| Tool stream / dispatch | `crates/common/xai-tool-runtime/src/` |
| Tool protocol frames | `crates/common/xai-tool-protocol/src/` |
| Chat state actor | `crates/codegen/xai-chat-state/src/` |
| Workflow product shell | shell `session/workflow/*` — **do not copy UI**; learn store/resume ports only |

### 3.3 Discipline borrowed

1. Protocol before implementation (kinds, hashes, span names).  
2. Fail-closed on budget, capability, journal fork.  
3. Cancel tokens through turn / tool / spawn children.  
4. Tool stream: Progress\* then Terminal.  
5. Stop gates after model stop.  
6. `validate_script` / probe host without real models.  
7. Stable observability fields + redaction.  
8. Test pyramid: unit → contract → stress; not only happy path.

### 3.4 Explicitly not borrowed

- TUI, pager, auth, marketplace, voice, mermaid product surface.  
- Shell-owned process trees as library defaults.  
- Any GPL-sensitive bulk paste; re-implement against contract tables.

### 3.5 Thickness delta (honest)

| Component | Grok (approx) | Machi (approx) | Implication |
|-----------|---------------|----------------|-------------|
| workflow engine.rs | ~1.8k LOC | ~0.8k | B still thin on edge cases / hints |
| workflow journal | ~0.7k | ~0.3k | durable/prune/divergence paths thinner |
| host spawn | multi-k product | ~0.4k host | depth/agent_type/fork missing |
| tool-runtime | large | modest | stream stress incomplete |

Machi wins on **clean DAG + firewall + dual-mode intentional API**. Grok wins on **edge thickness**. Roadmap closes thickness without importing product shell.

---

## 4. Baseline gap matrix (workbench)

| Area | Now | Gap class | Target phase |
|------|-----|-----------|--------------|
| dual_modes happy path | yes | B-class (need contract suite) | P1 |
| Host budget | yes | OK | maintain |
| Host max depth / concurrency | **no** | A-class hole | **P1** |
| SpawnOpts ↔ AgentOpts field parity | partial (drops agent_type, schema, fork) | A-class | **P1** |
| Journal durable load/resume | path exists | need cross-process + prune stress | **P1** |
| validate_script coverage | basic | more script patterns | P1 |
| Session SoT narrative | documented; demos still Vec-heavy | consistency | P1 docs/examples + P5 default |
| Turn stream sample path | present, thin | production thickness | P2 |
| Tool Progress observability | types exist | end-to-end tests | P2 |
| Toolkit | minimal jailed set | task-driven only | P2 |
| Metrics export | Prometheus text | snapshot CI | P3 |
| agent_type registry on spawn | fields only | resolve + test | P4 |
| hooks / memory / derive | absent | deferred | P4–P5 |
| isolation / MCP | absent | ports only | P6 |

**Class key:** A = dual-mode correctness hole (block “lib ready”); B = depth/tests; C = polish/ecosystem.

---

## 5. Phase plan (only path to Ultimate)

> **Rule:** do not open the next phase’s *new peripherals* until the current phase exit criteria pass. Deepening within a phase is always allowed.

### Phase 0 — Freeze goals & discipline ✅

- [x] Dual-mode product definition  
- [x] Non-goals  
- [x] Vertical slice definition  
- [x] This roadmap  
- **Exit:** README + ROADMAP agree; no phantom shipped crates in docs  

### Phase 1 — Dual-mode contract hardening ✅ (2026-08-08)

**Goal:** “dual multi-agent lib” is true at production-minimum, not demo-only.

| WP | Work | Status |
|----|------|--------|
| 1.1 | SpawnOpts ↔ AgentOpts field parity; no silent drop; fork/resume Unsupported | ✅ |
| 1.2 | `max_spawn_depth` + `max_concurrent_children` fail-closed | ✅ |
| 1.3 | Budget isomorphism A+B | ✅ dual_modes |
| 1.4 | Durable journal fsync + cross-process load + divergence | ✅ |
| 1.5 | validate_script (existing suite retained) | ✅ baseline |
| 1.6 | dual_modes contract suite (13 cases) | ✅ |
| 1.7 | `session_checkpoint` example (handle + checkpoint) | ✅ |

**Exit:** Mode A/B share host path with depth/budget/cancel proofs; journal resume durable; dual_modes is a contract suite not a demo.

**Forbidden remaining (P2+ only):** new toolkit tools, OTEL SDK, MCP, hooks crate, memory product, git feature expansion.

### Phase 2 — Turn & tools production thickness ✅ (2026-08-08)

| WP | Work | Status |
|----|------|--------|
| 2.1 | Turn cancel / deadline / max_steps / schema retry / completion gate | ✅ |
| 2.2 | `drain_with_progress` + dispatch timeout/cancel matrix | ✅ |
| 2.3 | Toolkit (existing e2e write/read retained; no new tools) | ✅ |
| 2.4 | StopGate chain + max-retries complete semantics | ✅ |
| 2.5 | `TokenThreshold` compaction + MaxMessages | ✅ |
| 2.6 | Turn `use_stream` aggregates `sample_stream` | ✅ |

**Exit:** single-agent + tool path thick enough to host serious nested agents.

### Phase 3 — Observability & hardening ✅ (2026-08-08)

| WP | Work | Status |
|----|------|--------|
| 3.1 | Metric/span golden catalogue snapshots | ✅ rename fails CI |
| 3.2 | Prometheus `# HELP`/`# TYPE` + catalogue smoke export | ✅ |
| 3.3 | Stress: cancel×64, tools×64, journal 1k, multi-agent resume | ✅ `tests/stress.rs` |
| 3.4 | OTEL SDK | deferred (sink trait sufficient; no phantom feature) |

**Exit:** production embedders can observe and load-test the kernel.

### Phase 4 — Agent system depth ✅ (2026-08-08)

| WP | Work | Status |
|----|------|--------|
| 4.1 | `AgentRegistry` (insert/merge/discover) + host `with_agent_registry` | ✅ |
| 4.2 | `PromptAssembler` + `ProjectPromptAssembler` (AGENTS.md) | ✅ |
| 4.3 | `fork_context` + `fork_messages` seed child state | ✅ |
| 4.4 | `machi-hooks` | deferred (no product demand) |
| 4.5 | `#[machi::tool]` derive | deferred |

### Phase 5 — Persistence & workflow store ✅ (2026-08-08)

| WP | Work | Status |
|----|------|--------|
| 5.1 | `FilePersistence::for_session`, `open_or_new`, `restore` usage | ✅ |
| 5.2 | `WorkflowRunStore` + Memory/File stores | ✅ |
| 5.3 | `MemoryPort` + `InMemoryMemory` / `NullMemory` | ✅ |

### Phase 6 — Isolation & tool sources (ports first) ✅ (2026-08-09)

| WP | Work | Status |
|----|------|--------|
| 6.1 | `IsolationBackend` + `InProcessIsolation`; host `with_isolation` | ✅ |
| 6.2 | `ToolSource` + `StaticToolSource` + `merge_tool_sources` | ✅ |
| 6.3 | Worktree / MCP product adapters | deferred (ports only) |

### Phase 7 — Ultimate polish → 1.0 freeze (in progress)

| WP | Work | Status |
|----|------|--------|
| 7.1 | API freeze policy + `CHANGELOG.md` | ✅ policy documented; freeze tag **not** cut |
| 7.2 | Smoke benches: turn/spawn + journal | ✅ `machi-runtime` / `machi-workflow` harness=false |
| 7.3 | `cargo deny check` + `SECURITY.md` | ✅ license allow-list + notes |
| 7.4 | Tests ≥500 meaningful | ⏳ ~172 listed; expand by contract, not padding |
| 7.5 | Docs = code; maturity tags on new ports | ⏳ ports tagged; ongoing |
| 7.6 | Tag dual multi-agent lib stable line | ❌ blocked on 7.4 + freeze sign-off |

#### API freeze policy (until declared)

1. Workspace version is already `1.0.0` for consolidation; **semver freeze is
   not active** until this section says freeze is declared and CHANGELOG records
   a freeze release.
2. Until freeze: breaking changes allowed without major bump (AGENTS.md: no BC
   shims). Prefer deleting hollow API over compatibility layers.
3. After freeze: follow SemVer; `core` maturity APIs require major for breaks;
   `experimental` may break in minor until reclassified.
4. New public API requires: production call site + test + maturity tag.

---

## 6. Module design notes (implementation guidance)

### 6.1 Error codes (host / dual-mode relevant)

Keep and extend only with real call sites:

- `HostBudget`, `HostCancelled`, `HostSpawn`, `HostIsolation`  
- Workflow: `HostError::{AgentCallQuotaExceeded, BudgetExceeded, Cancelled, Unsupported, Failed}` mapped at adapter edge  

New codes require: production path + test + rustdoc.

### 6.2 Cancel graph

```
parent cancel
  ├─ turn sample abort
  ├─ tool batch abort
  └─ spawn child tokens (child_token)
       └─ nested turn / tools
workflow cancel
  └─ host service loop reply Cancelled; inflight join
```

### 6.3 Budget graph

```
InProcessHost.agent_budget  ── Mode A slot reserve
workflow adapter budget     ── Reserve/Release + spent on successful spawn
```

Document whether workflow spent is **double-counted** with host budget when both wrap the same host — pick one accounting model in P1 and test it:

**Chosen model (P1):** host owns absolute admission; workflow adapter enforces script-level quota *before* calling host; host may have its own cap. Fail-closed if either rejects. dual_modes tests cover both layers.

### 6.4 dual_modes contract suite outline

Minimum cases (expand, do not shrink):

1. A: two concurrent workers ordered labels  
2. A: budget fail-closed  
3. A: cancel before start  
4. A: depth exceeded  
5. A: concurrent children cap  
6. A: SpawnAgentTool end-to-end with MockSampler  
7. B: plan + parallel barrier outputs  
8. B: resume no second sample  
9. B: journal divergence  
10. B: budget reserve exceed  
11. B: parent cancel mid parallel  
12. A+B: workflow spawn path hits same host spent counter semantics  

### 6.5 API maturity tags (rustdoc)

```rust
/// Maturity: core
/// Maturity: optional (feature = "…")
/// Maturity: experimental — may break without major bump during 0.x/1.x consolidation
```

Hollow experimental APIs without tests are **bugs**.

---

## 7. Quality gates (every phase exit)

```bash
cargo test --workspace --all-features
cargo +nightly clippy --workspace --fix --all-targets --all-features \
  --allow-dirty --allow-staged -- -D warnings
cargo fmt --all -- --check
# firewall: workflow ↛ llm ; types ↛ HTTP
```

Additional as phases land:

- P3+: metric name snapshot test  
- P7: `cargo deny check`, benches smoke  

### PR discipline

1. One work package per PR when possible.  
2. New public API requires: production call site + test + maturity tag.  
3. “Grok has X” is never sufficient reason; ask “does vertical dual-mode slice lack X?”  
4. Prefer deleting hollow API over documenting it.  
5. No backward-compat shims (AGENTS.md): redesign cleanly.

### Completion reporting

Report only: **% of Ultimate DoD** and **phase exit status**.  
Never: “number of exported symbols” or “lines added”.

---

## 8. Success narrative

| Milestone | Meaning |
|-----------|---------|
| MVP demo | ae98cca — paths exist |
| **Lib dual-mode ready** | Phase 1 exit |
| Production embeddable | Phase 2–3 exit |
| Ecosystem extensible | Phase 4–6 |
| Ultimate freeze | Phase 7 → 1.0.0 |

Any commit that **adds surface area while thinning main path** is a roadmap deviation.

---

## 9. Immediate execution queue

### N1 vertical slice (product-usable demos)

1. Rhai surface: `json_encode`, `write_scratch_file` / `read_scratch_file`, `budget`, bare `complete()`  
2. Examples: `repo_task` (toolkit), `workflow_plan` (plan+parallel+scratch), `session_resume`  
3. Next: Phase 6 ports when demos stay green  

### Phase 6 ✅

1. ~~`IsolationBackend` + InProcess~~  
2. ~~`ToolSource` merge~~  

### Phase 7 (active)

1. ~~CHANGELOG + freeze policy + deny/SECURITY + benches smoke~~  
2. Grow meaningful tests toward U7 (≥500) without hollow cases  
3. Maturity tags / docs pass on public surface  
4. Freeze tag only after U1–U8 honest green

---

## 10. Document control

| Date | Change |
|------|--------|
| 2026-08-08 | Master roadmap v1: dual-mode charter, L0–L7, P0–P7, gap matrix, P1 queue |
| 2026-08-08 | Phase 1 implemented: host depth/concurrency, SpawnOpts parity, dual_modes×13, durable journal |
| 2026-08-08 | Phase 2: turn state machine, tool stream matrix, TokenThreshold, sample_stream path |
| 2026-08-08 | Phase 3: metric/span golden, Prometheus HELP/TYPE, stress suite |
| 2026-08-08 | Phase 4: AgentRegistry, PromptAssembler, fork_messages |
| 2026-08-08 | Phase 5: session checkpoint paths, WorkflowRunStore, MemoryPort |
| 2026-08-09 | N1: json_encode/scratch API, repo_task + workflow_plan + session_resume demos |
| 2026-08-09 | Phase 6: IsolationBackend + ToolSource merge ports |
| 2026-08-09 | Phase 7 start: CHANGELOG, freeze policy, deny licenses, SECURITY, benches |

**Authority:** this file > ad-hoc chat memory > README feature enthusiasm.  
**Local drafts:** `docs/` is gitignored; do not rely on it for team truth. Commit architecture truth in `README.md` + this `ROADMAP.md` + rustdoc.
