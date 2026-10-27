// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Public reader checks for the first protocol validation passes.

use crate::support::checked_package::{
    admitted_protocol_families, application_node_key, canonical, domain_package_digest,
    domain_package_document, evidence_for, family_key, node_id, protocol_named_node,
    protocol_role_node, rebuild_source_map, refresh_identity, refusal, refusal_at,
    rekey_application_node, rename_node, selected_interface_model_node, settle, structural_key,
    v2_all_families,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedPackageReadLimits, CheckedPackageRefusalCause, CheckedPackageRefusalCode,
    CheckedPackageV2, CheckedPackageV2ReadResult, CheckedProtocolActivation,
    CheckedProtocolActivationRow, CheckedProtocolRefusalDetail,
};
use serde_json::{json, Value};

fn read(package: &Value) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(package),
    )
}

/// Trace: FR-038-AC-291
#[trace("TC-048")]
#[test]
fn protocol_finish_reads_over_at_its_stage() {
    let mut package = v2_all_families();
    let clause = clause_position(&package);
    let finish = protocol_position(&package, "obligation");
    let over = package["semantic_graph"]["nodes"][clause]["body"]["arguments"][0]["value"]
        ["target"]
        .clone();
    package["semantic_graph"]["nodes"][finish]["body"]["arguments"][1]["value"] =
        json!({"term": "reference", "target": over});
    settle(&mut package);
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

/// Trace: FR-038-AC-291
#[trace("TC-048")]
#[test]
fn protocol_check_refuses_over_before_scope() {
    let mut package = v2_all_families();
    let clause = clause_position(&package);
    let root = protocol_position(&package, "control");
    let over = package["semantic_graph"]["nodes"][clause]["body"]["arguments"][0]["value"]
        ["target"]
        .clone();
    package["semantic_graph"]["nodes"][root]["body"]["operation"]["member"]["operator"] =
        json!("check");
    package["semantic_graph"]["nodes"][root]["body"]["arguments"] = json!([
        {"term": "binding", "name": "condition", "value": {"term": "reference", "target": over}}
    ]);
    settle(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("check must refuse model-state input");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::IllTyped);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::OperatorIneligible)
    );
    assert_eq!(
        refusal.protocol_detail.as_deref(),
        Some(&CheckedProtocolRefusalDetail::ModelStateRead {
            slot: "check.condition".into()
        })
    );
}

/// Trace: FR-038-AC-291
#[trace("TC-048")]
#[test]
fn protocol_check_refuses_finish_binder_before_finish() {
    let mut package = v2_all_families();
    let root = protocol_position(&package, "control");
    let finish = protocol_position(&package, "obligation");
    let binder = package["semantic_graph"]["nodes"][finish]["body"]["arguments"][0]["value"]
        ["target"]
        .clone();
    package["semantic_graph"]["nodes"][root]["body"]["operation"]["member"]["operator"] =
        json!("check");
    package["semantic_graph"]["nodes"][root]["body"]["arguments"] = json!([
        {"term": "binding", "name": "condition", "value": {"term": "reference", "target": binder}}
    ]);
    settle(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("check must refuse a finish binder before finish");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MissingDeclaration);
    assert_eq!(refusal.cause, Some(CheckedPackageRefusalCause::MissingName));
    assert!(matches!(refusal.protocol_detail.as_deref(),
        Some(CheckedProtocolRefusalDetail::BinderScope { stage, read, .. })
        if stage.as_ref() == "run" && read.as_ref() == "stage"));
}

/// Trace: FR-038-AC-291
#[trace("TC-048")]
#[test]
fn await_event_binder_is_readable_on_then_but_not_timeout() {
    let (mut package, children) = protocol_holder_case("await");
    let event = children[0];
    let then = children[1];
    let timeout = children[2];
    let binder = package["semantic_graph"]["nodes"][event]["body"]["arguments"][2]["value"]
        ["target"]
        .clone();
    let check_body = json!([
        {"term": "binding", "name": "condition", "value": {"term": "reference", "target": binder}}
    ]);
    package["semantic_graph"]["nodes"][then]["body"]["operation"]["member"]["operator"] =
        json!("check");
    package["semantic_graph"]["nodes"][then]["body"]["arguments"] = check_body.clone();
    settle(&mut package);
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));

    package["semantic_graph"]["nodes"][then]["body"]["operation"]["member"]["operator"] =
        json!("sequence");
    package["semantic_graph"]["nodes"][then]["body"]["arguments"] = json!([
        {"term": "binding", "name": "children", "value": {"term": "aggregate", "members": []}}
    ]);
    package["semantic_graph"]["nodes"][timeout]["body"]["operation"]["member"]["operator"] =
        json!("check");
    package["semantic_graph"]["nodes"][timeout]["body"]["arguments"] = check_body;
    settle(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("timeout must not read the matched event binder");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MissingDeclaration);
    assert_eq!(refusal.cause, Some(CheckedPackageRefusalCause::MissingName));
    assert!(matches!(refusal.protocol_detail.as_deref(),
        Some(CheckedProtocolRefusalDetail::BinderScope { stage, read, .. })
        if stage.as_ref() == "run" && read.as_ref() == "stage"));
}

/// Trace: FR-038-AC-291
#[trace("TC-048")]
#[test]
fn sequence_successor_and_finish_read_predecessor_binders() {
    let (mut package, effect, _) = package_with_admitted_effect();
    let attempt_binder = package["semantic_graph"]["nodes"][effect - 1]["body"]["arguments"][4]
        ["value"]["target"]
        .clone();
    let effect_binder = package["semantic_graph"]["nodes"][effect]["body"]["arguments"][1]["value"]
        ["target"]
        .clone();
    package["semantic_graph"]["nodes"][effect]["body"]["arguments"][3]["value"] =
        json!({"term": "reference", "target": attempt_binder});
    let finish = protocol_position(&package, "obligation");
    package["semantic_graph"]["nodes"][finish]["body"]["arguments"][1]["value"] =
        json!({"term": "reference", "target": effect_binder});
    settle(&mut package);
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

fn decision_fact_fixture() -> (Value, usize, usize, usize) {
    let (mut package, effect, _) = package_with_admitted_effect();
    let root = protocol_position(&package, "control");
    let attempt = effect - 1;
    let old_binder = package["semantic_graph"]["nodes"][attempt]["body"]["arguments"][4]["value"]
        ["target"]
        .clone();
    let record_body = json!({"term": "aggregate", "members": [
        {"term": "binding", "name": "ok", "value": {"term": "reference",
            "target": node_id(&family_key("aaaa"))}}
    ]});
    let record_key = structural_key("composite_type", "record", None, &record_body);
    let mut record = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "composite_type")
        .expect("composite type")
        .clone();
    record["node_id"] = node_id(&record_key);
    record["semantic_form"] = json!("record");
    record["semantic_type"] = node_id(&record_key);
    record["body"] = record_body;
    record["dependencies"] = json!([node_id(&family_key("aaaa"))]);
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(record);
    let binder_position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_id"] == old_binder)
        .expect("attempt binder");
    package["semantic_graph"]["nodes"][binder_position]["semantic_type"] = node_id(&record_key);
    let binder_body = package["semantic_graph"]["nodes"][binder_position]["body"].clone();
    let binder_key = structural_key("value", "parameter", Some(&record_key), &binder_body);
    rename_node(
        &mut package,
        old_binder["digest"].as_str().expect("old binder key"),
        &binder_key,
    );
    let binder = node_id(&binder_key);
    let effect_binder = package["semantic_graph"]["nodes"][effect]["body"]["arguments"][1]["value"]
        ["target"]
        .clone();
    let role = package["semantic_graph"]["nodes"][attempt]["body"]["arguments"][0]["value"]
        ["target"]
        .clone();
    let existing = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "expression" && node["semantic_form"] == "reference")
        .expect("reference expression")
        .clone();
    let field_body = json!({
        "term": "application", "operator": "query",
        "operation": {"identity": "quire.op.record.project", "laws": [], "mode": null,
            "member": {"kind": "field", "declaration": node_id(&record_key),
                "name": "ok"}, "leaves": []},
        "result_type": node_id(&family_key("aaaa")),
        "arguments": [{"term": "reference", "target": binder}],
    });
    let mut field = existing;
    field["semantic_form"] = json!("query");
    field["body"] = field_body;
    field["node_id"] = node_id(&application_node_key(
        "expression",
        "query",
        &family_key("aaaa"),
        &field["body"],
    ));
    let field_position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(field);
    let field_target = package["semantic_graph"]["nodes"][field_position]["node_id"].clone();
    let child_body = package["semantic_graph"]["nodes"][root]["body"].clone();
    let mut children = Vec::new();
    for name in ["Left", "Right"] {
        let mut body = child_body.clone();
        body["arguments"][0]["value"]["members"] = json!([]);
        let child = protocol_named_node(
            "control",
            &["Example", "OrderFlow", "run", "Main", "Decision", name],
            body,
            &[],
        );
        children.push(child["node_id"].clone());
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(child);
    }
    let mut choice_body = package["semantic_graph"]["nodes"][root]["body"].clone();
    choice_body["operation"]["member"]["operator"] = json!("choice");
    choice_body["arguments"] = json!([
        {"term": "binding", "name": "owner", "value": {"term": "reference", "target": role}},
        {"term": "binding", "name": "visible", "value": {"term": "aggregate", "members": [
            {"term": "reference", "target": field_target}
        ]}},
        {"term": "binding", "name": "cases", "value": {"term": "aggregate", "members": [
            {"term": "aggregate", "members": [
                {"term": "binding", "name": "label", "value": {"term": "literal", "type": node_id(&family_key("a1a1")), "value_kind": "text", "value": "Left"}},
                {"term": "binding", "name": "guard", "value": {"term": "reference", "target": field_target}},
                {"term": "binding", "name": "body", "value": {"term": "reference", "target": children[0]}}
            ]},
            {"term": "aggregate", "members": [
                {"term": "binding", "name": "label", "value": {"term": "literal", "type": node_id(&family_key("a1a1")), "value_kind": "text", "value": "Right"}},
                {"term": "binding", "name": "guard", "value": {"term": "literal", "type": node_id(&family_key("aaaa")), "value_kind": "boolean", "value": true}},
                {"term": "binding", "name": "body", "value": {"term": "reference", "target": children[1]}}
            ]}
        ]}}
    ]);
    let choice = protocol_named_node(
        "control",
        &["Example", "OrderFlow", "run", "Main", "Decision"],
        choice_body,
        &[],
    );
    let decision = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    let choice_target = choice["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(choice);
    package["semantic_graph"]["nodes"][root]["body"]["arguments"][0]["value"]["members"]
        .as_array_mut()
        .expect("root children")
        .push(json!({"term": "reference", "target": choice_target}));
    settle(&mut package);
    let effect_binder_position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_id"] == effect_binder)
        .expect("effect binder");
    (package, decision, field_position, effect_binder_position)
}

/// Trace: FR-038-AC-292
#[trace("TC-048")]
#[test]
fn decision_fact_requires_individual_visible_entry_and_eligible_binder() {
    let (package, decision, field, effect_binder) = decision_fact_fixture();
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));

    let mut unlisted = package.clone();
    unlisted["semantic_graph"]["nodes"][decision]["body"]["arguments"][1]["value"]["members"] =
        json!([]);
    settle(&mut unlisted);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&unlisted) else {
        panic!("unlisted decision fact must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::WrongSnapshot);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::WrongObservation)
    );
    assert!(matches!(refusal.protocol_detail.as_deref(),
        Some(CheckedProtocolRefusalDetail::DecisionFact { reason, read, .. })
        if reason.as_ref() == "unlisted" && read.field.as_ref() == "ok"));

    let mut ineligible = package;
    ineligible["semantic_graph"]["nodes"][field]["body"]["arguments"][0]["target"] =
        ineligible["semantic_graph"]["nodes"][effect_binder]["node_id"].clone();
    settle(&mut ineligible);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&ineligible) else {
        panic!("effect binder is ineligible at a decision");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::WrongSnapshot);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::WrongObservation)
    );
    assert!(matches!(refusal.protocol_detail.as_deref(),
        Some(CheckedProtocolRefusalDetail::DecisionFact { reason, read, .. })
        if reason.as_ref() == "ineligible-kind" && read.field.as_ref() == "ok"));
}

/// Trace: FR-038-AC-292
#[trace("TC-048")]
#[test]
fn decision_fact_other_role_and_composite_entry_do_not_grant_observation() {
    let (package, decision, field, _) = decision_fact_fixture();
    let mut other_role = package.clone();
    let role_body = other_role["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "protocol" && node["semantic_form"] == "role")
        .expect("role")["body"]
        .clone();
    let other = protocol_named_node(
        "role",
        &["Example", "OrderFlow", "role", "Other"],
        role_body,
        &[],
    );
    let other_target = other["node_id"].clone();
    other_role["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(other);
    let clause = clause_position(&other_role);
    let roles = other_role["semantic_graph"]["nodes"][clause]["body"]["arguments"][3]["value"]
        ["members"]
        .as_array_mut()
        .expect("roles");
    roles.push(json!({"term": "reference", "target": other_target}));
    roles.sort_by(|left, right| {
        left["target"]["digest"]
            .as_str()
            .cmp(&right["target"]["digest"].as_str())
    });
    other_role["semantic_graph"]["nodes"][decision]["body"]["arguments"][0]["value"] =
        json!({"term": "reference", "target": other_target});
    settle(&mut other_role);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&other_role) else {
        panic!("another role cannot read the attempt's fact");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::WrongSnapshot);
    assert!(matches!(refusal.protocol_detail.as_deref(),
        Some(CheckedProtocolRefusalDetail::DecisionFact { reason, .. })
        if reason.as_ref() == "other-role"));

    let mut composite = package;
    let mut entry = composite["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "expression" && node["semantic_form"] == "reference")
        .expect("reference expression")
        .clone();
    let field_target = composite["semantic_graph"]["nodes"][field]["node_id"].clone();
    entry["semantic_form"] = json!("binary");
    entry["body"] = json!({
        "term": "application", "operator": "binary",
        "operation": {"identity": "quire.op.boolean.and", "laws": [], "mode": null,
            "member": null, "leaves": []},
        "result_type": node_id(&family_key("aaaa")),
        "arguments": [
            {"term": "reference", "target": field_target},
            {"term": "literal", "type": node_id(&family_key("aaaa")),
                "value_kind": "boolean", "value": true}
        ]
    });
    entry["node_id"] = node_id(&application_node_key(
        "expression",
        "binary",
        &family_key("aaaa"),
        &entry["body"],
    ));
    entry["dependencies"] = json!([field_target]);
    let entry_target = entry["node_id"].clone();
    composite["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(entry);
    composite["semantic_graph"]["nodes"][decision]["body"]["arguments"][1]["value"]["members"] =
        json!([{"term": "reference", "target": entry_target}]);
    settle(&mut composite);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&composite) else {
        panic!("listing a containing entry does not list its field operand");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::WrongSnapshot);
    assert!(matches!(refusal.protocol_detail.as_deref(),
        Some(CheckedProtocolRefusalDetail::DecisionFact { reason, .. })
        if reason.as_ref() == "unlisted"));
}

/// Trace: FR-038-AC-292
#[trace("TC-048")]
#[test]
fn nested_projection_exposes_its_unlisted_inner_fact() {
    let (mut package, decision, field, _) = decision_fact_fixture();
    let record = package["semantic_graph"]["nodes"][field]["body"]["operation"]["member"]
        ["declaration"]
        .clone();
    let binder =
        package["semantic_graph"]["nodes"][field]["body"]["arguments"][0]["target"].clone();
    let mut outer_record = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_id"] == record)
        .expect("record")
        .clone();
    outer_record["body"] = json!({"term": "aggregate", "members": [
        {"term": "binding", "name": "inner", "value": {"term": "reference", "target": record}}
    ]});
    let outer_key = structural_key("composite_type", "record", None, &outer_record["body"]);
    outer_record["node_id"] = node_id(&outer_key);
    outer_record["semantic_type"] = node_id(&outer_key);
    outer_record["dependencies"] = json!([record]);
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(outer_record);

    let binder_position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_id"] == binder)
        .expect("binder");
    package["semantic_graph"]["nodes"][binder_position]["semantic_type"] = node_id(&outer_key);
    let binder_body = package["semantic_graph"]["nodes"][binder_position]["body"].clone();
    let binder_key = structural_key("value", "parameter", Some(&outer_key), &binder_body);
    rename_node(
        &mut package,
        binder["digest"].as_str().expect("binder key"),
        &binder_key,
    );

    let mut outer_field = package["semantic_graph"]["nodes"][field].clone();
    package["semantic_graph"]["nodes"][field]["body"]["operation"]["member"]["declaration"] =
        node_id(&outer_key);
    package["semantic_graph"]["nodes"][field]["body"]["operation"]["member"]["name"] =
        json!("inner");
    package["semantic_graph"]["nodes"][field]["body"]["result_type"] = record.clone();
    package["semantic_graph"]["nodes"][field]["semantic_type"] = record;
    settle(&mut package);

    let inner = package["semantic_graph"]["nodes"][field]["node_id"].clone();
    outer_field["body"]["arguments"][0]["target"] = inner;
    outer_field["node_id"] = node_id(&application_node_key(
        "expression",
        "query",
        &family_key("aaaa"),
        &outer_field["body"],
    ));
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(outer_field);
    settle(&mut package);
    let outer = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .last()
        .expect("outer projection")["node_id"]
        .clone();
    package["semantic_graph"]["nodes"][decision]["body"]["arguments"][1]["value"]["members"] =
        json!([]);
    package["semantic_graph"]["nodes"][decision]["body"]["arguments"][2]["value"]["members"][0]
        ["members"][1]["value"] = json!({"term": "reference", "target": outer});
    settle(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("nested unlisted fact must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::WrongSnapshot);
    assert!(matches!(refusal.protocol_detail.as_deref(),
        Some(CheckedProtocolRefusalDetail::DecisionFact { reason, read, .. })
        if reason.as_ref() == "unlisted" && read.field.as_ref() == "inner"));
}

/// Trace: FR-038-AC-292
#[trace("TC-048")]
#[test]
fn repeat_decision_uses_visible_facts_after_binder_scope() {
    let (package, decision, field, _) = decision_fact_fixture();
    let mut out_of_scope = package.clone();
    let finish = protocol_position(&out_of_scope, "obligation");
    let finish_binder = out_of_scope["semantic_graph"]["nodes"][finish]["body"]["arguments"][0]
        ["value"]["target"]
        .clone();
    out_of_scope["semantic_graph"]["nodes"][field]["body"]["arguments"][0]["target"] =
        finish_binder;
    settle(&mut out_of_scope);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&out_of_scope) else {
        panic!("scope must precede decision-fact eligibility");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MissingDeclaration);
    assert_eq!(refusal.cause, Some(CheckedPackageRefusalCause::MissingName));
    assert!(matches!(refusal.protocol_detail.as_deref(),
        Some(CheckedProtocolRefusalDetail::BinderScope { stage, read, .. })
        if stage.as_ref() == "run" && read.as_ref() == "stage"));

    let mut repeated = package;
    let cases = repeated["semantic_graph"]["nodes"][decision]["body"]["arguments"][2]["value"]
        ["members"]
        .as_array()
        .expect("cases");
    let body = cases[0]["members"][2]["value"].clone();
    let exhausted = cases[1]["members"][2]["value"].clone();
    let body_position = repeated["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_id"] == body["target"])
        .expect("repeat body");
    repeated["semantic_graph"]["nodes"][body_position]["body"]["operation"]["member"]["operator"] =
        json!("fence");
    repeated["semantic_graph"]["nodes"][body_position]["body"]["arguments"] = json!([
        {"term": "binding", "name": "ordering", "value": {"term": "literal",
            "type": node_id(&family_key("a1a1")), "value_kind": "text", "value": "seq_cst"}}
    ]);
    let role =
        repeated["semantic_graph"]["nodes"][decision]["body"]["arguments"][0]["value"].clone();
    let fact = repeated["semantic_graph"]["nodes"][field]["node_id"].clone();
    repeated["semantic_graph"]["nodes"][decision]["body"]["operation"]["member"]["operator"] =
        json!("repeat");
    repeated["semantic_graph"]["nodes"][decision]["body"]["arguments"] = json!([
        {"term": "binding", "name": "owner", "value": role},
        {"term": "binding", "name": "visible", "value": {"term": "aggregate", "members": []}},
        {"term": "binding", "name": "maximum", "value": {"term": "aggregate", "members": [
            {"term": "literal", "type": node_id(&family_key("a3a3")),
                "value_kind": "integer", "value": "1"}
        ]}},
        {"term": "binding", "name": "proof", "value": {"term": "aggregate", "members": []}},
        {"term": "binding", "name": "guard", "value": {"term": "reference", "target": fact}},
        {"term": "binding", "name": "body", "value": body},
        {"term": "binding", "name": "exhausted", "value": {"term": "aggregate", "members": [exhausted]}}
    ]);
    settle(&mut repeated);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&repeated) else {
        panic!("repeat guard must list an observed field");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::WrongSnapshot);
    assert!(matches!(refusal.protocol_detail.as_deref(),
        Some(CheckedProtocolRefusalDetail::DecisionFact { reason, .. })
        if reason.as_ref() == "unlisted"));
}

/// Trace: FR-038-AC-292
#[trace("TC-048")]
#[test]
fn event_fact_is_visible_to_its_role() {
    let (mut package, _, field, event_binder_position) = decision_fact_fixture();
    let record = package["semantic_graph"]["nodes"][field]["body"]["operation"]["member"]
        ["declaration"]
        .clone();
    let old_binder = package["semantic_graph"]["nodes"][event_binder_position]["node_id"].clone();
    package["semantic_graph"]["nodes"][event_binder_position]["semantic_type"] = record.clone();
    let binder_body = package["semantic_graph"]["nodes"][event_binder_position]["body"].clone();
    let binder_key = structural_key(
        "value",
        "parameter",
        Some(record["digest"].as_str().expect("record key")),
        &binder_body,
    );
    rename_node(
        &mut package,
        old_binder["digest"].as_str().expect("old binder key"),
        &binder_key,
    );
    let event_binder = node_id(&binder_key);
    let event = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| {
            node["node_tag"] == "protocol"
                && node["semantic_form"] == "control"
                && node["body"]["operation"]["member"]["operator"] == "effect"
        })
        .expect("effect control");
    let attempt = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| {
            node["node_tag"] == "protocol"
                && node["semantic_form"] == "control"
                && node["body"]["operation"]["member"]["operator"] == "attempt"
        })
        .expect("attempt control");
    let role = package["semantic_graph"]["nodes"][attempt]["body"]["arguments"][0]["value"].clone();
    let constraint =
        package["semantic_graph"]["nodes"][event]["body"]["arguments"][3]["value"].clone();
    package["semantic_graph"]["nodes"][event]["body"]["operation"]["member"]["operator"] =
        json!("event");
    package["semantic_graph"]["nodes"][event]["body"]["arguments"] = json!([
        {"term": "binding", "name": "role", "value": role},
        {"term": "binding", "name": "compensation", "value": {"term": "aggregate", "members": []}},
        {"term": "binding", "name": "binder", "value": {"term": "reference", "target": event_binder}},
        {"term": "binding", "name": "related", "value": {"term": "aggregate", "members": []}},
        {"term": "binding", "name": "constraint", "value": constraint}
    ]);
    package["semantic_graph"]["nodes"][field]["body"]["arguments"][0]["target"] = event_binder;
    settle(&mut package);
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

/// Trace: FR-038-AC-292
#[trace("TC-048")]
#[test]
fn commit_record_is_ineligible_at_a_decision() {
    let (mut package, _, field, commit_binder_position) = decision_fact_fixture();
    let record = package["semantic_graph"]["nodes"][field]["body"]["operation"]["member"]
        ["declaration"]
        .clone();
    let old_binder = package["semantic_graph"]["nodes"][commit_binder_position]["node_id"].clone();
    package["semantic_graph"]["nodes"][commit_binder_position]["semantic_type"] = record.clone();
    let binder_body = package["semantic_graph"]["nodes"][commit_binder_position]["body"].clone();
    let binder_key = structural_key(
        "value",
        "parameter",
        Some(record["digest"].as_str().expect("record key")),
        &binder_body,
    );
    rename_node(
        &mut package,
        old_binder["digest"].as_str().expect("old binder key"),
        &binder_key,
    );
    let binder = node_id(&binder_key);
    let commit = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| {
            node["node_tag"] == "protocol"
                && node["semantic_form"] == "control"
                && node["body"]["operation"]["member"]["operator"] == "effect"
        })
        .expect("effect control");
    let attempt = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| {
            node["node_tag"] == "protocol"
                && node["semantic_form"] == "control"
                && node["body"]["operation"]["member"]["operator"] == "attempt"
        })
        .expect("attempt control");
    let role = package["semantic_graph"]["nodes"][attempt]["body"]["arguments"][0]["value"].clone();
    let constraint =
        package["semantic_graph"]["nodes"][commit]["body"]["arguments"][3]["value"].clone();
    package["semantic_graph"]["nodes"][commit]["body"]["operation"]["member"]["operator"] =
        json!("commit");
    package["semantic_graph"]["nodes"][commit]["body"]["arguments"] = json!([
        {"term": "binding", "name": "role", "value": role},
        {"term": "binding", "name": "binder", "value": {"term": "reference", "target": binder}},
        {"term": "binding", "name": "constraint", "value": constraint}
    ]);
    package["semantic_graph"]["nodes"][field]["body"]["arguments"][0]["target"] = binder;
    settle(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("commit binder is ineligible at a decision");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::WrongSnapshot);
    assert!(matches!(refusal.protocol_detail.as_deref(),
        Some(CheckedProtocolRefusalDetail::DecisionFact { reason, .. })
        if reason.as_ref() == "ineligible-kind"));
}

