// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! QSpec NI-01..NI-12 indexed node-path vectors through the production V2 reader.

use super::{code_word, fixtures_dir, proposal_dir_of, read_json, read_package};
use ix_trace_rs::trace;
use quire_contract_model::CheckedPackageV2ReadResult;
use serde_json::{json, Value};
use std::collections::BTreeSet;

fn apply_patch(package: &mut Value, patch: &Value) {
    let path = patch["path"].as_str().expect("published patch path");
    let (parent, member) = path.rsplit_once('/').expect("published patch member");
    let target = package.pointer_mut(parent).expect("published patch parent");
    let operation = patch["op"].as_str().expect("published patch operation");
    match (operation, target) {
        ("replace", Value::Array(items)) => {
            let index: usize = member.parse().expect("published array index");
            *items.get_mut(index).expect("published replacement target") = patch["value"].clone();
        }
        ("add", Value::Array(items)) => {
            let index: usize = member.parse().expect("published array index");
            assert!(index <= items.len(), "{path}: published insertion target");
            items.insert(index, patch["value"].clone());
        }
        _ => panic!("unsupported published patch {operation} at {path}"),
    }
}

/// Trace: FR-038-AC-377, FR-038-AC-378
#[trace("TC-048", "FR-038-AC-377", "FR-038-AC-378")]
#[test]
fn published_index_segment_refusals_precede_semantic_checks() {
    let fixtures = fixtures_dir();
    let vectors = read_json(&proposal_dir_of(&fixtures).join("node-index-vectors.json"))
        .expect("QSpec indexed node-path vectors");
    assert_eq!(vectors["version"], "quire.checked-node-index-vectors/v1");
    assert_eq!(vectors["base"], "fixtures/positive-dependency-call.json");
    let rows = vectors["cases"].as_array().expect("published cases");
    assert_eq!(rows.len(), 12, "NI-01..NI-12 are a closed published set");
    let base = read_json(&fixtures.join("positive-dependency-call.json"))
        .expect("published dependency-call fixture");
    let mut seen = BTreeSet::new();
    for row in rows {
        let id = row["id"].as_str().expect("vector id");
        assert!(seen.insert(id), "duplicate vector {id}");
        let mut package = base.clone();
        for patch in row["patch"].as_array().expect("published patches") {
            apply_patch(&mut package, patch);
        }
        let actual = read_package(&package).expect("production read");
        let CheckedPackageV2ReadResult::Refused(refusal) = actual else {
            panic!("{id}: expected a refusal, got {actual:?}");
        };
        assert_eq!(code_word(refusal.code), row["expected"], "{id}: code");
        assert!(
            refusal.cause.is_none(),
            "{id}: unexpected cause: {:?}",
            refusal.cause
        );
        assert_eq!(refusal.loci.len(), 1, "{id}: one precise locus");
        assert_eq!(
            refusal.loci[0]
                .path
                .as_ref()
                .expect("refusal path")
                .to_string(),
            row["locus"],
            "{id}: locus"
        );
    }
    for index in 1..=12 {
        assert!(
            seen.contains(format!("NI-{index:02}").as_str()),
            "missing NI-{index:02}"
        );
    }
}

fn reverse_graph_indexes(value: &mut Value, node_count: usize) -> usize {
    match value {
        Value::Object(map) if map.len() == 1 && map.contains_key("node") => {
            let old = map["node"].as_u64().expect("published index");
            let old: usize = old.try_into().expect("index fits usize");
            assert!(old < node_count, "published index in range");
            map.insert("node".into(), json!(node_count - 1 - old));
            1
        }
        Value::Object(map) => map
            .values_mut()
            .map(|part| reverse_graph_indexes(part, node_count))
            .sum(),
        Value::Array(items) => items
            .iter_mut()
            .map(|part| reverse_graph_indexes(part, node_count))
            .sum(),
        _ => 0,
    }
}

/// Trace: FR-038-AC-379
#[trace("TC-048", "FR-038-AC-379")]
#[test]
fn imported_index_resolves_in_the_importing_graph() {
    let fixtures = fixtures_dir();
    let mut importer_wire = read_json(&fixtures.join("positive-dependency-call.json"))
        .expect("published importing package");
    let library_wire = read_json(&fixtures.join("positive-dependency-library.json"))
        .expect("published library package");
    let nodes = importer_wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("importing graph nodes");
    assert_eq!(nodes.len(), 4, "NI-14 uses the four-node importing graph");
    nodes.reverse();
    let rewritten = reverse_graph_indexes(&mut importer_wire, 4);
    assert!(rewritten > 0, "NI-14 rewrites every indexed occurrence");
    assert_eq!(
        importer_wire["semantic_graph"]["nodes"][1]["node_id"]["path"][3],
        json!({"node":3}),
        "imported Int[0,9] names the importer's Integer node"
    );
    let CheckedPackageV2ReadResult::Admitted(importer) =
        read_package(&importer_wire).expect("importing package read")
    else {
        panic!("NI-14 importing package must admit");
    };
    let CheckedPackageV2ReadResult::Admitted(library) =
        read_package(&library_wire).expect("library package read")
    else {
        panic!("NI-14 library package must admit");
    };
    let importing_key = importer
        .structural_key_at(1)
        .expect("imported type structural key");
    let library_key = library
        .structural_key_at(3)
        .expect("library type structural key");
    assert_eq!(
        importing_key, library_key,
        "NI-14 resolves against importer graph"
    );
}
