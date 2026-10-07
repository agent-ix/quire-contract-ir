// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-226: the V2 reader re-derives the node key of the ten anonymous node
//! shapes `MemberType::node_key` derives (FR-038 "Anonymous structural node
//! bodies and keys", IR-627) from each node's own body, and refuses a node
//! whose key differs as `invalid_package`/`stale-node-key` at its `node_id`,
//! wherever the node is reached from.
//!
//! Every package is built here, node by node, over this crate's own
//! `v2_all_families()` fixture. Every key is computed by the test from
//! QSL FR-092's preimage (`structural_key`) and every tampered package gets
//! its `identity_projection` patched and its `package_id` recomputed through
//! `canonical`/`sha256_hex` in the test, never through the reader. The QSpec
//! conformance counterpart (AC-133) is read by `make conformance-qspec`.

use crate::support::checked_package::{
    application_node_key, bounds_body, canonical, evidence_for, family_key, integer_range_key,
    model_owner, node_id, nominal_package, over_body, owned_structural_node, rebuild_source_map,
    refresh_identity, refusal_at, rename_node, settle, sha256_hex, structural_key, typed_node_id,
    v2_all_families, v2_nominal, ExpectedNodeId, ExpectedRefusal, BOOLEAN_KEY, INTEGER_KEY,
};
use ix_trace_rs::trace;
use quire_contract_ir::{
    read_checked_package, CheckedPackageDispatchResult, CheckedPackageEvidence,
    CheckedPackageReadLimits, CheckedPackageRefusal, CheckedPackageRefusalCause as Cause,
    CheckedPackageRefusalCode as Code, CheckedPackageV2, CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};

const I128_MIN: &str = "-170141183460469231731687303715884105728";
const I128_MAX: &str = "170141183460469231731687303715884105727";
/// One below the `i128` maximum: a float conversion rounds it to the maximum.
const I128_MAX_LESS_ONE: &str = "170141183460469231731687303715884105726";
/// One above the `i128` maximum: no `i128` holds it.
const I128_MAX_PLUS_ONE: &str = "170141183460469231731687303715884105728";
const I128_MIN_LESS_ONE: &str = "-170141183460469231731687303715884105729";

/// An edit to one node of a package.
type Edit = Box<dyn Fn(&mut Value)>;

fn read(package: &Value, evidence: &CheckedPackageEvidence) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        evidence,
    )
}

fn admitted(what: &str, package: &Value) {
    match read(package, &evidence_for(package)) {
        CheckedPackageV2ReadResult::Admitted(_) => {}
        other => panic!("{what}: expected admission, read {other:?}"),
    }
}

fn refusal_of(
    what: &str,
    package: &Value,
    evidence: &CheckedPackageEvidence,
) -> CheckedPackageRefusal {
    match read(package, evidence) {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        other => panic!("{what}: expected a refusal, read {other:?}"),
    }
}

/// The position of the node keyed `key`.
fn at(package: &Value, key: &str) -> usize {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_id"]["digest"] == key)
        .unwrap_or_else(|| panic!("node {key}"))
}

/// The refusal the reader must give a node it cannot re-derive: the cause it
/// already uses for a key that does not match its node, at the node.
fn stale_at(package: &Value, key: &str) -> ExpectedRefusal {
    refusal_at(
        Code::InvalidPackage,
        &format!("/semantic_graph/nodes/{}/node_id", at(package, key)),
        Some(Cause::StaleNodeKey),
        key,
    )
}

fn assert_stale(what: &str, package: &Value, key: &str) {
    assert_eq!(
        refusal_of(what, package, &evidence_for(package)),
        stale_at(package, key),
        "{what}"
    );
}

fn assert_stale_absent(what: &str, package: &Value, key: &str) {
    let mut expected = stale_at(package, key);
    expected.expected_node_id = ExpectedNodeId::Absent;
    assert_eq!(
        refusal_of(what, package, &evidence_for(package)),
        expected,
        "{what}"
    );
}

/// A node with the `generated` occurrence the fixture's nodes carry.
fn node(
    key: &str,
    tag: &str,
    form: &str,
    semantic_type: &str,
    dependencies: &[&str],
    body: Value,
) -> Value {
    json!({
        "node_id": node_id(key),
        "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": tag,
        "semantic_form": form,
        "semantic_type": node_id(semantic_type),
        "dependencies": dependencies.iter().map(|key| node_id(key)).collect::<Vec<_>>(),
        "occurrences": [{"role": "generated", "ordinal": 0}],
        "body": body,
    })
}

/// An anonymous `integer_range` node over `[min, max]` under its derived key.
fn range(min: &str, max: &str) -> (String, Value) {
    let key = integer_range_key(min, max);
    let node = node(
        &key,
        "bounded_domain",
        "integer_range",
        INTEGER_KEY,
        &[INTEGER_KEY],
        bounds_body(min, max),
    );
    (key, node)
}

/// An anonymous self-typed `composite_type` of `form` over `element`.
fn collection(form: &str, element: &str) -> (String, Value) {
    let body = over_body(element);
    let key = structural_key("composite_type", form, None, &body);
    let node = node(&key, "composite_type", form, &key, &[element], body);
    (key, node)
}

/// A `collection_bounds` node `[min, max]` over the collection `over`.
fn bounded_collection(over: &str, min: &str, max: &str) -> (String, Value) {
    let key = structural_key(
        "bounded_domain",
        "collection_bounds",
        Some(over),
        &bounds_body(min, max),
    );
    let node = node(
        &key,
        "bounded_domain",
        "collection_bounds",
        over,
        &[over],
        bounds_body(min, max),
    );
    (key, node)
}

/// `value`/`parameter` typed at `ty`, with the closed parameter body.
fn parameter(name: &str, level: &str, ty: &str) -> (String, Value) {
    let text = family_key("a1a1");
    let body = json!({"term": "aggregate", "members": [
        {"term": "binding", "name": "name", "value": {
            "term": "literal", "type": node_id(&text), "value_kind": "text", "value": name}},
        {"term": "binding", "name": "level", "value": {
            "term": "literal", "type": node_id(INTEGER_KEY), "value_kind": "integer",
            "value": level}},
    ]});
    let key = structural_key("value", "parameter", Some(ty), &body);
    let mut parameter = node(&key, "value", "parameter", ty, &[], body);
    parameter["occurrences"] = json!([{"role": "expression", "ordinal": 0}]);
    (key, parameter)
}

/// `x + 1` over the parameter `x`: an `integer` `add` application, a scalar
/// operand that reaches the type of `x` without any member read.
fn add_one(x: &str) -> (String, Value) {
    let body = json!({
        "term": "application",
        "operator": "binary",
        "operation": {"identity": "quire.op.integer.add", "laws": [], "mode": null,
            "member": null, "leaves": []},
        "result_type": node_id(INTEGER_KEY),
        "arguments": [
            {"term": "reference", "target": node_id(x)},
            {"term": "literal", "type": node_id(INTEGER_KEY), "value_kind": "integer", "value": "1"},
        ],
    });
    let key = application_node_key("expression", "binary", INTEGER_KEY, &body);
    (
        key.clone(),
        node(&key, "expression", "binary", INTEGER_KEY, &[x], body),
    )
}

/// `package` with `added` appended and its source map and identity rebuilt.
fn with_nodes(mut package: Value, added: Vec<Value>) -> Value {
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend(added);
    rebuild_source_map(&mut package);
    refresh_identity(&mut package);
    package
}

/// `package` with the node keyed `key` edited, its `node_id` kept, and its
/// identity patched: the tamper the stage exists to catch.
fn tampered(package: &Value, key: &str, edit: impl FnOnce(&mut Value)) -> Value {
    let mut package = package.clone();
    let position = at(&package, key);
    edit(&mut package["semantic_graph"]["nodes"][position]);
    refresh_identity(&mut package);
    package
}

/// `package` with the node keyed `key` edited and then keyed again by the
/// derivation of its edited body, as an attacker who also recomputes the key
/// would: only the closed-body rule can refuse it. Returns the package and
/// the new key.
fn rekeyed(package: &Value, key: &str, edit: impl FnOnce(&mut Value)) -> (Value, String) {
    let mut package = package.clone();
    let position = at(&package, key);
    let node = &mut package["semantic_graph"]["nodes"][position];
    edit(node);
    let semantic_type = (node["node_tag"] == "bounded_domain").then(|| {
        node["semantic_type"]["digest"]
            .as_str()
            .expect("type")
            .to_owned()
    });
    let fresh = structural_key(
        node["node_tag"].as_str().expect("tag"),
        node["semantic_form"].as_str().expect("form"),
        semantic_type.as_deref(),
        &node["body"],
    );
    rename_node(&mut package, key, &fresh);
    refresh_identity(&mut package);
    (package, fresh)
}

