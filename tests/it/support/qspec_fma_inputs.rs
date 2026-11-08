// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Input builders shared with the original QSpec FMA conformance driver.

use crate::support::checked_package as fixture_support;
use serde_json::{json, Value};

fn type_id(width: &str) -> Value {
    fixture_support::node_id(&format!("fma/type/{width}"))
}

fn rounded_type_id(width: &str) -> Value {
    fixture_support::node_id(&format!("fma/rounding/{width}"))
}

#[ix_trace_rs::string_edge]
pub(crate) fn operand_type_id(width: &str) -> Value {
    if width == "integer" {
        type_id(width)
    } else {
        rounded_type_id(width)
    }
}

pub(crate) fn literal_id(width: &str, index: usize) -> Value {
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
        "dependencies": [], "occurrences": [{"role": "generated", "ordinal": 0}],
        "body": {"term": "literal", "type": ty, "value_kind": kind, "value": value},
    })
}

#[ix_trace_rs::string_edge]
pub(crate) fn package_for(case: &Value) -> (Value, usize) {
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
