// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Public-reader controls for declared-unit, aggregate and literal families.

use crate::support::checked_package::{
    application_node_key, canonical, evidence_for, node_id, nominal_fixture_members,
    nominal_package, rebuild_source_map, refresh_identity, settle, sha256_hex, structural_key,
    v2_all_families,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedPackageReadLimits, CheckedPackageRefusalCause, CheckedPackageRefusalCode,
    CheckedPackageV2, CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};

fn add_node(package: &mut Value, key: &str, tag: &str, form: &str, ty: &str, body: Value) {
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(json!({
            "node_id": node_id(key),
            "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": tag, "semantic_form": form,
            "semantic_type": node_id(ty),
            "dependencies": [],
            "occurrences": [{"role": "expression", "ordinal": 0}],
            "body": body,
        }));
}

fn fixture() -> (Value, String, String, String, String, String) {
    let members = nominal_fixture_members();
    let unit = members
        .iter()
        .find(|(p, _)| p["version"] == "quire.unit-node/v1")
        .expect("unit")
        .1
        .clone();
    let mut package = nominal_package(&members);
    let empty = json!({"term": "aggregate", "members": []});
    let scalar = |form| structural_key("scalar_type", form, None, &empty);
    let boolean = scalar("boolean");
    let integer = scalar("integer");
    let text = scalar("text");
    for (form, key) in [
        ("boolean", &boolean),
        ("integer", &integer),
        ("text", &text),
    ] {
        add_node(&mut package, key, "scalar_type", form, key, empty.clone());
    }
    let parameter = |name: &str| {
        json!({"term": "aggregate", "members": [
            {"term": "binding", "name": "name", "value":
                {"term": "literal", "type": node_id(&text), "value_kind": "text", "value": name}},
            {"term": "binding", "name": "level", "value":
                {"term": "literal", "type": node_id(&integer), "value_kind": "integer", "value": "0"}},
        ]})
    };
    let left_body = parameter("left");
    let right_body = parameter("right");
    let left = structural_key("value", "parameter", Some(&unit), &left_body);
    let right = structural_key("value", "parameter", Some(&unit), &right_body);
    add_node(&mut package, &left, "value", "parameter", &unit, left_body);
    add_node(
        &mut package,
        &right,
        "value",
        "parameter",
        &unit,
        right_body,
    );
    rebuild_source_map(&mut package);
    refresh_identity(&mut package);
    (package, unit, boolean, integer, left, right)
}

fn operation(identity: &str, mode: Value, member: Value) -> Value {
    json!({"identity": identity, "laws": [], "mode": mode, "member": member, "leaves": []})
}

fn reference(key: &str) -> Value {
    json!({"term": "reference", "target": node_id(key)})
}

fn append_application(
    package: &mut Value,
    operator: &str,
    form: &str,
    operation: Value,
    result: &str,
    arguments: Vec<Value>,
) -> usize {
    let body = json!({"term": "application", "operator": operator,
        "operation": operation, "result_type": node_id(result), "arguments": arguments});
    let key = application_node_key("expression", form, result, &body);
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    add_node(package, &key, "expression", form, result, body);
    settle(package);
    position
}

fn append_literal(package: &mut Value, ty: &str, kind: &str, value: Value) -> usize {
    let body = json!({"term": "literal", "type": node_id(ty),
        "value_kind": kind, "value": value});
    let key = structural_key("value", "literal", Some(ty), &body);
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    add_node(package, &key, "value", "literal", ty, body);
    settle(package);
    position
}

fn change_literal_type(package: &mut Value, position: usize, ty: &str) {
    let node = &mut package["semantic_graph"]["nodes"][position];
    node["body"]["type"] = node_id(ty);
    let own_type = node["semantic_type"]["digest"]
        .as_str()
        .expect("type")
        .to_owned();
    let key = structural_key("value", "literal", Some(&own_type), &node["body"]);
    node["node_id"] = node_id(&key);
    settle(package);
}

fn read(package: &Value) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(package),
    )
}

fn admits(package: &Value) {
    assert!(
        matches!(read(package), CheckedPackageV2ReadResult::Admitted(_)),
        "{:?}",
        read(package)
    );
}

