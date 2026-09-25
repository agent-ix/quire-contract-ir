// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! IR-287: `dependency_selections` entries are `DependencySelection`
//! `{identity, version, package_id}` (QSpec STD-105, FR-322-AC-35), one per
//! library identity in strictly ascending UTF-8 byte order of `identity`,
//! identical in the lock and the identity preimage.
//!
//! The hand-built tests below run everywhere. `dependency_selection_vectors`
//! reads QSpec's `dependency-selection-vectors.json` and its base package at
//! run time from the checkout `QSPEC_DIR` names; nothing of QSpec is copied
//! into this repository. It skips (and passes) when `QSPEC_DIR` is unset;
//! `make qspec-vectors` requires it.

use crate::support::checked_package::{
    admitted_dependency, canonical, evidence_for, pointer, positive_operation_identities,
    read_with_dependencies as read_with, refresh_identity, sha256_hex, v2_all_families,
};
use ix_trace_rs::trace;
use quire_contract_ir::{
    CheckedPackageReadLimits, CheckedPackageRefusal, CheckedPackageRefusalCause,
    CheckedPackageRefusalCode, CheckedPackageV2, CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};

const DIGEST_A: &str = "3333333333333333333333333333333333333333333333333333333333333333";
const DIGEST_B: &str = "4444444444444444444444444444444444444444444444444444444444444444";

fn entry(identity: &str, version: &str, digest: &str) -> Value {
    json!({
        "identity": identity,
        "version": version,
        "package_id": {
            "domain": "quire.package.semantic/v2", "algorithm": "sha256", "digest": digest,
        },
    })
}

/// `package` with `entries` as its lock's and identity preimage's
/// `dependency_selections`, identity re-derived.
fn with_selections(mut package: Value, entries: Vec<Value>) -> Value {
    package["lock"]["dependency_selections"] = Value::Array(entries);
    refresh_identity(&mut package);
    package
}

fn read(package: &Value) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(package),
    )
}

fn refused(package: &Value) -> CheckedPackageRefusal {
    match read(package) {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        other => panic!("expected a refusal, read {other:?}"),
    }
}

fn expect_refusal(
    package: &Value,
    code: CheckedPackageRefusalCode,
    path: &str,
    cause: Option<CheckedPackageRefusalCause>,
) {
    let refusal = refused(package);
    assert_eq!(refusal.code, code, "{refusal:?}");
    assert_eq!(refusal.path, Some(pointer(path)), "{refusal:?}");
    assert_eq!(refusal.cause, cause, "{refusal:?}");
}

/// Tracing: TC-048
/// ACs: FR-038-AC-31
#[trace("TC-048", "FR-038-AC-31")]
#[test]
fn tc_048_ascending_dependency_selections_admit_and_enter_the_package_id() {
    let base = v2_all_families();
    // Two distinct admitted dependency packages, supplied under their
    // entries' identities and versions.
    let (geometry_id, geometry) = admitted_dependency(&base);
    let (units_id, units) = admitted_dependency(&positive_operation_identities());
    let supplied = [
        ("test/geometry", "1", &geometry),
        ("test/units", "2", &units),
    ];
    let entries = vec![
        entry("test/geometry", "1", &geometry_id),
        entry("test/units", "2", &units_id),
    ];
    let package = with_selections(base.clone(), entries.clone());
    let CheckedPackageV2ReadResult::Admitted(admitted) = read_with(&package, &supplied) else {
        panic!("an ascending, one-per-identity selection list admits");
    };
    let lock = serde_json::to_value(admitted.lock()).expect("lock serializes");
    assert_eq!(lock["dependency_selections"], Value::Array(entries.clone()));
    let preimage = serde_json::to_value(admitted.identity_preimage()).expect("preimage");
    assert_eq!(
        preimage["dependency_selections"],
        lock["dependency_selections"]
    );

    // Changing a dependency's package_id changes the package identity.
    let mut changed = entries;
    changed[1] = entry("test/units", "2", &geometry_id);
    let other = with_selections(base, changed);
    let supplied_again = [
        ("test/geometry", "1", &geometry),
        ("test/units", "2", &geometry),
    ];
    assert_ne!(package["package_id"], other["package_id"]);
    // The reader holds the old id to the changed entries, and admits the new.
    let mut stale = other.clone();
    stale["package_id"] = package["package_id"].clone();
    let stale_read = read_with(&stale, &supplied_again);
    let CheckedPackageV2ReadResult::Refused(stale_refusal) = stale_read else {
        panic!("a stale package id refuses, read {stale_read:?}");
    };
    assert_eq!(
        stale_refusal.code,
        CheckedPackageRefusalCode::StaleDependency
    );
    assert_eq!(stale_refusal.path, Some(pointer("/package_id/digest")));
    assert!(
        matches!(
            read_with(&other, &supplied_again),
            CheckedPackageV2ReadResult::Admitted(_)
        ),
        "the changed entries admit under their own package_id"
    );
}

