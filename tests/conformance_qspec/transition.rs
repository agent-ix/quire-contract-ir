// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-064: the published QSpec transition is read through the public typed carrier.

use super::*;
use quire_contract_model::{encode_checked_package_v2, ApplicationOperator, CheckedTerm};
use std::collections::BTreeMap;

fn published() -> Value {
    read_json(&fixtures_dir().join("positive-state-transition.json"))
        .expect("QSpec state-transition fixture")
}

fn transition_position(package: &Value) -> usize {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("published nodes")
        .iter()
        .position(|node| node["node_tag"] == "state" && node["semantic_form"] == "transition")
        .expect("published transition")
}

fn write(read: &CheckedPackageV2) -> Value {
    let selection = read.identity_preimage();
    let preimage = CheckedPackageIdentityPreimageRefV2 {
        version: &selection.version,
        edition: &selection.edition,
        profile_selections: &selection.profile_selections,
        definition_selections: &selection.definition_selections,
        model_selections: &selection.model_selections,
        required_features: &selection.required_features,
        dependency_selections: &selection.dependency_selections,
        identity_projection: &read.graph().nodes,
    };
    let bytes = encode_checked_package_v2(
        &preimage,
        read.package_id(),
        read.lock(),
        read.graph(),
        quire_contract_model::CheckedPackageSourceCorrespondenceRef {
            source_map: read.source_map(),
            refinement_step_rows: read.refinement_step_rows(),
        },
        read.capability_report(),
        read.diagnostics(),
        Limits::new(1 << 24),
    )
    .expect("typed writer");
    serde_json::from_slice(&bytes).expect("canonical writer JSON")
}

fn target(package: &Value, tag: &str, form: &str) -> Value {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == tag && node["semantic_form"] == form)
        .unwrap_or_else(|| panic!("published {tag}/{form}"))["node_id"]
        .clone()
}

fn parameter(package: &Value, name: &str) -> Value {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| {
            node["semantic_form"] == "parameter"
                && node["body"]["members"][0]["value"]["value"] == name
        })
        .unwrap_or_else(|| panic!("published parameter {name}"))["node_id"]
        .clone()
}

fn collect_references(term: &Value, references: &mut BTreeMap<String, Value>) {
    match term["term"].as_str() {
        Some("reference") => {
            let target = &term["target"];
            references.insert(
                target["digest"].as_str().expect("reference digest").into(),
                target.clone(),
            );
        }
        Some("binding") => collect_references(&term["value"], references),
        Some("aggregate") => {
            for member in term["members"].as_array().expect("members") {
                collect_references(member, references);
            }
        }
        Some("application") => {
            for argument in term["arguments"].as_array().expect("arguments") {
                collect_references(argument, references);
            }
        }
        other => panic!("transition term family {other:?}"),
    }
}

fn refresh_transition(package: &mut Value, position: usize) {
    let mut references = BTreeMap::new();
    collect_references(
        &package["semantic_graph"]["nodes"][position]["body"],
        &mut references,
    );
    package["semantic_graph"]["nodes"][position]["dependencies"] =
        Value::Array(references.into_values().collect());
    fixture_support::rekey_application_node(package, position);
    fixture_support::refresh_identity_projection(package);
}

fn rekey_application_with_references(package: &mut Value, position: usize) {
    let mut references = BTreeMap::new();
    collect_references(
        &package["semantic_graph"]["nodes"][position]["body"],
        &mut references,
    );
    package["semantic_graph"]["nodes"][position]["dependencies"] =
        Value::Array(references.into_values().collect());
    fixture_support::rekey_application_node(package, position);
}

fn select_model_document(package: &mut Value, document: &Value) -> String {
    let digest = quire_canonical::sha256(document, Limits::new(DERIVATION_CEILING))
        .expect("selected document digest")
        .to_string();
    package["lock"]["model_selections"][0]["digest"] = json!(digest);
    package["identity_preimage"]["model_selections"][0]["digest"] = json!(digest);
    fixture_support::refresh_identity(package);
    digest
}

fn read_with_selected_model_document(
    package: &Value,
    document: &Value,
    digest: &str,
) -> CheckedPackageV2ReadResult {
    let mut evidence = CheckedPackageEvidence::new();
    for feature in package["lock"]["required_features"]
        .as_array()
        .expect("required features")
    {
        evidence.support_feature(feature.as_str().expect("feature"));
    }
    evidence
        .insert_domain_package_document(
            digest,
            serde_json::to_vec(document).expect("selected document bytes"),
        )
        .expect("selected model evidence");
    CheckedPackageV2::read(
        &serde_json::to_vec(package).expect("checked package bytes"),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    )
}

/// The QSpec-owned half of AC-7 reads the selected checkout; its QSL-authored
/// source/claim half remains separate evidence.
#[trace("TC-064", "FR-067-AC-1", "FR-067-AC-6", "FR-067-AC-7")]
#[test]
fn published_boolean_transition_retains_five_typed_bindings_through_writer() {
    let fixture = published();
    let CheckedPackageV2ReadResult::Admitted(read) =
        read_package(&fixture).expect("selected QSpec evidence")
    else {
        panic!("published Boolean transition must admit");
    };
    let transition = read
        .graph()
        .nodes
        .iter()
        .find(|node| {
            node.node_tag.as_ref() == "state" && node.semantic_form.as_ref() == "transition"
        })
        .expect("published transition node");
    let CheckedTerm::Application {
        operator: ApplicationOperator::StateTransition,
        arguments,
        result_type,
        ..
    } = &transition.body
    else {
        panic!("public transition is a typed application");
    };
    assert_eq!(transition.semantic_type, *result_type);
    let names: Vec<&str> = arguments
        .iter()
        .map(|term| match term {
            CheckedTerm::Binding { name, .. } => name.as_ref(),
            _ => panic!("transition argument is a typed binding"),
        })
        .collect();
    assert_eq!(names, ["anchor", "subject", "inputs", "pre", "post"]);
    assert_eq!(transition.occurrences.len(), 1);
    assert_eq!(
        transition.occurrences[0].role,
        quire_contract_model::CheckedOccurrenceRole::Claim
    );

    let written = write(&read);
    assert_eq!(written, fixture);
    let CheckedPackageV2ReadResult::Admitted(second) =
        read_package(&written).expect("selected QSpec evidence")
    else {
        panic!("written transition must read again");
    };
    assert_eq!(second.graph(), read.graph());
    assert_eq!(second.package_id(), read.package_id());
}

