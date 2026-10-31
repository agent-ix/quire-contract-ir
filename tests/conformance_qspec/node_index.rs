// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! QSpec NI-01..NI-12 indexed node-path vectors through the production V2 reader.

use super::{
    code_word, fixtures_dir, proposal_dir_of, read_json, read_package,
    read_package_with_document_and_extensions_at_limit, DOMAIN_DOCUMENT,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedNodePathSegment, CheckedPackageReadLimits, CheckedPackageV2ReadResult,
};
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

/// Trace: FR-038-AC-376
#[trace("TC-048", "FR-038-AC-376")]
#[test]
fn permutation_rebuilds_the_same_structural_keys_and_stale_operands_do_not() {
    let fixtures = fixtures_dir();
    let original_wire = read_json(&fixtures.join("positive-dependency-call.json"))
        .expect("published importing package");
    let CheckedPackageV2ReadResult::Admitted(original) =
        read_package(&original_wire).expect("published importer read")
    else {
        panic!("published importer must admit");
    };
    let count = original.graph().nodes.len();
    assert_eq!(count, 4, "published graph cardinality");
    let mut permuted_wire = original_wire.clone();
    permuted_wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("published graph nodes")
        .reverse();
    assert!(reverse_graph_indexes(&mut permuted_wire, count) > 0);
    let CheckedPackageV2ReadResult::Admitted(permuted) =
        read_package(&permuted_wire).expect("permuted importer read")
    else {
        panic!("graph permutation with rewritten indexes must admit");
    };
    for original_position in 0..count {
        let permuted_position = count - 1 - original_position;
        assert_eq!(
            original.structural_key_at(original_position),
            permuted.structural_key_at(permuted_position),
            "node {original_position}: permutation preserves resolved key"
        );
        assert_eq!(
            original.graph().nodes[original_position].node_tag,
            permuted.graph().nodes[permuted_position].node_tag,
            "node {original_position}: permutation preserves referenced kind"
        );
    }
    // Keep all rewritten references, but retarget the imported range's own
    // path operand from the Integer node (3) to the Boolean node (2).
    let mut stale = permuted_wire;
    stale["semantic_graph"]["nodes"][1]["node_id"]["path"][3] = json!({"node":2});
    match read_package(&stale).expect("stale-index package read") {
        CheckedPackageV2ReadResult::Admitted(changed) => assert_ne!(
            changed.structural_key_at(1),
            original.structural_key_at(2),
            "stale index must change the resolved operand key"
        ),
        CheckedPackageV2ReadResult::Refused(refusal) => assert!(
            matches!(
                code_word(refusal.code),
                "invalid_source_map" | "invalid_semantic_graph"
            ),
            "stale valid index must fail a structural join: {refusal:?}"
        ),
        CheckedPackageV2ReadResult::Incomplete(incomplete) => {
            panic!("stale index is not a resource limit: {incomplete:?}")
        }
    }
}

/// Trace: FR-038-AC-376
#[trace("TC-048", "FR-038-AC-376")]
#[test]
fn two_semantic_defects_choose_structural_order_over_graph_order() {
    let fixtures = fixtures_dir();
    let mut package = read_json(&fixtures.join("positive-operation-identities.json"))
        .expect("published local operation-identity package");
    let nodes = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("published graph nodes");
    assert_eq!(nodes[16]["semantic_form"], "integer");
    assert_eq!(nodes[17]["semantic_form"], "float64");
    assert_eq!(nodes[16]["node_id"]["package"], "example");
    assert_eq!(nodes[17]["node_id"]["package"], "example");
    assert!(
        nodes[17]["node_id"]["path"][1]
            .as_str()
            .expect("Float64 segment")
            < nodes[16]["node_id"]["path"][1]
                .as_str()
                .expect("Integer segment"),
        "Float64 precedes Integer structurally but follows it in the graph"
    );
    for position in [16, 17] {
        let node = &mut nodes[position];
        node["body"]["members"] = json!([{
            "term":"reference",
            "target":{"package":"example","path":["missing"],"ordinal":0}
        }]);
    }
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package(&package).expect("two-defect package read")
    else {
        panic!("two invalid scalar bodies must refuse");
    };
    assert_eq!(code_word(refusal.code), "invalid_semantic_graph");
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some("/semantic_graph/nodes/17/body".to_owned()),
        "structurally first Float64 defect must win"
    );
}

