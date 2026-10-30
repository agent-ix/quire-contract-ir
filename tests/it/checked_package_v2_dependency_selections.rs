// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Structural V2 dependency selections: exact names, closed entries, ordering,
//! and complete-value conflicts in the admitted dependency closure.

use crate::support::checked_package::{
    admitted_dependency, canonical, evidence_for, pointer, read_with_dependencies, v2_all_families,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedPackageReadLimits, CheckedPackageRefusal, CheckedPackageRefusalCause as Cause,
    CheckedPackageRefusalCode as Code, CheckedPackageV2, CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};

fn entry(identity: &str) -> Value {
    json!({"identity": identity})
}

fn empty_package(name: &str) -> Value {
    fn rename_package(value: &mut Value, name: &str) {
        match value {
            Value::Array(items) => items.iter_mut().for_each(|item| rename_package(item, name)),
            Value::Object(members) => {
                if let Some(package) = members.get_mut("package") {
                    if package == "test/families" {
                        *package = json!(name);
                    }
                }
                members
                    .values_mut()
                    .for_each(|item| rename_package(item, name));
            }
            _ => {}
        }
    }
    let mut package = v2_all_families();
    let scalar = package["semantic_graph"]["nodes"][0].clone();
    let scalar_id = scalar["node_id"].clone();
    package["semantic_graph"]["nodes"] = json!([scalar]);
    package["source_map"] = json!(package["source_map"]
        .as_array()
        .expect("source map")
        .iter()
        .filter(|entry| entry["node_id"] == scalar_id)
        .cloned()
        .collect::<Vec<_>>());
    rename_package(&mut package, name);
    package["name"] = json!(name);
    package
}

fn with_selections(mut package: Value, entries: Vec<Value>) -> Value {
    package["lock"]["dependency_selections"] = Value::Array(entries);
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
        other => panic!("expected refusal, got {other:?}"),
    }
}

fn expect_refusal(package: &Value, code: Code, path: &str, cause: Option<Cause>) {
    let refusal = refused(package);
    assert_eq!(refusal.code, code, "{refusal:?}");
    assert_eq!(refusal.cause, cause, "{refusal:?}");
    assert!(
        refusal
            .loci
            .iter()
            .any(|locus| locus.path == Some(pointer(path))),
        "{refusal:?}"
    );
}

/// FR-038-AC-31/218: two selected packages bind by their admitted names, and
/// the exact structural entries survive reading without an identity preimage.
#[trace("TC-048", "FR-038-AC-31", "FR-038-AC-218")]
#[test]
fn ascending_dependency_names_admit_and_are_retained() {
    let geometry_wire = empty_package("test/geometry");
    let units_wire = empty_package("test/units");
    let (_, geometry) = admitted_dependency(&geometry_wire);
    let (_, units) = admitted_dependency(&units_wire);
    let entries = vec![entry("test/geometry"), entry("test/units")];
    let importer = with_selections(empty_package("test/importer"), entries.clone());
    let CheckedPackageV2ReadResult::Admitted(admitted) = read_with_dependencies(
        &importer,
        &[("test/geometry", &geometry), ("test/units", &units)],
    ) else {
        panic!("two selected packages with matching names must admit");
    };
    assert_eq!(
        serde_json::to_value(&admitted.lock().dependency_selections).expect("selections"),
        json!(entries)
    );
    assert_eq!(admitted.name(), "test/importer");
    assert!(importer.get("package_id").is_none());
    assert!(importer.get("identity_preimage").is_none());
}

/// FR-038-AC-32/63: `{identity}` is the complete selection entry. An absent
/// or empty identity is malformed, and every removed legacy member is refused.
#[trace("TC-048", "FR-038-AC-32", "FR-038-AC-63")]
#[test]
fn selection_shape_is_closed() {
    let base = empty_package("test/importer");
    for (shape, path) in [
        (json!({}), "/lock/dependency_selections/0"),
        (
            json!({"identity": ""}),
            "/lock/dependency_selections/0/identity",
        ),
    ] {
        expect_refusal(
            &with_selections(base.clone(), vec![shape]),
            Code::MalformedWire,
            path,
            None,
        );
    }
    for member in ["package_id", "version", "digest", "role", "definition"] {
        let mut selection = entry("test/geometry");
        selection[member] = json!("removed");
        expect_refusal(
            &with_selections(base.clone(), vec![selection]),
            Code::UnknownMember,
            &format!("/lock/dependency_selections/0/{member}"),
            None,
        );
    }
}

