// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-474: the V2 reader admits QSpec FR-490's `correspondence`/
//! `implementation_guarantee` body and FR-491's `correspondence`/
//! `component_composition` body, and refuses every other shape with the code,
//! cause and locus FR-038 "Implementation guarantee and component composition
//! bodies" names, in reader order.
//!
//! Every package is built in-repo from the temporal fixture of
//! `checked_package_v2_temporal` (a clause selecting the infinite-trace
//! profile, the `Order` and `Sub` object types and their domain package) plus
//! an empty abstraction relation node, a `predicate` node and the node under
//! test. No QSL value is used: the node shapes are QSpec's.

use crate::checked_package_v2_temporal::{
    clause, model_key, node_id, node_with_key, orders_document, push, read, select_profile,
    with_model, INFINITE, LEFT_NODE, ORDERS, ORDER_NODE, SUB_NODE,
};
use crate::support::checked_package::{
    canonical, evidence_with_document, index_fixture_node_ids, settle, v2_all_families,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedNodeId, CheckedNodeKind, CheckedPackageReadLimits, CheckedPackageRefusalCause as Cause,
    CheckedPackageRefusalCode as Code, CheckedPackageV2, CheckedPackageV2ReadResult, CheckedTerm,
    CorrespondenceForm,
};
use serde_json::{json, Value};

const GUARANTEE: &str = "guarantee/G";
const COMPOSITION: &str = "composition/S";
const PREDICATE: &str = "guarantee/predicate";
/// The package the fixture's nodes are in, and the component package a
/// composition imports.
const DEPENDENCY: &str = "test/families";
const IMPORTER: &str = "test/importer";

/// A node of the importing package.
fn importer_id(label: &str) -> Value {
    let mut id = node_id(label);
    id["package"] = json!(IMPORTER);
    id
}

/// Moves every structural node id of `value` from the dependency's package
/// to the importer's.
fn rekey(value: &mut Value) {
    match value {
        Value::Object(object) => {
            if object.get("package") == Some(&json!(DEPENDENCY))
                && object.contains_key("path")
                && object.contains_key("ordinal")
            {
                object.insert("package".to_owned(), json!(IMPORTER));
            }
            object.values_mut().for_each(rekey);
        }
        Value::Array(items) => items.iter_mut().for_each(rekey),
        _ => {}
    }
}

/// The fixture as a package of its own that imports the same fixture, under
/// another name, as the one component package: the importer's nodes are in
/// `IMPORTER`, a composition's subjects and clauses in `DEPENDENCY`.
fn importer() -> Value {
    let mut package = base();
    rekey(&mut package);
    package["name"] = json!(IMPORTER);
    package["lock"]["dependency_selections"] = json!([{"identity": DEPENDENCY}]);
    settle(&mut package);
    package
}

/// Reads `package`, supplying the component package when it holds a
/// composition.
fn read_any(package: &Value) -> CheckedPackageV2ReadResult {
    let composes = nodes(package)
        .iter()
        .any(|node| node["semantic_form"] == "component_composition");
    if composes {
        read_composition_with_dependency(package, &base())
    } else {
        read(package)
    }
}
fn read_composition_with_dependency(
    package: &Value,
    component: &Value,
) -> CheckedPackageV2ReadResult {
    let dependency = match read(component) {
        CheckedPackageV2ReadResult::Admitted(dependency) => *dependency,
        other => panic!("the component package admits, read {other:?}"),
    };
    let mut evidence = evidence_with_document(package, ORDERS, &orders_document());
    evidence
        .insert_dependency_package(DEPENDENCY, dependency)
        .expect("dependency evidence agrees on admitted content");
    CheckedPackageV2::read(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    )
}

fn order() -> Value {
    node_id(&model_key(ORDER_NODE))
}

fn sub() -> Value {
    node_id(&model_key(SUB_NODE))
}

fn relation() -> Value {
    node_id("guarantee/relation")
}

fn predicate() -> Value {
    node_id("guarantee/predicate")
}

fn nodes(package: &Value) -> &Vec<Value> {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
}

/// The clause node's id in `package`.
fn clause_id(package: &Value) -> Value {
    nodes(package)[clause(package)]["node_id"].clone()
}

fn typed(id: &Value) -> CheckedNodeId {
    serde_json::from_value(id.clone()).expect("structural node id")
}

