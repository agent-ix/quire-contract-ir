// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! IR-287: `dependency_selections` entries are `DependencySelection`
//! `{identity, package_id}` (QSpec STD-105, FR-322-AC-35), one per
//! library identity in strictly ascending UTF-8 byte order of `identity`,
//! identical in the lock and the identity preimage.

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

fn entry(identity: &str, digest: &str) -> Value {
    json!({
        "identity": identity,
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
    // entries' identities.
    let (geometry_id, geometry) = admitted_dependency(&base);
    let (units_id, units) = admitted_dependency(&positive_operation_identities());
    let supplied = [("test/geometry", &geometry), ("test/units", &units)];
    let entries = vec![
        entry("test/geometry", &geometry_id),
        entry("test/units", &units_id),
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
    changed[1] = entry("test/units", &geometry_id);
    let other = with_selections(base, changed);
    let supplied_again = [("test/geometry", &geometry), ("test/units", &geometry)];
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
    let selection = entry("test/a", DIGEST_A);
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
    let no_package_id = refuse_with(&|e| {
        e.as_object_mut().expect("object").remove("package_id");
    });
    assert_eq!(no_package_id.code, CheckedPackageRefusalCode::MalformedWire);
    assert_eq!(
        no_package_id.path,
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
    let definition = json!({"authority": "test", "identity": "test/geometry"});
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
    let valid = vec![entry("test/a", DIGEST_A), entry("test/b", DIGEST_B)];
    let package = with_selections(v2_all_families(), valid.clone());
    // The preimage differs from a valid lock; its package_id is re-derived so
    // the mismatch, not a stale id, is what the reader meets.
    for (label, preimage_entries, at) in [
        (
            "different digest",
            vec![valid[0].clone(), entry("test/b", DIGEST_A)],
            "/lock/dependency_selections/1/package_id/digest",
        ),
        (
            "duplicate identity",
            vec![valid[0].clone(), entry("test/a", DIGEST_A)],
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
            entry("test/geometry", DIGEST_A),
            entry("test/geometry", DIGEST_B),
            entry("test/units", DIGEST_B),
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
            entry("test/a", DIGEST_A),
            entry("test/b", DIGEST_A),
            entry("test/a", DIGEST_B),
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
            entry("test/units", DIGEST_B),
            entry("test/geometry", DIGEST_A),
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
            entry("test/\u{ff61}", &dependency_id),
            entry("test/\u{1f600}", &dependency_id),
        ],
    );
    assert!(
        matches!(
            read_with(
                &byte_order,
                &[
                    ("test/\u{ff61}", &dependency),
                    ("test/\u{1f600}", &dependency),
                ]
            ),
            CheckedPackageV2ReadResult::Admitted(_)
        ),
        "entries in UTF-8 byte order admit"
    );
}

/// A `dependency_selections` entry is exactly `{identity, package_id}`: one
/// that also carries `version` refuses as `unknown_member` at that member,
/// wherever it sits and whatever the other entries are, and an old-shape
/// `{identity, version}` entry with no `package_id` is `malformed_wire` at
/// the entry. The reader never reads, drops or compares the `version`.
///
/// Tracing: TC-048, FR-038-AC-63
#[trace("TC-048", "FR-038-AC-63")]
#[test]
fn tc_048_a_dependency_entry_carrying_version_is_an_unknown_member() {
    let base = v2_all_families();
    let (geometry_id, geometry) = admitted_dependency(&base);
    let (units_id, units) = admitted_dependency(&positive_operation_identities());
    let supplied = [("test/geometry", &geometry), ("test/units", &units)];
    let entries = vec![
        entry("test/geometry", &geometry_id),
        entry("test/units", &units_id),
    ];
    let package = with_selections(base, entries.clone());
    assert!(
        matches!(
            read_with(&package, &supplied),
            CheckedPackageV2ReadResult::Admitted(_)
        ),
        "entries of exactly {{identity, package_id}} admit"
    );

    let refuse_versioned = |package: &Value, at: &str| {
        let CheckedPackageV2ReadResult::Refused(refusal) = read_with(package, &supplied) else {
            panic!("an entry carrying `version` refuses ({at})");
        };
        assert_eq!(
            refusal.code,
            CheckedPackageRefusalCode::UnknownMember,
            "{at}"
        );
        assert_eq!(refusal.path, Some(pointer(at)), "{at}");
        assert_eq!(refusal.cause, None, "{at}");
    };
    // In the lock only: the preimage mirror is left as it was.
    let mut lock_only = package.clone();
    lock_only["lock"]["dependency_selections"][0]["version"] = json!("1");
    refuse_versioned(&lock_only, "/lock/dependency_selections/0/version");
    // In the identity preimage only.
    let mut preimage_only = package.clone();
    preimage_only["identity_preimage"]["dependency_selections"][0]["version"] = json!("1");
    refuse_versioned(
        &preimage_only,
        "/identity_preimage/dependency_selections/0/version",
    );
    // In both, beside an otherwise well-formed entry: the first in document
    // order, which is the identity preimage's.
    let mut both = package.clone();
    both["lock"]["dependency_selections"][1]["version"] = json!("2");
    refresh_identity(&mut both);
    refuse_versioned(&both, "/identity_preimage/dependency_selections/1/version");

    // The old shape `{identity, version}` lacks `package_id`: malformed at the
    // entry, not an unknown member at `version`.
    let old_shape = with_selections(
        v2_all_families(),
        vec![json!({"identity": "test/geometry", "version": "1"})],
    );
    expect_refusal(
        &old_shape,
        CheckedPackageRefusalCode::MalformedWire,
        "/identity_preimage/dependency_selections/0",
        None,
    );
}

/// A `dependency_selections` entry binds the package the evidence supplies
/// under its `identity` when that package's `package_id` is the entry's, with
/// no version supplied or compared.
///
/// Tracing: TC-048, FR-038-AC-64
#[trace("TC-048", "FR-038-AC-64")]
#[test]
fn tc_048_a_dependency_entry_binds_by_identity_and_package_id() {
    let base = v2_all_families();
    let (geometry_id, geometry) = admitted_dependency(&base);
    let (_, units) = admitted_dependency(&positive_operation_identities());
    let package = with_selections(base, vec![entry("test/geometry", &geometry_id)]);
    assert!(
        matches!(
            read_with(&package, &[("test/geometry", &geometry)]),
            CheckedPackageV2ReadResult::Admitted(_)
        ),
        "the package supplied under the entry's identity, with its package_id, admits"
    );
    let CheckedPackageV2ReadResult::Refused(missing) =
        read_with(&package, &[("test/other", &geometry)])
    else {
        panic!("a package supplied under another identity refuses");
    };
    assert_eq!(missing.code, CheckedPackageRefusalCode::MissingImport);
    assert_eq!(
        missing.cause,
        Some(CheckedPackageRefusalCause::MissingSelection)
    );
    assert_eq!(missing.path, Some(pointer("/lock/dependency_selections/0")));
    let CheckedPackageV2ReadResult::Refused(stale) =
        read_with(&package, &[("test/geometry", &units)])
    else {
        panic!("a package of another package_id refuses");
    };
    assert_eq!(stale.code, CheckedPackageRefusalCode::StaleDependency);
    assert_eq!(
        stale.cause,
        Some(CheckedPackageRefusalCause::ByteDigestMismatch)
    );
    assert_eq!(
        stale.path,
        Some(pointer("/lock/dependency_selections/0/package_id/digest"))
    );
}
