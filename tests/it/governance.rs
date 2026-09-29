use std::{fs, path::Path};

const POLICY: &str = include_str!("../../spec/program/PGM-01-governance.md");

fn normalized_policy() -> String {
    POLICY.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Tracing: TC-001
/// TC-001.
/// FR-001-AC-2.
#[test]
fn tc_001_defines_schema_compatibility() {
    let policy = normalized_policy();
    for phrase in ["reject an unknown major version", "shall not guess"] {
        assert!(policy.contains(phrase), "missing policy phrase: {phrase}");
    }
}

/// Tracing: TC-004
/// TC-004.
/// FR-006-AC-1.
#[test]
fn tc_004_names_the_enforced_human_decision_owner() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let codeowners = fs::read_to_string(root.join(".github/CODEOWNERS")).unwrap();
    let contributing = fs::read_to_string(root.join("CONTRIBUTING.md")).unwrap();
    assert_eq!(codeowners.trim(), "* @kreneskyp");
    assert!(normalized_policy().contains("Only that human may record sufficiency"));
    assert!(contributing.contains("may not approve its own"));
}
