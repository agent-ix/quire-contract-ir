// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Published QSpec execution point preimages against independent canonical
//! derivation and IR's production application candidate.

use super::{fixtures_dir, proposal_dir_of, read_json};
use ix_trace_rs::trace;
use quire_canonical::Limits;
use quire_contract_model::{
    derive_application_node_id, ApplicationNodeDraft, CheckedNodeId, CheckedSemanticNodeV2,
};
use serde_json::{json, Value};

const VECTOR_FILE: &str = "node-identity-vectors.json";
const BYTE_LIMIT: u64 = 1 << 24;

/// Trace: FR-066-AC-1, FR-066-AC-2
#[trace("FR-066-AC-1", "FR-066-AC-2")]
#[test]
fn published_execution_point_application_vectors_match_ir_candidates() {
    let path = proposal_dir_of(&fixtures_dir()).join(VECTOR_FILE);
    let vectors = read_json(&path).expect("QSpec node identity vectors");
    let rows = vectors["operation_vectors"]
        .as_array()
        .expect("QSpec operation vectors array");
    let cases = [
        ("execution-point-initialization", "initialization"),
        ("execution-point-handler", "handler"),
        ("execution-point-pre", "pre"),
        ("execution-point-post", "post"),
    ];
    assert_eq!(
        rows.iter()
            .filter(|row| row["name"]
                .as_str()
                .is_some_and(|name| name.starts_with("execution-point-")))
            .count(),
        cases.len(),
        "the published execution point vector set changed"
    );

    for (name, kind) in cases {
        let matches = rows
            .iter()
            .filter(|row| row["name"] == name)
            .collect::<Vec<_>>();
        assert_eq!(matches.len(), 1, "{name}: exactly one published row");
        let row = matches[0];
        let preimage = &row["preimage"];
        assert_eq!(preimage["version"], "quire.application-node/v1", "{name}");
        assert_eq!(preimage["body"]["term"], "application", "{name}");
        assert_eq!(
            preimage["body"]["operation"]["member"]["execution_point"]["kind"], kind,
            "{name}: published execution point variant"
        );
        let recorded = row["sha256"].as_str().expect("published SHA-256");
        let independent = quire_canonical::sha256(preimage, Limits::new(BYTE_LIMIT))
            .expect("published preimage encodes")
            .to_string();
        assert_eq!(independent, recorded, "{name}: published preimage digest");

        let node: CheckedSemanticNodeV2<Value> = serde_json::from_value(json!({
            "node_id": CheckedNodeId {
                domain: "quire.checked-semantic-node/v1".into(),
                digest: "00".repeat(32).into(),
            },
            "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": preimage["node_tag"],
            "semantic_form": preimage["semantic_form"],
            "semantic_type": preimage["semantic_type"],
            "dependencies": [],
            "occurrences": [],
            "declaration": preimage["declaration"],
            "body": preimage["body"],
        }))
        .expect("published preimage makes an IR application draft");
        let candidate =
            derive_application_node_id(ApplicationNodeDraft::new(&node), None, BYTE_LIMIT)
                .expect("IR application candidate");
        assert_eq!(
            candidate.candidate_node_id().digest.as_ref(),
            recorded,
            "{name}: IR application candidate differs from QSpec"
        );
    }
}