/// Trace: FR-038-AC-380, FR-038-AC-381
#[trace("TC-048", "FR-038-AC-380", "FR-038-AC-381")]
#[test]
fn long_index_chain_admits_on_a_small_stack_with_shared_keys() {
    const APPLICATIONS: usize = 100_000;
    let fixtures = fixtures_dir();
    let mut package = read_json(&fixtures.join("positive-operation-identities.json"))
        .expect("published operation-identity fixture");
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("published nodes");
    let integer = nodes[16].clone();
    let application_template = nodes[35].clone();
    assert_eq!(integer["semantic_form"], "integer");
    assert_eq!(
        application_template["body"]["operation"]["identity"],
        "quire.op.integer.add"
    );
    let sources = package["source_map"]
        .as_array()
        .expect("published source map");
    let integer_source = sources[16].clone();
    let application_source = sources[35].clone();
    package["lock"]["definition_selections"] = json!([]);
    let mut chain = Vec::with_capacity(APPLICATIONS + 1);
    let mut source_map = Vec::with_capacity(APPLICATIONS + 1);
    let mut previous = integer["node_id"].clone();
    chain.push(integer);
    source_map.push(integer_source);
    for index in 1..=APPLICATIONS {
        let mut node = application_template.clone();
        node["node_id"]["path"] = json!(["expression", "binary", {"node": index - 1}]);
        if index > 1 {
            node["body"]["arguments"][0] = json!({"term":"reference", "target":previous});
            node["dependencies"] = json!([previous]);
        }
        let id = node["node_id"].clone();
        let id_bytes = serde_json::to_vec(&id).expect("fixed-depth id");
        assert!(id_bytes.len() <= 200, "index {index}: node id byte bound");
        let mut entry = application_source.clone();
        entry["node_id"] = id.clone();
        chain.push(node);
        source_map.push(entry);
        previous = id;
    }
    package["semantic_graph"]["nodes"] = Value::Array(chain);
    package["source_map"] = Value::Array(source_map);
    let bytes = serde_json::to_vec(&package).expect("fixed-depth package");
    let limits = CheckedPackageReadLimits {
        bytes: u64::try_from(bytes.len()).expect("package byte count") + 1024,
        nodes: u64::try_from(APPLICATIONS + 1).expect("node count"),
        edges: u64::try_from(APPLICATIONS).expect("edge count"),
        occurrences: u64::try_from((APPLICATIONS + 1) * 2).expect("occurrence count"),
        diagnostics: 1,
        work: 100_000_000,
    };
    let document = proposal_dir_of(&fixtures).join(DOMAIN_DOCUMENT);
    let reading = std::thread::Builder::new()
        .stack_size(256 * 1024)
        .spawn(move || {
            read_package_with_document_and_extensions_at_limit(
                &package,
                "NI-13 deep index chain",
                &document,
                true,
                limits,
            )
        })
        .expect("small-stack reader thread")
        .join()
        .expect("small-stack reader did not panic")
        .expect("reader evidence");
    let CheckedPackageV2ReadResult::Admitted(admitted) = reading else {
        panic!("NI-13 must admit: {reading:?}");
    };
    assert_eq!(admitted.graph().nodes.len(), APPLICATIONS + 1);
    for index in 1..=APPLICATIONS {
        let current = admitted.structural_key_at(index).expect("chain key");
        let prior = admitted
            .structural_key_at(index - 1)
            .expect("predecessor key");
        let Some(CheckedNodePathSegment::Node(last)) = current.path.last() else {
            panic!("index {index}: last key segment is not resolved");
        };
        assert!(
            std::ptr::eq(last.as_ref(), prior),
            "index {index}: key is shared"
        );
    }
}