/// Trace: FR-038-AC-292
#[trace("TC-048")]
#[test]
fn receive_fact_is_visible_to_its_channels_to_role() {
    let (mut package, decision, field, receive_binder_position) = decision_fact_fixture();
    let record = package["semantic_graph"]["nodes"][field]["body"]["operation"]["member"]
        ["declaration"]
        .clone();
    let old_binder = package["semantic_graph"]["nodes"][receive_binder_position]["node_id"].clone();
    package["semantic_graph"]["nodes"][receive_binder_position]["semantic_type"] = record.clone();
    let binder_body = package["semantic_graph"]["nodes"][receive_binder_position]["body"].clone();
    let binder_key = structural_key(
        "value",
        "parameter",
        Some(record["digest"].as_str().expect("record key")),
        &binder_body,
    );
    rename_node(
        &mut package,
        old_binder["digest"].as_str().expect("old binder key"),
        &binder_key,
    );
    let received = node_id(&binder_key);
    let role = protocol_position(&package, "role");
    let channel = add_member_channel(&mut package, role);
    package["semantic_graph"]["nodes"][channel]["body"]["arguments"][2]["value"] =
        json!({"term": "reference", "target": record});
    let channel_target = package["semantic_graph"]["nodes"][channel]["node_id"].clone();
    let mut queue_body = package["semantic_graph"]["nodes"][channel]["body"].clone();
    queue_body["operation"]["member"]["operator"] = json!("queue");
    queue_body["arguments"] = json!([
        {"term": "binding", "name": "channel", "value": {"term": "reference", "target": channel_target}},
        {"term": "binding", "name": "capacity", "value": {"term": "aggregate", "members": [
            {"term": "binding", "name": "kind", "value": {"term": "literal", "type": node_id(&family_key("a1a1")), "value_kind": "text", "value": "literal"}},
            {"term": "binding", "name": "value", "value": {"term": "literal", "type": node_id(&family_key("a3a3")), "value_kind": "integer", "value": "4"}}
        ]}},
        {"term": "binding", "name": "overflow", "value": {"term": "literal", "type": node_id(&family_key("a1a1")), "value_kind": "text", "value": "reject"}},
        {"term": "binding", "name": "loss", "value": {"term": "aggregate", "members": []}}
    ]);
    let mut queue = protocol_named_node("queue", &[], queue_body, &[]);
    queue
        .as_object_mut()
        .expect("queue node")
        .remove("declaration");
    queue["occurrences"][0]["role"] = json!("generated");
    let queue_target = queue["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(queue);
    let clause = clause_position(&package);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][6]["value"]["members"] =
        json!([{"term": "reference", "target": queue_target}]);

    let send = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| {
            node["node_tag"] == "protocol"
                && node["semantic_form"] == "control"
                && node["body"]["operation"]["member"]["operator"] == "attempt"
        })
        .expect("attempt control");
    let receive = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| {
            node["node_tag"] == "protocol"
                && node["semantic_form"] == "control"
                && node["body"]["operation"]["member"]["operator"] == "effect"
        })
        .expect("effect control");
    let sent = package["semantic_graph"]["nodes"][send]["body"]["arguments"][4]["value"].clone();
    let sent_binder = sent["target"].clone();
    let sent_constraint =
        package["semantic_graph"]["nodes"][send]["body"]["arguments"][6]["value"].clone();
    let received_constraint =
        package["semantic_graph"]["nodes"][receive]["body"]["arguments"][3]["value"].clone();
    let send_target = package["semantic_graph"]["nodes"][send]["node_id"].clone();
    package["semantic_graph"]["nodes"][send]["body"]["operation"]["member"]["operator"] =
        json!("send");
    package["semantic_graph"]["nodes"][send]["body"]["arguments"] = json!([
        {"term": "binding", "name": "channel", "value": {"term": "reference", "target": channel_target}},
        {"term": "binding", "name": "binder", "value": sent},
        {"term": "binding", "name": "related", "value": {"term": "aggregate", "members": []}},
        {"term": "binding", "name": "constraint", "value": sent_constraint}
    ]);
    package["semantic_graph"]["nodes"][receive]["body"]["operation"]["member"]["operator"] =
        json!("receive");
    package["semantic_graph"]["nodes"][receive]["body"]["arguments"] = json!([
        {"term": "binding", "name": "channel", "value": {"term": "reference", "target": channel_target}},
        {"term": "binding", "name": "of", "value": {"term": "reference", "target": send_target}},
        {"term": "binding", "name": "binder", "value": {"term": "reference", "target": received}},
        {"term": "binding", "name": "related", "value": {"term": "aggregate", "members": []}},
        {"term": "binding", "name": "constraint", "value": received_constraint}
    ]);
    package["semantic_graph"]["nodes"][field]["body"]["arguments"][0]["target"] = received;
    settle(&mut package);
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));

    let mut other_role = package.clone();
    let role_body = other_role["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "protocol" && node["semantic_form"] == "role")
        .expect("role")["body"]
        .clone();
    let other = protocol_named_node(
        "role",
        &["Example", "OrderFlow", "role", "Other"],
        role_body,
        &[],
    );
    let other_target = other["node_id"].clone();
    other_role["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(other);
    let clause = clause_position(&other_role);
    let roles = other_role["semantic_graph"]["nodes"][clause]["body"]["arguments"][3]["value"]
        ["members"]
        .as_array_mut()
        .expect("roles");
    roles.push(json!({"term": "reference", "target": other_target}));
    roles.sort_by(|left, right| {
        left["target"]["digest"]
            .as_str()
            .cmp(&right["target"]["digest"].as_str())
    });
    other_role["semantic_graph"]["nodes"][decision]["body"]["arguments"][0]["value"] =
        json!({"term": "reference", "target": other_target});
    settle(&mut other_role);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&other_role) else {
        panic!("another role cannot read the receive record");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::WrongSnapshot);
    assert!(matches!(refusal.protocol_detail.as_deref(),
        Some(CheckedProtocolRefusalDetail::DecisionFact { reason, .. })
        if reason.as_ref() == "other-role"));

    let mut wrong = package;
    wrong["semantic_graph"]["nodes"][field]["body"]["arguments"][0]["target"] = sent_binder;
    settle(&mut wrong);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&wrong) else {
        panic!("send binder must be ineligible at a receive decision");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::WrongSnapshot);
    assert!(matches!(refusal.protocol_detail.as_deref(),
        Some(CheckedProtocolRefusalDetail::DecisionFact { reason, .. })
        if reason.as_ref() == "ineligible-kind"));
}

/// Trace: FR-038-AC-291
#[trace("TC-048")]
#[test]
fn choice_case_binder_does_not_escape_to_finish() {
    let (mut package, children) = protocol_holder_case("choice");
    let branch = children[0];
    let binder = clause_parameter(&mut package, "branch", "1");
    let role =
        package["semantic_graph"]["nodes"][protocol_position(&package, "role")]["node_id"].clone();
    let body = &mut package["semantic_graph"]["nodes"][branch]["body"];
    body["operation"]["member"]["operator"] = json!("event");
    body["arguments"] = json!([
        {"term": "binding", "name": "role", "value": {"term": "reference", "target": role}},
        {"term": "binding", "name": "compensation", "value": {"term": "aggregate", "members": []}},
        {"term": "binding", "name": "binder", "value": {"term": "reference", "target": binder}},
        {"term": "binding", "name": "related", "value": {"term": "aggregate", "members": []}},
        {"term": "binding", "name": "constraint", "value": {"term": "literal", "type": node_id(&family_key("aaaa")), "value_kind": "boolean", "value": true}}
    ]);
    let finish = protocol_position(&package, "obligation");
    package["semantic_graph"]["nodes"][finish]["body"]["arguments"][1]["value"] =
        json!({"term": "reference", "target": binder});
    settle(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("choice case binder must not escape to finish");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MissingDeclaration);
    assert_eq!(refusal.cause, Some(CheckedPackageRefusalCause::MissingName));
    assert!(matches!(refusal.protocol_detail.as_deref(),
        Some(CheckedProtocolRefusalDetail::BinderScope { stage, read, .. })
        if stage.as_ref() == "finish" && read.as_ref() == "stage"));
}

/// Trace: FR-038-AC-291
#[trace("TC-048")]
#[test]
fn all_join_exposes_branch_binder_and_any_join_does_not() {
    let (mut package, children) = protocol_holder_case("parallel");
    let branch = children[0];
    let binder = clause_parameter(&mut package, "branch", "1");
    let role =
        package["semantic_graph"]["nodes"][protocol_position(&package, "role")]["node_id"].clone();
    let body = &mut package["semantic_graph"]["nodes"][branch]["body"];
    body["operation"]["member"]["operator"] = json!("event");
    body["arguments"] = json!([
        {"term": "binding", "name": "role", "value": {"term": "reference", "target": role}},
        {"term": "binding", "name": "compensation", "value": {"term": "aggregate", "members": []}},
        {"term": "binding", "name": "binder", "value": {"term": "reference", "target": binder}},
        {"term": "binding", "name": "related", "value": {"term": "aggregate", "members": []}},
        {"term": "binding", "name": "constraint", "value": {"term": "literal", "type": node_id(&family_key("aaaa")), "value_kind": "boolean", "value": true}}
    ]);
    let finish = protocol_position(&package, "obligation");
    let original_finish_binder = package["semantic_graph"]["nodes"][finish]["body"]["arguments"][0]
        ["value"]["target"]
        .clone();
    let joined_finish_binder = clause_parameter(&mut package, "joined_finish", "2");
    package["semantic_graph"]["nodes"][finish]["body"]["arguments"][0]["value"]["target"] =
        joined_finish_binder;
    package["semantic_graph"]["nodes"][finish]["body"]["arguments"][1]["value"] =
        json!({"term": "reference", "target": binder});
    settle(&mut package);
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));

    let holder = holder_position(&package);
    package["semantic_graph"]["nodes"][holder]["body"]["arguments"][2]["value"]["members"][0]
        ["value"]["value"] = json!("any");
    package["semantic_graph"]["nodes"][finish]["body"]["arguments"][0]["value"]["target"] =
        original_finish_binder;
    settle(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("any join must not expose an optional branch binder");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MissingDeclaration);
    assert_eq!(refusal.cause, Some(CheckedPackageRefusalCause::MissingName));
    assert!(matches!(refusal.protocol_detail.as_deref(),
        Some(CheckedProtocolRefusalDetail::BinderScope { stage, .. }) if stage.as_ref() == "finish"));
}

/// Trace: FR-038-AC-291
#[trace("TC-048")]
#[test]
fn compensation_capture_stages_and_trigger_guard_scope() {
    let (mut package, _, compensation) = package_with_admitted_compensation();
    let forward = package["semantic_graph"]["nodes"][compensation]["body"]["arguments"][1]["value"]
        ["target"]
        .clone();
    let clause = clause_position(&package);
    let over = package["semantic_graph"]["nodes"][clause]["body"]["arguments"][0]["value"]
        ["target"]
        .clone();
    package["semantic_graph"]["nodes"][compensation]["body"]["arguments"][5]["value"]["members"]
        [0]["members"][1]["value"] = json!({"term": "reference", "target": forward});
    package["semantic_graph"]["nodes"][compensation]["body"]["arguments"][7]["value"]["members"]
        [0]["members"][1]["value"] = json!({"term": "reference", "target": over});
    settle(&mut package);
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));

    package["semantic_graph"]["nodes"][compensation]["body"]["arguments"][6]["value"]["members"]
        [1]["value"] = json!({"term": "reference", "target": forward});
    settle(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("trigger guard must not read forward binder");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MissingDeclaration);
    assert_eq!(refusal.cause, Some(CheckedPackageRefusalCause::MissingName));
    assert!(matches!(refusal.protocol_detail.as_deref(),
        Some(CheckedProtocolRefusalDetail::BinderScope { stage, read, .. })
        if stage.as_ref() == "trigger-guard" && read.as_ref() == "stage"));

    package["semantic_graph"]["nodes"][compensation]["body"]["arguments"][6]["value"]["members"]
        [1]["value"] = json!({"term": "reference", "target": over});
    settle(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("trigger guard must bar model state before scope");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::IllTyped);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::OperatorIneligible)
    );
    assert_eq!(
        refusal.protocol_detail.as_deref(),
        Some(&CheckedProtocolRefusalDetail::ModelStateRead {
            slot: "trigger.guard".into()
        })
    );
}

/// Trace: FR-038-AC-291
#[trace("TC-048")]
#[test]
fn protocol_capture_initializer_refuses_its_own_forward_read() {
    let mut package = v2_all_families();
    let capture = clause_parameter(&mut package, "captured", "1");
    let clause = clause_position(&package);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][2]["value"]["members"] = json!([{"term": "aggregate", "members": [
        {"term": "binding", "name": "binder", "value": {"term": "reference", "target": capture}},
        {"term": "binding", "name": "initializer", "value": {"term": "reference", "target": capture}}
    ]}]);
    settle(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("capture must not read itself before it is initialized");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MissingDeclaration);
    assert_eq!(refusal.cause, Some(CheckedPackageRefusalCause::MissingName));
    assert!(matches!(refusal.protocol_detail.as_deref(),
        Some(CheckedProtocolRefusalDetail::BinderScope { stage, read, .. })
        if stage.as_ref() == "protocol-capture" && read.as_ref() == "forward"));
}

/// Trace: FR-038-AC-291
#[trace("TC-048")]
#[test]
fn terminal_origin_reads_over_but_each_has_no_readable_binder() {
    let mut package = v2_all_families();
    let clause = clause_position(&package);
    let over = package["semantic_graph"]["nodes"][clause]["body"]["arguments"][0]["value"]
        ["target"]
        .clone();
    let finish = protocol_position(&package, "obligation");
    let mut body = package["semantic_graph"]["nodes"][finish]["body"].clone();
    body["operation"]["member"]["operator"] = json!("terminal");
    body["arguments"] = json!([
        {"term": "binding", "name": "mode", "value": {
            "term": "literal", "type": node_id(&family_key("a1a1")),
            "value_kind": "text", "value": "when"}},
        {"term": "binding", "name": "condition", "value": {
            "term": "aggregate", "members": [{"term": "reference", "target": over}]}}
    ]);
    let terminal = protocol_named_node(
        "obligation",
        &["Example", "OrderFlow", "terminal"],
        body,
        &[],
    );
    let terminal_key = terminal["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(terminal);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][9]["value"]["members"] =
        json!([{"term": "reference", "target": terminal_key}]);
    settle(&mut package);
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));

    let trigger = clause_parameter(&mut package, "opened", "1");
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][1]["value"]["members"] = json!([
        {"term": "binding", "name": "kind", "value": {
            "term": "literal", "type": node_id(&family_key("a1a1")),
            "value_kind": "text", "value": "each"}},
        {"term": "binding", "name": "trigger", "value": {
            "term": "reference", "target": trigger}}
    ]);
    relevel_finish_binder(&mut package, 2);
    settle(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("terminal under each must not read over");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MissingDeclaration);
    assert_eq!(refusal.cause, Some(CheckedPackageRefusalCause::MissingName));
    assert!(matches!(refusal.protocol_detail.as_deref(),
        Some(CheckedProtocolRefusalDetail::BinderScope { stage, read, .. })
        if stage.as_ref() == "terminal" && read.as_ref() == "stage"));
}

/// Trace: FR-038-AC-291
#[trace("TC-048")]
#[test]
fn role_lifetime_reads_c0_and_refuses_a_finish_binder() {
    let (mut package, role) = package_with_replicated_role();
    let clause = clause_position(&package);
    let over = package["semantic_graph"]["nodes"][clause]["body"]["arguments"][0]["value"]
        ["target"]
        .clone();
    let finish = protocol_position(&package, "obligation");
    let finish_binder = package["semantic_graph"]["nodes"][finish]["body"]["arguments"][0]["value"]
        ["target"]
        .clone();
    let replication =
        &mut package["semantic_graph"]["nodes"][role]["body"]["arguments"][1]["value"]["members"];
    replication[3]["value"]["value"] = json!("until");
    replication
        .as_array_mut()
        .expect("replication")
        .push(json!({
            "term": "binding", "name": "until", "value": {"term": "reference", "target": over}
        }));
    settle(&mut package);
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));

    package["semantic_graph"]["nodes"][role]["body"]["arguments"][1]["value"]["members"][4]
        ["value"]["target"] = finish_binder;
    settle(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("role lifetime must not read finish binder");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MissingDeclaration);
    assert_eq!(refusal.cause, Some(CheckedPackageRefusalCause::MissingName));
    assert!(matches!(refusal.protocol_detail.as_deref(),
        Some(CheckedProtocolRefusalDetail::BinderScope { stage, .. })
        if stage.as_ref() == "role-lifetime"));
}

/// Trace: FR-038-AC-291
#[trace("TC-048")]
#[test]
fn fifo_key_reads_message_binder_and_refuses_a_finish_binder() {
    let (mut package, _) = package_with_member_queue();
    let channel = protocol_position(&package, "channel");
    let binder = clause_parameter(&mut package, "message", "1");
    let finish = protocol_position(&package, "obligation");
    let finish_binder = package["semantic_graph"]["nodes"][finish]["body"]["arguments"][0]["value"]
        ["target"]
        .clone();
    package["semantic_graph"]["nodes"][channel]["body"]["arguments"][3]["value"]["members"] = json!([
        {"term": "binding", "name": "kind", "value": {
            "term": "literal", "type": node_id(&family_key("a1a1")),
            "value_kind": "text", "value": "fifo"}},
        {"term": "binding", "name": "binder", "value": {
            "term": "reference", "target": binder}},
        {"term": "binding", "name": "key", "value": {
            "term": "reference", "target": binder}}
    ]);
    settle(&mut package);
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));

    package["semantic_graph"]["nodes"][channel]["body"]["arguments"][3]["value"]["members"][2]
        ["value"]["target"] = finish_binder;
    settle(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("FIFO key must not read finish binder");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MissingDeclaration);
    assert_eq!(refusal.cause, Some(CheckedPackageRefusalCause::MissingName));
    assert!(matches!(refusal.protocol_detail.as_deref(),
        Some(CheckedProtocolRefusalDetail::BinderScope { stage, .. })
        if stage.as_ref() == "channel-key"));
}

/// Trace: FR-038-AC-291
#[trace("TC-048")]
#[test]
fn repeat_body_binder_is_readable_there_but_does_not_escape_to_finish() {
    let (mut package, children) = protocol_holder_case("repeat");
    let body = children[0];
    let binder = clause_parameter(&mut package, "repeated_event", "1");
    let role =
        package["semantic_graph"]["nodes"][protocol_position(&package, "role")]["node_id"].clone();
    package["semantic_graph"]["nodes"][body]["body"]["operation"]["member"]["operator"] =
        json!("event");
    package["semantic_graph"]["nodes"][body]["body"]["arguments"] = json!([
        {"term": "binding", "name": "role", "value": {"term": "reference", "target": role}},
        {"term": "binding", "name": "compensation", "value": {"term": "aggregate", "members": []}},
        {"term": "binding", "name": "binder", "value": {"term": "reference", "target": binder}},
        {"term": "binding", "name": "related", "value": {"term": "aggregate", "members": []}},
        {"term": "binding", "name": "constraint", "value": {"term": "reference", "target": binder}}
    ]);
    settle(&mut package);
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));

    let finish = protocol_position(&package, "obligation");
    package["semantic_graph"]["nodes"][finish]["body"]["arguments"][1]["value"] =
        json!({"term": "reference", "target": binder});
    settle(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("repeat body binder must not escape to finish");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MissingDeclaration);
    assert_eq!(refusal.cause, Some(CheckedPackageRefusalCause::MissingName));
    assert!(matches!(refusal.protocol_detail.as_deref(),
        Some(CheckedProtocolRefusalDetail::BinderScope { stage, .. })
        if stage.as_ref() == "finish"));
}

/// Trace: FR-038-AC-291
#[trace("TC-048")]
#[test]
fn compensation_retry_and_recovery_read_captures_but_bar_over() {
    let (mut package, _, compensation) = package_with_admitted_compensation();
    let clause = clause_position(&package);
    let over = package["semantic_graph"]["nodes"][clause]["body"]["arguments"][0]["value"]
        ["target"]
        .clone();
    let registration = package["semantic_graph"]["nodes"][compensation]["body"]["arguments"][5]
        ["value"]["members"][0]["members"][0]["value"]["target"]
        .clone();
    let recovery = package["semantic_graph"]["nodes"][compensation]["body"]["arguments"][12]
        ["value"]["members"][0]["value"]["target"]
        .clone();
    package["semantic_graph"]["nodes"][compensation]["body"]["arguments"][10]["value"]["members"]
        [2]["value"] = json!({"term": "reference", "target": registration});
    package["semantic_graph"]["nodes"][compensation]["body"]["arguments"][12]["value"]["members"]
        [1]["value"] = json!({"term": "reference", "target": recovery});
    settle(&mut package);
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));

    package["semantic_graph"]["nodes"][compensation]["body"]["arguments"][10]["value"]["members"]
        [2]["value"] = json!({"term": "reference", "target": over});
    settle(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("retry relation must bar over");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::IllTyped);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::OperatorIneligible)
    );
    assert_eq!(
        refusal.protocol_detail.as_deref(),
        Some(&CheckedProtocolRefusalDetail::ModelStateRead {
            slot: "retry.relation".into()
        })
    );

    package["semantic_graph"]["nodes"][compensation]["body"]["arguments"][10]["value"]["members"]
        [2]["value"] = json!({"term": "reference", "target": registration});
    package["semantic_graph"]["nodes"][compensation]["body"]["arguments"][12]["value"]["members"]
        [1]["value"] = json!({"term": "reference", "target": over});
    settle(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("recovery predicate must bar over");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::IllTyped);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::OperatorIneligible)
    );
    assert_eq!(
        refusal.protocol_detail.as_deref(),
        Some(&CheckedProtocolRefusalDetail::ModelStateRead {
            slot: "recovery.predicate".into()
        })
    );
}

fn clause_position(package: &Value) -> usize {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| {
            node["node_tag"] == "protocol" && node["semantic_form"] == "protocol_clause"
        })
        .expect("protocol clause")
}

/// Trace: FR-064-AC-3, FR-064-AC-4, FR-064-AC-6
#[trace("TC-946")]
#[test]
fn protocol_owner_edge_has_closed_wire_presence_and_local_shape() {
    let package = admitted_protocol_families();
    let position = clause_position(&package);
    let baseline = read(&package);
    assert!(
        matches!(baseline, CheckedPackageV2ReadResult::Admitted(_)),
        "baseline {baseline:?}"
    );

    let mut absent = package.clone();
    absent["semantic_graph"]["nodes"][position]
        .as_object_mut()
        .expect("clause")
        .remove("owner_model");
    assert_eq!(
        read(&absent),
        refusal(
            CheckedPackageRefusalCode::MalformedWire,
            &format!("/semantic_graph/nodes/{position}")
        )
    );

    let mut forbidden = package.clone();
    forbidden["semantic_graph"]["nodes"][0]["owner_model"] =
        package["semantic_graph"]["nodes"][position]["owner_model"].clone();
    assert_eq!(
        read(&forbidden),
        refusal(
            CheckedPackageRefusalCode::MalformedWire,
            "/semantic_graph/nodes/0/owner_model"
        )
    );

    let mut external = package;
    let package_id = external["package_id"].clone();
    let target = external["semantic_graph"]["nodes"][position]["owner_model"].clone();
    external["semantic_graph"]["nodes"][position]["owner_model"] = json!({
        "package": package_id,
        "node": target,
    });
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&external) else {
        panic!("external owner key must fail closed wire decoding");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MalformedWire);
    assert_eq!(
        refusal.path.as_ref().map(|p| p.as_str()),
        Some(format!("/semantic_graph/nodes/{position}/owner_model").as_str())
    );

    let mut package = admitted_protocol_families();
    let position = clause_position(&package);
    package["semantic_graph"]["nodes"][position]["owner_model"] = node_id(&family_key("1010"));
    settle(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("unselected model declaration must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MissingDeclaration);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::MissingSelection)
    );
    assert_eq!(
        refusal.path.as_ref().map(|p| p.as_str()),
        Some(format!("/semantic_graph/nodes/{position}/owner_model").as_str())
    );
}

/// Trace: FR-064-AC-3, FR-064-AC-4
#[trace("TC-946")]
#[test]
fn protocol_owner_dependency_precedes_key_and_target_meaning() {
    let mut package = admitted_protocol_families();
    let position = clause_position(&package);
    let owner = package["semantic_graph"]["nodes"][position]["owner_model"].clone();
    package["semantic_graph"]["nodes"][position]["dependencies"]
        .as_array_mut()
        .expect("dependencies")
        .retain(|entry| entry != &owner);
    refresh_identity(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("missing owner dependency must refuse");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
    assert_eq!(
        refusal.path.as_ref().map(|p| p.as_str()),
        Some(format!("/semantic_graph/nodes/{position}/dependencies").as_str())
    );

    for duplicate in [true, false] {
        let mut package = admitted_protocol_families();
        let position = clause_position(&package);
        let entries = package["semantic_graph"]["nodes"][position]["dependencies"]
            .as_array_mut()
            .expect("dependencies");
        if duplicate {
            entries.push(owner.clone());
        } else {
            entries.reverse();
        }
        refresh_identity(&mut package);
        let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
            panic!("duplicate or out-of-order owner dependency must refuse");
        };
        assert_eq!(
            refusal.code,
            CheckedPackageRefusalCode::InvalidSemanticGraph
        );
        assert_eq!(
            refusal.path.as_ref().map(|p| p.as_str()),
            Some(format!("/semantic_graph/nodes/{position}/dependencies").as_str())
        );
    }

    let mut package = admitted_protocol_families();
    let position = clause_position(&package);
    package["semantic_graph"]["nodes"][position]["owner_model"] = node_id(&family_key("aaaa"));
    settle(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("non-model owner target must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::IllTyped);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::OperatorIneligible)
    );
    assert_eq!(
        refusal.path.as_ref().map(|p| p.as_str()),
        Some(format!("/semantic_graph/nodes/{position}/owner_model").as_str())
    );
}

/// Trace: FR-064-AC-3, FR-064-AC-4
#[trace("TC-946")]
#[test]
fn protocol_owner_exact_body_dependencies_precede_stale_key() {
    for omit_body_target in [true, false] {
        let mut package = admitted_protocol_families();
        let position = clause_position(&package);
        let body_target = package["semantic_graph"]["nodes"][position]["body"]["arguments"][0]
            ["value"]["target"]
            .clone();
        let extra = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .map(|node| &node["node_id"])
            .find(|id| {
                !package["semantic_graph"]["nodes"][position]["dependencies"]
                    .as_array()
                    .expect("dependencies")
                    .contains(id)
            })
            .expect("unreferenced local node")
            .clone();
        let dependencies = package["semantic_graph"]["nodes"][position]["dependencies"]
            .as_array_mut()
            .expect("dependencies");
        if omit_body_target {
            dependencies.retain(|id| id != &body_target);
        } else {
            dependencies.push(extra);
            dependencies
                .sort_by(|left, right| left["digest"].as_str().cmp(&right["digest"].as_str()));
        }
        // The body's changed literal makes the retained clause key stale too.
        package["semantic_graph"]["nodes"][position]["body"]["arguments"][1]["value"]["members"]
            [0]["value"]["value"] = json!("changed");
        refresh_identity(&mut package);
        let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
            panic!("exact body dependencies must refuse before the stale key");
        };
        assert_eq!(
            refusal.code,
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            "omit_body_target={omit_body_target}"
        );
        assert_eq!(
            refusal.path.as_ref().map(|path| path.as_str()),
            Some(format!("/semantic_graph/nodes/{position}/dependencies").as_str())
        );
    }
}

/// Trace: FR-064-AC-2, FR-064-AC-4
#[trace("TC-946")]
#[test]
fn protocol_owner_retarget_with_retained_key_refuses_before_target_meaning() {
    let mut package = admitted_protocol_families();
    let position = clause_position(&package);
    let old_key = package["semantic_graph"]["nodes"][position]["node_id"]["digest"]
        .as_str()
        .expect("old key")
        .to_owned();
    package["semantic_graph"]["nodes"][position]["owner_model"] = node_id(&family_key("aaaa"));
    settle(&mut package);
    let new_key = package["semantic_graph"]["nodes"][position]["node_id"]["digest"]
        .as_str()
        .expect("new key")
        .to_owned();
    assert_ne!(new_key, old_key);
    rename_node(&mut package, &new_key, &old_key);
    refresh_identity(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("retained clause key must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::StaleNodeKey)
    );
    assert_eq!(
        refusal.path.as_ref().map(|p| p.as_str()),
        Some(format!("/semantic_graph/nodes/{position}/node_id").as_str())
    );
}

/// Trace: FR-064-AC-2, FR-064-AC-4
#[trace("TC-946")]
#[test]
fn protocol_owner_retarget_to_second_selected_model_changes_package_identity() {
    let mut package = admitted_protocol_families();
    let position = clause_position(&package);
    let old_key = package["semantic_graph"]["nodes"][position]["node_id"].clone();
    let old_package = package["package_id"].clone();
    let second_model = selected_interface_model_node();
    let second_key = second_model["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(second_model);
    package["semantic_graph"]["nodes"][position]["owner_model"] = second_key.clone();
    settle(&mut package);
    assert_ne!(
        package["semantic_graph"]["nodes"][position]["node_id"],
        old_key
    );
    assert_ne!(package["package_id"], old_package);
    let result = read(&package);
    let CheckedPackageV2ReadResult::Admitted(admitted) = result else {
        panic!(
            "exact second selected model owner must admit without over-type inference: {result:?}"
        );
    };
    let clause = admitted
        .graph()
        .nodes
        .iter()
        .find(|node| {
            node.node_tag.as_ref() == "protocol" && node.semantic_form.as_ref() == "protocol_clause"
        })
        .expect("clause");
    assert_eq!(
        clause.owner_model.as_ref(),
        Some(&serde_json::from_value(second_key).expect("node id"))
    );
}

fn append_duplicate_clause(package: &mut Value, body: Value) -> (usize, String) {
    let original = clause_position(package);
    let dependencies = package["semantic_graph"]["nodes"][original]["dependencies"]
        .as_array()
        .expect("dependencies")
        .iter()
        .map(|id| id["digest"].as_str().expect("digest").to_owned())
        .collect::<Vec<_>>();
    let duplicate = protocol_named_node(
        "protocol_clause",
        &["Example", "OrderFlow"],
        body,
        &dependencies,
    );
    let key = duplicate["node_id"]["digest"]
        .as_str()
        .expect("digest")
        .to_owned();
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(duplicate);
    rebuild_source_map(package);
    refresh_identity(package);
    (position, key)
}

fn clause_parameter(package: &mut Value, name: &str, level: &str) -> Value {
    let over = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| {
            node["node_tag"] == "value"
                && node["semantic_form"] == "parameter"
                && node["body"]["members"][0]["value"]["value"] == "over"
        })
        .expect("over parameter")
        .clone();
    let mut parameter = over;
    parameter["body"]["members"][0]["value"]["value"] = json!(name);
    parameter["body"]["members"][1]["value"]["value"] = json!(level);
    let key = structural_key(
        "value",
        "parameter",
        Some(&family_key("aaaa")),
        &parameter["body"],
    );
    parameter["node_id"] = node_id(&key);
    let target = parameter["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(parameter);
    target
}

fn relevel_parameter(package: &mut Value, target: &Value, level: usize) -> Value {
    let mut parameter = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_id"] == *target)
        .expect("parameter target")
        .clone();
    assert_eq!(parameter["semantic_form"], "parameter");
    parameter["body"]["members"][1]["value"]["value"] = json!(level.to_string());
    let semantic_type = parameter["semantic_type"]["digest"]
        .as_str()
        .expect("parameter type")
        .to_owned();
    let key = structural_key(
        "value",
        "parameter",
        Some(&semantic_type),
        &parameter["body"],
    );
    parameter["node_id"] = node_id(&key);
    let target = parameter["node_id"].clone();
    if package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .any(|node| node["node_id"] == target)
    {
        return target;
    }
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(parameter);
    target
}

fn relevel_finish_binder(package: &mut Value, level: usize) {
    let finish = protocol_position(package, "obligation");
    let target = package["semantic_graph"]["nodes"][finish]["body"]["arguments"][0]["value"]
        ["target"]
        .clone();
    let replacement = relevel_parameter(package, &target, level);
    package["semantic_graph"]["nodes"][finish]["body"]["arguments"][0]["value"]["target"] =
        replacement;
}

fn temporal_clause(package: &Value) -> Value {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "temporal" && node["semantic_form"] == "temporal_clause")
        .expect("temporal clause")
        .clone()
}

fn require_temporal(package: &mut Value, keys: &[Value]) {
    let clause = clause_position(package);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][7]["value"]["members"] =
        Value::Array(
            keys.iter()
                .map(|key| json!({"term": "reference", "target": key}))
                .collect(),
        );
    settle(package);
}

