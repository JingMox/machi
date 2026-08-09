# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
with the freeze policy in [`ROADMAP.md`](./ROADMAP.md) § Phase 7.

## [Unreleased]

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
