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
        refusal.path.as_ref().map(ToString::to_string),
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
            refusal.path.as_ref().map(ToString::to_string),
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

#[trace("TC-048", "FR-038-AC-312")]
#[test]
fn qspec_hybrid_rate_edit_propagates_identity_through_claim() {
    let mut package = braking_package();
    assert!(matches!(
        read_package(&package).expect("QSpec package"),
        CheckedPackageV2ReadResult::Admitted(_)
    ));
    let model = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| {
            node["node_tag"] == "hybrid"
                && node["semantic_form"] == "hybrid_model"
                && node["declaration"]["qualified_name"] == json!(["Braking"])
        })
        .expect("Braking model");
    let mode_id = package["semantic_graph"]["nodes"][model]["body"]["members"][1]["value"]
        ["members"][0]["target"]
        .clone();
    let mode = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_id"] == mode_id)
        .expect("Moving mode");
    let claim = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| {
            node["body"]["operation"]["identity"] == "quire.op.temporal.clause"
                && node["body"]["arguments"][0]["target"] != Value::Null
                && node["body"]["arguments"][1]["value"] == "model-time"
        })
        .expect("SlowEnough claim");
    let before = [
        package["semantic_graph"]["nodes"][mode]["node_id"].clone(),
        package["semantic_graph"]["nodes"][model]["node_id"].clone(),
        package["semantic_graph"]["nodes"][claim]["body"]["arguments"][0]["target"].clone(),
        package["semantic_graph"]["nodes"][claim]["node_id"].clone(),
        package["package_id"].clone(),
    ];
    let replacement = package["semantic_graph"]["nodes"][mode]["body"]["members"][1]["value"]
        ["members"][1]["target"]
        .clone();
    package["semantic_graph"]["nodes"][mode]["body"]["members"][1]["value"]["members"][0]
        ["target"] = replacement;
    refresh_hybrid_dependencies(&mut package);
    fixture_support::mint_ungrouped_structural_keys(&mut package);
    refresh_hybrid_dependencies(&mut package);
    fixture_support::rekey_application_node(&mut package, claim);
    fixture_support::refresh_identity(&mut package);
    let after = [
        package["semantic_graph"]["nodes"][mode]["node_id"].clone(),
        package["semantic_graph"]["nodes"][model]["node_id"].clone(),
        package["semantic_graph"]["nodes"][claim]["body"]["arguments"][0]["target"].clone(),
        package["semantic_graph"]["nodes"][claim]["node_id"].clone(),
        package["package_id"].clone(),
    ];
    for (old, new) in before.iter().zip(&after) {
        assert_ne!(old, new, "rate edit must propagate through the claim");
    }
    let read = read_package(&package).expect("QSpec package");
    assert!(
        matches!(read, CheckedPackageV2ReadResult::Admitted(_)),
        "{read:?}"
    );
}