/// Trace: FR-038-AC-324
#[trace("TC-048")]
#[test]
fn activation_relation_has_exact_origin_and_each_rows_without_extras() {
    let mut package = v2_all_families();
    let clause = clause_position(&package);
    let temporal = temporal_clause(&package)["node_id"].clone();
    let empty = match read(&package) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("base protocol should admit: {other:?}"),
    };
    assert!(empty.protocol_activation_relation().is_empty());

    require_temporal(&mut package, std::slice::from_ref(&temporal));
    let clause_key = package["semantic_graph"]["nodes"][clause]["node_id"]["digest"]
        .as_str()
        .expect("clause key");
    let expected = vec![CheckedProtocolActivationRow {
        clause: serde_json::from_value(node_id(clause_key)).expect("clause key"),
        activation: CheckedProtocolActivation::Origin,
        required_temporal: serde_json::from_value(temporal.clone()).expect("temporal key"),
    }];
    let admitted = match read(&package) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("origin protocol should admit: {other:?}"),
    };
    let before = admitted.clone();
    assert_eq!(admitted.protocol_activation_relation(), expected);
    assert_eq!(admitted.protocol_activation_relation(), expected);
    assert_eq!(*admitted, *before);
    let equal = match read(&package) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("equal protocol should admit: {other:?}"),
    };
    assert_eq!(equal.protocol_activation_relation(), expected);

    let trigger = clause_parameter(&mut package, "opened", "1");
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][1]["value"]["members"] = json!([
        {"term": "binding", "name": "kind", "value": {"term": "literal", "type": node_id(&family_key("a1a1")), "value_kind": "text", "value": "each"}},
        {"term": "binding", "name": "trigger", "value": {"term": "reference", "target": trigger}},
    ]);
    relevel_finish_binder(&mut package, 2);
    settle(&mut package);
    let admitted = match read(&package) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("each protocol should admit: {other:?}"),
    };
    assert_eq!(
        admitted.protocol_activation_relation(),
        vec![CheckedProtocolActivationRow {
            clause: serde_json::from_value(
                package["semantic_graph"]["nodes"][clause]["node_id"].clone()
            )
            .expect("clause key"),
            activation: CheckedProtocolActivation::Each {
                trigger: serde_json::from_value(trigger).expect("trigger key")
            },
            required_temporal: serde_json::from_value(temporal).expect("temporal key"),
        }]
    );
}

/// Trace: FR-038-AC-324
#[trace("TC-048")]
#[test]
fn activation_relation_orders_distinct_requirements_and_ignores_other_temporal_clauses() {
    let mut package = v2_all_families();
    let first = temporal_clause(&package)["node_id"].clone();
    let mut second = temporal_clause(&package);
    second["body"]["arguments"][1]["value"] = json!("other-clause");
    let second_key = application_node_key(
        "temporal",
        "temporal_clause",
        &family_key("aaaa"),
        &second["body"],
    );
    second["node_id"] = node_id(&second_key);
    let second_id = second["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(second);
    settle(&mut package);

    require_temporal(&mut package, std::slice::from_ref(&first));
    let admitted = match read(&package) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("unrelated temporal clause should admit: {other:?}"),
    };
    assert_eq!(admitted.protocol_activation_relation().len(), 1);
    assert_eq!(
        admitted.protocol_activation_relation()[0].required_temporal,
        serde_json::from_value(first.clone()).expect("first key")
    );

    let mut keys = [first, second_id];
    keys.sort_by(|left, right| left["digest"].as_str().cmp(&right["digest"].as_str()));
    require_temporal(&mut package, &keys);
    let admitted = match read(&package) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("two requirements should admit: {other:?}"),
    };
    let rows = admitted.protocol_activation_relation();
    assert_eq!(rows.len(), 2);
    assert_eq!(
        rows.iter()
            .map(|row| row.required_temporal.digest.as_ref())
            .collect::<Vec<_>>(),
        keys.iter()
            .map(|key| key["digest"].as_str().expect("digest"))
            .collect::<Vec<_>>()
    );
    assert_eq!(rows[0].clause, rows[1].clause);
    assert_eq!(rows[0].activation, CheckedProtocolActivation::Origin);
    assert_eq!(rows[1].activation, CheckedProtocolActivation::Origin);
}

/// Trace: FR-038-AC-324
#[trace("TC-048")]
#[test]
fn activation_relation_orders_rows_across_local_protocol_clauses() {
    let mut package = v2_all_families();
    let clause = clause_position(&package);
    let temporal = temporal_clause(&package)["node_id"].clone();
    require_temporal(&mut package, std::slice::from_ref(&temporal));

    let clone_member = |package: &Value, form: &str, name: &str| {
        let original = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .find(|node| node["node_tag"] == "protocol" && node["semantic_form"] == form)
            .expect("protocol member");
        let dependencies = original["dependencies"]
            .as_array()
            .expect("dependencies")
            .iter()
            .map(|id| id["digest"].as_str().expect("digest").to_owned())
            .collect::<Vec<_>>();
        protocol_named_node(
            form,
            &["Example", "OtherFlow", name, "Main"],
            original["body"].clone(),
            &dependencies,
        )
    };
    let run = clone_member(&package, "control", "run");
    let finish = {
        let original = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .find(|node| node["node_tag"] == "protocol" && node["semantic_form"] == "obligation")
            .expect("finish");
        let dependencies = original["dependencies"]
            .as_array()
            .expect("dependencies")
            .iter()
            .map(|id| id["digest"].as_str().expect("digest").to_owned())
            .collect::<Vec<_>>();
        protocol_named_node(
            "obligation",
            &["Example", "OtherFlow", "finish", "Closed"],
            original["body"].clone(),
            &dependencies,
        )
    };
    let run_id = run["node_id"].clone();
    let finish_id = finish["node_id"].clone();
    let mut body = package["semantic_graph"]["nodes"][clause]["body"].clone();
    body["arguments"][11]["value"]["target"] = run_id.clone();
    body["arguments"][12]["value"]["target"] = finish_id.clone();
    let over = body["arguments"][0]["value"]["target"]["digest"]
        .as_str()
        .expect("over");
    let dependencies = [
        over,
        run_id["digest"].as_str().expect("run"),
        finish_id["digest"].as_str().expect("finish"),
        temporal["digest"].as_str().expect("temporal"),
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let other = protocol_named_node(
        "protocol_clause",
        &["Example", "OtherFlow"],
        body,
        &dependencies,
    );
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend([run, finish, other]);
    settle(&mut package);
    let admitted = match read(&package) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("two local protocols should admit: {other:?}"),
    };
    let rows = admitted.protocol_activation_relation();
    assert_eq!(rows.len(), 2);
    assert!(rows[0].clause.digest < rows[1].clause.digest);
    assert_eq!(rows[0].required_temporal, rows[1].required_temporal);
}

/// Trace: FR-038-AC-284, FR-038-AC-285
#[trace("TC-048")]
#[test]
fn clause_references_check_dependencies_before_target_forms() {
    let mut package = v2_all_families();
    let clause = clause_position(&package);
    let root = package["semantic_graph"]["nodes"][clause]["body"]["arguments"][11]["value"]
        ["target"]
        .clone();
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][0]["value"]["target"] = root;
    let wrong_relationship = package["semantic_graph"]["nodes"][clause]["body"]["arguments"][0]
        ["value"]["target"]
        .clone();
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][4]["value"]["members"] =
        json!([{"term": "reference", "target": wrong_relationship}]);
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][clause]["node_id"]["digest"]
        .as_str()
        .expect("clause key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidModelBinding,
            &format!("/semantic_graph/nodes/{clause}/body/arguments/0/value/target"),
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            key,
        )
    );
}

/// Trace: FR-038-AC-284, FR-038-AC-285
#[trace("TC-048")]
#[test]
fn clause_relationship_wrong_export_has_distinct_cause() {
    let mut package = v2_all_families();
    let clause = clause_position(&package);
    let over = package["semantic_graph"]["nodes"][clause]["body"]["arguments"][0]["value"]
        ["target"]
        .clone();
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][4]["value"]["members"] =
        json!([{"term": "reference", "target": over}]);
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][clause]["node_id"]["digest"]
        .as_str()
        .expect("clause key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidModelBinding,
            &format!("/semantic_graph/nodes/{clause}/body/arguments/4/value/members/0/target"),
            Some(CheckedPackageRefusalCause::WrongExport),
            key,
        )
    );
}

/// Trace: FR-038-AC-284, FR-038-AC-291, FR-038-AC-324
#[trace("TC-048")]
#[test]
fn clause_each_trigger_admits_and_checks_parameter_level() {
    let mut package = v2_all_families();
    let trigger = clause_parameter(&mut package, "opened", "1");
    let clause = clause_position(&package);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][1]["value"]["members"] = json!([
        {"term": "binding", "name": "kind", "value": {"term": "literal", "type": node_id(&family_key("a1a1")), "value_kind": "text", "value": "each"}},
        {"term": "binding", "name": "trigger", "value": {"term": "reference", "target": trigger}},
    ]);
    relevel_finish_binder(&mut package, 2);
    settle(&mut package);
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    let trigger_position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_id"] == trigger)
        .expect("trigger binder");
    package["semantic_graph"]["nodes"][trigger_position]["body"]["members"][1]["value"]["value"] =
        json!("2");
    let wrong_key = structural_key(
        "value",
        "parameter",
        Some(&family_key("aaaa")),
        &package["semantic_graph"]["nodes"][trigger_position]["body"],
    );
    package["semantic_graph"]["nodes"][trigger_position]["node_id"] = node_id(&wrong_key);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][1]["value"]["members"][1]
        ["value"]["target"] = node_id(&wrong_key);
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][trigger_position]["node_id"]["digest"]
        .as_str()
        .expect("clause key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            &format!("/semantic_graph/nodes/{trigger_position}"),
            None,
            key,
        )
    );
}

/// Trace: FR-038-AC-284, FR-038-AC-324
#[trace("TC-048")]
#[test]
fn clause_each_trigger_requires_declared_dependency() {
    let mut package = v2_all_families();
    let trigger = clause_parameter(&mut package, "opened", "1");
    let clause = clause_position(&package);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][1]["value"]["members"] = json!([
        {"term": "binding", "name": "kind", "value": {"term": "literal", "type": node_id(&family_key("a1a1")), "value_kind": "text", "value": "each"}},
        {"term": "binding", "name": "trigger", "value": {"term": "reference", "target": trigger}},
    ]);
    settle(&mut package);
    package["semantic_graph"]["nodes"][clause]["dependencies"]
        .as_array_mut()
        .expect("dependencies")
        .retain(|dependency| dependency != &trigger);
    refresh_identity(&mut package);
    let key = package["semantic_graph"]["nodes"][clause]["node_id"]["digest"]
        .as_str()
        .expect("clause key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::MissingDeclaration,
            &format!(
                "/semantic_graph/nodes/{clause}/body/arguments/1/value/members/1/value/target"
            ),
            Some(CheckedPackageRefusalCause::MissingName),
            key,
        )
    );
}

/// Trace: FR-038-AC-284
#[trace("TC-048")]
#[test]
fn clause_each_guard_requires_boolean_root() {
    let mut package = v2_all_families();
    let trigger = clause_parameter(&mut package, "opened", "1");
    let clause = clause_position(&package);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][1]["value"]["members"] = json!([
        {"term": "binding", "name": "kind", "value": {"term": "literal", "type": node_id(&family_key("a1a1")), "value_kind": "text", "value": "each"}},
        {"term": "binding", "name": "trigger", "value": {"term": "reference", "target": trigger}},
        {"term": "binding", "name": "guard", "value": {"term": "reference", "target": node_id(&family_key("a1a1"))}},
    ]);
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][clause]["node_id"]["digest"]
        .as_str()
        .expect("clause key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::IllTyped,
            &format!("/semantic_graph/nodes/{clause}/body/arguments/1/value/members/2/value"),
            Some(CheckedPackageRefusalCause::NonBooleanRoot),
            key,
        )
    );
}

/// Trace: FR-038-AC-284
#[trace("TC-048")]
#[test]
fn clause_capture_checks_binder_form_and_ordered_level() {
    let mut package = v2_all_families();
    let binder = clause_parameter(&mut package, "captured", "1");
    let clause = clause_position(&package);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][2]["value"]["members"] = json!([{"term": "aggregate", "members": [
        {"term": "binding", "name": "binder", "value": {"term": "reference", "target": binder}},
        {"term": "binding", "name": "initializer", "value": {"term": "literal", "type": node_id(&family_key("aaaa")), "value_kind": "boolean", "value": true}},
    ]}]);
    relevel_finish_binder(&mut package, 2);
    settle(&mut package);
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    let root = package["semantic_graph"]["nodes"][clause]["body"]["arguments"][11]["value"]
        ["target"]
        .clone();
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][2]["value"]["members"][0]
        ["members"][0]["value"]["target"] = root;
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][clause]["node_id"]["digest"]
        .as_str()
        .expect("clause key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidModelBinding,
            &format!(
                "/semantic_graph/nodes/{clause}/body/arguments/2/value/members/0/members/0/value/target"
            ),
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            key,
        )
    );
}

/// Trace: FR-038-AC-284, FR-038-AC-291
#[trace("TC-048")]
#[test]
fn clause_capture_order_sets_each_binder_level() {
    let mut package = v2_all_families();
    let first = clause_parameter(&mut package, "first", "1");
    let second = clause_parameter(&mut package, "second", "2");
    let clause = clause_position(&package);
    let capture = |binder: Value| {
        json!({"term": "aggregate", "members": [
            {"term": "binding", "name": "binder", "value": {"term": "reference", "target": binder}},
            {"term": "binding", "name": "initializer", "value": {"term": "literal", "type": node_id(&family_key("aaaa")), "value_kind": "boolean", "value": true}},
        ]})
    };
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][2]["value"]["members"] =
        json!([capture(first), capture(second)]);
    relevel_finish_binder(&mut package, 3);
    settle(&mut package);
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][2]["value"]["members"]
        .as_array_mut()
        .expect("captures")
        .swap(0, 1);
    settle(&mut package);
    let second_position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["body"]["members"][0]["value"]["value"] == "second")
        .expect("second binder");
    let key = package["semantic_graph"]["nodes"][second_position]["node_id"]["digest"]
        .as_str()
        .expect("second binder key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            &format!("/semantic_graph/nodes/{second_position}"),
            None,
            key,
        )
    );
}

/// Trace: FR-038-AC-291
#[trace("TC-048")]
#[test]
fn clause_rejects_second_capture_with_readable_name() {
    let mut package = v2_all_families();
    let first = clause_parameter(&mut package, "captured", "1");
    let second = clause_parameter(&mut package, "captured", "2");
    let clause = clause_position(&package);
    let capture = |binder: Value| {
        json!({"term": "aggregate", "members": [
            {"term": "binding", "name": "binder", "value": {"term": "reference", "target": binder}},
            {"term": "binding", "name": "initializer", "value": {"term": "literal", "type": node_id(&family_key("aaaa")), "value_kind": "boolean", "value": true}},
        ]})
    };
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][2]["value"]["members"] =
        json!([capture(first), capture(second)]);
    settle(&mut package);
    let second_position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
    let key = package["semantic_graph"]["nodes"][second_position]["node_id"]["digest"]
        .as_str()
        .expect("second binder key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::AmbiguousDeclaration,
            &format!("/semantic_graph/nodes/{second_position}"),
            Some(CheckedPackageRefusalCause::AmbiguousName),
            key,
        )
    );
}

/// Trace: FR-038-AC-284, FR-038-AC-324
#[trace("TC-048")]
#[test]
fn clause_required_temporal_target_is_checked_after_membership() {
    let mut package = v2_all_families();
    let temporal = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "temporal" && node["semantic_form"] == "temporal_clause")
        .expect("temporal clause")["node_id"]
        .clone();
    let clause = clause_position(&package);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][7]["value"]["members"] =
        json!([{"term": "reference", "target": temporal}]);
    settle(&mut package);
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][7]["value"]["members"] = json!([
        {"term": "reference", "target": temporal},
        {"term": "reference", "target": temporal},
    ]);
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][clause]["node_id"]["digest"]
        .as_str()
        .expect("clause key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            &format!("/semantic_graph/nodes/{clause}"),
            None,
            key,
        )
    );
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][7]["value"]["members"] =
        json!([{"term": "reference", "target": temporal}]);
    settle(&mut package);
    package["semantic_graph"]["nodes"][clause]["dependencies"]
        .as_array_mut()
        .expect("dependencies")
        .retain(|dependency| dependency != &temporal);
    refresh_identity(&mut package);
    let key = package["semantic_graph"]["nodes"][clause]["node_id"]["digest"]
        .as_str()
        .expect("clause key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::MissingDeclaration,
            &format!("/semantic_graph/nodes/{clause}/body/arguments/7/value/members/0/target"),
            Some(CheckedPackageRefusalCause::MissingName),
            key,
        )
    );
}

#[trace("TC-048")]
#[test]
fn protocol_clause_requires_the_selected_profile_law_in_the_public_reader() {
    let mut package = v2_all_families();
    let position = clause_position(&package);
    package["semantic_graph"]["nodes"][position]["body"]["operation"]["laws"] = json!([]);
    rekey_application_node(&mut package, position);
    refresh_identity(&mut package);
    let key = package["semantic_graph"]["nodes"][position]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidPackage,
            &format!("/semantic_graph/nodes/{position}/body/operation/laws"),
            Some(CheckedPackageRefusalCause::OperationLawMissing),
            key,
        )
    );
}

#[trace("TC-048")]
#[test]
fn malformed_protocol_enum_names_the_offending_term() {
    let mut package = v2_all_families();
    let position = clause_position(&package);
    package["semantic_graph"]["nodes"][position]["body"]["arguments"][1]["value"]["members"][0]
        ["value"]["value"] = json!("unknown-activation");
    assert_eq!(
        read(&package),
        refusal(
            CheckedPackageRefusalCode::MalformedWire,
            &format!("/semantic_graph/nodes/{position}/body/arguments/1/value/members/0/value"),
        )
    );

    let mut package = v2_all_families();
    let semantic_type = package["semantic_graph"]["nodes"][position]["semantic_type"].clone();
    package["semantic_graph"]["nodes"][position]["body"]["arguments"][10]["value"]["members"] = json!([{
        "term": "literal", "type": semantic_type,
        "value_kind": "text", "value": "unknown-scheduling"
    }]);
    assert_eq!(
        read(&package),
        refusal(
            CheckedPackageRefusalCode::MalformedWire,
            &format!("/semantic_graph/nodes/{position}/body/arguments/10/value/members/0"),
        )
    );
}

#[trace("TC-048")]
#[test]
fn protocol_operator_refusal_precedes_duplicate_declaration() {
    let mut package = v2_all_families();
    let original = clause_position(&package);
    let mut body = package["semantic_graph"]["nodes"][original]["body"].clone();
    body["operator"] = json!("call");
    let (position, key) = append_duplicate_clause(&mut package, body);
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::IllTyped,
            &format!("/semantic_graph/nodes/{position}/body"),
            Some(CheckedPackageRefusalCause::OperatorIneligible),
            &key,
        )
    );
}

#[trace("TC-048")]
#[test]
fn valid_protocol_operators_still_refuse_duplicate_declarations() {
    let mut package = v2_all_families();
    let original = clause_position(&package);
    let original_key = package["semantic_graph"]["nodes"][original]["node_id"]["digest"]
        .as_str()
        .expect("digest")
        .to_owned();
    let mut body = package["semantic_graph"]["nodes"][original]["body"].clone();
    let semantic_type = package["semantic_graph"]["nodes"][original]["semantic_type"].clone();
    body["arguments"][10]["value"]["members"] = json!([{
        "term": "literal", "type": semantic_type,
        "value_kind": "text", "value": "adversarial"
    }]);
    let (duplicate_position, duplicate_key) = append_duplicate_clause(&mut package, body);
    let (position, key) = if original_key < duplicate_key {
        (duplicate_position, duplicate_key)
    } else {
        (original, original_key)
    };
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::AmbiguousDeclaration,
            &format!("/semantic_graph/nodes/{position}/declaration/qualified_name"),
            Some(CheckedPackageRefusalCause::AmbiguousName),
            &key,
        )
    );
}

fn protocol_position(package: &Value, form: &str) -> usize {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_tag"] == "protocol" && node["semantic_form"] == form)
        .expect("protocol node")
}

#[trace("TC-048")]
#[test]
fn protocol_role_and_root_control_can_share_a_name() {
    let mut package = v2_all_families();
    let mut role = protocol_role_node();
    role["declaration"]["qualified_name"][3] = json!("Main");
    let role_key = role["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(role);
    let clause = clause_position(&package);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][3]["value"]["members"] =
        json!([{"term": "reference", "target": role_key}]);
    settle(&mut package);
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

#[trace("TC-048")]
#[test]
fn protocol_listed_role_must_use_its_clause_and_fixed_segment() {
    for declaration in [
        json!(["Other", "Protocol", "role", "Worker"]),
        json!(["Example", "OrderFlow", "channel", "Worker"]),
    ] {
        let mut package = v2_all_families();
        let mut role = protocol_role_node();
        role["declaration"]["qualified_name"] = declaration;
        let key = role["node_id"].clone();
        let position = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .len();
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(role);
        let clause = clause_position(&package);
        package["semantic_graph"]["nodes"][clause]["body"]["arguments"][3]["value"]["members"] =
            json!([{"term": "reference", "target": key}]);
        settle(&mut package);
        let key = package["semantic_graph"]["nodes"][position]["node_id"]["digest"]
            .as_str()
            .expect("digest");
        assert_eq!(
            read(&package),
            refusal_at(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                &format!("/semantic_graph/nodes/{position}"),
                None,
                key,
            )
        );
    }
}

#[trace("TC-048")]
#[test]
fn nested_control_uses_its_holder_path() {
    let mut package = v2_all_families();
    let root = protocol_position(&package, "control");
    let body = package["semantic_graph"]["nodes"][root]["body"].clone();
    let child = protocol_named_node(
        "control",
        &["Example", "OrderFlow", "run", "Done"],
        body,
        &[],
    );
    let child_key = child["node_id"].clone();
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(child);
    package["semantic_graph"]["nodes"][root]["body"]["arguments"][0]["value"]["members"] =
        json!([{"term": "reference", "target": child_key}]);
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][position]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            &format!("/semantic_graph/nodes/{position}"),
            None,
            key,
        )
    );

    package["semantic_graph"]["nodes"][position]["declaration"]["qualified_name"] =
        json!(["Other", "Protocol", "run", "Main", "Done"]);
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][position]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            &format!("/semantic_graph/nodes/{position}"),
            None,
            key,
        )
    );

    package["semantic_graph"]["nodes"][position]["declaration"]["qualified_name"] =
        json!(["Example", "OrderFlow", "run", "Main", "Done"]);
    settle(&mut package);
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

fn protocol_holder_case(operator: &str) -> (Value, Vec<usize>) {
    protocol_holder_case_on(v2_all_families(), operator)
}