/// Tracing: TC-048
/// ACs: FR-038-AC-32
#[trace("TC-048", "FR-038-AC-32")]
#[test]
fn tc_048_dependency_selection_shape_and_domain_refuse_at_the_entry() {
    let selection = entry("test/a", "1", DIGEST_A);
    let base = v2_all_families();
    let refuse_with = |mutate: &dyn Fn(&mut Value)| {
        let mut mutated = selection.clone();
        mutate(&mut mutated);
        refused(&with_selections(base.clone(), vec![mutated]))
    };
    let wrong_domain = refuse_with(&|e| e["package_id"]["domain"] = json!("quire.source.bytes/v1"));
    assert_eq!(
        wrong_domain.code,
        CheckedPackageRefusalCode::DigestDomainMismatch
    );
    assert_eq!(
        wrong_domain.path,
        Some(pointer("/lock/dependency_selections/0/package_id/domain"))
    );
    let bare = refuse_with(&|e| e["package_id"] = json!(DIGEST_A));
    assert_eq!(bare.code, CheckedPackageRefusalCode::MalformedWire);
    assert_eq!(
        bare.path,
        Some(pointer(
            "/identity_preimage/dependency_selections/0/package_id"
        ))
    );
    let no_version = refuse_with(&|e| {
        e.as_object_mut().expect("object").remove("version");
    });
    assert_eq!(no_version.code, CheckedPackageRefusalCode::MalformedWire);
    assert_eq!(
        no_version.path,
        Some(pointer("/identity_preimage/dependency_selections/0"))
    );
    let empty_identity = refuse_with(&|e| e["identity"] = json!(""));
    assert_eq!(
        empty_identity.code,
        CheckedPackageRefusalCode::MalformedWire
    );
    assert_eq!(
        empty_identity.path,
        Some(pointer("/lock/dependency_selections/0/identity"))
    );
    let short_digest = refuse_with(&|e| e["package_id"]["digest"] = json!("abc"));
    assert_eq!(short_digest.code, CheckedPackageRefusalCode::MalformedWire);
    assert_eq!(
        short_digest.path,
        Some(pointer("/lock/dependency_selections/0/package_id/digest"))
    );
    // An entry with every required member and one more is an unknown member
    // at the extra member.
    let role = refuse_with(&|e| e["role"] = json!("profile"));
    assert_eq!(role.code, CheckedPackageRefusalCode::UnknownMember);
    assert_eq!(
        role.path,
        Some(pointer("/identity_preimage/dependency_selections/0/role"))
    );
}

/// Tracing: TC-048
/// ACs: FR-038-AC-32
#[trace("TC-048", "FR-038-AC-32")]
#[test]
fn tc_048_selection_and_definition_ref_shapes_are_malformed_at_the_entry() {
    let definition = json!({
        "authority": "test", "identity": "test/geometry",
        "revision": {"namespace": "semver", "value": "1"},
        "digest_domain": "quire.definition.bytes/v1", "digest": DIGEST_A,
    });
    for shape in [
        json!({"role": "profile", "definition": definition}),
        definition.clone(),
    ] {
        let package = with_selections(v2_all_families(), vec![shape]);
        expect_refusal(
            &package,
            CheckedPackageRefusalCode::MalformedWire,
            "/identity_preimage/dependency_selections/0",
            None,
        );
    }
}

/// Tracing: TC-048
/// ACs: FR-038-AC-32
#[trace("TC-048", "FR-038-AC-32")]
#[test]
fn tc_048_dependency_is_no_longer_a_selection_role() {
    for member in ["/edition/role", "/profile_selections/0/role"] {
        let mut package = v2_all_families();
        *package["lock"]
            .pointer_mut(member)
            .expect("the fixture selects it") = json!("dependency");
        refresh_identity(&mut package);
        let refusal = refused(&package);
        assert_eq!(refusal.code, CheckedPackageRefusalCode::MalformedWire);
        assert_eq!(
            refusal.path,
            Some(pointer(&format!("/identity_preimage{member}"))),
            "{refusal:?}"
        );
    }
}

