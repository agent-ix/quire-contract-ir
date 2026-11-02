// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-322 AC-309: producer function paths and genuine admitted function bodies.

use super::monitor_plan_fixtures::*;
use super::*;
use std::collections::{BTreeMap, BTreeSet};

fn vectors() -> Value {
    read_json(&proposal_dir_of(&fixtures_dir()).join("entry-key-vectors.json"))
        .expect("selected EK recipes")
}
fn context(vectors: &Value) -> (Value, BTreeMap<u64, Value>) {
    let facts = recipes();
    let (mut wire, root) = materialize(&facts["base"]);
    assert_eq!(
        root + 1,
        wire["semantic_graph"]["nodes"].as_array().unwrap().len()
    );
    wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .pop();
    let integer = find(&wire, "integer");
    let text = find(&wire, "text");
    let boolean = find(&wire, "boolean");
    let mut nodes = BTreeMap::new();
    let rows = vectors["cases"].as_array().unwrap();
    let parameters = rows
        .iter()
        .filter_map(|r| r["parameters"].as_array())
        .flatten()
        .map(|i| i.as_u64().unwrap())
        .collect::<BTreeSet<_>>();
    for (level, index) in parameters.into_iter().enumerate() {
        let body = aggregate(vec![
            binding("name", literal(&text, "text", json!(format!("ek_{index}")))),
            binding(
                "level",
                literal(&integer, "integer", json!(level.to_string())),
            ),
        ]);
        nodes.insert(
            index,
            append(
                &mut wire,
                "value",
                "parameter",
                Some(&integer),
                body,
                &format!("ek parameter {index}"),
            ),
        );
    }
    let roots = rows
        .iter()
        .flat_map(|r| [r["body_root"].as_u64(), r["measure"].as_u64()])
        .flatten()
        .collect::<BTreeSet<_>>();
    for index in roots {
        assert!(
            !nodes.contains_key(&index),
            "published parameter and expression contexts must not conflict"
        );
        let constant = constant(&mut wire, &integer, &format!("ek constant {index}"));
        let body = json!({"term":"application","operator":"conditional","operation":{"identity":"quire.op.control.if","laws":[],"leaves":[],"member":null,"mode":null},"result_type":integer,"arguments":[literal(&boolean,"boolean",json!(true)),reference(&constant),reference(&constant)]});
        nodes.insert(
            index,
            append(
                &mut wire,
                "expression",
                "conditional",
                Some(&integer),
                body,
                &format!("ek expression {index}"),
            ),
        );
    }
    finish(&mut wire);
    (wire, nodes)
}
fn function(wire: &mut Value, nodes: &BTreeMap<u64, Value>, row: &Value) -> usize {
    let integer = find(wire, "integer");
    let parameters = row["parameters"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| reference(&nodes[&i.as_u64().unwrap()]))
        .collect();
    let mut body = vec![
        binding("parameters", aggregate(parameters)),
        binding(
            "body",
            reference(&nodes[&row["body_root"].as_u64().unwrap()]),
        ),
    ];
    if let Some(measure) = row["measure"].as_u64() {
        body.push(binding("measure", reference(&nodes[&measure])));
    }
    let id = append(
        wire,
        "function",
        row["semantic_form"].as_str().unwrap(),
        Some(&integer),
        aggregate(body),
        "EK function",
    );
    position(wire, &id)
}
fn relocated_path(path: &Value, wire: &Value, nodes: &BTreeMap<u64, Value>) -> Value {
    json!(path
        .as_array()
        .unwrap()
        .iter()
        .map(|segment| {
            segment.get("node").and_then(Value::as_u64).map_or_else(
                || segment.clone(),
                |index| json!({"node":position(wire,&nodes[&index])}),
            )
        })
        .collect::<Vec<_>>())
}
fn admit(wire: &Value) -> Box<CheckedPackageV2> {
    match read_package(wire).unwrap() {
        CheckedPackageV2ReadResult::Admitted(p) => p,
        other => panic!("complete ordinary EK function must admit: {other:?}"),
    }
}

