// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Focused FR-061 refinement and selected state-machine owner vectors.

use crate::support::checked_package::{
    canonical, evidence_for, node_id, protocol_named_node, rebuild_source_map, refresh_identity,
    rekey_application_node, settle, sha256_hex, structural_key, v2_all_families, v2_nominal,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    encode_checked_package_v2, CheckedCapability, CheckedDiagnosticsV2, CheckedPackageEvidence,
    CheckedPackageIdentityPreimageRefV2, CheckedPackageIdentityPreimageV2, CheckedPackageLockV2,
    CheckedPackageReadLimits, CheckedPackageRefusalCause, CheckedPackageRefusalCode,
    CheckedPackageV2, CheckedPackageV2ReadResult, CheckedRefinementStepRowRegion,
    CheckedSemanticGraphV2, CheckedSemanticId, CheckedSourceMapEntry, CheckedTerm,
    RefinementClassCatalog, RefinementConcreteClass, RefinementDrain, RefinementMemoryClass,
    RefinementMemoryModel, RefinementTsoModel,
};
use serde_json::{json, Value};

const MODEL_IDENTITY: &str = "test/refinement";
const MODEL_NODE: &str = "ix://test/refinement/Machine";

fn protocol_class_path(node: &Value) -> String {
    let path = node["declaration"]["qualified_name"]
        .as_array()
        .expect("protocol declaration");
    let kind = path
        .iter()
        .position(|part| {
            matches!(
                part.as_str(),
                Some("finish" | "run" | "channel" | "role" | "compensate")
            )
        })
        .expect("fixed protocol segment");
    path[..kind]
        .iter()
        .chain(path.last())
        .map(|part| part.as_str().expect("name segment"))
        .collect::<Vec<_>>()
        .join("::")
}

fn assert_unique_node_ids(package: &Value) {
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    for (index, node) in nodes.iter().enumerate() {
        if let Some(previous) = nodes[..index]
            .iter()
            .position(|other| other["node_id"] == node["node_id"])
        {
            panic!(
                "duplicate node IDs at {previous}/{index}: {}",
                node["node_id"]
            );
        }
    }
}

fn model_document() -> Value {
    json!({
        "contractVersion": "2.0.0",
        "package": {"identity": MODEL_IDENTITY, "version": "1"},
        "constructs": [{
            "kind": {"module": MODEL_IDENTITY, "name": "machine"},
            "construct": {"meaning": "quire.meaning.model.state-machine/v1"}
        }],
        "types": [{
            "identity": MODEL_NODE, "displayName": "Machine",
            "kind": {"module": MODEL_IDENTITY, "name": "machine"},
            "roles": [], "constraints": [], "extensions": [], "unknownPolicy": "reject",
            "supertypes": [], "fields": [],
            "operations": [{"identity":format!("{MODEL_NODE}/toggle"), "name":"toggle", "params":[]}],
            "states": [
                {"identity":format!("{MODEL_NODE}/closed"), "name":"closed"},
                {"identity":format!("{MODEL_NODE}/open"), "name":"open"}
            ],
            "transitions": [{"identity":format!("{MODEL_NODE}/open-transition"),
                "from":format!("{MODEL_NODE}/closed"), "to":format!("{MODEL_NODE}/open"),
                "trigger":format!("{MODEL_NODE}/toggle"), "emits":[]}]
        }]
    })
}

fn selected_form_node(form: &str, identity: &str) -> Value {
    selected_form_node_owned(form, MODEL_IDENTITY, identity)
}

fn selected_form_node_owned(form: &str, owner_identity: &str, identity: &str) -> Value {
    let owner = json!({"kind":"model", "identity":owner_identity, "node":identity});
    let body = json!({"term":"aggregate", "members":[]});
    let key = node_id(&sha256_hex(&canonical(&json!({
        "version":"quire.structural-node/v1", "node_tag":"model",
        "semantic_form":form, "semantic_type":null, "declaration":null,
        "recursion":null, "owner":owner, "body":body
    }))));
    json!({
        "node_id":key, "schema_version":"quire.checked-semantic-graph/v2",
        "node_tag":"model", "semantic_form":form, "semantic_type":key,
        "dependencies":[], "occurrences":[{"role":"type", "ordinal":0}],
        "owner":owner, "body":body
    })
}

fn selected_form_document() -> Value {
    let mut document = model_document();
    let constructs = document["constructs"].as_array_mut().expect("constructs");
    for (name, meaning) in [
        ("process", "quire.meaning.model.process/v1"),
        ("record", "quire.meaning.model.record-value-type/v1"),
        (
            "persistence",
            "quire.meaning.model.persistence-interface/v1",
        ),
        ("object", "quire.meaning.model.object-type/v1"),
        ("value", "quire.meaning.model.value-type/v1"),
    ] {
        constructs.push(json!({"kind":{"module":MODEL_IDENTITY,"name":name},
            "construct":{"meaning":meaning}}));
    }
    let types = document["types"].as_array_mut().expect("types");
    let mut declared = types[0].clone();
    declared["identity"] = json!(format!("ix://{MODEL_IDENTITY}/Flow"));
    declared["displayName"] = json!("Flow");
    declared["kind"]["name"] = json!("process");
    declared.as_object_mut().expect("Flow").remove("states");
    declared
        .as_object_mut()
        .expect("Flow")
        .remove("transitions");
    declared["fields"] = json!([{
        "identity":format!("ix://{MODEL_IDENTITY}/Flow/n"),
        "name":"n", "typeRef":"ix://quire/native/Integer",
        "presence":"required", "nullable":false, "defaultKind":"none",
        "multiplicity":{"lower":1,"upper":1,"ordered":false,"unique":true}
    }]);
    declared["operations"] = json!([{
        "identity":format!("ix://{MODEL_IDENTITY}/Flow/go"),
        "params":[{"identity":format!("ix://{MODEL_IDENTITY}/Flow/go/step"),
            "name":"step", "typeRef":"ix://quire/native/Integer",
            "multiplicity":{"lower":1,"upper":1,"ordered":false,"unique":true}}],
        "returns":{"typeRef":"ix://quire/native/Integer",
            "multiplicity":{"lower":1,"upper":1,"ordered":false,"unique":true}}
    }]);
    declared["identityFields"] = json!([format!("ix://{MODEL_IDENTITY}/Flow/n")]);
    declared["steps"] = json!([{
        "identity":format!("ix://{MODEL_IDENTITY}/Flow/advance"),
        "name":"advance", "stepKind":"command", "consumes":[], "emits":[]
    }]);
    types.push(declared.clone());
    declared["identity"] = json!(format!("ix://{MODEL_IDENTITY}/Store"));
    declared["displayName"] = json!("Store");
    declared["kind"]["name"] = json!("persistence");
    declared["fields"] = json!([]);
    declared
        .as_object_mut()
        .expect("Store")
        .remove("identityFields");
    declared.as_object_mut().expect("Store").remove("steps");
    declared["operations"] = json!([{
        "identity":format!("ix://{MODEL_IDENTITY}/Store/save"),
        "name":"save", "params":[]
    }]);
    declared["persists"] = json!([format!("ix://{MODEL_IDENTITY}/Account")]);
    types.push(declared.clone());
    declared["identity"] = json!(format!("ix://{MODEL_IDENTITY}/Address"));
    declared["displayName"] = json!("Address");
    declared["kind"]["name"] = json!("record");
    declared
        .as_object_mut()
        .expect("Address")
        .remove("persists");
    declared["operations"] = json!([]);
    declared["fields"] = json!([{
        "identity":format!("ix://{MODEL_IDENTITY}/Address/street"),
        "name":"street", "typeRef":format!("ix://{MODEL_IDENTITY}/Code"),
        "presence":"required", "nullable":false, "defaultKind":"none",
        "multiplicity":{"lower":1,"upper":1,"ordered":false,"unique":true}
    }]);
    types.push(declared.clone());
    declared["identity"] = json!(format!("ix://{MODEL_IDENTITY}/Account"));
    declared["displayName"] = json!("Account");
    declared["kind"]["name"] = json!("object");
    declared["fields"] = json!([{
        "identity":format!("ix://{MODEL_IDENTITY}/Account/address"),
        "name":"address", "typeRef":format!("ix://{MODEL_IDENTITY}/Address"),
        "presence":"required", "nullable":false, "defaultKind":"none",
        "multiplicity":{"lower":1,"upper":1,"ordered":false,"unique":true}
    }, {
        "identity":format!("ix://{MODEL_IDENTITY}/Account/id"),
        "name":"id", "typeRef":"ix://quire/native/UUID",
        "presence":"required", "nullable":false, "defaultKind":"none",
        "multiplicity":{"lower":1,"upper":1,"ordered":false,"unique":true}
    }]);
    declared["identityFields"] = json!([format!("ix://{MODEL_IDENTITY}/Account/id")]);
    types.push(declared);
    types.push(json!({
        "identity":format!("ix://{MODEL_IDENTITY}/Code"), "displayName":"Code",
        "kind":{"module":MODEL_IDENTITY,"name":"value"},
        "roles":[], "extensions":[], "unknownPolicy":"reject", "scalar":"string",
        "constraints":[
            {"keyword":"minLength","operands":{"value":0}},
            {"keyword":"maxLength","operands":{"value":64}},
            {"keyword":"format","operands":{"name":"text-profile:nfc"}}
        ]
    }));
    document
}

pub(crate) fn selected_forms_package() -> (Value, CheckedPackageEvidence) {
    selected_forms_package_with(selected_form_document())
}

fn selected_forms_package_with(document: Value) -> (Value, CheckedPackageEvidence) {
    let digest = sha256_hex(&canonical(&document));
    let mut package = v2_nominal();
    package["lock"]["model_selections"] = json!([{
        "identity":MODEL_IDENTITY, "digest_domain":"sha256-jcs", "digest":digest
    }]);
    let source = package["lock"]["sources"][0].clone();
    for (form, name) in [
        ("state_machine", "Machine"),
        ("process", "Flow"),
        ("persistence_interface", "Store"),
        ("record_value_type", "Address"),
        ("object_type", "Account"),
    ] {
        let node = selected_form_node(form, &format!("ix://{MODEL_IDENTITY}/{name}"));
        let id = node["node_id"].clone();
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(node);
        package["source_map"]
            .as_array_mut()
            .expect("map")
            .push(json!({
                "node_id":id, "role":"type", "ordinal":0,
                "regions":[{"source":source,"start":10,"end":11}]
            }));
    }
    for form in ["process", "state_machine"] {
        let target = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .find(|node| node["semantic_form"] == form)
            .expect("target")["node_id"]
            .clone();
        let body = json!({"term":"aggregate", "members":[{"term":"reference", "target":target}]});
        let key = structural_key("composite_type", "reference", None, &body);
        let id = node_id(&key);
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(json!({
                "node_id":id, "schema_version":"quire.checked-semantic-graph/v2",
                "node_tag":"composite_type", "semantic_form":"reference",
                "semantic_type":id, "dependencies":[target],
                "occurrences":[{"role":"type", "ordinal":0}], "body":body
            }));
        package["source_map"]
            .as_array_mut()
            .expect("map")
            .push(json!({
                "node_id":id, "role":"type", "ordinal":0,
                "regions":[{"source":source,"start":12,"end":13}]
            }));
    }
    refresh_identity(&mut package);
    let mut evidence = evidence_for(&package);
    evidence
        .insert_domain_package_document(digest, canonical(&document))
        .expect("document");
    (package, evidence)
}

/// Trace: FR-068-AC-1, FR-068-AC-3, QSpec FR-208-AC-5..7
#[trace("TC-463")]
#[test]
fn selected_process_machine_and_persistence_require_complete_meaning_declarations() {
    for (name, type_index, member) in [
        ("machine operations", 0, "operations"),
        ("machine states", 0, "states"),
        ("machine transitions", 0, "transitions"),
        ("process identity", 1, "identityFields"),
        ("process steps", 1, "steps"),
        ("persistence operations", 2, "operations"),
        ("persistence persists", 2, "persists"),
    ] {
        let mut document = selected_form_document();
        document["types"][type_index][member] = json!([]);
        let (package, evidence) = selected_forms_package_with(document);
        let CheckedPackageV2ReadResult::Refused(refusal) = read(&package, &evidence) else {
            panic!("{name}: incomplete selected meaning must refuse");
        };
        assert_eq!(
            refusal.code,
            CheckedPackageRefusalCode::InvalidModelBinding,
            "{name}"
        );
        assert_eq!(
            refusal.cause,
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            "{name}"
        );
        assert_eq!(
            refusal.path.as_ref().map(|path| path.as_str()),
            Some("/lock/model_selections/0"),
            "{name}"
        );
    }
    for name in [
        "process consumes object",
        "machine emits record",
        "persistence persists process",
    ] {
        let mut document = selected_form_document();
        match name {
            "process consumes object" => {
                document["types"][1]["steps"][0]["consumes"] =
                    json!([format!("ix://{MODEL_IDENTITY}/Account")])
            }
            "machine emits record" => {
                document["types"][0]["transitions"][0]["emits"] =
                    json!([format!("ix://{MODEL_IDENTITY}/Address")])
            }
            "persistence persists process" => {
                document["types"][2]["persists"] = json!([format!("ix://{MODEL_IDENTITY}/Flow")])
            }
            _ => unreachable!(),
        }
        let (package, evidence) = selected_forms_package_with(document);
        let CheckedPackageV2ReadResult::Refused(refusal) = read(&package, &evidence) else {
            panic!("{name}: wrong meaning must refuse");
        };
        assert_eq!(
            refusal.code,
            CheckedPackageRefusalCode::InvalidModelBinding,
            "{name}"
        );
        assert_eq!(
            refusal.cause,
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            "{name}"
        );
        assert_eq!(
            refusal.path.as_ref().map(|path| path.as_str()),
            Some("/lock/model_selections/0"),
            "{name}"
        );
    }
}

/// Trace: FR-068-AC-1, FR-068-AC-2, QSpec FR-322-AC-63, FR-322-AC-72
#[trace("TC-463")]
#[test]
fn selected_process_persistence_state_and_record_keep_owner_and_reference_identity() {
    let (package, evidence) = selected_forms_package();
    let result = read(&package, &evidence);
    let CheckedPackageV2ReadResult::Admitted(admitted) = result else {
        panic!("all selected forms must admit: {result:?}");
    };
    for form in [
        "process",
        "state_machine",
        "record_value_type",
        "persistence_interface",
    ] {
        let node = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .find(|node| node["semantic_form"] == form)
            .expect("selected form");
        let id: quire_contract_model::CheckedNodeId =
            serde_json::from_value(node["node_id"].clone()).expect("id");
        assert!(
            admitted.graph().nodes.iter().any(|node| node.node_id == id),
            "{form} must retain its selected key"
        );
    }
    for form in ["process", "state_machine"] {
        let target = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .find(|node| node["semantic_form"] == form)
            .expect("target")["node_id"]
            .clone();
        assert!(package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .any(|node| node["semantic_form"] == "reference"
                && node["body"]["members"][0]["target"] == target));
    }
    let encoded = writer_bytes(&package);
    assert_eq!(encoded, canonical(&package));
    let CheckedPackageV2ReadResult::Admitted(again) =
        CheckedPackageV2::read(&encoded, CheckedPackageReadLimits::bounded(), &evidence)
    else {
        panic!("first canonical reread must admit");
    };
    let second = writer_bytes(&serde_json::from_slice(&encoded).expect("canonical JSON"));
    assert_eq!(second, encoded);
    let CheckedPackageV2ReadResult::Admitted(third) =
        CheckedPackageV2::read(&second, CheckedPackageReadLimits::bounded(), &evidence)
    else {
        panic!("second canonical reread must admit");
    };
    assert_eq!(admitted.package_id(), again.package_id());
    assert_eq!(again.package_id(), third.package_id());
    for form in [
        "process",
        "state_machine",
        "record_value_type",
        "persistence_interface",
    ] {
        let node = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .find(|node| node["semantic_form"] == form)
            .expect("form");
        let id: quire_contract_model::CheckedNodeId =
            serde_json::from_value(node["node_id"].clone()).expect("id");
        assert!(
            again.graph().nodes.iter().any(|node| node.node_id == id),
            "{form}: first reread identity"
        );
        assert!(
            third.graph().nodes.iter().any(|node| node.node_id == id),
            "{form}: second reread identity"
        );
    }
}

/// Trace: FR-068-AC-3, QSpec FR-322-AC-65, FR-322-AC-72
#[trace("TC-463")]
#[test]
fn selected_form_owner_join_refuses_wrong_meaning_absence_and_missing_owner() {
    for (form, wrong) in [
        ("process", "Account"),
        ("state_machine", "Flow"),
        ("record_value_type", "Account"),
        ("persistence_interface", "Machine"),
    ] {
        for (label, owner_identity, node_identity) in [
            (
                "wrong meaning",
                MODEL_IDENTITY,
                format!("ix://{MODEL_IDENTITY}/{wrong}"),
            ),
            (
                "missing declaration",
                MODEL_IDENTITY,
                format!("ix://{MODEL_IDENTITY}/Absent"),
            ),
            (
                "unselected package",
                "other/model",
                format!("ix://{MODEL_IDENTITY}/{wrong}"),
            ),
        ] {
            let (mut package, evidence) = selected_forms_package();
            package["semantic_graph"]["nodes"]
                .as_array_mut()
                .expect("nodes")
                .retain(|node| node["semantic_form"] != "reference");
            let position = package["semantic_graph"]["nodes"]
                .as_array()
                .expect("nodes")
                .iter()
                .position(|node| node["semantic_form"] == form)
                .expect("form");
            let replacement = selected_form_node_owned(form, owner_identity, &node_identity);
            package["semantic_graph"]["nodes"][position] = replacement;
            rebuild_source_map(&mut package);
            refresh_identity(&mut package);
            let CheckedPackageV2ReadResult::Refused(refusal) = read(&package, &evidence) else {
                panic!("{form} {label} must refuse");
            };
            assert_eq!(
                refusal.code,
                CheckedPackageRefusalCode::MissingDeclaration,
                "{form} {label}"
            );
            assert_eq!(
                refusal.cause,
                Some(CheckedPackageRefusalCause::MissingSelection),
                "{form} {label}"
            );
            assert_eq!(
                refusal.path.as_ref().map(|path| path.as_str()),
                Some(format!("/semantic_graph/nodes/{position}/node_id").as_str()),
                "{form} {label}"
            );
        }
    }
    for form in [
        "process",
        "state_machine",
        "record_value_type",
        "persistence_interface",
    ] {
        let (mut package, evidence) = selected_forms_package();
        let position = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .position(|node| node["semantic_form"] == form)
            .expect("form");
        package["semantic_graph"]["nodes"][position]
            .as_object_mut()
            .expect("object")
            .remove("owner");
        refresh_identity(&mut package);
        let CheckedPackageV2ReadResult::Refused(refusal) = read(&package, &evidence) else {
            panic!("{form} missing required owner must refuse");
        };
        assert_eq!(
            refusal.code,
            CheckedPackageRefusalCode::MalformedWire,
            "{form}: {refusal:?}"
        );
    }
}

/// Trace: FR-068-AC-5, FR-064-AC-4, QSpec FR-322-AC-72
#[trace("TC-463", "TC-048")]
#[test]
fn selected_non_object_clause_owners_reach_the_clause_edge_refusal() {
    let mut document = selected_form_document();
    let machine = document["types"][0].clone();
    document["types"][0]["kind"]["name"] = json!("object");
    document["types"][1]["operations"] = json!([]);
    let mut gate = machine;
    gate["identity"] = json!(format!("ix://{MODEL_IDENTITY}/Gate"));
    gate["displayName"] = json!("Gate");
    gate["kind"]["name"] = json!("gate");
    let gate_id = format!("ix://{MODEL_IDENTITY}/Gate");
    gate["operations"][0]["identity"] = json!(format!("{gate_id}/toggle"));
    gate["states"][0]["identity"] = json!(format!("{gate_id}/closed"));
    gate["states"][1]["identity"] = json!(format!("{gate_id}/open"));
    gate["transitions"][0]["identity"] = json!(format!("{gate_id}/open-transition"));
    gate["transitions"][0]["from"] = json!(format!("{gate_id}/closed"));
    gate["transitions"][0]["to"] = json!(format!("{gate_id}/open"));
    gate["transitions"][0]["trigger"] = json!(format!("{gate_id}/toggle"));
    document["constructs"]
        .as_array_mut()
        .expect("constructs")
        .push(json!({
            "kind":{"module":MODEL_IDENTITY,"name":"gate"},
            "construct":{"meaning":"quire.meaning.model.state-machine/v1"}
        }));
    document["types"].as_array_mut().expect("types").push(gate);
    let extra = [
        ("process", "Flow"),
        ("state_machine", "Gate"),
        ("record_value_type", "Address"),
        ("persistence_interface", "Store"),
    ]
    .map(|(form, short)| selected_form_node(form, &format!("ix://{MODEL_IDENTITY}/{short}")));
    let gate = extra[1]["node_id"].clone();
    let gate_reference_body =
        json!({"term":"aggregate", "members":[{"term":"reference", "target":gate}]});
    let gate_reference = node_id(&structural_key(
        "composite_type",
        "reference",
        None,
        &gate_reference_body,
    ));
    let mut selected_nodes = extra.to_vec();
    selected_nodes.push(json!({
        "node_id":gate_reference, "schema_version":"quire.checked-semantic-graph/v2",
        "node_tag":"composite_type", "semantic_form":"reference",
        "semantic_type":gate_reference, "dependencies":[gate],
        "occurrences":[{"role":"type", "ordinal":0}], "body":gate_reference_body
    }));
    let (mut base, evidence) = concrete_protocol_refinement_with_model(
        v2_all_families(),
        ProtocolStepRows::Empty,
        ProtocolClauseOwner::SelectedModel,
        document,
        selected_nodes,
    );
    let finish = base["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["body"]["operation"]["member"]["operator"] == "finish")
        .expect("finish")["node_id"]
        .clone();
    let boolean = node_id(&crate::support::checked_package::family_key("aaaa"));
    set_concrete_catalog(
        &mut base,
        vec![
            json!({"class":"Example::OrderFlow::Closed", "node":finish, "binder_types":[boolean]}),
            json!({"class":"Gate::toggle", "node":extra[1]["node_id"], "binder_types":[gate_reference]}),
        ],
    );
    let refinement = base["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["semantic_form"] == "refinement")
        .expect("refinement");
    base["semantic_graph"]["nodes"][refinement]["body"]["class_catalog"]["abstract"] = json!([{"class":"Gate::toggle", "node":extra[1]["node_id"],
            "binder_types":[gate_reference]}]);
    rekey_refinement(&mut base, refinement);
    let control = read(&base, &evidence);
    assert!(
        matches!(control, CheckedPackageV2ReadResult::Admitted(_)),
        "selected control must admit: {control:?}"
    );
    for selected in extra {
        let form = selected["semantic_form"].as_str().expect("form");
        let mut package = base.clone();
        let clause = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .position(|node| node["semantic_form"] == "protocol_clause")
            .expect("clause");
        let clause_node = &mut package["semantic_graph"]["nodes"][clause];
        let prior_owner = clause_node["owner_model"].clone();
        let selected_owner = selected["node_id"].clone();
        clause_node["owner_model"] = selected_owner.clone();
        let dependencies = clause_node["dependencies"]
            .as_array_mut()
            .expect("dependencies");
        let owner_dependency = dependencies
            .iter_mut()
            .find(|dependency| **dependency == prior_owner)
            .expect("prior clause owner dependency");
        *owner_dependency = selected_owner;
        dependencies.sort_by(|left, right| left["digest"].as_str().cmp(&right["digest"].as_str()));
        dependencies.dedup();
        rekey_application_node(&mut package, clause);
        refresh_identity(&mut package);
        let CheckedPackageV2ReadResult::Refused(refusal) = read(&package, &evidence) else {
            panic!("selected {form} clause edge must refuse");
        };
        assert_eq!(
            refusal.code,
            CheckedPackageRefusalCode::InvalidModelBinding,
            "{form}: {refusal:?}"
        );
        assert_eq!(
            refusal.cause,
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            "{form}"
        );
        assert_eq!(
            refusal.path.as_ref().map(|path| path.as_str()),
            Some(format!("/semantic_graph/nodes/{clause}/owner_model").as_str()),
            "{form}"
        );
    }
}

pub(super) fn machine_package() -> (Value, CheckedPackageEvidence) {
    let mut package = v2_nominal();
    let document = model_document();
    let digest = sha256_hex(&canonical(&document));
    let mut evidence = evidence_for(&package);
    evidence
        .insert_domain_package_document(digest.clone(), canonical(&document))
        .expect("document");
    package["lock"]["model_selections"] = json!([{
        "identity": MODEL_IDENTITY, "digest_domain": "sha256-jcs", "digest": digest
    }]);
    let owner = json!({"kind": "model", "identity": MODEL_IDENTITY, "node": MODEL_NODE});
    let body = json!({"term": "aggregate", "members": []});
    let preimage = json!({
        "version": "quire.structural-node/v1", "node_tag": "model",
        "semantic_form": "state_machine", "semantic_type": null,
        "declaration": null, "recursion": null, "owner": owner,
        "body": body,
    });
    let id = node_id(&sha256_hex(&canonical(&preimage)));
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(json!({
            "node_id": id, "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": "model", "semantic_form": "state_machine",
            "semantic_type": id, "dependencies": [],
            "occurrences": [{"role": "type", "ordinal": 0}],
            "owner": owner, "body": body,
        }));
    let source = package["lock"]["sources"][0].clone();
    package["source_map"]
        .as_array_mut()
        .expect("source map")
        .push(json!({
            "node_id": id, "role": "type", "ordinal": 0,
            "regions": [{"source": source, "start": 10, "end": 11}]
        }));
    refresh_identity(&mut package);
    (package, evidence)
}

fn read(package: &Value, evidence: &CheckedPackageEvidence) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        evidence,
    )
}

