// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! External default-feature consumer of the typed V2 envelope encoder.

use crate::support::checked_package::{canonical, evidence_for, v2_all_families};
use ix_trace_rs::trace;
use quire_canonical::{Error, Limits};
use quire_contract_model::{
    encode_checked_package_v2, CheckedCapability, CheckedDiagnosticsV2,
    CheckedPackageIdentityPreimageV2, CheckedPackageLockV2, CheckedPackageReadLimits,
    CheckedPackageRefusalCode, CheckedPackageV2, CheckedPackageV2ReadResult,
    CheckedSemanticGraphV2, CheckedSemanticId, CheckedSourceMapEntry, CHECKED_PACKAGE_V2,
    PACKAGE_DOMAIN_V2,
};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

fn typed<T: DeserializeOwned>(value: &Value) -> T {
    serde_json::from_value(value.clone()).expect("typed fixture member")
}

fn encode(value: &Value, limits: Limits) -> Result<Vec<u8>, Error> {
    let preimage: CheckedPackageIdentityPreimageV2 = typed(&value["identity_preimage"]);
    let package_id: CheckedSemanticId = typed(&value["package_id"]);
    let lock: CheckedPackageLockV2 = typed(&value["lock"]);
    let graph: CheckedSemanticGraphV2 = typed(&value["semantic_graph"]);
    let source_map: Vec<CheckedSourceMapEntry> = typed(&value["source_map"]);
    let capabilities: Vec<CheckedCapability> = typed(&value["capability_report"]);
    let diagnostics: CheckedDiagnosticsV2 = typed(&value["diagnostics"]);
    encode_checked_package_v2(
        &preimage,
        &package_id,
        &lock,
        &graph,
        &source_map,
        &capabilities,
        &diagnostics,
        limits,
    )
}

fn refused(
    bytes: &[u8],
    evidence: &quire_contract_model::CheckedPackageEvidence,
) -> CheckedPackageRefusalCode {
    match CheckedPackageV2::read(bytes, CheckedPackageReadLimits::bounded(), evidence) {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal.code,
        other => panic!("expected refusal, got {other:?}"),
    }
}

#[trace("TC-451", "FR-051-AC-1", "FR-051-AC-2", "FR-051-AC-3")]
#[test]
fn tc_451_typed_envelope_has_exact_members_and_canonical_boundaries() {
    let value = v2_all_families();
    let bytes = encode(&value, Limits::new(u64::MAX)).expect("valid typed members encode");
    assert_eq!(bytes, canonical(&value));
    let encoded: Value = serde_json::from_slice(&bytes).expect("JSON object");
    let members = encoded.as_object().expect("envelope object");
    assert_eq!(members.len(), 8);
    assert_eq!(members["contract_version"], CHECKED_PACKAGE_V2);
    assert_eq!(
        members.keys().map(String::as_str).collect::<Vec<_>>(),
        [
            "capability_report",
            "contract_version",
            "diagnostics",
            "identity_preimage",
            "lock",
            "package_id",
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

    let mut invalid = value;
    invalid["diagnostics"]["entries"] = json!([{
        "stage": "package_read", "code": "invalid_package", "cause_tag": "invalid-value",
        "details": [9_007_199_254_740_993_u64], "loci": []
    }]);
    assert!(matches!(
        encode(&invalid, Limits::new(u64::MAX)),
        Err(Error::IntegerMagnitudeAboveMaximum(9_007_199_254_740_993))
    ));
}

#[trace("TC-451", "FR-051-AC-3")]
#[test]
fn tc_451_reader_admits_matching_typed_envelope_and_refuses_mutations() {
    let mut value = v2_all_families();
    let preimage: CheckedPackageIdentityPreimageV2 = typed(&value["identity_preimage"]);
    let digest = quire_canonical::sha256(&preimage, Limits::new(u64::MAX))
        .expect("preimage canonical")
        .to_string();
    value["package_id"] = json!({
        "domain": PACKAGE_DOMAIN_V2, "algorithm": "sha256", "digest": digest,
    });
    let evidence = evidence_for(&value);
    let bytes = encode(&value, Limits::new(u64::MAX)).expect("typed encode");
    assert!(matches!(
        CheckedPackageV2::read(&bytes, CheckedPackageReadLimits::bounded(), &evidence),
        CheckedPackageV2ReadResult::Admitted(_)
    ));

    let mut stale_id = value.clone();
    stale_id["package_id"]["digest"] = json!("0".repeat(64));
    let id_bytes = encode(&stale_id, Limits::new(u64::MAX)).expect("stale identity still encodes");
    assert_eq!(
        serde_json::from_slice::<Value>(&id_bytes).expect("JSON")["package_id"],
        stale_id["package_id"]
    );
    assert_eq!(
        refused(&id_bytes, &evidence),
        CheckedPackageRefusalCode::StaleDependency
    );

    let mut stale_preimage = value.clone();
    stale_preimage["identity_preimage"]["required_features"] = json!(["changed-feature"]);
    let preimage_bytes =
        encode(&stale_preimage, Limits::new(u64::MAX)).expect("stale preimage still encodes");
    assert_eq!(
        serde_json::from_slice::<Value>(&preimage_bytes).expect("JSON")["identity_preimage"],
        stale_preimage["identity_preimage"]
    );
    assert_eq!(
        refused(&preimage_bytes, &evidence),
        CheckedPackageRefusalCode::StaleDependency
    );

    let mut missing = value.clone();
    missing
        .as_object_mut()
        .expect("object")
        .remove("diagnostics");
    assert_eq!(
        refused(&canonical(&missing), &evidence),
        CheckedPackageRefusalCode::MalformedWire
    );
    let mut unknown = value;
    unknown["future"] = json!(true);
    assert_eq!(
        refused(&canonical(&unknown), &evidence),
        CheckedPackageRefusalCode::UnknownMember
    );
    let text = std::str::from_utf8(&bytes).expect("UTF-8");
    let duplicate = format!(
        "{{\"contract_version\":\"{CHECKED_PACKAGE_V2}\",{}",
        &text[1..]
    );
    assert_eq!(
        refused(duplicate.as_bytes(), &evidence),
        CheckedPackageRefusalCode::DuplicateMember
    );
}
