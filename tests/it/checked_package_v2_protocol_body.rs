// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Public reader checks for the first protocol validation passes.

use crate::support::checked_package::{
    canonical, evidence_for, family_key, node_id, protocol_named_node, protocol_role_node,
    rebuild_source_map, refresh_identity, refusal, refusal_at, rekey_application_node, settle,
    structural_key, v2_all_families,
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

fn protocol_holder_case(operator: &str) -> (Value, Vec<usize>) {
    let mut package = v2_all_families();
    let root = protocol_position(&package, "control");
    let leaf_body = package["semantic_graph"]["nodes"][root]["body"].clone();
    let role = protocol_role_node();
    let role_target = role["node_id"].clone();
    let names: &[&str] = if operator == "await" {
        &["Event", "Then", "Timeout"]
    } else {
        &["Left", "Right"]
    };
    let mut positions = Vec::new();
    let mut targets = Vec::new();
    for (ordinal, name) in names.iter().enumerate() {
        let mut path = vec!["Example", "OrderFlow", "run", "Main", "Holder"];
        path.push(name);
        let mut child_body = leaf_body.clone();
        if ordinal == 0 && operator == "repeat" {
            child_body["operation"]["member"]["operator"] = json!("fence");
            child_body["arguments"] = json!([{"term": "binding", "name": "ordering", "value": {
                "term": "literal", "type": node_id(&family_key("a1a1")),
                "value_kind": "text", "value": "seq_cst"
            }}]);
        }
        if ordinal == 0 && operator == "await" {
            child_body["operation"]["member"]["operator"] = json!("event");
            child_body["arguments"] = json!([
                {"term": "binding", "name": "role", "value": {"term": "reference", "target": role_target}},
                {"term": "binding", "name": "compensation", "value": {"term": "aggregate", "members": []}},
                {"term": "binding", "name": "binder", "value": {"term": "reference", "target": role_target}},
                {"term": "binding", "name": "related", "value": {"term": "aggregate", "members": []}},
                {"term": "binding", "name": "constraint", "value": {"term": "literal", "type": node_id(&family_key("aaaa")), "value_kind": "boolean", "value": true}}
            ]);
        }
        let node = protocol_named_node("control", &path, child_body, &[]);
        targets.push(node["node_id"].clone());
        positions.push(
            package["semantic_graph"]["nodes"]
                .as_array()
                .expect("nodes")
                .len(),
        );
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(node);
    }
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(role);
    let clause = clause_position(&package);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][3]["value"]["members"] =
        json!([{"term": "reference", "target": role_target}]);
    let reference = |target: &Value| json!({"term": "reference", "target": target});
    let binding =
        |name: &str, value: Value| json!({"term": "binding", "name": name, "value": value});
    let group = |members: Vec<Value>| json!({"term": "aggregate", "members": members});
    let text = |value: &str| {
        json!({
            "term": "literal", "type": node_id(&family_key("a1a1")),
            "value_kind": "text", "value": value
        })
    };
    let integer = |value: &str| {
        json!({
            "term": "literal", "type": node_id(&family_key("a3a3")),
            "value_kind": "integer", "value": value
        })
    };
    let boolean = || {
        json!({
            "term": "literal", "type": node_id(&family_key("aaaa")),
            "value_kind": "boolean", "value": true
        })
    };
    let mut body = leaf_body;
    body["operation"]["member"]["operator"] = json!(operator);
    body["arguments"] = match operator {
        "choice" => json!([
            binding("owner", reference(&role_target)),
            binding("visible", group(vec![])),
            binding(
                "cases",
                group(vec![
                    group(vec![
                        binding("label", text("Left")),
                        binding("guard", boolean()),
                        binding("body", reference(&targets[0]))
                    ]),
                    group(vec![
                        binding("label", text("Right")),
                        binding("guard", boolean()),
                        binding("body", reference(&targets[1]))
                    ]),
                ])
            ),
        ]),
        "parallel" => json!([
            binding("memory", group(vec![])),
            binding(
                "branches",
                group(vec![
                    group(vec![
                        binding("label", text("Left")),
                        binding("body", reference(&targets[0]))
                    ]),
                    group(vec![
                        binding("label", text("Right")),
                        binding("body", reference(&targets[1]))
                    ]),
                ])
            ),
            binding("join", group(vec![binding("policy", text("all"))])),
            binding("joined", group(vec![text("Left"), text("Right")])),
            binding("outstanding", group(vec![])),
        ]),
        "repeat" => json!([
            binding("owner", reference(&role_target)),
            binding("visible", group(vec![])),
            binding("maximum", group(vec![integer("1")])),
            binding("proof", group(vec![])),
            binding("guard", boolean()),
            binding("body", reference(&targets[0])),
            binding("exhausted", group(vec![reference(&targets[1])])),
        ]),
        "await" => json!([
            binding(
                "after",
                group(vec![
                    binding("kind", text("event")),
                    binding("target", reference(&targets[0]))
                ])
            ),
            binding("clock", text("clock")),
            binding(
                "within",
                group(vec![
                    binding("lower", integer("0")),
                    binding("upper", integer("1"))
                ])
            ),
            binding("event", reference(&targets[0])),
            binding("then", reference(&targets[1])),
            binding("timeout", reference(&targets[2])),
        ]),
        other => panic!("unknown holder fixture: {other}"),
    };
    let holder = protocol_named_node(
        "control",
        &["Example", "OrderFlow", "run", "Main", "Holder"],
        body,
        &[],
    );
    let holder_key = holder["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(holder);
    package["semantic_graph"]["nodes"][root]["body"]["arguments"][0]["value"]["members"] =
        json!([reference(&holder_key)]);
    settle(&mut package);
    (package, positions)
}

fn holder_position(package: &Value) -> usize {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| {
            node["declaration"]["qualified_name"]
                == json!(["Example", "OrderFlow", "run", "Main", "Holder"])
        })
        .expect("holder")
}

