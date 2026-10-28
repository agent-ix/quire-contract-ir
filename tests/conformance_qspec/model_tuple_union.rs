// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The published tuple and payload-variant fixture through production intake.

use super::{canonical_digest, fixtures_dir, proposal_dir_of, read_json};
use quire_contract_model::{
    CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageRefusalCause as Cause,
    CheckedPackageRefusalCode as Code, CheckedPackageV2, CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};
use std::collections::BTreeSet;

/// Apply one published RFC 6902 operation. The vector's patch is the source of
/// both the mutation and its expected package identity.
fn apply_wire_patch(package: &mut Value, patch: &Value) {
    let path = patch["path"].as_str().expect("published patch path");
    let (parent, member) = path.rsplit_once('/').expect("patch member");
    let target = package.pointer_mut(parent).expect("published patch parent");
    let operation = patch["op"].as_str().expect("published patch operation");
    match (operation, target) {
        ("replace", Value::Object(map)) => {
            *map.get_mut(member).expect("published replacement target") = patch["value"].clone();
        }
        ("replace", Value::Array(items)) => {
            let index = member.parse::<usize>().expect("published array index");
            *items.get_mut(index).expect("published replacement target") = patch["value"].clone();
        }
        ("add", Value::Object(map)) => {
            assert!(map
                .insert(member.to_owned(), patch["value"].clone())
                .is_none());
        }
        ("remove", Value::Object(map)) => {
            assert!(map.remove(member).is_some(), "published removal target");
        }
        ("remove", Value::Array(items)) => {
            let index = member.parse::<usize>().expect("published array index");
            assert!(index < items.len(), "published removal target");
            items.remove(index);
        }
        _ => panic!("unsupported published patch {operation} at {path}"),
    }
}

