// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Timed no-row qualification against QSpec's TN vectors by reference.

use super::*;
use quire_contract_model::CheckedModelTimeReadErrorCode;
use std::collections::BTreeSet;

fn vectors() -> Value {
    read_json(&proposal_dir_of(&fixtures_dir()).join("timed-clock-no-row-vectors.json"))
        .expect("published Timed no-row vectors")
}

fn timed_model_clause() -> Value {
    let mut wire = read_json(&fixtures_dir().join("positive-timed-model-nodes.json"))
        .expect("published timed model fixture");
    for selection in wire["lock"]["profile_selections"]
        .as_array_mut()
        .expect("profile selections")
    {
        if selection["role"] == "temporal_profile" {
            selection["definition"]["identity"] = json!("quire.temporal.timed/v1");
        }
    }
    for node in wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
    {
        if let Some(laws) = node
            .get_mut("body")
            .and_then(|body| body.get_mut("operation"))
            .and_then(|operation| operation.get_mut("laws"))
            .and_then(Value::as_array_mut)
        {
            for law in laws {
                if law["role"] == "temporal_profile" {
                    law["definition"]["identity"] = json!("quire.temporal.timed/v1");
                }
            }
        }
    }
    let clause = &mut wire["semantic_graph"]["nodes"][8];
    clause["body"]["arguments"][1]["value"] = json!("model-time");
    wire["semantic_graph"]["nodes"][29]["body"]["operation"]["member"] = json!({
        "kind":"temporal_interval", "interval": {
            "lower":{"numerator":"0","denominator":"1"},
            "upper":{"numerator":"3","denominator":"1"},
            "lower_end":"closed", "upper_end":"closed"
        }
    });
    fixture_support::settle(&mut wire);
    fixture_support::refresh_temporal_bindings(&mut wire);
    fixture_support::refresh_identity(&mut wire);
    wire
}

fn admitted(wire: &Value) -> Box<CheckedPackageV2> {
    match read_package(wire).expect("published selection evidence") {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("expected Timed admission: {other:?}"),
    }
}

fn refusal(wire: &Value) -> quire_contract_model::CheckedPackageRefusal {
    match read_package(wire).expect("published selection evidence") {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        other => panic!("expected Timed refusal: {other:?}"),
    }
}