fn holder_refusal(
    package: &Value,
    code: CheckedPackageRefusalCode,
    cause: Option<CheckedPackageRefusalCause>,
    path: &str,
) {
    let position = holder_position(package);
    let key = package["semantic_graph"]["nodes"][position]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    assert_eq!(
        read(package),
        refusal_at(
            code,
            &format!("/semantic_graph/nodes/{position}{path}"),
            cause,
            key
        )
    );
}

/// Trace: FR-038-AC-284, FR-038-AC-288
#[trace("TC-048")]
#[test]
fn choice_labels_precede_guard_type_and_parallel_join_is_checked() {
    let (mut package, _) = protocol_holder_case("choice");
    let holder = holder_position(&package);
    let cases =
        &mut package["semantic_graph"]["nodes"][holder]["body"]["arguments"][2]["value"]["members"];
    cases[1]["members"][0]["value"]["value"] = json!("Left");
    cases[0]["members"][1]["value"]["value_kind"] = json!("text");
    cases[0]["members"][1]["value"]["value"] = json!("false");
    settle(&mut package);
    holder_refusal(
        &package,
        CheckedPackageRefusalCode::AmbiguousDeclaration,
        Some(CheckedPackageRefusalCause::AmbiguousName),
        "",
    );

    let (mut package, _) = protocol_holder_case("parallel");
    let holder = holder_position(&package);
    package["semantic_graph"]["nodes"][holder]["body"]["arguments"][3]["value"]["members"]
        .as_array_mut()
        .expect("joined")
        .pop();
    settle(&mut package);
    holder_refusal(
        &package,
        CheckedPackageRefusalCode::InvalidPackage,
        Some(CheckedPackageRefusalCause::InvalidValue),
        "/body/arguments/3/value",
    );
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn control_choice_guard_and_parallel_join_labels_refuse_at_the_member() {
    let (mut package, _) = protocol_holder_case("choice");
    let holder = holder_position(&package);
    let guard = &mut package["semantic_graph"]["nodes"][holder]["body"]["arguments"][2]["value"]
        ["members"][0]["members"][1]["value"];
    guard["value_kind"] = json!("text");
    guard["value"] = json!("false");
    settle(&mut package);
    holder_refusal(
        &package,
        CheckedPackageRefusalCode::IllTyped,
        Some(CheckedPackageRefusalCause::NonBooleanRoot),
        "/body/arguments/2/value/members/0/members/1/value",
    );

    let (mut package, _) = protocol_holder_case("parallel");
    let holder = holder_position(&package);
    package["semantic_graph"]["nodes"][holder]["body"]["arguments"][3]["value"]["members"][1]
        ["value"] = json!("Absent");
    settle(&mut package);
    holder_refusal(
        &package,
        CheckedPackageRefusalCode::MissingDeclaration,
        Some(CheckedPackageRefusalCause::MissingName),
        "/body/arguments/3/value/members/1",
    );
}

/// Trace: FR-038-AC-284, FR-038-AC-288
#[trace("TC-048")]
#[test]
fn sequence_admits_ordered_controls_and_refuses_role_child() {
    let (mut package, children) = protocol_holder_case("parallel");
    let holder = holder_position(&package);
    let targets: Vec<Value> = children
        .iter()
        .map(|&position| package["semantic_graph"]["nodes"][position]["node_id"].clone())
        .collect();
    package["semantic_graph"]["nodes"][holder]["body"]["operation"]["member"]["operator"] =
        json!("sequence");
    package["semantic_graph"]["nodes"][holder]["body"]["arguments"] = json!([{
        "term": "binding", "name": "children", "value": {
            "term": "aggregate", "members": targets.iter().map(|target| json!({"term": "reference", "target": target})).collect::<Vec<_>>()
        }
    }]);
    settle(&mut package);
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));

    let mut empty = package.clone();
    let holder = holder_position(&empty);
    empty["semantic_graph"]["nodes"][holder]["body"]["arguments"][0]["value"]["members"] =
        json!([]);
    settle(&mut empty);
    assert!(matches!(
        read(&empty),
        CheckedPackageV2ReadResult::Admitted(_)
    ));

    let role = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["semantic_form"] == "role")
        .expect("role")["node_id"]
        .clone();
    let holder = holder_position(&package);
    package["semantic_graph"]["nodes"][holder]["body"]["arguments"][0]["value"]["members"][0]
        ["target"] = role;
    settle(&mut package);
    holder_refusal(
        &package,
        CheckedPackageRefusalCode::InvalidModelBinding,
        Some(CheckedPackageRefusalCause::MalformedDeclaration),
        "/body/arguments/0/value/members/0/target",
    );
}