#[trace("TC-048", "FR-038-AC-432")]
#[test]
fn entry_keys_published_paths_and_admission_cover_optional_measure() {
    let vectors = vectors();
    let rows = vectors["cases"].as_array().unwrap();
    assert_eq!(rows.len(), 8);
    let mut keys = BTreeMap::new();
    for row in rows
        .iter()
        .filter(|r| r["body_root"].is_number() && r["id"] != "EK-07")
    {
        let (mut wire, nodes) = context(&vectors);
        let n = function(&mut wire, &nodes, row);
        finish(&mut wire);
        assert_eq!(
            wire["semantic_graph"]["nodes"][n]["node_id"]["path"],
            relocated_path(&row["expected_path"], &wire, &nodes),
            "{} independent published path",
            row["id"]
        );
        let package = admit(&wire);
        let key = package.structural_key_at(n).unwrap().clone();
        if let Some(previous) = row["equal_to"].as_str() {
            assert_eq!(&key, &keys[previous]);
        }
        if let Some(previous) = row["differs_from"].as_str() {
            assert_ne!(&key, &keys[previous]);
        }
        keys.insert(row["id"].as_str().unwrap().to_owned(), key);
    }
    let (mut wire, nodes) = context(&vectors);
    let row = rows.iter().find(|r| r["id"] == "EK-07").unwrap();
    let first = function(&mut wire, &nodes, row);
    let duplicate = wire["semantic_graph"]["nodes"][first].clone();
    let second = wire["semantic_graph"]["nodes"].as_array().unwrap().len();
    wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    finish(&mut wire);
    let CheckedPackageV2ReadResult::Refused(refused) = read_package(&wire).unwrap() else {
        panic!("EK-07 duplicate identity must refuse")
    };
    assert_eq!(code_word(refused.code), row["code"]);
    assert_eq!(refused.cause, None);
    assert_eq!(
        refused.loci[0].path.as_ref().unwrap().as_str(),
        row["locus"]
            .as_str()
            .unwrap()
            .replace("{second}", &second.to_string())
    );
    wire["semantic_graph"]["nodes"][second]["node_id"]["ordinal"] = json!(1);
    finish(&mut wire);
    admit(&wire);
    let row = rows.iter().find(|r| r["id"] == "EK-08").unwrap();
    let (wire, _) = context(&vectors);
    let accessor = wire["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .position(|n| n["owner"]["kind"] == "model" && n["node_id"]["path"] == row["expected_path"])
        .expect("selected genuine model accessor");
    let package = admit(&wire);
    assert_eq!(
        serde_json::to_value(&package.graph().nodes[accessor].node_id).unwrap()["path"],
        row["expected_path"]
    );
}

#[trace("TC-048", "FR-038-AC-432")]
#[test]
fn entry_keys_measure_function_is_admitted_without_a_monitor_root() {
    let vectors = vectors();
    let row = vectors["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "EK-06")
        .unwrap();
    let (mut wire, nodes) = context(&vectors);
    let n = function(&mut wire, &nodes, row);
    finish(&mut wire);
    let package = admit(&wire);
    let body = package.term(&package.graph().nodes[n].node_id).unwrap();
    assert_eq!(
        serde_json::to_value(body).unwrap()["members"][2]["name"],
        "measure"
    );
    assert_eq!(package.monitor_plans().len(), 0);
}

#[trace("TC-048", "FR-038-AC-432")]
#[test]
fn entry_keys_assigned_label_is_resolved_without_recomputation() {
    let vectors = vectors();
    let row = &vectors["cases"][1];
    let (mut wire, nodes) = context(&vectors);
    let n = function(&mut wire, &nodes, row);
    finish(&mut wire);
    let before = admit(&wire);
    let old = wire["semantic_graph"]["nodes"][n]["node_id"].clone();
    let mut assigned = old.clone();
    assigned["path"] = json!(["assigned", "function", "label"]);
    fixture_support::reassign_node_id(&mut wire, &old, &assigned);
    finish(&mut wire);
    let after = admit(&wire);
    assert_eq!(
        after.structural_key_at(n).unwrap(),
        &serde_json::from_value::<CheckedNodeId>(assigned).unwrap()
    );
    assert_ne!(before.structural_key_at(n), after.structural_key_at(n));
    assert_eq!(before.graph().nodes[n].body, after.graph().nodes[n].body);
}
