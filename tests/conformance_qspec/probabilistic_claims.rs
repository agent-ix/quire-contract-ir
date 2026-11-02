// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX
//! Read the owning QSpec probabilistic-claim vectors through the strict reader.

use crate::fixture_support::{canonical, evidence_for, refresh_identity, settle, v2_all_families};
use ix_trace_rs::trace;
use quire_contract_model::{
    encode_checked_package_v2, CheckedCapability, CheckedDiagnosticsV2, CheckedNodeId,
    CheckedOccurrenceRole, CheckedPackageLimit, CheckedPackageLockV2, CheckedPackageReadLimits,
    CheckedPackageRefusalCause, CheckedPackageRefusalCode, CheckedPackageSidecarsRef,
    CheckedPackageSourceCorrespondenceRef, CheckedPackageV2, CheckedPackageV2ReadResult,
    CheckedProbabilisticClaim, CheckedRefinementStepRowRegion, CheckedSemanticGraphV2,
    CheckedSourceMapEntry, CheckedTermDataV2, TemporalBinding,
};
use serde_json::{json, Value};
use std::{fs, path::PathBuf};

fn vectors() -> Value {
    let root = PathBuf::from(
        std::env::var_os("QUIRE_SPECIFICATION_DIR")
            .expect("QUIRE_SPECIFICATION_DIR must select the owning QSpec checkout"),
    );
    let path = root.join("proposals/checked-package-v2/checked-term-data-vectors.json");
    serde_json::from_slice(&fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display())))
        .expect("QSpec checked-term vectors")
}

fn row(value: &Value, package: &str) -> Value {
    let mut declaration = value["declaration"].clone();
    if declaration["package"] == "example" {
        declaration["package"] = json!(package);
    }
    let mut row = json!({"declaration": declaration});
    for member in ["occurrence", "source_name"] {
        if let Some(value) = value.get(member) {
            row[member] = value.clone();
        }
    }
    row
}

