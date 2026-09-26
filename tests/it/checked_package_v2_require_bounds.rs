// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! IR-296: `require_bounds` tests the types a value or expression is typed
//! at, never a type that is only a `literal.type` annotation (FR-322 requires
//! one on every literal, including the name literal of every parameter).

use crate::support::checked_package::{
    canonical, evidence_for, node_id, nominal_package, rebuild_source_map, refresh_identity,
    sha256_hex, typed_node_id,
};
use ix_trace_rs::trace;
use quire_contract_ir::{
    CheckedNodeId, CheckedNodeTag, CheckedPackageReadLimits, CheckedPackageV2,
    CheckedPackageV2ReadResult, CompleteLoweringProfileV2, CompleteLoweringRecordV2,
};
use serde_json::{json, Value};

fn key(name: &str) -> String {
    sha256_hex(name.as_bytes())
}

fn node(
    name: &str,
    tag: &str,
    form: &str,
    semantic_type: &str,
    dependencies: &[&str],
    role: &str,
    body: Value,
) -> (String, Value) {
    let id = key(name);
    let mut dependencies = dependencies.iter().map(|k| key(k)).collect::<Vec<_>>();
    dependencies.sort_unstable();
    let node = json!({
        "node_id": node_id(&id),
        "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": tag,
        "semantic_form": form,
        "semantic_type": node_id(&key(semantic_type)),
        "dependencies": dependencies.iter().map(|k| node_id(k)).collect::<Vec<_>>(),
        "occurrences": [{"role": role, "ordinal": 0}],
        "body": body,
    });
    (id, node)
}

fn literal(ty: &str, value_kind: &str, value: &str) -> Value {
    json!({"term": "literal", "type": node_id(&key(ty)), "value_kind": value_kind, "value": value})
}

fn binding(name: &str, value: Value) -> Value {
    json!({"term": "binding", "name": name, "value": value})
}

fn empty() -> Value {
    json!({"term": "aggregate", "members": []})
}

/// The scalar `name`, self-typed.
fn scalar(name: &str, form: &str) -> (String, Value) {
    node(name, "scalar_type", form, name, &[], "type", empty())
}

/// A parameter named `name` typed at the node called `ty`. Its body's two
/// literals are annotated with `text` and `integer` (FR-322).
fn parameter(name: &str, ty: &str) -> (String, Value) {
    let body = json!({"term": "aggregate", "members": [
        binding("name", literal("text", "text", name)),
        binding("level", literal("integer", "integer", "0")),
    ]});
    node(name, "value", "parameter", ty, &[], "anchor", body)
}

/// `x + 1`: an integer `add` over the reference to `x` and the literal 1.
fn add_one(name: &str, x: &str) -> (String, Value) {
    let body = json!({
        "term": "application",
        "operator": "binary",
        "operation": {"identity": "quire.op.integer.add", "laws": [], "mode": null,
            "member": null, "leaves": []},
        "result_type": node_id(&key("integer")),
        "arguments": [
            {"term": "reference", "target": node_id(&key(x))},
            literal("integer", "integer", "1"),
        ],
    });
    // FR-322 application key: the key is re-derived by the reader.
    let id = sha256_hex(&canonical(&json!({
        "version": "quire.application-node/v1",
        "node_tag": "expression",
        "semantic_form": "binary",
        "semantic_type": node_id(&key("integer")),
        "declaration": null,
        "recursion": null,
        "body": body,
    })));
    let (_, mut expression) = node(
        name,
        "expression",
        "binary",
        "integer",
        &[x],
        "expression",
        body,
    );
    expression["node_id"] = node_id(&id);
    (id, expression)
}

/// A package holding `nodes`, ascending by key.
fn package(mut nodes: Vec<(String, Value)>) -> Value {
    nodes.sort_by(|left, right| left.0.cmp(&right.0));
    let mut package = nominal_package(&[]);
    package["semantic_graph"]["nodes"] =
        Value::Array(nodes.into_iter().map(|(_, node)| node).collect());
    rebuild_source_map(&mut package);
    refresh_identity(&mut package);
    package
}

fn admit(value: &Value) -> CheckedPackageV2 {
    match CheckedPackageV2::read(
        &canonical(value),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(value),
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("expected V2 admission, got {other:?}"),
    }
}

fn bounded_profile() -> CompleteLoweringProfileV2 {
    CompleteLoweringProfileV2 {
        supported_tags: CheckedNodeTag::ALL.iter().copied().collect(),
        require_bounds: true,
        work_limit: 100_000,
    }
}

fn id_of(name: &str) -> CheckedNodeId {
    typed_node_id(&key(name))
}

