// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! QSpec TC-966 hybrid nodes read from the owning checkout, without a fixture copy.

use super::*;

fn assert_locus(
    refusal: &quire_contract_model::CheckedPackageRefusal,
    position: usize,
    suffix: &str,
) {
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some(format!("/semantic_graph/nodes/{position}{suffix}")),
        "{refusal:?}"
    );
}

fn published_hybrid_package() -> Value {
    let package = read_json(&fixtures_dir().join("positive-all-families.json"))
        .expect("QSpec published package");
    let hybrid = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("published nodes")
        .iter()
        .filter(|node| node["node_tag"] == "hybrid")
        .count();
    assert_eq!(hybrid, 3, "QSpec #415 Tank model, mode and transition");
    package
}

#[trace("TC-048", "FR-038-AC-311", "FR-038-AC-317")]
#[test]
fn qspec_hybrid_family_in_published_package_admits() {
    let package = published_hybrid_package();
    let read = read_package(&package).expect("published evidence");
    let CheckedPackageV2ReadResult::Admitted(admitted) = read else {
        panic!("QSpec hybrid package must admit: {read:?}");
    };
    assert_eq!(
        admitted
            .graph()
            .nodes
            .iter()
            .filter(|node| node.node_tag.as_ref() == "hybrid")
            .count(),
        3
    );
}

#[trace("TC-048", "FR-038-AC-311")]
#[test]
fn qspec_hybrid_schema_defects_refuse_before_identity() {
    for (form, change, locus) in [
        ("hybrid_transition", "declaration", ""),
        ("hybrid_mode", "occurrence_role", "/occurrences/0/role"),
        ("hybrid_model", "reordered_bindings", "/body/members/0"),
        (
            "hybrid_model",
            "class",
            "/body/members/0/value/members/0/members/1/value/value",
        ),
        (
            "hybrid_transition",
            "reference_target_index",
            "/body/members/0/value",
        ),
        (
            "hybrid_mode",
            "empty_rates",
            "/body/members/1/value/members",
        ),
        ("hybrid_model", "unknown_form", "/semantic_form"),
        ("hybrid_model", "missing_owner", ""),
        (
            "hybrid_model",
            "empty_initial",
            "/body/members/2/value/members",
        ),
        (
            "hybrid_transition",
            "invalid_reset_relation",
            "/body/members/2/value/members/0/members/2/value/value",
        ),
    ] {
        let mut package = published_hybrid_package();
        let position = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("published nodes")
            .iter()
            .position(|node| node["node_tag"] == "hybrid" && node["semantic_form"] == form)
            .expect("published hybrid form");
        let node = &mut package["semantic_graph"]["nodes"][position];
        match change {
            "declaration" => node["declaration"] = json!({"qualified_name":["Invalid"]}),
            "occurrence_role" => node["occurrences"][0]["role"] = json!("claim"),
            "class" => {
                node["body"]["members"][0]["value"]["members"][0]["members"][1]["value"]["value"] =
                    json!("continuous")
            }
            "reordered_bindings" => node["body"]["members"].as_array_mut().expect("body members").swap(0, 1),
            "reference_target_index" => {
                node["body"]["members"][0]["value"] = node["body"]["members"][1]["value"].clone();
            }
            "empty_rates" => node["body"]["members"][1]["value"]["members"] = json!([]),
            "unknown_form" => node["semantic_form"] = json!("hybrid_flow"),
            "missing_owner" => { node.as_object_mut().expect("node").remove("owner"); }
            "empty_initial" => node["body"]["members"].as_array_mut().expect("body members")
                .push(json!({"term":"binding", "name":"initial", "value":{"term":"aggregate", "members":[]}})),
            "invalid_reset_relation" => {
                let text_type = node["body"]["members"][2]["value"]["members"][0]["members"][0]
                    ["value"]["type"].clone();
                node["body"]["members"][2]["value"]["members"][0]["members"]
                    .as_array_mut().expect("reset members")
                    .push(json!({"term":"binding", "name":"relation", "value":{
                        "term":"literal", "type":text_type, "value_kind":"text", "value":"approx"}}));
            }
            _ => unreachable!("closed mutation list"),
        }
        let read = read_package(&package).expect("published evidence");
        let CheckedPackageV2ReadResult::Refused(refusal) = read else {
            panic!("{form} {change} must refuse: {read:?}");
        };
        assert_eq!(refusal.code, Code::MalformedWire, "{form} {change}");
        assert_eq!(refusal.cause, None, "{form} {change}");
        assert_eq!(
            refusal.loci[0].path.as_ref().map(ToString::to_string),
            Some(format!("/semantic_graph/nodes/{position}{locus}")),
            "{form} {change}: {refusal:?}"
        );
    }
}

fn model_position(package: &Value) -> usize {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_tag"] == "hybrid" && node["semantic_form"] == "hybrid_model")
        .expect("Tank model")
}

fn braking_package() -> Value {
    let mut package = published_hybrid_package();
    let vectors = read_json(&proposal_dir_of(&fixtures_dir()).join("hybrid-node-vectors.json"))
        .expect("QSpec TC-966 vectors");
    append_published_nodes(
        &mut package,
        vectors["nodes"].as_array().expect("QSpec nodes"),
    );
    package["lock"]["profile_selections"]
        .as_array_mut()
        .expect("profile selections")
        .push(json!({
            "role": "temporal_profile",
            "definition": {"authority": "agent-ix", "identity": "quire.temporal.infinite-trace/v1"},
        }));
    fixture_support::refresh_identity(&mut package);
    package
}