fn refuses(
    package: &Value,
    code: CheckedPackageRefusalCode,
    cause: CheckedPackageRefusalCause,
    path: &str,
) {
    let CheckedPackageV2ReadResult::Refused(actual) = read(package) else {
        panic!("expected refusal, got {:?}", read(package));
    };
    assert_eq!(
        (
            actual.code,
            actual.cause,
            actual.path.as_ref().map(ToString::to_string)
        ),
        (code, Some(cause), Some(path.to_owned()))
    );
}

/// Trace: TC-048, FR-038-AC-210
#[trace("TC-048", "FR-038-AC-210")]
#[test]
fn quantity_family_checks_every_catalogued_quantity_operand() {
    let (base, unit, boolean, integer, left, right) = fixture();
    for identity in [
        "add", "sub", "mul", "div", "pow", "eq", "ne", "lt", "le", "gt", "ge", "convert",
    ] {
        let mut package = base.clone();
        let convert = identity == "convert";
        let result = if matches!(identity, "eq" | "ne" | "lt" | "le" | "gt" | "ge") {
            &boolean
        } else {
            &unit
        };
        let second = if identity == "pow" { &integer } else { &right };
        let args = if convert {
            vec![reference(&left)]
        } else {
            vec![reference(&left), reference(second)]
        };
        let mode = if convert {
            json!({"kind": "rounding", "value": "exact"})
        } else {
            Value::Null
        };
        let member = if convert {
            json!({"kind": "type_argument", "type_argument": node_id(&unit)})
        } else {
            Value::Null
        };
        let at = append_application(
            &mut package,
            if convert { "convert" } else { "binary" },
            if convert { "conversion" } else { "binary" },
            operation(&format!("quire.op.quantity.{identity}"), mode, member),
            result,
            args,
        );
        admits(&package);
        if !matches!(identity, "pow" | "convert") {
            let mut second = package.clone();
            second["semantic_graph"]["nodes"][at]["body"]["arguments"][1] = reference(&boolean);
            settle(&mut second);
            refuses(
                &second,
                CheckedPackageRefusalCode::IllTyped,
                CheckedPackageRefusalCause::OperatorIneligible,
                &format!("/semantic_graph/nodes/{at}/body/arguments/1"),
            );
        }
        package["semantic_graph"]["nodes"][at]["body"]["arguments"][0] = reference(&boolean);
        settle(&mut package);
        refuses(
            &package,
            CheckedPackageRefusalCode::IllTyped,
            CheckedPackageRefusalCause::OperatorIneligible,
            &format!("/semantic_graph/nodes/{at}/body/arguments/0"),
        );
    }
    let mut pow = base.clone();
    let at = append_application(
        &mut pow,
        "binary",
        "binary",
        operation("quire.op.quantity.pow", Value::Null, Value::Null),
        &unit,
        vec![reference(&left), reference(&integer)],
    );
    admits(&pow);
    pow["semantic_graph"]["nodes"][at]["body"]["arguments"][1] = reference(&right);
    settle(&mut pow);
    refuses(
        &pow,
        CheckedPackageRefusalCode::IllTyped,
        CheckedPackageRefusalCause::OperatorIneligible,
        &format!("/semantic_graph/nodes/{at}/body/arguments/1"),
    );

    let mut integer_add = base;
    let at = append_application(
        &mut integer_add,
        "binary",
        "binary",
        operation("quire.op.integer.add", Value::Null, Value::Null),
        &integer,
        vec![reference(&left), reference(&integer)],
    );
    refuses(
        &integer_add,
        CheckedPackageRefusalCode::IllTyped,
        CheckedPackageRefusalCause::OperatorIneligible,
        &format!("/semantic_graph/nodes/{at}/body/arguments/0"),
    );
}

/// Trace: TC-048, FR-038-AC-211
#[trace("TC-048", "FR-038-AC-211")]
#[test]
fn control_if_checks_quantity_branch_family() {
    let (mut package, unit, boolean, integer, left, right) = fixture();
    let at = append_application(
        &mut package,
        "conditional",
        "conditional",
        operation("quire.op.control.if", Value::Null, Value::Null),
        &unit,
        vec![reference(&boolean), reference(&left), reference(&right)],
    );
    admits(&package);
    package["semantic_graph"]["nodes"][at]["body"]["arguments"][2] = reference(&integer);
    settle(&mut package);
    refuses(
        &package,
        CheckedPackageRefusalCode::IllTyped,
        CheckedPackageRefusalCause::OperatorIneligible,
        &format!("/semantic_graph/nodes/{at}/body/arguments/2"),
    );
}

