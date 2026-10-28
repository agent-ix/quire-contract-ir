// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Timed-model wire evidence read from the owning QSpec checkout.
//! Suite-only traces retain the structural/round-trip and scoped refusal evidence;
//! full timed semantic criterion qualification belongs to the follow-on slice.
use super::*;
use quire_contract_model::{
    encode_checked_package_v2, CheckedCollectionKind, CheckedMemberType,
    CheckedPackageIdentityPreimageRefV2, CheckedTerm,
};

fn published() -> Value {
    read_json(&fixtures_dir().join("positive-timed-model-nodes.json"))
        .expect("published timed model must be available")
}

/// The slot fixture selects the complete FCD document, including its named
/// sequence and map definitions, rather than an isolated node projection.
#[trace("TC-048", "FR-038-AC-335", "FR-038-AC-339", "FR-038-AC-340")]
#[test]
fn timed_slot_selected_package_admits_nested_and_optional_clock_places() {
    let fixtures = fixtures_dir();
    let wire = read_json(&fixtures.join("positive-timed-slot-nodes.json"))
        .expect("published timed slot package");
    let document = proposal_dir_of(&fixtures).join("domain-package-acme-timed-slots.json");
    let result = read_package_with_document(&wire, "timed slot package", &document)
        .expect("selected timed slot evidence");
    let CheckedPackageV2ReadResult::Admitted(package) = result else {
        panic!("selected timed slot package must admit: {result:?}");
    };
    assert_eq!(
        package.graph().nodes.len(),
        wire["semantic_graph"]["nodes"].as_array().unwrap().len()
    );
    assert_eq!(
        Some(package.package_id().digest.as_ref()),
        wire["package_id"]["digest"].as_str()
    );
    let order = package
        .graph()
        .nodes
        .iter()
        .find(|node| {
            node.node_tag.as_ref() == "model"
                && node.semantic_form.as_ref() == "object_type"
                && node.owner.as_ref().is_some_and(|owner| {
                    matches!(owner, quire_contract_model::CheckedNodeOwner::Model { node, .. }
                        if node.as_ref() == "ix://acme/orders/Order")
                })
        })
        .expect("selected Order declaration");
    let fields = package
        .model_object_fields(&order.node_id)
        .expect("selected Order fields");
    let g = fields
        .fields()
        .iter()
        .find(|field| field.name() == "g")
        .expect("nested clock field");
    let Some(CheckedMemberType::Collection {
        kind: CheckedCollectionKind::Sequence,
        element: outer,
        bounds: None,
    }) = g.member_type()
    else {
        panic!("g must resolve as an unbounded sequence");
    };
    assert!(matches!(
        outer.as_ref(),
        CheckedMemberType::Collection {
            kind: CheckedCollectionKind::Sequence,
            element,
            bounds: None,
        } if element.as_ref() == &CheckedMemberType::Clock
    ));
    let q = fields
        .fields()
        .iter()
        .find(|field| field.name() == "q")
        .expect("optional clock sequence field");
    assert!(matches!(
        q.member_type(),
        Some(CheckedMemberType::Collection {
            kind: CheckedCollectionKind::Sequence,
            element,
            bounds: Some((0, 2)),
        }) if element.as_ref() == &CheckedMemberType::Option(Box::new(CheckedMemberType::Clock))
    ));
}

fn encode_package(package: &CheckedPackageV2) -> Vec<u8> {
    let selected = package.identity_preimage();
    let preimage = CheckedPackageIdentityPreimageRefV2 {
        version: &selected.version,
        edition: &selected.edition,
        profile_selections: &selected.profile_selections,
        definition_selections: &selected.definition_selections,
        model_selections: &selected.model_selections,
        required_features: &selected.required_features,
        dependency_selections: &selected.dependency_selections,
        identity_projection: &package.graph().nodes,
    };
    encode_checked_package_v2(
        &preimage,
        package.package_id(),
        package.lock(),
        package.graph(),
        quire_contract_model::CheckedPackageSourceCorrespondenceRef {
            source_map: package.source_map(),
            refinement_step_rows: package.refinement_step_rows(),
        },
        package.capability_report(),
        package.diagnostics(),
        Limits::new(1 << 24),
    )
    .expect("typed writer encodes admitted package")
}

