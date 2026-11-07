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
    CompositeTypeForm, ExactTypeProjectionErrorCode, SelectedExactIdentities,
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

#[trace("TC-048", "FR-038-AC-488", "FR-038-AC-489")]
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
            &mut quire_contract_model::NativeJoinBudget::new(100, u64::MAX),
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
            &mut quire_contract_model::NativeJoinBudget::new(2, u64::MAX),
        )
        .expect_err("Map key inspection consumes the bounded work budget");
    assert_eq!(
        denied.code,
        quire_contract_model::ExactTypeProjectionErrorCode::ExhaustedWork
    );
}

#[trace("TC-048", "FR-038-AC-490")]
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

#[trace("TC-048", "FR-038-AC-490")]
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

#[trace("TC-048", "TC-468", "FR-038-AC-489", "FR-359-AC-2")]
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
    let mut ample = quire_contract_model::NativeJoinBudget::new(100, u64::MAX);
    let projection = admitted
        .project_exact_type(
            &id,
            &quire_contract_model::SelectedExactIdentities::empty(),
            &mut ample,
        )
        .expect("bounded Map exact projection");
    let quire_exact::ValueType::Map(map) = projection.value_type() else {
        panic!("bounded Map");
    };
    assert_eq!(map.values(), &quire_exact::ValueType::Integer);
    let required = ample.consumed();
    let mut exact = quire_contract_model::NativeJoinBudget::new(required.work, required.storage);
    assert_eq!(
        admitted.project_exact_type(&id, &SelectedExactIdentities::empty(), &mut exact),
        Ok(projection.clone())
    );
    assert_eq!(exact.last_success_delta(), Some(required));
    let mut short =
        quire_contract_model::NativeJoinBudget::new(required.work, required.storage - 1);
    let refusal = admitted
        .project_exact_type(&id, &SelectedExactIdentities::empty(), &mut short)
        .unwrap_err();
    assert_eq!(refusal.code, ExactTypeProjectionErrorCode::ExhaustedStorage);
    assert_eq!(refusal.at, None);
    assert_eq!(
        short.first_unpaid().unwrap().kind,
        quire_contract_model::NativeJoinChargeKind::Storage
    );
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

#[trace("TC-048", "FR-038-AC-489")]
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

#[trace("TC-048", "FR-038-AC-488", "FR-038-AC-489")]
#[test]
fn non_text_scalar_key_admits_but_has_no_exact_projection() {
    let (mut package, position) = map_package();
    let boolean = package["semantic_graph"]["nodes"][0]["node_id"].clone();
    let integer = package["semantic_graph"]["nodes"][position - 1]["node_id"].clone();
    package["semantic_graph"]["nodes"][position]["node_id"]["path"][2] = json!({"node": 0});
    let map = package["semantic_graph"]["nodes"][position]["node_id"].clone();
    package["semantic_graph"]["nodes"][position]["semantic_type"] = map.clone();
    package["semantic_graph"]["nodes"][position]["body"]["members"][0]["value"]["target"] =
        boolean.clone();
    package["semantic_graph"]["nodes"][position]["dependencies"] = json!([boolean, integer]);
    settle(&mut package);
    let admitted = match read(&package) {
        CheckedPackageV2ReadResult::Admitted(value) => value,
        other => panic!("Map<Boolean,Integer> must admit: {other:?}"),
    };
    let id: CheckedNodeId = serde_json::from_value(map).expect("Map id");
    assert_eq!(
        admitted.node_kind(&id),
        Some(CheckedNodeKind::CompositeType(CompositeTypeForm::Map))
    );
    let refusal = admitted
        .project_exact_type(
            &id,
            &SelectedExactIdentities::empty(),
            &mut quire_contract_model::NativeJoinBudget::new(100, u64::MAX),
        )
        .expect_err("Exact Map requires a Text key");
    assert_eq!(
        refusal.code,
        ExactTypeProjectionErrorCode::UnsupportedExactType
    );
}

fn source_map_fixture() -> (CheckedPackageV2, quire_contract_model::SelectedSourceCollectionRow) {
    let (wire, position) = map_package();
    let CheckedPackageV2ReadResult::Admitted(package) = read(&wire) else {
        panic!("strict admitted map fixture");
    };
    let id: CheckedNodeId = serde_json::from_value(
        wire["semantic_graph"]["nodes"][position]["node_id"].clone(),
    ).expect("map id");
    let row = source_map_row(&package, &id, quire_exact::ValueType::Integer);
    (*package, row)
}

fn source_map_row(
    package: &CheckedPackageV2,
    id: &CheckedNodeId,
    value: quire_exact::ValueType,
) -> quire_contract_model::SelectedSourceCollectionRow {
    let CheckedTerm::Aggregate { members } = package.term(id).expect("map term") else {
        panic!("map operands");
    };
    let target = |ordinal: usize| {
        let CheckedTerm::Binding { value, .. } = &members[ordinal] else { panic!("map binding") };
        let CheckedTerm::Reference { target } = value.as_ref() else { panic!("map target") };
        target.clone()
    };
    quire_contract_model::SelectedSourceCollectionRow {
        structural_node: id.clone(),
        base_node: id.clone(),
        key_node: Some(target(0)),
        element_node: target(1),
        value_type: quire_exact::ValueType::map(quire_exact::MapType::new(value, None)),
    }
}

