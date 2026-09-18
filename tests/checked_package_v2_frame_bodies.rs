// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-340 frame-body semantics: entry eligibility, canonical member order,
//! and cross-defect refusal precedence for `quire.checked-package/v2`'s
//! `state`/`frame` nodes.

#[path = "support/checked_package.rs"]
mod checked_package;

use checked_package::{
    canonical, evidence_for, frame_refusal_cause, frame_refusal_code, node_id,
    node_identity_vectors, refresh_identity, typed_node_id, v2_all_families,
};
use ix_trace_rs::trace;
use quire_contract_ir::{
    CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageRefusal,
    CheckedPackageRefusalCode, CheckedPackageV2, CheckedPackageV2ReadResult,
};
use serde_json::Value;

/// Mirrors the reader's private `BODY_MODIFIES_PATH` (`v2/mod.rs`).
const BODY_MODIFIES_PATH: &str = "semantic_graph.nodes.body.modifies";
/// Mirrors the reader's private `BODY_CREATES_PATH` (`v2/mod.rs`).
const BODY_CREATES_PATH: &str = "semantic_graph.nodes.body.creates";
/// Mirrors the reader's private `BODY_DELETES_PATH` (`v2/mod.rs`).
const BODY_DELETES_PATH: &str = "semantic_graph.nodes.body.deletes";
/// Mirrors the reader's private canonical-order refusal path (`v2/mod.rs`).
const BODY_ORDER_PATH: &str = "semantic_graph.nodes.body";

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

/// The single `state`/`frame` node in `v2_all_families()`.
fn frame_index(package: &Value) -> usize {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_tag"] == "state" && node["semantic_form"] == "frame")
        .expect("base fixture carries exactly one frame node")
}

fn node_refs(vector: &Value, key: &str) -> Value {
    Value::Array(
        vector[key]
            .as_array()
            .unwrap_or_else(|| panic!("vector member {key}"))
            .iter()
            .map(|digest| node_id(digest.as_str().expect("digest string")))
            .collect(),
    )
}

/// Replaces `v2_all_families()`'s one frame node's `dependencies` and body
/// members with one `frame_mutations` vector's, splicing in `second_frame`
/// verbatim when the vector carries one (`cross-frame-lower-digest-wins`),
/// then rederives package identity.
fn apply_frame_mutation(base: &Value, vector: &Value) -> Value {
    let mut package = base.clone();
    let index = frame_index(&package);
    package["semantic_graph"]["nodes"][index]["dependencies"] = node_refs(vector, "dependencies");
    package["semantic_graph"]["nodes"][index]["body"]["modifies"] = node_refs(vector, "modifies");
    package["semantic_graph"]["nodes"][index]["body"]["creates"] = node_refs(vector, "creates");
    package["semantic_graph"]["nodes"][index]["body"]["deletes"] = node_refs(vector, "deletes");
    if let Some(second_frame) = vector.get("second_frame") {
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(second_frame.clone());
    }
    refresh_identity(&mut package);
    package
}

/// The exact structural path FR-340 reports a vector's refusal at: the
/// constant order-defect path for `invalid_semantic_graph`, or the member
/// whose array (`modifies`/`creates`/`deletes`) names the expected locus for
/// a meaning-join defect (`missing_declaration`/`invalid_model_binding`).
fn expected_path(vector: &Value, code: CheckedPackageRefusalCode) -> &'static str {
    if code == CheckedPackageRefusalCode::InvalidSemanticGraph {
        return BODY_ORDER_PATH;
    }
    let locus = vector["expected_locus_digest"]
        .as_str()
        .expect("expected_locus_digest");
    for (member, path) in [
        ("modifies", BODY_MODIFIES_PATH),
        ("creates", BODY_CREATES_PATH),
        ("deletes", BODY_DELETES_PATH),
    ] {
        let names_locus = vector[member]
            .as_array()
            .unwrap_or_else(|| panic!("vector member {member}"))
            .iter()
            .any(|digest| digest.as_str() == Some(locus));
        if names_locus {
            return path;
        }
    }
    panic!(
        "{}: expected_locus_digest {locus} names no frame member",
        vector["name"]
    );
}

/// Replays every vendored `frame_mutations` vector against
/// `v2_all_families()`, asserting the exact refused code, cause and locus
/// digest FR-340 pins for each, and that every published vector was
/// exercised (not merely a prefix of them).
///
/// Tracing: TC-053, FR-038-AC-13, FR-038-AC-14, FR-038-AC-15
#[trace("TC-053", "FR-038-AC-13", "FR-038-AC-14", "FR-038-AC-15")]
#[test]
fn tc_053_frame_mutations_vectors_refuse_the_published_code_cause_and_locus() {
    let document = node_identity_vectors();
    let vectors = document["frame_mutations"]
        .as_array()
        .expect("frame_mutations");
    let published = vectors.len();
    assert_eq!(published, 26, "published frame_mutations vector count");
    let base = v2_all_families();

    let mut replayed = 0_usize;
    for vector in vectors {
        let name = vector["name"].as_str().expect("name");
        let package = apply_frame_mutation(&base, vector);
        let refusal = refused(&package);

        let expected_code =
            frame_refusal_code(vector["expected_code"].as_str().expect("expected_code"));
        let expected_cause = frame_refusal_cause(vector["expected_cause"].as_str());
        let expected_locus = vector["expected_locus_digest"]
            .as_str()
            .expect("expected_locus_digest");
        let expected_path = expected_path(vector, expected_code);

        assert_eq!(refusal.code, expected_code, "{name} code");
        assert_eq!(refusal.cause, expected_cause, "{name} cause");
        assert_eq!(
            refusal.locus,
            Some(typed_node_id(expected_locus)),
            "{name} locus"
        );
        assert_eq!(refusal.path.as_ref(), expected_path, "{name} path");
        replayed += 1;
    }
    assert_eq!(
        replayed, published,
        "every published frame_mutations vector was replayed"
    );
}

/// The published `v2_all_families()` frame node already exercises 4 of the
/// 6 eligible `(member, tag, form)` triples (`relationship`/`field_declaration`
/// in `modifies`, `object_type` in `creates`, `process` in `deletes`); this
/// covers the remaining two — `process` in `creates` and `object_type` in
/// `deletes` — by reassigning the same two already-declared dependencies to
/// the other eligible member, so the whole package still admits.
///
/// Tracing: TC-053, FR-038-AC-12
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

    let admitted = admitted(&package);
    assert_eq!(admitted.graph().nodes.len(), 26);
}
