// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! QSpec's published temporal aggregate argument fragments in complete packages.

use super::{
    cause_word, code_word, fixture_support, fixtures_dir, proposal_dir_of, read_json, read_package,
};
use ix_trace_rs::trace;
use quire_contract_model::{CheckedNodeId, CheckedPackageV2ReadResult, CheckedTerm};
use serde_json::{json, Value};
use std::collections::BTreeSet;

fn vectors() -> Value {
    read_json(&proposal_dir_of(&fixtures_dir()).join("temporal-aggregate-vectors.json"))
        .expect("published QSpec temporal aggregate arguments")
}

fn id(path: &[&str]) -> Value {
    json!({"package": "example", "path": path, "ordinal": 0})
}

fn source_row(node: &Value) -> Value {
    let occurrence = &node["occurrences"][0];
    json!({
        "node_id": node["node_id"],
        "role": occurrence["role"],
        "ordinal": occurrence["ordinal"],
        "regions": [{
            "source": {"authority": "agent-ix", "identity": "example"},
            "start": 0, "end": 1,
        }],
    })
}

fn add_node(package: &mut Value, node: Value) {
    package["source_map"]
        .as_array_mut()
        .expect("source map")
        .push(source_row(&node));
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("graph nodes")
        .push(node);
}

fn dimension(name: &str) -> Value {
    let key = id(&["scalar_type", "dimension", name]);
    json!({
        "node_id": key, "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": "scalar_type", "semantic_form": "dimension", "semantic_type": key,
        "declaration": {"qualified_name": ["scalar_type", "dimension", name]},
        "dependencies": [], "occurrences": [{"role": "declaration", "ordinal": 0}],
        "owner": {"kind": "source", "authority": "agent-ix", "identity": "example"},
        "body": {"term": "dimension", "base": true, "terms": []},
    })
}

fn unit(name: &str, dimension: &str) -> Value {
    let key = id(&["scalar_type", "unit", name]);
    let dimension = id(&["scalar_type", "dimension", dimension]);
    let target = (name == "s").then(|| id(&["scalar_type", "unit", "ms"]));
    let scale = if name == "s" { "1000" } else { "1" };
    let dependencies = if let Some(target) = &target {
        json!([dimension, target])
    } else {
        json!([dimension])
    };
    json!({
        "node_id": key, "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": "scalar_type", "semantic_form": "unit", "semantic_type": dimension,
        "declaration": {"qualified_name": ["scalar_type", "unit", name]},
        "dependencies": dependencies, "occurrences": [{"role": "declaration", "ordinal": 0}],
        "owner": {"kind": "source", "authority": "agent-ix", "identity": "example"},
        "body": {"term": "unit", "dimension_node_id": dimension,
                 "target_unit_node_id": target,
                 "scale": {"numerator": scale, "denominator": "1"},
                 "offset": {"numerator": "0", "denominator": "1"}},
    })
}

fn expression(target: &Value, family: &str, value_type: &Value) -> (Value, Value) {
    let name = target["path"][1].as_str().expect("reference name");
    let value_id = id(&["value", "aggregate-input", name]);
    let (kind, value) = match family {
        "boolean" => ("boolean", json!(true)),
        "integer" => ("integer", json!("2")),
        "quantity" => ("quantity", json!("2")),
        _ => panic!("published reference family {family}"),
    };
    let value_node = json!({
        "node_id": value_id, "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": "value", "semantic_form": "literal", "semantic_type": value_type,
        "dependencies": [], "occurrences": [{"role": "expression", "ordinal": 0}],
        "body": {"term": "literal", "type": value_type, "value_kind": kind, "value": value},
    });
    let expression_node = json!({
        "node_id": target, "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": "expression", "semantic_form": "reference", "semantic_type": value_type,
        "dependencies": [value_id], "occurrences": [{"role": "expression", "ordinal": 0}],
        "body": {"term": "reference", "target": value_id},
    });
    (value_node, expression_node)
}

fn referenced_nodes(term: &Value, targets: &mut Vec<Value>) {
    match term {
        Value::Object(members) if members.get("term") == Some(&json!("reference")) => {
            targets.push(members["target"].clone());
        }
        Value::Object(members) => {
            for value in members.values() {
                referenced_nodes(value, targets);
            }
        }
        Value::Array(members) => {
            for value in members {
                referenced_nodes(value, targets);
            }
        }
        _ => {}
    }
}

