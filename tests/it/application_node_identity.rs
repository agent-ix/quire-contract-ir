// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! External construction and admission of structural checked-package identities.

use ix_trace_rs::trace;
use quire_canonical::Limits;
use quire_contract_model::{
    encode_checked_package_v2, CheckedArtifactRef, CheckedDiagnosticsV2, CheckedNodeId,
    CheckedNodePathSegment, CheckedPackageEvidence, CheckedPackageLockV2, CheckedPackageReadLimits,
    CheckedPackageRefusalCode, CheckedPackageSourceCorrespondenceRef, CheckedPackageV2,
    CheckedPackageV2ReadResult, CheckedSemanticGraphV2, CheckedSourceMapEntry,
};
use serde_json::{json, Value};

fn structural_id() -> CheckedNodeId {
    CheckedNodeId {
        package: "example".into(),
        path: vec![
            CheckedNodePathSegment::Name("scalar_type".into()),
            CheckedNodePathSegment::Name("boolean".into()),
        ],
        ordinal: 0,
    }
}

fn envelope() -> (
    CheckedPackageLockV2,
    CheckedSemanticGraphV2<Value>,
    Vec<CheckedSourceMapEntry>,
    CheckedDiagnosticsV2,
) {
    let id = structural_id();
    let source = json!({"authority": "agent-ix", "identity": "source"});
    let lock = serde_json::from_value(json!({
        "sources": [source],
        "edition": {"role": "edition", "definition": {"authority": "agent-ix", "identity": "edition"}},
        "profile_selections": [],
        "definition_selections": [],
        "required_features": [],
        "dependency_selections": [],
    }))
    .expect("typed lock");
    let graph = serde_json::from_value(json!({
        "graph_version": "quire.checked-semantic-graph/v2",
        "nodes": [{
            "node_id": id,
            "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": "scalar_type",
            "semantic_form": "boolean",
            "semantic_type": id,
            "dependencies": [],
            "occurrences": [{"role": "generated", "ordinal": 0}],
            "body": {"term": "aggregate", "members": []},
        }],
    }))
    .expect("typed graph");
    let source_map = serde_json::from_value(json!([{
        "node_id": id,
        "role": "generated",
        "ordinal": 0,
        "regions": [{"source": source, "start": 0, "end": 1}],
    }]))
    .expect("typed source correspondence");
    let diagnostics = CheckedDiagnosticsV2 {
        catalog: CheckedArtifactRef {
            authority: "agent-ix".into(),
            identity: "diagnostics".into(),
        },
        entries: Vec::new(),
    };
    (lock, graph, source_map, diagnostics)
}

fn encode(
    lock: &CheckedPackageLockV2,
    graph: &CheckedSemanticGraphV2<Value>,
    source_map: &[CheckedSourceMapEntry],
    diagnostics: &CheckedDiagnosticsV2,
    limit: u64,
) -> Result<Vec<u8>, quire_canonical::Error> {
    encode_checked_package_v2(
        "example",
        lock,
        graph,
        CheckedPackageSourceCorrespondenceRef {
            source_map,
            refinement_step_rows: None,
        },
        &[],
        diagnostics,
        Limits::new(limit),
    )
}

/// A public node ID has the exact caller-assigned structural value, and only a
/// complete reader admission turns its containing envelope into a package.
#[trace("TC-048", "FR-046-AC-1", "FR-048-AC-1")]
#[test]
fn structural_identity_requires_complete_reader_admission() {
    let (lock, graph, source_map, diagnostics) = envelope();
    let expected = structural_id();
    assert_eq!(graph.nodes[0].node_id, expected);
    let bytes = encode(&lock, &graph, &source_map, &diagnostics, 1 << 20)
        .expect("complete envelope encodes");
    let admitted = CheckedPackageV2::read(
        &bytes,
        CheckedPackageReadLimits::bounded(),
        &CheckedPackageEvidence::new(),
    );
    let CheckedPackageV2ReadResult::Admitted(package) = admitted else {
        panic!("complete structural package must admit: {admitted:?}");
    };
    assert_eq!(package.graph().nodes[0].node_id, expected);
}

/// The public encoder uses the exact complete envelope byte ceiling.
#[trace("TC-048", "FR-046-AC-5", "FR-048-AC-5")]
#[test]
fn structural_envelope_has_an_exact_canonical_byte_boundary() {
    let (lock, graph, source_map, diagnostics) = envelope();
    let bytes = encode(&lock, &graph, &source_map, &diagnostics, 1 << 20)
        .expect("complete envelope encodes");
    let length = bytes.len() as u64;
    assert_eq!(
        encode(&lock, &graph, &source_map, &diagnostics, length).expect("exact byte limit"),
        bytes
    );
    assert!(matches!(
        encode(&lock, &graph, &source_map, &diagnostics, length - 1),
        Err(quire_canonical::Error::Limit(refusal))
            if refusal.kind == quire_canonical::LimitKind::CanonicalBytes
    ));
}

/// Repeated assigned IDs are rejected by the reader, with no admitted package.
#[trace("TC-048", "FR-046-AC-4", "FR-048-AC-4")]
#[test]
fn repeated_structural_node_id_refuses_without_a_package() {
    let (lock, mut graph, source_map, diagnostics) = envelope();
    graph.nodes.push(graph.nodes[0].clone());
    let bytes = encode(&lock, &graph, &source_map, &diagnostics, 1 << 20)
        .expect("duplicate graph remains encodable");
    let result = CheckedPackageV2::read(
        &bytes,
        CheckedPackageReadLimits::bounded(),
        &CheckedPackageEvidence::new(),
    );
    assert!(matches!(
        result,
        CheckedPackageV2ReadResult::Refused(ref refusal)
            if refusal.code == CheckedPackageRefusalCode::InvalidSemanticGraph
    ));
}