/// The base every case below builds on: the all-families fixture plus the
/// anonymous nodes the cases tamper, re-point and read.
struct Base {
    package: Value,
    /// `Int[0, 1000]` (`cccc`, the fixture's own range) and `Int[0, 10]`.
    wide: String,
    narrow: String,
    zero: String,
    extremes: String,
    /// `Sequence<Integer>`, its `[0, 3]` bounds, `Sequence<Int[0, 10]>`,
    /// `Set<Int[0, 1000]>`, `Reference<Account>` and `Option<Int[0, 1000]>`.
    sequence: String,
    sequence_bounds: String,
    narrow_sequence: String,
    set: String,
    reference: String,
    option: String,
    /// A parameter typed at `Int[0, 1000]` and `x + 1` over it.
    parameter: String,
    add: String,
}

fn base() -> Base {
    let wide = family_key("cccc");
    let (narrow, narrow_node) = range("0", "10");
    let (zero, zero_node) = range("0", "0");
    let (extremes, extremes_node) = range(I128_MIN, I128_MAX);
    let (sequence, sequence_node) = collection("sequence", INTEGER_KEY);
    let (sequence_bounds, sequence_bounds_node) = bounded_collection(&sequence, "0", "3");
    let (narrow_sequence, narrow_sequence_node) = collection("sequence", &narrow);
    let (set, set_node) = collection("set", &wide);
    let (reference, reference_node) = collection("reference", &family_key("1515"));
    let (option, option_node) = collection("option", &wide);
    let (parameter, parameter_node) = parameter("x", "1", &wide);
    let (add, add_node) = add_one(&parameter);
    let package = with_nodes(
        v2_all_families(),
        vec![
            narrow_node,
            zero_node,
            extremes_node,
            sequence_node,
            sequence_bounds_node,
            narrow_sequence_node,
            set_node,
            reference_node,
            option_node,
            parameter_node,
            add_node,
        ],
    );
    admitted("the base package", &package);
    Base {
        package,
        wide,
        narrow,
        zero,
        extremes,
        sequence,
        sequence_bounds,
        narrow_sequence,
        set,
        reference,
        option,
        parameter,
        add,
    }
}

/// Owner-free structural forms beyond the original ten use their QSL FR-092
/// preimage too. A changed parameter body and a changed text type body must
/// fail at their own retained keys after package identity is refreshed.
/// Trace: FR-038-AC-131
#[trace("FR-038-AC-131")]
#[test]
fn tc_226_owner_free_parameter_and_text_keys_cover_their_bodies() {
    let base = base();
    let parameter = tampered(&base.package, &base.parameter, |node| {
        node["body"]["members"][1]["value"]["value"] = json!("2");
    });
    assert_stale("parameter level", &parameter, &base.parameter);

    let text = family_key("a1a1");
    let altered = tampered(&base.package, &text, |node| {
        node["body"]["members"] = json!([{"term": "reference", "target": node_id(INTEGER_KEY)}]);
    });
    assert_stale("text type body", &altered, &text);
}

/// Each included owner-free form is checked from its own QSL preimage, not
/// only the ten closed shapes handled by the specialized path.
/// Trace: FR-038-AC-131
#[trace("FR-038-AC-131")]
#[test]
fn tc_226_each_owner_free_form_rejects_a_body_mutation_at_its_key() {
    let base = base();
    let empty = json!({"term": "aggregate", "members": []});
    for (tag, form, semantic_type, body) in [
        (
            "bounded_domain",
            "rational_range",
            INTEGER_KEY,
            empty.clone(),
        ),
        (
            "bounded_domain",
            "decimal_range",
            INTEGER_KEY,
            empty.clone(),
        ),
        (
            "bounded_domain",
            "float_rounding",
            INTEGER_KEY,
            empty.clone(),
        ),
        ("bounded_domain", "text_bounds", INTEGER_KEY, empty.clone()),
        (
            "bounded_domain",
            "model_population",
            INTEGER_KEY,
            empty.clone(),
        ),
        ("scalar_type", "rational", "", empty.clone()),
        ("scalar_type", "decimal", "", empty.clone()),
        ("scalar_type", "float32", "", empty.clone()),
        ("scalar_type", "float64", "", empty.clone()),
        ("composite_type", "record", "", empty.clone()),
        ("composite_type", "tuple", "", empty.clone()),
        ("composite_type", "alias", "", empty.clone()),
        (
            "value",
            "literal",
            INTEGER_KEY,
            json!({
                "term": "literal", "type": node_id(INTEGER_KEY),
                "value_kind": "integer", "value": "1"
            }),
        ),
        ("value", "collection_value", INTEGER_KEY, empty.clone()),
        ("value", "record_value", INTEGER_KEY, empty.clone()),
        ("value", "tuple_value", INTEGER_KEY, empty.clone()),
        ("value", "option_value", INTEGER_KEY, empty.clone()),
    ] {
        let key = structural_key(
            tag,
            form,
            (!semantic_type.is_empty()).then_some(semantic_type),
            &body,
        );
        let ty = if semantic_type.is_empty() {
            key.as_str()
        } else {
            semantic_type
        };
        let added = node(&key, tag, form, ty, &[], body);
        let package = with_nodes(base.package.clone(), vec![added]);
        admitted(&format!("{tag}/{form}"), &package);
        let altered = tampered(&package, &key, |node| {
            node["body"] = json!({"term": "aggregate", "members": [
                {"term": "reference", "target": node_id(INTEGER_KEY)}
            ]});
        });
        assert_stale(&format!("{tag}/{form}"), &altered, &key);
    }
}

/// The forms with stricter body grammars also reach key re-derivation with
/// valid mutations of their own body values.
/// Trace: FR-038-AC-131
#[trace("FR-038-AC-131")]
#[test]
fn tc_226_compound_unit_and_union_forms_cover_their_bodies() {
    let nominal = v2_nominal();
    let nominal_nodes = nominal["semantic_graph"]["nodes"]
        .as_array()
        .expect("nominal nodes")
        .clone();
    let unit = nominal_nodes
        .iter()
        .find(|node| node["node_tag"] == "scalar_type" && node["semantic_form"] == "unit")
        .expect("root unit")["node_id"]["digest"]
        .as_str()
        .expect("unit key")
        .to_owned();
    let compound_body = |exponent: &str| {
        json!({"term": "aggregate", "members": [{
            "term": "aggregate", "members": [
                {"term": "binding", "name": "unit",
                 "value": {"term": "reference", "target": node_id(&unit)}},
                {"term": "binding", "name": "exponent",
                 "value": {"term": "literal", "type": node_id(INTEGER_KEY),
                           "value_kind": "integer", "value": exponent}},
            ]
        }]})
    };
    let body = compound_body("2");
    let key = structural_key("scalar_type", "compound_unit", None, &body);
    let mut compound = node(&key, "scalar_type", "compound_unit", &key, &[&unit], body);
    compound["occurrences"] = json!([{"role": "type", "ordinal": 0}]);
    let package = with_nodes(
        nominal,
        vec![
            node(
                INTEGER_KEY,
                "scalar_type",
                "integer",
                INTEGER_KEY,
                &[],
                json!({"term": "aggregate", "members": []}),
            ),
            compound,
        ],
    );
    admitted("compound unit", &package);
    let altered = tampered(&package, &key, |node| node["body"] = compound_body("3"));
    assert_stale("compound unit exponent", &altered, &key);

    let union_body = |name: &str| {
        json!({"term": "aggregate", "members": [{
            "term": "binding", "name": name,
            "value": {"term": "aggregate", "members": []}
        }]})
    };
    let body = union_body("One");
    let union = structural_key("composite_type", "union", None, &body);
    let union_node = node(&union, "composite_type", "union", &union, &[], body);
    let value_body = |name: &str| {
        json!({"term": "aggregate", "members": [{
            "term": "binding", "name": name,
            "value": {"term": "aggregate", "members": []}
        }]})
    };
    let value_key = structural_key("value", "union_value", Some(&union), &value_body("One"));
    let mut value_node = node(
        &value_key,
        "value",
        "union_value",
        &union,
        &[],
        value_body("One"),
    );
    value_node["occurrences"] = json!([{"role": "expression", "ordinal": 0}]);
    let package = with_nodes(base().package, vec![union_node, value_node]);
    admitted("union and union value", &package);
    let altered = tampered(&package, &union, |node| node["body"] = union_body("Two"));
    assert_stale("union member", &altered, &union);
    let altered = tampered(&package, &value_key, |node| {
        node["body"] = value_body("Two")
    });
    assert_stale("union value member", &altered, &value_key);
}

/// Edits the `min` or `max` literal's `value` of a bounds body.
fn set_bound(node: &mut Value, name: &str, value: &str) {
    for member in node["body"]["members"].as_array_mut().expect("members") {
        if member["name"] == name {
            member["value"]["value"] = json!(value);
        }
    }
}