fn replace_profile(value: &mut Value, selected: &str) {
    match value {
        Value::String(text) if text == "quire.temporal.event-position.false-extension/v1" => {
            *text = selected.to_owned();
        }
        Value::Array(items) => {
            for item in items {
                replace_profile(item, selected);
            }
        }
        Value::Object(members) => {
            for member in members.values_mut() {
                replace_profile(member, selected);
            }
        }
        _ => {}
    }
}

fn package_for(row: &Value, published: &Value) -> (Value, usize) {
    let mut package = read_json(&fixtures_dir().join("positive-all-families.json"))
        .expect("published positive package");
    let profile = row["profile"].as_str().expect("published profile");
    replace_profile(&mut package, profile);
    match profile {
        "quire.temporal.event-position.false-extension/v1" => {}
        "quire.temporal.timestamped-event.finite-window/v1" => {
            package["temporal_bindings"][0]["clock"] =
                json!({"kind": "timestamped_event", "timestamp_unit": "ms"});
        }
        "quire.temporal.infinite-trace/v1" => {
            package
                .as_object_mut()
                .expect("package")
                .remove("temporal_bindings");
        }
        other => panic!("published profile {other}"),
    }
    add_node(&mut package, dimension("Time"));
    add_node(&mut package, dimension("Length"));
    for (name, dimension) in [("ms", "Time"), ("s", "Time"), ("m", "Length")] {
        add_node(&mut package, unit(name, dimension));
    }
    for reference in published["references"].as_array().expect("references") {
        let family = reference["family"].as_str().expect("family");
        let value_type = match family {
            "boolean" => package["semantic_graph"]["nodes"][0]["node_id"].clone(),
            "integer" => id(&["scalar_type", "integer"]),
            "quantity" => published["clock_unit"].clone(),
            other => panic!("published family {other}"),
        };
        let (value, expression) = expression(&reference["target"], family, &value_type);
        add_node(&mut package, value);
        add_node(&mut package, expression);
    }
    let formula_index = 28;
    let formula = &mut package["semantic_graph"]["nodes"][formula_index];
    formula["body"]["operation"]["identity"] = json!("quire.op.temporal.aggregate");
    formula["body"]["arguments"] = json!([row["argument"]]);
    let mut targets = Vec::new();
    referenced_nodes(&row["argument"], &mut targets);
    targets.sort_by_key(fixture_support::structural_fixture_id);
    targets.dedup();
    formula["dependencies"] = json!(targets);
    (package, formula_index)
}

/// Trace: FR-038-AC-461, FR-038-AC-462; QSpec FR-370-AC-17/18, TC-303.
#[trace("TC-048", "FR-038-AC-461", "FR-038-AC-462")]
#[test]
fn published_temporal_aggregate_arguments_match_strict_reader() {
    let published = vectors();
    let positives = published["positive"].as_array().expect("positive cases");
    let readers = published["reader"].as_array().expect("reader cases");
    assert!(!positives.is_empty(), "published positives");
    assert!(!readers.is_empty(), "published reader mutations");
    let mut ids = BTreeSet::new();
    let mut mismatches = Vec::new();
    let mut matched = 0usize;
    for case in positives.iter().chain(readers) {
        let case_id = case["id"].as_str().expect("case id");
        assert!(ids.insert(case_id), "duplicate {case_id}");
        let (package, position) = package_for(case, &published);
        let result = read_package(&package).expect("complete package read");
        if case.get("expected").is_none() {
            let CheckedPackageV2ReadResult::Admitted(admitted) = result else {
                mismatches.push(format!("{case_id}: expected admission, got refusal"));
                continue;
            };
            let CheckedTerm::Application {
                operation,
                arguments,
                ..
            } = &admitted.graph().nodes[position].body
            else {
                panic!("{case_id}: temporal formula body");
            };
            assert_eq!(
                operation.identity(),
                Some("quire.op.temporal.aggregate"),
                "{case_id}"
            );
            assert_eq!(arguments.len(), 1, "{case_id}: one aggregate argument");
            assert_eq!(
                serde_json::to_value(&arguments[0]).expect("typed argument"),
                case["argument"],
                "{case_id}: published aggregate bindings retained"
            );
            let expected_id: CheckedNodeId = serde_json::from_value(
                package["semantic_graph"]["nodes"][position]["node_id"].clone(),
            )
            .expect("published formula identity");
            assert_eq!(
                admitted
                    .document_held_id(position)
                    .expect("graph node")
                    .clone(),
                expected_id,
                "{case_id}: formula identity retained"
            );
            matched += 1;
        } else {
            let CheckedPackageV2ReadResult::Refused(refusal) = result else {
                mismatches.push(format!(
                    "{case_id}: expected refusal, got admission/incomplete"
                ));
                continue;
            };
            let suffix = case["expected"]["pointer"].as_str().expect("pointer");
            let actual = (
                code_word(refusal.code),
                refusal.cause.map(cause_word),
                refusal.loci[0].path.as_ref().map(ToString::to_string),
            );
            let expected = (
                case["expected"]["code"].as_str().expect("code"),
                case["expected"]["cause"].as_str(),
                Some(format!("/semantic_graph/nodes/{position}/body{suffix}")),
            );
            if actual != expected {
                mismatches.push(format!("{case_id}: {actual:?} != {expected:?}"));
                continue;
            }
            let expected_locus: CheckedNodeId = serde_json::from_value(
                package["semantic_graph"]["nodes"][position]["node_id"].clone(),
            )
            .expect("formula node id");
            assert_eq!(
                refusal.locus.as_ref(),
                Some(&expected_locus),
                "{case_id}: locus"
            );
            matched += 1;
        }
    }
    let total = positives.len() + readers.len();
    assert!(
        mismatches.is_empty(),
        "{matched}/{total} matched: {mismatches:#?}"
    );
}

