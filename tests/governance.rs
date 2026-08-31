use std::{ffi::OsString, fs, path::Path, process::Command};

use serde_json::Value;

const POLICY: &str = include_str!("../spec/program/PGM-01-governance.md");

fn normalized_policy() -> String {
    POLICY.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn governance_python() -> OsString {
    std::env::var_os("QUIRE_GOVERNANCE_PYTHON").unwrap_or_else(|| OsString::from("python3"))
}

fn fixture_result(path: &str) -> (std::process::ExitStatus, Value) {
    let python = governance_python();
    let output = Command::new(&python)
        .arg("scripts/validate_governance.py")
        .arg("--fixture")
        .arg(path)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("the declared PGM-01 Python lane must execute");
    let report: Value = serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "validator stdout must be JSON ({error}); interpreter={python:?}; status={}; stderr={}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )
    });
    (output.status, report)
}

fn mutation_report() -> Value {
    let output = Command::new(governance_python())
        .arg("scripts/validate_governance.py")
        .arg("--mutation-probes")
        .arg("--json")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("the declared PGM-01 Python lane must execute");
    assert!(
        output.status.success(),
        "mutation validator failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("mutation stdout must be JSON")
}

fn assert_valid(path: &str) -> Value {
    let (status, report) = fixture_result(path);
    assert!(status.success(), "{report}");
    assert_eq!(report["valid"], true);
    assert_eq!(report["errors"], serde_json::json!([]));
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

fn assert_invalid_code(path: &str, expected: &str) {
    let (status, report) = fixture_result(path);
    assert!(!status.success(), "{report}");
    assert_eq!(report["valid"], false);
    let codes = report["errors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|error| error["code"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert!(
        codes.contains(&expected),
        "expected {expected}, got {codes:?}"
    );
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

/// Tracing: TC-005
/// TC-005.
/// FR-008-AC-1.
/// FR-008-AC-2.
#[test]
fn tc_005_accepts_generated_artifact_identity() {
    let fixture = assert_valid("corpus/governance/valid/generated-oracle.json");
    assert_eq!(fixture["backend"]["kind"], "none");
    let report = mutation_report();
    assert_eq!(report["detected"], true);
    let probes = report["probes"].as_array().unwrap();
    assert_eq!(probes.len(), 7);
    for name in [
        "invalid-recorded-at",
        "invalid-repository-uri",
        "invalid-artifact-uri",
    ] {
        assert!(probes
            .iter()
            .any(|probe| { probe["name"] == name && probe["detected"] == true }));
    }
}

/// Tracing: TC-006
/// TC-006.
/// FR-009-AC-1.
#[test]
fn tc_006_accepts_external_engine_and_retains_inconclusive_result() {
    let fixture = assert_valid("corpus/governance/valid/solver-analysis.json");
    assert_eq!(fixture["backend"]["kind"], "engine");
    assert_eq!(fixture["result"]["status"], "inconclusive");
}

/// Tracing: TC-007
/// TC-007.
/// FR-008-AC-3.
#[test]
fn tc_007_rejects_missing_backend_identity() {
    assert_invalid_code(
        "corpus/governance/invalid/missing-backend.json",
        "MISSING_BACKEND",
    );
    assert_invalid_code(
        "corpus/governance/invalid/missing-backend-configuration-digest.json",
        "SCHEMA_VIOLATION",
    );
}

/// Tracing: TC-008
/// TC-008.
/// FR-001-AC-1.
#[test]
fn tc_008_rejects_invalid_digest_and_unknown_schema() {
    assert_invalid_code(
        "corpus/governance/invalid/invalid-output-digest.json",
        "INVALID_DIGEST",
    );
    assert_invalid_code(
        "corpus/governance/invalid/unsupported-schema.json",
        "UNSUPPORTED_SCHEMA",
    );
}

/// Tracing: TC-009
/// TC-009.
#[test]
fn tc_009_rejects_missing_producer_and_provenance_identity() {
    assert_invalid_code(
        "corpus/governance/invalid/missing-producer.json",
        "MISSING_PRODUCER",
    );
    assert_invalid_code(
        "corpus/governance/invalid/missing-producer-executable-digest.json",
        "SCHEMA_VIOLATION",
    );
    assert_invalid_code(
        "corpus/governance/invalid/missing-contribution-method.json",
        "SCHEMA_VIOLATION",
    );
}

/// Tracing: TC-010
/// TC-010.
#[test]
fn tc_010_rejects_missing_input_identities() {
    assert_invalid_code(
        "corpus/governance/invalid/missing-inputs.json",
        "MISSING_INPUTS",
    );
}

/// Tracing: TC-011
/// TC-011.
#[test]
fn tc_011_rejects_missing_nested_schema_identity() {
    assert_invalid_code(
        "corpus/governance/invalid/missing-schema-identity.json",
        "MISSING_SCHEMA_IDENTITY",
    );
}

/// Tracing: TC-012
/// TC-012.
#[test]
fn tc_012_rejects_missing_output_identities() {
    assert_invalid_code(
        "corpus/governance/invalid/missing-outputs.json",
        "MISSING_OUTPUTS",
    );
    assert_invalid_code(
        "corpus/governance/invalid/missing-output-content-digest.json",
        "SCHEMA_VIOLATION",
    );
}
