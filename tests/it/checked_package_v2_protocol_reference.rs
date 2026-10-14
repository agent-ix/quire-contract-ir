// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Public V2 reader controls for the local protocol-reference carrier.

use crate::support::checked_package::{
    admitted_dependency, canonical, evidence_for, family_key, fixture_source, node_id,
    read_with_dependencies, rebuild_source_map, refresh_identity, refusal_at,
    rekey_application_node, source_owner, structural_key, v2_all_families, v2_nominal,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedNodeKind, CheckedPackageReadLimits, CheckedPackageRefusalCause,
    CheckedPackageRefusalCode, CheckedPackageV2, CheckedPackageV2ReadResult, ProtocolForm,
};
use serde_json::{json, Value};

fn read(package: &Value) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(package),
    )
}

fn carrier(target: &Value, semantic_type: &Value) -> Value {
    let body = json!({"term": "reference", "target": target});
    let key = structural_key(
        "protocol",
        "protocol_reference",
        semantic_type["digest"].as_str(),
        &body,
    );
    json!({
        "node_id": node_id(&key),
        "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": "protocol",
        "semantic_form": "protocol_reference",
        "semantic_type": semantic_type,
        "dependencies": [target],
        "occurrences": [{"role": "expression", "ordinal": 0}],
        "body": body,
    })
}

fn add_carrier(package: &mut Value, target: &Value, semantic_type: &Value) -> usize {
    let nodes = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    let position = nodes.len();
    nodes.push(carrier(target, semantic_type));
    rebuild_source_map(package);
    refresh_identity(package);
    position
}

fn retarget(package: &mut Value, position: usize, target: &Value) {
    let semantic_type = package["semantic_graph"]["nodes"][position]["semantic_type"].clone();
    package["semantic_graph"]["nodes"][position] = carrier(target, &semantic_type);
    rebuild_source_map(package);
    refresh_identity(package);
}

fn role_target(package: &mut Value) -> Value {
    let body = json!({"term": "aggregate", "members": []});
    let semantic_type = node_id(&family_key("aaaa"));
    let key = structural_key("protocol", "role", Some(&family_key("aaaa")), &body);
    let target = node_id(&key);
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(json!({
            "node_id": target,
            "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": "protocol",
            "semantic_form": "role",
            "semantic_type": semantic_type,
            "dependencies": [],
            "occurrences": [{"role": "generated", "ordinal": 0}],
            "body": body,
        }));
    target
}

/// Trace: FR-038-AC-239, TC-048
#[trace("FR-038-AC-239", "TC-048")]
#[test]
fn protocol_reference_reads_both_protocol_target_forms_and_rekeys_on_retarget() {
    let mut package = v2_all_families();
    let clause = node_id(&family_key("5050"));
    let semantic_type = node_id(&family_key("aaaa"));
    let position = add_carrier(&mut package, &clause, &semantic_type);
    let first_key = package["semantic_graph"]["nodes"][position]["node_id"].clone();
    let first = match read(&package) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("protocol clause carrier did not admit: {other:?}"),
    };
    assert_eq!(
        first.graph().nodes.len(),
        package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .len()
    );
    assert_eq!(first.source_map().len(), first.graph().nodes.len());
    assert_eq!(
        first.source_map()[position].node_id,
        first.graph().nodes[position].node_id
    );
    assert_eq!(
        first.source_map()[position].role,
        first.graph().nodes[position].occurrences[0].role
    );
    assert_eq!(
        CheckedNodeKind::decode(
            quire_contract_model::CheckedNodeTag::Protocol,
            &first.graph().nodes[position].semantic_form,
        ),
        Some(CheckedNodeKind::Protocol(ProtocolForm::ProtocolReference)),
    );
    assert!(first.graph().nodes.iter().any(|node| {
        node.node_tag.as_ref() == "expression" && node.semantic_form.as_ref() == "reference"
    }));
    assert_eq!(
        first.graph().nodes[position].dependencies,
        vec![
            serde_json::from_value(first.graph().nodes[position].body["target"].clone())
                .expect("typed target")
        ]
    );

    let role = role_target(&mut package);
    retarget(&mut package, position, &role);
    assert_ne!(
        package["semantic_graph"]["nodes"][position]["node_id"],
        first_key
    );
    match read(&package) {
        CheckedPackageV2ReadResult::Admitted(package) => {
            assert_eq!(
                package.graph().nodes[position].semantic_form.as_ref(),
                "protocol_reference"
            );
        }
        other => panic!("protocol role carrier did not admit: {other:?}"),
    }
}

