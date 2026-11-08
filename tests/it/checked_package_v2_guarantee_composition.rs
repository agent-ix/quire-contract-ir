// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-474: the V2 reader admits QSpec FR-490's `correspondence`/
//! `implementation_guarantee` body and FR-491's `correspondence`/
//! `component_composition` body, and refuses every other shape with the code,
//! cause and path FR-362 names.
//!
//! Every case edits the in-repo all-families fixture (selecting the
//! infinite-trace profile, so its temporal clause is a guarantee target) over
//! the orders domain package, adds an abstraction relation node, and adds the
//! guarantee or composition node under test.

use super::checked_package_v2_temporal::{
    admitted, find_form, model_key, node_id, nodes, read, with_model, INFINITE, ORDER_NODE,
    SUB_NODE,
};
use crate::support::checked_package::{node_id as fixture_node_id, settle, v2_all_families};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedNodeId, CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code,
    CheckedPackageV2ReadResult, CheckedTerm,
};
use serde_json::{json, Value};
use std::collections::BTreeSet;

fn id_of(label: &str) -> Value {
    fixture_node_id(label)
}

fn typed(id: &Value) -> CheckedNodeId {
    serde_json::from_value(id.clone()).expect("node id")
}

/// The context, clause and relation targets of the fixture.
struct Fixture {
    package: Value,
    order: Value,
    clause: Value,
    relation: Value,
}

fn node(path: &str, form: &str, body: Value, targets: &[&Value]) -> Value {
    let id = id_of(path);
    let dependencies = targets
        .iter()
        .map(|target| typed(target))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|id| serde_json::to_value(id).expect("id"))
        .collect::<Vec<_>>();
    json!({
        "node_id": id,
        "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": "correspondence",
        "semantic_form": form,
        "semantic_type": id,
        "dependencies": dependencies,
        "occurrences": [{"role": "declaration", "ordinal": 0}],
        "body": body,
    })
}

