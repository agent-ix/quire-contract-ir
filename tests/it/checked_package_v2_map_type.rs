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

fn sort_map_fixture_dependencies(wire: &mut Value) {
    for node in wire["semantic_graph"]["nodes"].as_array_mut().unwrap() {
        node["dependencies"]
            .as_array_mut()
            .unwrap()
            .sort_by(|left, right| {
                let left: CheckedNodeId = serde_json::from_value(left.clone()).unwrap();
                let right: CheckedNodeId = serde_json::from_value(right.clone()).unwrap();
                left.cmp(&right)
            });
    }
}

fn source_map_fixture() -> (
    CheckedPackageV2,
    quire_contract_model::SelectedSourceCollectionRow,
) {
    let (wire, position) = map_package();
    let CheckedPackageV2ReadResult::Admitted(package) = read(&wire) else {
        panic!("strict admitted map fixture");
    };
    let id: CheckedNodeId =
        serde_json::from_value(wire["semantic_graph"]["nodes"][position]["node_id"].clone())
            .expect("map id");
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
        let CheckedTerm::Binding { value, .. } = &members[ordinal] else {
            panic!("map binding")
        };
        let CheckedTerm::Reference { target } = value.as_ref() else {
            panic!("map target")
        };
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
#[trace("TC-473", "FR-357-AC-19")]
#[test]
fn source_map_row_joins_and_projects_its_complete_value() {
    let (package, row) = source_map_fixture();
    let mut budget = quire_contract_model::NativeJoinBudget::new(1_000_000, 1_000_000);
    let joined = SelectedExactIdentities::empty()
        .with_source_collection_rows(&package, [row.clone()], &mut budget)
        .expect("authentic map row joins");
    let projected = package
        .project_exact_type(&row.structural_node, &joined, &mut budget)
        .expect("joined map projects");
    assert_eq!(projected.value_type(), &row.value_type);
    assert_eq!(joined.source_collection_rows().next(), Some(&row));
}

/// Trace: FR-357-AC-19.
#[trace("TC-473", "FR-357-AC-19")]
#[test]
fn source_map_rows_preserve_key_and_non_collection_refusals() {
    use quire_contract_model::NativeJoinBudget;
    let (package, original) = source_map_fixture();
    for row in [
        quire_contract_model::SelectedSourceCollectionRow {
            key_node: None,
            ..original.clone()
        },
        quire_contract_model::SelectedSourceCollectionRow {
            key_node: Some(original.element_node.clone()),
            ..original.clone()
        },
        quire_contract_model::SelectedSourceCollectionRow {
            value_type: quire_exact::ValueType::Boolean,
            ..original.clone()
        },
        quire_contract_model::SelectedSourceCollectionRow {
            value_type: quire_exact::ValueType::map(quire_exact::MapType::new(
                quire_exact::ValueType::Integer,
                Some(quire_exact::CardinalityBound::new(0, 3).expect("bound")),
            )),
            ..original.clone()
        },
    ] {
        let mut budget = NativeJoinBudget::new(1_000_000, 1_000_000);
        let refusal = SelectedExactIdentities::empty()
            .with_source_collection_rows(&package, [row], &mut budget)
            .expect_err("a mismatched native row refuses");
        assert_eq!(
            refusal.code,
            ExactTypeProjectionErrorCode::MismatchedIdentityWitness
        );
        assert_eq!(refusal.at.as_deref(), Some(&original.structural_node));
        assert!(budget.consumed().work > 0);
        assert_eq!(budget.last_success_delta(), None);
    }
}

/// Trace: FR-357-AC-20.
#[trace("TC-473", "FR-357-AC-20")]
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
    assert_eq!(
        run(&mut exact).expect("exact cumulative map allowance"),
        expected
    );
    assert_eq!(exact.consumed(), used);
    let caller_locus = quire_contract_model::NativeJoinLocus {
        source: quire_contract_model::NativeJoinLocusSource::MonitorSeed(7),
        form: "caller_map_preparation",
    };
    let mut prefixed = NativeJoinBudget::new(used.work + 3, used.storage + 2);
    prefixed.resume();
    prefixed
        .accounted_charge(NativeJoinChargeKind::Work, 3, caller_locus)
        .unwrap();
    prefixed
        .accounted_charge(NativeJoinChargeKind::Storage, 2, caller_locus)
        .unwrap();
    assert_eq!(run(&mut prefixed).unwrap(), expected);
    assert_eq!(prefixed.consumed().work, used.work + 3);
    assert_eq!(prefixed.consumed().storage, used.storage + 2);
    let mut caller_short = NativeJoinBudget::new(used.work + 3, 0);
    caller_short.resume();
    caller_short
        .accounted_charge(NativeJoinChargeKind::Work, 3, caller_locus)
        .unwrap();
    let denied = caller_short
        .accounted_charge(NativeJoinChargeKind::Storage, 2, caller_locus)
        .unwrap_err();
    assert_eq!(
        denied,
        quire_contract_model::NativeJoinDeniedCharge {
            kind: NativeJoinChargeKind::Storage,
            amount: 2,
        }
    );
    assert_eq!(caller_short.first_unpaid_locus(), Some(caller_locus));
    assert_eq!(caller_short.consumed().work, 3);
    assert_eq!(caller_short.consumed().storage, 0);
    assert_eq!(caller_short.last_success_delta(), None);
    assert_eq!(caller_short.storage_limit(), 0);
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
        assert_eq!(
            error.configured_bound,
            Some(match kind {
                NativeJoinChargeKind::Work => limits.0,
                NativeJoinChargeKind::Storage => limits.1,
            })
        );
        assert_eq!(short.work_limit(), limits.0);
        assert_eq!(short.storage_limit(), limits.1);
        assert_eq!(short.last_success_delta(), None);
        let locus = short.first_unpaid_locus().expect("native locus");
        let consumed = short.consumed();
        assert!(
            consumed.work > 0 && consumed.storage > 0,
            "paid prefix remains spent"
        );
        let later = short
            .accounted_charge(kind, 99, locus)
            .expect_err("first denial sticks");
        assert_eq!(later, denied);
        assert_eq!(short.first_unpaid_locus(), Some(locus));
        assert_eq!(short.consumed(), consumed);
    }
}

