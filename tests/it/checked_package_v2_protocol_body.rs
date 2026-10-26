// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Public reader checks for the first protocol validation passes.

use crate::support::checked_package::{
    application_node_key, canonical, domain_package_digest, domain_package_document, evidence_for,
    family_key, node_id, protocol_named_node, protocol_role_node, rebuild_source_map,
    refresh_identity, refusal, refusal_at, rekey_application_node, settle, structural_key,
    v2_all_families,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedPackageReadLimits, CheckedPackageRefusalCause, CheckedPackageRefusalCode,
    CheckedPackageV2, CheckedPackageV2ReadResult, CheckedProtocolActivation,
    CheckedProtocolActivationRow, CheckedProtocolRefusalDetail,
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

fn clause_parameter(package: &mut Value, name: &str, level: &str) -> Value {
    let over = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| {
            node["node_tag"] == "value"
                && node["semantic_form"] == "parameter"
                && node["body"]["members"][0]["value"]["value"] == "over"
        })
        .expect("over parameter")
        .clone();
    let mut parameter = over;
    parameter["body"]["members"][0]["value"]["value"] = json!(name);
    parameter["body"]["members"][1]["value"]["value"] = json!(level);
    let key = structural_key(
        "value",
        "parameter",
        Some(&family_key("aaaa")),
        &parameter["body"],
    );
    parameter["node_id"] = node_id(&key);
    let target = parameter["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(parameter);
    target
}

fn temporal_clause(package: &Value) -> Value {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "temporal" && node["semantic_form"] == "temporal_clause")
        .expect("temporal clause")
        .clone()
}

fn require_temporal(package: &mut Value, keys: &[Value]) {
    let clause = clause_position(package);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][7]["value"]["members"] =
        Value::Array(
            keys.iter()
                .map(|key| json!({"term": "reference", "target": key}))
                .collect(),
        );
    settle(package);
}

/// Trace: FR-038-AC-324
#[trace("TC-048")]
#[test]
fn activation_relation_has_exact_origin_and_each_rows_without_extras() {
    let mut package = v2_all_families();
    let clause = clause_position(&package);
    let temporal = temporal_clause(&package)["node_id"].clone();
    let empty = match read(&package) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("base protocol should admit: {other:?}"),
    };
    assert!(empty.protocol_activation_relation().is_empty());

    require_temporal(&mut package, std::slice::from_ref(&temporal));
    let clause_key = package["semantic_graph"]["nodes"][clause]["node_id"]["digest"]
        .as_str()
        .expect("clause key");
    let expected = vec![CheckedProtocolActivationRow {
        clause: serde_json::from_value(node_id(clause_key)).expect("clause key"),
        activation: CheckedProtocolActivation::Origin,
        required_temporal: serde_json::from_value(temporal.clone()).expect("temporal key"),
    }];
    let admitted = match read(&package) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("origin protocol should admit: {other:?}"),
    };
    let before = admitted.clone();
    assert_eq!(admitted.protocol_activation_relation(), expected);
    assert_eq!(admitted.protocol_activation_relation(), expected);
    assert_eq!(*admitted, *before);
    let equal = match read(&package) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("equal protocol should admit: {other:?}"),
    };
    assert_eq!(equal.protocol_activation_relation(), expected);

    let trigger = clause_parameter(&mut package, "opened", "1");
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][1]["value"]["members"] = json!([
        {"term": "binding", "name": "kind", "value": {"term": "literal", "type": node_id(&family_key("a1a1")), "value_kind": "text", "value": "each"}},
        {"term": "binding", "name": "trigger", "value": {"term": "reference", "target": trigger}},
    ]);
    settle(&mut package);
    let admitted = match read(&package) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("each protocol should admit: {other:?}"),
    };
    assert_eq!(
        admitted.protocol_activation_relation(),
        vec![CheckedProtocolActivationRow {
            clause: serde_json::from_value(
                package["semantic_graph"]["nodes"][clause]["node_id"].clone()
            )
            .expect("clause key"),
            activation: CheckedProtocolActivation::Each {
                trigger: serde_json::from_value(trigger).expect("trigger key")
            },
            required_temporal: serde_json::from_value(temporal).expect("temporal key"),
        }]
    );
}

/// Trace: FR-038-AC-324
#[trace("TC-048")]
#[test]
fn activation_relation_orders_distinct_requirements_and_ignores_other_temporal_clauses() {
    let mut package = v2_all_families();
    let first = temporal_clause(&package)["node_id"].clone();
    let mut second = temporal_clause(&package);
    second["body"]["arguments"][1]["value"] = json!("other-clause");
    let second_key = application_node_key(
        "temporal",
        "temporal_clause",
        &family_key("aaaa"),
        &second["body"],
    );
    second["node_id"] = node_id(&second_key);
    let second_id = second["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(second);
    settle(&mut package);

    require_temporal(&mut package, std::slice::from_ref(&first));
    let admitted = match read(&package) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("unrelated temporal clause should admit: {other:?}"),
    };
    assert_eq!(admitted.protocol_activation_relation().len(), 1);
    assert_eq!(
        admitted.protocol_activation_relation()[0].required_temporal,
        serde_json::from_value(first.clone()).expect("first key")
    );

    let mut keys = [first, second_id];
    keys.sort_by(|left, right| left["digest"].as_str().cmp(&right["digest"].as_str()));
    require_temporal(&mut package, &keys);
    let admitted = match read(&package) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("two requirements should admit: {other:?}"),
    };
    let rows = admitted.protocol_activation_relation();
    assert_eq!(rows.len(), 2);
    assert_eq!(
        rows.iter()
            .map(|row| row.required_temporal.digest.as_ref())
            .collect::<Vec<_>>(),
        keys.iter()
            .map(|key| key["digest"].as_str().expect("digest"))
            .collect::<Vec<_>>()
    );
    assert_eq!(rows[0].clause, rows[1].clause);
    assert_eq!(rows[0].activation, CheckedProtocolActivation::Origin);
    assert_eq!(rows[1].activation, CheckedProtocolActivation::Origin);
}

/// Trace: FR-038-AC-324
#[trace("TC-048")]
#[test]
fn activation_relation_orders_rows_across_local_protocol_clauses() {
    let mut package = v2_all_families();
    let clause = clause_position(&package);
    let temporal = temporal_clause(&package)["node_id"].clone();
    require_temporal(&mut package, std::slice::from_ref(&temporal));

    let clone_member = |package: &Value, form: &str, name: &str| {
        let original = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .find(|node| node["node_tag"] == "protocol" && node["semantic_form"] == form)
            .expect("protocol member");
        let dependencies = original["dependencies"]
            .as_array()
            .expect("dependencies")
            .iter()
            .map(|id| id["digest"].as_str().expect("digest").to_owned())
            .collect::<Vec<_>>();
        protocol_named_node(
            form,
            &["Example", "OtherFlow", name, "Main"],
            original["body"].clone(),
            &dependencies,
        )
    };
    let run = clone_member(&package, "control", "run");
    let finish = {
        let original = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .find(|node| node["node_tag"] == "protocol" && node["semantic_form"] == "obligation")
            .expect("finish");
        let dependencies = original["dependencies"]
            .as_array()
            .expect("dependencies")
            .iter()
            .map(|id| id["digest"].as_str().expect("digest").to_owned())
            .collect::<Vec<_>>();
        protocol_named_node(
            "obligation",
            &["Example", "OtherFlow", "finish", "Closed"],
            original["body"].clone(),
            &dependencies,
        )
    };
    let run_id = run["node_id"].clone();
    let finish_id = finish["node_id"].clone();
    let mut body = package["semantic_graph"]["nodes"][clause]["body"].clone();
    body["arguments"][11]["value"]["target"] = run_id.clone();
    body["arguments"][12]["value"]["target"] = finish_id.clone();
    let over = body["arguments"][0]["value"]["target"]["digest"]
        .as_str()
        .expect("over");
    let dependencies = [
        over,
        run_id["digest"].as_str().expect("run"),
        finish_id["digest"].as_str().expect("finish"),
        temporal["digest"].as_str().expect("temporal"),
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let other = protocol_named_node(
        "protocol_clause",
        &["Example", "OtherFlow"],
        body,
        &dependencies,
    );
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend([run, finish, other]);
    settle(&mut package);
    let admitted = match read(&package) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("two local protocols should admit: {other:?}"),
    };
    let rows = admitted.protocol_activation_relation();
    assert_eq!(rows.len(), 2);
    assert!(rows[0].clause.digest < rows[1].clause.digest);
    assert_eq!(rows[0].required_temporal, rows[1].required_temporal);
}

/// Trace: FR-038-AC-284, FR-038-AC-285
#[trace("TC-048")]
#[test]
fn clause_references_check_dependencies_before_target_forms() {
    let mut package = v2_all_families();
    let clause = clause_position(&package);
    let root = package["semantic_graph"]["nodes"][clause]["body"]["arguments"][11]["value"]
        ["target"]
        .clone();
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][0]["value"]["target"] = root;
    let wrong_relationship = package["semantic_graph"]["nodes"][clause]["body"]["arguments"][0]
        ["value"]["target"]
        .clone();
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][4]["value"]["members"] =
        json!([{"term": "reference", "target": wrong_relationship}]);
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][clause]["node_id"]["digest"]
        .as_str()
        .expect("clause key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidModelBinding,
            &format!("/semantic_graph/nodes/{clause}/body/arguments/0/value/target"),
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            key,
        )
    );
}

/// Trace: FR-038-AC-284, FR-038-AC-285
#[trace("TC-048")]
#[test]
fn clause_relationship_wrong_export_has_distinct_cause() {
    let mut package = v2_all_families();
    let clause = clause_position(&package);
    let over = package["semantic_graph"]["nodes"][clause]["body"]["arguments"][0]["value"]
        ["target"]
        .clone();
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][4]["value"]["members"] =
        json!([{"term": "reference", "target": over}]);
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][clause]["node_id"]["digest"]
        .as_str()
        .expect("clause key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidModelBinding,
            &format!("/semantic_graph/nodes/{clause}/body/arguments/4/value/members/0/target"),
            Some(CheckedPackageRefusalCause::WrongExport),
            key,
        )
    );
}

/// Trace: FR-038-AC-284, FR-038-AC-291, FR-038-AC-324
#[trace("TC-048")]
#[test]
fn clause_each_trigger_admits_and_checks_parameter_level() {
    let mut package = v2_all_families();
    let trigger = clause_parameter(&mut package, "opened", "1");
    let clause = clause_position(&package);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][1]["value"]["members"] = json!([
        {"term": "binding", "name": "kind", "value": {"term": "literal", "type": node_id(&family_key("a1a1")), "value_kind": "text", "value": "each"}},
        {"term": "binding", "name": "trigger", "value": {"term": "reference", "target": trigger}},
    ]);
    settle(&mut package);
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    let trigger_position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
    package["semantic_graph"]["nodes"][trigger_position]["body"]["members"][1]["value"]["value"] =
        json!("2");
    let wrong_key = structural_key(
        "value",
        "parameter",
        Some(&family_key("aaaa")),
        &package["semantic_graph"]["nodes"][trigger_position]["body"],
    );
    package["semantic_graph"]["nodes"][trigger_position]["node_id"] = node_id(&wrong_key);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][1]["value"]["members"][1]
        ["value"]["target"] = node_id(&wrong_key);
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][trigger_position]["node_id"]["digest"]
        .as_str()
        .expect("clause key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            &format!("/semantic_graph/nodes/{trigger_position}"),
            None,
            key,
        )
    );
}

/// Trace: FR-038-AC-284, FR-038-AC-324
#[trace("TC-048")]
#[test]
fn clause_each_trigger_requires_declared_dependency() {
    let mut package = v2_all_families();
    let trigger = clause_parameter(&mut package, "opened", "1");
    let clause = clause_position(&package);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][1]["value"]["members"] = json!([
        {"term": "binding", "name": "kind", "value": {"term": "literal", "type": node_id(&family_key("a1a1")), "value_kind": "text", "value": "each"}},
        {"term": "binding", "name": "trigger", "value": {"term": "reference", "target": trigger}},
    ]);
    settle(&mut package);
    package["semantic_graph"]["nodes"][clause]["dependencies"]
        .as_array_mut()
        .expect("dependencies")
        .retain(|dependency| dependency != &trigger);
    refresh_identity(&mut package);
    let key = package["semantic_graph"]["nodes"][clause]["node_id"]["digest"]
        .as_str()
        .expect("clause key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::MissingDeclaration,
            &format!(
                "/semantic_graph/nodes/{clause}/body/arguments/1/value/members/1/value/target"
            ),
            Some(CheckedPackageRefusalCause::MissingName),
            key,
        )
    );
}

/// Trace: FR-038-AC-284
#[trace("TC-048")]
#[test]
fn clause_each_guard_requires_boolean_root() {
    let mut package = v2_all_families();
    let trigger = clause_parameter(&mut package, "opened", "1");
    let clause = clause_position(&package);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][1]["value"]["members"] = json!([
        {"term": "binding", "name": "kind", "value": {"term": "literal", "type": node_id(&family_key("a1a1")), "value_kind": "text", "value": "each"}},
        {"term": "binding", "name": "trigger", "value": {"term": "reference", "target": trigger}},
        {"term": "binding", "name": "guard", "value": {"term": "reference", "target": node_id(&family_key("a1a1"))}},
    ]);
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][clause]["node_id"]["digest"]
        .as_str()
        .expect("clause key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::IllTyped,
            &format!("/semantic_graph/nodes/{clause}/body/arguments/1/value/members/2/value"),
            Some(CheckedPackageRefusalCause::NonBooleanRoot),
            key,
        )
    );
}

/// Trace: FR-038-AC-284
#[trace("TC-048")]
#[test]
fn clause_capture_checks_binder_form_and_ordered_level() {
    let mut package = v2_all_families();
    let binder = clause_parameter(&mut package, "captured", "1");
    let clause = clause_position(&package);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][2]["value"]["members"] = json!([{"term": "aggregate", "members": [
        {"term": "binding", "name": "binder", "value": {"term": "reference", "target": binder}},
        {"term": "binding", "name": "initializer", "value": {"term": "literal", "type": node_id(&family_key("aaaa")), "value_kind": "boolean", "value": true}},
    ]}]);
    settle(&mut package);
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    let root = package["semantic_graph"]["nodes"][clause]["body"]["arguments"][11]["value"]
        ["target"]
        .clone();
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][2]["value"]["members"][0]
        ["members"][0]["value"]["target"] = root;
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][clause]["node_id"]["digest"]
        .as_str()
        .expect("clause key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidModelBinding,
            &format!(
                "/semantic_graph/nodes/{clause}/body/arguments/2/value/members/0/members/0/value/target"
            ),
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            key,
        )
    );
}

