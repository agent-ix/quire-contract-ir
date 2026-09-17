// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-035/TC-044 complete-V1 lowering qualification against the current
//! `quire.checked-package/v2` reader: exact per-family lowering, refusal of
//! source/type/anchor/identity/bound/dependency/version mutations before any
//! backend artifact is emitted, and exact/one-over resource accounting.
//!
//! `CheckedPackage` V1 is gone (owner ruling 2026-09-17): this file targets
//! only the current `CheckedPackageV2` reader and lowerer.

#[path = "support/checked_package.rs"]
mod checked_package;

use checked_package::{
    canonical, evidence_for, incomplete, json_depth, refresh_identity, refusal, typed_node_id,
    v2_all_families, ALL_FAMILIES_READ_WORK, NODE_DOMAIN,
};
use ix_trace_rs::trace;
use quire_contract_ir::{
    CheckedNodeId, CheckedNodeTag, CheckedPackageEvidence, CheckedPackageLimit,
    CheckedPackageReadLimits, CheckedPackageRefusal, CheckedPackageRefusalCode, CheckedPackageV2,
    CheckedPackageV2ReadResult, CompleteContractNodeV2, CompleteLoweringProfileV2,
    CompleteLoweringRecordV2,
};
use serde_json::{json, Value};
use std::collections::BTreeSet;

/// Every V2 family in the all-families fixture, in graph order.
const FAMILIES: [CheckedNodeTag; 13] = [
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
    CheckedNodeTag::Correspondence,
];

