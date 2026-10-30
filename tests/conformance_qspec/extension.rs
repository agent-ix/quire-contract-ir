// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-348 / QSpec TC-951 selected-checkout extension-node vectors.

use super::*;

fn read_with_retention_definition(
    package: &Value,
    document: &Value,
    definition: CheckedExtensionDefinition,
) -> CheckedPackageV2ReadResult {
    let mut evidence = CheckedPackageEvidence::new();
    for feature in package["lock"]["required_features"]
        .as_array()
        .expect("features")
    {
        evidence.support_feature(feature.as_str().expect("feature"));
    }
    for identity in fixture_support::selected_model_identities(package) {
        evidence
            .insert_domain_package_document(
                identity.as_str(),
                serde_json::to_vec(document).expect("domain document bytes"),
            )
            .expect("domain evidence");
    }
    evidence
        .insert_extension_definition(
            CheckedArtifactRef {
                authority: "example".into(),
                identity: "retention".into(),
            },
            definition,
        )
        .expect("extension evidence");
    CheckedPackageV2::read(
        &serde_json::to_vec(package).expect("package bytes"),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    )
}

fn replace_extension_subject(package: &mut Value, form: usize, target: Value) {
    package["semantic_graph"]["nodes"][form]["body"]["clauses"][0]["value"]["target"] = target;
    let mut dependencies = package["semantic_graph"]["nodes"][form]["body"]["clauses"]
        .as_array()
        .expect("clauses")
        .iter()
        .map(|clause| clause["value"]["target"].clone())
        .collect::<Vec<_>>();
    sort_node_ids(&mut dependencies);
    dependencies.dedup();
    package["semantic_graph"]["nodes"][form]["dependencies"] = json!(dependencies);
    refresh_extension(package, form);
}

fn record_extension_case(actual: &str) -> (Value, Value, usize) {
    let mut package =
        read_json(&fixtures_dir().join("positive-all-families.json")).expect("published package");
    let mut document = read_json(&proposal_dir_of(&fixtures_dir()).join(DOMAIN_DOCUMENT))
        .expect("published selected document");
    document["constructs"]
        .as_array_mut()
        .expect("constructs")
        .push(json!({
            "kind":{"module":"acme/orders","name":"record"},
            "construct":{"meaning":"quire.meaning.model.record-value-type/v1"}
        }));
    for (name, supertypes) in [
        ("Base", vec![]),
        ("Child", vec!["ix://acme/orders/Base"]),
        ("Unrelated", vec![]),
    ] {
        document["types"]
            .as_array_mut()
            .expect("types")
            .push(json!({
                "identity":format!("ix://acme/orders/{name}"), "displayName":name,
                "kind":{"module":"acme/orders","name":"record"},
                "roles":[], "constraints":[], "extensions":[], "unknownPolicy":"reject",
                "supertypes":supertypes, "fields":[], "operations":[]
            }));
    }

    let mut actual_id = Value::Null;
    for (offset, name) in ["Base", "Child", "Unrelated"].iter().enumerate() {
        let position = append_structural_node(
            &mut package,
            16,
            ['a', 'b', 'c'][offset],
            310 + 20 * offset as u64,
        );
        let provisional = package["semantic_graph"]["nodes"][position]["node_id"].clone();
        let node = &mut package["semantic_graph"]["nodes"][position];
        node.as_object_mut()
            .expect("model node")
            .remove("declaration");
        node["owner"] = json!({"kind":"model","identity":"acme/orders","node":format!("ix://acme/orders/{name}")});
        node["semantic_type"] = provisional;
        node["occurrences"] = json!([{"role":"type","ordinal":0}]);
        refresh_extension(&mut package, position);
        package["source_map"]
            .as_array_mut()
            .expect("source map")
            .last_mut()
            .expect("new model source")["role"] = json!("type");
        if *name == actual {
            actual_id = package["semantic_graph"]["nodes"][position]["node_id"].clone();
        }
    }
    let parameter = append_structural_node(&mut package, 27, 'd', 390);
    package["semantic_graph"]["nodes"][parameter]["semantic_type"] = actual_id;
    refresh_extension(&mut package, parameter);
    let form = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| {
            node["node_tag"] == "extension" && node["body"]["node_kind"] == "retention"
        })
        .expect("retention form");
    let target = package["semantic_graph"]["nodes"][parameter]["node_id"].clone();
    replace_extension_subject(&mut package, form, target);
    (package, document, form)
}