fn fixture(rows: Vec<Value>, vectors: &Value) -> Value {
    let mut wire = v2_all_families();
    let package = wire["name"].as_str().expect("fixture package").to_owned();
    let ty = wire["semantic_graph"]["nodes"][0]["semantic_type"].clone();
    let positives = vectors["positive"]["probabilistic_claims"]
        .as_array()
        .expect("positive claim vectors");
    let mut nodes = Vec::new();
    for positive in positives {
        let mut declaration = positive["declaration"].clone();
        declaration["package"] = json!(package);
        let occurrences = if positive["id"] == "PC-01-equal-name-span-left" {
            let duplicate_context = vectors["reader_refusals"]
                .as_array()
                .expect("reader refusals")
                .iter()
                .find(|case| case["id"] == "PC-R4-duplicate-declaration-occurrence-row")
                .expect("published second occurrence");
            duplicate_context["checked_claim_context"]["declarations"]
                .as_array()
                .expect("claim declarations")
                .iter()
                .find(|node| node["node_id"] == positive["declaration"])
                .expect("first claim context")["occurrences"]
                .clone()
        } else {
            positive["resolved_node"]["occurrences"].clone()
        };
        nodes.push(json!({
            "node_id": declaration,
            "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": positive["resolved_node"]["node_tag"],
            "semantic_form": positive["resolved_node"]["semantic_form"],
            "semantic_type": ty,
            "dependencies": [],
            "occurrences": occurrences,
            "body": {"term":"aggregate", "members":[]}
        }));
    }
    let other_context = vectors["reader_refusals"]
        .as_array()
        .expect("reader refusals")
        .iter()
        .find(|case| case["id"] == "PC-R2-wrong-kind-node")
        .expect("published non-probabilistic declaration");
    let other_node = other_context["checked_claim_context"]["declarations"]
        .as_array()
        .expect("claim declarations")
        .iter()
        .find(|node| node["node_id"] == other_context["value"]["declaration"])
        .expect("wrong-kind node context");
    let mut other = other_node["node_id"].clone();
    other["package"] = json!(package);
    nodes.push(json!({
        "node_id": other, "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": other_node["node_tag"], "semantic_form": other_node["semantic_form"], "semantic_type": ty,
        "dependencies": [], "occurrences": other_node["occurrences"],
        "body": {"term":"aggregate", "members":[]}
    }));
    wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("fixture nodes")
        .extend(nodes);
    wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("fixture nodes")
        .sort_by_key(|node| {
            serde_json::from_value::<CheckedNodeId>(node["node_id"].clone()).expect("node id")
        });
    settle(&mut wire);
    let first = row(&positives[0], &package)["declaration"].clone();
    let second = row(&positives[1], &package)["declaration"].clone();
    let same_span = wire["source_map"]
        .as_array()
        .expect("source map")
        .iter()
        .find(|entry| entry["node_id"] == first)
        .expect("first claim source")["regions"]
        .clone();
    for entry in wire["source_map"].as_array_mut().expect("source map") {
        if entry["node_id"] == first || entry["node_id"] == second {
            entry["regions"] = same_span.clone();
        }
    }
    let mut second_occurrence = wire["source_map"]
        .as_array()
        .expect("source map")
        .iter()
        .find(|entry| entry["node_id"] == first)
        .expect("first claim source")
        .clone();
    second_occurrence["ordinal"] = json!(1);
    wire["source_map"]
        .as_array_mut()
        .expect("source map")
        .push(second_occurrence);
    wire["source_map"]
        .as_array_mut()
        .expect("source map")
        .sort_by_key(|entry| {
            (
                serde_json::from_value::<CheckedNodeId>(entry["node_id"].clone()).expect("node id"),
                serde_json::from_value::<CheckedOccurrenceRole>(entry["role"].clone())
                    .expect("role"),
                entry["ordinal"].as_u64().expect("ordinal"),
            )
        });
    wire["checked_term_data"] = json!({
        "equality_decisions": [], "protocol_locations": [], "dispatch_tables": [],
        "probabilistic_claims": rows
    });
    refresh_identity(&mut wire);
    wire
}

fn read_at(wire: &Value, work: u64) -> CheckedPackageV2ReadResult {
    let mut limits = CheckedPackageReadLimits::bounded();
    limits.work = work;
    CheckedPackageV2::read(&canonical(wire), limits, &evidence_for(wire))
}

fn read(wire: &Value) -> CheckedPackageV2ReadResult {
    read_at(wire, CheckedPackageReadLimits::bounded().work)
}

fn encode_producer(wire: &Value, sidecar: Option<&CheckedTermDataV2>) -> Vec<u8> {
    let lock: CheckedPackageLockV2 = serde_json::from_value(wire["lock"].clone()).expect("lock");
    let graph: CheckedSemanticGraphV2<Value> =
        serde_json::from_value(wire["semantic_graph"].clone()).expect("graph");
    let source_map: Vec<CheckedSourceMapEntry> =
        serde_json::from_value(wire["source_map"].clone()).expect("source map");
    let refinement_step_rows: Option<Vec<CheckedRefinementStepRowRegion>> = wire
        .get("refinement_step_rows")
        .map(|value| serde_json::from_value(value.clone()).expect("refinement rows"));
    let temporal_bindings: Option<Vec<TemporalBinding>> = wire
        .get("temporal_bindings")
        .map(|value| serde_json::from_value(value.clone()).expect("temporal bindings"));
    let capabilities: Vec<CheckedCapability> =
        serde_json::from_value(wire["capability_report"].clone()).expect("capabilities");
    let diagnostics: CheckedDiagnosticsV2 =
        serde_json::from_value(wire["diagnostics"].clone()).expect("diagnostics");
    encode_checked_package_v2(
        wire["name"].as_str().expect("package name"),
        &lock,
        &graph,
        CheckedPackageSourceCorrespondenceRef {
            source_map: &source_map,
            refinement_step_rows: refinement_step_rows.as_deref(),
        },
        CheckedPackageSidecarsRef {
            checked_term_data: sidecar,
            temporal_bindings: temporal_bindings.as_deref(),
            schedulability_claims: None,
        },
        &capabilities,
        &diagnostics,
        quire_canonical::Limits::new(1 << 24),
    )
    .expect("typed producer envelope encodes")
}

