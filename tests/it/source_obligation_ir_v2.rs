// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-052-AC-25: owner-native IR V2 source inventory.

use ix_trace_rs::trace;
use quire_contract_model::source_obligation::{
    source_inventory_from_ir_v2, SourceInventoryCompleteness, SourceInventoryIdentity,
    SourceInventoryIncompleteReason, SourceInventoryOwner, SourceInventoryRow,
    SourceInventoryScope, SourceIrV2InventoryError, SourceSemanticOccurrenceId,
};
use quire_contract_model::{
    CheckedPackageReadLimits, CheckedPackageRefusalCode, CheckedPackageV2,
    CheckedPackageV2ReadResult, SourceDocumentId, SourceIdentity, SourceRevision,
};
use serde_json::json;

use crate::support::checked_package::{canonical, evidence_for, refresh_identity, v2_all_families};

fn source() -> SourceIdentity {
    SourceIdentity::new(
        SourceDocumentId::new("selected-ir-v2").expect("document"),
        SourceRevision::new(1).expect("revision"),
    )
}

#[trace("TC-452", "FR-052-AC-25")]
#[test]
fn ir_v2_inventory_matches_every_admitted_graph_node_and_occurrence() {
    let mut value = v2_all_families();
    value["semantic_graph"]["nodes"][0]["occurrences"]
        .as_array_mut()
        .expect("occurrences")
        .push(json!({"role": "expression", "ordinal": 1}));
    let mut second_source = value["source_map"][0].clone();
    second_source["role"] = json!("expression");
    second_source["ordinal"] = json!(1);
    value["source_map"]
        .as_array_mut()
        .expect("source map")
        .push(second_source);
    let read = CheckedPackageV2::read(
        &canonical(&value),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&value),
    );
    let graph = match &read {
        CheckedPackageV2ReadResult::Admitted(package) => package.graph(),
        other => panic!("fixture must admit: {other:?}"),
    };
    let selected = source();
    assert_eq!(graph.nodes[0].occurrences.len(), 2);
    assert!(graph
        .nodes
        .iter()
        .any(|node| node.semantic_type != node.node_id));
    let expected: Vec<_> = graph
        .nodes
        .iter()
        .flat_map(|node| {
            std::iter::once(SourceInventoryRow::new(
                selected.clone(),
                SourceInventoryIdentity::SemanticNode {
                    identity: node.node_id.clone(),
                    semantic_type: node.semantic_type.clone(),
                },
            ))
            .chain(node.occurrences.iter().cloned().map(|occurrence| {
                SourceInventoryRow::new(
                    selected.clone(),
                    SourceInventoryIdentity::SemanticOccurrence(SourceSemanticOccurrenceId::new(
                        node.node_id.clone(),
                        occurrence,
                    )),
                )
            }))
        })
        .collect();
    assert!(
        expected.len() > graph.nodes.len(),
        "fixture has occurrences"
    );

    let outcome = source_inventory_from_ir_v2(selected.clone(), read).expect("native inventory");
    assert_eq!(outcome.source(), &selected);
    assert_eq!(outcome.owner(), SourceInventoryOwner::IrV2);
    assert_eq!(
        outcome.scope(),
        SourceInventoryScope::IrV2SemanticGraphAndOccurrences
    );
    assert_eq!(outcome.items(), expected);
    assert_eq!(outcome.observed_count(), expected.len());
    assert_eq!(
        outcome.completeness(),
        &SourceInventoryCompleteness::Complete
    );
}

#[trace("TC-452", "FR-052-AC-25")]
#[test]
fn ir_v2_inventory_never_completes_a_node_without_source_occurrences() {
    let mut value = v2_all_families();
    for node in value["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
    {
        node["occurrences"] = json!([]);
    }
    value["source_map"] = json!([]);
    refresh_identity(&mut value);
    let read = CheckedPackageV2::read(
        &canonical(&value),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&value),
    );
    assert!(matches!(
        &read,
        CheckedPackageV2ReadResult::Refused(refusal)
            if refusal.code == CheckedPackageRefusalCode::InvalidSourceMap
    ));
    assert!(matches!(
        source_inventory_from_ir_v2(source(), read),
        Err(SourceIrV2InventoryError::Refused(refusal))
            if refusal.code == CheckedPackageRefusalCode::InvalidSourceMap
    ));
}

#[trace("TC-452", "FR-052-AC-25")]
#[test]
fn ir_v2_inventory_preserves_reader_limit_and_refusal_without_completion() {
    let value = v2_all_families();
    let bytes = canonical(&value);
    let evidence = evidence_for(&value);
    let mut limits = CheckedPackageReadLimits::bounded();
    limits.nodes = 0;
    let read = CheckedPackageV2::read(&bytes, limits, &evidence);
    let original = match &read {
        CheckedPackageV2ReadResult::Incomplete(cause) => cause.clone(),
        other => panic!("zero node ceiling must stop: {other:?}"),
    };
    let outcome = source_inventory_from_ir_v2(source(), read).expect("typed incomplete outcome");
    assert!(outcome.items().is_empty());
    assert_eq!(outcome.observed_count(), 0);
    assert_eq!(
        outcome.completeness(),
        &SourceInventoryCompleteness::Incomplete(SourceInventoryIncompleteReason::IrV2ReadLimit(
            original
        ))
    );

    let mut malformed = value;
    malformed["contract_version"] = json!("other-version");
    let read = CheckedPackageV2::read(
        &canonical(&malformed),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&malformed),
    );
    assert!(matches!(
        source_inventory_from_ir_v2(source(), read),
        Err(SourceIrV2InventoryError::Refused(refusal))
            if refusal.code == CheckedPackageRefusalCode::UnknownContractVersion
    ));
}
