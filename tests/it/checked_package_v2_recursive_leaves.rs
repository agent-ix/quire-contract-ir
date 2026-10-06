// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! IR-486: `structural.eq` over a compared type that reaches itself through a
//! record is admitted with its text leaves and its recursion leaves
//! (FR-038, "Recursive compared types"), read end to end through
//! `CheckedPackageV2::read`.
//!
//! Every package below is built here, node by node, from this crate's own
//! vocabulary: a record type node names each field's type by a `reference`
//! term, an optional field is a one-member union over its inner type, and the
//! cycle's members share one `recursion_group`.

use crate::support::checked_package::{
    canonical, evidence_for, node_id, nominal_package, pointer, refusal_at, rename_node, settle,
    sha256_hex, structural_key, BOOLEAN_KEY, INTEGER_KEY,
};
use ix_trace_rs::trace;
use quire_contract_ir::{
    CheckedPackageLimit, CheckedPackageReadLimits, CheckedPackageRefusalCause,
    CheckedPackageRefusalCode, CheckedPackageV2, CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};
use std::collections::BTreeSet;

type Typed = (String, Value);

/// The key of the node called `name`: the derived key (FR-038-AC-134) for
/// the anonymous `integer` and `boolean` nodes, the digest of the name
/// otherwise.
fn key(name: &str) -> String {
    match name {
        "integer" => INTEGER_KEY.to_owned(),
        "boolean" => BOOLEAN_KEY.to_owned(),
        "text" => structural_key("scalar_type", "text", None, &empty()),
        "nfc" => structural_key(
            "bounded_domain",
            "text_bounds",
            Some(&key("text")),
            &json!({"term": "aggregate", "members": [
                binding("text_profile", literal("text", "text", "nfc")),
            ]}),
        ),
        _ => sha256_hex(name.as_bytes()),
    }
}