/// Trace: FR-357-AC-19.
#[trace("TC-470", "FR-357-AC-19")]
#[test]
fn source_map_row_joins_and_projects_its_complete_value() {
    let (package, row) = source_map_fixture();
    let mut budget = quire_contract_model::NativeJoinBudget::new(1_000_000, 1_000_000);
    let joined = SelectedExactIdentities::empty()
        .with_source_collection_rows(&package, [row.clone()], &mut budget)
        .expect("authentic map row joins");
    let projected = package.project_exact_type(&row.structural_node, &joined, &mut budget)
        .expect("joined map projects");
    assert_eq!(projected.value_type(), &row.value_type);
    assert_eq!(joined.source_collection_rows().next(), Some(&row));
}

/// Trace: FR-357-AC-19.
#[trace("TC-470", "FR-357-AC-19")]
#[test]
fn source_map_rows_preserve_key_and_non_collection_refusals() {
    use quire_contract_model::NativeJoinBudget;
    let (package, original) = source_map_fixture();
    for row in [
        quire_contract_model::SelectedSourceCollectionRow { key_node: None, ..original.clone() },
        quire_contract_model::SelectedSourceCollectionRow {
            key_node: Some(original.element_node.clone()), ..original.clone()
        },
        quire_contract_model::SelectedSourceCollectionRow {
            value_type: quire_exact::ValueType::Boolean, ..original.clone()
        },
        quire_contract_model::SelectedSourceCollectionRow {
            value_type: quire_exact::ValueType::map(quire_exact::MapType::new(
                quire_exact::ValueType::Integer,
                Some(quire_exact::CardinalityBound::new(0, 3).expect("bound")),
            )), ..original.clone()
        },
    ] {
        let mut budget = NativeJoinBudget::new(1_000_000, 1_000_000);
        let refusal = SelectedExactIdentities::empty()
            .with_source_collection_rows(&package, [row], &mut budget)
            .expect_err("a mismatched native row refuses");
        assert_eq!(refusal.code, ExactTypeProjectionErrorCode::MismatchedIdentityWitness);
        assert_eq!(refusal.at.as_deref(), Some(&original.structural_node));
        assert!(budget.consumed().work > 0);
        assert_eq!(budget.last_success_delta(), None);
    }
}

/// Trace: FR-357-AC-20.
#[trace("TC-470", "FR-357-AC-20")]
#[test]
fn source_map_rows_compose_at_exact_budget_and_preserve_first_denial() {
    use quire_contract_model::{NativeJoinBudget, NativeJoinChargeKind};
    let (package, row) = source_map_fixture();
    let initial = SelectedExactIdentities::empty();
    let run = |budget: &mut NativeJoinBudget| {
        let joined = initial.with_source_collection_rows(&package, [row.clone()], budget)?;
        package.project_exact_type(&row.structural_node, &joined, budget)
    };
    let mut ample = NativeJoinBudget::new(1_000_000, 1_000_000);
    let expected = run(&mut ample).expect("ample map read");
    let used = ample.consumed();
    let mut exact = NativeJoinBudget::new(used.work, used.storage);
    assert_eq!(run(&mut exact).expect("exact cumulative map allowance"), expected);
    assert_eq!(exact.consumed(), used);
    for kind in [NativeJoinChargeKind::Work, NativeJoinChargeKind::Storage] {
        let limits = match kind {
            NativeJoinChargeKind::Work => (used.work - 1, used.storage),
            NativeJoinChargeKind::Storage => (used.work, used.storage - 1),
        };
        let mut short = NativeJoinBudget::new(limits.0, limits.1);
        let error = run(&mut short).expect_err("one under refuses the first unpaid operation");
        let denied = short.first_unpaid().expect("native first charge");
        assert_eq!(denied.kind, kind);
        assert!(denied.amount > 0);
        assert_eq!(error.denied, Some(denied));
        assert_eq!(error.configured_bound, Some(match kind {
            NativeJoinChargeKind::Work => limits.0,
            NativeJoinChargeKind::Storage => limits.1,
        }));
        assert_eq!(short.work_limit(), limits.0);
        assert_eq!(short.storage_limit(), limits.1);
        assert_eq!(short.last_success_delta(), None);
        let locus = short.first_unpaid_locus().expect("native locus");
        let consumed = short.consumed();
        assert!(consumed.work > 0 && consumed.storage > 0, "paid prefix remains spent");
        let later = short.accounted_charge(kind, 99, locus).expect_err("first denial sticks");
        assert_eq!(later, denied);
        assert_eq!(short.first_unpaid_locus(), Some(locus));
        assert_eq!(short.consumed(), consumed);
    }
}