#[trace("TC-459", "FR-348-AC-5")]
#[test]
fn tc_459_extension_record_clause_uses_selected_model_directional_conformance() {
    let mut definition = retention_definition();
    definition.typed_node_schema.nodes[0].clauses[0].type_ref = "ix://acme/orders/Base".into();
    for actual in ["Base", "Child"] {
        let (package, document, _) = record_extension_case(actual);
        let read = read_with_retention_definition(&package, &document, definition.clone());
        assert!(
            matches!(read, CheckedPackageV2ReadResult::Admitted(_)),
            "{actual}: {read:?}"
        );
    }
    let (package, document, form) = record_extension_case("Unrelated");
    let read = read_with_retention_definition(&package, &document, definition);
    let CheckedPackageV2ReadResult::Refused(refusal) = read else {
        panic!("unrelated record must refuse: {read:?}");
    };
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::IllTyped, Some(Cause::TypeMismatch))
    );
    assert_eq!(
        refusal.loci[0].path.clone().expect("path").to_string(),
        format!("/semantic_graph/nodes/{form}/body/clauses/0/value")
    );

    let (package, document, form) = record_extension_case("Base");
    let mut reverse = retention_definition();
    reverse.typed_node_schema.nodes[0].clauses[0].type_ref = "ix://acme/orders/Child".into();
    let read = read_with_retention_definition(&package, &document, reverse);
    let CheckedPackageV2ReadResult::Refused(refusal) = read else {
        panic!("ancestor at descendant must refuse: {read:?}");
    };
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::IllTyped, Some(Cause::TypeMismatch))
    );
    assert_eq!(
        refusal.loci[0].path.clone().expect("path").to_string(),
        format!("/semantic_graph/nodes/{form}/body/clauses/0/value")
    );
}

#[trace("TC-459", "FR-348-AC-5")]
#[test]
fn tc_459_extension_composite_record_name_is_not_selected_model_type() {
    let mut package =
        read_json(&fixtures_dir().join("positive-all-families.json")).expect("published package");
    let form = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| {
            node["node_tag"] == "extension" && node["body"]["node_kind"] == "retention"
        })
        .expect("retention form");
    let parameter = append_structural_node(&mut package, 27, 'e', 310);
    package["semantic_graph"]["nodes"][parameter]["semantic_type"] =
        package["semantic_graph"]["nodes"][1]["node_id"].clone();
    refresh_extension(&mut package, parameter);
    let target = package["semantic_graph"]["nodes"][parameter]["node_id"].clone();
    replace_extension_subject(&mut package, form, target);
    let mut definition = retention_definition();
    definition.typed_node_schema.nodes[0].clauses[0].type_ref = "Example.Point".into();
    let document =
        read_json(&proposal_dir_of(&fixtures_dir()).join(DOMAIN_DOCUMENT)).expect("document");
    let read = read_with_retention_definition(&package, &document, definition);
    let CheckedPackageV2ReadResult::Refused(refusal) = read else {
        panic!("composite record must refuse: {read:?}");
    };
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::IllTyped, Some(Cause::TypeMismatch))
    );
    assert_eq!(
        refusal.loci[0].path.clone().expect("path").to_string(),
        format!("/semantic_graph/nodes/{form}/body/clauses/0/value")
    );
}

#[trace("TC-459", "FR-348-AC-3")]
#[test]
fn tc_459_unheld_extension_refuses_at_its_definition() {
    let package = read_json(&fixtures_dir().join("positive-extension-nodes.json"))
        .expect("published QSpec extension fixture");
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("published graph nodes");
    let (position, node) = nodes
        .iter()
        .enumerate()
        .find(|(_, node)| node["node_tag"] == "extension")
        .expect("published extension node");
    let document = proposal_dir_of(&fixtures_dir()).join(DOMAIN_DOCUMENT);
    let read =
        read_package_with_document_and_extensions(&package, "unheld extension", &document, false)
            .expect("production read");
    let CheckedPackageV2ReadResult::Refused(refusal) = read else {
        panic!("unheld extension must refuse: {read:?}");
    };
    assert_eq!(refusal.code, Code::UnknownRequiredCapability);
    assert_eq!(refusal.cause, None);
    assert_eq!(
        refusal
            .document_pointer
            .as_ref()
            .expect("document pointer")
            .to_string(),
        format!("/semantic_graph/nodes/{position}/body/definition")
    );
    assert_eq!(
        refusal.loci[0]
            .path
            .clone()
            .expect("definition pointer")
            .to_string(),
        format!("/semantic_graph/nodes/{position}/body/definition")
    );
    assert_eq!(
        serde_json::to_value(refusal.locus.expect("extension node locus")).expect("node id"),
        node["node_id"]
    );
}