#[trace("TC-064", "FR-067-AC-1", "FR-067-AC-6")]
#[test]
fn pointed_clause_anchors_and_a_distinct_unpointed_transition_anchor_round_trip() {
    let mut package = published();
    let transition = transition_position(&package);
    let original_anchor = package["semantic_graph"]["nodes"][11].clone();
    let original_source = package["source_map"]
        .as_array()
        .expect("source map")
        .iter()
        .find(|row| row["node_id"] == original_anchor["node_id"])
        .expect("anchor source")
        .clone();
    for (clause, kind, start) in [(12, "pre", 180), (13, "post", 200)] {
        let point = json!({"kind":kind,"operation":"set"});
        let mut anchor = original_anchor.clone();
        anchor["body"]["members"]
            .as_array_mut()
            .expect("anchor members")
            .push(json!({"term":"binding","name":"execution_point","value":point}));
        let context = anchor["semantic_type"]["digest"]
            .as_str()
            .expect("anchor context");
        anchor["node_id"]["digest"] = json!(fixture_support::structural_key(
            "state",
            "operation_anchor",
            Some(context),
            &anchor["body"]
        ));
        let mut source = original_source.clone();
        source["node_id"] = anchor["node_id"].clone();
        source["regions"][0]["start"] = json!(start);
        source["regions"][0]["end"] = json!(start + 10);
        package["semantic_graph"]["nodes"][clause]["body"]["arguments"][1]["target"] =
            anchor["node_id"].clone();
        package["semantic_graph"]["nodes"][clause]["body"]["operation"]["member"]
            ["execution_point"] = point;
        let mut references = BTreeMap::new();
        collect_references(
            &package["semantic_graph"]["nodes"][clause]["body"],
            &mut references,
        );
        package["semantic_graph"]["nodes"][clause]["dependencies"] =
            Value::Array(references.into_values().collect());
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(anchor);
        package["source_map"]
            .as_array_mut()
            .expect("source map")
            .push(source);
        fixture_support::rekey_application_node(&mut package, clause);
    }
    refresh_transition(&mut package, transition);
    let CheckedPackageV2ReadResult::Admitted(read) =
        read_package(&package).expect("selected QSpec evidence")
    else {
        panic!("pointed clauses with separate unpointed transition anchor must admit");
    };
    assert_eq!(write(&read), package);
    assert!(matches!(
        read_package(&write(&read)).expect("selected QSpec evidence"),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

#[trace("TC-064", "FR-067-AC-5", "FR-067-AC-6")]
#[test]
fn two_source_claims_share_one_transition_identity_and_retain_regions() {
    let mut package = published();
    let original_id = package["package_id"].clone();
    let transition = transition_position(&package);
    let node_id = package["semantic_graph"]["nodes"][transition]["node_id"].clone();
    package["semantic_graph"]["nodes"][transition]["occurrences"]
        .as_array_mut()
        .expect("occurrences")
        .push(json!({"ordinal": 1, "role": "claim"}));
    let entry = package["source_map"]
        .as_array()
        .expect("source map")
        .iter()
        .position(|row| row["node_id"] == node_id && row["ordinal"] == 0)
        .expect("transition source row");
    let mut second = package["source_map"][entry].clone();
    second["ordinal"] = json!(1);
    second["regions"][0]["start"] = json!(100);
    second["regions"][0]["end"] = json!(110);
    package["source_map"]
        .as_array_mut()
        .expect("source map")
        .insert(entry + 1, second);
    assert_eq!(
        package["package_id"], original_id,
        "occurrences are outside identity"
    );
    let CheckedPackageV2ReadResult::Admitted(read) =
        read_package(&package).expect("selected QSpec evidence")
    else {
        panic!("two checked source declarations must admit");
    };
    let transition = read
        .graph()
        .nodes
        .iter()
        .find(|node| node.node_id.digest.as_ref() == node_id["digest"].as_str().expect("digest"))
        .expect("shared typed transition");
    assert_eq!(transition.occurrences.len(), 2);
    assert_eq!(transition.occurrences[0].ordinal, 0);
    assert_eq!(transition.occurrences[1].ordinal, 1);
    assert_eq!(
        read.package_id().digest.as_ref(),
        original_id["digest"].as_str().expect("digest")
    );
    assert_eq!(
        read.source_map()
            .iter()
            .filter(|row| row.node_id == transition.node_id)
            .count(),
        2
    );
    let written = write(&read);
    assert_eq!(written, package);
    let CheckedPackageV2ReadResult::Admitted(second_read) =
        read_package(&written).expect("selected QSpec evidence")
    else {
        panic!("two written source claims must read again");
    };
    assert_eq!(second_read.graph(), read.graph());
    assert_eq!(second_read.source_map(), read.source_map());
    assert_eq!(second_read.package_id(), read.package_id());
}

#[trace("TC-064", "FR-067-AC-3")]
#[test]
fn published_transition_join_mutations_refuse_at_the_first_target() {
    for case in [
        "wrong_anchor",
        "wrong_subject",
        "self_input",
        "missing_input",
        "wrong_pre",
        "wrong_post",
    ] {
        let mut package = published();
        let position = transition_position(&package);
        let model = target(&package, "model", "object_type");
        let self_parameter = parameter(&package, "self");
        let pre = package["semantic_graph"]["nodes"][position]["body"]["arguments"][3]["value"]
            ["members"][0]["target"]
            .clone();
        let post = package["semantic_graph"]["nodes"][position]["body"]["arguments"][4]["value"]
            ["target"]
            .clone();
        let (code, cause, path) = match case {
            "wrong_anchor" => {
                package["semantic_graph"]["nodes"][position]["body"]["arguments"][0]["value"]
                    ["target"] = model;
                (
                    Code::InvalidModelBinding,
                    Cause::MalformedDeclaration,
                    format!("/semantic_graph/nodes/{position}/body/arguments/0/value/target"),
                )
            }
            "wrong_subject" => {
                package["semantic_graph"]["nodes"][position]["body"]["arguments"][1]["value"]
                    ["target"] = self_parameter;
                (
                    Code::InvalidModelBinding,
                    Cause::MalformedDeclaration,
                    format!("/semantic_graph/nodes/{position}/body/arguments/1/value/target"),
                )
            }
            "self_input" => {
                package["semantic_graph"]["nodes"][position]["body"]["arguments"][2]["value"]
                    ["members"][0]["target"] = self_parameter;
                (
                    Code::InvalidModelBinding,
                    Cause::MalformedDeclaration,
                    format!(
                        "/semantic_graph/nodes/{position}/body/arguments/2/value/members/0/target"
                    ),
                )
            }
            "missing_input" => {
                package["semantic_graph"]["nodes"][position]["body"]["arguments"][2]["value"]
                    ["members"] = json!([]);
                (
                    Code::IllTyped,
                    Cause::OperatorIneligible,
                    format!("/semantic_graph/nodes/{position}"),
                )
            }
            "wrong_pre" => {
                package["semantic_graph"]["nodes"][position]["body"]["arguments"][3]["value"]
                    ["members"][0]["target"] = post;
                (
                    Code::InvalidModelBinding,
                    Cause::MalformedDeclaration,
                    format!(
                        "/semantic_graph/nodes/{position}/body/arguments/3/value/members/0/target"
                    ),
                )
            }
            "wrong_post" => {
                package["semantic_graph"]["nodes"][position]["body"]["arguments"][4]["value"]
                    ["target"] = pre;
                (
                    Code::InvalidModelBinding,
                    Cause::MalformedDeclaration,
                    format!("/semantic_graph/nodes/{position}/body/arguments/4/value/target"),
                )
            }
            _ => unreachable!("closed published mutation list"),
        };
        refresh_transition(&mut package, position);
        let CheckedPackageV2ReadResult::Refused(refusal) =
            read_package(&package).expect("selected QSpec evidence")
        else {
            panic!("{case} must refuse");
        };
        assert_eq!(
            (refusal.code, refusal.cause),
            (code, Some(cause)),
            "{case}: {refusal:?}"
        );
        assert_eq!(
            refusal.path.as_ref().map(ToString::to_string),
            Some(path),
            "{case}: {refusal:?}"
        );
    }
}

#[trace("TC-064", "FR-067-AC-2")]
#[test]
fn published_transition_closed_shape_mutations_refuse_before_identity() {
    for case in [
        "swapped_bindings",
        "missing_post",
        "sixth_binding",
        "two_pre",
        "subject_aggregate",
        "wrong_law_role",
        "wrong_role",
    ] {
        let mut package = published();
        let position = transition_position(&package);
        let node = &mut package["semantic_graph"]["nodes"][position];
        match case {
            "swapped_bindings" => node["body"]["arguments"]
                .as_array_mut()
                .expect("bindings")
                .swap(0, 1),
            "missing_post" => {
                node["body"]["arguments"]
                    .as_array_mut()
                    .expect("bindings")
                    .pop();
            }
            "sixth_binding" => {
                let post = node["body"]["arguments"][4].clone();
                node["body"]["arguments"]
                    .as_array_mut()
                    .expect("bindings")
                    .push(post);
            }
            "two_pre" => {
                let pre = node["body"]["arguments"][3]["value"]["members"][0].clone();
                node["body"]["arguments"][3]["value"]["members"]
                    .as_array_mut()
                    .expect("pre")
                    .push(pre);
            }
            "subject_aggregate" => {
                node["body"]["arguments"][1]["value"] = json!({"term":"aggregate","members":[]})
            }
            "wrong_law_role" => {
                node["body"]["operation"]["laws"][0]["role"] = json!("temporal_profile");
                node["body"]["operation"]["mode"] = json!({"kind":"rounding"});
                node["occurrences"][0]["role"] = json!("declaration");
            }
            "wrong_role" => node["occurrences"][0]["role"] = json!("declaration"),
            _ => unreachable!("closed QSpec shape mutation list"),
        }
        fixture_support::refresh_identity_projection(&mut package);
        let CheckedPackageV2ReadResult::Refused(refusal) =
            read_package(&package).expect("selected QSpec evidence")
        else {
            panic!("{case} must refuse");
        };
        assert_eq!(refusal.code, Code::MalformedWire, "{case}: {refusal:?}");
        assert_eq!(refusal.cause, None, "{case}: {refusal:?}");
        assert!(
            refusal.path.as_ref().is_some_and(|path| path
                .to_string()
                .starts_with(&format!("/semantic_graph/nodes/{position}"))),
            "{case}: {refusal:?}"
        );
    }
}

#[trace("TC-064", "FR-067-AC-2", "FR-067-AC-4")]
#[test]
fn transition_schema_defect_precedes_a_wrong_anchor() {
    let mut package = published();
    let position = transition_position(&package);
    let node = &mut package["semantic_graph"]["nodes"][position];
    node["body"]["arguments"]
        .as_array_mut()
        .expect("bindings")
        .swap(0, 1);
    node["body"]["arguments"][1]["value"]["target"] = target(&published(), "model", "object_type");
    fixture_support::refresh_identity_projection(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package(&package).expect("selected QSpec evidence")
    else {
        panic!("schema defect must refuse before state joins");
    };
    assert_eq!(refusal.code, Code::MalformedWire);
    assert_eq!(refusal.cause, None);
    assert_eq!(
        refusal.path.as_ref().map(ToString::to_string),
        Some(format!("/semantic_graph/nodes/{position}/body"))
    );
}

#[trace("TC-064", "FR-067-AC-5")]
#[test]
fn stale_transition_key_precedes_state_join_and_source_move_preserves_identity() {
    let mut package = published();
    let position = transition_position(&package);
    let original_node_id = package["semantic_graph"]["nodes"][position]["node_id"].clone();
    let original_package_id = package["package_id"].clone();
    let source = package["source_map"]
        .as_array_mut()
        .expect("source map")
        .iter_mut()
        .find(|row| row["node_id"] == original_node_id)
        .expect("transition source");
    source["regions"][0]["start"] = json!(125);
    source["regions"][0]["end"] = json!(135);
    assert_eq!(
        package["semantic_graph"]["nodes"][position]["node_id"],
        original_node_id
    );
    assert_eq!(package["package_id"], original_package_id);
    assert!(matches!(
        read_package(&package).expect("selected QSpec evidence"),
        CheckedPackageV2ReadResult::Admitted(_)
    ));

    package["semantic_graph"]["nodes"][position]["body"]["operation"]["laws"][0]["definition"]
        ["identity"] = json!("missing-profile");
    fixture_support::refresh_identity_projection(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package(&package).expect("selected QSpec evidence")
    else {
        panic!("stale transition key must refuse before unselected law");
    };
    assert_eq!(refusal.code, Code::InvalidPackage);
    assert_eq!(refusal.cause, Some(Cause::StaleNodeKey));
}

#[trace("TC-064", "FR-067-AC-2")]
#[test]
fn transition_member_and_profile_mutations_refuse() {
    let mut member = published();
    let position = transition_position(&member);
    member["semantic_graph"]["nodes"][position]["body"]["operation"]["member"]["operator"] =
        json!("sequence");
    refresh_transition(&mut member, position);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package(&member).expect("selected QSpec evidence")
    else {
        panic!("wrong transition operator member must refuse");
    };
    assert_eq!(refusal.code, Code::MalformedWire, "{refusal:?}");
    assert_eq!(refusal.cause, None);
    assert_eq!(
        refusal.path.as_ref().map(ToString::to_string),
        Some(format!(
            "/semantic_graph/nodes/{position}/body/operation/member/operator"
        ))
    );

    let mut control = read_json(&fixtures_dir().join("positive-clause-operations.json"))
        .expect("QSpec clause and protocol fixture");
    let control_position = control["semantic_graph"]["nodes"]
        .as_array()
        .expect("published nodes")
        .iter()
        .position(|node| {
            node["node_tag"] == "protocol"
                && node["semantic_form"] == "control"
                && node["body"]["operation"]["identity"] == "quire.op.protocol.control"
        })
        .expect("published control");
    control["semantic_graph"]["nodes"][control_position]["body"]["operation"]["member"]
        ["operator"] = json!("transition");
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package(&control).expect("selected QSpec evidence")
    else {
        panic!("protocol control cannot name the transition operator");
    };
    assert_eq!(refusal.code, Code::MalformedWire, "{refusal:?}");
    assert_eq!(refusal.cause, None);
    assert_eq!(
        refusal.path.as_ref().map(ToString::to_string),
        Some(format!(
            "/semantic_graph/nodes/{control_position}/body/operation/member/operator"
        ))
    );

    let mut profile = published();
    profile["identity_preimage"]["profile_selections"] = json!([]);
    profile["lock"]["profile_selections"] = json!([]);
    fixture_support::refresh_identity_projection(&mut profile);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package(&profile).expect("selected QSpec evidence")
    else {
        panic!("unselected protocol_profile must refuse");
    };
    assert_eq!(refusal.code, Code::InvalidPackage, "{refusal:?}");
    assert_eq!(refusal.cause, Some(Cause::OperationLawUnselected));
}

#[trace("TC-064", "FR-067-AC-5")]
#[test]
fn transition_identity_tracks_its_referenced_body_and_selected_profile() {
    let base = published();
    let position = transition_position(&base);
    let original_transition = base["semantic_graph"]["nodes"][position]["node_id"].clone();
    let original_package = base["package_id"].clone();
    for case in ["subject_field", "input", "pre", "post", "profile"] {
        let mut package = base.clone();
        match case {
            "subject_field" => {
                package["semantic_graph"]["nodes"][8]["body"]["operation"]["member"]["name"] =
                    json!("level");
                fixture_support::rekey_application_node(&mut package, 8);
            }
            "input" => {
                let self_parameter = parameter(&package, "self");
                package["semantic_graph"]["nodes"][position]["body"]["arguments"][2]["value"]
                    ["members"][0]["target"] = self_parameter;
            }
            "pre" => {
                package["semantic_graph"]["nodes"][position]["body"]["arguments"][3]["value"]
                    ["members"] = json!([]);
            }
            "post" => {
                let pre = package["semantic_graph"]["nodes"][position]["body"]["arguments"][3]
                    ["value"]["members"][0]["target"]
                    .clone();
                package["semantic_graph"]["nodes"][position]["body"]["arguments"][4]["value"]
                    ["target"] = pre;
            }
            "profile" => {
                let changed = json!({"authority":"agent-ix","identity":"quire.protocol.other/v1"});
                package["identity_preimage"]["profile_selections"][0]["definition"] =
                    changed.clone();
                package["lock"]["profile_selections"][0]["definition"] = changed.clone();
                package["semantic_graph"]["nodes"][position]["body"]["operation"]["laws"][0]
                    ["definition"] = changed;
            }
            _ => unreachable!("closed identity mutation list"),
        }
        refresh_transition(&mut package, position);
        assert_ne!(
            package["semantic_graph"]["nodes"][position]["node_id"], original_transition,
            "{case} must rekey the transition"
        );
        assert_ne!(
            package["package_id"], original_package,
            "{case} must rekey the package"
        );
    }

    let mut operation = base.clone();
    operation["semantic_graph"]["nodes"][11]["body"]["members"][1]["value"]["value"] =
        json!("reset");
    fixture_support::mint_ungrouped_structural_keys(&mut operation);
    for clause in [12, 13] {
        fixture_support::rekey_application_node(&mut operation, clause);
    }
    refresh_transition(&mut operation, position);
    assert_ne!(
        operation["semantic_graph"]["nodes"][11]["node_id"],
        base["semantic_graph"]["nodes"][11]["node_id"]
    );
    for clause in [12, 13] {
        assert_ne!(
            operation["semantic_graph"]["nodes"][clause]["node_id"],
            base["semantic_graph"]["nodes"][clause]["node_id"]
        );
    }
    assert_ne!(
        operation["semantic_graph"]["nodes"][position]["node_id"],
        original_transition
    );
    assert_ne!(operation["package_id"], original_package);
}

fn two_transitions_with_distinct_optional_preconditions() -> Value {
    let mut package = published();
    let position = transition_position(&package);
    let original = package["semantic_graph"]["nodes"][position].clone();
    let mut second = original.clone();
    second["body"]["arguments"][3]["value"]["members"] = json!([]);
    let mut references = BTreeMap::new();
    collect_references(&second["body"], &mut references);
    second["dependencies"] = Value::Array(references.into_values().collect());
    let boolean = second["semantic_type"]["digest"]
        .as_str()
        .expect("Boolean digest");
    second["node_id"]["digest"] = json!(fixture_support::application_node_key(
        "state",
        "transition",
        boolean,
        &second["body"]
    ));
    let source = package["source_map"]
        .as_array()
        .expect("source map")
        .iter()
        .find(|row| row["node_id"] == original["node_id"])
        .expect("first transition source")
        .clone();
    let mut second_source = source;
    second_source["node_id"] = second["node_id"].clone();
    second_source["regions"][0]["start"] = json!(140);
    second_source["regions"][0]["end"] = json!(150);
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(second);
    package["source_map"]
        .as_array_mut()
        .expect("source map")
        .push(second_source);
    fixture_support::refresh_identity_projection(&mut package);
    package
}

fn append_second_clause(package: &mut Value, position: usize, condition: usize) {
    let mut clause = package["semantic_graph"]["nodes"][position].clone();
    clause["body"]["arguments"][2]["target"] =
        package["semantic_graph"]["nodes"][condition]["node_id"].clone();
    let mut references = BTreeMap::new();
    collect_references(&clause["body"], &mut references);
    clause["dependencies"] = Value::Array(references.into_values().collect());
    let boolean = clause["semantic_type"]["digest"]
        .as_str()
        .expect("Boolean digest");
    clause["node_id"]["digest"] = json!(fixture_support::application_node_key(
        "state",
        "state_clause",
        boolean,
        &clause["body"]
    ));
    let mut source = package["source_map"]
        .as_array()
        .expect("source map")
        .iter()
        .find(|row| row["node_id"] == package["semantic_graph"]["nodes"][position]["node_id"])
        .expect("original clause source")
        .clone();
    source["node_id"] = clause["node_id"].clone();
    source["regions"][0]["start"] = json!(150);
    source["regions"][0]["end"] = json!(160);
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(clause);
    package["source_map"]
        .as_array_mut()
        .expect("source map")
        .push(source);
    fixture_support::refresh_identity_projection(package);
}

#[trace("TC-064", "FR-067-AC-3")]
#[test]
fn a_transition_binds_one_clause_while_its_anchor_has_several_pre_or_post_clauses() {
    for (name, original, alternative) in [("pre", 12, 8), ("post", 13, 9)] {
        let mut package = published();
        let bound = package["semantic_graph"]["nodes"][original]["node_id"].clone();
        append_second_clause(&mut package, original, alternative);
        let CheckedPackageV2ReadResult::Admitted(read) =
            read_package(&package).expect("selected QSpec evidence")
        else {
            panic!("a second {name} clause must not invalidate the transition");
        };
        assert_eq!(
            read.graph()
                .nodes
                .iter()
                .filter(|node| {
                    node.node_tag.as_ref() == "state"
                        && node.semantic_form.as_ref() == "state_clause"
                })
                .count(),
            3,
            "{name} adds a retained clause"
        );
        let transition = read
            .graph()
            .nodes
            .iter()
            .find(|node| {
                node.node_tag.as_ref() == "state" && node.semantic_form.as_ref() == "transition"
            })
            .expect("transition");
        let CheckedTerm::Application { arguments, .. } = &transition.body else {
            panic!("transition body");
        };
        let selected = serde_json::to_value(&arguments[if name == "pre" { 3 } else { 4 }])
            .expect("typed selected clause");
        let retained = if name == "pre" {
            &selected["value"]["members"][0]["target"]
        } else {
            &selected["value"]["target"]
        };
        assert_eq!(retained, &bound, "{name} binds the original clause");
    }
}

#[trace("TC-064", "FR-067-AC-3")]
#[test]
fn a_second_transition_for_the_same_anchor_and_subject_refuses() {
    let package = two_transitions_with_distinct_optional_preconditions();
    let first = transition_position(&package);
    let second = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
    let first_digest = package["semantic_graph"]["nodes"][first]["node_id"]["digest"]
        .as_str()
        .expect("first transition digest");
    let second_digest = package["semantic_graph"]["nodes"][second]["node_id"]["digest"]
        .as_str()
        .expect("second transition digest");
    let later = if first_digest < second_digest {
        second
    } else {
        first
    };
    let later_id = &package["semantic_graph"]["nodes"][later]["node_id"];
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package(&package).expect("selected QSpec evidence")
    else {
        panic!("duplicate anchor and subject must refuse");
    };
    assert_eq!(refusal.code, Code::AmbiguousDeclaration, "{refusal:?}");
    assert_eq!(refusal.cause, Some(Cause::AmbiguousName));
    assert_eq!(
        refusal.path.as_ref().map(ToString::to_string),
        Some(format!("/semantic_graph/nodes/{later}"))
    );
    assert_eq!(
        serde_json::to_value(refusal.locus.as_ref().expect("duplicate locus"))
            .expect("locus serializes"),
        *later_id
    );
}

#[trace("TC-064", "FR-067-AC-4")]
#[test]
fn two_transition_defects_refuse_in_ascending_node_digest_order() {
    let mut package = two_transitions_with_distinct_optional_preconditions();
    let model = target(&package, "model", "object_type");
    let first_position = transition_position(&package);
    let second_position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
    for position in [first_position, second_position] {
        package["semantic_graph"]["nodes"][position]["body"]["arguments"][0]["value"]["target"] =
            model.clone();
        refresh_transition(&mut package, position);
    }
    let first_digest = package["semantic_graph"]["nodes"][first_position]["node_id"]["digest"]
        .as_str()
        .expect("first digest");
    let second_digest = package["semantic_graph"]["nodes"][second_position]["node_id"]["digest"]
        .as_str()
        .expect("second digest");
    let earliest = if first_digest < second_digest {
        first_position
    } else {
        second_position
    };
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package(&package).expect("selected QSpec evidence")
    else {
        panic!("both bad transitions must refuse");
    };
    assert_eq!(refusal.code, Code::InvalidModelBinding, "{refusal:?}");
    assert_eq!(refusal.cause, Some(Cause::MalformedDeclaration));
    assert_eq!(
        refusal.path.as_ref().map(ToString::to_string),
        Some(format!(
            "/semantic_graph/nodes/{earliest}/body/arguments/0/value/target"
        ))
    );
}

#[trace("TC-064", "FR-067-AC-3")]
#[test]
fn integer_model_field_cannot_be_a_boolean_transition_subject() {
    let mut package = published();
    let position = transition_position(&package);
    let integer = target(&package, "scalar_type", "integer");
    let mut query = package["semantic_graph"]["nodes"][8].clone();
    query["semantic_type"] = integer.clone();
    query["body"]["result_type"] = integer;
    query["body"]["operation"]["member"]["name"] = json!("level");
    let integer_digest = query["semantic_type"]["digest"]
        .as_str()
        .expect("Integer digest");
    query["node_id"]["digest"] = json!(fixture_support::application_node_key(
        "expression",
        "query",
        integer_digest,
        &query["body"]
    ));
    let mut source = package["source_map"]
        .as_array()
        .expect("source map")
        .iter()
        .find(|row| row["node_id"] == package["semantic_graph"]["nodes"][8]["node_id"])
        .expect("query source")
        .clone();
    source["node_id"] = query["node_id"].clone();
    source["regions"][0]["start"] = json!(160);
    source["regions"][0]["end"] = json!(170);
    package["semantic_graph"]["nodes"][position]["body"]["arguments"][1]["value"]["target"] =
        query["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(query);
    package["source_map"]
        .as_array_mut()
        .expect("source map")
        .push(source);
    refresh_transition(&mut package, position);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package(&package).expect("selected QSpec evidence")
    else {
        panic!("Integer level cannot be a Boolean state subject");
    };
    assert_eq!(refusal.code, Code::IllTyped, "{refusal:?}");
    assert_eq!(refusal.cause, Some(Cause::OperatorIneligible));
    assert_eq!(
        refusal.path.as_ref().map(ToString::to_string),
        Some(format!("/semantic_graph/nodes/{position}"))
    );
}

#[trace("TC-064", "FR-067-AC-3")]
#[test]
fn selected_optional_boolean_field_refuses_at_the_transition() {
    let mut package = published();
    let position = transition_position(&package);
    let mut document = read_json(&proposal_dir_of(&fixtures_dir()).join(DOMAIN_DOCUMENT))
        .expect("published selected model document");
    let switch = document["types"]
        .as_array_mut()
        .expect("selected types")
        .iter_mut()
        .find(|kind| kind["identity"] == "ix://acme/orders/Switch")
        .expect("selected Switch");
    let on = switch["fields"]
        .as_array_mut()
        .expect("Switch fields")
        .iter_mut()
        .find(|field| field["identity"] == "ix://acme/orders/Switch/on")
        .expect("selected on field");
    on["presence"] = json!("optional");
    let digest = select_model_document(&mut package, &document);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_with_selected_model_document(&package, &document, &digest)
    else {
        panic!("optional Boolean model field cannot be a transition subject");
    };
    assert_eq!(refusal.code, Code::IllTyped, "{refusal:?}");
    assert_eq!(refusal.cause, Some(Cause::OperatorIneligible));
    assert_eq!(
        refusal.path.as_ref().map(ToString::to_string),
        Some(format!("/semantic_graph/nodes/{position}"))
    );
}

#[trace("TC-064", "FR-067-AC-3")]
#[test]
fn two_boolean_inputs_must_follow_the_selected_operation_order() {
    let mut package = published();
    let position = transition_position(&package);
    let mut document = read_json(&proposal_dir_of(&fixtures_dir()).join(DOMAIN_DOCUMENT))
        .expect("published selected model document");
    let switch = document["types"]
        .as_array_mut()
        .expect("selected types")
        .iter_mut()
        .find(|kind| kind["identity"] == "ix://acme/orders/Switch")
        .expect("selected Switch");
    let operation = switch["operations"]
        .as_array_mut()
        .expect("Switch operations")
        .iter_mut()
        .find(|operation| operation["name"] == "set")
        .expect("selected set operation");
    let mut other_slot = operation["params"][0].clone();
    other_slot["identity"] = json!("ix://acme/orders/Switch/set/other");
    other_slot["name"] = json!("other");
    operation["params"]
        .as_array_mut()
        .expect("operation parameters")
        .push(other_slot);

    let mut other = package["semantic_graph"]["nodes"][6].clone();
    other["node_id"]["digest"] =
        json!("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
    other["body"]["members"][0]["value"]["value"] = json!("other");
    other["body"]["members"][1]["value"]["value"] = json!("2");
    let mut source = package["source_map"]
        .as_array()
        .expect("source map")
        .iter()
        .find(|row| row["node_id"] == package["semantic_graph"]["nodes"][6]["node_id"])
        .expect("flag source")
        .clone();
    source["node_id"] = other["node_id"].clone();
    source["regions"][0]["start"] = json!(220);
    source["regions"][0]["end"] = json!(230);
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(other);
    package["source_map"]
        .as_array_mut()
        .expect("source map")
        .push(source);
    fixture_support::mint_ungrouped_structural_keys(&mut package);
    let other_id = parameter(&package, "other");
    for clause in [12, 13] {
        package["semantic_graph"]["nodes"][clause]["body"]["arguments"][0]["members"]
            .as_array_mut()
            .expect("clause parameters")
            .push(json!({"term":"reference","target":other_id}));
        let mut references = BTreeMap::new();
        collect_references(
            &package["semantic_graph"]["nodes"][clause]["body"],
            &mut references,
        );
        package["semantic_graph"]["nodes"][clause]["dependencies"] =
            Value::Array(references.into_values().collect());
        fixture_support::rekey_application_node(&mut package, clause);
    }
    package["semantic_graph"]["nodes"][position]["body"]["arguments"][2]["value"]["members"]
        .as_array_mut()
        .expect("transition inputs")
        .insert(0, json!({"term":"reference","target":other_id}));
    refresh_transition(&mut package, position);
    let digest = select_model_document(&mut package, &document);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_with_selected_model_document(&package, &document, &digest)
    else {
        panic!("reversed Boolean inputs must refuse");
    };
    assert_eq!(refusal.code, Code::IllTyped, "{refusal:?}");
    assert_eq!(refusal.cause, Some(Cause::OperatorIneligible));
    assert_eq!(
        refusal.path.as_ref().map(ToString::to_string),
        Some(format!("/semantic_graph/nodes/{position}"))
    );
}

#[trace("TC-064", "FR-067-AC-2")]
#[test]
fn transition_application_nested_in_boolean_or_refuses_at_the_inner_application() {
    let mut package = published();
    let position = transition_position(&package);
    let transition = package["semantic_graph"]["nodes"][position]["body"].clone();
    package["semantic_graph"]["nodes"][9]["body"]["arguments"][0] = transition;
    fixture_support::refresh_identity_projection(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package(&package).expect("selected QSpec evidence")
    else {
        panic!("nested transition must refuse");
    };
    assert_eq!(refusal.code, Code::MalformedWire, "{refusal:?}");
    assert_eq!(refusal.cause, None);
    assert_eq!(
        refusal.path.as_ref().map(ToString::to_string),
        Some("/semantic_graph/nodes/9/body/arguments/0".into())
    );
}

#[trace("TC-064", "FR-067-AC-4")]
#[test]
fn a_frame_member_defect_precedes_a_transition_anchor_defect() {
    let mut package = published();
    let position = transition_position(&package);
    package["semantic_graph"]["nodes"][10]["body"]["modifies"][0]["name"] = json!("not_a_field");
    fixture_support::mint_ungrouped_structural_keys(&mut package);
    for clause in [12, 13] {
        rekey_application_with_references(&mut package, clause);
    }
    let model = target(&package, "model", "object_type");
    package["semantic_graph"]["nodes"][position]["body"]["arguments"][0]["value"]["target"] = model;
    refresh_transition(&mut package, position);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package(&package).expect("selected QSpec evidence")
    else {
        panic!("bad frame must refuse before transition anchor");
    };
    assert_eq!(refusal.code, Code::MissingDeclaration, "{refusal:?}");
    assert_eq!(refusal.cause, Some(Cause::MissingName));
    assert!(
        refusal.path.as_ref().is_some_and(|path| path
            .to_string()
            .starts_with("/semantic_graph/nodes/10/body/modifies/0")),
        "{refusal:?}"
    );
}

fn with_bad_frame() -> Value {
    let mut package = published();
    let position = transition_position(&package);
    package["semantic_graph"]["nodes"][10]["body"]["modifies"][0]["name"] = json!("not_a_field");
    fixture_support::mint_ungrouped_structural_keys(&mut package);
    for clause in [12, 13] {
        rekey_application_with_references(&mut package, clause);
    }
    refresh_transition(&mut package, position);
    package
}

#[trace("TC-064", "FR-067-AC-4")]
#[test]
fn graph_dependency_and_selected_model_owner_refuse_before_a_frame_defect() {
    let mut graph = with_bad_frame();
    graph["semantic_graph"]["nodes"][9]["dependencies"] = json!([]);
    fixture_support::refresh_identity_projection(&mut graph);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package(&graph).expect("selected QSpec evidence")
    else {
        panic!("graph dependency defect must refuse before frame");
    };
    assert_eq!(refusal.code, Code::InvalidSemanticGraph, "{refusal:?}");
    assert_eq!(refusal.cause, None);
    assert_eq!(
        refusal.path.as_ref().map(ToString::to_string),
        Some("/semantic_graph/nodes/9/dependencies".into())
    );

    let mut selected = with_bad_frame();
    selected["semantic_graph"]["nodes"][3]["owner"]["node"] = json!("ix://acme/orders/Missing");
    fixture_support::refresh_identity_projection(&mut selected);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package(&selected).expect("selected QSpec evidence")
    else {
        panic!("selected-model owner defect must refuse before frame");
    };
    assert_eq!(refusal.code, Code::MissingDeclaration, "{refusal:?}");
    assert_eq!(refusal.cause, Some(Cause::MissingSelection));
    assert_eq!(
        refusal.path.as_ref().map(ToString::to_string),
        Some("/semantic_graph/nodes/3/node_id".into())
    );
}

#[trace("TC-064", "FR-067-AC-4")]
#[test]
fn anchor_refusal_precedes_transition_and_state_refusal_precedes_operation_catalog() {
    let mut anchor = published();
    let position = transition_position(&anchor);
    anchor["semantic_graph"]["nodes"][11]["body"]["members"][1]["value"]["value"] =
        json!("missing");
    fixture_support::mint_ungrouped_structural_keys(&mut anchor);
    for clause in [12, 13] {
        rekey_application_with_references(&mut anchor, clause);
    }
    let model = target(&anchor, "model", "object_type");
    anchor["semantic_graph"]["nodes"][position]["body"]["arguments"][0]["value"]["target"] = model;
    refresh_transition(&mut anchor, position);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package(&anchor).expect("selected QSpec evidence")
    else {
        panic!("anchor defect must refuse before transition defect");
    };
    assert_eq!(refusal.code, Code::MissingDeclaration, "{refusal:?}");
    assert_eq!(refusal.cause, Some(Cause::MissingName));
    assert!(
        refusal
            .path
            .as_ref()
            .is_some_and(|path| path.to_string().starts_with("/semantic_graph/nodes/11")),
        "{refusal:?}"
    );

    let mut operation = published();
    operation["semantic_graph"]["nodes"][position]["body"]["operation"]["laws"][0]["definition"]
        ["identity"] = json!("missing-profile");
    refresh_transition(&mut operation, position);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package(&operation).expect("selected QSpec evidence")
    else {
        panic!("unselected law must reach the operation step by itself");
    };
    assert_eq!(refusal.code, Code::InvalidPackage, "{refusal:?}");
    assert_eq!(refusal.cause, Some(Cause::OperationLawUnselected));
    assert_eq!(
        refusal.path.as_ref().map(ToString::to_string),
        Some(format!(
            "/semantic_graph/nodes/{position}/body/operation/laws/0/definition"
        ))
    );
    let model = target(&operation, "model", "object_type");
    operation["semantic_graph"]["nodes"][position]["body"]["arguments"][0]["value"]["target"] =
        model;
    refresh_transition(&mut operation, position);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package(&operation).expect("selected QSpec evidence")
    else {
        panic!("transition state defect must refuse before operation law defect");
    };
    assert_eq!(refusal.code, Code::InvalidModelBinding, "{refusal:?}");
    assert_eq!(refusal.cause, Some(Cause::MalformedDeclaration));
    assert_eq!(
        refusal.path.as_ref().map(ToString::to_string),
        Some(format!(
            "/semantic_graph/nodes/{position}/body/arguments/0/value/target"
        ))
    );
}

#[trace("TC-064", "FR-067-AC-4")]
#[test]
fn a_clause_anchor_defect_precedes_a_transition_anchor_defect() {
    let mut package = published();
    let position = transition_position(&package);
    let model = target(&package, "model", "object_type");
    package["semantic_graph"]["nodes"][12]["body"]["arguments"][1]["target"] = model.clone();
    let mut references = BTreeMap::new();
    collect_references(
        &package["semantic_graph"]["nodes"][12]["body"],
        &mut references,
    );
    package["semantic_graph"]["nodes"][12]["dependencies"] =
        Value::Array(references.into_values().collect());
    fixture_support::rekey_application_node(&mut package, 12);
    package["semantic_graph"]["nodes"][position]["body"]["arguments"][0]["value"]["target"] = model;
    refresh_transition(&mut package, position);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package(&package).expect("selected QSpec evidence")
    else {
        panic!("bad clause must refuse before transition");
    };
    assert_eq!(refusal.code, Code::InvalidModelBinding, "{refusal:?}");
    assert_eq!(refusal.cause, Some(Cause::MalformedDeclaration));
    assert_eq!(
        refusal.path.as_ref().map(ToString::to_string),
        Some("/semantic_graph/nodes/12/body/arguments/1/target".into())
    );
}

#[trace("TC-064", "FR-067-AC-2")]
#[test]
fn transition_application_at_a_binary_expression_root_refuses_before_anchor_join() {
    let mut package = published();
    let position = transition_position(&package);
    let mut transition_body = package["semantic_graph"]["nodes"][position]["body"].clone();
    // The published precondition contains this expression node. An empty
    // optional pre keeps the wrong-root case acyclic and schema-valid.
    transition_body["arguments"][3]["value"]["members"] = json!([]);
    let model = target(&package, "model", "object_type");
    package["semantic_graph"]["nodes"][9]["body"] = transition_body;
    package["semantic_graph"]["nodes"][position]["body"]["arguments"][0]["value"]["target"] = model;
    fixture_support::settle(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package(&package).expect("selected QSpec evidence")
    else {
        panic!("transition application on expression node must refuse");
    };
    assert_eq!(refusal.code, Code::IllTyped, "{refusal:?}");
    assert_eq!(refusal.cause, Some(Cause::OperatorIneligible));
    assert_eq!(
        refusal.path.as_ref().map(ToString::to_string),
        Some("/semantic_graph/nodes/9/body".into())
    );
}

#[trace("TC-064", "FR-067-AC-4")]
#[test]
fn transition_joins_refuse_in_anchor_subject_input_clause_order() {
    for earlier in ["anchor", "subject", "input", "pre"] {
        let mut package = published();
        let position = transition_position(&package);
        let model = target(&package, "model", "object_type");
        let self_parameter = parameter(&package, "self");
        let pre = package["semantic_graph"]["nodes"][position]["body"]["arguments"][3]["value"]
            ["members"][0]["target"]
            .clone();
        let post = package["semantic_graph"]["nodes"][position]["body"]["arguments"][4]["value"]
            ["target"]
            .clone();
        let node = &mut package["semantic_graph"]["nodes"][position];
        let target_path = match earlier {
            "anchor" => {
                node["body"]["arguments"][0]["value"]["target"] = model;
                node["body"]["arguments"][1]["value"]["target"] = self_parameter;
                format!("/semantic_graph/nodes/{position}/body/arguments/0/value/target")
            }
            "subject" => {
                node["body"]["arguments"][1]["value"]["target"] = self_parameter.clone();
                node["body"]["arguments"][2]["value"]["members"][0]["target"] = self_parameter;
                format!("/semantic_graph/nodes/{position}/body/arguments/1/value/target")
            }
            "input" => {
                node["body"]["arguments"][2]["value"]["members"][0]["target"] = self_parameter;
                node["body"]["arguments"][3]["value"]["members"][0]["target"] = post;
                format!("/semantic_graph/nodes/{position}/body/arguments/2/value/members/0/target")
            }
            "pre" => {
                node["body"]["arguments"][3]["value"]["members"][0]["target"] = post;
                node["body"]["arguments"][4]["value"]["target"] = pre;
                format!("/semantic_graph/nodes/{position}/body/arguments/3/value/members/0/target")
            }
            _ => unreachable!("closed ordering list"),
        };
        refresh_transition(&mut package, position);
        let CheckedPackageV2ReadResult::Refused(refusal) =
            read_package(&package).expect("selected QSpec evidence")
        else {
            panic!("{earlier} and later defect must refuse");
        };
        assert_eq!(
            refusal.code,
            Code::InvalidModelBinding,
            "{earlier}: {refusal:?}"
        );
        assert_eq!(refusal.cause, Some(Cause::MalformedDeclaration));
        assert_eq!(
            refusal.path.as_ref().map(ToString::to_string),
            Some(target_path)
        );
    }
}