/// Trace: FR-038-AC-282, FR-038-AC-288
#[trace("TC-048")]
#[test]
fn choice_case_order_and_parallel_quorum_bounds_refuse() {
    let (mut package, _) = protocol_holder_case("choice");
    let holder = holder_position(&package);
    package["semantic_graph"]["nodes"][holder]["body"]["arguments"][2]["value"]["members"]
        .as_array_mut()
        .expect("cases")
        .pop();
    assert_eq!(
        read(&package),
        refusal(
            CheckedPackageRefusalCode::MalformedWire,
            &format!("/semantic_graph/nodes/{holder}/body")
        )
    );

    let (mut package, _) = protocol_holder_case("choice");
    let holder = holder_position(&package);
    package["semantic_graph"]["nodes"][holder]["body"]["arguments"][2]["value"]["members"]
        .as_array_mut()
        .expect("cases")
        .reverse();
    settle(&mut package);
    holder_refusal(
        &package,
        CheckedPackageRefusalCode::InvalidSemanticGraph,
        None,
        "",
    );

    let (mut package, _) = protocol_holder_case("parallel");
    let holder = holder_position(&package);
    let integer_type = node_id(&family_key("a3a3"));
    package["semantic_graph"]["nodes"][holder]["body"]["arguments"][2]["value"]["members"] = json!([
        {"term": "binding", "name": "policy", "value": {"term": "literal", "type": node_id(&family_key("a1a1")), "value_kind": "text", "value": "quorum"}},
        {"term": "binding", "name": "count", "value": {"term": "literal", "type": integer_type, "value_kind": "integer", "value": "3"}}
    ]);
    settle(&mut package);
    holder_refusal(
        &package,
        CheckedPackageRefusalCode::InvalidPackage,
        Some(CheckedPackageRefusalCause::InvalidValue),
        "/body/arguments/2/value/members/1/value",
    );
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn parallel_any_quorum_and_predicate_joins_admit() {
    for policy in ["any", "quorum", "predicate"] {
        let (mut package, _) = protocol_holder_case("parallel");
        let holder = holder_position(&package);
        let function = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .find(|node| node["node_tag"] == "function")
            .expect("function")["node_id"]
            .clone();
        let text = |value: &str| {
            json!({
                "term": "literal", "type": node_id(&family_key("a1a1")),
                "value_kind": "text", "value": value
            })
        };
        let mut fields = vec![json!({"term": "binding", "name": "policy", "value": text(policy)})];
        if policy == "quorum" {
            fields.push(json!({"term": "binding", "name": "count", "value": {
                "term": "literal", "type": node_id(&family_key("a3a3")),
                "value_kind": "integer", "value": "1"
            }}));
        }
        if policy == "predicate" {
            fields.push(json!({"term": "binding", "name": "predicate", "value": {
                "term": "reference", "target": function
            }}));
        }
        package["semantic_graph"]["nodes"][holder]["body"]["arguments"][2]["value"]["members"] =
            json!(fields);
        package["semantic_graph"]["nodes"][holder]["body"]["arguments"][3]["value"]["members"] =
            json!([text("Left")]);
        settle(&mut package);
        assert!(
            matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
            "{policy} join"
        );
    }
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn parallel_predicate_target_requires_function_form() {
    let (mut package, _) = protocol_holder_case("parallel");
    let holder = holder_position(&package);
    let role = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["semantic_form"] == "role")
        .expect("role")["node_id"]
        .clone();
    package["semantic_graph"]["nodes"][holder]["body"]["arguments"][2]["value"]["members"] = json!([
        {"term": "binding", "name": "policy", "value": {
            "term": "literal", "type": node_id(&family_key("a1a1")), "value_kind": "text", "value": "predicate"
        }},
        {"term": "binding", "name": "predicate", "value": {"term": "reference", "target": role}}
    ]);
    settle(&mut package);
    holder_refusal(
        &package,
        CheckedPackageRefusalCode::InvalidModelBinding,
        Some(CheckedPackageRefusalCause::MalformedDeclaration),
        "/body/arguments/2/value/members/1/value/target",
    );
}

/// Trace: FR-038-AC-282, FR-038-AC-288
#[trace("TC-048")]
#[test]
fn parallel_memory_and_outstanding_reject_unknown_spelling_at_wire_boundary() {
    for (argument, value) in [(0, "weak"), (4, "pause")] {
        let (mut package, _) = protocol_holder_case("parallel");
        let holder = holder_position(&package);
        package["semantic_graph"]["nodes"][holder]["body"]["arguments"][argument]["value"]
            ["members"] = json!([{
            "term": "literal", "type": node_id(&family_key("a1a1")),
            "value_kind": "text", "value": value
        }]);
        assert_eq!(
            read(&package),
            refusal(
                CheckedPackageRefusalCode::MalformedWire,
                &format!(
                    "/semantic_graph/nodes/{holder}/body/arguments/{argument}/value/members/0"
                )
            ),
            "{value}"
        );
    }
}

#[trace("TC-048")]
#[test]
fn choice_parallel_repeat_and_await_hold_controls_at_their_own_paths() {
    for operator in ["choice", "parallel", "repeat", "await"] {
        let (package, children) = protocol_holder_case(operator);
        assert!(
            matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
            "{operator} holder with correctly named children must admit"
        );
        for (child_index, &position) in children.iter().enumerate() {
            let mut wrong = package.clone();
            let name = wrong["semantic_graph"]["nodes"][position]["declaration"]["qualified_name"]
                .as_array()
                .expect("path")
                .last()
                .expect("name")
                .clone();
            wrong["semantic_graph"]["nodes"][position]["declaration"]["qualified_name"] =
                json!(["Example", "OrderFlow", "run", "Main", name]);
            settle(&mut wrong);
            let key = wrong["semantic_graph"]["nodes"][position]["node_id"]["digest"]
                .as_str()
                .expect("digest");
            assert_eq!(
                read(&wrong),
                refusal_at(
                    CheckedPackageRefusalCode::InvalidSemanticGraph,
                    &format!("/semantic_graph/nodes/{position}"),
                    None,
                    key,
                ),
                "{operator} child {child_index} must use its holder path"
            );
        }
    }
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn repeat_checks_owner_body_exhausted_and_optional_pair() {
    let (package, children) = protocol_holder_case("repeat");
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    let holder = holder_position(&package);

    let mut missing_exhausted = package.clone();
    missing_exhausted["semantic_graph"]["nodes"][holder]["body"]["arguments"][6]["value"] =
        json!({"term": "aggregate", "members": []});
    settle(&mut missing_exhausted);
    assert_eq!(
        read(&missing_exhausted),
        refusal(
            CheckedPackageRefusalCode::MalformedWire,
            &format!("/semantic_graph/nodes/{holder}/body")
        )
    );

    let mut wrong_body = package.clone();
    wrong_body["semantic_graph"]["nodes"][holder]["body"]["arguments"][5]["value"]["target"] =
        wrong_body["semantic_graph"]["nodes"][clause_position(&package)]["node_id"].clone();
    settle(&mut wrong_body);
    holder_refusal(
        &wrong_body,
        CheckedPackageRefusalCode::InvalidModelBinding,
        Some(CheckedPackageRefusalCause::MalformedDeclaration),
        "/body/arguments/5/value/target",
    );

    let mut no_bound = package;
    no_bound["semantic_graph"]["nodes"][holder]["body"]["arguments"][2]["value"]["members"] =
        json!([]);
    no_bound["semantic_graph"]["nodes"][holder]["body"]["arguments"][6]["value"]["members"] =
        json!([]);
    settle(&mut no_bound);
    assert!(matches!(
        read(&no_bound),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    assert_ne!(children[0], children[1]);
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn await_checks_anchor_form_and_event_eligibility() {
    let (package, children) = protocol_holder_case("await");
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    let holder = holder_position(&package);

    let mut wrong_anchor = package.clone();
    wrong_anchor["semantic_graph"]["nodes"][holder]["body"]["arguments"][0]["value"]["members"]
        [1]["value"]["target"] =
        wrong_anchor["semantic_graph"]["nodes"][children[1]]["node_id"].clone();
    settle(&mut wrong_anchor);
    holder_refusal(
        &wrong_anchor,
        CheckedPackageRefusalCode::InvalidModelBinding,
        Some(CheckedPackageRefusalCause::MalformedDeclaration),
        "/body/arguments/0/value/members/1/value/target",
    );

    let mut wrong_event = package;
    wrong_event["semantic_graph"]["nodes"][holder]["body"]["arguments"][3]["value"]["target"] =
        wrong_event["semantic_graph"]["nodes"][children[1]]["node_id"].clone();
    settle(&mut wrong_event);
    holder_refusal(
        &wrong_event,
        CheckedPackageRefusalCode::UnsupportedConstruct,
        Some(CheckedPackageRefusalCause::DeclarationForm),
        "",
    );
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

fn package_with_member_role() -> (Value, usize) {
    let mut package = v2_all_families();
    let role = protocol_role_node();
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
    (package, position)
}

fn package_with_replicated_role() -> (Value, usize) {
    let (mut package, role) = package_with_member_role();
    let model =
        package["semantic_graph"]["nodes"][role]["body"]["arguments"][0]["value"]["target"].clone();
    let reference_body = json!({"term": "aggregate", "members": [
        {"term": "reference", "target": model}
    ]});
    let reference_type = structural_key("composite_type", "reference", None, &reference_body);
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(json!({
            "node_id": node_id(&reference_type),
            "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": "composite_type", "semantic_form": "reference",
            "semantic_type": node_id(&reference_type), "dependencies": [model],
            "occurrences": [{"role": "type", "ordinal": 0}],
            "body": reference_body,
        }));
    let mut binder = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_id"] == node_id(&family_key("p-close")))
        .expect("parameter")
        .clone();
    binder["semantic_type"] = node_id(&reference_type);
    let binder_key = structural_key("value", "parameter", Some(&reference_type), &binder["body"]);
    binder["node_id"] = node_id(&binder_key);
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(binder);
    package["semantic_graph"]["nodes"][role]["body"]["arguments"][1]["value"]["members"] = json!([
        {"term": "binding", "name": "population", "value": {
            "term": "literal", "type": node_id(&family_key("a1a1")),
            "value_kind": "text", "value": "orders"}},
        {"term": "binding", "name": "max", "value": {
            "term": "literal", "type": node_id(&family_key("a3a3")),
            "value_kind": "integer", "value": "2"}},
        {"term": "binding", "name": "binder", "value": {
            "term": "reference", "target": node_id(&binder_key)}},
        {"term": "binding", "name": "lifetime", "value": {
            "term": "literal", "type": node_id(&family_key("a1a1")),
            "value_kind": "text", "value": "workflow"}},
    ]);
    settle(&mut package);
    (package, role)
}

fn package_with_member_channel() -> (Value, usize, usize) {
    let (mut package, role) = package_with_member_role();
    let role_key = package["semantic_graph"]["nodes"][role]["node_id"].clone();
    let boolean = node_id(&family_key("aaaa"));
    let text = |value: &str| {
        json!({
            "term": "literal", "type": node_id(&family_key("a1a1")),
            "value_kind": "text", "value": value
        })
    };
    let binding =
        |name: &str, value: Value| json!({"term": "binding", "name": name, "value": value});
    let reference = |target: &Value| json!({"term": "reference", "target": target});
    let mut body =
        package["semantic_graph"]["nodes"][protocol_position(&package, "control")]["body"].clone();
    body["operation"]["member"]["operator"] = json!("channel");
    body["arguments"] = json!([
        binding("from", reference(&role_key)),
        binding("to", reference(&role_key)),
        binding("carries", reference(&boolean)),
        binding(
            "ordering",
            json!({"term": "aggregate", "members": [binding("kind", text("unordered"))]})
        ),
        binding("delivery", text("unknown")),
    ]);
    let channel = protocol_named_node(
        "channel",
        &["Example", "OrderFlow", "channel", "Link"],
        body,
        &[
            role_key["digest"].as_str().expect("digest").to_owned(),
            family_key("aaaa"),
        ],
    );
    let channel_key = channel["node_id"].clone();
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(channel);
    let clause = clause_position(&package);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][5]["value"]["members"] =
        json!([reference(&channel_key)]);
    settle(&mut package);
    (package, role, position)
}

fn package_with_member_queue() -> (Value, usize) {
    let (mut package, _, channel) = package_with_member_channel();
    let channel_key = package["semantic_graph"]["nodes"][channel]["node_id"].clone();
    let text = |value: &str| {
        json!({
            "term": "literal", "type": node_id(&family_key("a1a1")),
            "value_kind": "text", "value": value
        })
    };
    let binding =
        |name: &str, value: Value| json!({"term": "binding", "name": name, "value": value});
    let mut body = package["semantic_graph"]["nodes"][channel]["body"].clone();
    body["operation"]["member"]["operator"] = json!("queue");
    body["arguments"] = json!([
        binding(
            "channel",
            json!({"term": "reference", "target": channel_key})
        ),
        binding(
            "capacity",
            json!({"term": "aggregate", "members": [
                binding("kind", text("literal")),
                binding("value", json!({"term": "literal", "type": node_id(&family_key("a3a3")),
                    "value_kind": "integer", "value": "4"})),
            ]})
        ),
        binding("overflow", text("reject")),
        binding("loss", json!({"term": "aggregate", "members": []})),
    ]);
    let mut queue = protocol_named_node("queue", &[], body, &[]);
    queue
        .as_object_mut()
        .expect("queue node")
        .remove("declaration");
    queue["occurrences"][0]["role"] = json!("generated");
    let queue_key = queue["node_id"].clone();
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(queue);
    let clause = clause_position(&package);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][6]["value"]["members"] =
        json!([{"term": "reference", "target": queue_key}]);
    settle(&mut package);
    (package, position)
}

/// Trace: FR-038-AC-286
#[trace("TC-947")]
#[test]
fn protocol_queue_checks_channel_form() {
    let (mut package, queue) = package_with_member_queue();
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    package["semantic_graph"]["nodes"][queue]["body"]["arguments"][0]["value"]["target"] =
        node_id(&family_key("a4a4"));
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][queue]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidModelBinding,
            &format!("/semantic_graph/nodes/{queue}/body/arguments/0/value/target"),
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            key,
        )
    );
}

/// Trace: FR-038-AC-286
#[trace("TC-947")]
#[test]
fn protocol_queue_rejects_negative_literal_capacity() {
    let (mut package, queue) = package_with_member_queue();
    package["semantic_graph"]["nodes"][queue]["body"]["arguments"][1]["value"]["members"][1]
        ["value"]["value"] = json!("-1");
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][queue]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidPackage,
            &format!("/semantic_graph/nodes/{queue}/body/arguments/1/value/members/1/value"),
            Some(CheckedPackageRefusalCause::InvalidValue),
            key,
        )
    );
}