const ORDERS: &str = "acme/orders";
const ORDER: &str = "ix://acme/orders/Order";
const COUNT: &str = "ix://acme/orders/Count";

/// The domain package document: an `Order` with the field `count` of the
/// integer value type `Count`, whose bounds are `[0, maximum]`.
fn orders_document(maximum: i64) -> Value {
    json!({
        "contractVersion": "2.0.0",
        "package": {"identity": ORDERS, "version": "1.0.0"},
        "constructs": [
            {"kind": {"module": ORDERS, "name": "entity"},
             "construct": {"meaning": "quire.meaning.model.object-type/v1"}},
            {"kind": {"module": ORDERS, "name": "count"},
             "construct": {"meaning": "quire.meaning.model.value-type/v1"}},
        ],
        "types": [
            {
                "identity": ORDER, "displayName": "Order",
                "kind": {"module": ORDERS, "name": "entity"},
                "roles": [], "constraints": [], "extensions": [], "unknownPolicy": "reject",
                "supertypes": [],
                "fields": [{
                    "identity": format!("{ORDER}/count"), "name": "count", "typeRef": COUNT,
                    "presence": "required", "nullable": false, "defaultKind": "none",
                    "multiplicity": {"lower": 1, "upper": 1, "ordered": false, "unique": true},
                }],
                "operations": [],
            },
            {
                "identity": COUNT, "displayName": "Count",
                "kind": {"module": ORDERS, "name": "count"},
                "roles": [], "extensions": [], "unknownPolicy": "reject", "scalar": "integer",
                "constraints": [
                    {"keyword": "min", "operands": {"value": 0}},
                    {"keyword": "max", "operands": {"value": maximum}},
                ],
            },
        ],
    })
}

/// The model declaration node key of `Order` owned by the selected package.
fn order_key() -> String {
    sha256_hex(&canonical(&json!({
        "version": "quire.structural-node/v1", "node_tag": "model",
        "semantic_form": "object_type", "semantic_type": null, "declaration": null,
        "recursion": null, "body": {"term": "aggregate", "members": []},
        "owner": {"kind": "model", "identity": ORDERS, "node": ORDER},
    })))
}

/// A package whose lock selects `document` and whose one model-owned field
/// read, `Order.count` through `quire.op.record.project`, has the result type
/// `result`. Returns it with the evidence that supplies exactly `supplied`.
fn member_read(
    document: &Value,
    result: &str,
    supplied: &[&Value],
) -> (Value, CheckedPackageEvidence) {
    let mut package = nominal_package(&[]);
    package["lock"]["model_selections"] = json!([{
        "identity": ORDERS, "digest_domain": "sha256-jcs",
        "digest": sha256_hex(&canonical(document)),
    }]);
    let order = order_key();
    let empty = json!({"term": "aggregate", "members": []});
    let body = json!({
        "term": "application",
        "operator": "query",
        "operation": {
            "identity": "quire.op.record.project", "laws": [], "mode": null,
            "member": {"kind": "field", "declaration": node_id(&order), "name": "count"},
            "leaves": [],
        },
        "result_type": node_id(result),
        "arguments": [{"term": "reference", "target": node_id(&order)}],
    });
    let read_key = application_node_key("expression", "query", result, &body);
    let constraints = &document["types"][1]["constraints"];
    let bound = |position: usize| {
        constraints[position]["operands"]["value"]
            .as_str()
            .map(str::to_owned)
            .or_else(|| {
                constraints[position]["operands"]["value"]
                    .as_i64()
                    .map(|value| value.to_string())
            })
            .expect("integer bound")
    };
    let (range_key, range_node) = range(&bound(0), &bound(1));
    let field = &document["types"][0]["fields"][0];
    let mut type_nodes = vec![range_node];
    if field["presence"] == "optional" {
        let (key, wrapper) = collection("option", &range_key);
        assert_eq!(key, result);
        type_nodes.push(wrapper);
    } else if field["multiplicity"]["lower"] == 0 && field["multiplicity"]["upper"].is_null() {
        let form = if field["multiplicity"]["ordered"] == true {
            "sequence"
        } else {
            "set"
        };
        let (key, wrapper) = collection(form, &range_key);
        assert_eq!(key, result);
        type_nodes.push(wrapper);
    } else {
        assert_eq!(range_key, result);
    }
    let mut nodes = vec![
        node(
            INTEGER_KEY,
            "scalar_type",
            "integer",
            INTEGER_KEY,
            &[],
            empty.clone(),
        ),
        owned_structural_node(
            node(&order, "model", "object_type", &order, &[], empty),
            model_owner(ORDERS, ORDER),
        ),
        node(&read_key, "expression", "query", result, &[&order], body),
    ];
    nodes.append(&mut type_nodes);
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend(nodes);
    rebuild_source_map(&mut package);
    refresh_identity(&mut package);
    let mut evidence = evidence_for(&package);
    for document in supplied {
        evidence
            .insert_domain_package_document(sha256_hex(&canonical(document)), canonical(document));
    }
    (package, evidence)
}

/// The rows AC-123 tampers a `[0, 1000]` range by, as `(row, edit)`.
fn tamper_rows() -> Vec<(&'static str, Edit)> {
    vec![
        ("max changed to 10", Box::new(|n| set_bound(n, "max", "10"))),
        (
            "max changed to 5000",
            Box::new(|n| set_bound(n, "max", "5000")),
        ),
        ("min changed to 1", Box::new(|n| set_bound(n, "min", "1"))),
        (
            "min and max swapped",
            Box::new(|n| {
                set_bound(n, "min", "1000");
                set_bound(n, "max", "0");
            }),
        ),
    ]
}

/// Tracing: TC-226
/// ACs: FR-038-AC-123
#[trace("TC-226", "FR-038-AC-123")]
#[test]
fn tc_226_a_tampered_range_is_refused_at_its_node_wherever_it_is_reached_from() {
    let base = base();
    let unmutated = base.package.clone();
    admitted("the unmutated package", &unmutated);

    // `Int[0, 1000]` as the type of the parameter a scalar operand `x + 1`
    // reads, with no member read anywhere: the fixture's own `cccc`.
    for (row, edit) in tamper_rows() {
        let package = tampered(&unmutated, &base.wide, edit);
        assert_stale(
            &format!("a scalar operand's type: {row}"),
            &package,
            &base.wide,
        );
    }
    // The operand reads the tampered node: the application is never the node
    // named, and the refusal is never `ill_typed`.
    let package = tampered(&unmutated, &base.wide, |n| set_bound(n, "max", "10"));
    let refusal = refusal_of("the operand", &package, &evidence_for(&package));
    assert_eq!(refusal.locus, Some(typed_node_id(&base.wide)));
    assert_ne!(refusal.locus, Some(typed_node_id(&base.add)));
    assert_ne!(refusal.code, Code::IllTyped);

    // `Int[0, 0]` that no other node names: the rows hold for a node nothing
    // reads, so the stage is not tied to a referrer.
    for (row, edit) in [
        (
            "max changed to 1",
            (|n: &mut Value| set_bound(n, "max", "1")) as fn(&mut Value),
        ),
        ("min changed to -1", |n| set_bound(n, "min", "-1")),
    ] {
        let package = tampered(&unmutated, &base.zero, edit);
        assert_stale(
            &format!("an unreferenced node: {row}"),
            &package,
            &base.zero,
        );
    }

    // A state field's body target names the range through the fixture's
    // `state`/`snapshot` node. The state step never gets to read a tamper.
    let mut state = unmutated.clone();
    let snapshot = at(&state, &family_key("3030"));
    let field_body = json!({"term": "aggregate", "members": [{
        "term": "binding", "name": "count",
        "value": {"term": "reference", "target": node_id(&base.wide)}
    }]});
    state["semantic_graph"]["nodes"][snapshot]["body"] = field_body.clone();
    state["semantic_graph"]["nodes"][snapshot]["dependencies"] = json!([node_id(&base.wide)]);
    let state_key = structural_key("state", "snapshot", Some(BOOLEAN_KEY), &field_body);
    rename_node(&mut state, &family_key("3030"), &state_key);
    refresh_identity(&mut state);
    admitted("the state node naming the range", &state);
    for (row, edit) in tamper_rows() {
        let package = tampered(&state, &base.wide, edit);
        assert_stale(&format!("a state node's body: {row}"), &package, &base.wide);
    }

    // The node a model-owned field read names: `Order.count` declared
    // `Int[0, 1000]` in the selected document.
    let document = orders_document(1000);
    let wide = integer_range_key("0", "1000");
    let (member_read, evidence) = member_read(&document, &wide, &[&document]);
    match read(&member_read, &evidence) {
        CheckedPackageV2ReadResult::Admitted(_) => {}
        other => panic!("the unmutated member read: expected admission, read {other:?}"),
    }
    for (row, edit) in tamper_rows() {
        let package = tampered(&member_read, &wide, edit);
        assert_eq!(
            refusal_of(row, &package, &evidence),
            stale_at(&package, &wide),
            "a model-owned field read: {row}"
        );
    }
}