fn append_published_nodes(package: &mut Value, nodes: &[Value]) {
    let present = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("published nodes")
        .iter()
        .map(|node| node["node_id"].clone())
        .collect::<Vec<_>>();
    let source = package["lock"]["sources"][0].clone();
    for node in nodes {
        if present.contains(&node["node_id"]) {
            continue;
        }
        package["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("published nodes")
            .push(node.clone());
        for occurrence in node["occurrences"].as_array().expect("occurrences") {
            package["source_map"]
                .as_array_mut()
                .expect("source map")
                .push(json!({
                    "node_id": node["node_id"],
                    "role": occurrence["role"],
                    "ordinal": occurrence["ordinal"],
                    "regions": [{"source": source, "start": 0, "end": 1}],
                }));
        }
    }
}

fn restored_package() -> Value {
    let mut package = braking_package();
    let vectors = read_json(&proposal_dir_of(&fixtures_dir()).join("hybrid-node-vectors.json"))
        .expect("QSpec TC-966 vectors");
    append_published_nodes(
        &mut package,
        vectors["restored_nodes"]
            .as_array()
            .expect("restored nodes"),
    );
    package["lock"]["profile_selections"]
        .as_array_mut()
        .expect("profile selections")
        .push(json!({
            "role": "temporal_profile",
            "definition": {"authority": "agent-ix", "identity": "quire.temporal.timed/v1"},
        }));
    fixture_support::refresh_identity(&mut package);
    package
}

fn sort_dependencies(package: &mut Value) {
    for node in package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
    {
        sort_node_ids(node["dependencies"].as_array_mut().expect("dependencies"));
    }
}

fn refresh_hybrid_dependencies(package: &mut Value) {
    for node in package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
    {
        if node["node_tag"] != "hybrid" {
            continue;
        }
        let mut pending = vec![&node["body"]];
        let mut targets = std::collections::BTreeMap::new();
        while let Some(value) = pending.pop() {
            match value {
                Value::Object(members) => {
                    if members.get("term").and_then(Value::as_str) == Some("reference") {
                        let target = &value["target"];
                        targets.insert(typed_node_id(target), target.clone());
                    } else {
                        pending.extend(members.values());
                    }
                }
                Value::Array(items) => pending.extend(items),
                _ => {}
            }
        }
        node["dependencies"] = Value::Array(targets.into_values().collect());
    }
}

fn refresh_tank_hybrid_graph(package: &mut Value) {
    refresh_hybrid_dependencies(package);
    sort_dependencies(package);
    fixture_support::settle(package);
}

#[trace("TC-048", "FR-038-AC-312")]
#[test]
fn qspec_hybrid_body_edit_preserves_structural_identity_and_checks_body() {
    let mut package = published_hybrid_package();
    let model = model_position(&package);
    let original = package.clone();
    let id = package["semantic_graph"]["nodes"][model]["node_id"].clone();
    package["semantic_graph"]["nodes"][model]["body"]["members"][0]["value"]["members"][0]
        ["members"][1]["value"]["value"] = json!("continuous");
    assert_eq!(package["semantic_graph"]["nodes"][model]["node_id"], id);
    assert_ne!(package, original, "body edit changes the complete package");
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package(&package).expect("QSpec package")
    else {
        panic!("invalid hybrid class admitted");
    };
    assert_eq!(refusal.code, Code::MalformedWire);
    assert_locus(
        &refusal,
        model,
        "/body/members/0/value/members/0/members/1/value/value",
    );
}

#[trace("TC-048", "FR-038-AC-312")]
#[test]
fn qspec_hybrid_dependency_shape_precedes_body_checks() {
    let mut package = published_hybrid_package();
    let model = model_position(&package);
    let mode_id = package["semantic_graph"]["nodes"][model]["body"]["members"][1]["value"]
        ["members"][0]["target"]
        .clone();
    let mode = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_id"] == mode_id)
        .expect("mode");
    let repeated = package["semantic_graph"]["nodes"][mode]["body"]["members"][1]["value"]
        ["members"][0]
        .clone();
    package["semantic_graph"]["nodes"][mode]["body"]["members"][1]["value"]["members"]
        .as_array_mut()
        .expect("rates")
        .push(repeated);
    package["semantic_graph"]["nodes"][model]["dependencies"]
        .as_array_mut()
        .expect("dependencies")
        .pop();
    let CheckedPackageV2ReadResult::Refused(refusal) =
        read_package(&package).expect("QSpec package")
    else {
        panic!("independent graph and body defects admitted");
    };
    assert_eq!(refusal.code, Code::InvalidSemanticGraph, "{refusal:?}");
    assert_eq!(refusal.cause, None);
    assert_locus(&refusal, model, "/dependencies");
}

#[trace("TC-048", "FR-038-AC-316")]
#[test]
fn qspec_hybrid_capture_initializer_must_match_capture_type() {
    let mut package = restored_package();
    let claim = timed_claim_position(&package);
    let old = package["semantic_graph"]["nodes"][claim]["body"]["arguments"][3]["members"][0]
        ["value"]["target"]
        .clone();
    let boolean = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| {
            node["node_tag"] == "value"
                && node["semantic_form"] == "literal"
                && node["body"]["value_kind"] == "boolean"
        })
        .expect("boolean literal")["node_id"]
        .clone();
    package["semantic_graph"]["nodes"][claim]["body"]["arguments"][3]["members"][0]["value"]
        ["target"] = boolean.clone();
    let dependencies = package["semantic_graph"]["nodes"][claim]["dependencies"]
        .as_array_mut()
        .expect("dependencies");
    let dependency = dependencies
        .iter_mut()
        .find(|key| **key == old)
        .expect("capture dependency");
    *dependency = boolean;
    sort_node_ids(dependencies);
    let fair = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_tag"] == "temporal" && node["semantic_form"] == "fairness")
        .expect("fairness node");
    package["semantic_graph"]["nodes"][fair]["body"]["operation"]["member"]["name"] =
        json!("Halted");
    fixture_support::settle(&mut package);
    sort_dependencies(&mut package);
    fixture_support::settle(&mut package);
    fixture_support::refresh_identity(&mut package);
    let read = read_package(&package).expect("QSpec package");
    let CheckedPackageV2ReadResult::Refused(refusal) = read else {
        panic!("wrong capture type admitted")
    };
    assert_eq!(refusal.code, Code::IllTyped, "{refusal:?}");
    assert_eq!(refusal.cause, Some(Cause::TypeMismatch), "{refusal:?}");
    assert_locus(&refusal, claim, "/body/arguments/3/members/0");
}

