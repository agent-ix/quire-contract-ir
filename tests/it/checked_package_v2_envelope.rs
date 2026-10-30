// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Default-feature external consumer of the structural V2 envelope writer.

use crate::support::checked_package::{
    admitted_protocol_families, canonical, evidence_for, selected_interface_model_node, settle,
    v2_all_families, v2_nominal,
};
use ix_trace_rs::trace;
use quire_canonical::{Error, Limits};
use quire_contract_model::{
    encode_checked_package_v2, CheckedCapability, CheckedDiagnosticsV2, CheckedNodeProjectionV2,
    CheckedPackageLockV2, CheckedPackageReadLimits, CheckedPackageRefusalCode,
    CheckedPackageSourceCorrespondenceRef, CheckedPackageV2, CheckedPackageV2ReadResult,
    CheckedRefinementStepRowRegion, CheckedSemanticGraphV2, CheckedSourceMapEntry,
    CHECKED_PACKAGE_V2,
};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

fn typed<T: DeserializeOwned>(value: &Value) -> T {
    serde_json::from_value(value.clone()).expect("typed fixture member")
}

fn encode(value: &Value, limits: Limits) -> Result<Vec<u8>, Error> {
    let name = value["name"].as_str().expect("package name");
    let lock: CheckedPackageLockV2 = typed(&value["lock"]);
    let graph: CheckedSemanticGraphV2<Value> = typed(&value["semantic_graph"]);
    let source_map: Vec<CheckedSourceMapEntry> = typed(&value["source_map"]);
    let refinement_step_rows: Option<Vec<CheckedRefinementStepRowRegion>> =
        value.get("refinement_step_rows").map(typed);
    let capabilities: Vec<CheckedCapability> = typed(&value["capability_report"]);
    let diagnostics: CheckedDiagnosticsV2 = typed(&value["diagnostics"]);
    encode_checked_package_v2(
        name,
        &lock,
        &graph,
        CheckedPackageSourceCorrespondenceRef {
            source_map: &source_map,
            refinement_step_rows: refinement_step_rows.as_deref(),
        },
        &capabilities,
        &diagnostics,
        limits,
    )
}

fn refused(value: &Value) -> CheckedPackageRefusalCode {
    let evidence = evidence_for(value);
    match CheckedPackageV2::read(
        &canonical(value),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ) {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal.code,
        other => panic!("expected refusal, got {other:?}"),
    }
}

