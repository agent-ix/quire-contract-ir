// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Anonymous Map type admission at the public checked-package boundary.

use crate::support::checked_package::{
    bounds_body, canonical, evidence_for, index_fixture_node_ids, integer_key, node_id, settle,
    v2_all_families,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedNodeId, CheckedNodeKind, CheckedPackageReadLimits, CheckedPackageRefusalCause,
    CheckedPackageRefusalCode, CheckedPackageV2, CheckedPackageV2ReadResult, CheckedTerm,
    CompositeTypeForm,
};
use serde_json::{json, Value};

pub(crate) fn map_package() -> (Value, usize) {
    let mut package = v2_all_families();
    let name = package["name"].clone();
    let nodes = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    let mut ids = Vec::new();
    for form in ["text", "integer"] {
        let id = json!({"package": name, "path": ["scalar_type", form], "ordinal": 0});
        ids.push(id.clone());
        nodes.push(json!({
            "node_id": id, "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": "scalar_type", "semantic_form": form, "semantic_type": id,
            "dependencies": [], "occurrences": [{"role": "type", "ordinal": 0}],
            "body": {"term": "aggregate", "members": []}
        }));
    }
    let mut dependencies = ids.clone();
    dependencies.sort_by_key(|id| serde_json::from_value::<CheckedNodeId>(id.clone()).expect("id"));
    let id =
        json!({"package": name, "path": ["composite_type", "map", ids[0], ids[1]], "ordinal": 0});
    let position = nodes.len();
    nodes.push(json!({
        "node_id": id, "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": "composite_type", "semantic_form": "map", "semantic_type": id,
        "dependencies": dependencies, "occurrences": [{"role": "type", "ordinal": 0}],
        "body": {"term": "aggregate", "members": [
            {"term": "binding", "name": "key", "value": {"term": "reference", "target": ids[0]}},
            {"term": "binding", "name": "value", "value": {"term": "reference", "target": ids[1]}}
        ]}
    }));
    settle(&mut package);
    index_fixture_node_ids(&mut package);
    (package, position)
}

pub(crate) fn read(package: &Value) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(package),
    )
}

#[trace("TC-048", "FR-038-AC-482", "FR-038-AC-483")]
#[test]
fn map_keeps_both_ordered_type_operands() {
    let (package, position) = map_package();
    let admitted = match read(&package) {
        CheckedPackageV2ReadResult::Admitted(value) => value,
        other => panic!("Map<Text,Integer> must admit: {other:?}"),
    };
    let id: CheckedNodeId =
        serde_json::from_value(package["semantic_graph"]["nodes"][position]["node_id"].clone())
            .expect("map id");
    assert_eq!(
        admitted.node_kind(&id),
        Some(CheckedNodeKind::CompositeType(CompositeTypeForm::Map))
    );
    let Some(CheckedTerm::Aggregate { members }) = admitted.term(&id) else {
        panic!("map body");
    };
    assert_eq!(members.len(), 2);
    for (ordinal, name) in ["key", "value"].iter().enumerate() {
        let CheckedTerm::Binding {
            name: actual,
            value,
        } = &members[ordinal]
        else {
            panic!("binding");
        };
        assert_eq!(actual.as_ref(), *name);
        let CheckedTerm::Reference { target } = value.as_ref() else {
            panic!("reference");
        };
        let expected: CheckedNodeId = serde_json::from_value(
            package["semantic_graph"]["nodes"][position - 2 + ordinal]["node_id"].clone(),
        )
        .expect("type id");
        assert_eq!(target, &expected);
        assert_eq!(
            id.path[2 + ordinal],
            quire_contract_model::CheckedNodePathSegment::Index(
                u64::try_from(position - 2 + ordinal).expect("index")
            )
        );
    }
    assert_eq!(admitted.graph().nodes[position].dependencies.len(), 2);
    let projection = admitted
        .project_exact_type(
            &id,
            &quire_contract_model::SelectedExactIdentities::empty(),
            100,
        )
        .expect("Map<Text,Integer> has an exact value projection");
    let quire_exact::ValueType::Map(map) = projection.value_type() else {
        panic!("exact Map type");
    };
    assert_eq!(map.values(), &quire_exact::ValueType::Integer);
    let denied = admitted
        .project_exact_type(
            &id,
            &quire_contract_model::SelectedExactIdentities::empty(),
            2,
        )
        .expect_err("Map key inspection consumes the bounded work budget");
    assert_eq!(
        denied.code,
        quire_contract_model::ExactTypeProjectionErrorCode::ExhaustedWork
    );
}

