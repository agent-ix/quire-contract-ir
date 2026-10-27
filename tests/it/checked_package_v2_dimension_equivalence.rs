// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Public typed V2 encoding and strict admission of dimension equivalences.

use crate::support::checked_package::{
    canonical, evidence_for, fixture_source, node_id, nominal_package, pointer, refresh_identity,
    rename_node, sha256_hex, source_owner,
};
use ix_trace_rs::trace;
use quire_canonical::Limits;
use quire_contract_model::{
    encode_checked_package_v2, CheckedCapability, CheckedDiagnosticsV2,
    CheckedPackageIdentityPreimageRefV2, CheckedPackageIdentityPreimageV2, CheckedPackageLockV2,
    CheckedPackageReadLimits, CheckedPackageRefusalCode, CheckedPackageV2,
    CheckedPackageV2ReadResult, CheckedSemanticGraphV2, CheckedSemanticId, CheckedSourceMapEntry,
    NominalIdentityPreimage, ScalarTypeForm,
};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

fn typed<T: DeserializeOwned>(value: &Value) -> T {
    serde_json::from_value(value.clone()).expect("typed fixture")
}

fn dimension(name: &str, terms: Value) -> (Value, String) {
    let preimage = json!({
        "version": "quire.dimension-node/v1",
        "owner": source_owner(&fixture_source()),
        "qualified_declaration": ["Example", name],
        "terms": terms,
    });
    let key = sha256_hex(&canonical(&preimage));
    (preimage, key)
}

fn equivalence(left: &str, right: &str, numerator: &str, denominator: &str) -> (Value, String) {
    let mut ids = [left, right];
    ids.sort();
    let preimage = json!({
        "version": "quire.dimension-equivalence-node/v1",
        "owner": source_owner(&fixture_source()),
        "dimension_node_ids": [node_id(ids[0]), node_id(ids[1])],
        "root_scale": {"numerator": numerator, "denominator": denominator},
    });
    let key = sha256_hex(&canonical(&preimage));
    (preimage, key)
}

fn pair() -> Value {
    let first = dimension("RpcTime", json!([]));
    let second = dimension("FlowTime", json!([]));
    let edge = equivalence(&first.1, &second.1, "1", "1");
    nominal_package(&[first, second, edge])
}

fn encode(value: &Value) -> Vec<u8> {
    let owned: CheckedPackageIdentityPreimageV2 = typed(&value["identity_preimage"]);
    let package_id: CheckedSemanticId = typed(&value["package_id"]);
    let lock: CheckedPackageLockV2 = typed(&value["lock"]);
    let graph: CheckedSemanticGraphV2<Value> = typed(&value["semantic_graph"]);
    let source_map: Vec<CheckedSourceMapEntry> = typed(&value["source_map"]);
    let capabilities: Vec<CheckedCapability> = typed(&value["capability_report"]);
    let diagnostics: CheckedDiagnosticsV2 = typed(&value["diagnostics"]);
    let preimage = CheckedPackageIdentityPreimageRefV2 {
        version: &owned.version,
        edition: &owned.edition,
        profile_selections: &owned.profile_selections,
        definition_selections: &owned.definition_selections,
        model_selections: &owned.model_selections,
        required_features: &owned.required_features,
        dependency_selections: &owned.dependency_selections,
        identity_projection: &graph.nodes,
    };
    encode_checked_package_v2(
        &preimage,
        &package_id,
        &lock,
        &graph,
        quire_contract_model::CheckedPackageSourceCorrespondenceRef {
            source_map: &source_map,
            refinement_step_rows: None,
        },
        &capabilities,
        &diagnostics,
        Limits::new(u64::MAX),
    )
    .expect("typed V2 encoding")
}

fn admit(value: &Value) -> CheckedPackageV2 {
    match CheckedPackageV2::read(
        &encode(value),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(value),
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("expected admission, got {other:?}"),
    }
}

fn refused(value: &Value, path: &str) {
    match CheckedPackageV2::read(
        &canonical(value),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(value),
    ) {
        CheckedPackageV2ReadResult::Refused(result) => {
            assert_eq!(result.code, CheckedPackageRefusalCode::InvalidSemanticGraph);
            assert_eq!(result.cause, None);
            assert_eq!(result.path, Some(pointer(path)));
        }
        other => panic!("expected invalid semantic graph, got {other:?}"),
    }
}

