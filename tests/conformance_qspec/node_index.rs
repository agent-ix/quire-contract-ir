// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! QSpec NI-01..NI-24 operand-row and operand-slot vectors through the production V2 reader.

use super::fixture_support;
use super::{
    cause_word, code_word, fixtures_dir, proposal_dir_of, read_json, read_json_wire, read_package,
    read_package_with_document_and_extensions_at_limit, DOMAIN_DOCUMENT,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedNodeId, CheckedNodeKind, CheckedNodePathSegment, CheckedNodeTag, CheckedOperandEntry,
    CheckedPackageReadLimits, CheckedPackageV2ReadResult, CheckedSemanticNodeV2, CheckedTerm,
    CompleteLoweringProfileV2, CompleteLoweringRecordV2, ContractDependencies,
    ContractPackageReadResult, ContractPackageWireV1, ExpressionForm, LoweringProfileRef,
};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::sync::Arc;

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
        ("add", Value::Array(items)) if member == "-" => items.push(patch["value"].clone()),
        ("remove", Value::Array(items)) => {
            let index: usize = member.parse().expect("published array index");
            items.remove(index);
        }
        ("add", Value::Array(items)) => {
            let index: usize = member.parse().expect("published array index");
            assert!(index <= items.len(), "{path}: published insertion target");
            items.insert(index, patch["value"].clone());
        }
        _ => panic!("unsupported published patch {operation} at {path}"),
    }
}

#[trace(
    "TC-048",
    "FR-038-AC-377",
    "FR-038-AC-378",
    "FR-038-AC-520",
    "FR-038-AC-521"
)]
#[test]
fn published_operand_vectors_refuse_in_the_fr322_check_order() {
    let fixtures = fixtures_dir();
    let vectors = read_json(&proposal_dir_of(&fixtures).join("node-index-vectors.json"))
        .expect("QSpec operand vectors");
    assert_eq!(vectors["version"], "quire.checked-node-index-vectors/v1");
    assert_eq!(vectors["base"], "fixtures/positive-dependency-call.json");
    let rows = vectors["cases"].as_array().expect("published cases");
    assert_eq!(
        rows.len(),
        22,
        "NI-01..NI-12 and NI-15..NI-24 are a closed published set"
    );
    let base = read_json_wire(&fixtures.join("positive-dependency-call.json"))
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
        let expected = row["expected"].as_str().expect("vector expectation");
        if expected.starts_with("passes_") {
            // The operand checks (and, for NI-22, the duplicate check) pass:
            // a later semantic refusal is allowed, an operand or duplicate-id
            // refusal is not.
            if let CheckedPackageV2ReadResult::Refused(refusal) = &actual {
                assert_ne!(
                    code_word(refusal.code),
                    "malformed_wire",
                    "{id}: operand check"
                );
                assert!(
                    refusal
                        .loci
                        .iter()
                        .all(|locus| locus.path.as_ref().is_none_or(
                            |path| path.to_string() != "/semantic_graph/nodes/4/node_id"
                        )),
                    "{id}: duplicate-id check: {refusal:?}"
                );
            }
            continue;
        }
        let CheckedPackageV2ReadResult::Refused(refusal) = actual else {
            panic!("{id}: expected a refusal, got {actual:?}");
        };
        assert_eq!(code_word(refusal.code), expected, "{id}: code");
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
    for index in (1..=12).chain(15..=24) {
        assert!(
            seen.contains(format!("NI-{index:02}").as_str()),
            "missing NI-{index:02}"
        );
    }
}

fn vector_case<'a>(vectors: &'a Value, id: &str) -> &'a Value {
    vectors["cases"]
        .as_array()
        .expect("published cases")
        .iter()
        .find(|row| row["id"] == id)
        .unwrap_or_else(|| panic!("published {id}"))
}

fn admitted_wire(wire: &Value) -> Box<quire_contract_model::CheckedPackageV2> {
    match read_package(wire).expect("production read") {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("package must admit: {other:?}"),
    }
}

fn node_id_of(value: &Value) -> CheckedNodeId {
    serde_json::from_value(value.clone()).expect("published node id")
}

#[trace("TC-048", "FR-038-AC-520", "FR-038-AC-523")]
#[test]
fn operand_slots_rebuild_structural_keys_through_the_rows() {
    let fixtures = fixtures_dir();
    let wire = read_json_wire(&fixtures.join("positive-dependency-call.json"))
        .expect("published dependency-call fixture");
    let package = admitted_wire(&wire);
    assert_eq!(
        package.graph().operand_rows,
        vec![
            vec![],
            vec![],
            vec![CheckedOperandEntry::Node(0)],
            vec![CheckedOperandEntry::Node(0)]
        ]
    );
    let integer = package.structural_key_at(0).expect("Integer key");
    let bounded = package.structural_key_at(2).expect("Int[0, 9] key");
    let CheckedNodePathSegment::Node(operand) = &bounded.path[3] else {
        panic!(
            "operand slot 0 rebuilds as the Integer key: {:?}",
            bounded.path
        );
    };
    assert_eq!(operand.as_ref(), integer);
    assert_eq!(
        package.graph().nodes[2].node_id.path[3],
        CheckedNodePathSegment::Operand(0),
        "the graph keeps the slot the wire holds"
    );
    assert_eq!(
        package.document_held_id(2).map(|id| id.path[3].clone()),
        Some(CheckedNodePathSegment::Index(0)),
        "a request names the operand by its graph index"
    );
    // Node 3 names the same bounded domain through its own row.
    let body = serde_json::to_value(&package.graph().nodes[3].body).expect("body");
    let type_id = node_id_of(&body["arguments"][1]["type"]);
    assert_eq!(type_id.path[3], CheckedNodePathSegment::Operand(0));
    assert_eq!(
        package.resolve_node_held(3, &type_id).as_ref(),
        Some(bounded)
    );
    // The graph written again is the published graph.
    assert_eq!(
        serde_json::to_value(package.graph()).expect("graph"),
        wire["semantic_graph"]
    );
    // A published package is its resolved form and back.
    let mut resolved = wire.clone();
    fixture_support::resolve_operands(&mut resolved);
    assert!(resolved["semantic_graph"].get("operand_rows").is_none());
    assert_eq!(
        resolved["semantic_graph"]["nodes"][2]["node_id"]["path"][3],
        json!({"node":0})
    );
    fixture_support::wire_operands(&mut resolved);
    assert_eq!(resolved, wire);
}