fn invalid_value_at(result: CheckedPackageV2ReadResult, pointer: &str) {
    let CheckedPackageV2ReadResult::Refused(refusal) = result else {
        panic!("expected refusal at {pointer}");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::InvalidValue)
    );
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(pointer)
    );
}

fn writer_bytes(package: &Value) -> Vec<u8> {
    let preimage: CheckedPackageIdentityPreimageV2 =
        serde_json::from_value(package["identity_preimage"].clone()).expect("preimage");
    let package_id: CheckedSemanticId =
        serde_json::from_value(package["package_id"].clone()).expect("package id");
    let lock: CheckedPackageLockV2 = serde_json::from_value(package["lock"].clone()).expect("lock");
    let graph: CheckedSemanticGraphV2<CheckedTerm> =
        serde_json::from_value(package["semantic_graph"].clone()).expect("typed graph");
    let source_map: Vec<CheckedSourceMapEntry> =
        serde_json::from_value(package["source_map"].clone()).expect("source map");
    let rows: Option<Vec<CheckedRefinementStepRowRegion>> = package
        .get("refinement_step_rows")
        .map(|value| serde_json::from_value(value.clone()).expect("step rows"));
    let capabilities: Vec<CheckedCapability> =
        serde_json::from_value(package["capability_report"].clone()).expect("capabilities");
    let diagnostics: CheckedDiagnosticsV2 =
        serde_json::from_value(package["diagnostics"].clone()).expect("diagnostics");
    let borrowed = CheckedPackageIdentityPreimageRefV2 {
        version: &preimage.version,
        edition: &preimage.edition,
        profile_selections: &preimage.profile_selections,
        definition_selections: &preimage.definition_selections,
        model_selections: &preimage.model_selections,
        required_features: &preimage.required_features,
        dependency_selections: &preimage.dependency_selections,
        identity_projection: &graph.nodes,
    };
    encode_checked_package_v2(
        &borrowed,
        &package_id,
        &lock,
        &graph,
        quire_contract_model::CheckedPackageSourceCorrespondenceRef {
            source_map: &source_map,
            refinement_step_rows: rows.as_deref(),
        },
        &capabilities,
        &diagnostics,
        quire_canonical::Limits::new(u64::MAX),
    )
    .expect("public writer")
}

fn row_empty_refinement(package: &mut Value) {
    let model_root = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| {
            node["semantic_form"] == "model_root" && node["owner"]["identity"] == MODEL_IDENTITY
        })
        .expect("selected root")["node_id"]
        .clone();
    let profile = json!({"authority": "test", "identity": "quire.temporal.infinite-trace/v1"});
    package["lock"]["profile_selections"] = json!([{
        "role": "temporal_profile", "definition": profile
    }]);
    let body = json!({
        "term": "refinement", "abstract": model_root, "profile": profile,
        "populations": [], "objects": [], "steps": [], "histories": [],
        "assume": [], "ensure": []
    });
    let preimage = json!({
        "version": "quire.refinement-node/v1", "semantic_type": model_root, "body": body
    });
    assert_eq!(
        canonical(&preimage),
        quire_canonical::to_vec(&preimage, quire_canonical::Limits::new(u64::MAX)).expect("JCS")
    );
    let id = node_id(&sha256_hex(&canonical(&preimage)));
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(json!({
            "node_id": id, "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": "relation", "semantic_form": "refinement",
            "semantic_type": model_root, "dependencies": [model_root],
            "occurrences": [
                {"role": "declaration", "ordinal": 0},
                {"role": "generated", "ordinal": 0}
            ],
            "body": body,
        }));
    let source = package["lock"]["sources"][0].clone();
    for role in ["declaration", "generated"] {
        package["source_map"]
            .as_array_mut()
            .expect("source map")
            .push(json!({
                "node_id": id, "role": role, "ordinal": 0,
                "regions": [{"source": source, "start": 20, "end": 30}]
            }));
    }
    refresh_identity(package);
}

fn one_step_refinement_with_step(package: &mut Value, step: Value) {
    row_empty_refinement(package);
    let nodes = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    let node = nodes.last_mut().expect("refinement");
    let old_id = node["node_id"].clone();
    node["body"]["steps"] = json!([step]);
    if let Some(source) = node["body"]["steps"][0].get("source_node").cloned() {
        let mut dependencies = node["dependencies"]
            .as_array()
            .expect("dependencies")
            .clone();
        if !dependencies.contains(&source) {
            dependencies.push(source);
            dependencies
                .sort_by(|left, right| left["digest"].as_str().cmp(&right["digest"].as_str()));
        }
        node["dependencies"] = json!(dependencies);
    }
    node["occurrences"]
        .as_array_mut()
        .expect("occurrences")
        .push(json!({"role":"generated","ordinal":1}));
    let new_id = node_id(&sha256_hex(&canonical(&json!({
        "version": "quire.refinement-node/v1",
        "semantic_type": node["semantic_type"], "body": node["body"]
    }))));
    node["node_id"] = new_id.clone();
    let source = package["lock"]["sources"][0].clone();
    for entry in package["source_map"].as_array_mut().expect("source map") {
        if entry["node_id"] == old_id {
            entry["node_id"] = new_id.clone();
        }
    }
    package["source_map"]
        .as_array_mut()
        .expect("source map")
        .push(json!({
            "node_id": new_id, "role": "generated", "ordinal": 1,
            "regions": [{"source": source, "start": 31, "end": 32}]
        }));
    package["refinement_step_rows"] = json!([{
        "node_id": new_id, "ordinal": 0,
        "region": {"source": source, "start": 31, "end": 32}
    }]);
    refresh_identity(package);
}

fn one_step_refinement(package: &mut Value) {
    one_step_refinement_with_step(
        package,
        json!({"source_class": "memory flush", "target": {"kind": "stutter"}}),
    );
}

fn catalog_free_operation_package(form: &str) -> (Value, CheckedPackageEvidence, usize) {
    catalog_free_operation_package_with_inheritance(form, false)
}

fn catalog_free_operation_package_with_inheritance(
    form: &str,
    inherited: bool,
) -> (Value, CheckedPackageEvidence, usize) {
    catalog_free_operation_package_config(form, inherited, false, false)
}

fn rewrite_identity_prefix(value: &mut Value, old: &str, new: &str) {
    match value {
        Value::String(text) if text.starts_with(old) => {
            *text = format!("{new}{}", &text[old.len()..]);
        }
        Value::Array(items) => {
            for item in items {
                rewrite_identity_prefix(item, old, new);
            }
        }
        Value::Object(members) => {
            for member in members.values_mut() {
                rewrite_identity_prefix(member, old, new);
            }
        }
        _ => {}
    }
}

fn catalog_free_operation_package_config(
    form: &str,
    inherited: bool,
    inherited_field: bool,
    drop_supertype: bool,
) -> (Value, CheckedPackageEvidence, usize) {
    let mut document = selected_form_document();
    document["types"][0]["fields"] = json!([{
        "identity":format!("{MODEL_NODE}/openFlag"), "name":"openFlag",
        "typeRef":"ix://quire/native/Integer", "presence":"required",
        "nullable":false, "defaultKind":"none",
        "multiplicity":{"lower":1,"upper":1,"ordered":false,"unique":true}
    }]);
    document["types"][0]["operations"][0]["params"] = json!([{
        "identity":format!("{MODEL_NODE}/toggle/target"), "name":"target",
        "typeRef":format!("ix://{MODEL_IDENTITY}/Account"),
        "multiplicity":{"lower":1,"upper":1,"ordered":false,"unique":true}
    }]);
    document["types"][0]["operations"][0]["returns"] = json!({
        "typeRef":"ix://quire/native/Integer",
        "multiplicity":{"lower":1,"upper":1,"ordered":false,"unique":true}
    });
    document["types"][1]["operations"][0]["params"][0]["typeRef"] =
        json!(format!("ix://{MODEL_IDENTITY}/Account"));
    for (index, name) in [(0, "toggleAgain"), (1, "goAgain")] {
        let mut second = document["types"][index]["operations"][0].clone();
        let type_name = if index == 0 { "Machine" } else { "Flow" };
        second["identity"] = json!(format!("ix://{MODEL_IDENTITY}/{type_name}/{name}"));
        second["name"] = json!(name);
        second["params"][0]["identity"] = json!(format!(
            "ix://{MODEL_IDENTITY}/{type_name}/{name}/{}",
            if index == 0 { "target" } else { "step" }
        ));
        document["types"][index]["operations"]
            .as_array_mut()
            .expect("operations")
            .push(second);
    }
    if inherited_field {
        let (index, parent_name) = match form {
            "process" => (1, "BaseFlow"),
            "state_machine" => (0, "BaseMachine"),
            _ => panic!("inherited field fixture requires a process or state machine"),
        };
        let parent_identity = format!("ix://{MODEL_IDENTITY}/{parent_name}");
        let mut parent = document["types"][index].clone();
        let original_identity = parent["identity"]
            .as_str()
            .expect("parent identity")
            .to_owned();
        rewrite_identity_prefix(&mut parent, &original_identity, &parent_identity);
        parent["displayName"] = json!(parent_name);
        parent["fields"] = json!([{
            "identity":format!("{parent_identity}/inherited"), "name":"inherited",
            "typeRef":"ix://quire/native/Integer", "presence":"required",
            "nullable":false, "defaultKind":"none",
            "multiplicity":{"lower":1,"upper":1,"ordered":false,"unique":true}
        }]);
        if form == "process" {
            parent["identityFields"] = json!([format!("{parent_identity}/inherited")]);
            parent["operations"][0]["identity"] = json!(format!("{parent_identity}/baseGo"));
            parent["operations"][0]["name"] = json!("baseGo");
            parent["operations"][0]["params"][0]["identity"] =
                json!(format!("{parent_identity}/baseGo/step"));
        } else {
            parent
                .as_object_mut()
                .expect("parent")
                .remove("identityFields");
            parent["operations"][0]["identity"] = json!(format!("{parent_identity}/baseToggle"));
            parent["operations"][0]["name"] = json!("baseToggle");
            parent["operations"][0]["params"][0]["identity"] =
                json!(format!("{parent_identity}/baseToggle/target"));
            parent["transitions"][0]["trigger"] = json!(format!("{parent_identity}/baseToggle"));
        }
        document["types"][index]["supertypes"] = json!([parent_identity]);
        document["types"]
            .as_array_mut()
            .expect("types")
            .push(parent);
    }
    document["types"][4]["operations"] = json!([{
        "identity": format!("ix://{MODEL_IDENTITY}/Account/save"),
        "name": "save", "params": []
    }]);
    let child = if inherited {
        let (index, parent_name, child_name) = match form {
            "object_type" => (4, "Account", "SubAccount"),
            "process" => (1, "Flow", "SubFlow"),
            "state_machine" => (0, "Machine", "SubMachine"),
            _ => panic!("unsupported inherited model form"),
        };
        let mut child = document["types"][index].clone();
        let parent_identity = format!("ix://{MODEL_IDENTITY}/{parent_name}");
        let child_identity = format!("ix://{MODEL_IDENTITY}/{child_name}");
        rewrite_identity_prefix(&mut child, &parent_identity, &child_identity);
        child["displayName"] = json!(child_name);
        child["supertypes"] = if drop_supertype {
            json!([])
        } else {
            json!([parent_identity])
        };
        if form == "process" {
            child["fields"] = json!([{
                "identity":format!("{child_identity}/subId"), "name":"subId",
                "typeRef":"ix://quire/native/Integer", "presence":"required",
                "nullable":false, "defaultKind":"none",
                "multiplicity":{"lower":1,"upper":1,"ordered":false,"unique":true}
            }]);
            child["identityFields"] = json!([format!("{child_identity}/subId")]);
            child["operations"][0]["identity"] = json!(format!("{child_identity}/localGo"));
            child["operations"][0]["name"] = json!("localGo");
            child["operations"][0]["params"][0]["identity"] =
                json!(format!("{child_identity}/localGo/step"));
        } else if form == "state_machine" {
            child["fields"] = json!([]);
            child["operations"][0]["identity"] = json!(format!("{child_identity}/localToggle"));
            child["operations"][0]["name"] = json!("localToggle");
            child["operations"][0]["params"][0]["identity"] =
                json!(format!("{child_identity}/localToggle/target"));
            child["transitions"][0]["trigger"] = json!(format!("{child_identity}/localToggle"));
        } else {
            child["fields"] = json!([]);
            child["operations"] = json!([]);
            child
                .as_object_mut()
                .expect("subtype")
                .remove("identityFields");
        }
        document["types"].as_array_mut().expect("types").push(child);
        Some(selected_form_node(
            form,
            &format!("ix://{MODEL_IDENTITY}/{child_name}"),
        ))
    } else {
        None
    };
    let (mut package, evidence) = selected_forms_package_with(document);
    let child_id = child.as_ref().map(|node| node["node_id"].clone());
    let first_added = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    if let Some(child) = &child {
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(child.clone());
    }
    let declaring = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["semantic_form"] == form)
        .expect("declaring type")["node_id"]
        .clone();
    let operation = match form {
        "object_type" => "save",
        "process" => "go",
        "state_machine" => "toggle",
        _ => panic!("unsupported model form"),
    };
    let context = declaring["digest"].as_str().expect("context digest");
    let frame_body = json!({
        "term":"frame", "modifies":[], "creates":[], "deletes":[], "deletes_scoped":[]
    });
    let frame = node_id(&structural_key(
        "state",
        "frame",
        Some(context),
        &frame_body,
    ));
    let text_type = node_id(&crate::support::checked_package::family_key("a1a1"));
    let text_body = json!({"term":"aggregate", "members":[]});
    let anchor_body = json!({"term":"aggregate", "members":[
        {"term":"binding", "name":"context", "value":{"term":"reference", "target":declaring}},
        {"term":"binding", "name":"operation", "value":{"term":"literal", "type":text_type,
            "value_kind":"text", "value":operation}},
        {"term":"binding", "name":"frame", "value":{"term":"reference", "target":frame}}
    ]});
    let anchor = node_id(&structural_key(
        "state",
        "operation_anchor",
        Some(context),
        &anchor_body,
    ));
    let mut dependencies = vec![declaring.clone(), frame.clone()];
    dependencies.sort_by(|left, right| left["digest"].as_str().cmp(&right["digest"].as_str()));
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend([
            json!({"node_id":text_type, "schema_version":"quire.checked-semantic-graph/v2",
                "node_tag":"scalar_type", "semantic_form":"text", "semantic_type":text_type,
                "dependencies":[], "occurrences":[{"role":"type","ordinal":0}], "body":text_body}),
            json!({"node_id":frame, "schema_version":"quire.checked-semantic-graph/v2",
                "node_tag":"state", "semantic_form":"frame", "semantic_type":declaring,
                "dependencies":[], "occurrences":[{"role":"generated","ordinal":0}], "body":frame_body}),
            json!({"node_id":anchor, "schema_version":"quire.checked-semantic-graph/v2",
                "node_tag":"state", "semantic_form":"operation_anchor", "semantic_type":declaring,
                "dependencies":dependencies, "occurrences":[{"role":"anchor","ordinal":0}],
                "body":anchor_body}),
        ]);
    let source = package["lock"]["sources"][0].clone();
    let additions = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")[first_added..]
        .to_vec();
    for node in additions {
        let occurrence = &node["occurrences"][0];
        package["source_map"].as_array_mut().expect("source map").push(json!({
            "node_id":node["node_id"], "role":occurrence["role"], "ordinal":occurrence["ordinal"],
            "regions":[{"source":source,"start":10,"end":11}]
        }));
    }
    refresh_identity(&mut package);
    one_step_refinement_with_step(
        &mut package,
        json!({
            "source_class":operation, "source_node":anchor, "target":{"kind":"stutter"}
        }),
    );
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
    if let Some(child) = child.filter(|_| form == "object_type") {
        let child = child["node_id"].clone();
        let node = &mut package["semantic_graph"]["nodes"][position];
        node["body"]["objects"] = json!([{
            "abstract_type":child, "concrete_type":child, "fields":[]
        }]);
        let dependencies = node["dependencies"].as_array_mut().expect("dependencies");
        if !dependencies.contains(&child) {
            dependencies.push(child);
            dependencies
                .sort_by(|left, right| left["digest"].as_str().cmp(&right["digest"].as_str()));
        }
        rekey_refinement(&mut package, position);
        let node = &mut package["semantic_graph"]["nodes"][position];
        node["occurrences"]
            .as_array_mut()
            .expect("occurrences")
            .retain(|entry| entry["role"] == "declaration");
        let refinement_id = node["node_id"].clone();
        package["source_map"]
            .as_array_mut()
            .expect("source map")
            .retain(|entry| entry["node_id"] != refinement_id || entry["role"] == "declaration");
        refresh_identity(&mut package);
    }
    if let Some(child_id) = child_id {
        let inherited_name = match form {
            "object_type" => "address",
            "process" => "n",
            "state_machine" => "openFlag",
            _ => panic!("unsupported inherited model form"),
        };
        edit_catalog_free_frame(&mut package, position, |frame| {
            frame["dependencies"] = json!([child_id]);
            frame["body"]["modifies"] = json!([{
                "kind":"field", "declaration":child_id,
                "name":inherited_name, "scope":"every-object"
            }]);
        });
    }
    (package, evidence, position)
}

fn edit_catalog_free_frame(
    package: &mut Value,
    refinement: usize,
    edit: impl FnOnce(&mut Value),
) -> usize {
    let frame = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_tag"] == "state" && node["semantic_form"] == "frame")
        .expect("frame");
    let old_frame = package["semantic_graph"]["nodes"][frame]["node_id"]["digest"]
        .as_str()
        .expect("frame digest")
        .to_owned();
    edit(&mut package["semantic_graph"]["nodes"][frame]);
    let context = package["semantic_graph"]["nodes"][frame]["semantic_type"]["digest"]
        .as_str()
        .expect("context digest")
        .to_owned();
    let new_frame = structural_key(
        "state",
        "frame",
        Some(&context),
        &package["semantic_graph"]["nodes"][frame]["body"],
    );
    crate::support::checked_package::rename_node(package, &old_frame, &new_frame);
    let anchor = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_tag"] == "state" && node["semantic_form"] == "operation_anchor")
        .expect("anchor");
    let old_anchor = package["semantic_graph"]["nodes"][anchor]["node_id"]["digest"]
        .as_str()
        .expect("anchor digest")
        .to_owned();
    let new_anchor = structural_key(
        "state",
        "operation_anchor",
        Some(&context),
        &package["semantic_graph"]["nodes"][anchor]["body"],
    );
    crate::support::checked_package::rename_node(package, &old_anchor, &new_anchor);
    for node in package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
    {
        node["dependencies"]
            .as_array_mut()
            .expect("dependencies")
            .sort_by(|left, right| left["digest"].as_str().cmp(&right["digest"].as_str()));
    }
    rekey_refinement(package, refinement);
    frame
}

fn rebind_catalog_free_anchor(
    package: &mut Value,
    refinement: usize,
    context: &Value,
    operation: &str,
) -> usize {
    let anchor = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["semantic_form"] == "operation_anchor")
        .expect("anchor");
    let old = package["semantic_graph"]["nodes"][anchor]["node_id"]["digest"]
        .as_str()
        .expect("anchor digest")
        .to_owned();
    let node = &mut package["semantic_graph"]["nodes"][anchor];
    node["semantic_type"] = context.clone();
    node["body"]["members"][0]["value"]["target"] = context.clone();
    node["body"]["members"][1]["value"]["value"] = json!(operation);
    let mut dependencies = vec![
        context.clone(),
        node["body"]["members"][2]["value"]["target"].clone(),
    ];
    dependencies.sort_by(|left, right| left["digest"].as_str().cmp(&right["digest"].as_str()));
    node["dependencies"] = json!(dependencies);
    let new = structural_key(
        "state",
        "operation_anchor",
        context["digest"].as_str(),
        &node["body"],
    );
    crate::support::checked_package::rename_node(package, &old, &new);
    for node in package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
    {
        node["dependencies"]
            .as_array_mut()
            .expect("dependencies")
            .sort_by(|left, right| left["digest"].as_str().cmp(&right["digest"].as_str()));
    }
    rekey_refinement(package, refinement);
    anchor
}

#[trace("TC-056", "FR-040-AC-39")]
#[test]
fn process_and_machine_anchors_refuse_inherited_only_and_cross_type_frames() {
    for (form, own, inherited_name) in [
        ("process", "localGo", "go"),
        ("state_machine", "localToggle", "toggle"),
    ] {
        let (base, evidence, refinement) =
            catalog_free_operation_package_with_inheritance(form, true);
        let child = base["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .find(|node| {
                node["semantic_form"] == form
                    && node["owner"]["node"]
                        .as_str()
                        .is_some_and(|name| name.contains("/Sub"))
            })
            .expect("subtype")["node_id"]
            .clone();
        let mut cross_type = base.clone();
        let anchor = rebind_catalog_free_anchor(&mut cross_type, refinement, &child, own);
        assert_step_refusal(
            read(&cross_type, &evidence),
            CheckedPackageRefusalCode::InvalidModelBinding,
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            &format!("/semantic_graph/nodes/{anchor}"),
        );

        let mut inherited_only = base;
        edit_catalog_free_frame(&mut inherited_only, refinement, |frame| {
            frame["semantic_type"] = child.clone();
        });
        let anchor =
            rebind_catalog_free_anchor(&mut inherited_only, refinement, &child, inherited_name);
        assert_step_refusal(
            read(&inherited_only, &evidence),
            CheckedPackageRefusalCode::InvalidModelBinding,
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            &format!("/semantic_graph/nodes/{anchor}/body/members/1"),
        );
    }
}

#[trace("TC-056", "FR-040-AC-39")]
#[test]
fn two_own_operations_on_process_or_machine_share_one_frame() {
    for (form, other_operation) in [("process", "goAgain"), ("state_machine", "toggleAgain")] {
        let (mut package, evidence, _) = catalog_free_operation_package(form);
        let original = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .find(|node| node["semantic_form"] == "operation_anchor")
            .expect("anchor")
            .clone();
        let mut second = original.clone();
        second["body"]["members"][1]["value"]["value"] = json!(other_operation);
        let context = second["semantic_type"]["digest"].as_str().expect("context");
        second["node_id"] = node_id(&structural_key(
            "state",
            "operation_anchor",
            Some(context),
            &second["body"],
        ));
        assert_eq!(
            original["body"]["members"][2]["value"], second["body"]["members"][2]["value"],
            "the two anchors name the same frame"
        );
        let source = package["lock"]["sources"][0].clone();
        package["source_map"]
            .as_array_mut()
            .expect("source map")
            .push(json!({
                "node_id":second["node_id"], "role":"anchor", "ordinal":0,
                "regions":[{"source":source,"start":10,"end":11}]
            }));
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(second);
        refresh_identity(&mut package);
        let result = read(&package, &evidence);
        assert!(
            matches!(result, CheckedPackageV2ReadResult::Admitted(_)),
            "{form}: {result:?}"
        );
    }
}