#[test]
#[trace("TC-451", "TC-233", "FR-051-AC-6", "FR-019-AC-18")]
fn typed_pc_rows_encode_then_strictly_read() {
    let all = vectors();
    let rows: Vec<_> = all["positive"]["probabilistic_claims"]
        .as_array()
        .expect("published claim rows")
        .iter()
        .map(|value| row(value, "test/families"))
        .collect();
    assert_eq!(rows.len(), 2);
    let typed_rows: Vec<CheckedProbabilisticClaim> = rows
        .iter()
        .map(|row| CheckedProbabilisticClaim {
            declaration: serde_json::from_value(row["declaration"].clone())
                .expect("typed declaration"),
            occurrence: serde_json::from_value(row["occurrence"].clone())
                .expect("typed occurrence"),
        })
        .collect();
    let sidecar = CheckedTermDataV2 {
        probabilistic_claims: typed_rows.clone(),
        ..Default::default()
    };
    let wire = fixture(rows, &all);
    let bytes = encode_producer(&wire, Some(&sidecar));
    assert_eq!(bytes, canonical(&wire));
    let CheckedPackageV2ReadResult::Admitted(admitted) = CheckedPackageV2::read(
        &bytes,
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&wire),
    ) else {
        panic!("typed producer rows must strictly admit");
    };
    assert_eq!(
        admitted
            .checked_term_data()
            .expect("sidecar")
            .probabilistic_claims,
        typed_rows
    );
}

#[test]
#[trace("TC-451", "FR-051-AC-6", "FR-038-AC-390")]
fn absent_and_four_empty_arrays_remain_distinct_after_write_and_read() {
    let base = v2_all_families();
    let absent = encode_producer(&base, None);
    let empty = CheckedTermDataV2::default();
    let present = encode_producer(&base, Some(&empty));
    assert_ne!(absent, present);
    let CheckedPackageV2ReadResult::Admitted(absent_read) = CheckedPackageV2::read(
        &absent,
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&base),
    ) else {
        panic!("absent sidecar must admit");
    };
    let CheckedPackageV2ReadResult::Admitted(present_read) = CheckedPackageV2::read(
        &present,
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&base),
    ) else {
        panic!("present empty sidecar must admit");
    };
    assert!(absent_read.checked_term_data().is_none());
    assert_eq!(present_read.checked_term_data(), Some(&empty));
}

