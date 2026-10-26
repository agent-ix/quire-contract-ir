// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Focused FR-061 refinement and selected state-machine owner vectors.

use crate::support::checked_package::{
    canonical, evidence_for, node_id, refresh_identity, settle, sha256_hex, structural_key,
    v2_all_families, v2_nominal,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    encode_checked_package_v2, CheckedCapability, CheckedDiagnosticsV2, CheckedPackageEvidence,
    CheckedPackageIdentityPreimageRefV2, CheckedPackageIdentityPreimageV2, CheckedPackageLockV2,
    CheckedPackageReadLimits, CheckedPackageRefusalCause, CheckedPackageRefusalCode,
    CheckedPackageV2, CheckedPackageV2ReadResult, CheckedRefinementStepRowRegion,
    CheckedSemanticGraphV2, CheckedSemanticId, CheckedSourceMapEntry, CheckedTerm,
};
use serde_json::{json, Value};

const MODEL_IDENTITY: &str = "test/refinement";
const MODEL_NODE: &str = "ix://test/refinement/Machine";

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
            "supertypes": [], "fields": [], "operations": []
        }]
    })
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
    let machine = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["semantic_form"] == "state_machine")
        .expect("machine")["node_id"]
        .clone();
    let profile = json!({"authority": "test", "identity": "quire.temporal.infinite-trace/v1"});
    package["lock"]["profile_selections"] = json!([{
        "role": "temporal_profile", "definition": profile
    }]);
    let body = json!({
        "term": "refinement", "abstract": machine, "profile": profile,
        "populations": [], "objects": [], "steps": [], "histories": [],
        "assume": [], "ensure": []
    });
    let preimage = json!({
        "version": "quire.refinement-node/v1", "semantic_type": machine, "body": body
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
            "semantic_type": machine, "dependencies": [machine],
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
    if let Some(rows) = package["refinement_step_rows"].as_array_mut() {
        for row in rows {
            if row["node_id"] == old {
                row["node_id"] = replacement.clone();
            }
        }
    }
    refresh_identity(package);
}

fn concrete_protocol_refinement() -> (Value, CheckedPackageEvidence) {
    let (machine, _) = machine_package();
    let model = machine["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .last()
        .expect("state machine")
        .clone();
    let model_id = model["node_id"].clone();
    let document = model_document();
    let digest = sha256_hex(&canonical(&document));
    let mut package = v2_all_families();
    package["lock"]["model_selections"]
        .as_array_mut()
        .expect("model selections")
        .push(json!({
            "identity": MODEL_IDENTITY, "digest_domain": "sha256-jcs", "digest": digest
        }));
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(model);

    let mut over = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| {
            node["node_tag"] == "value"
                && node["semantic_form"] == "parameter"
                && node["body"]["members"][0]["value"]["value"] == "over"
        })
        .expect("protocol over")
        .clone();
    over["semantic_type"] = model_id.clone();
    over["node_id"] = node_id(&structural_key(
        "value",
        "parameter",
        model_id["digest"].as_str(),
        &over["body"],
    ));
    let over_id = over["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(over);
    let clause = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["semantic_form"] == "protocol_clause")
        .expect("protocol clause");
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][0]["value"]["target"] = over_id;
    settle(&mut package);

    let finish = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "protocol" && node["semantic_form"] == "obligation")
        .expect("declared finish")["node_id"]
        .clone();
    let profiles = package["lock"]["profile_selections"].clone();
    one_step_refinement_with_step(
        &mut package,
        json!({
            "source_class": "finish OrderFlow::Closed", "source_node": finish,
            "target": {"kind": "stutter"}
        }),
    );
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
    for entry in [
        json!({"class":"memory flush","memory_model":"tso","node":base["semantic_graph"]["nodes"][refinement]["semantic_type"]}),
        json!({"class":"memory flush","memory_model":"unknown"}),
        json!({"class":"finish P::end","node":base["semantic_graph"]["nodes"][refinement]["semantic_type"]}),
    ] {
        package = base.clone();
        package["semantic_graph"]["nodes"][refinement]["body"]["class_catalog"] =
            json!({"concrete":[entry],"abstract":[]});
        let CheckedPackageV2ReadResult::Refused(refusal) = read(&package, &evidence) else {
            panic!("malformed catalog entry admitted");
        };
        assert_eq!(
            refusal.code,
            CheckedPackageRefusalCode::InvalidSemanticGraph
        );
        assert_eq!(
            refusal.path.as_ref().map(|path| path.as_str()),
            Some(
                format!("/semantic_graph/nodes/{refinement}/body/class_catalog/concrete/0")
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

/// Trace: FR-038-AC-325
#[trace("TC-048")]
#[test]
fn refinement_catalog_presence_precedes_declaration_backed_step_joins() {
    let (mut package, evidence) = concrete_protocol_refinement();
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

    package["semantic_graph"]["nodes"][refinement]["body"]["class_catalog"] =
        json!({"concrete": [], "abstract": []});
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
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
    assert_eq!(
        refusal.path.as_ref().map(|path| path.as_str()),
        Some(format!("/semantic_graph/nodes/{position}/body/class_catalog").as_str())
    );
}

#[test]
fn refinement_state_machine_owner_and_kind_are_exact() {
    let (package, evidence) = machine_package();
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
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
