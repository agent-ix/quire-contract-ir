// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Published STD-621 clock selections, read from the selected QSpec checkout.

use super::*;

fn vectors() -> Value {
    read_json(&proposal_dir_of(&fixtures_dir()).join("temporal-clock-binding-vectors.json"))
        .expect("published temporal clock vectors")
}

fn clause_index(package: &Value) -> usize {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("semantic nodes")
        .iter()
        .position(|node| {
            node["node_tag"] == "temporal" && node["semantic_form"] == "temporal_clause"
        })
        .expect("temporal clause")
}

fn selected_package(case: &Value) -> Value {
    let published = vectors();
    let fixture = published["fixture"].as_str().expect("fixture path");
    let mut package = read_json(&proposal_dir_of(&fixtures_dir()).join(fixture))
        .expect("published temporal clause package");
    let index = clause_index(&package);
    let clause = &package["semantic_graph"]["nodes"][index];
    let old_id = clause["node_id"].clone();
    let old_profile = clause["body"]["operation"]["laws"][0]["definition"]["identity"]
        .as_str()
        .expect("selected clause profile")
        .to_owned();
    let profile = case["profile"].as_str().expect("case profile");
    let path = package["semantic_graph"]["nodes"][index]["node_id"]["path"]
        .as_array_mut()
        .expect("structural path");
    for segment in path {
        if segment.as_str() == Some(old_profile.as_str()) {
            *segment = json!(profile);
        }
    }
    let new_id = package["semantic_graph"]["nodes"][index]["node_id"].clone();
    package["semantic_graph"]["nodes"][index]["body"]["operation"]["laws"][0]["definition"]
        ["identity"] = json!(profile);
    let selections = package["lock"]["profile_selections"]
        .as_array_mut()
        .expect("profile selections");
    let selection = selections
        .iter_mut()
        .find(|row| row["role"] == "temporal_profile")
        .expect("temporal selection");
    selection["definition"]["identity"] = json!(profile);
    fixture_support::reassign_node_id(&mut package, &old_id, &new_id);
    package["temporal_bindings"] = json!([{
        "declaration": new_id,
        "clock_name": case["clock_name"],
        "clock": case["clock"]
    }]);
    package
}

fn add_second_clause(package: &mut Value) {
    let index = clause_index(package);
    let mut second = package["semantic_graph"]["nodes"][index].clone();
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    let formula = nodes[28]["node_id"].clone();
    let previous = second["body"]["arguments"][5]["target"].clone();
    second["body"]["arguments"][5]["target"] = formula.clone();
    for dependency in second["dependencies"].as_array_mut().expect("dependencies") {
        if *dependency == previous {
            *dependency = formula.clone();
        }
    }
    for part in second["node_id"]["path"].as_array_mut().expect("path") {
        if *part == json!({"node": 29}) {
            *part = json!({"node": 28});
        }
    }
    let second_id = second["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(second);
    let mut row = package["temporal_bindings"][0].clone();
    row["declaration"] = second_id;
    let rows = package["temporal_bindings"].as_array_mut().expect("rows");
    rows.push(row);
    rows.sort_by(|left, right| {
        let left: CheckedNodeId = serde_json::from_value(left["declaration"].clone()).expect("id");
        let right: CheckedNodeId =
            serde_json::from_value(right["declaration"].clone()).expect("id");
        left.cmp(&right)
    });
}

fn admitted(package: &Value, case: &Value) {
    let read = read_package(package).expect("published package read");
    assert!(
        matches!(read, CheckedPackageV2ReadResult::Admitted(_)),
        "{}: {read:?}",
        case["id"]
    );
}

fn refused(package: &Value, case: &Value) {
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package(package).expect("published package read")
    else {
        panic!("{} must refuse", case["id"]);
    };
    assert_eq!(
        code_word(refusal.code),
        case["expected_code"],
        "{}",
        case["id"]
    );
    if let Some(cause) = case["expected_cause"].as_str() {
        assert_eq!(refusal.cause.map(cause_word), Some(cause), "{}", case["id"]);
    }
    assert_eq!(
        refusal.loci[0]
            .path
            .as_ref()
            .expect("refusal path")
            .to_string(),
        case["locus"],
        "{}",
        case["id"]
    );
}

/// Trace: QSpec FR-322-AC-294, TC-134, TC-233.
#[trace("TC-134", "TC-233", "FR-322-AC-294")]
#[test]
fn temporal_clock_binding_vectors_read_at_their_exact_loci() {
    let published = vectors();
    let positives = published["positive"].as_array().expect("positive cases");
    assert_eq!(positives.len(), 3);
    for case in positives {
        admitted(&selected_package(case), case);
    }
    let mut multiple = selected_package(&positives[0]);
    add_second_clause(&mut multiple);
    admitted(&multiple, &published["multi_clause_positive"]);

    for case in published["schema_refusals"]
        .as_array()
        .expect("schema refusals")
    {
        let mut package = selected_package(&positives[0]);
        let clock = &mut package["temporal_bindings"][0]["clock"];
        match case["mutation"].as_str().expect("schema mutation") {
            "unknown-kind" => clock["kind"] = json!("unknown"),
            "missing-sequence-authority" => {
                clock
                    .as_object_mut()
                    .expect("clock")
                    .remove("sequence_authority");
            }
            "wrong-sequence-authority-type" => clock["sequence_authority"] = json!(7),
            other => panic!("unhandled schema mutation {other}"),
        }
        refused(&package, case);
    }

    for case in published["semantic_refusals"]
        .as_array()
        .expect("semantic refusals")
    {
        let mut package = selected_package(&positives[0]);
        match case["mutation"].as_str().expect("semantic mutation") {
            "remove-row" => {
                package
                    .as_object_mut()
                    .expect("package")
                    .remove("temporal_bindings");
            }
            "absent-declaration" => {
                package["temporal_bindings"][0]["declaration"] = case["declaration"].clone();
            }
            "wrong-kind-declaration" => {
                let index = case["node_index"].as_u64().expect("node index") as usize;
                package["temporal_bindings"][0]["declaration"] =
                    package["semantic_graph"]["nodes"][index]["node_id"].clone();
            }
            "duplicate-row" => {
                let row = package["temporal_bindings"][0].clone();
                package["temporal_bindings"]
                    .as_array_mut()
                    .expect("rows")
                    .push(row);
            }
            "clock-name-mismatch" => {
                package["temporal_bindings"][0]["clock_name"] = json!("other-clock");
            }
            "profile-configuration-mismatch" => {
                package["temporal_bindings"][0]["clock"] = json!({
                    "kind": "fixed_sample",
                    "epoch": {"kind": "integer", "decimal": "0"},
                    "period": {"kind": "integer", "decimal": "1"},
                    "unit": "seconds"
                });
            }
            other => panic!("unhandled semantic mutation {other}"),
        }
        refused(&package, case);
    }
}