/// Trace: FR-038-AC-284, FR-038-AC-291
#[trace("TC-048")]
#[test]
fn clause_capture_order_sets_each_binder_level() {
    let mut package = v2_all_families();
    let first = clause_parameter(&mut package, "first", "1");
    let second = clause_parameter(&mut package, "second", "2");
    let clause = clause_position(&package);
    let capture = |binder: Value| {
        json!({"term": "aggregate", "members": [
            {"term": "binding", "name": "binder", "value": {"term": "reference", "target": binder}},
            {"term": "binding", "name": "initializer", "value": {"term": "literal", "type": node_id(&family_key("aaaa")), "value_kind": "boolean", "value": true}},
        ]})
    };
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][2]["value"]["members"] =
        json!([capture(first), capture(second)]);
    settle(&mut package);
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][2]["value"]["members"]
        .as_array_mut()
        .expect("captures")
        .swap(0, 1);
    settle(&mut package);
    let second_position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
    let key = package["semantic_graph"]["nodes"][second_position]["node_id"]["digest"]
        .as_str()
        .expect("second binder key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            &format!("/semantic_graph/nodes/{second_position}"),
            None,
            key,
        )
    );
}

/// Trace: FR-038-AC-291
#[trace("TC-048")]
#[test]
fn clause_rejects_second_capture_with_readable_name() {
    let mut package = v2_all_families();
    let first = clause_parameter(&mut package, "captured", "1");
    let second = clause_parameter(&mut package, "captured", "2");
    let clause = clause_position(&package);
    let capture = |binder: Value| {
        json!({"term": "aggregate", "members": [
            {"term": "binding", "name": "binder", "value": {"term": "reference", "target": binder}},
            {"term": "binding", "name": "initializer", "value": {"term": "literal", "type": node_id(&family_key("aaaa")), "value_kind": "boolean", "value": true}},
        ]})
    };
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][2]["value"]["members"] =
        json!([capture(first), capture(second)]);
    settle(&mut package);
    let second_position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
    let key = package["semantic_graph"]["nodes"][second_position]["node_id"]["digest"]
        .as_str()
        .expect("second binder key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::AmbiguousDeclaration,
            &format!("/semantic_graph/nodes/{second_position}"),
            Some(CheckedPackageRefusalCause::AmbiguousName),
            key,
        )
    );
}

/// Trace: FR-038-AC-284, FR-038-AC-324
#[trace("TC-048")]
#[test]
fn clause_required_temporal_target_is_checked_after_membership() {
    let mut package = v2_all_families();
    let temporal = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "temporal" && node["semantic_form"] == "temporal_clause")
        .expect("temporal clause")["node_id"]
        .clone();
    let clause = clause_position(&package);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][7]["value"]["members"] =
        json!([{"term": "reference", "target": temporal}]);
    settle(&mut package);
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][7]["value"]["members"] = json!([
        {"term": "reference", "target": temporal},
        {"term": "reference", "target": temporal},
    ]);
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][clause]["node_id"]["digest"]
        .as_str()
        .expect("clause key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            &format!("/semantic_graph/nodes/{clause}"),
            None,
            key,
        )
    );
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][7]["value"]["members"] =
        json!([{"term": "reference", "target": temporal}]);
    settle(&mut package);
    package["semantic_graph"]["nodes"][clause]["dependencies"]
        .as_array_mut()
        .expect("dependencies")
        .retain(|dependency| dependency != &temporal);
    refresh_identity(&mut package);
    let key = package["semantic_graph"]["nodes"][clause]["node_id"]["digest"]
        .as_str()
        .expect("clause key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::MissingDeclaration,
            &format!("/semantic_graph/nodes/{clause}/body/arguments/7/value/members/0/target"),
            Some(CheckedPackageRefusalCause::MissingName),
            key,
        )
    );
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
    protocol_holder_case_on(v2_all_families(), operator)
}