#[trace("TC-459", "FR-348-AC-1", "FR-348-AC-8", "FR-349-AC-2")]
#[test]
fn tc_459_held_extension_fixtures_admit() {
    for name in [
        "positive-extension-nodes.json",
        "positive-all-families.json",
    ] {
        let package = read_json(&fixtures_dir().join(name)).expect("published QSpec fixture");
        let selected = package["lock"]["definition_selections"]
            .as_array()
            .expect("definitions");
        let features = package["lock"]["required_features"]
            .as_array()
            .expect("features");
        assert!(selected.iter().any(|row| row["identity"] == "retention"));
        assert!(features.iter().all(|feature| feature != "retention"));
        let nodes = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes");
        let form = nodes
            .iter()
            .find(|node| {
                node["node_tag"] == "extension" && node["body"]["node_kind"] == "retention"
            })
            .expect("retention form");
        assert_eq!(form["semantic_form"], "extension_form");
        assert_eq!(form["declaration"]["qualified_name"], json!(["Audit"]));
        assert_eq!(form["owner"]["kind"], "source");
        assert_eq!(
            form["occurrences"],
            json!([{"role":"declaration", "ordinal":0}])
        );
        assert_eq!(
            form["body"]["definition"],
            json!({"authority":"example", "identity":"retention"})
        );
        assert_eq!(form["body"]["clauses"][0]["name"], "subject");
        assert_eq!(form["body"]["clauses"][1]["name"], "days");
        let mut targets = form["body"]["clauses"]
            .as_array()
            .expect("clauses")
            .iter()
            .map(|clause| {
                assert_eq!(clause["term"], "binding");
                assert_eq!(clause["value"]["term"], "reference");
                clause["value"]["target"].clone()
            })
            .collect::<Vec<_>>();
        sort_node_ids(&mut targets);
        assert_eq!(form["dependencies"], json!(targets));
        for (ordinal, expected_form) in [(0, "text"), (1, "integer_range")] {
            let target = &form["body"]["clauses"][ordinal]["value"]["target"];
            let value_node = nodes
                .iter()
                .find(|node| node["node_id"] == *target)
                .expect("clause subnode");
            let type_node = nodes
                .iter()
                .find(|node| node["node_id"] == value_node["semantic_type"])
                .expect("clause type");
            assert_eq!(type_node["semantic_form"], expected_form);
        }
        let entry = package["source_map"]
            .as_array()
            .expect("source map")
            .iter()
            .find(|entry| entry["node_id"] == form["node_id"])
            .expect("form region");
        assert_eq!(entry["regions"].as_array().expect("regions").len(), 1);
        assert_eq!(
            entry["regions"][0]["end"].as_u64().expect("end")
                - entry["regions"][0]["start"].as_u64().expect("start"),
            46
        );
        let read = read_package(&package).expect("production read");
        let CheckedPackageV2ReadResult::Admitted(admitted) = read else {
            panic!("{name}: {read:?}");
        };
        let node_id = serde_json::from_value(form["node_id"].clone()).expect("form node id");
        let Some(quire_contract_model::CheckedTerm::ExtensionForm { clauses, .. }) =
            admitted.term(&node_id)
        else {
            panic!("{name}: admitted extension form has typed body");
        };
        assert_eq!(clauses.len(), 2);
        assert_eq!(
            serde_json::to_value(admitted.term(&node_id)).expect("typed extension serializes"),
            form["body"]
        );
    }
}

