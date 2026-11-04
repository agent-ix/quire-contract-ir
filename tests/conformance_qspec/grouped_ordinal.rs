// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Direct production reads of QSpec's corrected recursion-group fixtures.

use super::{
    fixture_support::reassign_node_id, fixtures_dir, proposal_dir_of, read_package_with_document,
    read_positive, CheckedPackageV2ReadResult, Code, DOMAIN_DOCUMENT,
};
use ix_trace_rs::trace;
use serde_json::Value;

/// QSpec FR-322-AC-274: a group member's structural path names its declaration;
/// its producer-assigned node ordinal is distinct from the recursion position.
#[trace("TC-048", "FR-038-AC-107", "FR-038-AC-176")]
#[test]
fn corrected_qspec_grouped_fixtures_admit_with_declared_paths() {
    let fixtures = fixtures_dir();
    let document = proposal_dir_of(&fixtures).join(DOMAIN_DOCUMENT);
    for name in [
        "positive-integer-literal-references.json",
        "positive-two-owners-a.json",
        "positive-two-owners-b.json",
        "positive-recursive-records.json",
    ] {
        let (wire, admitted) =
            read_positive(&fixtures, name, &document).unwrap_or_else(|why| panic!("{why}"));
        let nodes = wire["semantic_graph"]["nodes"]
            .as_array()
            .expect("published graph nodes");
        assert_eq!(admitted.graph().nodes.len(), nodes.len(), "{name}");
        let mut grouped = 0;
        for (index, published) in nodes.iter().enumerate() {
            if published.get("recursion_group").is_none_or(Value::is_null) {
                continue;
            }
            grouped += 1;
            let group = published["recursion_group"]
                .as_str()
                .expect("published group name");
            assert_eq!(
                published["node_id"]["path"],
                serde_json::json!([group]),
                "{name}: node {index}"
            );
            assert_eq!(
                serde_json::to_value(&admitted.graph().nodes[index].node_id)
                    .expect("typed node id"),
                published["node_id"],
                "{name}: node {index} retains its authored ordinal"
            );
        }
        assert!(grouped >= 2, "{name}: expected a real recursion group");
    }
}

/// QSpec FR-322-AC-274: changing an anonymous List member's structural path
/// while retaining every exact reference leaves only the group-path defect.
#[trace("TC-233", "FR-038-AC-145", "FR-038-AC-147", "FR-038-AC-133")]
#[test]
fn anonymous_group_member_with_non_declaration_path_refuses() {
    let fixtures = fixtures_dir();
    let document = proposal_dir_of(&fixtures).join(DOMAIN_DOCUMENT);
    let (mut package, _) = read_positive(&fixtures, "positive-recursive-records.json", &document)
        .expect("published recursive package admits");
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("published graph nodes");
    let position = nodes
        .iter()
        .position(|node| {
            node["recursion_group"] == "List"
                && node["semantic_form"] == "option"
                && node.get("declaration").is_none_or(Value::is_null)
        })
        .expect("anonymous List option");
    let original = nodes[position]["node_id"].clone();
    assert_eq!(original["path"], serde_json::json!(["List"]));
    let mut changed = original.clone();
    changed["path"] = serde_json::json!(["other"]);
    reassign_node_id(&mut package, &original, &changed);
    assert_eq!(
        package["semantic_graph"]["nodes"][position]["node_id"],
        changed
    );
    let result = read_package_with_document(&package, "mutated recursive package", &document)
        .expect("production read runs");
    let CheckedPackageV2ReadResult::Refused(refusal) = result else {
        panic!("anonymous group member path was admitted");
    };
    assert_eq!(refusal.code, Code::InvalidSemanticGraph);
    assert_eq!(refusal.cause, None);
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some(format!("/semantic_graph/nodes/{position}/node_id/path")),
        "{refusal:?}"
    );
}