#[trace("TC-056", "FR-040-AC-39")]
#[test]
fn process_and_machine_anchors_refuse_non_object_context_forms() {
    for form in ["process", "state_machine"] {
        let (base, evidence, refinement) = catalog_free_operation_package(form);
        for rejected in [
            "record_value_type",
            "event_type",
            "persistence_interface",
            "systems_interface",
        ] {
            let mut package = base.clone();
            let mut evidence = evidence.clone();
            let context = match rejected {
                "event_type" => {
                    let body = json!({"term":"aggregate","members":[]});
                    let id = node_id(&structural_key("model", "event_type", None, &body));
                    package["semantic_graph"]["nodes"].as_array_mut().expect("nodes").push(json!({
                        "node_id":id,"schema_version":"quire.checked-semantic-graph/v2",
                        "node_tag":"model","semantic_form":"event_type","semantic_type":id,
                        "dependencies":[],"occurrences":[{"role":"type","ordinal":0}],"body":body
                    }));
                    id
                }
                "systems_interface" => {
                    let document = crate::support::checked_package::family_model_document();
                    evidence
                        .insert_domain_package_document(
                            sha256_hex(&canonical(&document)),
                            canonical(&document),
                        )
                        .expect("family evidence");
                    let family = v2_all_families();
                    package["lock"]["model_selections"]
                        .as_array_mut()
                        .expect("selections")
                        .push(family["lock"]["model_selections"][0].clone());
                    package["lock"]["model_selections"]
                        .as_array_mut()
                        .expect("selections")
                        .sort_by(|left, right| {
                            left["identity"].as_str().cmp(&right["identity"].as_str())
                        });
                    let interface =
                        crate::support::checked_package::selected_interface_model_node();
                    let id = interface["node_id"].clone();
                    package["semantic_graph"]["nodes"]
                        .as_array_mut()
                        .expect("nodes")
                        .push(interface);
                    id
                }
                _ => package["semantic_graph"]["nodes"]
                    .as_array()
                    .expect("nodes")
                    .iter()
                    .find(|node| node["semantic_form"] == rejected)
                    .expect("rejected context")["node_id"]
                    .clone(),
            };
            if matches!(rejected, "event_type" | "systems_interface") {
                let source = package["lock"]["sources"][0].clone();
                package["source_map"]
                    .as_array_mut()
                    .expect("source map")
                    .push(json!({
                        "node_id":context,"role":"type","ordinal":0,
                        "regions":[{"source":source,"start":10,"end":11}]
                    }));
                refresh_identity(&mut package);
            }
            let anchor = rebind_catalog_free_anchor(&mut package, refinement, &context, "missing");
            assert_step_refusal(
                read(&package, &evidence),
                CheckedPackageRefusalCode::InvalidModelBinding,
                Some(CheckedPackageRefusalCause::MalformedDeclaration),
                &format!("/semantic_graph/nodes/{anchor}/body/members/0/value/target"),
            );
        }
    }
}

#[trace("TC-056", "TC-062", "FR-061-AC-14", "FR-040-AC-38", "FR-040-AC-39")]
#[test]
fn catalog_free_process_and_machine_frames_admit_declared_fields_and_scoped_parameters() {
    for (form, field_name, parameter_name) in [
        ("process", "n", "step"),
        ("state_machine", "openFlag", "target"),
    ] {
        let (mut package, evidence, refinement) =
            catalog_free_operation_package_config(form, false, true, false);
        let declaring = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .find(|node| node["node_tag"] == "model" && node["semantic_form"] == form)
            .expect("declaring node")["node_id"]
            .clone();
        let frame = edit_catalog_free_frame(&mut package, refinement, |node| {
            node["dependencies"] = json!([declaring]);
            let mut fields = [field_name, "inherited"];
            fields.sort();
            node["body"]["modifies"] = json!(fields
                .iter()
                .map(|name| json!({
                    "kind":"field", "declaration":declaring,
                    "name":name, "scope":"every-object"
                }))
                .collect::<Vec<_>>());
            node["body"]["deletes_scoped"] = json!([{
                "kind":"parameter", "name":parameter_name
            }]);
            if form == "process" {
                node["body"]["creates"] = json!([declaring]);
                node["body"]["deletes"] = json!([declaring]);
            }
        });
        let first = read(&package, &evidence);
        let CheckedPackageV2ReadResult::Admitted(first) = first else {
            panic!("{form} frame must admit: {first:?}");
        };
        let encoded = writer_bytes(&package);
        let CheckedPackageV2ReadResult::Admitted(second) =
            CheckedPackageV2::read(&encoded, CheckedPackageReadLimits::bounded(), &evidence)
        else {
            panic!("{form} frame writer must read back");
        };
        assert_eq!(first.package_id(), second.package_id(), "{form}");
        assert_eq!(
            serde_json::to_value(&first.graph().nodes[frame].body).expect("frame body"),
            serde_json::to_value(&second.graph().nodes[frame].body).expect("roundtrip frame"),
            "{form}"
        );
    }
}

#[trace("TC-056", "TC-062", "FR-040-AC-38", "FR-040-AC-39")]
#[test]
fn process_and_machine_frames_refuse_receiver_scopes_and_invalid_members() {
    for (form, field_name) in [("process", "n"), ("state_machine", "openFlag")] {
        let (base, evidence, refinement) = catalog_free_operation_package(form);
        let declaring = base["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .find(|node| node["node_tag"] == "model" && node["semantic_form"] == form)
            .expect("declaring node")["node_id"]
            .clone();
        let mut receiver_field = base.clone();
        let frame = edit_catalog_free_frame(&mut receiver_field, refinement, |node| {
            node["dependencies"] = json!([declaring]);
            node["body"]["modifies"] = json!([{
                "kind":"field", "declaration":declaring,
                "name":field_name, "scope":"receiver"
            }]);
        });
        assert_step_refusal(
            read(&receiver_field, &evidence),
            CheckedPackageRefusalCode::InvalidModelBinding,
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            &format!("/semantic_graph/nodes/{frame}/body/modifies/0"),
        );

        let mut receiver_delete = base.clone();
        let frame = edit_catalog_free_frame(&mut receiver_delete, refinement, |node| {
            node["body"]["deletes_scoped"] = json!([{"kind":"receiver"}]);
        });
        assert_step_refusal(
            read(&receiver_delete, &evidence),
            CheckedPackageRefusalCode::InvalidModelBinding,
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            &format!("/semantic_graph/nodes/{frame}"),
        );

        let mut unordered_delete = base.clone();
        let frame = edit_catalog_free_frame(&mut unordered_delete, refinement, |node| {
            node["body"]["deletes_scoped"] = json!([
                {"kind":"parameter", "name":"unknown"},
                {"kind":"receiver"}
            ]);
        });
        assert_step_refusal(
            read(&unordered_delete, &evidence),
            CheckedPackageRefusalCode::InvalidModelBinding,
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            &format!("/semantic_graph/nodes/{frame}"),
        );

        let mut ghost_field = base.clone();
        let frame = edit_catalog_free_frame(&mut ghost_field, refinement, |node| {
            node["dependencies"] = json!([declaring]);
            node["body"]["modifies"] = json!([{
                "kind":"field", "declaration":declaring,
                "name":"ghost", "scope":"every-object"
            }]);
        });
        assert_step_refusal(
            read(&ghost_field, &evidence),
            CheckedPackageRefusalCode::MissingDeclaration,
            Some(CheckedPackageRefusalCause::MissingName),
            &format!("/semantic_graph/nodes/{frame}/body/modifies/0"),
        );

        if form == "state_machine" {
            for member in ["creates", "deletes"] {
                let mut package = base.clone();
                let frame = edit_catalog_free_frame(&mut package, refinement, |node| {
                    node["dependencies"] = json!([declaring]);
                    node["body"][member] = json!([declaring]);
                });
                assert_step_refusal(
                    read(&package, &evidence),
                    CheckedPackageRefusalCode::InvalidModelBinding,
                    Some(CheckedPackageRefusalCause::MalformedDeclaration),
                    &format!("/semantic_graph/nodes/{frame}/body/{member}/0"),
                );
            }
        }
    }
}

#[trace("TC-056", "TC-062", "FR-040-AC-38", "FR-040-AC-39")]
#[test]
fn object_frame_retains_receiver_scoped_grants() {
    let (mut package, evidence, refinement) = catalog_free_operation_package("object_type");
    let model_nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    let declaring = |form| {
        model_nodes
            .iter()
            .find(|node| node["node_tag"] == "model" && node["semantic_form"] == form)
            .expect("declaring node")["node_id"]
            .clone()
    };
    let object = declaring("object_type");
    let process = declaring("process");
    let machine = declaring("state_machine");
    edit_catalog_free_frame(&mut package, refinement, |node| {
        let mut declarations = vec![object.clone(), process.clone(), machine.clone()];
        declarations.sort_by(|left, right| left["digest"].as_str().cmp(&right["digest"].as_str()));
        let mut fields = [
            (object, "address", "receiver"),
            (process, "n", "every-object"),
            (machine, "openFlag", "every-object"),
        ];
        fields.sort_by(|left, right| {
            left.0["digest"]
                .as_str()
                .cmp(&right.0["digest"].as_str())
                .then(left.1.cmp(right.1))
        });
        node["dependencies"] = json!(declarations);
        node["body"]["modifies"] = json!(fields
            .iter()
            .map(|(declaration, name, scope)| json!({
                "kind":"field", "declaration":declaration,
                "name":name, "scope":scope
            }))
            .collect::<Vec<_>>());
        node["body"]["deletes_scoped"] = json!([{"kind":"receiver"}]);
    });
    let result = read(&package, &evidence);
    assert!(
        matches!(result, CheckedPackageV2ReadResult::Admitted(_)),
        "{result:?}"
    );
}

#[trace("TC-062", "FR-061-AC-14")]
#[test]
fn catalog_free_operation_step_joins_selected_object_type() {
    for (form, inherited) in [
        ("object_type", false),
        ("object_type", true),
        ("process", false),
        ("process", true),
        ("state_machine", false),
        ("state_machine", true),
    ] {
        let (package, evidence, position) =
            catalog_free_operation_package_with_inheritance(form, inherited);
        let CheckedPackageV2ReadResult::Admitted(first) = read(&package, &evidence) else {
            panic!(
                "{form} declaring operation must admit: {:?}",
                read(&package, &evidence)
            );
        };
        if inherited {
            let child = package["semantic_graph"]["nodes"]
                .as_array()
                .expect("nodes")
                .iter()
                .find(|node| {
                    node["semantic_form"] == form
                        && node["owner"]["node"]
                            .as_str()
                            .is_some_and(|name| name.contains("/Sub"))
                })
                .expect("selected subtype");
            let anchor = package["semantic_graph"]["nodes"]
                .as_array()
                .expect("nodes")
                .iter()
                .find(|node| node["semantic_form"] == "operation_anchor")
                .expect("source anchor");
            assert_ne!(
                anchor["body"]["members"][0]["value"]["target"], child["node_id"],
                "inherited {form} operation stays anchored at its declaring type"
            );
        }
        let encoded = writer_bytes(&package);
        let CheckedPackageV2ReadResult::Admitted(second) =
            CheckedPackageV2::read(&encoded, CheckedPackageReadLimits::bounded(), &evidence)
        else {
            panic!("{form} public writer must read back");
        };
        assert_eq!(
            first.package_id(),
            second.package_id(),
            "{form} inherited={inherited}"
        );
        assert_eq!(
            serde_json::to_value(&first.graph().nodes[position].body).expect("row body"),
            serde_json::to_value(&second.graph().nodes[position].body).expect("roundtrip body"),
            "{form}"
        );
        if inherited {
            let (without_supertype, evidence, _) =
                catalog_free_operation_package_config(form, true, false, true);
            let frame = without_supertype["semantic_graph"]["nodes"]
                .as_array()
                .expect("nodes")
                .iter()
                .position(|node| node["node_tag"] == "state" && node["semantic_form"] == "frame")
                .expect("inherited frame");
            assert_step_refusal(
                read(&without_supertype, &evidence),
                CheckedPackageRefusalCode::MissingDeclaration,
                Some(CheckedPackageRefusalCause::MissingName),
                &format!("/semantic_graph/nodes/{frame}/body/modifies/0"),
            );
        }
    }
}

fn set_catalog_free_source(package: &mut Value, position: usize, source: Option<Value>) {
    let node = &mut package["semantic_graph"]["nodes"][position];
    let step = node["body"]["steps"][0].as_object_mut().expect("step");
    if let Some(source) = &source {
        step.insert("source_node".into(), source.clone());
    } else {
        step.remove("source_node");
    }
    let mut dependencies = vec![node["body"]["abstract"].clone()];
    if let Some(source) = source {
        dependencies.push(source);
    }
    dependencies.sort_by(|left, right| left["digest"].as_str().cmp(&right["digest"].as_str()));
    dependencies.dedup();
    node["dependencies"] = json!(dependencies);
    rekey_refinement(package, position);
}

fn assert_step_refusal(
    result: CheckedPackageV2ReadResult,
    code: CheckedPackageRefusalCode,
    cause: Option<CheckedPackageRefusalCause>,
    path: &str,
) {
    let CheckedPackageV2ReadResult::Refused(refusal) = result else {
        panic!("step must refuse at {path}");
    };
    assert_eq!(refusal.code, code, "{refusal:?}");
    assert_eq!(refusal.cause, cause, "{refusal:?}");
    assert_eq!(
        refusal.path.as_ref().map(|pointer| pointer.as_str()),
        Some(path),
        "{refusal:?}"
    );
}

#[trace("TC-062", "FR-061-AC-15")]
#[test]
fn catalog_free_operation_step_refuses_missing_wrong_kind_and_wrong_name() {
    let (baseline, evidence, position) = catalog_free_operation_package("object_type");
    let at = format!("/semantic_graph/nodes/{position}/body/steps/0");
    let mut missing = baseline.clone();
    set_catalog_free_source(&mut missing, position, None);
    assert_step_refusal(
        read(&missing, &evidence),
        CheckedPackageRefusalCode::InvalidSemanticGraph,
        None,
        &at,
    );

    let mut unresolved = baseline.clone();
    set_catalog_free_source(&mut unresolved, position, Some(node_id(&"ef".repeat(32))));
    assert_step_refusal(
        read(&unresolved, &evidence),
        CheckedPackageRefusalCode::MissingDeclaration,
        Some(CheckedPackageRefusalCause::MissingName),
        &format!("{at}/source_node"),
    );

    let mut wrong_kind = baseline.clone();
    let model_root = wrong_kind["semantic_graph"]["nodes"][position]["semantic_type"].clone();
    set_catalog_free_source(&mut wrong_kind, position, Some(model_root));
    assert_step_refusal(
        read(&wrong_kind, &evidence),
        CheckedPackageRefusalCode::InvalidModelBinding,
        Some(CheckedPackageRefusalCause::MalformedDeclaration),
        &format!("{at}/source_node"),
    );

    let mut wrong_name = baseline;
    wrong_name["semantic_graph"]["nodes"][position]["body"]["steps"][0]["source_class"] =
        json!("other");
    rekey_refinement(&mut wrong_name, position);
    assert_step_refusal(
        read(&wrong_name, &evidence),
        CheckedPackageRefusalCode::InvalidModelBinding,
        Some(CheckedPackageRefusalCause::MalformedDeclaration),
        &format!("{at}/source_class"),
    );
}

#[trace("TC-062", "FR-061-AC-15")]
#[test]
fn catalog_free_operation_step_refuses_foreign_selected_model_anchor() {
    let (mut package, mut evidence, position) = catalog_free_operation_package("object_type");
    let foreign_model = crate::support::checked_package::family_model_document();
    let foreign_digest = sha256_hex(&canonical(&foreign_model));
    evidence
        .insert_domain_package_document(foreign_digest, canonical(&foreign_model))
        .expect("selected foreign model evidence");
    let first_added = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    let account = crate::support::checked_package::add_family_reference_target(&mut package);
    package["lock"]["model_selections"]
        .as_array_mut()
        .expect("model selections")
        .sort_by(|left, right| left["identity"].as_str().cmp(&right["identity"].as_str()));
    assert_eq!(
        package["lock"]["model_selections"]
            .as_array()
            .expect("selections")
            .len(),
        2
    );
    assert_eq!(
        package["semantic_graph"]["nodes"][position]["semantic_type"],
        package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .find(|node| node["semantic_form"] == "model_root"
                && node["owner"]["identity"] == MODEL_IDENTITY)
            .expect("concrete selected root")["node_id"]
    );
    let context = node_id(&account);
    let frame_body = json!({
        "term":"frame", "modifies":[], "creates":[], "deletes":[], "deletes_scoped":[]
    });
    let frame = node_id(&structural_key(
        "state",
        "frame",
        Some(&account),
        &frame_body,
    ));
    let text_type = node_id(&crate::support::checked_package::family_key("a1a1"));
    let anchor_body = json!({"term":"aggregate", "members":[
        {"term":"binding", "name":"context", "value":{"term":"reference", "target":context}},
        {"term":"binding", "name":"operation", "value":{"term":"literal", "type":text_type,
            "value_kind":"text", "value":"deposit"}},
        {"term":"binding", "name":"frame", "value":{"term":"reference", "target":frame}}
    ]});
    let anchor = node_id(&structural_key(
        "state",
        "operation_anchor",
        Some(&account),
        &anchor_body,
    ));
    let mut dependencies = vec![context.clone(), frame.clone()];
    dependencies.sort_by(|left, right| left["digest"].as_str().cmp(&right["digest"].as_str()));
    package["semantic_graph"]["nodes"].as_array_mut().expect("nodes").extend([
        json!({"node_id":frame, "schema_version":"quire.checked-semantic-graph/v2",
            "node_tag":"state", "semantic_form":"frame", "semantic_type":context,
            "dependencies":[], "occurrences":[{"role":"generated","ordinal":0}], "body":frame_body}),
        json!({"node_id":anchor, "schema_version":"quire.checked-semantic-graph/v2",
            "node_tag":"state", "semantic_form":"operation_anchor", "semantic_type":context,
            "dependencies":dependencies, "occurrences":[{"role":"anchor","ordinal":0}],
            "body":anchor_body}),
    ]);
    let source = package["lock"]["sources"][0].clone();
    let additions = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")[first_added..]
        .to_vec();
    for node in additions {
        let occurrence = &node["occurrences"][0];
        package["source_map"].as_array_mut().expect("source map").push(json!({
            "node_id":node["node_id"], "role":occurrence["role"], "ordinal":occurrence["ordinal"],
            "regions":[{"source":source,"start":10,"end":11}]
        }));
    }
    set_catalog_free_source(&mut package, position, Some(anchor));
    assert_step_refusal(
        read(&package, &evidence),
        CheckedPackageRefusalCode::InvalidModelBinding,
        Some(CheckedPackageRefusalCause::MalformedDeclaration),
        &format!("/semantic_graph/nodes/{position}/body/steps/0/source_node"),
    );
}

#[trace("TC-062", "FR-061-AC-16")]
#[test]
fn catalog_free_operation_step_respects_earlier_reader_stages() {
    let (mut mismatch, evidence, position) = catalog_free_operation_package("object_type");
    let at = format!("/semantic_graph/nodes/{position}");
    mismatch["semantic_graph"]["nodes"][position]["body"]["steps"][0]["source_class"] =
        json!("other");
    rekey_refinement(&mut mismatch, position);
    let row_path = format!("{at}/body/steps/0/source_class");
    let assert_row = |package: &Value| {
        assert_step_refusal(
            read(package, &evidence),
            CheckedPackageRefusalCode::InvalidModelBinding,
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            &row_path,
        );
    };
    assert_row(&mismatch);

    let mut wire = mismatch.clone();
    wire["semantic_graph"]["nodes"][position]["body"]["extra"] = json!(true);
    rekey_refinement(&mut wire, position);
    assert_step_refusal(
        read(&wire, &evidence),
        CheckedPackageRefusalCode::InvalidSemanticGraph,
        None,
        &format!("{at}/body"),
    );
    wire["semantic_graph"]["nodes"][position]["body"]
        .as_object_mut()
        .expect("body")
        .remove("extra");
    rekey_refinement(&mut wire, position);
    assert_row(&wire);

    let mut stale = mismatch.clone();
    stale["semantic_graph"]["nodes"][position]["body"]["steps"][0]["source_class"] =
        json!("another");
    refresh_identity(&mut stale);
    assert_step_refusal(
        read(&stale, &evidence),
        CheckedPackageRefusalCode::InvalidPackage,
        Some(CheckedPackageRefusalCause::StaleNodeKey),
        &format!("{at}/node_id"),
    );
    stale["semantic_graph"]["nodes"][position]["body"]["steps"][0]["source_class"] = json!("other");
    refresh_identity(&mut stale);
    assert_row(&stale);

    let mut header = mismatch.clone();
    header["semantic_graph"]["nodes"][position]["body"]["profile"]["identity"] =
        json!("quire.temporal.unselected/v1");
    rekey_refinement(&mut header, position);
    assert_step_refusal(
        read(&header, &evidence),
        CheckedPackageRefusalCode::InvalidModelBinding,
        Some(CheckedPackageRefusalCause::MalformedDeclaration),
        &format!("{at}/body/profile"),
    );
    header["semantic_graph"]["nodes"][position]["body"]["profile"]["identity"] =
        json!("quire.temporal.infinite-trace/v1");
    rekey_refinement(&mut header, position);
    assert_row(&header);

    let mut catalog = mismatch;
    catalog["semantic_graph"]["nodes"][position]["body"]["class_catalog"] =
        json!({"concrete":[], "abstract":[]});
    rekey_refinement(&mut catalog, position);
    assert_step_refusal(
        read(&catalog, &evidence),
        CheckedPackageRefusalCode::InvalidSemanticGraph,
        None,
        &format!("{at}/body/class_catalog"),
    );
    catalog["semantic_graph"]["nodes"][position]["body"]
        .as_object_mut()
        .expect("body")
        .remove("class_catalog");
    rekey_refinement(&mut catalog, position);
    assert_row(&catalog);
}

fn append_catalog_free_step(package: &mut Value, position: usize, class: &str) {
    let first = package["semantic_graph"]["nodes"][position]["body"]["steps"][0].clone();
    let mut second = first;
    second["source_class"] = json!(class);
    let node = &mut package["semantic_graph"]["nodes"][position];
    node["body"]["steps"]
        .as_array_mut()
        .expect("steps")
        .push(second);
    node["occurrences"]
        .as_array_mut()
        .expect("occurrences")
        .push(json!({
            "role":"generated", "ordinal":2
        }));
    rekey_refinement(package, position);
    let id = package["semantic_graph"]["nodes"][position]["node_id"].clone();
    let source = package["lock"]["sources"][0].clone();
    package["source_map"]
        .as_array_mut()
        .expect("source map")
        .push(json!({
            "node_id":id, "role":"generated", "ordinal":2,
            "regions":[{"source":source,"start":31,"end":32}]
        }));
    package["refinement_step_rows"]
        .as_array_mut()
        .expect("step rows")
        .push(json!({
            "node_id":id, "ordinal":1,
            "region":{"source":source,"start":31,"end":32}
        }));
    refresh_identity(package);
}

#[trace("TC-062", "FR-061-AC-16")]
#[test]
fn catalog_free_operation_step_order_precedes_first_row_join() {
    let (mut package, evidence, position) = catalog_free_operation_package("object_type");
    append_catalog_free_step(&mut package, position, "other");
    let rows = package["semantic_graph"]["nodes"][position]["body"]["steps"]
        .as_array()
        .expect("steps");
    assert_eq!(rows[0]["source_class"], "save");
    assert_eq!(rows[1]["source_class"], "other");
    assert_eq!(rows[0]["source_node"], rows[1]["source_node"]);
    let at = format!("/semantic_graph/nodes/{position}/body/steps");
    assert_step_refusal(
        read(&package, &evidence),
        CheckedPackageRefusalCode::InvalidSemanticGraph,
        None,
        &format!("{at}/1"),
    );
    package["semantic_graph"]["nodes"][position]["body"]["steps"]
        .as_array_mut()
        .expect("steps")
        .reverse();
    rekey_refinement(&mut package, position);
    assert_step_refusal(
        read(&package, &evidence),
        CheckedPackageRefusalCode::InvalidModelBinding,
        Some(CheckedPackageRefusalCause::MalformedDeclaration),
        &format!("{at}/0/source_class"),
    );
    package["semantic_graph"]["nodes"][position]["body"]["steps"][1]["source_class"] =
        json!("wrong");
    rekey_refinement(&mut package, position);
    assert_step_refusal(
        read(&package, &evidence),
        CheckedPackageRefusalCode::InvalidModelBinding,
        Some(CheckedPackageRefusalCause::MalformedDeclaration),
        &format!("{at}/0/source_class"),
    );
}

fn rekey_refinement(package: &mut Value, position: usize) {
    let node = &mut package["semantic_graph"]["nodes"][position];
    let old = node["node_id"].clone();
    let replacement = node_id(&sha256_hex(&canonical(&json!({
        "version": "quire.refinement-node/v1",
        "semantic_type": node["semantic_type"], "body": node["body"]
    }))));
    node["node_id"] = replacement.clone();
    for entry in package["source_map"].as_array_mut().expect("source map") {
        if entry["node_id"] == old {
            entry["node_id"] = replacement.clone();
        }
    }
    if let Some(rows) = package
        .get_mut("refinement_step_rows")
        .and_then(Value::as_array_mut)
    {
        for row in rows {
            if row["node_id"] == old {
                row["node_id"] = replacement.clone();
            }
        }
    }
    refresh_identity(package);
}

fn set_concrete_catalog(package: &mut Value, entries: Vec<Value>) -> usize {
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["semantic_form"] == "refinement")
        .expect("refinement");
    let node = &mut package["semantic_graph"]["nodes"][position];
    let mut dependencies = vec![node["body"]["abstract"].clone()];
    for entry in &entries {
        if let Some(declared) = entry.get("node") {
            dependencies.push(declared.clone());
        }
        if let Some(binders) = entry.get("binder_types").and_then(Value::as_array) {
            dependencies.extend(binders.iter().cloned());
        }
    }
    dependencies.sort_by(|left, right| left["digest"].as_str().cmp(&right["digest"].as_str()));
    dependencies.dedup();
    node["body"]["class_catalog"] = json!({"concrete": entries, "abstract": []});
    node["dependencies"] = json!(dependencies);
    rekey_refinement(package, position);
    position
}

