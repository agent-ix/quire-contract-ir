use std::{collections::BTreeSet, fs, path::Path};

use sha2::{Digest, Sha256};

const POLICY: &str = include_str!("../spec/program/PGM-01-governance.md");
const RECONCILIATION: &str = include_str!("../docs/shared-assurance-governance.md");
const README: &str = include_str!("../README.md");
const CONTRIBUTING: &str = include_str!("../CONTRIBUTING.md");
const CARGO_TOML: &str = include_str!("../Cargo.toml");
const HISTORICAL_LOCK: &str = include_str!("fixtures/historical-pgm01-files.sha256");

fn normalized(document: &str) -> String {
    document.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn markdown_table_after<'a>(document: &'a str, marker: &str) -> Vec<Vec<&'a str>> {
    document
        .split_once(marker)
        .unwrap_or_else(|| panic!("missing table marker: {marker}"))
        .1
        .lines()
        .skip_while(|line| !line.starts_with('|'))
        .take_while(|line| line.starts_with('|'))
        .skip(2)
        .map(|line| {
            line.trim_matches('|')
                .split('|')
                .map(str::trim)
                .collect::<Vec<_>>()
        })
        .collect()
}

fn inventory_after(document: &str, marker: &str) -> BTreeSet<String> {
    document
        .split_once(marker)
        .unwrap_or_else(|| panic!("missing inventory marker: {marker}"))
        .1
        .lines()
        .skip_while(|line| line.is_empty())
        .take_while(|line| line.starts_with("- `"))
        .map(|line| {
            line.trim_start_matches("- `")
                .trim_end_matches('`')
                .to_owned()
        })
        .collect()
}

/// Tracing: TC-023.
/// FR-008-AC-4.
/// FR-021-AC-1.
#[test]
fn tc_023_assigns_each_shared_responsibility_exactly_once() {
    let rows = markdown_table_after(POLICY, "The shared responsibility assignment is exact:");
    let actual = rows.iter().map(|row| (row[0], row[1])).collect::<Vec<_>>();
    let expected = vec![
        (
            "Static verification definitions, obligations, relations, and locators",
            "Quire",
        ),
        (
            "Verification execution",
            "Contract or temporal domain producer",
        ),
        (
            "Structured domain result and diagnostic",
            "Originating contract or temporal producer",
        ),
        (
            "Evidence intake, retained bytes, integrity, audit, and report views",
            "Quoin",
        ),
        (
            "Human approval, rejection, revision, and workflow state",
            "ix-flow",
        ),
        (
            "Cross-repository campaign policy and release order",
            "PGM-01",
        ),
    ];
    assert_eq!(actual, expected);
}

/// Tracing: TC-024.
/// FR-009-AC-5.
#[test]
fn tc_024_locks_historical_bytes_and_requires_lossy_read_only_mapping() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut locked = BTreeSet::new();
    for line in HISTORICAL_LOCK.lines().filter(|line| !line.is_empty()) {
        let (expected, relative) = line
            .split_once("  ")
            .expect("historical lock rows use sha256sum format");
        assert!(
            locked.insert(relative),
            "duplicate historical path: {relative}"
        );
        let bytes = fs::read(root.join(relative)).unwrap_or_else(|error| {
            panic!("locked historical input {relative} is unreadable: {error}")
        });
        let actual = format!("{:x}", Sha256::digest(bytes));
        assert_eq!(actual, expected, "historical bytes changed: {relative}");
    }
    assert_eq!(locked.len(), 46, "the accepted historical set changed");

    let policy = normalized(POLICY);
    let reconciliation = normalized(RECONCILIATION);
    assert!(policy.contains("explicit read-only compatibility mapping"));
    assert!(policy.contains("never rewrite them into a newer schema"));
    assert!(policy.contains("treat an absent historical field as known"));
    assert!(reconciliation.contains("explicit read-only, lossy mapping"));
    assert!(reconciliation.contains("does not mutate source bytes, synthesize missing"));
}

