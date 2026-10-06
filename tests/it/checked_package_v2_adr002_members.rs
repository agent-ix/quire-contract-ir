// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-222: a V2 document attempting to carry an ADR-002 2.0.0 member the
//! reader has no admission path for is refused wholesale with a typed code
//! at the value at fault, never admitted with the member dropped.
//!
//! Every case mutates the in-repo `v2_all_families()` package.

use crate::support::checked_package::{canonical, evidence_for, refresh_identity, v2_all_families};
use ix_trace_rs::trace;
use quire_contract_ir::{
    CheckedPackageReadLimits, CheckedPackageRefusal, CheckedPackageRefusalCode, CheckedPackageV2,
    CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};

fn refused(value: &Value) -> CheckedPackageRefusal {
    match CheckedPackageV2::read(
        &canonical(value),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(value),
    ) {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        other => panic!("expected a refusal, read {other:?}"),
    }
}

fn position(value: &Value, tag: &str, form: &str) -> usize {
    value["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_tag"] == tag && node["semantic_form"] == form)
        .unwrap_or_else(|| panic!("a {tag}/{form} node"))
}

fn assert_refusal(case: &str, value: &Value, code: CheckedPackageRefusalCode, path: &str) {
    let refusal = refused(value);
    assert_eq!(refusal.code, code, "{case}: {refusal:?}");
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(path),
        "{case}: {refusal:?}"
    );
}

/// The all-families package with the node at `(tag, form)` edited and the
/// identity re-derived, and that node's position.
fn mutated(tag: &str, form: &str, edit: impl FnOnce(&mut Value)) -> (Value, usize) {
    let mut value = v2_all_families();
    let at = position(&value, tag, form);
    edit(&mut value["semantic_graph"]["nodes"][at]);
    refresh_identity(&mut value);
    (value, at)
}

/// FCD's published population shape: no `term` member.
fn population_body() -> Value {
    json!({
        "identity": "Accounts",
        "displayName": "Accounts",
        "kind": {"kind": "entity"},
        "members": [],
        "extent": "closed",
        "origin": {"kind": "declared"},
    })
}

/// Tracing: TC-222
/// ACs: FR-344-AC-1
#[trace("TC-222", "FR-344-AC-1")]
#[test]
fn tc_222_an_unrecognized_tag_or_form_refuses_at_the_graph_gate() {
    let (value, at) = mutated("model", "namespace", |node| {
        node["node_tag"] = json!("supertype")
    });
    assert_refusal(
        "synthetic node_tag",
        &value,
        CheckedPackageRefusalCode::UnsupportedNodeTag,
        &format!("/semantic_graph/nodes/{at}/node_tag"),
    );
    let (value, at) = mutated("model", "namespace", |node| {
        node["semantic_form"] = json!("abstract_flag");
    });
    assert_refusal(
        "synthetic semantic_form",
        &value,
        CheckedPackageRefusalCode::InvalidSemanticGraph,
        &format!("/semantic_graph/nodes/{at}/semantic_form"),
    );
}

/// Tracing: TC-222
/// ACs: FR-344-AC-2
#[trace("TC-222", "FR-344-AC-2")]
#[test]
fn tc_222_a_population_body_outside_the_closed_term_grammar_refuses_at_the_body() {
    let (value, at) = mutated("relation", "relationship", |node| {
        node["semantic_form"] = json!("population");
        node.as_object_mut().expect("node").remove("owner");
        node["body"] = population_body();
    });
    let body = format!("/semantic_graph/nodes/{at}/body");
    assert_refusal(
        "population body with no term member",
        &value,
        CheckedPackageRefusalCode::InvalidSemanticGraph,
        &body,
    );
    let (value, _) = mutated("relation", "relationship", |node| {
        node["semantic_form"] = json!("population");
        node.as_object_mut().expect("node").remove("owner");
        let mut with_term = population_body();
        with_term["term"] = json!("aggregate");
        node["body"] = with_term;
    });
    assert_refusal(
        "the same body with an aggregate term beside its members",
        &value,
        CheckedPackageRefusalCode::InvalidSemanticGraph,
        &body,
    );
}

/// Tracing: TC-222
/// ACs: FR-344-AC-3
#[trace("TC-222", "FR-344-AC-3")]
#[test]
fn tc_222_an_extra_node_member_refuses_as_unknown_member_in_the_closed_decode() {
    for member in ["subsets", "redefines"] {
        // The member is added after the identity projection is derived, so
        // the projection does not carry a copy the decoder would reach first.
        let mut value = v2_all_families();
        let at = position(&value, "model", "object_type");
        value["semantic_graph"]["nodes"][at][member] = json!([]);
        assert_refusal(
            member,
            &value,
            CheckedPackageRefusalCode::UnknownMember,
            &format!("/semantic_graph/nodes/{at}/{member}"),
        );
    }
}
