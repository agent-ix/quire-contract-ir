// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Structural V2 node identities: the reader retains authored positions,
//! rejects duplicates, and does not derive identifiers from node bodies.
//! The published multi-package and graph-order vectors run in conformance_qspec.

use crate::support::checked_package::{
    canonical, evidence_for, settle, structural_id, v2_all_families,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedPackageReadLimits, CheckedPackageRefusalCause, CheckedPackageRefusalCode,
    CheckedPackageV2, CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};

fn local_package() -> Value {
    let mut package = v2_all_families();
    let node = package["semantic_graph"]["nodes"][0].clone();
    let id = node["node_id"].clone();
    package["semantic_graph"]["nodes"] = json!([node]);
    package["source_map"] = json!(package["source_map"]
        .as_array()
        .expect("source map")
        .iter()
        .filter(|entry| entry["node_id"] == id)
        .cloned()
        .collect::<Vec<_>>());
    package
}

fn read(package: &Value) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(package),
    )
}

fn index_of(package: &Value, tag: &str, form: &str) -> usize {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("graph nodes")
        .iter()
        .position(|node| node["node_tag"] == tag && node["semantic_form"] == form)
        .unwrap_or_else(|| panic!("missing {tag}/{form}"))
}

fn append_shape(package: &mut Value, tag: &str, form: &str, mut node: Value) -> usize {
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("graph nodes")
        .len();
    let id = structural_id(
        package["name"].as_str().expect("package name"),
        vec![json!("derived-shape"), json!(form)],
        0,
    );
    node["node_id"] = id.clone();
    node["node_tag"] = json!(tag);
    node["semantic_form"] = json!(form);
    if tag == "scalar_type" || tag == "composite_type" {
        node["semantic_type"] = id.clone();
    }
    let mut source = package["source_map"][0].clone();
    source["node_id"] = id;
    source["role"] = node["occurrences"][0]["role"].clone();
    source["ordinal"] = node["occurrences"][0]["ordinal"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("graph nodes")
        .push(node);
    package["source_map"]
        .as_array_mut()
        .expect("source map")
        .push(source);
    position
}

fn all_ten_shapes() -> (Value, Vec<(&'static str, usize)>) {
    let mut package = v2_all_families();
    let selected = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("graph nodes")
        .iter()
        .filter(|node| {
            matches!(
                (node["node_tag"].as_str(), node["semantic_form"].as_str()),
                (Some("scalar_type"), Some("boolean" | "integer"))
                    | (Some("composite_type"), Some("option"))
                    | (Some("bounded_domain"), Some("integer_range"))
                    | (Some("model"), Some("object_type" | "model_root"))
            )
        })
        .cloned()
        .collect::<Vec<_>>();
    let selected_ids = selected
        .iter()
        .map(|node| node["node_id"].clone())
        .collect::<Vec<_>>();
    package["semantic_graph"]["nodes"] = json!(selected);
    package["source_map"] = json!(package["source_map"]
        .as_array()
        .expect("source map")
        .iter()
        .filter(|entry| selected_ids.contains(&entry["node_id"]))
        .cloned()
        .collect::<Vec<_>>());
    let boolean = package["semantic_graph"]["nodes"][index_of(&package, "scalar_type", "boolean")]
        ["node_id"]
        .clone();
    let integer = package["semantic_graph"]["nodes"][index_of(&package, "scalar_type", "integer")]
        ["node_id"]
        .clone();
    let object = package["semantic_graph"]["nodes"][index_of(&package, "model", "object_type")]
        ["node_id"]
        .clone();
    let mut positions = vec![
        ("boolean", index_of(&package, "scalar_type", "boolean")),
        ("integer", index_of(&package, "scalar_type", "integer")),
        ("option", index_of(&package, "composite_type", "option")),
        (
            "integer_range",
            index_of(&package, "bounded_domain", "integer_range"),
        ),
    ];
    let template =
        package["semantic_graph"]["nodes"][index_of(&package, "composite_type", "option")].clone();
    for form in ["reference", "set", "bag", "sequence", "ordered_set"] {
        let mut node = template.clone();
        node["body"]["members"][0]["target"] = if form == "reference" {
            object.clone()
        } else {
            boolean.clone()
        };
        let position = append_shape(&mut package, "composite_type", form, node);
        positions.push((form, position));
    }
    let mut bounds = package["semantic_graph"]["nodes"]
        [index_of(&package, "bounded_domain", "integer_range")]
    .clone();
    bounds["semantic_type"] = package["semantic_graph"]["nodes"]
        [index_of(&package, "composite_type", "set")]["node_id"]
        .clone();
    bounds["body"]["members"][0]["value"]["value"] = json!("0");
    bounds["body"]["members"][1]["value"]["value"] = json!("3");
    bounds["body"]["members"][0]["value"]["type"] = integer.clone();
    bounds["body"]["members"][1]["value"]["type"] = integer;
    let position = append_shape(&mut package, "bounded_domain", "collection_bounds", bounds);
    positions.push(("collection_bounds", position));
    settle(&mut package);
    (package, positions)
}

fn assert_derived_refusal(package: &Value, position: usize) {
    let CheckedPackageV2ReadResult::Refused(refusal) = read(package) else {
        panic!("derived shape at node {position} must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::InvalidDerivedShape)
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some(format!("/semantic_graph/nodes/{position}/node_id"))
    );
}

#[trace("TC-226", "FR-038-AC-124", "FR-038-AC-125")]
#[test]
fn ten_derived_shapes_admit_with_structural_ids_and_refuse_wrong_types() {
    let (package, positions) = all_ten_shapes();
    assert!(
        matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
        "ten valid shapes: {:?}",
        read(&package)
    );
    let boolean = package["semantic_graph"]["nodes"][index_of(&package, "scalar_type", "boolean")]
        ["node_id"]
        .clone();
    let integer = package["semantic_graph"]["nodes"][index_of(&package, "scalar_type", "integer")]
        ["node_id"]
        .clone();
    for (form, position) in positions {
        let mut wrong = package.clone();
        let node = &mut wrong["semantic_graph"]["nodes"][position];
        if form == "integer_range" || form == "collection_bounds" || form == "integer" {
            node["semantic_type"] = boolean.clone();
        } else {
            node["semantic_type"] = integer.clone();
        }
        assert_derived_refusal(&wrong, position);
    }
    for form in ["integer_range", "collection_bounds"] {
        let position = index_of(&package, "bounded_domain", form);
        let mut wrong = package.clone();
        wrong["semantic_graph"]["nodes"][position]["body"]["members"][0]["value"]["type"] =
            boolean.clone();
        assert_derived_refusal(&wrong, position);
    }
}

#[trace("TC-226", "FR-038-AC-124")]
#[test]
fn ten_derived_shapes_refuse_open_or_malformed_bodies_at_their_nodes() {
    let (package, positions) = all_ten_shapes();
    let boolean = package["semantic_graph"]["nodes"][index_of(&package, "scalar_type", "boolean")]
        ["node_id"]
        .clone();
    for (form, position) in positions {
        let mut missing = package.clone();
        let members = missing["semantic_graph"]["nodes"][position]["body"]["members"]
            .as_array_mut()
            .expect("aggregate members");
        if matches!(form, "boolean" | "integer") {
            members.push(json!({"term":"reference","target":boolean}));
        } else {
            members.pop();
        }
        assert_derived_refusal(&missing, position);
    }

    for form in ["integer_range", "collection_bounds"] {
        let position = index_of(&package, "bounded_domain", form);
        let mut reversed = package.clone();
        reversed["semantic_graph"]["nodes"][position]["body"]["members"]
            .as_array_mut()
            .expect("bounds members")
            .swap(0, 1);
        assert_derived_refusal(&reversed, position);

        let mut extra = package.clone();
        let member = extra["semantic_graph"]["nodes"][position]["body"]["members"][1].clone();
        extra["semantic_graph"]["nodes"][position]["body"]["members"]
            .as_array_mut()
            .expect("bounds members")
            .push(member);
        assert_derived_refusal(&extra, position);

        for malformed in ["01", "+1", "-0", "1.0", "one"] {
            let mut changed = package.clone();
            changed["semantic_graph"]["nodes"][position]["body"]["members"][0]["value"]["value"] =
                json!(malformed);
            assert_derived_refusal(&changed, position);
        }
    }
}

#[trace("TC-226", "FR-038-AC-128")]
#[test]
fn integer_range_extreme_bound_text_is_retained_without_rounding() {
    let (mut package, _) = all_ten_shapes();
    let position = index_of(&package, "bounded_domain", "integer_range");
    for (min, max) in [
        ("0", "0"),
        (
            "-170141183460469231731687303715884105728",
            "170141183460469231731687303715884105727",
        ),
        (
            "-170141183460469231731687303715884105729",
            "170141183460469231731687303715884105728",
        ),
        ("-5", "7"),
    ] {
        package["semantic_graph"]["nodes"][position]["body"]["members"][0]["value"]["value"] =
            json!(min);
        package["semantic_graph"]["nodes"][position]["body"]["members"][1]["value"]["value"] =
            json!(max);
        let CheckedPackageV2ReadResult::Admitted(admitted) = read(&package) else {
            panic!("canonical decimal endpoints {min}..{max} must admit");
        };
        let retained = serde_json::to_value(&admitted.graph().nodes[position].body)
            .expect("typed body serialization");
        assert_eq!(retained["members"][0]["value"]["value"], min);
        assert_eq!(retained["members"][1]["value"]["value"], max);
    }
}

#[trace("TC-048")]
#[test]
fn structural_identity_is_retained_without_body_derivation() {
    let first = local_package();
    let mut changed = first.clone();
    changed["semantic_graph"]["nodes"][0]["semantic_form"] = json!("integer");
    let CheckedPackageV2ReadResult::Admitted(first_read) = read(&first) else {
        panic!("original scalar must admit: {:?}", read(&first));
    };
    let CheckedPackageV2ReadResult::Admitted(changed_read) = read(&changed) else {
        panic!("changed scalar must admit: {:?}", read(&changed));
    };
    assert_eq!(
        first_read.graph().nodes[0].node_id,
        changed_read.graph().nodes[0].node_id
    );
    assert_ne!(first_read.graph().nodes[0], changed_read.graph().nodes[0]);
    assert_eq!(
        serde_json::to_value(&changed_read.graph().nodes[0].node_id).expect("node id"),
        first["semantic_graph"]["nodes"][0]["node_id"]
    );
}

#[trace("TC-048")]
#[test]
fn second_equal_structural_node_id_refuses_at_that_node() {
    let mut package = local_package();
    let duplicate = package["semantic_graph"]["nodes"][0].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(duplicate);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("duplicate node id must refuse: {:?}", read(&package));
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some("/semantic_graph/nodes/1/node_id".to_owned())
    );
}

#[trace("TC-048")]
#[test]
fn a_distinct_path_is_a_distinct_node_even_with_equal_bodies() {
    let mut package = local_package();
    let mut second = package["semantic_graph"]["nodes"][0].clone();
    second["node_id"]["path"] = json!(["other-boolean"]);
    second["semantic_type"] = second["node_id"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(second.clone());
    let mut source = package["source_map"][0].clone();
    source["node_id"] = second["node_id"].clone();
    package["source_map"]
        .as_array_mut()
        .expect("source map")
        .push(source);
    let CheckedPackageV2ReadResult::Admitted(admitted) = read(&package) else {
        panic!("distinct structural path must admit: {:?}", read(&package));
    };
    assert_ne!(
        admitted.graph().nodes[0].node_id,
        admitted.graph().nodes[1].node_id
    );
    assert_eq!(
        admitted.graph().nodes[0].body,
        admitted.graph().nodes[1].body
    );
}
