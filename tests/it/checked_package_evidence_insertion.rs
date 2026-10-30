// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Public evidence insertion and first-value retention controls.

use crate::support::checked_package::{
    canonical, domain_package_document, evidence_for, pointer, refresh_identity, select_model,
    settle, structural_id, v2_all_families, v2_nominal, COMPLETE_VALUE_FEATURE,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    read_checked_package, CheckedNodeId, CheckedPackageDispatchResult, CheckedPackageEvidence,
    CheckedPackageEvidenceInsertionError, CheckedPackageReadLimits, CheckedPackageRefusalCause,
    CheckedPackageRefusalCode, CheckedPackageV2, CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};
use std::sync::Arc;

fn read(package: &Value, evidence: &CheckedPackageEvidence) -> CheckedPackageV2ReadResult {
    let bytes = canonical(package);
    let limits = CheckedPackageReadLimits::bounded();
    let direct = CheckedPackageV2::read(&bytes, limits, evidence);
    let dispatched = read_checked_package(&bytes, limits, evidence);
    let expected = match &direct {
        CheckedPackageV2ReadResult::Admitted(package) => {
            CheckedPackageDispatchResult::AdmittedV2(package.clone())
        }
        CheckedPackageV2ReadResult::Refused(refusal) => {
            CheckedPackageDispatchResult::Refused(refusal.clone())
        }
        CheckedPackageV2ReadResult::Incomplete(incomplete) => {
            CheckedPackageDispatchResult::Incomplete(incomplete.clone())
        }
    };
    assert_eq!(dispatched, expected);
    direct
}

fn require_admission(result: CheckedPackageV2ReadResult) -> CheckedPackageV2 {
    match result {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("fixture must admit: {other:?}"),
    }
}

fn admit(package: &Value) -> Arc<CheckedPackageV2> {
    Arc::new(require_admission(read(package, &evidence_for(package))))
}

fn sentinels() -> CheckedPackageEvidence {
    let mut evidence = CheckedPackageEvidence::new();
    add_sentinels(&mut evidence);
    evidence
}

fn add_sentinels(evidence: &mut CheckedPackageEvidence) {
    evidence
        .insert_domain_package_document("unrelated domain", b"sentinel".to_vec())
        .expect("vacant sentinel domain");
    evidence
        .insert_dependency_package("unrelated dependency", admit(&v2_nominal()))
        .expect("vacant sentinel dependency");
    evidence.support_feature("unrelated feature");
    evidence.support_feature(COMPLETE_VALUE_FEATURE);
}

fn expect_domain_conflict(result: Result<(), CheckedPackageEvidenceInsertionError>, key: &str) {
    match result {
        Err(CheckedPackageEvidenceInsertionError::DomainDocumentConflict { identity }) => {
            assert_eq!(identity.as_ref(), key);
        }
        other => panic!("expected keyed domain conflict: {other:?}"),
    }
}

fn expect_dependency_conflict(result: Result<(), CheckedPackageEvidenceInsertionError>, key: &str) {
    match result {
        Err(CheckedPackageEvidenceInsertionError::DependencyPackageConflict { identity }) => {
            assert_eq!(identity.as_ref(), key);
        }
        other => panic!("expected keyed dependency conflict: {other:?}"),
    }
}

fn domain_parent(document: &Value) -> Value {
    let mut parent = v2_nominal();
    select_model(
        &mut parent,
        document["package"]["identity"].as_str().expect("identity"),
    );
    refresh_identity(&mut parent);
    parent
}

fn expect_missing_selection(result: CheckedPackageV2ReadResult, path: &str) {
    let CheckedPackageV2ReadResult::Refused(refusal) = result else {
        panic!("expected missing dependency selection: {result:?}");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MissingImport);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::MissingSelection)
    );
    assert_eq!(refusal.loci[0].path.clone(), Some(pointer(path)));
}

fn expect_refused(
    result: CheckedPackageV2ReadResult,
    code: CheckedPackageRefusalCode,
    cause: CheckedPackageRefusalCause,
) {
    let CheckedPackageV2ReadResult::Refused(refusal) = result else {
        panic!("expected evidence refusal: {result:?}");
    };
    assert_eq!(refusal.code, code);
    assert_eq!(refusal.cause, Some(cause));
}

