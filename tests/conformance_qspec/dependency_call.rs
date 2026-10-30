// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! QSpec dependency-call vectors through the public checked-package reader.

use super::{cause_word, code_word, fixtures_dir, proposal_dir_of, read_json, read_package};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageV2, CheckedPackageV2ReadResult,
};
use serde_json::Value;
use std::collections::BTreeSet;

fn apply_patch(package: &mut Value, patch: &Value) {
    let path = patch["path"].as_str().expect("published patch path");
    let (parent, member) = path.rsplit_once('/').expect("patch member");
    let target = package.pointer_mut(parent).expect("published patch parent");
    match (patch["op"].as_str().expect("published operation"), target) {
        ("replace", Value::Object(map)) => {
            *map.get_mut(member).expect("replaced member") = patch["value"].clone();
        }
        ("replace", Value::Array(items)) => {
            items[member.parse::<usize>().expect("array index")] = patch["value"].clone();
        }
        ("add", Value::Object(map)) => {
            assert!(map
                .insert(member.to_owned(), patch["value"].clone())
                .is_none());
        }
        (operation, _) => panic!("unsupported published patch {operation}: {path}"),
    }
}

fn without_dependency(package: &Value) -> CheckedPackageV2ReadResult {
    let mut evidence = CheckedPackageEvidence::new();
    for feature in package["lock"]["required_features"]
        .as_array()
        .expect("required features")
    {
        evidence.support_feature(feature.as_str().expect("feature"));
    }
    CheckedPackageV2::read(
        &serde_json::to_vec(package).expect("published package"),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    )
}

#[trace(
    "TC-048",
    "FR-038-AC-35",
    "FR-038-AC-36",
    "FR-038-AC-37",
    "FR-038-AC-38"
)]
#[test]
fn published_dependency_call_vectors_bind_every_reader_outcome_and_locus() {
    let fixtures = fixtures_dir();
    let vectors = read_json(&proposal_dir_of(&fixtures).join("dependency-call-vectors.json"))
        .expect("QSpec dependency call vectors");
    assert_eq!(vectors["base"], "fixtures/positive-dependency-call.json");
    assert_eq!(
        vectors["library"]["package"],
        "fixtures/positive-dependency-library.json"
    );
    let base = read_json(&fixtures.join("positive-dependency-call.json"))
        .expect("published positive dependency call");
    assert!(
        matches!(
            read_package(&base),
            Ok(CheckedPackageV2ReadResult::Admitted(_))
        ),
        "published dependency call must admit"
    );
    let rows = vectors["mutations"]
        .as_array()
        .expect("published mutations");
    assert_eq!(rows.len(), 11, "all QSpec dependency-call rows must run");
    let mut seen = BTreeSet::new();
    for row in rows {
        let id = row["id"].as_str().expect("row id");
        assert!(seen.insert(id), "duplicate row {id}");
        let mut package = base.clone();
        for patch in row["patch"].as_array().expect("patch list") {
            apply_patch(&mut package, patch);
        }
        let outcome = if row["supplied"] == false {
            without_dependency(&package)
        } else {
            read_package(&package).expect("published evidence")
        };
        let CheckedPackageV2ReadResult::Refused(refusal) = outcome else {
            panic!("{id}: expected refusal, got {outcome:?}")
        };
        let actual = match refusal.cause {
            Some(cause) => format!("refused:{}/{}", code_word(refusal.code), cause_word(cause)),
            None => format!("refused:{}", code_word(refusal.code)),
        };
        assert_eq!(actual, row["outcome"], "{id}: code and cause");
        assert_eq!(
            refusal.loci[0].path.as_ref().map(ToString::to_string),
            row["locus"].as_str().map(str::to_owned),
            "{id}: exact published locus"
        );
    }
}
