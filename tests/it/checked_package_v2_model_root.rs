// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Public model-root writer, reader and refusal controls.

use crate::support::checked_package::{
    canonical, domain_package_digest, domain_package_document, evidence_for, refresh_identity,
    refresh_identity_projection, sha256_hex, v2_all_families, v2_nominal,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    encode_checked_package_v2, CheckedNodeOwner, CheckedNodeProjectionV2,
    CheckedPackageIdentityPreimageRefV2, CheckedPackageReadLimits, CheckedPackageRefusalCause,
    CheckedPackageRefusalCode, CheckedPackageSourceCorrespondenceRef, CheckedPackageV2,
    CheckedPackageV2ReadResult, ModelForm,
};
use serde_json::{json, Value};

fn selected_package() -> Value {
    let mut package = v2_all_families();
    let document = domain_package_document("zz/model-root", "1", Vec::new());
    package["lock"]["model_selections"]
        .as_array_mut()
        .expect("selections")
        .push(json!({
            "identity": "zz/model-root", "digest_domain": "sha256-jcs",
            "digest": domain_package_digest(&document),
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

fn admitted(package: &Value) -> Box<CheckedPackageV2> {
    match read(package) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("expected model-root admission, got {other:?}"),
    }
}

fn root_position(package: &Value) -> usize {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| {
            node["semantic_form"] == "model_root" && node["owner"]["identity"] == "zz/model-root"
        })
        .expect("root")
}

fn root_source_position(package: &Value) -> usize {
    let root = root_position(package);
    let id = &package["semantic_graph"]["nodes"][root]["node_id"];
    package["source_map"]
        .as_array()
        .expect("source map")
        .iter()
        .position(|entry| &entry["node_id"] == id)
        .expect("root source map")
}

fn refused(
    package: &Value,
    code: CheckedPackageRefusalCode,
    cause: Option<CheckedPackageRefusalCause>,
    path: &str,
) {
    match read(package) {
        CheckedPackageV2ReadResult::Refused(refusal) => {
            assert_eq!(refusal.code, code);
            assert_eq!(refusal.cause, cause);
            assert_eq!(
                refusal.path.as_ref().map(ToString::to_string).as_deref(),
                Some(path)
            );
        }
        other => panic!("expected {code:?} at {path}, got {other:?}"),
    }
}

/// Trace: FR-065-AC-1, FR-065-AC-2, FR-065-AC-7
#[trace("TC-233", "FR-065-AC-1", "FR-065-AC-2", "FR-065-AC-7")]
#[test]
fn selected_root_roundtrips_through_public_typed_writer_and_reader() {
    let package = selected_package();
    let first = admitted(&package);
    let root = root_position(&package);
    assert_eq!(
        first
            .graph()
            .nodes
            .iter()
            .filter(|node| node.semantic_form.as_ref() == "model_root")
            .count(),
        first.lock().model_selections.len()
    );
    let node = &first.graph().nodes[root];
    assert_eq!(
        first.node_kinds()[root],
        quire_contract_model::CheckedNodeKind::Model(ModelForm::ModelRoot)
    );
    assert!(
        matches!(&node.owner, Some(CheckedNodeOwner::ModelRoot { identity, digest })
        if identity.as_ref() == "zz/model-root"
        && digest == &first.lock().model_selections[1].digest)
    );
    assert_eq!(node.semantic_type, node.node_id);
    assert_eq!(node.occurrences.len(), 1);
    assert_eq!(node.occurrences[0].ordinal, 0);
    assert_eq!(
        first.source_map()[root_source_position(&package)].regions,
        first.source_map()[0].regions
    );
    let projection = CheckedNodeProjectionV2::from(node);
    assert_eq!(projection.owner, node.owner);

    let selections = first.identity_preimage();
    let graph = first.graph();
    let borrowed = CheckedPackageIdentityPreimageRefV2 {
        version: &selections.version,
        edition: &selections.edition,
        profile_selections: &selections.profile_selections,
        definition_selections: &selections.definition_selections,
        model_selections: &selections.model_selections,
        required_features: &selections.required_features,
        dependency_selections: &selections.dependency_selections,
        identity_projection: &graph.nodes,
    };
    let bytes = encode_checked_package_v2(
        &borrowed,
        first.package_id(),
        first.lock(),
        graph,
        CheckedPackageSourceCorrespondenceRef {
            source_map: first.source_map(),
            refinement_step_rows: first.refinement_step_rows(),
        },
        first.capability_report(),
        first.diagnostics(),
        quire_canonical::Limits::new(16 * 1024 * 1024),
    )
    .expect("typed writer");
    let second = match CheckedPackageV2::read(
        &bytes,
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&package),
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("typed roundtrip refused: {other:?}"),
    };
    assert_eq!(second.graph().nodes[root], *node);
    assert_eq!(second.package_id(), first.package_id());
    assert_eq!(
        second.source_map()[root_source_position(&package)],
        first.source_map()[root_source_position(&package)]
    );
}

/// Trace: FR-065-AC-4
#[trace("TC-233", "FR-065-AC-4")]
#[test]
fn one_declaration_selection_has_one_root() {
    let (package, evidence) = super::checked_package_v2_refinement::machine_package();
    let admitted = match CheckedPackageV2::read(
        &canonical(&package),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("selected state machine refused: {other:?}"),
    };
    assert_eq!(admitted.lock().model_selections.len(), 1);
    assert_eq!(
        admitted
            .graph()
            .nodes
            .iter()
            .filter(|node| node.semantic_form.as_ref() == "model_root")
            .count(),
        1
    );
}

/// Trace: FR-065-AC-4
#[trace("TC-233", "FR-065-AC-4")]
#[test]
fn no_model_selection_has_no_root() {
    let package = v2_nominal();
    let admitted = admitted(&package);
    assert!(admitted.lock().model_selections.is_empty());
    assert!(admitted
        .graph()
        .nodes
        .iter()
        .all(|node| node.semantic_form.as_ref() != "model_root"));
}

/// Trace: FR-065-AC-3
#[trace("TC-233", "FR-065-AC-3")]
#[test]
fn published_model_root_preimage_has_the_published_key() {
    let preimage = json!({
        "version": "quire.structural-node/v1", "node_tag": "model",
        "semantic_form": "model_root", "semantic_type": null,
        "declaration": null, "recursion": null,
        "owner": {"kind": "model_root", "identity": "example/counter-cas",
            "digest": "0000000000000000000000000000000000000000000000000000000000000000"},
        "body": {"term": "aggregate", "members": []},
    });
    assert_eq!(
        sha256_hex(&canonical(&preimage)),
        "d6ee4495e48811c9e3c7551c768cba73391f8bac1cd14c05bfff37c0af67f757"
    );
}

/// Trace: FR-065-AC-4
#[trace("TC-233", "FR-065-AC-4")]
#[test]
fn missing_repeated_and_stale_second_roots_refuse_in_stage_order() {
    let valid = selected_package();
    admitted(&valid);
    let root = root_position(&valid);
    let mut missing = valid.clone();
    missing["semantic_graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .remove(root);
    missing["source_map"]
        .as_array_mut()
        .unwrap()
        .remove(root_source_position(&valid));
    refresh_identity_projection(&mut missing);
    refused(
        &missing,
        CheckedPackageRefusalCode::InvalidSemanticGraph,
        None,
        "/semantic_graph/nodes",
    );

    let mut repeated = valid.clone();
    let clone = repeated["semantic_graph"]["nodes"][root].clone();
    let second = repeated["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .len();
    repeated["semantic_graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .push(clone);
    refresh_identity_projection(&mut repeated);
    refused(
        &repeated,
        CheckedPackageRefusalCode::InvalidSemanticGraph,
        None,
        &format!("/semantic_graph/nodes/{second}/node_id"),
    );

    let mut stale = valid;
    let mut clone = stale["semantic_graph"]["nodes"][root].clone();
    let other = json!({"domain": "quire.checked-semantic-node/v1", "digest": "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"});
    clone["node_id"] = other.clone();
    clone["semantic_type"] = other.clone();
    stale["semantic_graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .push(clone);
    let mut entry = stale["source_map"][root_source_position(&stale)].clone();
    entry["node_id"] = other;
    stale["source_map"].as_array_mut().unwrap().push(entry);
    refresh_identity_projection(&mut stale);
    refused(
        &stale,
        CheckedPackageRefusalCode::InvalidPackage,
        Some(CheckedPackageRefusalCause::StaleNodeKey),
        &format!("/semantic_graph/nodes/{second}/node_id"),
    );
}

/// Trace: FR-065-AC-2, FR-065-AC-5
#[trace("TC-233", "FR-065-AC-2", "FR-065-AC-5")]
#[test]
fn root_owner_and_generated_source_occurrence_are_closed() {
    let valid = selected_package();
    let root = root_position(&valid);
    let root_at = format!("/semantic_graph/nodes/{root}");

    let mut absent = valid.clone();
    absent["semantic_graph"]["nodes"][root]
        .as_object_mut()
        .expect("root node")
        .remove("owner");
    refused(
        &absent,
        CheckedPackageRefusalCode::MalformedWire,
        None,
        &root_at,
    );

    let mut wrong = valid.clone();
    wrong["semantic_graph"]["nodes"][root]["owner"] =
        json!({"kind": "model", "identity": "zz/model-root", "node": "ix://zz/model-root/T"});
    refused(
        &wrong,
        CheckedPackageRefusalCode::MalformedWire,
        None,
        &format!("{root_at}/owner"),
    );

    let mut extra = valid.clone();
    extra["semantic_graph"]["nodes"][root]["owner"]["extra"] = json!("x");
    refused(
        &extra,
        CheckedPackageRefusalCode::MalformedWire,
        None,
        &format!("{root_at}/owner"),
    );

    for member in ["identity", "digest"] {
        let mut absent_member = valid.clone();
        absent_member["semantic_graph"]["nodes"][root]["owner"]
            .as_object_mut()
            .expect("root owner")
            .remove(member);
        let CheckedPackageV2ReadResult::Refused(refusal) = read(&absent_member) else {
            panic!("missing root owner {member} admitted");
        };
        assert_eq!(refusal.code, CheckedPackageRefusalCode::MalformedWire);
        assert!(
            refusal
                .path
                .as_ref()
                .is_some_and(|path| path.as_str().starts_with(&format!("{root_at}/owner"))),
            "missing root owner {member} was not located at its owner"
        );
    }

    let mut misplaced = valid.clone();
    let owner = misplaced["semantic_graph"]["nodes"][root]["owner"].clone();
    misplaced["semantic_graph"]["nodes"][0]["owner"] = owner;
    refused(
        &misplaced,
        CheckedPackageRefusalCode::MalformedWire,
        None,
        "/semantic_graph/nodes/0/owner",
    );

    let mut ordinal = valid.clone();
    ordinal["semantic_graph"]["nodes"][root]["occurrences"][0]["ordinal"] = json!(1);
    refused(
        &ordinal,
        CheckedPackageRefusalCode::InvalidSourceMap,
        None,
        &format!("{root_at}/occurrences/0/ordinal"),
    );

    for role in ["type", "declaration"] {
        let mut wrong_role = valid.clone();
        wrong_role["semantic_graph"]["nodes"][root]["occurrences"][0]["role"] = json!(role);
        refused(
            &wrong_role,
            CheckedPackageRefusalCode::InvalidSourceMap,
            None,
            &format!("{root_at}/occurrences/0/role"),
        );
    }

    let mut extra_occurrence = valid.clone();
    extra_occurrence["semantic_graph"]["nodes"][root]["occurrences"]
        .as_array_mut()
        .unwrap()
        .push(json!({"role": "generated", "ordinal": 1}));
    refused(
        &extra_occurrence,
        CheckedPackageRefusalCode::InvalidSourceMap,
        None,
        &format!("{root_at}/occurrences"),
    );

    let mut unselected_owner = valid.clone();
    unselected_owner["semantic_graph"]["nodes"][root]["owner"]["digest"] =
        json!("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
    assert!(matches!(
        read(&unselected_owner),
        CheckedPackageV2ReadResult::Refused(_)
    ));

    let mut absent_region = valid;
    let source_entry = root_source_position(&absent_region);
    absent_region["source_map"][source_entry]["regions"] = json!([]);
    refused(
        &absent_region,
        CheckedPackageRefusalCode::InvalidSourceMap,
        None,
        &format!("/source_map/{source_entry}/regions"),
    );
}

/// Trace: FR-065-AC-2
#[trace("TC-233", "FR-065-AC-2")]
#[test]
fn root_graph_shape_is_closed_before_key_checking() {
    let valid = selected_package();
    let root = root_position(&valid);
    let root_at = format!("/semantic_graph/nodes/{root}");
    let nominal_package = v2_nominal();
    let nominal = nominal_package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find_map(|node| node.get("nominal_identity_preimage"))
        .cloned()
        .expect("valid nominal preimage");
    for (member, value, expected_path) in [
        (
            "recursion_group",
            json!("r"),
            format!("{root_at}/recursion_group"),
        ),
        (
            "nominal_identity_preimage",
            nominal,
            format!("{root_at}/nominal_identity_preimage"),
        ),
        (
            "dependencies",
            json!([valid["semantic_graph"]["nodes"][0]["node_id"]]),
            format!("{root_at}/dependencies"),
        ),
        (
            "body",
            json!({"term": "aggregate", "members": [valid["semantic_graph"]["nodes"][0]["node_id"]]}),
            format!("{root_at}/body/members/0"),
        ),
    ] {
        let mut malformed = valid.clone();
        malformed["semantic_graph"]["nodes"][root][member] = value;
        refresh_identity_projection(&mut malformed);
        let CheckedPackageV2ReadResult::Refused(refusal) = read(&malformed) else {
            panic!("{member} was admitted");
        };
        assert_eq!(
            refusal.code,
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            "{member}"
        );
        assert_eq!(refusal.cause, None);
        assert_eq!(
            refusal.path.as_ref().map(ToString::to_string).as_deref(),
            Some(expected_path.as_str())
        );
    }
    let mut declaration = valid;
    declaration["semantic_graph"]["nodes"][root]["declaration"] =
        json!({"qualified_name": ["zz", "model_root", "Root"]});
    refresh_identity_projection(&mut declaration);
    refused(
        &declaration,
        CheckedPackageRefusalCode::InvalidSemanticGraph,
        None,
        &format!("{root_at}/declaration"),
    );
}

/// Trace: FR-065-AC-3
#[trace("TC-233", "FR-065-AC-3")]
#[test]
fn model_root_identity_depends_on_selection_not_source_region() {
    let original = selected_package();
    let root = root_position(&original);
    let first = admitted(&original);
    let mut moved_source = original.clone();
    let source_entry = root_source_position(&moved_source);
    moved_source["source_map"][source_entry]["regions"][0]["start"] = json!(420);
    moved_source["source_map"][source_entry]["regions"][0]["end"] = json!(421);
    let moved = admitted(&moved_source);
    assert_eq!(
        moved.graph().nodes[root].node_id,
        first.graph().nodes[root].node_id
    );
    assert_eq!(moved.package_id(), first.package_id());

    let mut other = v2_all_families();
    let document = domain_package_document("zz/other-model", "1", Vec::new());
    other["lock"]["model_selections"]
        .as_array_mut()
        .unwrap()
        .push(json!({
            "identity": "zz/other-model", "digest_domain": "sha256-jcs",
            "digest": domain_package_digest(&document),
        }));
    refresh_identity(&mut other);
    let changed = admitted(&other);
    let other_root = other["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .position(|node| node["owner"]["identity"] == "zz/other-model")
        .expect("other root");
    assert_ne!(
        changed.graph().nodes[other_root].node_id,
        first.graph().nodes[root].node_id
    );
    assert_ne!(changed.package_id(), first.package_id());

    let mut changed_content = v2_all_families();
    let document = domain_package_document("zz/model-root", "2", Vec::new());
    let digest = domain_package_digest(&document);
    changed_content["lock"]["model_selections"]
        .as_array_mut()
        .unwrap()
        .push(json!({
            "identity": "zz/model-root", "digest_domain": "sha256-jcs", "digest": digest,
        }));
    refresh_identity(&mut changed_content);
    let mut evidence = evidence_for(&changed_content);
    evidence
        .insert_domain_package_document(digest, canonical(&document))
        .expect("changed selected document");
    let rebuilt = match CheckedPackageV2::read(
        &canonical(&changed_content),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("changed selected content refused: {other:?}"),
    };
    assert_ne!(
        rebuilt.graph().nodes[root].node_id,
        first.graph().nodes[root].node_id
    );
    assert_ne!(rebuilt.package_id(), first.package_id());
}

/// Trace: FR-065-AC-3
#[trace("TC-233", "FR-065-AC-3")]
#[test]
fn root_only_package_keeps_the_root_key_when_its_source_alias_changes() {
    let mut package = v2_nominal();
    package["semantic_graph"]["nodes"] = json!([]);
    package["source_map"] = json!([]);
    let document = domain_package_document("zz/root-only", "1", Vec::new());
    package["lock"]["model_selections"] = json!([{
        "identity": "zz/root-only", "digest_domain": "sha256-jcs",
        "digest": domain_package_digest(&document),
    }]);
    refresh_identity(&mut package);
    let first = admitted(&package);
    assert_eq!(first.graph().nodes.len(), 1);
    assert_eq!(first.graph().nodes[0].semantic_form.as_ref(), "model_root");

    let mut renamed = package;
    renamed["lock"]["sources"][0]["identity"] = json!("test/renamed-source");
    renamed["source_map"][0]["regions"][0]["source"]["identity"] = json!("test/renamed-source");
    refresh_identity_projection(&mut renamed);
    let second = admitted(&renamed);
    assert_eq!(
        second.graph().nodes[0].node_id,
        first.graph().nodes[0].node_id
    );
    assert_eq!(second.package_id(), first.package_id());
}
