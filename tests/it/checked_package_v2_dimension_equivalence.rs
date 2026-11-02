// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Structural dimension-equivalence admission and refusal controls. QSpec's
//! three-package XP-01 graph runs in the conformance target.

use crate::support::checked_package::{
    admitted_dependency, canonical, evidence_for, fixture_source, index_fixture_node_ids, pointer,
    read_with_dependencies, refresh_identity, v2_all_families,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    encode_checked_package_v2, CheckedPackageReadLimits, CheckedPackageRefusalCause as Cause,
    CheckedPackageRefusalCode as Code, CheckedPackageSourceCorrespondenceRef, CheckedPackageV2,
    CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};

fn dimension(name: &str, source: &Value) -> Value {
    let id = json!({"package":"test/families","path":["Measure",name],"ordinal":0});
    json!({
        "node_id":id,
        "schema_version":"quire.checked-semantic-graph/v2",
        "node_tag":"scalar_type", "semantic_form":"dimension",
        "semantic_type":id, "dependencies":[],
        "declaration":{"qualified_name":["Measure",name]},
        "occurrences":[{"role":"declaration","ordinal":0}],
        "owner":{"kind":"source","authority":source["authority"],"identity":source["identity"]},
        "body":{"term":"dimension","terms":[]}
    })
}

fn equivalence(left: &Value, right: &Value, source: &Value) -> Value {
    let id = json!({
        "package":"test/families",
        "path":["scalar_type","dimension_equivalence",left,right],
        "ordinal":0
    });
    json!({
        "node_id":id,
        "schema_version":"quire.checked-semantic-graph/v2",
        "node_tag":"scalar_type", "semantic_form":"dimension_equivalence",
        "semantic_type":id, "dependencies":[left,right],
        "occurrences":[{"role":"anchor","ordinal":0}],
        "owner":{"kind":"source","authority":source["authority"],"identity":source["identity"]},
        "body":{"term":"dimension_equivalence","dimension_node_ids":[left,right],
            "root_scale":{"numerator":"1","denominator":"1"}}
    })
}

pub(super) fn package() -> Value {
    let mut package = v2_all_families();
    let source = fixture_source();
    let scalar = package["semantic_graph"]["nodes"][0].clone();
    let alpha = dimension("Alpha", &source);
    let beta = dimension("Beta", &source);
    let pair = equivalence(&alpha["node_id"], &beta["node_id"], &source);
    let nodes = vec![scalar, alpha, beta, pair];
    let source_map = nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            json!({
                "node_id":node["node_id"],
                "role":node["occurrences"][0]["role"],
                "ordinal":0,
                "regions":[{"source":source,"start":index,"end":index+1}]
            })
        })
        .collect::<Vec<_>>();
    package["semantic_graph"]["nodes"] = json!(nodes);
    package["source_map"] = json!(source_map);
    index_fixture_node_ids(&mut package);
    crate::support::checked_package::refresh_temporal_bindings(&mut package);
    package
}

fn read(value: &Value) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(
        &canonical(value),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(value),
    )
}

fn expect_refusal(value: &Value, code: Code, cause: Option<Cause>, at: &str) {
    let CheckedPackageV2ReadResult::Refused(refusal) = read(value) else {
        panic!("adverse equivalence package must refuse");
    };
    assert_eq!(refusal.code, code, "{refusal:?}");
    assert_eq!(refusal.cause, cause, "{refusal:?}");
    assert_eq!(refusal.loci[0].path, Some(pointer(at)), "{refusal:?}");
}

fn rename_package(value: &mut Value, old: &str, new: &str) {
    match value {
        Value::String(text) if text == old => *text = new.to_owned(),
        Value::Array(items) => {
            for item in items {
                rename_package(item, old, new);
            }
        }
        Value::Object(members) => {
            for item in members.values_mut() {
                rename_package(item, old, new);
            }
        }
        _ => {}
    }
}

