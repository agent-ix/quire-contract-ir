// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-038-AC-99, AC-101 and AC-106, at the package: the union type, union
//! value and `case` nodes of QSpec FR-440, `quire.op.structural.eq` over
//! unions, and the lowering of an admitted `expression`/`case` node.
//!
//! Every package is built here, node by node, from this crate's own
//! vocabulary and keyed by `settle`; nothing of QSpec's fixtures is copied in
//! (they are read by `make conformance-qspec`, FR-038-AC-107).

use crate::support::checked_package::{
    canonical, evidence_for, family_key, node_id, nominal_package, settle, sha256_hex,
    typed_node_id, v2_all_families,
};
use ix_trace_rs::trace;
use quire_contract_ir::{
    CheckedNodeTag, CheckedPackageReadLimits, CheckedPackageRefusalCause as Cause,
    CheckedPackageRefusalCode as Code, CheckedPackageV2, CheckedPackageV2ReadResult,
    CompleteLoweringProfileV2, CompleteLoweringRecordV2, CONTRACT_IR_SEMANTIC_DOMAIN,
};
use serde_json::{json, Value};
use std::collections::BTreeSet;

fn key(name: &str) -> String {
    sha256_hex(name.as_bytes())
}

fn node(
    name: &str,
    tag: &str,
    form: &str,
    semantic_type: &str,
    over: &[&str],
    role: &str,
    body: Value,
) -> Value {
    let dependencies: BTreeSet<String> = over.iter().map(|name| key(name)).collect();
    json!({
        "node_id": node_id(&key(name)),
        "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": tag, "semantic_form": form,
        "semantic_type": node_id(&key(semantic_type)),
        "dependencies": dependencies.iter().map(|digest| node_id(digest)).collect::<Vec<_>>(),
        "occurrences": [{"role": role, "ordinal": 0}],
        "body": body,
    })
}

fn literal(ty: &str, value_kind: &str, value: &str) -> Value {
    json!({"term": "literal", "type": node_id(&key(ty)), "value_kind": value_kind, "value": value})
}

fn binding(name: &str, value: Value) -> Value {
    json!({"term": "binding", "name": name, "value": value})
}

fn reference(name: &str) -> Value {
    json!({"term": "reference", "target": node_id(&key(name))})
}

fn aggregate(members: Vec<Value>) -> Value {
    json!({"term": "aggregate", "members": members})
}

fn scalar(name: &str, form: &str) -> Value {
    node(
        name,
        "scalar_type",
        form,
        name,
        &[],
        "type",
        aggregate(vec![]),
    )
}

/// `Text[nfc]`: a `text_bounds` domain binding the `nfc` profile over `text`.
fn nfc_text() -> Value {
    node(
        "nfc",
        "bounded_domain",
        "text_bounds",
        "text",
        &["text"],
        "type",
        aggregate(vec![binding(
            "text_profile",
            literal("text", "text", "nfc"),
        )]),
    )
}

/// A parameter named `name`, at binder level `level`, typed at `ty` (QSL FR-092).
fn parameter(name: &str, level: &str, ty: &str) -> Value {
    node(
        name,
        "value",
        "parameter",
        ty,
        &[],
        "expression",
        aggregate(vec![
            binding("name", literal("text", "text", name)),
            binding("level", literal("integer", "integer", level)),
        ]),
    )
}

/// An integer value node.
fn integer_value(name: &str, value: &str) -> Value {
    node(
        name,
        "value",
        "literal",
        "integer",
        &["integer"],
        "expression",
        literal("integer", "integer", value),
    )
}

/// A source-declared union type node: one member per `(name, payload types)`.
fn union_type(name: &str, members: &[(&str, &[&str])]) -> Value {
    let body = aggregate(
        members
            .iter()
            .map(|(member, payload)| {
                binding(
                    member,
                    aggregate(payload.iter().map(|ty| reference(ty)).collect()),
                )
            })
            .collect(),
    );
    let payload: Vec<&str> = members
        .iter()
        .flat_map(|(_, payload)| payload.iter().copied())
        .collect();
    let mut node = node(
        name,
        "composite_type",
        "union",
        name,
        &payload,
        "declaration",
        body,
    );
    node["declaration"] = json!({"qualified_name": [name]});
    node
}

