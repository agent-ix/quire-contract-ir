// SPDX-License-Identifier: AGPL-3.0-or-later
//! Read the owning QSpec probabilistic-claim vectors through the strict reader.

use crate::fixture_support::{canonical, evidence_for, refresh_identity, settle, v2_all_families};
use ix_trace_rs::trace;
use quire_contract_model::{
    encode_checked_package_v2, CheckedNodeId, CheckedOccurrenceRole, CheckedPackageLimit,
    CheckedPackageReadLimits, CheckedPackageRefusalCause, CheckedPackageRefusalCode,
    CheckedPackageSidecarsRef, CheckedPackageSourceCorrespondenceRef, CheckedPackageV2,
    CheckedPackageV2ReadResult,
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
        let mut occurrences = vec![positive["occurrence"].clone()];
        if positive["id"] == "PC-01-equal-name-span-left" {
            occurrences.push(json!({"role":"claim", "ordinal":1}));
        }
        nodes.push(json!({
            "node_id": declaration,
            "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": "claim",
            "semantic_form": "probabilistic_claim",
            "semantic_type": ty,
            "dependencies": [],
            "occurrences": occurrences,
            "body": {"term":"aggregate", "members":[]}
        }));
    }
    let mut other = positives[0]["declaration"].clone();
    other["package"] = json!(package);
    other["path"] = json!(["claim", "other"]);
    nodes.push(json!({
        "node_id": other, "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": "claim", "semantic_form": "analysis_claim", "semantic_type": ty,
        "dependencies": [], "occurrences": [{"role":"claim", "ordinal":0}],
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
}