#[test]
#[trace(
    "TC-233",
    "FR-038-AC-391",
    "FR-038-AC-392",
    "FR-038-AC-393",
    "FR-038-AC-394",
    "FR-019-AC-18"
)]
fn published_probabilistic_claim_vectors_join_and_refuse() {
    let all = vectors();
    let package = "test/families";
    let positive_rows: Vec<_> = all["positive"]["probabilistic_claims"]
        .as_array()
        .expect("positive claim vectors")
        .iter()
        .map(|value| row(value, package))
        .collect();
    let positive = fixture(positive_rows.clone(), &all);
    let claim_regions: Vec<_> = positive["source_map"]
        .as_array()
        .expect("source map")
        .iter()
        .filter(|entry| {
            positive_rows
                .iter()
                .any(|row| row["declaration"] == entry["node_id"])
        })
        .map(|entry| entry["regions"].clone())
        .collect();
    assert_eq!(claim_regions.len(), 3);
    assert!(claim_regions.windows(2).all(|pair| pair[0] == pair[1]));
    let CheckedPackageV2ReadResult::Admitted(admitted) = read(&positive) else {
        panic!(
            "both published claim identities must admit: {:?}",
            read(&positive)
        );
    };
    assert_eq!(
        serde_json::to_value(
            &admitted
                .checked_term_data()
                .expect("sidecar")
                .probabilistic_claims
        )
        .expect("typed rows"),
        json!(positive_rows)
    );
    let encoded = encode_checked_package_v2(
        admitted.name(),
        admitted.lock(),
        admitted.graph(),
        CheckedPackageSourceCorrespondenceRef {
            source_map: admitted.source_map(),
            refinement_step_rows: admitted.refinement_step_rows(),
        },
        CheckedPackageSidecarsRef {
            checked_term_data: admitted.checked_term_data(),
            temporal_bindings: admitted.temporal_bindings(),
            schedulability_claims: admitted.schedulability_claim_rows(),
        },
        admitted.capability_report(),
        admitted.diagnostics(),
        quire_canonical::Limits::new(1 << 24),
    )
    .expect("admitted claim rows write");
    assert_eq!(encoded, canonical(&positive));
    for category in ["schema_invalid", "reader_refusals"] {
        for vector in all[category]
            .as_array()
            .expect("adverse vectors")
            .iter()
            .filter(|case| case["id"].as_str().is_some_and(|id| id.starts_with("PC-")))
        {
            let values = vector["value"]
                .as_array()
                .cloned()
                .unwrap_or_else(|| vec![vector["value"].clone()]);
            let wire = fixture(
                values.iter().map(|value| row(value, package)).collect(),
                &all,
            );
            let result = read(&wire);
            let CheckedPackageV2ReadResult::Refused(refusal) = result else {
                panic!("{} must refuse: {result:?}", vector["id"]);
            };
            assert_eq!(
                serde_json::to_value(refusal.code).unwrap(),
                vector["expected_code"]
            );
            if let Some(cause) = vector.get("expected_cause") {
                assert_eq!(serde_json::to_value(refusal.cause).unwrap(), *cause);
            }
            assert_eq!(
                refusal.loci[0].path.as_ref().expect("path").as_str(),
                vector["locus"].as_str().expect("expected locus"),
                "{}",
                vector["id"]
            );
        }
    }
}