fn node<S: AsRef<str>>(
    name: &str,
    tag: &str,
    form: &str,
    semantic_type: &str,
    over: impl IntoIterator<Item = S>,
    role: &str,
    body: Value,
) -> Typed {
    let id = key(name);
    let dependencies: BTreeSet<String> = over.into_iter().map(|name| key(name.as_ref())).collect();
    let node = json!({
        "node_id": node_id(&id),
        "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": tag,
        "semantic_form": form,
        "semantic_type": node_id(&key(semantic_type)),
        "dependencies": dependencies.iter().map(|digest| node_id(digest)).collect::<Vec<_>>(),
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

fn reference(name: &str) -> Value {
    json!({"term": "reference", "target": node_id(&key(name))})
}

fn empty() -> Value {
    json!({"term": "aggregate", "members": []})
}

fn scalar(name: &str, form: &str) -> Typed {
    node(
        name,
        "scalar_type",
        form,
        name,
        [] as [&str; 0],
        "type",
        empty(),
    )
}

/// `Text[nfc]`: a `text_bounds` domain binding the `nfc` profile over `text`.
fn nfc_text() -> Typed {
    let body = json!({"term": "aggregate", "members": [
        binding("text_profile", literal("text", "text", "nfc")),
    ]});
    node(
        "nfc",
        "bounded_domain",
        "text_bounds",
        "text",
        ["text"],
        "type",
        body,
    )
}

/// A parameter named `name` typed at the node called `ty` (QSL FR-092).
fn parameter(name: &str, ty: &str) -> Typed {
    let body = json!({"term": "aggregate", "members": [
        binding("name", literal("text", "text", name)),
        binding("level", literal("integer", "integer", "0")),
    ]});
    let (_, mut parameter) = node(
        name,
        "value",
        "parameter",
        ty,
        [] as [&str; 0],
        "expression",
        body,
    );
    let id = structural_key("value", "parameter", Some(&key(ty)), &parameter["body"]);
    parameter["node_id"] = node_id(&id);
    (id, parameter)
}

/// A record type named `name`, one field per `(field, type name)`.
fn record(name: &str, fields: &[(&str, &str)]) -> Typed {
    let members: Vec<Value> = fields
        .iter()
        .map(|(field, ty)| binding(field, reference(ty)))
        .collect();
    node(
        name,
        "composite_type",
        "record",
        name,
        fields.iter().map(|(_, ty)| *ty),
        "type",
        json!({"term": "aggregate", "members": members}),
    )
}

/// The optional `inner` named `name`, as a union of one member `Some` over
/// `inner`. A derived-shape `option` node cannot sit in a recursion group (its
/// key hashes a preimage with a `null` recursion: FR-038, IR-627-Q4), so the
/// cycles these tests build pass through a union, whose leaves read
/// `member:Some`, `position:0`.
fn some_of(name: &str, inner: &str) -> Typed {
    node(
        name,
        "composite_type",
        "union",
        name,
        [inner],
        "type",
        json!({"term": "aggregate", "members": [
            binding("Some", json!({"term": "aggregate", "members": [reference(inner)]})),
        ]}),
    )
}

/// `Option<inner>` in the QSL wrapped optional-field encoding.
fn option_of(name: &str, inner: &str) -> Typed {
    node(
        name,
        "composite_type",
        "option",
        name,
        [inner],
        "type",
        json!({"term": "aggregate", "members": [reference(inner)]}),
    )
}

/// The leaf segments from a `some_of` node into its payload.
fn some_segments() -> [String; 2] {
    ["member:Some".to_owned(), "position:0".to_owned()]
}

/// A member of the one recursion group the cycle shares.
fn in_group((id, mut node): Typed) -> Typed {
    node["recursion_group"] = json!("g");
    (id, node)
}

/// A catalogued `text_profile` law definition, read from the catalog's home.
fn text_law() -> Value {
    let catalog: Value = serde_json::from_str(
        quire_verification_contracts::operation_catalog::CHECKED_OPERATION_CATALOG_V1,
    )
    .expect("the catalog is JSON");
    catalog["law_roles"]["text_profile"][0].clone()
}

fn text_leaf(path: &[String]) -> Value {
    json!({
        "path": path,
        "laws": [{"role": "text_profile", "definition": text_law()}],
        "mode": {"kind": "text_profile", "value": "nfc"},
    })
}

fn recursion_leaf(path: &[String]) -> Value {
    json!({"path": path, "laws": [], "mode": null})
}

/// `field:<name>` repeated for each of `names`, then `last`.
fn path(segments: &[&str], repeat: &str, times: usize, last: &str) -> Vec<String> {
    segments
        .iter()
        .map(|segment| (*segment).to_owned())
        .chain(std::iter::repeat_n(repeat.to_owned(), times))
        .chain(std::iter::once(last.to_owned()))
        .collect()
}

/// A package holding `types`, a parameter `p` typed at `root` and the
/// application `structural.eq(p, p)` carrying `leaves`. Returns it with the
/// application's node key.
fn equality_package(types: Vec<Typed>, root: &str, leaves: Vec<Value>) -> (Value, String) {
    let body = json!({
        "term": "application",
        "operator": "binary",
        "operation": {"identity": "quire.op.structural.eq", "laws": [], "mode": null,
            "member": null, "leaves": leaves},
        "result_type": node_id(&key("boolean")),
        "arguments": [reference("p"), reference("p")],
    });
    // FR-322 application key: the reader re-derives it.
    let id = sha256_hex(&canonical(&json!({
        "version": "quire.application-node/v1",
        "node_tag": "expression",
        "semantic_form": "binary",
        "semantic_type": node_id(&key("boolean")),
        "declaration": null,
        "recursion": null,
        "body": body,
    })));
    let (_, mut application) = node(
        "equality",
        "expression",
        "binary",
        "boolean",
        ["p"],
        "expression",
        body,
    );
    application["node_id"] = node_id(&id);

    let mut nodes = vec![
        scalar("integer", "integer"),
        scalar("text", "text"),
        scalar("boolean", "boolean"),
        nfc_text(),
        parameter("p", root),
        (id.clone(), application),
    ];
    nodes.extend(types);
    nodes.sort_by(|left, right| left.0.cmp(&right.0));
    let mut package = nominal_package(&[]);
    package["semantic_graph"]["nodes"] =
        Value::Array(nodes.into_iter().map(|(_, node)| node).collect());
    package["lock"]["definition_selections"]
        .as_array_mut()
        .expect("definition selections")
        .push(text_law());
    let parameter_id = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "value" && node["semantic_form"] == "parameter")
        .and_then(|node| node["node_id"]["digest"].as_str())
        .expect("parameter key")
        .to_owned();
    rename_node(&mut package, &key("p"), &parameter_id);
    settle(&mut package);
    let id = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "expression" && node["semantic_form"] == "binary")
        .and_then(|node| node["node_id"]["digest"].as_str())
        .expect("application key")
        .to_owned();
    (package, id)
}

fn read(package: &Value, limits: CheckedPackageReadLimits) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(&canonical(package), limits, &evidence_for(package))
}

