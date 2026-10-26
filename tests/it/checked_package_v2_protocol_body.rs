// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Public reader checks for the first protocol validation passes.

use crate::support::checked_package::{
    canonical, evidence_for, protocol_named_node, protocol_role_node, rebuild_source_map,
    refresh_identity, refusal, refusal_at, rekey_application_node, settle, v2_all_families,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedPackageReadLimits, CheckedPackageRefusalCause, CheckedPackageRefusalCode,
    CheckedPackageV2, CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};

fn read(package: &Value) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(package),
    )
}

fn clause_position(package: &Value) -> usize {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| {
            node["node_tag"] == "protocol" && node["semantic_form"] == "protocol_clause"
        })
        .expect("protocol clause")
}

fn append_duplicate_clause(package: &mut Value, body: Value) -> (usize, String) {
    let original = clause_position(package);
    let dependencies = package["semantic_graph"]["nodes"][original]["dependencies"]
        .as_array()
        .expect("dependencies")
        .iter()
        .map(|id| id["digest"].as_str().expect("digest").to_owned())
        .collect::<Vec<_>>();
    let duplicate = protocol_named_node(
        "protocol_clause",
        &["Example", "OrderFlow"],
        body,
        &dependencies,
    );
    let key = duplicate["node_id"]["digest"]
        .as_str()
        .expect("digest")
        .to_owned();
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(duplicate);
    rebuild_source_map(package);
    refresh_identity(package);
    (position, key)
}

#[trace("TC-048")]
#[test]
fn protocol_clause_requires_the_selected_profile_law_in_the_public_reader() {
    let mut package = v2_all_families();
    let position = clause_position(&package);
    package["semantic_graph"]["nodes"][position]["body"]["operation"]["laws"] = json!([]);
    rekey_application_node(&mut package, position);
    refresh_identity(&mut package);
    let key = package["semantic_graph"]["nodes"][position]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidPackage,
            &format!("/semantic_graph/nodes/{position}/body/operation/laws"),
            Some(CheckedPackageRefusalCause::OperationLawMissing),
            key,
        )
    );
}

#[trace("TC-048")]
#[test]
fn malformed_protocol_enum_names_the_offending_term() {
    let mut package = v2_all_families();
    let position = clause_position(&package);
    package["semantic_graph"]["nodes"][position]["body"]["arguments"][1]["value"]["members"][0]
        ["value"]["value"] = json!("unknown-activation");
    assert_eq!(
        read(&package),
        refusal(
            CheckedPackageRefusalCode::MalformedWire,
            &format!("/semantic_graph/nodes/{position}/body/arguments/1/value/members/0/value"),
        )
    );

    let mut package = v2_all_families();
    let semantic_type = package["semantic_graph"]["nodes"][position]["semantic_type"].clone();
    package["semantic_graph"]["nodes"][position]["body"]["arguments"][10]["value"]["members"] = json!([{
        "term": "literal", "type": semantic_type,
        "value_kind": "text", "value": "unknown-scheduling"
    }]);
    assert_eq!(
        read(&package),
        refusal(
            CheckedPackageRefusalCode::MalformedWire,
            &format!("/semantic_graph/nodes/{position}/body/arguments/10/value/members/0"),
        )
    );
}

#[trace("TC-048")]
#[test]
fn protocol_operator_refusal_precedes_duplicate_declaration() {
    let mut package = v2_all_families();
    let original = clause_position(&package);
    let mut body = package["semantic_graph"]["nodes"][original]["body"].clone();
    body["operator"] = json!("call");
    let (position, key) = append_duplicate_clause(&mut package, body);
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::IllTyped,
            &format!("/semantic_graph/nodes/{position}/body"),
            Some(CheckedPackageRefusalCause::OperatorIneligible),
            &key,
        )
    );
}

#[trace("TC-048")]
#[test]
fn valid_protocol_operators_still_refuse_duplicate_declarations() {
    let mut package = v2_all_families();
    let original = clause_position(&package);
    let original_key = package["semantic_graph"]["nodes"][original]["node_id"]["digest"]
        .as_str()
        .expect("digest")
        .to_owned();
    let mut body = package["semantic_graph"]["nodes"][original]["body"].clone();
    let semantic_type = package["semantic_graph"]["nodes"][original]["semantic_type"].clone();
    body["arguments"][10]["value"]["members"] = json!([{
        "term": "literal", "type": semantic_type,
        "value_kind": "text", "value": "adversarial"
    }]);
    let (duplicate_position, duplicate_key) = append_duplicate_clause(&mut package, body);
    let (position, key) = if original_key < duplicate_key {
        (duplicate_position, duplicate_key)
    } else {
        (original, original_key)
    };
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::AmbiguousDeclaration,
            &format!("/semantic_graph/nodes/{position}/declaration/qualified_name"),
            Some(CheckedPackageRefusalCause::AmbiguousName),
            &key,
        )
    );
}

