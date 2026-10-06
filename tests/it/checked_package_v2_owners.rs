// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Owner-wire admission and refusal cases built from this crate's public wire vocabulary.

use crate::support::checked_package::{
    canonical, evidence_for, node_id, refresh_identity, sha256_hex, v2_all_families, v2_nominal,
};
use ix_trace_rs::trace;
use quire_contract_ir::{
    CheckedPackageReadLimits, CheckedPackageRefusalCause, CheckedPackageRefusalCode,
    CheckedPackageV2, CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};

fn source_package_form(tag: &str, form: &str, name: &str) -> Value {
    let mut package = v2_nominal();
    let source = package["lock"]["sources"][0].clone();
    let owner = json!({
        "kind": "source", "authority": source["authority"], "identity": source["identity"],
    });
    let preimage = json!({
        "version": "quire.structural-node/v1", "node_tag": tag,
        "semantic_form": form, "semantic_type": null,
        "declaration": {"qualified_name": ["Example", name]},
        "recursion": null, "owner": owner,
        "body": {"term": "aggregate", "members": []},
    });
    let id = node_id(&sha256_hex(&canonical(&preimage)));
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(json!({
            "node_id": id,
            "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": tag, "semantic_form": form,
            "semantic_type": id, "dependencies": [],
            "occurrences": [{"role": "declaration", "ordinal": 0}],
            "declaration": {"qualified_name": ["Example", name]},
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

fn source_package() -> Value {
    source_package_form("composite_type", "record", "Point")
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

fn owned_clause(mut package: Value, document: &Value, declared: &str, clause: &str) -> Value {
    package["lock"]["model_selections"][0]["digest"] = json!(sha256_hex(&canonical(document)));
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
    let literal_type = package["semantic_graph"]["nodes"][0]["node_id"].clone();
    let node = &mut package["semantic_graph"]["nodes"][position];
    node["node_tag"] = json!("function");
    node["semantic_form"] = json!("pure_function");
    node["owner"]["node"] = json!(declared);
    let body = json!({"term": "aggregate", "members": [{
        "term": "binding", "name": "clause",
        "value": {"term": "literal", "type": literal_type,
                  "value_kind": "text", "value": clause},
    }]});
    node["body"] = body.clone();
    let preimage = json!({
        "version": "quire.structural-node/v1", "node_tag": "function",
        "semantic_form": "pure_function", "semantic_type": null,
        "declaration": null, "recursion": null,
        "owner": node["owner"], "body": body,
    });
    let id = node_id(&sha256_hex(&canonical(&preimage)));
    node["node_id"] = id.clone();
    node["semantic_type"] = id.clone();
    package["source_map"]
        .as_array_mut()
        .expect("map")
        .last_mut()
        .expect("entry")["node_id"] = id;
    refresh_identity(&mut package);
    package
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
fn tc_228_declared_record_tuple_and_function_carry_their_source_owner() {
    for (tag, form, name) in [
        ("composite_type", "record", "Point"),
        ("composite_type", "tuple", "Pair"),
        ("function", "pure_function", "compute"),
    ] {
        let package = source_package_form(tag, form, name);
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
}

/// Trace: FR-038-AC-153
#[trace("TC-228", "FR-038-AC-153")]
#[test]
fn tc_228_anonymous_locus_nominal_and_application_nodes_are_owner_free() {
    let package = v2_all_families();
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    let projection = package["identity_preimage"]["identity_projection"]
        .as_array()
        .expect("projection");
    for (name, find) in [
        ("anonymous type", ("scalar_type", "boolean")),
        ("source locus", ("correspondence", "source_locus")),
    ] {
        let position = nodes
            .iter()
            .position(|node| node["node_tag"] == find.0 && node["semantic_form"] == find.1)
            .expect(name);
        assert!(nodes[position].get("owner").is_none(), "{name}");
        assert!(projection[position].get("owner").is_none(), "{name}");
    }
    let nominal_package = v2_nominal();
    let nominal_nodes = nominal_package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nominal nodes");
    let nominal_projection = nominal_package["identity_preimage"]["identity_projection"]
        .as_array()
        .expect("nominal projection");
    let nominal = nominal_nodes
        .iter()
        .position(|node| node.get("nominal_identity_preimage").is_some())
        .expect("nominal node");
    assert!(nominal_nodes[nominal].get("owner").is_none());
    assert!(nominal_projection[nominal].get("owner").is_none());
    let application = nodes
        .iter()
        .position(|node| node["body"]["term"] == "application")
        .expect("application-keyed node");
    assert!(nodes[application].get("owner").is_none());
    assert!(projection[application].get("owner").is_none());
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    assert!(matches!(
        read(&nominal_package),
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

    let mut null_owner = base.clone();
    null_owner["semantic_graph"]["nodes"][position]["owner"] = Value::Null;
    refused(
        &null_owner,
        CheckedPackageRefusalCode::MalformedWire,
        &format!("{node_path}/owner"),
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

    let mut forbidden_projection = base.clone();
    forbidden_projection["identity_preimage"]["identity_projection"][0]["owner"] =
        json!({"kind": "source", "authority": "test", "identity": "test"});
    refused(
        &forbidden_projection,
        CheckedPackageRefusalCode::MalformedWire,
        "/identity_preimage/identity_projection/0/owner",
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
    let source_entry = wrong_region["source_map"].as_array().expect("map").len() - 1;
    assert_eq!(
        refusal.path.expect("path").as_str(),
        format!("/source_map/{source_entry}")
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

    let mut relation_document = document.clone();
    let relationship = "ix://acme/owners/relationship/Point-related-Point";
    relation_document["types"][0]["relationships"] = json!([{
        "identity": relationship,
        "category": "structural", "composite": false, "direction": "source-to-target",
        "sourceEnd": {
            "type": "ix://acme/owners/Point", "role": "related",
            "multiplicity": {"lower": 0, "upper": 1, "ordered": false, "unique": true},
        },
        "targetEnd": {
            "type": "ix://acme/owners/Point",
            "multiplicity": {"lower": 0, "upper": 1, "ordered": false, "unique": true},
        },
        "origin": {"source": {
            "sourceIdentity": "ix://acme/owners/Point", "path": "models/Point.md",
            "startLine": 1, "startColumn": 1,
        }},
    }]);
    let mut wrong_kind = base.clone();
    wrong_kind["lock"]["model_selections"][0]["digest"] =
        json!(sha256_hex(&canonical(&relation_document)));
    wrong_kind["semantic_graph"]["nodes"][position]["owner"]["node"] = json!(relationship);
    refresh_identity(&mut wrong_kind);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_with_model(&wrong_kind, &relation_document)
    else {
        panic!("a relationship cannot back an object type");
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

    let mut interface_document = document.clone();
    interface_document["constructs"][0]["construct"]["meaning"] =
        json!("quire.meaning.systems.interface/v1");
    interface_document["types"][0]["interfaceFeatures"] = json!({});
    let mut wrong_subtype = base.clone();
    wrong_subtype["lock"]["model_selections"][0]["digest"] =
        json!(sha256_hex(&canonical(&interface_document)));
    refresh_identity(&mut wrong_subtype);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_with_model(&wrong_subtype, &interface_document)
    else {
        panic!("an interface cannot back model/object_type");
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

    let mut missing_interface = base.clone();
    missing_interface["semantic_graph"]["nodes"][position]["semantic_form"] =
        json!("systems_interface");
    refresh_identity(&mut missing_interface);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_with_model(&missing_interface, &document)
    else {
        panic!("an ordinary object cannot back model/systems_interface");
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
}

/// Trace: FR-038-AC-153, FR-038-AC-155
#[trace("TC-228", "FR-038-AC-153", "FR-038-AC-155")]
#[test]
fn tc_228_operation_and_invariant_clause_owners_join_only_their_kind() {
    let (base, mut document) = model_package();
    let object = "ix://acme/owners/Point";
    let operation = format!("{object}/op");
    let absent_operation = format!("{object}/absent");
    let field = format!("{object}/field");
    document["types"][0]["operations"] = json!([{"identity": operation, "params": []}]);
    document["types"][0]["fields"] = json!([{
        "identity": field, "name": "field", "typeRef": "ix://quire/native/Integer",
        "presence": "required", "nullable": false, "defaultKind": "none",
        "multiplicity": {"lower": 1, "upper": 1, "ordered": false, "unique": true},
    }]);
    for (name, declared, clause, expected_admission) in [
        ("operation clause", operation.as_str(), "precondition", true),
        ("body clause", operation.as_str(), "body", true),
        ("invariant clause", object, "invariant", true),
        (
            "absent operation member on an unreachable clause",
            absent_operation.as_str(),
            "precondition",
            false,
        ),
        (
            "field cannot back a clause",
            field.as_str(),
            "precondition",
            false,
        ),
        (
            "object cannot back an operation clause",
            object,
            "precondition",
            false,
        ),
        (
            "operation cannot back an invariant",
            operation.as_str(),
            "invariant",
            false,
        ),
    ] {
        let package = owned_clause(base.clone(), &document, declared, clause);
        let position = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .len()
            - 1;
        match read_with_model(&package, &document) {
            CheckedPackageV2ReadResult::Admitted(_) if expected_admission => {}
            CheckedPackageV2ReadResult::Refused(refusal) if !expected_admission => {
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
                    format!("/semantic_graph/nodes/{position}/node_id"),
                    "{name}"
                );
            }
            other => panic!("{name}: unexpected {other:?}"),
        }
    }
}

/// This is the IR-627/IR-630 boundary: IR-627 validates only owner-free
/// structural keys. IR-630 must make this stale owner-bearing function key
/// refuse; admission here is not evidence of FR-092 owner-key validation.
///
/// Trace: TC-228
#[trace("TC-228")]
#[test]
fn tc_228_owner_bearing_function_key_remains_for_ir_630() {
    let (base, mut document) = model_package();
    let operation = "ix://acme/owners/Point/op";
    document["types"][0]["operations"] = json!([{"identity": operation, "params": []}]);
    let mut package = owned_clause(base, &document, operation, "precondition");
    assert!(matches!(
        read_with_model(&package, &document),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
    let node = &mut package["semantic_graph"]["nodes"][position];
    node["body"]["members"][0]["value"]["value"] = json!("body");
    let would_be_key = sha256_hex(&canonical(&json!({
        "version": "quire.structural-node/v1", "node_tag": "function",
        "semantic_form": "pure_function", "semantic_type": null,
        "declaration": null, "recursion": null,
        "owner": node["owner"], "body": node["body"],
    })));
    assert_ne!(node["node_id"]["digest"], would_be_key);
    refresh_identity(&mut package);
    assert!(matches!(
        read_with_model(&package, &document),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

/// Trace: FR-038-AC-155
#[trace("TC-228", "FR-038-AC-155")]
#[test]
fn tc_228_one_model_owner_lookup_consumes_one_validation_visit() {
    let (package, document) = model_package();
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
    let owner_path = format!("/semantic_graph/nodes/{position}/node_id");
    let mut evidence = evidence_for(&package);
    evidence
        .insert_domain_package_document(sha256_hex(&canonical(&document)), canonical(&document));
    let bytes = canonical(&package);
    for work in 0..4096 {
        let mut limits = CheckedPackageReadLimits::bounded();
        limits.work = work;
        let outcome = CheckedPackageV2::read(&bytes, limits, &evidence);
        let CheckedPackageV2ReadResult::Incomplete(incomplete) = outcome else {
            continue;
        };
        if incomplete
            .path
            .as_ref()
            .is_some_and(|path| path.as_str() == owner_path)
        {
            assert_eq!(incomplete.consumed, work + 1);
            limits.work = work + 1;
            let next = CheckedPackageV2::read(&bytes, limits, &evidence);
            assert!(
                !matches!(next, CheckedPackageV2ReadResult::Incomplete(ref incomplete)
                    if incomplete.path.as_ref().is_some_and(|path| path.as_str() == owner_path)),
                "the one-visit owner lookup completes at the next limit"
            );
            return;
        }
    }
    panic!("the bounded reader did not expose the owner lookup charge");
}

/// Trace: FR-038-AC-155
#[trace("TC-228", "FR-038-AC-155")]
#[test]
fn tc_228_owner_defects_use_digest_order_ahead_of_a_lower_stale_key() {
    let mut package = v2_all_families();
    let application = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .enumerate()
        .filter(|(_, node)| node["body"]["term"] == "application")
        .min_by_key(|(_, node)| {
            node["node_id"]["digest"]
                .as_str()
                .expect("digest")
                .to_owned()
        })
        .map(|(position, _)| position)
        .expect("application-keyed node");
    let stale_key = package["semantic_graph"]["nodes"][application]["node_id"]["digest"]
        .as_str()
        .expect("digest")
        .to_owned();
    let operation = package["semantic_graph"]["nodes"][application]["body"]["operation"]
        ["identity"]
        .as_str()
        .expect("operation")
        .to_owned();
    package["semantic_graph"]["nodes"][application]["body"]["operation"]["identity"] =
        json!(format!("{operation}.changed"));
    refresh_identity(&mut package);
    let CheckedPackageV2ReadResult::Refused(stale_refusal) = read(&package) else {
        panic!("the changed application preimage must have a stale key");
    };
    assert_eq!(
        stale_refusal.code,
        CheckedPackageRefusalCode::InvalidPackage
    );
    assert_eq!(
        stale_refusal.cause,
        Some(CheckedPackageRefusalCause::StaleNodeKey)
    );
    assert_eq!(
        stale_refusal.path.expect("path").as_str(),
        format!("/semantic_graph/nodes/{application}/node_id")
    );

    let mut owners = Vec::new();
    for index in 0..64 {
        let form = if index % 2 == 0 { "record" } else { "tuple" };
        let fixture = source_package_form("composite_type", form, &format!("Owner{index}"));
        let node = fixture["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .last()
            .expect("owner node")
            .clone();
        let digest = node["node_id"]["digest"].as_str().expect("digest");
        if digest > stale_key.as_str() {
            let entry = fixture["source_map"]
                .as_array()
                .expect("map")
                .last()
                .expect("entry")
                .clone();
            owners.push((node, entry));
            if owners.len() == 2 {
                break;
            }
        }
    }
    assert_eq!(
        owners.len(),
        2,
        "two owner keys above the stale application key"
    );
    owners.sort_by(|left, right| {
        right.0["node_id"]["digest"]
            .as_str()
            .cmp(&left.0["node_id"]["digest"].as_str())
    });
    for (node, entry) in owners {
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(node);
        package["source_map"]
            .as_array_mut()
            .expect("map")
            .push(entry);
    }
    let nodes = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    let positions = [nodes.len() - 2, nodes.len() - 1];
    for position in positions {
        nodes[position]["owner"]["identity"] = json!("unselected");
    }
    let first = positions
        .into_iter()
        .min_by_key(|position| {
            nodes[*position]["node_id"]["digest"]
                .as_str()
                .expect("digest")
                .to_owned()
        })
        .expect("owner positions");
    refresh_identity(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("the first unselected source owner must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MissingDeclaration);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::MissingSelection)
    );
    assert_eq!(
        refusal.path.expect("path").as_str(),
        format!("/semantic_graph/nodes/{first}/node_id")
    );
}

/// Trace: FR-038-AC-154
#[trace("TC-228", "FR-038-AC-154")]
#[test]
fn tc_228_model_owner_is_closed_and_required_in_both_wire_positions() {
    let (base, _) = model_package();
    let position = base["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
    for projection in [false, true] {
        let prefix = if projection {
            "/identity_preimage/identity_projection"
        } else {
            "/semantic_graph/nodes"
        };
        let mut absent = base.clone();
        let node = if projection {
            &mut absent["identity_preimage"]["identity_projection"][position]
        } else {
            &mut absent["semantic_graph"]["nodes"][position]
        };
        node.as_object_mut().expect("node").remove("owner");
        refused(
            &absent,
            CheckedPackageRefusalCode::MalformedWire,
            &format!("{prefix}/{position}"),
        );

        let mut versioned = base.clone();
        let owner = if projection {
            &mut versioned["identity_preimage"]["identity_projection"][position]["owner"]
        } else {
            &mut versioned["semantic_graph"]["nodes"][position]["owner"]
        };
        owner["version"] = json!("1.0.0");
        refused(
            &versioned,
            CheckedPackageRefusalCode::MalformedWire,
            &format!("{prefix}/{position}/owner"),
        );
    }
}
