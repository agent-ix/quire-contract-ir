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
    node(name, "value", "parameter", ty, &[], "expression", body)
}

/// An integer `add` application named `name` over `arguments`, whose
/// `reference` targets (named here) are its dependencies.
fn add(name: &str, targets: &[&str], arguments: Vec<Value>) -> (String, Value) {
    let body = json!({
        "term": "application",
        "operator": "binary",
        "operation": {"identity": "quire.op.integer.add", "laws": [], "mode": null,
            "member": null, "leaves": []},
        "result_type": node_id(&key("integer")),
        "arguments": arguments,
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
        targets,
        "expression",
        body,
    );
    expression["node_id"] = node_id(&id);
    (id, expression)
}

fn reference(name: &str) -> Value {
    json!({"term": "reference", "target": node_id(&key(name))})
}

/// `x + 1`: an integer `add` over the reference to `x` and the literal 1.
fn add_one(name: &str, x: &str) -> (String, Value) {
    add(
        name,
        &[x],
        vec![reference(x), literal("integer", "integer", "1")],
    )
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

/// A composite type node named `name`, self-typed, naming each of `over` as
/// an element or field type.
fn composite(name: &str, form: &str, over: &[&str]) -> (String, Value) {
    node(name, "composite_type", form, name, over, "type", empty())
}

/// `K<E>[0, 3]` over the composite `seq`: a `collection_bounds` domain whose
/// min and max literals are annotated with the unbounded `integer` type.
fn bounded_collection(name: &str, seq: &str) -> (String, Value) {
    let body = json!({"term": "aggregate", "members": [
        binding("min", literal("integer", "integer", "0")),
        binding("max", literal("integer", "integer", "3")),
    ]});
    node(
        name,
        "bounded_domain",
        "collection_bounds",
        seq,
        &[seq],
        "type",
        body,
    )
}

fn in_recursion_group((id, mut node): (String, Value)) -> (String, Value) {
    node["recursion_group"] = json!("g");
    (id, node)
}

/// Tracing: TC-050, FR-038-AC-40
#[trace("TC-050", "FR-038-AC-40")]
#[test]
fn tc_050_a_bound_over_the_shared_integer_does_not_cover_an_unbounded_field() {
    // `{n: Integer, k: Int[0,9]}`: `int09` is a domain over the same `integer`
    // node the field `n` names, and must not bound it.
    let value = package(vec![
        scalar("integer", "integer"),
        scalar("text", "text"),
        int_0_9(),
        composite("mixed", "record", &["integer", "int09"]),
    ]);
    assert_eq!(
        only_record(&value, &id_of("mixed")),
        CompleteLoweringRecordV2::RequiresBound {
            node_id: id_of("mixed"),
            unbounded_type: id_of("integer"),
        }
    );
}

/// Tracing: TC-050, FR-038-AC-40
#[trace("TC-050", "FR-038-AC-40")]
#[test]
fn tc_050_bounded_collections_and_ranged_fields_lower() {
    // `{xs: Sequence<Boolean>[0,3], ys: Sequence<Int[0,9]>[0,3], k: Int[0,9]}`:
    // every position is bounded, and the domains' own `integer` literals are
    // annotations, not positions.
    let value = package(vec![
        scalar("integer", "integer"),
        scalar("text", "text"),
        scalar("boolean", "boolean"),
        int_0_9(),
        composite("bools", "sequence", &["boolean"]),
        bounded_collection("bools03", "bools"),
        composite("ints", "sequence", &["int09"]),
        bounded_collection("ints03", "ints"),
        composite("box", "record", &["bools03", "ints03", "int09"]),
    ]);
    match only_record(&value, &id_of("box")) {
        CompleteLoweringRecordV2::Lowered { node } => {
            assert_eq!(
                node.bounds,
                vec![id_of("int09"), id_of("bools03"), id_of("ints03")]
                    .into_iter()
                    .collect::<std::collections::BTreeSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>()
            );
        }
        other => panic!("expected lowered, got {other:?}"),
    }
}

/// Tracing: TC-050, FR-038-AC-40
#[trace("TC-050", "FR-038-AC-40")]
#[test]
fn tc_050_a_collection_of_unranged_integers_requires_a_bound() {
    // `Sequence<Integer>[0,3]` beside `Int[0,9]`: the count is bounded, the
    // element is not.
    let value = package(vec![
        scalar("integer", "integer"),
        scalar("text", "text"),
        int_0_9(),
        composite("ints", "sequence", &["integer"]),
        bounded_collection("ints03", "ints"),
        composite("holder", "record", &["ints03", "int09"]),
    ]);
    assert_eq!(
        only_record(&value, &id_of("holder")),
        CompleteLoweringRecordV2::RequiresBound {
            node_id: id_of("holder"),
            unbounded_type: id_of("integer"),
        }
    );
}

/// Tracing: TC-050, FR-038-AC-41
#[trace("TC-050", "FR-038-AC-41")]
#[test]
fn tc_050_a_recursive_record_requires_a_bound() {
    // `{kids: Sequence<Tree>[0,3], k: Int[0,9]}` where `Tree` is the record:
    // the cycle shares one `recursion_group`, and depth has no domain.
    let value = package(vec![
        scalar("integer", "integer"),
        scalar("text", "text"),
        int_0_9(),
        in_recursion_group(composite("tree", "record", &["kids03", "int09"])),
        in_recursion_group(composite("kids", "sequence", &["tree"])),
        in_recursion_group(bounded_collection("kids03", "kids")),
    ]);
    let least = std::cmp::min(id_of("tree"), id_of("kids"));
    assert_eq!(
        only_record(&value, &id_of("tree")),
        CompleteLoweringRecordV2::RequiresBound {
            node_id: id_of("tree"),
            unbounded_type: least,
        }
    );
}

/// Tracing: TC-050, FR-038-AC-40
#[trace("TC-050", "FR-038-AC-40")]
#[test]
fn tc_050_a_bound_over_one_parameter_does_not_cover_another_of_the_same_type() {
    // `(x + 1) + n` over `x: Int[0,9]` and `n: Integer`: the range over
    // `integer` bounds `x`, not `n`.
    let (x, x_node) = parameter("x", "int09");
    let (n, n_node) = parameter("n", "integer");
    let (inner, inner_node) = add_one("x_plus_1", "x");
    let (outer, outer_node) = add(
        "outer",
        &[],
        vec![
            json!({"term": "reference", "target": node_id(&inner)}),
            reference("n"),
        ],
    );
    // The outer application depends on the inner one by its re-derived key.
    let mut outer_node = outer_node;
    let mut targets = [inner.clone(), key("n")];
    targets.sort_unstable();
    outer_node["dependencies"] = json!(targets.map(|digest| node_id(&digest)));
    let value = package(vec![
        scalar("integer", "integer"),
        scalar("text", "text"),
        int_0_9(),
        (x, x_node),
        (n, n_node),
        (inner, inner_node),
        (outer.clone(), outer_node),
    ]);
    let request: CheckedNodeId = typed_node_id(&outer);
    assert_eq!(
        only_record(&value, &request),
        CompleteLoweringRecordV2::RequiresBound {
            node_id: request,
            unbounded_type: id_of("integer"),
        }
    );
}