fn named_fixture(mut package: Value, name: &str) -> Value {
    package["name"] = json!(name);
    refresh_identity(&mut package);
    for node in package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("fixture graph")
    {
        let mut dependencies = node["dependencies"]
            .as_array()
            .expect("fixture dependencies")
            .iter()
            .map(|id| serde_json::from_value::<CheckedNodeId>(id.clone()).expect("structural id"))
            .collect::<Vec<_>>();
        dependencies.sort();
        node["dependencies"] = serde_json::to_value(dependencies).expect("dependencies");
    }
    package
}

#[trace("FR-047-AC-1", "FR-047-AC-2", "FR-047-AC-6")]
#[test]
fn domain_repeats_preserve_first_only_public_read_in_both_orders() {
    let document = domain_package_document("test/evidence", "1", Vec::new());
    let different = domain_package_document("test/other", "1", Vec::new());
    let bytes = canonical(&document);
    let key = "test/evidence".to_owned();
    let parent = domain_parent(&document);
    for (adverse, code, cause) in [
        (
            canonical(&different),
            CheckedPackageRefusalCode::InvalidModelBinding,
            CheckedPackageRefusalCause::WrongModelSelection,
        ),
        (
            b"not JSON".to_vec(),
            CheckedPackageRefusalCode::MalformedWire,
            CheckedPackageRefusalCause::MalformedJson,
        ),
    ] {
        for valid_first in [true, false] {
            let (first, second) = if valid_first {
                (&bytes, &adverse)
            } else {
                (&adverse, &bytes)
            };
            let mut evidence = sentinels();
            assert_eq!(
                evidence.insert_domain_package_document(key.clone(), first.clone()),
                Ok(())
            );
            let control = evidence.clone();
            let before = read(&parent, &control);
            if valid_first {
                require_admission(before.clone());
            } else {
                expect_refused(before.clone(), code, cause);
            }
            assert_eq!(
                evidence.insert_domain_package_document(key.clone(), first.clone()),
                Ok(())
            );
            assert_eq!(evidence, control);
            assert_eq!(read(&parent, &evidence), before);
            expect_domain_conflict(
                evidence.insert_domain_package_document(key.clone(), second.clone()),
                &key,
            );
            assert_eq!(evidence, control);
            assert_eq!(read(&parent, &evidence), before);
            let mut expected = control;
            assert_eq!(
                expected.insert_domain_package_document("later independent", b"later".to_vec()),
                Ok(())
            );
            assert_eq!(
                evidence.insert_domain_package_document("later independent", b"later".to_vec()),
                Ok(())
            );
            assert_eq!(evidence, expected);
            assert_eq!(read(&parent, &evidence), before);
        }
    }
}

#[trace("FR-047-AC-1", "FR-047-AC-2", "FR-047-AC-6")]
#[test]
fn domain_equality_is_raw_bytes_and_exact_keys_are_independent() {
    let keys = [
        " Key é/v1 ",
        " key é/v1 ",
        "Key é/v1",
        " Key e\u{301}/v1 ",
        " Key é/v2 ",
    ];
    let canonical_bytes = canonical(&json!({"value": 1}));
    let spaced_bytes = b"{ \"value\" : 1 }".to_vec();
    assert_ne!(canonical_bytes, spaced_bytes);
    assert_eq!(
        serde_json::from_slice::<Value>(&canonical_bytes).unwrap(),
        serde_json::from_slice::<Value>(&spaced_bytes).unwrap()
    );
    for (first, second) in [
        (&canonical_bytes, &spaced_bytes),
        (&spaced_bytes, &canonical_bytes),
    ] {
        let mut evidence = sentinels();
        for key in keys {
            assert_eq!(
                evidence.insert_domain_package_document(key, first.clone()),
                Ok(())
            );
        }
        assert_eq!(
            evidence.insert_dependency_package(" Key é/v1 ", admit(&v2_nominal())),
            Ok(())
        );
        let control = evidence.clone();
        for key in keys {
            assert_eq!(
                evidence.insert_domain_package_document(key, first.clone()),
                Ok(())
            );
            expect_domain_conflict(
                evidence.insert_domain_package_document(key, second.clone()),
                key,
            );
            assert_eq!(evidence, control);
        }
    }
}