#[trace("TC-048", "FR-038-AC-318", "FR-038-AC-322")]
#[test]
fn timed_model_published_tick_retains_typed_operation_anchor() {
    let wire = read_json(&fixtures_dir().join("positive-tick-timed-model-nodes.json"))
        .expect("published tick model");
    let CheckedPackageV2ReadResult::Admitted(package) = read_package(&wire).expect("tick read")
    else {
        panic!("healthy published tick model must admit");
    };
    let root = wire["semantic_graph"]["nodes"]
        .as_array()
        .expect("tick nodes")
        .iter()
        .find(|node| node["node_tag"] == "timed_model" && node["semantic_form"] == "timed_model")
        .expect("tick root");
    for member in &root["body"]["members"].as_array().expect("root members")[3..] {
        assert_eq!(
            member["value"]["members"],
            json!([]),
            "GD-01 empty aggregate"
        );
    }
    let anchor = package
        .graph()
        .nodes
        .iter()
        .find(|node| {
            node.node_tag.as_ref() == "state" && node.semantic_form.as_ref() == "operation_anchor"
        })
        .expect("published tick operation anchor");
    let CheckedTerm::OperationAnchorBody(body) =
        package.term(&anchor.node_id).expect("retained anchor")
    else {
        panic!("tick anchor must retain its typed body");
    };
    assert_eq!(body.members.len(), 3);
    assert!(body.execution_point.is_none());
    let bytes = encode_package(&package);
    let encoded: Value = serde_json::from_slice(&bytes).expect("writer JSON");
    assert_eq!(
        encoded, wire,
        "typed writer preserves the published tick package"
    );
    assert!(encoded["identity_preimage"]
        .get("identity_projection")
        .is_none());
    let CheckedPackageV2ReadResult::Admitted(again) = read_package(&encoded).expect("writer read")
    else {
        panic!("typed writer's tick package must read back");
    };
    assert_eq!(again.graph(), package.graph());
    assert_eq!(again.package_id(), package.package_id());
    assert_eq!(again.term(&anchor.node_id), package.term(&anchor.node_id));
    assert_eq!(
        encode_package(&again),
        bytes,
        "tick read/write/read is byte-identical"
    );
}

#[trace("TC-048")]
#[test]
fn timed_model_published_fixture_round_trips_complete_nodes() {
    let wire = published();
    assert_eq!(
        wire["semantic_graph"]["nodes"].as_array().unwrap().len(),
        58
    );
    let CheckedPackageV2ReadResult::Admitted(package) = read_package(&wire).unwrap() else {
        panic!(
            "published timed model must admit: {:?}",
            read_package(&wire)
        );
    };
    let timed: Vec<_> = package
        .graph()
        .nodes
        .iter()
        .filter(|n| n.node_tag.as_ref() == "timed_model")
        .collect();
    assert_eq!(timed.len(), 9);
    let forms: std::collections::BTreeSet<_> =
        timed.iter().map(|n| n.semantic_form.as_ref()).collect();
    assert_eq!(forms.len(), 8);
    for node in timed {
        assert_eq!(node.node_id, node.semantic_type);
        assert_eq!(
            serde_json::to_value(package.term(&node.node_id).expect("retained complete body"))
                .unwrap(),
            serde_json::to_value(&node.body).unwrap()
        );
    }
    let bytes = encode_package(&package);
    let encoded: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(encoded, wire);
    assert!(encoded["identity_preimage"]
        .get("identity_projection")
        .is_none());
    let CheckedPackageV2ReadResult::Admitted(again) = read_package(&encoded).unwrap() else {
        panic!("writer output must read back");
    };
    assert_eq!(again.graph(), package.graph());
    assert_eq!(again.package_id(), package.package_id());
}