/// The published wire cases exercise the production reader's closed schema,
/// declaration join, typed operation and key-validation stages. Every row in
/// the authoritative vector is replayed, including rows added later.
/// FR-069-AC-6 remains open until its full intake matrix is replayed.
/// Provenance: IR-1059, QSpec TW-00 through TW-26.
#[test]
fn published_model_tuple_union_wire_cases_match_reader_refusals() {
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let vectors = read_json(&proposal.join("model-tuple-union-vectors.json"))
        .expect("published tuple/union vectors");
    assert_eq!(vectors["base"], "fixtures/positive-model-tuple-union.json");
    let selected_path = vectors["selected_document"]
        .as_str()
        .expect("selected Semantic IR document path");
    let selected = read_json(&proposal.join(selected_path)).expect("selected document");
    let digest = canonical_digest(&selected);
    assert_eq!(vectors["selection_digest"], digest);
    let base = read_json(&fixtures.join("positive-model-tuple-union.json"))
        .expect("published positive package");
    let mut evidence = CheckedPackageEvidence::new();
    evidence
        .insert_domain_package_document(digest, super::fixture_support::canonical(&selected))
        .expect("selected document evidence");
    evidence.support_feature("quire.value.complete/v1");
    let rows = vectors["wire_cases"]
        .as_array()
        .expect("published wire cases");
    let mut seen = BTreeSet::new();
    let mut mismatches = Vec::new();
    for row in rows {
        let id = row["id"].as_str().expect("published wire case id");
        assert!(seen.insert(id), "duplicate published wire case {id}");
        let mut package = base.clone();
        for patch in row["patch"].as_array().expect("published patch list") {
            apply_wire_patch(&mut package, patch);
        }
        let result = CheckedPackageV2::read(
            &super::fixture_support::canonical(&package),
            CheckedPackageReadLimits::bounded(),
            &evidence,
        );
        let expected = row["outcome"].as_str().expect("published outcome");
        match (expected, result) {
            ("admitted", CheckedPackageV2ReadResult::Admitted(_)) => {}
            (expected, CheckedPackageV2ReadResult::Refused(refusal))
                if expected.strip_prefix("refused:") == Some(super::code_word(refusal.code)) =>
            {
                if refusal.cause.map(super::cause_word) != row["cause"].as_str() {
                    mismatches.push(format!("{id}: wrong refusal cause {refusal:?}"));
                }
                let actual_path = refusal.path.as_ref().map(ToString::to_string);
                if actual_path.as_deref() != row["locus"].as_str() {
                    mismatches.push(format!(
                        "{id}: expected locus {}, got {refusal:?}",
                        row["locus"]
                    ));
                }
            }
            (expected, CheckedPackageV2ReadResult::Admitted(_)) => {
                mismatches.push(format!("{id}: expected {expected}, got admitted"));
            }
            (expected, CheckedPackageV2ReadResult::Refused(refusal)) => {
                mismatches.push(format!("{id}: expected {expected}, got {refusal:?}"));
            }
            (expected, actual) => {
                mismatches.push(format!("{id}: expected {expected}, got {actual:?}"))
            }
        }
    }
    for ordinal in 0..=26 {
        let id = format!("TW-{ordinal:02}");
        assert!(
            seen.contains(id.as_str()),
            "missing published wire case {id}"
        );
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// A selected declaration rekey may leave its old type key in a parameter.
/// That dependent dangling reference waits for key authentication, while an
/// independent graph defect still wins and ordinary stale keys keep digest order.
/// Provenance: IR-1059, QSpec FR-322 graph-shape and stale-key precedence.
#[test]
fn selected_declaration_rekey_preserves_independent_refusal_order() {
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let vectors = read_json(&proposal.join("model-tuple-union-vectors.json"))
        .expect("published tuple/union vectors");
    let selected_path = vectors["selected_document"]
        .as_str()
        .expect("selected document path");
    let selected = read_json(&proposal.join(selected_path)).expect("selected document");
    let digest = canonical_digest(&selected);
    let mut evidence = CheckedPackageEvidence::new();
    evidence
        .insert_domain_package_document(digest, super::fixture_support::canonical(&selected))
        .expect("selected document evidence");
    evidence.support_feature("quire.value.complete/v1");
    let base = read_json(&fixtures.join("positive-model-tuple-union.json"))
        .expect("published positive package");
    let tw05 = vectors["wire_cases"]
        .as_array()
        .expect("published wire rows")
        .iter()
        .find(|row| row["id"] == "TW-05")
        .expect("published stale tuple row");
    let mut rekeyed = base.clone();
    for patch in tw05["patch"].as_array().expect("published patch list") {
        apply_wire_patch(&mut rekeyed, patch);
    }
    rekeyed["semantic_graph"]["nodes"][31]["dependencies"] = json!([]);
    super::fixture_support::refresh_identity(&mut rekeyed);
    let result = CheckedPackageV2::read(
        &super::fixture_support::canonical(&rekeyed),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    );
    let CheckedPackageV2ReadResult::Refused(refusal) = result else {
        panic!("independent graph defect must refuse: {result:?}");
    };
    assert_eq!(refusal.code, Code::InvalidSemanticGraph);
    assert_eq!(refusal.cause, None);
    assert_eq!(
        refusal.path.as_ref().map(ToString::to_string).as_deref(),
        Some("/semantic_graph/nodes/31/dependencies")
    );

    let mut two_stale = base;
    two_stale["semantic_graph"]["nodes"][0]["body"] = tw05["patch"][0]["value"]["body"].clone();
    two_stale["semantic_graph"]["nodes"][14]["body"]["members"][0]["value"]["value"] = json!("qq");
    super::fixture_support::refresh_identity(&mut two_stale);
    let result = CheckedPackageV2::read(
        &super::fixture_support::canonical(&two_stale),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    );
    let CheckedPackageV2ReadResult::Refused(refusal) = result else {
        panic!("lower-digest structural stale key must refuse: {result:?}");
    };
    assert_eq!(refusal.code, Code::InvalidPackage);
    assert_eq!(refusal.cause, Some(Cause::StaleNodeKey));
    assert_eq!(
        refusal.path.as_ref().map(ToString::to_string).as_deref(),
        Some("/semantic_graph/nodes/14/node_id")
    );
}

/// A parameter's old scalar type key is dependent on a rekeyed structural
/// type, just as TW-05's old selected-model key is dependent on Pair's rekey.
/// Provenance: IR-1059, QSpec FR-322-AC-154.
#[test]
fn rekeyed_scalar_type_precedes_its_dependent_parameter_reference() {
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let vectors = read_json(&proposal.join("model-tuple-union-vectors.json"))
        .expect("published tuple/union vectors");
    let selected = read_json(
        &proposal.join(
            vectors["selected_document"]
                .as_str()
                .expect("selected document path"),
        ),
    )
    .expect("selected document");
    let digest = canonical_digest(&selected);
    let mut evidence = CheckedPackageEvidence::new();
    evidence
        .insert_domain_package_document(digest, super::fixture_support::canonical(&selected))
        .expect("selected document evidence");
    evidence.support_feature("quire.value.complete/v1");
    let mut package = read_json(&fixtures.join("positive-model-tuple-union.json"))
        .expect("published positive package");
    let scalar = &package["semantic_graph"]["nodes"][7];
    assert_eq!(scalar["node_tag"], "scalar_type");
    assert_eq!(scalar["semantic_form"], "boolean");
    let old_type = scalar["node_id"].clone();
    assert_eq!(scalar["semantic_type"], old_type);
    let rekeyed = json!({
        "domain": "quire.checked-semantic-node/v1",
        "digest": "0000000000000000000000000000000000000000000000000000000000000001"
    });
    package["semantic_graph"]["nodes"][7]["node_id"] = rekeyed.clone();
    package["semantic_graph"]["nodes"][7]["semantic_type"] = rekeyed.clone();
    package["semantic_graph"]["nodes"][14]["semantic_type"] = old_type.clone();
    let source_map = package["source_map"].as_array_mut().expect("source map");
    let scalar_source = source_map
        .iter_mut()
        .find(|row| row["node_id"] == old_type)
        .expect("published scalar source");
    scalar_source["node_id"] = rekeyed;
    super::fixture_support::refresh_identity(&mut package);
    let result = CheckedPackageV2::read(
        &super::fixture_support::canonical(&package),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    );
    let CheckedPackageV2ReadResult::Refused(refusal) = result else {
        panic!("rekeyed scalar type must refuse before dependent parameter: {result:?}");
    };
    assert_eq!(refusal.code, Code::InvalidPackage);
    assert_eq!(refusal.cause, Some(Cause::StaleNodeKey));
    assert_eq!(
        refusal.path.as_ref().map(ToString::to_string).as_deref(),
        Some("/semantic_graph/nodes/7/node_id")
    );
}

/// Translate the published projection mutation to the corresponding member of
/// QSpec's full Semantic IR document. The projection ordinals are represented
/// by position identities in that document.
fn apply_published_intake_patch(document: &mut serde_json::Value, row: &serde_json::Value) {
    let name = row["name"].as_str().expect("published intake case name");
    let patch = row["patch"].as_array().expect("published intake patch");
    if name == "TK-10" {
        assert!(patch.is_empty(), "published positive intake case");
        return;
    }
    assert_eq!(patch.len(), 1, "{name}: one published mutation");
    let value = &patch[0]["value"];
    let expected_path = match name {
        "TK-01" => "/tuple_types/0/positions",
        "TK-02" | "TK-06" => "/tuple_types/0/positions/1/ordinal",
        "TK-03" | "TK-05" => "/tuple_types/0/positions/1/typeRef",
        "TK-04" => "/tuple_types/0/fields",
        "TK-07" | "TK-08" => "/variant_types/0/cases/0/payloadType",
        "TK-09" => "/tuple_types/0/positions/0/typeRef",
        _ => panic!("unrecognized published intake case {name}"),
    };
    assert_eq!(patch[0]["path"], expected_path, "{name}: mutation locus");
    let expected_op = match name {
        "TK-03" => "remove",
        "TK-04" => "add",
        _ => "replace",
    };
    assert_eq!(patch[0]["op"], expected_op, "{name}: mutation operation");
    match name {
        "TK-01" => document["types"][1]["positions"] = value.clone(),
        "TK-02" | "TK-06" => {
            document["types"][1]["positions"][1]["identity"] = json!(format!(
                "ix://acme/shapes/Pair/p{}",
                value.as_u64().expect("published ordinal")
            ))
        }
        "TK-03" => {
            let removed = document["types"][1]["positions"][1]
                .as_object_mut()
                .expect("published position")
                .remove("typeRef");
            assert!(removed.is_some(), "{name}: published remove target");
        }
        "TK-04" => document["types"][1]["fields"] = value.clone(),
        "TK-05" => document["types"][1]["positions"][1]["typeRef"] = value.clone(),
        "TK-09" => document["types"][1]["positions"][0]["typeRef"] = value.clone(),
        "TK-07" | "TK-08" => document["types"][5]["variants"][0]["payloadType"] = value.clone(),
        _ => unreachable!("checked published intake case"),
    }
}

/// Provenance: IR-1059, QSpec FR-322 intake rows TK-01 through TK-10.
/// The remaining FR-069-AC-6 wire and intake combinations stay open.
#[test]
fn published_model_tuple_union_intake_cases_refuse_at_exact_member() {
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let vectors = read_json(&proposal.join("model-tuple-union-vectors.json"))
        .expect("published tuple/union vectors");
    assert_eq!(vectors["base"], "fixtures/positive-model-tuple-union.json");
    let selected_path = vectors["selected_document"]
        .as_str()
        .expect("selected full Semantic IR document path");
    let selected =
        read_json(&proposal.join(selected_path)).expect("selected full Semantic IR document");
    let base = read_json(&fixtures.join("positive-model-tuple-union.json"))
        .expect("published positive package");
    let rows = vectors["intake_cases"]
        .as_array()
        .expect("published intake cases");
    assert_eq!(rows.len(), 10, "published intake case inventory");
    for row in rows {
        let name = row["name"].as_str().expect("published intake case name");
        let mut document = selected.clone();
        let mut package = base.clone();
        apply_published_intake_patch(&mut document, row);
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
        if row["expected"]["admitted"] == true {
            assert!(
                matches!(result, CheckedPackageV2ReadResult::Admitted(_)),
                "{name}: {result:?}"
            );
            continue;
        }
        let expected = &row["expected"]["refused"];
        let CheckedPackageV2ReadResult::Refused(refusal) = result else {
            panic!("{name}: expected published refusal, got {result:?}");
        };
        assert_eq!(
            super::code_word(refusal.code),
            expected["code"].as_str().expect("code"),
            "{name}"
        );
        assert_eq!(
            refusal.cause.map(super::cause_word),
            expected["cause"].as_str(),
            "{name}"
        );
        assert_eq!(
            refusal.path.as_ref().map(ToString::to_string).as_deref(),
            Some("/lock/model_selections/0"),
            "{name}"
        );
        let projection_locus = expected["at"].as_str().expect("published refusal locus");
        let document_pointer = if let Some(suffix) = projection_locus.strip_prefix("/tuple_types/0")
        {
            format!("/types/1{suffix}")
        } else if let Some(suffix) = projection_locus.strip_prefix("/variant_types/0/cases") {
            format!("/types/5/variants{suffix}")
        } else {
            panic!("{name}: unexpected published refusal locus {projection_locus}")
        };
        assert_eq!(
            refusal
                .document_pointer
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            Some(document_pointer.as_str()),
            "{name}"
        );
    }
}

/// QSpec FR-322 reads duplicate tuple ordinals before the resulting gap and
/// checks all tuples before any variant payload in one selected document.
/// Provenance: IR-1059 partial AC-6 combined-defect order; full AC-6 remains open.
#[test]
fn combined_tuple_and_variant_defects_keep_published_intake_order() {
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let vectors = read_json(&proposal.join("model-tuple-union-vectors.json"))
        .expect("published tuple/union vectors");
    let selected_path = vectors["selected_document"]
        .as_str()
        .expect("selected document path");
    let selected = read_json(&proposal.join(selected_path)).expect("selected document");
    let base = read_json(&fixtures.join("positive-model-tuple-union.json"))
        .expect("published positive package");
    let rows = vectors["intake_cases"].as_array().expect("intake cases");
    for (first, expected_cause) in [
        ("TK-06", "conflicting-binding"),
        ("TK-02", "malformed-declaration"),
    ] {
        let mut document = selected.clone();
        let mut package = base.clone();
        for name in [first, "TK-07"] {
            let row = rows
                .iter()
                .find(|row| row["name"] == name)
                .expect("published intake row");
            apply_published_intake_patch(&mut document, row);
        }
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
            panic!("{first}: expected tuple defect before variant defect: {result:?}");
        };
        assert_eq!(refusal.code, Code::InvalidModelBinding, "{first}");
        assert_eq!(
            refusal.cause.map(super::cause_word),
            Some(expected_cause),
            "{first}"
        );
        assert_eq!(
            refusal.path.as_ref().map(ToString::to_string).as_deref(),
            Some("/lock/model_selections/0"),
            "{first}"
        );
        assert_eq!(
            refusal
                .document_pointer
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            Some("/types/1/positions/1"),
            "{first}"
        );
    }
}

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
