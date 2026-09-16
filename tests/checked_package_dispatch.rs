// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Version dispatch between the frozen CheckedPackage V1 reader and V2.

#[path = "support/checked_package.rs"]
mod checked_package;

use checked_package::{
    canonical, evidence_for, refusal, v1_all_families, v1_nominal_source, v2_all_families,
    v2_nominal,
};
use ix_trace_rs::trace;
use quire_contract_ir::{
    read_checked_package, CheckedPackage, CheckedPackageDispatchResult, CheckedPackageEvidence,
    CheckedPackageReadContext, CheckedPackageReadLimits, CheckedPackageReadResult,
    CheckedPackageRefusalCode, CheckedPackageV2, CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};

const V1_ALL_FAMILIES: &str = "4813ee9de2c4289de080ab51b826f6466e0020b27e09f2cd075edbc243b790b0";
const V1_NOMINAL_SOURCE: &str = "961bc8ba63e51481ea0e43c7b65af331c3a244722d099554b82874f932491275";
const V2_ALL_FAMILIES: &str = "a991d4133f11c444ee2cfd20263071b6021feb80f1d2d9a6c480ca87d2f7552d";
const V2_NOMINAL: &str = "a4a3d1699e33ceed6084fab17ce174bd6391a21fcd504bbf5d153710c1d2df39";

fn dispatch(bytes: &[u8], evidence: &CheckedPackageEvidence) -> CheckedPackageDispatchResult {
    read_checked_package(bytes, CheckedPackageReadLimits::bounded(), evidence)
}

fn assert_dispatch_refused(
    result: CheckedPackageDispatchResult,
    code: CheckedPackageRefusalCode,
    path: &str,
) {
    match result {
        CheckedPackageDispatchResult::Refused(actual) => assert_eq!(actual, refusal(code, path)),
        other => panic!("expected {code:?} at {path}, got {other:?}"),
    }
}

/// Tracing: TC-047, FR-038-AC-1
#[trace("TC-047", "FR-038-AC-1")]
#[test]
fn tc_047_dispatch_routes_each_recorded_package_to_its_exact_version() {
    for (fixture, digest) in [
        (v1_all_families(), V1_ALL_FAMILIES),
        (v1_nominal_source(), V1_NOMINAL_SOURCE),
    ] {
        match dispatch(&canonical(&fixture), &evidence_for(&fixture)) {
            CheckedPackageDispatchResult::AdmittedV1(package) => {
                assert_eq!(
                    package.package_id().domain.as_ref(),
                    "quire.package.semantic/v1"
                );
                assert_eq!(package.package_id().digest.as_ref(), digest);
            }
            other => panic!("expected V1 admission, got {other:?}"),
        }
    }
    for (fixture, digest) in [
        (v2_all_families(), V2_ALL_FAMILIES),
        (v2_nominal(), V2_NOMINAL),
    ] {
        let bytes = canonical(&fixture);
        let evidence = evidence_for(&fixture);
        let dispatched = match dispatch(&bytes, &evidence) {
            CheckedPackageDispatchResult::AdmittedV2(package) => package,
            other => panic!("expected V2 admission, got {other:?}"),
        };
        assert_eq!(
            dispatched.package_id().domain.as_ref(),
            "quire.package.semantic/v2"
        );
        assert_eq!(dispatched.package_id().digest.as_ref(), digest);
        // Dispatch and the direct V2 reader agree on the admitted package.
        match CheckedPackageV2::read(&bytes, CheckedPackageReadLimits::bounded(), &evidence) {
            CheckedPackageV2ReadResult::Admitted(direct) => assert_eq!(direct, dispatched),
            other => panic!("expected direct V2 admission, got {other:?}"),
        }
    }
}