/// The pointer of `operation.leaves` of the application node keyed `id`.
fn leaves_pointer(package: &Value, id: &str) -> String {
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_id"]["digest"] == id)
        .expect("the application node");
    format!("/semantic_graph/nodes/{position}/body/operation/leaves")
}

/// `record Node { label: Text[nfc]; next?: Node; }`.
fn node_types() -> Vec<Typed> {
    vec![
        in_group(record("node", &[("label", "nfc"), ("next", "node_next")])),
        in_group(some_of("node_next", "node")),
    ]
}

/// The QSL record-field encoding of `next?: List` contributes one field edge.
fn wrapped_list(text_fields: &[&str]) -> Vec<Typed> {
    let mut fields = vec![("number", "integer")];
    if let Some(label) = text_fields.first() {
        fields.push((label, "nfc"));
    }
    fields.push(("next", "list_next"));
    fields.extend(text_fields.iter().skip(1).map(|name| (*name, "nfc")));
    let (id, mut list) = record("list", &fields);
    let next = list["body"]["members"]
        .as_array_mut()
        .expect("record members")
        .iter_mut()
        .find(|field| field["name"] == "next")
        .expect("next field");
    next["value"] = json!({"term": "aggregate", "members": [
        binding("optional", reference("list_next")),
    ]});
    vec![
        in_group((id, list)),
        in_group(option_of("list_next", "list")),
    ]
}

fn list_leaves(text_fields: &[&str]) -> Vec<Value> {
    let mut leaves: Vec<Value> = text_fields
        .first()
        .map(|name| text_leaf(&[field(name)]))
        .into_iter()
        .collect();
    leaves.push(recursion_leaf(&[
        field("next"),
        "inner".into(),
        "recursion:0".into(),
    ]));
    leaves.extend(
        text_fields
            .iter()
            .skip(1)
            .map(|name| text_leaf(&[field(name)])),
    );
    leaves
}

