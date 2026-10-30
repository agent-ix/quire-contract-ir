// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Structural V2 node identities: the reader retains authored positions,
//! rejects duplicates, and does not derive identifiers from node bodies.
//! The published multi-package and graph-order vectors run in conformance_qspec.

use crate::support::checked_package::{canonical, evidence_for, v2_all_families};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedPackageReadLimits, CheckedPackageRefusalCode, CheckedPackageV2,
    CheckedPackageV2ReadResult,
};
use serde_json::{Value, json};

fn local_package() -> Value {
    let mut package = v2_all_families();
    let node = package["semantic_graph"]["nodes"][0].clone();
    let id = node["node_id"].clone();
    package["semantic_graph"]["nodes"] = json!([node]);
    package["source_map"] = json!(
        package["source_map"]
            .as_array()
            .expect("source map")
            .iter()
            .filter(|entry| entry["node_id"] == id)
            .cloned()
            .collect::<Vec<_>>()
    );
    package
}

fn read(package: &Value) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(package),
    )
}

#[trace("TC-048")]
#[test]
fn structural_identity_is_retained_without_body_derivation() {
    let first = local_package();
    let mut changed = first.clone();
    changed["semantic_graph"]["nodes"][0]["semantic_form"] = json!("integer");
    let CheckedPackageV2ReadResult::Admitted(first_read) = read(&first) else {
        panic!("original scalar must admit: {:?}", read(&first));
    };
    let CheckedPackageV2ReadResult::Admitted(changed_read) = read(&changed) else {
        panic!("changed scalar must admit: {:?}", read(&changed));
    };
    assert_eq!(
        first_read.graph().nodes[0].node_id,
        changed_read.graph().nodes[0].node_id
    );
    assert_ne!(first_read.graph().nodes[0], changed_read.graph().nodes[0]);
    assert_eq!(
        serde_json::to_value(&changed_read.graph().nodes[0].node_id).expect("node id"),
        first["semantic_graph"]["nodes"][0]["node_id"]
    );
}

#[trace("TC-048")]
#[test]
fn second_equal_structural_node_id_refuses_at_that_node() {
    let mut package = local_package();
    let duplicate = package["semantic_graph"]["nodes"][0].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(duplicate);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("duplicate node id must refuse: {:?}", read(&package));
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some("/semantic_graph/nodes/1/node_id".to_owned())
    );
}

#[trace("TC-048")]
#[test]
fn a_distinct_path_is_a_distinct_node_even_with_equal_bodies() {
    let mut package = local_package();
    let mut second = package["semantic_graph"]["nodes"][0].clone();
    second["node_id"]["path"] = json!(["other-boolean"]);
    second["semantic_type"] = second["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(second.clone());
    let mut source = package["source_map"][0].clone();
    source["node_id"] = second["node_id"].clone();
    package["source_map"]
        .as_array_mut()
        .expect("source map")
        .push(source);
    let CheckedPackageV2ReadResult::Admitted(admitted) = read(&package) else {
        panic!("distinct structural path must admit: {:?}", read(&package));
    };
    assert_ne!(
        admitted.graph().nodes[0].node_id,
        admitted.graph().nodes[1].node_id
    );
    assert_eq!(
        admitted.graph().nodes[0].body,
        admitted.graph().nodes[1].body
    );
}
