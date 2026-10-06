// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Owner-wire admission and refusal cases built from this crate's public wire vocabulary.

use crate::support::checked_package::{
    canonical, evidence_for, node_id, refresh_identity, sha256_hex, v2_nominal,
};
use ix_trace_rs::trace;
use quire_contract_ir::{
    CheckedPackageReadLimits, CheckedPackageRefusalCause, CheckedPackageRefusalCode,
    CheckedPackageV2, CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};

fn source_package() -> Value {
    let mut package = v2_nominal();
    let id = node_id(&"f".repeat(64));
    let source = package["lock"]["sources"][0].clone();
    let owner = json!({
        "kind": "source", "authority": source["authority"], "identity": source["identity"],
    });
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(json!({
            "node_id": id,
            "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": "composite_type", "semantic_form": "record",
            "semantic_type": id, "dependencies": [],
            "occurrences": [{"role": "declaration", "ordinal": 0}],
            "declaration": {"qualified_name": ["Example", "Point"]},
            "owner": owner,
            "body": {"term": "aggregate", "members": []},
        }));
    package["source_map"]
        .as_array_mut()
        .expect("source map")
        .push(json!({
            "node_id": id, "role": "declaration", "ordinal": 0,
            "regions": [{"source": source, "start": 9, "end": 10}],
        }));
    refresh_identity(&mut package);
    package
}

fn read(package: &Value) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(package),
    )
}

fn model_document() -> Value {
    json!({
        "contractVersion": "2.0.0",
        "package": {"identity": "acme/owners", "version": "1.0.0"},
        "constructs": [{
            "kind": {"module": "acme/owners", "name": "entity"},
            "construct": {"meaning": "quire.meaning.model.object-type/v1"},
        }],
        "types": [{
            "identity": "ix://acme/owners/Point", "displayName": "Point",
            "kind": {"module": "acme/owners", "name": "entity"},
            "roles": [], "constraints": [], "extensions": [], "unknownPolicy": "reject",
            "supertypes": [], "fields": [], "operations": [],
        }],
    })
}

fn model_package() -> (Value, Value) {
    let document = model_document();
    let owner = json!({
        "kind": "model", "identity": "acme/owners", "node": "ix://acme/owners/Point",
    });
    let preimage = json!({
        "version": "quire.structural-node/v1", "node_tag": "model",
        "semantic_form": "object_type", "semantic_type": null,
        "declaration": null, "recursion": null, "owner": owner,
        "body": {"term": "aggregate", "members": []},
    });
    let id = node_id(&sha256_hex(&canonical(&preimage)));
    let mut package = v2_nominal();
    package["lock"]["model_selections"] = json!([{
        "identity": "acme/owners", "digest_domain": "sha256-jcs",
        "digest": sha256_hex(&canonical(&document)),
    }]);
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(json!({
            "node_id": id, "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": "model", "semantic_form": "object_type",
            "semantic_type": id, "dependencies": [],
            "occurrences": [{"role": "type", "ordinal": 0}],
            "owner": owner, "body": {"term": "aggregate", "members": []},
        }));
    let source = package["lock"]["sources"][0].clone();
    package["source_map"]
        .as_array_mut()
        .expect("map")
        .push(json!({
            "node_id": id, "role": "type", "ordinal": 0,
            "regions": [{"source": source, "start": 10, "end": 11}],
        }));
    refresh_identity(&mut package);
    (package, document)
}

fn read_with_model(package: &Value, document: &Value) -> CheckedPackageV2ReadResult {
    let mut evidence = evidence_for(package);
    evidence.insert_domain_package_document(sha256_hex(&canonical(document)), canonical(document));
    CheckedPackageV2::read(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    )
}

fn refused(package: &Value, code: CheckedPackageRefusalCode, path: &str) {
    let CheckedPackageV2ReadResult::Refused(refusal) = read(package) else {
        panic!("owner mutation must refuse");
    };
    assert_eq!(refusal.code, code);
    assert_eq!(refusal.path.expect("refusal path").as_str(), path);
}