/// The wire node identity recorded at `position` in the fixture's graph.
fn wire_node_id(value: &Value, position: usize) -> CheckedNodeId {
    serde_json::from_value(value["semantic_graph"]["nodes"][position]["node_id"].clone())
        .expect("node id")
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

fn refused(value: &Value, evidence: &CheckedPackageEvidence) -> CheckedPackageRefusal {
    refused_bytes(&canonical(value), evidence)
}

fn refused_bytes(bytes: &[u8], evidence: &CheckedPackageEvidence) -> CheckedPackageRefusal {
    match CheckedPackageV2::read(bytes, CheckedPackageReadLimits::bounded(), evidence) {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        other => panic!("expected V2 refusal, got {other:?}"),
    }
}

fn profile(work_limit: u64) -> CompleteLoweringProfileV2 {
    CompleteLoweringProfileV2 {
        supported_tags: FAMILIES.iter().copied().collect(),
        require_bounds: false,
        work_limit,
    }
}

fn lowered(record: &CompleteLoweringRecordV2) -> &CompleteContractNodeV2 {
    match record {
        CompleteLoweringRecordV2::Lowered { node } => node,
        other => panic!("expected lowered record, got {other:?}"),
    }
}

/// Tracing: TC-044, FR-035-AC-1, FR-035-AC-3, FR-035-AC-4
#[trace("TC-044", "FR-035-AC-1", "FR-035-AC-3", "FR-035-AC-4")]
#[test]
fn tc_044_reader_admits_and_lowers_every_public_node_family() {
    let value = v2_all_families();
    let package = admit(&value);
    assert_eq!(package.graph().nodes.len(), FAMILIES.len());

    let known_ids: BTreeSet<CheckedNodeId> = (0..FAMILIES.len())
        .map(|position| wire_node_id(&value, position))
        .collect();
    let requested = (0..FAMILIES.len())
        .map(|position| wire_node_id(&value, position))
        .collect::<Vec<_>>();
    let result = package.lower(&requested, &profile(u64::MAX));
    assert_eq!(result.records.len(), FAMILIES.len());
    for (position, (tag, record)) in FAMILIES.iter().zip(&result.records).enumerate() {
        let node = lowered(record);
        let wire = &value["semantic_graph"]["nodes"][position];

        // Every public family admits and lowers to an exact semantic vector:
        // the lowered node round-trips the wire node verbatim.
        assert_eq!(node.node_tag, *tag, "{position}");
        assert_eq!(
            node.node.node_id,
            wire_node_id(&value, position),
            "{position}"
        );
        assert_eq!(
            serde_json::to_value(&node.node).expect("node"),
            *wire,
            "{position}"
        );

        // Every reference resolves by stable identity to a reachable,
        // version-compatible node in the admitted graph.
        assert!(known_ids.contains(&node.semantic_type), "{position}");
        for dependency in &node.dependencies {
            assert!(known_ids.contains(dependency), "{position}");
        }

        // Every represented node has exact source correspondence.
        let source_map = value["source_map"]
            .as_array()
            .expect("source map")
            .iter()
            .filter(|entry| entry["node_id"] == wire["node_id"])
            .cloned()
            .collect::<Vec<_>>();
        assert!(!source_map.is_empty(), "{position}");
        assert_eq!(
            serde_json::to_value(&node.source_map).expect("source map"),
            Value::Array(source_map),
            "{position}"
        );
    }

    // Mixed supported, missing, and over-budget request: independent sibling
    // records, and no placeholder substituted for the ones that do not lower.
    let scalar_type = wire_node_id(&value, 0);
    let expression = wire_node_id(&value, 4);
    let missing = typed_node_id(&"9".repeat(64));
    let mixed = package.lower(
        &[scalar_type.clone(), missing.clone(), expression.clone()],
        &profile(4),
    );
    assert_eq!(mixed.records.len(), 3);
    assert_eq!(lowered(&mixed.records[0]).node.node_id, scalar_type);
    assert_eq!(
        mixed.records[1],
        CompleteLoweringRecordV2::InvalidInput { node_id: missing }
    );
    assert!(matches!(
        &mixed.records[2],
        CompleteLoweringRecordV2::Failed { node_id, limit: 4, consumed: 5, .. } if *node_id == expression
    ));

    // The successful sibling is exactly what an isolated request would
    // return: one sibling's disposition never leaks into another's.
    assert_eq!(
        mixed.records[0],
        package.lower(&[scalar_type], &profile(4)).records[0]
    );
}

/// Tracing: TC-044, FR-035-AC-2
#[trace("TC-044", "FR-035-AC-2")]
#[test]
fn tc_044_reader_refuses_strict_wire_and_identity_mutations() {
    let base = v2_all_families();
    let evidence = evidence_for(&base);
    let bytes = canonical(&base);
    let text = std::str::from_utf8(&bytes).expect("UTF-8 fixture");

    // Strict wire: a duplicated top-level member and a leading byte outside
    // the canonical form both refuse before any node is considered.
    assert_eq!(
        refused_bytes(
            format!(
                "{{\"contract_version\":\"quire.checked-package/v2\",{}",
                &text[1..]
            )
            .as_bytes(),
            &evidence,
        )
        .code,
        CheckedPackageRefusalCode::DuplicateMember
    );
    let mut spaced = bytes.clone();
    spaced.push(b'\n');
    assert_eq!(
        refused_bytes(&spaced, &evidence),
        refusal(CheckedPackageRefusalCode::NoncanonicalWire, "document")
    );

    // version: an unrecognised contract_version refuses before any node,
    // lock, or identity member is examined.
    let mut wrong_version = base.clone();
    wrong_version["contract_version"] = json!("quire.checked-package/v1");
    assert_eq!(
        refused(&wrong_version, &evidence).code,
        CheckedPackageRefusalCode::UnknownContractVersion
    );

    // identity: a tampered package digest refuses without admitting any node.
    let mut stale_identity = base.clone();
    stale_identity["package_id"]["digest"] = json!("0".repeat(64));
    assert_eq!(
        refused(&stale_identity, &evidence),
        refusal(
            CheckedPackageRefusalCode::StaleDependency,
            "package_id.digest"
        )
    );

    // source: an incomplete source map refuses instead of admitting a node
    // with no exact source correspondence.
    let mut missing_source = base.clone();
    missing_source["source_map"]
        .as_array_mut()
        .expect("source map")
        .pop();
    assert_eq!(
        refused(&missing_source, &evidence),
        refusal(CheckedPackageRefusalCode::InvalidSourceMap, "source_map")
    );

    // anchor: a source region rebound to an unlocked source refuses.
    let mut unlocked_anchor = base.clone();
    unlocked_anchor["source_map"][0]["regions"][0]["source"]["identity"] = json!("other");
    assert_eq!(
        refused(&unlocked_anchor, &evidence_for(&unlocked_anchor)),
        refusal(
            CheckedPackageRefusalCode::InvalidSourceMap,
            "source_map.regions.source",
        )
    );

    // dependency: a dependency naming a node outside the admitted graph
    // refuses rather than resolving to a substitute.
    let mut dangling_dependency = base.clone();
    dangling_dependency["semantic_graph"]["nodes"][1]["dependencies"] =
        json!([{"domain": NODE_DOMAIN, "digest": "9".repeat(64)}]);
    refresh_identity(&mut dangling_dependency);
    assert_eq!(
        refused(&dangling_dependency, &evidence_for(&dangling_dependency)),
        refusal(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            "semantic_graph.nodes.dependencies",
        )
    );

    // bound (a value's own type/target reference): a body reference naming a
    // node outside the admitted graph refuses.
    let mut dangling_target = base.clone();
    let expression = dangling_target["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_tag"] == json!("expression"))
        .expect("all-families fixture carries an expression node");
    dangling_target["semantic_graph"]["nodes"][expression]["body"]["target"]["digest"] =
        json!("9".repeat(64));
    refresh_identity(&mut dangling_target);
    assert_eq!(
        refused(&dangling_target, &evidence_for(&dangling_target)),
        refusal(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            "semantic_graph.nodes.body.target",
        )
    );

    // type: an unsupported node tag refuses rather than admitting an unknown
    // family into the graph.
    let mut unknown_tag = base.clone();
    unknown_tag["semantic_graph"]["nodes"][0]["node_tag"] = json!("future_node");
    assert_eq!(
        refused(&unknown_tag, &evidence),
        refusal(
            CheckedPackageRefusalCode::UnsupportedNodeTag,
            "semantic_graph.nodes.node_tag",
        )
    );

    // Every mutation above refuses: no case admits, so none reaches lowering
    // and none emits a backend artifact.
}

