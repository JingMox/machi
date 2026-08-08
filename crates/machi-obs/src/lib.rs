//! Observability spine for the Machi kernel.
//!
//! - Stable **metric series** names and record helpers ([`metrics`])
//! - Re-export of **span / field** catalogue from [`machi_protocol`]
//! - **Redaction** helpers so hosts never log secrets by accident
//!
//! Production hosts inject a real [`MetricsSink`] (Prometheus / OTEL adapters
//! live behind optional features in later phases). The kernel only depends on
//! the trait surface.

#![forbid(unsafe_code)]

pub mod metrics;
pub mod prometheus;
pub mod recording;
pub mod redact;

pub use metrics::{
    METRIC_COMPACTIONS_TOTAL, METRIC_SAMPLE_DURATION_MS, METRIC_SPAWNS_TOTAL, METRIC_TOKENS_TOTAL,
    METRIC_TOOL_CALLS_TOTAL, METRIC_TOOL_DURATION_MS, METRIC_TURN_DURATION_MS, METRIC_TURN_STEPS,
    METRIC_TURNS_TOTAL, METRIC_WORKFLOW_AGENTS_TOTAL, METRIC_WORKFLOW_RUNS_TOTAL, MetricsSink,
    NoopMetrics, SharedMetrics, emit_catalogue_smoke, metric_catalogue_snapshot, record_compaction,
    record_sample, record_spawn, record_tool_call, record_turn, record_workflow_agents,
    record_workflow_run, required_metric_names,
};
pub use prometheus::PrometheusRecorder;
pub use recording::{CounterEvent, GaugeEvent, HistogramEvent, RecordingMetrics};
pub use redact::{REDACTED, looks_like_secret_key, redact_key_value, redact_map};

// Span contract lives in protocol so pure crates can name spans without pulling
// metrics. Re-export for a single import path in hosts.
pub use machi_protocol::observability::{
    SPAN_COMPACT, SPAN_SAMPLE, SPAN_SESSION, SPAN_SPAWN, SPAN_TOOL, SPAN_TOOL_BATCH, SPAN_TURN,
    SPAN_WORKFLOW, SPAN_WORKFLOW_HOST, field, required_span_names, span_catalogue_snapshot,
};