#[trace("TC-459", "FR-348-AC-3")]
#[test]
fn tc_459_extension_evidence_conflict_retains_first_definition() {
    let reference = CheckedArtifactRef {
        authority: "example".into(),
        identity: "retention".into(),
    };
    let original = retention_definition();
    let mut evidence = CheckedPackageEvidence::new();
    evidence
        .insert_extension_definition(reference.clone(), original.clone())
        .expect("first insertion");
    evidence
        .insert_extension_definition(reference.clone(), original.clone())
        .expect("equal repeat");
    let mut conflicting = original;
    conflicting.typed_node_schema.nodes[0].clauses[1].type_ref = "Text".into();
    let err = evidence
        .insert_extension_definition(reference, conflicting)
        .expect_err("conflict");
    assert!(matches!(
        err,
        CheckedPackageEvidenceInsertionError::ExtensionDefinitionConflict { .. }
    ));
    let (package, _) = extension_fixture();
    let bytes = serde_json::to_vec(&package).expect("package bytes");
    for feature in package["lock"]["required_features"]
        .as_array()
        .expect("features")
    {
        evidence.support_feature(feature.as_str().expect("feature"));
    }
    let read = CheckedPackageV2::read(&bytes, CheckedPackageReadLimits::bounded(), &evidence);
    assert!(
        matches!(read, CheckedPackageV2ReadResult::Admitted(_)),
        "{read:?}"
    );
}

fn extension_fixture() -> (Value, usize) {
    let package = read_json(&fixtures_dir().join("positive-extension-nodes.json"))
        .expect("published QSpec extension fixture");
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_tag"] == "extension")
        .expect("extension node");
    (package, position)
}

/// Reconcile changed declarations and their references under structural identity.
/// A body-only edit leaves the node id unchanged by design.
fn refresh_extension(package: &mut Value, _position: usize) {
    fixture_support::refresh_identity(package);
}

fn append_structural_node(
    package: &mut Value,
    template: usize,
    placeholder: char,
    start: u64,
) -> usize {
    let original = package["semantic_graph"]["nodes"][template].clone();
    let original_id = original["node_id"].clone();
    let placeholder_id = fixture_support::structural_id(
        original_id["package"].as_str().expect("template package"),
        vec![
            original["node_tag"].clone(),
            original["semantic_form"].clone(),
            json!(format!("test-{placeholder}")),
        ],
        0,
    );
    let mut node = original.clone();
    node["node_id"] = placeholder_id.clone();
    if original["semantic_type"] == original_id {
        node["semantic_type"] = placeholder_id.clone();
    }
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(node.clone());
    let mut source = package["source_map"]
        .as_array()
        .expect("source map")
        .iter()
        .find(|entry| entry["node_id"] == original_id)
        .expect("template source map entry")
        .clone();
    source["node_id"] = placeholder_id;
    source["regions"][0]["start"] = json!(start);
    source["regions"][0]["end"] = json!(start + 10);
    package["source_map"]
        .as_array_mut()
        .expect("source map")
        .push(source);
    position
}

// FR-348-AC-2 is withdrawn under QSpec #517: content edits retain a valid
// structural id. The remaining TC-459 cases assert the independent schema,
// dependency, owner, clause and selected-definition checks.

fn assert_extension_refusal(
    package: &Value,
    code: Code,
    cause: Option<Cause>,
    path: String,
) -> quire_contract_model::CheckedPackageRefusal {
    let read = read_package(package).expect("production read");
    let CheckedPackageV2ReadResult::Refused(refusal) = read else {
        panic!("extension mutation must refuse: {read:?}");
    };
    assert_eq!(refusal.code, code);
    assert_eq!(refusal.cause, cause);
    assert_eq!(
        refusal.loci[0]
            .path
            .as_ref()
            .expect("refusal path")
            .to_string(),
        path
    );
    refusal
}

#[trace("TC-459", "FR-348-AC-3")]
#[test]
fn tc_459_unselected_extension_refuses_before_held_lookup() {
    let (mut package, position) = extension_fixture();
    package["lock"]["definition_selections"] = json!([]);
    assert_extension_refusal(
        &package,
        Code::MissingDeclaration,
        Some(Cause::MissingSelection),
        format!("/semantic_graph/nodes/{position}/body/definition"),
    );
}