/// The temporal aggregate stage precedes ordinary operation conformance.
#[trace("TC-048", "FR-038-AC-462")]
#[test]
fn aggregate_value_refusal_precedes_operation_result_refusal() {
    let published = vectors();
    let case = published["reader"]
        .as_array()
        .expect("reader cases")
        .iter()
        .find(|case| case["id"] == "AR-12-kind-median")
        .expect("published invalid kind");
    let (mut package, position) = package_for(case, &published);
    package["semantic_graph"]["nodes"][position]["body"]["result_type"] =
        id(&["scalar_type", "integer"]);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package(&package).expect("complete package")
    else {
        panic!("aggregate value defect must refuse");
    };
    assert_eq!(code_word(refusal.code), case["expected"]["code"]);
    assert_eq!(
        refusal.cause.map(cause_word),
        case["expected"]["cause"].as_str()
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some(format!(
            "/semantic_graph/nodes/{position}/body{}",
            case["expected"]["pointer"].as_str().unwrap()
        ))
    );
}

/// Existing interval bounds precede a reached aggregate's argument checks.
#[trace("TC-048", "FR-038-AC-462")]
#[test]
fn interval_bounds_refusal_precedes_reached_aggregate_refusal() {
    let published = vectors();
    let case = published["reader"]
        .as_array()
        .expect("reader cases")
        .iter()
        .find(|case| case["id"] == "AR-12-kind-median")
        .expect("published invalid kind");
    let (mut package, _) = package_for(case, &published);
    package["semantic_graph"]["nodes"][29]["body"]["operation"]["member"]["interval"]["lower"] =
        json!("4");
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package(&package).expect("complete package")
    else {
        panic!("interval defect must refuse");
    };
    assert_eq!(code_word(refusal.code), "invalid_package");
    assert_eq!(refusal.cause.map(cause_word), Some("invalid-value"));
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some("/semantic_graph/nodes/29/body".to_owned())
    );
}

