// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! External writer checks for the optional producer-captured V2 sidecar.

use crate::support::checked_package::{canonical, v2_all_families};
use ix_trace_rs::trace;
use quire_canonical::{Error, Limits};
use quire_contract_model::{
    encode_checked_package_v2, CheckedCapability, CheckedDiagnosticsV2, CheckedPackageLockV2,
    CheckedPackageSourceCorrespondenceRef, CheckedSemanticGraphV2, CheckedSourceMapEntry,
    CheckedTermDataV2,
};
use serde_json::{json, Value};

fn encode(
    package: &Value,
    term_data: Option<&CheckedTermDataV2>,
    limits: Limits,
) -> Result<Vec<u8>, Error> {
    let lock: CheckedPackageLockV2 =
        serde_json::from_value(package["lock"].clone()).expect("typed lock");
    let graph: CheckedSemanticGraphV2<Value> =
        serde_json::from_value(package["semantic_graph"].clone()).expect("typed graph");
    let source_map: Vec<CheckedSourceMapEntry> =
        serde_json::from_value(package["source_map"].clone()).expect("typed source map");
    let capabilities: Vec<CheckedCapability> =
        serde_json::from_value(package["capability_report"].clone()).expect("typed capabilities");
    let diagnostics: CheckedDiagnosticsV2 =
        serde_json::from_value(package["diagnostics"].clone()).expect("typed diagnostics");
    let temporal_bindings: Option<Vec<quire_contract_model::TemporalBinding>> = package
        .get("temporal_bindings")
        .map(|value| serde_json::from_value(value.clone()).expect("typed temporal bindings"));

    encode_checked_package_v2(
        package["name"].as_str().expect("package name"),
        &lock,
        &graph,
        CheckedPackageSourceCorrespondenceRef {
            source_map: &source_map,
            refinement_step_rows: None,
        },
        quire_contract_model::CheckedPackageSidecarsRef {
            checked_term_data: term_data,
            temporal_bindings: temporal_bindings.as_deref(),
        },
        &capabilities,
        &diagnostics,
        limits,
    )
}

fn empty_term_data() -> CheckedTermDataV2 {
    serde_json::from_value(json!({
        "equality_decisions": [],
        "protocol_locations": [],
        "dispatch_tables": []
    }))
    .expect("closed empty sidecar")
}

/// Trace: FR-051-AC-1, FR-051-AC-4, TC-451.
#[trace("FR-051-AC-1", "FR-051-AC-4", "TC-451")]
#[test]
fn absent_and_present_empty_term_data_have_distinct_canonical_bytes() {
    let package = v2_all_families();
    let absent = encode(&package, None, Limits::new(u64::MAX)).expect("absent sidecar encodes");
    assert_eq!(absent, canonical(&package));

    let empty = empty_term_data();
    let present = encode(&package, Some(&empty), Limits::new(u64::MAX))
        .expect("present empty sidecar encodes");
    let mut expected = package;
    expected["checked_term_data"] = json!({
        "equality_decisions": [],
        "protocol_locations": [],
        "dispatch_tables": []
    });
    assert_eq!(present, canonical(&expected));
    assert_ne!(present, absent);
}