#[trace("TC-226", "FR-038-AC-132")]
#[test]
fn authoritative_owner_declaration_and_group_forms_are_retained_and_fail_closed() {
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let documents =
        super::vector_selected_documents(&proposal).expect("published selected-document inventory");
    let mut owners = std::collections::BTreeSet::new();
    let mut declarations = 0;
    let mut groups = std::collections::BTreeSet::new();
    for name in super::POSITIVE_ALL {
        let document = proposal.join(super::positive_document(name, &documents));
        let (wire, admitted) =
            read_positive(&fixtures, name, &document).unwrap_or_else(|why| panic!("{name}: {why}"));
        let published = wire["semantic_graph"]["nodes"]
            .as_array()
            .expect("published nodes");
        assert_eq!(admitted.graph().nodes.len(), published.len(), "{name}");
        for (position, (node, golden)) in admitted.graph().nodes.iter().zip(published).enumerate() {
            assert_eq!(
                serde_json::to_value(&node.node_id).expect("id"),
                golden["node_id"],
                "{name}: {position} structural identity"
            );
            assert_eq!(
                serde_json::to_value(&node.body).expect("body"),
                golden["body"],
                "{name}: {position} complete body"
            );
            assert_eq!(
                serde_json::to_value(&node.owner).expect("owner"),
                golden["owner"],
                "{name}: {position} selected owner"
            );
            assert_eq!(
                serde_json::to_value(&node.declaration).expect("declaration"),
                golden["declaration"],
                "{name}: {position} declaration"
            );
            assert_eq!(
                serde_json::to_value(&node.recursion_group).expect("group"),
                golden["recursion_group"],
                "{name}: {position} recursion group"
            );
            if !golden["owner"].is_null() {
                owners.insert(golden["owner"].to_string());
            }
            if !golden["declaration"].is_null() {
                declarations += 1;
            }
            if let Some(group) = golden["recursion_group"].as_str() {
                assert!(
                    !group.is_empty(),
                    "{name}: {position} published nonempty group"
                );
                groups.insert(group.to_owned());
            }
        }
    }
    assert!(
        owners.len() >= 2,
        "distinct published owners must actually be exercised"
    );
    assert!(
        declarations > 0,
        "published declarations must actually be exercised"
    );
    assert!(
        groups.len() >= 2,
        "published labelled groups must actually be exercised"
    );

    for root in [None, Some("")] {
        assert!(
            super::fixtures_dir_of(root).is_err(),
            "absent source cannot select a fallback"
        );
    }
    let scratch =
        std::env::temp_dir().join(format!("ir-owner-group-inputs-{}", std::process::id()));
    std::fs::create_dir_all(&scratch).expect("scratch directory");
    assert!(
        super::fixtures_dir_of(scratch.to_str()).is_err(),
        "missing checkout must fail closed"
    );
    let document = proposal.join(DOMAIN_DOCUMENT);
    let missing = read_positive(&scratch, "absent.json", &document)
        .expect_err("missing golden must not become a derived substitute");
    assert!(missing.contains("absent.json"), "{missing}");
    std::fs::write(scratch.join("malformed.json"), b"not JSON").expect("malformed source control");
    let malformed = read_positive(&scratch, "malformed.json", &document)
        .expect_err("malformed golden must fail closed");
    assert!(malformed.contains("is not JSON"), "{malformed}");
    let missing_document = read_positive(
        &fixtures,
        "positive-all-families.json",
        &scratch.join("absent-domain.json"),
    )
    .expect_err("missing selected document must fail closed");
    assert!(
        missing_document.contains("absent-domain.json"),
        "{missing_document}"
    );
    std::fs::write(scratch.join("malformed-domain.json"), b"not JSON")
        .expect("malformed selected document control");
    let malformed_document = read_positive(
        &fixtures,
        "positive-all-families.json",
        &scratch.join("malformed-domain.json"),
    )
    .expect_err("malformed selected document must fail closed");
    assert!(
        malformed_document.contains("is not JSON"),
        "{malformed_document}"
    );
    std::fs::remove_dir_all(scratch).expect("remove scratch inputs");
}

