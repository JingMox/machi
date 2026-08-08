//! Integration: turn emits metrics into `RecordingMetrics` / Prometheus.
#![allow(
    unused_crate_dependencies,
    clippy::expect_used,
    clippy::tests_outside_test_module,
    reason = "integration test"
)]

use std::sync::Arc;

use machi_agent::AgentBuilder;
use machi_llm::MockSampler;
use machi_obs::{
    METRIC_SAMPLE_DURATION_MS, METRIC_TURNS_TOTAL, PrometheusRecorder, RecordingMetrics,
    SharedMetrics, record_turn,
};
use machi_runtime::{Session, TurnInput, TurnOptions, VecConversationState};

#[tokio::test]
async fn session_records_turn_metrics() {
    let rec = Arc::new(RecordingMetrics::new());
    let metrics: SharedMetrics = rec.clone();
    let sampler = Arc::new(MockSampler::new());
    sampler.push_text("ok");
    let agent = AgentBuilder::named("a")
        .model("mock")
        .build()
        .expect("agent");
    let mut state = VecConversationState::new();
    let mut session = Session::new();
    session
        .run_turn_with_metrics(
            &agent,
            sampler.as_ref(),
            &mut state,
            TurnInput::Text("hi".into()),
            TurnOptions::default().with_metrics(metrics),
            rec.as_ref(),
        )
        .await
        .expect("turn");
    assert!(rec.saw(METRIC_TURNS_TOTAL));
    assert!(rec.saw(METRIC_SAMPLE_DURATION_MS) || rec.counter_sum(METRIC_TURNS_TOTAL) >= 1);
}

#[test]
fn prometheus_text_nonempty() {
    let p = PrometheusRecorder::new();
    record_turn(&p, "ok", 3, 1.5);
    let text = p.render();
    assert!(text.contains(METRIC_TURNS_TOTAL), "{text}");
}
