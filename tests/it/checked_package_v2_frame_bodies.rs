// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-340 frame-body semantics: entry eligibility, canonical member order,
//! and cross-defect refusal precedence for `quire.checked-package/v2`'s
//! `state`/`frame` nodes.

use crate::support::checked_package::{
    canonical, evidence_for, node_id, refresh_identity, v2_all_families,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedNodeId, CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageRefusal,
    CheckedPackageRefusalCause, CheckedPackageRefusalCode, CheckedPackageV2,
    CheckedPackageV2ReadResult, CheckedTerm,
};
use serde_json::{json, Value};

/// A `modifies` entry granting the relationship `declaration` names.
fn relationship(declaration: Value) -> Value {
    json!({"kind": "relationship", "declaration": declaration})
}

/// The RFC 6901 pointer of entry `entry` of the frame node at `index`'s
/// `modifies` member: a meaning-join defect is located at the entry itself.
fn modifies_entry(index: usize, entry: usize) -> String {
    format!("/semantic_graph/nodes/{index}/body/modifies/{entry}")
}

/// The pointer of the frame node at `index`'s body: a canonical-order defect
/// is located at the frame body, not at any one member entry.
fn frame_body(index: usize) -> String {
    format!("/semantic_graph/nodes/{index}/body")
}

fn read(value: &Value, evidence: &CheckedPackageEvidence) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(
        &canonical(value),
        CheckedPackageReadLimits::bounded(),
        evidence,
    )
}

fn admitted(value: &Value) -> Box<CheckedPackageV2> {
    match read(value, &evidence_for(value)) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("expected V2 admission, got {other:?}"),
    }
}

fn refused(value: &Value) -> CheckedPackageRefusal {
    match read(value, &evidence_for(value)) {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        other => panic!("expected V2 refusal, got {other:?}"),
    }
}

fn id_order(left: &Value, right: &Value) -> std::cmp::Ordering {
    let left: CheckedNodeId = serde_json::from_value(left.clone()).expect("left node id");
    let right: CheckedNodeId = serde_json::from_value(right.clone()).expect("right node id");
    left.cmp(&right)
}

fn expect_refusal_at(
    package: &Value,
    code: CheckedPackageRefusalCode,
    path: &str,
    cause: Option<CheckedPackageRefusalCause>,
    locus: &Value,
) {
    let refusal = refused(package);
    assert_eq!(refusal.code, code, "{refusal:?}");
    assert_eq!(refusal.cause, cause, "{refusal:?}");
    assert_eq!(
        refusal.loci[0].path.as_ref().map(|path| path.as_str()),
        Some(path),
        "{refusal:?}"
    );
    assert_eq!(
        refusal
            .locus
            .as_ref()
            .map(|id| serde_json::to_value(id).expect("node id")),
        Some(locus.clone()),
        "{refusal:?}"
    );
}

/// The single `state`/`frame` node in `v2_all_families()`.
fn frame_index(package: &Value) -> usize {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_tag"] == "state" && node["semantic_form"] == "frame")
        .expect("base fixture carries exactly one frame node")
}

#[trace("FR-349-AC-2")]
#[test]
fn admitted_frame_body_is_typed_without_changing_the_wire() {
    let fixture = v2_all_families();
    let package = admitted(&fixture);
    let position = frame_index(&fixture);
    let node = &package.graph().nodes[position];
    let Some(CheckedTerm::Frame {
        modifies,
        creates,
        deletes,
        ..
    }) = package.term(&node.node_id)
    else {
        panic!("admitted frame must have a typed frame body");
    };
    assert!(!modifies.is_empty());
    assert!(!creates.is_empty());
    assert!(!deletes.is_empty());
    assert_eq!(
        serde_json::to_value(package.term(&node.node_id)).expect("typed frame serializes"),
        serde_json::to_value(&node.body).expect("typed body")
    );
}