#[trace("TC-226", "FR-038-AC-88")]
#[test]
fn grouped_member_and_outside_application_references_retain_exact_targets() {
    let fixtures = fixtures_dir();
    let document = proposal_dir_of(&fixtures).join(DOMAIN_DOCUMENT);
    let (mut wire, _) = read_positive(
        &fixtures,
        "positive-integer-literal-references.json",
        &document,
    )
    .expect("published two-member List group");
    let nodes = wire["semantic_graph"]["nodes"]
        .as_array()
        .expect("published nodes");
    let option = nodes
        .iter()
        .position(|node| node["recursion_group"] == "List" && node["semantic_form"] == "option")
        .expect("List option");
    let record = nodes
        .iter()
        .position(|node| node["recursion_group"] == "List" && node["semantic_form"] == "record")
        .expect("List record");
    let application = nodes
        .iter()
        .position(|node| node["body"]["term"] == "application")
        .expect("published application");
    let option_id = nodes[option]["node_id"].clone();
    let record_id = nodes[record]["node_id"].clone();
    let group_bodies = [
        (option, nodes[option]["body"].clone()),
        (record, nodes[record]["body"].clone()),
    ];
    let outside_position = nodes.len();
    let outside_id = serde_json::json!({"package":wire["name"], "path":["value","record_value",{"node":record}], "ordinal":0});
    let mut outside = nodes[application].clone();
    outside.as_object_mut().expect("node").remove("declaration");
    outside["node_id"] = outside_id.clone();
    outside["semantic_form"] = serde_json::json!("record_value");
    outside["semantic_type"] = record_id.clone();
    outside["occurrences"] = serde_json::json!([{"role":"expression", "ordinal":0}]);
    // List's sole field is optional, so the empty record is a real finite value.
    outside["body"] = serde_json::json!({"term":"aggregate", "members":[]});
    outside["dependencies"] = serde_json::json!([]);
    wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(outside);
    wire["semantic_graph"]["nodes"][application]["semantic_type"] = option_id.clone();
    wire["semantic_graph"]["nodes"][application]["body"] = serde_json::json!({
        "term":"application", "operator":"query", "result_type":option_id,
        "arguments":[{"term":"reference", "target":outside_id}],
        "operation":{"identity":"quire.op.record.project", "mode":null,
            "member":{"kind":"field", "declaration":record_id, "name":"next"},
            "leaves":[], "laws":[]}
    });
    super::fixture_support::settle(&mut wire);
    let outcome =
        read_package_with_document(&wire, "group projection", &document).expect("production read");
    let CheckedPackageV2ReadResult::Admitted(admitted) = outcome else {
        panic!("group projection and finite outside value must admit: {outcome:?}");
    };
    for (position, body) in group_bodies {
        let node = &admitted.graph().nodes[position];
        assert_eq!(serde_json::to_value(&node.body).expect("body"), body);
        assert_eq!(node.recursion_group.as_deref(), Some("List"));
        assert_eq!(
            serde_json::to_value(&node.node_id).expect("id"),
            wire["semantic_graph"]["nodes"][position]["node_id"]
        );
    }
    assert_ne!(
        admitted.graph().nodes[option].node_id,
        admitted.graph().nodes[record].node_id
    );
    assert_eq!(
        admitted.graph().nodes[outside_position].recursion_group,
        None
    );
    let application_id = &admitted.graph().nodes[application].node_id;
    let Some(quire_contract_model::CheckedTerm::Application {
        operation,
        arguments,
        result_type,
        ..
    }) = admitted.term(application_id)
    else {
        panic!("typed grouped projection");
    };
    assert_eq!(result_type, &admitted.graph().nodes[option].node_id);
    let quire_contract_model::CheckedOperationMember::Field { declaration, .. } = operation
        .read()
        .expect("operation view")
        .member
        .expect("field member")
    else {
        panic!("record field member");
    };
    assert_eq!(declaration, &admitted.graph().nodes[record].node_id);
    let [quire_contract_model::CheckedTerm::Reference { target }] = arguments.as_slice() else {
        panic!("one outside reference");
    };
    assert_eq!(target, &admitted.graph().nodes[outside_position].node_id);
    assert_eq!(
        admitted.graph().nodes[outside_position].semantic_type,
        admitted.graph().nodes[record].node_id
    );
    let Some(quire_contract_model::CheckedTerm::Aggregate { members }) =
        admitted.term(&admitted.graph().nodes[option].node_id)
    else {
        panic!("typed option body");
    };
    assert!(
        matches!(members.as_slice(), [quire_contract_model::CheckedTerm::Reference { target }] if target == &admitted.graph().nodes[record].node_id)
    );

    let mut malformed = wire.clone();
    malformed["semantic_graph"]["nodes"][option]["node_id"]["ordinal"] =
        serde_json::json!(9_007_199_254_740_992_u64);
    assert_group_refusal(
        &malformed,
        &document,
        Code::MalformedWire,
        &format!("/semantic_graph/nodes/{option}/node_id/ordinal"),
    );
    let mut duplicate = wire.clone();
    let repeated = duplicate["semantic_graph"]["nodes"][option].clone();
    duplicate["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(repeated);
    assert_group_refusal(
        &duplicate,
        &document,
        Code::InvalidSemanticGraph,
        &format!("/semantic_graph/nodes/{}/node_id", outside_position + 1),
    );
    let mut unresolved = wire;
    unresolved["semantic_graph"]["nodes"][option]["body"]["members"][0]["target"]["ordinal"] =
        serde_json::json!(99);
    assert_group_refusal(
        &unresolved,
        &document,
        Code::InvalidSemanticGraph,
        &format!("/semantic_graph/nodes/{option}/body/members/0/target"),
    );
}

fn assert_group_refusal(wire: &Value, document: &std::path::Path, code: Code, path: &str) {
    let outcome =
        read_package_with_document(wire, "group mutation", document).expect("production read");
    let CheckedPackageV2ReadResult::Refused(refusal) = outcome else {
        panic!("{path}: group mutation must refuse: {outcome:?}");
    };
    assert_eq!(refusal.code, code, "{refusal:?}");
    assert_eq!(refusal.cause, None, "{refusal:?}");
    assert_eq!(refusal.loci.len(), 1, "{refusal:?}");
    assert_eq!(
        refusal.loci[0]
            .path
            .as_ref()
            .map(ToString::to_string)
            .as_deref(),
        Some(path),
        "{refusal:?}"
    );
}