/// A union value of type `ty` constructing `member` over `payload` (value nodes).
fn union_value(name: &str, ty: &str, member: &str, payload: &[&str]) -> Value {
    node(
        name,
        "value",
        "union_value",
        ty,
        payload,
        "expression",
        aggregate(vec![binding(
            member,
            aggregate(payload.iter().map(|value| reference(value)).collect()),
        )]),
    )
}

/// One `case` arm: the member's name, its binder parameters and the arm body.
fn arm(member: &str, binders: &[&str], body: Value) -> Value {
    binding(
        member,
        aggregate(vec![
            aggregate(binders.iter().map(|binder| reference(binder)).collect()),
            body,
        ]),
    )
}

/// The `quire.op.control.case` application over `scrutinee` and `arms`.
fn case_body(scrutinee: &str, arms: Vec<Value>, result_type: &str) -> Value {
    let mut arguments = vec![reference(scrutinee)];
    arguments.extend(arms);
    json!({
        "term": "application", "operator": "case",
        "operation": {"identity": "quire.op.control.case", "laws": [], "mode": null,
                      "member": null, "leaves": []},
        "result_type": node_id(&key(result_type)), "arguments": arguments,
    })
}

fn case_node(body: Value) -> Value {
    node(
        "case",
        "expression",
        "case",
        "integer",
        &[],
        "expression",
        body,
    )
}

/// The three arms of `Shape`, in member declaration order.
fn shape_arms() -> Vec<Value> {
    vec![
        arm("Circle", &["r"], reference("r")),
        arm("Rect", &["w", "h"], reference("w")),
        arm("Empty", &[], reference("zero")),
    ]
}

/// The nodes every package below holds: the scalar types, the integer values
/// and the binder and scrutinee parameters.
fn common() -> Vec<Value> {
    vec![
        scalar("integer", "integer"),
        scalar("text", "text"),
        scalar("boolean", "boolean"),
        nfc_text(),
        integer_value("zero", "0"),
        integer_value("two", "2"),
        integer_value("three", "3"),
        parameter("s", "0", "shape"),
        parameter("r", "1", "integer"),
        parameter("w", "1", "integer"),
        parameter("h", "2", "integer"),
        parameter("t", "1", "text"),
    ]
}

/// `union Shape { Circle(Integer), Rect(Integer, Integer), Empty }`.
fn shape() -> Value {
    union_type(
        "shape",
        &[
            ("Circle", &["integer"]),
            ("Rect", &["integer", "integer"]),
            ("Empty", &[]),
        ],
    )
}

fn package_of(nodes: Vec<Value>) -> Value {
    let mut package = nominal_package(&[]);
    package["semantic_graph"]["nodes"] = Value::Array(nodes);
    settle(&mut package);
    package
}

/// The admitted `Shape` package: the union, `Shape::Rect(2, 3)` and the
/// three-arm `case` with every arm body an Integer.
fn shape_package() -> Value {
    let mut nodes = common();
    nodes.push(shape());
    nodes.push(union_value(
        "shape_rect",
        "shape",
        "Rect",
        &["two", "three"],
    ));
    nodes.push(case_node(case_body("s", shape_arms(), "integer")));
    package_of(nodes)
}

/// `shape_package` with `edit` applied to the node list before it is settled.
fn with_nodes(edit: impl FnOnce(&mut Vec<Value>)) -> Value {
    let mut nodes = common();
    nodes.push(shape());
    nodes.push(union_value(
        "shape_rect",
        "shape",
        "Rect",
        &["two", "three"],
    ));
    nodes.push(case_node(case_body("s", shape_arms(), "integer")));
    edit(&mut nodes);
    package_of(nodes)
}

/// Drops the nodes called `names`, so a defective union does not also
/// defect the nodes that name its members.
fn without(nodes: &mut Vec<Value>, names: &[&str]) {
    nodes.retain(|node| {
        !names
            .iter()
            .any(|name| node["node_id"]["digest"] == key(name).as_str())
    });
}

