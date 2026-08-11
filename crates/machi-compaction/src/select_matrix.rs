// W6: select_compaction_range keep_tail matrix with tool-pair invariant.
#[cfg(test)]
#[allow(
    clippy::expect_used,
    clippy::missing_assert_message,
    clippy::panic,
    reason = "matrix tests"
)]
mod select_matrix {
    use machi_types::{Message, Role, ToolCall, ToolCallId};
    use serde_json::json;

    use super::{
        apply_range, select_compaction_range, tool_pair_invariant_holds,
    };

    fn conversation() -> Vec<Message> {
        let mut msgs = vec![Message::system("s")];
        for i in 0..20 {
            let id = ToolCallId::new(format!("c{i}")).expect("id");
            msgs.push(Message::user(format!("u{i}")));
            msgs.push(Message::assistant_tools(vec![ToolCall {
                id: id.clone(),
                name: "x".into(),
                arguments: json!({}),
            }]));
            msgs.push(Message::tool_result(id, "x", "ok"));
        }
        msgs
    }

    #[test]
    fn keep_tail_1_with_system_keeps_system_only() {
        let msgs = conversation();
        let range = select_compaction_range(&msgs, 1).expect("must compact to system");
        let out = apply_range(msgs, range, None);
        assert!(tool_pair_invariant_holds(&out));
        assert_eq!(out.len(), 1);
        assert_eq!(out.first().map(|m| m.role), Some(Role::System));
    }

    #[test]
    fn every_keep_tail_that_compacts_is_safe() {
        let msgs = conversation();
        let n = msgs.len();
        // Compaction only when keep_tail < n.
        for keep in 1..n {
            let Some(range) = select_compaction_range(&msgs, keep) else {
                panic!("expected Some range for keep_tail={keep} on {n}-msg fixture");
            };
            assert!(range.split_idx > 0);
            assert!(range.split_idx <= n);
            let out = apply_range(msgs.clone(), range, None);
            assert!(
                tool_pair_invariant_holds(&out),
                "keep={keep} split={} out={out:?}",
                range.split_idx
            );
            // System preserved when present.
            assert_eq!(out.first().map(|m| m.role), Some(Role::System));
            // Never start kept tail mid-tool-result.
            if let Some(first_kept) = out.get(1) {
                assert_ne!(first_kept.role, Role::Tool, "keep={keep}");
            }
        }
    }

    #[test]
    fn no_compact_when_already_within_tail() {
        let msgs = conversation();
        let n = msgs.len();
        assert!(select_compaction_range(&msgs, n).is_none());
        assert!(select_compaction_range(&msgs, n + 5).is_none());
        assert!(select_compaction_range(&msgs, 0).is_none());
    }

    #[test]
    fn snap_end_of_list_is_safe() {
        use super::{is_safe_split, snap_split_forward};
        let msgs = conversation();
        let n = msgs.len();
        assert!(is_safe_split(&msgs, n));
        assert_eq!(snap_split_forward(&msgs, n), Some(n));
    }
}