fn protocol_holder_case_on(mut package: Value, operator: &str) -> (Value, Vec<usize>) {
    let root = protocol_position(&package, "control");
    let leaf_body = package["semantic_graph"]["nodes"][root]["body"].clone();
    let role = protocol_role_node();
    let role_target = role["node_id"].clone();
    let binder_target = package["semantic_graph"]["nodes"]
        [protocol_position(&package, "obligation")]["body"]["arguments"][0]["value"]["target"]
        .clone();
    let awaited_binder = if operator == "await" {
        let matched = clause_parameter(&mut package, "matched", "2");
        let finished = clause_parameter(&mut package, "finished_after_await", "2");
        let finish = protocol_position(&package, "obligation");
        package["semantic_graph"]["nodes"][finish]["body"]["arguments"][0]["value"]["target"] =
            finished;
        Some(matched)
    } else {
        None
    };
    let names: &[&str] = if operator == "await" {
        &["Event", "Then", "Timeout"]
    } else {
        &["Left", "Right"]
    };
    let mut positions = Vec::new();
    let mut targets = Vec::new();
    for (ordinal, name) in names.iter().enumerate() {
        let mut path = vec!["Example", "OrderFlow", "run", "Main", "Holder"];
        path.push(name);
        let mut child_body = leaf_body.clone();
        if ordinal == 0 && operator == "repeat" {
            child_body["operation"]["member"]["operator"] = json!("fence");
            child_body["arguments"] = json!([{"term": "binding", "name": "ordering", "value": {
                "term": "literal", "type": node_id(&family_key("a1a1")),
                "value_kind": "text", "value": "seq_cst"
            }}]);
        }
        if ordinal == 0 && operator == "await" {
            child_body["operation"]["member"]["operator"] = json!("event");
            child_body["arguments"] = json!([
                {"term": "binding", "name": "role", "value": {"term": "reference", "target": role_target}},
                {"term": "binding", "name": "compensation", "value": {"term": "aggregate", "members": []}},
                {"term": "binding", "name": "binder", "value": {"term": "reference", "target": awaited_binder.as_ref().expect("awaited binder")}},
                {"term": "binding", "name": "related", "value": {"term": "aggregate", "members": []}},
                {"term": "binding", "name": "constraint", "value": {"term": "literal", "type": node_id(&family_key("aaaa")), "value_kind": "boolean", "value": true}}
            ]);
        }
        let node = protocol_named_node("control", &path, child_body, &[]);
        targets.push(node["node_id"].clone());
        positions.push(
            package["semantic_graph"]["nodes"]
                .as_array()
                .expect("nodes")
                .len(),
        );
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(node);
    }
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(role);
    let clause = clause_position(&package);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][3]["value"]["members"] =
        json!([{"term": "reference", "target": role_target}]);
    let reference = |target: &Value| json!({"term": "reference", "target": target});
    let binding =
        |name: &str, value: Value| json!({"term": "binding", "name": name, "value": value});
    let group = |members: Vec<Value>| json!({"term": "aggregate", "members": members});
    let text = |value: &str| {
        json!({
            "term": "literal", "type": node_id(&family_key("a1a1")),
            "value_kind": "text", "value": value
        })
    };
    let integer = |value: &str| {
        json!({
            "term": "literal", "type": node_id(&family_key("a3a3")),
            "value_kind": "integer", "value": value
        })
    };
    let boolean = || {
        json!({
            "term": "literal", "type": node_id(&family_key("aaaa")),
            "value_kind": "boolean", "value": true
        })
    };
    let mut body = leaf_body;
    body["operation"]["member"]["operator"] = json!(operator);
    body["arguments"] = match operator {
        "choice" => json!([
            binding("owner", reference(&role_target)),
            binding("visible", group(vec![])),
            binding(
                "cases",
                group(vec![
                    group(vec![
                        binding("label", text("Left")),
                        binding("guard", boolean()),
                        binding("body", reference(&targets[0]))
                    ]),
                    group(vec![
                        binding("label", text("Right")),
                        binding("guard", boolean()),
                        binding("body", reference(&targets[1]))
                    ]),
                ])
            ),
        ]),
        "parallel" => json!([
            binding("memory", group(vec![])),
            binding(
                "branches",
                group(vec![
                    group(vec![
                        binding("label", text("Left")),
                        binding("body", reference(&targets[0]))
                    ]),
                    group(vec![
                        binding("label", text("Right")),
                        binding("body", reference(&targets[1]))
                    ]),
                ])
            ),
            binding("join", group(vec![binding("policy", text("all"))])),
            binding("joined", group(vec![text("Left"), text("Right")])),
            binding("outstanding", group(vec![])),
        ]),
        "repeat" => json!([
            binding("owner", reference(&role_target)),
            binding("visible", group(vec![])),
            binding("maximum", group(vec![integer("1")])),
            binding("proof", group(vec![])),
            binding("guard", boolean()),
            binding("body", reference(&targets[0])),
            binding("exhausted", group(vec![reference(&targets[1])])),
        ]),
        "await" => json!([
            binding(
                "after",
                group(vec![
                    binding("kind", text("event")),
                    binding("target", reference(&targets[0]))
                ])
            ),
            binding("clock", text("clock")),
            binding(
                "within",
                group(vec![
                    binding("lower", integer("0")),
                    binding("upper", integer("1"))
                ])
            ),
            binding("event", reference(&targets[0])),
            binding("then", reference(&targets[1])),
            binding("timeout", reference(&targets[2])),
        ]),
        other => panic!("unknown holder fixture: {other}"),
    };
    let anchor = if operator == "await" {
        let mut anchor_body = package["semantic_graph"]["nodes"][positions[0]]["body"].clone();
        anchor_body["arguments"][2]["value"]["target"] = binder_target;
        let anchor = protocol_named_node(
            "control",
            &["Example", "OrderFlow", "run", "Main", "Anchor"],
            anchor_body,
            &[],
        );
        let target = anchor["node_id"].clone();
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(anchor);
        body["arguments"][0]["value"]["members"][1]["value"]["target"] = target.clone();
        Some(target)
    } else {
        None
    };
    let holder = protocol_named_node(
        "control",
        &["Example", "OrderFlow", "run", "Main", "Holder"],
        body,
        &[],
    );
    let holder_key = holder["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(holder);
    package["semantic_graph"]["nodes"][root]["body"]["arguments"][0]["value"]["members"] =
        if let Some(anchor) = anchor {
            json!([reference(&anchor), reference(&holder_key)])
        } else {
            json!([reference(&holder_key)])
        };
    settle(&mut package);
    (package, positions)
}

fn holder_position(package: &Value) -> usize {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| {
            node["declaration"]["qualified_name"]
                == json!(["Example", "OrderFlow", "run", "Main", "Holder"])
        })
        .expect("holder")
}

fn holder_refusal(
    package: &Value,
    code: CheckedPackageRefusalCode,
    cause: Option<CheckedPackageRefusalCause>,
    path: &str,
) {
    let position = holder_position(package);
    let key = package["semantic_graph"]["nodes"][position]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    assert_eq!(
        read(package),
        refusal_at(
            code,
            &format!("/semantic_graph/nodes/{position}{path}"),
            cause,
            key
        )
    );
}

/// Trace: FR-038-AC-284, FR-038-AC-288
#[trace("TC-048")]
#[test]
fn choice_labels_precede_guard_type_and_parallel_join_is_checked() {
    let (mut package, _) = protocol_holder_case("choice");
    let holder = holder_position(&package);
    let cases =
        &mut package["semantic_graph"]["nodes"][holder]["body"]["arguments"][2]["value"]["members"];
    cases[1]["members"][0]["value"]["value"] = json!("Left");
    cases[0]["members"][1]["value"]["value_kind"] = json!("text");
    cases[0]["members"][1]["value"]["value"] = json!("false");
    settle(&mut package);
    holder_refusal(
        &package,
        CheckedPackageRefusalCode::AmbiguousDeclaration,
        Some(CheckedPackageRefusalCause::AmbiguousName),
        "",
    );

    let (mut package, _) = protocol_holder_case("parallel");
    let holder = holder_position(&package);
    package["semantic_graph"]["nodes"][holder]["body"]["arguments"][3]["value"]["members"]
        .as_array_mut()
        .expect("joined")
        .pop();
    settle(&mut package);
    holder_refusal(
        &package,
        CheckedPackageRefusalCode::InvalidPackage,
        Some(CheckedPackageRefusalCause::InvalidValue),
        "/body/arguments/3/value",
    );
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn control_choice_guard_and_parallel_join_labels_refuse_at_the_member() {
    let (mut package, _) = protocol_holder_case("choice");
    let holder = holder_position(&package);
    let guard = &mut package["semantic_graph"]["nodes"][holder]["body"]["arguments"][2]["value"]
        ["members"][0]["members"][1]["value"];
    guard["value_kind"] = json!("text");
    guard["value"] = json!("false");
    settle(&mut package);
    holder_refusal(
        &package,
        CheckedPackageRefusalCode::IllTyped,
        Some(CheckedPackageRefusalCause::NonBooleanRoot),
        "/body/arguments/2/value/members/0/members/1/value",
    );

    let (mut package, _) = protocol_holder_case("parallel");
    let holder = holder_position(&package);
    package["semantic_graph"]["nodes"][holder]["body"]["arguments"][3]["value"]["members"][1]
        ["value"] = json!("Absent");
    settle(&mut package);
    holder_refusal(
        &package,
        CheckedPackageRefusalCode::MissingDeclaration,
        Some(CheckedPackageRefusalCause::MissingName),
        "/body/arguments/3/value/members/1",
    );
}

/// Trace: FR-038-AC-284, FR-038-AC-288
#[trace("TC-048")]
#[test]
fn sequence_admits_ordered_controls_and_refuses_role_child() {
    let (mut package, children) = protocol_holder_case("parallel");
    let holder = holder_position(&package);
    let targets: Vec<Value> = children
        .iter()
        .map(|&position| package["semantic_graph"]["nodes"][position]["node_id"].clone())
        .collect();
    package["semantic_graph"]["nodes"][holder]["body"]["operation"]["member"]["operator"] =
        json!("sequence");
    package["semantic_graph"]["nodes"][holder]["body"]["arguments"] = json!([{
        "term": "binding", "name": "children", "value": {
            "term": "aggregate", "members": targets.iter().map(|target| json!({"term": "reference", "target": target})).collect::<Vec<_>>()
        }
    }]);
    settle(&mut package);
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));

    let mut empty = package.clone();
    let holder = holder_position(&empty);
    empty["semantic_graph"]["nodes"][holder]["body"]["arguments"][0]["value"]["members"] =
        json!([]);
    for &child in children.iter().rev() {
        empty["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .remove(child);
    }
    settle(&mut empty);
    assert!(matches!(
        read(&empty),
        CheckedPackageV2ReadResult::Admitted(_)
    ));

    let role = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["semantic_form"] == "role")
        .expect("role")["node_id"]
        .clone();
    let holder = holder_position(&package);
    package["semantic_graph"]["nodes"][holder]["body"]["arguments"][0]["value"]["members"][0]
        ["target"] = role;
    settle(&mut package);
    holder_refusal(
        &package,
        CheckedPackageRefusalCode::InvalidModelBinding,
        Some(CheckedPackageRefusalCause::MalformedDeclaration),
        "/body/arguments/0/value/members/0/target",
    );
}

/// Trace: FR-038-AC-282, FR-038-AC-288
#[trace("TC-048")]
#[test]
fn choice_case_order_and_parallel_quorum_bounds_refuse() {
    let (mut package, _) = protocol_holder_case("choice");
    let holder = holder_position(&package);
    package["semantic_graph"]["nodes"][holder]["body"]["arguments"][2]["value"]["members"]
        .as_array_mut()
        .expect("cases")
        .pop();
    assert_eq!(
        read(&package),
        refusal(
            CheckedPackageRefusalCode::MalformedWire,
            &format!("/semantic_graph/nodes/{holder}/body")
        )
    );

    let (mut package, _) = protocol_holder_case("choice");
    let holder = holder_position(&package);
    package["semantic_graph"]["nodes"][holder]["body"]["arguments"][2]["value"]["members"]
        .as_array_mut()
        .expect("cases")
        .reverse();
    settle(&mut package);
    holder_refusal(
        &package,
        CheckedPackageRefusalCode::InvalidSemanticGraph,
        None,
        "",
    );

    let (mut package, _) = protocol_holder_case("parallel");
    let holder = holder_position(&package);
    let integer_type = node_id(&family_key("a3a3"));
    package["semantic_graph"]["nodes"][holder]["body"]["arguments"][2]["value"]["members"] = json!([
        {"term": "binding", "name": "policy", "value": {"term": "literal", "type": node_id(&family_key("a1a1")), "value_kind": "text", "value": "quorum"}},
        {"term": "binding", "name": "count", "value": {"term": "literal", "type": integer_type, "value_kind": "integer", "value": "3"}}
    ]);
    settle(&mut package);
    holder_refusal(
        &package,
        CheckedPackageRefusalCode::InvalidPackage,
        Some(CheckedPackageRefusalCause::InvalidValue),
        "/body/arguments/2/value/members/1/value",
    );
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn parallel_any_quorum_and_predicate_joins_admit() {
    for policy in ["any", "quorum", "predicate"] {
        let (mut package, _) = protocol_holder_case("parallel");
        let holder = holder_position(&package);
        let function = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .find(|node| node["node_tag"] == "function")
            .expect("function")["node_id"]
            .clone();
        let text = |value: &str| {
            json!({
                "term": "literal", "type": node_id(&family_key("a1a1")),
                "value_kind": "text", "value": value
            })
        };
        let mut fields = vec![json!({"term": "binding", "name": "policy", "value": text(policy)})];
        if policy == "quorum" {
            fields.push(json!({"term": "binding", "name": "count", "value": {
                "term": "literal", "type": node_id(&family_key("a3a3")),
                "value_kind": "integer", "value": "1"
            }}));
        }
        if policy == "predicate" {
            fields.push(json!({"term": "binding", "name": "predicate", "value": {
                "term": "reference", "target": function
            }}));
        }
        package["semantic_graph"]["nodes"][holder]["body"]["arguments"][2]["value"]["members"] =
            json!(fields);
        package["semantic_graph"]["nodes"][holder]["body"]["arguments"][3]["value"]["members"] =
            json!([text("Left")]);
        settle(&mut package);
        assert!(
            matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
            "{policy} join"
        );
    }
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn parallel_predicate_target_requires_function_form() {
    let (mut package, _) = protocol_holder_case("parallel");
    let holder = holder_position(&package);
    let role = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["semantic_form"] == "role")
        .expect("role")["node_id"]
        .clone();
    package["semantic_graph"]["nodes"][holder]["body"]["arguments"][2]["value"]["members"] = json!([
        {"term": "binding", "name": "policy", "value": {
            "term": "literal", "type": node_id(&family_key("a1a1")), "value_kind": "text", "value": "predicate"
        }},
        {"term": "binding", "name": "predicate", "value": {"term": "reference", "target": role}}
    ]);
    settle(&mut package);
    holder_refusal(
        &package,
        CheckedPackageRefusalCode::InvalidModelBinding,
        Some(CheckedPackageRefusalCause::MalformedDeclaration),
        "/body/arguments/2/value/members/1/value/target",
    );
}

/// Trace: FR-038-AC-282, FR-038-AC-288
#[trace("TC-048")]
#[test]
fn parallel_memory_and_outstanding_reject_unknown_spelling_at_wire_boundary() {
    for (argument, value) in [(0, "weak"), (4, "pause")] {
        let (mut package, _) = protocol_holder_case("parallel");
        let holder = holder_position(&package);
        package["semantic_graph"]["nodes"][holder]["body"]["arguments"][argument]["value"]
            ["members"] = json!([{
            "term": "literal", "type": node_id(&family_key("a1a1")),
            "value_kind": "text", "value": value
        }]);
        assert_eq!(
            read(&package),
            refusal(
                CheckedPackageRefusalCode::MalformedWire,
                &format!(
                    "/semantic_graph/nodes/{holder}/body/arguments/{argument}/value/members/0"
                )
            ),
            "{value}"
        );
    }
}

#[trace("TC-048")]
#[test]
fn choice_parallel_repeat_and_await_hold_controls_at_their_own_paths() {
    for operator in ["choice", "parallel", "repeat", "await"] {
        let (package, children) = protocol_holder_case(operator);
        assert!(
            matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
            "{operator} holder with correctly named children must admit"
        );
        for (child_index, &position) in children.iter().enumerate() {
            let mut wrong = package.clone();
            let name = wrong["semantic_graph"]["nodes"][position]["declaration"]["qualified_name"]
                .as_array()
                .expect("path")
                .last()
                .expect("name")
                .clone();
            wrong["semantic_graph"]["nodes"][position]["declaration"]["qualified_name"] =
                json!(["Example", "OrderFlow", "run", "Main", name]);
            settle(&mut wrong);
            let key = wrong["semantic_graph"]["nodes"][position]["node_id"]["digest"]
                .as_str()
                .expect("digest");
            assert_eq!(
                read(&wrong),
                refusal_at(
                    CheckedPackageRefusalCode::InvalidSemanticGraph,
                    &format!("/semantic_graph/nodes/{position}"),
                    None,
                    key,
                ),
                "{operator} child {child_index} must use its holder path"
            );
        }
    }
}

/// Trace: FR-038-AC-288, FR-038-AC-290
#[trace("TC-048")]
#[test]
fn repeat_checks_owner_body_exhausted_and_optional_pair() {
    let (package, children) = protocol_holder_case("repeat");
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    let holder = holder_position(&package);

    let mut missing_exhausted = package.clone();
    missing_exhausted["semantic_graph"]["nodes"][holder]["body"]["arguments"][6]["value"] =
        json!({"term": "aggregate", "members": []});
    settle(&mut missing_exhausted);
    assert_eq!(
        read(&missing_exhausted),
        refusal(
            CheckedPackageRefusalCode::MalformedWire,
            &format!("/semantic_graph/nodes/{holder}/body")
        )
    );

    let mut wrong_body = package.clone();
    wrong_body["semantic_graph"]["nodes"][holder]["body"]["arguments"][5]["value"]["target"] =
        package["semantic_graph"]["nodes"][protocol_position(&package, "role")]["node_id"].clone();
    settle(&mut wrong_body);
    holder_refusal(
        &wrong_body,
        CheckedPackageRefusalCode::InvalidModelBinding,
        Some(CheckedPackageRefusalCause::MalformedDeclaration),
        "/body/arguments/5/value/target",
    );

    let mut no_bound = package;
    no_bound["semantic_graph"]["nodes"][holder]["body"]["arguments"][2]["value"]["members"] =
        json!([]);
    no_bound["semantic_graph"]["nodes"][holder]["body"]["arguments"][6]["value"]["members"] =
        json!([]);
    no_bound["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .remove(children[1]);
    settle(&mut no_bound);
    assert!(matches!(
        read(&no_bound),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    assert_ne!(children[0], children[1]);
}

/// Trace: FR-038-AC-288, FR-038-AC-290
#[trace("TC-048")]
#[test]
fn repeat_admits_proof_and_zero_maximum_and_refuses_both_unpaired_directions() {
    let (package, children) = protocol_holder_case("repeat");
    let holder = holder_position(&package);
    let mut with_proof = package.clone();
    with_proof["semantic_graph"]["nodes"][holder]["body"]["arguments"][3]["value"]["members"] = json!([
        {"term": "binding", "name": "invariant", "value": {"term": "literal", "type": node_id(&family_key("aaaa")), "value_kind": "boolean", "value": true}},
        {"term": "binding", "name": "variant", "value": {"term": "literal", "type": node_id(&family_key("a3a3")), "value_kind": "integer", "value": "1"}}
    ]);
    settle(&mut with_proof);
    assert!(matches!(
        read(&with_proof),
        CheckedPackageV2ReadResult::Admitted(_)
    ));

    let mut zero = with_proof.clone();
    zero["semantic_graph"]["nodes"][holder]["body"]["arguments"][2]["value"]["members"][0]
        ["value"] = json!("0");
    settle(&mut zero);
    assert!(matches!(
        read(&zero),
        CheckedPackageV2ReadResult::Admitted(_)
    ));

    let mut unbounded_proof = with_proof.clone();
    unbounded_proof["semantic_graph"]["nodes"][holder]["body"]["arguments"][2]["value"]
        ["members"] = json!([]);
    unbounded_proof["semantic_graph"]["nodes"][holder]["body"]["arguments"][6]["value"]
        ["members"] = json!([]);
    unbounded_proof["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .remove(children[1]);
    settle(&mut unbounded_proof);
    assert!(matches!(
        read(&unbounded_proof),
        CheckedPackageV2ReadResult::Admitted(_)
    ));

    for missing in [2, 6] {
        let mut unpaired = with_proof.clone();
        unpaired["semantic_graph"]["nodes"][holder]["body"]["arguments"][missing]["value"]
            ["members"] = json!([]);
        settle(&mut unpaired);
        assert_eq!(
            read(&unpaired),
            refusal(
                CheckedPackageRefusalCode::MalformedWire,
                &format!("/semantic_graph/nodes/{holder}/body")
            ),
            "unpaired argument {missing} must refuse"
        );
    }

    let mut proof_only = with_proof;
    for absent in [2, 6] {
        proof_only["semantic_graph"]["nodes"][holder]["body"]["arguments"][absent]["value"]
            ["members"] = json!([]);
    }
    proof_only["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .remove(children[1]);
    settle(&mut proof_only);
    assert!(matches!(
        read(&proof_only),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn repeat_non_boolean_guard_refuses_at_guard_not_wire() {
    let (mut package, _) = protocol_holder_case("repeat");
    let holder = holder_position(&package);
    package["semantic_graph"]["nodes"][holder]["body"]["arguments"][4]["value"] = json!({
        "term": "literal", "type": node_id(&family_key("a1a1")), "value_kind": "text", "value": "yes"
    });
    settle(&mut package);
    holder_refusal(
        &package,
        CheckedPackageRefusalCode::IllTyped,
        Some(CheckedPackageRefusalCause::NonBooleanRoot),
        "/body/arguments/4/value",
    );
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn await_checks_anchor_form_and_event_eligibility() {
    let (package, children) = protocol_holder_case("await");
    assert!(
        matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
        "{:?}",
        read(&package)
    );
    let holder = holder_position(&package);

    let mut wrong_anchor = package.clone();
    wrong_anchor["semantic_graph"]["nodes"][holder]["body"]["arguments"][0]["value"]["members"]
        [1]["value"]["target"] =
        wrong_anchor["semantic_graph"]["nodes"][children[1]]["node_id"].clone();
    settle(&mut wrong_anchor);
    holder_refusal(
        &wrong_anchor,
        CheckedPackageRefusalCode::InvalidModelBinding,
        Some(CheckedPackageRefusalCause::MalformedDeclaration),
        "/body/arguments/0/value/members/1/value/target",
    );

    let mut wrong_event = package;
    let role = wrong_event["semantic_graph"]["nodes"][protocol_position(&wrong_event, "role")]
        ["node_id"]
        .clone();
    wrong_event["semantic_graph"]["nodes"][children[0]]["body"]["operation"]["member"]
        ["operator"] = json!("send");
    wrong_event["semantic_graph"]["nodes"][children[0]]["body"]["arguments"] = json!([
        {"term": "binding", "name": "channel", "value": {"term": "reference", "target": role}},
        {"term": "binding", "name": "binder", "value": {"term": "reference", "target": role}},
        {"term": "binding", "name": "related", "value": {"term": "aggregate", "members": []}},
        {"term": "binding", "name": "constraint", "value": {"term": "literal", "type": node_id(&family_key("aaaa")), "value_kind": "boolean", "value": true}}
    ]);
    settle(&mut wrong_event);
    holder_refusal(
        &wrong_event,
        CheckedPackageRefusalCode::UnsupportedConstruct,
        Some(CheckedPackageRefusalCause::DeclarationForm),
        "",
    );
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&wrong_event) else {
        panic!("ineligible send event must refuse");
    };
    assert_eq!(
        refusal.protocol_detail.as_deref(),
        Some(&CheckedProtocolRefusalDetail::AwaitedEvent {
            form: "send".into(),
            profile: quire_contract_model::CheckedArtifactRef {
                authority: "agent-ix".into(),
                identity: "quire.protocol.finite-global/v1".into(),
            },
        })
    );
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn await_inverted_interval_refuses_with_written_bounds() {
    let (mut package, _) = protocol_holder_case("await");
    let holder = holder_position(&package);
    package["semantic_graph"]["nodes"][holder]["body"]["arguments"][2]["value"]["members"][0]
        ["value"]["value"] = json!("2");
    settle(&mut package);
    holder_refusal(
        &package,
        CheckedPackageRefusalCode::IllTyped,
        Some(CheckedPackageRefusalCause::TypeMismatch),
        "/body/arguments/2/value",
    );
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("inverted await interval must refuse");
    };
    assert_eq!(
        refusal.protocol_detail.as_deref(),
        Some(&CheckedProtocolRefusalDetail::Interval {
            member: "within".into(),
            expected: "closed interval with lower <= upper".into(),
            actual: "[2, 1]".into(),
        })
    );
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn await_effect_event_requires_an_attempt_source() {
    let (mut package, children) = protocol_holder_case("await");
    let role =
        package["semantic_graph"]["nodes"][protocol_position(&package, "role")]["node_id"].clone();
    let reference = || json!({"term": "reference", "target": role});
    let binding =
        |name: &str, value: Value| json!({"term": "binding", "name": name, "value": value});
    let arguments = json!([
        binding("of", reference()),
        binding("binder", reference()),
        binding("related", json!({"term": "aggregate", "members": []})),
        binding(
            "constraint",
            json!({
                "term": "literal", "type": node_id(&family_key("aaaa")), "value_kind": "boolean", "value": true
            })
        )
    ]);
    package["semantic_graph"]["nodes"][children[0]]["body"]["operation"]["member"]["operator"] =
        json!("effect");
    package["semantic_graph"]["nodes"][children[0]]["body"]["arguments"] = arguments;
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][children[0]]["node_id"]["digest"]
        .as_str()
        .expect("effect key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidModelBinding,
            &format!(
                "/semantic_graph/nodes/{}/body/arguments/0/value/target",
                children[0]
            ),
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            key
        )
    );
}

/// Trace: FR-038-AC-288, FR-038-AC-290
#[trace("TC-048")]
#[test]
fn await_admits_compensation_anchor_form() {
    let (state, _) = crate::checked_package_v2_frame_entries::state_operand_fixture();
    let (mut package, children) = protocol_holder_case_on(state, "await");
    let holder = holder_position(&package);
    let compensation = compensation_template(
        &mut package,
        &["Example", "OrderFlow", "compensate", "Undo"],
    );
    let awaited = package["semantic_graph"]["nodes"][children[0]]["body"]["arguments"][2]["value"]
        ["target"]
        .clone();
    let awaited = relevel_parameter(&mut package, &awaited, 4);
    package["semantic_graph"]["nodes"][children[0]]["body"]["arguments"][2]["value"]["target"] =
        awaited;
    let target = compensation["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(compensation);
    let clause = clause_position(&package);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][8]["value"]["members"] =
        json!([{"term": "reference", "target": target}]);
    package["semantic_graph"]["nodes"][holder]["body"]["arguments"][0]["value"]["members"][0]
        ["value"]["value"] = json!("compensation");
    package["semantic_graph"]["nodes"][holder]["body"]["arguments"][0]["value"]["members"][1]
        ["value"]["target"] = target;
    let root = protocol_position(&package, "control");
    package["semantic_graph"]["nodes"][root]["body"]["arguments"][0]["value"]["members"]
        .as_array_mut()
        .expect("root sequence")
        .rotate_right(2);
    settle(&mut package);
    assert!(
        matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
        "compensation anchor must admit"
    );

    let mut late_forward = package;
    let root = protocol_position(&late_forward, "control");
    late_forward["semantic_graph"]["nodes"][root]["body"]["arguments"][0]["value"]["members"]
        .as_array_mut()
        .expect("root sequence")
        .rotate_left(2);
    settle(&mut late_forward);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&late_forward) else {
        panic!("compensation forward effect after await must cycle");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::DefinitionCycle)
    );
    assert!(matches!(
        refusal.protocol_detail.as_deref(),
        Some(CheckedProtocolRefusalDetail::CausalCycle { controls }) if controls.len() >= 2
    ));
}

fn compensation_template(package: &mut Value, declaration: &[&str]) -> Value {
    let role =
        package["semantic_graph"]["nodes"][protocol_position(package, "role")]["node_id"].clone();
    let root = protocol_position(package, "control");
    let nested_await = package["semantic_graph"]["nodes"][root]["body"]["operation"]["member"]
        ["operator"]
        == "sequence";
    let attempt_binder = clause_parameter(package, "attempt", "1");
    let effect_binder = clause_parameter(package, "effect", "2");
    let event_binder = clause_parameter(package, "event", "3");
    let finish_binder = clause_parameter(package, "closed_after_effect", "4");
    let forward_binder = clause_parameter(package, "forward", "1");
    let trigger_binder = clause_parameter(package, "trigger", "3");
    let recovery_binder = clause_parameter(package, "recovery", "7");
    let finish = protocol_position(package, "obligation");
    package["semantic_graph"]["nodes"][finish]["body"]["arguments"][0]["value"]["target"] =
        finish_binder;
    let root_event = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| {
            node["semantic_form"] == "control"
                && node["body"]["operation"]["member"]["operator"] == "event"
                && (!nested_await || node["declaration"]["qualified_name"][4] == "Anchor")
        })
        .expect("event control");
    package["semantic_graph"]["nodes"][root_event]["body"]["arguments"][2]["value"]["target"] =
        event_binder;
    let anchor = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["semantic_form"] == "operation_anchor")
        .expect("operation anchor")["node_id"]
        .clone();
    let namespace = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "model" && node["semantic_form"] == "namespace")
        .expect("namespace model")
        .clone();
    let mut record_type = namespace;
    record_type["semantic_form"] = json!("record_value_type");
    let record_key = structural_key(
        "model",
        "record_value_type",
        record_type["semantic_type"]["digest"].as_str(),
        &record_type["body"],
    );
    record_type["node_id"] = node_id(&record_key);
    let record_target = record_type["node_id"].clone();
    let parameter = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "value" && node["semantic_form"] == "parameter")
        .expect("parameter")
        .clone();
    let retry_binder = |name: &str, level: usize| {
        let mut node = parameter.clone();
        node["semantic_type"] = record_target.clone();
        node["body"]["members"][0]["value"]["value"] = json!(name);
        node["body"]["members"][1]["value"]["value"] = json!(level.to_string());
        node["node_id"] = node_id(&structural_key(
            "value",
            "parameter",
            Some(&record_key),
            &node["body"],
        ));
        node
    };
    let earlier = retry_binder("earlier", 5);
    let later = retry_binder("later", 6);
    let earlier_target = earlier["node_id"].clone();
    let later_target = later["node_id"].clone();
    let capture_binder = |name: &str, level: usize| {
        let mut node = parameter.clone();
        node["body"]["members"][0]["value"]["value"] = json!(name);
        node["body"]["members"][1]["value"]["value"] = json!(level.to_string());
        node["node_id"] = node_id(&structural_key(
            "value",
            "parameter",
            Some(&family_key("aaaa")),
            &node["body"],
        ));
        node
    };
    let registration = capture_binder("registration", 2);
    let activation = capture_binder("activation", 4);
    let registration_target = registration["node_id"].clone();
    let activation_target = activation["node_id"].clone();
    let group = || json!({"term": "aggregate", "members": []});
    let binding =
        |name: &str, value: Value| json!({"term": "binding", "name": name, "value": value});
    let reference = |target: &Value| json!({"term": "reference", "target": target});
    let boolean = || {
        json!({"term": "literal", "type": node_id(&family_key("aaaa")),
        "value_kind": "boolean", "value": true})
    };
    let integer = |value: &str| {
        json!({"term": "literal", "type": node_id(&family_key("a3a3")),
        "value_kind": "integer", "value": value})
    };
    let mut attempt_body = package["semantic_graph"]["nodes"][root]["body"].clone();
    attempt_body["operation"]["member"]["operator"] = json!("attempt");
    attempt_body["arguments"] = json!([
        binding("role", reference(&role)),
        binding("operation", reference(&anchor)),
        binding("ordering", group()),
        binding("contracts", group()),
        binding("binder", reference(&attempt_binder)),
        binding("related", group()),
        binding("constraint", boolean())
    ]);
    let attempt = protocol_named_node(
        "control",
        &["Example", "OrderFlow", "run", "Main", "ForwardAttempt"],
        attempt_body,
        &[],
    );
    let attempt_target = attempt["node_id"].clone();
    let mut effect_body = package["semantic_graph"]["nodes"][root]["body"].clone();
    effect_body["operation"]["member"]["operator"] = json!("effect");
    effect_body["arguments"] = json!([
        binding("of", reference(&attempt_target)),
        binding("binder", reference(&effect_binder)),
        binding("related", group()),
        binding("constraint", boolean())
    ]);
    let effect = protocol_named_node(
        "control",
        &["Example", "OrderFlow", "run", "Main", "ForwardEffect"],
        effect_body,
        &[],
    );
    let effect_target = effect["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend([
            record_type,
            earlier,
            later,
            registration,
            activation,
            attempt,
            effect,
        ]);
    let root_body = &mut package["semantic_graph"]["nodes"][root]["body"];
    if root_body["operation"]["member"]["operator"] == "sequence" {
        root_body["arguments"][0]["value"]["members"]
            .as_array_mut()
            .expect("sequence children")
            .extend([reference(&attempt_target), reference(&effect_target)]);
    } else {
        let original_target = package["semantic_graph"]["nodes"][root]["node_id"].clone();
        package["semantic_graph"]["nodes"][root]["declaration"]["qualified_name"] =
            json!(["Example", "OrderFlow", "run", "Main", "Event"]);
        let mut sequence_body = package["semantic_graph"]["nodes"][root]["body"].clone();
        sequence_body["operation"]["member"]["operator"] = json!("sequence");
        sequence_body["arguments"] = json!([binding(
            "children",
            json!({"term": "aggregate", "members": [
                reference(&attempt_target),
                reference(&effect_target),
                reference(&original_target)
            ]})
        )]);
        let sequence = protocol_named_node(
            "control",
            &["Example", "OrderFlow", "run", "Main"],
            sequence_body,
            &[],
        );
        let sequence_target = sequence["node_id"].clone();
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(sequence);
        let clause = clause_position(package);
        package["semantic_graph"]["nodes"][clause]["body"]["arguments"][11]["value"]["target"] =
            sequence_target;
    }
    let body =
        package["semantic_graph"]["nodes"][protocol_position(package, "control")]["body"].clone();
    let mut compensation = protocol_named_node("compensation", declaration, body, &[]);
    compensation["body"]["operation"]["member"]["operator"] = json!("compensate");
    compensation["body"]["arguments"] = json!([
        binding("forward", reference(&effect_target)),
        binding("forward_binder", reference(&forward_binder)),
        binding("role", reference(&role)),
        binding("operation", reference(&anchor)),
        binding(
            "clock",
            json!({"term": "literal", "type": node_id(&family_key("a1a1")), "value_kind": "text", "value": "clock"})
        ),
        binding(
            "registration_captures",
            json!({"term": "aggregate", "members": [
                {"term": "aggregate", "members": [
                    binding("binder", reference(&registration_target)),
                    binding("initializer", boolean())
                ]}
            ]})
        ),
        binding(
            "trigger",
            json!({"term": "aggregate", "members": [
                binding("binder", reference(&trigger_binder)), binding("guard", boolean())
            ]})
        ),
        binding(
            "activation_captures",
            json!({"term": "aggregate", "members": [
                {"term": "aggregate", "members": [
                    binding("binder", reference(&activation_target)),
                    binding("initializer", boolean())
                ]}
            ]})
        ),
        binding(
            "within",
            json!({"term": "aggregate", "members": [
                binding("lower", integer("0")), binding("upper", integer("1"))
            ]})
        ),
        binding(
            "attempts",
            json!({"term": "aggregate", "members": [
                binding("maximum", integer("1")), binding("record_type", reference(&record_target))
            ]})
        ),
        binding(
            "retry",
            json!({"term": "aggregate", "members": [
                binding("earlier", reference(&earlier_target)),
                binding("later", reference(&later_target)), binding("relation", boolean())
            ]})
        ),
        binding("commit", group()),
        binding(
            "recovery",
            json!({"term": "aggregate", "members": [
                binding("binder", reference(&recovery_binder)), binding("predicate", boolean())
            ]})
        )
    ]);
    compensation
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn repeat_maximum_and_await_interval_reject_negative_bounds_at_wire() {
    let (mut repeated, _) = protocol_holder_case("repeat");
    let holder = holder_position(&repeated);
    repeated["semantic_graph"]["nodes"][holder]["body"]["arguments"][2]["value"]["members"][0]
        ["value"] = json!("-1");
    assert_eq!(
        read(&repeated),
        refusal_at(
            CheckedPackageRefusalCode::InvalidPackage,
            &format!("/semantic_graph/nodes/{holder}/body/arguments/2/value/members/0"),
            Some(CheckedPackageRefusalCause::InvalidValue),
            repeated["semantic_graph"]["nodes"][holder]["node_id"]["digest"]
                .as_str()
                .expect("id"),
        )
    );

    let (mut awaited, _) = protocol_holder_case("await");
    let holder = holder_position(&awaited);
    awaited["semantic_graph"]["nodes"][holder]["body"]["arguments"][2]["value"]["members"][1]
        ["value"]["value"] = json!("-1");
    assert_eq!(
        read(&awaited),
        refusal_at(
            CheckedPackageRefusalCode::InvalidPackage,
            &format!("/semantic_graph/nodes/{holder}/body/arguments/2/value/members/1/value"),
            Some(CheckedPackageRefusalCause::InvalidValue),
            awaited["semantic_graph"]["nodes"][holder]["node_id"]["digest"]
                .as_str()
                .expect("id"),
        )
    );
}

#[trace("TC-048")]
#[test]
fn protocol_declaration_occurrences_reject_generated_role() {
    let mut package = v2_all_families();
    let position = clause_position(&package);
    package["semantic_graph"]["nodes"][position]["occurrences"][0]["role"] = json!("generated");
    rebuild_source_map(&mut package);
    refresh_identity(&mut package);
    assert_eq!(
        read(&package),
        refusal(
            CheckedPackageRefusalCode::MalformedWire,
            &format!("/semantic_graph/nodes/{position}"),
        )
    );
}

#[trace("TC-048")]
#[test]
fn queue_requires_no_declaration_and_generated_occurrences() {
    for keep_declaration in [true, false] {
        let mut package = v2_all_families();
        let clause = clause_position(&package);
        let target = package["semantic_graph"]["nodes"][clause]["node_id"].clone();
        let text_type = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .find(|node| node["node_tag"] == "scalar_type" && node["semantic_form"] == "text")
            .expect("text type")["node_id"]
            .clone();
        let integer_type = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .find(|node| node["node_tag"] == "scalar_type" && node["semantic_form"] == "integer")
            .expect("integer type")["node_id"]
            .clone();
        let text = |value: &str| json!({"term": "literal", "type": text_type, "value_kind": "text", "value": value});
        let binding =
            |name: &str, value: Value| json!({"term": "binding", "name": name, "value": value});
        let mut node = protocol_role_node();
        node["semantic_form"] = json!("queue");
        node["body"]["operation"]["member"]["operator"] = json!("queue");
        node["body"]["arguments"] = json!([
            binding("channel", json!({"term": "reference", "target": target})),
            binding(
                "capacity",
                json!({"term": "aggregate", "members": [
                    binding("kind", text("literal")),
                    binding("value", json!({"term": "literal", "type": integer_type,
                        "value_kind": "integer", "value": "1"}))
                ]})
            ),
            binding("overflow", text("reject")),
            binding("loss", json!({"term": "aggregate", "members": []})),
        ]);
        if !keep_declaration {
            node.as_object_mut().expect("node").remove("declaration");
        } else {
            node["occurrences"][0]["role"] = json!("generated");
        }
        let position = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .len();
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(node);
        settle(&mut package);
        assert_eq!(
            read(&package),
            refusal(
                CheckedPackageRefusalCode::MalformedWire,
                &format!("/semantic_graph/nodes/{position}"),
            )
        );
    }
}

#[trace("TC-048")]
#[test]
fn duplicate_role_name_precedes_its_member_target_defect() {
    let mut package = v2_all_families();
    let first = protocol_role_node();
    let target = package["semantic_graph"]["nodes"][0]["node_id"].clone();
    let mut second_body = first["body"].clone();
    second_body["arguments"][0]["value"]["target"] = target.clone();
    let second = protocol_named_node(
        "role",
        &["Example", "OrderFlow", "role", "Worker"],
        second_body,
        &[target["digest"].as_str().expect("digest").to_owned()],
    );
    let first_key = first["node_id"].clone();
    let second_key = second["node_id"].clone();
    let base = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend([first, second]);
    let clause = clause_position(&package);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][3]["value"]["members"] = json!([
        {"term": "reference", "target": first_key},
        {"term": "reference", "target": second_key},
    ]);
    settle(&mut package);
    let first_key = package["semantic_graph"]["nodes"][base]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    let second_key = package["semantic_graph"]["nodes"][base + 1]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    let (position, key) = if first_key < second_key {
        (base + 1, second_key)
    } else {
        (base, first_key)
    };
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::AmbiguousDeclaration,
            &format!("/semantic_graph/nodes/{position}/declaration/qualified_name"),
            Some(CheckedPackageRefusalCause::AmbiguousName),
            key,
        )
    );
}