/// The fixture every case starts from: the clause under the infinite-trace
/// profile, the model nodes, an empty abstraction relation and a predicate.
fn base() -> Value {
    let mut package = v2_all_families();
    select_profile(&mut package, INFINITE);
    with_model(&mut package);
    push(
        &mut package,
        node_with_key(
            "guarantee/relation",
            "correspondence",
            "abstraction_relation",
            json!({"term": "abstraction_relation", "objects": [], "populations": [],
                "frames": []}),
        ),
    );
    // A predicate is a function node; an application body leaves it unowned.
    let mut predicate = nodes(&package)
        .iter()
        .find(|node| node["node_tag"] == "function" && node["body"]["term"] == "application")
        .expect("the fixture carries a function application")
        .clone();
    predicate["node_id"] = node_id("guarantee/predicate");
    predicate["semantic_form"] = json!("predicate");
    push(&mut package, predicate);
    // Give the clause an actual model subject rather than the general fixture's
    // Boolean parameter. Other families retain their original parameter.
    let clause_at = clause(&package);
    let old_over = nodes(&package)[clause_at]["body"]["arguments"][0]["target"].clone();
    let mut over = nodes(&package)
        .iter()
        .find(|node| node["node_id"] == old_over)
        .expect("over parameter")
        .clone();
    over["node_id"] = node_id("guarantee/model-over");
    let reference = json!({"package": DEPENDENCY, "path": ["composite_type", "reference", order()], "ordinal": 0});
    push(
        &mut package,
        json!({
            "node_id": reference, "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": "composite_type", "semantic_form": "reference",
            "semantic_type": reference, "dependencies": [order()],
            "occurrences": [{"role": "type", "ordinal": 0}],
            "body": {"term": "aggregate", "members": [{"term": "reference", "target": order()}]},
        }),
    );
    over["semantic_type"] = reference;
    let over_id = over["node_id"].clone();
    push(&mut package, over);
    package["semantic_graph"]["nodes"][clause_at]["body"]["arguments"][0]["target"] =
        over_id.clone();
    let deps = &mut package["semantic_graph"]["nodes"][clause_at]["dependencies"];
    for id in deps.as_array_mut().expect("dependencies") {
        if *id == old_over {
            *id = over_id.clone();
        }
    }
    // A second clause has an unbounded eventually and is liveness; the
    // original bounded eventually is safety under the infinite-trace profile.
    let formula_id = nodes(&package)[clause_at]["body"]["arguments"][5]["target"].clone();
    let mut formula = nodes(&package)
        .iter()
        .find(|node| node["node_id"] == formula_id)
        .expect("formula")
        .clone();
    formula["node_id"] = node_id("guarantee/live-formula");
    formula["body"]["operation"]["member"]["interval"] = Value::Null;
    push(&mut package, formula);
    let mut live = nodes(&package)[clause_at].clone();
    live["node_id"] = node_id("guarantee/live-clause");
    live["body"]["arguments"][1]["value"] = json!("live_clause");
    live["body"]["arguments"][5]["target"] = node_id("guarantee/live-formula");
    live["dependencies"] = json!([over_id, node_id("guarantee/live-formula")]);
    push(&mut package, live);
    let relation_at = nodes(&package)
        .iter()
        .position(|node| node["node_id"] == relation())
        .expect("relation");
    let node = &mut package["semantic_graph"]["nodes"][relation_at];
    node["semantic_type"] = node["node_id"].clone();
    node["occurrences"] = json!([{"role": "declaration", "ordinal": 0}]);
    index_fixture_node_ids(&mut package);
    settle(&mut package);
    package
}

fn fairness(kind: &str, granularity: &str, context: &Value, operation: &str) -> Value {
    json!({"kind": kind, "granularity": granularity, "context": context,
        "operation": operation})
}

fn environment_fairness(name: &str, context: &Value, operation: &str) -> Value {
    json!({"name": name, "supplier": "scheduler",
        "supplies": {"fairness": fairness("weak", "whole", context, operation)}})
}

fn environment_assumption(name: &str, predicate: &Value) -> Value {
    json!({"name": name, "supplier": "operator", "supplies": {"assumption": predicate}})
}

fn guarantee_body(package: &Value) -> Value {
    json!({
        "term": "implementation_guarantee",
        "clause": clause_id(package),
        "relation": relation(),
        "initial": ["implementation", "initial"],
        "progress": [
            {"context": order(), "operation": "reset",
             "measure": {"decreases": ["implementation", "measure"]}},
            {"context": order(), "operation": "scaled", "measure": {"bound": 3}},
        ],
        "environment": [
            environment_assumption("env_ok", &predicate()),
            environment_fairness("fair_reset", &order(), "reset"),
        ],
    })
}

fn clause_entry(component: &str, clause: &Value) -> Value {
    json!({"component": component, "clause": clause})
}

fn premise(guarantee: (&str, &Value), on: (&str, &Value)) -> Value {
    json!({"guarantee": clause_entry(guarantee.0, guarantee.1),
        "on": clause_entry(on.0, on.1)})
}

