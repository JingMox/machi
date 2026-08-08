//! Prometheus text exposition for captured metrics (zero external deps).

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::sync::Mutex;

use crate::metrics::MetricsSink;

type LabelSet = Vec<(String, String)>;
type SeriesKey = (String, LabelSet);

/// Metrics sink that accumulates samples and can render Prometheus text format 0.0.4.
#[derive(Debug, Default)]
pub struct PrometheusRecorder {
    counters: Mutex<BTreeMap<SeriesKey, u64>>,
    histograms: Mutex<BTreeMap<SeriesKey, Vec<f64>>>,
    gauges: Mutex<BTreeMap<SeriesKey, f64>>,
}

impl PrometheusRecorder {
    /// Empty recorder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Render Prometheus exposition text.
    #[must_use]
    pub fn render(&self) -> String {
        let mut out = String::new();
        {
            let counters = self
                .counters
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for ((name, labels), value) in counters.iter() {
                let _ = writeln!(
                    out,
                    "# TYPE {name} counter\n{name}{} {value}",
                    format_labels(labels)
                );
            }
        }
        {
            let gauges = self
                .gauges
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for ((name, labels), value) in gauges.iter() {
                let _ = writeln!(
                    out,
                    "# TYPE {name} gauge\n{name}{} {value}",
                    format_labels(labels)
                );
            }
        }
        {
            let histograms = self
                .histograms
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for ((name, labels), samples) in histograms.iter() {
                let count = samples.len();
                let sum: f64 = samples.iter().sum();
                let labs = format_labels(labels);
                let _ = writeln!(out, "# TYPE {name} summary");
                let _ = writeln!(out, "{name}_count{labs} {count}");
                let _ = writeln!(out, "{name}_sum{labs} {sum}");
            }
        }
        out
    }
}

impl MetricsSink for PrometheusRecorder {
    fn counter(&self, name: &str, value: u64, labels: &[(&str, &str)]) {
        let key = (name.to_owned(), owned_labels(labels));
        *self
            .counters
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .entry(key)
            .or_insert(0) += value;
    }

    fn histogram(&self, name: &str, value: f64, labels: &[(&str, &str)]) {
        let key = (name.to_owned(), owned_labels(labels));
        self.histograms
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .entry(key)
            .or_default()
            .push(value);
    }

    fn gauge(&self, name: &str, value: f64, labels: &[(&str, &str)]) {
        let key = (name.to_owned(), owned_labels(labels));
        self.gauges
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(key, value);
    }
}

fn owned_labels(labels: &[(&str, &str)]) -> Vec<(String, String)> {
    labels
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect()
}

fn format_labels(labels: &[(String, String)]) -> String {
    if labels.is_empty() {
        return String::new();
    }
    let parts: Vec<String> = labels
        .iter()
        .map(|(k, v)| format!("{k}=\"{}\"", escape_label(v)))
        .collect();
    format!("{{{}}}", parts.join(","))
}

fn escape_label(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::{METRIC_TURNS_TOTAL, record_turn};

    #[test]
    fn renders_counter() {
        let p = PrometheusRecorder::new();
        record_turn(&p, "ok", 1, 2.0);
        let text = p.render();
        assert!(text.contains(METRIC_TURNS_TOTAL), "{text}");
        assert!(text.contains("status=\"ok\""), "{text}");
        assert!(text.contains("_count") || text.contains("counter"), "{text}");
    }
}
