// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Structural V2 node identities: the reader retains authored positions,
//! rejects duplicates, and does not derive identifiers from node bodies.
//! The published multi-package and graph-order vectors run in conformance_qspec.

use crate::support::checked_package::{
    canonical, evidence_for, settle, structural_id, v2_all_families, v2_nominal, v2_reference_chain,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedNodeId, CheckedPackageLimit, CheckedPackageReadLimits, CheckedPackageRefusalCause,
    CheckedPackageRefusalCode, CheckedPackageV2, CheckedPackageV2ReadResult,
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

fn read_with_work(package: &Value, work: u64) -> CheckedPackageV2ReadResult {
    let mut limits = CheckedPackageReadLimits::bounded();
    limits.work = work;
    CheckedPackageV2::read(&canonical(package), limits, &evidence_for(package))
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

fn append_authored(package: &mut Value, node: Value) -> usize {
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("graph nodes")
        .len();
    let mut source = package["source_map"][0].clone();
    source["node_id"] = node["node_id"].clone();
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

#[trace("TC-226", "FR-038-AC-134")]
#[test]
fn full_in_repo_family_inventory_admits_and_retains_authored_ids() {
    let (chain_bytes, _) = v2_reference_chain(2);
    let chain: Value = serde_json::from_slice(&chain_bytes).expect("reference-chain JSON");
    for (label, package) in [
        ("all families", v2_all_families()),
        ("nominal", v2_nominal()),
        ("reference chain", chain),
    ] {
        let CheckedPackageV2ReadResult::Admitted(admitted) = read(&package) else {
            panic!("{label} fixture must admit: {:?}", read(&package));
        };
        let published = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("fixture nodes");
        assert_eq!(admitted.graph().nodes.len(), published.len(), "{label}");
        for (position, (node, wire)) in admitted.graph().nodes.iter().zip(published).enumerate() {
            assert_eq!(
                serde_json::to_value(&node.node_id).expect("node id"),
                wire["node_id"],
                "{label}: node {position}"
            );
            assert_eq!(
                serde_json::to_value(&node.owner).expect("owner"),
                wire["owner"],
                "{label}: node {position}"
            );
            assert_eq!(
                serde_json::to_value(&node.body).expect("body"),
                wire["body"],
                "{label}: node {position}"
            );
        }
    }
}

#[trace("TC-226", "FR-038-AC-134")]
#[test]
fn broken_structural_reference_refuses_without_a_placeholder_key() {
    let (mut package, _) = all_ten_shapes();
    let position = index_of(&package, "composite_type", "set");
    let missing = structural_id("test/families", vec![json!("Missing")], 0);
    package["semantic_graph"]["nodes"][position]["body"]["members"][0]["target"] = missing;
    settle(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("unresolved structural target must refuse");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph,
        "{refusal:?}"
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some(format!(
            "/semantic_graph/nodes/{position}/body/members/0/target"
        )),
        "{refusal:?}"
    );
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
    assert_eq!(
        refusal.locus,
        Some(
            serde_json::from_value(package["semantic_graph"]["nodes"][position]["node_id"].clone())
                .expect("authored structural locus")
        )
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

    for form in [
        "option",
        "reference",
        "set",
        "bag",
        "sequence",
        "ordered_set",
    ] {
        let position = index_of(&package, "composite_type", form);
        let mut extra = package.clone();
        let member = extra["semantic_graph"]["nodes"][position]["body"]["members"][0].clone();
        extra["semantic_graph"]["nodes"][position]["body"]["members"]
            .as_array_mut()
            .expect("reference member")
            .push(member);
        assert_derived_refusal(&extra, position);

        let mut wrong_term = package.clone();
        wrong_term["semantic_graph"]["nodes"][position]["body"]["members"][0] = json!({
            "term":"literal", "type":boolean, "value_kind":"boolean", "value":true
        });
        assert_derived_refusal(&wrong_term, position);
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

#[trace("TC-226", "FR-038-AC-128")]
#[test]
fn declared_record_field_joins_the_exact_integer_range_members() {
    let (mut package, _) = all_ten_shapes();
    let range = index_of(&package, "bounded_domain", "integer_range");
    let range_id = package["semantic_graph"]["nodes"][range]["node_id"].clone();
    let source = package["lock"]["sources"][0].clone();
    let name = package["name"].as_str().expect("package name").to_owned();
    let record_id = structural_id(&name, vec![json!("CountRecord")], 0);
    let mut record = package["semantic_graph"]["nodes"][0].clone();
    record["node_id"] = record_id.clone();
    record["node_tag"] = json!("composite_type");
    record["semantic_form"] = json!("record");
    record["semantic_type"] = record_id;
    record["dependencies"] = json!([range_id]);
    record["occurrences"] = json!([{"role":"declaration","ordinal":0}]);
    record["declaration"] = json!({"qualified_name":["CountRecord"]});
    record["owner"] =
        json!({"kind":"source","authority":source["authority"],"identity":source["identity"]});
    record["body"] = json!({"term":"aggregate","members":[
        {"term":"binding","name":"count","value":{"term":"reference","target":range_id}}
    ]});
    let record_position = append_authored(&mut package, record);
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
    ] {
        package["semantic_graph"]["nodes"][range]["body"]["members"][0]["value"]["value"] =
            json!(min);
        package["semantic_graph"]["nodes"][range]["body"]["members"][1]["value"]["value"] =
            json!(max);
        let CheckedPackageV2ReadResult::Admitted(admitted) = read(&package) else {
            panic!(
                "declared count field at {min}..{max} must admit: {:?}",
                read(&package)
            );
        };
        let retained_range =
            serde_json::to_value(&admitted.graph().nodes[range].body).expect("range body");
        assert_eq!(retained_range["members"][0]["value"]["value"], min);
        assert_eq!(retained_range["members"][1]["value"]["value"], max);
        let retained_record = serde_json::to_value(&admitted.graph().nodes[record_position].body)
            .expect("record body");
        assert_eq!(
            retained_record["members"][0]["value"]["target"],
            package["semantic_graph"]["nodes"][range]["node_id"]
        );
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

#[trace("TC-226", "FR-038-AC-127")]
#[test]
fn derived_shape_order_uses_structural_ids_after_graph_shape() {
    let (package, _) = all_ten_shapes();
    let mut two_bad = package.clone();
    let boolean = index_of(&two_bad, "scalar_type", "boolean");
    let set = index_of(&two_bad, "composite_type", "set");
    let boolean_id: CheckedNodeId =
        serde_json::from_value(two_bad["semantic_graph"]["nodes"][boolean]["node_id"].clone())
            .expect("Boolean id");
    let set_id: CheckedNodeId =
        serde_json::from_value(two_bad["semantic_graph"]["nodes"][set]["node_id"].clone())
            .expect("Set id");
    assert!(boolean_id < set_id);
    two_bad["semantic_graph"]["nodes"][boolean]["body"]["members"] =
        json!([{"term":"reference","target":set_id}]);
    two_bad["semantic_graph"]["nodes"][set]["body"]["members"] = json!([]);
    two_bad["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .swap(boolean, set);
    let boolean_position = index_of(&two_bad, "scalar_type", "boolean");
    let set_position = index_of(&two_bad, "composite_type", "set");
    assert!(boolean_position > set_position, "reverse graph order");
    assert_derived_refusal(&two_bad, boolean_position);

    let mut duplicate_and_bad = package.clone();
    let duplicate = duplicate_and_bad["semantic_graph"]["nodes"][boolean].clone();
    duplicate_and_bad["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(duplicate);
    duplicate_and_bad["semantic_graph"]["nodes"][set]["body"]["members"] = json!([]);
    let duplicate_position = duplicate_and_bad["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&duplicate_and_bad) else {
        panic!("duplicate graph id must refuse before derived body");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some(format!(
            "/semantic_graph/nodes/{duplicate_position}/node_id"
        ))
    );
}

#[trace("TC-226", "FR-038-AC-127")]
#[test]
fn source_owner_join_precedes_derived_shape_refusal() {
    let (mut package, option, record) = list_group();
    package["semantic_graph"]["nodes"][record]["owner"]["identity"] = json!("unselected-source");
    package["semantic_graph"]["nodes"][option]["body"]["members"] = json!([]);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("unselected source owner and malformed Option must refuse");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::MissingDeclaration,
        "{refusal:?}"
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some(format!("/semantic_graph/nodes/{record}/node_id")),
        "{refusal:?}"
    );
}

#[trace("TC-048", "FR-038-AC-185")]
#[test]
fn pregraph_refusal_invents_no_structural_node_locus() {
    let mut package = local_package();
    package["semantic_graph"]["nodes"][0]
        .as_object_mut()
        .expect("node")
        .remove("node_id");
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("missing required node_id must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MalformedWire);
    assert_eq!(refusal.locus, None);
    let display = refusal.to_string();
    assert!(!display.contains("expected_node_id"));
    let serialized = serde_json::to_value(&refusal).expect("public refusal serialization");
    assert!(serialized["locus"].is_null());
    assert!(serialized.get("expected_node_id").is_none());
}

fn self_option_group(ordinal: u64) -> (Value, usize) {
    let mut package = local_package();
    let mut option = package["semantic_graph"]["nodes"][0].clone();
    option["body"] = json!({"term":"aggregate","members":[]});
    option["recursion_group"] = json!("self-option");
    let position = append_shape(&mut package, "composite_type", "option", option);
    package["semantic_graph"]["nodes"][position]["node_id"]["ordinal"] = json!(ordinal);
    let self_id = package["semantic_graph"]["nodes"][position]["node_id"].clone();
    package["semantic_graph"]["nodes"][position]["semantic_type"] = self_id.clone();
    package["semantic_graph"]["nodes"][position]["dependencies"] = json!([self_id]);
    package["semantic_graph"]["nodes"][position]["body"]["members"] =
        json!([{"term":"reference","target":self_id}]);
    settle(&mut package);
    (package, position)
}

#[trace("TC-226", "FR-038-AC-145", "FR-038-AC-146")]
#[test]
fn owner_free_self_option_group_obeys_closed_shape_checks() {
    let (package, position) = self_option_group(7);
    assert!(
        matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
        "valid self-option group: {:?}",
        read(&package)
    );
    let mut malformed = package.clone();
    malformed["semantic_graph"]["nodes"][position]["body"]["members"] = json!([]);
    assert_derived_refusal(&malformed, position);

    let mut ungrouped = package;
    ungrouped["semantic_graph"]["nodes"][position]
        .as_object_mut()
        .expect("option node")
        .remove("recursion_group");
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&ungrouped) else {
        panic!("unlabelled self-reference must refuse");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some(format!("/semantic_graph/nodes/{position}")),
        "{refusal:?}"
    );
}

fn list_group() -> (Value, usize, usize) {
    let mut package = local_package();
    let name = package["name"].as_str().expect("package name");
    let list_id = structural_id(name, vec![json!("List")], 4);
    let option_id = structural_id(
        name,
        vec![json!("composite_type"), json!("option"), list_id.clone()],
        9,
    );
    let mut option = package["semantic_graph"]["nodes"][0].clone();
    option["node_id"] = option_id.clone();
    option["node_tag"] = json!("composite_type");
    option["semantic_form"] = json!("option");
    option["semantic_type"] = option_id.clone();
    option["dependencies"] = json!([list_id]);
    option["recursion_group"] = json!("List");
    option["body"] = json!({"term":"aggregate","members":[
        {"term":"reference","target":list_id}
    ]});
    let option_position = append_authored(&mut package, option);

    let source = package["lock"]["sources"][0].clone();
    let mut record = package["semantic_graph"]["nodes"][0].clone();
    record["node_id"] = list_id.clone();
    record["node_tag"] = json!("composite_type");
    record["semantic_form"] = json!("record");
    record["semantic_type"] = list_id;
    record["dependencies"] = json!([option_id]);
    record["occurrences"] = json!([{"role":"declaration","ordinal":0}]);
    record["recursion_group"] = json!("List");
    record["declaration"] = json!({"qualified_name":["List"]});
    record["owner"] =
        json!({"kind":"source","authority":source["authority"],"identity":source["identity"]});
    record["body"] = json!({"term":"aggregate","members":[
        {"term":"binding","name":"next","value":{
            "term":"aggregate","members":[
                {"term":"binding","name":"optional","value":{
                    "term":"reference","target":option_id
                }}
            ]
        }}
    ]});
    let record_position = append_authored(&mut package, record);
    settle(&mut package);
    (package, option_position, record_position)
}

#[trace("TC-226", "FR-038-AC-145", "FR-038-AC-150")]
#[test]
fn declared_list_group_admits_with_independent_node_ordinals() {
    let (package, option, record) = list_group();
    assert!(
        matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
        "declared List group: {:?}",
        read(&package)
    );
    assert_eq!(
        package["semantic_graph"]["nodes"][option]["node_id"]["ordinal"],
        9
    );
    assert_eq!(
        package["semantic_graph"]["nodes"][record]["node_id"]["ordinal"],
        4
    );

    let mut permuted = package.clone();
    permuted["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("group nodes")
        .swap(option, record);
    assert!(
        matches!(read(&permuted), CheckedPackageV2ReadResult::Admitted(_)),
        "consistent graph permutation retains producer-assigned node ids: {:?}",
        read(&permuted)
    );
    let mut missing_join = package;
    missing_join["semantic_graph"]["nodes"][record]["body"]["members"][0]["value"]["members"][0]
        ["value"]["target"]["ordinal"] = json!(99);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&missing_join) else {
        panic!("wrong group reference ordinal must refuse");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
}

fn tree_group() -> (Value, usize, usize, usize) {
    let mut package = local_package();
    let name = package["name"].as_str().expect("package name").to_owned();
    let tree = structural_id(&name, vec![json!("Tree")], 11);
    let sequence = structural_id(
        &name,
        vec![json!("composite_type"), json!("sequence"), tree.clone()],
        14,
    );
    let bounds = structural_id(
        &name,
        vec![
            json!("bounded_domain"),
            json!("collection_bounds"),
            sequence.clone(),
        ],
        23,
    );
    let integer = v2_all_families()["semantic_graph"]["nodes"]
        .as_array()
        .expect("family nodes")
        .iter()
        .find(|node| node["node_tag"] == "scalar_type" && node["semantic_form"] == "integer")
        .expect("Integer node")
        .clone();
    let integer_id = integer["node_id"].clone();
    append_authored(&mut package, integer);

    let source = package["lock"]["sources"][0].clone();
    let mut record = package["semantic_graph"]["nodes"][0].clone();
    record["node_id"] = tree.clone();
    record["node_tag"] = json!("composite_type");
    record["semantic_form"] = json!("record");
    record["semantic_type"] = tree.clone();
    record["dependencies"] = json!([bounds]);
    record["occurrences"] = json!([{"role":"declaration","ordinal":0}]);
    record["recursion_group"] = json!("Tree");
    record["declaration"] = json!({"qualified_name":["Tree"]});
    record["owner"] =
        json!({"kind":"source","authority":source["authority"],"identity":source["identity"]});
    record["body"] = json!({"term":"aggregate","members":[
        {"term":"binding","name":"kids","value":{
            "term":"reference","target":bounds
        }}
    ]});
    let record_position = append_authored(&mut package, record);

    let mut sequence_node = package["semantic_graph"]["nodes"][0].clone();
    sequence_node["node_id"] = sequence.clone();
    sequence_node["node_tag"] = json!("composite_type");
    sequence_node["semantic_form"] = json!("sequence");
    sequence_node["semantic_type"] = sequence.clone();
    sequence_node["dependencies"] = json!([tree]);
    sequence_node["recursion_group"] = json!("Tree");
    sequence_node["body"] = json!({"term":"aggregate","members":[
        {"term":"reference","target":tree}
    ]});
    let sequence_position = append_authored(&mut package, sequence_node);

    let mut bounds_node = package["semantic_graph"]["nodes"][0].clone();
    bounds_node["node_id"] = bounds.clone();
    bounds_node["node_tag"] = json!("bounded_domain");
    bounds_node["semantic_form"] = json!("collection_bounds");
    bounds_node["semantic_type"] = sequence.clone();
    bounds_node["dependencies"] = json!([sequence]);
    bounds_node["recursion_group"] = json!("Tree");
    bounds_node["body"] = json!({"term":"aggregate","members":[
        {"term":"binding","name":"min","value":{
            "term":"literal","type":integer_id,"value_kind":"integer","value":"0"
        }},
        {"term":"binding","name":"max","value":{
            "term":"literal","type":integer_id,"value_kind":"integer","value":"3"
        }}
    ]});
    let bounds_position = append_authored(&mut package, bounds_node);
    settle(&mut package);
    (package, record_position, sequence_position, bounds_position)
}

#[trace("TC-226", "FR-038-AC-145", "FR-038-AC-147")]
#[test]
fn declared_tree_group_checks_sequence_and_bounded_members() {
    let (package, record, sequence, bounds) = tree_group();
    assert!(
        matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
        "declared Tree group: {:?}",
        read(&package)
    );
    assert_eq!(
        package["semantic_graph"]["nodes"][record]["node_id"]["ordinal"],
        11
    );
    assert_eq!(
        package["semantic_graph"]["nodes"][sequence]["node_id"]["ordinal"],
        14
    );
    assert_eq!(
        package["semantic_graph"]["nodes"][bounds]["node_id"]["ordinal"],
        23
    );
    let mut malformed = package.clone();
    malformed["semantic_graph"]["nodes"][bounds]["body"]["members"][1]["name"] = json!("other");
    assert_derived_refusal(&malformed, bounds);
    let mut unresolved = package;
    unresolved["semantic_graph"]["nodes"][sequence]["body"]["members"][0]["target"]["ordinal"] =
        json!(99);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&unresolved) else {
        panic!("unresolved Tree reference must refuse");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
}

fn off_cycle_tree_group() -> (Value, usize) {
    let (mut package, _, _, _) = tree_group();
    let mut range = v2_all_families()["semantic_graph"]["nodes"]
        .as_array()
        .expect("family nodes")
        .iter()
        .find(|node| {
            node["node_tag"] == "bounded_domain" && node["semantic_form"] == "integer_range"
        })
        .expect("integer range")
        .clone();
    let range_id = structural_id(
        package["name"].as_str().expect("package name"),
        vec![json!("range"), json!("0"), json!("1000")],
        5,
    );
    range["node_id"] = range_id.clone();
    append_authored(&mut package, range);

    let mut set = package["semantic_graph"]["nodes"][0].clone();
    let set_id = structural_id(
        package["name"].as_str().expect("package name"),
        vec![json!("off-cycle"), json!("set"), range_id.clone()],
        31,
    );
    set["node_id"] = set_id.clone();
    set["node_tag"] = json!("composite_type");
    set["semantic_form"] = json!("set");
    set["semantic_type"] = set_id;
    set["dependencies"] = json!([range_id]);
    set["recursion_group"] = json!("Tree");
    set["body"] = json!({"term":"aggregate","members":[
        {"term":"reference","target":range_id}
    ]});
    let position = append_authored(&mut package, set);
    settle(&mut package);
    (package, position)
}

#[trace("TC-226", "FR-038-AC-147", "FR-038-AC-150")]
#[test]
fn off_cycle_set_member_participates_in_tree_group_checks() {
    let (package, position) = off_cycle_tree_group();
    assert!(
        matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
        "off-cycle member with Tree label: {:?}",
        read(&package)
    );
    let mut malformed = package.clone();
    malformed["semantic_graph"]["nodes"][position]["body"]["members"] = json!([]);
    assert_derived_refusal(&malformed, position);
    let mut missing = package;
    missing["semantic_graph"]["nodes"][position]["body"]["members"][0]["target"]["ordinal"] =
        json!(99);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&missing) else {
        panic!("off-cycle unresolved reference must refuse");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
}

#[trace("TC-226", "FR-038-AC-149")]
#[test]
fn recursive_group_reader_has_an_exact_work_boundary() {
    for (name, package) in [
        ("List", list_group().0),
        ("Tree", tree_group().0),
        ("Tree with off-cycle Set", off_cycle_tree_group().0),
    ] {
        let mut low = 0_u64;
        let mut high = CheckedPackageReadLimits::bounded().work;
        assert!(matches!(
            read_with_work(&package, high),
            CheckedPackageV2ReadResult::Admitted(_)
        ));
        while low + 1 < high {
            let mid = low + (high - low) / 2;
            match read_with_work(&package, mid) {
                CheckedPackageV2ReadResult::Admitted(_) => high = mid,
                CheckedPackageV2ReadResult::Incomplete(incomplete)
                    if incomplete.limit_kind == CheckedPackageLimit::Work =>
                {
                    low = mid;
                }
                other => panic!("{name}: unexpected work result at {mid}: {other:?}"),
            }
        }
        assert!(high > 0, "{name}: group visits consume work");
        assert!(matches!(
            read_with_work(&package, high),
            CheckedPackageV2ReadResult::Admitted(_)
        ));
        let CheckedPackageV2ReadResult::Incomplete(incomplete) = read_with_work(&package, high - 1)
        else {
            panic!("{name}: one less work must be incomplete");
        };
        assert_eq!(incomplete.limit_kind, CheckedPackageLimit::Work);
        assert_eq!(incomplete.limit, high - 1);
        assert_eq!(incomplete.consumed, high);
        assert!(incomplete.path.is_some(), "{name}: failing visit position");
    }
}