fn activate_each(package: &mut Value, trigger_type: &Value) {
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    let over = nodes
        .iter()
        .find(|node| {
            node["semantic_form"] == "parameter"
                && node["body"]["members"][0]["value"]["value"] == "over"
        })
        .expect("over parameter")
        .clone();
    let mut trigger = over;
    trigger["semantic_type"] = trigger_type.clone();
    trigger["body"]["members"][0]["value"]["value"] = json!("opened");
    trigger["body"]["members"][1]["value"]["value"] = json!("1");
    trigger["node_id"] = node_id(&structural_key(
        "value",
        "parameter",
        trigger_type["digest"].as_str(),
        &trigger["body"],
    ));
    let trigger_key = trigger["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(trigger);

    let finish = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["body"]["operation"]["member"]["operator"] == "finish")
        .expect("finish");
    let old_binder = package["semantic_graph"]["nodes"][finish]["body"]["arguments"][0]["value"]
        ["target"]
        .clone();
    let mut binder = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_id"] == old_binder)
        .expect("finish binder")
        .clone();
    binder["body"]["members"][1]["value"]["value"] = json!("2");
    binder["node_id"] = node_id(&structural_key(
        "value",
        "parameter",
        binder["semantic_type"]["digest"].as_str(),
        &binder["body"],
    ));
    let binder_key = binder["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(binder);
    package["semantic_graph"]["nodes"][finish]["body"]["arguments"][0]["value"]["target"] =
        binder_key;

    let clause = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["semantic_form"] == "protocol_clause")
        .expect("protocol clause");
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][1]["value"]["members"] = json!([
        {"term": "binding", "name": "kind", "value": {"term": "literal", "type": node_id(&crate::support::checked_package::family_key("a1a1")), "value_kind": "text", "value": "each"}},
        {"term": "binding", "name": "trigger", "value": {"term": "reference", "target": trigger_key}}
    ]);
    settle(package);
}

fn record_field(owner: &str, name: &str, type_ref: &str, redefines: Option<&str>) -> Value {
    let mut field = json!({
        "identity": format!("{owner}/{name}"), "name": name, "typeRef": type_ref,
        "presence": "required", "nullable": false, "defaultKind": "none",
        "multiplicity": {"lower": 1, "upper": 1, "ordered": false, "unique": true}
    });
    if let Some(target) = redefines {
        field["redefines"] = json!(target);
    }
    field
}

fn record_type(name: &str, supertypes: &[&str], fields: Vec<Value>) -> Value {
    let identity = format!("ix://{MODEL_IDENTITY}/{name}");
    json!({
        "identity": identity, "displayName": name,
        "kind": {"module": MODEL_IDENTITY, "name": "record"},
        "roles": [], "constraints": [], "extensions": [], "unknownPolicy": "reject",
        "supertypes": supertypes, "fields": fields, "operations": []
    })
}

fn record_declaration_node(record: &Value) -> Value {
    let identity = record["identity"].as_str().expect("record identity");
    let owner = json!({"kind":"model", "identity":MODEL_IDENTITY, "node":identity});
    let body = json!({"term":"aggregate", "members":[]});
    let key = node_id(&sha256_hex(&canonical(&json!({
        "version":"quire.structural-node/v1", "node_tag":"model",
        "semantic_form":"record_value_type", "semantic_type":null,
        "declaration":null, "recursion":null, "owner":owner, "body":body
    }))));
    json!({
        "node_id":key, "schema_version":"quire.checked-semantic-graph/v2",
        "node_tag":"model", "semantic_form":"record_value_type",
        "semantic_type":key, "dependencies":[],
        "occurrences":[{"role":"type", "ordinal":0}],
        "owner":owner, "body":body
    })
}

fn record_activation_case(
    records: Vec<Value>,
    trigger: &str,
    binders: &[Value],
) -> (Value, CheckedPackageEvidence, usize) {
    let record_nodes: Vec<Value> = records.iter().map(record_declaration_node).collect();
    let trigger_key = record_nodes
        .iter()
        .find(|node| node["owner"]["node"] == format!("ix://{MODEL_IDENTITY}/{trigger}"))
        .expect("trigger record")["node_id"]
        .clone();
    let mut document = model_document();
    document["constructs"]
        .as_array_mut()
        .expect("constructs")
        .push(json!({
            "kind":{"module":MODEL_IDENTITY,"name":"record"},
            "construct":{"meaning":"quire.meaning.model.record-value-type/v1"}
        }));
    document["types"]
        .as_array_mut()
        .expect("types")
        .extend(records);
    let mut base = v2_all_families();
    activate_each(&mut base, &trigger_key);
    let (mut package, evidence) = concrete_protocol_refinement_with_model(
        base,
        ProtocolStepRows::Empty,
        ProtocolClauseOwner::SelectedModel,
        document,
        record_nodes,
    );
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    let clause = nodes
        .iter()
        .find(|node| node["semantic_form"] == "protocol_clause")
        .expect("clause")["node_id"]
        .clone();
    let finish = nodes
        .iter()
        .find(|node| node["body"]["operation"]["member"]["operator"] == "finish")
        .expect("finish")["node_id"]
        .clone();
    let boolean = node_id(&crate::support::checked_package::family_key("aaaa"));
    let position = set_concrete_catalog(
        &mut package,
        vec![
            json!({"class":"Example::OrderFlow::Closed", "node":finish, "binder_types":[boolean]}),
            json!({"class":"activate Example::OrderFlow", "node":clause, "binder_types":binders}),
        ],
    );
    (package, evidence, position)
}

/// Trace: FR-038-AC-325
#[trace("TC-048")]
#[test]
fn refinement_catalog_activate_each_uses_clause_and_trigger_signature() {
    let boolean = node_id(&crate::support::checked_package::family_key("aaaa"));
    let mut base = v2_all_families();
    activate_each(&mut base, &boolean);
    let (mut package, evidence) = concrete_protocol_refinement_on(
        base,
        ProtocolStepRows::Empty,
        ProtocolClauseOwner::SelectedModel,
    );
    let clause = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["semantic_form"] == "protocol_clause")
        .expect("clause")["node_id"]
        .clone();
    let finish = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["body"]["operation"]["member"]["operator"] == "finish")
        .expect("finish")["node_id"]
        .clone();
    let finish_entry =
        json!({"class":"Example::OrderFlow::Closed", "node":finish, "binder_types":[boolean]});
    let activate = json!({
        "class":"activate Example::OrderFlow", "node":clause, "binder_types":[boolean]
    });
    let position = set_concrete_catalog(&mut package, vec![finish_entry.clone(), activate.clone()]);
    let bytes = writer_bytes(&package);
    assert_eq!(bytes, canonical(&package));
    let admitted = CheckedPackageV2::read(&bytes, CheckedPackageReadLimits::bounded(), &evidence);
    assert!(
        matches!(admitted, CheckedPackageV2ReadResult::Admitted(_)),
        "{admitted:?}"
    );

    let mut abstract_protocol = package.clone();
    let refinement = &mut abstract_protocol["semantic_graph"]["nodes"][position];
    refinement["body"]["protocol"] = clause.clone();
    refinement["body"]["class_catalog"]["abstract"] =
        json!([finish_entry.clone(), activate.clone()]);
    rekey_refinement(&mut abstract_protocol, position);
    let admitted = read(&abstract_protocol, &evidence);
    assert!(
        matches!(admitted, CheckedPackageV2ReadResult::Admitted(_)),
        "selected abstract clause activate must join: {admitted:?}"
    );
    let mut absent_abstract = abstract_protocol;
    absent_abstract["semantic_graph"]["nodes"][position]["body"]["class_catalog"]["abstract"] =
        json!([finish_entry.clone()]);
    rekey_refinement(&mut absent_abstract, position);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&absent_abstract, &evidence) else {
        panic!("abstract activate omission admitted");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{position}/body/class_catalog/abstract").as_str())
    );

    let mut wrong_node = package.clone();
    let mut wrong_entry = activate.clone();
    wrong_entry["node"] = finish.clone();
    set_concrete_catalog(&mut wrong_node, vec![finish_entry.clone(), wrong_entry]);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&wrong_node, &evidence) else {
        panic!("activate associated with the finish node admitted");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidModelBinding);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::MalformedDeclaration)
    );
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{position}/body/class_catalog/concrete/1").as_str())
    );

    let mut wrong_binder = package.clone();
    let mut wrong_entry = activate.clone();
    wrong_entry["binder_types"] = json!([node_id(&crate::support::checked_package::family_key(
        "a1a1"
    ))]);
    set_concrete_catalog(&mut wrong_binder, vec![finish_entry.clone(), wrong_entry]);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&wrong_binder, &evidence) else {
        panic!("activate with the wrong binder type admitted");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidModelBinding);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::MalformedDeclaration)
    );
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{position}/body/class_catalog/concrete/1").as_str())
    );

    let mut missing = package.clone();
    set_concrete_catalog(&mut missing, vec![finish_entry.clone()]);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&missing, &evidence) else {
        panic!("missing activate class admitted");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{position}/body/class_catalog/concrete").as_str())
    );

    let (mut origin, origin_evidence) =
        concrete_protocol_refinement(ProtocolStepRows::Empty, ProtocolClauseOwner::SelectedModel);
    let origin_clause = origin["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["semantic_form"] == "protocol_clause")
        .expect("clause")["node_id"]
        .clone();
    let origin_finish = origin["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["body"]["operation"]["member"]["operator"] == "finish")
        .expect("finish")["node_id"]
        .clone();
    let origin_position = set_concrete_catalog(
        &mut origin,
        vec![
            json!({"class":"Example::OrderFlow::Closed", "node":origin_finish, "binder_types":[boolean]}),
            json!({"class":"activate Example::OrderFlow", "node":origin_clause, "binder_types":[boolean]}),
        ],
    );
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&origin, &origin_evidence) else {
        panic!("origin clause gained an activate class");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(
            format!("/semantic_graph/nodes/{origin_position}/body/class_catalog/concrete/1")
                .as_str()
        )
    );
}

/// Trace: FR-038-AC-325
#[trace("TC-048")]
#[test]
fn refinement_catalog_activate_record_follows_authored_inherited_field_order() {
    let base = format!("ix://{MODEL_IDENTITY}/Base");
    let mid = format!("ix://{MODEL_IDENTITY}/Mid");
    let leaf = format!("ix://{MODEL_IDENTITY}/Leaf");
    let boolean = node_id(&crate::support::checked_package::family_key("aaaa"));
    let integer = node_id(&crate::support::checked_package::family_key("a3a3"));
    for (base_fields, expected) in [
        (
            vec![("b", "Boolean"), ("a", "Integer")],
            vec![
                boolean.clone(),
                integer.clone(),
                boolean.clone(),
                integer.clone(),
            ],
        ),
        (
            vec![("a", "Integer"), ("b", "Boolean")],
            vec![
                integer.clone(),
                boolean.clone(),
                boolean.clone(),
                integer.clone(),
            ],
        ),
    ] {
        let records = vec![
            record_type(
                "Base",
                &[],
                base_fields
                    .iter()
                    .map(|(name, ty)| {
                        record_field(&base, name, &format!("ix://quire/native/{ty}"), None)
                    })
                    .collect(),
            ),
            record_type(
                "Mid",
                &[&base],
                vec![record_field(&mid, "c", "ix://quire/native/Boolean", None)],
            ),
            record_type(
                "Leaf",
                &[&mid],
                vec![record_field(&leaf, "d", "ix://quire/native/Integer", None)],
            ),
        ];
        let (package, evidence, position) = record_activation_case(records, "Leaf", &expected);
        let bytes = writer_bytes(&package);
        assert_eq!(bytes, canonical(&package));
        let admitted =
            CheckedPackageV2::read(&bytes, CheckedPackageReadLimits::bounded(), &evidence);
        assert!(
            matches!(admitted, CheckedPackageV2ReadResult::Admitted(_)),
            "{admitted:?}"
        );
        let mut reordered = package.clone();
        reordered["semantic_graph"]["nodes"][position]["body"]["class_catalog"]["concrete"][1]
            ["binder_types"] = json!([expected[1], expected[0], expected[2], expected[3]]);
        rekey_refinement(&mut reordered, position);
        let CheckedPackageV2ReadResult::Refused(refusal) = read(&reordered, &evidence) else {
            panic!("reordered inherited binder admitted");
        };
        assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidModelBinding);
        assert_eq!(
            refusal.cause,
            Some(CheckedPackageRefusalCause::MalformedDeclaration)
        );
        assert_eq!(
            refusal.path.as_ref().map(|path| path.as_str()),
            Some(
                format!("/semantic_graph/nodes/{position}/body/class_catalog/concrete/1").as_str()
            )
        );
    }
}

/// Trace: FR-038-AC-325
#[trace("TC-048")]
#[test]
fn refinement_catalog_activate_record_respects_supertype_order_and_diamond_hiding() {
    let p = format!("ix://{MODEL_IDENTITY}/P");
    let q = format!("ix://{MODEL_IDENTITY}/Q");
    let both = format!("ix://{MODEL_IDENTITY}/Both");
    let boolean = node_id(&crate::support::checked_package::family_key("aaaa"));
    let integer = node_id(&crate::support::checked_package::family_key("a3a3"));
    for (supertypes, expected) in [
        (
            vec![p.as_str(), q.as_str()],
            vec![boolean.clone(), integer.clone(), boolean.clone()],
        ),
        (
            vec![q.as_str(), p.as_str()],
            vec![integer.clone(), boolean.clone(), boolean.clone()],
        ),
    ] {
        let records = vec![
            record_type(
                "P",
                &[],
                vec![record_field(&p, "p", "ix://quire/native/Boolean", None)],
            ),
            record_type(
                "Q",
                &[],
                vec![record_field(&q, "q", "ix://quire/native/Integer", None)],
            ),
            record_type(
                "Both",
                &supertypes,
                vec![record_field(&both, "r", "ix://quire/native/Boolean", None)],
            ),
        ];
        let (package, evidence, _) = record_activation_case(records, "Both", &expected);
        let admitted = read(&package, &evidence);
        assert!(
            matches!(admitted, CheckedPackageV2ReadResult::Admitted(_)),
            "{admitted:?}"
        );
    }

    let top = format!("ix://{MODEL_IDENTITY}/Top");
    let left = format!("ix://{MODEL_IDENTITY}/Left");
    let right = format!("ix://{MODEL_IDENTITY}/Right");
    let bottom = format!("ix://{MODEL_IDENTITY}/Bottom");
    for redefine in [false, true] {
        let mut left_fields = Vec::new();
        if redefine {
            left_fields.push(record_field(
                &left,
                "t_prime",
                "ix://quire/native/Integer",
                Some(&format!("{top}/t")),
            ));
        }
        left_fields.push(record_field(&left, "l", "ix://quire/native/Integer", None));
        let records = vec![
            record_type(
                "Top",
                &[],
                vec![record_field(&top, "t", "ix://quire/native/Boolean", None)],
            ),
            record_type("Left", &[&top], left_fields),
            record_type(
                "Right",
                &[&top],
                vec![record_field(&right, "r", "ix://quire/native/Boolean", None)],
            ),
            record_type(
                "Bottom",
                &[&left, &right],
                vec![record_field(
                    &bottom,
                    "z",
                    "ix://quire/native/Integer",
                    None,
                )],
            ),
        ];
        let first = if redefine {
            integer.clone()
        } else {
            boolean.clone()
        };
        let expected = [
            first.clone(),
            integer.clone(),
            boolean.clone(),
            integer.clone(),
        ];
        let (package, evidence, _) = record_activation_case(records.clone(), "Bottom", &expected);
        let admitted = read(&package, &evidence);
        assert!(
            matches!(admitted, CheckedPackageV2ReadResult::Admitted(_)),
            "{admitted:?}"
        );

        let mut reverse = records;
        reverse[3]["supertypes"] = json!([right, left]);
        let reverse_expected = [first, boolean.clone(), integer.clone(), integer.clone()];
        let (package, evidence, _) = record_activation_case(reverse, "Bottom", &reverse_expected);
        let admitted = read(&package, &evidence);
        assert!(
            matches!(admitted, CheckedPackageV2ReadResult::Admitted(_)),
            "reverse diamond: {admitted:?}"
        );
    }
}

/// Trace: FR-038-AC-325
#[trace("TC-048")]
#[test]
fn refinement_catalog_activate_ambiguous_record_refuses_at_binder_types() {
    let p = format!("ix://{MODEL_IDENTITY}/P");
    let q = format!("ix://{MODEL_IDENTITY}/Q");
    let both = format!("ix://{MODEL_IDENTITY}/Both");
    let records = vec![
        record_type(
            "P",
            &[],
            vec![record_field(&p, "x", "ix://quire/native/Boolean", None)],
        ),
        record_type(
            "Q",
            &[],
            vec![record_field(&q, "x", "ix://quire/native/Integer", None)],
        ),
        record_type(
            "Both",
            &[&p, &q],
            vec![record_field(&both, "z", "ix://quire/native/Boolean", None)],
        ),
    ];
    let (package, evidence, position) = record_activation_case(records, "Both", &[]);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package, &evidence) else {
        panic!("ambiguous inherited binder admitted");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::AmbiguousDeclaration
    );
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::AmbiguousName)
    );
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(
            format!("/semantic_graph/nodes/{position}/body/class_catalog/concrete/1/binder_types")
                .as_str()
        )
    );
}

fn memory_catalog_package() -> (Value, CheckedPackageEvidence, usize) {
    let (mut base, _) = crate::checked_package_v2_protocol_body::protocol_holder_case_on(
        v2_all_families(),
        "parallel",
    );
    let holder = base["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["body"]["operation"]["member"]["operator"] == "parallel")
        .expect("parallel");
    let original = base["semantic_graph"]["nodes"][holder].clone();
    for (suffix, model) in [("Tso", "tso"), ("Ra", "ra"), ("ScAgain", "sc")] {
        let mut body = original["body"].clone();
        body["arguments"][0]["value"]["members"] = json!([{
            "term":"literal", "type": node_id(&crate::support::checked_package::family_key("a1a1")),
            "value_kind":"text", "value":model
        }]);
        for (branch, name) in ["Left", "Right"].iter().enumerate() {
            let child = base["semantic_graph"]["nodes"]
                .as_array()
                .expect("nodes")
                .iter()
                .find(|node| {
                    node["declaration"]["qualified_name"]
                        == json!(["Example", "OrderFlow", "run", "Main", "Holder", name])
                })
                .expect("parallel child");
            let copied = crate::support::checked_package::protocol_named_node(
                "control",
                &[
                    "Example",
                    "OrderFlow",
                    "run",
                    "Main",
                    &format!("Holder{suffix}"),
                    name,
                ],
                child["body"].clone(),
                &[],
            );
            body["arguments"][1]["value"]["members"][branch]["members"][1]["value"]["target"] =
                copied["node_id"].clone();
            base["semantic_graph"]["nodes"]
                .as_array_mut()
                .expect("nodes")
                .push(copied);
        }
        let copy = crate::support::checked_package::protocol_named_node(
            "control",
            &[
                "Example",
                "OrderFlow",
                "run",
                "Main",
                &format!("Holder{suffix}"),
            ],
            body,
            &[],
        );
        let root = base["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .position(|node| {
                node["declaration"]["qualified_name"]
                    == json!(["Example", "OrderFlow", "run", "Main"])
            })
            .expect("sequence root");
        base["semantic_graph"]["nodes"][root]["body"]["arguments"][0]["value"]["members"]
            .as_array_mut()
            .expect("children")
            .push(json!({"term":"reference", "target":copy["node_id"]}));
        base["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(copy);
    }
    settle(&mut base);
    let (mut package, evidence) = concrete_protocol_refinement_on(
        base,
        ProtocolStepRows::Empty,
        ProtocolClauseOwner::SelectedModel,
    );
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    let finish = nodes
        .iter()
        .find(|node| node["body"]["operation"]["member"]["operator"] == "finish")
        .expect("finish")["node_id"]
        .clone();
    let binder = node_id(&crate::support::checked_package::family_key("aaaa"));
    let mut entries =
        vec![json!({"class":"Example::OrderFlow::Closed", "node":finish, "binder_types":[binder]})];
    for suffix in ["", "Ra", "ScAgain", "Tso"] {
        let name = format!("Holder{suffix}");
        let parallel = nodes
            .iter()
            .find(|node| {
                node["declaration"]["qualified_name"]
                    == json!(["Example", "OrderFlow", "run", "Main", name])
            })
            .expect("parallel")["node_id"]
            .clone();
        entries.push(
            json!({"class":format!("fork Example::OrderFlow::{name}"), "node":parallel, "binder_types":[]}),
        );
    }
    for suffix in ["", "Ra", "ScAgain", "Tso"] {
        let name = format!("Holder{suffix}");
        let parallel = nodes
            .iter()
            .find(|node| {
                node["declaration"]["qualified_name"]
                    == json!(["Example", "OrderFlow", "run", "Main", name])
            })
            .expect("parallel")["node_id"]
            .clone();
        entries.push(
            json!({"class":format!("join Example::OrderFlow::{name}"), "node":parallel, "binder_types":[]}),
        );
    }
    let mut concrete: Vec<RefinementConcreteClass> = entries
        .into_iter()
        .map(|entry| serde_json::from_value(entry).expect("declared class"))
        .collect();
    for memory_model in [
        RefinementMemoryModel::Ra,
        RefinementMemoryModel::Sc,
        RefinementMemoryModel::Tso,
    ] {
        concrete.push(RefinementConcreteClass::Memory(
            RefinementMemoryClass::Drain {
                memory_model,
                drain: RefinementDrain::StoreBufferFlush,
            },
        ));
    }
    concrete.push(RefinementConcreteClass::Memory(
        RefinementMemoryClass::Flush {
            memory_model: RefinementTsoModel::Tso,
        },
    ));
    let catalog = RefinementClassCatalog {
        concrete,
        r#abstract: vec![],
    };
    let encoded = serde_json::to_value(catalog).expect("typed catalog");
    let entries = encoded["concrete"].as_array().expect("catalog").clone();
    let position = set_concrete_catalog(&mut package, entries);
    (package, evidence, position)
}

/// Trace: FR-038-AC-325
#[trace(
    "TC-048",
    "FR-038-AC-325",
    "FR-061-AC-10",
    "FR-061-AC-12",
    "FR-061-AC-13"
)]
#[test]
fn memory_drain_catalog_roundtrips_and_joins_source_defaults() {
    let (package, evidence, position) = memory_catalog_package();
    let typed: CheckedSemanticGraphV2<CheckedTerm> =
        serde_json::from_value(package["semantic_graph"].clone()).expect("typed graph");
    let CheckedTerm::Refinement {
        class_catalog: Some(catalog),
        ..
    } = &typed.nodes[position].body
    else {
        panic!("typed refinement catalog missing");
    };
    assert!(matches!(
        &catalog.concrete[catalog.concrete.len() - 4],
        RefinementConcreteClass::Memory(RefinementMemoryClass::Drain {
            memory_model: RefinementMemoryModel::Ra,
            drain: RefinementDrain::StoreBufferFlush,
        })
    ));
    assert!(matches!(
        &catalog.concrete[catalog.concrete.len() - 1],
        RefinementConcreteClass::Memory(RefinementMemoryClass::Flush {
            memory_model: RefinementTsoModel::Tso,
        })
    ));
    let refinement_id: quire_contract_model::CheckedNodeId =
        serde_json::from_value(package["semantic_graph"]["nodes"][position]["node_id"].clone())
            .expect("refinement id");
    let assert_typed_drain = |admitted: &CheckedPackageV2| {
        let Some(CheckedTerm::Refinement {
            class_catalog: Some(class_catalog),
            ..
        }) = admitted.term(&refinement_id)
        else {
            panic!("admitted typed catalog missing");
        };
        assert_eq!(catalog, class_catalog);
    };
    let CheckedPackageV2ReadResult::Admitted(admitted) = read(&package, &evidence) else {
        panic!("source-default catalog refused");
    };
    assert_typed_drain(&admitted);
    let encoded = writer_bytes(&package);
    let written: Value = serde_json::from_slice(&encoded).expect("encoded package");
    let entries = written["semantic_graph"]["nodes"][position]["body"]["class_catalog"]["concrete"]
        .as_array()
        .expect("catalog");
    let memory: Vec<_> = entries
        .iter()
        .filter(|entry| {
            entry["class"]
                .as_str()
                .is_some_and(|class| class.starts_with("memory "))
        })
        .cloned()
        .collect();
    assert_eq!(
        memory,
        vec![
            json!({"class":"memory drain", "memory_model":"ra", "drain":"store-buffer-flush"}),
            json!({"class":"memory drain", "memory_model":"sc", "drain":"store-buffer-flush"}),
            json!({"class":"memory drain", "memory_model":"tso", "drain":"store-buffer-flush"}),
            json!({"class":"memory flush", "memory_model":"tso"}),
        ]
    );
    assert_eq!(written["package_id"], package["package_id"]);
    assert_eq!(
        written["semantic_graph"]["nodes"][position]["node_id"],
        package["semantic_graph"]["nodes"][position]["node_id"]
    );
    let CheckedPackageV2ReadResult::Admitted(reread) =
        CheckedPackageV2::read(&encoded, CheckedPackageReadLimits::bounded(), &evidence)
    else {
        panic!("written catalog refused on reread");
    };
    assert_typed_drain(&reread);
}