#[trace("TC-048", "FR-038-AC-316")]
#[test]
fn qspec_hybrid_capture_ignores_unrelated_same_name_and_level() {
    let mut package = restored_package();
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    let capture = nodes
        .iter()
        .find(|node| {
            node["node_tag"] == "value"
                && node["semantic_form"] == "parameter"
                && node["body"]["members"][0]["value"]["value"] == "v0"
                && node["body"]["members"][1]["value"]["value"] == "6"
        })
        .expect("published capture parameter");
    let boolean = nodes
        .iter()
        .find(|node| node["node_tag"] == "scalar_type" && node["semantic_form"] == "boolean")
        .expect("Boolean scalar type")["node_id"]
        .clone();
    let mut unrelated = capture.clone();
    unrelated["node_id"] = fixture_support::structural_id(
        "example",
        vec![json!("value"), json!("parameter"), json!("unrelated")],
        0,
    );
    unrelated["semantic_type"] = boolean;
    append_published_nodes(&mut package, &[unrelated]);
    fixture_support::settle(&mut package);
    fixture_support::refresh_identity(&mut package);
    let read = read_package(&package).expect("QSpec package");
    assert!(
        matches!(read, CheckedPackageV2ReadResult::Admitted(_)),
        "{read:?}"
    );
}

fn timed_claim_position(package: &Value) -> usize {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| {
            node["body"]["operation"]["identity"] == "quire.op.temporal.clause"
                && node["body"]["operation"]["laws"][0]["definition"]["identity"]
                    == "quire.temporal.timed/v1"
        })
        .expect("Slows claim")
}

#[trace("TC-048", "FR-038-AC-313", "FR-038-AC-316", "FR-038-AC-317")]
#[test]
fn qspec_hybrid_braking_model_and_claim_admit_from_published_nodes() {
    let package = braking_package();
    let read = read_package(&package).expect("published evidence");
    assert!(
        matches!(read, CheckedPackageV2ReadResult::Admitted(_)),
        "{read:?}"
    );
}

#[trace("TC-048", "FR-038-AC-315", "FR-038-AC-317")]
#[test]
fn qspec_hybrid_restored_model_and_claims_admit_from_published_nodes() {
    let package = restored_package();
    let read = read_package(&package).expect("published evidence");
    assert!(
        matches!(read, CheckedPackageV2ReadResult::Admitted(_)),
        "{read:?}"
    );
}

#[trace("TC-048", "FR-038-AC-315")]
#[test]
fn qspec_hybrid_timed_open_upper_admits_under_timed_profile() {
    let mut package = restored_package();
    let clause = timed_claim_position(&package);
    let formula_id =
        package["semantic_graph"]["nodes"][clause]["body"]["arguments"][5]["target"].clone();
    let formula = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_id"] == formula_id)
        .expect("timed formula");
    let interval =
        &mut package["semantic_graph"]["nodes"][formula]["body"]["operation"]["member"]["interval"];
    interval["upper"] = Value::Null;
    interval["upper_end"] = json!("open");
    fixture_support::settle(&mut package);
    sort_dependencies(&mut package);
    fixture_support::settle(&mut package);
    fixture_support::refresh_identity(&mut package);
    let read = read_package(&package).expect("published evidence");
    assert!(
        matches!(read, CheckedPackageV2ReadResult::Admitted(_)),
        "{read:?}"
    );
}

#[trace("TC-048", "FR-038-AC-315")]
#[test]
fn qspec_hybrid_package_keeps_nonhybrid_timed_null_wire_refusal() {
    let mut package = braking_package();
    let formula = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| {
            node["node_tag"] == "temporal"
                && node["semantic_form"] == "formula"
                && node["body"]["operation"]["identity"] == "quire.op.temporal.eventually"
                && node["body"]["operation"]["member"]["interval"]["upper"] == "3"
        })
        .expect("published nonhybrid formula");
    package["semantic_graph"]["nodes"][formula]["body"]["operation"]["member"]["interval"] = json!({"lower":{"numerator":"0","denominator":"1"}, "upper":null,
               "lower_end":"closed", "upper_end":"open"});
    let read = read_package(&package).expect("QSpec package");
    let CheckedPackageV2ReadResult::Refused(refusal) = read else {
        panic!("nonhybrid timed null admitted: {read:?}")
    };
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::InvalidPackage, Some(Cause::InvalidValue)),
        "{refusal:?}"
    );
    assert_locus(&refusal, formula, "/body/operation/member/interval/upper");
}