#[trace("TC-048")]
#[test]
fn duplicate_control_names_under_one_holder_refuse_in_digest_order() {
    let mut package = v2_all_families();
    let root = protocol_position(&package, "control");
    let first_body = package["semantic_graph"]["nodes"][root]["body"].clone();
    let mut second_body = first_body.clone();
    second_body["operation"]["member"]["operator"] = json!("check");
    let boolean = package["semantic_graph"]["nodes"][root]["semantic_type"].clone();
    second_body["arguments"] = json!([{
        "term": "binding", "name": "condition", "value": {
            "term": "literal", "type": boolean, "value_kind": "boolean", "value": true
        }
    }]);
    let path = &["Example", "OrderFlow", "run", "Main", "Done"];
    let first = protocol_named_node("control", path, first_body, &[]);
    let second = protocol_named_node("control", path, second_body, &[]);
    let first_key = first["node_id"].clone();
    let second_key = second["node_id"].clone();
    let base = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend([first, second]);
    package["semantic_graph"]["nodes"][root]["body"]["arguments"][0]["value"]["members"] = json!([
        {"term": "reference", "target": first_key},
        {"term": "reference", "target": second_key}
    ]);
    settle(&mut package);
    let first_key = package["semantic_graph"]["nodes"][base]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    let second_key = package["semantic_graph"]["nodes"][base + 1]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    let (position, key) = if first_key < second_key {
        (base + 1, second_key)
    } else {
        (base, first_key)
    };
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::AmbiguousDeclaration,
            &format!("/semantic_graph/nodes/{position}/declaration/qualified_name"),
            Some(CheckedPackageRefusalCause::AmbiguousName),
            key,
        )
    );
}

#[trace("TC-048")]
#[test]
fn protocol_application_placement_precedes_duplicate_declaration() {
    let mut package = v2_all_families();
    let clause = clause_position(&package);
    let mut duplicate_body = package["semantic_graph"]["nodes"][clause]["body"].clone();
    let boolean = package["semantic_graph"]["nodes"][clause]["semantic_type"].clone();
    duplicate_body["arguments"][10]["value"]["members"] = json!([{
        "term": "literal", "type": boolean,
        "value_kind": "text", "value": "adversarial"
    }]);
    append_duplicate_clause(&mut package, duplicate_body);
    let function = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| {
            node["node_tag"] == "function"
                && node["semantic_form"] == "pure_function"
                && node["body"]["term"] == "application"
        })
        .expect("function application");
    package["semantic_graph"]["nodes"][function]["body"] =
        package["semantic_graph"]["nodes"][clause]["body"].clone();
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][function]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::IllTyped,
            &format!("/semantic_graph/nodes/{function}/body"),
            Some(CheckedPackageRefusalCause::OperatorIneligible),
            key,
        )
    );
}

fn package_with_member_role() -> (Value, usize) {
    package_with_member_role_on(v2_all_families())
}

fn whole_model_role(identity: &str) -> (Value, usize) {
    let (mut package, role) = package_with_member_role();
    package["semantic_graph"]["nodes"][role]["body"]["arguments"][0]["value"] = json!({
        "term": "literal", "type": node_id(&family_key("a1a1")),
        "value_kind": "text", "value": identity,
    });
    settle(&mut package);
    (package, role)
}

fn package_with_member_role_on(mut package: Value) -> (Value, usize) {
    let role = protocol_role_node();
    let key = role["node_id"].clone();
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(role);
    let clause = clause_position(&package);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][3]["value"]["members"] =
        json!([{"term": "reference", "target": key}]);
    settle(&mut package);
    (package, position)
}

fn package_with_replicated_role() -> (Value, usize) {
    let (mut package, role) = package_with_member_role();
    let model =
        package["semantic_graph"]["nodes"][role]["body"]["arguments"][0]["value"]["target"].clone();
    let reference_body = json!({"term": "aggregate", "members": [
        {"term": "reference", "target": model}
    ]});
    let reference_type = structural_key("composite_type", "reference", None, &reference_body);
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(json!({
            "node_id": node_id(&reference_type),
            "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": "composite_type", "semantic_form": "reference",
            "semantic_type": node_id(&reference_type), "dependencies": [model],
            "occurrences": [{"role": "type", "ordinal": 0}],
            "body": reference_body,
        }));
    let mut binder = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_id"] == node_id(&family_key("p-close")))
        .expect("parameter")
        .clone();
    binder["semantic_type"] = node_id(&reference_type);
    let binder_key = structural_key("value", "parameter", Some(&reference_type), &binder["body"]);
    binder["node_id"] = node_id(&binder_key);
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(binder);
    package["semantic_graph"]["nodes"][role]["body"]["arguments"][1]["value"]["members"] = json!([
        {"term": "binding", "name": "population", "value": {
            "term": "literal", "type": node_id(&family_key("a1a1")),
            "value_kind": "text", "value": "orders"}},
        {"term": "binding", "name": "max", "value": {
            "term": "literal", "type": node_id(&family_key("a3a3")),
            "value_kind": "integer", "value": "2"}},
        {"term": "binding", "name": "binder", "value": {
            "term": "reference", "target": node_id(&binder_key)}},
        {"term": "binding", "name": "lifetime", "value": {
            "term": "literal", "type": node_id(&family_key("a1a1")),
            "value_kind": "text", "value": "workflow"}},
    ]);
    settle(&mut package);
    (package, role)
}

fn package_with_member_channel() -> (Value, usize, usize) {
    let (mut package, role) = package_with_member_role();
    let position = add_member_channel(&mut package, role);
    (package, role, position)
}

fn add_member_channel(package: &mut Value, role: usize) -> usize {
    let role_key = package["semantic_graph"]["nodes"][role]["node_id"].clone();
    let boolean = node_id(&family_key("aaaa"));
    let text = |value: &str| {
        json!({
            "term": "literal", "type": node_id(&family_key("a1a1")),
            "value_kind": "text", "value": value
        })
    };
    let binding =
        |name: &str, value: Value| json!({"term": "binding", "name": name, "value": value});
    let reference = |target: &Value| json!({"term": "reference", "target": target});
    let mut body =
        package["semantic_graph"]["nodes"][protocol_position(package, "control")]["body"].clone();
    body["operation"]["member"]["operator"] = json!("channel");
    body["arguments"] = json!([
        binding("from", reference(&role_key)),
        binding("to", reference(&role_key)),
        binding("carries", reference(&boolean)),
        binding(
            "ordering",
            json!({"term": "aggregate", "members": [binding("kind", text("unordered"))]})
        ),
        binding("delivery", text("unknown")),
    ]);
    let channel = protocol_named_node(
        "channel",
        &["Example", "OrderFlow", "channel", "Link"],
        body,
        &[
            role_key["digest"].as_str().expect("digest").to_owned(),
            family_key("aaaa"),
        ],
    );
    let channel_key = channel["node_id"].clone();
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(channel);
    let clause = clause_position(package);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][5]["value"]["members"] =
        json!([reference(&channel_key)]);
    settle(package);
    position
}

fn package_with_member_queue() -> (Value, usize) {
    let (mut package, _, channel) = package_with_member_channel();
    let channel_key = package["semantic_graph"]["nodes"][channel]["node_id"].clone();
    let text = |value: &str| {
        json!({
            "term": "literal", "type": node_id(&family_key("a1a1")),
            "value_kind": "text", "value": value
        })
    };
    let binding =
        |name: &str, value: Value| json!({"term": "binding", "name": name, "value": value});
    let mut body = package["semantic_graph"]["nodes"][channel]["body"].clone();
    body["operation"]["member"]["operator"] = json!("queue");
    body["arguments"] = json!([
        binding(
            "channel",
            json!({"term": "reference", "target": channel_key})
        ),
        binding(
            "capacity",
            json!({"term": "aggregate", "members": [
                binding("kind", text("literal")),
                binding("value", json!({"term": "literal", "type": node_id(&family_key("a3a3")),
                    "value_kind": "integer", "value": "4"})),
            ]})
        ),
        binding("overflow", text("reject")),
        binding("loss", json!({"term": "aggregate", "members": []})),
    ]);
    let mut queue = protocol_named_node("queue", &[], body, &[]);
    queue
        .as_object_mut()
        .expect("queue node")
        .remove("declaration");
    queue["occurrences"][0]["role"] = json!("generated");
    let queue_key = queue["node_id"].clone();
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(queue);
    let clause = clause_position(&package);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][6]["value"]["members"] =
        json!([{"term": "reference", "target": queue_key}]);
    settle(&mut package);
    (package, position)
}

fn package_with_send() -> (Value, usize, usize) {
    let (mut package, _) = package_with_member_queue();
    let channel = protocol_position(&package, "channel");
    let root = protocol_position(&package, "control");
    let binder = package["semantic_graph"]["nodes"][protocol_position(&package, "obligation")]
        ["body"]["arguments"][0]["value"]["target"]
        .clone();
    let channel_key = package["semantic_graph"]["nodes"][channel]["node_id"].clone();
    let body = &mut package["semantic_graph"]["nodes"][root]["body"];
    body["operation"]["member"]["operator"] = json!("send");
    body["arguments"] = json!([
        {"term": "binding", "name": "channel", "value": {"term": "reference", "target": channel_key}},
        {"term": "binding", "name": "binder", "value": {"term": "reference", "target": binder}},
        {"term": "binding", "name": "related", "value": {"term": "aggregate", "members": []}},
        {"term": "binding", "name": "constraint", "value": {
            "term": "literal", "type": node_id(&family_key("aaaa")), "value_kind": "boolean", "value": true
        }}
    ]);
    let finish_binder = clause_parameter(&mut package, "finished", "2");
    let finish = protocol_position(&package, "obligation");
    package["semantic_graph"]["nodes"][finish]["body"]["arguments"][0]["value"]["target"] =
        finish_binder;
    settle(&mut package);
    (package, root, channel)
}

fn package_with_step_control(operator: &str) -> (Value, usize) {
    package_with_step_control_on(v2_all_families(), operator)
}

fn package_with_step_control_on(package: Value, operator: &str) -> (Value, usize) {
    let (mut package, _) = package_with_member_role_on(package);
    let root = protocol_position(&package, "control");
    let role =
        package["semantic_graph"]["nodes"][protocol_position(&package, "role")]["node_id"].clone();
    let binder = package["semantic_graph"]["nodes"][protocol_position(&package, "obligation")]
        ["body"]["arguments"][0]["value"]["target"]
        .clone();
    let boolean = json!({
        "term": "literal", "type": node_id(&family_key("aaaa")),
        "value_kind": "boolean", "value": true
    });
    let arguments = match operator {
        "attempt" => {
            json!([
                {"term": "binding", "name": "role", "value": {"term": "reference", "target": role}},
                {"term": "binding", "name": "operation", "value": {"term": "reference", "target": role}},
                {"term": "binding", "name": "ordering", "value": {"term": "aggregate", "members": []}},
                {"term": "binding", "name": "contracts", "value": {"term": "aggregate", "members": []}},
                {"term": "binding", "name": "binder", "value": {"term": "reference", "target": binder}},
                {"term": "binding", "name": "related", "value": {"term": "aggregate", "members": []}},
                {"term": "binding", "name": "constraint", "value": boolean}
            ])
        }
        "event" => json!([
            {"term": "binding", "name": "role", "value": {"term": "reference", "target": role}},
            {"term": "binding", "name": "compensation", "value": {"term": "aggregate", "members": []}},
            {"term": "binding", "name": "binder", "value": {"term": "reference", "target": binder}},
            {"term": "binding", "name": "related", "value": {"term": "aggregate", "members": []}},
            {"term": "binding", "name": "constraint", "value": boolean}
        ]),
        "commit" => json!([
            {"term": "binding", "name": "role", "value": {"term": "reference", "target": role}},
            {"term": "binding", "name": "binder", "value": {"term": "reference", "target": binder}},
            {"term": "binding", "name": "constraint", "value": boolean}
        ]),
        "effect" => json!([
            {"term": "binding", "name": "of", "value": {"term": "reference", "target": role}},
            {"term": "binding", "name": "binder", "value": {"term": "reference", "target": binder}},
            {"term": "binding", "name": "related", "value": {"term": "aggregate", "members": []}},
            {"term": "binding", "name": "constraint", "value": boolean}
        ]),
        _ => panic!("unsupported fixture operator"),
    };
    let body = &mut package["semantic_graph"]["nodes"][root]["body"];
    body["operation"]["member"]["operator"] = json!(operator);
    body["arguments"] = arguments;
    let finish_binder = clause_parameter(&mut package, "finished", "2");
    let finish = protocol_position(&package, "obligation");
    package["semantic_graph"]["nodes"][finish]["body"]["arguments"][0]["value"]["target"] =
        finish_binder;
    settle(&mut package);
    (package, root)
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn event_and_commit_check_local_references_and_boolean_constraints() {
    for operator in ["event", "commit"] {
        let (mut package, root) = package_with_step_control(operator);
        assert!(
            matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
            "{operator}: {:?}",
            read(&package)
        );
        let ordinal = if operator == "event" { 4 } else { 2 };
        package["semantic_graph"]["nodes"][root]["body"]["arguments"][ordinal]["value"] = json!({"term": "literal", "type": node_id(&family_key("a1a1")),
                "value_kind": "text", "value": "wrong"});
        settle(&mut package);
        let key = package["semantic_graph"]["nodes"][root]["node_id"]["digest"]
            .as_str()
            .expect("control key");
        assert_eq!(
            read(&package),
            refusal_at(
                CheckedPackageRefusalCode::IllTyped,
                &format!("/semantic_graph/nodes/{root}/body/arguments/{ordinal}/value"),
                Some(CheckedPackageRefusalCause::NonBooleanRoot),
                key
            )
        );
    }
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn attempt_requires_a_state_operation_anchor() {
    let (package, root) = package_with_step_control("attempt");
    let key = package["semantic_graph"]["nodes"][root]["node_id"]["digest"]
        .as_str()
        .expect("attempt key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidModelBinding,
            &format!("/semantic_graph/nodes/{root}/body/arguments/1/value/target"),
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            key
        )
    );
}

fn package_with_admitted_attempt() -> (Value, usize, Value) {
    let (state, _) = crate::checked_package_v2_frame_entries::state_operand_fixture();
    let (mut package, root) = package_with_step_control_on(state, "attempt");
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    let anchor = nodes
        .iter()
        .find(|node| node["semantic_form"] == "operation_anchor")
        .expect("operation anchor")["node_id"]
        .clone();
    let precondition = nodes
        .iter()
        .find(|node| {
            node["semantic_form"] == "state_clause"
                && node["body"]["operation"]["member"]["clause"] == "precondition"
        })
        .expect("precondition")["node_id"]
        .clone();
    package["semantic_graph"]["nodes"][root]["body"]["arguments"][1]["value"]["target"] =
        anchor.clone();
    package["semantic_graph"]["nodes"][root]["body"]["arguments"][3]["value"]["members"] =
        json!([{"term": "reference", "target": precondition}]);
    settle(&mut package);
    (package, root, anchor)
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn attempt_admits_operation_contract_and_rejects_wrong_contract_kind_and_anchor() {
    let (package, root, anchor) = package_with_admitted_attempt();
    assert!(
        matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
        "{:?}",
        read(&package)
    );

    let mut wrong_kind = package.clone();
    let invariant = wrong_kind["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| {
            node["semantic_form"] == "state_clause"
                && node["body"]["operation"]["member"]["clause"] == "invariant"
        })
        .expect("invariant")["node_id"]
        .clone();
    wrong_kind["semantic_graph"]["nodes"][root]["body"]["arguments"][3]["value"]["members"][0]
        ["target"] = invariant;
    settle(&mut wrong_kind);
    let key = wrong_kind["semantic_graph"]["nodes"][root]["node_id"]["digest"]
        .as_str()
        .expect("attempt key");
    assert_eq!(
        read(&wrong_kind),
        refusal_at(
            CheckedPackageRefusalCode::InvalidModelBinding,
            &format!("/semantic_graph/nodes/{root}/body/arguments/3/value/members/0/target"),
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            key
        )
    );

    let mut wrong_anchor = package;
    let anchor_position = wrong_anchor["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_id"] == anchor)
        .expect("anchor position");
    let mut withdraw = wrong_anchor["semantic_graph"]["nodes"][anchor_position].clone();
    withdraw["body"]["members"][1]["value"]["value"] = json!("withdraw");
    let context = withdraw["semantic_type"]["digest"]
        .as_str()
        .expect("context key");
    let withdraw_key = structural_key(
        "state",
        "operation_anchor",
        Some(context),
        &withdraw["body"],
    );
    withdraw["node_id"] = node_id(&withdraw_key);
    wrong_anchor["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(withdraw);
    wrong_anchor["semantic_graph"]["nodes"][root]["body"]["arguments"][1]["value"]["target"] =
        node_id(&withdraw_key);
    settle(&mut wrong_anchor);
    let key = wrong_anchor["semantic_graph"]["nodes"][root]["node_id"]["digest"]
        .as_str()
        .expect("attempt key");
    assert_eq!(
        read(&wrong_anchor),
        refusal_at(
            CheckedPackageRefusalCode::InvalidModelBinding,
            &format!("/semantic_graph/nodes/{root}/body/arguments/3/value/members/0/target"),
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            key
        )
    );
}

fn package_with_admitted_effect() -> (Value, usize, Value) {
    let (mut package, root, anchor) = package_with_admitted_attempt();
    let attempt_body = package["semantic_graph"]["nodes"][root]["body"].clone();
    let binder = clause_parameter(&mut package, "effect", "2");
    let attempt = protocol_named_node(
        "control",
        &["Example", "OrderFlow", "run", "Main", "Attempt"],
        attempt_body,
        &[],
    );
    let attempt_key = attempt["node_id"].clone();
    let mut effect_body = package["semantic_graph"]["nodes"][root]["body"].clone();
    effect_body["operation"]["member"]["operator"] = json!("effect");
    effect_body["arguments"] = json!([
        {"term": "binding", "name": "of", "value": {"term": "reference", "target": attempt_key}},
        {"term": "binding", "name": "binder", "value": {"term": "reference", "target": binder}},
        {"term": "binding", "name": "related", "value": {"term": "aggregate", "members": []}},
        {"term": "binding", "name": "constraint", "value": {"term": "literal",
            "type": node_id(&family_key("aaaa")), "value_kind": "boolean", "value": true}}
    ]);
    let effect = protocol_named_node(
        "control",
        &["Example", "OrderFlow", "run", "Main", "Effect"],
        effect_body,
        &[],
    );
    let effect_key = effect["node_id"].clone();
    let effect_position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        + 1;
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend([attempt, effect]);
    package["semantic_graph"]["nodes"][root]["body"]["operation"]["member"]["operator"] =
        json!("sequence");
    package["semantic_graph"]["nodes"][root]["body"]["arguments"] = json!([
        {"term": "binding", "name": "children", "value": {"term": "aggregate", "members": [
            {"term": "reference", "target": attempt_key},
            {"term": "reference", "target": effect_key}
        ]}}
    ]);
    let finish_binder = clause_parameter(&mut package, "finished_after_effect", "3");
    let finish = protocol_position(&package, "obligation");
    package["semantic_graph"]["nodes"][finish]["body"]["arguments"][0]["value"]["target"] =
        finish_binder;
    settle(&mut package);
    (package, effect_position, anchor)
}

/// Trace: FR-038-AC-288, FR-038-AC-290
#[trace("TC-048")]
#[test]
fn effect_admits_a_referenced_attempt_in_the_same_control_tree() {
    let (package, effect, _) = package_with_admitted_effect();
    assert!(
        matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
        "{:?}",
        read(&package)
    );
    let mut reversed = package;
    let root = protocol_position(&reversed, "control");
    reversed["semantic_graph"]["nodes"][root]["body"]["arguments"][0]["value"]["members"]
        .as_array_mut()
        .expect("sequence children")
        .swap(0, 1);
    settle(&mut reversed);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&reversed) else {
        panic!("effect before its attempt must form a causal cycle");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::DefinitionCycle)
    );
    let Some(CheckedProtocolRefusalDetail::CausalCycle { controls }) =
        refusal.protocol_detail.as_deref()
    else {
        panic!("cycle must name the effect and attempt");
    };
    let expected: Vec<quire_contract_model::CheckedNodeId> = [effect, effect - 1]
        .into_iter()
        .map(|position| {
            serde_json::from_value(reversed["semantic_graph"]["nodes"][position]["node_id"].clone())
                .expect("control identity")
        })
        .collect();
    assert_eq!(controls, &expected);
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn effect_requires_an_attempt_target() {
    let (package, root) = package_with_step_control("effect");
    let key = package["semantic_graph"]["nodes"][root]["node_id"]["digest"]
        .as_str()
        .expect("effect key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidModelBinding,
            &format!("/semantic_graph/nodes/{root}/body/arguments/0/value/target"),
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            key
        )
    );
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn event_compensation_requires_a_template_target() {
    let (mut package, root) = package_with_step_control("event");
    let role =
        package["semantic_graph"]["nodes"][protocol_position(&package, "role")]["node_id"].clone();
    package["semantic_graph"]["nodes"][root]["body"]["arguments"][1]["value"]["members"] =
        json!([{"term": "reference", "target": role}]);
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][root]["node_id"]["digest"]
        .as_str()
        .expect("event key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidModelBinding,
            &format!("/semantic_graph/nodes/{root}/body/arguments/1/value/members/0/target"),
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            key
        )
    );
}