#[trace("FR-061-AC-12")]
#[test]
fn abstract_protocol_memory_default_is_catalogued_from_its_own_clause() {
    let (base, _) = crate::checked_package_v2_protocol_body::protocol_holder_case_on(
        v2_all_families(),
        "parallel",
    );
    let (mut package, evidence) = concrete_protocol_refinement_on(
        base,
        ProtocolStepRows::Empty,
        ProtocolClauseOwner::SelectedModel,
    );
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    let clause = nodes
        .iter()
        .find(|node| node["semantic_form"] == "protocol_clause")
        .expect("clause")["node_id"]
        .clone();
    let finish = nodes
        .iter()
        .find(|node| node["body"]["operation"]["member"]["operator"] == "finish")
        .expect("finish")["node_id"]
        .clone();
    let parallel = nodes
        .iter()
        .find(|node| node["body"]["operation"]["member"]["operator"] == "parallel")
        .expect("parallel")["node_id"]
        .clone();
    let binder = node_id(&crate::support::checked_package::family_key("aaaa"));
    let position = nodes.len() - 1;
    let abstract_key = nodes[position]["body"]["abstract"].clone();
    let entries = json!([
        {"class":"Example::OrderFlow::Closed", "node":finish.clone(), "binder_types":[binder.clone()]},
        {"class":"fork Example::OrderFlow::Holder", "node":parallel.clone(), "binder_types":[]},
        {"class":"join Example::OrderFlow::Holder", "node":parallel.clone(), "binder_types":[]},
        {"class":"memory drain", "memory_model":"sc", "drain":"store-buffer-flush"},
    ]);
    let mut dependencies = vec![abstract_key, clause.clone(), finish, parallel, binder];
    dependencies.sort_by(|left, right| left["digest"].as_str().cmp(&right["digest"].as_str()));
    dependencies.dedup();
    let refinement = &mut package["semantic_graph"]["nodes"][position];
    refinement["body"]["protocol"] = clause;
    refinement["body"]["class_catalog"] = json!({"concrete":entries, "abstract":entries});
    refinement["dependencies"] = json!(dependencies);
    rekey_refinement(&mut package, position);
    let admitted = read(&package, &evidence);
    assert!(
        matches!(admitted, CheckedPackageV2ReadResult::Admitted(_)),
        "{admitted:?}"
    );

    let mut extra = package.clone();
    extra["semantic_graph"]["nodes"][position]["body"]["class_catalog"]["concrete"]
        .as_array_mut()
        .expect("concrete classes")
        .insert(
            3,
            json!({"class":"memory drain", "memory_model":"ra", "drain":"store-buffer-flush"}),
        );
    rekey_refinement(&mut extra, position);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&extra, &evidence) else {
        panic!("memory class from absent source default admitted");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{position}/body/class_catalog/concrete/3").as_str())
    );

    package["semantic_graph"]["nodes"][position]["body"]["class_catalog"]["abstract"]
        .as_array_mut()
        .expect("classes")
        .pop();
    rekey_refinement(&mut package, position);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package, &evidence) else {
        panic!("abstract source-default drain omitted but admitted");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{position}/body/class_catalog/abstract").as_str())
    );
}

/// Trace: FR-038-AC-325
#[trace("TC-048", "FR-038-AC-325", "FR-061-AC-11")]
#[test]
fn memory_drain_wire_and_catalog_order_refuse_at_their_stage() {
    let (base, evidence, position) = memory_catalog_package();
    let path = format!("/semantic_graph/nodes/{position}/body/class_catalog/concrete");
    let first_drain = base["semantic_graph"]["nodes"][position]["body"]["class_catalog"]
        ["concrete"]
        .as_array()
        .expect("catalog")
        .iter()
        .position(|entry| entry["class"] == "memory drain")
        .expect("drain");
    for (label, edit, suffix) in [
        ("missing drain", 0, "/drain"),
        ("wrong drain", 1, "/drain"),
        ("numeric drain", 2, "/drain"),
        ("flush drain", 3, "/drain"),
        ("flush sc", 4, "/memory_model"),
        ("flush ra", 5, "/memory_model"),
        ("node", 6, "/node"),
        ("binder", 7, "/binder_types"),
        ("unknown", 8, "/unknown"),
    ] {
        let mut package = base.clone();
        let entries = package["semantic_graph"]["nodes"][position]["body"]["class_catalog"]
            ["concrete"]
            .as_array_mut()
            .expect("catalog");
        let drain = entries
            .iter()
            .position(|entry| entry["class"] == "memory drain")
            .expect("drain");
        let flush = entries
            .iter()
            .position(|entry| entry["class"] == "memory flush")
            .expect("flush");
        let index = match edit {
            0 => {
                entries[drain]
                    .as_object_mut()
                    .expect("object")
                    .remove("drain");
                drain
            }
            1 => {
                entries[drain]["drain"] = json!("other");
                drain
            }
            2 => {
                entries[drain]["drain"] = json!(1);
                drain
            }
            3 => {
                entries[flush]["drain"] = json!("store-buffer-flush");
                flush
            }
            4 => {
                entries[flush]["memory_model"] = json!("sc");
                flush
            }
            5 => {
                entries[flush]["memory_model"] = json!("ra");
                flush
            }
            6 => {
                entries[drain]["node"] =
                    base["semantic_graph"]["nodes"][position]["semantic_type"].clone();
                drain
            }
            7 => {
                entries[drain]["binder_types"] = json!([]);
                drain
            }
            8 => {
                entries[drain]["unknown"] = json!(true);
                drain
            }
            _ => unreachable!(),
        };
        let CheckedPackageV2ReadResult::Refused(refusal) = read(&package, &evidence) else {
            panic!("{label} admitted");
        };
        assert_eq!(
            refusal.code,
            CheckedPackageRefusalCode::MalformedWire,
            "{label}"
        );
        assert_eq!(
            refusal.path.as_ref().map(|path| path.as_str()),
            Some(format!("{path}/{index}{suffix}").as_str()),
            "{label}"
        );
    }
    for (label, mutate, expected_index) in [
        ("out of order", 0, first_drain + 1),
        ("duplicate drain", 1, first_drain + 2),
        ("duplicate declared", 2, 1),
    ] {
        let mut package = base.clone();
        let entries = package["semantic_graph"]["nodes"][position]["body"]["class_catalog"]
            ["concrete"]
            .as_array_mut()
            .expect("catalog");
        match mutate {
            0 => entries.swap(first_drain, first_drain + 1),
            1 => entries[first_drain + 2] = entries[first_drain + 1].clone(),
            2 => entries[1] = entries[0].clone(),
            _ => unreachable!(),
        }
        rekey_refinement(&mut package, position);
        let CheckedPackageV2ReadResult::Refused(refusal) = read(&package, &evidence) else {
            panic!("{label} admitted");
        };
        assert_eq!(
            refusal.code,
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            "{label}"
        );
        assert_eq!(
            refusal.path.as_ref().map(|path| path.as_str()),
            Some(format!("{path}/{expected_index}").as_str()),
            "{label}"
        );
    }
}

/// Trace: FR-038-AC-325
#[trace("TC-048", "FR-038-AC-325", "FR-061-AC-12", "FR-061-AC-13")]
#[test]
fn memory_catalog_completeness_identity_and_dependency_precedence() {
    let (base, evidence, position) = memory_catalog_package();
    let path = format!("/semantic_graph/nodes/{position}/body/class_catalog/concrete");
    let mut absent = base.clone();
    absent["semantic_graph"]["nodes"][position]["body"]
        .as_object_mut()
        .expect("body")
        .remove("class_catalog");
    let abstract_key = absent["semantic_graph"]["nodes"][position]["body"]["abstract"].clone();
    absent["semantic_graph"]["nodes"][position]["dependencies"] = json!([abstract_key]);
    rekey_refinement(&mut absent, position);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&absent, &evidence) else {
        panic!("required catalog omission admitted");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{position}/body/class_catalog").as_str())
    );
    let entries = base["semantic_graph"]["nodes"][position]["body"]["class_catalog"]["concrete"]
        .as_array()
        .expect("catalog");
    let first_drain = entries
        .iter()
        .position(|entry| entry["class"] == "memory drain")
        .expect("drain");
    let flush = entries
        .iter()
        .position(|entry| entry["class"] == "memory flush")
        .expect("flush");
    for (label, remove) in [("drain", first_drain), ("flush", flush)] {
        let mut package = base.clone();
        package["semantic_graph"]["nodes"][position]["body"]["class_catalog"]["concrete"]
            .as_array_mut()
            .expect("catalog")
            .remove(remove);
        rekey_refinement(&mut package, position);
        let CheckedPackageV2ReadResult::Refused(refusal) = read(&package, &evidence) else {
            panic!("missing {label} admitted");
        };
        assert_eq!(
            refusal.code,
            CheckedPackageRefusalCode::InvalidSemanticGraph
        );
        assert_eq!(
            refusal.path.as_ref().map(|path| path.as_str()),
            Some(path.as_str())
        );
    }
    let mut foreign = base.clone();
    foreign["semantic_graph"]["nodes"][position]["body"]["class_catalog"]["abstract"] =
        json!([{"class":"memory drain", "memory_model":"sc", "drain":"store-buffer-flush"}]);
    rekey_refinement(&mut foreign, position);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&foreign, &evidence) else {
        panic!("memory class from unselected abstract subject admitted");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{position}/body/class_catalog/abstract/0").as_str())
    );
    let mut changed = base.clone();
    let original_node = changed["semantic_graph"]["nodes"][position]["node_id"].clone();
    let original_package = changed["package_id"].clone();
    changed["semantic_graph"]["nodes"][position]["body"]["class_catalog"]["concrete"]
        [first_drain]["memory_model"] = json!("tso");
    refresh_identity(&mut changed);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&changed, &evidence) else {
        panic!("stale memory catalog node admitted");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::StaleNodeKey)
    );
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{position}/node_id").as_str())
    );
    rekey_refinement(&mut changed, position);
    assert_ne!(
        changed["semantic_graph"]["nodes"][position]["node_id"],
        original_node
    );
    assert_ne!(changed["package_id"], original_package);

    let declared =
        &base["semantic_graph"]["nodes"][position]["body"]["class_catalog"]["concrete"][0];
    for (label, dependency) in [
        ("node", declared["node"].clone()),
        ("binder type", declared["binder_types"][0].clone()),
    ] {
        let mut missing_dependency = base.clone();
        let dependencies = missing_dependency["semantic_graph"]["nodes"][position]["dependencies"]
            .as_array_mut()
            .expect("dependencies");
        let declared_position = dependencies
            .iter()
            .position(|key| key == &dependency)
            .expect("declared catalog dependency");
        dependencies.remove(declared_position);
        refresh_identity(&mut missing_dependency);
        let CheckedPackageV2ReadResult::Refused(refusal) = read(&missing_dependency, &evidence)
        else {
            panic!("missing declared catalog {label} dependency admitted");
        };
        assert_eq!(
            refusal.code,
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            "{label}"
        );
        assert_eq!(
            refusal.path.as_ref().map(|path| path.as_str()),
            Some(format!("/semantic_graph/nodes/{position}/dependencies").as_str()),
            "{label}"
        );
    }
}

fn operation_model_nodes() -> (Value, Value, Value, Value) {
    let mut document = model_document();
    let worker_identity = "ix://test/refinement/Worker";
    document["constructs"]
        .as_array_mut()
        .expect("constructs")
        .push(json!({
            "kind": {"module": MODEL_IDENTITY, "name": "worker"},
            "construct": {"meaning": "quire.meaning.model.object-type/v1"}
        }));
    document["types"]
        .as_array_mut()
        .expect("types")
        .push(json!({
            "identity": worker_identity, "displayName": "Worker",
            "kind": {"module": MODEL_IDENTITY, "name": "worker"},
            "roles": [], "constraints": [], "extensions": [], "unknownPolicy": "reject",
            "supertypes": [], "fields": [],
            "operations": [{
                "identity": "ix://test/refinement/Worker/run",
                "params": [
                    {"identity":"ix://test/refinement/Worker/run/count", "name":"count", "typeRef":"ix://quire/native/Integer", "multiplicity":{"lower":1,"upper":1,"ordered":false,"unique":true}},
                    {"identity":"ix://test/refinement/Worker/run/confirmed", "name":"confirmed", "typeRef":"ix://quire/native/Boolean", "multiplicity":{"lower":1,"upper":1,"ordered":false,"unique":true}}
                ]
            }]
        }));
    let body = json!({"term": "aggregate", "members": []});
    let owner = json!({"kind": "model", "identity": MODEL_IDENTITY, "node": worker_identity});
    let worker_key = node_id(&sha256_hex(&canonical(&json!({
        "version": "quire.structural-node/v1", "node_tag": "model",
        "semantic_form": "object_type", "semantic_type": null,
        "declaration": null, "recursion": null, "owner": owner, "body": body
    }))));
    let worker = json!({
        "node_id": worker_key, "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": "model", "semantic_form": "object_type",
        "semantic_type": worker_key, "dependencies": [],
        "occurrences": [{"role": "type", "ordinal": 0}],
        "owner": owner, "body": body
    });
    let reference_body = json!({"term":"aggregate", "members":[
        {"term":"reference", "target":worker_key}
    ]});
    let reference_key = node_id(&sha256_hex(&canonical(&json!({
        "version": "quire.structural-node/v1", "node_tag": "composite_type",
        "semantic_form": "reference", "semantic_type": null,
        "declaration": null, "recursion": null, "body": reference_body
    }))));
    let reference = json!({
        "node_id": reference_key, "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": "composite_type", "semantic_form": "reference",
        "semantic_type": reference_key, "dependencies": [worker_key],
        "occurrences": [{"role": "type", "ordinal": 0}], "body": reference_body
    });
    (document, worker, reference, reference_key)
}

enum ProtocolStepRows {
    Empty,
    Declared,
}

enum ProtocolClauseOwner {
    SelectedModel,
    OtherModel,
}

fn concrete_protocol_refinement(
    step_rows: ProtocolStepRows,
    clause_owner: ProtocolClauseOwner,
) -> (Value, CheckedPackageEvidence) {
    concrete_protocol_refinement_on(v2_all_families(), step_rows, clause_owner)
}

fn concrete_protocol_refinement_on(
    base: Value,
    step_rows: ProtocolStepRows,
    clause_owner: ProtocolClauseOwner,
) -> (Value, CheckedPackageEvidence) {
    concrete_protocol_refinement_with_model(
        base,
        step_rows,
        clause_owner,
        model_document(),
        Vec::new(),
    )
}

fn concrete_protocol_refinement_with_model(
    base: Value,
    step_rows: ProtocolStepRows,
    clause_owner: ProtocolClauseOwner,
    mut document: Value,
    extra_model_nodes: Vec<Value>,
) -> (Value, CheckedPackageEvidence) {
    let (machine, _) = machine_package();
    let mut model = machine["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["semantic_form"] == "state_machine")
        .expect("state machine")
        .clone();
    document["constructs"][0]["construct"]["meaning"] = json!("quire.meaning.model.object-type/v1");
    // The concrete refinement's selected root is an object with no operation
    // classes. The complete state-machine members belong to machine_package.
    document["types"][0]["operations"] = json!([]);
    document["types"][0]
        .as_object_mut()
        .expect("model type")
        .remove("states");
    document["types"][0]
        .as_object_mut()
        .expect("model type")
        .remove("transitions");
    model["semantic_form"] = json!("object_type");
    let model_key = node_id(&sha256_hex(&canonical(&json!({
        "version": "quire.structural-node/v1", "node_tag": "model",
        "semantic_form": "object_type", "semantic_type": null,
        "declaration": null, "recursion": null,
        "owner": model["owner"], "body": model["body"]
    }))));
    model["node_id"] = model_key.clone();
    model["semantic_type"] = model_key;
    let digest = sha256_hex(&canonical(&document));
    let mut package = base;
    package["lock"]["model_selections"]
        .as_array_mut()
        .expect("model selections")
        .push(json!({
            "identity": MODEL_IDENTITY, "digest_domain": "sha256-jcs", "digest": digest
        }));
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(model.clone());
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend(extra_model_nodes);
    if matches!(clause_owner, ProtocolClauseOwner::SelectedModel) {
        for clause in package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .iter_mut()
            .filter(|node| node["semantic_form"] == "protocol_clause")
        {
            clause["owner_model"] = model["node_id"].clone();
        }
    }
    settle(&mut package);

    let finish = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "protocol" && node["semantic_form"] == "obligation")
        .expect("declared finish")["node_id"]
        .clone();
    let profiles = package["lock"]["profile_selections"].clone();
    if matches!(step_rows, ProtocolStepRows::Declared) {
        one_step_refinement_with_step(
            &mut package,
            json!({
                "source_class": "finish OrderFlow::Closed", "source_node": finish,
                "target": {"kind": "stutter"}
            }),
        );
    } else {
        row_empty_refinement(&mut package);
    }
    let infinite = package["lock"]["profile_selections"][0].clone();
    package["lock"]["profile_selections"] = profiles;
    package["lock"]["profile_selections"]
        .as_array_mut()
        .expect("profile selections")
        .push(infinite);
    refresh_identity(&mut package);
    let mut evidence = evidence_for(&package);
    evidence
        .insert_domain_package_document(digest, canonical(&document))
        .expect("state machine document");
    (package, evidence)
}

fn make_step_owner_proof_ineligible(package: &mut Value) {
    let node = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .last_mut()
        .expect("refinement");
    let old_id = node["node_id"].clone();
    node["body"]["steps"][0]["target"] = json!({"kind":"any"});
    node["occurrences"]
        .as_array_mut()
        .expect("occurrences")
        .retain(|entry| entry["role"] == "declaration");
    let new_id = node_id(&sha256_hex(&canonical(&json!({
        "version": "quire.refinement-node/v1",
        "semantic_type": node["semantic_type"], "body": node["body"]
    }))));
    node["node_id"] = new_id.clone();
    let source_map = package["source_map"].as_array_mut().expect("source map");
    source_map.retain(|entry| entry["node_id"] != old_id || entry["role"] == "declaration");
    for entry in source_map {
        if entry["node_id"] == old_id {
            entry["node_id"] = new_id.clone();
        }
    }
    package["refinement_step_rows"][0]["node_id"] = new_id;
    refresh_identity(package);
}

#[trace("FR-061-AC-2")]
#[test]
fn refinement_class_catalog_entries_are_closed() {
    let (mut package, evidence) = machine_package();
    row_empty_refinement(&mut package);
    let refinement = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
    let base = package.clone();
    for (entry, code, member) in [
        (
            json!({"class":"memory flush","memory_model":"tso","node":base["semantic_graph"]["nodes"][refinement]["semantic_type"]}),
            CheckedPackageRefusalCode::MalformedWire,
            "/node",
        ),
        (
            json!({"class":"memory flush","memory_model":"unknown"}),
            CheckedPackageRefusalCode::MalformedWire,
            "/memory_model",
        ),
        (
            json!({"class":"finish P::end","node":base["semantic_graph"]["nodes"][refinement]["semantic_type"]}),
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            "",
        ),
    ] {
        package = base.clone();
        package["semantic_graph"]["nodes"][refinement]["body"]["class_catalog"] =
            json!({"concrete":[entry],"abstract":[]});
        let CheckedPackageV2ReadResult::Refused(refusal) = read(&package, &evidence) else {
            panic!("malformed catalog entry admitted");
        };
        assert_eq!(refusal.code, code);
        assert_eq!(
            refusal.path.as_ref().map(|path| path.as_str()),
            Some(
                format!("/semantic_graph/nodes/{refinement}/body/class_catalog/concrete/0{member}")
                    .as_str()
            )
        );
    }
}

#[trace("FR-061-AC-2")]
#[test]
fn refinement_open_argument_has_one_closed_spelling() {
    let (mut package, evidence) = machine_package();
    one_step_refinement(&mut package);
    let refinement = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
    package["semantic_graph"]["nodes"][refinement]["body"]["steps"][0]["target"] =
        json!({"kind":"protocol_step","class":"memory flush","arguments":[{"term":"open"}]});
    refresh_identity(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package, &evidence) else {
        panic!("unrefreshed node key must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::StaleNodeKey)
    );
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{refinement}/node_id").as_str())
    );

    package["semantic_graph"]["nodes"][refinement]["body"]["steps"][0]["target"]["arguments"][0]
        ["term"] = json!("other");
    refresh_identity(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package, &evidence) else {
        panic!("non-open argument spelling admitted");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(
            format!("/semantic_graph/nodes/{refinement}/body/steps/0/target/arguments/0").as_str()
        )
    );
}

#[trace("FR-061-AC-9")]
#[test]
fn refinement_class_catalog_is_in_dedicated_refinement_key() {
    let (mut package, evidence) = machine_package();
    row_empty_refinement(&mut package);
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
    package["semantic_graph"]["nodes"][position]["body"]["class_catalog"] =
        json!({"concrete":[],"abstract":[]});
    refresh_identity(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package, &evidence) else {
        panic!("stale refinement key admitted");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::StaleNodeKey)
    );
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{position}/node_id").as_str())
    );
}

#[trace("FR-061-AC-9")]
#[test]
fn refinement_step_regions_bind_generated_occurrences_before_row_join() {
    let (mut package, evidence) = machine_package();
    one_step_refinement(&mut package);
    let result = read(&package, &evidence);
    let CheckedPackageV2ReadResult::Refused(refusal) = result else {
        panic!("unbacked memory class must not admit");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
    let refinement = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{refinement}/body/class_catalog").as_str())
    );
    let bytes = writer_bytes(&package);
    assert_eq!(bytes, canonical(&package));
    let mut wrong = package.clone();
    wrong["refinement_step_rows"][0]["region"]["start"] = json!(30);
    assert_eq!(wrong["package_id"], package["package_id"]);
    let at = wrong["source_map"]
        .as_array()
        .expect("source map")
        .iter()
        .position(|entry| {
            entry["node_id"] == wrong["refinement_step_rows"][0]["node_id"]
                && entry["role"] == "generated"
                && entry["ordinal"] == 1
        })
        .expect("generated step");
    invalid_value_at(read(&wrong, &evidence), &format!("/source_map/{at}"));
}

#[test]
fn refinement_step_row_sidecar_shape_and_order_are_exact() {
    let (mut package, evidence) = machine_package();
    one_step_refinement(&mut package);
    let mut missing = package.clone();
    missing
        .as_object_mut()
        .expect("package")
        .remove("refinement_step_rows");
    invalid_value_at(read(&missing, &evidence), "/refinement_step_rows");

    let mut wrong_owner = package.clone();
    wrong_owner["refinement_step_rows"][0]["node_id"] =
        wrong_owner["semantic_graph"]["nodes"][0]["node_id"].clone();
    invalid_value_at(read(&wrong_owner, &evidence), "/refinement_step_rows/0");

    let mut out_of_range = package.clone();
    out_of_range["refinement_step_rows"][0]["ordinal"] = json!(1);
    invalid_value_at(read(&out_of_range, &evidence), "/refinement_step_rows/0");

    let mut duplicate = package.clone();
    let row = duplicate["refinement_step_rows"][0].clone();
    duplicate["refinement_step_rows"]
        .as_array_mut()
        .expect("rows")
        .push(row);
    invalid_value_at(read(&duplicate, &evidence), "/refinement_step_rows/1");

    let (mut empty, evidence) = machine_package();
    row_empty_refinement(&mut empty);
    empty["refinement_step_rows"] = json!([]);
    invalid_value_at(read(&empty, &evidence), "/refinement_step_rows");
}

#[test]
fn refinement_ineligible_step_owner_still_checks_sidecar_region() {
    let (mut package, evidence) = machine_package();
    one_step_refinement(&mut package);
    make_step_owner_proof_ineligible(&mut package);
    assert!(package["source_map"]
        .as_array()
        .expect("source map")
        .iter()
        .all(|entry| {
            entry["node_id"] != package["refinement_step_rows"][0]["node_id"]
                || entry["role"] == "declaration"
        }));

    let mut empty_interval = package.clone();
    empty_interval["refinement_step_rows"][0]["region"]["end"] =
        empty_interval["refinement_step_rows"][0]["region"]["start"].clone();
    invalid_value_at(
        read(&empty_interval, &evidence),
        "/refinement_step_rows/0/region",
    );

    let mut unlocked = package;
    unlocked["refinement_step_rows"][0]["region"]["source"]["identity"] = json!("test/unlocked");
    invalid_value_at(
        read(&unlocked, &evidence),
        "/refinement_step_rows/0/region/source",
    );
}

#[trace("FR-061-AC-9")]
#[test]
fn refinement_source_map_precedes_stale_refinement_key() {
    let (mut package, evidence) = machine_package();
    row_empty_refinement(&mut package);
    let refinement = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .last()
        .expect("refinement")["node_id"]
        .clone();
    let node = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .last_mut()
        .expect("refinement");
    node["body"]["profile"]["identity"] = json!("quire.temporal.other/v1");
    refresh_identity(&mut package);
    let stale = read(&package, &evidence);
    let CheckedPackageV2ReadResult::Refused(stale) = stale else {
        panic!("expected stale refinement key");
    };
    assert_eq!(stale.cause, Some(CheckedPackageRefusalCause::StaleNodeKey));

    package["source_map"]
        .as_array_mut()
        .expect("source map")
        .retain(|entry| !(entry["node_id"] == refinement && entry["role"] == "generated"));
    invalid_value_at(read(&package, &evidence), "/source_map");
}