#[trace("TC-048", "FR-038-AC-316")]
#[test]
fn qspec_hybrid_claim_refuses_clock_outside_model_vocabulary() {
    let mut package = braking_package();
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| {
            node["node_tag"] == "temporal"
                && node["semantic_form"] == "temporal_clause"
                && node["body"]["operation"]["laws"][0]["definition"]["identity"]
                    == "quire.temporal.infinite-trace/v1"
        })
        .expect("SlowEnough claim");
    package["semantic_graph"]["nodes"][position]["body"]["arguments"][1]["value"] =
        json!("wall-time");
    fixture_support::settle(&mut package);
    fixture_support::refresh_identity(&mut package);
    let read = read_package(&package).expect("published evidence");
    let CheckedPackageV2ReadResult::Refused(refusal) = read else {
        panic!("invalid hybrid clock admitted");
    };
    assert_eq!(refusal.code, Code::InvalidPackage, "{refusal:?}");
    assert_eq!(refusal.cause, Some(Cause::InvalidValue), "{refusal:?}");
    assert_locus(&refusal, position, "/body/arguments/1");
}

#[trace("TC-048", "FR-038-AC-316")]
#[test]
fn qspec_hybrid_claim_refuses_unknown_fairness_mode() {
    let mut package = restored_package();
    let fair = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_tag"] == "temporal" && node["semantic_form"] == "fairness")
        .expect("fairness node");
    package["semantic_graph"]["nodes"][fair]["body"]["operation"]["member"]["name"] =
        json!("Halted");
    fixture_support::settle(&mut package);
    sort_dependencies(&mut package);
    let claim_position = timed_claim_position(&package);
    fixture_support::settle(&mut package);
    fixture_support::refresh_identity(&mut package);
    let read = read_package(&package).expect("published evidence");
    let CheckedPackageV2ReadResult::Refused(refusal) = read else {
        panic!("unknown hybrid fairness mode admitted");
    };
    assert_eq!(refusal.code, Code::MissingDeclaration, "{refusal:?}");
    assert_eq!(refusal.cause, Some(Cause::MissingName), "{refusal:?}");
    assert_locus(&refusal, fair, "/body/operation/member/name");
}

#[trace("TC-048", "FR-038-AC-316")]
#[test]
fn qspec_hybrid_claim_refuses_trigger_typed_as_scalar() {
    let mut package = restored_package();
    let claim_position = timed_claim_position(&package);
    let model_key = package["semantic_graph"]["nodes"][claim_position]["body"]["arguments"][0]
        ["target"]
        .clone();
    let over_position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_id"] == model_key)
        .expect("over parameter");
    let hybrid_model_id =
        package["semantic_graph"]["nodes"][over_position]["semantic_type"].clone();
    let model_position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_id"] == hybrid_model_id)
        .expect("hybrid model");
    let variable = package["semantic_graph"]["nodes"][model_position]["body"]["members"][0]
        ["value"]["members"][0]["members"][0]["value"]["target"]
        .clone();
    package["semantic_graph"]["nodes"][claim_position]["body"]["arguments"][2]["members"][1]
        ["value"]["target"] = variable.clone();
    fixture_support::settle(&mut package);
    fixture_support::refresh_identity(&mut package);
    let read = read_package(&package).expect("published evidence");
    let CheckedPackageV2ReadResult::Refused(refusal) = read else {
        panic!("scalar-typed trigger admitted");
    };
    assert_eq!(refusal.code, Code::IllTyped, "{refusal:?}");
    assert_eq!(refusal.cause, Some(Cause::TypeMismatch), "{refusal:?}");
    assert_locus(&refusal, claim_position, "/body/arguments/2/members");
}

#[trace("TC-048", "FR-038-AC-316")]
#[test]
fn qspec_hybrid_claim_refuses_holds_read_of_over_parameter() {
    let mut package = braking_package();
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    let clause = nodes
        .iter()
        .position(|node| {
            node["body"]["operation"]["identity"] == "quire.op.temporal.clause"
                && node["body"]["operation"]["laws"][0]["definition"]["identity"]
                    == "quire.temporal.infinite-trace/v1"
        })
        .expect("SlowEnough");
    let always_id = nodes[clause]["body"]["arguments"][5]["target"].clone();
    let always = nodes
        .iter()
        .position(|node| node["node_id"] == always_id)
        .expect("always formula");
    let holds_id = nodes[always]["body"]["arguments"][0]["target"].clone();
    let holds = nodes
        .iter()
        .position(|node| node["node_id"] == holds_id)
        .expect("holds formula");
    let over = package["semantic_graph"]["nodes"][clause]["body"]["arguments"][0]["target"].clone();
    package["semantic_graph"]["nodes"][holds]["body"]["arguments"][0]["target"] = over.clone();
    package["semantic_graph"]["nodes"][holds]["dependencies"] = json!([over]);
    fixture_support::settle(&mut package);
    sort_dependencies(&mut package);
    fixture_support::settle(&mut package);
    sort_dependencies(&mut package);
    fixture_support::settle(&mut package);
    fixture_support::refresh_identity(&mut package);
    let read = read_package(&package).expect("published evidence");
    let CheckedPackageV2ReadResult::Refused(refusal) = read else {
        panic!("holds read of over parameter admitted");
    };
    assert_eq!(refusal.code, Code::MissingDeclaration, "{refusal:?}");
    assert_eq!(refusal.cause, Some(Cause::MissingName), "{refusal:?}");
    assert_locus(&refusal, holds, "");
}