/// Tracing: TC-048
/// ACs: FR-038-AC-31
#[trace("TC-048", "FR-038-AC-31")]
#[test]
fn tc_048_lock_and_identity_preimage_dependency_selections_must_match() {
    let valid = vec![
        entry("test/a", "1", DIGEST_A),
        entry("test/b", "1", DIGEST_B),
    ];
    let package = with_selections(v2_all_families(), valid.clone());
    // The preimage differs from a valid lock; its package_id is re-derived so
    // the mismatch, not a stale id, is what the reader meets.
    for (label, preimage_entries, at) in [
        (
            "different digest",
            vec![valid[0].clone(), entry("test/b", "1", DIGEST_A)],
            "/lock/dependency_selections/1/package_id/digest",
        ),
        (
            "duplicate identity",
            vec![valid[0].clone(), entry("test/a", "1", DIGEST_A)],
            "/lock/dependency_selections/1/identity",
        ),
        (
            "different length",
            vec![valid[0].clone()],
            "/lock/dependency_selections",
        ),
    ] {
        let mut mismatched = package.clone();
        mismatched["identity_preimage"]["dependency_selections"] = Value::Array(preimage_entries);
        mismatched["package_id"]["digest"] =
            json!(sha256_hex(&canonical(&mismatched["identity_preimage"])));
        let refusal = refused(&mismatched);
        assert_eq!(
            refusal.code,
            CheckedPackageRefusalCode::StaleDependency,
            "{label}: {refusal:?}"
        );
        assert_eq!(refusal.path, Some(pointer(at)), "{label}: {refusal:?}");
    }
}

/// Tracing: TC-048
/// ACs: FR-038-AC-33
#[trace("TC-048", "FR-038-AC-33")]
#[test]
fn tc_048_repeated_or_misordered_dependency_identity_refuses() {
    let base = v2_all_families();
    let duplicate = with_selections(
        base.clone(),
        vec![
            entry("test/geometry", "1", DIGEST_A),
            entry("test/geometry", "2", DIGEST_B),
            entry("test/units", "2", DIGEST_B),
        ],
    );
    expect_refusal(
        &duplicate,
        CheckedPackageRefusalCode::InvalidPackage,
        "/lock/dependency_selections/1",
        Some(CheckedPackageRefusalCause::ConflictingDefinition),
    );
    // A repeat that is not adjacent is still one identity twice.
    let separated = with_selections(
        base.clone(),
        vec![
            entry("test/a", "1", DIGEST_A),
            entry("test/b", "1", DIGEST_A),
            entry("test/a", "2", DIGEST_B),
        ],
    );
    expect_refusal(
        &separated,
        CheckedPackageRefusalCode::InvalidPackage,
        "/lock/dependency_selections/2",
        Some(CheckedPackageRefusalCause::ConflictingDefinition),
    );
    let misordered = with_selections(
        base.clone(),
        vec![
            entry("test/units", "2", DIGEST_B),
            entry("test/geometry", "1", DIGEST_A),
        ],
    );
    expect_refusal(
        &misordered,
        CheckedPackageRefusalCode::InvalidPackage,
        "/lock/dependency_selections/1",
        Some(CheckedPackageRefusalCause::InvalidValue),
    );
    // UTF-8 byte order, not UTF-16 code-unit order: U+FF61 sorts before
    // U+1F600 in UTF-8 bytes and after it in UTF-16.
    let (dependency_id, dependency) = admitted_dependency(&base);
    let byte_order = with_selections(
        base,
        vec![
            entry("test/\u{ff61}", "1", &dependency_id),
            entry("test/\u{1f600}", "1", &dependency_id),
        ],
    );
    assert!(
        matches!(
            read_with(
                &byte_order,
                &[
                    ("test/\u{ff61}", "1", &dependency),
                    ("test/\u{1f600}", "1", &dependency),
                ]
            ),
            CheckedPackageV2ReadResult::Admitted(_)
        ),
        "entries in UTF-8 byte order admit"
    );
}

const VECTORS: &str = "proposals/checked-package-v2/dependency-selection-vectors.json";

/// A file of the QSpec checkout `QSPEC_DIR` names, or `None` when unset.
fn qspec(relative: &str) -> Option<Value> {
    let root = std::env::var_os("QSPEC_DIR")?;
    let path = std::path::Path::new(&root).join(relative);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    Some(serde_json::from_str(&text).expect("QSpec JSON"))
}

fn text<'v>(value: &'v Value, member: &str) -> &'v str {
    value[member]
        .as_str()
        .unwrap_or_else(|| panic!("{member} is a string in {value}"))
}

fn code(name: &str) -> CheckedPackageRefusalCode {
    match name {
        "malformed_wire" => CheckedPackageRefusalCode::MalformedWire,
        "digest_domain_mismatch" => CheckedPackageRefusalCode::DigestDomainMismatch,
        "invalid_package" => CheckedPackageRefusalCode::InvalidPackage,
        other => panic!("vector code {other} is not mapped"),
    }
}