#[trace("TC-048")]
#[test]
fn timed_model_published_schema_vectors_precede_stale_identity() {
    let vectors = read_json(&fixtures_dir().join("../timed-model-node-vectors.json")).unwrap();
    let rows = vectors["schema_refusals"]
        .as_array()
        .expect("schema vectors");
    assert_eq!(rows.len(), 5);
    for row in rows {
        let mut wire = published();
        let index = vectors["node_indexes"][row["node"].as_str().unwrap()]
            .as_u64()
            .unwrap() as usize;
        let body = &mut wire["semantic_graph"]["nodes"][index]["body"]["members"];
        match row["mutate"].as_str().unwrap() {
            "remove-unit" => {
                body.as_array_mut().unwrap().remove(1);
            }
            "extra-binding" => {
                let extra = body[0].clone();
                body.as_array_mut().unwrap().push(extra);
            }
            "invalid-path-step-kind" => {
                body[1]["value"]["members"][0]["members"][0]["value"]["value"] = json!("slot")
            }
            "swap-model-source" => body.as_array_mut().unwrap().swap(0, 1),
            "empty-path" => body[1]["value"]["members"] = json!([]),
            other => panic!("unhandled published vector {other}"),
        }
        let CheckedPackageV2ReadResult::Refused(refusal) = read_package(&wire).unwrap() else {
            panic!("{} must refuse", row["id"]);
        };
        assert_eq!(
            refusal.code,
            Code::MalformedWire,
            "{}: {refusal:?}",
            row["id"]
        );
        assert!(refusal
            .path
            .unwrap()
            .to_string()
            .starts_with(&format!("/semantic_graph/nodes/{index}/body")));
    }
}