fn protocol_holder_case_on(mut package: Value, operator: &str) -> (Value, Vec<usize>) {
    let root = protocol_position(&package, "control");
    let leaf_body = package["semantic_graph"]["nodes"][root]["body"].clone();
    let role = protocol_role_node();
    let role_target = role["node_id"].clone();
    let binder_target = package["semantic_graph"]["nodes"]
        [protocol_position(&package, "obligation")]["body"]["arguments"][0]["value"]["target"]
        .clone();
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
                {"term": "binding", "name": "binder", "value": {"term": "reference", "target": binder_target}},
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
    let anchor = if operator == "await" {
        let anchor_body = package["semantic_graph"]["nodes"][positions[0]]["body"].clone();
        let anchor = protocol_named_node(
            "control",
            &["Example", "OrderFlow", "run", "Main", "Anchor"],
            anchor_body,
            &[],
        );
        let target = anchor["node_id"].clone();
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(anchor);
        body["arguments"][0]["value"]["members"][1]["value"]["target"] = target.clone();
        Some(target)
    } else {
        None
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
        if let Some(anchor) = anchor {
            json!([reference(&anchor), reference(&holder_key)])
        } else {
            json!([reference(&holder_key)])
        };
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
    for &child in children.iter().rev() {
        empty["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .remove(child);
    }
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
        package["semantic_graph"]["nodes"][protocol_position(&package, "role")]["node_id"].clone();
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
    no_bound["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .remove(children[1]);
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
fn repeat_admits_proof_and_zero_maximum_and_refuses_both_unpaired_directions() {
    let (package, children) = protocol_holder_case("repeat");
    let holder = holder_position(&package);
    let mut with_proof = package.clone();
    with_proof["semantic_graph"]["nodes"][holder]["body"]["arguments"][3]["value"]["members"] = json!([
        {"term": "binding", "name": "invariant", "value": {"term": "literal", "type": node_id(&family_key("aaaa")), "value_kind": "boolean", "value": true}},
        {"term": "binding", "name": "variant", "value": {"term": "literal", "type": node_id(&family_key("a3a3")), "value_kind": "integer", "value": "1"}}
    ]);
    settle(&mut with_proof);
    assert!(matches!(
        read(&with_proof),
        CheckedPackageV2ReadResult::Admitted(_)
    ));

    let mut zero = with_proof.clone();
    zero["semantic_graph"]["nodes"][holder]["body"]["arguments"][2]["value"]["members"][0]
        ["value"] = json!("0");
    settle(&mut zero);
    assert!(matches!(
        read(&zero),
        CheckedPackageV2ReadResult::Admitted(_)
    ));

    for missing in [2, 6] {
        let mut unpaired = with_proof.clone();
        unpaired["semantic_graph"]["nodes"][holder]["body"]["arguments"][missing]["value"]
            ["members"] = json!([]);
        settle(&mut unpaired);
        assert_eq!(
            read(&unpaired),
            refusal(
                CheckedPackageRefusalCode::MalformedWire,
                &format!("/semantic_graph/nodes/{holder}/body")
            ),
            "unpaired argument {missing} must refuse"
        );
    }

    let mut proof_only = with_proof;
    for absent in [2, 6] {
        proof_only["semantic_graph"]["nodes"][holder]["body"]["arguments"][absent]["value"]
            ["members"] = json!([]);
    }
    proof_only["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .remove(children[1]);
    settle(&mut proof_only);
    assert!(matches!(
        read(&proof_only),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn repeat_non_boolean_guard_refuses_at_guard_not_wire() {
    let (mut package, _) = protocol_holder_case("repeat");
    let holder = holder_position(&package);
    package["semantic_graph"]["nodes"][holder]["body"]["arguments"][4]["value"] = json!({
        "term": "literal", "type": node_id(&family_key("a1a1")), "value_kind": "text", "value": "yes"
    });
    settle(&mut package);
    holder_refusal(
        &package,
        CheckedPackageRefusalCode::IllTyped,
        Some(CheckedPackageRefusalCause::NonBooleanRoot),
        "/body/arguments/4/value",
    );
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn await_checks_anchor_form_and_event_eligibility() {
    let (package, children) = protocol_holder_case("await");
    assert!(
        matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
        "{:?}",
        read(&package)
    );
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
    let role = wrong_event["semantic_graph"]["nodes"][protocol_position(&wrong_event, "role")]
        ["node_id"]
        .clone();
    wrong_event["semantic_graph"]["nodes"][children[0]]["body"]["operation"]["member"]
        ["operator"] = json!("send");
    wrong_event["semantic_graph"]["nodes"][children[0]]["body"]["arguments"] = json!([
        {"term": "binding", "name": "channel", "value": {"term": "reference", "target": role}},
        {"term": "binding", "name": "binder", "value": {"term": "reference", "target": role}},
        {"term": "binding", "name": "related", "value": {"term": "aggregate", "members": []}},
        {"term": "binding", "name": "constraint", "value": {"term": "literal", "type": node_id(&family_key("aaaa")), "value_kind": "boolean", "value": true}}
    ]);
    settle(&mut wrong_event);
    holder_refusal(
        &wrong_event,
        CheckedPackageRefusalCode::UnsupportedConstruct,
        Some(CheckedPackageRefusalCause::DeclarationForm),
        "",
    );
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&wrong_event) else {
        panic!("ineligible send event must refuse");
    };
    assert_eq!(
        refusal.protocol_detail.as_deref(),
        Some(&CheckedProtocolRefusalDetail::AwaitedEvent {
            form: "send".into(),
            profile: quire_contract_model::CheckedArtifactRef {
                authority: "agent-ix".into(),
                identity: "quire.protocol.finite-global/v1".into(),
            },
        })
    );
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn await_inverted_interval_refuses_with_written_bounds() {
    let (mut package, _) = protocol_holder_case("await");
    let holder = holder_position(&package);
    package["semantic_graph"]["nodes"][holder]["body"]["arguments"][2]["value"]["members"][0]
        ["value"]["value"] = json!("2");
    settle(&mut package);
    holder_refusal(
        &package,
        CheckedPackageRefusalCode::IllTyped,
        Some(CheckedPackageRefusalCause::TypeMismatch),
        "/body/arguments/2/value",
    );
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("inverted await interval must refuse");
    };
    assert_eq!(
        refusal.protocol_detail.as_deref(),
        Some(&CheckedProtocolRefusalDetail::Interval {
            member: "within".into(),
            expected: "closed interval with lower <= upper".into(),
            actual: "[2, 1]".into(),
        })
    );
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn await_effect_event_requires_an_attempt_source() {
    let (mut package, children) = protocol_holder_case("await");
    let role =
        package["semantic_graph"]["nodes"][protocol_position(&package, "role")]["node_id"].clone();
    let reference = || json!({"term": "reference", "target": role});
    let binding =
        |name: &str, value: Value| json!({"term": "binding", "name": name, "value": value});
    let arguments = json!([
        binding("of", reference()),
        binding("binder", reference()),
        binding("related", json!({"term": "aggregate", "members": []})),
        binding(
            "constraint",
            json!({
                "term": "literal", "type": node_id(&family_key("aaaa")), "value_kind": "boolean", "value": true
            })
        )
    ]);
    package["semantic_graph"]["nodes"][children[0]]["body"]["operation"]["member"]["operator"] =
        json!("effect");
    package["semantic_graph"]["nodes"][children[0]]["body"]["arguments"] = arguments;
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][children[0]]["node_id"]["digest"]
        .as_str()
        .expect("effect key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidModelBinding,
            &format!(
                "/semantic_graph/nodes/{}/body/arguments/0/value/target",
                children[0]
            ),
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            key
        )
    );
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn await_admits_compensation_anchor_form() {
    let (state, _) = crate::checked_package_v2_frame_entries::state_operand_fixture();
    let (mut package, _) = protocol_holder_case_on(state, "await");
    let holder = holder_position(&package);
    let compensation = compensation_template(
        &mut package,
        &["Example", "OrderFlow", "compensate", "Undo"],
    );
    let target = compensation["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(compensation);
    let clause = clause_position(&package);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][8]["value"]["members"] =
        json!([{"term": "reference", "target": target}]);
    package["semantic_graph"]["nodes"][holder]["body"]["arguments"][0]["value"]["members"][0]
        ["value"]["value"] = json!("compensation");
    package["semantic_graph"]["nodes"][holder]["body"]["arguments"][0]["value"]["members"][1]
        ["value"]["target"] = target;
    let root = protocol_position(&package, "control");
    package["semantic_graph"]["nodes"][root]["body"]["arguments"][0]["value"]["members"]
        .as_array_mut()
        .expect("root sequence")
        .rotate_right(2);
    settle(&mut package);
    assert!(
        matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
        "compensation anchor must admit"
    );
}

fn compensation_template(package: &mut Value, declaration: &[&str]) -> Value {
    let role =
        package["semantic_graph"]["nodes"][protocol_position(package, "role")]["node_id"].clone();
    let binder = package["semantic_graph"]["nodes"][protocol_position(package, "obligation")]
        ["body"]["arguments"][0]["value"]["target"]
        .clone();
    let anchor = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["semantic_form"] == "operation_anchor")
        .expect("operation anchor")["node_id"]
        .clone();
    let namespace = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "model" && node["semantic_form"] == "namespace")
        .expect("namespace model")
        .clone();
    let mut record_type = namespace;
    record_type["semantic_form"] = json!("record_value_type");
    let record_key = structural_key(
        "model",
        "record_value_type",
        record_type["semantic_type"]["digest"].as_str(),
        &record_type["body"],
    );
    record_type["node_id"] = node_id(&record_key);
    let record_target = record_type["node_id"].clone();
    let parameter = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "value" && node["semantic_form"] == "parameter")
        .expect("parameter")
        .clone();
    let retry_binder = |name: &str| {
        let mut node = parameter.clone();
        node["semantic_type"] = record_target.clone();
        node["body"]["members"][0]["value"]["value"] = json!(name);
        node["node_id"] = node_id(&structural_key(
            "value",
            "parameter",
            Some(&record_key),
            &node["body"],
        ));
        node
    };
    let earlier = retry_binder("earlier");
    let later = retry_binder("later");
    let earlier_target = earlier["node_id"].clone();
    let later_target = later["node_id"].clone();
    let capture_binder = |name: &str| {
        let mut node = parameter.clone();
        node["body"]["members"][0]["value"]["value"] = json!(name);
        node["node_id"] = node_id(&structural_key(
            "value",
            "parameter",
            Some(&family_key("aaaa")),
            &node["body"],
        ));
        node
    };
    let registration = capture_binder("registration");
    let activation = capture_binder("activation");
    let registration_target = registration["node_id"].clone();
    let activation_target = activation["node_id"].clone();
    let group = || json!({"term": "aggregate", "members": []});
    let binding =
        |name: &str, value: Value| json!({"term": "binding", "name": name, "value": value});
    let reference = |target: &Value| json!({"term": "reference", "target": target});
    let boolean = || {
        json!({"term": "literal", "type": node_id(&family_key("aaaa")),
        "value_kind": "boolean", "value": true})
    };
    let integer = |value: &str| {
        json!({"term": "literal", "type": node_id(&family_key("a3a3")),
        "value_kind": "integer", "value": value})
    };
    let root = protocol_position(package, "control");
    let mut attempt_body = package["semantic_graph"]["nodes"][root]["body"].clone();
    attempt_body["operation"]["member"]["operator"] = json!("attempt");
    attempt_body["arguments"] = json!([
        binding("role", reference(&role)),
        binding("operation", reference(&anchor)),
        binding("ordering", group()),
        binding("contracts", group()),
        binding("binder", reference(&binder)),
        binding("related", group()),
        binding("constraint", boolean())
    ]);
    let attempt = protocol_named_node(
        "control",
        &["Example", "OrderFlow", "run", "Main", "ForwardAttempt"],
        attempt_body,
        &[],
    );
    let attempt_target = attempt["node_id"].clone();
    let mut effect_body = package["semantic_graph"]["nodes"][root]["body"].clone();
    effect_body["operation"]["member"]["operator"] = json!("effect");
    effect_body["arguments"] = json!([
        binding("of", reference(&attempt_target)),
        binding("binder", reference(&binder)),
        binding("related", group()),
        binding("constraint", boolean())
    ]);
    let effect = protocol_named_node(
        "control",
        &["Example", "OrderFlow", "run", "Main", "ForwardEffect"],
        effect_body,
        &[],
    );
    let effect_target = effect["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend([
            record_type,
            earlier,
            later,
            registration,
            activation,
            attempt,
            effect,
        ]);
    let root_body = &mut package["semantic_graph"]["nodes"][root]["body"];
    if root_body["operation"]["member"]["operator"] == "sequence" {
        root_body["arguments"][0]["value"]["members"]
            .as_array_mut()
            .expect("sequence children")
            .extend([reference(&attempt_target), reference(&effect_target)]);
    } else {
        let original_target = package["semantic_graph"]["nodes"][root]["node_id"].clone();
        package["semantic_graph"]["nodes"][root]["declaration"]["qualified_name"] =
            json!(["Example", "OrderFlow", "run", "Main", "Event"]);
        let mut sequence_body = package["semantic_graph"]["nodes"][root]["body"].clone();
        sequence_body["operation"]["member"]["operator"] = json!("sequence");
        sequence_body["arguments"] = json!([binding(
            "children",
            json!({"term": "aggregate", "members": [
                reference(&attempt_target),
                reference(&effect_target),
                reference(&original_target)
            ]})
        )]);
        let sequence = protocol_named_node(
            "control",
            &["Example", "OrderFlow", "run", "Main"],
            sequence_body,
            &[],
        );
        let sequence_target = sequence["node_id"].clone();
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(sequence);
        let clause = clause_position(package);
        package["semantic_graph"]["nodes"][clause]["body"]["arguments"][11]["value"]["target"] =
            sequence_target;
    }
    let body =
        package["semantic_graph"]["nodes"][protocol_position(package, "control")]["body"].clone();
    let mut compensation = protocol_named_node("compensation", declaration, body, &[]);
    compensation["body"]["operation"]["member"]["operator"] = json!("compensate");
    compensation["body"]["arguments"] = json!([
        binding("forward", reference(&effect_target)),
        binding("forward_binder", reference(&binder)),
        binding("role", reference(&role)),
        binding("operation", reference(&anchor)),
        binding(
            "clock",
            json!({"term": "literal", "type": node_id(&family_key("a1a1")), "value_kind": "text", "value": "clock"})
        ),
        binding(
            "registration_captures",
            json!({"term": "aggregate", "members": [
                {"term": "aggregate", "members": [
                    binding("binder", reference(&registration_target)),
                    binding("initializer", boolean())
                ]}
            ]})
        ),
        binding(
            "trigger",
            json!({"term": "aggregate", "members": [
                binding("binder", reference(&binder)), binding("guard", boolean())
            ]})
        ),
        binding(
            "activation_captures",
            json!({"term": "aggregate", "members": [
                {"term": "aggregate", "members": [
                    binding("binder", reference(&activation_target)),
                    binding("initializer", boolean())
                ]}
            ]})
        ),
        binding(
            "within",
            json!({"term": "aggregate", "members": [
                binding("lower", integer("0")), binding("upper", integer("1"))
            ]})
        ),
        binding(
            "attempts",
            json!({"term": "aggregate", "members": [
                binding("maximum", integer("1")), binding("record_type", reference(&record_target))
            ]})
        ),
        binding(
            "retry",
            json!({"term": "aggregate", "members": [
                binding("earlier", reference(&earlier_target)),
                binding("later", reference(&later_target)), binding("relation", boolean())
            ]})
        ),
        binding("commit", group()),
        binding(
            "recovery",
            json!({"term": "aggregate", "members": [
                binding("binder", reference(&binder)), binding("predicate", boolean())
            ]})
        )
    ]);
    compensation
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn repeat_maximum_and_await_interval_reject_negative_bounds_at_wire() {
    let (mut repeated, _) = protocol_holder_case("repeat");
    let holder = holder_position(&repeated);
    repeated["semantic_graph"]["nodes"][holder]["body"]["arguments"][2]["value"]["members"][0]
        ["value"] = json!("-1");
    assert_eq!(
        read(&repeated),
        refusal_at(
            CheckedPackageRefusalCode::InvalidPackage,
            &format!("/semantic_graph/nodes/{holder}/body/arguments/2/value/members/0"),
            Some(CheckedPackageRefusalCause::InvalidValue),
            repeated["semantic_graph"]["nodes"][holder]["node_id"]["digest"]
                .as_str()
                .expect("id"),
        )
    );

    let (mut awaited, _) = protocol_holder_case("await");
    let holder = holder_position(&awaited);
    awaited["semantic_graph"]["nodes"][holder]["body"]["arguments"][2]["value"]["members"][1]
        ["value"]["value"] = json!("-1");
    assert_eq!(
        read(&awaited),
        refusal_at(
            CheckedPackageRefusalCode::InvalidPackage,
            &format!("/semantic_graph/nodes/{holder}/body/arguments/2/value/members/1/value"),
            Some(CheckedPackageRefusalCause::InvalidValue),
            awaited["semantic_graph"]["nodes"][holder]["node_id"]["digest"]
                .as_str()
                .expect("id"),
        )
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
    package_with_member_role_on(v2_all_families())
}

fn whole_model_role(identity: &str) -> (Value, usize) {
    let (mut package, role) = package_with_member_role();
    package["semantic_graph"]["nodes"][role]["body"]["arguments"][0]["value"] = json!({
        "term": "literal", "type": node_id(&family_key("a1a1")),
        "value_kind": "text", "value": identity,
    });
    settle(&mut package);
    (package, role)
}

fn package_with_member_role_on(mut package: Value) -> (Value, usize) {
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
    let position = add_member_channel(&mut package, role);
    (package, role, position)
}

fn add_member_channel(package: &mut Value, role: usize) -> usize {
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
        package["semantic_graph"]["nodes"][protocol_position(package, "control")]["body"].clone();
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
    let clause = clause_position(package);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][5]["value"]["members"] =
        json!([reference(&channel_key)]);
    settle(package);
    position
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

fn package_with_send() -> (Value, usize, usize) {
    let (mut package, _) = package_with_member_queue();
    let channel = protocol_position(&package, "channel");
    let root = protocol_position(&package, "control");
    let binder = package["semantic_graph"]["nodes"][protocol_position(&package, "obligation")]
        ["body"]["arguments"][0]["value"]["target"]
        .clone();
    let channel_key = package["semantic_graph"]["nodes"][channel]["node_id"].clone();
    let body = &mut package["semantic_graph"]["nodes"][root]["body"];
    body["operation"]["member"]["operator"] = json!("send");
    body["arguments"] = json!([
        {"term": "binding", "name": "channel", "value": {"term": "reference", "target": channel_key}},
        {"term": "binding", "name": "binder", "value": {"term": "reference", "target": binder}},
        {"term": "binding", "name": "related", "value": {"term": "aggregate", "members": []}},
        {"term": "binding", "name": "constraint", "value": {
            "term": "literal", "type": node_id(&family_key("aaaa")), "value_kind": "boolean", "value": true
        }}
    ]);
    settle(&mut package);
    (package, root, channel)
}

fn package_with_step_control(operator: &str) -> (Value, usize) {
    package_with_step_control_on(v2_all_families(), operator)
}

fn package_with_step_control_on(package: Value, operator: &str) -> (Value, usize) {
    let (mut package, _) = package_with_member_role_on(package);
    let root = protocol_position(&package, "control");
    let role =
        package["semantic_graph"]["nodes"][protocol_position(&package, "role")]["node_id"].clone();
    let binder = package["semantic_graph"]["nodes"][protocol_position(&package, "obligation")]
        ["body"]["arguments"][0]["value"]["target"]
        .clone();
    let boolean = json!({
        "term": "literal", "type": node_id(&family_key("aaaa")),
        "value_kind": "boolean", "value": true
    });
    let arguments = match operator {
        "attempt" => {
            json!([
                {"term": "binding", "name": "role", "value": {"term": "reference", "target": role}},
                {"term": "binding", "name": "operation", "value": {"term": "reference", "target": role}},
                {"term": "binding", "name": "ordering", "value": {"term": "aggregate", "members": []}},
                {"term": "binding", "name": "contracts", "value": {"term": "aggregate", "members": []}},
                {"term": "binding", "name": "binder", "value": {"term": "reference", "target": binder}},
                {"term": "binding", "name": "related", "value": {"term": "aggregate", "members": []}},
                {"term": "binding", "name": "constraint", "value": boolean}
            ])
        }
        "event" => json!([
            {"term": "binding", "name": "role", "value": {"term": "reference", "target": role}},
            {"term": "binding", "name": "compensation", "value": {"term": "aggregate", "members": []}},
            {"term": "binding", "name": "binder", "value": {"term": "reference", "target": binder}},
            {"term": "binding", "name": "related", "value": {"term": "aggregate", "members": []}},
            {"term": "binding", "name": "constraint", "value": boolean}
        ]),
        "commit" => json!([
            {"term": "binding", "name": "role", "value": {"term": "reference", "target": role}},
            {"term": "binding", "name": "binder", "value": {"term": "reference", "target": binder}},
            {"term": "binding", "name": "constraint", "value": boolean}
        ]),
        "effect" => json!([
            {"term": "binding", "name": "of", "value": {"term": "reference", "target": role}},
            {"term": "binding", "name": "binder", "value": {"term": "reference", "target": binder}},
            {"term": "binding", "name": "related", "value": {"term": "aggregate", "members": []}},
            {"term": "binding", "name": "constraint", "value": boolean}
        ]),
        _ => panic!("unsupported fixture operator"),
    };
    let body = &mut package["semantic_graph"]["nodes"][root]["body"];
    body["operation"]["member"]["operator"] = json!(operator);
    body["arguments"] = arguments;
    settle(&mut package);
    (package, root)
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn event_and_commit_check_local_references_and_boolean_constraints() {
    for operator in ["event", "commit"] {
        let (mut package, root) = package_with_step_control(operator);
        assert!(
            matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
            "{operator}: {:?}",
            read(&package)
        );
        let ordinal = if operator == "event" { 4 } else { 2 };
        package["semantic_graph"]["nodes"][root]["body"]["arguments"][ordinal]["value"] = json!({"term": "literal", "type": node_id(&family_key("a1a1")),
                "value_kind": "text", "value": "wrong"});
        settle(&mut package);
        let key = package["semantic_graph"]["nodes"][root]["node_id"]["digest"]
            .as_str()
            .expect("control key");
        assert_eq!(
            read(&package),
            refusal_at(
                CheckedPackageRefusalCode::IllTyped,
                &format!("/semantic_graph/nodes/{root}/body/arguments/{ordinal}/value"),
                Some(CheckedPackageRefusalCause::NonBooleanRoot),
                key
            )
        );
    }
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn attempt_requires_a_state_operation_anchor() {
    let (package, root) = package_with_step_control("attempt");
    let key = package["semantic_graph"]["nodes"][root]["node_id"]["digest"]
        .as_str()
        .expect("attempt key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidModelBinding,
            &format!("/semantic_graph/nodes/{root}/body/arguments/1/value/target"),
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            key
        )
    );
}

fn package_with_admitted_attempt() -> (Value, usize, Value) {
    let (state, _) = crate::checked_package_v2_frame_entries::state_operand_fixture();
    let (mut package, root) = package_with_step_control_on(state, "attempt");
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    let anchor = nodes
        .iter()
        .find(|node| node["semantic_form"] == "operation_anchor")
        .expect("operation anchor")["node_id"]
        .clone();
    let precondition = nodes
        .iter()
        .find(|node| {
            node["semantic_form"] == "state_clause"
                && node["body"]["operation"]["member"]["clause"] == "precondition"
        })
        .expect("precondition")["node_id"]
        .clone();
    package["semantic_graph"]["nodes"][root]["body"]["arguments"][1]["value"]["target"] =
        anchor.clone();
    package["semantic_graph"]["nodes"][root]["body"]["arguments"][3]["value"]["members"] =
        json!([{"term": "reference", "target": precondition}]);
    settle(&mut package);
    (package, root, anchor)
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn attempt_admits_operation_contract_and_rejects_wrong_contract_kind_and_anchor() {
    let (package, root, anchor) = package_with_admitted_attempt();
    assert!(
        matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
        "{:?}",
        read(&package)
    );

    let mut wrong_kind = package.clone();
    let invariant = wrong_kind["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| {
            node["semantic_form"] == "state_clause"
                && node["body"]["operation"]["member"]["clause"] == "invariant"
        })
        .expect("invariant")["node_id"]
        .clone();
    wrong_kind["semantic_graph"]["nodes"][root]["body"]["arguments"][3]["value"]["members"][0]
        ["target"] = invariant;
    settle(&mut wrong_kind);
    let key = wrong_kind["semantic_graph"]["nodes"][root]["node_id"]["digest"]
        .as_str()
        .expect("attempt key");
    assert_eq!(
        read(&wrong_kind),
        refusal_at(
            CheckedPackageRefusalCode::InvalidModelBinding,
            &format!("/semantic_graph/nodes/{root}/body/arguments/3/value/members/0/target"),
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            key
        )
    );

    let mut wrong_anchor = package;
    let anchor_position = wrong_anchor["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_id"] == anchor)
        .expect("anchor position");
    let mut withdraw = wrong_anchor["semantic_graph"]["nodes"][anchor_position].clone();
    withdraw["body"]["members"][1]["value"]["value"] = json!("withdraw");
    let context = withdraw["semantic_type"]["digest"]
        .as_str()
        .expect("context key");
    let withdraw_key = structural_key(
        "state",
        "operation_anchor",
        Some(context),
        &withdraw["body"],
    );
    withdraw["node_id"] = node_id(&withdraw_key);
    wrong_anchor["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(withdraw);
    wrong_anchor["semantic_graph"]["nodes"][root]["body"]["arguments"][1]["value"]["target"] =
        node_id(&withdraw_key);
    settle(&mut wrong_anchor);
    let key = wrong_anchor["semantic_graph"]["nodes"][root]["node_id"]["digest"]
        .as_str()
        .expect("attempt key");
    assert_eq!(
        read(&wrong_anchor),
        refusal_at(
            CheckedPackageRefusalCode::InvalidModelBinding,
            &format!("/semantic_graph/nodes/{root}/body/arguments/3/value/members/0/target"),
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            key
        )
    );
}

fn package_with_admitted_effect() -> (Value, usize, Value) {
    let (mut package, root, anchor) = package_with_admitted_attempt();
    let attempt_body = package["semantic_graph"]["nodes"][root]["body"].clone();
    let binder = attempt_body["arguments"][4]["value"]["target"].clone();
    let attempt = protocol_named_node(
        "control",
        &["Example", "OrderFlow", "run", "Main", "Attempt"],
        attempt_body,
        &[],
    );
    let attempt_key = attempt["node_id"].clone();
    let mut effect_body = package["semantic_graph"]["nodes"][root]["body"].clone();
    effect_body["operation"]["member"]["operator"] = json!("effect");
    effect_body["arguments"] = json!([
        {"term": "binding", "name": "of", "value": {"term": "reference", "target": attempt_key}},
        {"term": "binding", "name": "binder", "value": {"term": "reference", "target": binder}},
        {"term": "binding", "name": "related", "value": {"term": "aggregate", "members": []}},
        {"term": "binding", "name": "constraint", "value": {"term": "literal",
            "type": node_id(&family_key("aaaa")), "value_kind": "boolean", "value": true}}
    ]);
    let effect = protocol_named_node(
        "control",
        &["Example", "OrderFlow", "run", "Main", "Effect"],
        effect_body,
        &[],
    );
    let effect_key = effect["node_id"].clone();
    let effect_position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        + 1;
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend([attempt, effect]);
    package["semantic_graph"]["nodes"][root]["body"]["operation"]["member"]["operator"] =
        json!("sequence");
    package["semantic_graph"]["nodes"][root]["body"]["arguments"] = json!([
        {"term": "binding", "name": "children", "value": {"term": "aggregate", "members": [
            {"term": "reference", "target": attempt_key},
            {"term": "reference", "target": effect_key}
        ]}}
    ]);
    settle(&mut package);
    (package, effect_position, anchor)
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn effect_admits_a_referenced_attempt_in_the_same_control_tree() {
    let (package, _, _) = package_with_admitted_effect();
    assert!(
        matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
        "{:?}",
        read(&package)
    );
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn effect_requires_an_attempt_target() {
    let (package, root) = package_with_step_control("effect");
    let key = package["semantic_graph"]["nodes"][root]["node_id"]["digest"]
        .as_str()
        .expect("effect key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidModelBinding,
            &format!("/semantic_graph/nodes/{root}/body/arguments/0/value/target"),
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            key
        )
    );
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn event_compensation_requires_a_template_target() {
    let (mut package, root) = package_with_step_control("event");
    let role =
        package["semantic_graph"]["nodes"][protocol_position(&package, "role")]["node_id"].clone();
    package["semantic_graph"]["nodes"][root]["body"]["arguments"][1]["value"]["members"] =
        json!([{"term": "reference", "target": role}]);
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][root]["node_id"]["digest"]
        .as_str()
        .expect("event key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidModelBinding,
            &format!("/semantic_graph/nodes/{root}/body/arguments/1/value/members/0/target"),
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            key
        )
    );
}

fn package_with_admitted_compensation() -> (Value, usize, usize) {
    let (state, _) = crate::checked_package_v2_frame_entries::state_operand_fixture();
    let (mut package, root) = package_with_step_control_on(state, "event");
    let local = compensation_template(
        &mut package,
        &["Example", "OrderFlow", "compensate", "Undo"],
    );
    let local_key = local["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(local);
    let clause = clause_position(&package);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][8]["value"]["members"] =
        json!([{"term": "reference", "target": local_key}]);
    package["semantic_graph"]["nodes"][root]["body"]["arguments"][1]["value"]["members"] =
        json!([{"term": "reference", "target": local_key}]);
    settle(&mut package);
    let compensation = protocol_position(&package, "compensation");
    (package, root, compensation)
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn event_admits_its_clause_template_and_refuses_a_foreign_template() {
    let (mut package, root, compensation) = package_with_admitted_compensation();
    assert!(
        matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
        "{:?}",
        read(&package)
    );

    let foreign = protocol_named_node(
        "compensation",
        &["Example", "Elsewhere", "compensate", "Undo"],
        package["semantic_graph"]["nodes"][compensation]["body"].clone(),
        &[],
    );
    let foreign_key = foreign["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(foreign);
    package["semantic_graph"]["nodes"][root]["body"]["arguments"][1]["value"]["members"][0]
        ["target"] = foreign_key;
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][root]["node_id"]["digest"]
        .as_str()
        .expect("event key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidModelBinding,
            &format!("/semantic_graph/nodes/{root}/body/arguments/1/value/members/0/target"),
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            key
        )
    );
}

/// Trace: FR-038-AC-282, FR-038-AC-289
#[trace("TC-048")]
#[test]
fn compensation_admits_local_bindings_and_rejects_wrong_target_forms() {
    let (package, _, compensation) = package_with_admitted_compensation();
    assert!(
        matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
        "{:?}",
        read(&package)
    );
    let role =
        package["semantic_graph"]["nodes"][protocol_position(&package, "role")]["node_id"].clone();
    for (ordinal, pointer) in [
        (
            0,
            format!("/semantic_graph/nodes/{compensation}/body/arguments/0/value/target"),
        ),
        (
            11,
            format!(
                "/semantic_graph/nodes/{compensation}/body/arguments/11/value/members/0/target"
            ),
        ),
    ] {
        let mut wrong = package.clone();
        if ordinal == 0 {
            wrong["semantic_graph"]["nodes"][compensation]["body"]["arguments"][0]["value"]
                ["target"] = role.clone();
        } else {
            wrong["semantic_graph"]["nodes"][compensation]["body"]["arguments"][11]["value"]
                ["members"] = json!([{"term": "reference", "target": role}]);
        }
        settle(&mut wrong);
        let key = wrong["semantic_graph"]["nodes"][compensation]["node_id"]["digest"]
            .as_str()
            .expect("compensation key");
        assert_eq!(
            read(&wrong),
            refusal_at(
                CheckedPackageRefusalCode::InvalidModelBinding,
                &pointer,
                Some(CheckedPackageRefusalCause::MalformedDeclaration),
                key,
            ),
            "argument {ordinal}"
        );
    }
}

/// Trace: FR-038-AC-282, FR-038-AC-289
#[trace("TC-048")]
#[test]
fn compensation_nonempty_captures_admit_and_check_record_binders() {
    let (package, _, compensation) = package_with_admitted_compensation();
    for ordinal in [5, 7] {
        assert_eq!(
            package["semantic_graph"]["nodes"][compensation]["body"]["arguments"][ordinal]["value"]
                ["members"]
                .as_array()
                .expect("capture records")
                .len(),
            1,
        );
    }
    assert!(
        matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
        "{:?}",
        read(&package)
    );

    let role =
        package["semantic_graph"]["nodes"][protocol_position(&package, "role")]["node_id"].clone();
    for ordinal in [5, 7] {
        let mut wrong_target = package.clone();
        wrong_target["semantic_graph"]["nodes"][compensation]["body"]["arguments"][ordinal]
            ["value"]["members"][0]["members"][0]["value"]["target"] = role.clone();
        settle(&mut wrong_target);
        let key = wrong_target["semantic_graph"]["nodes"][compensation]["node_id"]["digest"]
            .as_str()
            .expect("compensation key");
        assert_eq!(
            read(&wrong_target),
            refusal_at(
                CheckedPackageRefusalCode::InvalidModelBinding,
                &format!("/semantic_graph/nodes/{compensation}/body/arguments/{ordinal}/value/members/0/members/0/value/target"),
                Some(CheckedPackageRefusalCause::MalformedDeclaration),
                key,
            ),
            "capture binder at argument {ordinal}"
        );
    }

    let mut wrong_shape = package;
    wrong_shape["semantic_graph"]["nodes"][compensation]["body"]["arguments"][7]["value"]
        ["members"][0]["members"]
        .as_array_mut()
        .expect("capture fields")
        .swap(0, 1);
    assert_eq!(
        read(&wrong_shape),
        refusal(
            CheckedPackageRefusalCode::MalformedWire,
            &format!("/semantic_graph/nodes/{compensation}/body"),
        )
    );
}

/// Trace: FR-038-AC-282, FR-038-AC-289
#[trace("TC-048")]
#[test]
fn compensation_rejects_zero_attempts_inverted_window_and_non_boolean_guard() {
    let (package, _, compensation) = package_with_admitted_compensation();
    let cases = [
        (
            9,
            0,
            "0",
            CheckedPackageRefusalCode::IllTyped,
            CheckedPackageRefusalCause::TypeMismatch,
        ),
        (
            8,
            0,
            "2",
            CheckedPackageRefusalCode::IllTyped,
            CheckedPackageRefusalCause::TypeMismatch,
        ),
    ];
    for (ordinal, member, value, code, cause) in cases {
        let mut wrong = package.clone();
        wrong["semantic_graph"]["nodes"][compensation]["body"]["arguments"][ordinal]["value"]
            ["members"][member]["value"]["value"] = json!(value);
        settle(&mut wrong);
        let path = if ordinal == 9 {
            format!("/semantic_graph/nodes/{compensation}/body/arguments/9/value/members/0/value")
        } else {
            format!("/semantic_graph/nodes/{compensation}/body/arguments/8/value")
        };
        let key = wrong["semantic_graph"]["nodes"][compensation]["node_id"]["digest"]
            .as_str()
            .expect("compensation key");
        assert_eq!(read(&wrong), refusal_at(code, &path, Some(cause), key));
    }
    let mut wrong = package;
    wrong["semantic_graph"]["nodes"][compensation]["body"]["arguments"][6]["value"]["members"][1]
        ["value"] = json!({
        "term": "literal", "type": node_id(&family_key("a3a3")),
        "value_kind": "integer", "value": "1"
    });
    settle(&mut wrong);
    let key = wrong["semantic_graph"]["nodes"][compensation]["node_id"]["digest"]
        .as_str()
        .expect("compensation key");
    assert_eq!(
        read(&wrong),
        refusal_at(
            CheckedPackageRefusalCode::IllTyped,
            &format!("/semantic_graph/nodes/{compensation}/body/arguments/6/value/members/1/value"),
            Some(CheckedPackageRefusalCause::NonBooleanRoot),
            key,
        )
    );
}

/// Trace: FR-038-AC-282, FR-038-AC-289
#[trace("TC-048")]
#[test]
fn compensation_negative_window_bound_refuses_at_strict_wire() {
    let (mut package, _, compensation) = package_with_admitted_compensation();
    package["semantic_graph"]["nodes"][compensation]["body"]["arguments"][8]["value"]["members"]
        [0]["value"]["value"] = json!("-1");
    let key = package["semantic_graph"]["nodes"][compensation]["node_id"]["digest"]
        .as_str()
        .expect("compensation key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidPackage,
            &format!("/semantic_graph/nodes/{compensation}/body/arguments/8/value/members/0/value"),
            Some(CheckedPackageRefusalCause::InvalidValue),
            key,
        )
    );
}

/// Trace: FR-038-AC-282, FR-038-AC-289
#[trace("TC-048")]
#[test]
fn compensation_malformed_attempt_count_refuses_without_panicking() {
    let (mut package, _, compensation) = package_with_admitted_compensation();
    package["semantic_graph"]["nodes"][compensation]["body"]["arguments"][9]["value"]["members"]
        [0]["value"]["value"] = json!("not-an-integer");
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][compensation]["node_id"]["digest"]
        .as_str()
        .expect("compensation key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::MalformedWire,
            &format!("/semantic_graph/nodes/{compensation}/body/arguments/9/value/members/0/value"),
            None,
            key,
        )
    );
}

/// Trace: FR-038-AC-282, FR-038-AC-289
#[trace("TC-048")]
#[test]
fn compensation_text_and_integer_literals_require_their_scalar_types() {
    let (package, _, compensation) = package_with_admitted_compensation();
    let cases = [
        (
            4,
            None,
            format!("/semantic_graph/nodes/{compensation}/body/arguments/4/value"),
        ),
        (
            8,
            Some(0),
            format!("/semantic_graph/nodes/{compensation}/body/arguments/8/value/members/0/value"),
        ),
        (
            9,
            Some(0),
            format!("/semantic_graph/nodes/{compensation}/body/arguments/9/value/members/0/value"),
        ),
    ];
    for (ordinal, member, path) in cases {
        let mut wrong = package.clone();
        let literal = &mut wrong["semantic_graph"]["nodes"][compensation]["body"]["arguments"]
            [ordinal]["value"];
        if let Some(member) = member {
            literal["members"][member]["value"]["type"] = node_id(&family_key("aaaa"));
        } else {
            literal["type"] = node_id(&family_key("aaaa"));
        }
        settle(&mut wrong);
        let key = wrong["semantic_graph"]["nodes"][compensation]["node_id"]["digest"]
            .as_str()
            .expect("compensation key");
        assert_eq!(
            read(&wrong),
            refusal_at(
                CheckedPackageRefusalCode::IllTyped,
                &path,
                Some(CheckedPackageRefusalCause::TypeMismatch),
                key,
            ),
            "literal at argument {ordinal}"
        );
    }
}

/// Trace: FR-038-AC-289
#[trace("TC-048")]
#[test]
fn compensation_requires_forward_and_retry_binder_types() {
    let (package, _, compensation) = package_with_admitted_compensation();
    let forward_binder = package["semantic_graph"]["nodes"][compensation]["body"]["arguments"][1]
        ["value"]["target"]
        .clone();
    let earlier_binder = package["semantic_graph"]["nodes"][compensation]["body"]["arguments"][10]
        ["value"]["members"][0]["value"]["target"]
        .clone();
    for (ordinal, replacement) in [(1, earlier_binder), (10, forward_binder)] {
        let mut wrong = package.clone();
        if ordinal == 1 {
            wrong["semantic_graph"]["nodes"][compensation]["body"]["arguments"][1]["value"]
                ["target"] = replacement;
        } else {
            wrong["semantic_graph"]["nodes"][compensation]["body"]["arguments"][10]["value"]
                ["members"][0]["value"]["target"] = replacement;
        }
        settle(&mut wrong);
        let key = wrong["semantic_graph"]["nodes"][compensation]["node_id"]["digest"]
            .as_str()
            .expect("compensation key");
        assert_eq!(
            read(&wrong),
            refusal_at(
                CheckedPackageRefusalCode::IllTyped,
                &format!("/semantic_graph/nodes/{compensation}"),
                Some(CheckedPackageRefusalCause::TypeMismatch),
                key,
            ),
            "binder at argument {ordinal}"
        );
        let CheckedPackageV2ReadResult::Refused(refusal) = read(&wrong) else {
            panic!("mismatched binder must refuse");
        };
        assert!(matches!(
            refusal.protocol_detail.as_deref(),
            Some(CheckedProtocolRefusalDetail::PayloadType { expected, actual })
                if expected != actual
        ));
    }
}

fn package_with_present_commit() -> (Value, usize, usize) {
    let (mut package, _, compensation) = package_with_admitted_compensation();
    let role =
        package["semantic_graph"]["nodes"][protocol_position(&package, "role")]["node_id"].clone();
    let binder = package["semantic_graph"]["nodes"][compensation]["body"]["arguments"][1]["value"]
        ["target"]
        .clone();
    let mut body = package["semantic_graph"]["nodes"][compensation]["body"].clone();
    body["operation"]["member"]["operator"] = json!("commit");
    body["arguments"] = json!([
        {"term": "binding", "name": "role", "value": {"term": "reference", "target": role}},
        {"term": "binding", "name": "binder", "value": {"term": "reference", "target": binder}},
        {"term": "binding", "name": "constraint", "value": {"term": "literal",
            "type": node_id(&family_key("aaaa")), "value_kind": "boolean", "value": true}}
    ]);
    let commit = protocol_named_node(
        "control",
        &["Example", "OrderFlow", "run", "Main", "Commit"],
        body,
        &[],
    );
    let commit_target = commit["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(commit);
    let clause = clause_position(&package);
    let root_target =
        &package["semantic_graph"]["nodes"][clause]["body"]["arguments"][11]["value"]["target"];
    let root = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| &node["node_id"] == root_target)
        .expect("root control");
    package["semantic_graph"]["nodes"][root]["body"]["arguments"][0]["value"]["members"]
        .as_array_mut()
        .expect("sequence children")
        .push(json!({"term": "reference", "target": commit_target}));
    package["semantic_graph"]["nodes"][compensation]["body"]["arguments"][11]["value"]["members"] =
        json!([{"term": "reference", "target": commit_target}]);
    settle(&mut package);
    (package, compensation, root)
}

/// Trace: FR-038-AC-289
#[trace("TC-048")]
#[test]
fn compensation_admits_present_commit_and_refuses_wrong_record_type() {
    let (package, compensation, _) = package_with_present_commit();
    let role =
        package["semantic_graph"]["nodes"][protocol_position(&package, "role")]["node_id"].clone();
    assert!(
        matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
        "{:?}",
        read(&package)
    );

    let mut wrong = package;
    wrong["semantic_graph"]["nodes"][compensation]["body"]["arguments"][9]["value"]["members"][1]
        ["value"]["target"] = role;
    settle(&mut wrong);
    let key = wrong["semantic_graph"]["nodes"][compensation]["node_id"]["digest"]
        .as_str()
        .expect("compensation key");
    assert_eq!(
        read(&wrong),
        refusal_at(
            CheckedPackageRefusalCode::InvalidModelBinding,
            &format!("/semantic_graph/nodes/{compensation}/body/arguments/9/value/members/1/value/target"),
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            key,
        )
    );
}

/// Trace: FR-038-AC-289
#[trace("TC-048")]
#[test]
fn compensation_refuses_a_commit_taken_before_its_forward_effect() {
    let (mut package, compensation, root) = package_with_present_commit();
    let children = package["semantic_graph"]["nodes"][root]["body"]["arguments"][0]["value"]
        ["members"]
        .as_array_mut()
        .expect("sequence children");
    let commit_reference = children.pop().expect("commit child");
    children.insert(1, commit_reference);
    settle(&mut package);
    let commit_target = &package["semantic_graph"]["nodes"][compensation]["body"]["arguments"][11]
        ["value"]["members"][0]["target"];
    let effect_target = &package["semantic_graph"]["nodes"][compensation]["body"]["arguments"][0]
        ["value"]["target"];
    let commit = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| &node["node_id"] == commit_target)
        .expect("commit node");
    let key = commit_target["digest"].as_str().expect("commit key");
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("a preceding commit must refuse");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::UnsupportedConstruct
    );
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::DeclarationForm)
    );
    assert_eq!(
        refusal.path.as_ref().map(ToString::to_string),
        Some(format!("/semantic_graph/nodes/{commit}"))
    );
    assert_eq!(
        refusal.locus.as_ref().map(|node| node.digest.as_ref()),
        Some(key)
    );
    let Some(CheckedProtocolRefusalDetail::CommitPosition {
        profile,
        member,
        commit,
        effect,
        reason,
    }) = refusal.protocol_detail.as_deref()
    else {
        panic!("commit position detail must be retained");
    };
    assert_eq!(profile.identity.as_ref(), "quire.protocol.finite-global/v1");
    assert_eq!(member.as_ref(), "commit");
    assert_eq!(commit.digest.as_ref(), key);
    assert_eq!(
        effect.digest.as_ref(),
        effect_target["digest"].as_str().expect("effect key")
    );
    assert_eq!(reason.as_ref(), "commit-precedes-effect");
}

