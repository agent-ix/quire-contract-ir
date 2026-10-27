// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Model-root structural keys against QSpec's authoritative published vector.

use super::{fixtures_dir, proposal_dir_of, read_json};
use ix_trace_rs::trace;
use quire_contract_model::{derive_structural_candidates, CheckedSemanticNodeV2};
use serde_json::{json, Value};

/// Trace: FR-065-AC-3
#[trace("TC-233", "FR-065-AC-3")]
#[test]
fn published_model_root_preimage_derives_the_published_key() {
    let path = proposal_dir_of(&fixtures_dir()).join("node-identity-vectors.json");
    let vectors = read_json(&path).expect("QSpec node identity vectors");
    let row = vectors["vectors"]
        .as_array()
        .expect("vectors")
        .iter()
        .find(|row| row["name"] == "model-root-counter-cas")
        .expect("published model-root vector");
    let preimage = &row["preimage"];
    assert_eq!(preimage["version"], "quire.structural-node/v1");
    assert_eq!(preimage["node_tag"], "model");
    assert_eq!(preimage["semantic_form"], "model_root");
    assert!(preimage["semantic_type"].is_null());
    assert!(preimage["declaration"].is_null());
    assert!(preimage["recursion"].is_null());
    assert_eq!(preimage["owner"]["kind"], "model_root");
    let draft_key = json!({"domain": "quire.checked-semantic-node/v1", "digest": "draft"});
    let draft: CheckedSemanticNodeV2<Value> = serde_json::from_value(json!({
        "node_id": draft_key, "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": preimage["node_tag"], "semantic_form": preimage["semantic_form"],
        "semantic_type": draft_key, "dependencies": [],
        "occurrences": [{"role": "generated", "ordinal": 0}],
        "owner": preimage["owner"], "body": preimage["body"],
    }))
    .expect("typed published root draft");
    let candidates = derive_structural_candidates(&[draft], 1 << 20, 100)
        .expect("model-root structural candidate");
    assert_eq!(candidates.nodes().len(), 1);
    assert_eq!(
        candidates.nodes()[0].candidate_node_id().digest.as_ref(),
        row["sha256"].as_str().expect("published key")
    );
}
