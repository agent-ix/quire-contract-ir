// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! External default-feature consumer of the typed V2 envelope encoder.

use crate::support::checked_package::{
    admitted_protocol_families, canonical, evidence_for, selected_interface_model_node, settle,
    v2_all_families, v2_nominal,
};
use ix_trace_rs::trace;
use quire_canonical::{Error, Limits};
use quire_contract_model::{
    encode_checked_package_v2, CheckedCapability, CheckedDiagnosticsV2, CheckedNodeProjectionV2,
    CheckedPackageIdentityPreimageRefV2, CheckedPackageIdentityPreimageV2, CheckedPackageLockV2,
    CheckedPackageReadLimits, CheckedPackageRefusalCode, CheckedPackageV2,
    CheckedPackageV2ReadResult, CheckedSemanticGraphV2, CheckedSemanticId, CheckedSemanticNodeV2,
    CheckedSourceMapEntry, CHECKED_PACKAGE_V2, PACKAGE_DOMAIN_V2,
};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

fn typed<T: DeserializeOwned>(value: &Value) -> T {
    serde_json::from_value(value.clone()).expect("typed fixture member")
}

fn borrowed_preimage<'a, Body>(
    owned: &'a CheckedPackageIdentityPreimageV2,
    nodes: &'a [CheckedSemanticNodeV2<Body>],
) -> CheckedPackageIdentityPreimageRefV2<'a, Body> {
    CheckedPackageIdentityPreimageRefV2 {
        version: &owned.version,
        edition: &owned.edition,
        profile_selections: &owned.profile_selections,
        definition_selections: &owned.definition_selections,
        model_selections: &owned.model_selections,
        required_features: &owned.required_features,
        dependency_selections: &owned.dependency_selections,
        identity_projection: nodes,
    }
}

fn hash_input(value: &Value) -> Value {
    let mut input = value["identity_preimage"].clone();
    input["identity_projection"] = Value::Array(
        value["semantic_graph"]["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|node| {
                let mut node = node.clone();
                node.as_object_mut().unwrap().remove("occurrences");
                node
            })
            .collect(),
    );
    input
}

fn encode(value: &Value, limits: Limits) -> Result<Vec<u8>, Error> {
    let preimage: CheckedPackageIdentityPreimageV2 = typed(&value["identity_preimage"]);
    let package_id: CheckedSemanticId = typed(&value["package_id"]);
    let lock: CheckedPackageLockV2 = typed(&value["lock"]);
    let graph: CheckedSemanticGraphV2<Value> = typed(&value["semantic_graph"]);
    let source_map: Vec<CheckedSourceMapEntry> = typed(&value["source_map"]);
    let capabilities: Vec<CheckedCapability> = typed(&value["capability_report"]);
    let diagnostics: CheckedDiagnosticsV2 = typed(&value["diagnostics"]);
    let preimage = borrowed_preimage(&preimage, &graph.nodes);
    encode_checked_package_v2(
        &preimage,
        &package_id,
        &lock,
        &graph,
        quire_contract_model::CheckedPackageSourceCorrespondenceRef {
            source_map: &source_map,
            refinement_step_rows: None,
        },
        &capabilities,
        &diagnostics,
        limits,
    )
}

fn refused(
    bytes: &[u8],
    evidence: &quire_contract_model::CheckedPackageEvidence,
) -> CheckedPackageRefusalCode {
    match CheckedPackageV2::read(bytes, CheckedPackageReadLimits::bounded(), evidence) {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal.code,
        other => panic!("expected refusal, got {other:?}"),
    }
}

