// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Public V2 structural identity and canonical-byte controls over an authored
//! single-node fixture. The published QSpec readback runs in conformance_qspec.

use ix_trace_rs::trace;
use quire_canonical::{Error, LimitKind, Limits};
use quire_contract_model::{
    encode_checked_package_v2, CheckedPackageEvidence, CheckedPackageReadLimits,
    CheckedPackageSourceCorrespondenceRef, CheckedPackageV2, CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};

use crate::support::checked_package::v2_all_families;

fn local_package() -> Value {
    let mut package = v2_all_families();
    let scalar = package["semantic_graph"]["nodes"][0].clone();
    let scalar_id = scalar["node_id"].clone();
    package["semantic_graph"]["nodes"] = json!([scalar]);
    package["source_map"] = json!(package["source_map"]
        .as_array()
        .expect("source map")
        .iter()
        .filter(|entry| entry["node_id"] == scalar_id)
        .cloned()
        .collect::<Vec<_>>());
    package
}

fn admit(value: &Value) -> Box<CheckedPackageV2> {
    let mut evidence = CheckedPackageEvidence::new();
    evidence.support_feature("quire.value.complete/v1");
    let bytes = quire_canonical::to_vec(value, Limits::new(u64::MAX)).expect("fixture bytes");
    match CheckedPackageV2::read(&bytes, CheckedPackageReadLimits::bounded(), &evidence) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("structural package must admit, got {other:?}"),
    }
}

fn encode(package: &CheckedPackageV2, limit: u64) -> Result<Vec<u8>, Error> {
    encode_checked_package_v2(
        package.name(),
        package.lock(),
        package.graph(),
        CheckedPackageSourceCorrespondenceRef {
            source_map: package.source_map(),
            refinement_step_rows: package.refinement_step_rows(),
        },
        package.capability_report(),
        package.diagnostics(),
        Limits::new(limit),
    )
}

/// The independently authored path is not computed from a body or digest.
#[trace("TC-048", "FR-038-AC-89")]
#[test]
fn structural_id_and_complete_body_survive_reading() {
    let wire = local_package();
    let package = admit(&wire);
    let expected = json!({"package":"test/families","path":["builtin/boolean"],"ordinal":0});
    let nodes = wire["semantic_graph"]["nodes"].as_array().expect("nodes");
    assert_eq!(package.name(), "test/families");
    assert_eq!(package.graph().nodes.len(), 1);
    assert_eq!(nodes[0]["node_id"], expected);
    let retained = serde_json::to_value(&package.graph().nodes[0]).expect("retained node");
    assert_eq!(retained["node_id"], expected);
    assert_eq!(retained["body"], nodes[0]["body"]);
    assert_eq!(retained["owner"], nodes[0]["owner"]);
}

/// Canonical envelope bytes are checked through the public writer against
/// an independently constructed JSON value from the local fixture.
#[trace("TC-048", "FR-038-AC-91")]
#[test]
fn canonical_envelope_bytes_follow_semantic_and_source_edits() {
    let wire = local_package();
    let original = admit(&wire);
    let expected = quire_canonical::to_vec(&wire, Limits::new(u64::MAX)).expect("expected bytes");
    assert_eq!(encode(&original, u64::MAX).expect("writer bytes"), expected);

    let mut semantic = wire.clone();
    semantic["semantic_graph"]["nodes"][0]["semantic_form"] = json!("integer");
    let semantic_package = admit(&semantic);
    let semantic_bytes = encode(&semantic_package, u64::MAX).expect("semantic bytes");
    assert_ne!(semantic_bytes, expected);
    assert_eq!(semantic_package.source_map(), original.source_map());
    assert_eq!(semantic_package.name(), original.name());

    let mut source = wire;
    source["source_map"][0]["regions"][0]["start"] = json!(100);
    source["source_map"][0]["regions"][0]["end"] = json!(101);
    let source_package = admit(&source);
    let source_bytes = encode(&source_package, u64::MAX).expect("source bytes");
    assert_ne!(source_bytes, expected);
    assert_ne!(source_bytes, semantic_bytes);
    assert_eq!(source_package.graph().nodes, original.graph().nodes);
}

/// The typed node and complete envelope both admit at their exact canonical
/// byte ceilings and fail one byte below without yielding partial bytes.
#[trace("TC-048", "FR-038-AC-95")]
#[test]
fn typed_node_and_envelope_encoding_have_exact_byte_limits() {
    let package = admit(&local_package());
    let node = &package.graph().nodes[0];
    let node_bytes = quire_canonical::to_vec(node, Limits::new(u64::MAX)).expect("node bytes");
    assert_eq!(
        quire_canonical::to_vec(node, Limits::new(node_bytes.len() as u64))
            .expect("exact node ceiling"),
        node_bytes
    );
    assert!(matches!(
        quire_canonical::to_vec(node, Limits::new(node_bytes.len() as u64 - 1)),
        Err(Error::Limit(limit)) if limit.kind == LimitKind::CanonicalBytes
    ));

    let bytes = encode(&package, u64::MAX).expect("envelope bytes");
    assert_eq!(
        encode(&package, bytes.len() as u64).expect("exact ceiling"),
        bytes
    );
    assert!(matches!(
        encode(&package, bytes.len() as u64 - 1),
        Err(Error::Limit(limit)) if limit.kind == LimitKind::CanonicalBytes
    ));
}
