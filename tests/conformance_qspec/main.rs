// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-038-AC-107: QSpec's positive CheckedPackage V2 fixtures admit end to end.
//!
//! The fixtures are QSpec's and are read from a `quire-specification` checkout
//! named by `QUIRE_SPECIFICATION_DIR`; nothing of them is copied into this
//! repository. This target is outside the run of `make test` (its manifest
//! entry has `test = false`, and a plain `make test` has no checkout) and is
//! run by `make conformance-qspec` alone. It fails, and never skips, when the
//! variable is unset or empty, when it names a path that does not hold
//! `proposals/checked-package-v2/fixtures/`, when a fixture cannot be read and
//! when a fixture does not admit with its recorded `package_id`.

use ix_trace_rs::trace;
use quire_contract_ir::{
    CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageV2, CheckedPackageV2ReadResult,
};
use serde_json::Value;
use std::path::PathBuf;

/// The environment variable naming the root of a `quire-specification` checkout.
const SPECIFICATION_DIR: &str = "QUIRE_SPECIFICATION_DIR";

/// Where QSpec publishes the CheckedPackage V2 fixtures, from its root.
const FIXTURES: &str = "proposals/checked-package-v2/fixtures";

/// The three positive fixtures FR-038-AC-107 reads.
const POSITIVE: [&str; 3] = [
    "positive-all-families.json",
    "positive-clause-operations.json",
    "positive-union-nodes.json",
];

/// The fixtures directory of the named checkout; a test failure naming the
/// variable when it is unset, empty or names no fixtures directory.
fn fixtures_dir() -> PathBuf {
    let root = std::env::var(SPECIFICATION_DIR).unwrap_or_default();
    assert!(
        !root.is_empty(),
        "{SPECIFICATION_DIR} is unset or empty: set it to the root of a quire-specification checkout"
    );
    let dir = PathBuf::from(&root).join(FIXTURES);
    assert!(
        dir.is_dir(),
        "{SPECIFICATION_DIR}={root} holds no {FIXTURES}/: it is not a quire-specification checkout"
    );
    dir
}

/// Tracing: TC-048, FR-038-AC-107
#[trace("TC-048", "FR-038-AC-107")]
#[test]
fn tc_048_qspec_positive_fixtures_admit() {
    let dir = fixtures_dir();
    for name in POSITIVE {
        let path = dir.join(name);
        let text = std::fs::read(&path)
            .unwrap_or_else(|error| panic!("{name}: {} cannot be read: {error}", path.display()));
        let package: Value = serde_json::from_slice(&text)
            .unwrap_or_else(|error| panic!("{name}: not JSON: {error}"));
        // The reader takes canonical bytes; the published files are indented.
        let bytes = serde_json::to_vec(&package).expect("a JSON value serializes");
        let mut evidence = CheckedPackageEvidence::new();
        for feature in package["lock"]["required_features"]
            .as_array()
            .unwrap_or_else(|| panic!("{name}: lock.required_features is not an array"))
        {
            evidence.support_feature(
                feature
                    .as_str()
                    .unwrap_or_else(|| panic!("{name}: a required feature is not a string")),
            );
        }
        match CheckedPackageV2::read(&bytes, CheckedPackageReadLimits::bounded(), &evidence) {
            CheckedPackageV2ReadResult::Admitted(admitted) => {
                let derived =
                    serde_json::to_value(admitted.package_id()).expect("a package id serializes");
                assert_eq!(
                    derived, package["package_id"],
                    "{name}: recorded package_id"
                );
            }
            other => panic!("{name} does not admit: {other:?}"),
        }
    }
}