#[trace("TC-048", "FR-038-AC-523")]
#[test]
fn replace_node_id_replaces_slot_form_ids_as_written() {
    let fixtures = fixtures_dir();
    let wire = read_json_wire(&fixtures.join("positive-dependency-call.json"))
        .expect("published dependency-call fixture");
    let package = admitted_wire(&wire);
    let mut node = package.graph().nodes[3].clone();
    let body = serde_json::to_value(&node.body).expect("body");
    let old = node_id_of(&body["arguments"][1]["type"]);
    let mut new = old.clone();
    new.ordinal = 5;
    node.replace_node_id(&old, &new);
    let body = serde_json::to_value(&node.body).expect("body");
    assert_eq!(body["arguments"][1]["type"]["ordinal"], 5);
    assert_eq!(
        body["arguments"][1]["type"]["path"][3],
        json!({"operand": 0})
    );
    assert_eq!(
        serde_json::to_value(&package.graph().nodes[3].body).expect("body")["arguments"][1]["type"]
            ["ordinal"],
        0,
        "the admitted package is not changed"
    );
}

#[trace("TC-048", "FR-038-AC-522")]
#[test]
fn unrelated_nodes_leave_other_nodes_bytes_and_keys_unchanged() {
    let fixtures = fixtures_dir();
    let vectors = read_json(&proposal_dir_of(&fixtures).join("node-index-vectors.json"))
        .expect("QSpec operand vectors");
    let base = read_json_wire(&fixtures.join("positive-dependency-call.json"))
        .expect("published dependency-call fixture");
    let original = admitted_wire(&base);

    // A graph reversed with its rows and indexes rewritten: unrelated order.
    let mut reversed = read_json(&fixtures.join("positive-dependency-call.json"))
        .expect("published dependency-call fixture");
    reversed["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("published graph")
        .reverse();
    assert!(reverse_graph_indexes(&mut reversed, 4) > 0);
    let after = admitted_wire(&reversed);
    assert_eq!(
        after.graph().operand_rows,
        vec![
            vec![CheckedOperandEntry::Node(3)],
            vec![CheckedOperandEntry::Node(3)],
            vec![],
            vec![]
        ]
    );
    for position in 0..4 {
        assert_eq!(
            serde_json::to_value(&original.graph().nodes[position]).expect("node"),
            serde_json::to_value(&after.graph().nodes[3 - position]).expect("node"),
            "node {position} keeps its bytes"
        );
        assert_eq!(
            original.structural_key_at(position),
            after.structural_key_at(3 - position),
            "node {position} keeps its key"
        );
    }

    // NI-22: equal bytes over a different operand are a different key.
    let mut copied = base;
    for patch in vector_case(&vectors, "NI-22")["patch"]
        .as_array()
        .expect("patches")
    {
        apply_patch(&mut copied, patch);
    }
    if let CheckedPackageV2ReadResult::Admitted(package) = read_package(&copied).expect("read") {
        assert_eq!(
            serde_json::to_value(&package.document_held_id(4).expect("graph node").clone())
                .expect("id"),
            serde_json::to_value(&package.document_held_id(2).expect("graph node").clone())
                .expect("id"),
        );
        assert_ne!(package.structural_key_at(4), package.structural_key_at(2));
    }
}

fn reverse_graph_indexes(value: &mut Value, node_count: usize) -> usize {
    let old_to_new = (0..node_count).rev().collect::<Vec<_>>();
    remap_graph_indexes(value, &old_to_new)
}

pub(super) fn remap_graph_indexes(value: &mut Value, old_to_new: &[usize]) -> usize {
    match value {
        Value::Object(map) if map.len() == 1 && map.contains_key("node") => {
            let old = map["node"].as_u64().expect("published index");
            let old: usize = old.try_into().expect("index fits usize");
            let new = old_to_new.get(old).expect("published index in range");
            assert_ne!(*new, usize::MAX, "retained fixture names a dropped node");
            map.insert("node".into(), json!(new));
            1
        }
        Value::Object(map) => map
            .values_mut()
            .map(|part| remap_graph_indexes(part, old_to_new))
            .sum(),
        Value::Array(items) => items
            .iter_mut()
            .map(|part| remap_graph_indexes(part, old_to_new))
            .sum(),
        _ => 0,
    }
}

fn remap_source_references(value: &mut Value, old_to_new: &[usize]) -> usize {
    match value {
        Value::Object(map) if map.len() == 1 && map.contains_key("source_key") => {
            let old = map["source_key"].as_u64().expect("source-key reference");
            let old: usize = old.try_into().expect("source-key position fits usize");
            let new = old_to_new.get(old).expect("source-key reference in range");
            map.insert("source_key".into(), json!(new));
            1
        }
        Value::Object(map) => map
            .values_mut()
            .map(|part| remap_source_references(part, old_to_new))
            .sum(),
        Value::Array(items) => items
            .iter_mut()
            .map(|part| remap_source_references(part, old_to_new))
            .sum(),
        _ => 0,
    }
}

// Resolve this small published fixture's output indexes independently of the
// reader, so the source and output tables can be compared by structural key.
fn fixture_structural_key_from_ids(ids: &[CheckedNodeId], position: usize) -> CheckedNodeId {
    fixture_structural_key_for_id(ids, &ids[position])
}

pub(super) fn fixture_structural_key_for_id(
    ids: &[CheckedNodeId],
    id: &CheckedNodeId,
) -> CheckedNodeId {
    CheckedNodeId {
        package: id.package.clone(),
        path: id
            .path
            .iter()
            .map(|segment| match segment {
                CheckedNodePathSegment::Name(name) => CheckedNodePathSegment::Name(name.clone()),
                CheckedNodePathSegment::Index(index) => {
                    let operand = usize::try_from(*index).expect("fixture index fits usize");
                    CheckedNodePathSegment::Node(Arc::new(fixture_structural_key_from_ids(
                        ids, operand,
                    )))
                }
                CheckedNodePathSegment::Node(node) => CheckedNodePathSegment::Node(node.clone()),
                // A dependency position is already the resolved form of a
                // foreign operand; it names a node of another graph.
                CheckedNodePathSegment::Dependency { .. } => segment.clone(),
                CheckedNodePathSegment::Operand(_) => {
                    panic!("resolve an operand slot through its row first")
                }
            })
            .collect(),
        ordinal: id.ordinal,
    }
}

/// Every node's id, in resolved index form: a wire graph's operand slots are
/// read through the node's row.
pub(super) fn fixture_graph_ids(wire: &Value) -> Vec<CheckedNodeId> {
    let rows = wire["semantic_graph"]["operand_rows"].as_array();
    wire["semantic_graph"]["nodes"]
        .as_array()
        .expect("fixture graph nodes")
        .iter()
        .enumerate()
        .map(|(position, node)| {
            let id = serde_json::from_value::<CheckedNodeId>(node["node_id"].clone())
                .expect("fixture node id");
            let Some(rows) = rows else {
                return id;
            };
            let row = rows[position]
                .as_array()
                .expect("fixture operand row")
                .iter()
                .map(|entry| {
                    serde_json::from_value::<CheckedOperandEntry>(entry.clone())
                        .expect("fixture row entry")
                })
                .collect::<Vec<_>>();
            resolve_slots(&id, &row)
        })
        .collect()
}

/// An id with each operand slot replaced by the index segment its row names.
pub(super) fn resolve_slots(id: &CheckedNodeId, row: &[CheckedOperandEntry]) -> CheckedNodeId {
    CheckedNodeId {
        package: id.package.clone(),
        path: id
            .path
            .iter()
            .map(|segment| match segment {
                CheckedNodePathSegment::Operand(slot) => {
                    match &row[usize::try_from(*slot).expect("slot fits usize")] {
                        CheckedOperandEntry::Node(index) => CheckedNodePathSegment::Index(*index),
                        CheckedOperandEntry::Dependency(operand) => {
                            CheckedNodePathSegment::Dependency {
                                package: operand.package.clone(),
                                node: operand.node,
                            }
                        }
                    }
                }
                other => other.clone(),
            })
            .collect(),
        ordinal: id.ordinal,
    }
}

pub(super) fn sort_fixture_node_refs(wire: &Value, references: &mut [Value]) {
    let ids = fixture_graph_ids(wire);
    sort_fixture_node_refs_with_ids(&ids, references);
}

pub(super) fn sort_fixture_node_refs_with_ids(ids: &[CheckedNodeId], references: &mut [Value]) {
    references.sort_by(|left, right| {
        let left: CheckedNodeId = serde_json::from_value(left.clone()).expect("left node ref");
        let right: CheckedNodeId = serde_json::from_value(right.clone()).expect("right node ref");
        fixture_structural_key_for_id(ids, &left).cmp(&fixture_structural_key_for_id(ids, &right))
    });
}

pub(super) fn fixture_structural_key<Body>(
    nodes: &[CheckedSemanticNodeV2<Body>],
    operand_rows: &[Vec<CheckedOperandEntry>],
    position: usize,
) -> CheckedNodeId {
    let ids = nodes
        .iter()
        .zip(operand_rows)
        .map(|(node, row)| resolve_slots(&node.node_id, row))
        .collect::<Vec<_>>();
    fixture_structural_key_from_ids(&ids, position)
}

/// The structural key of a graph whose ids already hold index segments, as a
/// ContractPackage output graph does.
fn fixture_indexed_structural_key<Body>(
    nodes: &[CheckedSemanticNodeV2<Body>],
    position: usize,
) -> CheckedNodeId {
    let ids = nodes
        .iter()
        .map(|node| node.node_id.clone())
        .collect::<Vec<_>>();
    fixture_structural_key_from_ids(&ids, position)
}

fn fixture_source_ref_key(wire: &Value, reference: &Value) -> CheckedNodeId {
    let ids = wire["source_keys"]
        .as_array()
        .expect("source keys")
        .iter()
        .map(|id| serde_json::from_value(id.clone()).expect("source key shape"))
        .collect::<Vec<_>>();
    let position = reference["source_key"]
        .as_u64()
        .expect("source-key reference") as usize;
    fixture_structural_key_from_ids(&ids, position)
}

// Source and output frames retain independently chosen graph orders.
#[trace("TC-934", "FR-330-AC-16", "FR-035-AC-34")]
#[test]
fn contract_source_and_output_index_tables_permute_independently() {
    let mut source_wire = read_json(&fixtures_dir().join("positive-operation-identities.json"))
        .expect("published indexed operations");
    source_wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("published graph")
        .swap(31, 34);
    let source_count = source_wire["semantic_graph"]["nodes"]
        .as_array()
        .expect("published graph")
        .len();
    let mut source_positions = (0..source_count).collect::<Vec<_>>();
    source_positions.swap(31, 34);
    assert!(remap_graph_indexes(&mut source_wire, &source_positions) > 0);
    let outcome = read_package(&source_wire).expect("published source read");
    let CheckedPackageV2ReadResult::Admitted(source) = outcome else {
        panic!("published source admits: {outcome:?}");
    };
    let requests = [25, 26].map(|position| {
        source
            .document_held_id(position)
            .expect("graph node")
            .clone()
    });
    let mut absent = requests[0].clone();
    absent.path[0] = CheckedNodePathSegment::Name("absent".into());
    let profile = CompleteLoweringProfileV2 {
        reference: LoweringProfileRef {
            authority: "quire".into(),
            identity: "complete-v1".into(),
        },
        required_source_selections: Vec::new(),
        supported_tags: CheckedNodeTag::ALL.iter().copied().collect(),
        require_bounds: false,
        work_limit: u64::MAX,
    };
    let lowered = source.lower(
        &[requests[0].clone(), requests[1].clone(), absent],
        &profile,
    );
    assert!(
        matches!(
            lowered.records.as_slice(),
            [
                CompleteLoweringRecordV2::Lowered { .. },
                CompleteLoweringRecordV2::Lowered { .. },
                CompleteLoweringRecordV2::InvalidInput { .. }
            ]
        ),
        "both indexed roots lower and absent derived key refuses: {:?}",
        lowered.records
    );
    let original_wire: Value = serde_json::from_slice(
        lowered
            .package
            .canonical_bytes()
            .expect("canonical package"),
    )
    .expect("ContractPackage JSON");
    let original = ContractPackageWireV1::read(
        lowered
            .package
            .canonical_bytes()
            .expect("canonical package"),
        CheckedPackageReadLimits::bounded(),
        &ContractDependencies::new(),
    );
    let ContractPackageReadResult::Admitted(original) = original else {
        panic!("producer package must admit: {original:?}");
    };

    let output_len = original_wire["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    let source_len = original_wire["source_keys"]
        .as_array()
        .expect("source keys")
        .len();
    assert!(
        output_len > 1 && source_len > 1,
        "both tables are nontrivial"
    );
    let output_positions = (0..output_len).rev().collect::<Vec<_>>();
    let source_positions = (0..source_len).rev().collect::<Vec<_>>();
    let output_keys = |wire: &Value| {
        let ids = wire["semantic_graph"]["nodes"]
            .as_array()
            .expect("output nodes")
            .iter()
            .map(|node| serde_json::from_value(node["node_id"].clone()).expect("output node id"))
            .collect::<Vec<_>>();
        (0..ids.len())
            .map(|position| fixture_structural_key_from_ids(&ids, position))
            .collect::<BTreeSet<_>>()
    };
    let expected_output_keys = output_keys(&original_wire);
    for (request_position, source_position) in [(0_usize, 25_usize), (1, 26)] {
        let expected = source
            .structural_key_at(source_position)
            .expect("source structural key");
        let source_ref = &original_wire["feature_accounting"][request_position]["node_id"];
        assert_eq!(
            fixture_source_ref_key(&original_wire, source_ref),
            *expected
        );
        assert!(
            expected_output_keys.contains(expected),
            "lowered source root joins output graph"
        );
    }
    for (output, source_table) in [(false, false), (false, true), (true, false), (true, true)] {
        let mut wire = original_wire.clone();
        if output {
            wire["semantic_graph"]["nodes"]
                .as_array_mut()
                .expect("nodes")
                .reverse();
            wire["source_map"]["nodes"]
                .as_array_mut()
                .expect("node map")
                .reverse();
            assert!(remap_graph_indexes(&mut wire["semantic_graph"], &output_positions) > 0);
            assert!(remap_graph_indexes(&mut wire["source_map"]["nodes"], &output_positions) > 0);
        }
        if source_table {
            wire["source_keys"]
                .as_array_mut()
                .expect("source keys")
                .reverse();
            assert!(remap_graph_indexes(&mut wire["source_keys"], &source_positions) > 0);
            assert!(remap_source_references(&mut wire, &source_positions) > 0);
        }
        let permuted = ContractPackageWireV1::read(
            &super::fixture_support::canonical(&wire),
            CheckedPackageReadLimits::bounded(),
            &ContractDependencies::new(),
        );
        let ContractPackageReadResult::Admitted(permuted) = permuted else {
            panic!(
                "output={output} source={source_table}: permuted tables must admit: {permuted:?}"
            );
        };
        assert_eq!(output_keys(&wire), expected_output_keys);
        assert_eq!(
            permuted.wire().feature_accounting.len(),
            original.wire().feature_accounting.len()
        );
        for position in 0..3 {
            for path in [
                format!("/feature_accounting/{position}/node_id"),
                format!("/source_map/requested_items/{position}/requested_item"),
            ] {
                assert_eq!(
                    fixture_source_ref_key(
                        &wire,
                        wire.pointer(&path).expect("permuted source reference")
                    ),
                    fixture_source_ref_key(
                        &original_wire,
                        original_wire
                            .pointer(&path)
                            .expect("original source reference")
                    ),
                    "output={output} source={source_table}: {path} compares by structural key"
                );
            }
        }
        assert_eq!(
            fixture_source_ref_key(&wire, &wire["source_map"]["refusals"][0]["requested_item"]),
            fixture_source_ref_key(
                &original_wire,
                &original_wire["source_map"]["refusals"][0]["requested_item"]
            ),
            "refusal pointer compares the requested structural key"
        );
        let output_ids = fixture_graph_ids(&wire);
        for (position, mapped) in wire["source_map"]["nodes"]
            .as_array()
            .expect("node sources")
            .iter()
            .enumerate()
        {
            let output_key = fixture_structural_key_from_ids(&output_ids, position);
            let mapped_id: CheckedNodeId =
                serde_json::from_value(mapped["node_id"].clone()).expect("mapped output id");
            assert_eq!(
                fixture_structural_key_for_id(&output_ids, &mapped_id),
                output_key
            );
            for occurrence in mapped["occurrences"]
                .as_array()
                .expect("source occurrences")
            {
                assert_eq!(
                    fixture_source_ref_key(&wire, &occurrence["node_id"]),
                    output_key,
                    "output={output} source={source_table}: cross-frame occurrence {position}"
                );
            }
        }
        let first = wire["feature_accounting"][0]["node_id"].clone();
        let second = wire["feature_accounting"][1]["node_id"].clone();
        assert_ne!(
            fixture_source_ref_key(&wire, &first),
            fixture_source_ref_key(&wire, &second)
        );
        // Unequal keys must not join even when both local references are in range.
        for path in [
            "/feature_accounting/0/node_id",
            "/source_map/refusals/0/requested_item",
        ] {
            let mut unequal = wire.clone();
            *unequal.pointer_mut(path).expect("source-frame join") = second.clone();
            assert_v1_refusal(
                &unequal,
                "invalid_package",
                Some("invalid-value"),
                "/feature_accounting",
            );
        }
        let mut unequal_output = wire.clone();
        unequal_output["source_map"]["nodes"][0]["node_id"] =
            wire["semantic_graph"]["nodes"][1]["node_id"].clone();
        assert_v1_refusal(
            &unequal_output,
            "invalid_package",
            Some("invalid-value"),
            "/source_map/nodes/0/node_id",
        );
        let (position, mapped) = wire["source_map"]["nodes"]
            .as_array()
            .expect("node sources")
            .iter()
            .enumerate()
            .find(|(_, mapped)| {
                !mapped["occurrences"]
                    .as_array()
                    .expect("source occurrences")
                    .is_empty()
            })
            .expect("a real cross-frame source occurrence");
        let current = fixture_source_ref_key(&wire, &mapped["occurrences"][0]["node_id"]);
        let wrong = if current == fixture_source_ref_key(&wire, &first) {
            second
        } else {
            first
        };
        let mut unequal_occurrence = wire.clone();
        unequal_occurrence["source_map"]["nodes"][position]["occurrences"][0]["node_id"] = wrong;
        assert_v1_refusal(
            &unequal_occurrence,
            "invalid_package",
            Some("invalid-value"),
            &format!("/source_map/nodes/{position}/occurrences/0"),
        );
    }
}

fn assert_v1_refusal(wire: &Value, code: &str, cause: Option<&str>, path: &str) {
    let outcome = ContractPackageWireV1::read(
        &super::fixture_support::canonical(wire),
        CheckedPackageReadLimits::bounded(),
        &ContractDependencies::new(),
    );
    let ContractPackageReadResult::Refused(refusal) = outcome else {
        panic!("{path}: unequal or malformed frame must refuse: {outcome:?}");
    };
    assert_eq!(code_word(refusal.code), code, "{path}: {refusal:?}");
    assert_eq!(refusal.cause.map(cause_word), cause, "{path}: {refusal:?}");
    assert_eq!(refusal.loci.len(), 1, "{path}: {refusal:?}");
    assert_eq!(
        refusal.loci[0]
            .path
            .as_ref()
            .map(ToString::to_string)
            .as_deref(),
        Some(path),
        "{refusal:?}"
    );
}

#[trace("TC-047", "FR-035-AC-23")]
#[test]
fn v1_indexed_frame_joins_use_resolved_node_keys() {
    let mut wire = read_json(&fixtures_dir().join("positive-operation-identities.json"))
        .expect("published indexed operations");
    wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("published nodes")
        .swap(31, 34);
    let mut old_to_new =
        (0..wire["semantic_graph"]["nodes"].as_array().unwrap().len()).collect::<Vec<_>>();
    old_to_new.swap(31, 34);
    assert!(remap_graph_indexes(&mut wire, &old_to_new) > 0);
    let outcome = read_package(&wire).expect("published source read");
    let CheckedPackageV2ReadResult::Admitted(source) = outcome else {
        panic!("published source admits: {outcome:?}");
    };
    assert!(
        source.document_held_id(25).expect("graph node").clone()
            < source.document_held_id(26).expect("graph node").clone()
    );
    assert!(source.structural_key_at(26) < source.structural_key_at(25));
    let requests = [25, 26].map(|position| {
        source
            .document_held_id(position)
            .expect("graph node")
            .clone()
    });
    let profile = CompleteLoweringProfileV2 {
        reference: LoweringProfileRef {
            authority: "quire".into(),
            identity: "complete-v1".into(),
        },
        required_source_selections: Vec::new(),
        supported_tags: CheckedNodeTag::ALL.iter().copied().collect(),
        require_bounds: false,
        work_limit: u64::MAX,
    };
    let lowered = source.lower(&requests, &profile);
    assert!(
        lowered
            .records
            .iter()
            .all(|record| matches!(record, CompleteLoweringRecordV2::Lowered { .. })),
        "both published indexed applications lower: {:?}",
        lowered.records
    );
    let produced = lowered.package.wire().expect("lowered wire").wire();
    let output_keys = (0..produced.semantic_graph.nodes.len())
        .map(|position| fixture_indexed_structural_key(&produced.semantic_graph.nodes, position))
        .collect::<Vec<_>>();
    let earlier = output_keys
        .iter()
        .position(|key| Some(key) == source.structural_key_at(26))
        .expect("second source key has a structurally equal output root");
    let later = output_keys
        .iter()
        .position(|key| Some(key) == source.structural_key_at(25))
        .expect("first source key has a structurally equal output root");
    assert_eq!(
        later,
        earlier + 1,
        "producer emits resolved-key order, despite wire-index order"
    );
    let bytes = lowered
        .package
        .canonical_bytes()
        .expect("canonical V1 bytes");
    let read_back = ContractPackageWireV1::read(
        bytes,
        CheckedPackageReadLimits::bounded(),
        &ContractDependencies::new(),
    );
    let ContractPackageReadResult::Admitted(read_back) = read_back else {
        panic!("V1 reader admits indexed producer output: {read_back:?}");
    };
    let producer = lowered.package.wire().expect("producer package");
    for node in &produced.semantic_graph.nodes {
        assert_eq!(
            producer.term(&node.node_id),
            read_back.term(&node.node_id),
            "producer and V1 reader retain identical typed body for projected key"
        );
    }
    let authenticated = ContractPackageWireV1::read_with_source(
        bytes,
        CheckedPackageReadLimits::bounded(),
        &ContractDependencies::new(),
        &source,
    )
    .expect("structural source correspondence after V1 projection");
    let ContractPackageReadResult::Admitted(authenticated) = authenticated else {
        panic!("indexed V1 source handoff must admit: {authenticated:?}");
    };
    let key = &produced.semantic_graph.nodes[later].node_id;
    let Some(CheckedTerm::Application { operation, .. }) = authenticated.term(key) else {
        panic!("indexed source handoff retains application");
    };
    assert!(
        operation.mode().is_ok(),
        "source handoff restores admitted operation views"
    );
    let Some(CheckedTerm::Application { operation, .. }) = producer.term(key) else {
        panic!("indexed producer retains application");
    };
    assert!(
        operation.mode().is_ok(),
        "producer retains admitted operation views"
    );
    let mut wrong_order: Value = serde_json::from_slice(bytes).expect("canonical V1 value");
    wrong_order["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("projected graph")
        .swap(earlier, later);
    let wrong_bytes = serde_json::to_vec(&wrong_order).expect("mutated canonical JSON");
    let wrong_result = ContractPackageWireV1::read(
        &wrong_bytes,
        CheckedPackageReadLimits::bounded(),
        &ContractDependencies::new(),
    );
    let ContractPackageReadResult::Refused(refusal) = wrong_result else {
        panic!(
            "V1 reader must refuse an unrepaired output/source-map permutation: {wrong_result:?}"
        );
    };
    assert_eq!(code_word(refusal.code), "invalid_package");
    assert_eq!(
        cause_word(refusal.cause.expect("invalid-value cause")),
        "invalid-value"
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some(format!("/source_map/nodes/{earlier}/node_id")),
        "swapping output nodes without the corresponding source-map entries breaks their join"
    );
}

#[trace("TC-933", "TC-934", "FR-330-AC-2", "FR-330-AC-14", "FR-035-AC-32")]
#[test]
fn v1_absent_indexed_request_retains_invalid_input_and_exact_key() {
    let wire = read_json(&fixtures_dir().join("positive-operation-identities.json"))
        .expect("published indexed operations");
    let source_outcome = read_package(&wire).expect("published source read");
    let CheckedPackageV2ReadResult::Admitted(source) = source_outcome else {
        panic!("published source admits: {source_outcome:?}");
    };
    let mut absent = source.document_held_id(25).expect("graph node").clone();
    absent.path[0] = CheckedNodePathSegment::Name("absent".into());
    assert_eq!(absent.path[2], CheckedNodePathSegment::Index(34));
    assert_eq!(absent.path[7], CheckedNodePathSegment::Index(21));
    let profile = CompleteLoweringProfileV2 {
        reference: LoweringProfileRef {
            authority: "quire".into(),
            identity: "complete-v1".into(),
        },
        required_source_selections: Vec::new(),
        supported_tags: CheckedNodeTag::ALL.iter().copied().collect(),
        require_bounds: false,
        work_limit: u64::MAX,
    };
    let result = source.lower(std::slice::from_ref(&absent), &profile);
    let [CompleteLoweringRecordV2::InvalidInput { node_id }] = result.records.as_slice() else {
        panic!(
            "absent indexed request must remain InvalidInput: {:?}",
            result.records
        );
    };
    assert_eq!(
        node_id, &absent,
        "invalid input retains exact requested key"
    );
    let bytes = result
        .package
        .canonical_bytes()
        .expect("canonical refusal package")
        .to_vec();
    let emitted: Value = serde_json::from_slice(&bytes).expect("canonical package value");
    assert_eq!(emitted["semantic_graph"]["nodes"], json!([]));
    let source_keys = emitted["source_keys"].as_array().expect("source-key table");
    let request_index = emitted["feature_accounting"][0]["node_id"]["source_key"]
        .as_u64()
        .expect("request has a source-key reference") as usize;
    let request_key = source_keys
        .get(request_index)
        .expect("requested source key");
    assert_eq!(request_key["path"][0], "absent");
    let source_ids = source_keys
        .iter()
        .map(|id| serde_json::from_value(id.clone()).expect("source-key shape"))
        .collect::<Vec<_>>();
    for (segment, source_position) in [(2_usize, 34_usize), (7, 21)] {
        let operand = request_key["path"][segment]["node"]
            .as_u64()
            .expect("indexed operand") as usize;
        assert!(
            source_keys.get(operand).is_some(),
            "source operand in table"
        );
        assert_eq!(
            fixture_structural_key_from_ids(&source_ids, operand),
            *source
                .structural_key_at(source_position)
                .expect("source operand key"),
            "path segment {segment} names its original source operand"
        );
    }
    let mut expected_absent = source
        .structural_key_at(25)
        .expect("derived source key")
        .clone();
    expected_absent.path[0] = CheckedNodePathSegment::Name("absent".into());
    assert_eq!(
        fixture_source_ref_key(&emitted, &emitted["feature_accounting"][0]["node_id"]),
        expected_absent,
        "request retains the source-authored structural identity"
    );
    assert_eq!(
        emitted["source_map"]["requested_items"][0]["requested_item"],
        json!({"source_key": request_index})
    );
    assert_eq!(
        emitted["source_map"]["requested_items"][0]["occurrences"],
        json!([])
    );
    assert_eq!(
        emitted["source_map"]["refusals"][0]["requested_item"],
        json!({"source_key": request_index})
    );
    // No emitted reference can depend on the source package remaining available.
    drop(result);
    drop(source);
    drop(wire);
    let read_back = ContractPackageWireV1::read(
        &bytes,
        CheckedPackageReadLimits::bounded(),
        &ContractDependencies::new(),
    );
    let ContractPackageReadResult::Admitted(read_back) = read_back else {
        panic!("reader admits exact invalid-input record without its source: {read_back:?}");
    };
    let mut with_extra = emitted.clone();
    let extra_position = with_extra["source_keys"]
        .as_array()
        .expect("source keys")
        .len();
    let extra = json!({"package":"example", "path":["unused-source-key"], "ordinal":0});
    with_extra["source_keys"]
        .as_array_mut()
        .expect("source keys")
        .push(extra.clone());
    let extra_read = ContractPackageWireV1::read(
        &super::fixture_support::canonical(&with_extra),
        CheckedPackageReadLimits::bounded(),
        &ContractDependencies::new(),
    );
    let ContractPackageReadResult::Admitted(extra_read) = extra_read else {
        panic!("unused unique source keys are permitted: {extra_read:?}");
    };
    assert_eq!(
        serde_json::to_value(&extra_read.wire().source_keys[extra_position]).expect("extra key"),
        extra
    );
    assert!(extra_read.wire().semantic_graph.nodes.is_empty());
    for (bad, code, cause, suffix) in [
        (
            emitted["source_keys"][0].clone(),
            "invalid_package",
            Some("invalid-value"),
            "",
        ),
        (
            json!({"package":"example", "path":["unused-source-key"], "ordinal":-1}),
            "invalid_package",
            Some("invalid-value"),
            "/ordinal",
        ),
        (
            json!({"package":"example", "path":[{"node":extra_position + 1}], "ordinal":0}),
            "malformed_wire",
            None,
            "/path/0/node",
        ),
    ] {
        let mut invalid = with_extra.clone();
        invalid["source_keys"][extra_position] = bad;
        assert_v1_refusal(
            &invalid,
            code,
            cause,
            &format!("/source_keys/{extra_position}{suffix}"),
        );
    }
    // Every source-frame ID position remains an explicit source-key reference.
    for reference in [
        &emitted["feature_accounting"][0]["node_id"],
        &emitted["source_map"]["requested_items"][0]["requested_item"],
        &emitted["source_map"]["refusals"][0]["requested_item"],
    ] {
        assert_eq!(reference.as_object().expect("reference").len(), 1);
        assert_eq!(fixture_source_ref_key(&emitted, reference), expected_absent);
    }
    assert!(read_back.wire().semantic_graph.nodes.is_empty());
    assert!(matches!(
        read_back.wire().feature_accounting[0].disposition(),
        quire_contract_model::ContractDispositionV1::InvalidInput
    ));
}

#[trace("TC-935", "FR-330-AC-2")]
#[test]
fn v1_indexed_unsupported_request_authenticates_its_source_kind() {
    let wire = read_json(&fixtures_dir().join("positive-operation-identities.json"))
        .expect("published indexed operations");
    let source_outcome = read_package(&wire).expect("published source read");
    let CheckedPackageV2ReadResult::Admitted(source) = source_outcome else {
        panic!("published source admits: {source_outcome:?}");
    };
    let unsupported = source.document_held_id(25).expect("graph node").clone();
    let supported = source.document_held_id(34).expect("graph node").clone();
    let profile = CompleteLoweringProfileV2 {
        reference: LoweringProfileRef {
            authority: "quire".into(),
            identity: "complete-v1".into(),
        },
        required_source_selections: Vec::new(),
        supported_tags: CheckedNodeTag::ALL
            .iter()
            .copied()
            .filter(|tag| *tag != CheckedNodeTag::Expression)
            .collect(),
        require_bounds: false,
        work_limit: u64::MAX,
    };
    let result = source.lower(&[unsupported, supported], &profile);
    assert!(
        matches!(
            result.records.as_slice(),
            [
                CompleteLoweringRecordV2::Unsupported { .. },
                CompleteLoweringRecordV2::Lowered { .. }
            ]
        ),
        "unsupported sibling and lowered operand: {:?}",
        result.records
    );
    let bytes = result
        .package
        .canonical_bytes()
        .expect("canonical V1 bytes");
    assert_ne!(
        result
            .package
            .wire()
            .expect("producer package")
            .wire()
            .feature_accounting[0]
            .node_id(),
        &source.document_held_id(25).expect("graph node").clone(),
        "unsupported key is genuinely reindexed in the V1 projection"
    );
    let handoff = ContractPackageWireV1::read_with_source(
        bytes,
        CheckedPackageReadLimits::bounded(),
        &ContractDependencies::new(),
        &source,
    )
    .expect("indexed unsupported source correspondence");
    let ContractPackageReadResult::Admitted(package) = handoff else {
        panic!("indexed unsupported package admits: {handoff:?}");
    };
    assert_eq!(
        package.unsupported_source_kind_at(0),
        Ok(Some(CheckedNodeKind::Expression(ExpressionForm::Binary)))
    );
}

/// Trace: FR-038-AC-376, FR-038-AC-96
#[trace("TC-048", "FR-038-AC-376", "FR-038-AC-96")]
#[test]
fn graph_reindexing_preserves_structural_dependency_order() {
    let fixtures = fixtures_dir();
    let mut package = read_json(&fixtures.join("positive-operation-identities.json"))
        .expect("published operation fixture");
    let count = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("published graph")
        .len();
    let original = package.clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("published graph")
        .reverse();
    assert!(reverse_graph_indexes(&mut package, count) > 0);
    let mut opposed_wire_order = false;
    for wire in [&original, &package] {
        let result = read_package(wire).expect("published package read");
        let CheckedPackageV2ReadResult::Admitted(admitted) = result else {
            panic!("structurally ordered dependency graph must admit: {result:?}");
        };
        for (position, node) in admitted.resolved_graph().nodes.iter().enumerate() {
            let resolved = node
                .dependencies
                .iter()
                .map(|id| {
                    admitted
                        .resolve_node_held(position, id)
                        .expect("local dependency key")
                })
                .collect::<Vec<_>>();
            assert!(
                resolved.windows(2).all(|pair| pair[0] < pair[1]),
                "node {position}: resolved dependency order"
            );
            opposed_wire_order |= node.dependencies.windows(2).any(|pair| pair[0] > pair[1]);
        }
    }
    assert!(
        opposed_wire_order,
        "indexed wire and structural order differ"
    );
    let position = 13;
    assert_eq!(
        original["semantic_graph"]["nodes"][position]["semantic_form"],
        "binary"
    );
    let mut unsorted = original;
    unsorted["semantic_graph"]["nodes"][position]["dependencies"]
        .as_array_mut()
        .expect("multi-dependency node")
        .swap(0, 1);
    let result = read_package(&unsorted).expect("unsorted package read");
    let CheckedPackageV2ReadResult::Refused(refusal) = result else {
        panic!("structurally unsorted application dependencies must refuse");
    };
    assert_eq!(code_word(refusal.code), "invalid_semantic_graph");
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some(format!("/semantic_graph/nodes/{position}/dependencies"))
    );
}

/// Trace: FR-038-AC-96, FR-038-AC-376
#[trace("TC-048", "FR-038-AC-96", "FR-038-AC-376")]
#[test]
fn published_extension_and_hybrid_dependencies_follow_resolved_key_order() {
    let fixtures = fixtures_dir();
    let package = read_json(&fixtures.join("positive-all-families.json"))
        .expect("published all-families fixture");
    let position = 34;
    let node = &package["semantic_graph"]["nodes"][position];
    assert_eq!(node["node_tag"], "extension");
    assert_eq!(node["node_id"]["path"], json!(["Audit"]));
    assert_eq!(
        node["dependencies"]
            .as_array()
            .expect("Audit dependencies")
            .len(),
        2
    );
    assert_eq!(node["dependencies"][0]["path"][2], json!({"node":24}));
    assert_eq!(node["dependencies"][1]["path"][2], json!({"node":23}));

    let result = read_package(&package).expect("published package read");
    let CheckedPackageV2ReadResult::Admitted(admitted) = result else {
        panic!("published Audit extension must admit: {result:?}");
    };
    let hybrid_position = 40;
    assert_eq!(
        package["semantic_graph"]["nodes"][hybrid_position]["node_tag"],
        "hybrid"
    );
    assert_eq!(
        package["semantic_graph"]["nodes"][hybrid_position]["semantic_form"],
        "hybrid_mode"
    );
    for (position, label) in [(position, "Audit"), (hybrid_position, "hybrid mode")] {
        let dependencies = &admitted.graph().nodes[position].dependencies;
        let resolved = dependencies
            .iter()
            .map(|id| {
                admitted
                    .resolve_node_held(position, id)
                    .expect("local dependency key")
            })
            .collect::<Vec<_>>();
        assert!(
            resolved.windows(2).all(|pair| pair[0] < pair[1]),
            "{label} dependencies ascend by rebuilt keys"
        );

        let mut unsorted = package.clone();
        unsorted["semantic_graph"]["nodes"][position]["dependencies"]
            .as_array_mut()
            .expect("published dependencies")
            .swap(0, 1);
        let result = read_package(&unsorted).expect("unsorted package read");
        let CheckedPackageV2ReadResult::Refused(refusal) = result else {
            panic!("structurally unsorted {label} dependencies must refuse");
        };
        assert_eq!(code_word(refusal.code), "invalid_semantic_graph");
        assert_eq!(
            refusal.loci[0].path.as_ref().map(ToString::to_string),
            Some(format!("/semantic_graph/nodes/{position}/dependencies"))
        );
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
    let importer_result = read_package(&importer_wire).expect("importing package read");
    let CheckedPackageV2ReadResult::Admitted(importer) = importer_result else {
        panic!("NI-14 importing package must admit: {importer_result:?}");
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
    let permuted_result = read_package(&permuted_wire).expect("permuted importer read");
    let CheckedPackageV2ReadResult::Admitted(permuted) = permuted_result else {
        panic!("graph permutation with rewritten indexes must admit: {permuted_result:?}");
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
        CheckedPackageV2ReadResult::CatalogUnavailable(error) => {
            panic!("production operation catalog unavailable: {error:?}")
        }
    }
}

/// Trace: FR-038-AC-376
#[trace("TC-048", "FR-038-AC-376")]
#[test]
fn two_semantic_defects_choose_structural_order_over_graph_order() {
    let fixtures = fixtures_dir();
    let mut base = read_json(&fixtures.join("positive-operation-identities.json"))
        .expect("published local operation-identity package");
    let count = base["semantic_graph"]["nodes"]
        .as_array()
        .expect("published graph nodes")
        .len();
    assert_eq!(count, 62, "selected QSpec operation fixture cardinality");
    // Swap operand nodes 31 and 34, retaining application positions 25 and
    // 26. Rewriting all indexes keeps the positive package's meaning intact.
    base["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("published graph nodes")
        .swap(31, 34);
    let mut old_to_new = (0..count).collect::<Vec<_>>();
    old_to_new.swap(31, 34);
    assert!(remap_graph_indexes(&mut base, &old_to_new) > 0);
    let nodes = base["semantic_graph"]["nodes"]
        .as_array()
        .expect("published graph nodes");
    assert_eq!(
        nodes[25]["body"]["operation"]["identity"],
        "quire.op.ieee.float64.add"
    );
    assert_eq!(
        nodes[26]["body"]["operation"]["identity"],
        "quire.op.ieee.float64.add"
    );
    assert_eq!(nodes[25]["node_id"]["package"], "example");
    assert_eq!(nodes[26]["node_id"]["package"], "example");
    assert_eq!(
        &nodes[25]["node_id"]["path"].as_array().expect("path")[..2],
        &nodes[26]["node_id"]["path"].as_array().expect("path")[..2],
        "the first differing path element is an index operand"
    );
    assert_eq!(nodes[25]["node_id"]["path"][2], json!({"node":31}));
    assert_eq!(nodes[26]["node_id"]["path"][2], json!({"node":34}));
    let CheckedPackageV2ReadResult::Admitted(admitted) =
        read_package(&base).expect("permuted operation fixture read")
    else {
        panic!("valid operand permutation must admit");
    };
    assert!(
        admitted
            .structural_key_at(26)
            .expect("second application key")
            < admitted
                .structural_key_at(25)
                .expect("first application key"),
        "node 26's resolved operand sorts first although graph and wire-index order put it later"
    );
    for position in [25, 26] {
        let mut single = base.clone();
        single["semantic_graph"]["nodes"][position]["body"]["operation"]["identity"] =
            json!("quire.op.not-catalogued");
        let CheckedPackageV2ReadResult::Refused(refusal) =
            read_package(&single).expect("single operation-defect read")
        else {
            panic!("one unknown operation must refuse");
        };
        assert_eq!(code_word(refusal.code), "invalid_package");
        assert_eq!(refusal.cause.map(cause_word), Some("unknown-operation"));
        assert_eq!(
            refusal.loci[0].path.as_ref().map(ToString::to_string),
            Some(format!(
                "/semantic_graph/nodes/{position}/body/operation/identity"
            )),
            "each defect independently reaches the operation step"
        );
    }
    let mut package = base;
    for position in [25, 26] {
        package["semantic_graph"]["nodes"][position]["body"]["operation"]["identity"] =
            json!("quire.op.not-catalogued");
    }
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package(&package).expect("two-defect package read")
    else {
        panic!("two unknown operation identities must refuse");
    };
    assert_eq!(code_word(refusal.code), "invalid_package");
    assert_eq!(refusal.cause.map(cause_word), Some("unknown-operation"));
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some("/semantic_graph/nodes/26/body/operation/identity".to_owned()),
        "resolved operand order must win over graph and wire-index order"
    );
}

/// Trace: FR-038-AC-376
#[trace("TC-048", "FR-038-AC-376")]
#[test]
fn mixed_operation_defects_choose_resolved_operand_order() {
    let fixtures = fixtures_dir();
    let mut base = read_json(&fixtures.join("positive-operation-identities.json"))
        .expect("published local operation-identity package");
    let count = base["semantic_graph"]["nodes"]
        .as_array()
        .expect("published graph nodes")
        .len();
    assert_eq!(count, 62);
    base["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("published graph nodes")
        .swap(31, 34);
    let mut old_to_new = (0..count).collect::<Vec<_>>();
    old_to_new.swap(31, 34);
    assert!(remap_graph_indexes(&mut base, &old_to_new) > 0);
    let CheckedPackageV2ReadResult::Admitted(positive) =
        read_package(&base).expect("permuted operation fixture read")
    else {
        panic!("permuted operation fixture must admit");
    };
    assert_eq!(
        base["semantic_graph"]["nodes"][25]["node_id"]["path"][2],
        json!({"node":31})
    );
    assert_eq!(
        base["semantic_graph"]["nodes"][26]["node_id"]["path"][2],
        json!({"node":34})
    );
    assert!(
        positive.structural_key_at(26).expect("second key")
            < positive.structural_key_at(25).expect("first key"),
        "resolved operand order opposes graph and wire index order"
    );
    let unknown = |wire: &mut Value| {
        wire["semantic_graph"]["nodes"][25]["body"]["operation"]["identity"] =
            json!("quire.op.not-catalogued");
    };
    let wrong_class = |wire: &mut Value| {
        wire["semantic_graph"]["nodes"][26]["body"]["operator"] = json!("unary");
    };
    for (position, mutate, cause, member) in [
        (
            25,
            unknown as fn(&mut Value),
            "unknown-operation",
            "operation/identity",
        ),
        (
            26,
            wrong_class as fn(&mut Value),
            "operation-class-mismatch",
            "operator",
        ),
    ] {
        let mut single = base.clone();
        mutate(&mut single);
        let CheckedPackageV2ReadResult::Refused(refusal) =
            read_package(&single).expect("single mixed-defect read")
        else {
            panic!("one operation defect must refuse");
        };
        assert_eq!(code_word(refusal.code), "invalid_package");
        assert_eq!(refusal.cause.map(cause_word), Some(cause));
        assert_eq!(
            refusal.loci[0].path.as_ref().map(ToString::to_string),
            Some(format!("/semantic_graph/nodes/{position}/body/{member}")),
            "each defect independently reaches the operation step"
        );
    }
    let mut paired = base;
    unknown(&mut paired);
    wrong_class(&mut paired);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package(&paired).expect("paired mixed-defect read")
    else {
        panic!("paired operation defects must refuse");
    };
    assert_eq!(code_word(refusal.code), "invalid_package");
    assert_eq!(
        refusal.cause.map(cause_word),
        Some("operation-class-mismatch")
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some("/semantic_graph/nodes/26/body/operator".to_owned()),
        "structurally earlier node 26 must win with its distinct typed cause"
    );
}

/// Syntax and duplicate-member precedence already has a small-stack control
/// in `checked_package_v2_flat_wire`; noncanonical bytes have their independent
/// `checked_package_v2_canonical_encoding` controls.
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
    let bytes = fixture_support::canonical(&package);
    let limits = CheckedPackageReadLimits {
        bytes: u64::try_from(bytes.len()).expect("package byte count") + 1024,
        nodes: u64::try_from(APPLICATIONS + 1).expect("node count"),
        edges: u64::try_from(APPLICATIONS).expect("edge count"),
        occurrences: u64::try_from((APPLICATIONS + 1) * 2).expect("occurrence count"),
        diagnostics: 1,
        work: 100_000_000,
    };
    let document = proposal_dir_of(&fixtures).join(DOMAIN_DOCUMENT);
    std::thread::Builder::new()
        .stack_size(256 * 1024)
        .spawn(move || {
            let reading = read_package_with_document_and_extensions_at_limit(
                &package,
                "NI-13 deep index chain",
                &document,
                true,
                limits,
            )
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
            let deepest = admitted
                .structural_key_at(APPLICATIONS)
                .expect("deepest key");
            let previous = admitted
                .structural_key_at(APPLICATIONS - 1)
                .expect("previous key");
            let cloned = deepest.clone();
            assert_eq!(
                deepest, &cloned,
                "deep key clone retains structural equality"
            );
            assert_ne!(
                deepest, previous,
                "deep comparison distinguishes adjacent keys"
            );
            drop(cloned);
            drop(admitted);

            // Reuse this generated package, trimming it to a 2049-node prefix:
            // the second read is genuinely deep without constructing or reading
            // another 100000-node package.
            const DEEP_BAD_POSITION: usize = 2048;
            package["semantic_graph"]["nodes"]
                .as_array_mut()
                .expect("generated graph")
                .truncate(DEEP_BAD_POSITION + 1);
            package["source_map"]
                .as_array_mut()
                .expect("generated source map")
                .truncate(DEEP_BAD_POSITION + 1);
            package["semantic_graph"]["nodes"][DEEP_BAD_POSITION]["node_id"]["path"][2] =
                json!({"node": DEEP_BAD_POSITION + 1});
            let malformed = read_package_with_document_and_extensions_at_limit(
                &package,
                "NI-13 deep malformed index",
                &document,
                true,
                limits,
            )
            .expect("reader evidence");
            let CheckedPackageV2ReadResult::Refused(refusal) = malformed else {
                panic!("deep out-of-range index must refuse: {malformed:?}");
            };
            assert_eq!(code_word(refusal.code), "malformed_wire");
            assert!(refusal.cause.is_none(), "deep index has no cause");
            assert_eq!(
                refusal.loci[0].path.as_ref().map(ToString::to_string),
                Some(format!(
                    "/semantic_graph/operand_rows/{DEEP_BAD_POSITION}/1"
                )),
                "deep malformed operand points to its own row entry"
            );
        })
        .expect("small-stack reader thread")
        .join()
        .expect("small-stack reader did not panic");
}