#[trace("TC-048", "FR-038-AC-313", "FR-038-AC-314")]
#[test]
fn qspec_hybrid_model_checks_duplicate_modes_and_clock_rates() {
    for change in ["duplicate_mode", "clock_non_literal_rate"] {
        let mut package = published_hybrid_package();
        let position = model_position(&package);
        let model = &mut package["semantic_graph"]["nodes"][position];
        match change {
            "duplicate_mode" => {
                let first = model["body"]["members"][1]["value"]["members"][0].clone();
                model["body"]["members"][1]["value"]["members"]
                    .as_array_mut()
                    .expect("mode refs")
                    .push(first);
            }
            "clock_non_literal_rate" => {
                model["body"]["members"][0]["value"]["members"][0]["members"][1]["value"]
                    ["value"] = json!("clock");
            }
            _ => unreachable!("closed changes"),
        }
        fixture_support::settle(&mut package);
        let read = read_package(&package).expect("published evidence");
        let CheckedPackageV2ReadResult::Refused(refusal) = read else {
            panic!("{change} must refuse: {read:?}");
        };
        assert_eq!(
            refusal.code,
            if change == "duplicate_mode" {
                Code::InvalidSemanticGraph
            } else {
                Code::InvalidPackage
            },
            "{change}: {refusal:?}"
        );
        assert_locus(
            &refusal,
            position,
            if change == "duplicate_mode" {
                "/body/members/1/value/members/1"
            } else {
                "/body/members/0/value/members/0/members/1/value/value"
            },
        );
    }
}

/// A clock may use the exact rational one rate. A different rational value
/// refuses at the selected variable class after the package keys are rebuilt.
#[trace("TC-048", "FR-038-AC-314")]
#[test]
fn qspec_hybrid_clock_rate_requires_exact_rational_one() {
    let mut package = published_hybrid_package();
    let model = model_position(&package);
    package["semantic_graph"]["nodes"][model]["body"]["members"][0]["value"]["members"][1]
        ["members"][1]["value"]["value"] = json!("clock");
    let mode_id = package["semantic_graph"]["nodes"][model]["body"]["members"][1]["value"]
        ["members"][0]["target"]
        .clone();
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    let mode = nodes
        .iter()
        .position(|node| node["node_id"] == mode_id)
        .expect("mode");
    let rate_id = package["semantic_graph"]["nodes"][mode]["body"]["members"][1]["value"]
        ["members"][1]["target"]
        .clone();
    let rate = nodes
        .iter()
        .position(|node| node["node_id"] == rate_id)
        .expect("rate");
    refresh_tank_hybrid_graph(&mut package);
    assert!(matches!(
        read_package(&package).expect("published evidence"),
        CheckedPackageV2ReadResult::Admitted(_)
    ));

    for value in ["2", "+1", "01"] {
        let mut changed = package.clone();
        changed["semantic_graph"]["nodes"][rate]["body"]["value"] = json!(value);
        refresh_tank_hybrid_graph(&mut changed);
        let read = read_package(&changed).expect("published evidence");
        let CheckedPackageV2ReadResult::Refused(refusal) = read else {
            panic!("{value:?} clock rate admitted: {read:?}");
        };
        assert_eq!(
            (refusal.code, refusal.cause),
            (Code::InvalidPackage, Some(Cause::InvalidValue)),
            "{value:?}: {refusal:?}"
        );
        assert_locus(
            &refusal,
            model,
            "/body/members/0/value/members/1/members/1/value/value",
        );
    }
}

