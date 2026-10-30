// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Model-root structural keys against QSpec's authoritative published vector.

use super::{code_word, fixtures_dir, proposal_dir_of, read_json, read_package};
use ix_trace_rs::trace;
use quire_contract_model::CheckedPackageV2ReadResult;
use serde_json::{json, Value};
use std::collections::BTreeSet;

fn apply_vector_patch(package: &mut Value, patch: &Value) {
    let path = patch["path"].as_str().expect("published patch path");
    let (parent, member) = path.rsplit_once('/').expect("patch member");
    let target = package.pointer_mut(parent).expect("published patch parent");
    match (patch["op"].as_str().expect("published operation"), target) {
        ("add", Value::Array(items)) if member == "-" => items.push(patch["value"].clone()),
        ("add", Value::Object(map)) => {
            map.insert(member.to_owned(), patch["value"].clone());
        }
        ("replace", Value::Object(map)) => {
            *map.get_mut(member).expect("replaced member") = patch["value"].clone();
        }
        ("remove", Value::Array(items)) => {
            items.remove(member.parse::<usize>().expect("array index"));
        }
        ("remove", Value::Object(map)) => {
            map.remove(member).expect("removed member");
        }
        (operation, _) => panic!("unsupported published patch {operation}: {path}"),
    }
}

/// Trace: FR-065-AC-3
#[trace("TC-233", "FR-065-AC-3")]
#[test]
fn published_model_root_vectors_reach_the_reader_stage_and_locus() {
    let vectors = read_json(&proposal_dir_of(&fixtures_dir()).join("model-root-vectors.json"))
        .expect("QSpec model-root vectors");
    assert_eq!(vectors["base"], "fixtures/positive-all-families.json");
    let base =
        read_json(&fixtures_dir().join("positive-all-families.json")).expect("published base");
    let mut seen = BTreeSet::new();
    for row in vectors["vectors"].as_array().expect("published vectors") {
        let id = row["id"].as_str().expect("vector id");
        assert!(seen.insert(id), "duplicate vector {id}");
        let mut package = base.clone();
        for patch in row["patch"].as_array().expect("published patch") {
            apply_vector_patch(&mut package, patch);
        }
        let actual = read_package(&package).expect("production read");
        match (row["outcome"].as_str().expect("outcome"), actual) {
            ("admitted", CheckedPackageV2ReadResult::Admitted(_)) => {}
            (expected, CheckedPackageV2ReadResult::Refused(refusal))
                if expected.strip_prefix("refused:") == Some(code_word(refusal.code)) =>
            {
                assert_eq!(
                    refusal.cause.map(super::cause_word),
                    row["cause"].as_str(),
                    "{id}"
                );
                assert_eq!(
                    refusal.loci[0]
                        .path
                        .clone()
                        .expect("refusal locus")
                        .to_string(),
                    row["locus"],
                    "{id}"
                );
            }
            (expected, actual) => panic!("{id}: expected {expected}, got {actual:?}"),
        }
    }
}

/// Trace: FR-065-AC-3
#[trace("TC-233", "FR-065-AC-3")]
#[test]
fn published_selected_model_fixtures_carry_one_ordered_and_sourced_root() {
    for name in [
        "positive-all-families.json",
        "positive-clause-operations.json",
        "positive-dependency-library.json",
        "positive-operation-identities.json",
        "positive-state-transition.json",
        "positive-tick-timed-model-nodes.json",
        "positive-timed-model-nodes.json",
    ] {
        let package = read_json(&fixtures_dir().join(name)).expect("published package");
        let nodes = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes");
        let sources = package["source_map"].as_array().expect("source map");
        let lock_sources = package["lock"]["sources"].as_array().expect("lock sources");
        let roots = nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| node["semantic_form"] == "model_root")
            .collect::<Vec<_>>();
        assert!(!roots.is_empty(), "{name}: a selected root");
        let identities = roots
            .iter()
            .map(|(_, root)| root["owner"]["identity"].as_str().expect("identity"))
            .collect::<Vec<_>>();
        let mut ordered = identities.clone();
        ordered.sort_unstable();
        ordered.dedup();
        assert_eq!(
            identities, ordered,
            "{name}: one root per identity, ordered"
        );
        for (offset, (position, root)) in roots.iter().copied().enumerate() {
            assert_eq!(
                position,
                nodes.len() - roots.len() + offset,
                "{name}: root order"
            );
            assert_eq!(root["owner"]["kind"], "model_root", "{name}: owner kind");
            assert!(
                root["owner"].get("digest").is_none(),
                "{name}: owner carries no digest"
            );
            assert_eq!(
                root["occurrences"],
                json!([{"role":"generated","ordinal":0}]),
                "{name}: occurrence"
            );
            let entries = sources
                .iter()
                .filter(|entry| entry["node_id"] == root["node_id"])
                .collect::<Vec<_>>();
            assert_eq!(entries.len(), 1, "{name}: one root source entry");
            assert_eq!(entries[0]["role"], "generated", "{name}: source role");
            assert_eq!(entries[0]["ordinal"], 0, "{name}: source ordinal");
            assert!(
                !entries[0]["regions"]
                    .as_array()
                    .expect("regions")
                    .is_empty(),
                "{name}: sourced root"
            );
            for region in entries[0]["regions"].as_array().expect("regions") {
                assert!(
                    lock_sources.contains(&region["source"]),
                    "{name}: source is selected"
                );
                assert!(
                    region["start"].as_u64().expect("start") < region["end"].as_u64().expect("end"),
                    "{name}: source region has width"
                );
            }
        }
    }
}