/// Trace: FR-038-AC-153
#[trace("TC-228", "FR-038-AC-153")]
#[test]
fn tc_228_a_declared_record_carries_its_source_owner_in_both_wire_positions() {
    let package = source_package();
    let node = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .last()
        .expect("record");
    let projection = package["identity_preimage"]["identity_projection"]
        .as_array()
        .expect("projection")
        .last()
        .expect("record projection");
    assert_eq!(node["owner"], projection["owner"]);
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

/// Trace: FR-038-AC-154
#[trace("TC-228", "FR-038-AC-154")]
#[test]
fn tc_228_required_and_forbidden_owner_shapes_refuse_before_identity() {
    let base = source_package();
    let position = base["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
    let node_path = format!("/semantic_graph/nodes/{position}");
    let projection_path = format!("/identity_preimage/identity_projection/{position}");

    let mut missing_node = base.clone();
    missing_node["semantic_graph"]["nodes"][position]
        .as_object_mut()
        .expect("record")
        .remove("owner");
    refused(
        &missing_node,
        CheckedPackageRefusalCode::MalformedWire,
        &node_path,
    );

    let mut missing_projection = base.clone();
    missing_projection["identity_preimage"]["identity_projection"][position]
        .as_object_mut()
        .expect("projection")
        .remove("owner");
    refused(
        &missing_projection,
        CheckedPackageRefusalCode::MalformedWire,
        &projection_path,
    );

    let mut wrong_kind = base.clone();
    wrong_kind["semantic_graph"]["nodes"][position]["owner"] =
        json!({"kind": "model", "identity": "acme/orders", "node": "Order"});
    refused(
        &wrong_kind,
        CheckedPackageRefusalCode::MalformedWire,
        &format!("{node_path}/owner"),
    );

    let mut forbidden = base.clone();
    forbidden["semantic_graph"]["nodes"][0]["owner"] =
        json!({"kind": "source", "authority": "test", "identity": "test"});
    refused(
        &forbidden,
        CheckedPackageRefusalCode::MalformedWire,
        "/semantic_graph/nodes/0/owner",
    );

    let mut different = base;
    different["identity_preimage"]["identity_projection"][position]["owner"]["identity"] =
        json!("another-source");
    refresh_package_id(&mut different);
    refused(
        &different,
        CheckedPackageRefusalCode::StaleDependency,
        &format!("{projection_path}/owner/identity"),
    );
}

fn refresh_package_id(package: &mut Value) {
    package["package_id"]["digest"] = json!(crate::support::checked_package::sha256_hex(
        &canonical(&package["identity_preimage"])
    ));
}

/// Trace: FR-038-AC-155
#[trace("TC-228", "FR-038-AC-155")]
#[test]
fn tc_228_source_owner_joins_selected_source_and_declaration_region() {
    let base = source_package();
    let position = base["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
    let mut unselected = base.clone();
    unselected["semantic_graph"]["nodes"][position]["owner"]["identity"] = json!("other");
    refresh_identity(&mut unselected);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&unselected) else {
        panic!("unselected source must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MissingDeclaration);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::MissingSelection)
    );
    assert_eq!(
        refusal.path.expect("path").as_str(),
        format!("/semantic_graph/nodes/{position}/node_id")
    );

    let mut wrong_region = base;
    let second_source = wrong_region["lock"]["sources"][1].clone();
    wrong_region["source_map"]
        .as_array_mut()
        .expect("map")
        .last_mut()
        .expect("entry")["regions"][0]["source"] = second_source;
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&wrong_region) else {
        panic!("a region from another selected source must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::InvalidValue)
    );
}

/// Trace: FR-038-AC-153, FR-038-AC-155
#[trace("TC-228", "FR-038-AC-153", "FR-038-AC-155")]
#[test]
fn tc_228_an_unreachable_model_node_joins_its_declaring_object_type() {
    let (base, document) = model_package();
    assert!(matches!(
        read_with_model(&base, &document),
        CheckedPackageV2ReadResult::Admitted(_)
    ));

    let position = base["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
    for (name, identity, node) in [
        ("unselected package", "acme/other", "ix://acme/owners/Point"),
        ("unresolved node", "acme/owners", "ix://acme/owners/Missing"),
    ] {
        let mut changed = base.clone();
        changed["semantic_graph"]["nodes"][position]["owner"]["identity"] = json!(identity);
        changed["semantic_graph"]["nodes"][position]["owner"]["node"] = json!(node);
        refresh_identity(&mut changed);
        let CheckedPackageV2ReadResult::Refused(refusal) = read_with_model(&changed, &document)
        else {
            panic!("{name} must refuse even when nothing reaches the node");
        };
        assert_eq!(
            refusal.code,
            CheckedPackageRefusalCode::MissingDeclaration,
            "{name}"
        );
        assert_eq!(
            refusal.cause,
            Some(CheckedPackageRefusalCause::MissingSelection),
            "{name}"
        );
        assert_eq!(
            refusal.path.expect("path").as_str(),
            format!("/semantic_graph/nodes/{position}/node_id")
        );
    }
}