#[test]
#[trace("TC-233", "FR-038-AC-391", "FR-038-AC-394")]
fn claim_row_coverage_and_exact_work_boundary() {
    let all = vectors();
    let rows: Vec<_> = all["positive"]["probabilistic_claims"]
        .as_array()
        .expect("positive rows")
        .iter()
        .map(|value| row(value, "test/families"))
        .collect();
    let wire = fixture(rows, &all);
    let (mut low, mut high) = (0, CheckedPackageReadLimits::bounded().work);
    assert!(matches!(
        read_at(&wire, high),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    while low < high {
        let middle = low + (high - low) / 2;
        match read_at(&wire, middle) {
            CheckedPackageV2ReadResult::Admitted(_) => high = middle,
            CheckedPackageV2ReadResult::Incomplete(_) => low = middle + 1,
            other => panic!("expected admission or work exhaustion: {other:?}"),
        }
    }
    let CheckedPackageV2ReadResult::Incomplete(incomplete) = read_at(&wire, low - 1) else {
        panic!("one below exact work limit must be incomplete")
    };
    assert_eq!(incomplete.limit_kind, CheckedPackageLimit::Work);
    assert_eq!(incomplete.consumed, low);
    assert_eq!(
        incomplete.path.expect("charged row").as_str(),
        "/checked_term_data/probabilistic_claims/1"
    );
    let mut missing = wire.clone();
    missing["checked_term_data"]["probabilistic_claims"]
        .as_array_mut()
        .expect("rows")
        .pop();
    refresh_identity(&mut missing);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&missing) else {
        panic!("uncovered claim must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::InvalidValue)
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().expect("path").as_str(),
        "/checked_term_data/probabilistic_claims"
    );
    missing
        .as_object_mut()
        .expect("wire")
        .remove("checked_term_data");
    refresh_identity(&mut missing);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&missing) else {
        panic!("missing sidecar must not erase claim rows");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
}

#[test]
#[trace("TC-233", "FR-038-AC-390", "FR-038-AC-392", "FR-038-AC-395")]
fn four_array_schema_and_graph_admission_precede_sidecar_join() {
    let all = vectors();
    let rows = all["positive"]["probabilistic_claims"]
        .as_array()
        .expect("positive rows")
        .iter()
        .map(|value| row(value, "test/families"))
        .collect();
    let valid = fixture(rows, &all);
    for field in [
        "equality_decisions",
        "protocol_locations",
        "dispatch_tables",
        "probabilistic_claims",
    ] {
        let mut missing = valid.clone();
        missing["checked_term_data"]
            .as_object_mut()
            .expect("sidecar")
            .remove(field);
        refresh_identity(&mut missing);
        let CheckedPackageV2ReadResult::Refused(refusal) = read(&missing) else {
            panic!("missing {field} must refuse");
        };
        assert_eq!(
            refusal.code,
            CheckedPackageRefusalCode::MalformedWire,
            "{field}"
        );
    }
    let mut extra = valid.clone();
    extra["checked_term_data"]["unexpected"] = json!([]);
    refresh_identity(&mut extra);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&extra) else {
        panic!("unknown sidecar field must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MalformedWire);

    let mut bad_role = valid.clone();
    bad_role["checked_term_data"]["probabilistic_claims"][0]["occurrence"]["role"] =
        json!("expression");
    refresh_identity(&mut bad_role);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&bad_role) else {
        panic!("wrong occurrence role must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.loci[0].path.as_ref().expect("path").as_str(),
        "/checked_term_data/probabilistic_claims/0/occurrence"
    );

    let mut wrong_kind = valid.clone();
    let other_kind = valid["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "scalar_type")
        .expect("scalar type")["node_id"]
        .clone();
    wrong_kind["checked_term_data"]["probabilistic_claims"][0]["declaration"] = other_kind;
    refresh_identity(&mut wrong_kind);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&wrong_kind) else {
        panic!("wrong-kind declaration must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.loci[0].path.as_ref().expect("path").as_str(),
        "/checked_term_data/probabilistic_claims/0/declaration"
    );

    let claim_position = valid["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["semantic_form"] == "probabilistic_claim")
        .expect("claim node");
    let mut malformed_body = valid.clone();
    malformed_body["semantic_graph"]["nodes"][claim_position]["body"] =
        json!({"term":"no_such_term"});
    refresh_identity(&mut malformed_body);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&malformed_body) else {
        panic!("malformed claim body must refuse before sidecar join");
    };
    assert!(refusal.loci[0]
        .path
        .as_ref()
        .expect("path")
        .as_str()
        .starts_with("/semantic_graph/nodes/"));

    let mut unknown_form = valid.clone();
    unknown_form["semantic_graph"]["nodes"][claim_position]["semantic_form"] =
        json!("unknown_claim_form");
    refresh_identity(&mut unknown_form);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&unknown_form) else {
        panic!("unknown claim form must refuse before sidecar join");
    };
    assert!(refusal.loci[0]
        .path
        .as_ref()
        .expect("path")
        .as_str()
        .starts_with("/semantic_graph/nodes/"));

    let mut broken_graph = valid.clone();
    let mut missing_dependency =
        broken_graph["semantic_graph"]["nodes"][claim_position]["node_id"].clone();
    missing_dependency["path"] = json!(["claim", "absent-dependency"]);
    broken_graph["semantic_graph"]["nodes"][claim_position]["dependencies"] =
        json!([missing_dependency]);
    refresh_identity(&mut broken_graph);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&broken_graph) else {
        panic!("broken graph join must refuse before sidecar join");
    };
    assert!(refusal.loci[0]
        .path
        .as_ref()
        .expect("path")
        .as_str()
        .starts_with("/semantic_graph/nodes/"));
}
