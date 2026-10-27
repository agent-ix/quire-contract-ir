// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Focused FR-061 refinement and selected state-machine owner vectors.

use crate::support::checked_package::{
    canonical, evidence_for, node_id, refresh_identity, settle, sha256_hex, v2_all_families,
    v2_nominal,
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
    let (machine, _) = machine_package();
    let model = machine["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["semantic_form"] == "state_machine")
        .expect("state machine")
        .clone();
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
        .push(model.clone());
    if matches!(clause_owner, ProtocolClauseOwner::SelectedModel) {
        let clause = package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .iter_mut()
            .find(|node| node["semantic_form"] == "protocol_clause")
            .expect("protocol clause");
        clause["owner_model"] = model["node_id"].clone();
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

/// Provenance: IR-874 catalog graph join.
#[test]
fn refinement_catalog_joins_owned_finish_without_a_step_row() {
    let (mut package, evidence) =
        concrete_protocol_refinement(ProtocolStepRows::Empty, ProtocolClauseOwner::SelectedModel);
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
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
    let position = nodes.len() - 1;
    package["semantic_graph"]["nodes"][position]["body"]["class_catalog"] = json!({
        "concrete": [{"class": "OrderFlow::Closed", "node": finish, "binder_types": [binder_type]}],
        "abstract": []
    });
    let dependencies = package["semantic_graph"]["nodes"][position]["dependencies"]
        .as_array_mut()
        .expect("dependencies");
    dependencies.push(finish.clone());
    dependencies.push(binder_type.clone());
    dependencies.sort_by(|left, right| left["digest"].as_str().cmp(&right["digest"].as_str()));
    dependencies.dedup();
    rekey_refinement(&mut package, position);
    let admitted = read(&package, &evidence);
    assert!(
        matches!(admitted, CheckedPackageV2ReadResult::Admitted(_)),
        "{admitted:?}"
    );

    let mut omitted = package.clone();
    omitted["semantic_graph"]["nodes"][position]["body"]["class_catalog"]["concrete"] = json!([]);
    omitted["semantic_graph"]["nodes"][position]["dependencies"] =
        json!([omitted["semantic_graph"]["nodes"][position]["semantic_type"].clone()]);
    rekey_refinement(&mut omitted, position);
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
