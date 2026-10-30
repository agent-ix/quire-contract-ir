// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The published QSpec reaches-population descriptors through the IR reader.

use super::*;
use quire_contract_model::CheckedNodeId;

fn node_index(package: &Value, tag: &str, form: &str) -> usize {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_tag"] == tag && node["semantic_form"] == form)
        .unwrap_or_else(|| panic!("missing {tag}/{form} node"))
}

fn append_node(package: &mut Value, node: Value) -> Value {
    let id = node["node_id"].clone();
    let mut source = package["source_map"][0].clone();
    source["node_id"] = id.clone();
    source["role"] = node["occurrences"][0]["role"].clone();
    source["ordinal"] = node["occurrences"][0]["ordinal"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(node);
    package["source_map"]
        .as_array_mut()
        .expect("source map")
        .push(source);
    id
}

fn append_rekeyed(package: &mut Value, mut node: Value) -> Value {
    let self_typed = node["semantic_type"] == node["node_id"];
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    let id = fixture_support::structural_id(
        package["name"].as_str().expect("package name"),
        vec![
            node["node_tag"].clone(),
            node["semantic_form"].clone(),
            json!(format!("population-case-{position}")),
        ],
        0,
    );
    node["node_id"] = id.clone();
    if self_typed {
        node["semantic_type"] = id.clone();
    }
    append_node(package, node)
}

fn append_population(package: &mut Value, object: &Value, max: &str) -> Value {
    let integer = package["semantic_graph"]["nodes"][node_index(package, "scalar_type", "integer")]
        ["node_id"]
        .clone();
    let mut reference = package["semantic_graph"]["nodes"]
        [node_index(package, "composite_type", "reference")]
    .clone();
    reference["body"]["members"][0]["target"] = object.clone();
    reference["dependencies"] = json!([object]);
    let reference = append_rekeyed(package, reference);

    let mut set =
        package["semantic_graph"]["nodes"][node_index(package, "composite_type", "set")].clone();
    set["body"]["members"][0]["target"] = reference.clone();
    set["dependencies"] = json!([reference]);
    let set = append_rekeyed(package, set);

    let mut population = package["semantic_graph"]["nodes"]
        [node_index(package, "bounded_domain", "integer_range")]
    .clone();
    population["semantic_form"] = json!("model_population");
    population["semantic_type"] = set.clone();
    population["body"] = json!({"term":"aggregate", "members":[{
        "name":"max", "term":"binding", "value":{
            "term":"literal", "type":integer, "value_kind":"integer", "value":max
        }
    }]});
    let mut dependencies = vec![integer, set];
    dependencies.sort_by(|a, b| {
        let a: CheckedNodeId = serde_json::from_value(a.clone()).expect("node id");
        let b: CheckedNodeId = serde_json::from_value(b.clone()).expect("node id");
        a.cmp(&b)
    });
    population["dependencies"] = json!(dependencies);
    append_rekeyed(package, population)
}

fn candidate(base: &Value, case: &Value) -> Value {
    let mut package = base.clone();
    let application = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["body"]["operation"]["identity"] == "quire.op.model.reaches")
        .expect("reaches application");
    let descriptor = &case["population"];
    let link = if descriptor.is_null() {
        None
    } else if descriptor["absent"] == true {
        Some(fixture_support::node_id("missing-population"))
    } else if let Some(owner) = descriptor["model_population_over"].as_str() {
        let order_node = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .find(|node| {
                node["node_tag"] == "model" && node["owner"]["node"] == "ix://acme/orders/Order"
            })
            .expect("Order declaration")
            .clone();
        let order = order_node["node_id"].clone();
        let object = if owner == "ix://acme/orders/Order" {
            order
        } else {
            let mut other = order_node;
            other["owner"]["node"] = json!(owner);
            append_rekeyed(&mut package, other)
        };
        Some(append_population(
            &mut package,
            &object,
            descriptor["max"].as_str().expect("max"),
        ))
    } else {
        Some(
            package["semantic_graph"]["nodes"][node_index(
                &package,
                descriptor["node_tag"].as_str().expect("tag"),
                descriptor["semantic_form"].as_str().expect("form"),
            )]["node_id"]
                .clone(),
        )
    };
    if let Some(link) = link {
        let node = &mut package["semantic_graph"]["nodes"][application];
        node["body"]["operation"]["member"]["population"] = link.clone();
        if case["dependencies"] != "without-population" {
            let mut dependencies = node["dependencies"].as_array().expect("deps").clone();
            dependencies.push(link);
            dependencies.sort_by(|a, b| {
                let a: CheckedNodeId = serde_json::from_value(a.clone()).expect("node id");
                let b: CheckedNodeId = serde_json::from_value(b.clone()).expect("node id");
                a.cmp(&b)
            });
            dependencies.dedup();
            node["dependencies"] = json!(dependencies);
        }
    }
    // The published source-map regions are authoritative. `settle` rebuilds
    // every row with synthetic position regions and loses the declaration
    // correspondence of the QSpec base (including Example.Counter). The
    // application dependency was updated above; normalize only changed
    // owner paths and their exact references.
    fixture_support::refresh_identity(&mut package);
    package
}

#[trace(
    "TC-048",
    "FR-038-AC-269",
    "FR-038-AC-272",
    "FR-038-AC-273",
    "FR-038-AC-274",
    "FR-038-AC-276"
)]
#[test]
fn tc_048_qspec_reaches_population_cases_match_the_reader() {
    let dir = fixtures_dir();
    let adverse = read_json(&dir.join(ADVERSE)).expect("QSpec adverse fixture");
    let rows = mutations(&adverse, "reaches_population_cases").expect("published cases");
    assert_eq!(rows.len(), 7);
    let mut seen = std::collections::BTreeSet::new();
    for row in rows {
        let id = text_of(row, "id").expect("case id");
        assert!(seen.insert(id), "duplicate case {id}");
        let base = text_of(row, "base").expect("base");
        let package = read_json(&dir.join(format!("{base}.json"))).expect("published base");
        let package = candidate(&package, row);
        let got = reading_of(&package).unwrap_or_else(|why| panic!("{id}: {why}"));
        let expected = if row["outcome"] == "admitted" {
            "admitted".to_owned()
        } else if let Some(cause) = row["expected_cause"].as_str() {
            format!(
                "refused:{}/{cause}",
                text_of(row, "expected_code").expect("code")
            )
        } else {
            format!("refused:{}", text_of(row, "expected_code").expect("code"))
        };
        assert_eq!(got, expected, "{id}: {:?}", read_package(&package));
    }
}