fn package_with_admitted_compensation() -> (Value, usize, usize) {
    let (state, _) = crate::checked_package_v2_frame_entries::state_operand_fixture();
    let (mut package, root) = package_with_step_control_on(state, "event");
    let local = compensation_template(
        &mut package,
        &["Example", "OrderFlow", "compensate", "Undo"],
    );
    let local_key = local["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(local);
    let clause = clause_position(&package);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][8]["value"]["members"] =
        json!([{"term": "reference", "target": local_key}]);
    package["semantic_graph"]["nodes"][root]["body"]["arguments"][1]["value"]["members"] =
        json!([{"term": "reference", "target": local_key}]);
    settle(&mut package);
    let compensation = protocol_position(&package, "compensation");
    (package, root, compensation)
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn event_admits_its_clause_template_and_refuses_a_foreign_template() {
    let (mut package, root, compensation) = package_with_admitted_compensation();
    assert!(
        matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
        "{:?}",
        read(&package)
    );

    let foreign = protocol_named_node(
        "compensation",
        &["Example", "Elsewhere", "compensate", "Undo"],
        package["semantic_graph"]["nodes"][compensation]["body"].clone(),
        &[],
    );
    let foreign_key = foreign["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(foreign);
    package["semantic_graph"]["nodes"][root]["body"]["arguments"][1]["value"]["members"][0]
        ["target"] = foreign_key;
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][root]["node_id"]["digest"]
        .as_str()
        .expect("event key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidModelBinding,
            &format!("/semantic_graph/nodes/{root}/body/arguments/1/value/members/0/target"),
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            key
        )
    );
}

/// Trace: FR-038-AC-282, FR-038-AC-289
#[trace("TC-048")]
#[test]
fn compensation_admits_local_bindings_and_rejects_wrong_target_forms() {
    let (package, _, compensation) = package_with_admitted_compensation();
    assert!(
        matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
        "{:?}",
        read(&package)
    );
    let role =
        package["semantic_graph"]["nodes"][protocol_position(&package, "role")]["node_id"].clone();
    for (ordinal, pointer) in [
        (
            0,
            format!("/semantic_graph/nodes/{compensation}/body/arguments/0/value/target"),
        ),
        (
            11,
            format!(
                "/semantic_graph/nodes/{compensation}/body/arguments/11/value/members/0/target"
            ),
        ),
    ] {
        let mut wrong = package.clone();
        if ordinal == 0 {
            wrong["semantic_graph"]["nodes"][compensation]["body"]["arguments"][0]["value"]
                ["target"] = role.clone();
        } else {
            wrong["semantic_graph"]["nodes"][compensation]["body"]["arguments"][11]["value"]
                ["members"] = json!([{"term": "reference", "target": role}]);
        }
        settle(&mut wrong);
        let key = wrong["semantic_graph"]["nodes"][compensation]["node_id"]["digest"]
            .as_str()
            .expect("compensation key");
        assert_eq!(
            read(&wrong),
            refusal_at(
                CheckedPackageRefusalCode::InvalidModelBinding,
                &pointer,
                Some(CheckedPackageRefusalCause::MalformedDeclaration),
                key,
            ),
            "argument {ordinal}"
        );
    }
}

/// Trace: FR-038-AC-282, FR-038-AC-289
#[trace("TC-048")]
#[test]
fn compensation_nonempty_captures_admit_and_check_record_binders() {
    let (package, _, compensation) = package_with_admitted_compensation();
    for ordinal in [5, 7] {
        assert_eq!(
            package["semantic_graph"]["nodes"][compensation]["body"]["arguments"][ordinal]["value"]
                ["members"]
                .as_array()
                .expect("capture records")
                .len(),
            1,
        );
    }
    assert!(
        matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
        "{:?}",
        read(&package)
    );

    let role =
        package["semantic_graph"]["nodes"][protocol_position(&package, "role")]["node_id"].clone();
    for ordinal in [5, 7] {
        let mut wrong_target = package.clone();
        wrong_target["semantic_graph"]["nodes"][compensation]["body"]["arguments"][ordinal]
            ["value"]["members"][0]["members"][0]["value"]["target"] = role.clone();
        settle(&mut wrong_target);
        let key = wrong_target["semantic_graph"]["nodes"][compensation]["node_id"]["digest"]
            .as_str()
            .expect("compensation key");
        assert_eq!(
            read(&wrong_target),
            refusal_at(
                CheckedPackageRefusalCode::InvalidModelBinding,
                &format!("/semantic_graph/nodes/{compensation}/body/arguments/{ordinal}/value/members/0/members/0/value/target"),
                Some(CheckedPackageRefusalCause::MalformedDeclaration),
                key,
            ),
            "capture binder at argument {ordinal}"
        );
    }

    let mut wrong_shape = package;
    wrong_shape["semantic_graph"]["nodes"][compensation]["body"]["arguments"][7]["value"]
        ["members"][0]["members"]
        .as_array_mut()
        .expect("capture fields")
        .swap(0, 1);
    assert_eq!(
        read(&wrong_shape),
        refusal(
            CheckedPackageRefusalCode::MalformedWire,
            &format!("/semantic_graph/nodes/{compensation}/body"),
        )
    );
}

/// Trace: FR-038-AC-282, FR-038-AC-289
#[trace("TC-048")]
#[test]
fn compensation_rejects_zero_attempts_inverted_window_and_non_boolean_guard() {
    let (package, _, compensation) = package_with_admitted_compensation();
    let cases = [
        (
            9,
            0,
            "0",
            CheckedPackageRefusalCode::IllTyped,
            CheckedPackageRefusalCause::TypeMismatch,
        ),
        (
            8,
            0,
            "2",
            CheckedPackageRefusalCode::IllTyped,
            CheckedPackageRefusalCause::TypeMismatch,
        ),
    ];
    for (ordinal, member, value, code, cause) in cases {
        let mut wrong = package.clone();
        wrong["semantic_graph"]["nodes"][compensation]["body"]["arguments"][ordinal]["value"]
            ["members"][member]["value"]["value"] = json!(value);
        settle(&mut wrong);
        let path = if ordinal == 9 {
            format!("/semantic_graph/nodes/{compensation}/body/arguments/9/value/members/0/value")
        } else {
            format!("/semantic_graph/nodes/{compensation}/body/arguments/8/value")
        };
        let key = wrong["semantic_graph"]["nodes"][compensation]["node_id"]["digest"]
            .as_str()
            .expect("compensation key");
        assert_eq!(read(&wrong), refusal_at(code, &path, Some(cause), key));
    }
    let mut wrong = package;
    wrong["semantic_graph"]["nodes"][compensation]["body"]["arguments"][6]["value"]["members"][1]
        ["value"] = json!({
        "term": "literal", "type": node_id(&family_key("a3a3")),
        "value_kind": "integer", "value": "1"
    });
    settle(&mut wrong);
    let key = wrong["semantic_graph"]["nodes"][compensation]["node_id"]["digest"]
        .as_str()
        .expect("compensation key");
    assert_eq!(
        read(&wrong),
        refusal_at(
            CheckedPackageRefusalCode::IllTyped,
            &format!("/semantic_graph/nodes/{compensation}/body/arguments/6/value/members/1/value"),
            Some(CheckedPackageRefusalCause::NonBooleanRoot),
            key,
        )
    );
}

/// Trace: FR-038-AC-282, FR-038-AC-289
#[trace("TC-048")]
#[test]
fn compensation_negative_window_bound_refuses_at_strict_wire() {
    let (mut package, _, compensation) = package_with_admitted_compensation();
    package["semantic_graph"]["nodes"][compensation]["body"]["arguments"][8]["value"]["members"]
        [0]["value"]["value"] = json!("-1");
    let key = package["semantic_graph"]["nodes"][compensation]["node_id"]["digest"]
        .as_str()
        .expect("compensation key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidPackage,
            &format!("/semantic_graph/nodes/{compensation}/body/arguments/8/value/members/0/value"),
            Some(CheckedPackageRefusalCause::InvalidValue),
            key,
        )
    );
}

/// Trace: FR-038-AC-282, FR-038-AC-289
#[trace("TC-048")]
#[test]
fn compensation_malformed_attempt_count_refuses_without_panicking() {
    let (mut package, _, compensation) = package_with_admitted_compensation();
    package["semantic_graph"]["nodes"][compensation]["body"]["arguments"][9]["value"]["members"]
        [0]["value"]["value"] = json!("not-an-integer");
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][compensation]["node_id"]["digest"]
        .as_str()
        .expect("compensation key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::MalformedWire,
            &format!("/semantic_graph/nodes/{compensation}/body/arguments/9/value/members/0/value"),
            None,
            key,
        )
    );
}

/// Trace: FR-038-AC-282, FR-038-AC-289
#[trace("TC-048")]
#[test]
fn compensation_text_and_integer_literals_require_their_scalar_types() {
    let (package, _, compensation) = package_with_admitted_compensation();
    let cases = [
        (
            4,
            None,
            format!("/semantic_graph/nodes/{compensation}/body/arguments/4/value"),
        ),
        (
            8,
            Some(0),
            format!("/semantic_graph/nodes/{compensation}/body/arguments/8/value/members/0/value"),
        ),
        (
            9,
            Some(0),
            format!("/semantic_graph/nodes/{compensation}/body/arguments/9/value/members/0/value"),
        ),
    ];
    for (ordinal, member, path) in cases {
        let mut wrong = package.clone();
        let literal = &mut wrong["semantic_graph"]["nodes"][compensation]["body"]["arguments"]
            [ordinal]["value"];
        if let Some(member) = member {
            literal["members"][member]["value"]["type"] = node_id(&family_key("aaaa"));
        } else {
            literal["type"] = node_id(&family_key("aaaa"));
        }
        settle(&mut wrong);
        let key = wrong["semantic_graph"]["nodes"][compensation]["node_id"]["digest"]
            .as_str()
            .expect("compensation key");
        assert_eq!(
            read(&wrong),
            refusal_at(
                CheckedPackageRefusalCode::IllTyped,
                &path,
                Some(CheckedPackageRefusalCause::TypeMismatch),
                key,
            ),
            "literal at argument {ordinal}"
        );
    }
}

/// Trace: FR-038-AC-289
#[trace("TC-048")]
#[test]
fn compensation_requires_forward_and_retry_binder_types() {
    let (package, _, compensation) = package_with_admitted_compensation();
    let forward_binder = package["semantic_graph"]["nodes"][compensation]["body"]["arguments"][1]
        ["value"]["target"]
        .clone();
    let earlier_binder = package["semantic_graph"]["nodes"][compensation]["body"]["arguments"][10]
        ["value"]["members"][0]["value"]["target"]
        .clone();
    for (ordinal, replacement) in [(1, earlier_binder), (10, forward_binder)] {
        let mut wrong = package.clone();
        if ordinal == 1 {
            wrong["semantic_graph"]["nodes"][compensation]["body"]["arguments"][1]["value"]
                ["target"] = replacement;
        } else {
            wrong["semantic_graph"]["nodes"][compensation]["body"]["arguments"][10]["value"]
                ["members"][0]["value"]["target"] = replacement;
        }
        settle(&mut wrong);
        let key = wrong["semantic_graph"]["nodes"][compensation]["node_id"]["digest"]
            .as_str()
            .expect("compensation key");
        assert_eq!(
            read(&wrong),
            refusal_at(
                CheckedPackageRefusalCode::IllTyped,
                &format!("/semantic_graph/nodes/{compensation}"),
                Some(CheckedPackageRefusalCause::TypeMismatch),
                key,
            ),
            "binder at argument {ordinal}"
        );
        let CheckedPackageV2ReadResult::Refused(refusal) = read(&wrong) else {
            panic!("mismatched binder must refuse");
        };
        assert!(matches!(
            refusal.protocol_detail.as_deref(),
            Some(CheckedProtocolRefusalDetail::PayloadType { expected, actual })
                if expected != actual
        ));
    }
}

fn package_with_present_commit() -> (Value, usize, usize) {
    let (mut package, _, compensation) = package_with_admitted_compensation();
    let role =
        package["semantic_graph"]["nodes"][protocol_position(&package, "role")]["node_id"].clone();
    let binder = clause_parameter(&mut package, "commit", "4");
    let finish_binder = clause_parameter(&mut package, "closed_after_commit", "5");
    let finish = protocol_position(&package, "obligation");
    package["semantic_graph"]["nodes"][finish]["body"]["arguments"][0]["value"]["target"] =
        finish_binder;
    let mut body = package["semantic_graph"]["nodes"][compensation]["body"].clone();
    body["operation"]["member"]["operator"] = json!("commit");
    body["arguments"] = json!([
        {"term": "binding", "name": "role", "value": {"term": "reference", "target": role}},
        {"term": "binding", "name": "binder", "value": {"term": "reference", "target": binder}},
        {"term": "binding", "name": "constraint", "value": {"term": "literal",
            "type": node_id(&family_key("aaaa")), "value_kind": "boolean", "value": true}}
    ]);
    let commit = protocol_named_node(
        "control",
        &["Example", "OrderFlow", "run", "Main", "Commit"],
        body,
        &[],
    );
    let commit_target = commit["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(commit);
    let clause = clause_position(&package);
    let root_target =
        &package["semantic_graph"]["nodes"][clause]["body"]["arguments"][11]["value"]["target"];
    let root = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| &node["node_id"] == root_target)
        .expect("root control");
    package["semantic_graph"]["nodes"][root]["body"]["arguments"][0]["value"]["members"]
        .as_array_mut()
        .expect("sequence children")
        .push(json!({"term": "reference", "target": commit_target}));
    package["semantic_graph"]["nodes"][compensation]["body"]["arguments"][11]["value"]["members"] =
        json!([{"term": "reference", "target": commit_target}]);
    settle(&mut package);
    (package, compensation, root)
}

/// Trace: FR-038-AC-289
#[trace("TC-048")]
#[test]
fn compensation_admits_present_commit_and_refuses_wrong_record_type() {
    let (package, compensation, _) = package_with_present_commit();
    let role =
        package["semantic_graph"]["nodes"][protocol_position(&package, "role")]["node_id"].clone();
    assert!(
        matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
        "{:?}",
        read(&package)
    );

    let mut wrong = package;
    wrong["semantic_graph"]["nodes"][compensation]["body"]["arguments"][9]["value"]["members"][1]
        ["value"]["target"] = role;
    settle(&mut wrong);
    let key = wrong["semantic_graph"]["nodes"][compensation]["node_id"]["digest"]
        .as_str()
        .expect("compensation key");
    assert_eq!(
        read(&wrong),
        refusal_at(
            CheckedPackageRefusalCode::InvalidModelBinding,
            &format!("/semantic_graph/nodes/{compensation}/body/arguments/9/value/members/1/value/target"),
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            key,
        )
    );
}

/// Trace: FR-038-AC-289
#[trace("TC-048")]
#[test]
fn compensation_refuses_a_commit_taken_before_its_forward_effect() {
    let (mut package, compensation, root) = package_with_present_commit();
    let children = package["semantic_graph"]["nodes"][root]["body"]["arguments"][0]["value"]
        ["members"]
        .as_array_mut()
        .expect("sequence children");
    let commit_reference = children.pop().expect("commit child");
    children.insert(1, commit_reference);
    settle(&mut package);
    let commit_target = &package["semantic_graph"]["nodes"][compensation]["body"]["arguments"][11]
        ["value"]["members"][0]["target"];
    let effect_target = &package["semantic_graph"]["nodes"][compensation]["body"]["arguments"][0]
        ["value"]["target"];
    let commit = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| &node["node_id"] == commit_target)
        .expect("commit node");
    let key = commit_target["digest"].as_str().expect("commit key");
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("a preceding commit must refuse");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::UnsupportedConstruct
    );
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::DeclarationForm)
    );
    assert_eq!(
        refusal.path.as_ref().map(ToString::to_string),
        Some(format!("/semantic_graph/nodes/{commit}"))
    );
    assert_eq!(
        refusal.locus.as_ref().map(|node| node.digest.as_ref()),
        Some(key)
    );
    let Some(CheckedProtocolRefusalDetail::CommitPosition {
        profile,
        member,
        commit,
        effect,
        reason,
    }) = refusal.protocol_detail.as_deref()
    else {
        panic!("commit position detail must be retained");
    };
    assert_eq!(profile.identity.as_ref(), "quire.protocol.finite-global/v1");
    assert_eq!(member.as_ref(), "commit");
    assert_eq!(commit.digest.as_ref(), key);
    assert_eq!(
        effect.digest.as_ref(),
        effect_target["digest"].as_str().expect("effect key")
    );
    assert_eq!(reason.as_ref(), "commit-precedes-effect");
}

fn compensation_with_branch_before_effect(operator: &str, policy: &str) -> (Value, usize) {
    let (mut package, compensation, root) = package_with_present_commit();
    let children = package["semantic_graph"]["nodes"][root]["body"]["arguments"][0]["value"]
        ["members"]
        .as_array()
        .expect("sequence children");
    let attempt = children[0].clone();
    let effect = children[1].clone();
    let event = children[2].clone();
    let commit = children[3].clone();
    let role =
        package["semantic_graph"]["nodes"][protocol_position(&package, "role")]["node_id"].clone();
    let text = |value: &str| {
        json!({"term": "literal", "type": node_id(&family_key("a1a1")),
            "value_kind": "text", "value": value})
    };
    let binding =
        |name: &str, value: Value| json!({"term": "binding", "name": name, "value": value});
    let left = json!({"term": "aggregate", "members": [
        binding("label", text("Left")),
        binding("guard", json!({"term": "literal", "type": node_id(&family_key("aaaa")),
            "value_kind": "boolean", "value": true})),
        binding("body", commit.clone())
    ]});
    let right = json!({"term": "aggregate", "members": [
        binding("label", text("Right")),
        binding("guard", json!({"term": "literal", "type": node_id(&family_key("aaaa")),
            "value_kind": "boolean", "value": true})),
        binding("body", event.clone())
    ]});
    let mut body = package["semantic_graph"]["nodes"][root]["body"].clone();
    body["operation"]["member"]["operator"] = json!(operator);
    body["arguments"] = if operator == "choice" {
        json!([
            binding("owner", json!({"term": "reference", "target": role})),
            binding("visible", json!({"term": "aggregate", "members": []})),
            binding(
                "cases",
                json!({"term": "aggregate", "members": [left, right]})
            )
        ])
    } else {
        let mut left = left;
        let mut right = right;
        left["members"].as_array_mut().expect("fields").remove(1);
        right["members"].as_array_mut().expect("fields").remove(1);
        json!([
            binding("memory", json!({"term": "aggregate", "members": []})),
            binding(
                "branches",
                json!({"term": "aggregate", "members": [left, right]})
            ),
            binding(
                "join",
                json!({"term": "aggregate", "members": [binding("policy", text(policy))]})
            ),
            binding(
                "joined",
                json!({"term": "aggregate", "members": [text("Left"), text("Right")]})
            ),
            binding("outstanding", json!({"term": "aggregate", "members": []}))
        ])
    };
    let holder = protocol_named_node(
        "control",
        &["Example", "OrderFlow", "run", "Main", "Holder"],
        body,
        &[],
    );
    let holder_target = holder["node_id"].clone();
    let nodes = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    for (target, name) in [(&commit, "Commit"), (&event, "Event")] {
        let child = nodes
            .iter_mut()
            .find(|node| node["node_id"] == target["target"])
            .expect("branch child");
        child["declaration"]["qualified_name"] =
            json!(["Example", "OrderFlow", "run", "Main", "Holder", name]);
    }
    nodes.push(holder);
    package["semantic_graph"]["nodes"][root]["body"]["arguments"][0]["value"]["members"] =
        json!([attempt, {"term": "reference", "target": holder_target}, effect]);
    let commit_position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_id"] == commit["target"])
        .expect("commit control");
    let event_position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_id"] == event["target"])
        .expect("event control");
    let commit_binder = clause_parameter(&mut package, "branch_commit", "2");
    let event_binder = clause_parameter(&mut package, "branch_event", "2");
    package["semantic_graph"]["nodes"][commit_position]["body"]["arguments"][1]["value"]
        ["target"] = commit_binder;
    package["semantic_graph"]["nodes"][event_position]["body"]["arguments"][2]["value"]["target"] =
        event_binder;
    relevel_finish_binder(&mut package, 3);
    settle(&mut package);
    (package, compensation)
}

/// Trace: FR-038-AC-289
#[trace("TC-048")]
#[test]
fn compensation_commit_in_one_choice_or_non_all_branch_does_not_dominate() {
    for (operator, policy) in [("choice", ""), ("parallel", "any")] {
        let (package, _) = compensation_with_branch_before_effect(operator, policy);
        assert!(
            matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
            "{operator}: {:?}",
            read(&package)
        );
    }
    let (package, _) = compensation_with_branch_before_effect("parallel", "all");
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("parallel all commits must dominate");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::UnsupportedConstruct
    );
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::DeclarationForm)
    );
    assert!(matches!(refusal.protocol_detail.as_deref(),
        Some(CheckedProtocolRefusalDetail::CommitPosition { reason, .. })
            if reason.as_ref() == "commit-precedes-effect"));
}

/// Trace: FR-038-AC-284, FR-038-AC-289
#[trace("TC-048")]
#[test]
fn compensation_commit_in_every_branch_refuses_before_repeated_membership() {
    for (operator, policy) in [("choice", ""), ("parallel", "any")] {
        let (mut package, _) = compensation_with_branch_before_effect(operator, policy);
        let holder = holder_position(&package);
        let branches = if operator == "choice" { 2 } else { 1 };
        let body = if operator == "choice" { 2 } else { 1 };
        let commit = package["semantic_graph"]["nodes"][holder]["body"]["arguments"][branches]
            ["value"]["members"][0]["members"][body]["value"]
            .clone();
        package["semantic_graph"]["nodes"][holder]["body"]["arguments"][branches]["value"]
            ["members"][1]["members"][body]["value"] = commit;
        settle(&mut package);
        let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
            panic!("{operator} with commit in every branch must refuse");
        };
        assert_eq!(
            refusal.code,
            CheckedPackageRefusalCode::UnsupportedConstruct,
            "{operator}"
        );
        assert_eq!(
            refusal.cause,
            Some(CheckedPackageRefusalCause::DeclarationForm)
        );
    }
}

/// Trace: FR-038-AC-289
#[trace("TC-048")]
#[test]
fn compensation_commit_in_repeat_body_does_not_dominate() {
    let (mut package, _) = compensation_with_branch_before_effect("parallel", "all");
    let holder = holder_position(&package);
    let role =
        package["semantic_graph"]["nodes"][protocol_position(&package, "role")]["node_id"].clone();
    let branches =
        &package["semantic_graph"]["nodes"][holder]["body"]["arguments"][1]["value"]["members"];
    let commit = branches[0]["members"][1]["value"].clone();
    let event = branches[1]["members"][1]["value"].clone();
    let body = &mut package["semantic_graph"]["nodes"][holder]["body"];
    body["operation"]["member"]["operator"] = json!("repeat");
    body["arguments"] = json!([
        {"term": "binding", "name": "owner", "value": {"term": "reference", "target": role}},
        {"term": "binding", "name": "visible", "value": {"term": "aggregate", "members": []}},
        {"term": "binding", "name": "maximum", "value": {"term": "aggregate", "members": [
            {"term": "literal", "type": node_id(&family_key("a3a3")), "value_kind": "integer", "value": "1"}
        ]}},
        {"term": "binding", "name": "proof", "value": {"term": "aggregate", "members": []}},
        {"term": "binding", "name": "guard", "value": {"term": "literal", "type": node_id(&family_key("aaaa")),
            "value_kind": "boolean", "value": true}},
        {"term": "binding", "name": "body", "value": commit},
        {"term": "binding", "name": "exhausted", "value": {"term": "aggregate", "members": [event]}}
    ]);
    settle(&mut package);
    assert!(
        matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
        "{:?}",
        read(&package)
    );
}

/// Trace: FR-038-AC-284, FR-038-AC-289
#[trace("TC-048")]
#[test]
fn compensation_dominance_precedes_unattached_control_membership() {
    let (mut package, _, root) = package_with_present_commit();
    let children = package["semantic_graph"]["nodes"][root]["body"]["arguments"][0]["value"]
        ["members"]
        .as_array_mut()
        .expect("sequence children");
    let commit = children.pop().expect("commit child");
    children.insert(1, commit.clone());
    let nodes = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    let mut orphan_body = nodes
        .iter()
        .find(|node| node["node_id"] == commit["target"])
        .expect("commit node")["body"]
        .clone();
    orphan_body["arguments"][2]["value"]["value"] = json!(false);
    nodes.push(protocol_named_node(
        "control",
        &["Example", "OrderFlow", "run", "Main", "Orphan"],
        orphan_body,
        &[],
    ));
    settle(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("dominance must precede unattached membership");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::UnsupportedConstruct
    );
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::DeclarationForm)
    );
}

