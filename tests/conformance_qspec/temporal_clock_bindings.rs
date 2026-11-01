// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Published STD-621 clock selections, read from the selected QSpec checkout.

use super::*;
use quire_contract_model::ClockConfiguration;

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
        .iter()
        .find(|row| row["role"] == "temporal_profile")
        .expect("temporal selection")
        .clone();
    if profile != old_profile {
        let mut selected_profile = selection;
        selected_profile["definition"]["identity"] = json!(profile);
        selections.insert(1, selected_profile);
    }
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
    let first_id = package["semantic_graph"]["nodes"][index]["node_id"].clone();
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
    row["declaration"] = second_id.clone();
    let rows = package["temporal_bindings"].as_array_mut().expect("rows");
    rows.push(row);
    rows.sort_by(|left, right| {
        let left: CheckedNodeId = serde_json::from_value(left["declaration"].clone()).expect("id");
        let right: CheckedNodeId =
            serde_json::from_value(right["declaration"].clone()).expect("id");
        left.cmp(&right)
    });
    let source_map = package["source_map"].as_array_mut().expect("source map");
    let mut source_entry = source_map
        .iter()
        .find(|entry| entry["node_id"] == first_id)
        .expect("first clause source")
        .clone();
    source_entry["node_id"] = second_id;
    source_map.push(source_entry);
}

fn read_temporal_package(package: &Value) -> CheckedPackageV2ReadResult {
    let proposal = proposal_dir_of(&fixtures_dir());
    let selected = vector_selected_documents(&proposal).expect("published selected documents");
    let document = proposal.join(positive_document(
        "positive-timed-clause-nodes.json",
        &selected,
    ));
    read_package_with_document(package, "temporal clock package", &document)
        .expect("published package read")
}

fn admitted(package: &Value, case: &Value, expected_rows: usize) -> Box<CheckedPackageV2> {
    let read = read_temporal_package(package);
    let CheckedPackageV2ReadResult::Admitted(admitted) = read else {
        panic!("{}: {read:?}", case["id"]);
    };
    let rows = admitted.temporal_bindings().expect("typed temporal rows");
    assert_eq!(rows.len(), expected_rows, "{}", case["id"]);
    if expected_rows == 1 {
        let expected: ClockConfiguration =
            serde_json::from_value(case["clock"].clone()).expect("vector clock");
        assert_eq!(rows[0].clock, expected, "{}", case["id"]);
    } else {
        assert_ne!(rows[0].declaration, rows[1].declaration);
    }
    admitted
}

fn refused(package: &Value, case: &Value) {
    let CheckedPackageV2ReadResult::Refused(refusal) = read_temporal_package(package) else {
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
        admitted(&selected_package(case), case, 1);
    }
    let mut multiple = selected_package(&positives[0]);
    add_second_clause(&mut multiple);
    let read = admitted(&multiple, &published["multi_clause_positive"], 2);
    let rows = read.temporal_bindings().expect("two typed clock rows");
    let expected_rows = multiple["temporal_bindings"]
        .as_array()
        .expect("published rows");
    assert_eq!(expected_rows.len(), 2);
    let expected_formula_indices = published["multi_clause_positive"]
        ["canonical_formula_node_indices"]
        .as_array()
        .expect("published formula indices");
    assert_eq!(expected_formula_indices.len(), 2);
    for (index, (row, expected)) in rows.iter().zip(expected_rows).enumerate() {
        let exact_id: CheckedNodeId =
            serde_json::from_value(expected["declaration"].clone()).expect("exact clause ID");
        assert_eq!(
            row.declaration, exact_id,
            "row {index} keeps its declaration"
        );
        assert_eq!(
            expected["declaration"]["path"][7]["node"], expected_formula_indices[index],
            "row {index} names the published formula identity"
        );
        assert_eq!(
            row.clock_name.as_ref(),
            published["multi_clause_positive"]["clock_names"][index]
                .as_str()
                .expect("published clock name")
        );
    }
    assert!(
        read.structural_key(&rows[0].declaration)
            .expect("first structural clause key")
            < read
                .structural_key(&rows[1].declaration)
                .expect("second structural clause key"),
        "the two rows remain in ascending structural key order"
    );
    let second_id = &expected_rows[1]["declaration"];
    let second_node = multiple["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| &node["node_id"] == second_id)
        .expect("second clause node");
    let mut missing_second = multiple.clone();
    missing_second["temporal_bindings"]
        .as_array_mut()
        .expect("rows")
        .remove(1);
    refused(
        &missing_second,
        &json!({
            "id": "TCB-11-missing-second-row",
            "expected_code": "invalid_package",
            "expected_cause": "timed-node-reference",
            "locus": format!("/semantic_graph/nodes/{second_node}/body")
        }),
    );

    for (period, locus) in [
        (
            json!({"kind": "integer", "decimal": "0"}),
            "/temporal_bindings/0/clock/period/decimal",
        ),
        (
            json!({"kind": "rational", "numerator": "2", "denominator": "4"}),
            "/temporal_bindings/0/clock/period",
        ),
    ] {
        let mut package = selected_package(&positives[1]);
        package["temporal_bindings"][0]["clock"]["period"] = period;
        refused(
            &package,
            &json!({"id": "fixed-sample-invalid-period", "expected_code": "malformed_wire", "locus": locus}),
        );
    }

    let schema_refusals = published["schema_refusals"]
        .as_array()
        .expect("schema refusals");
    assert_eq!(schema_refusals.len(), 3);
    for case in schema_refusals {
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

    let semantic_refusals = published["semantic_refusals"]
        .as_array()
        .expect("semantic refusals");
    assert_eq!(semantic_refusals.len(), 6);
    for case in semantic_refusals {
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

/// Trace: QSpec FR-322-AC-294, TC-134.
#[trace("TC-134", "FR-322-AC-294")]
#[test]
fn temporal_clock_writer_retains_a_configured_clause_row() {
    let published = vectors();
    let case = &published["positive"][0];
    let source = selected_package(case);
    let read = admitted(&source, case, 1);
    let encoded = encode_checked_package_v2(
        read.name(),
        read.lock(),
        read.graph(),
        CheckedPackageSourceCorrespondenceRef {
            source_map: read.source_map(),
            refinement_step_rows: read.refinement_step_rows(),
        },
        quire_contract_model::CheckedPackageSidecarsRef {
            checked_term_data: read.checked_term_data(),
            temporal_bindings: read.temporal_bindings(),
        },
        read.capability_report(),
        read.diagnostics(),
        quire_canonical::Limits::new(1 << 24),
    )
    .expect("typed temporal package encodes");
    let written: Value = serde_json::from_slice(&encoded).expect("canonical package JSON");
    assert_eq!(written["temporal_bindings"], source["temporal_bindings"]);
    admitted(&written, case, 1);
}