/// Trace: FR-051-AC-5, TC-451.
#[trace("FR-051-AC-5", "TC-451")]
#[test]
fn typed_sidecar_preserves_all_arrays_candidate_order_and_byte_ceiling() {
    let package = v2_all_families();
    let node = package["semantic_graph"]["nodes"][0]["node_id"].clone();
    let occurrence = json!({"role": "expression", "ordinal": 0});
    let mut raw = json!({
        "equality_decisions": [{
            "application": node,
            "occurrence": occurrence,
            "term_path": [],
            "operator": "equal",
            "left": {"operand": 0, "source_type": node, "conversion_target": null},
            "right": {"operand": 1, "source_type": node, "conversion_target": null},
            "schedule": "text",
            "units": [],
            "enum_members": []
        }],
        "protocol_locations": [{
            "application": node,
            "occurrence": occurrence,
            "term_ordinal": 0,
            "location": {
                "origin": {"kind": "type_declaration", "name": "Record"},
                "path": [1, 0]
            }
        }],
        "dispatch_tables": [{
            "application": node,
            "occurrence": occurrence,
            "operation": {"declaring": node, "operation": "run"},
            "context": node,
            "candidates": [
                {
                    "selection_key": {
                        "owner_effective_type": null,
                        "original": {"package": "example", "node": "Zeta"}
                    },
                    "operation_body": node,
                    "effective_precondition": null
                },
                {
                    "selection_key": {
                        "owner_effective_type": null,
                        "original": {"package": "example", "node": "Alpha"}
                    },
                    "operation_body": node,
                    "effective_precondition": node
                }
            ]
        }]
    });
    let mut second_equality = raw["equality_decisions"][0].clone();
    second_equality["occurrence"]["ordinal"] = json!(1);
    raw["equality_decisions"]
        .as_array_mut()
        .expect("equality array")
        .insert(0, second_equality);
    let mut second_protocol = raw["protocol_locations"][0].clone();
    second_protocol["term_ordinal"] = json!(1);
    raw["protocol_locations"]
        .as_array_mut()
        .expect("protocol array")
        .insert(0, second_protocol);
    let term_data: CheckedTermDataV2 =
        serde_json::from_value(raw.clone()).expect("closed typed sidecar");
    let bytes =
        encode(&package, Some(&term_data), Limits::new(u64::MAX)).expect("typed sidecar encodes");
    let mut expected = package;
    expected["checked_term_data"] = raw;
    assert_eq!(bytes, canonical(&expected));
    let written: Value = serde_json::from_slice(&bytes).expect("canonical JSON");
    assert_eq!(
        written["checked_term_data"]["equality_decisions"][0]["occurrence"]["ordinal"],
        1
    );
    assert_eq!(
        written["checked_term_data"]["protocol_locations"][0]["term_ordinal"],
        1
    );
    assert_eq!(
        written["checked_term_data"]["dispatch_tables"][0]["candidates"][0]["selection_key"]
            ["original"]["node"],
        "Zeta"
    );
    assert_eq!(
        written["checked_term_data"]["dispatch_tables"][0]["candidates"][1]["selection_key"]
            ["original"]["node"],
        "Alpha"
    );

    let exact = u64::try_from(bytes.len()).expect("byte length fits u64");
    assert_eq!(
        encode(&expected, Some(&term_data), Limits::new(exact)).expect("exact byte ceiling"),
        bytes
    );
    assert!(matches!(
        encode(&expected, Some(&term_data), Limits::new(exact - 1)),
        Err(Error::Limit(_))
    ));
}

/// Trace: FR-051-AC-4, FR-051-AC-5, TC-451.
#[trace("FR-051-AC-4", "FR-051-AC-5", "TC-451")]
#[test]
fn present_dispatch_table_with_empty_candidates_is_preserved() {
    let package = v2_all_families();
    let node = package["semantic_graph"]["nodes"][0]["node_id"].clone();
    let raw = json!({
        "equality_decisions": [],
        "protocol_locations": [],
        "dispatch_tables": [{
            "application": node,
            "occurrence": {"role": "expression", "ordinal": 0},
            "operation": {"declaring": node, "operation": "run"},
            "context": node,
            "candidates": []
        }]
    });
    let term_data: CheckedTermDataV2 =
        serde_json::from_value(raw.clone()).expect("closed empty candidate table");
    let bytes = encode(&package, Some(&term_data), Limits::new(u64::MAX))
        .expect("empty candidate table encodes");
    let mut expected = package;
    expected["checked_term_data"] = raw;
    assert_eq!(bytes, canonical(&expected));
    let written: Value = serde_json::from_slice(&bytes).expect("JSON");
    assert_eq!(
        written["checked_term_data"]["dispatch_tables"][0]["candidates"],
        json!([])
    );
}