fn malformed(value: &Value, position: usize) {
    match CheckedPackageV2::read(
        &canonical(value),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(value),
    ) {
        CheckedPackageV2ReadResult::Refused(result) => {
            assert_eq!(result.code, CheckedPackageRefusalCode::MalformedWire);
            assert_eq!(result.cause, None);
            assert_eq!(
                result.path,
                Some(pointer(&format!("/semantic_graph/nodes/{position}")))
            );
        }
        other => panic!("expected malformed wire at node {position}, got {other:?}"),
    }
}

fn rekey_equivalence(value: &mut Value, position: usize) {
    let node = &value["semantic_graph"]["nodes"][position];
    let fresh = sha256_hex(&canonical(&node["nominal_identity_preimage"]));
    let old = node["node_id"]["digest"]
        .as_str()
        .expect("node digest")
        .to_owned();
    rename_node(value, &old, &fresh);
    refresh_identity(value);
}

/// Trace: FR-063-AC-1, FR-063-AC-2, FR-063-AC-6
#[trace("FR-063-AC-1", "FR-063-AC-2", "FR-063-AC-6")]
#[test]
fn typed_pair_round_trips_with_normalized_key_and_anchor() {
    let value = pair();
    let bytes = encode(&value);
    assert_eq!(bytes, canonical(&value));
    let admitted = admit(&value);
    let edge = admitted.graph().nodes.last().expect("equivalence");
    assert_eq!(edge.semantic_form.as_ref(), "dimension_equivalence");
    assert_eq!(edge.semantic_type, edge.node_id);
    assert!(edge.declaration.is_none());
    assert_eq!(edge.occurrences.len(), 1);
    assert_eq!(edge.occurrences[0].ordinal, 0);
    assert_eq!(admitted.source_map().last().expect("anchor").ordinal, 0);
    assert_eq!(
        admitted.node_kinds().last(),
        Some(&quire_contract_model::CheckedNodeKind::ScalarType(
            ScalarTypeForm::DimensionEquivalence
        ))
    );
    let Some(NominalIdentityPreimage::DimensionEquivalence(preimage)) =
        &edge.nominal_identity_preimage
    else {
        panic!("typed equivalence preimage")
    };
    assert_eq!(preimage.dimension_node_ids.as_slice(), edge.dependencies);
    assert!(preimage.dimension_node_ids[0].digest < preimage.dimension_node_ids[1].digest);
    assert_eq!(preimage.root_scale.numerator.as_ref(), "1");
    assert_eq!(preimage.root_scale.denominator.as_ref(), "1");
    let source = fixture_source();
    assert_eq!(
        preimage.owner.authority.as_ref(),
        source["authority"].as_str().expect("authority")
    );
    assert_eq!(
        preimage.owner.identity.as_ref(),
        source["identity"].as_str().expect("identity")
    );
    assert_eq!(
        edge.nominal_identity_preimage
            .as_ref()
            .expect("preimage")
            .digest(u64::MAX)
            .expect("canonical key"),
        edge.node_id.digest.as_ref()
    );
    let mut retained = value.clone();
    retained["semantic_graph"] = serde_json::to_value(admitted.graph()).expect("retained graph");
    assert_eq!(encode(&retained), bytes);
    assert_eq!(
        admitted.package_id(),
        &typed::<CheckedSemanticId>(&value["package_id"])
    );
}

/// Trace: FR-063-AC-5
#[trace("FR-063-AC-5")]
#[test]
fn equivalence_schema_defects_refuse_at_the_node_before_identity_checks() {
    let value = pair();
    let mut absent = value.clone();
    absent["semantic_graph"]["nodes"][2]["nominal_identity_preimage"]
        .as_object_mut()
        .expect("preimage")
        .remove("root_scale");
    malformed(&absent, 2);

    let mut one_id = value.clone();
    one_id["semantic_graph"]["nodes"][2]["nominal_identity_preimage"]["dimension_node_ids"]
        .as_array_mut()
        .expect("dimension ids")
        .pop();
    malformed(&one_id, 2);

    for member in ["numerator", "denominator"] {
        for spelling in ["0", "-1"] {
            let mut invalid = value.clone();
            invalid["semantic_graph"]["nodes"][2]["nominal_identity_preimage"]["root_scale"]
                [member] = json!(spelling);
            malformed(&invalid, 2);
        }
    }

    let mut declared = value;
    declared["semantic_graph"]["nodes"][2]["declaration"] =
        json!({"qualified_name":["Example","TimeJoin"]});
    malformed(&declared, 2);

    let mut null_declaration = declared;
    null_declaration["semantic_graph"]["nodes"][2]["declaration"] = Value::Null;
    refresh_identity(&mut null_declaration);
    malformed(&null_declaration, 2);
}