/// The `v2_all_families()` frame node already exercises `object_type` in
/// `creates` and `process` in `deletes`; this
/// covers the remaining two — `process` in `creates` and `object_type` in
/// `deletes` — by reassigning the same two already-declared dependencies to
/// the other eligible member, so the whole package still admits.
#[trace("TC-053", "FR-038-AC-12")]
#[test]
fn tc_053_process_in_creates_and_object_type_in_deletes_are_admitted() {
    let base = v2_all_families();
    let index = frame_index(&base);
    let dependencies = base["semantic_graph"]["nodes"][index]["dependencies"].clone();
    let modifies = base["semantic_graph"]["nodes"][index]["body"]["modifies"].clone();
    let object_type = base["semantic_graph"]["nodes"][index]["body"]["creates"][0].clone();
    let process = base["semantic_graph"]["nodes"][index]["body"]["deletes"][0].clone();

    let mut package = base.clone();
    package["semantic_graph"]["nodes"][index]["dependencies"] = dependencies;
    package["semantic_graph"]["nodes"][index]["body"]["modifies"] = modifies;
    package["semantic_graph"]["nodes"][index]["body"]["creates"] = Value::Array(vec![process]);
    package["semantic_graph"]["nodes"][index]["body"]["deletes"] = Value::Array(vec![object_type]);
    refresh_identity(&mut package);

    admitted(&package);
}

/// FR-340's headline precedence rule (`spec/checked_package/functional/FR-038-consume-checked-package-v2.md`:
/// "A frame node carrying more than one defect refuses for exactly one of
/// them ... any meaning-join defect ... outranks a canonical-order defect
/// outright") has no vendored vector that puts both defect classes in one
/// frame. This case is authored locally.
///
/// It narrows the frame's `dependencies` to the fixture's own `object_type`
/// and `process` nodes, places `object_type` as a `modifies` relationship
/// entry, whose declaring node must be a `relation`/`relationship`,
/// (a meaning-join defect), and places both entries into `deletes` in
/// descending structural ID order (`process` before `object_type`; both are
/// individually eligible for `deletes`, so this is a pure canonical-order
/// defect there, not a second meaning-join defect). FR-340 requires the
/// meaning-join defect to win: `invalid_model_binding`/`malformed-declaration`
/// located at `object_type`, never `invalid_semantic_graph` at the frame
/// node for the `deletes` misorder.
#[trace("TC-053", "FR-038-AC-14", "FR-349-AC-4")]
#[test]
fn tc_053_meaning_join_defect_outranks_a_co_occurring_order_defect() {
    let base = v2_all_families();
    let index = frame_index(&base);
    let object_type = base["semantic_graph"]["nodes"][index]["body"]["creates"][0].clone();
    let process = base["semantic_graph"]["nodes"][index]["body"]["deletes"][0].clone();

    let mut package = base.clone();
    package["semantic_graph"]["nodes"][index]["dependencies"] =
        Value::Array(vec![object_type.clone(), process.clone()]);
    package["semantic_graph"]["nodes"][index]["body"]["modifies"] =
        Value::Array(vec![relationship(object_type.clone())]);
    package["semantic_graph"]["nodes"][index]["body"]["creates"] = Value::Array(vec![]);
    // Descending ID order: `process` ("1616") before `object_type`
    // ("1515") — a canonical-order defect, since both entries are
    // individually eligible for `deletes`.
    package["semantic_graph"]["nodes"][index]["body"]["deletes"] =
        Value::Array(vec![process, object_type.clone()]);
    refresh_identity(&mut package);

    expect_refusal_at(
        &package,
        CheckedPackageRefusalCode::InvalidModelBinding,
        &modifies_entry(index, 0),
        Some(CheckedPackageRefusalCause::MalformedDeclaration),
        &object_type,
    );
}