#[trace("TC-048", "FR-038-AC-313", "FR-038-AC-314")]
#[test]
fn qspec_hybrid_model_rebuilt_key_boundaries() {
    for change in [
        "variable_level",
        "rate_count",
        "target_mode",
        "orphan_transition",
    ] {
        let mut package = published_hybrid_package();
        let model = model_position(&package);
        let mode_id = package["semantic_graph"]["nodes"][model]["body"]["members"][1]["value"]
            ["members"][0]["target"]
            .clone();
        let mode = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .position(|node| node["node_id"] == mode_id)
            .expect("mode");
        let transition_id = package["semantic_graph"]["nodes"][mode]["body"]["members"][2]["value"]
            ["members"][0]["target"]
            .clone();
        let transition = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .position(|node| node["node_id"] == transition_id)
            .expect("transition");
        match change {
            "variable_level" => {
                let parameter_id = package["semantic_graph"]["nodes"][model]["body"]["members"][0]
                    ["value"]["members"][0]["members"][0]["value"]["target"]
                    .clone();
                let parameter = package["semantic_graph"]["nodes"]
                    .as_array()
                    .expect("nodes")
                    .iter()
                    .position(|node| node["node_id"] == parameter_id)
                    .expect("parameter");
                package["semantic_graph"]["nodes"][parameter]["body"]["members"][1]["value"]
                    ["value"] = json!("7");
            }
            "rate_count" => {
                let first = package["semantic_graph"]["nodes"][mode]["body"]["members"][1]["value"]
                    ["members"][0]
                    .clone();
                package["semantic_graph"]["nodes"][mode]["body"]["members"][1]["value"]["members"]
                    .as_array_mut()
                    .expect("rates")
                    .push(first);
            }
            "target_mode" => {
                package["semantic_graph"]["nodes"][transition]["body"]["members"][0]["value"]
                    ["value"] = json!("2")
            }
            "orphan_transition" => {
                package["semantic_graph"]["nodes"][mode]["body"]["members"][2]["value"]["members"] =
                    json!([])
            }
            _ => unreachable!("closed changes"),
        }
        refresh_tank_hybrid_graph(&mut package);
        let read = read_package(&package).expect("published evidence");
        let CheckedPackageV2ReadResult::Refused(refusal) = read else {
            panic!("{change} admitted");
        };
        assert_eq!(
            refusal.code,
            match change {
                "variable_level" | "rate_count" => Code::InvalidModelBinding,
                "target_mode" => Code::MissingDeclaration,
                "orphan_transition" => Code::InvalidSemanticGraph,
                _ => unreachable!("closed changes"),
            },
            "{change}: {refusal:?}"
        );
        let (locus_node, suffix) = match change {
            "variable_level" => (
                model,
                "/body/members/0/value/members/0/members/0/value/target",
            ),
            "rate_count" => (mode, "/body/members/1/value"),
            "target_mode" => (transition, "/body/members/0/value"),
            "orphan_transition" => (transition, ""),
            _ => unreachable!("closed changes"),
        };
        assert_locus(&refusal, locus_node, suffix);
    }
}

#[trace("TC-048", "FR-038-AC-313")]
#[test]
fn qspec_hybrid_initial_and_reset_checks_after_rekey() {
    for change in [
        "written_default",
        "initial_mode",
        "initial_type",
        "reset_conflict",
    ] {
        let mut package = published_hybrid_package();
        let model = model_position(&package);
        let mode_id = package["semantic_graph"]["nodes"][model]["body"]["members"][1]["value"]
            ["members"][0]["target"]
            .clone();
        let mode = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .position(|node| node["node_id"] == mode_id)
            .expect("mode");
        let true_node =
            package["semantic_graph"]["nodes"][mode]["body"]["members"][0]["value"].clone();
        let rational_one = package["semantic_graph"]["nodes"][mode]["body"]["members"][1]["value"]
            ["members"][1]
            .clone();
        let transition_id = package["semantic_graph"]["nodes"][mode]["body"]["members"][2]["value"]
            ["members"][0]["target"]
            .clone();
        let transition = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .position(|node| node["node_id"] == transition_id)
            .expect("transition");
        let integer_type = package["semantic_graph"]["nodes"][transition]["body"]["members"][0]
            ["value"]["type"]
            .clone();
        match change {
            "written_default" | "initial_mode" | "initial_type" => {
                let entry = json!({"term":"aggregate", "members":[
                    {"term":"binding", "name":"mode", "value":{
                        "term":"literal", "type":integer_type, "value_kind":"integer", "value":if change == "initial_mode" {"2"} else {"0"}}},
                    {"term":"binding", "name":"condition", "value":if change == "initial_type" {rational_one} else {true_node}}
                ]});
                package["semantic_graph"]["nodes"][model]["body"]["members"]
                    .as_array_mut()
                    .expect("model members")
                    .push(json!({"term":"binding", "name":"initial", "value":{
                        "term":"aggregate", "members":[entry]}}));
            }
            "reset_conflict" => {
                let reset = package["semantic_graph"]["nodes"][transition]["body"]["members"][2]
                    ["value"]["members"][0]
                    .clone();
                package["semantic_graph"]["nodes"][transition]["body"]["members"][2]["value"]
                    ["members"]
                    .as_array_mut()
                    .expect("resets")
                    .push(reset);
            }
            _ => unreachable!("closed changes"),
        }
        refresh_tank_hybrid_graph(&mut package);
        let read = read_package(&package).expect("published evidence");
        let CheckedPackageV2ReadResult::Refused(refusal) = read else {
            panic!("{change} admitted");
        };
        let expected = match change {
            "written_default" => (Code::InvalidSemanticGraph, None),
            "initial_mode" => (Code::MissingDeclaration, Some(Cause::MissingName)),
            "initial_type" => (Code::IllTyped, Some(Cause::NonBooleanRoot)),
            "reset_conflict" => (Code::InvalidPackage, Some(Cause::ConflictingDefinition)),
            _ => unreachable!("closed changes"),
        };
        assert_eq!(
            (refusal.code, refusal.cause),
            expected,
            "{change}: {refusal:?}"
        );
        let (locus_node, suffix) = match change {
            "written_default" => (model, "/body/members/2/value"),
            "initial_mode" => (model, "/body/members/2/value/members/0/members/0/value"),
            "initial_type" => (
                model,
                "/body/members/2/value/members/0/members/1/value/target",
            ),
            "reset_conflict" => (transition, "/body/members/2/value/members/1"),
            _ => unreachable!("closed changes"),
        };
        assert_locus(&refusal, locus_node, suffix);
    }
}

