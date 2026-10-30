// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Public V2 structural identity and canonical-byte controls. The positive
//! fixture is read from the authoritative QSpec checkout at test time.

use ix_trace_rs::trace;
use quire_canonical::{Error, LimitKind, Limits};
use quire_contract_model::{
    encode_checked_package_v2, CheckedPackageEvidence, CheckedPackageReadLimits,
    CheckedPackageSourceCorrespondenceRef, CheckedPackageV2, CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

fn qspec_nominal() -> Value {
    let root = std::env::var_os("QUIRE_SPECIFICATION_DIR")
        .expect("QUIRE_SPECIFICATION_DIR is required for QSpec conformance");
    let path = Path::new(&root)
        .join("proposals/checked-package-v2/fixtures/positive-nominal-identities.json");
    serde_json::from_slice(&fs::read(path).expect("authoritative QSpec nominal fixture"))
        .expect("QSpec fixture JSON")
}

fn admit(value: &Value) -> Box<CheckedPackageV2> {
    let mut evidence = CheckedPackageEvidence::new();
    evidence.support_feature("quire.value.complete/v1");
    let bytes = quire_canonical::to_vec(value, Limits::new(u64::MAX)).expect("fixture bytes");
    match CheckedPackageV2::read(&bytes, CheckedPackageReadLimits::bounded(), &evidence) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("QSpec positive package must admit, got {other:?}"),
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

/// The independent expected paths come from the QSpec declarations and the
/// published structural path grammar, not from an identity helper in IR.
#[trace("TC-048", "FR-038-AC-89")]
#[test]
fn qspec_declared_and_derived_ids_and_complete_bodies_survive_reading() {
    let wire = qspec_nominal();
    let package = admit(&wire);
    let name = wire["name"].as_str().expect("package name");
    let id = |path: Vec<Value>| json!({"package":name,"path":path,"ordinal":0});
    let status = id(vec![json!("Example"), json!("Status")]);
    let length = id(vec![json!("Example"), json!("Length")]);
    let distance = id(vec![json!("Example"), json!("Distance")]);
    let expected = [
        id(vec![
            json!("value"),
            json!("enum_value"),
            status.clone(),
            json!("READY"),
        ]),
        status,
        id(vec![json!("Example"), json!("metre")]),
        length.clone(),
        distance.clone(),
        id(vec![
            json!("scalar_type"),
            json!("dimension_equivalence"),
            distance,
            length,
        ]),
    ];
    let nodes = wire["semantic_graph"]["nodes"]
        .as_array()
        .expect("QSpec nodes");
    assert_eq!(package.name(), name);
    assert_eq!(package.graph().nodes.len(), expected.len());
    for ((retained, source), expected_id) in package.graph().nodes.iter().zip(nodes).zip(&expected)
    {
        assert_eq!(
            serde_json::to_value(&retained.node_id).expect("id"),
            *expected_id
        );
        assert_eq!(source["node_id"], *expected_id);
        let retained = serde_json::to_value(retained).expect("retained node");
        assert_eq!(retained["body"], source["body"]);
        assert_eq!(retained["owner"], source["owner"]);
    }
}

/// Canonical envelope bytes are checked through the public writer against
/// an independently constructed JSON value from the QSpec fixture.
#[trace("TC-048", "FR-038-AC-91")]
#[test]
fn canonical_envelope_bytes_follow_semantic_and_source_edits() {
    let wire = qspec_nominal();
    let original = admit(&wire);
    let expected = quire_canonical::to_vec(&wire, Limits::new(u64::MAX)).expect("expected bytes");
    assert_eq!(encode(&original, u64::MAX).expect("writer bytes"), expected);

    let mut semantic = wire.clone();
    semantic["semantic_graph"]["nodes"][1]["body"]["ordered"] = json!(true);
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
    let package = admit(&qspec_nominal());
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