fn fixture() -> Fixture {
    let mut package = v2_all_families();
    super::checked_package_v2_temporal::select_profile(&mut package, INFINITE);
    with_model(&mut package);
    let clause =
        nodes(&package)[find_form(&package, "temporal", "temporal_clause")]["node_id"].clone();
    let order = node_id(&model_key(ORDER_NODE));
    let relation = node(
        "relation/0",
        "abstraction_relation",
        json!({"term": "abstraction_relation", "objects": [], "populations": [], "frames": []}),
        &[],
    );
    let relation_id = relation["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(relation);
    Fixture {
        package,
        order,
        clause,
        relation: relation_id,
    }
}

fn scheduling(fixture: &Fixture, operation: &str) -> Value {
    json!({"kind": "weak", "granularity": "whole",
        "context": fixture.order, "operation": operation})
}

fn guarantee_body(fixture: &Fixture) -> Value {
    json!({
        "term": "implementation_guarantee",
        "clause": fixture.clause,
        "relation": fixture.relation,
        "initial": ["store", "init"],
        "progress": [
            {"context": fixture.order, "operation": "reset", "measure": {"bound": 2}},
            {"context": fixture.order, "operation": "scaled",
             "measure": {"decreases": ["store", "fuel"]}},
        ],
        "environment": [
            {"name": "sched", "supplier": "os",
             "supplies": {"fairness": scheduling(fixture, "scaled")}},
        ],
    })
}

fn targets_of(body: &Value) -> Vec<Value> {
    fn walk(value: &Value, found: &mut Vec<Value>) {
        match value {
            Value::Object(object) if object.contains_key("ordinal") => found.push(value.clone()),
            Value::Object(object) => object.values().for_each(|member| walk(member, found)),
            Value::Array(items) => items.iter().for_each(|item| walk(item, found)),
            _ => {}
        }
    }
    let mut found = Vec::new();
    walk(body, &mut found);
    found
}

/// The fixture holding the node under test, with its `dependencies` computed
/// from the body.
fn with_body(fixture: &Fixture, form: &str, body: Value) -> Value {
    let targets = targets_of(&body)
        .into_iter()
        .filter(|target| target["package"] != "acme/peer")
        .collect::<Vec<_>>();
    let refs = targets.iter().collect::<Vec<_>>();
    let mut package = fixture.package.clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(node("declaration/0", form, body, &refs));
    settle(&mut package);
    package
}

fn guarantee(fixture: &Fixture) -> Value {
    with_body(fixture, "implementation_guarantee", guarantee_body(fixture))
}

fn last(package: &Value) -> usize {
    nodes(package).len() - 1
}

fn refusal(case: &str, package: &Value) -> (Code, Option<Cause>, String) {
    match read(package) {
        CheckedPackageV2ReadResult::Refused(refusal) => (
            refusal.code,
            refusal.cause,
            refusal.loci[0]
                .path
                .as_ref()
                .map(|path| path.as_str().to_owned())
                .unwrap_or_default(),
        ),
        other => panic!("{case}: expected a refusal, read {other:?}"),
    }
}

fn expect(case: &str, package: &Value, code: Code, cause: Option<Cause>, suffix: &str) {
    let path = format!("/semantic_graph/nodes/{}{suffix}", last(package));
    assert_eq!(refusal(case, package), (code, cause, path), "{case}");
}

fn edited(fixture: &Fixture, form: &str, edit: impl FnOnce(&mut Value)) -> Value {
    let mut body = if form == "implementation_guarantee" {
        guarantee_body(fixture)
    } else {
        composition_body(fixture)
    };
    edit(&mut body);
    let mut package = with_body(fixture, form, body.clone());
    // `dependencies` follow the edited body, which `with_body` already did.
    settle(&mut package);
    package
}

fn composition_body(fixture: &Fixture) -> Value {
    let foreign =
        json!({"package": "acme/peer", "path": ["temporal", "temporal_clause", "g"], "ordinal": 0});
    json!({
        "term": "component_composition",
        "rule": "assume-guarantee",
        "components": [
            {"name": "consumer", "package": "acme/config", "subject": fixture.order},
            {"name": "producer", "package": "acme/peer", "subject": foreign},
        ],
        "guarantees": [
            {"component": "consumer", "clause": fixture.clause},
            {"component": "producer", "clause": foreign},
        ],
        "relies": [{"guarantee": {"component": "consumer", "clause": fixture.clause},
            "on": {"component": "producer", "clause": foreign}}],
        "liveness_dependencies": [],
        "supplies": [scheduling(fixture, "reset")],
        "environment": [],
    })
}

/// ACs: FR-362-AC-1
#[trace("TC-474", "FR-362-AC-1")]
#[test]
fn tc_474_an_implementation_guarantee_admits_and_reads_back_typed() {
    let fixture = fixture();
    let package = guarantee(&fixture);
    let admitted = admitted("guarantee", &package);
    let body = &nodes(&package)[last(&package)];
    let Some(CheckedTerm::ImplementationGuarantee {
        clause,
        relation,
        initial,
        progress,
        environment,
    }) = admitted.term(&typed(&body["node_id"]))
    else {
        panic!("admitted guarantee must have a typed body");
    };
    assert_eq!(clause, &typed(&fixture.clause));
    assert_eq!(relation, &typed(&fixture.relation));
    assert_eq!(
        initial.iter().map(AsRef::as_ref).collect::<Vec<_>>(),
        ["store", "init"]
    );
    assert_eq!(progress.len(), 2);
    assert_eq!(environment.len(), 1);
    assert_eq!(
        serde_json::to_value(admitted.term(&typed(&body["node_id"]))).expect("typed body"),
        body["body"]
    );
}

/// ACs: FR-362-AC-2
#[trace("TC-474", "FR-362-AC-2")]
#[test]
fn tc_474_a_composition_admits_with_its_four_premise_lists_separate() {
    let fixture = fixture();
    let mut body = composition_body(&fixture);
    body["liveness_dependencies"] = body["relies"].clone();
    body["relies"] = json!([]);
    let package = with_body(&fixture, "component_composition", body.clone());
    let admitted = admitted("composition", &package);
    let id = typed(&nodes(&package)[last(&package)]["node_id"]);
    let Some(CheckedTerm::ComponentComposition {
        rule,
        components,
        guarantees,
        relies,
        liveness_dependencies,
        supplies,
        environment,
    }) = admitted.term(&id)
    else {
        panic!("admitted composition must have a typed body");
    };
    assert_eq!(rule.wire(), "assume-guarantee");
    assert_eq!(components.len(), 2);
    assert_eq!(guarantees.len(), 2);
    assert!(relies.is_empty());
    assert_eq!(liveness_dependencies.len(), 1);
    assert_eq!(supplies.len(), 1);
    assert!(environment.is_empty());
    assert_eq!(
        serde_json::to_value(admitted.term(&id)).expect("typed body"),
        body
    );
}

/// ACs: FR-362-AC-3
#[trace("TC-474", "FR-362-AC-3")]
#[test]
fn tc_474_a_body_off_its_closed_shape_refuses_at_the_body() {
    let fixture = fixture();
    let guarantee_cases: [(&str, fn(&mut Value)); 8] = [
        ("extra member", |b| b["extra"] = json!(1)),
        ("missing initial", |b| {
            b.as_object_mut().expect("object").remove("initial");
        }),
        ("empty initial path", |b| b["initial"] = json!([])),
        ("bound zero", |b| {
            b["progress"][0]["measure"] = json!({"bound": 0})
        }),
        ("both measures", |b| {
            b["progress"][0]["measure"] = json!({"bound": 1, "decreases": ["x"]});
        }),
        ("non-identifier operation", |b| {
            b["progress"][0]["operation"] = json!("9lives")
        }),
        ("empty supplier", |b| {
            b["environment"][0]["supplier"] = json!("")
        }),
        ("wrong term", |b| b["term"] = json!("component_composition")),
    ];
    for (case, edit) in guarantee_cases {
        let package = edited(&fixture, "implementation_guarantee", edit);
        expect(case, &package, Code::InvalidSemanticGraph, None, "/body");
    }
    let composition_cases: [(&str, fn(&mut Value)); 3] = [
        ("one component", |b| {
            b["components"].as_array_mut().expect("array").pop();
        }),
        ("unknown rule", |b| b["rule"] = json!("magic")),
        ("missing supplies", |b| {
            b.as_object_mut().expect("object").remove("supplies");
        }),
    ];
    for (case, edit) in composition_cases {
        let package = edited(&fixture, "component_composition", edit);
        expect(case, &package, Code::InvalidSemanticGraph, None, "/body");
    }
}

/// ACs: FR-362-AC-4
#[trace("TC-474", "FR-362-AC-4")]
#[test]
fn tc_474_identity_and_canonical_order_refuse_at_the_node_and_the_array() {
    let fixture = fixture();
    let mut stale = guarantee(&fixture);
    let at = last(&stale);
    stale["semantic_graph"]["nodes"][at]["dependencies"] = json!([]);
    settle(&mut stale);
    expect("dependencies", &stale, Code::InvalidSemanticGraph, None, "");

    let swapped = edited(&fixture, "implementation_guarantee", |b| {
        b["progress"].as_array_mut().expect("array").reverse();
    });
    expect(
        "progress order",
        &swapped,
        Code::InvalidSemanticGraph,
        None,
        "/body/progress",
    );

    let mut named = guarantee_body(&fixture);
    named["environment"] = json!([
        {"name": "b", "supplier": "os", "supplies": {"fairness": scheduling(&fixture, "scaled")}},
        {"name": "a", "supplier": "os", "supplies": {"fairness": scheduling(&fixture, "scaled")}},
    ]);
    let package = with_body(&fixture, "implementation_guarantee", named);
    expect(
        "environment order",
        &package,
        Code::InvalidSemanticGraph,
        None,
        "/body/environment",
    );

    let package = edited(&fixture, "component_composition", |b| {
        b["components"].as_array_mut().expect("array").reverse();
    });
    expect(
        "components order",
        &package,
        Code::InvalidSemanticGraph,
        None,
        "/body/components",
    );
}

/// ACs: FR-362-AC-5
#[trace("TC-474", "FR-362-AC-5")]
#[test]
fn tc_474_a_target_of_the_wrong_kind_refuses_as_a_malformed_declaration() {
    let fixture = fixture();
    let malformed = |case: &str, package: Value, suffix: &str| {
        expect(
            case,
            &package,
            Code::InvalidModelBinding,
            Some(Cause::MalformedDeclaration),
            suffix,
        )
    };
    // A relation naming an object type, a clause naming the relation node, a
    // progress context naming the temporal clause.
    let package = edited(&fixture, "implementation_guarantee", |b| {
        b["relation"] = fixture.order.clone()
    });
    malformed("relation kind", package, "/body/relation");
    let package = edited(&fixture, "implementation_guarantee", |b| {
        b["clause"] = fixture.relation.clone()
    });
    malformed("clause kind", package, "/body/clause");
    let package = edited(&fixture, "implementation_guarantee", |b| {
        b["progress"] = json!([{"context": fixture.clause, "operation": "reset",
            "measure": {"bound": 2}}]);
    });
    malformed("context kind", package, "/body/progress/0");
    let package = edited(&fixture, "implementation_guarantee", |b| {
        b["environment"][0]["supplies"] = json!({"assumption": fixture.order});
    });
    malformed("assumption kind", package, "/body/environment/0/supplies");
    let package = edited(&fixture, "component_composition", |b| {
        b["guarantees"][0]["clause"] = fixture.order.clone();
    });
    malformed("composition clause kind", package, "/body/guarantees/0");
}

/// ACs: FR-362-AC-6
#[trace("TC-474", "FR-362-AC-6")]
#[test]
fn tc_474_members_refuse_unknown_operations_bad_paths_and_repeated_keys() {
    let fixture = fixture();
    let package = edited(&fixture, "implementation_guarantee", |b| {
        b["progress"][0]["operation"] = json!("noSuchOperation");
    });
    expect(
        "unknown operation",
        &package,
        Code::MissingDeclaration,
        Some(Cause::MissingName),
        "/body/progress/0",
    );
    let package = edited(&fixture, "implementation_guarantee", |b| {
        b["initial"] = json!(["crate", "init"])
    });
    expect(
        "keyword path",
        &package,
        Code::InvalidModelBinding,
        Some(Cause::MalformedDeclaration),
        "/body/initial",
    );
    let package = edited(&fixture, "implementation_guarantee", |b| {
        b["progress"][1]["operation"] = json!("reset");
    });
    expect(
        "repeated progress",
        &package,
        Code::InvalidModelBinding,
        Some(Cause::ConflictingBinding),
        "/body/progress/1",
    );
    let package = edited(&fixture, "implementation_guarantee", |b| {
        let first = b["environment"][0].clone();
        b["environment"].as_array_mut().expect("array").push(first);
    });
    expect(
        "repeated environment",
        &package,
        Code::InvalidModelBinding,
        Some(Cause::ConflictingBinding),
        "/body/environment/1",
    );
    let package = edited(&fixture, "component_composition", |b| {
        b["components"][1]["name"] = json!("consumer");
    });
    expect(
        "repeated component",
        &package,
        Code::InvalidModelBinding,
        Some(Cause::ConflictingBinding),
        "/body/components/1",
    );
    // An operation the context only inherits is the context's, not Sub's.
    let sub = node_id(&model_key(SUB_NODE));
    let package = edited(&fixture, "implementation_guarantee", |b| {
        b["progress"] = json!([{"context": sub, "operation": "reset", "measure": {"bound": 2}}]);
    });
    expect(
        "inherited operation",
        &package,
        Code::InvalidModelBinding,
        Some(Cause::MalformedDeclaration),
        "/body/progress/0",
    );
}
