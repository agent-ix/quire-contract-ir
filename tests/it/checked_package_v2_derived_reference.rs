// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Reader-boundary controls for the derived `Reference<T>` target join.

use crate::support::checked_package::{
    canonical, evidence_for, index_fixture_node_ids, selected_interface_model_node, settle,
    v2_all_families,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedNodeId, CheckedPackageReadLimits, CheckedPackageRefusalCause, CheckedPackageRefusalCode,
    CheckedPackageV2, CheckedPackageV2ReadResult, CheckedTerm, ExactTypeProjectionErrorCode,
    SelectedExactIdentities,
};
use serde_json::{json, Value};

fn reference_package(tag: &str, form: &str) -> (Value, Value, usize) {
    let mut package = v2_all_families();
    if form == "systems_interface" {
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("fixture nodes")
            .push(selected_interface_model_node());
    }
    let target = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("fixture nodes")
        .iter()
        .find(|node| node["node_tag"] == tag && node["semantic_form"] == form)
        .unwrap_or_else(|| panic!("missing fixture target {tag}/{form}"))["node_id"]
        .clone();
    let reference = json!({
        "package": package["name"],
        "path": ["composite_type", "reference", target],
        "ordinal": 0,
    });
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("fixture nodes")
        .len();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("fixture nodes")
        .push(json!({
            "node_id": reference,
            "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": "composite_type",
            "semantic_form": "reference",
            "semantic_type": reference,
            "dependencies": [target],
            "occurrences": [{"role": "type", "ordinal": 0}],
            "body": {"term": "aggregate", "members": [
                {"term": "reference", "target": target}
            ]},
        }));
    settle(&mut package);
    index_fixture_node_ids(&mut package);
    let retained = package["semantic_graph"]["nodes"][position]["node_id"].clone();
    (package, retained, position)
}

fn read(package: &Value) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(package),
    )
}

/// Trace: FR-352-AC-3, FR-352-AC-9
#[trace("FR-352-AC-3", "FR-352-AC-9")]
#[test]
fn admitted_reference_requires_selected_exact_identity_row() {
    let (document, reference, _) = reference_package("model", "object_type");
    let admitted = match read(&document) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("reference fixture must admit: {other:?}"),
    };
    let id: CheckedNodeId = serde_json::from_value(reference).expect("reference ID");
    let rows = SelectedExactIdentities::from_stage_rows(&admitted, [], [])
        .expect("empty selected identity rows");
    let refusal = admitted
        .project_exact_type(&id, &rows, 3)
        .expect_err("missing object row");
    assert_eq!(
        refusal.code,
        ExactTypeProjectionErrorCode::UnavailableIdentityWitness
    );
}

/// Trace: QSpec FR-322-AC-62, RM-REF-TARGET. Model and relationship
/// declarations are eligible targets; a scalar target is refused at the
/// derived reference node's structural identity.
#[trace("TC-233", "FR-322-AC-62")]
#[test]
fn derived_reference_target_joins_eligible_declaration_kinds() {
    for (tag, form) in [
        ("model", "object_type"),
        ("model", "process"),
        ("model", "systems_interface"),
        ("relation", "relationship"),
    ] {
        let (package, reference, position) = reference_package(tag, form);
        let result = read(&package);
        let CheckedPackageV2ReadResult::Admitted(admitted) = result else {
            panic!("{tag}/{form} must admit as a reference target: {result:?}");
        };
        let id: CheckedNodeId =
            serde_json::from_value(reference.clone()).expect("reference identity");
        assert_eq!(
            serde_json::to_value(&admitted.graph().nodes[position].node_id)
                .expect("retained structural identity"),
            reference,
            "{tag}/{form}"
        );
        let Some(CheckedTerm::Aggregate { members }) = admitted.term(&id) else {
            panic!("{tag}/{form}: typed aggregate reference body is required");
        };
        let [CheckedTerm::Reference { target }] = members.as_slice() else {
            panic!("{tag}/{form}: one typed target reference is required");
        };
        assert_eq!(
            serde_json::to_value(target).expect("retained target"),
            package["semantic_graph"]["nodes"][position]["body"]["members"][0]["target"]
        );
    }

    // A state machine requires a selected model declaration document. Reuse
    // the in-repo selected-machine fixture and its authentic evidence rather
    // than changing the form of an unrelated graph node.
    let (mut machine, evidence) = crate::checked_package_v2_refinement::machine_package();
    let target = machine["semantic_graph"]["nodes"]
        .as_array()
        .expect("selected machine nodes")
        .iter()
        .find(|node| node["node_tag"] == "model" && node["semantic_form"] == "state_machine")
        .expect("selected state machine")["node_id"]
        .clone();
    let reference = json!({
        "package": machine["name"],
        "path": ["composite_type", "reference", target],
        "ordinal": 0,
    });
    machine["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("machine nodes")
        .push(json!({
            "node_id": reference,
            "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": "composite_type", "semantic_form": "reference",
            "semantic_type": reference, "dependencies": [target],
            "occurrences": [{"role": "type", "ordinal": 0}],
            "body": {"term": "aggregate", "members": [
                {"term": "reference", "target": target}
            ]},
        }));
    let mut source = machine["source_map"][0].clone();
    source["node_id"] = reference.clone();
    source["role"] = json!("type");
    source["ordinal"] = json!(0);
    machine["source_map"]
        .as_array_mut()
        .expect("machine source map")
        .push(source);
    index_fixture_node_ids(&mut machine);
    let reference = machine["semantic_graph"]["nodes"]
        .as_array()
        .expect("machine nodes")
        .last()
        .expect("reference node")["node_id"]
        .clone();
    let result = CheckedPackageV2::read(
        &canonical(&machine),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    );
    let CheckedPackageV2ReadResult::Admitted(admitted) = result else {
        panic!("selected state_machine reference must admit: {result:?}");
    };
    let id: CheckedNodeId = serde_json::from_value(reference).expect("reference identity");
    assert!(matches!(
        admitted.term(&id),
        Some(CheckedTerm::Aggregate { members }) if matches!(
            members.as_slice(), [CheckedTerm::Reference { target: actual }] if
                serde_json::to_value(actual).expect("target") == target
        )
    ));

    let (package, reference, position) = reference_package("scalar_type", "boolean");
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("scalar target must refuse as an invalid derived reference");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::InvalidDerivedShape)
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some(format!("/semantic_graph/nodes/{position}/node_id"))
    );
    assert_eq!(
        serde_json::to_value(refusal.locus).expect("refusal node identity"),
        reference
    );
}