fn protocol_position(package: &Value, form: &str) -> usize {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_tag"] == "protocol" && node["semantic_form"] == form)
        .expect("protocol node")
}

#[trace("TC-048")]
#[test]
fn protocol_role_and_root_control_can_share_a_name() {
    let mut package = v2_all_families();
    let mut role = protocol_role_node();
    role["declaration"]["qualified_name"][3] = json!("Main");
    let role_key = role["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(role);
    let clause = clause_position(&package);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][3]["value"]["members"] =
        json!([{"term": "reference", "target": role_key}]);
    settle(&mut package);
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

#[trace("TC-048")]
#[test]
fn protocol_listed_role_must_use_its_clause_and_fixed_segment() {
    for declaration in [
        json!(["Other", "Protocol", "role", "Worker"]),
        json!(["Example", "OrderFlow", "channel", "Worker"]),
    ] {
        let mut package = v2_all_families();
        let mut role = protocol_role_node();
        role["declaration"]["qualified_name"] = declaration;
        let key = role["node_id"].clone();
        let position = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .len();
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(role);
        let clause = clause_position(&package);
        package["semantic_graph"]["nodes"][clause]["body"]["arguments"][3]["value"]["members"] =
            json!([{"term": "reference", "target": key}]);
        settle(&mut package);
        let key = package["semantic_graph"]["nodes"][position]["node_id"]["digest"]
            .as_str()
            .expect("digest");
        assert_eq!(
            read(&package),
            refusal_at(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                &format!("/semantic_graph/nodes/{position}"),
                None,
                key,
            )
        );
    }
}

#[trace("TC-048")]
#[test]
fn nested_control_uses_its_holder_path() {
    let mut package = v2_all_families();
    let root = protocol_position(&package, "control");
    let body = package["semantic_graph"]["nodes"][root]["body"].clone();
    let child = protocol_named_node(
        "control",
        &["Example", "OrderFlow", "run", "Done"],
        body,
        &[],
    );
    let child_key = child["node_id"].clone();
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(child);
    package["semantic_graph"]["nodes"][root]["body"]["arguments"][0]["value"]["members"] =
        json!([{"term": "reference", "target": child_key}]);
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][position]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            &format!("/semantic_graph/nodes/{position}"),
            None,
            key,
        )
    );

    package["semantic_graph"]["nodes"][position]["declaration"]["qualified_name"] =
        json!(["Other", "Protocol", "run", "Main", "Done"]);
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][position]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            &format!("/semantic_graph/nodes/{position}"),
            None,
            key,
        )
    );

    package["semantic_graph"]["nodes"][position]["declaration"]["qualified_name"] =
        json!(["Example", "OrderFlow", "run", "Main", "Done"]);
    settle(&mut package);
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

#[trace("TC-048")]
#[test]
fn protocol_declaration_occurrences_reject_generated_role() {
    let mut package = v2_all_families();
    let position = clause_position(&package);
    package["semantic_graph"]["nodes"][position]["occurrences"][0]["role"] = json!("generated");
    rebuild_source_map(&mut package);
    refresh_identity(&mut package);
    assert_eq!(
        read(&package),
        refusal(
            CheckedPackageRefusalCode::MalformedWire,
            &format!("/semantic_graph/nodes/{position}"),
        )
    );
}

#[trace("TC-048")]
#[test]
fn queue_requires_no_declaration_and_generated_occurrences() {
    for keep_declaration in [true, false] {
        let mut package = v2_all_families();
        let clause = clause_position(&package);
        let target = package["semantic_graph"]["nodes"][clause]["node_id"].clone();
        let text_type = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .find(|node| node["node_tag"] == "scalar_type" && node["semantic_form"] == "text")
            .expect("text type")["node_id"]
            .clone();
        let integer_type = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .find(|node| node["node_tag"] == "scalar_type" && node["semantic_form"] == "integer")
            .expect("integer type")["node_id"]
            .clone();
        let text = |value: &str| json!({"term": "literal", "type": text_type, "value_kind": "text", "value": value});
        let binding =
            |name: &str, value: Value| json!({"term": "binding", "name": name, "value": value});
        let mut node = protocol_role_node();
        node["semantic_form"] = json!("queue");
        node["body"]["operation"]["member"]["operator"] = json!("queue");
        node["body"]["arguments"] = json!([
            binding("channel", json!({"term": "reference", "target": target})),
            binding(
                "capacity",
                json!({"term": "aggregate", "members": [
                    binding("kind", text("literal")),
                    binding("value", json!({"term": "literal", "type": integer_type,
                        "value_kind": "integer", "value": "1"}))
                ]})
            ),
            binding("overflow", text("reject")),
            binding("loss", json!({"term": "aggregate", "members": []})),
        ]);
        if !keep_declaration {
            node.as_object_mut().expect("node").remove("declaration");
        } else {
            node["occurrences"][0]["role"] = json!("generated");
        }
        let position = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .len();
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(node);
        settle(&mut package);
        assert_eq!(
            read(&package),
            refusal(
                CheckedPackageRefusalCode::MalformedWire,
                &format!("/semantic_graph/nodes/{position}"),
            )
        );
    }
}

