// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The published tuple and payload-variant fixture through production intake.

use super::{canonical_digest, fixtures_dir, proposal_dir_of, read_json};
use quire_contract_model::{
    CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageRefusalCause as Cause,
    CheckedPackageRefusalCode as Code, CheckedPackageV2, CheckedPackageV2ReadResult,
};
use serde_json::json;

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

/// IR-1059 slice 2: unsupported selected payloads fail at model intake with
/// the selected document member retained as a public location. The complete
/// published FR-069-AC-6 mutation matrix remains open.
#[test]
fn unsupported_selected_variant_payloads_refuse_at_document_member() {
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let selected = read_json(&proposal.join("domain-package-acme-shapes.json"))
        .expect("selected full Semantic IR document");
    let base = read_json(&fixtures.join("positive-model-tuple-union.json"))
        .expect("published positive package");
    for payload_type in [
        "ix://quire/native/Rational",
        "ix://quire/native/Decimal",
        "ix://quire/native/Text",
        "ix://acme/shapes/Raw",
    ] {
        let mut document = selected.clone();
        let mut package = base.clone();
        if payload_type == "ix://acme/shapes/Raw" {
            document["constructs"]
                .as_array_mut()
                .expect("constructs")
                .push(json!({"kind":{"module":"acme/shapes","name":"raw"},
                    "construct":{"meaning":"quire.meaning.model.value-type/v1"}}));
            document["types"]
                .as_array_mut()
                .expect("types")
                .push(json!({"identity":payload_type,
                    "kind":{"module":"acme/shapes","name":"raw"},
                    "scalar":"integer","constraints":[]}));
        }
        document["types"][5]["variants"][0]["payloadType"] = json!(payload_type);
        let digest = canonical_digest(&document);
        package["lock"]["model_selections"][0]["digest"] = json!(digest);
        package["identity_preimage"]["model_selections"][0]["digest"] = json!(digest);
        super::fixture_support::refresh_identity(&mut package);

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
        let CheckedPackageV2ReadResult::Refused(refusal) = result else {
            panic!("{payload_type}: expected unsupported payload intake refusal: {result:?}");
        };
        assert_eq!(refusal.code, Code::UnsupportedConstruct, "{payload_type}");
        assert_eq!(
            refusal.cause,
            Some(Cause::DeclarationForm),
            "{payload_type}"
        );
        assert_eq!(
            refusal.path.as_ref().map(ToString::to_string).as_deref(),
            Some("/lock/model_selections/0"),
            "{payload_type}",
        );
        assert_eq!(
            refusal
                .document_pointer
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            Some("/types/5/variants/0/payloadType"),
            "{payload_type}",
        );
    }
}