#[trace("TC-459", "FR-348-AC-4")]
#[test]
fn tc_459_extension_clause_names_follow_held_schema_order() {
    let (mut package, position) = extension_fixture();
    package["semantic_graph"]["nodes"][position]["body"]["node_kind"] = json!("retentions");
    refresh_extension(&mut package, position);
    assert_extension_refusal(
        &package,
        Code::MissingDeclaration,
        Some(Cause::MissingName),
        format!("/semantic_graph/nodes/{position}/body/node_kind"),
    );

    let (mut package, position) = extension_fixture();
    package["semantic_graph"]["nodes"][position]["body"]["clauses"]
        .as_array_mut()
        .expect("clauses")
        .swap(0, 1);
    refresh_extension(&mut package, position);
    assert_extension_refusal(
        &package,
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        format!("/semantic_graph/nodes/{position}/body/clauses/0"),
    );

    let (mut package, position) = extension_fixture();
    package["semantic_graph"]["nodes"][position]["body"]["clauses"]
        .as_array_mut()
        .expect("clauses")
        .pop();
    package["semantic_graph"]["nodes"][position]["dependencies"] = json!([
        package["semantic_graph"]["nodes"][position]["body"]["clauses"][0]["value"]["target"]
    ]);
    refresh_extension(&mut package, position);
    let missing = assert_extension_refusal(
        &package,
        Code::InvalidPackage,
        Some(Cause::MissingMember),
        format!("/semantic_graph/nodes/{position}/body/clauses"),
    );
    assert_eq!(
        missing
            .extension_detail
            .expect("missing clause")
            .member
            .as_ref(),
        "days"
    );

    for name in ["owner", "subject"] {
        let (mut package, position) = extension_fixture();
        package["semantic_graph"]["nodes"][position]["body"]["clauses"][1]["name"] = json!(name);
        refresh_extension(&mut package, position);
        assert_extension_refusal(
            &package,
            Code::InvalidPackage,
            Some(Cause::InvalidValue),
            format!("/semantic_graph/nodes/{position}/body/clauses/1"),
        );
    }
}

#[trace("TC-459", "FR-348-AC-5")]
#[test]
fn tc_459_extension_clause_targets_are_typed_and_present() {
    let (mut package, position) = extension_fixture();
    let subject = package["semantic_graph"]["nodes"][position]["body"]["clauses"][0]["value"]
        ["target"]
        .clone();
    package["semantic_graph"]["nodes"][position]["body"]["clauses"][1]["value"]["target"] =
        subject.clone();
    package["semantic_graph"]["nodes"][position]["dependencies"] = json!([subject]);
    refresh_extension(&mut package, position);
    let mismatch = assert_extension_refusal(
        &package,
        Code::IllTyped,
        Some(Cause::TypeMismatch),
        format!("/semantic_graph/nodes/{position}/body/clauses/1/value"),
    );
    let detail = mismatch.extension_detail.expect("clause type detail");
    assert_eq!(detail.member.as_ref(), "days");
    assert_eq!(detail.expected_type.as_deref(), Some("Int[1, 3650]"));
    assert_eq!(detail.actual_type.as_deref(), Some("Text"));
    let text_value = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| {
            node["node_id"]
                == package["semantic_graph"]["nodes"][position]["body"]["clauses"][0]["value"]
                    ["target"]
        })
        .expect("text clause subnode");
    assert_eq!(
        serde_json::to_value(detail.actual_semantic_type.as_ref().expect("semantic type"))
            .expect("node id"),
        text_value["semantic_type"]
    );

    let (mut package, position) = extension_fixture();
    let absent = fixture_support::structural_id(
        "example",
        vec![json!("value"), json!("literal"), json!("absent")],
        0,
    );
    package["semantic_graph"]["nodes"][position]["body"]["clauses"][1]["value"]["target"] =
        absent.clone();
    let subject = package["semantic_graph"]["nodes"][position]["body"]["clauses"][0]["value"]
        ["target"]
        .clone();
    package["semantic_graph"]["nodes"][position]["dependencies"] = json!([subject, absent]);
    refresh_extension(&mut package, position);
    assert_extension_refusal(
        &package,
        Code::MissingDeclaration,
        Some(Cause::MissingName),
        format!("/semantic_graph/nodes/{position}/body/clauses/1/value"),
    );
}

