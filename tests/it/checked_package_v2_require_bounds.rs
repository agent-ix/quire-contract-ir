// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! IR-296: `require_bounds` tests the types a value or expression is typed
//! at, never a type that is only a `literal.type` annotation (FR-322 requires
//! one on every literal, including the name literal of every parameter).

use crate::support::checked_package::{
    bounds_body, canonical, evidence_for, integer_range_key, node_id, nominal_fixture_members,
    nominal_package, over_body, rebuild_source_map, refresh_identity, sha256_hex, structural_key,
    typed_node_id, BOOLEAN_KEY, INTEGER_KEY,
};
use ix_trace_rs::trace;
use quire_contract_ir::{
    CheckedNodeId, CheckedNodeTag, CheckedPackageReadLimits, CheckedPackageV2,
    CheckedPackageV2ReadResult, CompleteLoweringProfileV2, CompleteLoweringRecordV2,
};
use serde_json::{json, Value};
use std::sync::LazyLock;

/// The key of the nominal unit `Example::Metre` the quantity tests name as
/// `metre`; its node comes from [`quantity_package`], not from [`node`].
static METRE: LazyLock<String> = LazyLock::new(|| nominal_fixture_members()[2].1.clone());

/// The key of the node called `name`: the derived key (FR-038-AC-134) for
/// each anonymous node of the ten derived shapes this file builds, the key
/// of the nominal unit for `metre`, and the digest of the name otherwise.
fn key(name: &str) -> String {
    match name {
        "metre" => METRE.clone(),
        "integer" => INTEGER_KEY.to_owned(),
        "boolean" => BOOLEAN_KEY.to_owned(),
        "int09" => integer_range_key("0", "9"),
        "bools" => collection_key("sequence", "boolean"),
        "ints" => collection_key("sequence", "int09"),
        "plain_ints" => collection_key("sequence", "integer"),
        "seq" => collection_key("sequence", "metre"),
        "opt" => collection_key("option", "metre"),
        "bools03" => bounded_key("bools"),
        "ints03" => bounded_key("ints"),
        "plain_ints03" => bounded_key("plain_ints"),
        "seq03" => bounded_key("seq"),
        _ => sha256_hex(name.as_bytes()),
    }
}

/// The derived key of the self-typed `form` node over the node `element`.
fn collection_key(form: &str, element: &str) -> String {
    structural_key("composite_type", form, None, &over_body(&key(element)))
}

/// The derived key of the `collection_bounds` `[0, 3]` over the node `over`.
fn bounded_key(over: &str) -> String {
    structural_key(
        "bounded_domain",
        "collection_bounds",
        Some(&key(over)),
        &bounds_body("0", "3"),
    )
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
    application(name, "integer", targets, arguments)
}

