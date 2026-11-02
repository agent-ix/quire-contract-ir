// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-048: QSpec's published FMA application cases at the strict reader boundary.

use super::{cause_word, code_word, fixture_support, fixtures_dir, proposal_dir_of, read_json};
use ix_trace_rs::trace;
use quire_contract_model::{
    ApplicationOperator, CheckedPackageReadLimits, CheckedPackageV2, CheckedPackageV2ReadResult,
    CheckedTerm,
};
use serde_json::{json, Value};
use std::collections::BTreeSet;

fn type_id(width: &str) -> Value {
    fixture_support::node_id(&format!("fma/type/{width}"))
}

fn rounded_type_id(width: &str) -> Value {
    fixture_support::node_id(&format!("fma/rounding/{width}"))
}

#[ix_trace_rs::string_edge]
fn operand_type_id(width: &str) -> Value {
    if width == "integer" {
        type_id(width)
    } else {
        rounded_type_id(width)
    }
}

fn literal_id(width: &str, index: usize) -> Value {
    fixture_support::node_id(&format!("fma/literal/{index}/{width}"))
}

fn type_node(width: &str) -> Value {
    let id = type_id(width);
    json!({
        "node_id": id, "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": "scalar_type", "semantic_form": width, "semantic_type": id,
        "dependencies": [], "occurrences": [{"role": "generated", "ordinal": 0}],
        "body": {"term": "aggregate", "members": []},
    })
}

fn rounded_type_node(width: &str) -> Value {
    let scalar = type_id(width);
    json!({
        "node_id": rounded_type_id(width),
        "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": "bounded_domain", "semantic_form": "float_rounding",
        "semantic_type": scalar, "dependencies": [scalar],
        "occurrences": [{"role": "generated", "ordinal": 0}],
        "body": {"term": "aggregate", "members": [{
            "term": "binding", "name": "rounding",
            "value": {"term": "literal", "type": type_id("text"),
                      "value_kind": "text", "value": "nearest-even"}
        }]},
    })
}

fn literal_node(width: &str, index: usize) -> Value {
    let (kind, value) = match width {
        "float32" => ("float32_bits", "3f800000"),
        "float64" => ("float64_bits", "3ff0000000000000"),
        "integer" => ("integer", "1"),
        _ => panic!("published FMA width {width}"),
    };
    let ty = operand_type_id(width);
    json!({
        "node_id": literal_id(width, index), "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": "value", "semantic_form": "literal", "semantic_type": ty,
        "dependencies": [ty], "occurrences": [{"role": "generated", "ordinal": 0}],
        "body": {"term": "literal", "type": ty, "value_kind": kind, "value": value},
    })
}

#[ix_trace_rs::string_edge]
fn package_for(case: &Value) -> (Value, usize) {
    let mut package = fixture_support::v2_all_families();
    let law = json!({"authority": "agent-ix", "identity": "quire.value.ieee754-2019-default/v1"});
    package["lock"]["definition_selections"] = json!([law.clone()]);
    let widths = case["argument_widths"]
        .as_array()
        .expect("published argument widths");
    let arguments = widths
        .iter()
        .enumerate()
        .map(|(index, width)| {
            json!({
                "term": "reference", "target": literal_id(width.as_str().expect("width"), index)
            })
        })
        .collect::<Vec<_>>();
    let laws = case["laws"]
        .as_array()
        .expect("published laws")
        .iter()
        .map(|role| json!({"role": role, "definition": law}))
        .collect::<Vec<_>>();
    let mode = if case["mode"] == "rounding" {
        json!({"kind": "rounding", "value": "nearest-even"})
    } else {
        assert!(case["mode"].is_null(), "published mode is rounding or null");
        Value::Null
    };
    let result = operand_type_id(case["result_width"].as_str().expect("result width"));
    let application = json!({
        "node_id": fixture_support::node_id("fma/application"),
        "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": "expression", "semantic_form": "call", "semantic_type": result,
        "dependencies": [], "occurrences": [{"role": "generated", "ordinal": 0}],
        "body": {
            "term": "application", "operator": "call", "result_type": result,
            "operation": {
                "identity": case["identity"], "laws": laws, "mode": mode,
                "member": null, "leaves": [],
            },
            "arguments": arguments,
        },
    });
    let nodes = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("graph nodes");
    for width in ["float32", "float64", "integer", "text"] {
        nodes.push(type_node(width));
    }
    for width in ["float32", "float64"] {
        nodes.push(rounded_type_node(width));
    }
    for (index, width) in widths.iter().enumerate() {
        nodes.push(literal_node(width.as_str().expect("width"), index));
    }
    let position = nodes.len();
    nodes.push(application);
    fixture_support::settle(&mut package);
    (package, position)
}

