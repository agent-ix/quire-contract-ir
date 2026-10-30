// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Structural dimension-equivalence admission and refusal controls. QSpec's
//! three-package XP-01 graph runs in the conformance target.

use crate::support::checked_package::{
    canonical, evidence_for, fixture_source, pointer, v2_all_families,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedPackageReadLimits, CheckedPackageRefusalCause as Cause,
    CheckedPackageRefusalCode as Code, CheckedPackageV2, CheckedPackageV2ReadResult,
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