#[trace("TC-048", "FR-038-AC-314")]
#[test]
fn qspec_hybrid_model_refuses_foreign_parameter_read() {
    let mut package = published_hybrid_package();
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    let foreign = nodes
        .iter()
        .find(|node| {
            node["node_tag"] == "value"
                && node["semantic_form"] == "parameter"
                && node["body"]["members"][0]["value"]["value"] == "self"
        })
        .expect("foreign parameter")["node_id"]
        .clone();
    let boolean = nodes
        .iter()
        .find(|node| node["node_tag"] == "scalar_type" && node["semantic_form"] == "boolean")
        .expect("Boolean type")["node_id"]
        .clone();
    let mut read = nodes
        .iter()
        .find(|node| {
            node["node_tag"] == "expression"
                && node["semantic_form"] == "reference"
                && node["semantic_type"] == boolean
        })
        .expect("Boolean reference term")
        .clone();
    let placeholder = fixture_support::structural_id(
        "example",
        vec![
            json!("expression"),
            json!("reference"),
            json!("foreign-parameter"),
        ],
        0,
    );
    read["node_id"] = placeholder.clone();
    read["body"]["target"] = foreign.clone();
    read["dependencies"] = json!([foreign]);
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(read);
    let source = package["lock"]["sources"][0].clone();
    package["source_map"]
        .as_array_mut()
        .expect("source map")
        .push(json!({
            "node_id":placeholder.clone(), "role":"expression", "ordinal":0,
            "regions":[{"source":source,"start":0,"end":1}]
        }));
    let model = model_position(&package);
    let mode_id = package["semantic_graph"]["nodes"][model]["body"]["members"][1]["value"]
        ["members"][0]["target"]
        .clone();
    let mode = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_id"] == mode_id)
        .expect("mode");
    package["semantic_graph"]["nodes"][mode]["body"]["members"][0]["value"]["target"] = placeholder;
    refresh_tank_hybrid_graph(&mut package);
    let read = read_package(&package).expect("published evidence");
    let CheckedPackageV2ReadResult::Refused(refusal) = read else {
        panic!("foreign parameter read admitted");
    };
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::MissingDeclaration, Some(Cause::MissingName)),
        "{refusal:?}"
    );
    assert_locus(&refusal, mode, "/body/members/0/value");
}

#[trace("TC-048", "FR-038-AC-314")]
#[test]
fn qspec_hybrid_call_refuses_ineligible_function_signature() {
    for defect in ["result", "parameter"] {
        let mut package = published_hybrid_package();
        let nodes = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes");
        let text_type = nodes
            .iter()
            .find(|node| node["node_tag"] == "scalar_type" && node["semantic_form"] == "text")
            .expect("text type")["node_id"]
            .clone();
        let mut function = nodes
            .iter()
            .find(|node| {
                node["node_tag"] == "function"
                    && node["declaration"]["qualified_name"] == json!(["Example", "helper"])
            })
            .expect("published function")
            .clone();
        if defect == "parameter" {
            let mut parameter = nodes
                .iter()
                .find(|node| node["node_tag"] == "value" && node["semantic_form"] == "parameter")
                .expect("published parameter")
                .clone();
            parameter["node_id"] = fixture_support::structural_id(
                "example",
                vec![json!("value"), json!("parameter"), json!("hybrid_argument")],
                0,
            );
            parameter["semantic_type"] = text_type.clone();
            parameter["body"]["members"][0]["value"]["value"] = json!("hybrid_argument");
            append_published_nodes(&mut package, &[parameter]);
            fixture_support::settle(&mut package);
            let key = package["semantic_graph"]["nodes"]
                .as_array()
                .expect("nodes")
                .last()
                .expect("bad parameter")["node_id"]
                .clone();
            function["body"] = json!({"term":"aggregate", "members":[
                {"term":"binding", "name":"parameters", "value":{
                    "term":"aggregate", "members":[{"term":"reference", "target":key}]
                }}
            ]});
            function["dependencies"] = json!([key]);
        } else {
            function["semantic_type"] = text_type;
        }
        function["node_id"] = fixture_support::structural_id(
            "example",
            vec![json!("Example"), json!("hybrid_bad")],
            0,
        );
        function["declaration"]["qualified_name"] = json!(["Example", "hybrid_bad"]);
        append_published_nodes(&mut package, &[function]);
        fixture_support::settle(&mut package);
        let bad_function = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .last()
            .expect("bad function")["node_id"]
            .clone();
        let nodes = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes");
        let mut call = nodes
            .iter()
            .find(|node| {
                node["node_tag"] == "function"
                    && node["declaration"]["qualified_name"] == json!(["Example", "run"])
            })
            .expect("published call body")
            .clone();
        call.as_object_mut().expect("call").remove("declaration");
        call["node_tag"] = json!("expression");
        call["semantic_form"] = json!("call");
        call["occurrences"][0]["role"] = json!("expression");
        call["node_id"] = fixture_support::structural_id(
            "example",
            vec![json!("expression"), json!("call"), json!("hybrid_bad")],
            0,
        );
        call["body"]["arguments"][0]["target"] = bad_function.clone();
        call["dependencies"] = json!([bad_function]);
        append_published_nodes(&mut package, &[call]);
        let call_position = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .len()
            - 1;
        fixture_support::settle(&mut package);
        let mode_id = package["semantic_graph"]["nodes"][model_position(&package)]["body"]
            ["members"][1]["value"]["members"][0]["target"]
            .clone();
        let mode = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .position(|node| node["node_id"] == mode_id)
            .expect("Tank mode");
        package["semantic_graph"]["nodes"][mode]["body"]["members"][0]["value"]["target"] =
            package["semantic_graph"]["nodes"][call_position]["node_id"].clone();
        refresh_tank_hybrid_graph(&mut package);
        let read = read_package(&package).expect("QSpec package");
        let CheckedPackageV2ReadResult::Refused(refusal) = read else {
            panic!("bad function {defect} admitted: {read:?}")
        };
        assert_eq!(
            (refusal.code, refusal.cause),
            (Code::IllTyped, Some(Cause::TypeMismatch)),
            "{refusal:?}"
        );
        assert_locus(&refusal, mode, "/body/members/0/value");
    }
}