#[trace("TC-459", "FR-348-AC-6", "FR-349-AC-4")]
#[test]
fn tc_459_extension_wire_and_dependency_shape_refuse_early() {
    let (mut package, position) = extension_fixture();
    package["semantic_graph"]["nodes"][position]["body"]["clauses"][0]["value"] = json!({"term":"literal","value_kind":"text","value":"audit","type":package["semantic_graph"]["nodes"][position]["semantic_type"]});
    assert_extension_refusal(
        &package,
        Code::MalformedWire,
        None,
        format!("/semantic_graph/nodes/{position}/body"),
    );

    let (mut package, position) = extension_fixture();
    let occurrence = package["semantic_graph"]["nodes"][position]["occurrences"][0].clone();
    package["semantic_graph"]["nodes"][position]["occurrences"]
        .as_array_mut()
        .expect("occurrences")
        .push(occurrence);
    assert_extension_refusal(
        &package,
        Code::MalformedWire,
        None,
        format!("/semantic_graph/nodes/{position}/occurrences"),
    );

    let (mut package, position) = extension_fixture();
    package["semantic_graph"]["nodes"][position]["dependencies"]
        .as_array_mut()
        .expect("dependencies")
        .pop();
    assert_extension_refusal(
        &package,
        Code::InvalidSemanticGraph,
        None,
        format!("/semantic_graph/nodes/{position}/dependencies"),
    );

    let (mut package, position) = extension_fixture();
    package["semantic_graph"]["nodes"][position]["occurrences"][0]["role"] = json!("expression");
    assert_extension_refusal(
        &package,
        Code::MalformedWire,
        None,
        format!("/semantic_graph/nodes/{position}/occurrences"),
    );

    let (mut package, position) = extension_fixture();
    package["semantic_graph"]["nodes"][position]["semantic_form"] = json!("retention");
    assert_extension_refusal(
        &package,
        Code::MalformedWire,
        None,
        format!("/semantic_graph/nodes/{position}/semantic_form"),
    );
}

#[trace("TC-459", "FR-348-AC-1")]
#[test]
fn tc_459_extension_has_one_whole_form_source_region() {
    let (mut package, position) = extension_fixture();
    let id = package["semantic_graph"]["nodes"][position]["node_id"].clone();
    let entry_index = package["source_map"]
        .as_array()
        .expect("source map")
        .iter()
        .position(|entry| entry["node_id"] == id)
        .expect("form source entry");
    let mut second = package["source_map"][entry_index]["regions"][0].clone();
    second["start"] = json!(300);
    second["end"] = json!(310);
    package["source_map"][entry_index]["regions"]
        .as_array_mut()
        .expect("regions")
        .push(second);
    assert_extension_refusal(
        &package,
        Code::InvalidSourceMap,
        None,
        format!("/source_map/{entry_index}/regions"),
    );
}

#[trace("TC-459", "FR-348-AC-6")]
#[test]
fn tc_459_extension_and_record_share_the_types_namespace() {
    let mut package = read_json(&fixtures_dir().join("positive-all-families.json"))
        .expect("published QSpec package");
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    let record = nodes
        .iter()
        .position(|node| node["node_tag"] == "composite_type" && node["semantic_form"] == "record")
        .expect("record");
    let extension = nodes
        .iter()
        .position(|node| node["node_tag"] == "extension")
        .expect("extension");
    package["semantic_graph"]["nodes"][record]["declaration"]["qualified_name"] = json!(["Audit"]);
    refresh_extension(&mut package, record);
    let record_id: CheckedNodeId =
        serde_json::from_value(package["semantic_graph"]["nodes"][record]["node_id"].clone())
            .expect("record id");
    let extension_id: CheckedNodeId =
        serde_json::from_value(package["semantic_graph"]["nodes"][extension]["node_id"].clone())
            .expect("extension id");
    let later = if record_id > extension_id {
        record
    } else {
        extension
    };
    assert_extension_refusal(
        &package,
        Code::AmbiguousDeclaration,
        Some(Cause::AmbiguousName),
        format!("/semantic_graph/nodes/{later}/declaration/qualified_name"),
    );
}

