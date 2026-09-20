// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! QSpec #76 operation-law semantics (IR-216): a V2 `application` term's
//! `operation` member is validated against the vendored
//! `quire.checked-operation-catalog/v1`, closing the gap
//! `crates/quire-contract-model/src/checked_package/common.rs` used to
//! describe as admitted opaquely. Mirrors `checked_package_v2_frame_bodies`'s
//! `frame_mutations` treatment: every vendored `operation_vectors` entry
//! re-derives its digest and admits; every vendored `operation_mutations`
//! entry refuses with its own declared code and cause, never a single
//! generic code standing in for all ten.

#[path = "support/checked_package.rs"]
mod checked_package;

use checked_package::{
    canonical, evidence_for, fixture, node_identity_vectors, refresh_identity, sha256_hex,
};
use quire_contract_ir::{
    CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageRefusal,
    CheckedPackageRefusalCause, CheckedPackageRefusalCode, CheckedPackageV2,
    CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

fn read(value: &Value, evidence: &CheckedPackageEvidence) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(
        &canonical(value),
        CheckedPackageReadLimits::bounded(),
        evidence,
    )
}

fn refused(value: &Value) -> CheckedPackageRefusal {
    match read(value, &evidence_for(value)) {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        other => panic!("expected V2 refusal, got {other:?}"),
    }
}

fn operation_base() -> Value {
    fixture("checked-package-v2/fixtures/positive-operation-identities.json")
}

/// Parses a vendored `operation_mutations`/`operation_vectors` `expected_code`.
fn operation_refusal_code(expected: &str) -> CheckedPackageRefusalCode {
    match expected {
        "invalid_package" => CheckedPackageRefusalCode::InvalidPackage,
        "ill_typed" => CheckedPackageRefusalCode::IllTyped,
        other => panic!("unknown vendored operation refusal code {other}"),
    }
}

/// Parses a vendored `operation_mutations` `expected_cause`.
fn operation_refusal_cause(expected: &str) -> CheckedPackageRefusalCause {
    match expected {
        "stale-node-key" => CheckedPackageRefusalCause::StaleNodeKey,
        "unknown-operation" => CheckedPackageRefusalCause::UnknownOperation,
        "operation-class-mismatch" => CheckedPackageRefusalCause::OperationClassMismatch,
        "operation-law-missing" => CheckedPackageRefusalCause::OperationLawMissing,
        "operation-law-mismatch" => CheckedPackageRefusalCause::OperationLawMismatch,
        "operation-law-unselected" => CheckedPackageRefusalCause::OperationLawUnselected,
        "operation-mode-mismatch" => CheckedPackageRefusalCause::OperationModeMismatch,
        "operation-mode-type-mismatch" => CheckedPackageRefusalCause::OperationModeTypeMismatch,
        "operation-member-mismatch" => CheckedPackageRefusalCause::OperationMemberMismatch,
        "operator-ineligible" => CheckedPackageRefusalCause::OperatorIneligible,
        other => panic!("unknown vendored operation refusal cause {other}"),
    }
}

/// The position of the node an `operation_vectors`/`operation_mutations`
/// entry's preimage belongs at in [`operation_base`]: the node whose current
/// `node_id.digest` is the vector's own recorded `sha256` when one already
/// carries it verbatim, else (the two alternate integer-division law
/// vectors no fixture node is keyed under) the one node whose
/// `body.operation.identity` matches — unambiguous, since
/// `positive-operation-identities.json` carries exactly one application of
/// any given identity.
fn anchor_position(package: &Value, vector: &Value) -> usize {
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    let digest = vector["sha256"].as_str().expect("sha256");
    if let Some(position) = nodes
        .iter()
        .position(|node| node["node_id"]["digest"] == digest)
    {
        return position;
    }
    let identity = &vector["preimage"]["body"]["operation"]["identity"];
    nodes
        .iter()
        .position(|node| &node["body"]["operation"]["identity"] == identity)
        .unwrap_or_else(|| panic!("{}: no anchor node for identity {identity}", vector["name"]))
}