/// Trace: FR-038-AC-287
#[trace("TC-947")]
#[test]
fn protocol_finish_constraint_must_be_boolean() {
    let mut package = v2_all_families();
    let finish = protocol_position(&package, "obligation");
    package["semantic_graph"]["nodes"][finish]["body"]["arguments"][1]["value"] = json!({
        "term": "literal", "type": node_id(&family_key("a3a3")),
        "value_kind": "integer", "value": "1"
    });
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][finish]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::IllTyped,
            &format!("/semantic_graph/nodes/{finish}"),
            Some(CheckedPackageRefusalCause::NonBooleanRoot),
            key,
        )
    );
}

/// Trace: FR-038-AC-287
#[trace("TC-947")]
#[test]
fn protocol_terminal_when_requires_boolean_condition() {
    let mut package = v2_all_families();
    let finish = protocol_position(&package, "obligation");
    let mut body = package["semantic_graph"]["nodes"][finish]["body"].clone();
    body["operation"]["member"]["operator"] = json!("terminal");
    body["arguments"] = json!([
        {"term": "binding", "name": "mode", "value": {
            "term": "literal", "type": node_id(&family_key("a1a1")),
            "value_kind": "text", "value": "any"}},
        {"term": "binding", "name": "condition", "value": {
            "term": "aggregate", "members": []}},
    ]);
    let terminal = protocol_named_node(
        "obligation",
        &["Example", "OrderFlow", "terminal"],
        body,
        &[],
    );
    let key = terminal["node_id"].clone();
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(terminal);
    let clause = clause_position(&package);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][9]["value"]["members"] =
        json!([{"term": "reference", "target": key}]);
    settle(&mut package);
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    package["semantic_graph"]["nodes"][position]["body"]["arguments"][0]["value"]["value"] =
        json!("when");
    package["semantic_graph"]["nodes"][position]["body"]["arguments"][1]["value"]["members"] = json!([{"term": "literal", "type": node_id(&family_key("a3a3")),
            "value_kind": "integer", "value": "1"}]);
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][position]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::IllTyped,
            &format!("/semantic_graph/nodes/{position}"),
            Some(CheckedPackageRefusalCause::NonBooleanRoot),
            key,
        )
    );
}