#[trace("TC-048")]
#[test]
fn duplicate_role_name_precedes_its_member_target_defect() {
    let mut package = v2_all_families();
    let first = protocol_role_node();
    let target = package["semantic_graph"]["nodes"][0]["node_id"].clone();
    let mut second_body = first["body"].clone();
    second_body["arguments"][0]["value"]["target"] = target.clone();
    let second = protocol_named_node(
        "role",
        &["Example", "OrderFlow", "role", "Worker"],
        second_body,
        &[target["digest"].as_str().expect("digest").to_owned()],
    );
    let first_key = first["node_id"].clone();
    let second_key = second["node_id"].clone();
    let base = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend([first, second]);
    let clause = clause_position(&package);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][3]["value"]["members"] = json!([
        {"term": "reference", "target": first_key},
        {"term": "reference", "target": second_key},
    ]);
    settle(&mut package);
    let first_key = package["semantic_graph"]["nodes"][base]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    let second_key = package["semantic_graph"]["nodes"][base + 1]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    let (position, key) = if first_key < second_key {
        (base + 1, second_key)
    } else {
        (base, first_key)
    };
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::AmbiguousDeclaration,
            &format!("/semantic_graph/nodes/{position}/declaration/qualified_name"),
            Some(CheckedPackageRefusalCause::AmbiguousName),
            key,
        )
    );
}

#[trace("TC-048")]
#[test]
fn duplicate_control_names_under_one_holder_refuse_in_digest_order() {
    let mut package = v2_all_families();
    let root = protocol_position(&package, "control");
    let first_body = package["semantic_graph"]["nodes"][root]["body"].clone();
    let mut second_body = first_body.clone();
    second_body["operation"]["member"]["operator"] = json!("check");
    let boolean = package["semantic_graph"]["nodes"][root]["semantic_type"].clone();
    second_body["arguments"] = json!([{
        "term": "binding", "name": "condition", "value": {
            "term": "literal", "type": boolean, "value_kind": "boolean", "value": true
        }
    }]);
    let path = &["Example", "OrderFlow", "run", "Main", "Done"];
    let first = protocol_named_node("control", path, first_body, &[]);
    let second = protocol_named_node("control", path, second_body, &[]);
    let first_key = first["node_id"].clone();
    let second_key = second["node_id"].clone();
    let base = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend([first, second]);
    package["semantic_graph"]["nodes"][root]["body"]["arguments"][0]["value"]["members"] = json!([
        {"term": "reference", "target": first_key},
        {"term": "reference", "target": second_key}
    ]);
    settle(&mut package);
    let first_key = package["semantic_graph"]["nodes"][base]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    let second_key = package["semantic_graph"]["nodes"][base + 1]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    let (position, key) = if first_key < second_key {
        (base + 1, second_key)
    } else {
        (base, first_key)
    };
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::AmbiguousDeclaration,
            &format!("/semantic_graph/nodes/{position}/declaration/qualified_name"),
            Some(CheckedPackageRefusalCause::AmbiguousName),
            key,
        )
    );
}

#[trace("TC-048")]
#[test]
fn protocol_application_placement_precedes_duplicate_declaration() {
    let mut package = v2_all_families();
    let clause = clause_position(&package);
    let mut duplicate_body = package["semantic_graph"]["nodes"][clause]["body"].clone();
    let boolean = package["semantic_graph"]["nodes"][clause]["semantic_type"].clone();
    duplicate_body["arguments"][10]["value"]["members"] = json!([{
        "term": "literal", "type": boolean,
        "value_kind": "text", "value": "adversarial"
    }]);
    append_duplicate_clause(&mut package, duplicate_body);
    let function = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| {
            node["node_tag"] == "function"
                && node["semantic_form"] == "pure_function"
                && node["body"]["term"] == "application"
        })
        .expect("function application");
    package["semantic_graph"]["nodes"][function]["body"] =
        package["semantic_graph"]["nodes"][clause]["body"].clone();
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][function]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::IllTyped,
            &format!("/semantic_graph/nodes/{function}/body"),
            Some(CheckedPackageRefusalCause::OperatorIneligible),
            key,
        )
    );
}