/// Splices one vector's full preimage (`node_tag`, `semantic_form`,
/// `semantic_type`, `body`) into `package` at `position`, sets the node's
/// key to the vector's own recorded (and here re-derived) digest, and adds
/// each law the preimage declares to the lock's `definition_selections`
/// when not already present, so a vector whose law the base fixture never
/// otherwise selects (the two alternate integer-division law vectors) can
/// admit on its own account rather than inheriting an unrelated refusal.
fn admit_vector(package: &Value, position: usize, vector: &Value) -> Value {
    let mut package = package.clone();
    let preimage = &vector["preimage"];
    let recorded = vector["sha256"].as_str().expect("sha256");
    assert_eq!(
        sha256_hex(&canonical(preimage)),
        recorded,
        "{}: vector re-derives its recorded digest",
        vector["name"]
    );
    let original_digest = package["semantic_graph"]["nodes"][position]["node_id"]["digest"]
        .as_str()
        .expect("node_id.digest")
        .to_owned();
    let node = &mut package["semantic_graph"]["nodes"][position];
    node["node_tag"] = preimage["node_tag"].clone();
    node["semantic_form"] = preimage["semantic_form"].clone();
    node["semantic_type"] = preimage["semantic_type"].clone();
    node["body"] = preimage["body"].clone();
    node["node_id"]["digest"] = json!(recorded);
    // A vector whose own digest differs from the anchor node's original one
    // (the two alternate integer-division law vectors) needs its
    // `source_map` entry rekeyed too, or the now-renamed node's occurrence
    // no longer matches any source-map entry.
    if original_digest != recorded {
        for entry in package["source_map"]
            .as_array_mut()
            .expect("source_map")
            .iter_mut()
        {
            if entry["node_id"]["digest"] == original_digest {
                entry["node_id"]["digest"] = json!(recorded);
            }
        }
    }
    let laws = preimage["body"]["operation"]["laws"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let selections = package["lock"]["definition_selections"]
        .as_array_mut()
        .expect("definition_selections");
    for law in laws {
        let definition = law["definition"].clone();
        if !selections.contains(&definition) {
            selections.push(definition);
        }
    }
    refresh_identity(&mut package);
    package
}

/// Splices one mutation's *patched* preimage body into `package` at
/// `position`, retaining the anchor node's original key when `rekey` is
/// false (the `stale_key` mutations, which must retain a now-wrong key) or
/// recomputing it from the patched preimage when `rekey` is true (every
/// other mutation, which must carry no *other* defect than the one the
/// mutation names).
fn apply_operation_mutation(
    package: &Value,
    position: usize,
    base_vector: &Value,
    patch: &Value,
    rekey: bool,
) -> Value {
    let mut package = package.clone();
    let mut preimage = base_vector["preimage"].clone();
    checked_package::apply_patch(&mut preimage, patch);
    package["semantic_graph"]["nodes"][position]["body"] = preimage["body"].clone();
    if rekey {
        let original_digest = package["semantic_graph"]["nodes"][position]["node_id"]["digest"]
            .as_str()
            .expect("node_id.digest")
            .to_owned();
        let fresh = sha256_hex(&canonical(&preimage));
        package["semantic_graph"]["nodes"][position]["node_id"]["digest"] = json!(fresh);
        // Mirrors admit_vector's own source_map rekeying: a rekeyed node's
        // occurrence must keep matching its source-map entry, or the graph
        // stage's InvalidSourceMap fires as a second, incidental defect
        // before this mutation's own named defect is ever reached.
        if original_digest != fresh {
            for entry in package["source_map"]
                .as_array_mut()
                .expect("source_map")
                .iter_mut()
            {
                if entry["node_id"]["digest"] == original_digest {
                    entry["node_id"]["digest"] = json!(fresh);
                }
            }
        }
    }
    refresh_identity(&mut package);
    package
}

/// FR-038-AC-16 records the vendored fixture tree's byte-identity to its
/// upstream QSpec commit; this is the production home's own byte-identity
/// to that *same* vendored file, so `schemas/checked-operation-catalog-v1.json`
/// (the file `operation_catalog::operation_catalog` embeds) and
/// `tests/fixtures/checked-package/checked-package-v2/operation-catalog.json`
/// (PROVENANCE's own digest, never edited) are proven to be one set of
/// bytes kept in two places, not two catalogs that can silently drift apart.
// Deliberately untraced: TC-054 is "Kani counterexample replay through the
// complete-V1 executor conforms" (FR-031-AC-3), status planned and
// discharged by the QSL crossing test at quire-spec-language#243, which has
// not started. This test only proves the production and vendored
// operation-catalog bytes stay identical — not what TC-054 names — so it
// binds itself to no criterion rather than claim that unstarted work is
// done.
#[test]
fn tc_054_production_operation_catalog_matches_the_vendored_fixture() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let production = fs::read(root.join("schemas/checked-operation-catalog-v1.json"))
        .expect("production catalog readable");
    let vendored = fs::read(
        root.join("tests/fixtures/checked-package/checked-package-v2/operation-catalog.json"),
    )
    .expect("vendored catalog readable");
    assert_eq!(
        production, vendored,
        "schemas/checked-operation-catalog-v1.json must stay byte-identical to the vendored fixture"
    );
}