/// Trace: FR-063-AC-5
#[trace("FR-063-AC-5")]
#[test]
fn equivalence_graph_defects_refuse_at_the_node_after_wire_checks() {
    let value = pair();

    let mut unreduced = value.clone();
    unreduced["semantic_graph"]["nodes"][2]["nominal_identity_preimage"]["root_scale"] =
        json!({"numerator":"2","denominator":"2"});
    rekey_equivalence(&mut unreduced, 2);
    refused(&unreduced, "/semantic_graph/nodes/2");

    let mut missing_dependency = value.clone();
    missing_dependency["semantic_graph"]["nodes"][2]["dependencies"]
        .as_array_mut()
        .expect("dependencies")
        .pop();
    refresh_identity(&mut missing_dependency);
    refused(&missing_dependency, "/semantic_graph/nodes/2");

    let mut non_self = value.clone();
    non_self["semantic_graph"]["nodes"][2]["semantic_type"] =
        value["semantic_graph"]["nodes"][0]["node_id"].clone();
    refresh_identity(&mut non_self);
    refused(&non_self, "/semantic_graph/nodes/2");

    let mut unselected_owner = value.clone();
    unselected_owner["semantic_graph"]["nodes"][2]["nominal_identity_preimage"]["owner"]
        ["identity"] = json!("absent-source");
    rekey_equivalence(&mut unselected_owner, 2);
    refused(&unselected_owner, "/semantic_graph/nodes/2");

    let mut stale_scale = value.clone();
    stale_scale["semantic_graph"]["nodes"][2]["nominal_identity_preimage"]["root_scale"]
        ["numerator"] = json!("2");
    refresh_identity(&mut stale_scale);
    refused(&stale_scale, "/semantic_graph/nodes/2");

    let mut stale_pair = value;
    let first_id = stale_pair["semantic_graph"]["nodes"][0]["node_id"].clone();
    stale_pair["semantic_graph"]["nodes"][2]["nominal_identity_preimage"]["dimension_node_ids"]
        [1] = first_id;
    refresh_identity(&mut stale_pair);
    refused(&stale_pair, "/semantic_graph/nodes/2");
}

/// Trace: FR-063-AC-5
#[trace("FR-063-AC-5")]
#[test]
fn equivalence_wire_defect_precedes_independent_graph_defect() {
    let mut value = pair();
    let first_id = value["semantic_graph"]["nodes"][0]["node_id"].clone();
    value["semantic_graph"]["nodes"][2]["semantic_type"] = first_id;
    value["semantic_graph"]["nodes"][2]["nominal_identity_preimage"]["root_scale"]["numerator"] =
        json!("0");
    refresh_identity(&mut value);
    malformed(&value, 2);

    value["semantic_graph"]["nodes"][2]["nominal_identity_preimage"]["root_scale"]["numerator"] =
        json!("1");
    refresh_identity(&mut value);
    refused(&value, "/semantic_graph/nodes/2");
}

/// Trace: FR-063-AC-5
#[trace("FR-063-AC-5")]
#[test]
fn nominal_negative_zero_spellings_refuse_at_wire_stage() {
    let first = dimension("RpcTime", json!([]));
    let derived = dimension(
        "RpcTimeSquared",
        json!([{"dimension_node_id":node_id(&first.1),"exponent":"2"}]),
    );
    let mut bad_dimension = nominal_package(&[first.clone(), derived]);
    bad_dimension["semantic_graph"]["nodes"][1]["nominal_identity_preimage"]["terms"][0]
        ["exponent"] = json!("-0");
    malformed(&bad_dimension, 1);

    let unit = json!({
        "version":"quire.unit-node/v1",
        "owner":source_owner(&fixture_source()),
        "qualified_declaration":["Example","Second"],
        "dimension_node_id":node_id(&first.1),
        "target_unit_node_id":null,
        "scale":{"numerator":"1","denominator":"1"},
        "offset":{"numerator":"0","denominator":"1"},
    });
    let key = sha256_hex(&canonical(&unit));
    let value = nominal_package(&[first, (unit, key)]);
    for field in ["scale", "offset"] {
        for member in ["numerator", "denominator"] {
            let mut bad_unit = value.clone();
            bad_unit["semantic_graph"]["nodes"][1]["nominal_identity_preimage"][field][member] =
                json!("-0");
            malformed(&bad_unit, 1);
        }
    }
}

