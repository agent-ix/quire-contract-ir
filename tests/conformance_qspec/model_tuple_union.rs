// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The published tuple and payload-variant fixture through production intake.

use super::{canonical_digest, fixtures_dir, proposal_dir_of, read_json};
use quire_contract_model::{
    CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageV2, CheckedPackageV2ReadResult,
};

/// Provenance: IR-1059 slice 2. Full TC-940 and FR-069 conformance remains open.
#[test]
fn published_model_tuple_union_fixture_admits() {
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let vector = read_json(&proposal.join("model-tuple-union-vectors.json"))
        .expect("published tuple/union vector");
    let selected = vector["selected_document"]
        .as_str()
        .expect("selected full Semantic IR document path");
    let document = read_json(&proposal.join(selected)).expect("selected full document");
    let package = read_json(&fixtures.join("positive-model-tuple-union.json"))
        .expect("published positive package");
    let digest = canonical_digest(&document);
    assert_eq!(vector["selection_digest"].as_str(), Some(digest.as_str()));
    assert_eq!(
        package["lock"]["model_selections"][0]["digest"].as_str(),
        Some(digest.as_str())
    );

    let mut evidence = CheckedPackageEvidence::new();
    evidence
        .insert_domain_package_document(digest, super::fixture_support::canonical(&document))
        .expect("selected document evidence");
    evidence.support_feature("quire.value.complete/v1");
    let result = CheckedPackageV2::read(
        &super::fixture_support::canonical(&package),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    );
    assert!(
        matches!(result, CheckedPackageV2ReadResult::Admitted(_)),
        "{result:?}"
    );
}