/// Trace: TC-048, FR-038-AC-211
#[trace("TC-048", "FR-038-AC-211")]
#[test]
fn aggregate_arguments_admit_at_clause_positions_and_refuse_at_boolean_position() {
    let temporal = v2_all_families();
    let temporal_clause = temporal["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["body"]["operation"]["identity"] == "quire.op.temporal.clause")
        .expect("temporal clause");
    for position in [2, 3, 4] {
        assert_eq!(
            temporal_clause["body"]["arguments"][position]["term"],
            "aggregate"
        );
    }
    admits(&temporal);

    let (mut state, at) = crate::checked_package_v2_frame_entries::state_operand_fixture();
    assert_eq!(
        state["semantic_graph"]["nodes"][at]["body"]["arguments"][0]["term"],
        "aggregate"
    );
    admits(&state);
    let boolean = state["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "scalar_type" && node["semantic_form"] == "boolean")
        .expect("Boolean")["node_id"]
        .clone();
    state["semantic_graph"]["nodes"][at]["body"]["arguments"][0] =
        json!({"term": "reference", "target": boolean});
    settle(&mut state);
    let CheckedPackageV2ReadResult::Refused(actual) = read(&state) else {
        panic!("state shape must refuse before operation family");
    };
    assert_eq!(actual.code, CheckedPackageRefusalCode::InvalidSemanticGraph);
    assert_eq!(
        actual.path.expect("path").as_str(),
        format!("/semantic_graph/nodes/{at}/body")
    );

    let (mut package, _, boolean, _, _, _) = fixture();
    let at = append_application(
        &mut package,
        "binary",
        "binary",
        operation("quire.op.boolean.and", Value::Null, Value::Null),
        &boolean,
        vec![
            json!({"term": "aggregate", "members": []}),
            reference(&boolean),
        ],
    );
    refuses(
        &package,
        CheckedPackageRefusalCode::IllTyped,
        CheckedPackageRefusalCause::OperatorIneligible,
        &format!("/semantic_graph/nodes/{at}/body/arguments/0"),
    );
}

/// Trace: TC-048, FR-038-AC-212
#[trace("TC-048", "FR-038-AC-212")]
#[test]
fn literal_kind_is_checked_at_its_own_type_even_outside_family_operands() {
    let (base, unit, boolean, integer, _, _) = fixture();
    let enum_key = nominal_fixture_members()
        .iter()
        .find(|(preimage, _)| preimage["version"] == "quire.enum-declaration-node/v1")
        .expect("enum")
        .1
        .clone();
    let mut with_option = base.clone();
    let option_body = json!({"term": "aggregate", "members": [reference(&integer)]});
    let option = structural_key("composite_type", "option", None, &option_body);
    add_node(
        &mut with_option,
        &option,
        "composite_type",
        "option",
        &option,
        option_body,
    );
    settle(&mut with_option);

    for (name, package, ty, kind, value, wrong) in [
        (
            "integer",
            base.clone(),
            integer.as_str(),
            "integer",
            json!("1"),
            boolean.as_str(),
        ),
        (
            "enum",
            base.clone(),
            enum_key.as_str(),
            "enum",
            json!("ACTIVE"),
            boolean.as_str(),
        ),
        (
            "none",
            with_option,
            option.as_str(),
            "none",
            Value::Null,
            boolean.as_str(),
        ),
        (
            "unit",
            base,
            integer.as_str(),
            "integer",
            json!("2"),
            unit.as_str(),
        ),
    ] {
        let mut positive = package;
        let at = append_literal(&mut positive, ty, kind, value);
        admits(&positive);
        change_literal_type(&mut positive, at, wrong);
        refuses(
            &positive,
            CheckedPackageRefusalCode::IllTyped,
            CheckedPackageRefusalCause::OperatorIneligible,
            &format!("/semantic_graph/nodes/{at}/body/type"),
        );
        assert_eq!(
            positive["semantic_graph"]["nodes"][at]["body"]["value_kind"], kind,
            "{name}"
        );
    }

    let mut ordered_members = nominal_fixture_members();
    let declaration = ordered_members
        .iter()
        .position(|(preimage, _)| preimage["version"] == "quire.enum-declaration-node/v1")
        .expect("declaration");
    ordered_members[declaration].0["ordered"] = json!(true);
    let ordered_key = sha256_hex(&canonical(&ordered_members[declaration].0));
    ordered_members[declaration].1 = ordered_key.clone();
    let member = ordered_members
        .iter()
        .position(|(preimage, _)| preimage["version"] == "quire.enum-member-node/v1")
        .expect("member");
    ordered_members[member].0["declaration_node_id"] = node_id(&ordered_key);
    ordered_members[member].1 = sha256_hex(&canonical(&ordered_members[member].0));
    let mut ordered = nominal_package(&ordered_members);
    let at = append_literal(&mut ordered, &ordered_key, "enum", json!("ACTIVE"));
    admits(&ordered);
    change_literal_type(&mut ordered, at, &unit);
    refuses(
        &ordered,
        CheckedPackageRefusalCode::IllTyped,
        CheckedPackageRefusalCause::OperatorIneligible,
        &format!("/semantic_graph/nodes/{at}/body/type"),
    );

    // The enclosing `any_value` operand accepts either branch's family, so
    // only the literal's own kind/type rule can reject this mutation.
    let (mut control, _, boolean, integer, _, _) = fixture();
    let integer_literal = json!({"term": "literal", "type": node_id(&integer),
        "value_kind": "integer", "value": "3"});
    let at = append_application(
        &mut control,
        "conditional",
        "conditional",
        operation("quire.op.control.if", Value::Null, Value::Null),
        &integer,
        vec![reference(&boolean), integer_literal, reference(&integer)],
    );
    admits(&control);
    control["semantic_graph"]["nodes"][at]["body"]["arguments"][1]["type"] = node_id(&boolean);
    settle(&mut control);
    refuses(
        &control,
        CheckedPackageRefusalCode::IllTyped,
        CheckedPackageRefusalCause::OperatorIneligible,
        &format!("/semantic_graph/nodes/{at}/body/arguments/1/type"),
    );
}

/// Trace: TC-048, FR-038-AC-213
#[trace("TC-048", "FR-038-AC-213")]
#[test]
fn quantity_conversion_defaults_to_exact_rounding() {
    let (mut package, unit, _, _, left, _) = fixture();
    let at = append_application(
        &mut package,
        "convert",
        "conversion",
        operation(
            "quire.op.quantity.convert",
            json!({"kind": "rounding", "value": "exact"}),
            json!({"kind": "type_argument", "type_argument": node_id(&unit)}),
        ),
        &unit,
        vec![reference(&left)],
    );
    admits(&package);
    package["semantic_graph"]["nodes"][at]["body"]["operation"]["mode"]["value"] =
        json!("toward-zero");
    settle(&mut package);
    refuses(
        &package,
        CheckedPackageRefusalCode::InvalidPackage,
        CheckedPackageRefusalCause::OperationModeTypeMismatch,
        &format!("/semantic_graph/nodes/{at}/body/operation/mode/value"),
    );
}

/// Trace: TC-048, FR-038-AC-213
#[trace("TC-048", "FR-038-AC-213")]
#[test]
fn quantity_sum_uses_its_result_type_exact_rounding_pin() {
    let (mut package, unit, _, _, left, _) = fixture();
    let set_body = json!({"term": "aggregate", "members": [reference(&unit)]});
    let set = structural_key("composite_type", "set", None, &set_body);
    add_node(&mut package, &set, "composite_type", "set", &set, set_body);
    let at = append_application(
        &mut package,
        "collection",
        "collection",
        operation(
            "quire.op.collection.sum.quantity",
            json!({"kind": "rounding", "value": "exact"}),
            json!({"kind": "type_argument", "type_argument": node_id(&unit)}),
        ),
        &unit,
        vec![
            reference(&set),
            json!({"term": "binding", "name": "item", "value": reference(&left)}),
        ],
    );
    admits(&package);
    package["semantic_graph"]["nodes"][at]["body"]["operation"]["mode"]["value"] =
        json!("toward-zero");
    settle(&mut package);
    refuses(
        &package,
        CheckedPackageRefusalCode::InvalidPackage,
        CheckedPackageRefusalCause::OperationModeTypeMismatch,
        &format!("/semantic_graph/nodes/{at}/body/operation/mode/value"),
    );
}