/// Trace: FR-038-AC-240, TC-048
#[trace("FR-038-AC-240", "TC-048")]
#[test]
fn protocol_reference_joins_only_the_local_typed_protocol_target() {
    let mut base = v2_all_families();
    let clause = node_id(&family_key("5050"));
    let boolean = node_id(&family_key("aaaa"));
    let position = add_carrier(&mut base, &clause, &boolean);
    let at = format!("/semantic_graph/nodes/{position}/body/target");

    let absent = node_id(&"9".repeat(64));
    let mut missing = base.clone();
    retarget(&mut missing, position, &absent);
    let missing_key = missing["semantic_graph"]["nodes"][position]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    assert_eq!(
        read(&missing),
        refusal_at(
            CheckedPackageRefusalCode::MissingDeclaration,
            &at,
            Some(CheckedPackageRefusalCause::MissingName),
            missing_key,
        )
    );

    let mut dependency_document = v2_all_families();
    let selected_target = role_target(&mut dependency_document);
    rebuild_source_map(&mut dependency_document);
    refresh_identity(&mut dependency_document);
    let (digest, admitted) = admitted_dependency(&dependency_document);
    let mut selected_only = base.clone();
    retarget(&mut selected_only, position, &selected_target);
    selected_only["lock"]["dependency_selections"] = json!([{
        "identity": "quire.fixture.selected-protocol/v1",
        "package_id": dependency_document["package_id"],
    }]);
    refresh_identity(&mut selected_only);
    assert_eq!(digest, dependency_document["package_id"]["digest"]);
    let selected_key = selected_only["semantic_graph"]["nodes"][position]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    assert_eq!(
        read_with_dependencies(
            &selected_only,
            &[("quire.fixture.selected-protocol/v1", &admitted)]
        ),
        refusal_at(
            CheckedPackageRefusalCode::MissingDeclaration,
            &at,
            Some(CheckedPackageRefusalCause::MissingName),
            selected_key
        ),
    );

    let mut wrong_family = base.clone();
    retarget(&mut wrong_family, position, &boolean);
    let wrong_key = wrong_family["semantic_graph"]["nodes"][position]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    assert_eq!(
        read(&wrong_family),
        refusal_at(
            CheckedPackageRefusalCode::IllTyped,
            &at,
            Some(CheckedPackageRefusalCause::OperatorIneligible),
            wrong_key,
        )
    );

    let mut wrong_type = base.clone();
    let role = role_target(&mut wrong_type);
    let integer = node_id(&family_key("a3a3"));
    wrong_type["semantic_graph"]["nodes"][position] = carrier(&role, &integer);
    rebuild_source_map(&mut wrong_type);
    refresh_identity(&mut wrong_type);
    let wrong_type_key = wrong_type["semantic_graph"]["nodes"][position]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    assert_eq!(
        read(&wrong_type),
        refusal_at(
            CheckedPackageRefusalCode::IllTyped,
            &at,
            Some(CheckedPackageRefusalCause::OperatorIneligible),
            wrong_type_key,
        )
    );
}

/// Trace: FR-038-AC-240, TC-048
#[trace("FR-038-AC-240", "TC-048")]
#[test]
fn protocol_reference_target_refusal_precedes_an_operation_refusal() {
    let mut package = v2_all_families();
    let role = role_target(&mut package);
    let position = add_carrier(&mut package, &role, &node_id(&family_key("aaaa")));
    retarget(&mut package, position, &node_id(&"9".repeat(64)));
    let operation = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| {
            node["node_tag"] == "protocol" && node["semantic_form"] == "protocol_clause"
        })
        .expect("protocol clause");
    package["semantic_graph"]["nodes"][operation]["body"]["operation"]["identity"] =
        json!("quire.op.protocol.unknown");
    rekey_application_node(&mut package, operation);
    refresh_identity(&mut package);
    let key = package["semantic_graph"]["nodes"][position]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::MissingDeclaration,
            &format!("/semantic_graph/nodes/{position}/body/target"),
            Some(CheckedPackageRefusalCause::MissingName),
            key,
        )
    );
}

/// Trace: FR-038-AC-240, TC-048
#[trace("FR-038-AC-240", "TC-048")]
#[test]
fn protocol_reference_dependency_shape_precedes_key_and_local_join() {
    let mut base = v2_all_families();
    let clause = node_id(&family_key("5050"));
    let semantic_type = node_id(&family_key("aaaa"));
    let position = add_carrier(&mut base, &clause, &semantic_type);
    let path = format!("/semantic_graph/nodes/{position}/dependencies");
    for dependencies in [
        json!([]),
        json!([semantic_type]),
        json!([clause, semantic_type]),
    ] {
        let mut changed = base.clone();
        changed["semantic_graph"]["nodes"][position]["dependencies"] = dependencies;
        refresh_identity(&mut changed);
        assert_eq!(
            read(&changed),
            refusal_at(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                &path,
                None,
                base["semantic_graph"]["nodes"][position]["node_id"]["digest"]
                    .as_str()
                    .expect("digest"),
            )
        );
    }
    let mut stale = base.clone();
    stale["semantic_graph"]["nodes"][position]["body"]["target"] = node_id(&"9".repeat(64));
    stale["semantic_graph"]["nodes"][position]["dependencies"] = json!([node_id(&"9".repeat(64))]);
    refresh_identity(&mut stale);
    assert_eq!(
        read(&stale),
        refusal_at(
            CheckedPackageRefusalCode::InvalidPackage,
            &format!("/semantic_graph/nodes/{position}/node_id"),
            Some(CheckedPackageRefusalCause::StaleNodeKey),
            base["semantic_graph"]["nodes"][position]["node_id"]["digest"]
                .as_str()
                .expect("digest"),
        )
    );
}