fn mixed_package() -> (Value, String, CheckedPackageV2) {
    let (dependency_name, dependency) = admitted_dependency(&package());
    let mut importer = package();
    rename_package(&mut importer, "test/families", "test/importer");
    let local = importer["semantic_graph"]["nodes"][1]["node_id"].clone();
    let foreign = dependency.graph().nodes[2].node_id.clone();
    let foreign = serde_json::to_value(foreign).expect("foreign id");
    let equivalence = &mut importer["semantic_graph"]["nodes"][3];
    let pair = json!([{"package":dependency_name,"node":foreign}, local]);
    let id = json!({
        "package":"test/importer",
        "path":["scalar_type","dimension_equivalence","acme/clock","Beta","test/importer","Alpha"],
        "ordinal":0
    });
    equivalence["node_id"] = id.clone();
    equivalence["semantic_type"] = id.clone();
    equivalence["dependencies"] = json!([local]);
    equivalence["body"]["dimension_node_ids"] = pair;
    importer["source_map"][3]["node_id"] = id;
    importer["lock"]["dependency_selections"] = json!([{"identity":dependency_name}]);
    refresh_identity(&mut importer);
    (importer, dependency_name, dependency)
}

#[trace("TC-280", "FR-038-AC-387", "FR-322-AC-299")]
#[test]
fn selected_dimension_operand_admits_and_retains_package_form() {
    let (wire, name, dependency) = mixed_package();
    let CheckedPackageV2ReadResult::Admitted(admitted) =
        read_with_dependencies(&wire, &[(name.as_str(), &dependency)])
    else {
        panic!("directly selected foreign dimension must admit");
    };
    let retained = serde_json::to_value(&admitted.graph().nodes[3]).expect("retained node");
    assert_eq!(
        retained["body"]["dimension_node_ids"],
        wire["semantic_graph"]["nodes"][3]["body"]["dimension_node_ids"]
    );
    assert_eq!(
        retained["dependencies"],
        wire["semantic_graph"]["nodes"][3]["dependencies"]
    );
    let encoded = encode_checked_package_v2(
        admitted.name(),
        admitted.lock(),
        admitted.graph(),
        CheckedPackageSourceCorrespondenceRef {
            source_map: admitted.source_map(),
            refinement_step_rows: admitted.refinement_step_rows(),
        },
        quire_contract_model::CheckedPackageSidecarsRef {
            operation_effects: None,
            checked_term_data: admitted.checked_term_data(),
            temporal_bindings: admitted.temporal_bindings(),
        },
        admitted.capability_report(),
        admitted.diagnostics(),
        quire_canonical::Limits::new(u64::MAX),
    )
    .expect("canonical writer admits the mixed dimension operands");
    let encoded: Value = serde_json::from_slice(&encoded).expect("canonical package JSON");
    assert_eq!(
        encoded["semantic_graph"]["nodes"][3]["body"]["dimension_node_ids"],
        wire["semantic_graph"]["nodes"][3]["body"]["dimension_node_ids"],
        "canonical writer preserves both authored operand forms"
    );
    assert_eq!(
        encoded["semantic_graph"]["nodes"][3]["dependencies"],
        wire["semantic_graph"]["nodes"][3]["dependencies"],
        "canonical writer lists only the local operand as a graph dependency"
    );
}

