// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Literal nodes have one literal body and no wire dependencies.

use crate::support::checked_package::{
    canonical, evidence_for, family_key, node_id, v2_all_families,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedPackageReadLimits, CheckedPackageRefusalCode, CheckedPackageV2,
    CheckedPackageV2ReadResult, CheckedTerm, LiteralKind,
};
use serde_json::{json, Value};

fn read(value: &Value) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(
        &canonical(value),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(value),
    )
}

fn literal_position(value: &Value) -> usize {
    value["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_tag"] == "value" && node["semantic_form"] == "literal")
        .expect("literal node")
}

fn assert_refused(value: &Value, position: usize, member: &str) {
    match read(value) {
        CheckedPackageV2ReadResult::Refused(refusal) => {
            assert_eq!(
                refusal.code,
                if member == "body" {
                    CheckedPackageRefusalCode::MalformedWire
                } else {
                    CheckedPackageRefusalCode::InvalidSemanticGraph
                }
            );
            assert_eq!(refusal.cause, None);
            assert_eq!(
                refusal.loci[0].path.as_ref().expect("pointer").as_str(),
                format!("/semantic_graph/nodes/{position}/{member}")
            );
        }
        other => panic!("literal rule must refuse without an admitted view: {other:?}"),
    }
}

#[trace("FR-038-AC-472")]
#[test]
fn typed_literal_with_empty_dependencies_preserves_its_body_and_type() {
    let value = v2_all_families();
    let position = literal_position(&value);
    let CheckedPackageV2ReadResult::Admitted(package) = read(&value) else {
        panic!("valid literal package must admit");
    };
    let node = &package.graph().nodes[position];
    assert!(node.dependencies.is_empty());
    assert_eq!(
        node.semantic_type,
        serde_json::from_value(node_id(&family_key("aaaa"))).expect("type")
    );
    let CheckedTerm::Literal {
        type_id,
        value_kind,
        value: body_value,
    } = &node.body
    else {
        panic!("admission must retain the literal");
    };
    assert_eq!(type_id, &node.semantic_type);
    assert_eq!(*value_kind, LiteralKind::Boolean);
    assert_eq!(
        serde_json::to_value(body_value).expect("JSON literal"),
        json!(true)
    );
}

#[trace("FR-038-AC-472")]
#[test]
fn every_nonliteral_body_refuses_at_the_literal_body_before_graph_checks() {
    let base = v2_all_families();
    let position = literal_position(&base);
    let target = node_id(&family_key("aaaa"));
    // All nonliteral terms in the published SemanticTerm vocabulary, plus
    // the closed body forms used by other node families. The application is
    // copied from this admitted IR fixture, not from an external corpus.
    let application = base["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["body"]["term"] == "application")
        .expect("application")["body"]
        .clone();
    for body in [
        json!({"term": "reference", "target": target}),
        json!({"term": "dependency_reference", "package": "absent", "node": target}),
        json!({"term": "aggregate", "members": []}),
        json!({"term": "aggregate", "members": [base["semantic_graph"]["nodes"][position]["body"].clone()]}),
        json!({"term": "binding", "name": "name", "value": base["semantic_graph"]["nodes"][position]["body"].clone()}),
        application,
        json!({"term": "frame", "modifies": [], "creates": [], "deletes": [], "deletes_scoped": []}),
        json!({"term": "abstraction_relation", "objects": [], "populations": [], "frames": []}),
        json!({"term": "refinement"}),
        json!({"term": "extension_form", "clauses": []}),
        json!({"term": "enum", "ordered": false, "members": ["one"]}),
        json!({"term": "dimension", "base": true, "terms": []}),
        json!({"term": "unit"}),
        json!({"term": "dimension_equivalence"}),
    ] {
        let mut value = base.clone();
        value["semantic_graph"]["nodes"][position]["body"] = body;
        assert_refused(&value, position, "body");
        value["semantic_graph"]["nodes"][position]["dependencies"] = json!([target]);
        assert_refused(&value, position, "body");
    }
}

#[trace("FR-038-AC-472")]
#[test]
fn literal_dependencies_refuse_even_when_the_dependency_is_its_type() {
    let mut value = v2_all_families();
    let position = literal_position(&value);
    value["semantic_graph"]["nodes"][position]["dependencies"] =
        json!([node_id(&family_key("aaaa"))]);
    assert_refused(&value, position, "dependencies");
}