#[trace("TC-048")]
#[test]
fn qspec_protocol_reference_join_refuses_at_its_target() {
    let mut package = published_hybrid_package();
    let boolean = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "scalar_type" && node["semantic_form"] == "boolean")
        .expect("Boolean type")["node_id"]
        .clone();
    let body = json!({"term":"reference", "target":boolean});
    let key = fixture_support::structural_id(
        "example",
        vec![
            json!("protocol"),
            json!("protocol_reference"),
            boolean.clone(),
        ],
        0,
    );
    let carrier = json!({
        "node_id":key,
        "schema_version":"quire.checked-semantic-graph/v2",
        "node_tag":"protocol", "semantic_form":"protocol_reference",
        "semantic_type":boolean, "dependencies":[body["target"].clone()],
        "occurrences":[{"role":"expression","ordinal":0}], "body":body,
    });
    append_published_nodes(&mut package, &[carrier]);
    fixture_support::refresh_identity(&mut package);
    let carrier_position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
    let read = read_package(&package).expect("QSpec package");
    let CheckedPackageV2ReadResult::Refused(refusal) = read else {
        panic!("bad protocol carrier admitted: {read:?}")
    };
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::IllTyped, Some(Cause::OperatorIneligible)),
        "{refusal:?}"
    );
    assert_locus(&refusal, carrier_position, "/body/target");
}

#[trace("TC-048", "FR-038-AC-316")]
#[test]
fn qspec_hybrid_step_precedes_later_protocol_body_defect() {
    let mut package = braking_package();
    let claim = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| {
            node["node_tag"] == "temporal"
                && node["semantic_form"] == "temporal_clause"
                && node["body"]["operation"]["laws"][0]["definition"]["identity"]
                    == "quire.temporal.infinite-trace/v1"
        })
        .expect("published hybrid claim");
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    let over = &nodes
        .iter()
        .find(|node| node["node_id"] == nodes[claim]["body"]["arguments"][0]["target"])
        .expect("claim over parameter")["semantic_type"];
    assert!(nodes.iter().any(|node| {
        node["node_id"] == *over
            && node["node_tag"] == "hybrid"
            && node["semantic_form"] == "hybrid_model"
    }));
    assert!(matches!(
        read_package(&package).expect("published hybrid claim"),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    let protocol = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| {
            node["node_tag"] == "protocol" && node["semantic_form"] == "protocol_clause"
        })
        .expect("published protocol clause");
    package["semantic_graph"]["nodes"][protocol]["body"]["operation"]["laws"][0]["role"] =
        json!("temporal_profile");
    fixture_support::settle(&mut package);
    fixture_support::refresh_identity(&mut package);
    let read = read_package(&package).expect("QSpec package");
    let CheckedPackageV2ReadResult::Refused(refusal) = read else {
        panic!("wrong protocol clause law admitted: {read:?}")
    };
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::InvalidPackage, Some(Cause::OperationLawMismatch)),
        "{refusal:?}"
    );
    assert_locus(&refusal, protocol, "/body/operation/laws/0/role");

    package["semantic_graph"]["nodes"][claim]["body"]["arguments"][1]["value"] = json!("wall-time");
    fixture_support::settle(&mut package);
    fixture_support::refresh_identity(&mut package);
    let read = read_package(&package).expect("QSpec package");
    let CheckedPackageV2ReadResult::Refused(refusal) = read else {
        panic!("hybrid defect admitted: {read:?}")
    };
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::InvalidPackage, Some(Cause::InvalidValue)),
        "{refusal:?}"
    );
    assert_locus(&refusal, claim, "/body/arguments/1");
}

#[trace("TC-048", "FR-038-AC-315")]
#[test]
fn qspec_hybrid_wrong_interval_profile_precedes_fairness_defect() {
    let mut package = restored_package();
    let claim = timed_claim_position(&package);
    let formula_id =
        package["semantic_graph"]["nodes"][claim]["body"]["arguments"][5]["target"].clone();
    let formula = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_id"] == formula_id)
        .expect("timed formula");
    package["semantic_graph"]["nodes"][formula]["body"]["operation"]["member"]["interval"] =
        json!({"lower":"0", "upper":"2"});
    fixture_support::settle(&mut package);
    let fair = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_tag"] == "temporal" && node["semantic_form"] == "fairness")
        .expect("fairness node");
    package["semantic_graph"]["nodes"][fair]["body"]["operation"]["member"]["name"] =
        json!("Halted");
    fixture_support::settle(&mut package);
    sort_dependencies(&mut package);
    fixture_support::settle(&mut package);
    fixture_support::refresh_identity(&mut package);
    let read = read_package(&package).expect("QSpec package");
    let CheckedPackageV2ReadResult::Refused(refusal) = read else {
        panic!("bad interval admitted: {read:?}")
    };
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::InvalidPackage, Some(Cause::OperationMemberMismatch)),
        "{refusal:?}"
    );
    assert_locus(&refusal, formula, "/body");
}
