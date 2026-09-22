// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-340 frame-body semantics: entry eligibility, canonical member order,
//! and cross-defect refusal precedence for `quire.checked-package/v2`'s
//! `state`/`frame` nodes.

use crate::support::checked_package::{canonical, evidence_for, refresh_identity, refusal_at, v2_all_families};
use ix_trace_rs::trace;
use quire_contract_ir::{
    CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageRefusal,
    CheckedPackageRefusalCause, CheckedPackageRefusalCode, CheckedPackageV2,
    CheckedPackageV2ReadResult,
};
use serde_json::Value;

/// Mirrors the reader's private `BODY_MODIFIES_PATH` (`v2/mod.rs`).
const BODY_MODIFIES_PATH: &str = "semantic_graph.nodes.body.modifies";

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

// `tc_053_frame_mutations_vectors_refuse_the_published_code_cause_and_locus`
// was removed here: it replayed the `frame_mutations` array inside
// `node_identity_vectors()`, the independent node-identity conformance oracle
// copied from a private upstream repository and deleted along with the rest
// of the private-sourced fixture tree — the issue's own account names this
// exact test as one to remove rather than rewrite (AGE-1961).
// FR-340's frame-body precedence rules (canonical order, meaning-join
// eligibility, and meaning-join-outranks-order) are still exercised, locally
// authored rather than vendored, by the two tests below.

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
    // 13 public-family nodes plus 5 supporting nodes: a second `pure_function`
    // the `function` family's `application` argument names, the
    // `field_declaration`/`object_type`/`process` triple the frame's own
    // `modifies`/`creates`/`deletes` name, and the dedicated `state`/`frame`
    // node itself (`build_v2_all_families` in `tests/support/checked_package.rs`).
    assert_eq!(admitted.graph().nodes.len(), 18);
}

/// FR-340's headline precedence rule (`spec/contract/FR-038-consume-checked-package-v2.md`:
/// "A frame node carrying more than one defect refuses for exactly one of
/// them ... any meaning-join defect ... outranks a canonical-order defect
/// outright") has no vendored vector that puts both defect classes in one
/// frame: every vector `tc_053_frame_mutations_vectors_refuse_the_published_code_cause_and_locus`
/// replays carries either a meaning-join defect or a canonical-order defect,
/// never both. This case is authored locally, not vendored, to close that
/// gap.
///
/// It narrows the frame's `dependencies` to the fixture's own `object_type`
/// and `process` nodes, places `object_type` — ineligible for `modifies`,
/// which admits only `relationship`/`field_declaration` — into `modifies`
/// (a meaning-join defect), and places both entries into `deletes` in
/// descending digest order (`process` before `object_type`; both are
/// individually eligible for `deletes`, so this is a pure canonical-order
/// defect there, not a second meaning-join defect). FR-340 requires the
/// meaning-join defect to win: `invalid_model_binding`/`malformed-declaration`
/// located at `object_type`, never `invalid_semantic_graph` at the frame
/// node for the `deletes` misorder.
///
/// Tracing: TC-053, FR-038-AC-14
#[trace("TC-053", "FR-038-AC-14")]
#[test]
fn tc_053_meaning_join_defect_outranks_a_co_occurring_order_defect() {
    let base = v2_all_families();
    let index = frame_index(&base);
    let object_type = base["semantic_graph"]["nodes"][index]["body"]["creates"][0].clone();
    let process = base["semantic_graph"]["nodes"][index]["body"]["deletes"][0].clone();
    let object_type_digest = object_type["digest"]
        .as_str()
        .expect("digest string")
        .to_owned();

    let mut package = base.clone();
    package["semantic_graph"]["nodes"][index]["dependencies"] =
        Value::Array(vec![object_type.clone(), process.clone()]);
    package["semantic_graph"]["nodes"][index]["body"]["modifies"] =
        Value::Array(vec![object_type.clone()]);
    package["semantic_graph"]["nodes"][index]["body"]["creates"] = Value::Array(vec![]);
    // Descending digest order: `process` ("92...") before `object_type`
    // ("91...") — a canonical-order defect, since both entries are
    // individually eligible for `deletes`.
    package["semantic_graph"]["nodes"][index]["body"]["deletes"] =
        Value::Array(vec![process, object_type]);
    refresh_identity(&mut package);

    let refusal = refused(&package);
    assert_eq!(
        refusal,
        refusal_at(
            CheckedPackageRefusalCode::InvalidModelBinding,
            BODY_MODIFIES_PATH,
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            &object_type_digest,
        ),
        "meaning-join defect must win over the co-occurring order defect"
    );
}