/// Trace: FR-038-AC-286
#[trace("TC-947")]
#[test]
fn protocol_channel_checks_endpoint_and_message_type() {
    let (mut package, _role, channel) = package_with_member_channel();
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    package["semantic_graph"]["nodes"][channel]["body"]["arguments"][2]["value"]["target"] =
        node_id(&family_key("a4a4"));
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][channel]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidModelBinding,
            &format!("/semantic_graph/nodes/{channel}/body/arguments/2/value/target"),
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            key,
        )
    );
}

/// Trace: FR-038-AC-286
#[trace("TC-947")]
#[test]
fn protocol_channel_admits_fifo_with_message_binder_and_stable_key() {
    let (mut package, _, channel) = package_with_member_channel();
    package["semantic_graph"]["nodes"][channel]["body"]["arguments"][3]["value"]["members"] = json!([
        {"term": "binding", "name": "kind", "value": {
            "term": "literal", "type": node_id(&family_key("a1a1")),
            "value_kind": "text", "value": "fifo"}},
        {"term": "binding", "name": "binder", "value": {
            "term": "reference", "target": node_id(&family_key("p-close"))}},
        {"term": "binding", "name": "key", "value": {
            "term": "literal", "type": node_id(&family_key("aaaa")),
            "value_kind": "boolean", "value": true}},
    ]);
    settle(&mut package);
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

/// Trace: FR-038-AC-286
#[trace("TC-947")]
#[test]
fn protocol_channel_fifo_refuses_float_key() {
    let (mut package, _, channel) = package_with_member_channel();
    let body = json!({"term": "aggregate", "members": []});
    let digest = structural_key("scalar_type", "float32", None, &body);
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(json!({
            "node_id": node_id(&digest),
            "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": "scalar_type", "semantic_form": "float32",
            "semantic_type": node_id(&digest), "dependencies": [],
            "occurrences": [{"role": "generated", "ordinal": 0}],
            "body": body,
        }));
    package["semantic_graph"]["nodes"][channel]["body"]["arguments"][3]["value"]["members"] = json!([
        {"term": "binding", "name": "kind", "value": {
            "term": "literal", "type": node_id(&family_key("a1a1")),
            "value_kind": "text", "value": "fifo"}},
        {"term": "binding", "name": "binder", "value": {
            "term": "reference", "target": node_id(&family_key("p-close"))}},
        {"term": "binding", "name": "key", "value": {
            "term": "literal", "type": node_id(&digest),
            "value_kind": "float32_bits", "value": "3f800000"}},
    ]);
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][channel]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::IllTyped,
            &format!("/semantic_graph/nodes/{channel}/body/arguments/3/value/members/2/value"),
            Some(CheckedPackageRefusalCause::OperatorIneligible),
            key,
        )
    );
}