fn replace(nodes: &mut [Value], name: &str, with: Value) {
    let position = nodes
        .iter()
        .position(|node| node["node_id"]["digest"] == key(name).as_str())
        .unwrap_or_else(|| panic!("node {name}"));
    nodes[position] = with;
}

fn read(package: &Value) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(package),
    )
}

fn admitted(case: &str, package: &Value) -> CheckedPackageV2 {
    match read(package) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("{case}: expected admission, read {other:?}"),
    }
}

fn position_of_form(package: &Value, tag: &str, form: &str) -> usize {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_tag"] == tag && node["semantic_form"] == form)
        .expect("the node")
}

fn position_of(package: &Value, name: &str) -> usize {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_id"]["digest"] == key(name).as_str())
        .unwrap_or_else(|| panic!("node {name}"))
}

fn expect(case: &str, package: &Value, code: Code, cause: Cause, path: &str) {
    match read(package) {
        CheckedPackageV2ReadResult::Refused(refusal) => assert_eq!(
            (
                refusal.code,
                refusal.cause,
                refusal.path.as_ref().map(|path| path.as_str())
            ),
            (code, Some(cause), Some(path)),
            "{case}"
        ),
        other => panic!("{case}: expected a refusal, read {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// AC-99
// ---------------------------------------------------------------------------

/// Tracing: TC-048, FR-038-AC-99
#[trace("TC-048", "FR-038-AC-99")]
#[test]
fn tc_048_the_union_nodes_and_the_case_over_them_admit() {
    let package = shape_package();
    let reader = admitted("the Shape package", &package);
    let kinds: Vec<(&str, &str)> = reader
        .graph()
        .nodes
        .iter()
        .map(|node| (node.node_tag.as_ref(), node.semantic_form.as_ref()))
        .collect();
    for form in [
        ("composite_type", "union"),
        ("value", "union_value"),
        ("expression", "case"),
    ] {
        assert!(kinds.contains(&form), "{form:?}");
    }
}

/// Tracing: TC-048, FR-038-AC-99
#[trace("TC-048", "FR-038-AC-99")]
#[test]
fn tc_048_a_case_whose_arms_do_not_follow_the_union_refuses_at_the_case_node() {
    let arms_of = |arms: Vec<Value>| {
        with_nodes(|nodes| replace(nodes, "case", case_node(case_body("s", arms, "integer"))))
    };
    let cases: Vec<(&str, Value)> = vec![
        (
            "arms out of member order",
            arms_of(vec![
                arm("Rect", &["w", "h"], reference("w")),
                arm("Circle", &["r"], reference("r")),
                arm("Empty", &[], reference("zero")),
            ]),
        ),
        (
            "the Empty arm omitted",
            arms_of(vec![
                arm("Circle", &["r"], reference("r")),
                arm("Rect", &["w", "h"], reference("w")),
            ]),
        ),
        (
            "the Circle arm repeated",
            arms_of(vec![
                arm("Circle", &["r"], reference("r")),
                arm("Circle", &["r"], reference("r")),
                arm("Rect", &["w", "h"], reference("w")),
                arm("Empty", &[], reference("zero")),
            ]),
        ),
        (
            "a Rect binder aggregate of one reference",
            arms_of(vec![
                arm("Circle", &["r"], reference("r")),
                arm("Rect", &["w"], reference("w")),
                arm("Empty", &[], reference("zero")),
            ]),
        ),
        (
            "a Circle binder typed Text",
            arms_of(vec![
                arm("Circle", &["t"], reference("r")),
                arm("Rect", &["w", "h"], reference("w")),
                arm("Empty", &[], reference("zero")),
            ]),
        ),
        (
            "one arm body of another type than the result type",
            arms_of(vec![
                arm("Circle", &["r"], reference("t")),
                arm("Rect", &["w", "h"], reference("w")),
                arm("Empty", &[], reference("zero")),
            ]),
        ),
    ];
    for (case, package) in cases {
        let position = position_of_form(&package, "expression", "case");
        expect(
            case,
            &package,
            Code::IllTyped,
            Cause::OperatorIneligible,
            &format!("/semantic_graph/nodes/{position}"),
        );
    }
}

/// Tracing: TC-048, FR-038-AC-99
#[trace("TC-048", "FR-038-AC-99")]
#[test]
fn tc_048_a_union_type_refuses_a_repeated_member_and_a_payload_that_is_no_type() {
    let repeated = with_nodes(|nodes| {
        replace(
            nodes,
            "shape",
            union_type(
                "shape",
                &[
                    ("Circle", &["integer"]),
                    ("Circle", &["integer"]),
                    ("Empty", &[]),
                ],
            ),
        );
        without(nodes, &["case", "shape_rect"]);
    });
    let at = position_of(&repeated, "shape");
    expect(
        "Circle twice",
        &repeated,
        Code::InvalidPackage,
        Cause::DuplicateMember,
        &format!("/semantic_graph/nodes/{at}"),
    );
    let not_a_type = with_nodes(|nodes| {
        replace(
            nodes,
            "shape",
            union_type(
                "shape",
                &[
                    ("Circle", &["zero"]),
                    ("Rect", &["integer", "integer"]),
                    ("Empty", &[]),
                ],
            ),
        );
        without(nodes, &["case", "shape_rect"]);
    });
    let at = position_of(&not_a_type, "shape");
    expect(
        "a payload that is a value node",
        &not_a_type,
        Code::IllTyped,
        Cause::OperatorIneligible,
        &format!("/semantic_graph/nodes/{at}"),
    );
}

/// Tracing: TC-048, FR-038-AC-99
#[trace("TC-048", "FR-038-AC-99")]
#[test]
fn tc_048_a_union_value_must_construct_a_member_of_its_union() {
    for (case, value) in [
        (
            "a member that is not declared",
            union_value("shape_rect", "shape", "Triangle", &["two"]),
        ),
        (
            "a Rect with one payload term",
            union_value("shape_rect", "shape", "Rect", &["two"]),
        ),
        (
            "a Circle over a Text payload",
            union_value("shape_rect", "shape", "Circle", &["t"]),
        ),
        (
            "a type that is no union",
            union_value("shape_rect", "integer", "Circle", &["two"]),
        ),
    ] {
        let package = with_nodes(|nodes| replace(nodes, "shape_rect", value));
        let at = position_of(&package, "shape_rect");
        expect(
            case,
            &package,
            Code::IllTyped,
            Cause::TypeMismatch,
            &format!("/semantic_graph/nodes/{at}"),
        );
    }
    // The nullary member constructs over no payload.
    let package = with_nodes(|nodes| {
        replace(
            nodes,
            "shape_rect",
            union_value("shape_rect", "shape", "Empty", &[]),
        );
    });
    admitted("Shape::Empty", &package);
}

// ---------------------------------------------------------------------------
// AC-106
// ---------------------------------------------------------------------------

fn text_law() -> Value {
    let catalog: Value = serde_json::from_str(
        quire_verification_contracts::operation_catalog::CHECKED_OPERATION_CATALOG_V1,
    )
    .expect("the catalog is JSON");
    catalog["law_roles"]["text_profile"][0].clone()
}

fn text_leaf(path: &[&str]) -> Value {
    json!({
        "path": path,
        "laws": [{"role": "text_profile", "definition": text_law()}],
        "mode": {"kind": "text_profile", "value": "nfc"},
    })
}

/// `quire.op.structural.eq` over `left` and `right`, carrying `leaves`.
fn equality(left: &str, right: &str, leaves: Vec<Value>) -> Value {
    node(
        "equality",
        "expression",
        "binary",
        "boolean",
        &[left, right],
        "expression",
        json!({
            "term": "application", "operator": "binary",
            "operation": {"identity": "quire.op.structural.eq", "laws": [], "mode": null,
                          "member": null, "leaves": leaves},
            "result_type": node_id(&key("boolean")),
            "arguments": [reference(left), reference(right)],
        }),
    )
}

/// `union Label { Named(Text[nfc]), Tagged(Integer, Text[nfc]), Empty }` and a
/// parameter `p` of it, beside `Shape` and its scrutinee, and an equality.
fn label_package(leaves: Vec<Value>) -> Value {
    let mut nodes = common();
    nodes.push(shape());
    nodes.push(union_type(
        "label",
        &[
            ("Named", &["nfc"]),
            ("Tagged", &["integer", "nfc"]),
            ("Empty", &[]),
        ],
    ));
    nodes.push(parameter("p", "3", "label"));
    nodes.push(equality("p", "p", leaves));
    let mut package = nominal_package(&[]);
    package["semantic_graph"]["nodes"] = Value::Array(nodes);
    package["lock"]["definition_selections"]
        .as_array_mut()
        .expect("definition selections")
        .push(text_law());
    settle(&mut package);
    package
}

fn leaves_at(package: &Value, tail: &str) -> String {
    format!(
        "/semantic_graph/nodes/{}/body/operation/leaves{tail}",
        position_of_form(package, "expression", "binary")
    )
}

/// Tracing: TC-048, FR-038-AC-106
#[trace("TC-048", "FR-038-AC-106")]
#[test]
fn tc_048_equality_over_values_of_one_union_admits_and_of_two_unions_refuses() {
    // Two `Shape` values: no payload reaches `text`, so `leaves` is empty.
    let with = |nodes: Vec<Value>| package_of(nodes);
    let mut shapes = common();
    shapes.push(shape());
    shapes.push(parameter("p", "3", "shape"));
    shapes.push(equality("s", "p", vec![]));
    admitted("two Shape values", &with(shapes));

    // A `Shape` value against a value of another union.
    let mut mixed = common();
    mixed.push(shape());
    mixed.push(union_type("other", &[("Only", &["integer"])]));
    mixed.push(parameter("p", "3", "other"));
    mixed.push(equality("s", "p", vec![]));
    let package = with(mixed);
    let at = position_of_form(&package, "expression", "binary");
    expect(
        "a Shape and another union",
        &package,
        Code::IllTyped,
        Cause::OperatorIneligible,
        &format!("/semantic_graph/nodes/{at}/body/arguments/1"),
    );
}

/// Tracing: TC-048, FR-038-AC-106
#[trace("TC-048", "FR-038-AC-106")]
#[test]
fn tc_048_the_leaves_of_a_union_are_member_and_position_paths_in_member_order() {
    let named = || text_leaf(&["member:Named", "position:0"]);
    let tagged = || text_leaf(&["member:Tagged", "position:1"]);
    admitted("both leaves", &label_package(vec![named(), tagged()]));

    for (case, leaves) in [
        ("only the Named leaf", vec![named()]),
        ("only the Tagged leaf", vec![tagged()]),
        ("no leaf", vec![]),
    ] {
        let package = label_package(leaves);
        expect(
            case,
            &package,
            Code::InvalidPackage,
            Cause::OperationLawMissing,
            &leaves_at(&package, ""),
        );
    }
    for (case, leaves) in [
        ("the Named leaf again", vec![named(), tagged(), named()]),
        (
            "a leaf for Empty",
            vec![named(), tagged(), text_leaf(&["member:Empty"])],
        ),
    ] {
        let package = label_package(leaves);
        expect(
            case,
            &package,
            Code::InvalidPackage,
            Cause::OperationLawMismatch,
            &leaves_at(&package, "/2"),
        );
    }
    // The members are in declaration order, and a leaf names its payload
    // position within the member, even for a single payload.
    for (case, leaves) in [
        ("members out of order", vec![tagged(), named()]),
        (
            "a Named leaf at the wrong position",
            vec![text_leaf(&["member:Named", "position:1"]), tagged()],
        ),
        (
            "a Named leaf without its position",
            vec![text_leaf(&["member:Named"]), tagged()],
        ),
    ] {
        let package = label_package(leaves);
        expect(
            case,
            &package,
            Code::InvalidPackage,
            Cause::OperationLawMismatch,
            &leaves_at(&package, "/0/path"),
        );
    }
}

/// `union <name> { Cons(<payload>, <name>), Nil }`, one recursion group, and a
/// parameter `p` of it under an equality carrying `leaves`.
fn list_package(name: &str, payload: &str, leaves: Vec<Value>) -> Value {
    let mut list = union_type(name, &[("Cons", &[payload, name]), ("Nil", &[])]);
    list["recursion_group"] = json!("g");
    let mut nodes = common();
    // `common` holds `s`, a parameter typed at `shape`.
    nodes.push(shape());
    nodes.push(list);
    nodes.push(parameter("p", "3", name));
    nodes.push(equality("p", "p", leaves));
    let mut package = nominal_package(&[]);
    package["semantic_graph"]["nodes"] = Value::Array(nodes);
    package["lock"]["definition_selections"]
        .as_array_mut()
        .expect("definition selections")
        .push(text_law());
    settle(&mut package);
    package
}

fn recursion_leaf(path: &[&str]) -> Value {
    json!({"path": path, "laws": [], "mode": null})
}

/// A cycle through a union is a recursion leaf, never `ill_typed`: equality over
/// `IntList` admits with no leaf, and over `TextList` with its text leaf and one
/// recursion leaf; a missing recursion leaf, `recursion:1` and a recursion leaf at
/// an `IntList` reentry are refused as for a record.
///
/// Tracing: TC-048, FR-038-AC-106
#[trace("TC-048", "FR-038-AC-106")]
#[test]
fn tc_048_a_cycle_through_a_union_is_a_recursion_leaf() {
    admitted("IntList", &list_package("int_list", "integer", vec![]));

    let text = || text_leaf(&["member:Cons", "position:0"]);
    let recursion = || recursion_leaf(&["member:Cons", "position:1", "recursion:0"]);
    admitted(
        "TextList",
        &list_package("text_list", "nfc", vec![text(), recursion()]),
    );

    let package = list_package("text_list", "nfc", vec![text()]);
    expect(
        "the recursion leaf missing",
        &package,
        Code::InvalidPackage,
        Cause::OperationLawMissing,
        &leaves_at(&package, ""),
    );
    let package = list_package(
        "text_list",
        "nfc",
        vec![
            text(),
            recursion_leaf(&["member:Cons", "position:1", "recursion:1"]),
        ],
    );
    expect(
        "recursion:1",
        &package,
        Code::InvalidPackage,
        Cause::OperationLawMismatch,
        &leaves_at(&package, "/1/path"),
    );
    let package = list_package("int_list", "integer", vec![recursion()]);
    expect(
        "a recursion leaf at an IntList reentry",
        &package,
        Code::InvalidPackage,
        Cause::OperationLawMismatch,
        &leaves_at(&package, "/0/path"),
    );
}

/// `union Chain { Link(Text[nfc], Chain), End }` in one recursion group, the
/// option `Option<Chain>` outside it, and a parameter `p` of the option under an
/// equality carrying `leaves`.
fn option_chain_package(leaves: Vec<Value>) -> Value {
    let mut chain = union_type("chain", &[("Link", &["nfc", "chain"]), ("End", &[])]);
    chain["recursion_group"] = json!("g");
    let option = node(
        "chain_option",
        "composite_type",
        "option",
        "chain_option",
        &["chain"],
        "type",
        aggregate(vec![reference("chain")]),
    );
    let mut nodes = common();
    nodes.push(shape());
    nodes.push(chain);
    nodes.push(option);
    nodes.push(parameter("p", "3", "chain_option"));
    nodes.push(equality("p", "p", leaves));
    let mut package = nominal_package(&[]);
    package["semantic_graph"]["nodes"] = Value::Array(nodes);
    package["lock"]["definition_selections"]
        .as_array_mut()
        .expect("definition selections")
        .push(text_law());
    settle(&mut package);
    package
}

/// Over `Option<Chain>` the union is entered one segment in, so its recursion
/// leaf reads `recursion:1` (QSpec FR-322 "Structural leaf walk"): the leaves
/// `["inner", "member:Link", "position:0"]` and `["inner", "member:Link",
/// "position:1", "recursion:1"]` admit; the recursion leaf missing, one reading
/// `recursion:0` and the text leaf alone are refused.
///
/// Tracing: TC-048, FR-038-AC-106
#[trace("TC-048", "FR-038-AC-106")]
#[test]
fn tc_048_a_union_cycle_under_an_option_enters_at_recursion_one() {
    let text = || text_leaf(&["inner", "member:Link", "position:0"]);
    let recursion = |depth: &str| recursion_leaf(&["inner", "member:Link", "position:1", depth]);
    admitted(
        "Option<Chain>",
        &option_chain_package(vec![text(), recursion("recursion:1")]),
    );

    let package = option_chain_package(vec![text()]);
    expect(
        "the recursion leaf missing",
        &package,
        Code::InvalidPackage,
        Cause::OperationLawMissing,
        &leaves_at(&package, ""),
    );
    let package = option_chain_package(vec![text(), recursion("recursion:0")]);
    expect(
        "recursion:0",
        &package,
        Code::InvalidPackage,
        Cause::OperationLawMismatch,
        &leaves_at(&package, "/1/path"),
    );
}

/// A union type or union value body that is not its closed shape (an empty
/// union, a union value with two bindings, a binding whose value is no
/// `aggregate`) is `invalid_semantic_graph` at the node's `body`.
///
/// Tracing: TC-048, FR-038-AC-99
#[trace("TC-048", "FR-038-AC-99")]
#[test]
fn tc_048_a_malformed_union_body_refuses_at_the_node_body() {
    let two_bindings = node(
        "shape_rect",
        "value",
        "union_value",
        "shape",
        &["two", "three"],
        "expression",
        aggregate(vec![
            binding(
                "Rect",
                aggregate(vec![reference("two"), reference("three")]),
            ),
            binding("Circle", aggregate(vec![reference("two")])),
        ]),
    );
    let mut not_an_aggregate = node(
        "shape",
        "composite_type",
        "union",
        "shape",
        &["integer"],
        "declaration",
        aggregate(vec![binding("Circle", reference("integer"))]),
    );
    not_an_aggregate["declaration"] = json!({"qualified_name": ["shape"]});
    for (name, malformed) in [
        ("shape", union_type("shape", &[])),
        ("shape", not_an_aggregate),
        ("shape_rect", two_bindings),
    ] {
        let package = with_nodes(|nodes| replace(nodes, name, malformed));
        let at = position_of(&package, name);
        match read(&package) {
            CheckedPackageV2ReadResult::Refused(refusal) => assert_eq!(
                (
                    refusal.code,
                    refusal.cause,
                    refusal.path.as_ref().map(|path| path.as_str())
                ),
                (
                    Code::InvalidSemanticGraph,
                    None,
                    Some(format!("/semantic_graph/nodes/{at}/body").as_str())
                ),
                "{name}"
            ),
            other => panic!("{name}: expected a refusal, read {other:?}"),
        }
    }
}

/// The package with one diagnostics entry whose `details` is `term`.
fn with_details(mut package: Value, term: Value) -> Value {
    package["diagnostics"]["entries"] = json!([{
        "stage": "type_checking", "code": "ill_typed", "cause_tag": "invalid-value",
        "details": [term], "loci": [],
    }]);
    package
}

/// A `details` term that references a union or union value node is an ordinary
/// reference and admits; one that references an `expression`/`case` node is
/// refused `ill_typed`/`operator-ineligible` at the entry
/// (`/diagnostics/entries/{e}/details/{d}`), with no node key as locus.
///
/// Tracing: TC-048, FR-038-AC-100
#[trace("TC-048", "FR-038-AC-100")]
#[test]
fn tc_048_a_details_reference_to_a_union_node_admits_and_to_a_case_node_refuses() {
    let package = shape_package();
    admitted(
        "a union reference",
        &with_details(package.clone(), reference("shape")),
    );
    admitted(
        "a union value reference",
        &with_details(package.clone(), reference("shape_rect")),
    );
    let case = position_of_form(&package, "expression", "case");
    let case_key = package["semantic_graph"]["nodes"][case]["node_id"]["digest"]
        .as_str()
        .expect("digest")
        .to_owned();
    let refused = with_details(package, reference_to_key(&case_key));
    match read(&refused) {
        CheckedPackageV2ReadResult::Refused(refusal) => assert_eq!(
            (
                refusal.code,
                refusal.cause,
                refusal.path.as_ref().map(|path| path.as_str()),
                refusal.locus
            ),
            (
                Code::IllTyped,
                Some(Cause::OperatorIneligible),
                Some("/diagnostics/entries/0/details/0"),
                None
            )
        ),
        other => panic!("a case node reference: expected a refusal, read {other:?}"),
    }
}

fn reference_to_key(digest: &str) -> Value {
    json!({"term": "reference", "target": node_id(digest)})
}

// ---------------------------------------------------------------------------
// AC-101
// ---------------------------------------------------------------------------

/// The identity the lowerer derives for `node`, from the admitted wire alone
/// (as for every node).
fn derived_identity(wire: &Value, lowered: &quire_contract_ir::CompleteContractNodeV2) -> String {
    let mut projection = wire.clone();
    projection
        .as_object_mut()
        .expect("node")
        .remove("occurrences");
    sha256_hex(&canonical(&json!({
        "version": "quire.contract-ir.lowered-node/v1",
        "node": projection,
        "dependencies": lowered.dependencies,
        "bounds": lowered.bounds,
        "claims": lowered.claims,
    })))
}

/// An admitted node lowers, under a profile that supports its tag, as data: its
/// lowered body is the admitted body and its identity is derived from it; a
/// profile lacking the tag returns `unsupported` naming it.
fn lowers_as_data(case: &str, package: &Value, node: &str, tag: CheckedNodeTag) {
    let reader = admitted(case, package);
    let wire = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|candidate| candidate["node_id"]["digest"] == node)
        .expect("the node")
        .clone();
    let requested = [typed_node_id(node)];
    let every_tag = [
        CheckedNodeTag::ScalarType,
        CheckedNodeTag::CompositeType,
        CheckedNodeTag::BoundedDomain,
        CheckedNodeTag::Value,
        CheckedNodeTag::Expression,
        CheckedNodeTag::Temporal,
    ];
    let supported = CompleteLoweringProfileV2 {
        supported_tags: every_tag.into_iter().collect(),
        require_bounds: false,
        work_limit: u64::MAX,
    };
    let lowered = reader.lower(&requested, &supported);
    let CompleteLoweringRecordV2::Lowered { node: lowered } = &lowered.records[0] else {
        panic!(
            "{case}: expected a lowered record, got {:?}",
            lowered.records[0]
        );
    };
    assert_eq!(
        serde_json::to_value(&lowered.node).expect("node"),
        wire,
        "{case}"
    );
    assert_eq!(lowered.node.body, wire["body"], "{case}");
    assert_eq!(lowered.ir_id.domain.as_ref(), CONTRACT_IR_SEMANTIC_DOMAIN);
    assert_eq!(
        lowered.ir_id.digest.as_ref(),
        derived_identity(&wire, lowered),
        "{case}"
    );

    let mut without = supported;
    without.supported_tags.remove(&tag);
    let refused = reader.lower(&requested, &without);
    assert!(
        matches!(
            &refused.records[0],
            CompleteLoweringRecordV2::Unsupported { node_tag, .. } if *node_tag == tag
        ),
        "{case}: {:?}",
        refused.records[0]
    );
}

/// Tracing: TC-048, FR-038-AC-101
#[trace("TC-048", "FR-038-AC-101")]
#[test]
fn tc_048_an_admitted_case_node_lowers_as_data() {
    let package = shape_package();
    let case = position_of_form(&package, "expression", "case");
    let digest = package["semantic_graph"]["nodes"][case]["node_id"]["digest"]
        .as_str()
        .expect("digest")
        .to_owned();
    lowers_as_data(
        "the case node",
        &package,
        &digest,
        CheckedNodeTag::Expression,
    );
    // The union value is data too.
    lowers_as_data(
        "the union value",
        &package,
        &key("shape_rect"),
        CheckedNodeTag::Value,
    );
}

/// Tracing: TC-048, FR-038-AC-101
#[trace("TC-048", "FR-038-AC-101")]
#[test]
fn tc_048_an_admitted_formula_node_lowers_as_data() {
    let package = v2_all_families();
    lowers_as_data(
        "the formula node",
        &package,
        &family_key("a2a2"),
        CheckedNodeTag::Temporal,
    );
}