#[test]
fn refinement_selected_state_machine_owner_is_admitted() {
    let (package, evidence) = machine_package();
    let result = read(&package, &evidence);
    assert!(
        matches!(result, CheckedPackageV2ReadResult::Admitted(_)),
        "{result:?}"
    );
}

#[test]
fn refinement_row_empty_model_refinement_is_admitted() {
    let (mut package, evidence) = machine_package();
    row_empty_refinement(&mut package);
    let result = read(&package, &evidence);
    assert!(
        matches!(result, CheckedPackageV2ReadResult::Admitted(_)),
        "{result:?}"
    );
    let bytes = writer_bytes(&package);
    assert_eq!(bytes, canonical(&package));
    assert!(matches!(
        CheckedPackageV2::read(&bytes, CheckedPackageReadLimits::bounded(), &evidence),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

#[trace("TC-233", "FR-065-AC-6")]
#[test]
fn refinement_headers_require_selected_roots_not_state_machine_declarations() {
    let (mut package, evidence) = machine_package();
    row_empty_refinement(&mut package);
    let position = package["semantic_graph"]["nodes"].as_array().unwrap().len() - 1;
    let state_machine = package["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["semantic_form"] == "state_machine")
        .expect("selected declaration")["node_id"]
        .clone();

    let mut concrete_declaration = package.clone();
    concrete_declaration["semantic_graph"]["nodes"][position]["semantic_type"] =
        state_machine.clone();
    rekey_refinement(&mut concrete_declaration, position);
    refresh_identity(&mut concrete_declaration);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&concrete_declaration, &evidence)
    else {
        panic!("state-machine concrete header admitted");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidModelBinding);
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{position}/semantic_type").as_str())
    );

    let mut abstract_declaration = package;
    abstract_declaration["semantic_graph"]["nodes"][position]["body"]["abstract"] =
        state_machine.clone();
    abstract_declaration["semantic_graph"]["nodes"][position]["dependencies"] =
        json!([state_machine]);
    rekey_refinement(&mut abstract_declaration, position);
    refresh_identity(&mut abstract_declaration);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&abstract_declaration, &evidence)
    else {
        panic!("state-machine abstract header admitted");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidModelBinding);
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{position}/body/abstract").as_str())
    );
}

#[trace("TC-233", "FR-065-AC-6")]
#[test]
fn refinement_external_abstract_resolves_only_the_directly_selected_package_root() {
    let (dependency_wire, dependency_evidence) = machine_package();
    let dependency = match read(&dependency_wire, &dependency_evidence) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("dependency package refused: {other:?}"),
    };
    let dependency_root = dependency_wire["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["semantic_form"] == "model_root")
        .expect("dependency root")["node_id"]
        .clone();
    let dependency_declaration = dependency_wire["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["semantic_form"] == "state_machine")
        .expect("dependency declaration")["node_id"]
        .clone();
    let (mut package, mut evidence) = machine_package();
    package["lock"]["dependency_selections"] = json!([{
        "identity": "test/abstract",
        "package_id": dependency_wire["package_id"],
    }]);
    evidence
        .insert_dependency_package("test/abstract", dependency)
        .expect("direct dependency evidence");
    row_empty_refinement(&mut package);
    let position = package["semantic_graph"]["nodes"].as_array().unwrap().len() - 1;
    let external = json!({
        "package": dependency_wire["package_id"], "node": dependency_root,
    });
    package["semantic_graph"]["nodes"][position]["body"]["abstract"] = external.clone();
    package["semantic_graph"]["nodes"][position]["dependencies"] = json!([]);
    rekey_refinement(&mut package, position);
    refresh_identity(&mut package);
    assert!(matches!(
        read(&package, &evidence),
        CheckedPackageV2ReadResult::Admitted(_)
    ));

    let mut declaration = package.clone();
    declaration["semantic_graph"]["nodes"][position]["body"]["abstract"]["node"] =
        dependency_declaration;
    rekey_refinement(&mut declaration, position);
    refresh_identity(&mut declaration);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&declaration, &evidence) else {
        panic!("external state-machine declaration admitted");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidModelBinding);
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{position}/body/abstract").as_str())
    );

    let mut unselected = package;
    unselected["lock"]["dependency_selections"] = json!([]);
    refresh_identity(&mut unselected);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&unselected, &evidence) else {
        panic!("unselected dependency root admitted");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MissingDeclaration);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::MissingSelection)
    );
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{position}/body/abstract/package").as_str())
    );
}

#[trace("TC-048")]
#[test]
fn refinement_catalog_presence_precedes_declaration_backed_step_joins() {
    let (mut package, evidence) = concrete_protocol_refinement(
        ProtocolStepRows::Declared,
        ProtocolClauseOwner::SelectedModel,
    );
    let refinement = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
    let catalog_at = format!("/semantic_graph/nodes/{refinement}/body/class_catalog");
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package, &evidence) else {
        panic!("concrete protocol with no catalog must refuse");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(catalog_at.as_str())
    );

    let declared =
        package["semantic_graph"]["nodes"][refinement]["body"]["steps"][0]["source_node"].clone();
    package["semantic_graph"]["nodes"][refinement]["body"]["class_catalog"] = json!({
        "concrete": [{"class": "finish Other::Closed", "node": declared, "binder_types": []}],
        "abstract": []
    });
    rekey_refinement(&mut package, refinement);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package, &evidence) else {
        panic!("declaration-backed row without its catalog class must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MissingDeclaration);
    assert_eq!(refusal.cause, Some(CheckedPackageRefusalCause::MissingName));
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{refinement}/body/steps/0/source_class").as_str())
    );

    let (mut nonprotocol, evidence) = machine_package();
    let model = nonprotocol["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["semantic_form"] == "state_machine")
        .expect("state machine")["node_id"]
        .clone();
    one_step_refinement_with_step(
        &mut nonprotocol,
        json!({
            "source_class": "operation", "source_node": model,
            "target": {"kind": "stutter"}
        }),
    );
    let position = nonprotocol["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&nonprotocol, &evidence) else {
        panic!("non-protocol row with a wrong-kind source must refuse at the row");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidModelBinding);
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{position}/body/steps/0/source_node").as_str())
    );

    let (mut model_only, evidence) = machine_package();
    row_empty_refinement(&mut model_only);
    assert!(matches!(
        read(&model_only, &evidence),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    let position = model_only["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
    model_only["semantic_graph"]["nodes"][position]["body"]["class_catalog"] =
        json!({"concrete": [], "abstract": []});
    rekey_refinement(&mut model_only, position);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&model_only, &evidence) else {
        panic!("model-only refinement with a catalog must refuse");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph,
        "{refusal:?}"
    );
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{position}/body/class_catalog").as_str())
    );
}

#[trace("TC-062", "FR-061-AC-16")]
#[test]
fn refinement_catalog_present_step_refuses_wrong_row_source_node() {
    let (mut package, evidence) = concrete_protocol_refinement(
        ProtocolStepRows::Declared,
        ProtocolClauseOwner::SelectedModel,
    );
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    let finish = nodes
        .iter()
        .find(|node| node["node_tag"] == "protocol" && node["semantic_form"] == "obligation")
        .expect("finish obligation")["node_id"]
        .clone();
    let binder = node_id(&crate::support::checked_package::family_key("aaaa"));
    let position = set_concrete_catalog(
        &mut package,
        vec![json!({
            "class":"Example::OrderFlow::Closed", "node":finish, "binder_types":[binder]
        })],
    );
    package["semantic_graph"]["nodes"][position]["body"]["steps"][0]["source_class"] =
        json!("Example::OrderFlow::Closed");
    rekey_refinement(&mut package, position);
    let baseline = read(&package, &evidence);
    assert!(
        matches!(baseline, CheckedPackageV2ReadResult::Admitted(_)),
        "{baseline:?}"
    );

    let wrong = package["semantic_graph"]["nodes"][position]["semantic_type"].clone();
    package["semantic_graph"]["nodes"][position]["body"]["steps"][0]["source_node"] = wrong;
    rekey_refinement(&mut package, position);
    assert_step_refusal(
        read(&package, &evidence),
        CheckedPackageRefusalCode::InvalidModelBinding,
        Some(CheckedPackageRefusalCause::MalformedDeclaration),
        &format!("/semantic_graph/nodes/{position}/body/steps/0/source_node"),
    );
}

#[trace("TC-062", "FR-061-AC-16")]
#[test]
fn refinement_catalog_present_step_requires_a_concrete_class() {
    let (mut package, evidence) = concrete_protocol_refinement(
        ProtocolStepRows::Declared,
        ProtocolClauseOwner::SelectedModel,
    );
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    let finish = nodes
        .iter()
        .find(|node| node["node_tag"] == "protocol" && node["semantic_form"] == "obligation")
        .expect("finish obligation")["node_id"]
        .clone();
    let binder = node_id(&crate::support::checked_package::family_key("aaaa"));
    let position = set_concrete_catalog(
        &mut package,
        vec![json!({
            "class":"Example::OrderFlow::Closed", "node":finish, "binder_types":[binder]
        })],
    );
    package["semantic_graph"]["nodes"][position]["body"]["steps"][0]["source_class"] =
        json!("Other::Missing");
    rekey_refinement(&mut package, position);
    assert_step_refusal(
        read(&package, &evidence),
        CheckedPackageRefusalCode::MissingDeclaration,
        Some(CheckedPackageRefusalCause::MissingName),
        &format!("/semantic_graph/nodes/{position}/body/steps/0/source_class"),
    );
}

/// Trace: FR-064-AC-4
#[trace("TC-048")]
#[test]
fn refinement_step_defect_precedes_wrong_protocol_clause_owner() {
    let (mut package, evidence) = concrete_protocol_refinement(
        ProtocolStepRows::Declared,
        ProtocolClauseOwner::SelectedModel,
    );
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    let finish = nodes
        .iter()
        .find(|node| node["node_tag"] == "protocol" && node["semantic_form"] == "obligation")
        .expect("finish")["node_id"]
        .clone();
    let binder_type = node_id(&crate::support::checked_package::family_key("aaaa"));
    let refinement = set_concrete_catalog(
        &mut package,
        vec![json!({
            "class": "Example::OrderFlow::Closed", "node": finish,
            "binder_types": [binder_type]
        })],
    );
    package["semantic_graph"]["nodes"][refinement]["body"]["steps"][0]["source_class"] =
        json!("Example::OrderFlow::Closed");
    rekey_refinement(&mut package, refinement);
    let baseline = read(&package, &evidence);
    assert!(
        matches!(baseline, CheckedPackageV2ReadResult::Admitted(_)),
        "{baseline:?}"
    );

    let clause = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["semantic_form"] == "protocol_clause")
        .expect("clause");
    let previous_owner = package["semantic_graph"]["nodes"][clause]["owner_model"].clone();
    let wrong_owner = node_id(&crate::support::checked_package::family_key("2020"));
    let clause_node = &mut package["semantic_graph"]["nodes"][clause];
    clause_node["owner_model"] = wrong_owner.clone();
    let dependencies = clause_node["dependencies"]
        .as_array_mut()
        .expect("dependencies");
    dependencies.retain(|id| id != &previous_owner);
    dependencies.push(wrong_owner);
    dependencies.sort_by(|left, right| left["digest"].as_str().cmp(&right["digest"].as_str()));
    rekey_application_node(&mut package, clause);
    refresh_identity(&mut package);
    let CheckedPackageV2ReadResult::Refused(wrong_owner_refusal) = read(&package, &evidence) else {
        panic!("wrong-kind clause owner must refuse on its own");
    };
    assert_eq!(
        wrong_owner_refusal.code,
        CheckedPackageRefusalCode::InvalidModelBinding
    );
    assert_eq!(
        wrong_owner_refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{clause}/owner_model").as_str())
    );

    package["semantic_graph"]["nodes"][refinement]["body"]["steps"][0]["source_class"] =
        json!("Other::Closed");
    rekey_refinement(&mut package, refinement);

    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package, &evidence) else {
        panic!("refinement step defect must precede wrong clause owner");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::MissingDeclaration,
        "{refusal:?}"
    );
    assert_eq!(refusal.cause, Some(CheckedPackageRefusalCause::MissingName));
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{refinement}/body/steps/0/source_class").as_str())
    );
}

#[trace("TC-048")]
#[test]
fn refinement_catalog_presence_uses_clause_owner_without_step_rows() {
    let (owned, evidence) =
        concrete_protocol_refinement(ProtocolStepRows::Empty, ProtocolClauseOwner::SelectedModel);
    let position = owned["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
    let catalog_at = format!("/semantic_graph/nodes/{position}/body/class_catalog");
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&owned, &evidence) else {
        panic!("same-model clause without a named step still needs a catalog");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(catalog_at.as_str())
    );

    let (mut foreign, evidence) =
        concrete_protocol_refinement(ProtocolStepRows::Empty, ProtocolClauseOwner::OtherModel);
    let result = read(&foreign, &evidence);
    assert!(
        matches!(result, CheckedPackageV2ReadResult::Admitted(_)),
        "{result:?}"
    );
    let position = foreign["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
    foreign["semantic_graph"]["nodes"][position]["body"]["class_catalog"] =
        json!({"concrete": [], "abstract": []});
    rekey_refinement(&mut foreign, position);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&foreign, &evidence) else {
        panic!("catalog without same-model classes must refuse");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{position}/body/class_catalog").as_str())
    );

    let (foreign_row, evidence) =
        concrete_protocol_refinement(ProtocolStepRows::Declared, ProtocolClauseOwner::OtherModel);
    let position = foreign_row["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&foreign_row, &evidence) else {
        panic!("foreign protocol row must refuse at its binding");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidModelBinding);
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{position}/body/steps/0/source_node").as_str())
    );
}

/// Trace: FR-038-AC-325, FR-064-AC-2, FR-064-AC-5
/// Provenance: IR-1053 catalog graph join.
#[test]
fn refinement_catalog_joins_owned_finish_without_a_step_row() {
    let original = v2_all_families();
    let original_clause = original["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["semantic_form"] == "protocol_clause")
        .expect("original clause");
    let original_clause_key = original_clause["node_id"].clone();
    let original_package_id = original["package_id"].clone();
    let (mut package, evidence) =
        concrete_protocol_refinement(ProtocolStepRows::Empty, ProtocolClauseOwner::SelectedModel);
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    let object_b = nodes
        .iter()
        .find(|node| {
            node["semantic_form"] == "object_type" && node["owner"]["identity"] == MODEL_IDENTITY
        })
        .expect("second selected object type")["node_id"]
        .clone();
    let clause = nodes
        .iter()
        .find(|node| node["semantic_form"] == "protocol_clause")
        .expect("retargeted clause");
    assert_eq!(clause["owner_model"], object_b);
    assert_ne!(clause["owner_model"], original_clause["owner_model"]);
    assert_eq!(clause["body"], original_clause["body"]);
    let clause_key = clause["node_id"].clone();
    assert_ne!(clause["node_id"], original_clause_key);
    assert_ne!(package["package_id"], original_package_id);
    let concrete_root = nodes
        .iter()
        .find(|node| {
            node["semantic_form"] == "model_root" && node["owner"]["identity"] == MODEL_IDENTITY
        })
        .expect("selected concrete model root")["node_id"]
        .clone();
    assert_eq!(
        nodes.last().expect("refinement")["semantic_type"],
        concrete_root
    );
    let finish = nodes
        .iter()
        .find(|node| node["node_tag"] == "protocol" && node["semantic_form"] == "obligation")
        .expect("finish")["node_id"]
        .clone();
    let binder_type = nodes
        .iter()
        .find(|node| {
            node["node_id"]["digest"] == json!(crate::support::checked_package::family_key("aaaa"))
        })
        .expect("binder type")["node_id"]
        .clone();
    let position = set_concrete_catalog(
        &mut package,
        vec![
            json!({"class": "Example::OrderFlow::Closed", "node": finish, "binder_types": [binder_type]}),
        ],
    );
    assert_eq!(
        package["semantic_graph"]["nodes"][position]["body"]["class_catalog"]["concrete"]
            .as_array()
            .expect("B-owned classes")
            .len(),
        1
    );
    let admitted = read(&package, &evidence);
    assert!(
        matches!(admitted, CheckedPackageV2ReadResult::Admitted(_)),
        "{admitted:?}"
    );
    let written = writer_bytes(&package);
    assert_eq!(written, canonical(&package));
    let written_read =
        CheckedPackageV2::read(&written, CheckedPackageReadLimits::bounded(), &evidence);
    let CheckedPackageV2ReadResult::Admitted(round_trip) = written_read else {
        panic!("typed writer output must read back: {written_read:?}");
    };
    assert_eq!(
        serde_json::to_value(round_trip.package_id()).expect("package id"),
        package["package_id"]
    );
    let round_trip_clause = round_trip
        .graph()
        .nodes
        .iter()
        .find(|node| node.semantic_form.as_ref() == "protocol_clause")
        .expect("admitted clause");
    assert_eq!(
        serde_json::to_value(&round_trip_clause.node_id).expect("clause key"),
        clause_key
    );
    assert_eq!(
        serde_json::to_value(&round_trip_clause.owner_model).expect("owner edge"),
        object_b
    );
    let round_trip_refinement = round_trip
        .graph()
        .nodes
        .iter()
        .find(|node| node.semantic_form.as_ref() == "refinement")
        .expect("admitted refinement");
    assert_eq!(
        serde_json::to_value(&round_trip_refinement.node_id).expect("node id"),
        package["semantic_graph"]["nodes"][position]["node_id"]
    );

    let mut omitted = package.clone();
    set_concrete_catalog(&mut omitted, Vec::new());
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&omitted, &evidence) else {
        panic!("omitted owned class admitted");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{position}/body/class_catalog").as_str())
    );

    let mut wrong_node = package.clone();
    let control = wrong_node["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "protocol" && node["semantic_form"] == "control")
        .expect("control")["node_id"]
        .clone();
    wrong_node["semantic_graph"]["nodes"][position]["body"]["class_catalog"]["concrete"][0]
        ["node"] = control.clone();
    wrong_node["semantic_graph"]["nodes"][position]["dependencies"] = json!([
        control,
        binder_type,
        wrong_node["semantic_graph"]["nodes"][position]["semantic_type"].clone()
    ]);
    wrong_node["semantic_graph"]["nodes"][position]["dependencies"]
        .as_array_mut()
        .expect("dependencies")
        .sort_by(|left, right| left["digest"].as_str().cmp(&right["digest"].as_str()));
    rekey_refinement(&mut wrong_node, position);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&wrong_node, &evidence) else {
        panic!("wrong declaration association admitted");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidModelBinding);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::MalformedDeclaration)
    );
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{position}/body/class_catalog/concrete/0").as_str())
    );

    let (mut foreign, evidence) =
        concrete_protocol_refinement(ProtocolStepRows::Empty, ProtocolClauseOwner::OtherModel);
    let position = foreign["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
    foreign["semantic_graph"]["nodes"][position]["body"]["class_catalog"] = package
        ["semantic_graph"]["nodes"][package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1]["body"]["class_catalog"]
        .clone();
    foreign["semantic_graph"]["nodes"][position]["dependencies"] = package["semantic_graph"]
        ["nodes"][package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1]["dependencies"]
        .clone();
    rekey_refinement(&mut foreign, position);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&foreign, &evidence) else {
        panic!("foreign owned class admitted");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{position}/body/class_catalog/concrete/0").as_str())
    );
}

/// Provenance: IR-874 selected model operation class join.
#[test]
fn refinement_catalog_joins_concrete_operation_and_abstract_protocol_without_step_rows() {
    let (document, worker, reference, receiver_type) = operation_model_nodes();
    let worker_key = worker["node_id"].clone();
    let (mut package, evidence) = concrete_protocol_refinement_with_model(
        v2_all_families(),
        ProtocolStepRows::Empty,
        ProtocolClauseOwner::SelectedModel,
        document,
        vec![worker, reference],
    );
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    let finish = nodes
        .iter()
        .find(|node| node["body"]["operation"]["member"]["operator"] == "finish")
        .expect("finish")["node_id"]
        .clone();
    let finish_binder = node_id(&crate::support::checked_package::family_key("aaaa"));
    let integer_binder = node_id(&crate::support::checked_package::family_key("a3a3"));
    let entries = vec![
        json!({"class": "Example::OrderFlow::Closed", "node": finish, "binder_types": [finish_binder]}),
        json!({"class": "Worker::run", "node": worker_key, "binder_types": [receiver_type, integer_binder, finish_binder]}),
    ];
    let position = set_concrete_catalog(&mut package, entries.clone());
    package["semantic_graph"]["nodes"][position]["body"]["class_catalog"]["abstract"] =
        json!([entries[1].clone()]);
    rekey_refinement(&mut package, position);
    let admitted = read(&package, &evidence);
    assert!(
        matches!(admitted, CheckedPackageV2ReadResult::Admitted(_)),
        "concrete operation class must join through the selected root: {admitted:?}"
    );

    let clause = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["semantic_form"] == "protocol_clause")
        .expect("selected abstract clause")["node_id"]
        .clone();
    let mut abstract_protocol = package.clone();
    let refinement = &mut abstract_protocol["semantic_graph"]["nodes"][position];
    refinement["body"]["protocol"] = clause.clone();
    refinement["body"]["class_catalog"]["abstract"] = json!([entries[0].clone()]);
    let dependencies = refinement["dependencies"]
        .as_array_mut()
        .expect("dependencies");
    dependencies.push(clause);
    dependencies.sort_by(|left, right| left["digest"].as_str().cmp(&right["digest"].as_str()));
    dependencies.dedup();
    rekey_refinement(&mut abstract_protocol, position);
    let admitted = read(&abstract_protocol, &evidence);
    assert!(
        matches!(admitted, CheckedPackageV2ReadResult::Admitted(_)),
        "selected abstract protocol and concrete operation must both join: {admitted:?}"
    );

    let mut absent_abstract = abstract_protocol;
    absent_abstract["semantic_graph"]["nodes"][position]["body"]["class_catalog"]["abstract"] =
        json!([]);
    rekey_refinement(&mut absent_abstract, position);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&absent_abstract, &evidence) else {
        panic!("selected abstract protocol class omitted from catalog admitted");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{position}/body/class_catalog/abstract").as_str())
    );

    let mut omitted = package.clone();
    set_concrete_catalog(&mut omitted, vec![entries[0].clone()]);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&omitted, &evidence) else {
        panic!("omitted concrete operation class admitted");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{position}/body/class_catalog/concrete").as_str())
    );

    let mut wrong_node = package.clone();
    let machine = wrong_node["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["owner"]["node"] == MODEL_NODE)
        .expect("machine")["node_id"]
        .clone();
    let mut wrong_entries = entries.clone();
    wrong_entries[1]["node"] = machine;
    set_concrete_catalog(&mut wrong_node, wrong_entries);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&wrong_node, &evidence) else {
        panic!("operation class bound to another model declaration admitted");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidModelBinding);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::MalformedDeclaration)
    );
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{position}/body/class_catalog/concrete/1").as_str())
    );

    let mut reordered = package.clone();
    let mut wrong_entries = entries.clone();
    wrong_entries[1]["binder_types"] = json!([receiver_type, finish_binder, integer_binder]);
    set_concrete_catalog(&mut reordered, wrong_entries);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&reordered, &evidence) else {
        panic!("reordered operation parameters admitted");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidModelBinding);
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{position}/body/class_catalog/concrete/1").as_str())
    );

    let mut wrong_binder = package.clone();
    let mut wrong_entries = entries;
    wrong_entries[1]["binder_types"] = wrong_entries[0]["binder_types"].clone();
    set_concrete_catalog(&mut wrong_binder, wrong_entries);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&wrong_binder, &evidence) else {
        panic!("operation class bound to another receiver type admitted");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidModelBinding);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::MalformedDeclaration)
    );
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{position}/body/class_catalog/concrete/1").as_str())
    );
}