/// Trace: FR-063-AC-2
#[trace("FR-063-AC-2")]
#[test]
fn reversed_source_operands_preserve_the_public_writer_and_reader_identity() {
    let first = dimension("RpcTime", json!([]));
    let second = dimension("FlowTime", json!([]));
    let mut canonical_ids = [first.1.as_str(), second.1.as_str()];
    canonical_ids.sort();

    let forward = nominal_package(&[
        first.clone(),
        second.clone(),
        equivalence(canonical_ids[0], canonical_ids[1], "2", "1"),
    ]);

    // The independently authored reverse declaration says high/low = 1/2.
    // Inverting its ratio when ordering the retained pair yields low/high = 2/1.
    let reverse_operands = [canonical_ids[1], canonical_ids[0]];
    let reverse_ratio = ("1", "2");
    assert!(reverse_operands[0] > reverse_operands[1]);
    let reversed_preimage = json!({
        "version": "quire.dimension-equivalence-node/v1",
        "owner": source_owner(&fixture_source()),
        "dimension_node_ids": [node_id(reverse_operands[1]), node_id(reverse_operands[0])],
        "root_scale": {
            "numerator": reverse_ratio.1,
            "denominator": reverse_ratio.0,
        },
    });
    let reversed_key = sha256_hex(&canonical(&reversed_preimage));
    let reverse = nominal_package(&[first, second, (reversed_preimage, reversed_key)]);

    let forward_bytes = encode(&forward);
    let reverse_bytes = encode(&reverse);
    assert_eq!(forward_bytes, reverse_bytes);
    let forward_admitted = admit(&forward);
    let reverse_admitted = admit(&reverse);
    let forward_edge = forward_admitted.graph().nodes.last().expect("forward edge");
    let reverse_edge = reverse_admitted.graph().nodes.last().expect("reverse edge");
    assert_eq!(forward_edge.node_id, reverse_edge.node_id);
    assert_eq!(forward_edge.dependencies, reverse_edge.dependencies);
    assert_eq!(
        forward_edge.nominal_identity_preimage,
        reverse_edge.nominal_identity_preimage
    );
    assert_eq!(forward_admitted.package_id(), reverse_admitted.package_id());
}

/// Trace: FR-063-AC-3, FR-063-AC-4
#[trace("FR-063-AC-3", "FR-063-AC-4")]
#[test]
fn three_roots_admit_one_exact_class_and_bad_cycle_refuses() {
    let dimensions = [
        dimension("RpcTime", json!([])),
        dimension("FlowTime", json!([])),
        dimension("ClockTime", json!([])),
    ];
    let mut keys = dimensions
        .iter()
        .map(|member| member.1.as_str())
        .collect::<Vec<_>>();
    keys.sort();
    let mut members = dimensions.to_vec();
    members.push(equivalence(keys[0], keys[1], "1", "1"));
    members.push(equivalence(keys[0], keys[2], "1000", "1"));
    members.push(equivalence(keys[1], keys[2], "1000", "1"));
    let value = nominal_package(&members);
    let admitted = admit(&value);
    assert_eq!(admitted.graph().nodes.len(), 6);
    assert_eq!(
        admitted
            .graph()
            .nodes
            .iter()
            .skip(3)
            .map(|node| node.occurrences[0].ordinal)
            .collect::<Vec<_>>(),
        [0, 1, 2]
    );
    let mut bad = value.clone();
    bad["semantic_graph"]["nodes"][5]["nominal_identity_preimage"]["root_scale"]["numerator"] =
        json!("1001");
    rekey_equivalence(&mut bad, 5);
    refused(&bad, "/semantic_graph/nodes/5");
}

/// Trace: FR-063-AC-2, FR-063-AC-4
#[trace("FR-063-AC-2", "FR-063-AC-4")]
#[test]
fn stale_equal_descending_and_repeated_pairs_refuse() {
    let value = pair();
    let mut stale = value.clone();
    stale["semantic_graph"]["nodes"][2]["nominal_identity_preimage"]["root_scale"]["numerator"] =
        json!("2");
    refresh_identity(&mut stale);
    refused(&stale, "/semantic_graph/nodes/2");

    let mut stale_pair = value.clone();
    stale_pair["semantic_graph"]["nodes"][2]["nominal_identity_preimage"]["dimension_node_ids"]
        [1] = value["semantic_graph"]["nodes"][0]["node_id"].clone();
    refresh_identity(&mut stale_pair);
    refused(&stale_pair, "/semantic_graph/nodes/2");

    let first = value["semantic_graph"]["nodes"][0]["node_id"].clone();
    let mut equal = value.clone();
    equal["semantic_graph"]["nodes"][2]["nominal_identity_preimage"]["dimension_node_ids"] =
        json!([first.clone(), first]);
    rekey_equivalence(&mut equal, 2);
    refused(&equal, "/semantic_graph/nodes/2");

    let mut descending = value.clone();
    descending["semantic_graph"]["nodes"][2]["nominal_identity_preimage"]["dimension_node_ids"]
        .as_array_mut()
        .expect("pair")
        .reverse();
    rekey_equivalence(&mut descending, 2);
    refused(&descending, "/semantic_graph/nodes/2");

    let first = dimension("RpcTime", json!([]));
    let second = dimension("FlowTime", json!([]));
    let repeated = nominal_package(&[
        first.clone(),
        second.clone(),
        equivalence(&first.1, &second.1, "1", "1"),
        equivalence(&first.1, &second.1, "2", "1"),
    ]);
    refused(&repeated, "/semantic_graph/nodes/3");
}

