// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Timed-model wire evidence read from the owning QSpec checkout.
//! Suite-only traces retain the structural/round-trip and scoped refusal evidence;
//! full timed semantic criterion qualification belongs to the follow-on slice.
use super::*;
use quire_contract_model::{encode_checked_package_v2, CheckedPackageIdentityPreimageRefV2};

fn published() -> Value {
    read_json(&fixtures_dir().join("positive-timed-model-nodes.json"))
        .expect("published timed model must be available")
}

#[trace("TC-048")]
#[test]
fn timed_model_published_fixture_round_trips_complete_nodes() {
    let wire = published();
    assert_eq!(
        wire["semantic_graph"]["nodes"].as_array().unwrap().len(),
        53
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
            node.body
        );
    }
    let s = package.identity_preimage();
    let preimage = CheckedPackageIdentityPreimageRefV2 {
        version: &s.version,
        edition: &s.edition,
        profile_selections: &s.profile_selections,
        definition_selections: &s.definition_selections,
        model_selections: &s.model_selections,
        required_features: &s.required_features,
        dependency_selections: &s.dependency_selections,
        identity_projection: &package.graph().nodes,
    };
    let bytes = encode_checked_package_v2(
        &preimage,
        package.package_id(),
        package.lock(),
        package.graph(),
        package.source_map(),
        package.capability_report(),
        package.diagnostics(),
        Limits::new(1 << 24),
    )
    .unwrap();
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
            "bad-slot-selector" => {
                body[1]["value"]["members"][1]["members"][2]["value"]["value_kind"] = json!("text")
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
    let places = root["body"]["members"][2]["value"]["members"]
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
        .ends_with("/body/members/2/value/members/1"));
}

#[trace("TC-048")]
#[test]
fn timed_model_rebuilt_semantic_defects_keep_paired_causes() {
    for (change, form, member, code, cause) in [
        (
            "zero_period",
            "time_source",
            3,
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
        let mut wire = published();
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
