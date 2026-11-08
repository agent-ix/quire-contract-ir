// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Text-vector input construction shared with the original QSpec driver.

use crate::support::checked_package as fixture_support;
use serde_json::{json, Value};

fn index(package: &Value, tag: &str, form: &str) -> usize {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_tag"] == tag && node["semantic_form"] == form)
        .unwrap_or_else(|| panic!("missing {tag}/{form}"))
}

pub(crate) fn bounded_literal(min: u64, max: u64, profile: &str, value: Value) -> (Value, usize) {
    let mut package = fixture_support::v2_all_families();
    let text = fixture_support::node_id(&fixture_support::family_key("a1a1"));
    let integer = fixture_support::node_id(&fixture_support::family_key("a3a3"));
    let range = index(&package, "bounded_domain", "integer_range");
    let mut bounds = package["semantic_graph"]["nodes"][range].clone();
    let bounds_id = fixture_support::structural_id(
        "test/families",
        vec![json!("derived-shape"), json!("text_bounds")],
        0,
    );
    bounds["node_id"] = bounds_id.clone();
    bounds["semantic_form"] = json!("text_bounds");
    bounds["semantic_type"] = text.clone();
    bounds["dependencies"] = json!([text]);
    bounds["body"] = json!({"term": "aggregate", "members": [
        {"term": "binding", "name": "min", "value": {
            "term": "literal", "type": integer, "value_kind": "integer", "value": min.to_string()}},
        {"term": "binding", "name": "max", "value": {
            "term": "literal", "type": integer, "value_kind": "integer", "value": max.to_string()}},
        {"term": "binding", "name": "text_profile", "value": {
            "term": "literal", "type": text, "value_kind": "text", "value": profile}}
    ]});
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(bounds);

    let literal_position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    let original = index(&package, "value", "literal");
    let mut literal = package["semantic_graph"]["nodes"][original].clone();
    literal["node_id"] = fixture_support::node_id("text-literal-membership");
    literal["semantic_type"] = bounds_id.clone();
    // A literal node carries its type by `semantic_type` and has no dependencies.
    literal["dependencies"] = json!([]);
    literal["body"] = json!({
        "term": "literal", "type": bounds_id, "value_kind": "text", "value": value
    });
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(literal);
    fixture_support::settle(&mut package);
    (package, literal_position)
}

pub(crate) fn text_bounds_package(members: &[Value], literal: &str) -> (Value, usize) {
    let (mut package, literal_position) = bounded_literal(0, 64, "nfc", json!(literal));
    let bounds_position = literal_position - 1;
    let integer = fixture_support::node_id(&fixture_support::family_key("a3a3"));
    let text = fixture_support::node_id(&fixture_support::family_key("a1a1"));
    let members = members
        .iter()
        .map(|member| {
            let value_kind = member["value_kind"].as_str().expect("member value kind");
            let type_id = match value_kind {
                "integer" => &integer,
                "text" => &text,
                other => panic!("unknown member value kind {other}"),
            };
            json!({"term": "binding", "name": member["name"], "value": {
                "term": "literal", "type": type_id, "value_kind": value_kind,
                "value": member["value"]
            }})
        })
        .collect::<Vec<_>>();
    package["semantic_graph"]["nodes"][bounds_position]["body"] =
        json!({"term": "aggregate", "members": members});
    fixture_support::settle(&mut package);
    (package, bounds_position)
}
