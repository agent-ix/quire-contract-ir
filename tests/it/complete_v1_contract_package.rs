// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-035-AC-5/TC-047: one complete-V1 lowering call emits a single canonical
//! cycle-free versioned `ContractPackage` holding every `lowered` node of the
//! call and nothing for any other disposition.

use crate::support::checked_package::{
    self, canonical, evidence_for, mint_ungrouped_structural_keys, settle, sha256_hex,
    structural_key, typed_node_id, v2_all_families,
};
use ix_trace_rs::trace;
use quire_contract_ir::{
    CheckedNodeId, CheckedNodeTag, CheckedPackageReadLimits, CheckedPackageV2,
    CheckedPackageV2ReadResult, CompleteContractPackageV2, CompleteLoweringProfileV2,
    CompleteLoweringRecordV2, CompleteLoweringResultV2, CONTRACT_PACKAGE_VERSION,
};
use serde_json::{json, Value};
use std::collections::BTreeSet;

/// The canonical bytes of a package lowered under the default limits.
fn bytes(package: &CompleteContractPackageV2) -> &[u8] {
    package.canonical_bytes().expect("the package is encoded")
}

fn id(prefix: &str) -> CheckedNodeId {
    if prefix == "dddd" {
        // The mixed fixture retypes this structural value at Integer. Its
        // expected key comes from the FR-092 preimage, independent of read.
        typed_node_id(&structural_key(
            "value",
            "literal",
            Some(&checked_package::family_key("a3a3")),
            &json!({"term": "aggregate", "members": []}),
        ))
    } else {
        typed_node_id(&checked_package::family_key(prefix))
    }
}

fn missing() -> CheckedNodeId {
    typed_node_id(&"0123456789abcdef".repeat(4))
}

fn admit(value: &Value) -> CheckedPackageV2 {
    match CheckedPackageV2::read(
        &canonical(value),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(value),
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("expected V2 admission, got {other:?}"),
    }
}