/// Tracing: TC-226
/// ACs: FR-038-AC-124
#[trace("TC-226", "FR-038-AC-124")]
#[test]
fn tc_226_a_body_off_the_closed_form_has_no_derivable_key() {
    let base = base();
    let package = &base.package;

    // The closed bodies admit, an `integer_range` `min` of `"-5"` included.
    let (_, negative) = range("-5", "1000");
    admitted(
        "an integer_range with min -5",
        &with_nodes(package.clone(), vec![negative]),
    );

    // Each body below is keyed again by the derivation of its own bytes, as
    // an attacker would: only the closed-body rule can refuse it.
    let literal = |ty: &str, kind: &str, value: &str| json!({"term": "literal", "type": node_id(ty), "value_kind": kind, "value": value});
    let set_max_literal =
        |literal: Value| move |n: &mut Value| n["body"]["members"][1]["value"] = literal.clone();
    let mut bounds_rows: Vec<(String, Edit)> = vec![
        (
            "max absent".into(),
            Box::new(|n| {
                n["body"]["members"].as_array_mut().expect("members").pop();
            }),
        ),
        (
            "a third binding".into(),
            Box::new(|n| {
                let third = n["body"]["members"][1].clone();
                n["body"]["members"]
                    .as_array_mut()
                    .expect("members")
                    .push(third);
            }),
        ),
        (
            "max before min".into(),
            Box::new(|n| {
                n["body"]["members"]
                    .as_array_mut()
                    .expect("members")
                    .reverse();
            }),
        ),
        (
            "max typed at Boolean".into(),
            Box::new(set_max_literal(literal(BOOLEAN_KEY, "integer", "5"))),
        ),
        (
            "max typed at another node".into(),
            Box::new(set_max_literal(literal(
                &family_key("a1a1"),
                "integer",
                "5",
            ))),
        ),
        (
            "max a text literal".into(),
            Box::new(set_max_literal(literal(&family_key("a1a1"), "text", "5"))),
        ),
    ];
    for spelling in ["01", "+5", "-0", "-01", ""] {
        bounds_rows.push((
            format!("max {spelling:?}"),
            Box::new(move |n| set_bound(n, "max", spelling)),
        ));
    }
    for (row, edit) in &bounds_rows {
        for (form, key) in [
            ("integer_range", base.zero.clone()),
            ("collection_bounds", base.sequence_bounds.clone()),
        ] {
            let (package, fresh) = rekeyed(package, &key, edit);
            assert_stale_absent(&format!("{form}: {row}"), &package, &fresh);
        }
    }
    // A collection bound is a count: a negative `min` has no derivable key.
    let (negative_count, fresh) = rekeyed(package, &base.sequence_bounds, |n| {
        set_bound(n, "min", "-1");
    });
    assert_stale_absent("collection_bounds: min -1", &negative_count, &fresh);

    // A `scalar_type`/`integer` node with a non-empty body, and a `reference`,
    // `option` or collection node with no member, two members or a member that
    // is not a `reference`.
    let integer = tampered(package, INTEGER_KEY, |n| {
        n["body"] = over_body(&family_key("a1a1"));
    });
    assert_stale_absent("integer with a body", &integer, INTEGER_KEY);
    for (what, key) in [
        ("reference", base.reference.clone()),
        ("option", base.option.clone()),
        ("set", base.set.clone()),
    ] {
        let rows: [(&str, Edit); 3] = [
            ("no member", Box::new(|n| n["body"]["members"] = json!([]))),
            (
                "two members",
                Box::new(|n| {
                    let second = n["body"]["members"][0].clone();
                    n["body"]["members"]
                        .as_array_mut()
                        .expect("members")
                        .push(second);
                }),
            ),
            (
                "a member that is no reference",
                Box::new(|n| {
                    n["body"]["members"] = json!([{
                        "term": "binding", "name": "x",
                        "value": {"term": "aggregate", "members": []},
                    }]);
                }),
            ),
        ];
        for (row, edit) in rows {
            let (package, fresh) = rekeyed(package, &key, edit);
            assert_stale_absent(&format!("{what}: {row}"), &package, &fresh);
        }
    }
}

/// Tracing: TC-226
/// ACs: FR-038-AC-125
#[trace("TC-226", "FR-038-AC-125")]
#[test]
fn tc_226_a_node_is_its_own_type_and_the_key_covers_semantic_type_and_literal_type() {
    let base = base();
    let package = &base.package;
    let retype = |key: &str, ty: &str| tampered(package, key, |n| n["semantic_type"] = node_id(ty));
    // A scalar or composite node of a derived shape typed at another node
    // refuses although its key is unchanged.
    assert_stale_absent(
        "boolean typed at Integer",
        &retype(BOOLEAN_KEY, INTEGER_KEY),
        BOOLEAN_KEY,
    );
    assert_stale_absent(
        "integer typed at Boolean",
        &retype(INTEGER_KEY, BOOLEAN_KEY),
        INTEGER_KEY,
    );
    assert_stale_absent(
        "set typed at its element",
        &retype(&base.set, &base.wide),
        &base.set,
    );
    assert_stale_absent(
        "option typed at Boolean",
        &retype(&family_key("bbbb"), BOOLEAN_KEY),
        &family_key("bbbb"),
    );
    // A `collection_bounds` re-pointed at a collection of another element
    // range, and an `integer_range` re-pointed at a node other than `Integer`.
    assert_stale(
        "collection_bounds re-pointed at another collection",
        &retype(&base.sequence_bounds, &base.narrow_sequence),
        &base.sequence_bounds,
    );
    assert_stale_absent(
        "integer_range re-pointed at Boolean",
        &retype(&base.zero, BOOLEAN_KEY),
        &base.zero,
    );
    // Rekeying after the semantic-type edit must not turn an invalid form
    // into an admitted one. Both target nodes are genuinely keyed.
    for (what, key) in [
        ("integer_range typed at Boolean", &base.zero),
        ("collection_bounds typed at Boolean", &base.sequence_bounds),
    ] {
        let (package, fresh) = rekeyed(package, key, |n| {
            n["semantic_type"] = node_id(BOOLEAN_KEY);
        });
        assert_stale_absent(what, &package, &fresh);
    }
    // The literal `type` of `min` and `max` re-pointed at a genuinely keyed
    // node of another type.
    for key in [&base.sequence_bounds, &base.zero] {
        let package = tampered(package, key, |n| {
            for member in n["body"]["members"].as_array_mut().expect("members") {
                member["value"]["type"] = node_id(BOOLEAN_KEY);
            }
        });
        assert_stale_absent("literal type re-pointed", &package, key);
    }
}