#[trace("TC-280", "FR-038-AC-388")]
#[test]
fn package_operand_requires_direct_selection_and_declared_node() {
    let (wire, name, dependency) = mixed_package();
    let mut unselected = wire.clone();
    unselected["lock"]["dependency_selections"] = json!([]);
    let CheckedPackageV2ReadResult::Refused(refusal) = read_with_dependencies(&unselected, &[])
    else {
        panic!("unselected package operand must refuse");
    };
    assert_eq!(refusal.code, Code::MissingDeclaration, "{refusal:?}");
    assert_eq!(refusal.cause, Some(Cause::MissingSelection), "{refusal:?}");
    assert_eq!(
        refusal.loci[0].path,
        Some(pointer(
            "/semantic_graph/nodes/3/body/dimension_node_ids/0/package"
        )),
        "{refusal:?}"
    );

    let mut missing = wire.clone();
    missing["semantic_graph"]["nodes"][3]["body"]["dimension_node_ids"][0]["node"]["path"] =
        json!(["Missing"]);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_with_dependencies(&missing, &[(name.as_str(), &dependency)])
    else {
        panic!("absent selected node must refuse");
    };
    assert_eq!(refusal.code, Code::MissingDeclaration, "{refusal:?}");
    assert_eq!(refusal.cause, Some(Cause::MissingName), "{refusal:?}");
    assert_eq!(
        refusal.loci[0].path,
        Some(pointer(
            "/semantic_graph/nodes/3/body/dimension_node_ids/0/node"
        )),
        "{refusal:?}"
    );

    let mut wrong_package = wire;
    wrong_package["semantic_graph"]["nodes"][3]["body"]["dimension_node_ids"][0]["node"]
        ["package"] = json!("another/package");
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_with_dependencies(&wrong_package, &[(name.as_str(), &dependency)])
    else {
        panic!("a node outside the selected package must refuse");
    };
    assert_eq!(refusal.code, Code::MissingDeclaration, "{refusal:?}");
    assert_eq!(refusal.cause, Some(Cause::MissingName), "{refusal:?}");
    assert_eq!(
        refusal.loci[0].path,
        Some(pointer(
            "/semantic_graph/nodes/3/body/dimension_node_ids/0/node"
        )),
        "{refusal:?}"
    );
}

#[trace("TC-280", "FR-038-AC-389")]
#[test]
fn foreign_operand_wrong_kind_and_self_minted_local_refuse_at_operand() {
    let (wire, name, dependency) = mixed_package();
    let mut wrong_kind = wire.clone();
    wrong_kind["semantic_graph"]["nodes"][3]["body"]["dimension_node_ids"][0]["node"] =
        serde_json::to_value(&dependency.graph().nodes[0].node_id).expect("non-dimension id");
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_with_dependencies(&wrong_kind, &[(name.as_str(), &dependency)])
    else {
        panic!("selected non-dimension operand must refuse");
    };
    assert_eq!(refusal.code, Code::InvalidSemanticGraph, "{refusal:?}");
    assert_eq!(refusal.cause, None, "{refusal:?}");
    assert_eq!(
        refusal.loci[0].path,
        Some(pointer("/semantic_graph/nodes/3/body/dimension_node_ids/0")),
        "{refusal:?}"
    );

    let mut self_minted = wire;
    let foreign =
        self_minted["semantic_graph"]["nodes"][3]["body"]["dimension_node_ids"][0]["node"].clone();
    self_minted["semantic_graph"]["nodes"][3]["body"]["dimension_node_ids"][0] = foreign;
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_with_dependencies(&self_minted, &[(name.as_str(), &dependency)])
    else {
        panic!("foreign node written as a local ref must refuse");
    };
    assert_eq!(refusal.code, Code::InvalidSemanticGraph, "{refusal:?}");
    assert_eq!(refusal.cause, None, "{refusal:?}");
    assert_eq!(
        refusal.loci[0].path,
        Some(pointer("/semantic_graph/nodes/3/body/dimension_node_ids/0")),
        "{refusal:?}"
    );
}

/// A typed equivalence retains its exact structural id, pair, reduced scale,
/// two dependencies, source owner and anchor occurrence through the reader.
#[trace("TC-048", "FR-063-AC-1", "FR-063-AC-6")]
#[test]
fn typed_equivalence_retains_complete_structural_node() {
    let wire = package();
    let CheckedPackageV2ReadResult::Admitted(admitted) = read(&wire) else {
        panic!("well-formed equivalence package must admit");
    };
    let retained = serde_json::to_value(&admitted.graph().nodes[3]).expect("retained node");
    let expected = &wire["semantic_graph"]["nodes"][3];
    assert_eq!(&retained, expected);
    assert_eq!(
        retained["dependencies"],
        retained["body"]["dimension_node_ids"]
    );
    assert_eq!(retained["semantic_type"], retained["node_id"]);
    assert!(retained.get("declaration").is_none());
    assert_eq!(
        retained["occurrences"],
        json!([{"role":"anchor","ordinal":0}])
    );
}

