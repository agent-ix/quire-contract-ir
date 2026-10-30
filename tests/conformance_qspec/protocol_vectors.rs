// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Public-reader controls for QSpec's protocol vector rows.

use super::{
    fixture_support, fixtures_dir, proposal_dir_of, read_json, read_package_with_document,
    DOMAIN_DOCUMENT,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedPackageReadLimits, CheckedPackageV2, CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};

fn vector(id: &str) -> Value {
    let path = proposal_dir_of(&fixtures_dir()).join("fixtures/protocol-node-vectors.json");
    let vectors = read_json(&path).expect("QSpec protocol vectors");
    let cases = vectors["cases"].as_array().expect("QSpec cases");
    let rows = cases
        .iter()
        .filter(|row| row["id"] == id)
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 1, "{id}: one authoritative row");
    rows[0].clone()
}

fn read_authored(package: &Value) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(
        &fixture_support::canonical(package),
        CheckedPackageReadLimits::bounded(),
        &fixture_support::evidence_for(package),
    )
}

#[trace("TC-946")]
#[test]
fn qspec_c4_removing_scheduling_retains_declared_clause_id() {
    let row = vector("C4");
    assert_eq!(row["test_case"], "TC-946");
    let mut package = fixture_support::admitted_protocol_families();
    let clause = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["semantic_form"] == "protocol_clause")
        .expect("protocol clause");
    let original_id = package["semantic_graph"]["nodes"][clause]["node_id"].clone();
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][10]["value"]["members"] = json!([{
        "term": "literal",
        "type": fixture_support::node_id(&fixture_support::family_key("a1a1")),
        "value_kind": "text",
        "value": "adversarial"
    }]);
    fixture_support::settle(&mut package);
    let with_scheduling = package["semantic_graph"]["nodes"][clause]["body"].clone();
    assert!(
        matches!(
            read_authored(&package),
            CheckedPackageV2ReadResult::Admitted(_)
        ),
        "C4 base with scheduling admits"
    );
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][10]["value"]["members"] =
        json!([]);
    fixture_support::settle(&mut package);
    let CheckedPackageV2ReadResult::Admitted(admitted) = read_authored(&package) else {
        panic!("C4 removal of scheduling must admit");
    };
    assert_ne!(
        package["semantic_graph"]["nodes"][clause]["body"],
        with_scheduling
    );
    assert_eq!(
        package["semantic_graph"]["nodes"][clause]["node_id"],
        original_id
    );
    assert_eq!(
        serde_json::to_value(&admitted.graph().nodes[clause].node_id).expect("retained id"),
        original_id
    );
}

#[trace("TC-946")]
#[test]
fn qspec_k10_acyclic_protocol_member_admits_a_lone_group_label() {
    let row = vector("K10");
    assert_eq!(row["test_case"], "TC-946");
    assert!(row["Expected"]
        .as_str()
        .expect("K10 expected result")
        .starts_with("admitted"));
    let fixtures = fixtures_dir();
    let document = proposal_dir_of(&fixtures).join(DOMAIN_DOCUMENT);
    let mut package = read_json(&fixtures.join("positive-clause-operations.json"))
        .expect("published protocol control package");
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("protocol graph");
    let position = nodes
        .iter()
        .position(|node| node["node_tag"] == "protocol" && node["semantic_form"] == "control")
        .expect("published acyclic protocol control");
    assert!(nodes[position]
        .get("recursion_group")
        .is_none_or(Value::is_null));
    assert!(matches!(
        read_package_with_document(&package, "K10 base", &document).expect("base reader"),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    let original_id = package["semantic_graph"]["nodes"][position]["node_id"].clone();
    package["semantic_graph"]["nodes"][position]["recursion_group"] = json!("lone-group");
    let result =
        read_package_with_document(&package, "K10 mutation", &document).expect("mutated reader");
    let CheckedPackageV2ReadResult::Admitted(admitted) = result else {
        panic!("K10 acyclic protocol with a lone group label must admit");
    };
    assert_eq!(
        admitted.graph().nodes[position].recursion_group.as_deref(),
        Some("lone-group")
    );
    assert_eq!(
        serde_json::to_value(&admitted.graph().nodes[position].node_id).expect("retained id"),
        original_id
    );
}