fn cause(name: &str) -> CheckedPackageRefusalCause {
    match name {
        "conflicting-definition" => CheckedPackageRefusalCause::ConflictingDefinition,
        "invalid-value" => CheckedPackageRefusalCause::InvalidValue,
        other => panic!("vector cause {other} is not mapped"),
    }
}

/// The expected refusal a vector's `refused:<code>[/<cause>]` outcome names.
fn expected(
    outcome: &str,
) -> (
    CheckedPackageRefusalCode,
    Option<CheckedPackageRefusalCause>,
) {
    let named = outcome
        .strip_prefix("refused:")
        .unwrap_or_else(|| panic!("outcome {outcome}"));
    match named.split_once('/') {
        Some((named, because)) => (code(named), Some(cause(because))),
        None => (code(named), None),
    }
}

/// Tracing: TC-048
/// ACs: FR-038-AC-31, FR-038-AC-32, FR-038-AC-33
#[trace("TC-048", "FR-038-AC-31", "FR-038-AC-32", "FR-038-AC-33")]
#[test]
fn dependency_selection_vectors() {
    let Some(vectors) = qspec(VECTORS) else {
        println!("skipped: QSPEC_DIR not set");
        return;
    };
    let base = qspec(&format!(
        "proposals/checked-package-v2/{}",
        text(&vectors, "base")
    ))
    .expect("QSPEC_DIR is set");
    let entries = vectors["dependency_selections"]
        .as_array()
        .expect("entries")
        .clone();

    // The valid package: the recorded package_id recomputes from the entries
    // and the reader admits it.
    let mut valid = with_selections(base.clone(), entries.clone());
    assert_eq!(
        valid["package_id"]["digest"], vectors["package_id"],
        "the recorded package_id recomputes from the entries"
    );
    valid["package_id"]["digest"] = vectors["package_id"].clone();
    // The entries pass every array check; QSpec's dependencies are digests,
    // not packages, so none is supplied and the reader refuses at the first
    // entry (FR-322: an entry with no supplied package).
    expect_refusal(
        &valid,
        CheckedPackageRefusalCode::MissingImport,
        "/lock/dependency_selections/0",
        Some(CheckedPackageRefusalCause::MissingSelection),
    );

    // Entry mutations: the entry replaces the first entry in both members.
    let mutations = vectors["entry_mutations"].as_array().expect("mutations");
    for mutation in mutations {
        let id = text(mutation, "id");
        let mut mutated = entries.clone();
        mutated[0] = mutation["entry"].clone();
        let package = with_selections(base.clone(), mutated);
        let refusal = refused(&package);
        let (want_code, want_cause) = expected(text(mutation, "outcome"));
        assert_eq!(refusal.code, want_code, "{id}: {refusal:?}");
        assert_eq!(refusal.cause, want_cause, "{id}: {refusal:?}");
        let at = refusal.path.as_ref().expect("a refusal pointer").as_str();
        assert!(
            [
                "/lock/dependency_selections/0",
                "/identity_preimage/dependency_selections/0"
            ]
            .iter()
            .any(|entry| at == *entry || at.starts_with(&format!("{entry}/"))),
            "{id}: refusal at {at}, not at the mutated entry"
        );
    }

    // Order vectors: each decided through the reader, at its last locus.
    let orders = vectors["order_vectors"].as_array().expect("order vectors");
    for order in orders {
        let id = text(order, "id");
        let package = with_selections(
            base.clone(),
            order["dependency_selections"]
                .as_array()
                .expect("entries")
                .clone(),
        );
        let outcome = text(order, "outcome");
        if outcome == "admitted" {
            // Admitted by the order rule: the reader gets past every array
            // check and refuses only for the dependency it was not supplied.
            expect_refusal(
                &package,
                CheckedPackageRefusalCode::MissingImport,
                "/lock/dependency_selections/0",
                Some(CheckedPackageRefusalCause::MissingSelection),
            );
            continue;
        }
        let (want_code, want_cause) = expected(outcome);
        let refusal = refused(&package);
        assert_eq!(refusal.code, want_code, "{id}: {refusal:?}");
        assert_eq!(refusal.cause, want_cause, "{id}: {refusal:?}");
        // The refusal carries one pointer; a vector with two loci (a repeat)
        // is located at the repeating, later entry.
        let loci = order["loci"].as_array().expect("loci");
        let last = loci.last().and_then(Value::as_str).expect("a locus");
        assert_eq!(refusal.path, Some(pointer(last)), "{id}: {refusal:?}");
    }
    println!(
        "conformance: dependency-selection-vectors valid + {} entry mutations + {} order vectors",
        mutations.len(),
        orders.len()
    );
}
