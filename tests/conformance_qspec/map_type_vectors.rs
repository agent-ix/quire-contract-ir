// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Replay the owner-published Map forms without retaining the vector in IR.

use super::{fixtures_dir, map_type, proposal_dir_of, read_json, read_package_with_document};
use ix_trace_rs::trace;
use quire_contract_model::CheckedPackageV2ReadResult;
use serde_json::{json, Value};

fn mutate(case: &str, package: &mut Value, position: usize) {
    let package_name = package["name"].clone();
    let nodes = &mut package["semantic_graph"]["nodes"];
    match case {
        "MT-03" => {
            let boolean = nodes[0]["node_id"].clone();
            nodes[position]["node_id"]["path"][3] = json!({"node": 0});
            nodes[position]["semantic_type"] = nodes[position]["node_id"].clone();
            nodes[position]["body"]["members"][1]["value"]["target"] = boolean.clone();
            let text = nodes[position - 2]["node_id"].clone();
            nodes[position]["dependencies"] = json!([boolean, text]);
        }
        "MT-04" => {
            let integer = nodes[position - 1]["node_id"].clone();
            let sequence = json!({"package": package_name, "path": [
                "composite_type", "sequence", integer
            ], "ordinal": 0});
            let sequence_position = nodes.as_array().expect("nodes").len();
            nodes.as_array_mut().expect("nodes").push(json!({
                "node_id": sequence, "schema_version": "quire.checked-semantic-graph/v2",
                "node_tag": "composite_type", "semantic_form": "sequence",
                "semantic_type": sequence, "dependencies": [integer],
                "occurrences": [{"role": "type", "ordinal": 0}],
                "body": {"term": "aggregate", "members": [{"term": "reference", "target": integer}]}
            }));
            super::fixture_support::index_fixture_node_ids(package);
            let nodes = &mut package["semantic_graph"]["nodes"];
            let sequence = nodes[sequence_position]["node_id"].clone();
            nodes[position]["node_id"]["path"][2] = json!({"node": sequence_position});
            nodes[position]["semantic_type"] = nodes[position]["node_id"].clone();
            nodes[position]["body"]["members"][0]["value"]["target"] = sequence.clone();
            let integer = nodes[position - 1]["node_id"].clone();
            nodes[position]["dependencies"] = json!([sequence, integer]);
        }
        "MT-05" => {
            nodes[position]["body"]["members"]
                .as_array_mut()
                .expect("members")
                .pop();
        }
        "MT-06" => {
            let integer = nodes[position - 1]["node_id"].clone();
            nodes[position]["body"]["members"][0]["value"]["target"] = integer.clone();
            nodes[position]["dependencies"] = json!([integer]);
        }
        _ => {}
    }
    super::fixture_support::settle(package);
}

#[trace("TC-048", "FR-038-AC-488")]
#[test]
fn owner_map_type_vectors_match_public_reader() {
    let proposal = proposal_dir_of(&fixtures_dir());
    let vectors = read_json(&proposal.join("map-type-vectors.json")).expect("QSpec Map vectors");
    assert_eq!(vectors["version"], 1);
    let forms = vectors["forms"].as_array().expect("Map forms");
    assert_eq!(forms.len(), 7);
    for (index, form) in forms.iter().enumerate() {
        let id = format!("MT-{:02}", index + 1);
        assert_eq!(form["id"], id);
        if id == "MT-07" {
            assert_eq!(
                form["expected"],
                "admitted-clock-place-resolution-per-FR-415"
            );
            let timed = read_json(&proposal.join("fixtures/positive-timed-slot-nodes.json"))
                .expect("timed Map control");
            let nodes = timed["semantic_graph"]["nodes"]
                .as_array()
                .expect("timed nodes");
            let map_place = nodes
                .iter()
                .find(|node| {
                    node["semantic_form"] == "clock_place"
                        && node.pointer("/body/members/1/value/members/1/members/1/value/value")
                            == Some(&json!("map"))
                })
                .expect("published Map clock place");
            assert_eq!(
                map_place.pointer("/body/members/1/value/members/1/members/2/value/target/path"),
                Some(&json!(["scalar_type", "text"])),
            );
            assert!(!nodes.iter().any(|node| node["semantic_form"] == "clock"));
            let result = read_package_with_document(
                &timed,
                "MT-07",
                &proposal.join("domain-package-acme-timed-slots.json"),
            )
            .expect("selected timed Map evidence");
            assert!(
                matches!(result, CheckedPackageV2ReadResult::Admitted(_)),
                "{result:?}"
            );
            continue;
        }
        let (mut package, position) = map_type::map_package();
        let original = package["semantic_graph"]["nodes"][position]["node_id"].clone();
        mutate(&id, &mut package, position);
        let result = map_type::read(&package);
        match id.as_str() {
            "MT-01" => {
                assert_eq!(form["expected"], "admitted");
                assert_eq!(form["key"], "text");
                assert_eq!(form["value"], "integer");
                assert_eq!(
                    form["path"],
                    json!(["composite_type", "map", "<text-node>", "<integer-node>"])
                );
                let body = form["body"].as_array().expect("published Map body");
                assert_eq!(body.len(), 2);
                for (ordinal, name) in ["key", "value"].iter().enumerate() {
                    assert_eq!(body[ordinal]["term"], "binding");
                    assert_eq!(body[ordinal]["name"], *name);
                    assert_eq!(body[ordinal]["value"]["term"], "reference");
                    assert_eq!(body[ordinal]["value"]["target"], form[*name]);
                }
                assert!(
                    matches!(result, CheckedPackageV2ReadResult::Admitted(_)),
                    "{result:?}"
                );
            }
            "MT-02" => {
                assert_eq!(form["expected"], "same-node");
                assert_eq!(
                    package["semantic_graph"]["nodes"][position]["node_id"],
                    original
                );
                assert!(
                    matches!(result, CheckedPackageV2ReadResult::Admitted(_)),
                    "{result:?}"
                );
            }
            "MT-03" => {
                assert_eq!(form["expected"], "distinct-node");
                assert_ne!(
                    package["semantic_graph"]["nodes"][position]["node_id"],
                    original
                );
                assert!(
                    matches!(result, CheckedPackageV2ReadResult::Admitted(_)),
                    "{result:?}"
                );
            }
            "MT-04" | "MT-05" | "MT-06" => {
                let expected = &form["expected"];
                let CheckedPackageV2ReadResult::Refused(refusal) = result else {
                    panic!("{id} must refuse: {result:?}");
                };
                assert_eq!(
                    serde_json::to_value(refusal.code).expect("code"),
                    expected["code"]
                );
                assert_eq!(
                    serde_json::to_value(refusal.cause).expect("cause"),
                    expected.get("cause").cloned().unwrap_or(Value::Null)
                );
                let pointer = expected["pointer"]
                    .as_str()
                    .expect("pointer")
                    .replace("{n}", &position.to_string());
                assert_eq!(
                    refusal.loci[0].path.as_ref().map(ToString::to_string),
                    Some(pointer)
                );
            }
            _ => unreachable!(),
        }
    }
}