/// Tracing: TC-226
/// ACs: FR-038-AC-126
#[trace("TC-226", "FR-038-AC-126")]
#[test]
fn tc_226_a_body_reference_re_pointed_at_a_narrower_range_is_refused_at_its_node() {
    let base = base();
    let package = &base.package;
    // Option, collections and `Reference` over `Int[0, 1000]`, their key kept,
    // their body `reference` re-pointed at the genuinely keyed `Int[0, 10]`.
    for (what, key) in [
        ("Option", base.option.clone()),
        ("Set", base.set.clone()),
        ("Sequence", base.sequence.clone()),
        ("Reference", base.reference.clone()),
    ] {
        let package = tampered(package, &key, |n| {
            n["body"]["members"][0]["target"] = node_id(&base.narrow);
        });
        let refusal = refusal_of(what, &package, &evidence_for(&package));
        assert_eq!(refusal, stale_at(&package, &key), "{what}");
        assert_ne!(refusal.code, Code::IllTyped, "{what}");
    }
    // The parameter's type node holds the body `max` 10: refused at the type
    // node, never as an `ill_typed` application that reads it.
    let package = tampered(package, &base.wide, |n| set_bound(n, "max", "10"));
    let refusal = refusal_of("the parameter's type", &package, &evidence_for(&package));
    assert_eq!(refusal, stale_at(&package, &base.wide));
    assert_ne!(refusal.locus, Some(typed_node_id(&base.parameter)));
    assert_ne!(refusal.locus, Some(typed_node_id(&base.add)));

    // The selected document declares the field's wrapped type, so the
    // member-read operation reaches each wrapper through a real declaration.
    for (what, form) in [
        ("optional field", "option"),
        ("set field", "set"),
        ("sequence field", "sequence"),
    ] {
        let mut document = orders_document(1000);
        if form == "option" {
            document["types"][0]["fields"][0]["presence"] = json!("optional");
        } else {
            document["types"][0]["fields"][0]["multiplicity"] = json!({
                "lower": 0,
                "ordered": form == "sequence", "unique": form == "set"
            });
        }
        let key = structural_key("composite_type", form, None, &over_body(&base.wide));
        let (package, evidence) = member_read(&document, &key, &[&document]);
        match read(&package, &evidence) {
            CheckedPackageV2ReadResult::Admitted(_) => {}
            other => panic!("{what}: expected admission, read {other:?}"),
        }
        let altered = tampered(&package, &key, |node| {
            node["body"]["members"][0]["target"] = node_id(&base.narrow);
        });
        // The genuinely keyed narrow target is supplied as a node, so the
        // mismatch is the declared wrapper's retained key, not a dangling ref.
        let (_, narrow_node) = range("0", "10");
        let altered = with_nodes(altered, vec![narrow_node]);
        assert_eq!(
            refusal_of(what, &altered, &evidence),
            stale_at(&altered, &key),
            "{what}"
        );
    }

    // A selected operation declares a bounded parameter and return. Its
    // dispatch call reads that parameter through the model member table.
    let mut document = orders_document(1000);
    document["types"][0]["operations"] = json!([{
        "identity": format!("{ORDER}/total"),
        "params": [{
            "typeRef": COUNT,
            "multiplicity": {"lower": 1, "upper": 1,
                             "ordered": false, "unique": true}
        }],
        "returns": {
            "typeRef": COUNT,
            "multiplicity": {"lower": 1, "upper": 1,
                             "ordered": false, "unique": true}
        }
    }]);
    let (mut package, evidence) = member_read(&document, &base.wide, &[&document]);
    let order = order_key();
    let (reference, reference_node) = collection("reference", &order);
    let (receiver, receiver_node) = parameter("receiver", "0", &reference);
    let (argument, argument_node) = parameter("argument", "1", &base.wide);
    let body = json!({
        "term": "application", "operator": "call",
        "operation": {
            "identity": "quire.op.model.dispatch_call", "laws": [], "mode": null,
            "member": {"kind": "operation", "declaration": node_id(&order), "name": "total"},
            "leaves": []
        },
        "result_type": node_id(&base.wide),
        "arguments": [
            {"term": "reference", "target": node_id(&receiver)},
            {"term": "reference", "target": node_id(&argument)}
        ]
    });
    let call = application_node_key("expression", "call", &base.wide, &body);
    let mut deps = vec![order.as_str(), receiver.as_str(), argument.as_str()];
    deps.sort_unstable();
    package = with_nodes(
        package,
        vec![
            node(
                &family_key("a1a1"),
                "scalar_type",
                "text",
                &family_key("a1a1"),
                &[],
                json!({"term": "aggregate", "members": []}),
            ),
            reference_node,
            receiver_node,
            argument_node,
            node(&call, "expression", "call", &base.wide, &deps, body),
        ],
    );
    match read(&package, &evidence) {
        CheckedPackageV2ReadResult::Admitted(_) => {}
        other => panic!("declared operation parameter: expected admission, read {other:?}"),
    }
    let altered = tampered(&package, &base.wide, |node| set_bound(node, "max", "10"));
    assert_eq!(
        refusal_of("declared operation parameter range", &altered, &evidence),
        stale_at(&altered, &base.wide)
    );
    let altered = tampered(&package, &reference, |node| {
        node["body"]["members"][0]["target"] = node_id(&base.narrow);
    });
    let (_, narrow_node) = range("0", "10");
    let altered = with_nodes(altered, vec![narrow_node]);
    assert_eq!(
        refusal_of("declared operation receiver reference", &altered, &evidence),
        stale_at(&altered, &reference)
    );
}

/// `package` with the body of the application keyed `key` edited and the
/// application keyed again by its own preimage.
fn edited_application(
    package: &Value,
    key: &str,
    edit: impl FnOnce(&mut Value),
) -> (Value, String) {
    let mut package = package.clone();
    let position = at(&package, key);
    let node = &mut package["semantic_graph"]["nodes"][position];
    edit(&mut node["body"]);
    let result = node["semantic_type"]["digest"]
        .as_str()
        .expect("type")
        .to_owned();
    let fresh = application_node_key(
        node["node_tag"].as_str().expect("tag"),
        node["semantic_form"].as_str().expect("form"),
        &result,
        &node["body"],
    );
    rename_node(&mut package, key, &fresh);
    refresh_identity(&mut package);
    (package, fresh)
}

/// Tracing: TC-226
/// ACs: FR-038-AC-127
#[trace("TC-226", "FR-038-AC-127")]
#[test]
fn tc_226_the_derived_key_stage_runs_after_the_application_keys_and_before_the_operations() {
    let base = base();
    let package = &base.package;
    let wide_tamper =
        |package: &Value| tampered(package, &base.wide, |n| set_bound(n, "max", "10"));

    // A graph-shape defect (a parameter whose body is not the closed
    // parameter body) is reported ahead of a tampered node.
    let graph_defect = tampered(package, &base.parameter, |n| {
        n["body"] = json!({"term": "aggregate", "members": []});
    });
    let graph_refusal = refusal_of(
        "the graph defect",
        &graph_defect,
        &evidence_for(&graph_defect),
    );
    assert_eq!(graph_refusal.code, Code::InvalidSemanticGraph);
    assert_eq!(graph_refusal.cause, None);
    let both = wide_tamper(&graph_defect);
    assert_eq!(
        refusal_of("graph defect and tamper", &both, &evidence_for(&both)),
        graph_refusal
    );

    // A stale application key is reported ahead of a tampered node, at the
    // application.
    let stale_application = tampered(package, &base.add, |n| {
        n["body"]["arguments"][1]["value"] = json!("2");
    });
    assert_stale("the stale application", &stale_application, &base.add);
    let both = wide_tamper(&stale_application);
    assert_eq!(
        refusal_of("application key and tamper", &both, &evidence_for(&both)),
        stale_at(&both, &base.add)
    );

    // A tampered node and an `ill_typed` application that reads it: the
    // tampered node, at itself, and never `ill_typed`.
    let (ill_typed, _) = edited_application(package, &base.add, |body| {
        body["arguments"].as_array_mut().expect("arguments").pop();
    });
    let defect = refusal_of(
        "the ill-typed application",
        &ill_typed,
        &evidence_for(&ill_typed),
    );
    assert_eq!(defect.code, Code::IllTyped, "{defect:?}");
    let both = wide_tamper(&ill_typed);
    assert_eq!(
        refusal_of(
            "ill-typed application and tamper",
            &both,
            &evidence_for(&both)
        ),
        stale_at(&both, &base.wide)
    );

    // Two tampered nodes: the one whose own `node_id` digest is lower, even
    // when the other sits earlier in the graph. Choose a pair whose graph
    // order and digest order disagree, so that a stage that visited nodes in
    // graph order would report the other node.
    let candidates = [&base.wide, &base.narrow, &base.zero, &base.extremes];
    let mut by_position: Vec<&String> = candidates.to_vec();
    by_position.sort_by_key(|key| at(package, key));
    let (earlier, later) = by_position
        .iter()
        .enumerate()
        .flat_map(|(at, earlier)| {
            by_position[at + 1..]
                .iter()
                .map(move |later| (*earlier, *later))
        })
        .find(|(earlier, later)| later < earlier)
        .expect("a pair whose lower digest sits later in the graph");
    assert!(at(package, earlier) < at(package, later) && later < earlier);
    let edit = |n: &mut Value| set_bound(n, "max", "7");
    let both = tampered(&tampered(package, earlier, edit), later, edit);
    assert_eq!(
        refusal_of("two tampered nodes", &both, &evidence_for(&both)),
        stale_at(&both, later),
        "the lower digest, not the earlier position"
    );
    assert_stale(
        "only the earlier node",
        &tampered(package, earlier, edit),
        earlier,
    );
}