/// FR-038-AC-13: a frame entry naming an ID that is not among the frame
/// node's own `dependencies` refuses as `missing_declaration`/`missing-name`
/// at that entry — whether the digest belongs to a real node elsewhere in the
/// graph (declared to the package, just never to this frame) or to no node
/// anywhere in the graph at all. Both are the same refusal: the check
/// (`frame_defect`'s `!declared.contains(&entry)` branch) looks only at
/// membership in the frame's own `dependencies`, before it ever asks whether
/// the ID resolves to a real node.
#[trace("TC-053", "FR-038-AC-13")]
#[test]
fn tc_053_frame_entry_outside_dependencies_refuses_as_missing_declaration() {
    let base = v2_all_families();
    let index = frame_index(&base);
    let real_elsewhere = base["semantic_graph"]["nodes"][0]["node_id"].clone();
    let nowhere = node_id("absent/frame-entry");

    for entry in [real_elsewhere, nowhere] {
        let mut package = base.clone();
        package["semantic_graph"]["nodes"][index]["body"]["modifies"] =
            Value::Array(vec![relationship(entry.clone())]);
        refresh_identity(&mut package);
        expect_refusal_at(
            &package,
            CheckedPackageRefusalCode::MissingDeclaration,
            &modifies_entry(index, 0),
            Some(CheckedPackageRefusalCause::MissingName),
            &entry,
        );
    }
}

/// FR-038-AC-15: with two defective `state`/`frame` nodes, the reader reports
/// the first one it visits (ascending structural `node_id`) rather than comparing
/// every frame's defect.
///
/// Adds a second frame node keyed below the published frame's own ID,
/// built from the same entries the published frame already declares, but
/// with `deletes` in descending ID order — a pure canonical-order defect
/// (the *weaker* of the two defect classes; both entries stay individually
/// eligible for `deletes`, so there is no meaning-join defect alongside it).
/// The published frame itself is then mutated to carry a missing-declaration
/// defect (an entry naming a real node outside its dependencies). AC-15
/// requires the lower-keyed frame's own defect to be reported instead: the
/// higher-keyed frame is never reached.
#[trace("TC-053", "FR-038-AC-15")]
#[test]
fn tc_053_two_defective_frames_refuse_at_the_lower_keyed_frame() {
    let base = v2_all_families();
    let index = frame_index(&base);
    let mut lower_frame = base["semantic_graph"]["nodes"][index].clone();
    let object_type = base["semantic_graph"]["nodes"][index]["body"]["creates"][0].clone();
    let process = base["semantic_graph"]["nodes"][index]["body"]["deletes"][0].clone();

    // A second frame, keyed below the published frame's own ID, sharing
    // the same declared dependencies and `modifies` entries, but with
    // `deletes` reversed.
    lower_frame["node_id"] = node_id("0505");
    lower_frame["body"]["creates"] = Value::Array(Vec::new());
    let mut deletes = vec![process, object_type];
    deletes.sort_by(|left, right| id_order(right, left));
    lower_frame["body"]["deletes"] = Value::Array(deletes);

    let mut package = base.clone();
    let nodes = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    let lower_index = nodes.len();
    nodes.push(lower_frame);

    // Choose a real node outside the frame's dependencies whose independently
    // derived stronger-defect frame key sorts after the weaker frame's key.
    let package = (0..lower_index)
        .filter_map(|candidate| {
            let target = package["semantic_graph"]["nodes"][candidate]["node_id"].clone();
            if package["semantic_graph"]["nodes"][index]["dependencies"]
                .as_array()
                .expect("dependencies")
                .contains(&target)
            {
                return None;
            }
            let mut variant = package.clone();
            variant["semantic_graph"]["nodes"][index]["body"]["modifies"] =
                Value::Array(vec![relationship(target)]);
            let lower = &variant["semantic_graph"]["nodes"][lower_index]["node_id"];
            let stronger = &variant["semantic_graph"]["nodes"][index]["node_id"];
            (id_order(lower, stronger).is_lt()).then_some(variant)
        })
        .next()
        .expect("valid derived keys with the weaker frame first");
    let mut package = package;
    refresh_identity(&mut package);
    let lower_id = &package["semantic_graph"]["nodes"][lower_index]["node_id"];
    let stronger_id = &package["semantic_graph"]["nodes"][index]["node_id"];
    assert!(
        id_order(lower_id, stronger_id).is_lt(),
        "weaker defect must sort first"
    );

    expect_refusal_at(
        &package,
        CheckedPackageRefusalCode::InvalidSemanticGraph,
        &frame_body(lower_index),
        None,
        lower_id,
    );
}