/// Trace: FR-038-AC-328
#[trace("TC-048", "FR-038-AC-328")]
#[test]
fn refinement_catalog_reconstructs_abstract_model_operations_without_protocol() {
    let (document, worker, reference, receiver_type) = operation_model_nodes();
    let worker_key = worker["node_id"].clone();
    let (mut package, evidence) = concrete_protocol_refinement_with_model(
        v2_all_families(),
        ProtocolStepRows::Empty,
        ProtocolClauseOwner::SelectedModel,
        document,
        vec![worker, reference],
    );
    let finish = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["body"]["operation"]["member"]["operator"] == "finish")
        .expect("finish")["node_id"]
        .clone();
    let finish_binder = node_id(&crate::support::checked_package::family_key("aaaa"));
    let integer_binder = node_id(&crate::support::checked_package::family_key("a3a3"));
    let operation = json!({
        "class":"Worker::run", "node":worker_key,
        "binder_types":[receiver_type, integer_binder, finish_binder]
    });
    let refinement = set_concrete_catalog(
        &mut package,
        vec![
            json!({"class":"Example::OrderFlow::Closed", "node":finish, "binder_types":[finish_binder]}),
            operation.clone(),
        ],
    );
    let abstract_model = package["semantic_graph"]["nodes"][refinement]["semantic_type"].clone();
    package["semantic_graph"]["nodes"][refinement]["body"]["abstract"] = abstract_model;
    package["semantic_graph"]["nodes"][refinement]["body"]["class_catalog"]["abstract"] =
        json!([operation]);
    rekey_refinement(&mut package, refinement);
    let admitted = read(&package, &evidence);
    assert!(
        matches!(admitted, CheckedPackageV2ReadResult::Admitted(_)),
        "{admitted:?}"
    );

    let mut missing = package.clone();
    missing["semantic_graph"]["nodes"][refinement]["body"]["class_catalog"]["abstract"] = json!([]);
    rekey_refinement(&mut missing, refinement);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&missing, &evidence) else {
        panic!("omitted abstract operation admitted");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{refinement}/body/class_catalog/abstract").as_str())
    );

    let mut extra = package;
    let foreign = json!({"class":"Worker::other", "node":worker_key,
        "binder_types":[receiver_type, integer_binder, finish_binder]});
    extra["semantic_graph"]["nodes"][refinement]["body"]["class_catalog"]["abstract"] = json!([
        foreign,
        extra["semantic_graph"]["nodes"][refinement]["body"]["class_catalog"]["abstract"][0]
            .clone()
    ]);
    rekey_refinement(&mut extra, refinement);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&extra, &evidence) else {
        panic!("extra abstract operation admitted");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{refinement}/body/class_catalog/abstract/0").as_str())
    );
}

/// Trace: FR-038-AC-325, FR-038-AC-330
#[trace("TC-048", "FR-038-AC-325", "FR-038-AC-330")]
#[test]
fn refinement_catalog_includes_compensation_template_end_without_a_step_row() {
    let (base, _, _) = crate::checked_package_v2_protocol_body::package_with_present_commit();
    let (mut package, evidence) = concrete_protocol_refinement_on(
        base,
        ProtocolStepRows::Empty,
        ProtocolClauseOwner::SelectedModel,
    );
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    let by_operator = |operator: &str| {
        nodes
            .iter()
            .find(|node| node["body"]["operation"]["member"]["operator"] == operator)
            .expect("protocol operator")["node_id"]
            .clone()
    };
    let template = by_operator("compensate");
    let finish = by_operator("finish");
    let commit = by_operator("commit");
    let event = by_operator("event");
    let attempt = by_operator("attempt");
    let effect = by_operator("effect");
    let boolean = node_id(&crate::support::checked_package::family_key("aaaa"));
    let anchor = nodes
        .iter()
        .find(|node| node["semantic_form"] == "operation_anchor")
        .expect("operation anchor");
    let context = &anchor["body"]["members"][0]["value"]["target"];
    let receiver = nodes
        .iter()
        .find(|node| {
            node["semantic_form"] == "reference" && node["body"]["members"][0]["target"] == *context
        })
        .expect("receiver type")["node_id"]
        .clone();
    let mut entries = vec![
        json!({"class":"Example::OrderFlow::Undo", "node":template, "binder_types":[receiver]}),
        json!({"class":"Example::OrderFlow::Closed", "node":finish, "binder_types":[boolean]}),
        json!({"class":"Example::OrderFlow::Commit", "node":commit, "binder_types":[boolean]}),
        json!({"class":"Example::OrderFlow::Event", "node":event, "binder_types":[boolean]}),
        json!({"class":"Example::OrderFlow::ForwardAttempt", "node":attempt, "binder_types":[receiver]}),
        json!({"class":"Example::OrderFlow::ForwardEffect", "node":effect, "binder_types":[boolean]}),
        json!({"class":"cend Example::OrderFlow::Undo", "node":template, "binder_types":[]}),
    ];
    entries.sort_by(|left, right| left["class"].as_str().cmp(&right["class"].as_str()));
    let refinement = set_concrete_catalog(&mut package, entries.clone());
    let admitted = read(&package, &evidence);
    assert!(
        matches!(admitted, CheckedPackageV2ReadResult::Admitted(_)),
        "{admitted:?}"
    );

    for (class_name, class_index, other_node) in [
        ("Commit", 1, effect.clone()),
        ("ForwardEffect", 4, commit.clone()),
    ] {
        let mut missing = package.clone();
        let mut changed = entries.clone();
        changed.remove(class_index);
        set_concrete_catalog(&mut missing, changed);
        let CheckedPackageV2ReadResult::Refused(refusal) = read(&missing, &evidence) else {
            panic!("omitted {class_name} admitted");
        };
        assert_eq!(
            refusal.code,
            CheckedPackageRefusalCode::InvalidSemanticGraph
        );
        assert_eq!(
            refusal.path.as_ref().map(|path| path.as_str()),
            Some(
                format!("/semantic_graph/nodes/{refinement}/body/class_catalog/concrete").as_str()
            )
        );

        let mut duplicate = package.clone();
        let mut changed = entries.clone();
        changed.insert(class_index + 1, changed[class_index].clone());
        set_concrete_catalog(&mut duplicate, changed);
        let CheckedPackageV2ReadResult::Refused(refusal) = read(&duplicate, &evidence) else {
            panic!("duplicate {class_name} admitted");
        };
        assert_eq!(
            refusal.code,
            CheckedPackageRefusalCode::InvalidSemanticGraph
        );
        assert_eq!(
            refusal.path.as_ref().map(|path| path.as_str()),
            Some(
                format!(
                    "/semantic_graph/nodes/{refinement}/body/class_catalog/concrete/{}",
                    class_index + 1
                )
                .as_str()
            )
        );

        let mut wrong_node = package.clone();
        let mut changed = entries.clone();
        changed[class_index]["node"] = other_node;
        set_concrete_catalog(&mut wrong_node, changed);
        let CheckedPackageV2ReadResult::Refused(refusal) = read(&wrong_node, &evidence) else {
            panic!("{class_name} associated with another node admitted");
        };
        assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidModelBinding);
        assert_eq!(
            refusal.cause,
            Some(CheckedPackageRefusalCause::MalformedDeclaration)
        );
        assert_eq!(
            refusal.path.as_ref().map(|path| path.as_str()),
            Some(
                format!(
                    "/semantic_graph/nodes/{refinement}/body/class_catalog/concrete/{class_index}"
                )
                .as_str()
            )
        );

        let mut wrong_binder = package.clone();
        let mut changed = entries.clone();
        changed[class_index]["binder_types"] = json!([receiver.clone()]);
        set_concrete_catalog(&mut wrong_binder, changed);
        let CheckedPackageV2ReadResult::Refused(refusal) = read(&wrong_binder, &evidence) else {
            panic!("{class_name} with wrong binder admitted");
        };
        assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidModelBinding);
        assert_eq!(
            refusal.cause,
            Some(CheckedPackageRefusalCause::MalformedDeclaration)
        );
        assert_eq!(
            refusal.path.as_ref().map(|path| path.as_str()),
            Some(
                format!(
                    "/semantic_graph/nodes/{refinement}/body/class_catalog/concrete/{class_index}"
                )
                .as_str()
            )
        );
    }

    let mut abstract_package = package.clone();
    let clause = abstract_package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["semantic_form"] == "protocol_clause")
        .expect("clause")["node_id"]
        .clone();
    let owner = &mut abstract_package["semantic_graph"]["nodes"][refinement];
    owner["body"]["protocol"] = clause.clone();
    owner["body"]["class_catalog"]["abstract"] = json!(entries.clone());
    owner["dependencies"]
        .as_array_mut()
        .expect("dependencies")
        .push(clause);
    owner["dependencies"]
        .as_array_mut()
        .expect("dependencies")
        .sort_by(|left, right| left["digest"].as_str().cmp(&right["digest"].as_str()));
    rekey_refinement(&mut abstract_package, refinement);
    let admitted = read(&abstract_package, &evidence);
    assert!(
        matches!(admitted, CheckedPackageV2ReadResult::Admitted(_)),
        "abstract template catalog must also admit: {admitted:?}"
    );

    let mut missing = package.clone();
    set_concrete_catalog(&mut missing, entries[..6].to_vec());
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&missing, &evidence) else {
        panic!("omitted cend admitted");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{refinement}/body/class_catalog/concrete").as_str())
    );

    let mut wrong_node = package.clone();
    let wrong = wrong_node["semantic_graph"]["nodes"][refinement]["body"]["class_catalog"]
        ["concrete"]
        .as_array()
        .expect("catalog")
        .clone();
    let mut wrong = wrong;
    wrong[6]["node"] = finish;
    set_concrete_catalog(&mut wrong_node, wrong);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&wrong_node, &evidence) else {
        panic!("cend on another node admitted");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidModelBinding);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::MalformedDeclaration)
    );
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{refinement}/body/class_catalog/concrete/6").as_str())
    );

    let (mut no_template, no_template_evidence) =
        concrete_protocol_refinement(ProtocolStepRows::Empty, ProtocolClauseOwner::SelectedModel);
    let no_template_finish = no_template["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["body"]["operation"]["member"]["operator"] == "finish")
        .expect("finish")["node_id"]
        .clone();
    let no_template_position = set_concrete_catalog(
        &mut no_template,
        vec![
            json!({"class":"Example::OrderFlow::Closed", "node":no_template_finish,
                "binder_types":[boolean]}),
            json!({"class":"cend Example::OrderFlow::Undo", "node":no_template_finish,
                "binder_types":[]}),
        ],
    );
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&no_template, &no_template_evidence)
    else {
        panic!("cend without a selected template admitted");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(
            format!("/semantic_graph/nodes/{no_template_position}/body/class_catalog/concrete/1")
                .as_str()
        )
    );
}

/// Trace: FR-038-AC-330
#[trace("TC-048", "FR-038-AC-330")]
#[test]
fn refinement_catalog_rejects_cend_associated_with_another_template() {
    let (mut base, _, _) =
        crate::checked_package_v2_protocol_body::package_with_admitted_compensation();
    let template_position = base["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["semantic_form"] == "compensation")
        .expect("template");
    let first_template = base["semantic_graph"]["nodes"][template_position]["node_id"].clone();
    let second_template = protocol_named_node(
        "compensation",
        &["Example", "OrderFlow", "compensate", "OtherUndo"],
        base["semantic_graph"]["nodes"][template_position]["body"].clone(),
        &[],
    );
    let second_key = second_template["node_id"].clone();
    base["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(second_template);
    let clause = base["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["semantic_form"] == "protocol_clause")
        .expect("clause");
    base["semantic_graph"]["nodes"][clause]["body"]["arguments"][8]["value"]["members"]
        .as_array_mut()
        .expect("compensations")
        .push(json!({"term":"reference", "target":second_key}));
    settle(&mut base);
    let (mut package, evidence) = concrete_protocol_refinement_on(
        base,
        ProtocolStepRows::Empty,
        ProtocolClauseOwner::SelectedModel,
    );
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    let by_operator = |operator: &str| {
        nodes
            .iter()
            .find(|node| node["body"]["operation"]["member"]["operator"] == operator)
            .expect("operator")["node_id"]
            .clone()
    };
    let finish = by_operator("finish");
    let event = by_operator("event");
    let attempt = by_operator("attempt");
    let effect = by_operator("effect");
    let anchor = nodes
        .iter()
        .find(|node| node["semantic_form"] == "operation_anchor")
        .expect("anchor");
    let context = &anchor["body"]["members"][0]["value"]["target"];
    let receiver = nodes
        .iter()
        .find(|node| {
            node["semantic_form"] == "reference" && node["body"]["members"][0]["target"] == *context
        })
        .expect("receiver")["node_id"]
        .clone();
    let boolean = node_id(&crate::support::checked_package::family_key("aaaa"));
    let mut entries = vec![
        json!({"class":"Example::OrderFlow::OtherUndo", "node":second_key, "binder_types":[receiver]}),
        json!({"class":"Example::OrderFlow::Undo", "node":first_template, "binder_types":[receiver]}),
        json!({"class":"Example::OrderFlow::Closed", "node":finish, "binder_types":[boolean]}),
        json!({"class":"Example::OrderFlow::Event", "node":event, "binder_types":[boolean]}),
        json!({"class":"Example::OrderFlow::ForwardAttempt", "node":attempt, "binder_types":[receiver]}),
        json!({"class":"Example::OrderFlow::ForwardEffect", "node":effect, "binder_types":[boolean]}),
        json!({"class":"cend Example::OrderFlow::OtherUndo", "node":second_key, "binder_types":[]}),
        json!({"class":"cend Example::OrderFlow::Undo", "node":first_template, "binder_types":[]}),
    ];
    entries.sort_by(|left, right| left["class"].as_str().cmp(&right["class"].as_str()));
    let refinement = set_concrete_catalog(&mut package, entries.clone());
    assert_unique_node_ids(&package);
    let admitted = read(&package, &evidence);
    assert!(
        matches!(admitted, CheckedPackageV2ReadResult::Admitted(_)),
        "{admitted:?}"
    );
    entries[7]["node"] = second_key;
    set_concrete_catalog(&mut package, entries);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package, &evidence) else {
        panic!("cend associated with another template admitted");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidModelBinding);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::MalformedDeclaration)
    );
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{refinement}/body/class_catalog/concrete/7").as_str())
    );
}

/// Provenance: IR-874 catalog graph join.
/// Trace: FR-038-AC-329
#[trace("TC-048", "FR-038-AC-329")]
#[test]
fn refinement_catalog_distinguishes_fork_and_join_on_one_parallel_node() {
    let (base, _) = crate::checked_package_v2_protocol_body::protocol_holder_case_on(
        v2_all_families(),
        "parallel",
    );
    let (mut package, evidence) = concrete_protocol_refinement_on(
        base,
        ProtocolStepRows::Empty,
        ProtocolClauseOwner::SelectedModel,
    );
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    let parallel = nodes
        .iter()
        .find(|node| node["body"]["operation"]["member"]["operator"] == "parallel")
        .expect("parallel")["node_id"]
        .clone();
    let finish = nodes
        .iter()
        .find(|node| node["body"]["operation"]["member"]["operator"] == "finish")
        .expect("finish")["node_id"]
        .clone();
    let binder_type = node_id(&crate::support::checked_package::family_key("aaaa"));
    let finish_entry =
        json!({"class":"Example::OrderFlow::Closed", "node":finish, "binder_types":[binder_type]});
    let fork_entry =
        json!({"class":"fork Example::OrderFlow::Holder", "node":parallel, "binder_types":[]});
    let join_entry =
        json!({"class":"join Example::OrderFlow::Holder", "node":parallel, "binder_types":[]});
    let position = set_concrete_catalog(
        &mut package,
        vec![
            finish_entry.clone(),
            fork_entry.clone(),
            join_entry,
            json!({"class":"memory drain", "memory_model":"sc", "drain":"store-buffer-flush"}),
        ],
    );
    let admitted = read(&package, &evidence);
    assert!(
        matches!(admitted, CheckedPackageV2ReadResult::Admitted(_)),
        "{admitted:?}"
    );

    for (class_index, wrong_class, path) in [
        (
            1,
            "fork OrderFlow::Holder",
            format!("/semantic_graph/nodes/{position}/body/class_catalog/concrete"),
        ),
        (
            1,
            "fork OtherFlow::Holder",
            format!("/semantic_graph/nodes/{position}/body/class_catalog/concrete"),
        ),
        (
            1,
            "fork Example::OrderFlow::run::Holder",
            format!("/semantic_graph/nodes/{position}/body/class_catalog/concrete"),
        ),
        (
            1,
            "fork Example::OrderFlow::Main::Holder",
            format!("/semantic_graph/nodes/{position}/body/class_catalog/concrete"),
        ),
        (
            2,
            "join Example::OrderFlow::Main::Holder",
            format!("/semantic_graph/nodes/{position}/body/class_catalog/concrete"),
        ),
    ] {
        let mut wrong = package.clone();
        wrong["semantic_graph"]["nodes"][position]["body"]["class_catalog"]["concrete"]
            [class_index]["class"] = json!(wrong_class);
        rekey_refinement(&mut wrong, position);
        let CheckedPackageV2ReadResult::Refused(refusal) = read(&wrong, &evidence) else {
            panic!("truncated or substituted declaration path admitted: {wrong_class}");
        };
        assert_eq!(
            refusal.code,
            CheckedPackageRefusalCode::InvalidSemanticGraph
        );
        assert_eq!(
            refusal.path.as_ref().map(|path| path.as_str()),
            Some(path.as_str()),
            "{wrong_class}"
        );
    }

    let mut omitted = package.clone();
    set_concrete_catalog(&mut omitted, vec![finish_entry, fork_entry]);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&omitted, &evidence) else {
        panic!("missing join class admitted");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{position}/body/class_catalog/concrete").as_str())
    );

    let mut abstract_package = package.clone();
    let clause = abstract_package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["semantic_form"] == "protocol_clause")
        .expect("clause")["node_id"]
        .clone();
    let node = &mut abstract_package["semantic_graph"]["nodes"][position];
    node["body"]["protocol"] = clause.clone();
    let concrete_entries = node["body"]["class_catalog"]["concrete"].clone();
    node["body"]["class_catalog"]["abstract"] = concrete_entries;
    node["dependencies"]
        .as_array_mut()
        .expect("dependencies")
        .push(clause);
    node["dependencies"]
        .as_array_mut()
        .expect("dependencies")
        .sort_by(|left, right| left["digest"].as_str().cmp(&right["digest"].as_str()));
    rekey_refinement(&mut abstract_package, position);
    let admitted = read(&abstract_package, &evidence);
    assert!(
        matches!(admitted, CheckedPackageV2ReadResult::Admitted(_)),
        "{admitted:?}"
    );

    let abstract_entries = abstract_package["semantic_graph"]["nodes"][position]["body"]
        ["class_catalog"]["abstract"]
        .as_array_mut()
        .expect("abstract classes");
    abstract_entries.remove(1); // omit fork before the still present join
    rekey_refinement(&mut abstract_package, position);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&abstract_package, &evidence) else {
        panic!("missing earlier abstract fork admitted");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{position}/body/class_catalog/abstract").as_str())
    );
}