/// Trace: FR-063-AC-4
#[trace("FR-063-AC-4")]
#[test]
fn unit_key_and_unequal_derived_maps_refuse() {
    let first = dimension("RpcTime", json!([]));
    let second = dimension("FlowTime", json!([]));
    let unit = json!({
        "version":"quire.unit-node/v1",
        "owner":source_owner(&fixture_source()),
        "qualified_declaration":["Example","Second"],
        "dimension_node_id":node_id(&second.1),
        "target_unit_node_id":null,
        "scale":{"numerator":"1","denominator":"1"},
        "offset":{"numerator":"0","denominator":"1"},
    });
    let unit_key = sha256_hex(&canonical(&unit));
    let wrong_kind = nominal_package(&[
        first.clone(),
        second.clone(),
        (unit, unit_key.clone()),
        equivalence(&first.1, &unit_key, "1", "1"),
    ]);
    refused(&wrong_kind, "/semantic_graph/nodes/3");

    let derived_first = dimension(
        "RpcSquared",
        json!([
            {"dimension_node_id":node_id(&first.1),"exponent":"2"}
        ]),
    );
    let derived_second = dimension(
        "FlowCubed",
        json!([
            {"dimension_node_id":node_id(&second.1),"exponent":"3"}
        ]),
    );
    let mismatch = nominal_package(&[
        first,
        second,
        derived_first.clone(),
        derived_second.clone(),
        equivalence(&derived_first.1, &derived_second.1, "1", "1"),
    ]);
    refused(&mismatch, "/semantic_graph/nodes/4");
}

/// Trace: FR-063-AC-4
#[trace("FR-063-AC-4")]
#[test]
fn derived_ratio_is_fixed_by_base_roots() {
    let first = dimension("RpcTime", json!([]));
    let second = dimension("FlowTime", json!([]));
    let mut base_keys = [first.1.clone(), second.1.clone()];
    base_keys.sort();
    let derived_first = dimension(
        "RpcDerived",
        json!([
            {"dimension_node_id":node_id(&base_keys[0]),"exponent":"1"}
        ]),
    );
    let derived_second = dimension(
        "FlowDerived",
        json!([
            {"dimension_node_id":node_id(&base_keys[1]),"exponent":"1"}
        ]),
    );
    let mut derived_keys = [derived_first.1.clone(), derived_second.1.clone()];
    derived_keys.sort();
    let (numerator, denominator) = if derived_keys[0] == derived_first.1 {
        ("2", "1")
    } else {
        ("1", "2")
    };
    let members = [
        first,
        second,
        derived_first,
        derived_second,
        equivalence(&base_keys[0], &base_keys[1], "2", "1"),
        equivalence(&derived_keys[0], &derived_keys[1], numerator, denominator),
    ];
    let consistent = nominal_package(&members);
    assert_eq!(admit(&consistent).graph().nodes.len(), 6);
    let mut inconsistent = consistent.clone();
    inconsistent["semantic_graph"]["nodes"][5]["nominal_identity_preimage"]["root_scale"] =
        json!({"numerator":"3","denominator":"1"});
    rekey_equivalence(&mut inconsistent, 5);
    refused(&inconsistent, "/semantic_graph/nodes/5");
}

/// Trace: FR-063-AC-1, FR-063-AC-3
#[trace("FR-063-AC-1", "FR-063-AC-3")]
#[test]
fn anchor_ordinal_follows_source_declaration_order() {
    let mut value = pair();
    value["semantic_graph"]["nodes"][2]["occurrences"][0]["ordinal"] = json!(1);
    value["source_map"][2]["ordinal"] = json!(1);
    refused(&value, "/semantic_graph/nodes/2/occurrences");
}