#[trace("TC-048", "QSpec-TM-09", "QSpec-TM-21")]
#[test]
fn timed_model_semantic_vectors_refuse_at_the_published_cause_and_locus() {
    let vectors = read_json(&fixtures_dir().join("../timed-model-node-vectors.json")).unwrap();
    for row in vectors["semantic_refusals"].as_array().unwrap() {
        let mutation = row["mutation"].as_str().unwrap();
        if matches!(
            row["id"].as_str(),
            Some("TM-23" | "TM-24" | "TM-25" | "TM-26" | "TM-27" | "TM-28" | "TM-29")
        ) {
            continue;
        }
        let fixture = match row["fixture"].as_str().unwrap_or("dense") {
            "dense" => "positive-timed-model-nodes.json",
            "tick" => "positive-tick-timed-model-nodes.json",
            other => panic!("{}: unknown fixture {other}", row["id"]),
        };
        let mut wire = read_json(&fixtures_dir().join(fixture)).unwrap();
        match mutation {
            "remove-timed-root" => {
                let root_id = wire["semantic_graph"]["nodes"][52]["node_id"].clone();
                wire["semantic_graph"]["nodes"]
                    .as_array_mut()
                    .unwrap()
                    .remove(52);
                wire["source_map"]
                    .as_array_mut()
                    .unwrap()
                    .retain(|entry| entry["node_id"] != root_id);
            }
            "dangling-tick-operation" => {
                let target =
                    json!({"domain":"quire.checked-semantic-node/v1", "digest":"0".repeat(64)});
                let old = wire["semantic_graph"]["nodes"][44]["body"]["members"][3]["value"]
                    ["target"]
                    .clone();
                wire["semantic_graph"]["nodes"][44]["body"]["members"][3]["value"]["target"] =
                    target.clone();
                replace_dependency(&mut wire["semantic_graph"]["nodes"][44], &old, &target);
            }
            "function-as-tick-operation" => {
                let old = wire["semantic_graph"]["nodes"][44]["body"]["members"][3]["value"]
                    ["target"]
                    .clone();
                let target = wire["semantic_graph"]["nodes"][5]["node_id"].clone();
                wire["semantic_graph"]["nodes"][44]["body"]["members"][3]["value"]["target"] =
                    target.clone();
                replace_dependency(&mut wire["semantic_graph"]["nodes"][44], &old, &target);
            }
            "duplicate-clock-place" => {
                let places = wire["semantic_graph"]["nodes"][52]["body"]["members"][3]["value"]
                    ["members"]
                    .as_array_mut()
                    .unwrap();
                let duplicate = places[0].clone();
                places.push(duplicate);
            }
            "noncanonical-place-path" => {
                wire["semantic_graph"]["nodes"][45]["body"]["members"][1]["value"]["members"][0]
                    ["members"][1]["value"]["value"] = json!("not-clock")
            }
            "zero-tick-period" => {
                wire["semantic_graph"]["nodes"][44]["body"]["members"][4]["value"]["value"] =
                    json!("0")
            }
            "missing-clock-resolution-row" => {
                let steps = wire["semantic_graph"]["nodes"][45]["body"]["members"][1]["value"]
                    ["members"]
                    .as_array_mut()
                    .unwrap();
                let duplicate = steps[0].clone();
                steps.push(duplicate);
            }
            "mismatched-clock-path-step" => {
                wire["semantic_graph"]["nodes"][45]["body"]["members"][2]["value"]["members"][0]
                    ["members"][0]["value"]["value"] = json!(1)
            }
            "wrong-clock-collection-metadata" => {
                wire["semantic_graph"]["nodes"][45]["body"]["members"][2]["value"]["members"][0]
                    ["members"][4]["value"]["value"] = json!("sequence")
            }
            "undeclared-clock-member" => {
                wire["semantic_graph"]["nodes"][45]["body"]["members"][2]["value"]["members"][0]
                    ["members"][2]["value"]["value"] = json!("ix://acme/orders/Order/unknown")
            }
            "wrong-terminal-clock-type" => {
                wire["semantic_graph"]["nodes"][45]["body"]["members"][2]["value"]["members"][0]
                    ["members"][3]["value"]["value"] = json!("ix://quire/native/Integer")
            }
            "coherent-forged-selected-chain" => {
                let place = &mut wire["semantic_graph"]["nodes"][45]["body"]["members"];
                place[1]["value"]["members"][1]["members"][1]["value"]["value"] = json!("sequence");
                let rows = &mut place[2]["value"]["members"];
                rows[0]["members"][3]["value"]["value"] = json!("Sequence<Evil>");
                rows[0]["members"][4]["value"]["value"] = json!("sequence");
                rows[1]["members"][1]["value"]["value"] = json!("Evil");
                rows[1]["members"][2]["value"]["value"] = json!("Sequence<Evil>[slot]");
            }
            "tick-model-context-mismatch" => {
                let old = wire["semantic_graph"]["nodes"][44]["body"]["members"][2]["value"]
                    ["target"]
                    .clone();
                let target = wire["semantic_graph"]["nodes"][14]["node_id"].clone();
                wire["semantic_graph"]["nodes"][44]["body"]["members"][2]["value"]["target"] =
                    target.clone();
                replace_dependency(&mut wire["semantic_graph"]["nodes"][44], &old, &target);
            }
            "unresolved-tick-operation" => {
                wire["semantic_graph"]["nodes"][46]["body"]["members"][1]["value"]["value"] =
                    json!("missing")
            }
            other => panic!("{}: unknown semantic vector {other}", row["id"]),
        }
        fixture_support::mint_ungrouped_structural_keys(&mut wire);
        for node in wire["semantic_graph"]["nodes"].as_array_mut().unwrap() {
            node["dependencies"]
                .as_array_mut()
                .unwrap()
                .sort_by(|left, right| left["digest"].as_str().cmp(&right["digest"].as_str()));
        }
        fixture_support::refresh_identity(&mut wire);
        let CheckedPackageV2ReadResult::Refused(refusal) = read_package(&wire).unwrap() else {
            panic!("{} must refuse", row["id"]);
        };
        assert_eq!(
            code_word(refusal.code),
            row["expected_code"].as_str().unwrap(),
            "{}: {refusal:?}",
            row["id"]
        );
        assert_eq!(
            refusal.cause.map(cause_word),
            row["expected_cause"].as_str(),
            "{}: {refusal:?}",
            row["id"]
        );
        assert_eq!(
            refusal.path.unwrap().to_string(),
            row["locus"].as_str().unwrap(),
            "{}",
            row["id"]
        );
    }
}

