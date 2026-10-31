// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Published model field read-form vectors through strict package admission.

use super::{
    fixtures_dir, positive_document, proposal_dir_of, read_json, read_package_with_document,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    encode_checked_package_v2, CheckedFieldReadForm, CheckedOperationMember,
    CheckedPackageRefusalCode, CheckedPackageSourceCorrespondenceRef, CheckedPackageV2,
    CheckedPackageV2ReadResult, CheckedTerm,
};
use serde_json::Value;

fn fill_model_read_forms(value: &mut Value) {
    match value {
        Value::Object(map) => {
            if map.get("identity").and_then(Value::as_str) == Some("quire.op.record.project") {
                if let Some(member) = map.get_mut("member").and_then(Value::as_object_mut) {
                    let model_object = member
                        .get("declaration")
                        .and_then(|id| id.get("path"))
                        .and_then(Value::as_array)
                        .is_some_and(|path| {
                            path.first().and_then(Value::as_str) == Some("model")
                                && path.get(1).and_then(Value::as_str) == Some("object_type")
                        });
                    if model_object {
                        member.insert("read_form".into(), Value::String("bare".into()));
                    }
                }
            }
            for child in map.values_mut() {
                fill_model_read_forms(child);
            }
        }
        Value::Array(items) => {
            for item in items {
                fill_model_read_forms(item);
            }
        }
        _ => {}
    }
}

fn first_projection_read_form(package: &CheckedPackageV2) -> Option<CheckedFieldReadForm> {
    package.graph().nodes.iter().find_map(|node| {
        let CheckedTerm::Application { operation, .. } = &node.body else {
            return None;
        };
        if operation.identity() != Some("quire.op.record.project") {
            return None;
        }
        let read = operation.read().ok()?;
        match read.member? {
            CheckedOperationMember::Field { read_form, .. } => *read_form,
            _ => None,
        }
    })
}

/// Trace: FR-349-AC-8, QSpec FR-322-AC-34.
#[trace("FR-349-AC-8", "FR-322-AC-34")]
#[test]
fn published_model_field_read_forms_survive_admission_and_round_trip() {
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let selections = super::vector_selected_documents(&proposal).expect("document selections");
    let document = proposal.join(positive_document(
        "positive-state-transition.json",
        &selections,
    ));
    let mut base = read_json(&fixtures.join("positive-state-transition.json"))
        .expect("published model projection package");
    fill_model_read_forms(&mut base);
    let vectors = read_json(&proposal.join("model-member-type-vectors.json"))
        .expect("published model member vectors");
    let cases = vectors["projection_cases"]
        .as_array()
        .expect("projection cases");
    for name in ["RP-09", "RP-10", "RP-11", "RP-12", "RP-13"] {
        let row = cases
            .iter()
            .find(|row| row["name"] == name)
            .expect("published row");
        let mut wire = base.clone();
        let nodes = wire["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("graph nodes");
        let projection = nodes
            .iter_mut()
            .find(|node| {
                node["body"]["operation"]["identity"] == "quire.op.record.project"
                    && node["body"]["operation"]["member"]["declaration"]["path"][1]
                        == "object_type"
            })
            .expect("published model projection");
        if name == "RP-13" {
            let declaration = row["member"]["declaration"].clone();
            // The vector's undereferenced operand names the declaring object.
            // The positive package's field is the same object, so its existing
            // selected graph node supplies the concrete structural identity.
            let declaration = if declaration.is_string() {
                projection["body"]["operation"]["member"]["declaration"].clone()
            } else {
                declaration
            };
            projection["body"]["arguments"][0]["target"] = declaration.clone();
            projection["dependencies"] = Value::Array(vec![declaration]);
        }
        let member = projection["body"]["operation"]["member"]
            .as_object_mut()
            .expect("field member");
        match row["member"].get("read_form") {
            Some(form) => {
                member.insert("read_form".into(), form.clone());
            }
            None => {
                member.remove("read_form");
            }
        }
        let result = read_package_with_document(&wire, name, &document)
            .expect("read published vector mutation");
        match row["expected"]["refused"]["code"].as_str() {
            Some("malformed_wire") => assert!(
                matches!(
                    result,
                    CheckedPackageV2ReadResult::Refused(refusal)
                        if refusal.code == CheckedPackageRefusalCode::MalformedWire
                ),
                "{name}: expected malformed_wire"
            ),
            None => {
                let CheckedPackageV2ReadResult::Admitted(package) = result else {
                    panic!("{name}: expected admission, got {result:?}");
                };
                let expected = match row["expected"]["read_form"].as_str() {
                    Some("bare") => CheckedFieldReadForm::Bare,
                    Some("explicit_deref") => CheckedFieldReadForm::ExplicitDeref,
                    other => panic!("{name}: unknown published read form {other:?}"),
                };
                assert_eq!(
                    first_projection_read_form(&package),
                    Some(expected),
                    "{name}: typed read form"
                );
                let encoded = encode_checked_package_v2(
                    package.name(),
                    package.lock(),
                    package.graph(),
                    CheckedPackageSourceCorrespondenceRef {
                        source_map: package.source_map(),
                        refinement_step_rows: package.refinement_step_rows(),
                    },
                    package.capability_report(),
                    package.diagnostics(),
                    quire_canonical::Limits::new(u64::MAX),
                )
                .expect("encode admitted package");
                let encoded: Value = serde_json::from_slice(&encoded).expect("encoded JSON");
                assert_eq!(
                    encoded["semantic_graph"]["nodes"], wire["semantic_graph"]["nodes"],
                    "{name}: graph round trip"
                );
                let CheckedPackageV2ReadResult::Admitted(read_back) =
                    read_package_with_document(&encoded, name, &document)
                        .expect("read encoded package")
                else {
                    panic!("{name}: encoded package must admit");
                };
                assert_eq!(
                    first_projection_read_form(&read_back),
                    Some(expected),
                    "{name}: read-back form"
                );
            }
            other => panic!("{name}: unsupported published outcome {other:?}"),
        }
    }
}

/// Trace: FR-349-AC-8, QSpec FR-322-AC-34.
#[trace("FR-349-AC-8", "FR-322-AC-34")]
#[test]
fn published_record_projection_rejects_inapplicable_read_form() {
    let fixtures = fixtures_dir();
    let proposal = proposal_dir_of(&fixtures);
    let selections = super::vector_selected_documents(&proposal).expect("document selections");
    let document = proposal.join(positive_document(
        "positive-operation-identities.json",
        &selections,
    ));
    let vectors = read_json(&proposal.join("model-member-type-vectors.json"))
        .expect("published model member vectors");
    let row = vectors["projection_cases"]
        .as_array()
        .expect("projection cases")
        .iter()
        .find(|row| row["name"] == "RP-14")
        .expect("RP-14");
    let mut wire = read_json(&fixtures.join("positive-operation-identities.json"))
        .expect("published record projection package");
    let projection = wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("graph nodes")
        .iter_mut()
        .find(|node| node["body"]["operation"]["identity"] == "quire.op.record.project")
        .expect("record projection");
    projection["body"]["operation"]["member"]["read_form"] = row["member"]["read_form"].clone();
    let result = read_package_with_document(&wire, "RP-14", &document)
        .expect("read published record mutation");
    assert_eq!(row["expected"]["refused"]["code"], "malformed_wire");
    assert!(
        matches!(result, CheckedPackageV2ReadResult::Refused(refusal)
        if refusal.code == CheckedPackageRefusalCode::MalformedWire)
    );
}