/// One structural clause can represent claims with distinct clock-unit bindings.
#[trace("TC-048", "FR-038-AC-462")]
#[test]
fn duration_bounds_match_every_occurrence_clock_unit() {
    let published = vectors();
    let case = published["positive"]
        .as_array()
        .expect("positive cases")
        .iter()
        .find(|case| case["id"] == "AG-07-duration-quantile")
        .expect("published duration case");
    let (mut package, formula) = package_for(case, &published);
    let clause = 8;
    let clause_id = package["semantic_graph"]["nodes"][clause]["node_id"].clone();
    package["semantic_graph"]["nodes"][clause]["occurrences"]
        .as_array_mut()
        .expect("clause occurrences")
        .push(json!({"role": "claim", "ordinal": 1}));
    let source_rows = package["source_map"].as_array_mut().expect("source map");
    let source_at = source_rows
        .iter()
        .position(|row| row["node_id"] == clause_id)
        .expect("clause source association");
    let mut second_source = source_rows[source_at].clone();
    second_source["ordinal"] = json!(1);
    second_source["regions"][0]["start"] = json!(11);
    second_source["regions"][0]["end"] = json!(12);
    source_rows.insert(source_at + 1, second_source);
    package["temporal_clause_names"]
        .as_array_mut()
        .expect("clause names")
        .insert(
            1,
            json!({"declaration": clause_id, "occurrence": 1, "name": "Clause1"}),
        );
    let bindings = package["temporal_bindings"]
        .as_array_mut()
        .expect("bindings");
    let mut second_binding = bindings[0].clone();
    second_binding["occurrence"] = json!(1);
    bindings.insert(1, second_binding);
    let CheckedPackageV2ReadResult::Admitted(admitted) =
        read_package(&package).expect("complete package")
    else {
        panic!("both claim occurrences using ms must admit");
    };
    assert_eq!(
        serde_json::to_value(&admitted.graph().nodes[formula].body).expect("typed formula"),
        package["semantic_graph"]["nodes"][formula]["body"]
    );

    package["temporal_bindings"][1]["clock"]["timestamp_unit"] = json!("s");
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package(&package).expect("complete package")
    else {
        panic!("second claim's s clock must refuse an ms bound");
    };
    assert_eq!(code_word(refusal.code), "ill_typed");
    assert_eq!(refusal.cause.map(cause_word), Some("unit-mismatch"));
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some(format!(
            "/semantic_graph/nodes/{formula}/body/arguments/0/members/5/value"
        ))
    );
    let expected_id: CheckedNodeId =
        serde_json::from_value(package["semantic_graph"]["nodes"][formula]["node_id"].clone())
            .expect("formula identity");
    assert_eq!(refusal.locus.as_ref(), Some(&expected_id));
}

/// Operand order does not choose the first aggregate refusal.
#[trace("TC-048", "FR-038-AC-462")]
#[test]
fn reachable_aggregate_refusals_follow_formula_identity_order() {
    let published = vectors();
    let readers = published["reader"].as_array().expect("reader cases");
    let first = readers
        .iter()
        .find(|case| case["id"] == "AR-12-kind-median")
        .expect("first mutation");
    let second = readers
        .iter()
        .find(|case| case["id"] == "AR-13-kind-not-text")
        .expect("second mutation");
    let (mut package, first_position) = package_for(first, &published);
    let mut second_node = package["semantic_graph"]["nodes"][first_position].clone();
    let first_id = package["semantic_graph"]["nodes"][first_position]["node_id"].clone();
    let mut second_id = first_id.clone();
    second_id["path"]
        .as_array_mut()
        .expect("formula path")
        .push(json!("zz-aggregate"));
    second_node["node_id"] = second_id.clone();
    second_node["body"]["arguments"] = json!([second["argument"]]);
    add_node(&mut package, second_node);
    let second_position = package["semantic_graph"]["nodes"].as_array().unwrap().len() - 1;
    let parent = &mut package["semantic_graph"]["nodes"][29];
    parent["body"]["operation"]["identity"] = json!("quire.op.temporal.and");
    parent["body"]["operation"]["member"] = Value::Null;
    parent["body"]["arguments"] = json!([
        {"term": "reference", "target": second_id},
        {"term": "reference", "target": first_id},
    ]);
    let mut dependencies = vec![first_id, second_id];
    dependencies.sort_by_key(|value| {
        serde_json::from_value::<CheckedNodeId>(value.clone()).expect("formula identity")
    });
    parent["dependencies"] = json!(dependencies);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package(&package).expect("complete package")
    else {
        panic!("two defective aggregates must refuse");
    };
    assert_eq!(code_word(refusal.code), first["expected"]["code"]);
    assert_eq!(
        refusal.cause.map(cause_word),
        first["expected"]["cause"].as_str()
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some(format!(
            "/semantic_graph/nodes/{first_position}/body{}",
            first["expected"]["pointer"].as_str().unwrap()
        ))
    );
    assert_ne!(first_position, second_position);
}