#[trace("TC-367", "FR-038-AC-323")]
#[test]
fn timed_model_tm_23_to_tm_29_enforce_dense_time_and_base_time_identity() {
    let vectors = read_json(&fixtures_dir().join("../timed-model-node-vectors.json")).unwrap();
    for row in vectors["semantic_refusals"].as_array().unwrap() {
        let id = row["id"].as_str().unwrap();
        if !matches!(
            id,
            "TM-23" | "TM-24" | "TM-25" | "TM-26" | "TM-27" | "TM-28" | "TM-29"
        ) {
            continue;
        }
        let fixture = match row["fixture"].as_str().unwrap_or("dense") {
            "dense" => "positive-timed-model-nodes.json",
            "tick" => "positive-tick-timed-model-nodes.json",
            other => panic!("{id}: unknown fixture {other}"),
        };
        let mut wire = read_json(&fixtures_dir().join(fixture)).unwrap();
        match row["mutation"].as_str().unwrap() {
            "tick-source-with-clock-place" => {
                let dense = published();
                let place = dense["semantic_graph"]["nodes"][45].clone();
                let place_id = place["node_id"].clone();
                wire["semantic_graph"]["nodes"]
                    .as_array_mut()
                    .unwrap()
                    .push(place);
                let root = &mut wire["semantic_graph"]["nodes"][45];
                root["body"]["members"][3]["value"]["members"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({"term":"reference", "target":place_id}));
                root["dependencies"].as_array_mut().unwrap().push(place_id);
                root["dependencies"]
                    .as_array_mut()
                    .unwrap()
                    .sort_by(|a, b| a["digest"].as_str().cmp(&b["digest"].as_str()));
            }
            "source-unit-length" | "source-unit-derived-non-time" | "source-unit-derived-time" => {
                let mutation = row["mutation"].as_str().unwrap();
                let name = match mutation {
                    "source-unit-length" => "Length",
                    "source-unit-derived-non-time" => "Area",
                    _ => "Duration",
                };
                let time_id = wire["semantic_graph"]["nodes"][42]["node_id"].clone();
                let mut dimension = wire["semantic_graph"]["nodes"][42].clone();
                dimension["nominal_identity_preimage"]["qualified_declaration"] =
                    json!(["Example", name]);
                dimension["declaration"]["qualified_name"] = json!(["Example", name]);
                if name == "Area" || name == "Duration" {
                    let base_id = if name == "Area" {
                        let mut length = dimension.clone();
                        length["nominal_identity_preimage"]["qualified_declaration"] =
                            json!(["Example", "Length"]);
                        length["declaration"]["qualified_name"] = json!(["Example", "Length"]);
                        let id = fixture_support::node_id(&fixture_support::sha256_hex(
                            &fixture_support::canonical(&length["nominal_identity_preimage"]),
                        ));
                        length["node_id"] = id.clone();
                        length["semantic_type"] = id.clone();
                        wire["semantic_graph"]["nodes"]
                            .as_array_mut()
                            .unwrap()
                            .push(length);
                        id
                    } else {
                        time_id
                    };
                    dimension["nominal_identity_preimage"]["terms"] = json!([{
                        "dimension_node_id": base_id,
                        "exponent": if name == "Area" { "2" } else { "1" }
                    }]);
                    dimension["dependencies"] = json!([base_id]);
                }
                let dimension_id = fixture_support::node_id(&fixture_support::sha256_hex(
                    &fixture_support::canonical(&dimension["nominal_identity_preimage"]),
                ));
                dimension["node_id"] = dimension_id.clone();
                dimension["semantic_type"] = dimension_id.clone();
                wire["semantic_graph"]["nodes"]
                    .as_array_mut()
                    .unwrap()
                    .push(dimension);
                let mut unit = wire["semantic_graph"]["nodes"][43].clone();
                unit["nominal_identity_preimage"]["qualified_declaration"] =
                    json!(["Example", "u"]);
                unit["nominal_identity_preimage"]["dimension_node_id"] = dimension_id.clone();
                unit["node_id"] = fixture_support::node_id(&fixture_support::sha256_hex(
                    &fixture_support::canonical(&unit["nominal_identity_preimage"]),
                ));
                unit["semantic_type"] = dimension_id;
                unit["dependencies"] = json!([unit["semantic_type"].clone()]);
                unit["declaration"]["qualified_name"] = json!(["Example", "u"]);
                let unit_id = unit["node_id"].clone();
                wire["semantic_graph"]["nodes"]
                    .as_array_mut()
                    .unwrap()
                    .push(unit);
                let source = &mut wire["semantic_graph"]["nodes"][44];
                let old = source["body"]["members"][1]["value"]["target"].clone();
                source["body"]["members"][1]["value"]["target"] = unit_id.clone();
                let deps = source["dependencies"].as_array_mut().unwrap();
                deps.retain(|dep| dep != &old);
                deps.push(unit_id);
                deps.sort_by(|a, b| a["digest"].as_str().cmp(&b["digest"].as_str()));
            }
            other => panic!("{id}: unknown mutation {other}"),
        }
        fixture_support::mint_ungrouped_structural_keys(&mut wire);
        for node in wire["semantic_graph"]["nodes"].as_array_mut().unwrap() {
            node["dependencies"]
                .as_array_mut()
                .unwrap()
                .sort_by(|left, right| left["digest"].as_str().cmp(&right["digest"].as_str()));
        }
        fixture_support::refresh_identity(&mut wire);
        let CheckedPackageV2ReadResult::Refused(refusal) = read_package(&wire).unwrap() else {
            panic!("{id} must refuse");
        };
        assert_eq!(
            code_word(refusal.code),
            row["expected_code"].as_str().unwrap(),
            "{id}: {refusal:?}"
        );
        assert_eq!(
            refusal.cause.map(cause_word),
            row["expected_cause"].as_str(),
            "{id}: {refusal:?}"
        );
        assert_eq!(
            refusal.path.unwrap().to_string(),
            row["locus"].as_str().unwrap(),
            "{id}"
        );
    }
}