/// Tracing: TC-226
/// ACs: FR-038-AC-128
#[trace("TC-226", "FR-038-AC-128")]
#[test]
fn tc_226_bounds_are_compared_exactly_at_the_i128_extremes() {
    let base = base();
    let package = &base.package;
    // `Int[0, 0]` and `Int[i128::MIN, i128::MAX]` admit (the base holds both).
    admitted("the extremes", package);
    for (row, key, edit) in [
        ("zero: max one above", &base.zero, ("max", "1")),
        ("zero: min one below", &base.zero, ("min", "-1")),
        (
            "max one below the i128 maximum",
            &base.extremes,
            ("max", I128_MAX_LESS_ONE),
        ),
        (
            "max one above the i128 maximum",
            &base.extremes,
            ("max", I128_MAX_PLUS_ONE),
        ),
        (
            "min one below the i128 minimum",
            &base.extremes,
            ("min", I128_MIN_LESS_ONE),
        ),
        (
            "min one above the i128 minimum",
            &base.extremes,
            ("min", "-170141183460469231731687303715884105727"),
        ),
    ] {
        let package = tampered(package, key, |n| set_bound(n, edit.0, edit.1));
        assert_stale(row, &package, key);
    }

    // The same exact comparison is exercised through a selected document's
    // declared field, including bounds outside JSON's i64 integer range.
    for (what, min, max, rows) in [
        ("declared zero", "0", "0", vec![("max", "1"), ("min", "-1")]),
        (
            "declared i128 extremes",
            I128_MIN,
            I128_MAX,
            vec![
                ("max", I128_MAX_LESS_ONE),
                ("max", I128_MAX_PLUS_ONE),
                ("min", I128_MIN_LESS_ONE),
                ("min", "-170141183460469231731687303715884105727"),
            ],
        ),
    ] {
        let mut document = orders_document(0);
        document["types"][1]["constraints"][0]["operands"]["value"] = json!(min);
        document["types"][1]["constraints"][1]["operands"]["value"] = json!(max);
        let key = integer_range_key(min, max);
        let (package, evidence) = member_read(&document, &key, &[&document]);
        match read(&package, &evidence) {
            CheckedPackageV2ReadResult::Admitted(_) => {}
            other => panic!("{what}: expected admission, read {other:?}"),
        }
        for (member, changed) in rows {
            let altered = tampered(&package, &key, |node| set_bound(node, member, changed));
            assert_eq!(
                refusal_of(what, &altered, &evidence),
                stale_at(&altered, &key),
                "{what}: {member} changed to {changed}"
            );
        }
    }
}

/// Tracing: TC-226
/// ACs: FR-038-AC-129
#[trace("TC-226", "FR-038-AC-129")]
#[test]
fn tc_226_a_re_pointed_model_selection_is_decided_by_the_evidence_the_caller_holds() {
    let original = orders_document(1000);
    let narrow_document = orders_document(10);
    let narrow = integer_range_key("0", "10");
    // The selection re-pointed at the narrower document, its node re-keyed to
    // `Int[0, 10]`, its `package_id` recomputed.
    let (package, with_both) =
        member_read(&narrow_document, &narrow, &[&original, &narrow_document]);
    let (_, only_original) = member_read(&narrow_document, &narrow, &[&original]);

    let refusal = refusal_of("the original document only", &package, &only_original);
    assert_eq!(
        (
            refusal.code,
            refusal.cause,
            refusal.path.as_ref().map(|p| p.as_str())
        ),
        (
            Code::MissingImport,
            Some(Cause::MissingSelection),
            Some("/lock/model_selections/0/digest"),
        )
    );
    // The reader admits a package against whatever evidence it is given: with
    // the re-pointed document supplied the package admits. This records the
    // limit of the trust root; which document is authoritative is the
    // caller's decision.
    match read(&package, &with_both) {
        CheckedPackageV2ReadResult::Admitted(_) => {}
        other => panic!("the re-pointed document supplied: expected admission, read {other:?}"),
    }
}

/// Tracing: TC-226
/// ACs: FR-038-AC-134
#[trace("TC-226", "FR-038-AC-134")]
#[test]
fn tc_226_the_in_repo_fixtures_carry_derived_keys_and_closed_bodies() {
    admitted("v2_all_families", &v2_all_families());
    admitted("v2_nominal", &v2_nominal());
    admitted(
        "positive_operation_identities",
        &crate::support::checked_package::positive_operation_identities(),
    );

    // The placeholder `aaaa` key restored on the boolean node. Every node
    // whose key covers the boolean's key is keyed again over the placeholder
    // (the `option`, and the application nodes through `settle`), so the
    // boolean is the one node whose key is not the derived one.
    let restored = "0".repeat(64);
    let mut placeholder = v2_all_families();
    let position = at(&placeholder, BOOLEAN_KEY);
    rename_node(&mut placeholder, BOOLEAN_KEY, &restored);
    let option = family_key("bbbb");
    let option_over_placeholder =
        structural_key("composite_type", "option", None, &over_body(&restored));
    rename_node(&mut placeholder, &option, &option_over_placeholder);
    settle(&mut placeholder);
    assert_eq!(
        refusal_of(
            "a restored placeholder",
            &placeholder,
            &evidence_for(&placeholder)
        ),
        refusal_at(
            Code::InvalidPackage,
            &format!("/semantic_graph/nodes/{position}/node_id"),
            Some(Cause::StaleNodeKey),
            &restored,
        )
    );

    // The `bbbb` option left as the fixture once held it: typed at the
    // boolean with an empty aggregate body.
    let option = family_key("bbbb");
    let base = v2_all_families();
    let unmigrated_body = tampered(&base, &option, |n| {
        n["body"] = json!({"term": "aggregate", "members": []});
    });
    assert_stale_absent("the body left unmigrated", &unmigrated_body, &option);
    let unmigrated_type = tampered(&base, &option, |n| {
        n["semantic_type"] = node_id(BOOLEAN_KEY);
    });
    assert_stale_absent(
        "the semantic type left unmigrated",
        &unmigrated_type,
        &option,
    );
    // And keyed again by the preimage of that old shape: no key derives.
    let (old_shape, fresh) = rekeyed(&base, &option, |n| {
        n["semantic_type"] = node_id(BOOLEAN_KEY);
        n["body"] = json!({"term": "aggregate", "members": []});
    });
    assert_stale_absent("the old shape keyed again", &old_shape, &fresh);
}

/// The scalar keys the fixtures record as literals are the derived ones.
///
/// Tracing: TC-226
/// ACs: FR-038-AC-134
#[trace("TC-226", "FR-038-AC-134")]
#[test]
fn tc_226_the_recorded_scalar_keys_are_the_derived_ones() {
    let empty = json!({"term": "aggregate", "members": []});
    assert_eq!(
        structural_key("scalar_type", "boolean", None, &empty),
        BOOLEAN_KEY
    );
    assert_eq!(
        structural_key("scalar_type", "integer", None, &empty),
        INTEGER_KEY
    );
}

/// The recursive packages `record List { next?: List; }` (an `Option`
/// self-reference, one group) and `record Tree { kids: Sequence<Tree>[0, 3]; }`
/// (a `Sequence` self-reference and its `collection_bounds`, one group), added
/// to the base package. Every in-group node carries a placeholder key: QSL keys
/// such a node under a group digest this reader cannot compute (IR-627-Q4), so
/// the stage skips it.
struct Recursive {
    package: Value,
    /// The `collection_bounds` node of `Tree`, and the `Sequence<Tree>` node.
    bounds: String,
    sequence: String,
    tree: String,
}

fn recursive() -> Recursive {
    let base = base();
    let key = |name: &str| sha256_hex(format!("recursive {name}").as_bytes());
    let (list, option) = (key("list"), key("option"));
    let (tree, sequence) = (key("tree"), key("sequence"));
    // The bounds carry the key their own body derives, so that dropping the
    // group label below leaves the key stage nothing to refuse and the cycle
    // rule is what refuses the package.
    let bounds = structural_key(
        "bounded_domain",
        "collection_bounds",
        Some(&sequence),
        &bounds_body("0", "3"),
    );
    let record = |key: &str, field: &str, target: &str| {
        let body = json!({"term": "aggregate", "members": [{
            "term": "binding", "name": field,
            "value": {"term": "reference", "target": node_id(target)},
        }]});
        node(key, "composite_type", "record", key, &[target], body)
    };
    let in_group = |mut node: Value, group: &str| {
        node["recursion_group"] = json!(group);
        node
    };
    let nodes = vec![
        in_group(record(&list, "next", &option), "list"),
        in_group(
            node(
                &option,
                "composite_type",
                "option",
                &option,
                &[&list],
                over_body(&list),
            ),
            "list",
        ),
        in_group(record(&tree, "kids", &bounds), "tree"),
        in_group(
            node(
                &sequence,
                "composite_type",
                "sequence",
                &sequence,
                &[&tree],
                over_body(&tree),
            ),
            "tree",
        ),
        in_group(
            node(
                &bounds,
                "bounded_domain",
                "collection_bounds",
                &sequence,
                &[&sequence],
                bounds_body("0", "3"),
            ),
            "tree",
        ),
    ];
    let package = with_nodes(base.package, nodes);
    admitted("the recursive packages", &package);
    Recursive {
        package,
        bounds,
        sequence,
        tree,
    }
}