fn dependency_parent(dependency: &CheckedPackageV2, identity: &str) -> Value {
    let mut parent = v2_all_families();
    parent["lock"]["dependency_selections"] = json!([{"identity": identity}]);
    let nodes = parent["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    let position = nodes
        .iter()
        .position(|node| node["node_tag"] == "function" && node["body"]["term"] == "application")
        .expect("function call");
    nodes[position]["body"]["arguments"][0] = json!({
        "term": "dependency_reference", "package": identity,
        "node": structural_id(dependency.name(), vec![json!("Example"), json!("Function")], 0),
    });
    nodes[position]["dependencies"] = json!([]);
    settle(&mut parent);
    parent
}

#[trace("FR-047-AC-3", "FR-047-AC-6")]
#[test]
fn equal_independent_admissions_retain_the_first_arc_and_ceiling_in_both_orders() {
    let document = named_fixture(v2_all_families(), "test/library");
    let bytes = canonical(&document);
    let ceiling = u64::try_from(bytes.len()).expect("fixture length");
    let admit_at = |bytes_limit| {
        let mut limits = CheckedPackageReadLimits::bounded();
        limits.bytes = bytes_limit;
        Arc::new(require_admission(CheckedPackageV2::read(
            &bytes,
            limits,
            &evidence_for(&document),
        )))
    };
    for (first_limit, second_limit) in [(ceiling, ceiling + 4096), (ceiling + 4096, ceiling)] {
        let first = admit_at(first_limit);
        let second = admit_at(second_limit);
        assert_eq!(first, second);
        assert!(!Arc::ptr_eq(&first, &second));
        let mut evidence = evidence_for(&document);
        add_sentinels(&mut evidence);
        assert_eq!(Arc::strong_count(&first), 1);
        assert_eq!(Arc::strong_count(&second), 1);
        assert_eq!(
            evidence.insert_dependency_package("test/library", Arc::clone(&first)),
            Ok(())
        );
        assert_eq!(Arc::strong_count(&first), 2);
        assert_eq!(
            evidence.insert_dependency_package("test/library", Arc::clone(&second)),
            Ok(())
        );
        assert_eq!(Arc::strong_count(&first), 2);
        assert_eq!(Arc::strong_count(&second), 1);
        let parent = dependency_parent(&first, "test/library");
        let before = read(&parent, &evidence);
        require_admission(before.clone());
        let control = evidence.clone();
        assert_eq!(
            evidence.insert_dependency_package("test/library", Arc::clone(&second)),
            Ok(())
        );
        assert_eq!(evidence, control);
        assert_eq!(read(&parent, &evidence), before);
        for identity in ["Test/library", "test/library ", "test/library/v2"] {
            let separate = admit(&named_fixture(document.clone(), identity));
            assert_eq!(
                evidence.insert_dependency_package(identity, Arc::clone(&separate)),
                Ok(())
            );
            let selected = dependency_parent(&separate, identity);
            require_admission(read(&selected, &evidence));
        }
    }
}

#[trace("FR-047-AC-4", "FR-047-AC-6")]
#[test]
fn dependency_conflicts_preserve_first_only_public_read_in_both_orders() {
    let document = named_fixture(v2_all_families(), "test/library");
    let matching = admit(&document);
    let different = admit(&named_fixture(v2_nominal(), "test/other"));
    assert_ne!(matching.name(), different.name());
    let identity = "test/library";
    let parent = dependency_parent(&matching, identity);
    for matching_first in [true, false] {
        let (first, second) = if matching_first {
            (&matching, &different)
        } else {
            (&different, &matching)
        };
        let mut evidence = evidence_for(&document);
        add_sentinels(&mut evidence);
        assert_eq!(
            evidence.insert_dependency_package(identity, Arc::clone(first)),
            Ok(())
        );
        let control = evidence.clone();
        let before = read(&parent, &control);
        if matching_first {
            require_admission(before.clone());
        } else {
            expect_missing_selection(before.clone(), "/lock/dependency_selections/0");
        }
        expect_dependency_conflict(
            evidence.insert_dependency_package(identity, Arc::clone(second)),
            identity,
        );
        assert_eq!(evidence, control);
        assert_eq!(read(&parent, &evidence), before);
        let mut expected = control;
        assert_eq!(
            expected.insert_dependency_package("later independent", Arc::clone(second)),
            Ok(())
        );
        assert_eq!(
            evidence.insert_dependency_package("later independent", Arc::clone(second)),
            Ok(())
        );
        assert_eq!(evidence, expected);
        assert_eq!(read(&parent, &evidence), before);
    }
}

#[trace("FR-047-AC-4", "FR-047-AC-6")]
#[test]
fn equal_package_names_do_not_hide_independently_admitted_metadata_conflicts() {
    let original = v2_nominal();
    let mut source_map = original.clone();
    source_map["source_map"][0]["regions"][0]["end"] = json!(100);
    let mut diagnostics = original.clone();
    diagnostics["diagnostics"]["entries"] = json!([{
        "stage": "package_read", "code": "invalid_package", "cause_tag": "invalid-value",
        "details": [], "loci": [],
    }]);
    let mut capability_base = original.clone();
    capability_base["capability_report"]
        .as_array_mut()
        .unwrap()
        .push(json!({"feature": "optional/v1", "disposition": "available"}));
    let mut capability = capability_base.clone();
    capability["capability_report"][1]["disposition"] = json!("unsupported");
    let mut occurrences = original.clone();
    occurrences["semantic_graph"]["nodes"][0]["occurrences"][0]["ordinal"] = json!(1);
    occurrences["source_map"][0]["ordinal"] = json!(1);
    let mut renamed_source = original.clone();
    let old_source = original["lock"]["sources"][1].clone();
    renamed_source["lock"]["sources"][1]["identity"] =
        json!("quire.fixture.source.secondary-renamed/v1");
    let new_source = renamed_source["lock"]["sources"][1].clone();
    for entry in renamed_source["source_map"].as_array_mut().unwrap() {
        for region in entry["regions"].as_array_mut().unwrap() {
            if region["source"] == old_source {
                region["source"] = new_source.clone();
            }
        }
    }
    for (label, base, changed) in [
        ("source map", &original, &source_map),
        ("diagnostics", &original, &diagnostics),
        ("capability", &capability_base, &capability),
        ("occurrences", &original, &occurrences),
        ("source identity", &original, &renamed_source),
    ] {
        let left = admit(base);
        let right = admit(changed);
        assert_eq!(left.name(), right.name(), "{label}");
        assert_ne!(left, right, "{label}");
        for (first, second) in [(&left, &right), (&right, &left)] {
            let mut evidence = sentinels();
            assert_eq!(
                evidence.insert_dependency_package(" Exact é/v1 ", Arc::clone(first)),
                Ok(())
            );
            let control = evidence.clone();
            expect_dependency_conflict(
                evidence.insert_dependency_package(" Exact é/v1 ", Arc::clone(second)),
                " Exact é/v1 ",
            );
            assert_eq!(evidence, control, "{label}");
        }
    }
}

#[trace("FR-047-AC-5")]
#[test]
fn exact_feature_repeats_are_idempotent_and_variants_do_not_imply_support() {
    for (required, distinct) in [
        ("feature/v1", "Feature/v1"),
        ("feature/v1", " feature/v1"),
        ("feature/v1", "feature/v1 "),
        ("feature/é/v1", "feature/e\u{301}/v1"),
        ("feature/v1", "feature/v2"),
    ] {
        assert_ne!(required, distinct);
        let mut parent = v2_nominal();
        parent["lock"]["required_features"] = json!([required]);
        parent["capability_report"] = json!([{"feature": required, "disposition": "available"}]);
        refresh_identity(&mut parent);
        let mut exact = sentinels();
        exact.support_feature(required);
        let control = exact.clone();
        let admitted = read(&parent, &exact);
        require_admission(admitted.clone());
        exact.support_feature(required);
        assert_eq!(exact, control);
        assert_eq!(read(&parent, &exact), admitted);
        let mut only_distinct = sentinels();
        only_distinct.support_feature(distinct);
        let CheckedPackageV2ReadResult::Refused(refusal) = read(&parent, &only_distinct) else {
            panic!("distinct feature must not satisfy required feature");
        };
        assert_eq!(
            refusal.code,
            CheckedPackageRefusalCode::UnknownRequiredCapability
        );
        assert_eq!(
            refusal.loci[0].path.clone(),
            Some(pointer("/lock/required_features/0"))
        );
        let mut both = control;
        both.support_feature(distinct);
        only_distinct.support_feature(required);
        assert_eq!(only_distinct, both);
        for feature in [required, distinct] {
            only_distinct.support_feature(feature);
            assert_eq!(only_distinct, both);
        }
        assert_eq!(read(&parent, &only_distinct), admitted);
        assert_eq!(read(&parent, &both), admitted);
    }
}