/// Trace: FR-357-AC-20.
#[trace("TC-470", "FR-357-AC-20")]
#[test]
fn source_map_zero_storage_refuses_without_allocating_rows_or_diagnostics() {
    use quire_contract_model::{NativeJoinBudget, NativeJoinChargeKind};
    let (package, row) = source_map_fixture();
    let initial = SelectedExactIdentities::empty();
    let mut budget = NativeJoinBudget::new(1_000_000, 0);
    let mut result = None;
    let measured = allocation_counter::measure(|| {
        result = Some(initial.with_source_collection_rows(&package, [row], &mut budget));
    });
    let refusal = result.expect("attempted join").expect_err("zero storage");
    assert_eq!(measured.count_total, 0, "no allocation before the first storage admission");
    assert_eq!(refusal.code, ExactTypeProjectionErrorCode::ExhaustedStorage);
    assert_eq!(refusal.denied, budget.first_unpaid());
    assert_eq!(refusal.denied.expect("storage charge").kind, NativeJoinChargeKind::Storage);
    assert_eq!(refusal.configured_bound, Some(0));
    assert_eq!(refusal.at, None);
    assert_eq!(refusal.requested, None);
    assert_eq!(refusal.form, None);
    assert_eq!(budget.consumed().storage, 0);
    assert_eq!(budget.last_success_delta(), None);
}

/// Trace: FR-357-AC-19.
#[trace("TC-470", "FR-357-AC-19")]
#[test]
fn nested_and_bounded_source_maps_preserve_all_value_rows_and_bounds() {
    use quire_contract_model::{NativeJoinBudget, SelectedSourceCollectionRow};
    use quire_exact::{CardinalityBound, MapType, ValueType};
    let (mut wire, inner_position) = map_package();
    let inner = wire["semantic_graph"]["nodes"][inner_position]["node_id"].clone();
    let text = wire["semantic_graph"]["nodes"][inner_position - 2]["node_id"].clone();
    let outer = json!({"package":wire["name"], "path":["composite_type","map",text,inner],"ordinal":0});
    let outer_position = wire["semantic_graph"]["nodes"].as_array().unwrap().len();
    wire["semantic_graph"]["nodes"].as_array_mut().unwrap().push(json!({
        "schema_version":"quire.checked-semantic-graph/v2", "node_id":outer,
        "node_tag":"composite_type", "semantic_form":"map", "semantic_type":outer,
        "dependencies":[text,inner], "occurrences":[{"role":"type","ordinal":0}],
        "body":{"term":"aggregate", "members":[
            {"term":"binding","name":"key","value":{"term":"reference","target":text}},
            {"term":"binding","name":"value","value":{"term":"reference","target":inner}}
        ]}}));
    let integer = node_id(&integer_key());
    let bounded = json!({"package":wire["name"], "path":["bounded_domain","collection_bounds",
        "min",integer,"1","max",integer,"3",outer], "ordinal":0});
    let bounded_position = wire["semantic_graph"]["nodes"].as_array().unwrap().len();
    wire["semantic_graph"]["nodes"].as_array_mut().unwrap().push(json!({
        "schema_version":"quire.checked-semantic-graph/v2", "node_id":bounded,
        "node_tag":"bounded_domain", "semantic_form":"collection_bounds", "semantic_type":outer,
        "dependencies":[outer], "occurrences":[{"role":"type","ordinal":0}], "body":bounds_body("1","3")
    }));
    settle(&mut wire);
    index_fixture_node_ids(&mut wire);
    let CheckedPackageV2ReadResult::Admitted(package) = read(&wire) else {
        panic!("nested bounded map strictly admits");
    };
    let id = |position: usize| serde_json::from_value::<CheckedNodeId>(
        wire["semantic_graph"]["nodes"][position]["node_id"].clone()).unwrap();
    let inner = source_map_row(&package, &id(inner_position), ValueType::Integer);
    let outer = source_map_row(&package, &id(outer_position), inner.value_type.clone());
    let bounded = SelectedSourceCollectionRow {
        structural_node: id(bounded_position),
        value_type: ValueType::map(MapType::new(inner.value_type.clone(), Some(
            CardinalityBound::new(1,3).unwrap()))),
        ..outer.clone()
    };
    let mut budget = NativeJoinBudget::new(1_000_000, 1_000_000);
    let joined = SelectedExactIdentities::empty().with_source_collection_rows(&package,
        [inner.clone(), outer.clone(), bounded.clone()], &mut budget).expect("nested rows join");
    for row in [&inner, &outer, &bounded] {
        let projection = package.project_exact_type(&row.structural_node, &joined, &mut budget)
            .expect("nested map projection");
        assert_eq!(projection.value_type(), &row.value_type);
        assert_eq!(projection.type_node(), &row.base_node);
    }
    let missing_inner = SelectedExactIdentities::empty().with_source_collection_rows(&package,
        [outer.clone()], &mut NativeJoinBudget::new(1_000_000, 1_000_000)).unwrap();
    let error = package.project_exact_type(&outer.structural_node, &missing_inner,
        &mut NativeJoinBudget::new(1_000_000, 1_000_000)).expect_err("nested map row is required");
    assert_eq!(error.code, ExactTypeProjectionErrorCode::UnavailableIdentityWitness);
    assert_eq!(error.at.as_deref(), Some(&inner.structural_node));
}