/// Trace: FR-038-AC-281, FR-038-AC-284
#[trace("TC-947")]
#[test]
fn protocol_role_checks_model_form_at_target() {
    let (mut package, position) = package_with_member_role();
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    let wrong = node_id(&family_key("a4a4"));
    package["semantic_graph"]["nodes"][position]["body"]["arguments"][0]["value"]["target"] = wrong;
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][position]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidModelBinding,
            &format!("/semantic_graph/nodes/{position}/body/arguments/0/value/target"),
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            key,
        )
    );
}

/// Trace: FR-038-AC-281, FR-038-AC-284
#[trace("TC-947")]
#[test]
fn protocol_role_dependency_precedes_wrong_model_form() {
    let (mut package, position) = package_with_member_role();
    package["semantic_graph"]["nodes"][position]["body"]["arguments"][0]["value"]["target"] =
        node_id(&family_key("a4a4"));
    settle(&mut package);
    package["semantic_graph"]["nodes"][position]["dependencies"] = json!([]);
    refresh_identity(&mut package);
    let key = package["semantic_graph"]["nodes"][position]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::MissingDeclaration,
            &format!("/semantic_graph/nodes/{position}/body/arguments/0/value/target"),
            Some(CheckedPackageRefusalCause::MissingName),
            key,
        )
    );
}