/// Trace: FR-064-AC-1, FR-064-AC-6, TC-946.
#[trace("TC-946")]
#[test]
fn typed_writer_round_trips_protocol_clause_owner_model() {
    let mut value = admitted_protocol_families();
    value["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(selected_interface_model_node());
    settle(&mut value);
    let evidence = evidence_for(&value);
    let first = match CheckedPackageV2::read(
        &canonical(&value),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("owner-bearing fixture must admit: {other:?}"),
    };
    let clause = first
        .graph()
        .nodes
        .iter()
        .find(|node| {
            node.node_tag.as_ref() == "protocol" && node.semantic_form.as_ref() == "protocol_clause"
        })
        .expect("protocol clause");
    let owner = clause.owner_model.as_ref().expect("exact owner edge");
    assert!(clause.dependencies.contains(owner));
    assert_eq!(
        CheckedNodeProjectionV2::from(clause).owner_model.as_ref(),
        Some(owner)
    );

    let bytes = encode(&value, Limits::new(u64::MAX)).expect("typed writer");
    let written: Value = serde_json::from_slice(&bytes).expect("encoded JSON");
    let written_clause = written["semantic_graph"]["nodes"]
        .as_array()
        .expect("graph nodes")
        .iter()
        .find(|node| node["node_id"] == serde_json::to_value(&clause.node_id).expect("id"))
        .expect("written clause");
    assert_eq!(
        written_clause["owner_model"],
        serde_json::to_value(owner).expect("owner")
    );
    let second =
        match CheckedPackageV2::read(&bytes, CheckedPackageReadLimits::bounded(), &evidence) {
            CheckedPackageV2ReadResult::Admitted(package) => package,
            other => panic!("encoded owner-bearing fixture must admit: {other:?}"),
        };
    assert_eq!(second.name(), first.name());
    assert_eq!(second.graph().nodes, first.graph().nodes);
    assert_eq!(
        second
            .graph()
            .nodes
            .iter()
            .find(|node| node.node_id == clause.node_id)
            .expect("clause")
            .node_id,
        clause.node_id
    );
}

/// Trace: FR-051-AC-1, FR-051-AC-2, TC-451.
#[trace("TC-451")]
#[test]
fn typed_envelope_has_exact_members_and_canonical_boundaries() {
    let value = v2_all_families();
    let bytes = encode(&value, Limits::new(u64::MAX)).expect("valid typed members encode");
    assert_eq!(bytes, canonical(&value));
    let encoded: Value = serde_json::from_slice(&bytes).expect("JSON object");
    let members = encoded.as_object().expect("envelope object");
    assert_eq!(members.len(), 7);
    assert_eq!(members["contract_version"], CHECKED_PACKAGE_V2);
    assert_eq!(members["name"], value["name"]);
    assert_eq!(
        members.keys().map(String::as_str).collect::<Vec<_>>(),
        [
            "capability_report",
            "contract_version",
            "diagnostics",
            "lock",
            "name",
            "semantic_graph",
            "source_map"
        ]
    );
    let length = u64::try_from(bytes.len()).expect("byte length fits u64");
    assert_eq!(
        encode(&value, Limits::new(length)).expect("exact ceiling"),
        bytes
    );
    assert!(matches!(
        encode(&value, Limits::new(length - 1)),
        Err(Error::Limit(_))
    ));

    let mut invalid = value.clone();
    invalid["diagnostics"]["entries"] = json!([{
        "stage": "package_read", "code": "invalid_package", "cause_tag": "invalid-value",
        "details": [9_007_199_254_740_993_u64], "loci": []
    }]);
    assert!(matches!(
        encode(&invalid, Limits::new(u64::MAX)),
        Err(Error::IntegerMagnitudeAboveMaximum(9_007_199_254_740_993))
    ));

    let mut empty_name = value;
    empty_name["name"] = json!("");
    assert_eq!(
        refused(&empty_name),
        CheckedPackageRefusalCode::MalformedWire
    );
}

/// Trace: FR-051-AC-4, TC-451.
#[trace("TC-451")]
#[test]
fn borrowed_writer_preserves_body_values_order_and_optional_rows() {
    for mut value in [v2_all_families(), v2_nominal()] {
        let nodes = value["semantic_graph"]["nodes"].as_array().expect("nodes");
        assert!(!nodes.is_empty());
        assert!(nodes.iter().any(|node| !node["occurrences"]
            .as_array()
            .expect("occurrences")
            .is_empty()));
        let bytes = encode(&value, Limits::new(u64::MAX)).expect("typed encode");
        assert_eq!(bytes, canonical(&value));
        let written: Value = serde_json::from_slice(&bytes).expect("JSON");
        assert_eq!(
            written["semantic_graph"]["nodes"],
            value["semantic_graph"]["nodes"]
        );

        value["semantic_graph"]["nodes"][0]["body"] = json!({});
        assert_eq!(
            encode(&value, Limits::new(u64::MAX)).expect("empty body"),
            canonical(&value)
        );
        let mut nested = json!([null, true, -9_007_199_254_740_992_i64]);
        for _ in 0..48 {
            nested = json!({"child": nested});
        }
        value["semantic_graph"]["nodes"][0]["body"] = nested;
        let changed = encode(&value, Limits::new(u64::MAX)).expect("nested body");
        assert_eq!(changed, canonical(&value));
        let exact = u64::try_from(changed.len()).expect("length");
        assert_eq!(encode(&value, Limits::new(exact)).expect("exact"), changed);
        assert!(matches!(
            encode(&value, Limits::new(exact - 1)),
            Err(Error::Limit(_))
        ));
    }

    let mut optional = v2_all_families();
    let first = optional["source_map"][0].clone();
    optional["refinement_step_rows"] = json!([{
        "node_id": first["node_id"], "ordinal": 0,
        "region": first["regions"][0],
    }]);
    let bytes = encode(&optional, Limits::new(u64::MAX)).expect("optional rows");
    assert_eq!(bytes, canonical(&optional));
    assert_eq!(
        serde_json::from_slice::<Value>(&bytes).expect("JSON")["refinement_step_rows"],
        optional["refinement_step_rows"]
    );
}

/// Trace: FR-051-AC-3, TC-451.
#[trace("TC-451")]
#[test]
fn structural_package_round_trips_and_closed_envelope_refuses_mutations() {
    let value = v2_all_families();
    let evidence = evidence_for(&value);
    let bytes = encode(&value, Limits::new(u64::MAX)).expect("typed encode");
    assert!(matches!(
        CheckedPackageV2::read(&bytes, CheckedPackageReadLimits::bounded(), &evidence),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    let mut missing = value.clone();
    missing
        .as_object_mut()
        .expect("object")
        .remove("diagnostics");
    assert_eq!(refused(&missing), CheckedPackageRefusalCode::MalformedWire);
    let mut unknown = value.clone();
    unknown["future"] = json!(true);
    assert_eq!(refused(&unknown), CheckedPackageRefusalCode::UnknownMember);
    let text = std::str::from_utf8(&bytes).expect("UTF-8");
    let duplicate = format!(
        "{{\"name\":\"{}\",{}",
        value["name"].as_str().expect("name"),
        &text[1..]
    );
    assert!(matches!(
        CheckedPackageV2::read(duplicate.as_bytes(), CheckedPackageReadLimits::bounded(), &evidence),
        CheckedPackageV2ReadResult::Refused(refusal) if refusal.code == CheckedPackageRefusalCode::DuplicateMember
    ));
}