fn composition_body(package: &Value) -> Value {
    let clause = clause_id(package);
    let live = node_id("guarantee/live-clause");
    json!({
        "term": "component_composition",
        "rule": "assume-guarantee",
        "components": [
            {"name": "Consumer", "package": DEPENDENCY, "subject": sub()},
            {"name": "Producer", "package": DEPENDENCY, "subject": order()},
        ],
        "guarantees": [
            clause_entry("Consumer", &clause),
            clause_entry("Producer", &live),
        ],
        "relies": [premise(("Consumer", &clause), ("Producer", &clause))],
        "liveness_dependencies": [premise(("Producer", &live), ("Consumer", &live))],
        "supplies": [
            fairness("strong", "each", &order(), "reset"),
            fairness("weak", "whole", &order(), "reset"),
        ],
        "environment": [environment_assumption("env_ok", &importer_id(PREDICATE))],
    })
}

/// Every target of `body`, unique and in structural order.
fn targets(body: &Value) -> Vec<Value> {
    fn collect(value: &Value, ids: &mut Vec<CheckedNodeId>) {
        match value {
            Value::Object(object) => {
                for (name, member) in object {
                    let node = matches!(
                        name.as_str(),
                        "clause" | "relation" | "context" | "assumption" | "subject"
                    );
                    if node && member.get("package").is_some() {
                        ids.push(typed(member));
                    } else {
                        collect(member, ids);
                    }
                }
            }
            Value::Array(items) => items.iter().for_each(|item| collect(item, ids)),
            _ => {}
        }
    }
    let mut ids = Vec::new();
    collect(body, &mut ids);
    ids.sort();
    ids.dedup();
    ids.iter()
        .map(|id| serde_json::to_value(id).expect("node id"))
        .collect()
}

fn node_of(label: &str, form: &str, body: Value) -> Value {
    let mut node = node_with_key(label, "correspondence", form, body.clone());
    node["semantic_type"] = node["node_id"].clone();
    node["occurrences"] = json!([{"role": "declaration", "ordinal": 0}]);
    node["dependencies"] = json!(targets(&body));
    node
}

fn guarantee_node(body: Value) -> Value {
    node_of(GUARANTEE, "implementation_guarantee", body)
}

fn composition_node(body: Value) -> Value {
    node_of(COMPOSITION, "component_composition", body)
}

/// `package` holding `node`, settled.
fn with(mut package: Value, node: Value) -> (Value, usize) {
    let position = push(&mut package, node);
    settle(&mut package);
    (package, position)
}

fn guaranteed(edit: impl FnOnce(&mut Value)) -> (Value, usize) {
    let package = base();
    let mut body = guarantee_body(&package);
    edit(&mut body);
    with(package, guarantee_node(body))
}

/// The importer holding the composition node `edit` changes after it is
/// built from `body`.
fn composed_with(body: Value, edit: impl FnOnce(&mut Value)) -> (Value, usize) {
    let mut node = composition_node(body);
    node["node_id"]["package"] = json!(IMPORTER);
    node["semantic_type"] = node["node_id"].clone();
    edit(&mut node);
    with(importer(), node)
}

fn composed(edit: impl FnOnce(&mut Value)) -> (Value, usize) {
    let mut body = composition_body(&base());
    edit(&mut body);
    composed_with(body, |_| {})
}

fn admitted(case: &str, package: &Value) -> CheckedPackageV2 {
    match read_any(package) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("{case}: expected admission, read {other:?}"),
    }
}

/// Asserts the refusal's code, cause and pointer.
fn expect(case: &str, package: &Value, code: Code, cause: Option<Cause>, path: &str) {
    let refusal = match read_any(package) {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        other => panic!("{case}: expected a refusal, read {other:?}"),
    };
    assert_eq!(refusal.code, code, "{case}: {refusal:?}");
    assert_eq!(refusal.cause, cause, "{case}: {refusal:?}");
    assert_eq!(
        refusal.loci[0].path.as_ref().map(|path| path.as_str()),
        Some(path),
        "{case}: {refusal:?}"
    );
}

fn at(position: usize, tail: &str) -> String {
    format!("/semantic_graph/nodes/{position}{tail}")
}

const GRAPH: Code = Code::InvalidSemanticGraph;
const MALFORMED: (Code, Option<Cause>) =
    (Code::InvalidModelBinding, Some(Cause::MalformedDeclaration));
const CONFLICT: (Code, Option<Cause>) =
    (Code::InvalidModelBinding, Some(Cause::ConflictingBinding));
const MISSING: (Code, Option<Cause>) = (Code::MissingDeclaration, Some(Cause::MissingName));

/// One refused case: how the body is edited, the code and cause, and the
/// pointer tail within the node.
type Case<'a> = (&'a str, fn(&mut Value), (Code, Option<Cause>), &'a str);

