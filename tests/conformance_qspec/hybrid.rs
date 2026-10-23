// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! QSpec TC-966 hybrid nodes read from the owning checkout, without a fixture copy.

use super::*;

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
    for (form, change) in [
        ("hybrid_transition", "declaration"),
        ("hybrid_mode", "occurrence_role"),
        ("hybrid_model", "reordered_bindings"),
        ("hybrid_model", "class"),
        ("hybrid_transition", "reference_target_index"),
        ("hybrid_mode", "empty_rates"),
        ("hybrid_model", "unknown_form"),
        ("hybrid_model", "missing_owner"),
        ("hybrid_model", "empty_initial"),
        ("hybrid_transition", "invalid_reset_relation"),
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
        assert!(
            refusal.path.as_ref().is_some_and(|path| path
                .to_string()
                .starts_with(&format!("/semantic_graph/nodes/{position}"))),
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

fn rekey_model(package: &mut Value, position: usize) {
    let node = &package["semantic_graph"]["nodes"][position];
    let old = node["node_id"]["digest"]
        .as_str()
        .expect("node id")
        .to_owned();
    let preimage = json!({
        "version": "quire.structural-node/v1",
        "node_tag": node["node_tag"],
        "semantic_form": node["semantic_form"],
        "semantic_type": null,
        "declaration": node["declaration"],
        "recursion": null,
        "owner": node["owner"],
        "body": node["body"],
    });
    let fresh = canonical_digest(&preimage);
    rewrite_digest(package, &old, &fresh);
    package["package_id"]["digest"] = json!(derived_with(package, None).expect("package id"));
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
        .map(|node| {
            node["node_id"]["digest"]
                .as_str()
                .expect("digest")
                .to_owned()
        })
        .collect::<std::collections::BTreeSet<_>>();
    let source = package["lock"]["sources"][0].clone();
    for node in nodes {
        if present.contains(node["node_id"]["digest"].as_str().expect("digest")) {
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
        node["dependencies"]
            .as_array_mut()
            .expect("dependencies")
            .sort_by(|left, right| left["digest"].as_str().cmp(&right["digest"].as_str()));
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
                        targets.insert(
                            target["digest"].as_str().expect("digest").to_owned(),
                            target.clone(),
                        );
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

fn refresh_tank_hybrid_keys(package: &mut Value) {
    refresh_hybrid_dependencies(package);
    fixture_support::mint_ungrouped_structural_keys(package);
    sort_dependencies(package);
    fixture_support::refresh_identity(package);
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
    if let CheckedPackageV2ReadResult::Refused(refusal) = read {
        panic!("Braking refused: {refusal:?}");
    }
}

#[trace("TC-048", "FR-038-AC-315", "FR-038-AC-317")]
#[test]
fn qspec_hybrid_restored_model_and_claims_admit_from_published_nodes() {
    let package = restored_package();
    let read = read_package(&package).expect("published evidence");
    if let CheckedPackageV2ReadResult::Refused(refusal) = read {
        panic!("Braking2 refused: {refusal:?}");
    }
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
    fixture_support::rekey_application_node(&mut package, formula);
    sort_dependencies(&mut package);
    fixture_support::rekey_application_node(&mut package, clause);
    fixture_support::refresh_identity(&mut package);
    let read = read_package(&package).expect("published evidence");
    if let CheckedPackageV2ReadResult::Refused(refusal) = read {
        panic!("hybrid timed open upper refused: {refusal:?}");
    }
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
    fixture_support::rekey_application_node(&mut package, position);
    fixture_support::refresh_identity(&mut package);
    let read = read_package(&package).expect("published evidence");
    let CheckedPackageV2ReadResult::Refused(refusal) = read else {
        panic!("invalid hybrid clock admitted");
    };
    assert_eq!(refusal.code, Code::InvalidPackage, "{refusal:?}");
    assert_eq!(refusal.cause, Some(Cause::InvalidValue), "{refusal:?}");
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
    fixture_support::rekey_application_node(&mut package, fair);
    sort_dependencies(&mut package);
    let claim_position = timed_claim_position(&package);
    fixture_support::rekey_application_node(&mut package, claim_position);
    fixture_support::refresh_identity(&mut package);
    let read = read_package(&package).expect("published evidence");
    let CheckedPackageV2ReadResult::Refused(refusal) = read else {
        panic!("unknown hybrid fairness mode admitted");
    };
    assert_eq!(refusal.code, Code::MissingDeclaration, "{refusal:?}");
    assert_eq!(refusal.cause, Some(Cause::MissingName), "{refusal:?}");
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
    let old_trigger = package["semantic_graph"]["nodes"][claim_position]["body"]["arguments"][2]
        ["members"][1]["value"]["target"]
        .clone();
    package["semantic_graph"]["nodes"][claim_position]["body"]["arguments"][2]["members"][1]
        ["value"]["target"] = variable.clone();
    let old = old_trigger["digest"].as_str().expect("digest");
    let new = variable["digest"].as_str().expect("digest");
    for dependency in package["semantic_graph"]["nodes"][claim_position]["dependencies"]
        .as_array_mut()
        .expect("dependencies")
    {
        if dependency["digest"] == old {
            dependency["digest"] = json!(new);
        }
    }
    sort_dependencies(&mut package);
    fixture_support::rekey_application_node(&mut package, claim_position);
    fixture_support::refresh_identity(&mut package);
    let read = read_package(&package).expect("published evidence");
    let CheckedPackageV2ReadResult::Refused(refusal) = read else {
        panic!("scalar-typed trigger admitted");
    };
    assert_eq!(refusal.code, Code::IllTyped, "{refusal:?}");
    assert_eq!(refusal.cause, Some(Cause::TypeMismatch), "{refusal:?}");
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
    fixture_support::rekey_application_node(&mut package, holds);
    sort_dependencies(&mut package);
    fixture_support::rekey_application_node(&mut package, always);
    sort_dependencies(&mut package);
    fixture_support::rekey_application_node(&mut package, clause);
    fixture_support::refresh_identity(&mut package);
    let read = read_package(&package).expect("published evidence");
    let CheckedPackageV2ReadResult::Refused(refusal) = read else {
        panic!("holds read of over parameter admitted");
    };
    assert_eq!(refusal.code, Code::MissingDeclaration, "{refusal:?}");
    assert_eq!(refusal.cause, Some(Cause::MissingName), "{refusal:?}");
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
        rekey_model(&mut package, position);
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
        refresh_tank_hybrid_keys(&mut package);
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
        refresh_tank_hybrid_keys(&mut package);
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
    let placeholder = json!({"domain":"quire.checked-semantic-node/v1", "digest":"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"});
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
            "node_id":placeholder, "role":"expression", "ordinal":0,
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
    package["semantic_graph"]["nodes"][mode]["body"]["members"][0]["value"]["target"] = json!({"domain":"quire.checked-semantic-node/v1", "digest":"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"});
    refresh_tank_hybrid_keys(&mut package);
    let read = read_package(&package).expect("published evidence");
    let CheckedPackageV2ReadResult::Refused(refusal) = read else {
        panic!("foreign parameter read admitted");
    };
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::MissingDeclaration, Some(Cause::MissingName)),
        "{refusal:?}"
    );
}
