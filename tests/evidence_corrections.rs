use std::{fs, path::Path};

use serde_json::Value;

const CORRECTION_PATH: &str = "evidence/corrections/COR-001-pr12-code-review.json";

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// Tracing: TC-022
/// TC-022.
/// Implements: FR-009.
/// Implements: NFR-004.
/// NFR-004-AC-4.
#[test]
fn tc_022_corrects_the_pr12_review_claim_without_rewriting_evidence() {
    let bytes = fs::read(root().join(CORRECTION_PATH)).expect("correction record must exist");
    let record: Value = serde_json::from_slice(&bytes).expect("correction record must be JSON");

    assert_eq!(record["schemaVersion"], "quire.evidence-correction/v1");
    assert_eq!(record["correctedStatus"], "inconclusive");
    assert!(record["immutability"]
        .as_str()
        .unwrap()
        .contains("byte-for-byte unchanged"));
    assert!(record["decisionEffect"]
        .as_str()
        .unwrap()
        .contains("already merged"));

    let affected = record["affectedClaims"].as_array().unwrap();
    assert_eq!(affected.len(), 1);
    assert_eq!(affected[0]["record"], "pgm-01-568bd05");

    let findings = record["findingRefs"].as_array().unwrap();
    for expected in ["5062152784", "5062178118"] {
        assert!(findings
            .iter()
            .any(|reference| reference.as_str().unwrap().contains(expected)));
    }
}
