use std::{fs, path::Path};

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn read(relative: &str) -> String {
    fs::read_to_string(root().join(relative))
        .unwrap_or_else(|error| panic!("cannot read {relative}: {error}"))
}

/// Tracing: TC-014
/// TC-014.
/// Implements: NFR-004.
/// NFR-004-AC-1.
#[test]
fn tc_014_baseline_is_dual_licensed_manual_only_and_unpublished() {
    let cargo = read("Cargo.toml");
    assert!(cargo.contains("license = \"MIT OR Apache-2.0\""));
    assert!(cargo.contains("publish = false"));
    assert!(root().join("LICENSE-MIT").is_file());
    assert!(root().join("LICENSE-APACHE").is_file());

    let workflow = read(".github/workflows/ci.yml");
    assert_eq!(workflow.matches("workflow_dispatch:").count(), 1);
    for automatic_trigger in ["pull_request:", "push:", "schedule:"] {
        assert!(
            !workflow.contains(automatic_trigger),
            "automatic CI trigger must stay absent: {automatic_trigger}"
        );
    }

    assert_eq!(read(".github/CODEOWNERS").trim(), "* @kreneskyp");
}

/// Tracing: TC-020
/// TC-020.
/// Implements: StR-003.
/// Implements: NFR-004.
/// NFR-004-AC-2.
#[test]
fn tc_020_assurance_packet_names_boundaries_evidence_failures_and_owner() {
    let artifacts = [
        (
            "spec/assurance/AP-001-contract-ir-v01.md",
            "type: AssuranceProfile",
            [
                "## Decision Boundary",
                "## Evidence Policy",
                "owner: kreneskyp",
            ],
        ),
        (
            "spec/assurance/AD-001-contract-ir-architecture.md",
            "type: ArchitectureDescription",
            ["## System Boundary", "## Risks", "owner: kreneskyp"],
        ),
        (
            "spec/assurance/CAC-001-semantic-validator.md",
            "type: ComponentAssuranceContract",
            [
                "## Component Boundary",
                "## Failure Handling",
                "owner: kreneskyp",
            ],
        ),
        (
            "spec/assurance/MP-001-contract-conformance.md",
            "type: MeasurementPlan",
            [
                "## Decision Use",
                "## Collection Procedure",
                "owner: kreneskyp",
            ],
        ),
        (
            "spec/assurance/AA-001-contract-ir-v01.md",
            "type: AssuranceArgument",
            ["## Claim", "## Challenges", "owner: kreneskyp"],
        ),
    ];

    for (path, artifact_type, required) in artifacts {
        let document = read(path);
        assert!(
            document.contains(artifact_type),
            "{path} has the wrong type"
        );
        for marker in required {
            assert!(document.contains(marker), "{path} is missing {marker}");
        }
    }

    let argument = read("spec/assurance/AA-001-contract-ir-v01.md");
    assert!(argument.contains("status: active"));
    assert!(argument.contains("remains open"));
}

/// Tracing: TC-021
/// TC-021.
/// Implements: NFR-004.
/// NFR-004-AC-3.
#[test]
fn tc_021_plan_and_review_preserve_the_spec_first_dependency_gate() {
    let plan = read("plan/PLAN-002-contract-ir-v01/plan.md");
    for edge in [
        "TASK-005 issue #5",
        "-> TASK-006 issue #6",
        "-> TASK-007 issue #8",
        "-> TASK-008 issue #9",
        "-> TASK-009 issue #10",
        "-> epic #11",
    ] {
        assert!(plan.contains(edge), "PLAN-002 is missing DAG edge {edge}");
    }

    for completed in [
        "TASK-005-foundation.md",
        "TASK-006-identities.md",
        "TASK-007-expressions.md",
        "TASK-008-canonicalization.md",
        "TASK-009-conformance.md",
    ] {
        let path = format!("plan/PLAN-002-contract-ir-v01/{completed}");
        assert!(
            read(&path).contains("status: done"),
            "{completed} is not done"
        );
    }
    let current_review = read("reviews/SR-012-canonicalization-spec-review.md");
    assert!(current_review.contains("FND-085"));
    assert!(current_review.contains("FND-096"));
    assert!(current_review.contains("`No actionable findings.`"));

    let review = read("reviews/REV-002-contract-ir-foundation.md");
    for dimension in [
        "| Dependency |",
        "| Risk and complexity |",
        "| Evidence |",
        "| Integrity |",
        "| Scope |",
        "| Failure domains |",
        "| Architecture |",
        "| Authority |",
    ] {
        assert!(review.contains(dimension), "review is missing {dimension}");
    }
}