/// Trace: FR-038-AC-431, FR-038-AC-432; QSpec FR-322-AC-307.
#[trace("TC-048", "FR-038-AC-431", "FR-038-AC-432")]
#[test]
fn tc_048_fma_applications_follow_published_qspec_cases() {
    let vectors = read_json(&proposal_dir_of(&fixtures_dir()).join("fma-application-vectors.json"))
        .expect("published QSpec FMA application vectors");
    let cases = vectors["cases"].as_array().expect("published cases");
    let mut seen = BTreeSet::new();
    for case in cases {
        let id = case["id"].as_str().expect("case id");
        assert!(seen.insert(id), "duplicate {id}");
        let (package, position) = package_for(case);
        let result = CheckedPackageV2::read(
            &fixture_support::canonical(&package),
            CheckedPackageReadLimits::bounded(),
            &fixture_support::evidence_for(&package),
        );
        match case["expected"].as_str().expect("expected result") {
            "admitted" => {
                let CheckedPackageV2ReadResult::Admitted(admitted) = result else {
                    panic!("{id}: expected admission, got {result:?}");
                };
                let node = &admitted.graph().nodes[position];
                let CheckedTerm::Application {
                    operator,
                    operation,
                    result_type,
                    arguments,
                } = &node.body
                else {
                    panic!("{id}: admitted node is not an application");
                };
                assert_eq!(operation.identity(), case["identity"].as_str(), "{id}");
                assert_eq!(*operator, ApplicationOperator::Call, "{id}: generic call");
                let read = operation.read().expect("admitted FMA operation");
                assert_eq!(
                    read.mode.map(|mode| (mode.kind.as_wire(), mode.value)),
                    Some(("rounding", "nearest-even")),
                    "{id}: rounding mode"
                );
                assert_eq!(read.laws.len(), 1, "{id}: selected IEEE profile");
                assert_eq!(read.laws[0].role.as_wire(), "ieee_profile", "{id}");
                assert!(read.member.is_none(), "{id}: no member");
                assert!(read.leaves.is_empty(), "{id}: no leaves");
                let width = case["result_width"].as_str().expect("result width");
                assert_eq!(
                    result_type,
                    &fixture_support::structural_fixture_id(&operand_type_id(width)),
                    "{id}: result type"
                );
                assert_eq!(arguments.len(), 3, "{id}: a, b, c");
                for (index, argument) in arguments.iter().enumerate() {
                    let width = case["argument_widths"][index].as_str().expect("width");
                    let expected =
                        fixture_support::structural_fixture_id(&literal_id(width, index));
                    assert!(
                        matches!(argument, CheckedTerm::Reference { target } if target == &expected),
                        "{id}: argument {index} references its operand"
                    );
                }
            }
            "refused" => {
                let CheckedPackageV2ReadResult::Refused(refusal) = result else {
                    panic!("{id}: expected refusal, got {result:?}");
                };
                assert_eq!(
                    code_word(refusal.code),
                    case["code"].as_str().unwrap(),
                    "{id}: code"
                );
                assert_eq!(
                    refusal.cause.map(cause_word),
                    case["cause"].as_str(),
                    "{id}: cause"
                );
            }
            other => panic!("{id}: unknown expected result {other}"),
        }
    }
    let expected = (1..=12)
        .map(|index| format!("FM-{index:02}"))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        seen,
        expected.iter().map(String::as_str).collect(),
        "exact published FM set"
    );
}
