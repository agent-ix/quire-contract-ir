// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-035-AC-5/TC-047: one complete-V1 lowering call emits a single canonical
//! cycle-free versioned `ContractPackage` holding every `lowered` node of the
//! call and nothing for any other disposition.

use crate::support::checked_package::{
    self, canonical, evidence_for, refresh_identity, sha256_hex, typed_node_id, v2_all_families,
};
use ix_trace_rs::trace;
use quire_contract_ir::{
    CheckedNodeId, CheckedNodeTag, CheckedPackageReadLimits, CheckedPackageV2,
    CheckedPackageV2ReadResult, CompleteLoweringProfileV2, CompleteLoweringRecordV2,
    CompleteLoweringResultV2, CONTRACT_PACKAGE_VERSION,
};
use serde_json::{json, Value};
use std::collections::BTreeSet;

fn id(prefix: &str) -> CheckedNodeId {
    typed_node_id(&checked_package::family_key(prefix))
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

/// The fixture with an unbounded `integer` scalar at `aaaa`, bounded by
/// `cccc` only for nodes that reach it: `dddd` does, `bbbb` does not.
fn mixed_fixture() -> Value {
    let mut value = v2_all_families();
    value["semantic_graph"]["nodes"][0]["semantic_form"] = json!("integer");
    value["semantic_graph"]["nodes"][3]["dependencies"] =
        json!([value["semantic_graph"]["nodes"][2]["node_id"]]);
    refresh_identity(&mut value);
    value
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
    vec![id("dddd"), id("bbbb"), id("7070"), missing(), id("dddd")]
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
            ("requires_bound", &id("bbbb")),
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
    assert_eq!(dependency_keys, vec![id("aaaa"), id("cccc")]);
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
    for refused in [id("bbbb"), id("7070"), missing()] {
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
    assert_eq!(first.canonical_bytes(), second.canonical_bytes());
    assert_eq!(first.package_id(), second.package_id());

    // The identity is the digest of exactly these bytes, in its own domain.
    let identity = first.package_id();
    assert_eq!(identity.domain.as_ref(), CONTRACT_PACKAGE_VERSION);
    assert_eq!(identity.algorithm.as_ref(), "sha256");
    assert_eq!(
        identity.digest.as_ref(),
        sha256_hex(first.canonical_bytes())
    );

    // Canonical: re-encoding the decoded bytes reproduces them, and request
    // order or duplication does not reach the package.
    let decoded: Value = serde_json::from_slice(first.canonical_bytes()).expect("json");
    assert_eq!(
        serde_json::to_vec(&decoded).expect("encode"),
        first.canonical_bytes()
    );
    assert_eq!(decoded["version"], json!(CONTRACT_PACKAGE_VERSION));
    let mut reordered = mixed_request();
    reordered.reverse();
    reordered.push(id("dddd"));
    let reordered = admit(&value).lower(&reordered, &mixed_profile()).package;
    assert_eq!(reordered.canonical_bytes(), first.canonical_bytes());
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
        assert_ne!(changed.canonical_bytes(), base.canonical_bytes(), "{label}");
        assert_ne!(changed.package_id(), base.package_id(), "{label}");
        // The change is carried by the represented node itself, not only by
        // the source package identity the package also records.
        assert_ne!(
            without_source_identity(changed.canonical_bytes()),
            without_source_identity(base.canonical_bytes()),
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
        .lower(&[id("bbbb"), missing()], &mixed_profile())
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
