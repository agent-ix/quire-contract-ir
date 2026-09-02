use std::{fs, path::Path};

const POLICY: &str = include_str!("../spec/program/PGM-01-governance.md");

fn normalized_policy() -> String {
    POLICY.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Tracing: TC-001
/// TC-001.
/// FR-001-AC-2.
/// FR-002-AC-1.
#[test]
fn tc_001_defines_compatibility_and_exact_pins() {
    let policy = normalized_policy();
    for phrase in [
        "reject an unknown major version",
        "shall not guess",
        "exact source revisions",
        "schema identity and SHA-256 digest",
    ] {
        assert!(policy.contains(phrase), "missing policy phrase: {phrase}");
    }
}

/// Tracing: TC-002
/// TC-002.
/// FR-003-AC-1.
/// FR-007-AC-1.
#[test]
fn tc_002_classifies_all_repositories_and_orders_tags() {
    let policy = normalized_policy();
    for repository in [
        "quire-contract-ir",
        "quire-contract-runtime",
        "quire-contract-codegen",
        "quire-analyze",
        "tl-syntax",
        "tl-parse",
        "tl-rewrite",
        "tl-mltl",
    ] {
        assert!(
            policy.contains(repository),
            "missing repository: {repository}"
        );
    }
    for classification in [
        "linked runtime",
        "direct development tool",
        "analysis/evidence tool",
        "external engine adapter",
    ] {
        assert!(
            policy.contains(classification),
            "missing class: {classification}"
        );
    }
    assert!(policy.contains("are independent initial source-tag roots"));
    assert!(policy.contains("quire-contract-codegen` follows the IR and runtime"));
    assert!(policy.contains("quire-analyze` follows the IR"));
    assert!(policy.contains("tl-rewrite` follows `tl-syntax` plus retained evaluator evidence"));
}

/// Tracing: TC-003
/// TC-003.
/// FR-004-AC-1.
/// FR-005-AC-1.
/// FR-010-AC-1.
#[test]
fn tc_003_defines_license_clean_room_agent_and_qualification_boundaries() {
    let policy = normalized_policy();
    for phrase in [
        "MIT OR Apache-2.0",
        "shall not be copied, translated, mechanically transformed",
        "agent-assisted",
        "does **not** validate or accredit",
    ] {
        assert!(policy.contains(phrase), "missing policy phrase: {phrase}");
    }
}

/// Tracing: TC-004
/// TC-004.
/// FR-006-AC-1.
/// FR-009-AC-2.
#[test]
fn tc_004_names_the_enforced_human_decision_owner() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let codeowners = fs::read_to_string(root.join(".github/CODEOWNERS")).unwrap();
    let contributing = fs::read_to_string(root.join("CONTRIBUTING.md")).unwrap();
    assert_eq!(codeowners.trim(), "* @kreneskyp");
    assert!(normalized_policy().contains("Only that human may record sufficiency"));
    assert!(contributing.contains("may not approve its own"));
}