fn run_guarantee(cases: &[Case<'_>]) {
    for (name, edit, (code, cause), tail) in cases {
        let (package, position) = guaranteed(edit);
        expect(name, &package, *code, *cause, &at(position, tail));
    }
}

fn run_composition(cases: &[Case<'_>]) {
    for (name, edit, (code, cause), tail) in cases {
        let (package, position) = composed(edit);
        expect(name, &package, *code, *cause, &at(position, tail));
    }
}

#[trace("TC-474", "FR-038-AC-531")]
#[test]
fn tc_474_an_implementation_guarantee_node_admits_with_its_typed_term() {
    let (package, position) = guaranteed(|_| {});
    let admitted = admitted("guarantee", &package);
    assert_eq!(
        admitted.node_kinds()[position],
        CheckedNodeKind::Correspondence(CorrespondenceForm::ImplementationGuarantee)
    );
    let node = &admitted.graph().nodes[position];
    let Some(CheckedTerm::ImplementationGuarantee {
        initial,
        progress,
        environment,
        ..
    }) = admitted.term(&node.node_id)
    else {
        panic!("an admitted guarantee has a typed guarantee body");
    };
    assert_eq!(initial.len(), 2);
    assert_eq!(progress.len(), 2);
    assert_eq!(environment.len(), 2);
    assert_eq!(
        serde_json::to_value(&node.body).expect("typed body"),
        guarantee_body(&package)
    );
    // A safety guarantee needs no progress member and no environment member.
    let (bare, _) = guaranteed(|body| {
        body["progress"] = json!([]);
        body["environment"] = json!([]);
    });
    admitted_ok("a guarantee with empty lists", &bare);
    // A fairness context no progress member names is a dependency too.
    let (left, _) = guaranteed(|body| {
        body["environment"][1] = environment_fairness("fair_reset", &left_object(), "act");
    });
    admitted_ok("a fairness context of its own", &left);
}

fn left_object() -> Value {
    node_id(&model_key(LEFT_NODE))
}

fn admitted_ok(case: &str, package: &Value) {
    drop(admitted(case, package));
}

#[trace("TC-474", "FR-038-AC-532")]
#[test]
fn tc_474_a_component_composition_node_admits_with_its_typed_term() {
    let (package, position) = composed(|_| {});
    let admitted = admitted("composition", &package);
    assert_eq!(
        admitted.node_kinds()[position],
        CheckedNodeKind::Correspondence(CorrespondenceForm::ComponentComposition)
    );
    let node = &admitted.graph().nodes[position];
    let Some(CheckedTerm::ComponentComposition {
        rule,
        components,
        guarantees,
        relies,
        liveness_dependencies,
        supplies,
        environment,
    }) = admitted.term(&node.node_id)
    else {
        panic!("an admitted composition has a typed composition body");
    };
    assert_eq!(&**rule, "assume-guarantee");
    assert_eq!(components.len(), 2);
    assert_eq!(guarantees.len(), 2);
    // The premise lists stay separate.
    assert_eq!((relies.len(), liveness_dependencies.len()), (1, 1));
    assert_eq!((supplies.len(), environment.len()), (2, 1));
    assert_eq!(
        serde_json::to_value(&node.body).expect("typed body"),
        composition_body(&base())
    );
    for rule in ["protocol-refinement", "refinement-transfer", "interface"] {
        let (other, _) = composed(|body| body["rule"] = json!(rule));
        admitted_ok(rule, &other);
    }
}

#[trace("TC-474", "FR-038-AC-533")]
#[test]
fn tc_474_a_body_of_the_wrong_shape_refuses_at_the_offending_entry() {
    run_guarantee(&[
        (
            "a missing member",
            |body| {
                body.as_object_mut().expect("body").remove("initial");
            },
            (GRAPH, None),
            "/body",
        ),
        (
            "an extra member",
            |body| body["extra"] = json!(true),
            (GRAPH, None),
            "/body",
        ),
        (
            "a term of the other form",
            |body| body["term"] = json!("component_composition"),
            (GRAPH, None),
            "/body",
        ),
        (
            "an empty initial path",
            |body| body["initial"] = json!([]),
            (GRAPH, None),
            "/body",
        ),
        (
            "a measure of two members",
            |body| body["progress"][0]["measure"] = json!({"bound": 1, "decreases": ["m"]}),
            (GRAPH, None),
            "/body/progress/0/measure",
        ),
        (
            "an unknown fairness kind",
            |body| {
                body["environment"][1]["supplies"]["fairness"]["kind"] = json!("sometimes");
            },
            (GRAPH, None),
            "/body/environment/1/supplies/fairness",
        ),
        (
            "a supplies of neither form",
            |body| body["environment"][0]["supplies"] = json!({"other": predicate()}),
            (GRAPH, None),
            "/body/environment/0/supplies",
        ),
    ]);
    run_composition(&[
        (
            "an unknown rule",
            |body| body["rule"] = json!("magic"),
            (GRAPH, None),
            "/body",
        ),
        (
            "a component without its package",
            |body| {
                body["components"][0]
                    .as_object_mut()
                    .expect("component")
                    .remove("package");
            },
            (GRAPH, None),
            "/body/components/0",
        ),
        (
            "a premise with an extra side",
            |body| body["relies"][0]["extra"] = json!(1),
            (GRAPH, None),
            "/body/relies/0",
        ),
        (
            "a term of the other form",
            |body| body["term"] = json!("implementation_guarantee"),
            (GRAPH, None),
            "/body",
        ),
    ]);
    // An application at the body root is the closed term grammar's.
    let mut package = base();
    let applied = nodes(&package)
        .iter()
        .find(|node| node["semantic_form"] == "predicate")
        .expect("the predicate")
        .clone();
    let mut node = guarantee_node(guarantee_body(&package));
    node["body"] = applied["body"].clone();
    node["dependencies"] = applied["dependencies"].clone();
    let position = push(&mut package, node);
    settle(&mut package);
    expect(
        "an application root",
        &package,
        Code::IllTyped,
        Some(Cause::OperatorIneligible),
        &at(position, ""),
    );
}

#[trace("TC-474", "FR-038-AC-534")]
#[test]
fn tc_474_semantic_type_and_dependencies_must_be_the_nodes_own_and_its_targets() {
    for form in ["guarantee", "composition"] {
        let build = |edit: &dyn Fn(&mut Value)| {
            if form == "guarantee" {
                let package = base();
                let mut node = guarantee_node(guarantee_body(&package));
                edit(&mut node);
                with(package, node)
            } else {
                composed_with(composition_body(&base()), edit)
            }
        };
        let own = if form == "guarantee" {
            node_id(PREDICATE)
        } else {
            importer_id(PREDICATE)
        };
        let (package, position) = build(&|node| node["semantic_type"] = own.clone());
        expect(form, &package, GRAPH, None, &at(position, ""));
        let (package, position) = build(&|node| {
            node["dependencies"]
                .as_array_mut()
                .expect("dependencies")
                .pop();
        });
        expect(form, &package, GRAPH, None, &at(position, ""));
        let (package, position) = build(&|node| {
            let extra = typed(&relation());
            let mut ids: Vec<CheckedNodeId> =
                serde_json::from_value(node["dependencies"].clone()).expect("ids");
            if !ids.contains(&extra) {
                ids.push(extra);
            }
            ids.sort();
            ids.reverse();
            node["dependencies"] = serde_json::to_value(ids).expect("ids");
        });
        expect(form, &package, GRAPH, None, &at(position, ""));
    }
}

/// QSpec FR-490: `dependencies` are `clause`, `relation` and every `context`
/// and `assumption` target, so a scheduling environment member's `context`
/// is one too.
#[trace("TC-474", "FR-038-AC-534")]
#[test]
fn tc_474_a_fairness_context_belongs_to_the_dependencies() {
    let context = left_object();
    let (package, position) = guaranteed(|body| {
        body["environment"][1] = environment_fairness("fair_reset", &context, "act");
    });
    let mut package = package;
    let node = &mut package["semantic_graph"]["nodes"][position];
    let kept: Vec<Value> = node["dependencies"]
        .as_array()
        .expect("dependencies")
        .iter()
        .filter(|id| **id != context)
        .cloned()
        .collect();
    assert_eq!(
        kept.len() + 1,
        node["dependencies"].as_array().expect("deps").len()
    );
    node["dependencies"] = Value::Array(kept);
    settle(&mut package);
    expect(
        "an omitted fairness context",
        &package,
        GRAPH,
        None,
        &at(position, ""),
    );
}

#[trace("TC-474", "FR-038-AC-535")]
#[test]
fn tc_474_an_array_out_of_canonical_order_refuses_at_the_array() {
    run_guarantee(&[
        (
            "progress",
            |body| body["progress"].as_array_mut().expect("progress").reverse(),
            (GRAPH, None),
            "/body/progress",
        ),
        (
            "environment",
            |body| {
                body["environment"]
                    .as_array_mut()
                    .expect("environment")
                    .reverse()
            },
            (GRAPH, None),
            "/body/environment",
        ),
    ]);
    run_composition(&[
        (
            "components",
            |body| {
                body["components"]
                    .as_array_mut()
                    .expect("components")
                    .reverse()
            },
            (GRAPH, None),
            "/body/components",
        ),
        (
            "guarantees",
            |body| {
                body["guarantees"]
                    .as_array_mut()
                    .expect("guarantees")
                    .reverse()
            },
            (GRAPH, None),
            "/body/guarantees",
        ),
        (
            "relies",
            |body| {
                let clause = body["guarantees"][0]["clause"].clone();
                body["relies"] = json!([
                    premise(("Producer", &clause), ("Consumer", &clause)),
                    premise(("Consumer", &clause), ("Producer", &clause)),
                ]);
            },
            (GRAPH, None),
            "/body/relies",
        ),
        (
            "liveness_dependencies",
            |body| {
                let clause = body["guarantees"][0]["clause"].clone();
                body["liveness_dependencies"] = json!([
                    premise(("Producer", &clause), ("Consumer", &clause)),
                    premise(("Consumer", &clause), ("Producer", &clause)),
                ]);
            },
            (GRAPH, None),
            "/body/liveness_dependencies",
        ),
        // The spellings order: `strong` before `weak`, `each` before `whole`.
        (
            "supplies by kind spelling",
            |body| body["supplies"].as_array_mut().expect("supplies").reverse(),
            (GRAPH, None),
            "/body/supplies",
        ),
        (
            "supplies by granularity spelling",
            |body| {
                body["supplies"] = json!([
                    fairness("weak", "whole", &order(), "reset"),
                    fairness("weak", "each", &order(), "reset"),
                ]);
            },
            (GRAPH, None),
            "/body/supplies",
        ),
        (
            "environment",
            |body| {
                let predicate = importer_id(PREDICATE);
                body["environment"] = json!([
                    environment_assumption("b", &predicate),
                    environment_assumption("a", &predicate),
                ]);
            },
            (GRAPH, None),
            "/body/environment",
        ),
    ]);
}

#[trace("TC-474", "FR-038-AC-536")]
#[test]
fn tc_474_a_guarantee_target_must_be_a_declared_node_of_its_kind() {
    run_guarantee(&[
        (
            "a clause that is no temporal clause",
            |body| body["clause"] = relation(),
            MALFORMED,
            "/body/clause",
        ),
        (
            "a relation that is no abstraction relation",
            |body| body["relation"] = predicate(),
            MALFORMED,
            "/body/relation",
        ),
        (
            "a progress context that is no object type",
            |body| {
                body["progress"][0]["context"] = predicate();
                body["progress"][1]["context"] = predicate();
            },
            MALFORMED,
            "/body/progress/0",
        ),
        (
            "a fairness context that is no object type",
            |body| {
                body["environment"][1]["supplies"]["fairness"]["context"] = predicate();
            },
            MALFORMED,
            "/body/environment/1",
        ),
        (
            "an assumption that is no predicate",
            |body| body["environment"][0]["supplies"] = json!({"assumption": relation()}),
            MALFORMED,
            "/body/environment/0",
        ),
    ]);
    // A clause that selects another temporal profile.
    let mut package = base();
    select_profile(
        &mut package,
        "quire.temporal.event-position.false-extension/v1",
    );
    let live_formula = nodes(&package)
        .iter()
        .position(|node| node["node_id"] == node_id("guarantee/live-formula"))
        .expect("live formula");
    package["semantic_graph"]["nodes"][live_formula]["body"]["operation"]["member"]["interval"] =
        json!({"lower": "0", "upper": "3"});
    let body = guarantee_body(&package);
    let (package, position) = with(package, guarantee_node(body));
    expect(
        "a bounded-profile clause",
        &package,
        MALFORMED.0,
        MALFORMED.1,
        &at(position, "/body/clause"),
    );
}

#[trace("TC-474", "FR-038-AC-537")]
#[test]
fn tc_474_a_guarantee_member_must_resolve_spell_and_be_unique() {
    run_guarantee(&[
        (
            "a progress operation the subject does not declare",
            |body| body["progress"][0]["operation"] = json!("missing"),
            MISSING,
            "/body/progress/0",
        ),
        (
            "a scheduling operation the subject does not declare",
            |body| {
                body["environment"][1]["supplies"]["fairness"]["operation"] = json!("missing");
            },
            MISSING,
            "/body/environment/1",
        ),
        (
            "an operation the context only inherits",
            |body| {
                body["progress"][0]["context"] = sub();
                body["progress"][1]["context"] = sub();
            },
            MALFORMED,
            "/body/progress/0",
        ),
        (
            "an initial path that is not a Rust path",
            |body| body["initial"] = json!(["9lives"]),
            MALFORMED,
            "/body/initial",
        ),
        (
            "a measure path that is not a Rust path",
            |body| body["progress"][0]["measure"] = json!({"decreases": ["crate"]}),
            MALFORMED,
            "/body/progress/0",
        ),
        (
            "a zero bound",
            |body| body["progress"][1]["measure"] = json!({"bound": 0}),
            MALFORMED,
            "/body/progress/1",
        ),
        (
            "two progress members for one operation",
            |body| body["progress"][1]["operation"] = json!("reset"),
            CONFLICT,
            "/body/progress/1",
        ),
        (
            "two environment members of one name",
            |body| body["environment"][1]["name"] = json!("env_ok"),
            CONFLICT,
            "/body/environment/1",
        ),
    ]);
}

#[trace("TC-474", "FR-038-AC-538")]
#[test]
fn tc_474_a_composition_member_must_name_its_components_and_guarantees() {
    run_composition(&[
        (
            "two components of one name",
            |body| body["components"][1]["name"] = json!("Consumer"),
            CONFLICT,
            "/body/components/1",
        ),
        (
            "two components over one subject",
            |body| body["components"][1]["subject"] = sub(),
            CONFLICT,
            "/body/components/1",
        ),
        (
            "a guarantee of an undeclared component",
            |body| body["guarantees"][1]["component"] = json!("Stranger"),
            MISSING,
            "/body/guarantees/1",
        ),
        (
            "a relies premise on an undeclared component",
            |body| body["relies"][0]["on"]["component"] = json!("Stranger"),
            MISSING,
            "/body/relies/0",
        ),
        (
            "a relies premise whose guarantee is not declared",
            |body| {
                body["guarantees"]
                    .as_array_mut()
                    .expect("guarantees")
                    .remove(0);
            },
            MALFORMED,
            "/body/relies/0",
        ),
        (
            "a liveness dependency of a component on itself",
            |body| body["liveness_dependencies"][0]["on"]["component"] = json!("Producer"),
            MALFORMED,
            "/body/liveness_dependencies/0",
        ),
        (
            "a component of a package the unit does not import",
            |body| body["components"][0]["package"] = json!("acme/stranger"),
            MISSING,
            "/body/components/0/package",
        ),
        (
            "a supplies operation no component subject declares",
            |body| body["supplies"][0]["operation"] = json!("missing"),
            MISSING,
            "/body/supplies/0",
        ),
        (
            "a scheduling operation no component subject declares",
            |body| {
                body["environment"] = json!([
                    environment_assumption("env_ok", &importer_id(PREDICATE)),
                    environment_fairness("fair", &order(), "missing"),
                ]);
            },
            MISSING,
            "/body/environment/1",
        ),
        (
            "two environment members of one name",
            |body| {
                let predicate = importer_id(PREDICATE);
                body["environment"] = json!([
                    environment_assumption("env_ok", &predicate),
                    environment_assumption("env_ok", &predicate),
                ]);
            },
            CONFLICT,
            "/body/environment/1",
        ),
    ]);
}

#[trace("TC-474", "FR-038-AC-538")]
#[test]
fn tc_474_a_composition_target_resolves_in_its_component_package() {
    run_composition(&[
        (
            "a subject that is no object type",
            |body| body["components"][0]["subject"] = relation(),
            MALFORMED,
            "/body/components/0",
        ),
        (
            "a guarantee that is no temporal clause",
            |body| body["guarantees"][0]["clause"] = order(),
            MALFORMED,
            "/body/guarantees/0",
        ),
        (
            "a supplies context that is no object type",
            |body| body["supplies"] = json!([fairness("strong", "each", &relation(), "reset")]),
            MALFORMED,
            "/body/supplies/0",
        ),
        (
            "a supplies context in no component package",
            |body| {
                body["supplies"] =
                    json!([fairness("strong", "each", &importer_id(PREDICATE), "reset")]);
            },
            MISSING,
            "/body/supplies/0",
        ),
        (
            "an assumption that is no predicate",
            |body| {
                body["environment"][0]["supplies"] =
                    json!({"assumption": importer_id("guarantee/relation")});
            },
            MALFORMED,
            "/body/environment/0",
        ),
    ]);
}

#[trace("TC-474", "FR-038-AC-538")]
#[test]
fn tc_474_composition_premises_keep_safety_and_liveness_separate() {
    run_composition(&[
        (
            "rely on liveness",
            |body| {
                body["relies"][0]["on"]["clause"] = node_id("guarantee/live-clause");
            },
            MALFORMED,
            "/body/relies/0",
        ),
        (
            "depends on safety",
            |body| {
                body["liveness_dependencies"][0]["on"]["clause"] = clause_id(&base());
            },
            MALFORMED,
            "/body/liveness_dependencies/0",
        ),
        (
            "depends from safety",
            |body| {
                body["liveness_dependencies"][0]["guarantee"] = body["guarantees"][0].clone();
                body["liveness_dependencies"][0]["on"]["component"] = json!("Producer");
            },
            MALFORMED,
            "/body/liveness_dependencies/0",
        ),
    ]);
}

#[trace("TC-474", "FR-038-AC-538")]
#[test]
fn tc_474_a_component_clause_must_be_over_its_model_subject() {
    let (package, position) = composed(|_| {});
    let mut component = base();
    let parameter = nodes(&component)
        .iter()
        .position(|node| node["node_id"] == node_id("guarantee/model-over"))
        .expect("parameter");
    component["semantic_graph"]["nodes"][parameter]["semantic_type"] = nodes(&component)
        .iter()
        .find(|node| node["node_tag"] == "scalar_type" && node["semantic_form"] == "boolean")
        .expect("Boolean type")["node_id"]
        .clone();
    settle(&mut component);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_composition_with_dependency(&package, &component)
    else {
        panic!("a clause over a different subject must refuse");
    };
    assert_eq!((refusal.code, refusal.cause), MISSING);
    assert_eq!(
        refusal.loci[0].path.as_ref().map(|p| p.as_str()),
        Some(at(position, "/body/guarantees/0").as_str())
    );
}

#[trace("TC-474", "FR-038-AC-533")]
#[test]
fn tc_474_a_malformed_target_key_refuses_as_wire() {
    let (mut package, position) = guaranteed(|_| {});
    package["semantic_graph"]["nodes"][position]["body"]["relation"]["ordinal"] = json!("wrong");
    // Keep the malformed key out of dependencies so only the body key is invalid.
    expect(
        "a nonnumeric target ordinal",
        &package,
        Code::MalformedWire,
        None,
        &at(position, "/body/relation"),
    );
}

#[trace("TC-474", "FR-038-AC-538")]
#[test]
fn tc_474_premise_classification_uses_negation_normal_form() {
    for (operator, negated, liveness) in [
        ("eventually", false, true),
        ("eventually", true, false),
        ("always", false, false),
        ("always", true, true),
        ("until", false, true),
        ("until", true, false),
        ("release", false, false),
        ("release", true, true),
    ] {
        let (package, position) = composed(|_| {});
        let mut component = base();
        let root = nodes(&component)
            .iter()
            .position(|node| node["node_id"] == node_id("guarantee/live-formula"))
            .expect("live formula");
        let formula = &mut component["semantic_graph"]["nodes"][root];
        formula["body"]["operation"]["identity"] = json!(format!("quire.op.temporal.{operator}"));
        if matches!(operator, "until" | "release") {
            let argument = formula["body"]["arguments"][0].clone();
            formula["body"]["arguments"]
                .as_array_mut()
                .expect("arguments")
                .push(argument);
        }
        if negated {
            let mut inner = formula.clone();
            inner["node_id"] = node_id("guarantee/negated-formula");
            formula["body"]["operation"]["identity"] = json!("quire.op.temporal.not");
            formula["body"]["operation"]["member"] = Value::Null;
            formula["body"]["arguments"] =
                json!([{"term": "reference", "target": inner["node_id"]}]);
            push(&mut component, inner);
        }
        settle(&mut component);
        let outcome = read_composition_with_dependency(&package, &component);
        if liveness {
            assert!(
                matches!(outcome, CheckedPackageV2ReadResult::Admitted(_)),
                "{operator} negated={negated}: {outcome:?}"
            );
        } else {
            let CheckedPackageV2ReadResult::Refused(refusal) = outcome else {
                panic!("{operator} negated={negated} must refuse a depends premise")
            };
            assert_eq!(
                (refusal.code, refusal.cause),
                MALFORMED,
                "{operator} negated={negated}"
            );
            assert_eq!(
                refusal.loci[0].path.as_ref().map(|p| p.as_str()),
                Some(at(position, "/body/liveness_dependencies/0").as_str())
            );
        }
    }
}

#[trace("TC-474", "FR-038-AC-537")]
#[test]
fn tc_474_guarantee_operations_must_belong_to_the_clause_model_subject() {
    let mut package = base();
    let parameter = nodes(&package)
        .iter()
        .position(|node| node["node_id"] == node_id("guarantee/model-over"))
        .expect("parameter");
    package["semantic_graph"]["nodes"][parameter]["semantic_type"] = nodes(&package)
        .iter()
        .find(|node| node["node_tag"] == "scalar_type" && node["semantic_form"] == "boolean")
        .expect("Boolean type")["node_id"]
        .clone();
    let body = guarantee_body(&package);
    let (package, position) = with(package, guarantee_node(body));
    expect(
        "operations of a different subject",
        &package,
        MISSING.0,
        MISSING.1,
        &at(position, "/body/progress/0"),
    );
}
