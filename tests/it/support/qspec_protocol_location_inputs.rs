// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Package construction shared with QSpec's protocol-location driver.

use crate::support::checked_package::{
    positive_operation_identities, refresh_identity, settle, v2_all_families,
};
use serde_json::{json, Value};

pub(crate) fn fixture(mut rows: Vec<Value>) -> (Value, usize) {
    let mut wire = positive_operation_identities();
    let nodes = wire["semantic_graph"]["nodes"].as_array().unwrap();
    let position = nodes
        .iter()
        .position(|node| node["node_tag"] == "expression" && node["body"]["term"] == "application")
        .expect("the admitted fixture has a local application");
    let application = nodes[position]["node_id"].clone();
    let occurrence = nodes[position]["occurrences"][0].clone();
    for row in &mut rows {
        row["application"] = application.clone();
        row["occurrence"] = occurrence.clone();
    }
    wire["checked_term_data"] = json!({"equality_decisions":[], "protocol_locations":rows, "dispatch_tables":[], "probabilistic_claims":[]});
    refresh_identity(&mut wire);
    (wire, position)
}

pub(crate) fn published_leaf_fixture(vector: &Value) -> Value {
    let mut wire = v2_all_families();
    let call = positive_operation_identities()["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["node_tag"] == "expression" && node["body"]["term"] == "application")
        .unwrap()
        .clone();
    wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .push(call);
    settle(&mut wire);
    let nodes = wire["semantic_graph"]["nodes"].as_array().unwrap();
    let application = nodes
        .iter()
        .find(|node| node["node_tag"] == "expression" && node["body"]["term"] == "application")
        .unwrap();
    let published_nodes = vector["graph"]["nodes"].as_array().unwrap();
    let rows = vector["value"].as_array().unwrap();
    let bound_rows: Vec<_> = rows
        .iter()
        .map(|row| {
            let published = published_nodes
                .iter()
                .find(|node| node["node_id"] == row["application"])
                .expect("published row names a graph node");
            let selected =
                if published["node_tag"] == "value" || published["semantic_form"] == "reference" {
                    nodes
                        .iter()
                        .find(|node| {
                            node["node_tag"] == published["node_tag"]
                                && node["semantic_form"] == published["semantic_form"]
                        })
                        .expect("admitted fixture contains the published leaf form")
                } else {
                    assert_eq!(published["node_tag"], "expression");
                    assert_eq!(published["body"]["term"], "application");
                    application
                };
            let mut bound = row.clone();
            bound["application"] = selected["node_id"].clone();
            bound["occurrence"] = selected["occurrences"][0].clone();
            bound
        })
        .collect();
    wire["checked_term_data"] = json!({"equality_decisions":[], "protocol_locations":bound_rows, "dispatch_tables":[], "probabilistic_claims":[]});
    refresh_identity(&mut wire);
    wire
}