fn compensation_with_branch_before_effect(operator: &str, policy: &str) -> (Value, usize) {
    let (mut package, compensation, root) = package_with_present_commit();
    let children = package["semantic_graph"]["nodes"][root]["body"]["arguments"][0]["value"]
        ["members"]
        .as_array()
        .expect("sequence children");
    let attempt = children[0].clone();
    let effect = children[1].clone();
    let event = children[2].clone();
    let commit = children[3].clone();
    let role =
        package["semantic_graph"]["nodes"][protocol_position(&package, "role")]["node_id"].clone();
    let text = |value: &str| {
        json!({"term": "literal", "type": node_id(&family_key("a1a1")),
            "value_kind": "text", "value": value})
    };
    let binding =
        |name: &str, value: Value| json!({"term": "binding", "name": name, "value": value});
    let left = json!({"term": "aggregate", "members": [
        binding("label", text("Left")),
        binding("guard", json!({"term": "literal", "type": node_id(&family_key("aaaa")),
            "value_kind": "boolean", "value": true})),
        binding("body", commit.clone())
    ]});
    let right = json!({"term": "aggregate", "members": [
        binding("label", text("Right")),
        binding("guard", json!({"term": "literal", "type": node_id(&family_key("aaaa")),
            "value_kind": "boolean", "value": true})),
        binding("body", event.clone())
    ]});
    let mut body = package["semantic_graph"]["nodes"][root]["body"].clone();
    body["operation"]["member"]["operator"] = json!(operator);
    body["arguments"] = if operator == "choice" {
        json!([
            binding("owner", json!({"term": "reference", "target": role})),
            binding("visible", json!({"term": "aggregate", "members": []})),
            binding(
                "cases",
                json!({"term": "aggregate", "members": [left, right]})
            )
        ])
    } else {
        let mut left = left;
        let mut right = right;
        left["members"].as_array_mut().expect("fields").remove(1);
        right["members"].as_array_mut().expect("fields").remove(1);
        json!([
            binding("memory", json!({"term": "aggregate", "members": []})),
            binding(
                "branches",
                json!({"term": "aggregate", "members": [left, right]})
            ),
            binding(
                "join",
                json!({"term": "aggregate", "members": [binding("policy", text(policy))]})
            ),
            binding(
                "joined",
                json!({"term": "aggregate", "members": [text("Left"), text("Right")]})
            ),
            binding("outstanding", json!({"term": "aggregate", "members": []}))
        ])
    };
    let holder = protocol_named_node(
        "control",
        &["Example", "OrderFlow", "run", "Main", "Holder"],
        body,
        &[],
    );
    let holder_target = holder["node_id"].clone();
    let nodes = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    for (target, name) in [(&commit, "Commit"), (&event, "Event")] {
        let child = nodes
            .iter_mut()
            .find(|node| node["node_id"] == target["target"])
            .expect("branch child");
        child["declaration"]["qualified_name"] =
            json!(["Example", "OrderFlow", "run", "Main", "Holder", name]);
    }
    nodes.push(holder);
    package["semantic_graph"]["nodes"][root]["body"]["arguments"][0]["value"]["members"] =
        json!([attempt, {"term": "reference", "target": holder_target}, effect]);
    settle(&mut package);
    (package, compensation)
}