/// Trace: FR-038-AC-329
#[trace("TC-048", "FR-038-AC-329")]
#[test]
fn refinement_catalog_preserves_two_clause_paths_of_different_depths() {
    let (mut base, _) = crate::checked_package_v2_protocol_body::protocol_holder_case_on(
        v2_all_families(),
        "parallel",
    );
    let mut nested = base.clone();
    for node in nested["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
    {
        if node["node_tag"] != "protocol" {
            continue;
        }
        let Some(path) = node["declaration"]["qualified_name"].as_array_mut() else {
            continue;
        };
        if path.len() >= 2 && path[0] == "Example" && path[1] == "OrderFlow" {
            path.insert(1, json!("Nested"));
            path[2] = json!("OtherFlow");
        }
    }
    settle(&mut nested);
    let second = nested["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .filter(|node| {
            node["node_tag"] == "protocol"
                && node["declaration"]["qualified_name"][0] == "Example"
                && node["declaration"]["qualified_name"][1] == "Nested"
        })
        .cloned()
        .collect::<Vec<_>>();
    base["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend(second);
    rebuild_source_map(&mut base);
    settle(&mut base);
    let (mut package, evidence) = concrete_protocol_refinement_on(
        base,
        ProtocolStepRows::Empty,
        ProtocolClauseOwner::SelectedModel,
    );
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    let mut entries = Vec::new();
    let boolean = node_id(&crate::support::checked_package::family_key("aaaa"));
    for node in nodes {
        if node["node_tag"] != "protocol" {
            continue;
        }
        let Some(operator) = node["body"]["operation"]["member"]["operator"].as_str() else {
            continue;
        };
        match operator {
            "finish" => entries.push(
                json!({"class":protocol_class_path(node), "node":node["node_id"],
                "binder_types":[boolean]}),
            ),
            "parallel" => {
                let path = protocol_class_path(node);
                entries.push(
                    json!({"class":format!("fork {path}"), "node":node["node_id"],
                    "binder_types":[]}),
                );
                entries.push(
                    json!({"class":format!("join {path}"), "node":node["node_id"],
                    "binder_types":[]}),
                );
            }
            _ => {}
        }
    }
    entries.push(json!({"class":"memory drain", "memory_model":"sc",
        "drain":"store-buffer-flush"}));
    entries.sort_by(|left, right| left["class"].as_str().cmp(&right["class"].as_str()));
    assert!(entries
        .iter()
        .any(|entry| entry["class"] == "Example::OrderFlow::Closed"));
    assert!(entries
        .iter()
        .any(|entry| entry["class"] == "Example::Nested::OtherFlow::Closed"));
    assert!(entries
        .iter()
        .any(|entry| entry["class"] == "fork Example::Nested::OtherFlow::Holder"));
    set_concrete_catalog(&mut package, entries.clone());
    assert_unique_node_ids(&package);
    let admitted = read(&package, &evidence);
    assert!(
        matches!(admitted, CheckedPackageV2ReadResult::Admitted(_)),
        "{admitted:?}"
    );

    for (original, substituted) in [
        (
            "Example::Nested::OtherFlow::Closed",
            "Example::Nested::OrderFlow::Closed",
        ),
        (
            "fork Example::Nested::OtherFlow::Holder",
            "fork Nested::OtherFlow::Holder",
        ),
    ] {
        let mut wrong = package.clone();
        let mut changed = entries.clone();
        let entry = changed
            .iter_mut()
            .find(|entry| entry["class"] == original)
            .expect("class in positive catalog");
        entry["class"] = json!(substituted);
        changed.sort_by(|left, right| left["class"].as_str().cmp(&right["class"].as_str()));
        let changed_index = changed
            .iter()
            .position(|entry| entry["class"] == substituted)
            .expect("substituted class in catalog");
        let position = set_concrete_catalog(&mut wrong, changed);
        let CheckedPackageV2ReadResult::Refused(refusal) = read(&wrong, &evidence) else {
            panic!("substituted class {substituted} admitted");
        };
        assert_eq!(
            refusal.code,
            CheckedPackageRefusalCode::InvalidSemanticGraph
        );
        assert_eq!(
            refusal.path.as_ref().map(|path| path.as_str()),
            Some(
                if substituted.starts_with("fork ") {
                    format!("/semantic_graph/nodes/{position}/body/class_catalog/concrete")
                } else {
                    format!(
                        "/semantic_graph/nodes/{position}/body/class_catalog/concrete/{changed_index}"
                    )
                }
                .as_str()
            ),
            "wrong refusal locus for {substituted}"
        );
    }
}

/// Trace: FR-038-AC-329
#[trace("TC-048", "FR-038-AC-329")]
#[test]
fn refinement_catalog_assigns_nested_members_only_to_inner_clause() {
    let (mut base, _) = crate::checked_package_v2_protocol_body::protocol_holder_case_on(
        v2_all_families(),
        "parallel",
    );
    let mut nested = base.clone();
    for node in nested["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
    {
        if node["node_tag"] != "protocol" {
            continue;
        }
        let Some(path) = node["declaration"]["qualified_name"].as_array_mut() else {
            continue;
        };
        if path.len() >= 2 && path[0] == "Example" && path[1] == "OrderFlow" {
            path.insert(2, json!("Inner"));
            for segment in path.iter_mut() {
                if *segment == "Closed" {
                    *segment = json!("InnerClosed");
                } else if *segment == "Holder" {
                    *segment = json!("InnerHolder");
                }
            }
        }
    }
    settle(&mut nested);
    let nested_nodes = nested["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .filter(|node| {
            node["node_tag"] == "protocol"
                && node["declaration"]["qualified_name"][0] == "Example"
                && node["declaration"]["qualified_name"][1] == "OrderFlow"
                && node["declaration"]["qualified_name"][2] == "Inner"
        })
        .cloned()
        .collect::<Vec<_>>();
    base["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend(nested_nodes);
    rebuild_source_map(&mut base);
    settle(&mut base);
    let (mut package, evidence) = concrete_protocol_refinement_on(
        base,
        ProtocolStepRows::Empty,
        ProtocolClauseOwner::SelectedModel,
    );
    let mut entries = Vec::new();
    let boolean = node_id(&crate::support::checked_package::family_key("aaaa"));
    for node in package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
    {
        let Some(operator) = node["body"]["operation"]["member"]["operator"].as_str() else {
            continue;
        };
        match operator {
            "finish" => entries.push(json!({"class":protocol_class_path(node),
                "node":node["node_id"], "binder_types":[boolean]})),
            "parallel" => {
                let path = protocol_class_path(node);
                for kind in ["fork", "join"] {
                    entries.push(json!({"class":format!("{kind} {path}"),
                        "node":node["node_id"], "binder_types":[]}));
                }
            }
            _ => {}
        }
    }
    entries.push(json!({"class":"memory drain", "memory_model":"sc",
        "drain":"store-buffer-flush"}));
    entries.sort_by(|left, right| left["class"].as_str().cmp(&right["class"].as_str()));
    assert!(entries
        .iter()
        .any(|entry| entry["class"] == "Example::OrderFlow::Inner::InnerClosed"));
    assert!(entries
        .iter()
        .any(|entry| entry["class"] == "fork Example::OrderFlow::Inner::InnerHolder"));
    set_concrete_catalog(&mut package, entries.clone());
    assert_unique_node_ids(&package);
    let admitted = read(&package, &evidence);
    assert!(
        matches!(admitted, CheckedPackageV2ReadResult::Admitted(_)),
        "nested clause catalog must admit: {admitted:?}"
    );

    let mut wrong = package.clone();
    let inner_finish = entries
        .iter_mut()
        .find(|entry| entry["class"] == "Example::OrderFlow::Inner::InnerClosed")
        .expect("inner finish");
    inner_finish["class"] = json!("Example::OrderFlow::InnerClosed");
    entries.sort_by(|left, right| left["class"].as_str().cmp(&right["class"].as_str()));
    let position = set_concrete_catalog(&mut wrong, entries);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&wrong, &evidence) else {
        panic!("outer-derived class for inner finish admitted");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{position}/body/class_catalog/concrete").as_str())
    );
}

/// Trace: FR-038-AC-329
#[trace("TC-048", "FR-038-AC-329")]
#[test]
fn refinement_catalog_reads_single_segment_protocol_clause() {
    let (mut base, _) = crate::checked_package_v2_protocol_body::protocol_holder_case_on(
        v2_all_families(),
        "parallel",
    );
    for node in base["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
    {
        if node["node_tag"] != "protocol" {
            continue;
        }
        let Some(path) = node["declaration"]["qualified_name"].as_array_mut() else {
            continue;
        };
        if path.len() >= 2 && path[0] == "Example" && path[1] == "OrderFlow" {
            path.drain(..2);
            path.insert(0, json!("Impl"));
        }
    }
    settle(&mut base);
    let (mut package, evidence) = concrete_protocol_refinement_on(
        base,
        ProtocolStepRows::Empty,
        ProtocolClauseOwner::SelectedModel,
    );
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    let by_operator = |operator: &str| {
        nodes
            .iter()
            .find(|node| node["body"]["operation"]["member"]["operator"] == operator)
            .expect("protocol operator")["node_id"]
            .clone()
    };
    let finish = by_operator("finish");
    let parallel = by_operator("parallel");
    let binder = node_id(&crate::support::checked_package::family_key("aaaa"));
    set_concrete_catalog(
        &mut package,
        vec![
            json!({"class":"Impl::Closed", "node":finish, "binder_types":[binder]}),
            json!({"class":"fork Impl::Holder", "node":parallel, "binder_types":[]}),
            json!({"class":"join Impl::Holder", "node":parallel, "binder_types":[]}),
            json!({"class":"memory drain", "memory_model":"sc",
                "drain":"store-buffer-flush"}),
        ],
    );
    let admitted = read(&package, &evidence);
    assert!(
        matches!(admitted, CheckedPackageV2ReadResult::Admitted(_)),
        "{admitted:?}"
    );
}

/// Provenance: IR-874 timeout class on an admitted await node.
/// Trace: FR-038-AC-325
#[trace("TC-048", "FR-038-AC-325")]
#[test]
fn refinement_catalog_joins_timeout_on_await_node() {
    let (base, _) = crate::checked_package_v2_protocol_body::protocol_holder_case_on(
        v2_all_families(),
        "await",
    );
    let (mut package, evidence) = concrete_protocol_refinement_on(
        base,
        ProtocolStepRows::Empty,
        ProtocolClauseOwner::SelectedModel,
    );
    let binder = node_id(&crate::support::checked_package::family_key("aaaa"));
    let mut entries = Vec::new();
    for node in package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
    {
        let Some(operator) = node["body"]["operation"]["member"]["operator"].as_str() else {
            continue;
        };
        if node["declaration"].is_null() {
            continue;
        }
        let binder_types = match operator {
            "await" => json!([]),
            "event" | "finish" => json!([binder]),
            _ => continue,
        };
        entries.push(json!({
            "class": protocol_class_path(node),
            "node": node["node_id"],
            "binder_types": binder_types
        }));
    }
    entries.sort_by(|left, right| left["class"].as_str().cmp(&right["class"].as_str()));
    assert_eq!(entries.len(), 4, "finish, event, two await timeouts");
    set_concrete_catalog(&mut package, entries);
    let admitted = read(&package, &evidence);
    assert!(
        matches!(admitted, CheckedPackageV2ReadResult::Admitted(_)),
        "{admitted:?}"
    );
}

/// Provenance: IR-874 catalog graph join.
#[test]
fn refinement_catalog_uses_channel_carries_for_send_binder() {
    let (base, _, _) = crate::checked_package_v2_protocol_body::package_with_send();
    let (mut package, evidence) = concrete_protocol_refinement_on(
        base,
        ProtocolStepRows::Empty,
        ProtocolClauseOwner::SelectedModel,
    );
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    let send_node = nodes
        .iter()
        .find(|node| node["body"]["operation"]["member"]["operator"] == "send")
        .expect("send");
    let send = send_node["node_id"].clone();
    let send_name = protocol_class_path(send_node);
    let channel_node = nodes
        .iter()
        .find(|node| node["semantic_form"] == "channel")
        .expect("channel");
    let channel = channel_node["node_id"].clone();
    let channel_name = protocol_class_path(channel_node);
    let finish = nodes
        .iter()
        .find(|node| node["node_tag"] == "protocol" && node["semantic_form"] == "obligation")
        .expect("finish")["node_id"]
        .clone();
    let binder = node_id(&crate::support::checked_package::family_key("aaaa"));
    let mut entries = vec![
        json!({"class":"Example::OrderFlow::Closed", "node":finish, "binder_types":[binder]}),
        json!({"class":send_name, "node":send,
            "binder_types":[binder]}),
        json!({"class":format!("duplicate {channel_name}"),
            "node":channel, "binder_types":[]}),
        json!({"class":format!("lose {channel_name}"),
            "node":channel, "binder_types":[]}),
    ];
    entries.sort_by(|left, right| left["class"].as_str().cmp(&right["class"].as_str()));
    let position = set_concrete_catalog(&mut package, entries);
    let admitted = read(&package, &evidence);
    assert!(
        matches!(admitted, CheckedPackageV2ReadResult::Admitted(_)),
        "{admitted:?}"
    );

    let mut wrong = package.clone();
    let send_index = wrong["semantic_graph"]["nodes"][position]["body"]["class_catalog"]
        ["concrete"]
        .as_array()
        .expect("catalog")
        .iter()
        .position(|entry| entry["class"] == send_name)
        .expect("send entry");
    wrong["semantic_graph"]["nodes"][position]["body"]["class_catalog"]["concrete"][send_index]
        ["binder_types"] = json!([]);
    let wrong_entries = wrong["semantic_graph"]["nodes"][position]["body"]["class_catalog"]
        ["concrete"]
        .as_array()
        .expect("catalog")
        .clone();
    set_concrete_catalog(&mut wrong, wrong_entries);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&wrong, &evidence) else {
        panic!("send with wrong binder admitted");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidModelBinding);
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(
            format!("/semantic_graph/nodes/{position}/body/class_catalog/concrete/{send_index}")
                .as_str()
        )
    );
}

/// Provenance: IR-874 catalog graph join.
#[test]
fn refinement_catalog_joins_selected_abstract_clause_and_binder() {
    let (mut package, evidence) =
        concrete_protocol_refinement(ProtocolStepRows::Empty, ProtocolClauseOwner::SelectedModel);
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    let clause = nodes
        .iter()
        .find(|node| node["semantic_form"] == "protocol_clause")
        .expect("clause")["node_id"]
        .clone();
    let finish = nodes
        .iter()
        .find(|node| node["node_tag"] == "protocol" && node["semantic_form"] == "obligation")
        .expect("finish")["node_id"]
        .clone();
    let binder = node_id(&crate::support::checked_package::family_key("aaaa"));
    let position = set_concrete_catalog(
        &mut package,
        vec![json!({"class":"Example::OrderFlow::Closed", "node":finish,
            "binder_types":[binder]})],
    );
    let node = &mut package["semantic_graph"]["nodes"][position];
    node["body"]["protocol"] = clause.clone();
    node["body"]["class_catalog"]["abstract"] = json!([{
        "class":"Example::OrderFlow::Closed", "node":finish, "binder_types":[binder]
    }]);
    let mut dependencies = node["dependencies"]
        .as_array()
        .expect("dependencies")
        .clone();
    dependencies.push(clause);
    dependencies.sort_by(|left, right| left["digest"].as_str().cmp(&right["digest"].as_str()));
    dependencies.dedup();
    node["dependencies"] = json!(dependencies);
    rekey_refinement(&mut package, position);
    let admitted = read(&package, &evidence);
    assert!(
        matches!(admitted, CheckedPackageV2ReadResult::Admitted(_)),
        "{admitted:?}"
    );

    let mut wrong_binder = package.clone();
    wrong_binder["semantic_graph"]["nodes"][position]["body"]["class_catalog"]["abstract"][0]
        ["binder_types"] = json!([]);
    rekey_refinement(&mut wrong_binder, position);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&wrong_binder, &evidence) else {
        panic!("abstract class with wrong binder admitted");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidModelBinding);
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{position}/body/class_catalog/abstract/0").as_str())
    );
}

/// Provenance: IR-874 catalog graph join.
#[test]
fn refinement_catalog_joins_abstract_class_from_direct_dependency() {
    let (mut dependency_wire, dependency_evidence) =
        concrete_protocol_refinement(ProtocolStepRows::Empty, ProtocolClauseOwner::SelectedModel);
    let dependency_nodes = dependency_wire["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    let abstract_model = dependency_nodes
        .iter()
        .find(|node| {
            node["semantic_form"] == "model_root" && node["owner"]["identity"] == MODEL_IDENTITY
        })
        .expect("model")["node_id"]
        .clone();
    let abstract_clause = dependency_nodes
        .iter()
        .find(|node| node["semantic_form"] == "protocol_clause")
        .expect("clause")["node_id"]
        .clone();
    let abstract_finish = dependency_nodes
        .iter()
        .find(|node| node["node_tag"] == "protocol" && node["semantic_form"] == "obligation")
        .expect("finish")["node_id"]
        .clone();
    let binder = node_id(&crate::support::checked_package::family_key("aaaa"));
    set_concrete_catalog(
        &mut dependency_wire,
        vec![
            json!({"class":"Example::OrderFlow::Closed", "node":abstract_finish,
            "binder_types":[binder]}),
        ],
    );
    let CheckedPackageV2ReadResult::Admitted(dependency) =
        read(&dependency_wire, &dependency_evidence)
    else {
        panic!("direct dependency fixture must admit");
    };
    let dependency_id = dependency_wire["package_id"].clone();
    let external = |node: Value| json!({"package":dependency_id, "node":node});

    let (mut package, mut evidence) =
        concrete_protocol_refinement(ProtocolStepRows::Empty, ProtocolClauseOwner::SelectedModel);
    let concrete_finish = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "protocol" && node["semantic_form"] == "obligation")
        .expect("finish")["node_id"]
        .clone();
    let position = set_concrete_catalog(
        &mut package,
        vec![
            json!({"class":"Example::OrderFlow::Closed", "node":concrete_finish,
            "binder_types":[binder]}),
        ],
    );
    let node = &mut package["semantic_graph"]["nodes"][position];
    node["body"]["abstract"] = external(abstract_model);
    node["body"]["protocol"] = external(abstract_clause);
    node["body"]["class_catalog"]["abstract"] = json!([{
        "class":"Example::OrderFlow::Closed", "node":external(abstract_finish),
        "binder_types":[external(binder.clone())]
    }]);
    node["dependencies"] = json!([concrete_finish, binder]);
    node["dependencies"]
        .as_array_mut()
        .expect("dependencies")
        .sort_by(|left, right| left["digest"].as_str().cmp(&right["digest"].as_str()));
    rekey_refinement(&mut package, position);
    package["lock"]["dependency_selections"] = json!([{
        "identity":"test/abstract-protocol", "package_id":dependency_id
    }]);
    refresh_identity(&mut package);
    evidence
        .insert_dependency_package("test/abstract-protocol", *dependency)
        .expect("selected dependency");
    let admitted = read(&package, &evidence);
    assert!(
        matches!(admitted, CheckedPackageV2ReadResult::Admitted(_)),
        "{admitted:?}"
    );

    let mut wrong_binder = package.clone();
    wrong_binder["semantic_graph"]["nodes"][position]["body"]["class_catalog"]["abstract"][0]
        ["binder_types"] = json!([]);
    rekey_refinement(&mut wrong_binder, position);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&wrong_binder, &evidence) else {
        panic!("external abstract class with wrong binder admitted");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidModelBinding);
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{position}/body/class_catalog/abstract/0").as_str())
    );
}

/// Trace: FR-038-AC-328
#[trace("TC-048", "FR-038-AC-328")]
#[test]
fn refinement_catalog_joins_external_abstract_model_operations_without_protocol() {
    let (document, worker, reference, receiver) = operation_model_nodes();
    let worker_key = worker["node_id"].clone();
    let (mut dependency_wire, dependency_evidence) = concrete_protocol_refinement_with_model(
        v2_all_families(),
        ProtocolStepRows::Empty,
        ProtocolClauseOwner::SelectedModel,
        document,
        vec![worker, reference],
    );
    let dependency_nodes = dependency_wire["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    let abstract_model = dependency_nodes
        .iter()
        .find(|node| {
            node["semantic_form"] == "model_root" && node["owner"]["identity"] == MODEL_IDENTITY
        })
        .expect("abstract model")["node_id"]
        .clone();
    let finish = dependency_nodes
        .iter()
        .find(|node| node["body"]["operation"]["member"]["operator"] == "finish")
        .expect("finish")["node_id"]
        .clone();
    let boolean = node_id(&crate::support::checked_package::family_key("aaaa"));
    let integer = node_id(&crate::support::checked_package::family_key("a3a3"));
    let operation = json!({"class":"Worker::run", "node":worker_key,
        "binder_types":[receiver, integer, boolean]});
    let dep_refinement = set_concrete_catalog(
        &mut dependency_wire,
        vec![
            json!({"class":"Example::OrderFlow::Closed", "node":finish,
            "binder_types":[boolean]}),
            operation.clone(),
        ],
    );
    dependency_wire["semantic_graph"]["nodes"][dep_refinement]["body"]["class_catalog"]
        ["abstract"] = json!([operation]);
    rekey_refinement(&mut dependency_wire, dep_refinement);
    let CheckedPackageV2ReadResult::Admitted(dependency) =
        read(&dependency_wire, &dependency_evidence)
    else {
        panic!("selected dependency must admit");
    };
    let dependency_id = dependency_wire["package_id"].clone();
    let external = |node: Value| json!({"package":dependency_id, "node":node});

    let (mut package, mut evidence) =
        concrete_protocol_refinement(ProtocolStepRows::Empty, ProtocolClauseOwner::SelectedModel);
    let concrete_finish = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["body"]["operation"]["member"]["operator"] == "finish")
        .expect("concrete finish")["node_id"]
        .clone();
    let position = set_concrete_catalog(
        &mut package,
        vec![
            json!({"class":"Example::OrderFlow::Closed", "node":concrete_finish,
            "binder_types":[boolean]}),
        ],
    );
    let node = &mut package["semantic_graph"]["nodes"][position];
    node["body"]["abstract"] = external(abstract_model);
    node["body"]["class_catalog"]["abstract"] = json!([{
        "class":"Worker::run", "node":external(worker_key),
        "binder_types":[external(receiver), external(integer), external(boolean.clone())]
    }]);
    node["dependencies"] = json!([concrete_finish, boolean]);
    node["dependencies"]
        .as_array_mut()
        .expect("dependencies")
        .sort_by(|left, right| left["digest"].as_str().cmp(&right["digest"].as_str()));
    rekey_refinement(&mut package, position);
    package["lock"]["dependency_selections"] = json!([{
        "identity":"test/abstract-model", "package_id":dependency_id
    }]);
    refresh_identity(&mut package);
    evidence
        .insert_dependency_package("test/abstract-model", *dependency)
        .expect("selected dependency");
    let admitted = read(&package, &evidence);
    assert!(
        matches!(admitted, CheckedPackageV2ReadResult::Admitted(_)),
        "{admitted:?}"
    );

    let mut missing = package;
    missing["semantic_graph"]["nodes"][position]["body"]["class_catalog"]["abstract"] = json!([]);
    rekey_refinement(&mut missing, position);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&missing, &evidence) else {
        panic!("external abstract operation omitted from catalog admitted");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{position}/body/class_catalog/abstract").as_str())
    );
}

#[test]
fn refinement_state_machine_owner_and_kind_are_exact() {
    let (package, evidence) = machine_package();
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["semantic_form"] == "state_machine")
        .expect("state machine");
    let mut wrong_owner = package.clone();
    wrong_owner["semantic_graph"]["nodes"][position]["owner"]["node"] =
        json!("ix://test/refinement/Absent");
    refresh_identity(&mut wrong_owner);
    assert!(matches!(
        read(&wrong_owner, &evidence),
        CheckedPackageV2ReadResult::Refused(_)
    ));

    let mut wrong_kind = package;
    wrong_kind["semantic_graph"]["nodes"][position]["semantic_form"] = json!("object_type");
    refresh_identity(&mut wrong_kind);
    assert!(matches!(
        read(&wrong_kind, &evidence),
        CheckedPackageV2ReadResult::Refused(_)
    ));
}

#[trace("TC-056", "FR-040-AC-39")]
#[test]
fn process_and_machine_clause_signatures_use_declared_context() {
    use crate::support::checked_package::{BOOLEAN_KEY, INTEGER_KEY};
    for form in ["process", "state_machine"] {
        let (mut package, evidence, refinement) = catalog_free_operation_package(form);
        let original_nodes = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .len();
        let original_source_map = package["source_map"].clone();
        let nodes = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes");
        let context = nodes
            .iter()
            .find(|node| node["semantic_form"] == form)
            .expect("context")["node_id"]
            .clone();
        let anchor = nodes
            .iter()
            .find(|node| node["semantic_form"] == "operation_anchor")
            .expect("anchor")["node_id"]
            .clone();
        let account = nodes
            .iter()
            .find(|node| node["semantic_form"] == "object_type")
            .expect("account")["node_id"]
            .clone();
        let text_type = node_id(&crate::support::checked_package::family_key("a1a1"));
        let mk_ref_type = |target: &Value| {
            let body = json!({"term":"aggregate","members":[{"term":"reference","target":target}]});
            let key = structural_key("composite_type", "reference", None, &body);
            (key, body)
        };
        let (self_type, self_body) = mk_ref_type(&context);
        let (argument_type, argument_body) = mk_ref_type(&account);
        let mk_parameter = |name: &str, ty: &str, level: &str| {
            let body = json!({"term":"aggregate","members":[
                {"term":"binding","name":"name","value":{"term":"literal","type":text_type,"value_kind":"text","value":name}},
                {"term":"binding","name":"level","value":{"term":"literal","type":node_id(INTEGER_KEY),"value_kind":"integer","value":level}}
            ]});
            let key = structural_key("value", "parameter", Some(ty), &body);
            (key, body)
        };
        let (self_key, self_parameter) = mk_parameter("self", &self_type, "0");
        let (argument_key, argument_parameter) = mk_parameter(
            if form == "process" { "step" } else { "target" },
            &argument_type,
            "1",
        );
        let (result_key, result_parameter) = mk_parameter("result", INTEGER_KEY, "1");
        let family = v2_all_families();
        for key in [
            BOOLEAN_KEY,
            INTEGER_KEY,
            &crate::support::checked_package::family_key("dddd"),
        ] {
            let node = family["semantic_graph"]["nodes"]
                .as_array()
                .expect("family nodes")
                .iter()
                .find(|node| node["node_id"] == node_id(key))
                .expect("family type or condition")
                .clone();
            let nodes = package["semantic_graph"]["nodes"]
                .as_array_mut()
                .expect("nodes");
            if !nodes
                .iter()
                .any(|existing| existing["node_id"] == node["node_id"])
            {
                nodes.push(node);
            }
        }
        let node = |key: &str,
                    tag: &str,
                    semantic_form: &str,
                    ty: &str,
                    deps: Vec<Value>,
                    role: &str,
                    body: Value| {
            json!({"node_id":node_id(key),"schema_version":"quire.checked-semantic-graph/v2",
                "node_tag":tag,"semantic_form":semantic_form,"semantic_type":node_id(ty),
                "dependencies":deps,"occurrences":[{"role":role,"ordinal":0}],"body":body})
        };
        let added = vec![
            node(
                &self_type,
                "composite_type",
                "reference",
                &self_type,
                vec![context.clone()],
                "type",
                self_body,
            ),
            node(
                &argument_type,
                "composite_type",
                "reference",
                &argument_type,
                vec![account],
                "type",
                argument_body,
            ),
            node(
                &self_key,
                "value",
                "parameter",
                &self_type,
                vec![],
                "expression",
                self_parameter,
            ),
            node(
                &argument_key,
                "value",
                "parameter",
                &argument_type,
                vec![],
                "expression",
                argument_parameter,
            ),
            node(
                &result_key,
                "value",
                "parameter",
                INTEGER_KEY,
                vec![],
                "expression",
                result_parameter,
            ),
        ];
        for node in added {
            let nodes = package["semantic_graph"]["nodes"]
                .as_array_mut()
                .expect("nodes");
            if !nodes
                .iter()
                .any(|existing| existing["node_id"] == node["node_id"])
            {
                nodes.push(node);
            }
        }
        let condition = node_id(&crate::support::checked_package::family_key("dddd"));
        let mut positions = Vec::new();
        for (index, (clause, anchor_id, params)) in [
            ("invariant", context.clone(), vec![self_key.as_str()]),
            ("constraint", context.clone(), vec![self_key.as_str()]),
            (
                "precondition",
                anchor.clone(),
                vec![self_key.as_str(), argument_key.as_str()],
            ),
            (
                "postcondition",
                anchor.clone(),
                vec![
                    self_key.as_str(),
                    result_key.as_str(),
                    argument_key.as_str(),
                ],
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let members: Vec<Value> = params
                .iter()
                .map(|key| json!({"term":"reference","target":node_id(key)}))
                .collect();
            let body = json!({"term":"application","operator":"state_clause",
            "operation":{"identity":"quire.op.state.clause","laws":[],"mode":null,
                "member":{"kind":"state_clause","clause":clause},"leaves":[]},
            "result_type":node_id(BOOLEAN_KEY),"arguments":[
                {"term":"aggregate","members":members},
                {"term":"reference","target":anchor_id},
                {"term":"reference","target":condition}
            ]});
            let mut deps: Vec<Value> = params
                .iter()
                .map(|key| node_id(key))
                .chain([anchor_id, condition.clone()])
                .collect();
            deps.sort_by(|left, right| left["digest"].as_str().cmp(&right["digest"].as_str()));
            deps.dedup();
            let position = package["semantic_graph"]["nodes"]
                .as_array()
                .expect("nodes")
                .len();
            positions.push(position);
            let key = format!("{:02x}", 0xe0 + index).repeat(32);
            package["semantic_graph"]["nodes"]
                .as_array_mut()
                .expect("nodes")
                .push(node(
                    &key,
                    "state",
                    "state_clause",
                    BOOLEAN_KEY,
                    deps,
                    "claim",
                    body,
                ));
        }
        crate::support::checked_package::mint_ungrouped_structural_keys(&mut package);
        rekey_refinement(&mut package, refinement);
        settle(&mut package);
        let source = package["lock"]["sources"][0].clone();
        let mut source_map = original_source_map.as_array().expect("source map").clone();
        for node in &package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")[original_nodes..]
        {
            let occurrence = &node["occurrences"][0];
            source_map.push(json!({"node_id":node["node_id"],"role":occurrence["role"],
                "ordinal":occurrence["ordinal"],"regions":[{"source":source,"start":10,"end":11}]}));
        }
        package["source_map"] = json!(source_map);
        refresh_identity(&mut package);
        assert_unique_node_ids(&package);
        let admitted = read(&package, &evidence);
        assert!(
            matches!(admitted, CheckedPackageV2ReadResult::Admitted(_)),
            "{form}: {admitted:?}"
        );
        for position in positions {
            let mut wrong = package.clone();
            let old_clause = wrong["semantic_graph"]["nodes"][position]["node_id"].clone();
            wrong["semantic_graph"]["nodes"][position]["body"]["arguments"][0]["members"][0]
                ["target"] = node_id(&result_key);
            settle(&mut wrong);
            let new_clause = wrong["semantic_graph"]["nodes"][position]["node_id"].clone();
            let mut source_map = package["source_map"].clone();
            for entry in source_map.as_array_mut().expect("source map") {
                if entry["node_id"] == old_clause {
                    entry["node_id"] = new_clause.clone();
                }
            }
            wrong["source_map"] = source_map;
            refresh_identity(&mut wrong);
            assert_step_refusal(
                read(&wrong, &evidence),
                CheckedPackageRefusalCode::IllTyped,
                Some(CheckedPackageRefusalCause::OperatorIneligible),
                &format!("/semantic_graph/nodes/{position}/body/arguments/0"),
            );
        }
    }
}
