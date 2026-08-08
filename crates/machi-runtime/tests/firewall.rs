//! Dependency firewall: workflow must not depend on llm/http.
#![allow(
    unused_crate_dependencies,
    clippy::expect_used,
    clippy::tests_outside_test_module,
    reason = "integration asserts; test binary links crate deps"
)]

use std::fs;
use std::path::PathBuf;

#[test]
fn workflow_cargo_toml_has_no_llm_or_http() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workflow = root.join("..").join("machi-workflow").join("Cargo.toml");
    let text = fs::read_to_string(&workflow).expect("read workflow Cargo.toml");
    for forbidden in ["machi-llm", "machi-runtime", "reqwest", "hyper", "ureq"] {
        assert!(
            !text.contains(forbidden),
            "machi-workflow must not depend on {forbidden}"
        );
    }
    assert!(text.contains("rhai"), "machi-workflow should use rhai");
}

#[test]
fn types_cargo_toml_stays_pure() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let types = root.join("..").join("machi-types").join("Cargo.toml");
    let text = fs::read_to_string(&types).expect("read types Cargo.toml");
    for forbidden in ["reqwest", "machi-llm", "machi-runtime", "tokio"] {
        assert!(
            !text.contains(forbidden),
            "machi-types must not depend on {forbidden}"
        );
    }
}