/// The fixture with `1010` and `dddd` typed at the unbounded `Integer` node
/// `a3a3`, bounded by `cccc` only for nodes that reach it: `dddd` does,
/// `1010` does not.
fn mixed_fixture() -> Value {
    let mut value = v2_all_families();
    let integer = checked_package::node_id(&checked_package::family_key("a3a3"));
    for node in value["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
    {
        if node["node_id"] == checked_package::node_id(&checked_package::family_key("1010")) {
            node["semantic_type"] = integer.clone();
            node["dependencies"] = json!([integer]);
        } else if node["node_id"] == checked_package::node_id(&checked_package::family_key("dddd"))
        {
            node["semantic_type"] = integer.clone();
            node["dependencies"] = json!([checked_package::node_id(&checked_package::family_key(
                "cccc"
            ))]);
        }
    }
    mint_ungrouped_structural_keys(&mut value);
    settle(&mut value);
    value
}

/// The ids in ascending key order, the order a package lists them in.
fn ascending(mut keys: Vec<CheckedNodeId>) -> Vec<CheckedNodeId> {
    keys.sort();
    keys
}

/// Every family but `correspondence`, with bounds required.
fn mixed_profile() -> CompleteLoweringProfileV2 {
    CompleteLoweringProfileV2 {
        supported_tags: [
            CheckedNodeTag::ScalarType,
            CheckedNodeTag::CompositeType,
            CheckedNodeTag::BoundedDomain,
            CheckedNodeTag::Value,
            CheckedNodeTag::Expression,
            CheckedNodeTag::Function,
            CheckedNodeTag::Model,
            CheckedNodeTag::Relation,
            CheckedNodeTag::State,
            CheckedNodeTag::Temporal,
            CheckedNodeTag::Protocol,
            CheckedNodeTag::Claim,
        ]
        .into_iter()
        .collect(),
        require_bounds: true,
        work_limit: u64::MAX,
    }
}

/// Supported (twice), requires-bound, unsupported and absent items.
fn mixed_request() -> Vec<CheckedNodeId> {
    vec![id("dddd"), id("1010"), id("7070"), missing(), id("dddd")]
}

fn lower_mixed(value: &Value) -> CompleteLoweringResultV2 {
    admit(value).lower(&mixed_request(), &mixed_profile())
}

fn record_key(record: &CompleteLoweringRecordV2) -> (&'static str, &CheckedNodeId) {
    match record {
        CompleteLoweringRecordV2::Lowered { node } => ("lowered", &node.node.node_id),
        CompleteLoweringRecordV2::Unsupported { node_id, .. } => ("unsupported", node_id),
        CompleteLoweringRecordV2::RequiresBound { node_id, .. } => ("requires_bound", node_id),
        CompleteLoweringRecordV2::InvalidInput { node_id } => ("invalid_input", node_id),
        CompleteLoweringRecordV2::InvalidBody { node_id, .. } => ("invalid_body", node_id),
        CompleteLoweringRecordV2::BodyIncomplete { node_id, .. } => ("body_incomplete", node_id),
        CompleteLoweringRecordV2::Failed { node_id, .. } => ("failed", node_id),
    }
}

/// Tracing: TC-047, FR-035-AC-5
#[trace("TC-047", "FR-035-AC-5")]
#[test]
fn tc_047_mixed_call_emits_one_package_of_exactly_its_lowered_nodes() {
    let admitted = admit(&mixed_fixture());
    let result = admitted.lower(&mixed_request(), &mixed_profile());
    let dispositions = result.records.iter().map(record_key).collect::<Vec<_>>();
    assert_eq!(
        dispositions,
        vec![
            ("lowered", &id("dddd")),
            ("requires_bound", &id("1010")),
            ("unsupported", &id("7070")),
            ("invalid_input", &missing()),
            ("lowered", &id("dddd")),
        ]
    );

    let package = &result.package;
    assert_eq!(package.version(), CONTRACT_PACKAGE_VERSION);
    assert_eq!(package.source_package_id(), admitted.package_id());

    // Every lowered node of the call, once, exactly as its record carries it.
    assert_eq!(package.lowered().len(), 1);
    assert_eq!(package.lowered()[0], *lowered(&result.records[0]));

    // The admitted nodes the lowered node reaches, and nothing else, so
    // every reference inside the package resolves inside it.
    let dependency_keys = package
        .dependencies()
        .iter()
        .map(|dependency| dependency.node.node_id.clone())
        .collect::<Vec<_>>();
    assert_eq!(dependency_keys, ascending(vec![id("a3a3"), id("cccc")]));
    assert_eq!(package.lowered()[0].dependencies, dependency_keys);
    for dependency in package.dependencies() {
        assert!(!dependency.source_map.is_empty());
        assert!(dependency
            .source_map
            .iter()
            .all(|entry| entry.node_id == dependency.node.node_id));
    }

    // No node, lowered or reached, for any refused request.
    let represented = package
        .lowered()
        .iter()
        .map(|node| node.node.node_id.clone())
        .chain(dependency_keys)
        .collect::<BTreeSet<_>>();
    for refused in [id("1010"), id("7070"), missing()] {
        assert!(!represented.contains(&refused), "{refused:?}");
    }
}

/// Tracing: TC-047, FR-035-AC-5
#[trace("TC-047", "FR-035-AC-5")]
#[test]
fn tc_047_package_bytes_are_canonical_and_stable_across_identical_calls() {
    let value = mixed_fixture();
    let first = lower_mixed(&value).package;
    let second = lower_mixed(&value).package;
    assert_eq!(bytes(&first), bytes(&second));
    assert_eq!(first.package_id(), second.package_id());

    // The identity is the digest of exactly these bytes, in its own domain.
    let identity = first.package_id().expect("the package is identified");
    assert_eq!(identity.domain.as_ref(), CONTRACT_PACKAGE_VERSION);
    assert_eq!(identity.algorithm.as_ref(), "sha256");
    assert_eq!(identity.digest.as_ref(), sha256_hex(bytes(&first)));

    // Canonical: re-encoding the decoded bytes reproduces them, and request
    // order or duplication does not reach the package.
    let decoded: Value = serde_json::from_slice(bytes(&first)).expect("json");
    assert_eq!(serde_json::to_vec(&decoded).expect("encode"), bytes(&first));
    assert_eq!(decoded["version"], json!(CONTRACT_PACKAGE_VERSION));
    let mut reordered = mixed_request();
    reordered.reverse();
    reordered.push(id("dddd"));
    let reordered = admit(&value).lower(&reordered, &mixed_profile()).package;
    assert_eq!(bytes(&reordered), bytes(&first));
}

/// Tracing: TC-047, FR-035-AC-5
#[trace("TC-047", "FR-035-AC-5")]
#[test]
fn tc_047_changing_any_represented_node_changes_bytes_and_digest() {
    let base = lower_mixed(&mixed_fixture()).package;

    // The lowered node's own source correspondence.
    let mut lowered_source = mixed_fixture();
    lowered_source["source_map"][3]["regions"][0]["end"] = json!(5);
    // A reached dependency's source correspondence.
    let mut dependency_source = mixed_fixture();
    dependency_source["source_map"][2]["regions"][0]["end"] = json!(4);

    for (label, value) in [
        ("lowered node source", lowered_source),
        ("dependency source", dependency_source),
    ] {
        let changed = lower_mixed(&value).package;
        assert_eq!(changed.lowered().len(), 1, "{label}");
        assert_ne!(bytes(&changed), bytes(&base), "{label}");
        assert_ne!(changed.package_id(), base.package_id(), "{label}");
        // The change is carried by the represented node itself, not only by
        // the source package identity the package also records.
        assert_ne!(
            without_source_identity(bytes(&changed)),
            without_source_identity(bytes(&base)),
            "{label}"
        );
    }
}

fn without_source_identity(bytes: &[u8]) -> Value {
    let mut value: Value = serde_json::from_slice(bytes).expect("json");
    value
        .as_object_mut()
        .expect("package object")
        .remove("source_package_id")
        .expect("source_package_id");
    value
}

/// Tracing: TC-047, FR-035-AC-5
#[trace("TC-047", "FR-035-AC-5")]
#[test]
fn tc_047_a_call_that_lowers_nothing_emits_an_empty_package() {
    let package = admit(&mixed_fixture())
        .lower(&[id("1010"), missing()], &mixed_profile())
        .package;
    assert!(package.lowered().is_empty());
    assert!(package.dependencies().is_empty());
    assert_eq!(package.version(), CONTRACT_PACKAGE_VERSION);
}

fn lowered(record: &CompleteLoweringRecordV2) -> &quire_contract_ir::CompleteContractNodeV2 {
    match record {
        CompleteLoweringRecordV2::Lowered { node } => node,
        other => panic!("expected lowered record, got {other:?}"),
    }
}

/// Tracing: TC-047, FR-035-AC-5
#[trace("TC-047", "FR-035-AC-5")]
#[test]
fn tc_047_a_lowered_node_another_lowered_node_reaches_is_held_once() {
    let package = admit(&mixed_fixture())
        .lower(&[id("dddd"), id("cccc")], &mixed_profile())
        .package;
    let lowered = package
        .lowered()
        .iter()
        .map(|node| node.node.node_id.clone())
        .collect::<Vec<_>>();
    let mut expected = vec![id("cccc"), id("dddd")];
    expected.sort();
    assert_eq!(lowered, expected);
    // `cccc` is reached by `dddd` but lowered in its own right, so it is not
    // repeated among the dependencies.
    let dependencies = package
        .dependencies()
        .iter()
        .map(|dependency| dependency.node.node_id.clone())
        .collect::<Vec<_>>();
    assert_eq!(dependencies, vec![id("a3a3")]);
}

/// Tracing: TC-047, FR-035-AC-5
#[trace("TC-047", "FR-035-AC-5")]
#[test]
fn tc_047_package_bytes_carry_every_member_of_every_represented_node() {
    let value = mixed_fixture();
    let result = lower_mixed(&value);
    let package = &result.package;
    let decoded: Value = serde_json::from_slice(bytes(package)).expect("json");
    let keys = |value: &Value| {
        value
            .as_object()
            .expect("object")
            .keys()
            .cloned()
            .collect::<Vec<_>>()
    };
    assert_eq!(
        keys(&decoded),
        ["dependencies", "lowered", "source_package_id", "version"]
    );
    let wire_node = |key: &CheckedNodeId| {
        value["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .find(|node| node["node_id"] == serde_json::to_value(key).expect("key"))
            .cloned()
            .expect("wire node")
    };

    let lowered = &decoded["lowered"][0];
    let node = &package.lowered()[0];
    assert_eq!(
        keys(lowered),
        [
            "bounds",
            "claims",
            "dependencies",
            "ir_id",
            "node",
            "node_tag",
            "semantic_type",
            "source_map"
        ]
    );
    assert_eq!(lowered["node"], wire_node(&node.node.node_id));
    assert_eq!(lowered["node_tag"], json!("value"));
    assert_eq!(lowered["dependencies"], json!(node.dependencies));
    assert_eq!(lowered["bounds"], json!([id("cccc")]));
    assert_eq!(lowered["claims"], json!(node.claims));
    assert_eq!(lowered["ir_id"], json!(node.ir_id));
    assert_eq!(lowered["source_map"], json!(node.source_map));

    for (position, dependency) in package.dependencies().iter().enumerate() {
        let encoded = &decoded["dependencies"][position];
        assert_eq!(keys(encoded), ["node", "node_tag", "source_map"]);
        assert_eq!(encoded["node"], wire_node(&dependency.node.node_id));
        assert_eq!(encoded["node_tag"], json!(dependency.node_tag.as_wire()));
        assert_eq!(encoded["source_map"], json!(dependency.source_map));
    }
}

/// Tracing: TC-047, FR-035-AC-5
#[trace("TC-047", "FR-035-AC-5")]
#[test]
fn tc_047_package_bytes_are_rfc_8785_key_ordered() {
    let package = lower_mixed(&mixed_fixture()).package;
    let bytes = std::str::from_utf8(bytes(&package)).expect("utf-8");
    assert!(bytes.starts_with("{\"dependencies\":["), "{bytes}");
    let positions = ["\"lowered\":", "\"source_package_id\":", "\"version\":"]
        .map(|member| bytes.rfind(member).expect(member));
    assert!(
        positions.windows(2).all(|pair| pair[0] < pair[1]),
        "{bytes}"
    );
    assert!(!bytes.contains(": ") && !bytes.contains(", "), "{bytes}");
}

/// Tracing: TC-047, FR-035-AC-5
#[trace("TC-047", "FR-035-AC-5")]
#[test]
fn tc_047_a_refused_request_is_represented_only_as_reached_meaning() {
    // `sequence` is an unbounded sequence of booleans and `bounds` a bounding
    // domain over it, both under their derived keys (FR-038-AC-134); `dddd`
    // is typed at the sequence and depends on both. Requested alone, the
    // sequence reaches no bounding domain and is refused; `dddd` reaches the
    // bound, so it lowers and carries the sequence as exact reached meaning,
    // never as a lowered node.
    let mut value = mixed_fixture();
    let sequence_body = checked_package::over_body(checked_package::BOOLEAN_KEY);
    let sequence_key =
        checked_package::structural_key("composite_type", "sequence", None, &sequence_body);
    let bounds_key = checked_package::structural_key(
        "bounded_domain",
        "collection_bounds",
        Some(&sequence_key),
        &checked_package::bounds_body("0", "3"),
    );
    let mut sequence = value["semantic_graph"]["nodes"][1].clone();
    sequence["node_id"] = checked_package::node_id(&sequence_key);
    sequence["semantic_form"] = json!("sequence");
    sequence["semantic_type"] = checked_package::node_id(&sequence_key);
    sequence["dependencies"] = json!([checked_package::node_id(checked_package::BOOLEAN_KEY)]);
    sequence["body"] = sequence_body;
    let mut bounds = value["semantic_graph"]["nodes"][2].clone();
    bounds["node_id"] = checked_package::node_id(&bounds_key);
    bounds["semantic_form"] = json!("collection_bounds");
    bounds["semantic_type"] = checked_package::node_id(&sequence_key);
    bounds["dependencies"] = json!([checked_package::node_id(&sequence_key)]);
    bounds["body"] = checked_package::bounds_body("0", "3");
    value["semantic_graph"]["nodes"][1] = sequence;
    value["semantic_graph"]["nodes"][2] = bounds;
    value["semantic_graph"]["nodes"][3]["semantic_type"] = checked_package::node_id(&sequence_key);
    let dependencies = ascending(vec![
        typed_node_id(&sequence_key),
        typed_node_id(&bounds_key),
    ]);
    value["semantic_graph"]["nodes"][3]["dependencies"] = json!(dependencies);
    let retyped_value = typed_node_id(&structural_key(
        "value",
        "literal",
        Some(&sequence_key),
        &json!({"term": "aggregate", "members": []}),
    ));
    mint_ungrouped_structural_keys(&mut value);
    settle(&mut value);
    let sequence_id = typed_node_id(&sequence_key);
    let request = vec![
        retyped_value.clone(),
        sequence_id.clone(),
        id("7070"),
        missing(),
        retyped_value.clone(),
    ];
    let result = admit(&value).lower(&request, &mixed_profile());
    let dispositions = result.records.iter().map(record_key).collect::<Vec<_>>();
    assert_eq!(dispositions[0], ("lowered", &retyped_value));
    assert_eq!(dispositions[1], ("requires_bound", &sequence_id));
    let package = &result.package;
    assert!(package
        .lowered()
        .iter()
        .all(|node| node.node.node_id != sequence_id));
    assert!(package
        .dependencies()
        .iter()
        .any(|dependency| dependency.node.node_id == sequence_id));
}