/// Trace: FR-038-AC-289
#[trace("TC-048")]
#[test]
fn compensation_commit_in_one_choice_or_non_all_branch_does_not_dominate() {
    for (operator, policy) in [("choice", ""), ("parallel", "any")] {
        let (package, _) = compensation_with_branch_before_effect(operator, policy);
        assert!(
            matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
            "{operator}: {:?}",
            read(&package)
        );
    }
    let (package, _) = compensation_with_branch_before_effect("parallel", "all");
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("parallel all commits must dominate");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::UnsupportedConstruct
    );
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::DeclarationForm)
    );
    assert!(matches!(refusal.protocol_detail.as_deref(),
        Some(CheckedProtocolRefusalDetail::CommitPosition { reason, .. })
            if reason.as_ref() == "commit-precedes-effect"));
}

/// Trace: FR-038-AC-284, FR-038-AC-289
#[trace("TC-048")]
#[test]
fn compensation_commit_in_every_branch_refuses_before_repeated_membership() {
    for (operator, policy) in [("choice", ""), ("parallel", "any")] {
        let (mut package, _) = compensation_with_branch_before_effect(operator, policy);
        let holder = holder_position(&package);
        let branches = if operator == "choice" { 2 } else { 1 };
        let body = if operator == "choice" { 2 } else { 1 };
        let commit = package["semantic_graph"]["nodes"][holder]["body"]["arguments"][branches]
            ["value"]["members"][0]["members"][body]["value"]
            .clone();
        package["semantic_graph"]["nodes"][holder]["body"]["arguments"][branches]["value"]
            ["members"][1]["members"][body]["value"] = commit;
        settle(&mut package);
        let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
            panic!("{operator} with commit in every branch must refuse");
        };
        assert_eq!(
            refusal.code,
            CheckedPackageRefusalCode::UnsupportedConstruct,
            "{operator}"
        );
        assert_eq!(
            refusal.cause,
            Some(CheckedPackageRefusalCause::DeclarationForm)
        );
    }
}

/// Trace: FR-038-AC-289
#[trace("TC-048")]
#[test]
fn compensation_commit_in_repeat_body_does_not_dominate() {
    let (mut package, _) = compensation_with_branch_before_effect("parallel", "all");
    let holder = holder_position(&package);
    let role =
        package["semantic_graph"]["nodes"][protocol_position(&package, "role")]["node_id"].clone();
    let branches =
        &package["semantic_graph"]["nodes"][holder]["body"]["arguments"][1]["value"]["members"];
    let commit = branches[0]["members"][1]["value"].clone();
    let event = branches[1]["members"][1]["value"].clone();
    let body = &mut package["semantic_graph"]["nodes"][holder]["body"];
    body["operation"]["member"]["operator"] = json!("repeat");
    body["arguments"] = json!([
        {"term": "binding", "name": "owner", "value": {"term": "reference", "target": role}},
        {"term": "binding", "name": "visible", "value": {"term": "aggregate", "members": []}},
        {"term": "binding", "name": "maximum", "value": {"term": "aggregate", "members": [
            {"term": "literal", "type": node_id(&family_key("a3a3")), "value_kind": "integer", "value": "1"}
        ]}},
        {"term": "binding", "name": "proof", "value": {"term": "aggregate", "members": []}},
        {"term": "binding", "name": "guard", "value": {"term": "literal", "type": node_id(&family_key("aaaa")),
            "value_kind": "boolean", "value": true}},
        {"term": "binding", "name": "body", "value": commit},
        {"term": "binding", "name": "exhausted", "value": {"term": "aggregate", "members": [event]}}
    ]);
    settle(&mut package);
    assert!(
        matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
        "{:?}",
        read(&package)
    );
}