fn replace_dependency(node: &mut Value, old: &Value, new: &Value) {
    for dependency in node["dependencies"].as_array_mut().expect("dependencies") {
        if dependency == old {
            *dependency = new.clone();
        }
    }
    node["dependencies"]
        .as_array_mut()
        .unwrap()
        .sort_by(|left, right| left["digest"].as_str().cmp(&right["digest"].as_str()));
}

#[trace("TC-048")]
#[test]
fn timed_model_missing_root_precedes_later_quantity_validation() {
    let mut wire = published();
    let root = wire["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["node_tag"] == "timed_model" && node["semantic_form"] == "timed_model")
        .expect("published root")["node_id"]
        .clone();
    wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .retain(|node| node["node_id"] != root);
    wire["source_map"]
        .as_array_mut()
        .unwrap()
        .retain(|entry| entry["node_id"] != root);
    fixture_support::refresh_identity(&mut wire);
    let CheckedPackageV2ReadResult::Refused(refusal) = read_package(&wire).unwrap() else {
        panic!("a timed clause without a root must refuse");
    };
    assert_eq!(refusal.code, Code::InvalidPackage, "{refusal:?}");
    assert_eq!(refusal.cause, Some(Cause::MissingTimedModel), "{refusal:?}");
    assert!(
        refusal.locus.is_some(),
        "missing root must retain affected node"
    );
}

#[trace("TC-048")]
#[test]
fn timed_model_repeated_place_precedes_later_quantity_validation() {
    let mut wire = published();
    let root = wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|node| node["node_tag"] == "timed_model" && node["semantic_form"] == "timed_model")
        .expect("published root");
    let places = root["body"]["members"][3]["value"]["members"]
        .as_array_mut()
        .unwrap();
    assert_eq!(places.len(), 1);
    places.push(places[0].clone());
    fixture_support::mint_ungrouped_structural_keys(&mut wire);
    fixture_support::refresh_identity(&mut wire);
    let CheckedPackageV2ReadResult::Refused(refusal) = read_package(&wire).unwrap() else {
        panic!("repeated model place must refuse");
    };
    assert_eq!(refusal.code, Code::InvalidPackage, "{refusal:?}");
    assert_eq!(
        refusal.cause,
        Some(Cause::DuplicateClockPlace),
        "{refusal:?}"
    );
    assert!(refusal
        .path
        .unwrap()
        .to_string()
        .ends_with("/body/members/3/value/members/1"));
}