fn quota_package() -> (Value, usize, usize) {
    let (mut package, retention) = extension_fixture();
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    let bounded = nodes
        .iter()
        .position(|node| {
            node["node_tag"] == "bounded_domain" && node["semantic_form"] == "integer_range"
        })
        .expect("bounded type");
    let days = nodes
        .iter()
        .position(|node| node["node_tag"] == "value" && node["body"]["value"] == "30")
        .expect("days literal");

    let quota_type = append_structural_node(&mut package, bounded, 'a', 200);
    package["semantic_graph"]["nodes"][quota_type]["body"]["members"][1]["value"]["value"] =
        json!("100");
    refresh_extension(&mut package, quota_type);
    let quota_type_id = package["semantic_graph"]["nodes"][quota_type]["node_id"].clone();

    let limit = append_structural_node(&mut package, days, 'c', 220);
    package["semantic_graph"]["nodes"][limit]["semantic_type"] = quota_type_id;
    package["semantic_graph"]["nodes"][limit]["body"]["value"] = json!("5");
    refresh_extension(&mut package, limit);
    let limit_id = package["semantic_graph"]["nodes"][limit]["node_id"].clone();

    let quota = append_structural_node(&mut package, retention, 'd', 240);
    let node = &mut package["semantic_graph"]["nodes"][quota];
    node["declaration"]["qualified_name"] = json!(["Q"]);
    node["body"]["definition"]["identity"] = json!("quota");
    node["body"]["node_kind"] = json!("quota");
    node["body"]["clauses"] = json!([{"term":"binding", "name":"limit", "value":{"term":"reference", "target": limit_id.clone()}}]);
    node["dependencies"] = json!([limit_id]);
    refresh_extension(&mut package, quota);

    let selection = json!({"authority":"example", "identity":"quota"});
    package["lock"]["definition_selections"]
        .as_array_mut()
        .expect("definitions")
        .insert(0, selection);
    (package, retention, quota)
}

#[trace("TC-459", "FR-348-AC-7")]
#[test]
fn tc_459_second_extension_kind_uses_the_same_tag_and_form() {
    let (package, _, quota) = quota_package();
    assert_eq!(
        package["semantic_graph"]["nodes"][quota]["node_tag"],
        "extension"
    );
    assert_eq!(
        package["semantic_graph"]["nodes"][quota]["semantic_form"],
        "extension_form"
    );
    let read = read_package(&package).expect("production read");
    assert!(
        matches!(read, CheckedPackageV2ReadResult::Admitted(_)),
        "{read:?}"
    );
}

#[trace("TC-459", "FR-348-AC-5")]
#[test]
fn tc_459_extension_kind_clause_requires_the_named_kind() {
    let (mut package, retention, quota) = quota_package();
    let quota_id = package["semantic_graph"]["nodes"][quota]["node_id"].clone();
    let policy = append_structural_node(&mut package, retention, 'e', 260);
    let node = &mut package["semantic_graph"]["nodes"][policy];
    node["declaration"]["qualified_name"] = json!(["P"]);
    node["body"]["definition"]["identity"] = json!("policy");
    node["body"]["node_kind"] = json!("policy");
    node["body"]["clauses"] = json!([{"term":"binding", "name":"retain", "value":{"term":"reference", "target":quota_id.clone()}}]);
    node["dependencies"] = json!([quota_id]);
    refresh_extension(&mut package, policy);
    let selection = json!({"authority":"example", "identity":"policy"});
    package["lock"]["definition_selections"]
        .as_array_mut()
        .expect("definitions")
        .insert(1, selection);
    let mismatch = assert_extension_refusal(
        &package,
        Code::IllTyped,
        Some(Cause::TypeMismatch),
        format!("/semantic_graph/nodes/{policy}/body/clauses/0/value"),
    );
    let detail = mismatch.extension_detail.expect("extension type detail");
    assert_eq!(detail.member.as_ref(), "retain");
    assert_eq!(detail.expected_type.as_deref(), Some("retention"));
    assert_eq!(detail.actual_type.as_deref(), Some("quota"));
    assert_eq!(
        serde_json::to_value(detail.actual_semantic_type.as_ref().expect("semantic type"))
            .expect("node id"),
        package["semantic_graph"]["nodes"][quota]["semantic_type"]
    );

    let retention_id = package["semantic_graph"]["nodes"][retention]["node_id"].clone();
    package["semantic_graph"]["nodes"][policy]["body"]["clauses"][0]["value"]["target"] =
        retention_id.clone();
    package["semantic_graph"]["nodes"][policy]["dependencies"] = json!([retention_id]);
    refresh_extension(&mut package, policy);
    let read = read_package(&package).expect("production read");
    assert!(
        matches!(read, CheckedPackageV2ReadResult::Admitted(_)),
        "{read:?}"
    );
}

