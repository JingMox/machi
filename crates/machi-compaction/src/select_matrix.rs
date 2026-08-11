// W6: select_compaction_range keep_tail matrix with tool-pair invariant.
#[cfg(test)]
#[allow(clippy::expect_used, clippy::missing_assert_message, reason = "matrix tests")]
mod select_matrix {
    use machi_types::{Message, ToolCall, ToolCallId};
    use serde_json::json;
    use super::{apply_range, select_compaction_range, tool_pair_invariant_holds};

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
    fn keep_tail_1_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 1) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_2_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 2) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_3_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 3) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_4_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 4) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_5_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 5) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_6_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 6) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_7_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 7) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_8_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 8) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_9_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 9) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_10_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 10) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_11_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 11) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_12_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 12) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_13_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 13) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_14_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 14) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_15_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 15) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_16_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 16) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_17_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 17) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_18_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 18) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_19_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 19) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_20_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 20) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_21_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 21) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_22_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 22) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_23_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 23) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_24_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 24) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_25_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 25) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_26_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 26) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_27_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 27) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_28_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 28) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_29_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 29) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_30_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 30) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_31_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 31) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_32_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 32) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_33_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 33) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_34_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 34) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_35_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 35) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_36_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 36) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_37_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 37) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_38_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 38) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_39_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 39) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_40_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 40) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_41_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 41) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_42_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 42) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_43_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 43) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_44_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 44) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_45_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 45) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_46_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 46) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_47_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 47) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_48_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 48) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_49_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 49) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_50_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 50) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_51_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 51) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_52_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 52) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_53_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 53) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_54_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 54) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_55_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 55) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_56_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 56) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_57_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 57) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_58_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 58) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_59_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 59) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_60_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 60) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_61_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 61) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_62_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 62) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_63_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 63) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }

    #[test]
    fn keep_tail_64_preserves_invariant() {
        let msgs = conversation();
        let n = msgs.len();
        if let Some(range) = select_compaction_range(&msgs, 64) {
            let out = apply_range(msgs, range, None);
            assert!(tool_pair_invariant_holds(&out));
            assert!(out.len() <= n);
        } else {
            // no safe split is a valid outcome (no-op compaction)
        }
    }
}
