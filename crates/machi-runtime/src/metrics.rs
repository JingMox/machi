//! Re-export observability metrics from [`machi_obs`].
//!
//! Runtime keeps this module path so call sites stay short; the source of truth
//! is the `machi-obs` crate.

pub use machi_obs::{
    METRIC_COMPACTIONS_TOTAL, METRIC_SAMPLE_DURATION_MS, METRIC_SPAWNS_TOTAL, METRIC_TOKENS_TOTAL,
    METRIC_TOOL_CALLS_TOTAL, METRIC_TOOL_DURATION_MS, METRIC_TURN_DURATION_MS, METRIC_TURN_STEPS,
    METRIC_TURNS_TOTAL, METRIC_WORKFLOW_AGENTS_TOTAL, METRIC_WORKFLOW_RUNS_TOTAL, MetricsSink,
    NoopMetrics, SharedMetrics, record_compaction, record_sample, record_spawn, record_tool_call,
    record_turn, record_workflow_agents, record_workflow_run, required_metric_names,
};