/// Trace: FR-038-AC-286
#[trace("TC-947")]
#[test]
fn protocol_role_max_precedes_wrong_replication_binder_type() {
    let (mut package, role) = package_with_member_role();
    let text = |value: &str| {
        json!({
            "term": "literal", "type": node_id(&family_key("a1a1")),
            "value_kind": "text", "value": value
        })
    };
    package["semantic_graph"]["nodes"][role]["body"]["arguments"][1]["value"]["members"] = json!([
        {"term": "binding", "name": "population", "value": text("orders")},
        {"term": "binding", "name": "max", "value": {
            "term": "literal", "type": node_id(&family_key("a3a3")),
            "value_kind": "integer", "value": "0"}},
        {"term": "binding", "name": "binder", "value": {
            "term": "reference", "target": node_id(&family_key("p-close"))}},
        {"term": "binding", "name": "lifetime", "value": text("workflow")},
    ]);
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][role]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::IllTyped,
            &format!("/semantic_graph/nodes/{role}/body/arguments/1/value/members/1/value"),
            Some(CheckedPackageRefusalCause::TypeMismatch),
            key,
        )
    );
}

/// Trace: FR-038-AC-286
#[trace("TC-947")]
#[test]
fn protocol_replicated_role_admits_positive_max_and_reference_binder() {
    let (package, _) = package_with_replicated_role();
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

/// Trace: FR-038-AC-286
#[trace("TC-947")]
#[test]
fn protocol_replicated_role_rejects_wrong_binder_type_after_positive_max() {
    let (mut package, role) = package_with_replicated_role();
    package["semantic_graph"]["nodes"][role]["body"]["arguments"][1]["value"]["members"][2]
        ["value"]["target"] = node_id(&family_key("p-close"));
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][role]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::IllTyped,
            &format!("/semantic_graph/nodes/{role}"),
            Some(CheckedPackageRefusalCause::TypeMismatch),
            key,
        )
    );
}