#[trace("TC-459", "FR-348-AC-7")]
#[test]
fn tc_459_extension_refusal_order_follows_graph_stages_then_structural_id() {
    let (mut package, retention, quota) = quota_package();
    for position in [retention, quota] {
        package["semantic_graph"]["nodes"][position]["body"]["node_kind"] = json!("unknown");
        refresh_extension(&mut package, position);
    }
    let retention_id: CheckedNodeId =
        serde_json::from_value(package["semantic_graph"]["nodes"][retention]["node_id"].clone())
            .expect("retention id");
    let quota_id: CheckedNodeId =
        serde_json::from_value(package["semantic_graph"]["nodes"][quota]["node_id"].clone())
            .expect("quota id");
    let first = if retention_id < quota_id {
        retention
    } else {
        quota
    };
    let refusal = assert_extension_refusal(
        &package,
        Code::MissingDeclaration,
        Some(Cause::MissingName),
        format!("/semantic_graph/nodes/{first}/body/node_kind"),
    );
    assert_eq!(
        refusal.locus.as_ref().expect("locus"),
        if first == retention {
            &retention_id
        } else {
            &quota_id
        }
    );

    let mut malformed = package.clone();
    malformed["semantic_graph"]["nodes"][quota]["body"]["clauses"][0]["value"]["term"] =
        json!("literal");
    assert_extension_refusal(
        &malformed,
        Code::MalformedWire,
        None,
        format!("/semantic_graph/nodes/{quota}/body"),
    );

    let mut bad_graph = package.clone();
    bad_graph["semantic_graph"]["nodes"][quota]["dependencies"] = json!([]);
    assert_extension_refusal(
        &bad_graph,
        Code::InvalidSemanticGraph,
        None,
        format!("/semantic_graph/nodes/{quota}/dependencies"),
    );

    let mut duplicate = package;
    duplicate["semantic_graph"]["nodes"][quota]["node_id"] =
        duplicate["semantic_graph"]["nodes"][retention]["node_id"].clone();
    assert_extension_refusal(
        &duplicate,
        Code::InvalidSemanticGraph,
        None,
        format!("/semantic_graph/nodes/{quota}/node_id"),
    );
}

#[trace("TC-459", "FR-348-AC-7")]
#[test]
fn tc_459_extension_join_precedes_frame_semantic_join() {
    let mut package =
        read_json(&fixtures_dir().join("positive-all-families.json")).expect("published package");
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    let frame = nodes
        .iter()
        .position(|node| node["node_tag"] == "state" && node["semantic_form"] == "frame")
        .expect("frame");
    let extension = nodes
        .iter()
        .position(|node| node["node_tag"] == "extension")
        .expect("extension");
    let scalar = nodes
        .iter()
        .find(|node| node["node_tag"] == "scalar_type" && node["semantic_form"] == "integer")
        .expect("integer type")["node_id"]
        .clone();
    package["semantic_graph"]["nodes"][frame]["body"]["creates"][0] = scalar.clone();
    let node = &package["semantic_graph"]["nodes"][frame];
    let mut dependencies = vec![
        scalar,
        node["body"]["deletes"][0].clone(),
        node["body"]["modifies"][0]["declaration"].clone(),
        node["body"]["modifies"][1]["declaration"].clone(),
    ];
    sort_node_ids(&mut dependencies);
    dependencies.dedup();
    package["semantic_graph"]["nodes"][frame]["dependencies"] = json!(dependencies);
    refresh_extension(&mut package, frame);
    let held = read_package(&package).expect("held read");
    assert!(
        matches!(held, CheckedPackageV2ReadResult::Refused(_)),
        "frame mutation must refuse: {held:?}"
    );
    let document = proposal_dir_of(&fixtures_dir()).join(DOMAIN_DOCUMENT);
    let unheld = read_package_with_document_and_extensions(&package, "unheld", &document, false)
        .expect("unheld read");
    let CheckedPackageV2ReadResult::Refused(refusal) = unheld else {
        panic!("unheld extension must refuse: {unheld:?}");
    };
    assert_eq!(refusal.code, Code::UnknownRequiredCapability);
    assert_eq!(
        refusal.loci[0].path.as_ref().expect("pointer").to_string(),
        format!("/semantic_graph/nodes/{extension}/body/definition")
    );
}