/// Trace: FR-038-AC-284, FR-038-AC-289
#[trace("TC-048")]
#[test]
fn compensation_dominance_precedes_unattached_control_membership() {
    let (mut package, _, root) = package_with_present_commit();
    let children = package["semantic_graph"]["nodes"][root]["body"]["arguments"][0]["value"]
        ["members"]
        .as_array_mut()
        .expect("sequence children");
    let commit = children.pop().expect("commit child");
    children.insert(1, commit.clone());
    let nodes = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    let mut orphan_body = nodes
        .iter()
        .find(|node| node["node_id"] == commit["target"])
        .expect("commit node")["body"]
        .clone();
    orphan_body["arguments"][2]["value"]["value"] = json!(false);
    nodes.push(protocol_named_node(
        "control",
        &["Example", "OrderFlow", "run", "Main", "Orphan"],
        orphan_body,
        &[],
    ));
    settle(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("dominance must precede unattached membership");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::UnsupportedConstruct
    );
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::DeclarationForm)
    );
}

/// Trace: FR-038-AC-284, FR-038-AC-289
#[trace("TC-048")]
#[test]
fn compensation_first_template_dominance_precedes_later_template_bounds() {
    let (mut package, compensation, root) = package_with_present_commit();
    let children = package["semantic_graph"]["nodes"][root]["body"]["arguments"][0]["value"]
        ["members"]
        .as_array_mut()
        .expect("sequence children");
    let commit = children.pop().expect("commit child");
    children.insert(1, commit);
    settle(&mut package);
    let first = package["semantic_graph"]["nodes"][compensation]["node_id"].clone();
    let first_digest = first["digest"].as_str().expect("first digest");
    let mut later = None;
    for ordinal in 0..32 {
        let mut body = package["semantic_graph"]["nodes"][compensation]["body"].clone();
        body["arguments"][9]["value"]["members"][0]["value"]["value"] = json!("0");
        body["arguments"][4]["value"]["value"] = json!(format!("later-{ordinal}"));
        let candidate = protocol_named_node(
            "compensation",
            &["Example", "OrderFlow", "compensate", "Later"],
            body,
            &[],
        );
        if candidate["node_id"]["digest"]
            .as_str()
            .is_some_and(|digest| digest > first_digest)
        {
            later = Some(candidate);
            break;
        }
    }
    let later = later.expect("a later digest candidate");
    let later_target = later["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(later);
    let clause = clause_position(&package);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][8]["value"]["members"] = json!([
        {"term": "reference", "target": first},
        {"term": "reference", "target": later_target}
    ]);
    settle(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("first template dominance must precede later bounds");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::UnsupportedConstruct
    );
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::DeclarationForm)
    );
}

/// Trace: FR-038-AC-289
#[trace("TC-048")]
#[test]
fn compensation_commit_in_a_sibling_branch_is_unordered_against_effect() {
    let (mut package, _) = compensation_with_branch_before_effect("parallel", "all");
    let clause = clause_position(&package);
    let root_target = package["semantic_graph"]["nodes"][clause]["body"]["arguments"][11]["value"]
        ["target"]
        .clone();
    let nodes = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    let root = nodes
        .iter()
        .position(|node| node["node_id"] == root_target)
        .expect("root");
    let children = nodes[root]["body"]["arguments"][0]["value"]["members"]
        .as_array()
        .expect("sequence children");
    let effect = children[2].clone();
    let holder_target = children[1]["target"].clone();
    let holder = nodes
        .iter()
        .position(|node| node["node_id"] == holder_target)
        .expect("holder");
    let event =
        nodes[holder]["body"]["arguments"][1]["value"]["members"][1]["members"][1]["value"].clone();
    nodes[holder]["body"]["arguments"][1]["value"]["members"][1]["members"][1]["value"] =
        effect.clone();
    nodes[root]["body"]["arguments"][0]["value"]["members"][2] = event.clone();
    for (target, path) in [
        (
            &effect,
            json!([
                "Example",
                "OrderFlow",
                "run",
                "Main",
                "Holder",
                "ForwardEffect"
            ]),
        ),
        (
            &event,
            json!(["Example", "OrderFlow", "run", "Main", "Event"]),
        ),
    ] {
        let child = nodes
            .iter_mut()
            .find(|node| node["node_id"] == target["target"])
            .expect("moved control");
        child["declaration"]["qualified_name"] = path;
    }
    settle(&mut package);
    assert!(
        matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
        "{:?}",
        read(&package)
    );
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn attempt_ordering_rejects_an_unknown_spelling_at_wire_validation() {
    let (mut package, root) = package_with_step_control("attempt");
    package["semantic_graph"]["nodes"][root]["body"]["arguments"][2]["value"]["members"] = json!([{"term": "literal", "type": node_id(&family_key("a1a1")),
            "value_kind": "text", "value": "unordered"}]);
    assert_eq!(
        read(&package),
        refusal(
            CheckedPackageRefusalCode::MalformedWire,
            &format!("/semantic_graph/nodes/{root}/body/arguments/2/value/members/0")
        )
    );
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn check_condition_refuses_non_boolean_at_its_slot_and_fence_rejects_relaxed() {
    let mut package = v2_all_families();
    let root = protocol_position(&package, "control");
    let body = &mut package["semantic_graph"]["nodes"][root]["body"];
    body["operation"]["member"]["operator"] = json!("check");
    body["arguments"] = json!([{"term": "binding", "name": "condition", "value": {
        "term": "literal", "type": node_id(&family_key("aaaa")), "value_kind": "boolean", "value": true
    }}]);
    settle(&mut package);
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));

    package["semantic_graph"]["nodes"][root]["body"]["arguments"][0]["value"] = json!({
        "term": "literal", "type": node_id(&family_key("a1a1")), "value_kind": "text", "value": "yes"
    });
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][root]["node_id"]["digest"]
        .as_str()
        .expect("check key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::IllTyped,
            &format!("/semantic_graph/nodes/{root}/body/arguments/0/value"),
            Some(CheckedPackageRefusalCause::NonBooleanRoot),
            key,
        )
    );

    let body = &mut package["semantic_graph"]["nodes"][root]["body"];
    body["operation"]["member"]["operator"] = json!("fence");
    body["arguments"] = json!([{"term": "binding", "name": "ordering", "value": {
        "term": "literal", "type": node_id(&family_key("a1a1")), "value_kind": "text", "value": "seq_cst"
    }}]);
    settle(&mut package);
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    package["semantic_graph"]["nodes"][root]["body"]["arguments"][0]["value"]["value"] =
        json!("relaxed");
    assert_eq!(
        read(&package),
        refusal(
            CheckedPackageRefusalCode::MalformedWire,
            &format!("/semantic_graph/nodes/{root}/body/arguments/0/value"),
        )
    );
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn send_admits_channel_payload_and_checks_binder_type_and_condition() {
    let (package, root, channel) = package_with_send();
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));

    let mut wrong_channel = package.clone();
    wrong_channel["semantic_graph"]["nodes"][root]["body"]["arguments"][0]["value"]["target"] =
        wrong_channel["semantic_graph"]["nodes"][protocol_position(&wrong_channel, "role")]
            ["node_id"]
            .clone();
    settle(&mut wrong_channel);
    let key = wrong_channel["semantic_graph"]["nodes"][root]["node_id"]["digest"]
        .as_str()
        .expect("send key");
    assert_eq!(
        read(&wrong_channel),
        refusal_at(
            CheckedPackageRefusalCode::InvalidModelBinding,
            &format!("/semantic_graph/nodes/{root}/body/arguments/0/value/target"),
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            key,
        )
    );

    let mut wrong_type = package.clone();
    wrong_type["semantic_graph"]["nodes"][channel]["body"]["arguments"][2]["value"]["target"] =
        node_id(&family_key("a1a1"));
    settle(&mut wrong_type);
    let key = wrong_type["semantic_graph"]["nodes"][root]["node_id"]["digest"]
        .as_str()
        .expect("send key");
    assert_eq!(
        read(&wrong_type),
        refusal_at(
            CheckedPackageRefusalCode::IllTyped,
            &format!("/semantic_graph/nodes/{root}"),
            Some(CheckedPackageRefusalCause::TypeMismatch),
            key,
        )
    );
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&wrong_type) else {
        panic!("wrong payload type must refuse");
    };
    assert_eq!(
        refusal.protocol_detail.as_deref(),
        Some(&CheckedProtocolRefusalDetail::PayloadType {
            expected: serde_json::from_value(node_id(&family_key("a1a1"))).expect("text ID"),
            actual: serde_json::from_value(node_id(&family_key("aaaa"))).expect("boolean ID"),
        })
    );

    let mut non_boolean = package;
    non_boolean["semantic_graph"]["nodes"][root]["body"]["arguments"][3]["value"] = json!({
        "term": "literal", "type": node_id(&family_key("a1a1")), "value_kind": "text", "value": "yes"
    });
    settle(&mut non_boolean);
    let key = non_boolean["semantic_graph"]["nodes"][root]["node_id"]["digest"]
        .as_str()
        .expect("send key");
    assert_eq!(
        read(&non_boolean),
        refusal_at(
            CheckedPackageRefusalCode::IllTyped,
            &format!("/semantic_graph/nodes/{root}/body/arguments/3/value"),
            Some(CheckedPackageRefusalCause::NonBooleanRoot),
            key,
        )
    );
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn receive_requires_a_send_target_before_payload_type_checks() {
    let (mut package, root, _) = package_with_send();
    let role =
        package["semantic_graph"]["nodes"][protocol_position(&package, "role")]["node_id"].clone();
    let body = &mut package["semantic_graph"]["nodes"][root]["body"];
    body["operation"]["member"]["operator"] = json!("receive");
    let arguments = body["arguments"].as_array_mut().expect("send arguments");
    arguments.insert(
        1,
        json!({
            "term": "binding", "name": "of", "value": {"term": "reference", "target": role}
        }),
    );
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][root]["node_id"]["digest"]
        .as_str()
        .expect("receive key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidModelBinding,
            &format!("/semantic_graph/nodes/{root}/body/arguments/1/value/target"),
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            key,
        )
    );
}