/// Tracing: TC-048, FR-038-AC-151
#[trace("TC-048", "FR-038-AC-151")]
#[test]
fn tc_048_wrapped_optional_record_fields_keep_their_leaf_order() {
    let limits = CheckedPackageReadLimits::bounded();
    let (integer_list, _) = equality_package(wrapped_list(&[]), "list", vec![]);
    assert!(matches!(
        read(&integer_list, limits),
        CheckedPackageV2ReadResult::Admitted(_)
    ));

    for text_fields in [&["label"][..], &["label", "tail"][..]] {
        let (package, _) =
            equality_package(wrapped_list(text_fields), "list", list_leaves(text_fields));
        match read(&package, limits) {
            CheckedPackageV2ReadResult::Admitted(_) => {}
            other => panic!("wrapped optional with {text_fields:?} must admit: {other:?}"),
        }
    }

    let (package, id) = equality_package(
        wrapped_list(&["label"]),
        "list",
        vec![text_leaf(&[field("label")])],
    );
    assert_eq!(
        read(&package, limits),
        CheckedPackageV2ReadResult::Refused(refusal_at(
            CheckedPackageRefusalCode::InvalidPackage,
            &leaves_pointer(&package, &id),
            Some(CheckedPackageRefusalCause::OperationLawMissing),
            &id,
        )),
    );

    let mut direct = wrapped_list(&["label"]);
    direct[0].1["body"]["members"]
        .as_array_mut()
        .expect("record members")
        .iter_mut()
        .find(|field| field["name"] == "next")
        .expect("next field")["value"] = reference("list_next");
    let (package, _) = equality_package(direct, "list", list_leaves(&["label"]));
    assert!(matches!(
        read(&package, limits),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

/// Tracing: TC-048, FR-038-AC-152
#[trace("TC-048", "FR-038-AC-152")]
#[test]
fn tc_048_malformed_optional_record_fields_refuse_at_operation_leaves() {
    let malformed = [
        json!({"term": "aggregate", "members": []}),
        json!({"term": "aggregate", "members": [
            binding("optional", reference("list_next")),
            binding("optional", reference("list_next")),
        ]}),
        json!({"term": "aggregate", "members": [binding("other", reference("list_next"))]}),
        json!({"term": "aggregate", "members": [reference("list_next")]}),
        json!({"term": "aggregate", "members": [binding("optional", reference("integer"))]}),
        json!({"term": "aggregate", "members": [binding("optional", literal("integer", "integer", "1"))]}),
    ];
    for value in malformed {
        let mut types = wrapped_list(&["label"]);
        types[0].1["body"]["members"]
            .as_array_mut()
            .expect("record members")
            .iter_mut()
            .find(|field| field["name"] == "next")
            .expect("next field")["value"] = value;
        // Keep the group's independent cycle valid while `next` is malformed.
        // The operation step must still examine the malformed optional field.
        types[0].1["body"]["members"]
            .as_array_mut()
            .expect("record members")
            .push(binding("cycle", reference("list_next")));
        let (package, id) = equality_package(types, "list", list_leaves(&["label"]));
        assert_eq!(
            read(&package, CheckedPackageReadLimits::bounded()),
            CheckedPackageV2ReadResult::Refused(refusal_at(
                CheckedPackageRefusalCode::IllTyped,
                &leaves_pointer(&package, &id),
                Some(CheckedPackageRefusalCause::OperatorIneligible),
                &id,
            )),
            "malformed next: {}",
            package["semantic_graph"]["nodes"],
        );
    }
}

/// Trace: FR-038-AC-145
#[trace("FR-038-AC-145")]
#[test]
fn tc_226_a_severed_only_cycle_rederives_the_former_group_member_first() {
    let mut types = wrapped_list(&["label"]);
    types[0].1["body"]["members"]
        .as_array_mut()
        .expect("record members")
        .iter_mut()
        .find(|field| field["name"] == "next")
        .expect("next field")["value"] = empty();
    let (package, _) = equality_package(types, "list", list_leaves(&["label"]));
    let option = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["semantic_form"] == "option")
        .expect("former group option");
    let key = package["semantic_graph"]["nodes"][option]["node_id"]["digest"]
        .as_str()
        .expect("option key");
    assert_eq!(
        read(&package, CheckedPackageReadLimits::bounded()),
        CheckedPackageV2ReadResult::Refused(refusal_at(
            CheckedPackageRefusalCode::InvalidPackage,
            &format!("/semantic_graph/nodes/{option}/node_id"),
            Some(CheckedPackageRefusalCause::StaleNodeKey),
            key,
        )),
    );
}

fn field(name: &str) -> String {
    format!("field:{name}")
}

/// Equality over `Node` is admitted end to end with its text leaf and its
/// recursion leaf; the text leaf alone refuses `operation-law-missing` at
/// `operation.leaves`, and a further text leaf refuses
/// `operation-law-mismatch` at that entry.
///
/// Tracing: TC-048, FR-038-AC-70
#[trace("TC-048", "FR-038-AC-70")]
#[test]
fn tc_048_the_reader_admits_equality_over_a_recursive_record() {
    let label = text_leaf(&[field("label")]);
    let [some, payload] = some_segments();
    let recursion = recursion_leaf(&[field("next"), some, payload, "recursion:0".into()]);
    let limits = CheckedPackageReadLimits::bounded();

    let (package, _) =
        equality_package(node_types(), "node", vec![label.clone(), recursion.clone()]);
    match read(&package, limits) {
        CheckedPackageV2ReadResult::Admitted(_) => {}
        other => panic!("expected V2 admission, got {other:?}"),
    }

    let (package, id) = equality_package(node_types(), "node", vec![label.clone()]);
    assert_eq!(
        read(&package, limits),
        CheckedPackageV2ReadResult::Refused(refusal_at(
            CheckedPackageRefusalCode::InvalidPackage,
            &leaves_pointer(&package, &id),
            Some(CheckedPackageRefusalCause::OperationLawMissing),
            &id,
        )),
        "the text leaf alone"
    );

    let (package, id) =
        equality_package(node_types(), "node", vec![label.clone(), recursion, label]);
    assert_eq!(
        read(&package, limits),
        CheckedPackageV2ReadResult::Refused(refusal_at(
            CheckedPackageRefusalCode::InvalidPackage,
            &format!("{}/2", leaves_pointer(&package, &id)),
            Some(CheckedPackageRefusalCause::OperationLawMismatch),
            &id,
        )),
        "a text leaf with no place left"
    );
}

/// A tuple type named `name`, one position per type name.
fn tuple_of(name: &str, positions: &[&str]) -> Typed {
    let members: Vec<Value> = positions.iter().map(|ty| reference(ty)).collect();
    node(
        name,
        "composite_type",
        "tuple",
        name,
        positions.iter().copied(),
        "type",
        json!({"term": "aggregate", "members": members}),
    )
}

/// `record Cell { item: (Text[nfc], Option<Cell>); }`: the cycle passes
/// through a tuple and an option, and the record is entered at the empty path,
/// so its recursion leaf reads `recursion:0` (QSpec FR-322 "Structural leaf
/// walk"). The text leaf alone and a recursion leaf reading `recursion:1` are
/// refused.
///
/// Tracing: TC-048, FR-038-AC-70
#[trace("TC-048", "FR-038-AC-70")]
#[test]
fn tc_048_the_reader_admits_equality_over_a_record_cycling_through_a_tuple() {
    let types = || {
        vec![
            in_group(record("cell", &[("item", "cell_item")])),
            in_group(tuple_of("cell_item", &["nfc", "cell_next"])),
            in_group(some_of("cell_next", "cell")),
        ]
    };
    let item = |last: &[&str]| -> Vec<String> {
        [field("item")]
            .into_iter()
            .chain(last.iter().map(|segment| (*segment).to_owned()))
            .collect()
    };
    let text = text_leaf(&item(&["position:0"]));
    let recursion =
        |depth: &str| recursion_leaf(&item(&["position:1", "member:Some", "position:0", depth]));
    let limits = CheckedPackageReadLimits::bounded();

    let (package, _) = equality_package(
        types(),
        "cell",
        vec![text.clone(), recursion("recursion:0")],
    );
    match read(&package, limits) {
        CheckedPackageV2ReadResult::Admitted(_) => {}
        other => panic!("expected V2 admission, got {other:?}"),
    }

    let (package, id) = equality_package(types(), "cell", vec![text.clone()]);
    assert_eq!(
        read(&package, limits),
        CheckedPackageV2ReadResult::Refused(refusal_at(
            CheckedPackageRefusalCode::InvalidPackage,
            &leaves_pointer(&package, &id),
            Some(CheckedPackageRefusalCause::OperationLawMissing),
            &id,
        )),
        "the text leaf alone"
    );

    let (package, id) = equality_package(types(), "cell", vec![text, recursion("recursion:1")]);
    assert_eq!(
        read(&package, limits),
        CheckedPackageV2ReadResult::Refused(refusal_at(
            CheckedPackageRefusalCode::InvalidPackage,
            &format!("{}/1/path", leaves_pointer(&package, &id)),
            Some(CheckedPackageRefusalCause::OperationLawMismatch),
            &id,
        )),
        "a recursion leaf with the wrong depth"
    );
}

/// A ring of `size` records, each holding one text field and an optional
/// field naming the next, the last naming the first, and the leaves of the
/// ring compared at the first: its `size` text leaves and its one recursion
/// leaf (the last record's field reenters the first).
fn ring_of_text_records(size: usize) -> (Value, String) {
    let names: Vec<String> = (0..size).map(|at| format!("ring{at}")).collect();
    let optionals: Vec<String> = (0..size).map(|at| format!("ring{at}_next")).collect();
    let mut types = Vec::new();
    for at in 0..size {
        let following = (at + 1) % size;
        types.push(in_group(record(
            &names[at],
            &[("t", "nfc"), ("next", &optionals[at])],
        )));
        types.push(in_group(some_of(&optionals[at], &names[following])));
    }
    let step = |times: usize| -> Vec<String> {
        (0..times)
            .flat_map(|_| {
                let [some, payload] = some_segments();
                [field("next"), some, payload]
            })
            .collect()
    };
    let mut leaves: Vec<Value> = (0..size)
        .map(|at| {
            let mut leaf = step(at);
            leaf.push(field("t"));
            text_leaf(&leaf)
        })
        .collect();
    let mut reentry = step(size);
    reentry.push("recursion:0".to_owned());
    leaves.push(recursion_leaf(&reentry));
    equality_package(types, &names[0], leaves)
}

/// The least work limit under which `package` reads to admission, found by
/// bisection against the real reader.
fn work_read_used(package: &Value) -> u64 {
    let bounded = CheckedPackageReadLimits::bounded();
    let (mut low, mut high) = (0_u64, bounded.work);
    while low + 1 < high {
        let middle = low + (high - low) / 2;
        let mut limits = bounded;
        limits.work = middle;
        match read(package, limits) {
            CheckedPackageV2ReadResult::Admitted(_) => high = middle,
            _ => low = middle,
        }
    }
    high
}

/// A ring of 12 records with its 12 text leaves and its recursion leaf
/// admits at the exact work a read used, and one below that returns
/// `incomplete` for `work`.
///
/// Tracing: TC-048, FR-038-AC-72
#[trace("TC-048", "FR-038-AC-72")]
#[test]
fn tc_048_a_ring_of_twelve_records_is_decided_at_its_exact_work() {
    let (package, _) = ring_of_text_records(12);
    let used = work_read_used(&package);
    let mut limits = CheckedPackageReadLimits::bounded();
    limits.work = used;
    assert!(
        matches!(
            read(&package, limits),
            CheckedPackageV2ReadResult::Admitted(_)
        ),
        "the exact work admits"
    );
    limits.work = used - 1;
    match read(&package, limits) {
        CheckedPackageV2ReadResult::Incomplete(incomplete) => {
            assert_eq!(incomplete.limit_kind, CheckedPackageLimit::Work);
            assert_eq!(incomplete.limit, used - 1);
        }
        other => panic!("one below the work a read used must be incomplete, got {other:?}"),
    }
}

/// Ten records that each hold a text field and an optional field naming every
/// other record, compared at the first with `leaves` empty, return
/// `incomplete` for `work` at `operation.leaves` under the default read
/// limits: the `(n - 1)!` leaves they unfold to are never listed.
///
/// Tracing: TC-048, FR-038-AC-72
#[trace("TC-048", "FR-038-AC-72")]
#[test]
fn tc_048_ten_records_naming_each_other_exhaust_the_work_budget() {
    let size = 10;
    let names: Vec<String> = (0..size).map(|at| format!("r{at}")).collect();
    let optionals: Vec<String> = (0..size).map(|at| format!("o{at}")).collect();
    let mut types = Vec::new();
    for at in 0..size {
        let mut fields: Vec<(&str, &str)> = vec![("t", "nfc")];
        fields.extend(
            (0..size)
                .filter(|other| *other != at)
                .map(|other| (optionals[other].as_str(), optionals[other].as_str())),
        );
        types.push(in_group(record(&names[at], &fields)));
        types.push(in_group(some_of(&optionals[at], &names[at])));
    }
    let (package, id) = equality_package(types, &names[0], Vec::new());
    let limits = CheckedPackageReadLimits::bounded();
    match read(&package, limits) {
        CheckedPackageV2ReadResult::Incomplete(incomplete) => {
            assert_eq!(incomplete.limit_kind, CheckedPackageLimit::Work);
            assert_eq!(incomplete.limit, limits.work);
            assert_eq!(
                incomplete.path,
                Some(pointer(&leaves_pointer(&package, &id)))
            );
        }
        other => panic!("expected incomplete for work, got {other:?}"),
    }
}

/// A cycle of 20000 record nodes, each holding an integer field and naming
/// the next, the last naming the first and also holding one text field,
/// admits with its one 20000-segment text leaf and its recursion leaf on a
/// thread whose stack is 256 KiB, under byte, node, edge and work limits
/// raised to admit it; the default limits read it as `incomplete` for
/// `bytes`, since the package is larger than the default 1 MiB.
///
/// Tracing: TC-048, FR-038-AC-72
#[trace("TC-048", "FR-038-AC-72")]
#[test]
fn tc_048_a_cycle_of_twenty_thousand_records_is_decided_on_a_small_stack() {
    const SIZE: usize = 20_000;
    let names: Vec<String> = (0..SIZE).map(|at| format!("link{at}")).collect();
    let mut types = Vec::with_capacity(SIZE);
    for at in 0..SIZE {
        let following = &names[(at + 1) % SIZE];
        let mut fields = vec![("k", "integer"), ("next", following.as_str())];
        if at + 1 == SIZE {
            fields.insert(1, ("t", "nfc"));
        }
        types.push(in_group(record(&names[at], &fields)));
    }
    let text = path(&[], &field("next"), SIZE - 1, &field("t"));
    let reentry = path(&[], &field("next"), SIZE, "recursion:0");
    assert_eq!(text.len(), SIZE);
    let (package, _) = equality_package(
        types,
        &names[0],
        vec![text_leaf(&text), recursion_leaf(&reentry)],
    );

    let mut limits = CheckedPackageReadLimits::bounded();
    match read(&package, limits) {
        CheckedPackageV2ReadResult::Incomplete(incomplete) => {
            assert_eq!(incomplete.limit_kind, CheckedPackageLimit::Bytes);
        }
        other => panic!("the default byte limit is below the package, got {other:?}"),
    }
    limits.bytes = 64 * 1024 * 1024;
    limits.nodes = 100_000;
    limits.edges = 1_000_000;
    limits.occurrences = 1_000_000;
    limits.work = 1_000_000_000;
    let reader = std::thread::Builder::new()
        .stack_size(256 * 1024)
        .spawn(move || match read(&package, limits) {
            CheckedPackageV2ReadResult::Admitted(_) => Ok(()),
            other => Err(format!("{other:?}")),
        })
        .expect("spawn the 256 KiB reader thread");
    assert_eq!(reader.join().expect("the reader did not overflow"), Ok(()));
}