/// Every vendored `operation_vectors` entry re-derives its recorded digest
/// (checked inside [`admit_vector`]) and, spliced into the position its own
/// identity or digest anchors it to in `positive-operation-identities.json`
/// (adding any law definition the base fixture does not already select),
/// admits.
// Deliberately untraced: TC-054 is "Kani counterexample replay through the
// complete-V1 executor conforms" (FR-031-AC-3), status planned and
// discharged by the QSL crossing test at quire-spec-language#243, which has
// not started — not what this test verifies. FR-038-AC-17 covers the
// `operation` member's presence, not law validation, and FR-038-AC-5 covers
// node-identity vector re-derivation, not operation-law admission; neither
// is what this test checks. FR-038 has no acceptance criterion for
// catalog-driven operation-law validation, so there is no row to bind to,
// and binding to any of these would be a false claim.
#[test]
fn tc_054_operation_vectors_admit() {
    let document = node_identity_vectors();
    let vectors = document["operation_vectors"]
        .as_array()
        .expect("operation_vectors");
    assert_eq!(vectors.len(), 21, "published operation_vectors count");
    let base = operation_base();
    for vector in vectors {
        let name = vector["name"].as_str().expect("name");
        let position = anchor_position(&base, vector);
        let package = admit_vector(&base, position, vector);
        match read(&package, &evidence_for(&package)) {
            CheckedPackageV2ReadResult::Admitted(_) => {}
            other => panic!("{name}: expected V2 admission, got {other:?}"),
        }
    }
}

/// Replays every vendored `operation_mutations` vector, asserting the exact
/// refused code and cause FR-038/QSpec #76 pins for each — never a single
/// code standing in for all ten declared causes. A `stale_key` mutation
/// retains its anchor node's original key (the defect under test); every
/// other mutation rekeys first, so the *only* defect left standing is the
/// one its own `expected_cause` names.
// Deliberately untraced: TC-054 is "Kani counterexample replay through the
// complete-V1 executor conforms" (FR-031-AC-3), status planned and
// discharged by the QSL crossing test at quire-spec-language#243, which has
// not started — not what this test verifies. FR-038-AC-5 covers
// node-identity vector re-derivation, not operation-law refusal causes,
// which is also not what this test checks. FR-038 has no acceptance
// criterion for catalog-driven operation-law validation, so there is no row
// to bind to, and binding to either would be a false claim.
#[test]
fn tc_054_operation_mutations_vectors_refuse_their_own_code_and_cause() {
    let document = node_identity_vectors();
    let vectors = document["operation_vectors"]
        .as_array()
        .expect("operation_vectors");
    let by_name = |name: &str| {
        vectors
            .iter()
            .find(|vector| vector["name"] == name)
            .unwrap_or_else(|| panic!("operation_vectors base {name}"))
    };
    let mutations = document["operation_mutations"]
        .as_array()
        .expect("operation_mutations");
    assert_eq!(mutations.len(), 24, "published operation_mutations count");
    let base_package = operation_base();

    let mut replayed = 0_usize;
    for mutation in mutations {
        let name = mutation["name"].as_str().expect("name");
        let base_vector = by_name(mutation["base"].as_str().expect("base"));
        let position = anchor_position(&base_package, base_vector);
        let rekey = mutation["kind"] != json!("stale_key");
        let package = apply_operation_mutation(
            &base_package,
            position,
            base_vector,
            &mutation["patch"],
            rekey,
        );
        let refusal = refused(&package);
        let expected_code =
            operation_refusal_code(mutation["expected_code"].as_str().expect("expected_code"));
        let expected_cause =
            operation_refusal_cause(mutation["expected_cause"].as_str().expect("expected_cause"));
        assert_eq!(refusal.code, expected_code, "{name}");
        assert_eq!(refusal.cause, Some(expected_cause), "{name}");
        replayed += 1;
    }
    assert_eq!(
        replayed,
        mutations.len(),
        "every published operation_mutations vector was replayed"
    );
}