/// Trace: FR-038-AC-284, FR-038-AC-289
#[trace("TC-048")]
#[test]
fn compensation_first_template_dominance_precedes_later_template_bounds() {
    let (mut package, compensation, root) = package_with_present_commit();
    let children = package["semantic_graph"]["nodes"][root]["body"]["arguments"][0]["value"]
        ["members"]
        .as_array_mut()
        .expect("sequence children");
    let commit = children.pop().expect("commit child");
    children.insert(1, commit);
    settle(&mut package);
    let first = package["semantic_graph"]["nodes"][compensation]["node_id"].clone();
    let first_digest = first["digest"].as_str().expect("first digest");
    let mut later = None;
    for ordinal in 0..32 {
        let mut body = package["semantic_graph"]["nodes"][compensation]["body"].clone();
        body["arguments"][9]["value"]["members"][0]["value"]["value"] = json!("0");
        body["arguments"][4]["value"]["value"] = json!(format!("later-{ordinal}"));
        let candidate = protocol_named_node(
            "compensation",
            &["Example", "OrderFlow", "compensate", "Later"],
            body,
            &[],
        );
        if candidate["node_id"]["digest"]
            .as_str()
            .is_some_and(|digest| digest > first_digest)
        {
            later = Some(candidate);
            break;
        }
    }
    let later = later.expect("a later digest candidate");
    let later_target = later["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(later);
    let clause = clause_position(&package);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][8]["value"]["members"] = json!([
        {"term": "reference", "target": first},
        {"term": "reference", "target": later_target}
    ]);
    settle(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("first template dominance must precede later bounds");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::UnsupportedConstruct
    );
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::DeclarationForm)
    );
}

/// Trace: FR-038-AC-289
#[trace("TC-048")]
#[test]
fn compensation_commit_in_a_sibling_branch_is_unordered_against_effect() {
    let (mut package, _) = compensation_with_branch_before_effect("parallel", "all");
    let clause = clause_position(&package);
    let root_target = package["semantic_graph"]["nodes"][clause]["body"]["arguments"][11]["value"]
        ["target"]
        .clone();
    let nodes = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    let root = nodes
        .iter()
        .position(|node| node["node_id"] == root_target)
        .expect("root");
    let children = nodes[root]["body"]["arguments"][0]["value"]["members"]
        .as_array()
        .expect("sequence children");
    let effect = children[2].clone();
    let holder_target = children[1]["target"].clone();
    let holder = nodes
        .iter()
        .position(|node| node["node_id"] == holder_target)
        .expect("holder");
    let event =
        nodes[holder]["body"]["arguments"][1]["value"]["members"][1]["members"][1]["value"].clone();
    nodes[holder]["body"]["arguments"][1]["value"]["members"][1]["members"][1]["value"] =
        effect.clone();
    nodes[root]["body"]["arguments"][0]["value"]["members"][2] = event.clone();
    for (target, path) in [
        (
            &effect,
            json!([
                "Example",
                "OrderFlow",
                "run",
                "Main",
                "Holder",
                "ForwardEffect"
            ]),
        ),
        (
            &event,
            json!(["Example", "OrderFlow", "run", "Main", "Event"]),
        ),
    ] {
        let child = nodes
            .iter_mut()
            .find(|node| node["node_id"] == target["target"])
            .expect("moved control");
        child["declaration"]["qualified_name"] = path;
    }
    let event_position = nodes
        .iter()
        .position(|node| node["node_id"] == event["target"])
        .expect("moved event");
    let event_binder = nodes[event_position]["body"]["arguments"][2]["value"]["target"].clone();
    let event_binder = relevel_parameter(&mut package, &event_binder, 4);
    package["semantic_graph"]["nodes"][event_position]["body"]["arguments"][2]["value"]["target"] =
        event_binder;
    relevel_finish_binder(&mut package, 5);
    settle(&mut package);
    assert!(
        matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
        "{:?}",
        read(&package)
    );
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn attempt_ordering_rejects_an_unknown_spelling_at_wire_validation() {
    let (mut package, root) = package_with_step_control("attempt");
    package["semantic_graph"]["nodes"][root]["body"]["arguments"][2]["value"]["members"] = json!([{"term": "literal", "type": node_id(&family_key("a1a1")),
            "value_kind": "text", "value": "unordered"}]);
    assert_eq!(
        read(&package),
        refusal(
            CheckedPackageRefusalCode::MalformedWire,
            &format!("/semantic_graph/nodes/{root}/body/arguments/2/value/members/0")
        )
    );
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn check_condition_refuses_non_boolean_at_its_slot_and_fence_rejects_relaxed() {
    let mut package = v2_all_families();
    let root = protocol_position(&package, "control");
    let body = &mut package["semantic_graph"]["nodes"][root]["body"];
    body["operation"]["member"]["operator"] = json!("check");
    body["arguments"] = json!([{"term": "binding", "name": "condition", "value": {
        "term": "literal", "type": node_id(&family_key("aaaa")), "value_kind": "boolean", "value": true
    }}]);
    settle(&mut package);
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));

    package["semantic_graph"]["nodes"][root]["body"]["arguments"][0]["value"] = json!({
        "term": "literal", "type": node_id(&family_key("a1a1")), "value_kind": "text", "value": "yes"
    });
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][root]["node_id"]["digest"]
        .as_str()
        .expect("check key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::IllTyped,
            &format!("/semantic_graph/nodes/{root}/body/arguments/0/value"),
            Some(CheckedPackageRefusalCause::NonBooleanRoot),
            key,
        )
    );

    let body = &mut package["semantic_graph"]["nodes"][root]["body"];
    body["operation"]["member"]["operator"] = json!("fence");
    body["arguments"] = json!([{"term": "binding", "name": "ordering", "value": {
        "term": "literal", "type": node_id(&family_key("a1a1")), "value_kind": "text", "value": "seq_cst"
    }}]);
    settle(&mut package);
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    package["semantic_graph"]["nodes"][root]["body"]["arguments"][0]["value"]["value"] =
        json!("relaxed");
    assert_eq!(
        read(&package),
        refusal(
            CheckedPackageRefusalCode::MalformedWire,
            &format!("/semantic_graph/nodes/{root}/body/arguments/0/value"),
        )
    );
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn send_admits_channel_payload_and_checks_binder_type_and_condition() {
    let (package, root, channel) = package_with_send();
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));

    let mut wrong_channel = package.clone();
    wrong_channel["semantic_graph"]["nodes"][root]["body"]["arguments"][0]["value"]["target"] =
        wrong_channel["semantic_graph"]["nodes"][protocol_position(&wrong_channel, "role")]
            ["node_id"]
            .clone();
    settle(&mut wrong_channel);
    let key = wrong_channel["semantic_graph"]["nodes"][root]["node_id"]["digest"]
        .as_str()
        .expect("send key");
    assert_eq!(
        read(&wrong_channel),
        refusal_at(
            CheckedPackageRefusalCode::InvalidModelBinding,
            &format!("/semantic_graph/nodes/{root}/body/arguments/0/value/target"),
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            key,
        )
    );

    let mut wrong_type = package.clone();
    wrong_type["semantic_graph"]["nodes"][channel]["body"]["arguments"][2]["value"]["target"] =
        node_id(&family_key("a1a1"));
    settle(&mut wrong_type);
    let key = wrong_type["semantic_graph"]["nodes"][root]["node_id"]["digest"]
        .as_str()
        .expect("send key");
    assert_eq!(
        read(&wrong_type),
        refusal_at(
            CheckedPackageRefusalCode::IllTyped,
            &format!("/semantic_graph/nodes/{root}"),
            Some(CheckedPackageRefusalCause::TypeMismatch),
            key,
        )
    );
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&wrong_type) else {
        panic!("wrong payload type must refuse");
    };
    assert_eq!(
        refusal.protocol_detail.as_deref(),
        Some(&CheckedProtocolRefusalDetail::PayloadType {
            expected: serde_json::from_value(node_id(&family_key("a1a1"))).expect("text ID"),
            actual: serde_json::from_value(node_id(&family_key("aaaa"))).expect("boolean ID"),
        })
    );

    let mut non_boolean = package;
    non_boolean["semantic_graph"]["nodes"][root]["body"]["arguments"][3]["value"] = json!({
        "term": "literal", "type": node_id(&family_key("a1a1")), "value_kind": "text", "value": "yes"
    });
    settle(&mut non_boolean);
    let key = non_boolean["semantic_graph"]["nodes"][root]["node_id"]["digest"]
        .as_str()
        .expect("send key");
    assert_eq!(
        read(&non_boolean),
        refusal_at(
            CheckedPackageRefusalCode::IllTyped,
            &format!("/semantic_graph/nodes/{root}/body/arguments/3/value"),
            Some(CheckedPackageRefusalCause::NonBooleanRoot),
            key,
        )
    );
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn receive_requires_a_send_target_before_payload_type_checks() {
    let (mut package, root, _) = package_with_send();
    let role =
        package["semantic_graph"]["nodes"][protocol_position(&package, "role")]["node_id"].clone();
    let body = &mut package["semantic_graph"]["nodes"][root]["body"];
    body["operation"]["member"]["operator"] = json!("receive");
    let arguments = body["arguments"].as_array_mut().expect("send arguments");
    arguments.insert(
        1,
        json!({
            "term": "binding", "name": "of", "value": {"term": "reference", "target": role}
        }),
    );
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][root]["node_id"]["digest"]
        .as_str()
        .expect("receive key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidModelBinding,
            &format!("/semantic_graph/nodes/{root}/body/arguments/1/value/target"),
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            key,
        )
    );
}

/// Trace: FR-038-AC-288, FR-038-AC-290
#[trace("TC-048")]
#[test]
fn receive_accepts_its_send_channel_and_refuses_another_channel() {
    let (mut package, root, channel) = package_with_send();
    let send_body = package["semantic_graph"]["nodes"][root]["body"].clone();
    let send = protocol_named_node(
        "control",
        &["Example", "OrderFlow", "run", "Main", "Send"],
        send_body.clone(),
        &[],
    );
    let send_key = send["node_id"].clone();
    let send_position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(send);

    let mut receive_body = send_body;
    receive_body["operation"]["member"]["operator"] = json!("receive");
    receive_body["arguments"]
        .as_array_mut()
        .expect("arguments")
        .insert(
            1,
            json!({
                "term": "binding", "name": "of", "value": {"term": "reference", "target": send_key}
            }),
        );
    let receive_binder = clause_parameter(&mut package, "received", "2");
    receive_body["arguments"][2]["value"]["target"] = receive_binder;
    let receive = protocol_named_node(
        "control",
        &["Example", "OrderFlow", "run", "Main", "Receive"],
        receive_body,
        &[],
    );
    let receive_position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    let receive_key = receive["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(receive);

    let root_body = &mut package["semantic_graph"]["nodes"][root]["body"];
    root_body["operation"]["member"]["operator"] = json!("sequence");
    root_body["arguments"] = json!([{"term": "binding", "name": "children", "value": {
        "term": "aggregate", "members": [
            {"term": "reference", "target": send_key},
            {"term": "reference", "target": receive_key}
        ]
    }}]);
    relevel_finish_binder(&mut package, 3);
    settle(&mut package);
    assert!(
        matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
        "send and receive on one channel must admit locally: {:?}",
        read(&package)
    );

    let mut reversed = package.clone();
    reversed["semantic_graph"]["nodes"][root]["body"]["arguments"][0]["value"]["members"]
        .as_array_mut()
        .expect("sequence children")
        .swap(0, 1);
    settle(&mut reversed);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&reversed) else {
        panic!("receive sequenced before its send must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::DefinitionCycle)
    );
    let Some(CheckedProtocolRefusalDetail::CausalCycle { controls }) =
        refusal.protocol_detail.as_deref()
    else {
        panic!("cycle must name its controls");
    };
    let expected: Vec<quire_contract_model::CheckedNodeId> = [receive_position, send_position]
        .into_iter()
        .map(|position| {
            serde_json::from_value(reversed["semantic_graph"]["nodes"][position]["node_id"].clone())
                .expect("control identity")
        })
        .collect();
    assert_eq!(controls, &expected);

    let mut parallel = package.clone();
    parallel["semantic_graph"]["nodes"][root]["body"]["operation"]["member"]["operator"] =
        json!("parallel");
    parallel["semantic_graph"]["nodes"][root]["body"]["arguments"] = json!([
        {"term": "binding", "name": "memory", "value": {"term": "aggregate", "members": []}},
        {"term": "binding", "name": "branches", "value": {"term": "aggregate", "members": [
            {"term": "aggregate", "members": [
                {"term": "binding", "name": "label", "value": {"term": "literal", "type": node_id(&family_key("a1a1")), "value_kind": "text", "value": "Left"}},
                {"term": "binding", "name": "body", "value": {"term": "reference", "target": receive_key}}
            ]},
            {"term": "aggregate", "members": [
                {"term": "binding", "name": "label", "value": {"term": "literal", "type": node_id(&family_key("a1a1")), "value_kind": "text", "value": "Right"}},
                {"term": "binding", "name": "body", "value": {"term": "reference", "target": send_key}}
            ]}
        ]}},
        {"term": "binding", "name": "join", "value": {"term": "aggregate", "members": [
            {"term": "binding", "name": "policy", "value": {"term": "literal", "type": node_id(&family_key("a1a1")), "value_kind": "text", "value": "all"}}
        ]}},
        {"term": "binding", "name": "joined", "value": {"term": "aggregate", "members": [
            {"term": "literal", "type": node_id(&family_key("a1a1")), "value_kind": "text", "value": "Left"},
            {"term": "literal", "type": node_id(&family_key("a1a1")), "value_kind": "text", "value": "Right"}
        ]}},
        {"term": "binding", "name": "outstanding", "value": {"term": "aggregate", "members": []}}
    ]);
    let receive_binder = parallel["semantic_graph"]["nodes"][receive_position]["body"]["arguments"]
        [2]["value"]["target"]
        .clone();
    let receive_binder = relevel_parameter(&mut parallel, &receive_binder, 1);
    parallel["semantic_graph"]["nodes"][receive_position]["body"]["arguments"][2]["value"]
        ["target"] = receive_binder;
    settle(&mut parallel);
    assert!(
        matches!(read(&parallel), CheckedPackageV2ReadResult::Admitted(_)),
        "independent parallel send and receive must admit: {:?}",
        read(&parallel)
    );

    let mut wrong_binder = package.clone();
    let mut text_binder = wrong_binder["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "value" && node["semantic_form"] == "parameter")
        .expect("parameter")
        .clone();
    text_binder["semantic_type"] = node_id(&family_key("a1a1"));
    text_binder["body"]["members"][0]["value"]["value"] = json!("wrong_receive");
    let text_key = structural_key(
        "value",
        "parameter",
        Some(&family_key("a1a1")),
        &text_binder["body"],
    );
    text_binder["node_id"] = node_id(&text_key);
    wrong_binder["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(text_binder);
    wrong_binder["semantic_graph"]["nodes"][receive_position]["body"]["arguments"][2]["value"]
        ["target"] = node_id(&text_key);
    settle(&mut wrong_binder);
    let wrong_key = wrong_binder["semantic_graph"]["nodes"][receive_position]["node_id"]["digest"]
        .as_str()
        .expect("receive key");
    assert_eq!(
        read(&wrong_binder),
        refusal_at(
            CheckedPackageRefusalCode::IllTyped,
            &format!("/semantic_graph/nodes/{receive_position}"),
            Some(CheckedPackageRefusalCause::TypeMismatch),
            wrong_key,
        )
    );

    let other = protocol_named_node(
        "channel",
        &["Example", "OrderFlow", "channel", "Other"],
        package["semantic_graph"]["nodes"][channel]["body"].clone(),
        &[],
    );
    let other_key = other["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(other);
    let clause = clause_position(&package);
    let channels = package["semantic_graph"]["nodes"][clause]["body"]["arguments"][5]["value"]
        ["members"]
        .as_array_mut()
        .expect("channels");
    channels.push(json!({"term": "reference", "target": other_key}));
    channels.sort_by(|left, right| {
        left["target"]["digest"]
            .as_str()
            .cmp(&right["target"]["digest"].as_str())
    });
    package["semantic_graph"]["nodes"][receive_position]["body"]["arguments"][0]["value"]
        ["target"] = other_key;
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][receive_position]["node_id"]["digest"]
        .as_str()
        .expect("receive key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::IllTyped,
            &format!("/semantic_graph/nodes/{receive_position}"),
            Some(CheckedPackageRefusalCause::OperatorIneligible),
            key,
        )
    );
    assert_ne!(send_position, receive_position);
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn send_related_records_require_exact_shape_and_canonical_order() {
    let (mut package, root, _) = package_with_send();
    let role =
        package["semantic_graph"]["nodes"][protocol_position(&package, "role")]["node_id"].clone();
    let record = json!({"term": "aggregate", "members": [
        {"term": "binding", "name": "relationship", "value": {"term": "reference", "target": role}},
        {"term": "binding", "name": "from", "value": {"term": "reference", "target": role}},
        {"term": "binding", "name": "to", "value": {"term": "reference", "target": role}}
    ]});
    package["semantic_graph"]["nodes"][root]["body"]["arguments"][2]["value"]["members"] =
        json!([record.clone(), record]);
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][root]["node_id"]["digest"]
        .as_str()
        .expect("send key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            &format!("/semantic_graph/nodes/{root}"),
            None,
            key,
        )
    );

    package["semantic_graph"]["nodes"][root]["body"]["arguments"][2]["value"]["members"]
        .as_array_mut()
        .expect("related")
        .pop();
    package["semantic_graph"]["nodes"][root]["body"]["arguments"][2]["value"]["members"][0]
        ["members"][1]["name"] = json!("wrong");
    assert_eq!(
        read(&package),
        refusal(
            CheckedPackageRefusalCode::MalformedWire,
            &format!("/semantic_graph/nodes/{root}/body"),
        )
    );
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn send_related_requires_listed_relationship_and_typed_ends() {
    let (mut package, root, _) = package_with_send();
    let relationship = node_id(&family_key("2020"));
    let role =
        package["semantic_graph"]["nodes"][protocol_position(&package, "role")]["node_id"].clone();
    let record = |target: Value| {
        json!({"term": "aggregate", "members": [
            {"term": "binding", "name": "relationship", "value": {"term": "reference", "target": target}},
            {"term": "binding", "name": "from", "value": {"term": "literal",
                "type": node_id(&family_key("aaaa")), "value_kind": "boolean", "value": true}},
            {"term": "binding", "name": "to", "value": {"term": "literal",
                "type": node_id(&family_key("aaaa")), "value_kind": "boolean", "value": true}}
        ]})
    };
    package["semantic_graph"]["nodes"][root]["body"]["arguments"][2]["value"]["members"] =
        json!([record(role)]);
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][root]["node_id"]["digest"]
        .as_str()
        .expect("send key");
    assert_eq!(read(&package), refusal_at(
        CheckedPackageRefusalCode::InvalidModelBinding,
        &format!("/semantic_graph/nodes/{root}/body/arguments/2/value/members/0/members/0/value/target"),
        Some(CheckedPackageRefusalCause::WrongExport), key,
    ));

    package["semantic_graph"]["nodes"][root]["body"]["arguments"][2]["value"]["members"] =
        json!([record(relationship.clone())]);
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][root]["node_id"]["digest"]
        .as_str()
        .expect("send key");
    assert_eq!(read(&package), refusal_at(
        CheckedPackageRefusalCode::MissingDeclaration,
        &format!("/semantic_graph/nodes/{root}/body/arguments/2/value/members/0/members/0/value/target"),
        Some(CheckedPackageRefusalCause::MissingName), key,
    ));

    let clause = clause_position(&package);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][4]["value"]["members"] =
        json!([{"term": "reference", "target": relationship.clone()}]);
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][root]["node_id"]["digest"]
        .as_str()
        .expect("send key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::IllTyped,
            &format!(
                "/semantic_graph/nodes/{root}/body/arguments/2/value/members/0/members/1/value"
            ),
            Some(CheckedPackageRefusalCause::TypeMismatch),
            key,
        )
    );
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("wrong related end must refuse");
    };
    assert_eq!(
        refusal.protocol_detail.as_deref(),
        Some(&CheckedProtocolRefusalDetail::RelatedEndType {
            relationship: serde_json::from_value(relationship).expect("relationship ID"),
            end: "from".into(),
            expected: serde_json::from_value(node_id(&family_key("1515")))
                .expect("source endpoint ID"),
            actual: Some(
                serde_json::from_value(node_id(&family_key("aaaa"))).expect("Boolean type ID")
            ),
        })
    );
}

/// Trace: FR-038-AC-288
#[trace("TC-048")]
#[test]
fn send_related_accepts_selected_relationship_reference_ends() {
    let (mut package, root, channel) = package_with_send();
    let object = node_id(&family_key("1515"));
    let mut reference_type = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "composite_type" && node["semantic_form"] == "option")
        .expect("option type")
        .clone();
    reference_type["semantic_form"] = json!("reference");
    reference_type["body"] = json!({"term": "aggregate", "members": [
        {"term": "reference", "target": object}
    ]});
    let reference_key =
        structural_key("composite_type", "reference", None, &reference_type["body"]);
    reference_type["node_id"] = node_id(&reference_key);
    reference_type["semantic_type"] = node_id(&reference_key);
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(reference_type);

    let mut binder = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "value" && node["semantic_form"] == "parameter")
        .expect("parameter")
        .clone();
    binder["semantic_type"] = node_id(&reference_key);
    binder["body"]["members"][0]["value"]["value"] = json!("payload");
    binder["body"]["members"][1]["value"]["value"] = json!("2");
    let binder_key = structural_key("value", "parameter", Some(&reference_key), &binder["body"]);
    binder["node_id"] = node_id(&binder_key);
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(binder.clone());

    binder["body"]["members"][0]["value"]["value"] = json!("peer");
    binder["body"]["members"][1]["value"]["value"] = json!("1");
    let peer_key = structural_key("value", "parameter", Some(&reference_key), &binder["body"]);
    binder["node_id"] = node_id(&peer_key);
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(binder);

    package["semantic_graph"]["nodes"][channel]["body"]["arguments"][2]["value"]["target"] =
        node_id(&reference_key);
    package["semantic_graph"]["nodes"][root]["body"]["arguments"][1]["value"]["target"] =
        node_id(&binder_key);
    let relationship = node_id(&family_key("2020"));
    let related_end = json!({"term": "reference", "target": node_id(&binder_key)});
    package["semantic_graph"]["nodes"][root]["body"]["arguments"][2]["value"]["members"] = json!([{"term": "aggregate", "members": [
        {"term": "binding", "name": "relationship", "value": {"term": "reference", "target": relationship}},
        {"term": "binding", "name": "from", "value": related_end},
        {"term": "binding", "name": "to", "value": related_end}
    ]}]);
    let clause = clause_position(&package);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][4]["value"]["members"] =
        json!([{"term": "reference", "target": relationship}]);
    let role =
        package["semantic_graph"]["nodes"][protocol_position(&package, "role")]["node_id"].clone();
    let mut event_body = package["semantic_graph"]["nodes"][root]["body"].clone();
    event_body["operation"]["member"]["operator"] = json!("event");
    event_body["arguments"] = json!([
        {"term": "binding", "name": "role", "value": {"term": "reference", "target": role}},
        {"term": "binding", "name": "compensation", "value": {"term": "aggregate", "members": []}},
        {"term": "binding", "name": "binder", "value": {"term": "reference", "target": node_id(&peer_key)}},
        {"term": "binding", "name": "related", "value": {"term": "aggregate", "members": []}},
        {"term": "binding", "name": "constraint", "value": {"term": "literal",
            "type": node_id(&family_key("aaaa")), "value_kind": "boolean", "value": true}}
    ]);
    let event = protocol_named_node(
        "control",
        &["Example", "OrderFlow", "run", "Main", "Peer"],
        event_body,
        &[],
    );
    let event_key = event["node_id"].clone();
    let send_key = package["semantic_graph"]["nodes"][root]["node_id"].clone();
    package["semantic_graph"]["nodes"][root]["declaration"]["qualified_name"] =
        json!(["Example", "OrderFlow", "run", "Main", "Send"]);
    let mut sequence_body = package["semantic_graph"]["nodes"][root]["body"].clone();
    sequence_body["operation"]["member"]["operator"] = json!("sequence");
    sequence_body["arguments"] = json!([{"term": "binding", "name": "children", "value": {
    "term": "aggregate", "members": [
        {"term": "reference", "target": event_key},
        {"term": "reference", "target": send_key}
    ]}}]);
    let sequence = protocol_named_node(
        "control",
        &["Example", "OrderFlow", "run", "Main"],
        sequence_body,
        &[],
    );
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][11]["value"]["target"] =
        sequence["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend([event, sequence]);
    relevel_finish_binder(&mut package, 3);
    settle(&mut package);
    assert!(
        matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
        "selected relationship ends of the binder's reference type must admit: {:?}",
        read(&package)
    );

    let record = |to: &str| {
        json!({"term": "aggregate", "members": [
            {"term": "binding", "name": "relationship", "value": {"term": "reference", "target": relationship}},
            {"term": "binding", "name": "from", "value": {"term": "reference", "target": node_id(&binder_key)}},
            {"term": "binding", "name": "to", "value": {"term": "reference", "target": node_id(to)}}
        ]})
    };
    let (first, second) = if binder_key < peer_key {
        (record(&binder_key), record(&peer_key))
    } else {
        (record(&peer_key), record(&binder_key))
    };
    let related =
        &mut package["semantic_graph"]["nodes"][root]["body"]["arguments"][2]["value"]["members"];
    *related = json!([first.clone(), second.clone()]);
    settle(&mut package);
    assert!(
        matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
        "distinct related records in JCS order must admit: {:?}",
        read(&package)
    );

    package["semantic_graph"]["nodes"][root]["body"]["arguments"][2]["value"]["members"] =
        json!([second, first.clone()]);
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][root]["node_id"]["digest"]
        .as_str()
        .expect("send key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            &format!("/semantic_graph/nodes/{root}"),
            None,
            key,
        )
    );

    let mut bad_to = first;
    bad_to["members"][2]["value"] = json!({
        "term": "literal", "type": node_id(&family_key("aaaa")),
        "value_kind": "boolean", "value": true
    });
    package["semantic_graph"]["nodes"][root]["body"]["arguments"][2]["value"]["members"] =
        json!([bad_to]);
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][root]["node_id"]["digest"]
        .as_str()
        .expect("send key");
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("wrong to endpoint must refuse");
    };
    assert_eq!(
        refusal,
        refusal_at(
            CheckedPackageRefusalCode::IllTyped,
            &format!(
                "/semantic_graph/nodes/{root}/body/arguments/2/value/members/0/members/2/value"
            ),
            Some(CheckedPackageRefusalCause::TypeMismatch),
            key,
        )
    );
    assert_eq!(
        refusal.protocol_detail.as_deref(),
        Some(&CheckedProtocolRefusalDetail::RelatedEndType {
            relationship: serde_json::from_value(relationship).expect("relationship ID"),
            end: "to".into(),
            expected: serde_json::from_value(node_id(&family_key("1515")))
                .expect("target endpoint ID"),
            actual: Some(
                serde_json::from_value(node_id(&family_key("aaaa"))).expect("Boolean type ID")
            ),
        })
    );
}

/// Trace: FR-038-AC-286
#[trace("TC-947")]
#[test]
fn protocol_queue_checks_channel_form() {
    let (mut package, queue) = package_with_member_queue();
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    package["semantic_graph"]["nodes"][queue]["body"]["arguments"][0]["value"]["target"] =
        node_id(&family_key("a4a4"));
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][queue]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidModelBinding,
            &format!("/semantic_graph/nodes/{queue}/body/arguments/0/value/target"),
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            key,
        )
    );
}

/// Trace: FR-038-AC-286
#[trace("TC-947")]
#[test]
fn protocol_queue_rejects_negative_literal_capacity() {
    let (mut package, queue) = package_with_member_queue();
    package["semantic_graph"]["nodes"][queue]["body"]["arguments"][1]["value"]["members"][1]
        ["value"]["value"] = json!("-1");
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][queue]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidPackage,
            &format!("/semantic_graph/nodes/{queue}/body/arguments/1/value/members/1/value"),
            Some(CheckedPackageRefusalCause::InvalidValue),
            key,
        )
    );
}

