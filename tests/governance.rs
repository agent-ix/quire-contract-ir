use std::{fs, path::Path, process::Command};

const POLICY: &str = include_str!("../spec/program/PGM-01-governance.md");

fn fixture_result(path: &str) -> std::process::Output {
    Command::new("python3")
        .arg("scripts/validate_governance.py")
        .arg("--fixture")
        .arg(path)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("PGM-01 validator must execute")
}

#[test]
fn pgm01_t01_defines_compatibility_and_exact_pins() {
    for phrase in [
        "reject an unknown major version",
        "shall not guess",
        "exact source revisions",
        "schema identity and SHA-256 digest",
    ] {
        assert!(POLICY.contains(phrase), "missing policy phrase: {phrase}");
    }
}

#[test]
fn pgm01_t02_classifies_all_repositories_and_orders_tags() {
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
            POLICY.contains(repository),
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
            POLICY.contains(classification),
            "missing class: {classification}"
        );
    }
    assert!(POLICY.contains("are independent\ninitial source-tag roots"));
    assert!(POLICY.contains("quire-contract-codegen`\nfollows the IR and runtime"));
    assert!(POLICY.contains("quire-analyze` follows the IR"));
    assert!(POLICY.contains("tl-rewrite` follows `tl-syntax` plus retained\nevaluator evidence"));
}

#[test]
fn pgm01_t03_defines_license_clean_room_agent_and_qualification_boundaries() {
    for phrase in [
        "MIT OR Apache-2.0",
        "shall not be copied, translated, mechanically transformed",
        "agent-assisted",
        "does **not** validate or accredit",
    ] {
        assert!(POLICY.contains(phrase), "missing policy phrase: {phrase}");
    }
}

#[test]
fn pgm01_t04_names_the_enforced_human_decision_owner() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let codeowners = fs::read_to_string(root.join(".github/CODEOWNERS")).unwrap();
    let contributing = fs::read_to_string(root.join("CONTRIBUTING.md")).unwrap();
    assert_eq!(codeowners.trim(), "* @kreneskyp");
    assert!(POLICY.contains("Only that human may record sufficiency"));
    assert!(contributing.contains("may not approve its own"));
}

#[test]
fn pgm01_t05_accepts_generated_artifact_identity() {
    let result = fixture_result("corpus/governance/valid/generated-oracle.json");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stdout)
    );
}

#[test]
fn pgm01_t06_accepts_external_engine_and_non_conclusive_result() {
    let result = fixture_result("corpus/governance/valid/solver-analysis.json");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stdout)
    );
}

#[test]
fn pgm01_t07_rejects_missing_backend_identity() {
    let result = fixture_result("corpus/governance/invalid/missing-backend.json");
    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(!result.status.success());
    assert!(stdout.contains("MISSING_BACKEND"), "{stdout}");
}

#[test]
fn pgm01_t08_rejects_invalid_output_and_unknown_schema_identity() {
    let digest = fixture_result("corpus/governance/invalid/invalid-output-digest.json");
    let schema = fixture_result("corpus/governance/invalid/unsupported-schema.json");
    assert!(!digest.status.success());
    assert!(!schema.status.success());
    assert!(String::from_utf8_lossy(&digest.stdout).contains("INVALID_DIGEST"));
    assert!(String::from_utf8_lossy(&schema.stdout).contains("UNSUPPORTED_SCHEMA"));
}

#[test]
fn pgm01_t09_rejects_missing_producer_tool_identity() {
    let result = fixture_result("corpus/governance/invalid/missing-producer.json");
    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(!result.status.success());
    assert!(stdout.contains("MISSING_PRODUCER"), "{stdout}");
}

#[test]
fn pgm01_t10_rejects_missing_input_identities() {
    let result = fixture_result("corpus/governance/invalid/missing-inputs.json");
    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(!result.status.success());
    assert!(stdout.contains("MISSING_INPUTS"), "{stdout}");
}

#[test]
fn pgm01_t11_rejects_missing_nested_schema_identity() {
    let result = fixture_result("corpus/governance/invalid/missing-schema-identity.json");
    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(!result.status.success());
    assert!(stdout.contains("MISSING_SCHEMA_IDENTITY"), "{stdout}");
}

#[test]
fn pgm01_t12_rejects_missing_output_identities() {
    let result = fixture_result("corpus/governance/invalid/missing-outputs.json");
    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(!result.status.success());
    assert!(stdout.contains("MISSING_OUTPUTS"), "{stdout}");
}