/// FR-038-AC-33: duplicate detection precedes monotonicity for an identity
/// repeated at any position; distinct entries use UTF-8 byte order.
#[trace("TC-048", "FR-038-AC-33")]
#[test]
fn duplicate_and_misordered_names_refuse_at_the_repeating_entry() {
    let base = empty_package("test/importer");
    for (entries, at, cause) in [
        (
            vec![entry("test/a"), entry("test/a")],
            1,
            Cause::ConflictingDefinition,
        ),
        (
            vec![entry("test/a"), entry("test/b"), entry("test/a")],
            2,
            Cause::ConflictingDefinition,
        ),
        (
            vec![entry("test/b"), entry("test/a")],
            1,
            Cause::InvalidValue,
        ),
    ] {
        expect_refusal(
            &with_selections(base.clone(), entries),
            Code::InvalidPackage,
            &format!("/lock/dependency_selections/{at}"),
            Some(cause),
        );
    }
    let first_wire = empty_package("test/\u{ff61}");
    let second_wire = empty_package("test/\u{1f600}");
    let (_, first) = admitted_dependency(&first_wire);
    let (_, second) = admitted_dependency(&second_wire);
    let ordered = with_selections(base, vec![entry("test/\u{ff61}"), entry("test/\u{1f600}")]);
    assert!(matches!(
        read_with_dependencies(
            &ordered,
            &[("test/\u{ff61}", &first), ("test/\u{1f600}", &second)]
        ),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

/// FR-038-AC-64/217/218: selection binding uses package name; a selected
/// name without an identically named admitted package is a typed missing import.
#[trace("TC-048", "FR-038-AC-64", "FR-038-AC-217", "FR-038-AC-218")]
#[test]
fn selection_binds_by_exact_package_name() {
    let wire = empty_package("test/geometry");
    let (_, geometry) = admitted_dependency(&wire);
    let importer = with_selections(empty_package("test/importer"), vec![entry("test/geometry")]);
    assert!(matches!(
        read_with_dependencies(&importer, &[("test/geometry", &geometry)]),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    for supplied in [
        Vec::<(&str, &CheckedPackageV2)>::new(),
        vec![("test/other", &geometry)],
    ] {
        let CheckedPackageV2ReadResult::Refused(refusal) =
            read_with_dependencies(&importer, &supplied)
        else {
            panic!("unsupplied selection must refuse");
        };
        assert_eq!(refusal.code, Code::MissingImport);
        assert_eq!(refusal.cause, Some(Cause::MissingSelection));
        assert_eq!(
            refusal.loci[0].path,
            Some(pointer("/lock/dependency_selections/0"))
        );
    }
}

/// FR-038-AC-293: the same name reached through a diamond must bind one
/// complete package value; the two importer-rooted loci remain ordered.
#[trace("TC-048", "FR-038-AC-293")]
#[test]
fn diamond_with_unequal_complete_leaf_packages_refuses() {
    let first_wire = empty_package("test/families");
    let mut second_wire = first_wire.clone();
    second_wire["source_map"][0]["regions"][0]["end"] = json!(100);
    let (_, first_leaf) = admitted_dependency(&first_wire);
    let (_, second_leaf) = admitted_dependency(&second_wire);
    assert_eq!(first_leaf.name(), second_leaf.name());
    assert_ne!(canonical(&first_wire), canonical(&second_wire));

    let left_wire = with_selections(empty_package("test/left"), vec![entry(first_leaf.name())]);
    let right_wire = with_selections(empty_package("test/right"), vec![entry(second_leaf.name())]);
    let CheckedPackageV2ReadResult::Admitted(left) =
        read_with_dependencies(&left_wire, &[(first_leaf.name(), &first_leaf)])
    else {
        panic!("left dependency admits");
    };
    let CheckedPackageV2ReadResult::Admitted(right) =
        read_with_dependencies(&right_wire, &[(second_leaf.name(), &second_leaf)])
    else {
        panic!("right dependency admits");
    };
    let root = with_selections(
        empty_package("test/root"),
        vec![entry(left.name()), entry(right.name())],
    );
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_with_dependencies(&root, &[(left.name(), &left), (right.name(), &right)])
    else {
        panic!("unequal complete values for one transitive name must refuse");
    };
    assert_eq!(refusal.code, Code::InvalidPackage);
    assert_eq!(refusal.cause, Some(Cause::ConflictingDefinition));
    assert_eq!(
        refusal.loci[0].path,
        Some(pointer(
            "/lock/dependency_selections/0/lock/dependency_selections/0"
        ))
    );
    assert_eq!(
        refusal.loci[1].path,
        Some(pointer(
            "/lock/dependency_selections/1/lock/dependency_selections/0"
        ))
    );
}

/// AC-366 equal-value diamond and direct-lock-only projection slice. Both
/// branches supply the same admitted leaf value under one selected name.
#[trace("TC-048")]
#[test]
fn equal_complete_leaf_diamond_admits_without_a_transitive_lock_row() {
    let leaf_wire = empty_package("test/leaf");
    let (_, leaf) = admitted_dependency(&leaf_wire);
    let left_wire = with_selections(empty_package("test/left"), vec![entry(leaf.name())]);
    let right_wire = with_selections(empty_package("test/right"), vec![entry(leaf.name())]);
    let CheckedPackageV2ReadResult::Admitted(left) =
        read_with_dependencies(&left_wire, &[(leaf.name(), &leaf)])
    else {
        panic!("left branch must admit");
    };
    let CheckedPackageV2ReadResult::Admitted(right) =
        read_with_dependencies(&right_wire, &[(leaf.name(), &leaf)])
    else {
        panic!("right branch must admit");
    };
    let direct = vec![entry(left.name()), entry(right.name())];
    let root = with_selections(empty_package("test/root"), direct.clone());
    let CheckedPackageV2ReadResult::Admitted(admitted) =
        read_with_dependencies(&root, &[(left.name(), &left), (right.name(), &right)])
    else {
        panic!("equal complete leaf values in a diamond must admit");
    };
    assert_eq!(
        serde_json::to_value(&admitted.lock().dependency_selections).expect("direct selections"),
        json!(direct),
        "the root lock retains only its two direct selections"
    );
}

/// An explicit direct claim for a name already reached transitively is valid
/// when it supplies the identical complete admitted package value.
#[trace("TC-048", "FR-038-AC-31", "FR-038-AC-217")]
#[test]
fn overlisted_transitive_selection_admits_when_values_agree() {
    let leaf_wire = empty_package("test/leaf");
    let (_, leaf) = admitted_dependency(&leaf_wire);
    let direct_wire = with_selections(empty_package("test/direct"), vec![entry(leaf.name())]);
    let CheckedPackageV2ReadResult::Admitted(direct) =
        read_with_dependencies(&direct_wire, &[(leaf.name(), &leaf)])
    else {
        panic!("direct dependency admits");
    };
    let root = with_selections(
        empty_package("test/root"),
        vec![entry(direct.name()), entry(leaf.name())],
    );
    assert!(matches!(
        read_with_dependencies(&root, &[(direct.name(), &direct), (leaf.name(), &leaf)]),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

/// One direct selection also detects a collision with a differently valued
/// package reached below it, using both importer-rooted paths.
#[trace("TC-048", "FR-038-AC-293")]
#[test]
fn one_direct_edge_refuses_conflicting_transitive_name() {
    let leaf_wire = empty_package("test/repeated");
    let (_, leaf) = admitted_dependency(&leaf_wire);
    let middle_wire = with_selections(empty_package("test/middle"), vec![entry(leaf.name())]);
    let CheckedPackageV2ReadResult::Admitted(middle) =
        read_with_dependencies(&middle_wire, &[(leaf.name(), &leaf)])
    else {
        panic!("middle dependency admits");
    };
    let child_wire = with_selections(empty_package("test/repeated"), vec![entry(middle.name())]);
    let CheckedPackageV2ReadResult::Admitted(child) =
        read_with_dependencies(&child_wire, &[(middle.name(), &middle)])
    else {
        panic!("child admits before its importer's selection is checked");
    };
    let root = with_selections(empty_package("test/root"), vec![entry(child.name())]);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_with_dependencies(&root, &[(child.name(), &child)])
    else {
        panic!("unequal complete values under the same name must refuse");
    };
    assert_eq!(refusal.code, Code::InvalidPackage);
    assert_eq!(refusal.cause, Some(Cause::ConflictingDefinition));
    assert_eq!(
        refusal.loci[0].path,
        Some(pointer("/lock/dependency_selections/0"))
    );
    assert_eq!(
        refusal.loci[1].path,
        Some(pointer(
            "/lock/dependency_selections/0/lock/dependency_selections/0/lock/dependency_selections/0"
        ))
    );
}