#[trace("TC-048", "FR-038-AC-312")]
#[test]
fn qspec_hybrid_key_and_dependency_checks_precede_body_checks() {
    for change in ["stale_key", "dependency_mismatch"] {
        let mut package = published_hybrid_package();
        let model = model_position(&package);
        if change == "stale_key" {
            package["semantic_graph"]["nodes"][model]["body"]["members"][0]["value"]["members"]
                [0]["members"][1]["value"]["value"] = json!("clock");
        } else {
            package["semantic_graph"]["nodes"][model]["dependencies"]
                .as_array_mut()
                .expect("dependencies")
                .pop();
        }
        fixture_support::refresh_identity(&mut package);
        let read = read_package(&package).expect("QSpec package");
        let CheckedPackageV2ReadResult::Refused(refusal) = read else {
            panic!("{change} admitted")
        };
        assert_eq!(
            refusal.code,
            if change == "stale_key" {
                Code::InvalidPackage
            } else {
                Code::InvalidSemanticGraph
            },
            "{change}: {refusal:?}"
        );
        assert_locus(
            &refusal,
            model,
            if change == "stale_key" {
                "/node_id"
            } else {
                "/dependencies"
            },
        );
        assert_eq!(
            refusal.cause,
            if change == "stale_key" {
                Some(Cause::StaleNodeKey)
            } else {
                None
            },
            "{change}: {refusal:?}"
        );
    }
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
    dependencies.sort_by(|left, right| left["digest"].as_str().cmp(&right["digest"].as_str()));
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
    fixture_support::rekey_application_node(&mut package, claim);
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
    unrelated["node_id"]["digest"] =
        json!("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
    unrelated["semantic_type"] = boolean;
    append_published_nodes(&mut package, &[unrelated]);
    fixture_support::mint_ungrouped_structural_keys(&mut package);
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
    fixture_support::rekey_application_node(&mut package, formula);
    sort_dependencies(&mut package);
    fixture_support::rekey_application_node(&mut package, clause);
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
    fixture_support::rekey_application_node(&mut package, position);
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
    refresh_tank_hybrid_keys(&mut package);
    assert!(matches!(
        read_package(&package).expect("published evidence"),
        CheckedPackageV2ReadResult::Admitted(_)
    ));

    package["semantic_graph"]["nodes"][rate]["body"]["value"] = json!("2");
    refresh_tank_hybrid_keys(&mut package);
    let read = read_package(&package).expect("published evidence");
    let CheckedPackageV2ReadResult::Refused(refusal) = read else {
        panic!("non-unit clock rate admitted: {read:?}");
    };
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::InvalidPackage, Some(Cause::InvalidValue))
    );
    assert_locus(
        &refusal,
        model,
        "/body/members/0/value/members/1/members/1/value/value",
    );
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
            parameter["node_id"]["digest"] =
                json!("dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd");
            parameter["semantic_type"] = text_type.clone();
            parameter["body"]["members"][0]["value"]["value"] = json!("hybrid_argument");
            append_published_nodes(&mut package, &[parameter]);
            fixture_support::mint_ungrouped_structural_keys(&mut package);
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
        function["node_id"]["digest"] =
            json!("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
        function["declaration"]["qualified_name"] = json!(["Example", "hybrid_bad"]);
        append_published_nodes(&mut package, &[function]);
        fixture_support::mint_ungrouped_structural_keys(&mut package);
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
        call["node_id"]["digest"] =
            json!("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee");
        call["body"]["arguments"][0]["target"] = bad_function.clone();
        call["dependencies"] = json!([bad_function]);
        append_published_nodes(&mut package, &[call]);
        let call_position = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .len()
            - 1;
        fixture_support::rekey_application_node(&mut package, call_position);
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
        refresh_tank_hybrid_keys(&mut package);
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

#[trace("TC-048", "FR-038-AC-316")]
#[test]
fn qspec_hybrid_step_precedes_protocol_reference_join() {
    let mut package = published_hybrid_package();
    let boolean = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "scalar_type" && node["semantic_form"] == "boolean")
        .expect("Boolean type")["node_id"]
        .clone();
    let body = json!({"term":"reference", "target":boolean});
    let key = fixture_support::structural_key(
        "protocol",
        "protocol_reference",
        boolean["digest"].as_str(),
        &body,
    );
    let carrier = json!({
        "node_id":fixture_support::node_id(&key),
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

    let model = model_position(&package);
    package["semantic_graph"]["nodes"][model]["body"]["members"][0]["value"]["members"][0]
        ["members"][1]["value"]["value"] = json!("clock");
    rekey_model(&mut package, model);
    let read = read_package(&package).expect("QSpec package");
    let CheckedPackageV2ReadResult::Refused(refusal) = read else {
        panic!("hybrid defect admitted: {read:?}")
    };
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::InvalidPackage, Some(Cause::InvalidValue)),
        "{refusal:?}"
    );
    assert_locus(
        &refusal,
        model,
        "/body/members/0/value/members/0/members/1/value/value",
    );
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
    fixture_support::rekey_application_node(&mut package, formula);
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
    fixture_support::rekey_application_node(&mut package, claim);
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
