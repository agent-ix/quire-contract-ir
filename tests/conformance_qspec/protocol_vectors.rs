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
fn qspec_k2_role_rename_changes_declared_and_operand_ids() {
    let row = vector("K2");
    assert_eq!(row["test_case"], "TC-946");
    let mut package = fixture_support::admitted_protocol_families();
    let clause = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["semantic_form"] == "protocol_clause")
        .expect("protocol clause");
    let role = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    let mut role_node = fixture_support::protocol_role_node();
    role_node["declaration"]["qualified_name"][3] = json!("Fulfillment");
    let original_role_id = role_node["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(role_node);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][3]["value"]["members"] =
        json!([{"term": "reference", "target": original_role_id}]);
    let derived = role + 1;
    let derived_id = fixture_support::structural_id(
        "test/families",
        vec![
            json!("expression"),
            json!("reference"),
            original_role_id.clone(),
        ],
        0,
    );
    let boolean = fixture_support::node_id(&fixture_support::family_key("aaaa"));
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(json!({
            "schema_version": "quire.checked-semantic-graph/v2",
            "node_id": derived_id,
            "node_tag": "expression",
            "semantic_form": "reference",
            "semantic_type": boolean,
            "dependencies": [original_role_id],
            "occurrences": [{"role": "generated", "ordinal": 0}],
            "body": {"term": "reference", "target": original_role_id}
        }));
    fixture_support::settle(&mut package);
    let before_role = package["semantic_graph"]["nodes"][role]["node_id"].clone();
    let before_derived = package["semantic_graph"]["nodes"][derived]["node_id"].clone();
    assert!(
        matches!(
            read_authored(&package),
            CheckedPackageV2ReadResult::Admitted(_)
        ),
        "K2 Fulfillment base admits"
    );
    package["semantic_graph"]["nodes"][role]["declaration"]["qualified_name"][3] =
        json!("Shipping");
    fixture_support::settle(&mut package);
    let CheckedPackageV2ReadResult::Admitted(admitted) = read_authored(&package) else {
        panic!("K2 Shipping rename must admit");
    };
    let after_role = package["semantic_graph"]["nodes"][role]["node_id"].clone();
    let after_derived = package["semantic_graph"]["nodes"][derived]["node_id"].clone();
    assert_ne!(before_role, after_role);
    assert_ne!(before_derived, after_derived);
    assert_eq!(after_derived["path"][2], after_role);
    assert_eq!(
        serde_json::to_value(&admitted.graph().nodes[derived].node_id).expect("retained id"),
        after_derived
    );
}

#[trace("TC-946")]
#[test]
fn qspec_k4_poll_maximum_changes_body_without_changing_declared_id() {
    let row = vector("K4");
    assert_eq!(row["test_case"], "TC-946");
    let mut package = fixture_support::admitted_protocol_families();
    let root = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| {
            node["semantic_form"] == "control"
                && node["declaration"]["qualified_name"]
                    == json!(["Example", "OrderFlow", "run", "Main"])
        })
        .expect("Main control");
    let clause = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["semantic_form"] == "protocol_clause")
        .expect("protocol clause");
    let role = fixture_support::protocol_role_node();
    let role_id = role["node_id"].clone();
    let step_path = ["Example", "OrderFlow", "run", "Main", "Poll", "Step"];
    let mut step_body = package["semantic_graph"]["nodes"][root]["body"].clone();
    step_body["operation"]["member"]["operator"] = json!("fence");
    step_body["arguments"] = json!([{"term": "binding", "name": "ordering", "value": {
        "term": "literal",
        "type": fixture_support::node_id(&fixture_support::family_key("a1a1")),
        "value_kind": "text", "value": "seq_cst"
    }}]);
    let step = fixture_support::protocol_named_node("control", &step_path, step_body.clone(), &[]);
    let step_id = step["node_id"].clone();
    let exhausted_path = ["Example", "OrderFlow", "run", "Main", "Poll", "Exhausted"];
    let exhausted =
        fixture_support::protocol_named_node("control", &exhausted_path, step_body, &[]);
    let exhausted_id = exhausted["node_id"].clone();
    let poll_path = ["Example", "OrderFlow", "run", "Main", "Poll"];
    let mut poll_body = package["semantic_graph"]["nodes"][root]["body"].clone();
    poll_body["operation"]["member"]["operator"] = json!("repeat");
    poll_body["arguments"] = json!([
        {"term": "binding", "name": "owner", "value": {"term": "reference", "target": role_id}},
        {"term": "binding", "name": "visible", "value": {"term": "aggregate", "members": []}},
        {"term": "binding", "name": "maximum", "value": {"term": "aggregate", "members": [
            {"term": "literal",
             "type": fixture_support::node_id(&fixture_support::family_key("a3a3")),
             "value_kind": "integer", "value": "3"}
        ]}},
        {"term": "binding", "name": "proof", "value": {"term": "aggregate", "members": []}},
        {"term": "binding", "name": "guard", "value": {"term": "literal",
            "type": fixture_support::node_id(&fixture_support::family_key("aaaa")),
            "value_kind": "boolean", "value": true}},
        {"term": "binding", "name": "body", "value": {"term": "reference", "target": step_id}},
        {"term": "binding", "name": "exhausted", "value": {"term": "aggregate", "members": [
            {"term": "reference", "target": exhausted_id}
        ]}}
    ]);
    let poll = fixture_support::protocol_named_node("control", &poll_path, poll_body, &[]);
    let poll_id = poll["node_id"].clone();
    let poll_position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend([step, exhausted, role, poll]);
    package["semantic_graph"]["nodes"][root]["body"]["arguments"][0]["value"]["members"] =
        json!([{"term": "reference", "target": poll_id}]);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][3]["value"]["members"] =
        json!([{"term": "reference", "target": role_id}]);
    fixture_support::settle(&mut package);
    let poll_position = poll_position + 3;
    let original_id = package["semantic_graph"]["nodes"][poll_position]["node_id"].clone();
    let original_body = package["semantic_graph"]["nodes"][poll_position]["body"].clone();
    assert!(
        matches!(
            read_authored(&package),
            CheckedPackageV2ReadResult::Admitted(_)
        ),
        "K4 max 3 base admits: {:?}",
        read_authored(&package)
    );
    package["semantic_graph"]["nodes"][poll_position]["body"]["arguments"][2]["value"]["members"]
        [0]["value"] = json!("4");
    fixture_support::settle(&mut package);
    let CheckedPackageV2ReadResult::Admitted(admitted) = read_authored(&package) else {
        panic!("K4 max 4 must admit: {:?}", read_authored(&package));
    };
    assert_ne!(
        package["semantic_graph"]["nodes"][poll_position]["body"],
        original_body
    );
    assert_eq!(
        package["semantic_graph"]["nodes"][poll_position]["node_id"],
        original_id
    );
    assert_eq!(
        serde_json::to_value(&admitted.graph().nodes[poll_position].node_id).expect("retained id"),
        original_id
    );
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