/// A node of the ten shapes that carries a `recursion_group` is skipped, not
/// refused: the recursive `List` (an `Option`) and `Tree` (a `Sequence` and its
/// bounds) packages admit. The skip is exactly the grouped nodes: a derived
/// node outside every group, in the same package, still refuses; and the
/// `Tree` bounds with the group removed is a cycle outside a declared group, so
/// the cycle rule refuses it (FR-038-AC-18) and it is never admitted.
///
/// Tracing: TC-226
#[trace("TC-226")]
#[test]
fn tc_226_a_derived_shape_node_in_a_recursion_group_is_skipped_and_one_outside_is_not() {
    let recursive = recursive();
    let package = &recursive.package;

    let base = base();
    let outside = tampered(package, &base.wide, |n| set_bound(n, "max", "10"));
    assert_stale("a derived node outside every group", &outside, &base.wide);

    let ungrouped = tampered(package, &recursive.bounds, |n| {
        n.as_object_mut().expect("node").remove("recursion_group");
    });
    // With the label gone from one member the component is no longer exactly
    // the labelled set, so the others are verified as ungrouped too (their
    // keys are the QSL group keys, not ungrouped ones) and the package is
    // never admitted: the key stage refuses before the cycle rule would.
    let refusal = refusal_of("the group removed", &ungrouped, &evidence_for(&ungrouped));
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::InvalidPackage, Some(Cause::StaleNodeKey))
    );
}

/// The stated soundness limit, recorded as a test: the reader does not verify
/// the key of an in-group derived-shape node, so each single mutation of the
/// `Tree` group below, its `node_id` kept and the identity recomputed, is
/// admitted. When the in-group re-derivation lands (gated on the declared
/// member's `SourceOwner`, IR-627-Q1 and Q4) these rows move to refusals in the
/// same change. A consumer must not treat a range, element type or bound read
/// through such a node as verified.
///
/// Tracing: TC-226
#[trace("TC-226")]
#[test]
fn tc_226_an_in_group_tamper_is_admitted_under_the_skip_the_recorded_limit() {
    let recursive = recursive();
    let rows: [(&str, String, Edit); 2] = [
        (
            "bounds max changed to 5",
            recursive.bounds.clone(),
            Box::new(|n| set_bound(n, "max", "5")),
        ),
        (
            "bounds min changed to 1",
            recursive.bounds.clone(),
            Box::new(|n| set_bound(n, "min", "1")),
        ),
    ];
    for (row, key, edit) in rows {
        let package = tampered(&recursive.package, &key, edit);
        admitted(row, &package);
    }
    // Re-pointing the sequence's body reference at the bounds leaves a names
    // cycle of the sequence and the bounds alone, every member of it carrying
    // the one label: skipped too, and admitted.
    let bounds = recursive.bounds.clone();
    let package = tampered(&recursive.package, &recursive.sequence, move |n| {
        n["body"]["members"][0]["target"] = node_id(&bounds);
    });
    admitted("the sequence re-pointed at the bounds", &package);
    let _ = &recursive.tree;
}

/// A second recorded limit: the stage does not cross-check a node's wire tag
/// and form against the type its key was minted for. A node keyed
/// `Int[0, 1000]` retagged `collection_bounds` (its body unchanged), typed at a
/// collection whose body names it, with both labelled, forges a names cycle of
/// two skippable shapes and is skipped, so it is admitted. Whether a
/// cross-check is added is open in the amendment IR #298; when it lands this
/// row moves to a refusal.
///
/// Tracing: TC-226
#[trace("TC-226")]
#[test]
fn tc_226_a_range_retagged_collection_bounds_on_a_forged_cycle_is_admitted_the_recorded_limit() {
    let base = base();
    // `Int[0, 0]`, which no application reads, so that only the key stage
    // could have refused it.
    let package = tampered(&base.package, &base.zero, |n| {
        n["semantic_form"] = json!("collection_bounds");
        n["semantic_type"] = node_id(&base.set);
        n["recursion_group"] = json!("forged");
        set_bound(n, "max", "10");
    });
    let package = tampered(&package, &base.set, |n| {
        n["recursion_group"] = json!("forged");
        n["body"]["members"][0]["target"] = node_id(&base.zero);
    });
    admitted("the retagged range", &package);
}

/// A `recursion_group` label added to a node that is on no cycle is not a
/// group: a tamperer who labels the `Int[0, 1000]` node `solo` and changes its
/// `max` must still be refused `stale-node-key` at the node, under any design
/// of the in-group skip. (The existing admission of a lone label on an acyclic
/// node, `tc_048_...`, holds for nodes the stage does not verify.)
///
/// Tracing: TC-226
#[trace("TC-226")]
#[test]
fn tc_226_a_lone_group_label_does_not_exempt_a_tampered_range() {
    let base = base();
    for (what, key) in [
        ("integer_range", base.wide.clone()),
        ("set", base.set.clone()),
    ] {
        let package = tampered(&base.package, &key, |n| {
            n["recursion_group"] = json!("solo");
            if n["semantic_form"] == "integer_range" {
                set_bound(n, "max", "10");
            } else {
                n["body"]["members"][0]["target"] = node_id(&base.narrow);
            }
        });
        assert_stale(&format!("a labelled acyclic {what}"), &package, &key);
    }
}

/// The skip reads the names graph, not `dependencies`: a collection that
/// lists itself as a dependency, or two that list each other, and carries a
/// group label is on no names cycle and is verified. And an `integer_range`
/// is verified whatever label it carries, inside a package whose real groups
/// are still skipped.
///
/// Tracing: TC-226
#[trace("TC-226")]
#[test]
fn tc_226_only_a_names_cycle_that_is_exactly_the_group_exempts_a_node() {
    let base = base();
    let repoint = |n: &mut Value| n["body"]["members"][0]["target"] = node_id(&base.narrow);
    // A dependency on itself.
    let package = tampered(&base.package, &base.set, |n| {
        n["recursion_group"] = json!("forged");
        n["dependencies"] = json!([node_id(&base.set)]);
        repoint(n);
    });
    assert_stale("a self-dependency", &package, &base.set);
    // Two collections naming each other through `dependencies` only.
    let package = tampered(&base.package, &base.set, |n| {
        n["recursion_group"] = json!("forged");
        n["dependencies"] = json!([node_id(&base.option)]);
        repoint(n);
    });
    let package = tampered(&package, &base.option, |n| {
        n["recursion_group"] = json!("forged");
        n["dependencies"] = json!([node_id(&base.set)]);
    });
    let refusal = refusal_of("a dependency cycle", &package, &evidence_for(&package));
    assert_eq!(refusal.cause, Some(Cause::StaleNodeKey), "{refusal:?}");

    // A label on an `integer_range` beside a real group: still verified.
    let recursive = recursive();
    let package = tampered(&recursive.package, &base.wide, |n| {
        n["recursion_group"] = json!("other");
        set_bound(n, "max", "10");
    });
    assert_stale("a labelled integer_range", &package, &base.wide);
}

/// The stated limit, recorded as a test: a tamperer who re-points a
/// collection's body `reference` at a `collection_bounds` node that names the
/// collection at its `semantic_type`, and labels both, forges a names cycle
/// whose component is exactly the labelled set, so the stage skips both and
/// the package is admitted with the collection's element type and the count
/// unverified. They rest on the package producer until the in-group
/// re-derivation lands (IR-630). A range read from an `integer_range` node is
/// never reached this way.
///
/// Tracing: TC-226
#[trace("TC-226")]
#[test]
fn tc_226_a_forged_names_cycle_through_collection_bounds_is_admitted_the_recorded_limit() {
    let base = base();
    let (bounds, bounds_node) = bounded_collection(&base.set, "0", "3");
    let package = with_nodes(base.package.clone(), vec![bounds_node]);
    admitted("the collection and its bounds", &package);
    let forged = tampered(&package, &base.set, |n| {
        n["recursion_group"] = json!("forged");
        n["body"]["members"][0]["target"] = node_id(&bounds);
    });
    let forged = tampered(&forged, &bounds, |n| {
        n["recursion_group"] = json!("forged");
        set_bound(n, "max", "5000");
    });
    admitted("the forged group", &forged);
    // The same forgery over an `integer_range` is refused.
    let range = tampered(&package, &base.wide, |n| {
        n["recursion_group"] = json!("forged");
        set_bound(n, "max", "10");
    });
    assert_stale("a labelled integer_range", &range, &base.wide);
}

