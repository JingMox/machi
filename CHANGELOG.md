# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

**Stability:** workspace crates are **`0.9.x` pre-stability**. The crates.io
publish of `1.0.0` was premature and is **not** an API freeze. Breaking changes
land without compatibility layers ([`AGENTS.md`](./AGENTS.md)). A real `1.0`
requires W7–W12 charter planes closed — see [`ROADMAP.md`](./ROADMAP.md).

## [Unreleased]

### Added

- **`TurnEvent` live observation surface (W7):**
  - `machi-protocol::{TurnEvent, TurnEventKind}` (serde, `non_exhaustive`)
  - `EventBus` / `EventSink`; `TurnOptions::with_event_tx` / `with_events`
  - Turn loop emits started/step/tools/compaction/stationarity/finished/aborted
  - Stream path forwards `TextDelta` / `ReasoningDelta`
  - Nested spawn emits `SpawnStarted` / `SpawnFinished` on the parent bus
  - Mode B: `run_workflow_configured_with_events` wires the same spawn events
  - Example: `cargo run -p machi --example live_events --features runtime`
- **Jail realpath (W8.1):** `resolve_jailed` canonicalizes deepest existing ancestor;
  in-jail symlink → outside host path is rejected (`EscapesJail`).

### Changed

- **Version:** workspace package version **`0.9.0`** (pre-stability). Drop
  freeze / v1.0-stability narrative from ROADMAP.
- **Control plane fail-closed:**
  - Tool dispatch: per-call cancel child; **timeout cancels** nested work.
  - Host: **refund** agent budget on pre-start failures (isolation/build/state/resume err).
  - Completion gate: exhausted reminders → `GateDecision::Fail` → `ErrorCode::RuntimeGate`
    (no silent success).
- **Errors:** remove `MachiError::cancelled` alias; use `runtime_cancelled` /
  `llm_cancelled` by domain.

### Prior (historical)

See git history for W1–W6 work while the tree was mis-labeled `1.0.0`.

## [0.9.0] — 2026-08-12

Pre-stability line after correcting the premature `1.0.0` label.

[Unreleased]: https://github.com/qntx/machi/compare/v0.9.0...HEAD
[0.9.0]: https://github.com/qntx/machi/releases/tag/v0.9.0
