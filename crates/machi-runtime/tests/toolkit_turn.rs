//! Integration: `ReadFile` + `WriteFile` through `TurnRuntime` with mock sampler.
#![allow(
    unused_crate_dependencies,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::tests_outside_test_module,
    reason = "integration tests"
)]

use std::sync::Arc;

use machi_agent::AgentBuilder;
use machi_llm::MockSampler;
use machi_runtime::{ConversationState, TurnInput, TurnOptions, TurnRuntime, VecConversationState};
use machi_toolkit::{ReadFileTool, WriteFileTool};
use machi_types::{Message, ToolCall, ToolCallId};
use serde_json::json;
use tempfile::tempdir;

#[tokio::test]
async fn write_then_read_via_tools() {
    let dir = tempdir().expect("tmp");
    let tools: Vec<Arc<dyn machi_tools::DynTool>> = vec![
        Arc::new(WriteFileTool::with_jail(dir.path())),
        Arc::new(ReadFileTool::with_jail(dir.path())),
    ];

    let write_id = ToolCallId::new("w1").expect("id");
    let read_id = ToolCallId::new("r1").expect("id");

    let sampler = Arc::new(MockSampler::new());
    sampler.push_tools(Message::assistant_tools(vec![ToolCall {
        id: write_id,
        name: "write_file".into(),
        arguments: json!({"path": "note.txt", "content": "hello-toolkit"}),
    }]));
    sampler.push_tools(Message::assistant_tools(vec![ToolCall {
        id: read_id,
        name: "read_file".into(),
        arguments: json!({"path": "note.txt"}),
    }]));
    sampler.push_text("done");

    let agent = AgentBuilder::named("coder")
        .model("mock")
        .tools(tools)
        .build()
        .expect("agent");

    let mut state = VecConversationState::new();
    let out = TurnRuntime::new()
        .run(
            &agent,
            sampler.as_ref(),
            &mut state,
            TurnInput::Text("write and read".into()),
            TurnOptions {
                cwd: Some(dir.path().to_path_buf()),
                max_steps: Some(8),
                ..TurnOptions::default()
            },
        )
        .await
        .expect("turn");

    assert_eq!(out.output_text, "done");
    let body = std::fs::read_to_string(dir.path().join("note.txt")).expect("file");
    assert_eq!(body, "hello-toolkit");
    let joined: String = state
        .messages()
        .iter()
        .map(Message::text)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(joined.contains("hello-toolkit"), "{joined}");
}