/// The scalar body is closed, and malformed body/owner shape wins over graph
/// consistency checks at the node where it occurs.
#[trace("TC-048", "FR-063-AC-5")]
#[test]
fn malformed_equivalence_shape_refuses_at_its_node() {
    let base = package();
    type Case = (fn(&mut Value), &'static str);
    let cases: [Case; 4] = [
        (
            |v: &mut Value| {
                v["semantic_graph"]["nodes"][3]["body"]
                    .as_object_mut()
                    .unwrap()
                    .remove("root_scale");
            },
            "/semantic_graph/nodes/3/body",
        ),
        (
            |v: &mut Value| {
                v["semantic_graph"]["nodes"][3]["body"]["dimension_node_ids"] =
                    json!([v["semantic_graph"]["nodes"][1]["node_id"]]);
            },
            "/semantic_graph/nodes/3/body/dimension_node_ids",
        ),
        (
            |v: &mut Value| {
                v["semantic_graph"]["nodes"][3]
                    .as_object_mut()
                    .unwrap()
                    .remove("owner");
            },
            "/semantic_graph/nodes/3",
        ),
        (
            |v: &mut Value| {
                v["semantic_graph"]["nodes"][3]["declaration"] =
                    json!({"qualified_name":["Measure","Pair"]});
            },
            "/semantic_graph/nodes/3",
        ),
    ];
    for (mutate, at) in cases {
        let mut adverse = base.clone();
        mutate(&mut adverse);
        expect_refusal(&adverse, Code::MalformedWire, None, at);
    }
}

/// Equal or reversed pair ids and an unreduced scale are graph failures,
/// rather than stale-key refusals from the retired content identity scheme.
#[trace("TC-048", "FR-063-AC-4", "FR-063-AC-5")]
#[test]
fn invalid_pair_and_scale_refuse_at_equivalence_node() {
    let base = package();
    let mut equal = base.clone();
    equal["semantic_graph"]["nodes"][3]["body"]["dimension_node_ids"] = json!([
        base["semantic_graph"]["nodes"][1]["node_id"],
        base["semantic_graph"]["nodes"][1]["node_id"]
    ]);
    expect_refusal(
        &equal,
        Code::InvalidSemanticGraph,
        None,
        "/semantic_graph/nodes/3",
    );

    let mut reversed = base.clone();
    reversed["semantic_graph"]["nodes"][3]["body"]["dimension_node_ids"] = json!([
        base["semantic_graph"]["nodes"][2]["node_id"],
        base["semantic_graph"]["nodes"][1]["node_id"]
    ]);
    expect_refusal(
        &reversed,
        Code::InvalidSemanticGraph,
        None,
        "/semantic_graph/nodes/3",
    );

    let mut unreduced = base;
    unreduced["semantic_graph"]["nodes"][3]["body"]["root_scale"] =
        json!({"numerator":"2","denominator":"2"});
    expect_refusal(
        &unreduced,
        Code::InvalidSemanticGraph,
        None,
        "/semantic_graph/nodes/3",
    );
}

/// The graph step requires the exact pair as dependencies and a self type;
/// owner resolution precedes those graph checks.
#[trace("TC-048", "FR-063-AC-5")]
#[test]
fn dependency_type_and_owner_joins_refuse_at_their_own_seam() {
    let base = package();
    let mut missing_dependency = base.clone();
    missing_dependency["semantic_graph"]["nodes"][3]["dependencies"] =
        json!([base["semantic_graph"]["nodes"][1]["node_id"]]);
    expect_refusal(
        &missing_dependency,
        Code::InvalidSemanticGraph,
        None,
        "/semantic_graph/nodes/3",
    );

    let mut foreign_type = base.clone();
    foreign_type["semantic_graph"]["nodes"][3]["semantic_type"] =
        base["semantic_graph"]["nodes"][1]["node_id"].clone();
    expect_refusal(
        &foreign_type,
        Code::InvalidSemanticGraph,
        None,
        "/semantic_graph/nodes/3",
    );

    let mut unselected_owner = base;
    unselected_owner["semantic_graph"]["nodes"][3]["owner"]["identity"] =
        json!("test/missing-source");
    expect_refusal(
        &unselected_owner,
        Code::MissingDeclaration,
        Some(Cause::MissingSelection),
        "/semantic_graph/nodes/3/node_id",
    );
}