/// Trace: FR-038-AC-183, FR-038-AC-185, TC-048, TC-226
#[trace("FR-038-AC-183", "FR-038-AC-185", "TC-048", "TC-226")]
#[test]
fn tc_048_derived_expected_keys_rekey_unreferenced_nodes_in_both_branches() {
    let base = base();
    let (parameter_key, parameter_node) = parameter("free", "1", &base.wide);
    let package = with_nodes(base.package, vec![parameter_node]);
    admitted("unreferenced parameter", &package);

    for (key, edit) in [
        (
            base.zero.as_str(),
            (|node: &mut Value| set_bound(node, "max", "1")) as fn(&mut Value),
        ),
        (parameter_key.as_str(), |node: &mut Value| {
            node["body"]["members"][1]["value"]["value"] = json!("2");
        }),
    ] {
        let stale = tampered(&package, key, edit);
        let refusal = refusal_of("derived key", &stale, &evidence_for(&stale));
        assert_eq!(refusal, stale_at(&stale, key));
        let expected = refusal.expected_node_id().expect("derived key");
        assert_eq!(expected, &typed_node_id(&expected.digest));
        assert_ne!(expected.digest.as_ref(), key);
        let mut rekeyed = stale.clone();
        rename_node(&mut rekeyed, key, &expected.digest);
        rebuild_source_map(&mut rekeyed);
        refresh_identity(&mut rekeyed);
        admitted("fresh package with returned key", &rekeyed);
    }

    let changed_once = tampered(&package, &base.zero, |node| set_bound(node, "max", "1"));
    let changed_twice = tampered(&package, &base.zero, |node| set_bound(node, "max", "2"));
    let first = refusal_of("first bound", &changed_once, &evidence_for(&changed_once));
    let second = refusal_of(
        "second bound",
        &changed_twice,
        &evidence_for(&changed_twice),
    );
    assert_eq!(first.code, second.code);
    assert_eq!(first.path, second.path);
    assert_eq!(first.cause, second.cause);
    assert_eq!(first.locus, second.locus);
    assert_ne!(first.expected_node_id(), second.expected_node_id());
    assert_ne!(first, second, "the private key participates in Eq");
    assert_eq!(first.clone().expected_node_id(), first.expected_node_id());
    let mut rekeyed_second = changed_twice;
    rename_node(
        &mut rekeyed_second,
        &base.zero,
        &second
            .expected_node_id()
            .expect("second derived key")
            .digest,
    );
    rebuild_source_map(&mut rekeyed_second);
    refresh_identity(&mut rekeyed_second);
    admitted("second bound with returned key", &rekeyed_second);
}

/// Trace: FR-038-AC-184, FR-038-AC-185, TC-048
#[trace("FR-038-AC-184", "FR-038-AC-185", "TC-048")]
#[test]
fn tc_048_application_expected_key_survives_reader_and_dispatch() {
    let base = base();
    admitted("base application", &base.package);
    let stale = tampered(&base.package, &base.add, |node| {
        node["body"]["arguments"][1]["value"] = json!("2");
    });
    let refusal = refusal_of("application key", &stale, &evidence_for(&stale));
    assert_eq!(refusal, stale_at(&stale, &base.add));
    let expected = refusal.expected_node_id().expect("application key");
    assert_ne!(expected.digest.as_ref(), base.add);

    let dispatched = read_checked_package(
        &canonical(&stale),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&stale),
    );
    let CheckedPackageDispatchResult::Refused(forwarded) = dispatched else {
        panic!("dispatch did not forward the refusal");
    };
    assert_eq!(forwarded.code, Code::InvalidPackage);
    assert_eq!(forwarded.cause, Some(Cause::StaleNodeKey));
    assert_eq!(forwarded.locus, Some(typed_node_id(&base.add)));
    assert_eq!(forwarded.expected_node_id(), Some(expected));

    let mut rekeyed = stale.clone();
    rename_node(&mut rekeyed, &base.add, &expected.digest);
    rebuild_source_map(&mut rekeyed);
    refresh_identity(&mut rekeyed);
    admitted("application with returned key", &rekeyed);

    let another = tampered(&base.package, &base.add, |node| {
        node["body"]["arguments"][1]["value"] = json!("3");
    });
    let another_refusal = refusal_of("second application key", &another, &evidence_for(&another));
    assert_eq!(another_refusal, stale_at(&another, &base.add));
    assert_ne!(
        another_refusal.expected_node_id(),
        refusal.expected_node_id()
    );
    let mut rekeyed_another = another;
    rename_node(
        &mut rekeyed_another,
        &base.add,
        &another_refusal
            .expected_node_id()
            .expect("second application key")
            .digest,
    );
    rebuild_source_map(&mut rekeyed_another);
    refresh_identity(&mut rekeyed_another);
    admitted("second application with returned key", &rekeyed_another);
}

/// Trace: FR-038-AC-183, FR-038-AC-185, TC-048
#[trace("FR-038-AC-183", "FR-038-AC-185", "TC-048")]
#[test]
fn tc_048_nonderivable_stale_keys_retain_absence() {
    let base = base();
    let (option_key, option_node) = collection("option", &base.zero);
    let package = with_nodes(base.package, vec![option_node]);
    admitted("unreferenced option", &package);

    let fresh = family_key("eded");
    let mut stale_self_type = package.clone();
    rename_node(&mut stale_self_type, &option_key, &fresh);
    let position = at(&stale_self_type, &fresh);
    stale_self_type["semantic_graph"]["nodes"][position]["semantic_type"] = node_id(&option_key);
    rebuild_source_map(&mut stale_self_type);
    refresh_identity(&mut stale_self_type);
    let refusal = refusal_of(
        "stale self type",
        &stale_self_type,
        &evidence_for(&stale_self_type),
    );
    assert_eq!(refusal.code, Code::InvalidPackage);
    assert_eq!(refusal.cause, Some(Cause::StaleNodeKey));
    assert_eq!(refusal.locus, Some(typed_node_id(&fresh)));
    assert_eq!(refusal.expected_node_id(), None);

    let noncollection = tampered(&package, &base.sequence_bounds, |node| {
        node["semantic_type"] = node_id(BOOLEAN_KEY);
    });
    let refusal = refusal_of(
        "noncollection bounds",
        &noncollection,
        &evidence_for(&noncollection),
    );
    assert_eq!(refusal.code, Code::InvalidPackage);
    assert_eq!(refusal.cause, Some(Cause::StaleNodeKey));
    assert_eq!(refusal.locus, Some(typed_node_id(&base.sequence_bounds)));
    assert_eq!(refusal.expected_node_id(), None);

    let malformed = tampered(&package, &base.zero, |node| {
        node["body"] = json!({"term":"aggregate", "members":[]});
    });
    let refusal = refusal_of(
        "malformed closed shape",
        &malformed,
        &evidence_for(&malformed),
    );
    assert_eq!(refusal.code, Code::InvalidPackage);
    assert_eq!(refusal.cause, Some(Cause::StaleNodeKey));
    assert_eq!(refusal.expected_node_id(), None);
}

/// Trace: FR-038-AC-183, FR-038-AC-185, TC-048
#[trace("FR-038-AC-183", "FR-038-AC-185", "TC-048")]
#[test]
fn tc_048_each_self_typed_shape_needs_its_id_and_type_changed_together() {
    let base = base();
    let empty = json!({"term":"aggregate", "members":[]});
    for form in ["boolean", "integer"] {
        let key = structural_key("scalar_type", form, None, &empty);
        let package = with_nodes(
            nominal_package(&[]),
            vec![node(&key, "scalar_type", form, &key, &[], empty.clone())],
        );
        assert_self_type_control(form, &package, &key);
    }
    for form in [
        "reference",
        "option",
        "set",
        "bag",
        "sequence",
        "ordered_set",
    ] {
        let (key, member) = collection(form, &base.zero);
        let package = with_nodes(base.package.clone(), vec![member]);
        assert_self_type_control(form, &package, &key);
    }
}

fn assert_self_type_control(form: &str, package: &Value, key: &str) {
    admitted(form, package);
    let changed = family_key("eded");
    let mut stale_type = package.clone();
    rename_node(&mut stale_type, key, &changed);
    let position = at(&stale_type, &changed);
    stale_type["semantic_graph"]["nodes"][position]["semantic_type"] = node_id(key);
    rebuild_source_map(&mut stale_type);
    refresh_identity(&mut stale_type);
    let refusal = refusal_of(form, &stale_type, &evidence_for(&stale_type));
    assert_eq!(refusal.code, Code::InvalidPackage, "{form}");
    assert_eq!(refusal.cause, Some(Cause::StaleNodeKey), "{form}");
    assert_eq!(refusal.locus, Some(typed_node_id(&changed)), "{form}");
    assert_eq!(refusal.expected_node_id(), None, "{form}");

    stale_type["semantic_graph"]["nodes"][position]["semantic_type"] = node_id(&changed);
    refresh_identity(&mut stale_type);
    let refusal = refusal_of(form, &stale_type, &evidence_for(&stale_type));
    assert_eq!(refusal.code, Code::InvalidPackage, "{form}");
    assert_eq!(refusal.cause, Some(Cause::StaleNodeKey), "{form}");
    assert_eq!(refusal.locus, Some(typed_node_id(&changed)), "{form}");
    let expected = refusal.expected_node_id().expect("self-typed derived key");
    assert_eq!(expected, &typed_node_id(&expected.digest), "{form}");
}