/// Trace: FR-038-AC-287
#[trace("TC-947")]
#[test]
fn protocol_finish_constraint_must_be_boolean() {
    let mut package = v2_all_families();
    let finish = protocol_position(&package, "obligation");
    package["semantic_graph"]["nodes"][finish]["body"]["arguments"][1]["value"] = json!({
        "term": "literal", "type": node_id(&family_key("a3a3")),
        "value_kind": "integer", "value": "1"
    });
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][finish]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::IllTyped,
            &format!("/semantic_graph/nodes/{finish}"),
            Some(CheckedPackageRefusalCause::NonBooleanRoot),
            key,
        )
    );
}

/// Trace: FR-038-AC-287
#[trace("TC-947")]
#[test]
fn protocol_terminal_when_requires_boolean_condition() {
    let mut package = v2_all_families();
    let finish = protocol_position(&package, "obligation");
    let mut body = package["semantic_graph"]["nodes"][finish]["body"].clone();
    body["operation"]["member"]["operator"] = json!("terminal");
    body["arguments"] = json!([
        {"term": "binding", "name": "mode", "value": {
            "term": "literal", "type": node_id(&family_key("a1a1")),
            "value_kind": "text", "value": "any"}},
        {"term": "binding", "name": "condition", "value": {
            "term": "aggregate", "members": []}},
    ]);
    let terminal = protocol_named_node(
        "obligation",
        &["Example", "OrderFlow", "terminal"],
        body,
        &[],
    );
    let key = terminal["node_id"].clone();
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(terminal);
    let clause = clause_position(&package);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][9]["value"]["members"] =
        json!([{"term": "reference", "target": key}]);
    settle(&mut package);
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    package["semantic_graph"]["nodes"][position]["body"]["arguments"][0]["value"]["value"] =
        json!("when");
    package["semantic_graph"]["nodes"][position]["body"]["arguments"][1]["value"]["members"] = json!([{"term": "literal", "type": node_id(&family_key("a3a3")),
            "value_kind": "integer", "value": "1"}]);
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][position]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::IllTyped,
            &format!("/semantic_graph/nodes/{position}"),
            Some(CheckedPackageRefusalCause::NonBooleanRoot),
            key,
        )
    );
}

/// Trace: FR-038-AC-286
#[trace("TC-947")]
#[test]
fn protocol_channel_checks_endpoint_and_message_type() {
    let (mut package, _) = package_with_member_queue();
    let channel = protocol_position(&package, "channel");
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    package["semantic_graph"]["nodes"][channel]["body"]["arguments"][2]["value"]["target"] =
        node_id(&family_key("a4a4"));
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][channel]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidModelBinding,
            &format!("/semantic_graph/nodes/{channel}/body/arguments/2/value/target"),
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            key,
        )
    );
}

/// Trace: FR-038-AC-286
#[trace("TC-947")]
#[test]
fn protocol_channel_admits_fifo_with_message_binder_and_stable_key() {
    let (mut package, _) = package_with_member_queue();
    let channel = protocol_position(&package, "channel");
    package["semantic_graph"]["nodes"][channel]["body"]["arguments"][3]["value"]["members"] = json!([
        {"term": "binding", "name": "kind", "value": {
            "term": "literal", "type": node_id(&family_key("a1a1")),
            "value_kind": "text", "value": "fifo"}},
        {"term": "binding", "name": "binder", "value": {
            "term": "reference", "target": node_id(&family_key("p-close"))}},
        {"term": "binding", "name": "key", "value": {
            "term": "literal", "type": node_id(&family_key("aaaa")),
            "value_kind": "boolean", "value": true}},
    ]);
    settle(&mut package);
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

/// Trace: FR-038-AC-286
#[trace("TC-947")]
#[test]
fn protocol_channel_fifo_refuses_float_key() {
    let (mut package, _) = package_with_member_queue();
    let channel = protocol_position(&package, "channel");
    let body = json!({"term": "aggregate", "members": []});
    let digest = structural_key("scalar_type", "float32", None, &body);
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(json!({
            "node_id": node_id(&digest),
            "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": "scalar_type", "semantic_form": "float32",
            "semantic_type": node_id(&digest), "dependencies": [],
            "occurrences": [{"role": "generated", "ordinal": 0}],
            "body": body,
        }));
    package["semantic_graph"]["nodes"][channel]["body"]["arguments"][3]["value"]["members"] = json!([
        {"term": "binding", "name": "kind", "value": {
            "term": "literal", "type": node_id(&family_key("a1a1")),
            "value_kind": "text", "value": "fifo"}},
        {"term": "binding", "name": "binder", "value": {
            "term": "reference", "target": node_id(&family_key("p-close"))}},
        {"term": "binding", "name": "key", "value": {
            "term": "literal", "type": node_id(&digest),
            "value_kind": "float32_bits", "value": "3f800000"}},
    ]);
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][channel]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::IllTyped,
            &format!("/semantic_graph/nodes/{channel}/body/arguments/3/value/members/2/value"),
            Some(CheckedPackageRefusalCause::OperatorIneligible),
            key,
        )
    );
}

/// Trace: FR-038-AC-281, FR-038-AC-284
#[trace("TC-947")]
#[test]
fn protocol_role_checks_model_form_at_target() {
    let (mut package, position) = package_with_member_role();
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    let wrong = node_id(&family_key("a4a4"));
    package["semantic_graph"]["nodes"][position]["body"]["arguments"][0]["value"]["target"] = wrong;
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][position]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidModelBinding,
            &format!("/semantic_graph/nodes/{position}/body/arguments/0/value/target"),
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            key,
        )
    );
}

/// Trace: FR-038-AC-282, FR-038-AC-286
#[trace("TC-947")]
#[test]
fn protocol_whole_model_role_admits_selected_text_without_model_dependency() {
    let (mut package, role) = whole_model_role("test/families");
    assert_eq!(
        package["semantic_graph"]["nodes"][role]["dependencies"],
        json!([])
    );
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));

    let first_key = package["semantic_graph"]["nodes"][role]["node_id"].clone();
    let other = domain_package_document("test/other", "1", Vec::new());
    package["lock"]["model_selections"]
        .as_array_mut()
        .expect("model selections")
        .push(json!({
            "identity": "test/other", "digest_domain": "sha256-jcs",
            "digest": domain_package_digest(&other),
        }));
    package["semantic_graph"]["nodes"][role]["body"]["arguments"][0]["value"]["value"] =
        json!("test/other");
    settle(&mut package);
    assert_ne!(
        package["semantic_graph"]["nodes"][role]["node_id"],
        first_key
    );
    assert_eq!(
        package["semantic_graph"]["nodes"][role]["dependencies"],
        json!([])
    );
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

/// Trace: FR-038-AC-282, FR-038-AC-286
#[trace("TC-947")]
#[test]
fn protocol_clause_admits_two_whole_model_roles_across_over_types() {
    for over in [family_key("a3a3"), family_key("a1a1")] {
        let (mut package, first) = whole_model_role("test/families");
        let mut over_parameter = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .find(|node| {
                node["node_tag"] == "value"
                    && node["semantic_form"] == "parameter"
                    && node["body"]["members"][0]["value"]["value"] == "over"
            })
            .expect("over parameter")
            .clone();
        over_parameter["semantic_type"] = node_id(&over);
        let over_key = structural_key("value", "parameter", Some(&over), &over_parameter["body"]);
        over_parameter["node_id"] = node_id(&over_key);
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(over_parameter);
        let other = domain_package_document("test/other", "1", Vec::new());
        package["lock"]["model_selections"]
            .as_array_mut()
            .expect("model selections")
            .push(json!({
                "identity": "test/other", "digest_domain": "sha256-jcs",
                "digest": domain_package_digest(&other),
            }));

        let mut second_body = package["semantic_graph"]["nodes"][first]["body"].clone();
        second_body["arguments"][0]["value"]["value"] = json!("test/other");
        let second_node = protocol_named_node(
            "role",
            &["Example", "OrderFlow", "role", "Other"],
            second_body,
            &[],
        );
        let second = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .len();
        let first_key = package["semantic_graph"]["nodes"][first]["node_id"].clone();
        let second_key = second_node["node_id"].clone();
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(second_node);
        let mut role_keys = [first_key.clone(), second_key.clone()];
        role_keys.sort_by(|a, b| a["digest"].as_str().cmp(&b["digest"].as_str()));
        let clause = clause_position(&package);
        package["semantic_graph"]["nodes"][clause]["body"]["arguments"][0]["value"] =
            json!({"term": "reference", "target": node_id(&over_key)});
        package["semantic_graph"]["nodes"][clause]["body"]["arguments"][3]["value"]["members"] =
            json!(role_keys
                .iter()
                .map(|key| json!({"term": "reference", "target": key}))
                .collect::<Vec<_>>());
        settle(&mut package);

        assert_ne!(first_key, second_key);
        assert_ne!(first_key, protocol_role_node()["node_id"]);
        assert_eq!(
            package["semantic_graph"]["nodes"][first]["dependencies"],
            json!([])
        );
        assert_eq!(
            package["semantic_graph"]["nodes"][second]["dependencies"],
            json!([])
        );
        assert!(
            matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
            "typed over parameter should admit: {:?}",
            read(&package)
        );

        package["lock"]["model_selections"]
            .as_array_mut()
            .expect("model selections")
            .pop();
        refresh_identity(&mut package);
        let key = package["semantic_graph"]["nodes"][second]["node_id"]["digest"]
            .as_str()
            .expect("role key");
        assert_eq!(
            read(&package),
            refusal_at(
                CheckedPackageRefusalCode::MissingDeclaration,
                &format!("/semantic_graph/nodes/{second}"),
                Some(CheckedPackageRefusalCause::MissingSelection),
                key,
            )
        );
    }
}

/// Trace: FR-038-AC-282, FR-038-AC-286
#[trace("TC-947")]
#[test]
fn protocol_whole_model_role_checks_selection_before_replication() {
    let (mut package, role) = package_with_replicated_role();
    package["semantic_graph"]["nodes"][role]["body"]["arguments"][0]["value"] = json!({
        "term": "literal", "type": node_id(&family_key("a1a1")),
        "value_kind": "text", "value": "test/missing",
    });
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][role]["node_id"]["digest"]
        .as_str()
        .expect("role key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::MissingDeclaration,
            &format!("/semantic_graph/nodes/{role}"),
            Some(CheckedPackageRefusalCause::MissingSelection),
            key,
        )
    );

    package["semantic_graph"]["nodes"][role]["body"]["arguments"][0]["value"]["value"] =
        json!("test/families");
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][role]["node_id"]["digest"]
        .as_str()
        .expect("role key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidModelBinding,
            &format!("/semantic_graph/nodes/{role}"),
            Some(CheckedPackageRefusalCause::MalformedDeclaration),
            key,
        )
    );
}

/// Trace: FR-038-AC-282, FR-038-AC-286
#[trace("TC-947")]
#[test]
fn protocol_role_model_rejects_non_reference_non_text_wire() {
    let (mut package, role) = package_with_member_role();
    package["semantic_graph"]["nodes"][role]["body"]["arguments"][0]["value"] = json!({
        "term": "literal", "type": node_id(&family_key("a3a3")),
        "value_kind": "integer", "value": "1"
    });
    settle(&mut package);
    assert_eq!(
        read(&package),
        refusal(
            CheckedPackageRefusalCode::MalformedWire,
            &format!("/semantic_graph/nodes/{role}/body"),
        )
    );
}

/// Trace: FR-038-AC-281, FR-038-AC-284
#[trace("TC-947")]
#[test]
fn protocol_role_dependency_precedes_wrong_model_form() {
    let (mut package, position) = package_with_member_role();
    package["semantic_graph"]["nodes"][position]["body"]["arguments"][0]["value"]["target"] =
        node_id(&family_key("a4a4"));
    settle(&mut package);
    package["semantic_graph"]["nodes"][position]["dependencies"] = json!([]);
    refresh_identity(&mut package);
    let key = package["semantic_graph"]["nodes"][position]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::MissingDeclaration,
            &format!("/semantic_graph/nodes/{position}/body/arguments/0/value/target"),
            Some(CheckedPackageRefusalCause::MissingName),
            key,
        )
    );
}

/// Trace: FR-038-AC-286
#[trace("TC-947")]
#[test]
fn protocol_role_max_precedes_wrong_replication_binder_type() {
    let (mut package, role) = package_with_member_role();
    let text = |value: &str| {
        json!({
            "term": "literal", "type": node_id(&family_key("a1a1")),
            "value_kind": "text", "value": value
        })
    };
    package["semantic_graph"]["nodes"][role]["body"]["arguments"][1]["value"]["members"] = json!([
        {"term": "binding", "name": "population", "value": text("orders")},
        {"term": "binding", "name": "max", "value": {
            "term": "literal", "type": node_id(&family_key("a3a3")),
            "value_kind": "integer", "value": "0"}},
        {"term": "binding", "name": "binder", "value": {
            "term": "reference", "target": node_id(&family_key("p-close"))}},
        {"term": "binding", "name": "lifetime", "value": text("workflow")},
    ]);
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][role]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::IllTyped,
            &format!("/semantic_graph/nodes/{role}/body/arguments/1/value/members/1/value"),
            Some(CheckedPackageRefusalCause::TypeMismatch),
            key,
        )
    );
}

/// Trace: FR-038-AC-286
#[trace("TC-947")]
#[test]
fn protocol_replicated_role_admits_positive_max_and_reference_binder() {
    let (package, _) = package_with_replicated_role();
    assert!(matches!(
        read(&package),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
}

/// Trace: FR-038-AC-286
#[trace("TC-947")]
#[test]
fn protocol_replicated_role_rejects_wrong_binder_type_after_positive_max() {
    let (mut package, role) = package_with_replicated_role();
    package["semantic_graph"]["nodes"][role]["body"]["arguments"][1]["value"]["members"][2]
        ["value"]["target"] = node_id(&family_key("p-close"));
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][role]["node_id"]["digest"]
        .as_str()
        .expect("digest");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::IllTyped,
            &format!("/semantic_graph/nodes/{role}"),
            Some(CheckedPackageRefusalCause::TypeMismatch),
            key,
        )
    );
}

/// Trace: FR-038-AC-285, FR-038-AC-286
#[trace("TC-947")]
#[test]
fn protocol_queue_rejects_foreign_clause_channel() {
    let (mut package, queue) = package_with_member_queue();
    let original_role = protocol_position(&package, "role");
    let original_channel = protocol_position(&package, "channel");
    let original_root = protocol_position(&package, "control");
    let original_finish = protocol_position(&package, "obligation");
    let original_clause = clause_position(&package);

    let foreign_role = protocol_named_node(
        "role",
        &["Example", "Elsewhere", "role", "Worker"],
        package["semantic_graph"]["nodes"][original_role]["body"].clone(),
        &[],
    );
    let foreign_role_key = foreign_role["node_id"].clone();
    let mut foreign_channel_body =
        package["semantic_graph"]["nodes"][original_channel]["body"].clone();
    for ordinal in [0, 1] {
        foreign_channel_body["arguments"][ordinal]["value"]["target"] = foreign_role_key.clone();
    }
    let foreign_channel = protocol_named_node(
        "channel",
        &["Example", "Elsewhere", "channel", "Link"],
        foreign_channel_body,
        &[],
    );
    let foreign_channel_key = foreign_channel["node_id"].clone();
    let foreign_root = protocol_named_node(
        "control",
        &["Example", "Elsewhere", "run", "Main"],
        package["semantic_graph"]["nodes"][original_root]["body"].clone(),
        &[],
    );
    let foreign_root_key = foreign_root["node_id"].clone();
    let foreign_finish = protocol_named_node(
        "obligation",
        &["Example", "Elsewhere", "finish", "Closed"],
        package["semantic_graph"]["nodes"][original_finish]["body"].clone(),
        &[],
    );
    let foreign_finish_key = foreign_finish["node_id"].clone();
    let mut foreign_clause_body =
        package["semantic_graph"]["nodes"][original_clause]["body"].clone();
    for (ordinal, target) in [(3, foreign_role_key), (5, foreign_channel_key.clone())] {
        foreign_clause_body["arguments"][ordinal]["value"]["members"] =
            json!([{"term": "reference", "target": target}]);
    }
    foreign_clause_body["arguments"][6]["value"]["members"] = json!([]);
    foreign_clause_body["arguments"][11]["value"]["target"] = foreign_root_key;
    foreign_clause_body["arguments"][12]["value"]["target"] = foreign_finish_key;
    let foreign_clause = protocol_named_node(
        "protocol_clause",
        &["Example", "Elsewhere"],
        foreign_clause_body,
        &[],
    );
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend([
            foreign_role,
            foreign_channel,
            foreign_root,
            foreign_finish,
            foreign_clause,
        ]);

    let mut second_queue = package["semantic_graph"]["nodes"][queue].clone();
    second_queue["body"]["arguments"][1]["value"]["members"][1]["value"]["value"] = json!("5");
    second_queue["node_id"] = node_id(&family_key("queue-original-placeholder"));
    let second_queue_position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(second_queue);
    rekey_application_node(&mut package, second_queue_position);
    let second_queue_key =
        package["semantic_graph"]["nodes"][second_queue_position]["node_id"].clone();
    let original_queue_key = package["semantic_graph"]["nodes"][queue]["node_id"].clone();
    package["semantic_graph"]["nodes"][original_clause]["body"]["arguments"][6]["value"]
        ["members"] = json!([
        {"term": "reference", "target": original_queue_key},
        {"term": "reference", "target": second_queue_key}
    ]);
    package["semantic_graph"]["nodes"][original_clause]["body"]["arguments"][6]["value"]["members"]
        .as_array_mut()
        .expect("queues")
        .sort_by(|left, right| {
            left["target"]["digest"]
                .as_str()
                .cmp(&right["target"]["digest"].as_str())
        });
    package["semantic_graph"]["nodes"][queue]["body"]["arguments"][0]["value"]["target"] =
        foreign_channel_key;
    settle(&mut package);
    package["semantic_graph"]["nodes"][original_clause]["body"]["arguments"][6]["value"]["members"]
        .as_array_mut()
        .expect("queues")
        .sort_by(|left, right| {
            left["target"]["digest"]
                .as_str()
                .cmp(&right["target"]["digest"].as_str())
        });
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][queue]["node_id"]["digest"]
        .as_str()
        .expect("queue key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            &format!("/semantic_graph/nodes/{queue}"),
            None,
            key,
        )
    );
}

/// Trace: FR-038-AC-285
#[trace("TC-048")]
#[test]
fn protocol_membership_refuses_unlisted_role_and_repeated_control_holder() {
    let (mut unlisted, role) = package_with_member_role();
    let clause = clause_position(&unlisted);
    unlisted["semantic_graph"]["nodes"][clause]["body"]["arguments"][3]["value"]["members"] =
        json!([]);
    settle(&mut unlisted);
    let key = unlisted["semantic_graph"]["nodes"][role]["node_id"]["digest"]
        .as_str()
        .expect("role key");
    assert_eq!(
        read(&unlisted),
        refusal_at(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            &format!("/semantic_graph/nodes/{role}"),
            None,
            key,
        )
    );

    let (mut repeated_listing, listed_role) = package_with_member_role();
    let clause = clause_position(&repeated_listing);
    let role_target = repeated_listing["semantic_graph"]["nodes"][listed_role]["node_id"].clone();
    repeated_listing["semantic_graph"]["nodes"][clause]["body"]["arguments"][3]["value"]
        ["members"] = json!([
        {"term": "reference", "target": role_target},
        {"term": "reference", "target": role_target}
    ]);
    settle(&mut repeated_listing);
    let key = repeated_listing["semantic_graph"]["nodes"][listed_role]["node_id"]["digest"]
        .as_str()
        .expect("role key");
    assert_eq!(
        read(&repeated_listing),
        refusal_at(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            &format!("/semantic_graph/nodes/{listed_role}"),
            None,
            key,
        )
    );

    let mut repeated = v2_all_families();
    let root = protocol_position(&repeated, "control");
    let child = protocol_named_node(
        "control",
        &["Example", "OrderFlow", "run", "Main", "Child"],
        repeated["semantic_graph"]["nodes"][root]["body"].clone(),
        &[],
    );
    let child_key = child["node_id"].clone();
    let child_position = repeated["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    repeated["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(child);
    let root_body = &mut repeated["semantic_graph"]["nodes"][root]["body"];
    root_body["operation"]["member"]["operator"] = json!("sequence");
    root_body["arguments"] = json!([{"term": "binding", "name": "children", "value": {
        "term": "aggregate", "members": [
            {"term": "reference", "target": child_key},
            {"term": "reference", "target": child_key}
        ]
    }}]);
    settle(&mut repeated);
    let key = repeated["semantic_graph"]["nodes"][child_position]["node_id"]["digest"]
        .as_str()
        .expect("child key");
    assert_eq!(
        read(&repeated),
        refusal_at(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            &format!("/semantic_graph/nodes/{child_position}"),
            None,
            key,
        )
    );
}

/// Trace: FR-038-AC-285
#[trace("TC-048")]
#[test]
fn protocol_membership_requires_exactly_one_queue_per_channel() {
    let (without_queue, _, channel) = package_with_member_channel();
    let key = without_queue["semantic_graph"]["nodes"][channel]["node_id"]["digest"]
        .as_str()
        .expect("channel key");
    assert_eq!(
        read(&without_queue),
        refusal_at(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            &format!("/semantic_graph/nodes/{channel}"),
            None,
            key,
        )
    );

    let (mut two_queues, first) = package_with_member_queue();
    let mut second = two_queues["semantic_graph"]["nodes"][first].clone();
    second["body"]["arguments"][1]["value"]["members"][1]["value"]["value"] = json!("5");
    second["node_id"] = node_id(&family_key("queue-two-placeholder"));
    let second_position = two_queues["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    two_queues["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(second);
    rekey_application_node(&mut two_queues, second_position);
    let second_key = two_queues["semantic_graph"]["nodes"][second_position]["node_id"].clone();
    let clause = clause_position(&two_queues);
    two_queues["semantic_graph"]["nodes"][clause]["body"]["arguments"][6]["value"]["members"]
        .as_array_mut()
        .expect("queue list")
        .push(json!({"term": "reference", "target": second_key}));
    two_queues["semantic_graph"]["nodes"][clause]["body"]["arguments"][6]["value"]["members"]
        .as_array_mut()
        .expect("queue list")
        .sort_by(|left, right| {
            left["target"]["digest"]
                .as_str()
                .cmp(&right["target"]["digest"].as_str())
        });
    settle(&mut two_queues);
    let channel = protocol_position(&two_queues, "channel");
    let key = two_queues["semantic_graph"]["nodes"][channel]["node_id"]["digest"]
        .as_str()
        .expect("channel key");
    assert_eq!(
        read(&two_queues),
        refusal_at(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            &format!("/semantic_graph/nodes/{channel}"),
            None,
            key,
        )
    );
}

/// Trace: FR-038-AC-285
#[trace("TC-048")]
#[test]
fn await_anchor_reference_does_not_hold_an_unattached_control() {
    let (mut package, children) = protocol_holder_case("await");
    let event = children[0];
    let orphan = protocol_named_node(
        "control",
        &["Example", "OrderFlow", "run", "Main", "Detached"],
        package["semantic_graph"]["nodes"][event]["body"].clone(),
        &[],
    );
    let orphan_target = orphan["node_id"].clone();
    let orphan_position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(orphan);
    let holder = holder_position(&package);
    package["semantic_graph"]["nodes"][holder]["body"]["arguments"][0]["value"]["members"][1]
        ["value"]["target"] = orphan_target;
    settle(&mut package);
    let key = package["semantic_graph"]["nodes"][orphan_position]["node_id"]["digest"]
        .as_str()
        .expect("orphan key");
    assert_eq!(
        read(&package),
        refusal_at(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            &format!("/semantic_graph/nodes/{orphan_position}"),
            None,
            key,
        )
    );
}

/// Trace: FR-038-AC-290
#[trace("TC-048")]
#[test]
fn repeat_body_without_necessary_step_refuses_after_local_checks() {
    let (mut package, children) = protocol_holder_case("repeat");
    let body = &mut package["semantic_graph"]["nodes"][children[0]]["body"];
    body["operation"]["member"]["operator"] = json!("check");
    body["arguments"] = json!([{
        "term": "binding", "name": "condition", "value": {
            "term": "literal", "type": node_id(&family_key("aaaa")),
            "value_kind": "boolean", "value": true
        }
    }]);
    settle(&mut package);
    let holder = holder_position(&package);
    let key = package["semantic_graph"]["nodes"][holder]["node_id"]["digest"]
        .as_str()
        .expect("repeat key");
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("repeat without progress must refuse");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::UnsupportedConstruct
    );
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::DeclarationForm)
    );
    assert_eq!(
        refusal.path.as_ref().map(ToString::to_string),
        Some(format!("/semantic_graph/nodes/{holder}"))
    );
    assert_eq!(
        refusal.locus.as_ref().map(|id| id.digest.as_ref()),
        Some(key)
    );
    assert!(matches!(
        refusal.protocol_detail.as_deref(),
        Some(CheckedProtocolRefusalDetail::RepeatProgress { .. })
    ));
}

/// Trace: FR-038-AC-290
#[trace("TC-048")]
#[test]
fn await_anchored_to_its_own_event_refuses_causal_cycle() {
    let (mut package, children) = protocol_holder_case("await");
    let holder = holder_position(&package);
    let event = package["semantic_graph"]["nodes"][children[0]]["node_id"].clone();
    package["semantic_graph"]["nodes"][holder]["body"]["arguments"][0]["value"]["members"][1]
        ["value"]["target"] = event;
    settle(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("await after its own event must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::DefinitionCycle)
    );
    let Some(CheckedProtocolRefusalDetail::CausalCycle { controls }) =
        refusal.protocol_detail.as_deref()
    else {
        panic!("cycle must name its controls");
    };
    let expected: Vec<quire_contract_model::CheckedNodeId> = [holder, children[0]]
        .into_iter()
        .map(|position| {
            serde_json::from_value(package["semantic_graph"]["nodes"][position]["node_id"].clone())
                .expect("control identity")
        })
        .collect();
    assert_eq!(controls, &expected);
}

/// Trace: FR-038-AC-284, FR-038-AC-290
#[trace("TC-048")]
#[test]
fn await_after_later_sibling_cycles_before_clause_guard_slot() {
    let (mut package, children) = protocol_holder_case("await");
    let root = protocol_position(&package, "control");
    package["semantic_graph"]["nodes"][root]["body"]["arguments"][0]["value"]["members"]
        .as_array_mut()
        .expect("sequence children")
        .swap(0, 1);
    let trigger = clause_parameter(&mut package, "opened", "1");
    let clause = clause_position(&package);
    package["semantic_graph"]["nodes"][clause]["body"]["arguments"][1]["value"]["members"] = json!([
        {"term": "binding", "name": "kind", "value": {"term": "literal", "type": node_id(&family_key("a1a1")), "value_kind": "text", "value": "each"}},
        {"term": "binding", "name": "trigger", "value": {"term": "reference", "target": trigger}},
        {"term": "binding", "name": "guard", "value": {"term": "reference", "target": node_id(&family_key("a1a1"))}}
    ]);
    settle(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("await anchored at a later sibling must cycle");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::DefinitionCycle)
    );
    let Some(CheckedProtocolRefusalDetail::CausalCycle { controls }) =
        refusal.protocol_detail.as_deref()
    else {
        panic!("cycle must name the await and later anchor");
    };
    let holder = holder_position(&package);
    let expected: Vec<quire_contract_model::CheckedNodeId> = [holder, children[0], children[1]]
        .into_iter()
        .map(|position| {
            serde_json::from_value(package["semantic_graph"]["nodes"][position]["node_id"].clone())
                .expect("control identity")
        })
        .chain(std::iter::once(
            serde_json::from_value(
                package["semantic_graph"]["nodes"][root]["body"]["arguments"][0]["value"]
                    ["members"][1]["target"]
                    .clone(),
            )
            .expect("later anchor identity"),
        ))
        .collect();
    assert_eq!(controls, &expected);
}