#[trace("TC-048", "FR-038-AC-484")]
#[test]
fn malformed_and_mismatched_map_refuse_at_published_locations() {
    let (mut package, position) = map_package();
    package["semantic_graph"]["nodes"][position]["body"]["members"]
        .as_array_mut()
        .expect("members")
        .pop();
    settle(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("missing value must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MalformedWire);
    assert_eq!(refusal.cause, None);
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some(format!("/semantic_graph/nodes/{position}/body"))
    );

    let (mut package, position) = map_package();
    let other = package["semantic_graph"]["nodes"][position - 1]["node_id"].clone();
    package["semantic_graph"]["nodes"][position]["body"]["members"][0]["value"]["target"] =
        other.clone();
    package["semantic_graph"]["nodes"][position]["dependencies"] = json!([other]);
    settle(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("wrong key path must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::InvalidDerivedShape)
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some(format!("/semantic_graph/nodes/{position}"))
    );
}

#[trace("TC-048", "FR-038-AC-484")]
#[test]
fn nonscalar_key_refuses_at_map_node() {
    let (mut package, position) = map_package();
    let integer = package["semantic_graph"]["nodes"][position - 1]["node_id"].clone();
    let sequence = json!({"package": package["name"], "path": [
        "composite_type", "sequence", integer
    ], "ordinal": 0});
    let sequence_position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(json!({
            "node_id": sequence, "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": "composite_type", "semantic_form": "sequence",
            "semantic_type": sequence, "dependencies": [integer],
            "occurrences": [{"role": "type", "ordinal": 0}],
            "body": {"term": "aggregate", "members": [{"term": "reference", "target": integer}]}
        }));
    index_fixture_node_ids(&mut package);
    let sequence = package["semantic_graph"]["nodes"][sequence_position]["node_id"].clone();
    package["semantic_graph"]["nodes"][position]["node_id"]["path"][2] =
        json!({"node": sequence_position});
    let map = package["semantic_graph"]["nodes"][position]["node_id"].clone();
    package["semantic_graph"]["nodes"][position]["semantic_type"] = map;
    package["semantic_graph"]["nodes"][position]["body"]["members"][0]["value"]["target"] =
        sequence.clone();
    package["semantic_graph"]["nodes"][position]["dependencies"] = json!([sequence, integer]);
    settle(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("Map<Sequence<Integer>,Text> shape must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::InvalidDerivedShape)
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some(format!("/semantic_graph/nodes/{position}"))
    );
}

#[trace("TC-048", "FR-038-AC-483")]
#[test]
fn collection_bounds_remain_a_separate_node() {
    let (mut package, map_position) = map_package();
    let map = package["semantic_graph"]["nodes"][map_position]["node_id"].clone();
    let integer = node_id(&integer_key());
    let bound = json!({"package": package["name"], "path": [
        "bounded_domain", "collection_bounds", "min", integer, "1", "max", integer, "3", map
    ], "ordinal": 0});
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(json!({
            "node_id": bound, "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": "bounded_domain", "semantic_form": "collection_bounds",
            "semantic_type": map, "dependencies": [map],
            "occurrences": [{"role": "type", "ordinal": 0}], "body": bounds_body("1", "3")
        }));
    settle(&mut package);
    index_fixture_node_ids(&mut package);
    let admitted = match read(&package) {
        CheckedPackageV2ReadResult::Admitted(value) => value,
        other => panic!("bounded Map must admit: {other:?}"),
    };
    let id: CheckedNodeId =
        serde_json::from_value(package["semantic_graph"]["nodes"][position]["node_id"].clone())
            .expect("bound id");
    let projection = admitted
        .project_exact_type(
            &id,
            &quire_contract_model::SelectedExactIdentities::empty(),
            100,
        )
        .expect("bounded Map exact projection");
    let quire_exact::ValueType::Map(map) = projection.value_type() else {
        panic!("bounded Map");
    };
    assert_eq!(map.values(), &quire_exact::ValueType::Integer);
    assert_eq!(admitted.graph().nodes[map_position].dependencies.len(), 2);
    let CheckedTerm::Aggregate { members } = admitted
        .term(
            &serde_json::from_value(
                package["semantic_graph"]["nodes"][map_position]["node_id"].clone(),
            )
            .expect("map id"),
        )
        .expect("map term")
    else {
        panic!("map body");
    };
    assert_eq!(members.len(), 2);
}

#[trace("TC-048", "FR-038-AC-483")]
#[test]
fn changing_either_operand_changes_the_admitted_map_identity() {
    let (original, position) = map_package();
    let original_id = original["semantic_graph"]["nodes"][position]["node_id"].clone();
    let boolean = original["semantic_graph"]["nodes"][0]["node_id"].clone();
    for operand in [0_usize, 1] {
        let mut package = original.clone();
        package["semantic_graph"]["nodes"][position]["node_id"]["path"][operand + 2] =
            json!({"node": 0});
        let changed = package["semantic_graph"]["nodes"][position]["node_id"].clone();
        package["semantic_graph"]["nodes"][position]["semantic_type"] = changed.clone();
        package["semantic_graph"]["nodes"][position]["body"]["members"][operand]["value"]
            ["target"] = boolean.clone();
        let other = package["semantic_graph"]["nodes"][position - 1 - operand]["node_id"].clone();
        package["semantic_graph"]["nodes"][position]["dependencies"] = json!([boolean, other]);
        settle(&mut package);
        assert_ne!(changed, original_id);
        let admitted = match read(&package) {
            CheckedPackageV2ReadResult::Admitted(value) => value,
            other => panic!("changed Map operand must admit: {other:?}"),
        };
        let id: CheckedNodeId = serde_json::from_value(changed).expect("changed map id");
        assert_eq!(
            admitted.node_kind(&id),
            Some(CheckedNodeKind::CompositeType(CompositeTypeForm::Map))
        );
    }
}