/// Tracing: TC-047, FR-038-AC-1
#[trace("TC-047", "FR-038-AC-1")]
#[test]
fn tc_047_dispatch_refuses_unknown_absent_and_malformed_versions() {
    let fixture = v2_all_families();
    let evidence = evidence_for(&fixture);
    let with_version = |version: Value| {
        let mut value = fixture.clone();
        value["contract_version"] = version;
        canonical(&value)
    };
    for unknown in ["quire.checked-package/v3", "quire.checked-package/v0", ""] {
        assert_dispatch_refused(
            dispatch(&with_version(json!(unknown)), &evidence),
            CheckedPackageRefusalCode::UnknownContractVersion,
            "contract_version",
        );
    }
    for malformed in [json!(2), json!(null), json!(["quire.checked-package/v2"])] {
        assert_dispatch_refused(
            dispatch(&with_version(malformed), &evidence),
            CheckedPackageRefusalCode::MalformedWire,
            "contract_version",
        );
    }
    let mut absent = fixture.clone();
    absent
        .as_object_mut()
        .expect("fixture object")
        .remove("contract_version");
    assert_dispatch_refused(
        dispatch(&canonical(&absent), &evidence),
        CheckedPackageRefusalCode::MalformedWire,
        "contract_version",
    );
    assert_dispatch_refused(
        dispatch(b"[]", &evidence),
        CheckedPackageRefusalCode::MalformedWire,
        "document",
    );
    assert_dispatch_refused(
        dispatch(b"{\"contract_version\":", &evidence),
        CheckedPackageRefusalCode::MalformedWire,
        "document",
    );

    // The strict parse runs once, before any version is selected.
    let bytes = canonical(&fixture);
    let body = std::str::from_utf8(&bytes).expect("UTF-8 fixture");
    let duplicate = format!(
        "{{\"contract_version\":\"quire.checked-package/v1\",{}",
        body.trim_start_matches('{')
    );
    assert!(matches!(
        dispatch(duplicate.as_bytes(), &evidence),
        CheckedPackageDispatchResult::Refused(ref refused)
            if refused.code == CheckedPackageRefusalCode::DuplicateMember
    ));
    let mut spaced = b" ".to_vec();
    spaced.extend_from_slice(&bytes);
    assert_dispatch_refused(
        dispatch(&spaced, &evidence),
        CheckedPackageRefusalCode::NoncanonicalWire,
        "document",
    );
}

/// Tracing: TC-047, FR-038-AC-1
#[trace("TC-047", "FR-038-AC-1")]
#[test]
fn tc_047_frozen_v1_and_v2_readers_refuse_each_others_wire() {
    let empty = CheckedPackageReadContext::new();
    let v1_refusal = |value: &Value| match CheckedPackage::read(
        &canonical(value),
        CheckedPackageReadLimits::bounded(),
        &empty,
    ) {
        CheckedPackageReadResult::Refused(refused) => refused,
        other => panic!("frozen V1 reader must refuse V2 wire, got {other:?}"),
    };
    // V1 is not widened: a V2 label is unknown, and V2-only members are unknown.
    assert_eq!(
        v1_refusal(&v2_all_families()),
        refusal(
            CheckedPackageRefusalCode::UnknownContractVersion,
            "contract_version"
        )
    );
    let mut relabelled_nominal = v2_nominal();
    relabelled_nominal["contract_version"] = json!("quire.checked-package/v1");
    assert_eq!(
        v1_refusal(&relabelled_nominal),
        refusal(CheckedPackageRefusalCode::UnknownMember, "document")
    );

    let v1 = v1_all_families();
    let evidence = evidence_for(&v1);
    let v2_refusal = |value: &Value| match CheckedPackageV2::read(
        &canonical(value),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ) {
        CheckedPackageV2ReadResult::Refused(refused) => refused,
        other => panic!("V2 reader must refuse V1 wire, got {other:?}"),
    };
    assert_eq!(
        v2_refusal(&v1),
        refusal(
            CheckedPackageRefusalCode::UnknownContractVersion,
            "contract_version"
        )
    );
    // Relabelling V1 as V2 does not smuggle a V1 identity past the V2 reader.
    let mut relabelled = v1.clone();
    relabelled["contract_version"] = json!("quire.checked-package/v2");
    assert_eq!(
        v2_refusal(&relabelled),
        refusal(
            CheckedPackageRefusalCode::DigestDomainMismatch,
            "package_id.domain"
        )
    );
    relabelled["package_id"]["domain"] = json!("quire.package.semantic/v2");
    assert_eq!(
        v2_refusal(&relabelled),
        refusal(
            CheckedPackageRefusalCode::MalformedWire,
            "identity_preimage.version"
        )
    );

    // A V2 package carrying the V1 package domain is refused on V2.
    let mut cross = v2_all_families();
    cross["package_id"]["domain"] = json!("quire.package.semantic/v1");
    assert_dispatch_refused(
        dispatch(&canonical(&cross), &evidence_for(&cross)),
        CheckedPackageRefusalCode::DigestDomainMismatch,
        "package_id.domain",
    );
}