/// Tracing: TC-044, FR-035-AC-2
#[trace("TC-044", "FR-035-AC-2")]
#[test]
fn tc_044_reader_reports_exact_and_one_over_resource_accounting() {
    let value = v2_all_families();
    let evidence = evidence_for(&value);
    let bytes = canonical(&value);
    let mut exact = CheckedPackageReadLimits {
        bytes: u64::try_from(bytes.len()).expect("fixture length"),
        depth: json_depth(&value),
        nodes: u64::try_from(FAMILIES.len()).expect("node count"),
        edges: 0,
        occurrences: u64::try_from(FAMILIES.len() * 2).expect("occurrence count"),
        diagnostics: 0,
        work: ALL_FAMILIES_READ_WORK,
    };
    assert!(matches!(
        CheckedPackageV2::read(&bytes, exact, &evidence),
        CheckedPackageV2ReadResult::Admitted(_)
    ));

    exact.bytes -= 1;
    assert_incomplete(
        &bytes,
        exact,
        &evidence,
        CheckedPackageLimit::Bytes,
        exact.bytes,
    );
    exact.bytes += 1;

    exact.depth -= 1;
    assert_incomplete(
        &bytes,
        exact,
        &evidence,
        CheckedPackageLimit::Depth,
        exact.depth,
    );
    exact.depth += 1;

    exact.nodes -= 1;
    assert_incomplete(
        &bytes,
        exact,
        &evidence,
        CheckedPackageLimit::Nodes,
        exact.nodes,
    );
    exact.nodes += 1;

    exact.occurrences -= 1;
    assert_incomplete(
        &bytes,
        exact,
        &evidence,
        CheckedPackageLimit::Occurrences,
        exact.occurrences,
    );
    exact.occurrences += 1;

    exact.work -= 1;
    assert_incomplete(
        &bytes,
        exact,
        &evidence,
        CheckedPackageLimit::Work,
        exact.work,
    );

    // edges: the base fixture carries no dependency edges, so the exact/
    // one-over boundary is proven on a variant with exactly one.
    let mut edge_value = value.clone();
    edge_value["semantic_graph"]["nodes"][1]["dependencies"] =
        json!([value["semantic_graph"]["nodes"][0]["node_id"]]);
    refresh_identity(&mut edge_value);
    let edge_bytes = canonical(&edge_value);
    let edge_evidence = evidence_for(&edge_value);
    let mut edge_limits = CheckedPackageReadLimits::bounded();
    edge_limits.bytes = u64::try_from(edge_bytes.len()).expect("fixture length");
    edge_limits.edges = 1;
    assert!(matches!(
        CheckedPackageV2::read(&edge_bytes, edge_limits, &edge_evidence),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    edge_limits.edges = 0;
    assert_incomplete(
        &edge_bytes,
        edge_limits,
        &edge_evidence,
        CheckedPackageLimit::Edges,
        0,
    );

    // diagnostics: likewise, the boundary is proven on a variant with exactly
    // one diagnostic entry.
    let mut diagnostic_value = value;
    diagnostic_value["diagnostics"]["entries"] = json!([{
        "stage": "type_checking",
        "code": "ill_typed",
        "cause_tag": "invalid-value",
        "details": [],
        "loci": [],
    }]);
    let diagnostic_bytes = canonical(&diagnostic_value);
    let diagnostic_evidence = evidence_for(&diagnostic_value);
    let mut diagnostic_limits = CheckedPackageReadLimits::bounded();
    diagnostic_limits.bytes = u64::try_from(diagnostic_bytes.len()).expect("fixture length");
    diagnostic_limits.diagnostics = 1;
    assert!(matches!(
        CheckedPackageV2::read(&diagnostic_bytes, diagnostic_limits, &diagnostic_evidence),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    diagnostic_limits.diagnostics = 0;
    assert_incomplete(
        &diagnostic_bytes,
        diagnostic_limits,
        &diagnostic_evidence,
        CheckedPackageLimit::Diagnostics,
        0,
    );
}

fn assert_incomplete(
    bytes: &[u8],
    limits: CheckedPackageReadLimits,
    evidence: &CheckedPackageEvidence,
    kind: CheckedPackageLimit,
    limit: u64,
) {
    match CheckedPackageV2::read(bytes, limits, evidence) {
        CheckedPackageV2ReadResult::Incomplete(actual) => {
            assert_eq!(actual, incomplete(kind, limit, limit + 1), "{kind:?}");
        }
        other => panic!("{kind:?} one over must be incomplete, got {other:?}"),
    }
}