/// Trace: FR-038-AC-288, FR-038-AC-290
#[trace("TC-048")]
#[test]
fn receive_accepts_its_send_channel_and_refuses_another_channel() {
    let (mut package, root, channel) = package_with_send();
    let send_body = package["semantic_graph"]["nodes"][root]["body"].clone();
    let send = protocol_named_node(
        "control",
        &["Example", "OrderFlow", "run", "Main", "Send"],
        send_body.clone(),
        &[],
    );
    let send_key = send["node_id"].clone();
    let send_position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(send);

    let mut receive_body = send_body;
    receive_body["operation"]["member"]["operator"] = json!("receive");
    receive_body["arguments"]
        .as_array_mut()
        .expect("arguments")
        .insert(
            1,
            json!({
                "term": "binding", "name": "of", "value": {"term": "reference", "target": send_key}
            }),
        );
    let receive = protocol_named_node(
        "control",
        &["Example", "OrderFlow", "run", "Main", "Receive"],
        receive_body,
        &[],
    );
    let receive_position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    let receive_key = receive["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(receive);

    let root_body = &mut package["semantic_graph"]["nodes"][root]["body"];
    root_body["operation"]["member"]["operator"] = json!("sequence");
    root_body["arguments"] = json!([{"term": "binding", "name": "children", "value": {
        "term": "aggregate", "members": [
            {"term": "reference", "target": send_key},
            {"term": "reference", "target": receive_key}
        ]
    }}]);
    settle(&mut package);
    assert!(
        matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
        "send and receive on one channel must admit locally: {:?}",
        read(&package)
    );

    let mut reversed = package.clone();
    reversed["semantic_graph"]["nodes"][root]["body"]["arguments"][0]["value"]["members"]
        .as_array_mut()
        .expect("sequence children")
        .swap(0, 1);
    settle(&mut reversed);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&reversed) else {
        panic!("receive sequenced before its send must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::DefinitionCycle)
    );
    let Some(CheckedProtocolRefusalDetail::CausalCycle { controls }) =
        refusal.protocol_detail.as_deref()
    else {
        panic!("cycle must name its controls");
    };
    let expected: Vec<quire_contract_model::CheckedNodeId> = [receive_position, send_position]
        .into_iter()
        .map(|position| {
            serde_json::from_value(reversed["semantic_graph"]["nodes"][position]["node_id"].clone())
                .expect("control identity")
        })
        .collect();
    assert_eq!(controls, &expected);

    let mut parallel = package.clone();
    parallel["semantic_graph"]["nodes"][root]["body"]["operation"]["member"]["operator"] =
        json!("parallel");
    parallel["semantic_graph"]["nodes"][root]["body"]["arguments"] = json!([
        {"term": "binding", "name": "memory", "value": {"term": "aggregate", "members": []}},
        {"term": "binding", "name": "branches", "value": {"term": "aggregate", "members": [
            {"term": "aggregate", "members": [
                {"term": "binding", "name": "label", "value": {"term": "literal", "type": node_id(&family_key("a1a1")), "value_kind": "text", "value": "Left"}},
                {"term": "binding", "name": "body", "value": {"term": "reference", "target": receive_key}}
            ]},
            {"term": "aggregate", "members": [
                {"term": "binding", "name": "label", "value": {"term": "literal", "type": node_id(&family_key("a1a1")), "value_kind": "text", "value": "Right"}},
                {"term": "binding", "name": "body", "value": {"term": "reference", "target": send_key}}
            ]}
        ]}},
        {"term": "binding", "name": "join", "value": {"term": "aggregate", "members": [
            {"term": "binding", "name": "policy", "value": {"term": "literal", "type": node_id(&family_key("a1a1")), "value_kind": "text", "value": "all"}}
        ]}},
        {"term": "binding", "name": "joined", "value": {"term": "aggregate", "members": [
            {"term": "literal", "type": node_id(&family_key("a1a1")), "value_kind": "text", "value": "Left"},
            {"term": "literal", "type": node_id(&family_key("a1a1")), "value_kind": "text", "value": "Right"}
        ]}},
        {"term": "binding", "name": "outstanding", "value": {"term": "aggregate", "members": []}}
    ]);
    settle(&mut parallel);
    assert!(
        matches!(read(&parallel), CheckedPackageV2ReadResult::Admitted(_)),
        "independent parallel send and receive must admit: {:?}",
        read(&parallel)
    );

    let mut wrong_binder = package.clone();
    let mut text_binder = wrong_binder["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "value" && node["semantic_form"] == "parameter")
        .expect("parameter")
        .clone();
    text_binder["semantic_type"] = node_id(&family_key("a1a1"));
    text_binder["body"]["members"][0]["value"]["value"] = json!("wrong_receive");
    let text_key = structural_key(
        "value",
        "parameter",
        Some(&family_key("a1a1")),
        &text_binder["body"],
    );
    text_binder["node_id"] = node_id(&text_key);
    wrong_binder["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(text_binder);
    wrong_binder["semantic_graph"]["nodes"][receive_position]["body"]["arguments"][2]["value"]
        ["target"] = node_id(&text_key);
    settle(&mut wrong_binder);
    let wrong_key = wrong_binder["semantic_graph"]["nodes"][receive_position]["node_id"]["digest"]
        .as_str()
        .expect("receive key");
    assert_eq!(
        read(&wrong_binder),
        refusal_at(
            CheckedPackageRefusalCode::IllTyped,
            &format!("/semantic_graph/nodes/{receive_position}"),
            Some(CheckedPackageRefusalCause::TypeMismatch),
            wrong_key,
        )
    );

    let other = protocol_named_node(
        "channel",
        &["Example", "OrderFlow", "channel", "Other"],
        package["semantic_graph"]["nodes"][channel]["body"].clone(),
        &[],
    );
    let other_key = other["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(other);
    let clause = clause_position(&package);
    let channels = package["semantic_graph"]["nodes"][clause]["body"]["arguments"][5]["value"]
        ["members"]
        .as_array_mut()
        .expect("channels");
    channels.push(json!({"term": "reference", "target": other_key}));
    channels.sort_by(|left, right| {
        left["target"]["digest"]
            .as_str()
            .cmp(&right["target"]["digest"].as_str())
    });
    package["semantic_graph"]["nodes"][receive_position]["body"]["arguments"][0]["value"]
        ["target"] = other_key;
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][receive_position]["node_id"]["digest"]
        .as_str()
        .expect("receive key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::IllTyped,
            &format!("/semantic_graph/nodes/{receive_position}"),
            Some(CheckedPackageRefusalCause::OperatorIneligible),
            key,
        )
    );
    assert_ne!(send_position, receive_position);
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn send_related_records_require_exact_shape_and_canonical_order() {
    let (mut package, root, _) = package_with_send();
    let role =
        package["semantic_graph"]["nodes"][protocol_position(&package, "role")]["node_id"].clone();
    let record = json!({"term": "aggregate", "members": [
        {"term": "binding", "name": "relationship", "value": {"term": "reference", "target": role}},
        {"term": "binding", "name": "from", "value": {"term": "reference", "target": role}},
        {"term": "binding", "name": "to", "value": {"term": "reference", "target": role}}
    ]});
    package["semantic_graph"]["nodes"][root]["body"]["arguments"][2]["value"]["members"] =
        json!([record.clone(), record]);
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][root]["node_id"]["digest"]
        .as_str()
        .expect("send key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            &format!("/semantic_graph/nodes/{root}"),
            None,
            key,
        )
    );

    package["semantic_graph"]["nodes"][root]["body"]["arguments"][2]["value"]["members"]
        .as_array_mut()
        .expect("related")
        .pop();
    package["semantic_graph"]["nodes"][root]["body"]["arguments"][2]["value"]["members"][0]
        ["members"][1]["name"] = json!("wrong");
    assert_eq!(
        read(&package),
        refusal(
            CheckedPackageRefusalCode::MalformedWire,
            &format!("/semantic_graph/nodes/{root}/body"),
        )
    );
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn send_related_requires_listed_relationship_and_typed_ends() {
    let (mut package, root, _) = package_with_send();
    let relationship = node_id(&family_key("2020"));
    let role =
        package["semantic_graph"]["nodes"][protocol_position(&package, "role")]["node_id"].clone();
    let record = |target: Value| {
        json!({"term": "aggregate", "members": [
            {"term": "binding", "name": "relationship", "value": {"term": "reference", "target": target}},
            {"term": "binding", "name": "from", "value": {"term": "literal",
                "type": node_id(&family_key("aaaa")), "value_kind": "boolean", "value": true}},
            {"term": "binding", "name": "to", "value": {"term": "literal",
                "type": node_id(&family_key("aaaa")), "value_kind": "boolean", "value": true}}
        ]})
    };
    package["semantic_graph"]["nodes"][root]["body"]["arguments"][2]["value"]["members"] =
        json!([record(role)]);
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][root]["node_id"]["digest"]
        .as_str()
        .expect("send key");
    assert_eq!(read(&package), refusal_at(
        CheckedPackageRefusalCode::InvalidModelBinding,
        &format!("/semantic_graph/nodes/{root}/body/arguments/2/value/members/0/members/0/value/target"),
        Some(CheckedPackageRefusalCause::WrongExport), key,
    ));

    package["semantic_graph"]["nodes"][root]["body"]["arguments"][2]["value"]["members"] =
        json!([record(relationship.clone())]);
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][root]["node_id"]["digest"]
        .as_str()
        .expect("send key");
    assert_eq!(read(&package), refusal_at(
        CheckedPackageRefusalCode::MissingDeclaration,
        &format!("/semantic_graph/nodes/{root}/body/arguments/2/value/members/0/members/0/value/target"),
        Some(CheckedPackageRefusalCause::MissingName), key,
    ));

    let clause = clause_position(&package);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][4]["value"]["members"] =
        json!([{"term": "reference", "target": relationship.clone()}]);
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][root]["node_id"]["digest"]
        .as_str()
        .expect("send key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::IllTyped,
            &format!(
                "/semantic_graph/nodes/{root}/body/arguments/2/value/members/0/members/1/value"
            ),
            Some(CheckedPackageRefusalCause::TypeMismatch),
            key,
        )
    );
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("wrong related end must refuse");
    };
    assert_eq!(
        refusal.protocol_detail.as_deref(),
        Some(&CheckedProtocolRefusalDetail::RelatedEndType {
            relationship: serde_json::from_value(relationship).expect("relationship ID"),
            end: "from".into(),
            expected: serde_json::from_value(node_id(&family_key("1515")))
                .expect("source endpoint ID"),
            actual: Some(
                serde_json::from_value(node_id(&family_key("aaaa"))).expect("Boolean type ID")
            ),
        })
    );
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn send_related_accepts_selected_relationship_reference_ends() {
    let (mut package, root, channel) = package_with_send();
    let object = node_id(&family_key("1515"));
    let mut reference_type = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "composite_type" && node["semantic_form"] == "option")
        .expect("option type")
        .clone();
    reference_type["semantic_form"] = json!("reference");
    reference_type["body"] = json!({"term": "aggregate", "members": [
        {"term": "reference", "target": object}
    ]});
    let reference_key =
        structural_key("composite_type", "reference", None, &reference_type["body"]);
    reference_type["node_id"] = node_id(&reference_key);
    reference_type["semantic_type"] = node_id(&reference_key);
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(reference_type);

    let mut binder = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "value" && node["semantic_form"] == "parameter")
        .expect("parameter")
        .clone();
    binder["semantic_type"] = node_id(&reference_key);
    binder["body"]["members"][0]["value"]["value"] = json!("payload");
    let binder_key = structural_key("value", "parameter", Some(&reference_key), &binder["body"]);
    binder["node_id"] = node_id(&binder_key);
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(binder.clone());

    binder["body"]["members"][0]["value"]["value"] = json!("peer");
    let peer_key = structural_key("value", "parameter", Some(&reference_key), &binder["body"]);
    binder["node_id"] = node_id(&peer_key);
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(binder);

    package["semantic_graph"]["nodes"][channel]["body"]["arguments"][2]["value"]["target"] =
        node_id(&reference_key);
    package["semantic_graph"]["nodes"][root]["body"]["arguments"][1]["value"]["target"] =
        node_id(&binder_key);
    let relationship = node_id(&family_key("2020"));
    let related_end = json!({"term": "reference", "target": node_id(&binder_key)});
    package["semantic_graph"]["nodes"][root]["body"]["arguments"][2]["value"]["members"] = json!([{"term": "aggregate", "members": [
        {"term": "binding", "name": "relationship", "value": {"term": "reference", "target": relationship}},
        {"term": "binding", "name": "from", "value": related_end},
        {"term": "binding", "name": "to", "value": related_end}
    ]}]);
    let clause = clause_position(&package);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][4]["value"]["members"] =
        json!([{"term": "reference", "target": relationship}]);
    settle(&mut package);
    assert!(
        matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
        "selected relationship ends of the binder's reference type must admit: {:?}",
        read(&package)
    );

    let record = |to: &str| {
        json!({"term": "aggregate", "members": [
            {"term": "binding", "name": "relationship", "value": {"term": "reference", "target": relationship}},
            {"term": "binding", "name": "from", "value": {"term": "reference", "target": node_id(&binder_key)}},
            {"term": "binding", "name": "to", "value": {"term": "reference", "target": node_id(to)}}
        ]})
    };
    let (first, second) = if binder_key < peer_key {
        (record(&binder_key), record(&peer_key))
    } else {
        (record(&peer_key), record(&binder_key))
    };
    let related =
        &mut package["semantic_graph"]["nodes"][root]["body"]["arguments"][2]["value"]["members"];
    *related = json!([first.clone(), second.clone()]);
    settle(&mut package);
    assert!(
        matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
        "distinct related records in JCS order must admit: {:?}",
        read(&package)
    );

    package["semantic_graph"]["nodes"][root]["body"]["arguments"][2]["value"]["members"] =
        json!([second, first.clone()]);
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][root]["node_id"]["digest"]
        .as_str()
        .expect("send key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            &format!("/semantic_graph/nodes/{root}"),
            None,
            key,
        )
    );

    let mut bad_to = first;
    bad_to["members"][2]["value"] = json!({
        "term": "literal", "type": node_id(&family_key("aaaa")),
        "value_kind": "boolean", "value": true
    });
    package["semantic_graph"]["nodes"][root]["body"]["arguments"][2]["value"]["members"] =
        json!([bad_to]);
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][root]["node_id"]["digest"]
        .as_str()
        .expect("send key");
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("wrong to endpoint must refuse");
    };
    assert_eq!(
        refusal,
        refusal_at(
            CheckedPackageRefusalCode::IllTyped,
            &format!(
                "/semantic_graph/nodes/{root}/body/arguments/2/value/members/0/members/2/value"
            ),
            Some(CheckedPackageRefusalCause::TypeMismatch),
            key,
        )
    );
    assert_eq!(
        refusal.protocol_detail.as_deref(),
        Some(&CheckedProtocolRefusalDetail::RelatedEndType {
            relationship: serde_json::from_value(relationship).expect("relationship ID"),
            end: "to".into(),
            expected: serde_json::from_value(node_id(&family_key("1515")))
                .expect("target endpoint ID"),
            actual: Some(
                serde_json::from_value(node_id(&family_key("aaaa"))).expect("Boolean type ID")
            ),
        })
    );
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
    let (mut package, _) = package_with_member_queue();
    let channel = protocol_position(&package, "channel");
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
    let (mut package, _) = package_with_member_queue();
    let channel = protocol_position(&package, "channel");
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
    let (mut package, _) = package_with_member_queue();
    let channel = protocol_position(&package, "channel");
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

/// Trace: FR-038-AC-282, FR-038-AC-286
#[trace("TC-947")]
#[test]
fn protocol_whole_model_role_admits_selected_text_without_model_dependency() {
    let (mut package, role) = whole_model_role("test/families");
    assert_eq!(
        package["semantic_graph"]["nodes"][role]["dependencies"],
        json!([])
    );
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));

    let first_key = package["semantic_graph"]["nodes"][role]["node_id"].clone();
    let other = domain_package_document("test/other", "1", Vec::new());
    package["lock"]["model_selections"]
        .as_array_mut()
        .expect("model selections")
        .push(json!({
            "identity": "test/other", "digest_domain": "sha256-jcs",
            "digest": domain_package_digest(&other),
        }));
    package["semantic_graph"]["nodes"][role]["body"]["arguments"][0]["value"]["value"] =
        json!("test/other");
    settle(&mut package);
    assert_ne!(
        package["semantic_graph"]["nodes"][role]["node_id"],
        first_key
    );
    assert_eq!(
        package["semantic_graph"]["nodes"][role]["dependencies"],
        json!([])
    );
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

/// Trace: FR-038-AC-282, FR-038-AC-286
#[trace("TC-947")]
#[test]
fn protocol_clause_admits_two_whole_model_roles_across_over_types() {
    for over in [family_key("a3a3"), family_key("a1a1")] {
        let (mut package, first) = whole_model_role("test/families");
        let mut over_parameter = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .find(|node| {
                node["node_tag"] == "value"
                    && node["semantic_form"] == "parameter"
                    && node["body"]["members"][0]["value"]["value"] == "over"
            })
            .expect("over parameter")
            .clone();
        over_parameter["semantic_type"] = node_id(&over);
        let over_key = structural_key("value", "parameter", Some(&over), &over_parameter["body"]);
        over_parameter["node_id"] = node_id(&over_key);
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(over_parameter);
        let other = domain_package_document("test/other", "1", Vec::new());
        package["lock"]["model_selections"]
            .as_array_mut()
            .expect("model selections")
            .push(json!({
                "identity": "test/other", "digest_domain": "sha256-jcs",
                "digest": domain_package_digest(&other),
            }));

        let mut second_body = package["semantic_graph"]["nodes"][first]["body"].clone();
        second_body["arguments"][0]["value"]["value"] = json!("test/other");
        let second_node = protocol_named_node(
            "role",
            &["Example", "OrderFlow", "role", "Other"],
            second_body,
            &[],
        );
        let second = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .len();
        let first_key = package["semantic_graph"]["nodes"][first]["node_id"].clone();
        let second_key = second_node["node_id"].clone();
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(second_node);
        let mut role_keys = [first_key.clone(), second_key.clone()];
        role_keys.sort_by(|a, b| a["digest"].as_str().cmp(&b["digest"].as_str()));
        let clause = clause_position(&package);
        package["semantic_graph"]["nodes"][clause]["body"]["arguments"][0]["value"] =
            json!({"term": "reference", "target": node_id(&over_key)});
        package["semantic_graph"]["nodes"][clause]["body"]["arguments"][3]["value"]["members"] =
            json!(role_keys
                .iter()
                .map(|key| json!({"term": "reference", "target": key}))
                .collect::<Vec<_>>());
        settle(&mut package);

        assert_ne!(first_key, second_key);
        assert_ne!(first_key, protocol_role_node()["node_id"]);
        assert_eq!(
            package["semantic_graph"]["nodes"][first]["dependencies"],
            json!([])
        );
        assert_eq!(
            package["semantic_graph"]["nodes"][second]["dependencies"],
            json!([])
        );
        assert!(
            matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
            "typed over parameter should admit: {:?}",
            read(&package)
        );

        package["lock"]["model_selections"]
            .as_array_mut()
            .expect("model selections")
            .pop();
        refresh_identity(&mut package);
        let key = package["semantic_graph"]["nodes"][second]["node_id"]["digest"]
            .as_str()
            .expect("role key");
        assert_eq!(
            read(&package),
            refusal_at(
                CheckedPackageRefusalCode::MissingDeclaration,
                &format!("/semantic_graph/nodes/{second}"),
                Some(CheckedPackageRefusalCause::MissingSelection),
                key,
            )
        );
    }
}