/// Trace: FR-038-AC-241, TC-048
#[trace("FR-038-AC-241", "TC-048")]
#[test]
fn protocol_reference_requires_closed_body_and_single_expression_occurrence() {
    let mut base = v2_all_families();
    let position = add_carrier(
        &mut base,
        &node_id(&family_key("5050")),
        &node_id(&family_key("aaaa")),
    );
    for body in [
        json!({"term": "aggregate", "members": []}),
        json!({"term": "dependency_reference", "package": node_id(&"a".repeat(64)), "node": node_id(&"b".repeat(64))}),
        json!({"term": "reference", "target": node_id(&family_key("5050")), "extra": 1}),
    ] {
        let mut changed = base.clone();
        changed["semantic_graph"]["nodes"][position]["body"] = body;
        refresh_identity(&mut changed);
        let actual = match read(&changed) {
            CheckedPackageV2ReadResult::Refused(value) => value,
            other => panic!("expected closed-body refusal: {other:?}"),
        };
        assert_eq!(actual.code, CheckedPackageRefusalCode::MalformedWire);
        assert_eq!(
            actual.path.as_ref().expect("path").to_string(),
            format!("/semantic_graph/nodes/{position}/body")
        );
    }
    let mut changed = base.clone();
    changed["semantic_graph"]["nodes"][position]["occurrences"] = json!([
        {"role": "expression", "ordinal": 0}, {"role": "expression", "ordinal": 1}
    ]);
    rebuild_source_map(&mut changed);
    refresh_identity(&mut changed);
    let actual = match read(&changed) {
        CheckedPackageV2ReadResult::Refused(value) => value,
        other => panic!("expected occurrence refusal: {other:?}"),
    };
    assert_eq!(actual.code, CheckedPackageRefusalCode::InvalidSemanticGraph);
    assert_eq!(
        actual.path.as_ref().expect("path").to_string(),
        format!("/semantic_graph/nodes/{position}/occurrences/1")
    );
}

/// Trace: FR-038-AC-241, TC-048
#[trace("FR-038-AC-241", "TC-048")]
#[test]
fn protocol_reference_refuses_forbidden_carrier_members_before_target_join() {
    let mut base = v2_all_families();
    let position = add_carrier(
        &mut base,
        &node_id(&family_key("5050")),
        &node_id(&family_key("aaaa")),
    );
    retarget(&mut base, position, &node_id(&"9".repeat(64)));
    let nominal = v2_nominal()["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find_map(|node| node.get("nominal_identity_preimage"))
        .expect("nominal preimage")
        .clone();
    for (member, value) in [
        (
            "declaration",
            json!({"qualified_name": ["Example", "Carrier"]}),
        ),
        ("owner", source_owner(&fixture_source())),
        ("nominal_identity_preimage", nominal),
        ("recursion_group", json!("group")),
    ] {
        let mut changed = base.clone();
        changed["semantic_graph"]["nodes"][position][member] = value;
        refresh_identity(&mut changed);
        let actual = match read(&changed) {
            CheckedPackageV2ReadResult::Refused(value) => value,
            other => panic!("expected forbidden {member} refusal: {other:?}"),
        };
        assert!(
            matches!(
                actual.code,
                CheckedPackageRefusalCode::MalformedWire
                    | CheckedPackageRefusalCode::InvalidSemanticGraph
            ),
            "{member}: {actual:?}"
        );
        assert_eq!(
            actual.path.as_ref().expect("path").to_string(),
            format!("/semantic_graph/nodes/{position}/{member}"),
            "{member}"
        );
    }

    let mut wrong_role = base.clone();
    wrong_role["semantic_graph"]["nodes"][position]["occurrences"][0]["role"] = json!("generated");
    rebuild_source_map(&mut wrong_role);
    refresh_identity(&mut wrong_role);
    let actual = match read(&wrong_role) {
        CheckedPackageV2ReadResult::Refused(value) => value,
        other => panic!("expected role refusal: {other:?}"),
    };
    assert_eq!(actual.code, CheckedPackageRefusalCode::InvalidSemanticGraph);
    assert_eq!(
        actual.path.as_ref().expect("path").to_string(),
        format!("/semantic_graph/nodes/{position}/occurrences/0/role")
    );
}