/// An `add` application named `name` whose `result_type` and `semantic_type`
/// are the node called `result`.
fn application(
    name: &str,
    result: &str,
    targets: &[&str],
    arguments: Vec<Value>,
) -> (String, Value) {
    let body = json!({
        "term": "application",
        "operator": "binary",
        "operation": {"identity": "quire.op.integer.add", "laws": [], "mode": null,
            "member": null, "leaves": []},
        "result_type": node_id(&key(result)),
        "arguments": arguments,
    });
    // FR-322 application key: the key is re-derived by the reader.
    let id = sha256_hex(&canonical(&json!({
        "version": "quire.application-node/v1",
        "node_tag": "expression",
        "semantic_form": "binary",
        "semantic_type": node_id(&key(result)),
        "declaration": null,
        "recursion": null,
        "body": body,
    })));
    let (_, mut expression) = node(
        name,
        "expression",
        "binary",
        result,
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
        bounds_body("0", "9"),
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
    // A node of a derived shape holds the closed body of its form: one
    // `reference` to its element (FR-038-AC-134).
    let body = if matches!(
        form,
        "option" | "reference" | "set" | "bag" | "sequence" | "ordered_set"
    ) {
        over_body(&key(over[0]))
    } else {
        empty()
    };
    node(name, "composite_type", form, name, over, "type", body)
}

/// `K<E>[0, 3]` over the composite `seq`: a `collection_bounds` domain whose
/// min and max literals are annotated with the unbounded `integer` type.
fn bounded_collection(name: &str, seq: &str) -> (String, Value) {
    let body = bounds_body("0", "3");
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
        composite("plain_ints", "sequence", &["integer"]),
        bounded_collection("plain_ints03", "plain_ints"),
        composite("holder", "record", &["plain_ints03", "int09"]),
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
    // the cycle shares one `recursion_group`, and depth has no domain. The
    // sequence and its bounds sit in the group, where the derived-key stage
    // skips them (FR-038, IR-627-Q4).
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

/// A package, built from this crate's own vocabulary in QSL's shapes, holding
/// the nominal dimension and unit `metre`, the compound unit `cu` over it, the
/// scalars the bodies annotate, `int09`, and every node of the quantity
/// cases below. Each request's closure is its own, so one package serves all.
fn quantity_package() -> Value {
    let members = nominal_fixture_members();
    let mut package = nominal_package(&[members[3].clone(), members[2].clone()]);
    let compound = json!({"term": "aggregate", "members": [
        {"term": "aggregate", "members": [
            binding("unit", reference("metre")),
            binding("exponent", literal("integer", "integer", "2")),
        ]},
    ]});
    let (x, x_node) = parameter("x", "int09");
    let mut nodes = vec![
        scalar("integer", "integer"),
        scalar("text", "text"),
        scalar("boolean", "boolean"),
        int_0_9(),
        node(
            "cu",
            "scalar_type",
            "compound_unit",
            "cu",
            &["metre"],
            "type",
            compound,
        ),
        // A position typed directly at a unit and at a compound unit.
        composite("unit_field", "record", &["metre"]),
        composite("cu_field", "record", &["cu"]),
        // A collection, an option and an alias of a quantity.
        composite("seq", "sequence", &["metre"]),
        bounded_collection("seq03", "seq"),
        composite("opt", "option", &["metre"]),
        composite("alias", "alias", &["metre"]),
        parameter("p_unit", "metre"),
        parameter("p_cu", "cu"),
        // Domains over a unit, a domain, and a compound unit.
        quantity_domain("dom", "metre"),
        quantity_domain("dom2", "dom"),
        quantity_domain("cdom", "cu"),
        composite("dom_field", "record", &["dom"]),
        composite("dom2_field", "record", &["dom2"]),
        composite("cdom_field", "record", &["cdom"]),
        // Two quantity positions in one closure.
        composite("both", "record", &["metre", "cu"]),
        // A unit named only by a `literal.type` annotation.
        node(
            "lit",
            "value",
            "literal",
            "boolean",
            &[],
            "anchor",
            literal("metre", "integer", "1"),
        ),
        composite("annotated", "record", &["boolean", "lit"]),
        // A domain chain that loops, and a field typed at it.
        in_recursion_group(quantity_domain("loop_a", "loop_b")),
        in_recursion_group(quantity_domain("loop_b", "loop_a")),
        composite("loop_field", "record", &["loop_a"]),
        (x, x_node),
    ];
    // An application whose `result_type` is a quantity, over a parameter
    // typed at a bounded type.
    nodes.push(application(
        "q_app",
        "metre",
        &["x"],
        vec![reference("x"), literal("integer", "integer", "1")],
    ));
    nodes.extend(
        package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .map(|node| {
                let digest = node["node_id"]["digest"].as_str().expect("key");
                (digest.to_owned(), node.clone())
            }),
    );
    nodes.sort_by(|left, right| left.0.cmp(&right.0));
    package["semantic_graph"]["nodes"] =
        Value::Array(nodes.into_iter().map(|(_, node)| node).collect());
    rebuild_source_map(&mut package);
    refresh_identity(&mut package);
    package
}

/// A `rational_range` domain named `name` over the type called `base`.
fn quantity_domain(name: &str, base: &str) -> (String, Value) {
    node(
        name,
        "bounded_domain",
        "rational_range",
        base,
        &[base],
        "type",
        empty(),
    )
}

fn record_with(value: &Value, name: &str, require_bounds: bool) -> CompleteLoweringRecordV2 {
    let profile = CompleteLoweringProfileV2 {
        require_bounds,
        ..bounded_profile()
    };
    let result = admit(value).lower(&[id_of(name)], &profile);
    result.records.into_iter().next().expect("one record")
}

fn requires_bound_at(request: &str, unbounded: &str) -> CompleteLoweringRecordV2 {
    CompleteLoweringRecordV2::RequiresBound {
        node_id: id_of(request),
        unbounded_type: id_of(unbounded),
    }
}

/// Tracing: TC-050, FR-038-AC-73
#[trace("TC-050", "FR-038-AC-73")]
#[test]
fn tc_050_a_position_typed_at_a_quantity_requires_a_bound_naming_the_unit() {
    let value = quantity_package();
    // Each request is raised at the `unit` or `compound_unit` node, never at
    // the composite, the parameter or a `bounded_domain` between them.
    let cases = [
        ("unit_field", "metre"),
        ("cu_field", "cu"),
        ("seq03", "metre"),
        ("opt", "metre"),
        ("alias", "metre"),
        ("p_unit", "metre"),
        ("p_cu", "cu"),
        ("dom_field", "metre"),
        ("dom2_field", "metre"),
        ("cdom_field", "cu"),
    ];
    for (request, unit) in cases {
        assert_eq!(
            record_with(&value, request, true),
            requires_bound_at(request, unit),
            "{request}"
        );
    }
}

/// Tracing: TC-050, FR-038-AC-73
#[trace("TC-050", "FR-038-AC-73")]
#[test]
fn tc_050_the_least_quantity_among_positions_is_named() {
    let value = quantity_package();
    let least = std::cmp::min(id_of("metre"), id_of("cu"));
    assert_eq!(
        record_with(&value, "both", true),
        CompleteLoweringRecordV2::RequiresBound {
            node_id: id_of("both"),
            unbounded_type: least,
        }
    );
}

/// Tracing: TC-050, FR-038-AC-73
#[trace("TC-050", "FR-038-AC-73")]
#[test]
fn tc_050_a_unit_is_not_a_quantity_position_merely_by_being_reached() {
    let value = quantity_package();
    // A requested unit and compound unit lower, the compound unit with its
    // unit in `dependencies`.
    for request in ["metre", "cu"] {
        assert!(
            matches!(
                record_with(&value, request, true),
                CompleteLoweringRecordV2::Lowered { .. }
            ),
            "{request}"
        );
    }
    match record_with(&value, "cu", true) {
        CompleteLoweringRecordV2::Lowered { node } => {
            assert!(node.dependencies.contains(&id_of("metre")));
        }
        other => panic!("expected lowered, got {other:?}"),
    }
    // A unit reached only as a `literal.type` annotation.
    match record_with(&value, "annotated", true) {
        CompleteLoweringRecordV2::Lowered { node } => {
            assert!(node.dependencies.contains(&id_of("metre")));
        }
        other => panic!("expected lowered, got {other:?}"),
    }
    // An application's `result_type` is not a position.
    let application = value["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["semantic_form"] == "binary")
        .expect("application")["node_id"]["digest"]
        .as_str()
        .expect("key")
        .to_owned();
    let result = admit(&value).lower(&[typed_node_id(&application)], &bounded_profile());
    match &result.records[0] {
        CompleteLoweringRecordV2::Lowered { node } => {
            assert!(node.dependencies.contains(&id_of("metre")));
        }
        other => panic!("expected lowered, got {other:?}"),
    }
}

/// Tracing: TC-050, FR-038-AC-73
#[trace("TC-050", "FR-038-AC-73")]
#[test]
fn tc_050_a_cyclic_domain_chain_ends_without_a_quantity() {
    let value = quantity_package();
    assert!(matches!(
        record_with(&value, "loop_field", true),
        CompleteLoweringRecordV2::Lowered { .. }
    ));
}

/// Tracing: TC-050, FR-038-AC-73
#[trace("TC-050", "FR-038-AC-73")]
#[test]
fn tc_050_without_require_bounds_every_quantity_position_lowers() {
    let value = quantity_package();
    for request in [
        "unit_field",
        "cu_field",
        "seq03",
        "opt",
        "alias",
        "p_unit",
        "p_cu",
        "dom_field",
        "dom2_field",
        "cdom_field",
        "both",
    ] {
        assert!(
            matches!(
                record_with(&value, request, false),
                CompleteLoweringRecordV2::Lowered { .. }
            ),
            "{request}"
        );
    }
}