/// Tracing: TC-025.
/// FR-009-AC-6.
/// FR-021-AC-2.
#[test]
fn tc_025_preserves_domain_ownership_nonexecution_and_runtime_independence() {
    let policy = normalized(POLICY);
    for phrase in [
        "Domain repositories still own their producers and structured results",
        "Quire and Quoin are explicitly non-executing",
        "Published runtime and domain crates shall not acquire runtime dependencies on Quire or Quoin",
        "Quoin owns retention, integrity checks, audit, and report views",
        "ix-flow owns any human decision",
    ] {
        assert!(policy.contains(phrase), "missing ownership rule: {phrase}");
    }

    let dependencies = CARGO_TOML
        .split_once("[dependencies]")
        .expect("Cargo manifest declares runtime dependencies")
        .1
        .split("\n[")
        .next()
        .unwrap()
        .to_ascii_lowercase();
    for line in dependencies.lines().filter(|line| !line.trim().is_empty()) {
        let (key, value) = line
            .split_once('=')
            .expect("runtime dependency uses key/value syntax");
        assert!(!matches!(key.trim(), "quire" | "quire-rs" | "quoin"));
        for package in [
            "package = \"quire\"",
            "package = \"quire-rs\"",
            "package = \"quoin\"",
        ] {
            assert!(
                !value.contains(package),
                "forbidden runtime dependency: {line}"
            );
        }
    }
}

/// Tracing: TC-027.
/// FR-021-AC-4.
#[test]
fn tc_027_separates_preserved_cases_from_rejected_architecture() {
    let accepted_adversarial = inventory_after(RECONCILIATION, "Accepted adversarial cases:");
    let accepted_domain = inventory_after(RECONCILIATION, "Accepted domain cases:");
    let rejected = inventory_after(RECONCILIATION, "Rejected architecture:");

    assert_eq!(accepted_adversarial.len(), 9);
    assert_eq!(accepted_domain.len(), 4);
    assert_eq!(rejected.len(), 7);
    assert!(accepted_adversarial.is_disjoint(&rejected));
    assert!(accepted_domain.is_disjoint(&rejected));
    for required in [
        "checksum-reseal-tamper",
        "failed-versus-unavailable",
        "mutation-must-turn-domain-oracle-red",
    ] {
        assert!(accepted_adversarial.contains(required));
    }
    for required in [
        "contract-validation-and-canonicalization",
        "property-proof-and-vacuity-results",
        "smt-nonconclusive-results",
        "temporal-parse-rewrite-and-evaluation-results",
    ] {
        assert!(accepted_domain.contains(required));
    }
    for required in [
        "generic-command-executor",
        "central-execution-profile",
        "aggregate-overall-verdict",
        "evidence-authority-index",
        "repository-adoption-command",
        "parallel-result-family",
        "retention-layout-or-store",
    ] {
        assert!(rejected.contains(required));
    }
}

/// Tracing: TC-028.
/// FR-021-AC-5.
#[test]
fn tc_028_removes_conflicting_campaign_prescriptions() {
    for (name, document) in [("README", README), ("CONTRIBUTING", CONTRIBUTING)] {
        for obsolete in [
            "common PGM-01 evidence envelope is used",
            "Quoin and Quire may integrate later",
            "This program does not refactor Quoin, Quire-rs",
        ] {
            assert!(
                !document.contains(obsolete),
                "{name} retains obsolete text: {obsolete}"
            );
        }
        assert!(
            document.contains("PGM-01"),
            "{name} must point to the normative policy"
        );
    }
    assert!(README.contains("Quire and Quoin are non-executing"));
    assert!(CONTRIBUTING.contains("not runtime dependencies or a shared producer runner"));
    assert!(RECONCILIATION.contains("The eight migration issues are not part of this gate"));
}