/// Trace: FR-038-AC-282, FR-038-AC-286
#[trace("TC-947")]
#[test]
fn protocol_whole_model_role_checks_selection_before_replication() {
    let (mut package, role) = package_with_replicated_role();
    package["semantic_graph"]["nodes"][role]["body"]["arguments"][0]["value"] = json!({
        "term": "literal", "type": node_id(&family_key("a1a1")),
        "value_kind": "text", "value": "test/missing",
    });
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][role]["node_id"]["digest"]
        .as_str()
        .expect("role key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::MissingDeclaration,
            &format!("/semantic_graph/nodes/{role}"),
            Some(CheckedPackageRefusalCause::MissingSelection),
            key,
        )
    );

    package["semantic_graph"]["nodes"][role]["body"]["arguments"][0]["value"]["value"] =
        json!("test/families");
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][role]["node_id"]["digest"]
        .as_str()
        .expect("role key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidModelBinding,
            &format!("/semantic_graph/nodes/{role}"),
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            key,
        )
    );
}

/// Trace: FR-038-AC-282, FR-038-AC-286
#[trace("TC-947")]
#[test]
fn protocol_role_model_rejects_non_reference_non_text_wire() {
    let (mut package, role) = package_with_member_role();
    package["semantic_graph"]["nodes"][role]["body"]["arguments"][0]["value"] = json!({
        "term": "literal", "type": node_id(&family_key("a3a3")),
        "value_kind": "integer", "value": "1"
    });
    settle(&mut package);
    assert_eq!(
        read(&package),
        refusal(
            CheckedPackageRefusalCode::MalformedWire,
            &format!("/semantic_graph/nodes/{role}/body"),
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

/// Trace: FR-038-AC-285, FR-038-AC-286
#[trace("TC-947")]
#[test]
fn protocol_queue_rejects_foreign_clause_channel() {
    let (mut package, queue) = package_with_member_queue();
    let original_role = protocol_position(&package, "role");
    let original_channel = protocol_position(&package, "channel");
    let original_root = protocol_position(&package, "control");
    let original_finish = protocol_position(&package, "obligation");
    let original_clause = clause_position(&package);

    let foreign_role = protocol_named_node(
        "role",
        &["Example", "Elsewhere", "role", "Worker"],
        package["semantic_graph"]["nodes"][original_role]["body"].clone(),
        &[],
    );
    let foreign_role_key = foreign_role["node_id"].clone();
    let mut foreign_channel_body =
        package["semantic_graph"]["nodes"][original_channel]["body"].clone();
    for ordinal in [0, 1] {
        foreign_channel_body["arguments"][ordinal]["value"]["target"] = foreign_role_key.clone();
    }
    let foreign_channel = protocol_named_node(
        "channel",
        &["Example", "Elsewhere", "channel", "Link"],
        foreign_channel_body,
        &[],
    );
    let foreign_channel_key = foreign_channel["node_id"].clone();
    let foreign_root = protocol_named_node(
        "control",
        &["Example", "Elsewhere", "run", "Main"],
        package["semantic_graph"]["nodes"][original_root]["body"].clone(),
        &[],
    );
    let foreign_root_key = foreign_root["node_id"].clone();
    let foreign_finish = protocol_named_node(
        "obligation",
        &["Example", "Elsewhere", "finish", "Closed"],
        package["semantic_graph"]["nodes"][original_finish]["body"].clone(),
        &[],
    );
    let foreign_finish_key = foreign_finish["node_id"].clone();
    let mut foreign_clause_body =
        package["semantic_graph"]["nodes"][original_clause]["body"].clone();
    for (ordinal, target) in [(3, foreign_role_key), (5, foreign_channel_key.clone())] {
        foreign_clause_body["arguments"][ordinal]["value"]["members"] =
            json!([{"term": "reference", "target": target}]);
    }
    foreign_clause_body["arguments"][6]["value"]["members"] = json!([]);
    foreign_clause_body["arguments"][11]["value"]["target"] = foreign_root_key;
    foreign_clause_body["arguments"][12]["value"]["target"] = foreign_finish_key;
    let foreign_clause = protocol_named_node(
        "protocol_clause",
        &["Example", "Elsewhere"],
        foreign_clause_body,
        &[],
    );
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend([
            foreign_role,
            foreign_channel,
            foreign_root,
            foreign_finish,
            foreign_clause,
        ]);

    let mut second_queue = package["semantic_graph"]["nodes"][queue].clone();
    second_queue["body"]["arguments"][1]["value"]["members"][1]["value"]["value"] = json!("5");
    second_queue["node_id"] = node_id(&family_key("queue-original-placeholder"));
    let second_queue_position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(second_queue);
    rekey_application_node(&mut package, second_queue_position);
    let second_queue_key =
        package["semantic_graph"]["nodes"][second_queue_position]["node_id"].clone();
    let original_queue_key = package["semantic_graph"]["nodes"][queue]["node_id"].clone();
    package["semantic_graph"]["nodes"][original_clause]["body"]["arguments"][6]["value"]
        ["members"] = json!([
        {"term": "reference", "target": original_queue_key},
        {"term": "reference", "target": second_queue_key}
    ]);
    package["semantic_graph"]["nodes"][original_clause]["body"]["arguments"][6]["value"]["members"]
        .as_array_mut()
        .expect("queues")
        .sort_by(|left, right| {
            left["target"]["digest"]
                .as_str()
                .cmp(&right["target"]["digest"].as_str())
        });
    package["semantic_graph"]["nodes"][queue]["body"]["arguments"][0]["value"]["target"] =
        foreign_channel_key;
    settle(&mut package);
    package["semantic_graph"]["nodes"][original_clause]["body"]["arguments"][6]["value"]["members"]
        .as_array_mut()
        .expect("queues")
        .sort_by(|left, right| {
            left["target"]["digest"]
                .as_str()
                .cmp(&right["target"]["digest"].as_str())
        });
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][queue]["node_id"]["digest"]
        .as_str()
        .expect("queue key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            &format!("/semantic_graph/nodes/{queue}"),
            None,
            key,
        )
    );
}

/// Trace: FR-038-AC-285
#[trace("TC-048")]
#[test]
fn protocol_membership_refuses_unlisted_role_and_repeated_control_holder() {
    let (mut unlisted, role) = package_with_member_role();
    let clause = clause_position(&unlisted);
    unlisted["semantic_graph"]["nodes"][clause]["body"]["arguments"][3]["value"]["members"] =
        json!([]);
    settle(&mut unlisted);
    let key = unlisted["semantic_graph"]["nodes"][role]["node_id"]["digest"]
        .as_str()
        .expect("role key");
    assert_eq!(
        read(&unlisted),
        refusal_at(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            &format!("/semantic_graph/nodes/{role}"),
            None,
            key,
        )
    );

    let (mut repeated_listing, listed_role) = package_with_member_role();
    let clause = clause_position(&repeated_listing);
    let role_target = repeated_listing["semantic_graph"]["nodes"][listed_role]["node_id"].clone();
    repeated_listing["semantic_graph"]["nodes"][clause]["body"]["arguments"][3]["value"]
        ["members"] = json!([
        {"term": "reference", "target": role_target},
        {"term": "reference", "target": role_target}
    ]);
    settle(&mut repeated_listing);
    let key = repeated_listing["semantic_graph"]["nodes"][listed_role]["node_id"]["digest"]
        .as_str()
        .expect("role key");
    assert_eq!(
        read(&repeated_listing),
        refusal_at(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            &format!("/semantic_graph/nodes/{listed_role}"),
            None,
            key,
        )
    );

    let mut repeated = v2_all_families();
    let root = protocol_position(&repeated, "control");
    let child = protocol_named_node(
        "control",
        &["Example", "OrderFlow", "run", "Main", "Child"],
        repeated["semantic_graph"]["nodes"][root]["body"].clone(),
        &[],
    );
    let child_key = child["node_id"].clone();
    let child_position = repeated["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    repeated["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(child);
    let root_body = &mut repeated["semantic_graph"]["nodes"][root]["body"];
    root_body["operation"]["member"]["operator"] = json!("sequence");
    root_body["arguments"] = json!([{"term": "binding", "name": "children", "value": {
        "term": "aggregate", "members": [
            {"term": "reference", "target": child_key},
            {"term": "reference", "target": child_key}
        ]
    }}]);
    settle(&mut repeated);
    let key = repeated["semantic_graph"]["nodes"][child_position]["node_id"]["digest"]
        .as_str()
        .expect("child key");
    assert_eq!(
        read(&repeated),
        refusal_at(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            &format!("/semantic_graph/nodes/{child_position}"),
            None,
            key,
        )
    );
}

/// Trace: FR-038-AC-285
#[trace("TC-048")]
#[test]
fn protocol_membership_requires_exactly_one_queue_per_channel() {
    let (without_queue, _, channel) = package_with_member_channel();
    let key = without_queue["semantic_graph"]["nodes"][channel]["node_id"]["digest"]
        .as_str()
        .expect("channel key");
    assert_eq!(
        read(&without_queue),
        refusal_at(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            &format!("/semantic_graph/nodes/{channel}"),
            None,
            key,
        )
    );

    let (mut two_queues, first) = package_with_member_queue();
    let mut second = two_queues["semantic_graph"]["nodes"][first].clone();
    second["body"]["arguments"][1]["value"]["members"][1]["value"]["value"] = json!("5");
    second["node_id"] = node_id(&family_key("queue-two-placeholder"));
    let second_position = two_queues["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    two_queues["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(second);
    rekey_application_node(&mut two_queues, second_position);
    let second_key = two_queues["semantic_graph"]["nodes"][second_position]["node_id"].clone();
    let clause = clause_position(&two_queues);
    two_queues["semantic_graph"]["nodes"][clause]["body"]["arguments"][6]["value"]["members"]
        .as_array_mut()
        .expect("queue list")
        .push(json!({"term": "reference", "target": second_key}));
    two_queues["semantic_graph"]["nodes"][clause]["body"]["arguments"][6]["value"]["members"]
        .as_array_mut()
        .expect("queue list")
        .sort_by(|left, right| {
            left["target"]["digest"]
                .as_str()
                .cmp(&right["target"]["digest"].as_str())
        });
    settle(&mut two_queues);
    let channel = protocol_position(&two_queues, "channel");
    let key = two_queues["semantic_graph"]["nodes"][channel]["node_id"]["digest"]
        .as_str()
        .expect("channel key");
    assert_eq!(
        read(&two_queues),
        refusal_at(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            &format!("/semantic_graph/nodes/{channel}"),
            None,
            key,
        )
    );
}

/// Trace: FR-038-AC-285
#[trace("TC-048")]
#[test]
fn await_anchor_reference_does_not_hold_an_unattached_control() {
    let (mut package, children) = protocol_holder_case("await");
    let event = children[0];
    let orphan = protocol_named_node(
        "control",
        &["Example", "OrderFlow", "run", "Main", "Detached"],
        package["semantic_graph"]["nodes"][event]["body"].clone(),
        &[],
    );
    let orphan_target = orphan["node_id"].clone();
    let orphan_position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(orphan);
    let holder = holder_position(&package);
    package["semantic_graph"]["nodes"][holder]["body"]["arguments"][0]["value"]["members"][1]
        ["value"]["target"] = orphan_target;
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][orphan_position]["node_id"]["digest"]
        .as_str()
        .expect("orphan key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            &format!("/semantic_graph/nodes/{orphan_position}"),
            None,
            key,
        )
    );
}

/// Trace: FR-038-AC-290
#[trace("TC-048")]
#[test]
fn repeat_body_without_necessary_step_refuses_after_local_checks() {
    let (mut package, children) = protocol_holder_case("repeat");
    let body = &mut package["semantic_graph"]["nodes"][children[0]]["body"];
    body["operation"]["member"]["operator"] = json!("check");
    body["arguments"] = json!([{
        "term": "binding", "name": "condition", "value": {
            "term": "literal", "type": node_id(&family_key("aaaa")),
            "value_kind": "boolean", "value": true
        }
    }]);
    settle(&mut package);
    let holder = holder_position(&package);
    let key = package["semantic_graph"]["nodes"][holder]["node_id"]["digest"]
        .as_str()
        .expect("repeat key");
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("repeat without progress must refuse");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::UnsupportedConstruct
    );
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::DeclarationForm)
    );
    assert_eq!(
        refusal.path.as_ref().map(ToString::to_string),
        Some(format!("/semantic_graph/nodes/{holder}"))
    );
    assert_eq!(
        refusal.locus.as_ref().map(|id| id.digest.as_ref()),
        Some(key)
    );
    assert!(matches!(
        refusal.protocol_detail.as_deref(),
        Some(CheckedProtocolRefusalDetail::RepeatProgress { .. })
    ));
}

/// Trace: FR-038-AC-290
#[trace("TC-048")]
#[test]
fn await_anchored_to_its_own_event_refuses_causal_cycle() {
    let (mut package, children) = protocol_holder_case("await");
    let holder = holder_position(&package);
    let event = package["semantic_graph"]["nodes"][children[0]]["node_id"].clone();
    package["semantic_graph"]["nodes"][holder]["body"]["arguments"][0]["value"]["members"][1]
        ["value"]["target"] = event;
    settle(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("await after its own event must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::DefinitionCycle)
    );
    let Some(CheckedProtocolRefusalDetail::CausalCycle { controls }) =
        refusal.protocol_detail.as_deref()
    else {
        panic!("cycle must name its controls");
    };
    let expected: Vec<quire_contract_model::CheckedNodeId> = [holder, children[0]]
        .into_iter()
        .map(|position| {
            serde_json::from_value(package["semantic_graph"]["nodes"][position]["node_id"].clone())
                .expect("control identity")
        })
        .collect();
    assert_eq!(controls, &expected);
}
