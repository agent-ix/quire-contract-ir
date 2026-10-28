// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! QSpec's nominal rekey rows through the public V2 reader.

use super::{fixtures_dir, model_tuple_union::apply_wire_patch, proposal_dir_of, read_json};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageRefusalCode, CheckedPackageV2,
    CheckedPackageV2ReadResult,
};
use serde_json::json;
use std::collections::BTreeSet;

/// Trace: FR-038-AC-343
#[trace("TC-048", "FR-038-AC-343")]
#[test]
fn published_nominal_rekey_rows_match_reader_stage_and_locus() {
    let fixtures = fixtures_dir();
    let vectors = read_json(&proposal_dir_of(&fixtures).join("nominal-rekey-vectors.json"))
        .expect("published nominal rekey vectors");
    assert_eq!(vectors["base"], "fixtures/positive-compound-unit.json");
    let base = read_json(&fixtures.join("positive-compound-unit.json"))
        .expect("published compound unit package");
    let rows = vectors["wire_cases"]
        .as_array()
        .expect("published rekey rows");
    let mut seen = BTreeSet::new();
    let mut evidence = CheckedPackageEvidence::new();
    evidence.support_feature("quire.value.complete/v1");
    for row in rows {
        let id = row["id"].as_str().expect("published rekey row id");
        assert!(
            seen.insert(id.to_owned()),
            "duplicate published rekey row {id}"
        );
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
                assert_eq!(
                    refusal.cause.map(super::cause_word),
                    row["cause"].as_str(),
                    "{id}"
                );
                assert_eq!(
                    refusal.path.as_ref().map(ToString::to_string).as_deref(),
                    row["locus"].as_str(),
                    "{id}"
                );
            }
            (expected, actual) => panic!("{id}: expected {expected}, got {actual:?}"),
        }
    }
    assert_eq!(
        seen,
        (0..=14).map(|index| format!("NK-{index:02}")).collect()
    );
}

/// Trace: FR-038-AC-343
#[trace("TC-048", "FR-038-AC-343")]
#[test]
fn independent_dangling_unit_reference_precedes_nominal_rekey() {
    let fixtures = fixtures_dir();
    let vectors = read_json(&proposal_dir_of(&fixtures).join("nominal-rekey-vectors.json"))
        .expect("published nominal rekey vectors");
    let base = read_json(&fixtures.join("positive-compound-unit.json"))
        .expect("published compound unit package");
    let mut evidence = CheckedPackageEvidence::new();
    evidence.support_feature("quire.value.complete/v1");
    for id in ["NK-01", "NK-05"] {
        let row = vectors["wire_cases"]
            .as_array()
            .expect("published rekey rows")
            .iter()
            .find(|row| row["id"] == id)
            .expect("published dependent reference row");
        let mut package = base.clone();
        for patch in row["patch"].as_array().expect("published patch list") {
            apply_wire_patch(&mut package, patch);
        }
        let missing = json!({
            "domain": "quire.checked-semantic-node/v1",
            "digest": "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
        });
        package["semantic_graph"]["nodes"][3]["body"]["members"][0]["members"][0]["value"]
            ["target"] = missing.clone();
        package["semantic_graph"]["nodes"][3]["dependencies"] = json!([missing]);
        super::fixture_support::refresh_identity(&mut package);
        let result = CheckedPackageV2::read(
            &super::fixture_support::canonical(&package),
            CheckedPackageReadLimits::bounded(),
            &evidence,
        );
        let CheckedPackageV2ReadResult::Refused(refusal) = result else {
            panic!("{id}: independent dangling target must refuse: {result:?}");
        };
        assert_eq!(
            refusal.code,
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            "{id}"
        );
        assert_eq!(refusal.cause, None, "{id}");
        assert_eq!(
            refusal.path.as_ref().map(ToString::to_string).as_deref(),
            Some("/semantic_graph/nodes/3"),
            "{id}"
        );
    }
}
