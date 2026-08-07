//! Facade-level integration: dynamic delegation + journaled workflow modes.
#![allow(
    unused_crate_dependencies,
    reason = "integration binary links facade feature deps"
)]

#[cfg(test)]
mod dual {
    #![allow(
        clippy::expect_used,
        clippy::unwrap_used,
        clippy::indexing_slicing,
        clippy::missing_assert_message,
        reason = "integration tests use expect for setup and assert outcomes"
    )]

    use std::sync::Arc;

    use machi::{
        ErrorCode, InProcessHost, Journal, MockSampler, SessionHost, SpawnOpts, WorkflowOutcome,
        WorkflowRunParams, run_workflow_on_host,
    };
    use tokio::sync::mpsc;
    use tokio_util::sync::CancellationToken;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn dynamic_delegation_two_workers_via_facade() {
        let sampler = Arc::new(MockSampler::new());
        sampler.map_user_text("do A", "result-A");
        sampler.map_user_text("do B", "result-B");

        let host = InProcessHost::new(sampler, Vec::new()).with_agent_budget(8);
        let results = host
            .spawn_agents(vec![
                SpawnOpts::new("do A").with_label("A"),
                SpawnOpts::new("do B").with_label("B"),
            ])
            .await
            .expect("spawn_agents");

        assert_eq!(results.len(), 2, "expected two worker results");
        assert_eq!(results[0].label.as_deref(), Some("A"));
        assert_eq!(results[1].label.as_deref(), Some("B"));
        assert_eq!(results[0].output.as_str(), Some("result-A"));
        assert_eq!(results[1].output.as_str(), Some("result-B"));
        assert!(results.iter().all(|r| r.success && !r.cancelled));
        assert_eq!(host.agents_spent(), 2);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn dynamic_delegation_budget_fail_closed() {
        let sampler = Arc::new(MockSampler::new());
        sampler.map_user_text("only", "ok");
        let host = InProcessHost::new(sampler, Vec::new()).with_agent_budget(1);
        host.spawn_agent(SpawnOpts::new("only"))
            .await
            .expect("first ok");
        let err = host
            .spawn_agent(SpawnOpts::new("second"))
            .await
            .expect_err("budget");
        assert_eq!(err.code(), ErrorCode::HostBudget);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn workflow_mode_plan_and_parallel_via_facade() {
        let sampler = Arc::new(MockSampler::new());
        // Key concurrent workers; plan stays FIFO (runs before parallel barrier).
        sampler.push_text("plan-out");
        sampler.map_user_text("w0", "w0-out");
        sampler.map_user_text("w1", "w1-out");
        let host: Arc<dyn SessionHost> = Arc::new(InProcessHost::new(sampler, Vec::new()));

        let script = r#"
            let meta = #{ name: "it-fanout", description: "integration" };
            let plan = agent("plan", #{ label: "planner" });
            let ws = parallel([
                #{ prompt: "w0", label: "w0" },
                #{ prompt: "w1", label: "w1" },
            ]);
            complete(#{ plan: plan, workers: ws });
        "#;
        let (tx, _rx) = mpsc::unbounded_channel();
        let outcome = run_workflow_on_host(
            host,
            WorkflowRunParams {
                script: script.into(),
                args: serde_json::json!({}),
                journal: Journal::new(None),
                host_tx: tx,
                cancel: CancellationToken::new(),
                max_ops: WorkflowRunParams::DEFAULT_MAX_OPS,
            },
            Some(16),
        )
        .await
        .expect("workflow");

        let WorkflowOutcome::Completed { result } = outcome else {
            unreachable!("expected completed outcome");
        };
        assert_eq!(
            result.pointer("/plan/output").and_then(|v| v.as_str()),
            Some("plan-out")
        );
        let workers = result
            .get("workers")
            .and_then(|v| v.as_array())
            .expect("workers");
        assert_eq!(workers.len(), 2);
        assert_eq!(
            workers[0].get("output").and_then(|v| v.as_str()),
            Some("w0-out")
        );
        assert_eq!(
            workers[1].get("output").and_then(|v| v.as_str()),
            Some("w1-out")
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn workflow_resume_skips_completed_host_calls() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        use async_trait::async_trait;
        use machi::{LlmSampler, MachiError, SampleRequest, SampleResponse};

        struct CountMock {
            inner: MockSampler,
            calls: AtomicUsize,
        }

        #[async_trait]
        impl LlmSampler for CountMock {
            async fn sample(&self, request: SampleRequest) -> Result<SampleResponse, MachiError> {
                self.calls.fetch_add(1, Ordering::SeqCst);
                self.inner.sample(request).await
            }
        }

        let dir = tempfile::tempdir().expect("tmp");
        let path = dir.path().join("j.jsonl");
        let sampler = Arc::new(CountMock {
            inner: MockSampler::new(),
            calls: AtomicUsize::new(0),
        });
        sampler.inner.push_text("first");
        sampler.inner.push_text("second");

        let host: Arc<dyn SessionHost> = Arc::new(InProcessHost::new(sampler.clone(), Vec::new()));
        let script = r#"
            let meta = #{ name: "it-resume", description: "integration" };
            let a = agent("a");
            let b = agent("b");
            complete(#{ a: a, b: b });
        "#;

        let (tx, _rx) = mpsc::unbounded_channel();
        let o1 = run_workflow_on_host(
            Arc::clone(&host),
            WorkflowRunParams {
                script: script.into(),
                args: serde_json::json!({}),
                journal: Journal::new(Some(path.clone())),
                host_tx: tx,
                cancel: CancellationToken::new(),
                max_ops: WorkflowRunParams::DEFAULT_MAX_OPS,
            },
            Some(8),
        )
        .await
        .expect("run1");
        assert!(matches!(o1, WorkflowOutcome::Completed { .. }));
        let after_first = sampler.calls.load(Ordering::SeqCst);
        assert_eq!(after_first, 2, "first run samples twice");

        let journal = Journal::load(path).expect("load");
        assert_eq!(journal.len(), 2);
        let (tx2, _rx2) = mpsc::unbounded_channel();
        let o2 = run_workflow_on_host(
            host,
            WorkflowRunParams {
                script: script.into(),
                args: serde_json::json!({}),
                journal,
                host_tx: tx2,
                cancel: CancellationToken::new(),
                max_ops: WorkflowRunParams::DEFAULT_MAX_OPS,
            },
            Some(8),
        )
        .await
        .expect("run2");
        assert!(matches!(o2, WorkflowOutcome::Completed { .. }));
        assert_eq!(
            sampler.calls.load(Ordering::SeqCst),
            after_first,
            "resume must not re-sample"
        );
    }
}