/// `Int[0,9]`: an `integer_range` domain over `integer`.
fn int_0_9() -> (String, Value) {
    node(
        "int09",
        "bounded_domain",
        "integer_range",
        "integer",
        &["integer"],
        "type",
        empty(),
    )
}

fn bounded_x_plus_one() -> Value {
    let (x, x_node) = parameter("x", "int09");
    let (_, expression) = add_one("x_plus_1", "x");
    package(vec![
        scalar("integer", "integer"),
        scalar("text", "text"),
        int_0_9(),
        (x, x_node),
        (
            expression["node_id"]["digest"]
                .as_str()
                .expect("key")
                .to_owned(),
            expression,
        ),
    ])
}

fn only_record(value: &Value, request: &CheckedNodeId) -> CompleteLoweringRecordV2 {
    let result = admit(value).lower(std::slice::from_ref(request), &bounded_profile());
    result.records.into_iter().next().expect("one record")
}

/// Tracing: TC-050, FR-038-AC-39
#[trace("TC-050", "FR-038-AC-39")]
#[test]
fn tc_050_x_plus_one_over_a_bounded_parameter_lowers_under_require_bounds() {
    let value = bounded_x_plus_one();
    let expression = value["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["semantic_form"] == "binary")
        .expect("expression")["node_id"]
        .clone();
    let request: CheckedNodeId = serde_json::from_value(expression).expect("id");
    match only_record(&value, &request) {
        CompleteLoweringRecordV2::Lowered { node } => {
            assert_eq!(node.bounds, vec![id_of("int09")]);
        }
        other => panic!("expected lowered, got {other:?}"),
    }
}

/// Tracing: TC-050, FR-038-AC-39
#[trace("TC-050", "FR-038-AC-39")]
#[test]
fn tc_050_a_value_typed_at_an_unbounded_type_still_requires_a_bound() {
    let (y, y_node) = parameter("y", "integer");
    let value = package(vec![
        scalar("integer", "integer"),
        scalar("text", "text"),
        (y, y_node),
    ]);
    assert_eq!(
        only_record(&value, &id_of("y")),
        CompleteLoweringRecordV2::RequiresBound {
            node_id: id_of("y"),
            unbounded_type: id_of("integer"),
        }
    );
}

/// Tracing: TC-050, FR-038-AC-39
#[trace("TC-050", "FR-038-AC-39")]
#[test]
fn tc_050_a_literal_type_annotation_is_in_the_closure_but_never_refuses() {
    // `x` is typed at `Int[0,9]`; `text` and (through the domain) `integer`
    // are reached as annotations. `text` is unbounded and only an annotation.
    let value = bounded_x_plus_one();
    match only_record(&value, &id_of("x")) {
        CompleteLoweringRecordV2::Lowered { node } => {
            assert!(node.dependencies.contains(&id_of("text")));
        }
        other => panic!("expected lowered, got {other:?}"),
    }
    // The same annotation alone, with no bounding domain anywhere.
    let (z, z_node) = parameter("z", "rational");
    let bare = package(vec![
        scalar("integer", "integer"),
        scalar("text", "text"),
        scalar("rational", "rational"),
        (z, z_node),
    ]);
    // `z` is typed at unbounded `rational`: refused for that. The `text` and
    // `integer` annotations (keys 982d.. and 2dba..) sort before `rational`
    // (fda5..), so a check that also tested annotations would name one of them
    // first; this half is what catches an always-true `typed` check.
    assert_eq!(
        only_record(&bare, &id_of("z")),
        CompleteLoweringRecordV2::RequiresBound {
            node_id: id_of("z"),
            unbounded_type: id_of("rational"),
        }
    );
}

/// Tracing: TC-050, FR-038-AC-39
#[trace("TC-050", "FR-038-AC-39")]
#[test]
fn tc_050_a_type_named_only_through_dependencies_still_requires_a_bound() {
    // `v` names unbounded `rational` in `dependencies` alone: no body
    // reference and no `semantic_type` reaches it.
    let (v, v_node) = node(
        "v",
        "value",
        "literal",
        "int09",
        &["rational"],
        "anchor",
        empty(),
    );
    let value = package(vec![
        scalar("integer", "integer"),
        scalar("text", "text"),
        scalar("rational", "rational"),
        int_0_9(),
        (v, v_node),
    ]);
    assert_eq!(
        only_record(&value, &id_of("v")),
        CompleteLoweringRecordV2::RequiresBound {
            node_id: id_of("v"),
            unbounded_type: id_of("rational"),
        }
    );
}