/// Trace: FR-038-AC-482, FR-038-AC-483, FR-038-AC-484, TC-048.
#[trace("TC-048", "FR-038-AC-482", "FR-038-AC-483", "FR-038-AC-484")]
#[test]
fn timed_clock_no_row_vectors_join_model_and_refuse_extra_row() {
    let published = vectors();
    let cases = published["cases"].as_array().expect("TN cases");
    assert_eq!(
        cases
            .iter()
            .map(|case| case["id"].as_str().expect("case id"))
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["TN-01", "TN-02", "TN-03", "TN-04", "TN-05"])
    );
    let case = |id: &str| cases.iter().find(|case| case["id"] == id).expect("TN case");
    assert_eq!(case("TN-01")["clock_name"], "model-time");
    assert!(case("TN-01")["temporal_bindings"].is_null());

    let wire = timed_model_clause();
    assert!(wire.get("temporal_bindings").is_none(), "TN-01 has no row");
    let package = admitted(&wire);
    let clause = &package.graph().nodes[8];
    let binding = package
        .model_time_for_clause(&clause.node_id, 100)
        .expect("bounded model-time read")
        .expect("model subject");
    assert_eq!(binding.clause(), &clause.node_id);
    assert_eq!(
        binding.timed_root(),
        &package.document_held_id(52).expect("graph node").clone()
    );
    assert_eq!(
        binding.model(),
        &package.document_held_id(53).expect("graph node").clone()
    );
    assert_eq!(
        binding.source(),
        &package.document_held_id(44).expect("graph node").clone()
    );
    assert_eq!(
        binding.unit(),
        &package.document_held_id(43).expect("graph node").clone()
    );
    assert_eq!(
        package
            .model_time_for_clause(&clause.node_id, binding.work())
            .unwrap(),
        Some(binding)
    );
    let denied = package
        .model_time_for_clause(&clause.node_id, binding.work() - 1)
        .unwrap_err();
    assert_eq!(
        (denied.code, denied.work, denied.attempted_work),
        (
            CheckedModelTimeReadErrorCode::WorkLimit,
            binding.work() - 1,
            1
        )
    );
    assert_eq!(*denied.at, *binding.unit());
    let wrong_clause = package
        .model_time_for_clause(binding.source(), 100)
        .unwrap_err();
    assert_eq!(
        wrong_clause.code,
        CheckedModelTimeReadErrorCode::WrongClause
    );
    assert_eq!(*wrong_clause.at, *binding.source());

    // TN-02: a non-model parameter supplies trace timestamps through the
    // selected profile; the checked package manufactures no source or unit.
    let mut trace = wire.clone();
    trace["semantic_graph"]["nodes"][8]["body"]["arguments"][0]["target"] =
        trace["semantic_graph"]["nodes"][36]["node_id"].clone();
    fixture_support::settle(&mut trace);
    fixture_support::refresh_temporal_bindings(&mut trace);
    fixture_support::refresh_identity(&mut trace);
    let trace_package = admitted(&trace);
    let trace_clause = &trace_package
        .document_held_id(8)
        .expect("graph node")
        .clone();
    assert!(trace_package
        .model_time_for_clause(trace_clause, 100)
        .unwrap()
        .is_none());

    // TN-03: the clause selects a second admitted model type while the only
    // checked timed-model root still belongs to the original model subject.
    let mut missing = wire.clone();
    let alternate_model = missing["semantic_graph"]["nodes"][13]["node_id"].clone();
    let mut alternate_type = missing["semantic_graph"]["nodes"][26].clone();
    alternate_type["node_id"]["path"][2] = json!({"node":13});
    alternate_type["semantic_type"] = alternate_type["node_id"].clone();
    alternate_type["body"]["members"][0]["target"] = alternate_model.clone();
    alternate_type["dependencies"] = json!([alternate_model]);
    let alternate_type_id = alternate_type["node_id"].clone();
    missing["semantic_graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .push(alternate_type);
    let mut alternate = missing["semantic_graph"]["nodes"][27].clone();
    alternate["node_id"]["path"][4] = json!("alternate");
    alternate["node_id"]["path"][8] = json!({"node":60});
    alternate["semantic_type"] = alternate_type_id;
    alternate["body"]["members"][0]["value"]["value"] = json!("alternate");
    let alternate_id = alternate["node_id"].clone();
    missing["semantic_graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .push(alternate);
    missing["semantic_graph"]["nodes"][8]["body"]["arguments"][0]["target"] = alternate_id;
    fixture_support::settle(&mut missing);
    fixture_support::refresh_temporal_bindings(&mut missing);
    fixture_support::refresh_identity(&mut missing);
    let absent = refusal(&missing);
    assert_eq!(
        (
            absent.code,
            absent.cause,
            absent.loci[0].path.as_ref().map(ToString::to_string)
        ),
        (
            Code::InvalidPackage,
            Some(Cause::TimedNodeReference),
            Some(
                case("TN-03")["expected"]["pointer"]
                    .as_str()
                    .unwrap()
                    .replace("{clause-index}", "8")
            )
        )
    );
    assert_eq!(
        absent.locus,
        Some(
            serde_json::from_value(missing["semantic_graph"]["nodes"][8]["node_id"].clone())
                .unwrap()
        )
    );

    let mut wrong_name = wire.clone();
    wrong_name["semantic_graph"]["nodes"][8]["body"]["arguments"][1]["value"] =
        json!("model-steps");
    fixture_support::settle(&mut wrong_name);
    fixture_support::refresh_identity(&mut wrong_name);
    let wrong = refusal(&wrong_name);
    assert_eq!(
        (
            wrong.code,
            wrong.cause,
            wrong.loci[0].path.as_ref().map(ToString::to_string)
        ),
        (
            Code::InvalidPackage,
            Some(Cause::TimedNodeReference),
            Some("/semantic_graph/nodes/8/body".to_owned())
        )
    );

    let mut extra = wire.clone();
    let mut original = read_json(&fixtures_dir().join("positive-timed-model-nodes.json"))
        .expect("configured control");
    fixture_support::refresh_temporal_bindings(&mut original);
    fixture_support::refresh_identity_projection(&mut original);
    let mut row = original["temporal_bindings"][0].clone();
    row["declaration"] = extra["semantic_graph"]["nodes"][8]["node_id"].clone();
    extra["temporal_bindings"] = json!([row]);
    fixture_support::refresh_identity_projection(&mut extra);
    let actual = refusal(&extra);
    assert_eq!(
        (
            actual.code,
            actual.cause,
            actual.loci[0].path.as_ref().map(ToString::to_string)
        ),
        (
            Code::InvalidPackage,
            Some(Cause::TimedNodeReference),
            Some(
                case("TN-04")["expected"]["pointer"]
                    .as_str()
                    .unwrap()
                    .to_owned()
            )
        )
    );
    assert_eq!(
        actual.locus,
        Some(
            serde_json::from_value(extra["semantic_graph"]["nodes"][8]["node_id"].clone()).unwrap()
        )
    );

    // TN-05: the source fixture retains its configured row and still admits.
    assert_eq!(admitted(&original).temporal_bindings().unwrap().len(), 1);
}