/// Trace: FR-038-AC-286
#[trace("TC-947")]
#[test]
fn protocol_queue_rejects_foreign_clause_channel() {
    let (mut package, queue) = package_with_member_queue();
    let original_role = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["semantic_form"] == "role")
        .expect("role");
    let original_channel = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["semantic_form"] == "channel")
        .expect("channel");
    let role_dependencies = package["semantic_graph"]["nodes"][original_role]["dependencies"]
        .as_array()
        .expect("role dependencies")
        .iter()
        .map(|id| id["digest"].as_str().expect("digest").to_owned())
        .collect::<Vec<_>>();
    let foreign_role = protocol_named_node(
        "role",
        &["Example", "Elsewhere", "role", "Worker"],
        package["semantic_graph"]["nodes"][original_role]["body"].clone(),
        &role_dependencies,
    );
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(foreign_role);
    settle(&mut package);
    let foreign_role = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
    let foreign_role_key = package["semantic_graph"]["nodes"][foreign_role]["node_id"].clone();
    let mut foreign_body = package["semantic_graph"]["nodes"][original_channel]["body"].clone();
    for ordinal in [0, 1] {
        foreign_body["arguments"][ordinal]["value"]["target"] = foreign_role_key.clone();
    }
    let foreign_channel = protocol_named_node(
        "channel",
        &["Example", "Elsewhere", "channel", "Link"],
        foreign_body,
        &[
            foreign_role_key["digest"]
                .as_str()
                .expect("digest")
                .to_owned(),
            family_key("aaaa"),
        ],
    );
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(foreign_channel);
    settle(&mut package);
    let foreign_channel = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
    let foreign_channel_key =
        package["semantic_graph"]["nodes"][foreign_channel]["node_id"].clone();
    package["semantic_graph"]["nodes"][queue]["body"]["arguments"][0]["value"]["target"] =
        foreign_channel_key;
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][queue]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidModelBinding,
            &format!("/semantic_graph/nodes/{queue}/body/arguments/0/value/target"),
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            key,
        )
    );
}
