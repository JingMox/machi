//! Optional metrics sink for production hosts.

use std::sync::Arc;

/// No-op and host-provided metrics.
pub trait MetricsSink: Send + Sync {
    /// Increment a counter.
    fn counter(&self, name: &str, value: u64, labels: &[(&str, &str)]);
    /// Observe a histogram sample.
    fn histogram(&self, name: &str, value: f64, labels: &[(&str, &str)]);
    /// Set a gauge.
    fn gauge(&self, name: &str, value: f64, labels: &[(&str, &str)]);
}

/// Discards all metrics.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopMetrics;

impl MetricsSink for NoopMetrics {
    fn counter(&self, _name: &str, _value: u64, _labels: &[(&str, &str)]) {}
    fn histogram(&self, _name: &str, _value: f64, _labels: &[(&str, &str)]) {}
    fn gauge(&self, _name: &str, _value: f64, _labels: &[(&str, &str)]) {}
}

/// Shared metrics handle.
pub type SharedMetrics = Arc<dyn MetricsSink>;

/// Record a completed turn.
pub fn record_turn(metrics: &dyn MetricsSink, status: &str, steps: u64, duration_ms: f64) {
    metrics.counter("machi_turns_total", 1, &[("status", status)]);
    metrics.histogram("machi_turn_steps", steps as f64, &[]); // steps is u64; f64 is fine for telemetry
    metrics.histogram("machi_turn_duration_ms", duration_ms, &[]);
}

/// Record a nested spawn.
pub fn record_spawn(metrics: &dyn MetricsSink, status: &str) {
    metrics.counter("machi_spawns_total", 1, &[("status", status)]);
}