#[trace("TC-048")]
#[test]
fn timed_model_rebuilt_semantic_defects_keep_paired_causes() {
    for (change, form, member, code, cause) in [
        (
            "zero_period",
            "time_source",
            4,
            Code::IllTyped,
            Cause::InvalidTimeQuantity,
        ),
        (
            "unreduced_bound",
            "clock_constraint",
            2,
            Code::IllTyped,
            Cause::InvalidTimeQuantity,
        ),
        (
            "wrong_predicate",
            "clock_guard",
            0,
            Code::InvalidPackage,
            Cause::TimedNodeReference,
        ),
    ] {
        let mut wire = if change == "zero_period" {
            read_json(&fixtures_dir().join("positive-tick-timed-model-nodes.json")).unwrap()
        } else {
            published()
        };
        let scalar = wire["semantic_graph"]["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|node| node["node_tag"] == "scalar_type" && node["semantic_form"] == "integer")
            .expect("published integer type")["node_id"]
            .clone();
        let node = wire["semantic_graph"]["nodes"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|node| node["node_tag"] == "timed_model" && node["semantic_form"] == form)
            .expect("published timed form");
        match change {
            "zero_period" => node["body"]["members"][member]["value"]["value"] = json!("0/1"),
            "unreduced_bound" => node["body"]["members"][member]["value"]["value"] = json!("6/4"),
            "wrong_predicate" => {
                node["body"]["members"][member]["value"]["target"] = scalar.clone();
                node["dependencies"] = json!([scalar]);
            }
            _ => unreachable!("test case list is closed"),
        }
        fixture_support::mint_ungrouped_structural_keys(&mut wire);
        for node in wire["semantic_graph"]["nodes"].as_array_mut().unwrap() {
            node["dependencies"]
                .as_array_mut()
                .unwrap()
                .sort_by(|left, right| {
                    (left["domain"].as_str(), left["digest"].as_str())
                        .cmp(&(right["domain"].as_str(), right["digest"].as_str()))
                });
        }
        fixture_support::refresh_identity(&mut wire);
        let CheckedPackageV2ReadResult::Refused(refusal) = read_package(&wire).unwrap() else {
            panic!("{change} must refuse");
        };
        assert_eq!(
            (refusal.code, refusal.cause),
            (code, Some(cause)),
            "{change}: {refusal:?}"
        );
        assert!(
            refusal
                .path
                .unwrap()
                .to_string()
                .contains(&format!("/body/members/{member}/value")),
            "{change}"
        );
    }
}

/// A root that stops listing an existing guard still leaves a real guard node.
/// STD-540 owns GD-03's exact locus; this control checks its published code
/// and cause after authenticated package identities are rebuilt.
/// Provenance: IR-1083, QSpec STD-526 GD-03.
#[trace("TC-048")]
#[test]
fn timed_model_unlisted_guard_refuses_at_the_timed_join() {
    let vectors = read_json(&fixtures_dir().join("../timed-slot-selector-vectors.json"))
        .expect("published timed slot vectors");
    let gd03 = vectors["guards"]
        .as_array()
        .expect("published guard cases")
        .iter()
        .find(|row| row["id"] == "GD-03")
        .expect("GD-03 row");
    assert_eq!(gd03["expected_code"], "invalid_package");
    assert_eq!(gd03["expected_cause"], "timed-node-reference");
    let mut wire = published();
    let root = wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .iter_mut()
        .find(|node| node["node_tag"] == "timed_model" && node["semantic_form"] == "timed_model")
        .expect("timed root");
    let guard = root["body"]["members"][5]["value"]["members"]
        .as_array_mut()
        .expect("root guards")
        .pop()
        .expect("published guard")["target"]
        .clone();
    root["dependencies"]
        .as_array_mut()
        .expect("root dependencies")
        .retain(|dependency| *dependency != guard);
    fixture_support::mint_ungrouped_structural_keys(&mut wire);
    fixture_support::refresh_identity(&mut wire);
    let result = read_package(&wire).expect("read unlisted guard package");
    let CheckedPackageV2ReadResult::Refused(refusal) = result else {
        panic!("GD-03 must refuse, got {result:?}");
    };
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::InvalidPackage, Some(Cause::TimedNodeReference)),
        "{refusal:?}"
    );
}