/// Trace: FR-357-AC-20.
#[trace("TC-473", "FR-357-AC-20")]
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
    assert_eq!(
        measured.count_total, 0,
        "no allocation before the first storage admission"
    );
    assert_eq!(refusal.code, ExactTypeProjectionErrorCode::ExhaustedStorage);
    assert_eq!(refusal.denied, budget.first_unpaid());
    assert_eq!(
        refusal.denied.expect("storage charge").kind,
        NativeJoinChargeKind::Storage
    );
    assert_eq!(refusal.configured_bound, Some(0));
    assert_eq!(refusal.at, None);
    assert_eq!(refusal.requested, None);
    assert_eq!(refusal.form, None);
    assert_eq!(budget.consumed().storage, 0);
    assert_eq!(budget.last_success_delta(), None);
}

/// Trace: FR-357-AC-19.
#[trace("TC-473", "FR-357-AC-19")]
#[trace("TC-473", "FR-357-AC-20")]
#[test]
fn nested_and_bounded_source_maps_preserve_all_value_rows_and_bounds() {
    use quire_contract_model::{NativeJoinBudget, SelectedSourceCollectionRow};
    use quire_exact::{CardinalityBound, MapType, ValueType};
    let (mut wire, inner_position) = map_package();
    let inner = wire["semantic_graph"]["nodes"][inner_position]["node_id"].clone();
    let text = wire["semantic_graph"]["nodes"][inner_position - 2]["node_id"].clone();
    let outer =
        json!({"package":wire["name"], "path":["composite_type","map",text,inner],"ordinal":0});
    let outer_position = wire["semantic_graph"]["nodes"].as_array().unwrap().len();
    wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .push(json!({
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
    sort_map_fixture_dependencies(&mut wire);
    settle(&mut wire);
    index_fixture_node_ids(&mut wire);
    let CheckedPackageV2ReadResult::Admitted(package) = read(&wire) else {
        panic!("nested bounded map strictly admits");
    };
    let id = |position: usize| {
        serde_json::from_value::<CheckedNodeId>(
            wire["semantic_graph"]["nodes"][position]["node_id"].clone(),
        )
        .unwrap()
    };
    let inner = source_map_row(&package, &id(inner_position), ValueType::Integer);
    let outer = source_map_row(&package, &id(outer_position), inner.value_type.clone());
    let bounded = SelectedSourceCollectionRow {
        structural_node: id(bounded_position),
        value_type: ValueType::map(MapType::new(
            inner.value_type.clone(),
            Some(CardinalityBound::new(1, 3).unwrap()),
        )),
        ..outer.clone()
    };
    let mut budget = NativeJoinBudget::new(1_000_000, 1_000_000);
    let joined = SelectedExactIdentities::empty()
        .with_source_collection_rows(
            &package,
            [inner.clone(), outer.clone(), bounded.clone()],
            &mut budget,
        )
        .expect("nested rows join");
    let join_work = budget.consumed().work;
    for (form, at) in [
        ("source_map_key", outer.key_node.as_ref().unwrap()),
        ("source_map_value", &outer.element_node),
    ] {
        let expected_locus = quire_contract_model::NativeJoinLocus {
            source: quire_contract_model::NativeJoinLocusSource::ReachedRow(1),
            form,
        };
        let mut observed = false;
        for allowance in 0..join_work {
            let mut short = NativeJoinBudget::new(allowance, 1_000_000);
            let error = SelectedExactIdentities::empty()
                .with_source_collection_rows(
                    &package,
                    [inner.clone(), outer.clone(), bounded.clone()],
                    &mut short,
                )
                .expect_err("work below complete native acquisition");
            if short.first_unpaid_locus() != Some(expected_locus) {
                continue;
            }
            let expected_charge = quire_contract_model::NativeJoinDeniedCharge {
                kind: quire_contract_model::NativeJoinChargeKind::Work,
                amount: 1,
            };
            assert_eq!(error.denied, Some(expected_charge));
            assert_eq!(error.configured_bound, Some(allowance));
            assert_eq!(error.at.as_deref(), Some(at));
            assert_eq!(short.consumed().work, allowance);
            assert!(
                short.consumed().storage > 0,
                "the preceding authentic map row remains paid"
            );
            assert_eq!(short.last_success_delta(), None);
            let spent = short.consumed();
            let later = short
                .accounted_charge(
                    quire_contract_model::NativeJoinChargeKind::Storage,
                    99,
                    expected_locus,
                )
                .unwrap_err();
            assert_eq!(later, expected_charge);
            assert_eq!(short.first_unpaid_locus(), Some(expected_locus));
            assert_eq!(short.consumed(), spent);
            observed = true;
            break;
        }
        assert!(
            observed,
            "native key/value work must expose its independently named original row phase"
        );
    }
    let mut projection_budget = NativeJoinBudget::new(1_000_000, 1_000_000);
    package
        .project_exact_type(&inner.structural_node, &joined, &mut projection_budget)
        .unwrap();
    for (form, at) in [
        ("source_map_key", inner.key_node.as_ref().unwrap()),
        ("source_map_value", &inner.element_node),
    ] {
        let expected_locus = quire_contract_model::NativeJoinLocus {
            source: quire_contract_model::NativeJoinLocusSource::GraphNode(inner_position),
            form,
        };
        let mut observed = false;
        for allowance in 0..projection_budget.consumed().work {
            let mut short = NativeJoinBudget::new(allowance, 1_000_000);
            let error = package
                .project_exact_type(&inner.structural_node, &joined, &mut short)
                .expect_err("below complete projection work");
            if short.first_unpaid_locus() != Some(expected_locus) {
                continue;
            }
            assert_eq!(
                error.denied,
                Some(quire_contract_model::NativeJoinDeniedCharge {
                    kind: quire_contract_model::NativeJoinChargeKind::Work,
                    amount: 1,
                })
            );
            assert_eq!(error.configured_bound, Some(allowance));
            assert_eq!(
                error.at, None,
                "native projection resource errors remain compact"
            );
            assert_eq!(short.last_success_delta(), None);
            assert!(
                package.graph().nodes.iter().any(|node| &node.node_id == at),
                "the independently selected key/value node is admitted"
            );
            observed = true;
            break;
        }
        assert!(
            observed,
            "fresh projection must expose the authentic requested graph node and phase"
        );
    }
    for row in [&inner, &outer, &bounded] {
        let projection = package
            .project_exact_type(&row.structural_node, &joined, &mut budget)
            .expect("nested map projection");
        assert_eq!(projection.value_type(), &row.value_type);
        assert_eq!(projection.type_node(), &row.base_node);
    }
    let missing_inner = SelectedExactIdentities::empty()
        .with_source_collection_rows(
            &package,
            [outer.clone()],
            &mut NativeJoinBudget::new(1_000_000, 1_000_000),
        )
        .unwrap();
    let error = package
        .project_exact_type(
            &outer.structural_node,
            &missing_inner,
            &mut NativeJoinBudget::new(1_000_000, 1_000_000),
        )
        .expect_err("nested map row is required");
    assert_eq!(
        error.code,
        ExactTypeProjectionErrorCode::UnavailableIdentityWitness
    );
    assert_eq!(error.at.as_deref(), Some(&inner.structural_node));
}

/// Trace: FR-357-AC-20.
#[trace("TC-473", "FR-357-AC-20")]
#[test]
fn source_map_copy_allocation_follows_its_storage_charge_after_a_paid_prefix() {
    use quire_contract_model::{NativeJoinBudget, NativeJoinChargeKind, NativeJoinLocusSource};
    let (package, row) = source_map_fixture();
    let initial = SelectedExactIdentities::empty();
    let acquire = |budget: &mut NativeJoinBudget| {
        initial.with_source_collection_rows(&package, [row.clone()], budget)
    };
    let mut ample = NativeJoinBudget::new(1_000_000, 1_000_000);
    acquire(&mut ample).expect("complete acquisition");
    let at_row = |limit: &u64| {
        let mut budget = NativeJoinBudget::new(1_000_000, *limit);
        acquire(&mut budget).is_err()
            && budget
                .first_unpaid_locus()
                .is_some_and(|locus| locus.form == "source_collection_row")
    };
    let prefix_storage = (0..ample.consumed().storage).find(at_row).unwrap();
    let copy_boundary = (0..ample.consumed().storage).rfind(at_row).unwrap();
    let mut short = NativeJoinBudget::new(1_000_000, copy_boundary);
    let refusal = acquire(&mut short).expect_err("one below complete row acquisition");
    let denied = short.first_unpaid().unwrap();
    assert_eq!(denied.kind, NativeJoinChargeKind::Storage);
    assert_eq!(refusal.denied, Some(denied));
    assert_eq!(refusal.configured_bound, Some(copy_boundary));
    let expected_locus = quire_contract_model::NativeJoinLocus {
        source: NativeJoinLocusSource::ReachedRow(0),
        form: "source_collection_row",
    };
    assert_eq!(short.first_unpaid_locus(), Some(expected_locus));
    assert!(short.consumed().storage >= prefix_storage);
    assert_eq!(short.last_success_delta(), None);
    // One-under may pay for an owned error after the row refusal. Measure the
    // same row operation at the start of its storage reservation instead: no
    // diagnostic fits there, so these two scopes differ by actual acquisition.
    let unpaid_row = row.clone();
    let mut unpaid = NativeJoinBudget::new(1_000_000, prefix_storage);
    let mut result = None;
    let before = allocation_counter::measure(|| {
        result = Some(initial.with_source_collection_rows(&package, [unpaid_row], &mut unpaid));
    });
    let error = result.unwrap().expect_err("no row storage remains");
    assert_eq!(error.denied, Some(denied));
    assert_eq!(unpaid.first_unpaid_locus(), Some(expected_locus));
    assert_eq!(error.requested, None);
    assert_eq!(error.at, None);
    assert_eq!(error.form, None);
    assert_eq!(unpaid.consumed().storage, prefix_storage);
    assert!(
        before.count_total > 0,
        "previously admitted node copies really allocate"
    );
    let paid_row = row.clone();
    let mut paid = NativeJoinBudget::new(1_000_000, copy_boundary + 1);
    let mut result = None;
    let after = allocation_counter::measure(|| {
        result = Some(initial.with_source_collection_rows(&package, [paid_row], &mut paid));
    });
    let error = result
        .unwrap()
        .expect_err("later owner map copy remains unpaid");
    assert_eq!(
        paid.first_unpaid_locus().unwrap().form,
        "source_collection_prior_map_copy"
    );
    assert_eq!(error.requested, None);
    assert_eq!(error.at, None);
    assert_eq!(error.form, None);
    assert_eq!(
        paid.consumed().storage - unpaid.consumed().storage,
        u64::try_from(denied.amount).unwrap()
    );
    assert!(
        after.count_total > before.count_total,
        "the retained row and native map entry allocate only after their storage admission"
    );
    assert_eq!(paid.last_success_delta(), None);
}

/// Trace: FR-357-AC-19, FR-357-AC-20.
#[trace("TC-473", "FR-357-AC-19")]
#[trace("TC-473", "FR-357-AC-20")]
#[test]
fn cold_integer_map_rows_inspect_and_compare_before_only_admitted_allocations() {
    use crate::support::checked_package::{integer_range_key, reassign_node_id};
    use quire_contract_model::{NativeJoinBudget, NativeJoinChargeKind, NativeJoinLocusSource};
    use quire_exact::{Integer, IntegerInterval, MapType, ValueType};
    let lower = "170141183460469231731687303715884105728";
    let upper = "680564733841876926926749214863536422913";
    let (mut wire, map_position) = map_package();
    let range = node_id(&integer_range_key(lower, upper));
    let integer = node_id(&integer_key());
    wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .push(json!({
            "schema_version":"quire.checked-semantic-graph/v2", "node_id":range,
            "node_tag":"bounded_domain", "semantic_form":"integer_range", "semantic_type":integer,
            "dependencies":[integer], "occurrences":[{"role":"type","ordinal":0}],
            "body":bounds_body(lower,upper)
        }));
    let old = wire["semantic_graph"]["nodes"][map_position]["node_id"].clone();
    let mut fresh = old.clone();
    fresh["path"][3] = range.clone();
    reassign_node_id(&mut wire, &old, &fresh);
    wire["semantic_graph"]["nodes"][map_position]["body"]["members"][1]["value"]["target"] =
        range.clone();
    let text = wire["semantic_graph"]["nodes"][map_position - 2]["node_id"].clone();
    wire["semantic_graph"]["nodes"][map_position]["dependencies"] = json!([text, range]);
    sort_map_fixture_dependencies(&mut wire);
    settle(&mut wire);
    index_fixture_node_ids(&mut wire);
    let CheckedPackageV2ReadResult::Admitted(package) = read(&wire) else {
        panic!("Map<Text,Int[large,larger]> strictly admits");
    };
    let id = serde_json::from_value::<CheckedNodeId>(
        wire["semantic_graph"]["nodes"][map_position]["node_id"].clone(),
    )
    .unwrap();
    let low: Integer = lower.parse().unwrap();
    let high: Integer = upper.parse().unwrap();
    let interval =
        IntegerInterval::new(low.try_clone().unwrap(), high.try_clone().unwrap()).unwrap();
    let row = || source_map_row(&package, &id, ValueType::Int(interval.clone()));
    let initial = SelectedExactIdentities::empty();
    let mut ample = NativeJoinBudget::new(1_000_000, 1_000_000);
    let joined = initial
        .with_source_collection_rows(&package, [row()], &mut ample)
        .unwrap();
    let join_storage = ample.consumed().storage;
    let projection = package
        .project_exact_type(&id, &joined, &mut ample)
        .expect("cold numeric map projects");
    assert_eq!(
        projection.value_type(),
        &ValueType::map(MapType::new(ValueType::Int(interval.clone()), None))
    );
    let boundary = (0..join_storage)
        .rfind(|limit| {
            let mut budget = NativeJoinBudget::new(1_000_000, *limit);
            initial
                .with_source_collection_rows(&package, [row()], &mut budget)
                .is_err()
                && budget
                    .first_unpaid_locus()
                    .is_some_and(|locus| locus.form == "source_collection_row")
        })
        .expect("native row retention boundary");
    for limit in [0, boundary] {
        let cold_row = row();
        let mut budget = NativeJoinBudget::new(1_000_000, limit);
        let mut result = None;
        let allocations = allocation_counter::measure(|| {
            result = Some(initial.with_source_collection_rows(&package, [cold_row], &mut budget));
        });
        let error = result.unwrap().expect_err("unpaid cold map row");
        if limit == 0 {
            assert_eq!(
                allocations.count_total, 0,
                "zero storage admits no allocation"
            );
        } else {
            let ordinary_interval = IntegerInterval::new(
                lower.parse::<Integer>().unwrap(),
                upper.parse::<Integer>().unwrap(),
            )
            .unwrap();
            let ordinary_row = source_map_row(&package, &id, ValueType::Int(ordinary_interval));
            let mut ordinary_budget = NativeJoinBudget::new(1_000_000, limit);
            let mut ordinary_result = None;
            let ordinary = allocation_counter::measure(|| {
                ordinary_result = Some(initial.with_source_collection_rows(
                    &package,
                    [ordinary_row],
                    &mut ordinary_budget,
                ));
            });
            let ordinary_error = ordinary_result
                .unwrap()
                .expect_err("same late row reservation");
            assert_eq!(ordinary_error.denied, error.denied);
            assert_eq!(ordinary_budget.consumed(), budget.consumed());
            assert_eq!(allocations.count_total, ordinary.count_total,
                "a fresh cold copied domain adds no cache allocation to the identical native prefix");
            assert_eq!(allocations.bytes_total, ordinary.bytes_total);
        }
        assert_eq!(error.denied, budget.first_unpaid());
        assert_eq!(error.configured_bound, Some(limit));
        assert_eq!(budget.last_success_delta(), None);
        if limit != 0 {
            assert!(budget.consumed().storage > 0);
            assert_eq!(
                budget.first_unpaid_locus().unwrap().source,
                NativeJoinLocusSource::ReachedRow(0)
            );
            assert_eq!(
                budget.first_unpaid_locus().unwrap().form,
                "source_collection_row"
            );
        }
    }
    // A fresh joined payload keeps the integer images cold until the real
    // bounded owner comparison; locate its logical handle admission directly.
    let joined = initial
        .with_source_collection_rows(
            &package,
            [row()],
            &mut NativeJoinBudget::new(1_000_000, 1_000_000),
        )
        .unwrap();
    let mut measured = NativeJoinBudget::new(1_000_000, 1_000_000);
    package
        .project_exact_type(&id, &joined, &mut measured)
        .unwrap();
    let handle_boundary = (0..measured.consumed().storage)
        .find(|limit| {
            let mut budget = NativeJoinBudget::new(1_000_000, *limit);
            package
                .project_exact_type(&id, &joined, &mut budget)
                .is_err()
                && budget
                    .first_unpaid_locus()
                    .is_some_and(|locus| locus.form == "source_collection_comparison_handle")
        })
        .expect("bounded numeric comparison handle charge");
    let fresh_joined = initial
        .with_source_collection_rows(
            &package,
            [row()],
            &mut NativeJoinBudget::new(1_000_000, 1_000_000),
        )
        .unwrap();
    let mut short = NativeJoinBudget::new(1_000_000, handle_boundary);
    let mut result = None;
    let before = allocation_counter::measure(|| {
        result = Some(package.project_exact_type(&id, &fresh_joined, &mut short));
    });
    let error = result
        .unwrap()
        .expect_err("logical comparison handle is unpaid");
    let denied = short.first_unpaid().unwrap();
    assert_eq!(denied.kind, NativeJoinChargeKind::Storage);
    assert_eq!(
        denied.amount, 1,
        "one shared Arc handle in FR-358 logical units"
    );
    assert_eq!(error.denied, Some(denied));
    assert_eq!(error.configured_bound, Some(handle_boundary));
    assert_eq!(
        short.first_unpaid_locus(),
        Some(quire_contract_model::NativeJoinLocus {
            source: NativeJoinLocusSource::GraphNode(map_position),
            form: "source_collection_comparison_handle",
        })
    );
    assert_eq!(
        short.first_unpaid_locus().unwrap().form,
        "source_collection_comparison_handle"
    );
    let fresh_joined = initial
        .with_source_collection_rows(
            &package,
            [row()],
            &mut NativeJoinBudget::new(1_000_000, 1_000_000),
        )
        .unwrap();
    let mut paid = NativeJoinBudget::new(1_000_000, handle_boundary + 1);
    let mut result = None;
    let after = allocation_counter::measure(|| {
        result = Some(package.project_exact_type(&id, &fresh_joined, &mut paid));
    });
    result.unwrap().expect_err("later result remains unpaid");
    assert_eq!(
        after.count_total,
        before.count_total + 1,
        "only the admitted cancellation Arc allocates; cold comparison adds no cache allocation"
    );
    assert_eq!(paid.consumed().storage, short.consumed().storage + 1);
    assert_eq!(short.last_success_delta(), None);
}

fn nested_bounded_quantity_maps() -> (
    CheckedPackageV2,
    CheckedNodeId,
    CheckedNodeId,
    CheckedNodeId,
) {
    use crate::support::checked_package::{nominal_fixture_members, nominal_package};
    let members = nominal_fixture_members();
    let unit = node_id(
        &members
            .iter()
            .find(|(body, _)| body["version"] == "quire.unit-node/v1")
            .expect("existing declared unit")
            .1,
    );
    let mut wire = nominal_package(&members);
    let mut scalar = |form: &str| {
        let id = json!({"package":wire["name"], "path":["scalar_type",form], "ordinal":0});
        wire["semantic_graph"]["nodes"]
            .as_array_mut()
            .unwrap()
            .push(json!({
                "schema_version":"quire.checked-semantic-graph/v2", "node_id":id,
                "node_tag":"scalar_type", "semantic_form":form, "semantic_type":id,
                "dependencies":[], "occurrences":[{"role":"type","ordinal":0}],
                "body":{"term":"aggregate","members":[]}
            }));
        id
    };
    let text = scalar("text");
    let rational = scalar("rational");
    let minimum = "170141183460469231731687303715884105728";
    let maximum = "680564733841876926926749214863536422913";
    // The same admitted bounded-quantity shape used by carried_quantity's
    // Package::bounded fixture, with ordinary rational bound literals.
    let bounded = json!({"package":wire["name"], "path":["scalar_type","bounded_quantity",
        unit,"min",rational,minimum,"max",rational,maximum], "ordinal":0});
    let bounded_position = wire["semantic_graph"]["nodes"].as_array().unwrap().len();
    wire["semantic_graph"]["nodes"].as_array_mut().unwrap().push(json!({
        "schema_version":"quire.checked-semantic-graph/v2", "node_id":bounded,
        "node_tag":"scalar_type", "semantic_form":"bounded_quantity", "semantic_type":bounded,
        "dependencies":[unit,rational], "occurrences":[{"role":"type","ordinal":0}],
        "body":{"term":"aggregate","members":[
            {"term":"binding","name":"unit","value":{"term":"reference","target":unit}},
            {"term":"binding","name":"min","value":{"term":"literal","type":rational,"value_kind":"rational","value":minimum}},
            {"term":"binding","name":"max","value":{"term":"literal","type":rational,"value_kind":"rational","value":maximum}}
        ]}
    }));
    let mut append_map = |value: &Value| {
        let id = json!({"package":wire["name"], "path":["composite_type","map",text,value], "ordinal":0});
        let position = wire["semantic_graph"]["nodes"].as_array().unwrap().len();
        wire["semantic_graph"]["nodes"]
            .as_array_mut()
            .unwrap()
            .push(json!({
                "schema_version":"quire.checked-semantic-graph/v2", "node_id":id,
                "node_tag":"composite_type", "semantic_form":"map", "semantic_type":id,
                "dependencies":[text,value], "occurrences":[{"role":"type","ordinal":0}],
                "body":{"term":"aggregate","members":[
                    {"term":"binding","name":"key","value":{"term":"reference","target":text}},
                    {"term":"binding","name":"value","value":{"term":"reference","target":value}}
                ]}
            }));
        (position, id)
    };
    let (inner_position, inner) = append_map(&bounded);
    let (outer_position, _) = append_map(&inner);
    settle(&mut wire);
    index_fixture_node_ids(&mut wire);
    sort_map_fixture_dependencies(&mut wire);
    settle(&mut wire);
    crate::support::checked_package::rebuild_complete_source_map(&mut wire);
    let admitted = read(&wire);
    let CheckedPackageV2ReadResult::Admitted(package) = admitted else {
        panic!("genuine nested bounded quantity maps must strictly admit: {admitted:?}");
    };
    let id = |position: usize| {
        serde_json::from_value::<CheckedNodeId>(
            wire["semantic_graph"]["nodes"][position]["node_id"].clone(),
        )
        .unwrap()
    };
    (
        *package,
        id(outer_position),
        id(inner_position),
        id(bounded_position),
    )
}

/// Trace: FR-357-AC-20.
#[trace("TC-473", "FR-357-AC-20")]
#[test]
fn unsupported_nested_bounded_quantities_refuse_without_comparing_cold_payloads() {
    use quire_contract_model::{NativeJoinBudget, NativeJoinLocusSource};
    use quire_exact::{
        BoundedQuantityType, Integer, MapType, NodeKey, PathSegment, Quantity, Rational, UnitId,
        ValueType,
    };
    let (package, outer, inner, bounded) = nested_bounded_quantity_maps();
    assert_eq!(
        package.node_kinds()[package
            .graph()
            .nodes
            .iter()
            .position(|node| node.node_id == bounded)
            .unwrap()],
        CheckedNodeKind::ScalarType(quire_contract_model::ScalarTypeForm::BoundedQuantity)
    );
    let unit = UnitId::declared(
        NodeKey::new(
            package.name(),
            vec![
                PathSegment::Text("Example".into()),
                PathSegment::Text("Metre".into()),
            ],
            0,
        )
        .unwrap(),
    );
    let leaf = |cold: bool| {
        let quantity = |decimal: &str| {
            let parsed: Integer = decimal.parse().unwrap();
            let magnitude = if cold {
                parsed.try_clone().unwrap()
            } else {
                parsed
            };
            Quantity::new(Rational::from_integer(magnitude), unit.clone())
        };
        ValueType::bounded_quantity(
            BoundedQuantityType::new(
                unit.clone(),
                Some(quantity("170141183460469231731687303715884105728")),
                Some(quantity("680564733841876926926749214863536422913")),
            )
            .unwrap(),
        )
    };
    let rows = |cold: bool| {
        let inner_payload = leaf(cold);
        let outer_payload = leaf(cold);
        let (ValueType::BoundedQuantity(left), ValueType::BoundedQuantity(right)) =
            (&inner_payload, &outer_payload)
        else {
            panic!("bounded payloads")
        };
        assert!(
            !std::sync::Arc::ptr_eq(left, right),
            "the equality operands must be independently owned"
        );
        assert!(
            left.min()
                .unwrap()
                .magnitude()
                .numerator()
                .copy_footprint()
                .bytes
                > 0
        );
        let inner_row = source_map_row(&package, &inner, inner_payload);
        let outer_row = source_map_row(
            &package,
            &outer,
            ValueType::map(MapType::new(outer_payload, None)),
        );
        [inner_row, outer_row]
    };
    let acquire = |cold: bool| {
        SelectedExactIdentities::empty()
            .with_source_collection_rows(
                &package,
                rows(cold),
                &mut NativeJoinBudget::new(1_000_000, 1_000_000),
            )
            .unwrap()
    };
    let mut leaf_budget = NativeJoinBudget::new(1_000_000, 1_000_000);
    let leaf_error = package
        .project_exact_type(
            &bounded,
            &SelectedExactIdentities::empty(),
            &mut leaf_budget,
        )
        .expect_err("bounded quantity Exact leaves remain unsupported");
    assert_eq!(
        leaf_error.code,
        ExactTypeProjectionErrorCode::UnsupportedExactType
    );
    let cold = acquire(true);
    let ordinary = acquire(false);
    let mut cold_budget = NativeJoinBudget::new(1_000_000, 1_000_000);
    let mut ordinary_budget = NativeJoinBudget::new(1_000_000, 1_000_000);
    let (mut cold_result, mut ordinary_result) = (None, None);
    let cold_allocations = allocation_counter::measure(|| {
        cold_result = Some(package.project_exact_type(&outer, &cold, &mut cold_budget));
    });
    let ordinary_allocations = allocation_counter::measure(|| {
        ordinary_result = Some(package.project_exact_type(&outer, &ordinary, &mut ordinary_budget));
    });
    let cold_error = cold_result.unwrap().expect_err("unsupported nested leaf");
    let ordinary_error = ordinary_result
        .unwrap()
        .expect_err("unsupported nested leaf");
    assert_eq!(
        cold_error.code,
        ExactTypeProjectionErrorCode::UnsupportedExactType
    );
    assert_eq!(ordinary_error.code, cold_error.code);
    assert_eq!(cold_budget.consumed(), ordinary_budget.consumed());
    assert_eq!(
        cold_allocations.count_total, ordinary_allocations.count_total,
        "unsupported leaf refusal must not initialize independently owned cold BQ equality caches"
    );
    assert_eq!(
        cold_allocations.bytes_total,
        ordinary_allocations.bytes_total
    );
    assert_eq!(cold_budget.last_success_delta(), None);
    let storage = cold_budget.consumed().storage;
    for limit in [0, storage.saturating_sub(1)] {
        let cold = acquire(true);
        let ordinary = acquire(false);
        let mut cold_budget = NativeJoinBudget::new(1_000_000, limit);
        let mut ordinary_budget = NativeJoinBudget::new(1_000_000, limit);
        let (mut cold_result, mut ordinary_result) = (None, None);
        let cold_allocations = allocation_counter::measure(|| {
            cold_result = Some(package.project_exact_type(&outer, &cold, &mut cold_budget));
        });
        let ordinary_allocations = allocation_counter::measure(|| {
            ordinary_result =
                Some(package.project_exact_type(&outer, &ordinary, &mut ordinary_budget));
        });
        let error = cold_result
            .unwrap()
            .expect_err("resource or unsupported refusal");
        let control = ordinary_result.unwrap().expect_err("same refusal");
        assert_eq!(error.code, control.code);
        assert_eq!(
            cold_allocations.count_total,
            ordinary_allocations.count_total
        );
        assert_eq!(
            cold_allocations.bytes_total,
            ordinary_allocations.bytes_total
        );
        assert_eq!(cold_budget.first_unpaid(), ordinary_budget.first_unpaid());
        assert_eq!(
            cold_budget.first_unpaid_locus(),
            ordinary_budget.first_unpaid_locus()
        );
        if let Some(denied) = cold_budget.first_unpaid() {
            assert_eq!(error.denied, Some(denied));
            assert_eq!(error.configured_bound, Some(limit));
            let locus = cold_budget.first_unpaid_locus().unwrap();
            assert_eq!(
                locus.source,
                NativeJoinLocusSource::GraphNode(
                    package
                        .graph()
                        .nodes
                        .iter()
                        .position(|node| node.node_id == outer)
                        .unwrap()
                )
            );
        }
        assert_eq!(cold_budget.storage_limit(), limit);
        assert_eq!(cold_budget.last_success_delta(), None);
    }
}
