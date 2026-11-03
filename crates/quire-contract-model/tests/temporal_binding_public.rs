// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Default-feature external consumer of the admitted temporal binding read.

use ix_trace_rs::trace;
use quire_contract_model::{
    encode_checked_package_v2, CheckedArtifactRef, CheckedExtensionDefinition, CheckedNodeId,
    CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageSidecarsRef,
    CheckedPackageSourceCorrespondenceRef, CheckedPackageV2, CheckedPackageV2ReadResult,
    ClockConfiguration, TemporalBinding, TemporalClauseName,
};
use serde_json::Value;
use std::{fs, path::PathBuf};

fn borrowed_rows(package: &CheckedPackageV2) -> Option<&[TemporalBinding]> {
    package.temporal_bindings()
}

fn resolve_clause(value: &mut Value, clause: &Value) {
    match value {
        Value::String(text) if text == "<clause>" => *value = clause.clone(),
        Value::Array(items) => {
            for item in items {
                resolve_clause(item, clause);
            }
        }
        Value::Object(members) => {
            for member in members.values_mut() {
                resolve_clause(member, clause);
            }
        }
        _ => {}
    }
}

/// Trace: FR-019-AC-22, FR-019-AC-25, TC-058.
#[trace("FR-019-AC-22", "FR-019-AC-25", "TC-058")]
#[test]
fn default_feature_consumer_borrows_admitted_clock_rows() {
    let root = match std::env::var_os("QUIRE_SPECIFICATION_DIR") {
        Some(root) => root,
        None if std::env::var_os("QSPEC_CONFORMANCE").is_none() => return,
        None => panic!("QUIRE_SPECIFICATION_DIR required for temporal binding conformance"),
    };
    let proposal = PathBuf::from(root).join("proposals/checked-package-v2");
    let fixture_name = "fixtures/positive-timed-clause-nodes.json";
    let mut fixture: Value = serde_json::from_slice(
        &fs::read(proposal.join(fixture_name)).expect("published checked package"),
    )
    .expect("checked package JSON");
    let vectors: Value = serde_json::from_slice(
        &fs::read(proposal.join("temporal-clause-name-vectors.json"))
            .expect("published name vectors"),
    )
    .expect("name vectors JSON");
    let case = &vectors["positive"][1];
    assert_eq!(case["id"], "CN-P2-shared-node-two-names");
    let clause_index = vectors["clause_graph_index"]
        .as_u64()
        .expect("clause index") as usize;
    let clause = fixture["semantic_graph"]["nodes"][clause_index]["node_id"].clone();
    for patch in case["patch"].as_array().expect("twin patches") {
        assert_eq!(patch["op"], "add");
        let path = patch["path"].as_str().expect("patch path");
        let (parent, index) = path.rsplit_once('/').expect("array member");
        let mut value = patch["value"].clone();
        resolve_clause(&mut value, &clause);
        fixture
            .pointer_mut(parent)
            .expect("published patch parent")
            .as_array_mut()
            .expect("published array")
            .insert(index.parse().expect("array index"), value);
    }
    let selections: Value = serde_json::from_slice(
        &fs::read(proposal.join("selected-documents.json")).expect("published document map"),
    )
    .expect("document map JSON");
    let document_name = selections["selected_documents"][fixture_name]
        .as_str()
        .expect("selected document");
    let document = fs::read(proposal.join(document_name)).expect("selected domain document");
    let mut evidence = CheckedPackageEvidence::new();
    evidence
        .insert_domain_package_document("acme/orders", document)
        .expect("domain evidence");
    let extensions: Value = serde_json::from_slice(
        &fs::read(proposal.join("extension-definitions.json"))
            .expect("published extension definitions"),
    )
    .expect("extension definitions JSON");
    for selection in fixture["lock"]["definition_selections"]
        .as_array()
        .expect("definition selections")
    {
        let published = extensions["definitions"]
            .as_array()
            .expect("published definitions")
            .iter()
            .find(|entry| entry["reference"] == *selection)
            .expect("selected published definition");
        let reference: CheckedArtifactRef =
            serde_json::from_value(selection.clone()).expect("selected reference");
        let definition: CheckedExtensionDefinition =
            serde_json::from_value(published["definition"].clone()).expect("typed definition");
        evidence
            .insert_extension_definition(reference, definition)
            .expect("extension evidence");
    }
    for feature in fixture["lock"]["required_features"]
        .as_array()
        .expect("required features")
    {
        evidence.support_feature(feature.as_str().expect("feature"));
    }
    let bytes = serde_json::to_vec(&fixture).expect("canonical test package");
    let package =
        match CheckedPackageV2::read(&bytes, CheckedPackageReadLimits::bounded(), &evidence) {
            CheckedPackageV2ReadResult::Admitted(package) => package,
            other => panic!("published temporal package must admit: {other:?}"),
        };
    let rows = borrowed_rows(&package).expect("clock rows");
    assert_eq!(rows.len(), 2);
    let expected: CheckedNodeId =
        serde_json::from_value(fixture["temporal_bindings"][0]["declaration"].clone())
            .expect("clause structural ID");
    let declaration: &CheckedNodeId = &rows[0].declaration;
    assert_eq!(declaration, &expected);
    assert_eq!(rows[1].declaration, expected);
    assert_eq!((rows[0].occurrence, rows[1].occurrence), (0, 1));
    assert_eq!(rows[0].clock_name.as_ref(), "model-steps");
    assert!(matches!(
        rows[0].clock,
        ClockConfiguration::EventPosition { ref sequence_authority }
            if sequence_authority.as_ref() == "order-events"
    ));
    let names = package
        .temporal_clause_names()
        .expect("admitted clause names");
    assert_eq!(names.len(), 2);
    assert_eq!(names[0].declaration, expected);
    assert_eq!(names[1].declaration, expected);
    assert_eq!((names[0].occurrence, names[1].occurrence), (0, 1));
    assert_eq!(
        (names[0].name.as_ref(), names[1].name.as_ref()),
        ("Clause0", "Clause1")
    );

    let supplied = names
        .iter()
        .map(|row| TemporalClauseName {
            declaration: row.declaration.clone(),
            occurrence: row.occurrence,
            name: row.name.clone(),
        })
        .collect::<Vec<_>>();
    let encoded = encode_checked_package_v2(
        package.name(),
        package.lock(),
        package.graph(),
        CheckedPackageSourceCorrespondenceRef {
            source_map: package.source_map(),
            refinement_step_rows: package.refinement_step_rows(),
        },
        CheckedPackageSidecarsRef {
            checked_term_data: package.checked_term_data(),
            temporal_bindings: package.temporal_bindings(),
            temporal_clause_names: Some(&supplied),
            schedulability_claims: None,
            operation_effects: None,
        },
        package.capability_report(),
        package.diagnostics(),
        quire_canonical::Limits::new(1 << 24),
    )
    .expect("typed external writer");
    let CheckedPackageV2ReadResult::Admitted(reread) =
        CheckedPackageV2::read(&encoded, CheckedPackageReadLimits::bounded(), &evidence)
    else {
        panic!("typed external writer must read back");
    };
    assert_eq!(reread.temporal_clause_names(), Some(supplied.as_slice()));
}