/// Trace: FR-064-AC-1, FR-064-AC-6
#[trace("TC-946")]
#[test]
fn typed_writer_round_trips_protocol_clause_owner_model() {
    let mut value = admitted_protocol_families();
    value["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(selected_interface_model_node());
    settle(&mut value);
    let evidence = evidence_for(&value);
    let CheckedPackageV2ReadResult::Admitted(first) = CheckedPackageV2::read(
        &canonical(&value),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ) else {
        panic!("owner-bearing fixture must admit");
    };
    let clause = first
        .graph()
        .nodes
        .iter()
        .find(|node| {
            node.node_tag.as_ref() == "protocol" && node.semantic_form.as_ref() == "protocol_clause"
        })
        .expect("protocol clause");
    let owner = clause.owner_model.as_ref().expect("exact owner edge");
    assert!(clause.dependencies.contains(owner));
    let projection = CheckedNodeProjectionV2::from(clause);
    assert_eq!(projection.owner_model.as_ref(), Some(owner));

    let bytes = encode(&value, Limits::new(u64::MAX)).expect("typed writer");
    let written: Value = serde_json::from_slice(&bytes).expect("encoded JSON");
    let written_clause = written["identity_preimage"]["identity_projection"]
        .as_array()
        .expect("projection")
        .iter()
        .find(|node| node["node_id"] == serde_json::to_value(&clause.node_id).expect("id"))
        .expect("projected clause");
    assert_eq!(
        written_clause["owner_model"],
        serde_json::to_value(owner).expect("owner")
    );
    let CheckedPackageV2ReadResult::Admitted(second) =
        CheckedPackageV2::read(&bytes, CheckedPackageReadLimits::bounded(), &evidence)
    else {
        panic!("encoded owner-bearing fixture must admit");
    };
    assert_eq!(second.package_id(), first.package_id());
    assert_eq!(second.graph().nodes, first.graph().nodes);
}

#[trace("TC-451", "FR-051-AC-4")]
#[test]
fn tc_451_borrowed_projection_matches_owned_for_empty_nested_and_optional_cases() {
    fn assert_same(value: &Value) {
        let owned: CheckedPackageIdentityPreimageV2 = typed(&value["identity_preimage"]);
        let graph: CheckedSemanticGraphV2<Value> = typed(&value["semantic_graph"]);
        let borrowed = borrowed_preimage(&owned, &graph.nodes);
        let limits = Limits::new(u64::MAX);
        let expected_input = hash_input(value);
        let owned_bytes =
            quire_canonical::to_vec(&expected_input, limits).expect("independent hash input");
        assert_eq!(
            quire_canonical::to_vec(&borrowed, limits).expect("borrowed preimage"),
            owned_bytes
        );
        let exact = u64::try_from(owned_bytes.len()).expect("preimage length fits u64");
        assert_eq!(
            quire_canonical::to_vec(&borrowed, Limits::new(exact)).expect("borrowed exact limit"),
            owned_bytes
        );
        assert_eq!(
            quire_canonical::to_vec(&expected_input, Limits::new(exact))
                .expect("owned exact limit"),
            owned_bytes
        );
        assert!(matches!(
            quire_canonical::to_vec(&borrowed, Limits::new(exact - 1)),
            Err(Error::Limit(_))
        ));
        assert!(matches!(
            quire_canonical::to_vec(&expected_input, Limits::new(exact - 1)),
            Err(Error::Limit(_))
        ));
        assert_eq!(
            quire_canonical::sha256(&borrowed, limits).expect("borrowed identity"),
            quire_canonical::sha256(&expected_input, limits).expect("owned identity")
        );
        assert_eq!(
            encode(value, limits).expect("borrowed envelope"),
            canonical(value)
        );
    }

    let value = v2_all_families();
    let graph: CheckedSemanticGraphV2<Value> = typed(&value["semantic_graph"]);
    assert!(graph.nodes.iter().any(|node| !node.occurrences.is_empty()));
    assert!(graph.nodes.iter().any(|node| node.declaration.is_some()));
    assert!(graph.nodes.iter().any(|node| node.owner.is_some()));
    assert_same(&value);
    let nominal = v2_nominal();
    let nominal_graph: CheckedSemanticGraphV2 = typed(&nominal["semantic_graph"]);
    assert!(nominal_graph
        .nodes
        .iter()
        .any(|node| node.nominal_identity_preimage.is_some()));
    assert_same(&nominal);

    let mut grouped = value.clone();
    grouped["semantic_graph"]["nodes"][0]["recursion_group"] = json!("group");
    assert_same(&grouped);

    let owned: CheckedPackageIdentityPreimageV2 = typed(&value["identity_preimage"]);
    let mut without_occurrences = graph.clone();
    for node in &mut without_occurrences.nodes {
        node.occurrences.clear();
    }
    assert_eq!(
        quire_canonical::to_vec(
            &borrowed_preimage(&owned, &graph.nodes),
            Limits::new(u64::MAX)
        )
        .expect("preimage with occurrences"),
        quire_canonical::to_vec(
            &borrowed_preimage(&owned, &without_occurrences.nodes),
            Limits::new(u64::MAX)
        )
        .expect("preimage without occurrences")
    );

    let mut absent = value.clone();
    for node in absent["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("graph nodes")
    {
        let object = node.as_object_mut().expect("node object");
        for field in [
            "recursion_group",
            "nominal_identity_preimage",
            "declaration",
            "owner",
        ] {
            object.remove(field);
        }
    }
    assert_same(&absent);

    let mut nested = value.clone();
    let body = json!({"é😀": [null, {"nested": [true, -9_007_199_254_740_992_i64]}]});
    nested["semantic_graph"]["nodes"][0]["body"] = body.clone();
    assert_same(&nested);

    let mut empty_body = value.clone();
    empty_body["semantic_graph"]["nodes"][0]["body"] = json!({});
    assert_same(&empty_body);

    let mut empty = value;
    empty["semantic_graph"]["nodes"] = json!([]);
    assert_same(&empty);
}

#[trace("TC-451", "FR-051-AC-4")]
#[test]
fn tc_451_borrowed_and_owned_projection_refuse_the_same_invalid_body() {
    let mut value = v2_all_families();
    let too_large = json!(9_007_199_254_740_993_u64);
    value["semantic_graph"]["nodes"][0]["body"] = too_large.clone();
    let owned: CheckedPackageIdentityPreimageV2 = typed(&value["identity_preimage"]);
    let graph: CheckedSemanticGraphV2<Value> = typed(&value["semantic_graph"]);
    let borrowed = borrowed_preimage(&owned, &graph.nodes);
    assert!(matches!(
        quire_canonical::to_vec(&borrowed, Limits::new(u64::MAX)),
        Err(Error::IntegerMagnitudeAboveMaximum(9_007_199_254_740_993))
    ));
    assert!(matches!(
        quire_canonical::to_vec(&hash_input(&value), Limits::new(u64::MAX)),
        Err(Error::IntegerMagnitudeAboveMaximum(9_007_199_254_740_993))
    ));
    assert!(matches!(
        encode(&value, Limits::new(u64::MAX)),
        Err(Error::IntegerMagnitudeAboveMaximum(9_007_199_254_740_993))
    ));
}

#[trace("TC-451", "FR-051-AC-3", "FR-051-AC-4")]
#[test]
fn tc_451_borrowed_projection_can_differ_from_the_supplied_graph() {
    let mut value = v2_all_families();
    let original: CheckedPackageIdentityPreimageV2 = typed(&value["identity_preimage"]);
    let graph: CheckedSemanticGraphV2 = typed(&value["semantic_graph"]);
    let original_digest = quire_canonical::sha256(
        &borrowed_preimage(&original, &graph.nodes),
        Limits::new(u64::MAX),
    )
    .expect("original package identity")
    .to_string();
    value["package_id"] = json!({
        "domain": PACKAGE_DOMAIN_V2, "algorithm": "sha256", "digest": original_digest,
    });
    let evidence = evidence_for(&value);

    let mut independent_nodes = graph.nodes.clone();
    let changed_body = json!({"term":"reference","target":independent_nodes[0].node_id});
    independent_nodes[0].body =
        serde_json::from_value(changed_body.clone()).expect("typed changed body");
    let borrowed = borrowed_preimage(&original, &independent_nodes);
    let different_digest = quire_canonical::sha256(&borrowed, Limits::new(u64::MAX))
        .unwrap()
        .to_string();
    assert_ne!(different_digest, value["package_id"]["digest"]);
    value["package_id"]["digest"] = json!(different_digest);

    let package_id: CheckedSemanticId = typed(&value["package_id"]);
    let lock: CheckedPackageLockV2 = typed(&value["lock"]);
    let source_map: Vec<CheckedSourceMapEntry> = typed(&value["source_map"]);
    let capabilities: Vec<CheckedCapability> = typed(&value["capability_report"]);
    let diagnostics: CheckedDiagnosticsV2 = typed(&value["diagnostics"]);
    let bytes = encode_checked_package_v2(
        &borrowed,
        &package_id,
        &lock,
        &graph,
        quire_contract_model::CheckedPackageSourceCorrespondenceRef {
            source_map: &source_map,
            refinement_step_rows: None,
        },
        &capabilities,
        &diagnostics,
        Limits::new(u64::MAX),
    )
    .expect("independent projection encodes");
    let expected = value.clone();
    assert_eq!(bytes, canonical(&expected));
    let encoded: Value = serde_json::from_slice(&bytes).expect("encoded envelope");
    assert_eq!(encoded["semantic_graph"], value["semantic_graph"]);
    assert_eq!(encoded["identity_preimage"], value["identity_preimage"]);
    assert_eq!(
        refused(&bytes, &evidence),
        CheckedPackageRefusalCode::StaleDependency
    );
}

#[trace("TC-451", "FR-051-AC-1", "FR-051-AC-2", "FR-051-AC-3", "FR-051-AC-4")]
#[test]
fn tc_451_typed_envelope_has_exact_members_and_canonical_boundaries() {
    let value = v2_all_families();
    let bytes = encode(&value, Limits::new(u64::MAX)).expect("valid typed members encode");
    assert_eq!(bytes, canonical(&value));
    let encoded: Value = serde_json::from_slice(&bytes).expect("JSON object");
    let members = encoded.as_object().expect("envelope object");
    assert_eq!(members.len(), 8);
    assert_eq!(members["contract_version"], CHECKED_PACKAGE_V2);
    assert_eq!(
        members.keys().map(String::as_str).collect::<Vec<_>>(),
        [
            "capability_report",
            "contract_version",
            "diagnostics",
            "identity_preimage",
            "lock",
            "package_id",
            "semantic_graph",
            "source_map"
        ]
    );
    let length = u64::try_from(bytes.len()).expect("byte length fits u64");
    assert_eq!(
        encode(&value, Limits::new(length)).expect("exact ceiling"),
        bytes
    );
    assert!(matches!(
        encode(&value, Limits::new(length - 1)),
        Err(Error::Limit(_))
    ));

    let mut invalid = value;
    invalid["diagnostics"]["entries"] = json!([{
        "stage": "package_read", "code": "invalid_package", "cause_tag": "invalid-value",
        "details": [9_007_199_254_740_993_u64], "loci": []
    }]);
    assert!(matches!(
        encode(&invalid, Limits::new(u64::MAX)),
        Err(Error::IntegerMagnitudeAboveMaximum(9_007_199_254_740_993))
    ));
}

#[trace("TC-451", "FR-051-AC-3")]
#[test]
fn tc_451_reader_admits_matching_typed_envelope_and_refuses_mutations() {
    let mut value = v2_all_families();
    let preimage: CheckedPackageIdentityPreimageV2 = typed(&value["identity_preimage"]);
    let graph: CheckedSemanticGraphV2 = typed(&value["semantic_graph"]);
    let borrowed = borrowed_preimage(&preimage, &graph.nodes);
    let digest = quire_canonical::sha256(&borrowed, Limits::new(u64::MAX))
        .expect("preimage canonical")
        .to_string();
    value["package_id"] = json!({
        "domain": PACKAGE_DOMAIN_V2, "algorithm": "sha256", "digest": digest,
    });
    let evidence = evidence_for(&value);
    let bytes = encode(&value, Limits::new(u64::MAX)).expect("typed encode");
    assert!(matches!(
        CheckedPackageV2::read(&bytes, CheckedPackageReadLimits::bounded(), &evidence),
        CheckedPackageV2ReadResult::Admitted(_)
    ));

    let mut stale_id = value.clone();
    stale_id["package_id"]["digest"] = json!("0".repeat(64));
    let id_bytes = encode(&stale_id, Limits::new(u64::MAX)).expect("stale identity still encodes");
    assert_eq!(
        serde_json::from_slice::<Value>(&id_bytes).expect("JSON")["package_id"],
        stale_id["package_id"]
    );
    assert_eq!(
        refused(&id_bytes, &evidence),
        CheckedPackageRefusalCode::StaleDependency
    );

    let mut stale_preimage = value.clone();
    stale_preimage["identity_preimage"]["required_features"] = json!(["changed-feature"]);
    let preimage_bytes =
        encode(&stale_preimage, Limits::new(u64::MAX)).expect("stale preimage still encodes");
    assert_eq!(
        serde_json::from_slice::<Value>(&preimage_bytes).expect("JSON")["identity_preimage"],
        stale_preimage["identity_preimage"]
    );
    assert_eq!(
        refused(&preimage_bytes, &evidence),
        CheckedPackageRefusalCode::StaleDependency
    );

    let mut missing = value.clone();
    missing
        .as_object_mut()
        .expect("object")
        .remove("diagnostics");
    assert_eq!(
        refused(&canonical(&missing), &evidence),
        CheckedPackageRefusalCode::MalformedWire
    );
    let mut unknown = value;
    unknown["future"] = json!(true);
    assert_eq!(
        refused(&canonical(&unknown), &evidence),
        CheckedPackageRefusalCode::UnknownMember
    );
    let text = std::str::from_utf8(&bytes).expect("UTF-8");
    let duplicate = format!(
        "{{\"contract_version\":\"{CHECKED_PACKAGE_V2}\",{}",
        &text[1..]
    );
    assert_eq!(
        refused(duplicate.as_bytes(), &evidence),
        CheckedPackageRefusalCode::DuplicateMember
    );
}