#[trace("TC-048")]
#[test]
fn timed_model_distinct_place_nodes_with_same_model_path_and_different_guards_refuse() {
    let mut wire = published();
    assert!(
        matches!(
            read_package(&wire).unwrap(),
            CheckedPackageV2ReadResult::Admitted(_)
        ),
        "unchanged owner fixture must admit"
    );
    let original = wire["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["node_tag"] == "timed_model" && node["semantic_form"] == "clock_place")
        .expect("published clock place")
        .clone();
    let mut alternate = original.clone();
    let provisional = json!({"domain":"quire.checked-semantic-node/v1", "digest":"f".repeat(64)});
    assert!(!wire["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|node| node["node_id"] == provisional));
    alternate["node_id"] = provisional.clone();
    alternate["semantic_type"] = provisional.clone();
    let guards = alternate["body"]["members"][3]["value"]["members"]
        .as_array_mut()
        .unwrap();
    assert_eq!(
        guards.len(),
        1,
        "published place has a genuine Boolean guard"
    );
    guards.clear();
    alternate["dependencies"] = json!([
        alternate["body"]["members"][0]["value"]["target"].clone(),
        alternate["body"]["members"][1]["value"]["members"][1]["members"][2]["value"]["target"]
            .clone()
    ]);
    alternate["dependencies"]
        .as_array_mut()
        .unwrap()
        .sort_by(|left, right| left["digest"].as_str().cmp(&right["digest"].as_str()));
    assert_eq!(
        original["body"]["members"][0],
        alternate["body"]["members"][0]
    );
    assert_eq!(
        original["body"]["members"][1],
        alternate["body"]["members"][1]
    );
    assert_ne!(
        original["body"]["members"][3],
        alternate["body"]["members"][3]
    );
    let mut source = wire["source_map"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["node_id"] == original["node_id"])
        .expect("published place provenance")
        .clone();
    source["node_id"] = provisional.clone();
    wire["source_map"].as_array_mut().unwrap().push(source);
    wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .push(alternate);
    let root = wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|node| node["node_tag"] == "timed_model" && node["semantic_form"] == "timed_model")
        .unwrap();
    root["body"]["members"][3]["value"]["members"]
        .as_array_mut()
        .unwrap()
        .push(json!({"term":"reference", "target":provisional.clone()}));
    root["dependencies"]
        .as_array_mut()
        .unwrap()
        .push(provisional);
    fixture_support::mint_ungrouped_structural_keys(&mut wire);
    for node in wire["semantic_graph"]["nodes"].as_array_mut().unwrap() {
        node["dependencies"]
            .as_array_mut()
            .unwrap()
            .sort_by(|left, right| {
                (left["domain"].as_str(), left["digest"].as_str())
                    .cmp(&(right["domain"].as_str(), right["digest"].as_str()))
            });
    }
    fixture_support::refresh_identity(&mut wire);
    let root_position = wire["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .position(|node| {
            node["node_tag"] == "timed_model" && node["semantic_form"] == "timed_model"
        })
        .unwrap();
    let root = &wire["semantic_graph"]["nodes"][root_position];
    let places = root["body"]["members"][3]["value"]["members"]
        .as_array()
        .unwrap();
    assert_ne!(
        places[0]["target"], places[1]["target"],
        "different guards must produce distinct authenticated node keys"
    );
    let refusal = match read_package(&wire).unwrap() {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        CheckedPackageV2ReadResult::Admitted(_) => panic!(
            "distinct authenticated place nodes with the same model/path were wrongly admitted"
        ),
        other => panic!("duplicate place must give a typed refusal: {other:?}"),
    };
    assert_eq!(
        (refusal.code, refusal.cause),
        (Code::InvalidPackage, Some(Cause::DuplicateClockPlace)),
        "{refusal:?}"
    );
    assert_eq!(
        refusal.path.unwrap().to_string(),
        format!("/semantic_graph/nodes/{root_position}/body/members/3/value/members/1")
    );
    assert_eq!(
        serde_json::to_value(refusal.locus).unwrap(),
        root["node_id"]
    );
}
