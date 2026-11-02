// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The published QSpec reaches-population descriptors through the IR reader.

use super::*;
use quire_contract_model::selected_population::{
    select_population, CheckedSelectedPopulationOutcome as PopulationOutcome,
    CheckedSelectedPopulationSource as PopulationSource,
};
use quire_contract_model::CheckedNodeId;

fn node_index(package: &Value, tag: &str, form: &str) -> usize {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_tag"] == tag && node["semantic_form"] == form)
        .unwrap_or_else(|| panic!("missing {tag}/{form} node"))
}

fn append_node(package: &mut Value, node: Value) -> Value {
    let id = node["node_id"].clone();
    let mut source = package["source_map"][0].clone();
    source["node_id"] = id.clone();
    source["role"] = node["occurrences"][0]["role"].clone();
    source["ordinal"] = node["occurrences"][0]["ordinal"].clone();
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(node);
    package["source_map"]
        .as_array_mut()
        .expect("source map")
        .push(source);
    id
}

fn append_rekeyed(package: &mut Value, mut node: Value) -> Value {
    let self_typed = node["semantic_type"] == node["node_id"];
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    let id = fixture_support::structural_id(
        package["name"].as_str().expect("package name"),
        vec![
            node["node_tag"].clone(),
            node["semantic_form"].clone(),
            json!(format!("population-case-{position}")),
        ],
        0,
    );
    node["node_id"] = id.clone();
    if self_typed {
        node["semantic_type"] = id.clone();
    }
    append_node(package, node)
}

fn append_population(package: &mut Value, object: &Value, max: Option<&str>, extra: bool) -> Value {
    let integer = package["semantic_graph"]["nodes"][node_index(package, "scalar_type", "integer")]
        ["node_id"]
        .clone();
    let mut reference = package["semantic_graph"]["nodes"]
        [node_index(package, "composite_type", "reference")]
    .clone();
    reference["body"]["members"][0]["target"] = object.clone();
    reference["dependencies"] = json!([object]);
    let reference = append_rekeyed(package, reference);

    let mut set =
        package["semantic_graph"]["nodes"][node_index(package, "composite_type", "set")].clone();
    set["body"]["members"][0]["target"] = reference.clone();
    set["dependencies"] = json!([reference]);
    let set = append_rekeyed(package, set);
    let set_index = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;

    let mut population = package["semantic_graph"]["nodes"]
        [node_index(package, "bounded_domain", "integer_range")]
    .clone();
    population["semantic_form"] = json!("model_population");
    population["semantic_type"] = set.clone();
    population["body"] = match max {
        Some(value) => json!({"term":"aggregate", "members":[{
            "name":"max", "term":"binding", "value":{
                "term":"literal", "type":integer, "value_kind":"integer", "value":value
            }
        }]}),
        None => json!({"term":"aggregate", "members":[]}),
    };
    if extra {
        population["body"]["members"]
            .as_array_mut()
            .expect("population members")
            .push(json!({"name":"other", "term":"binding", "value":{
                "term":"literal", "type":integer, "value_kind":"integer", "value":"1"
            }}));
    }
    let mut dependencies = vec![set.clone()];
    if max.is_some() || extra {
        dependencies.push(integer);
    }
    super::node_index::sort_fixture_node_refs(package, &mut dependencies);
    population["dependencies"] = json!(dependencies);
    let mut path = vec![
        json!("bounded_domain"),
        json!("model_population"),
        json!({"node":set_index}),
    ];
    if let Some(max) = max {
        path.push(json!(max));
    }
    let id =
        fixture_support::structural_id(package["name"].as_str().expect("package name"), path, 0);
    population["node_id"] = id.clone();
    append_node(package, population)
}

fn candidate(base: &Value, case: &Value) -> Value {
    let mut package = base.clone();
    let application = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["body"]["operation"]["identity"] == "quire.op.model.reaches")
        .expect("reaches application");
    let descriptor = &case["population"];
    let link = if descriptor.is_null() {
        None
    } else if descriptor["absent"] == true {
        Some(fixture_support::node_id("missing-population"))
    } else if let Some(owner) = descriptor["model_population_over"].as_str() {
        let order_node = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .find(|node| {
                node["node_tag"] == "model" && node["owner"]["node"] == "ix://acme/orders/Order"
            })
            .expect("Order declaration")
            .clone();
        let order = order_node["node_id"].clone();
        let object = if owner == "ix://acme/orders/Order" {
            order
        } else {
            let mut other = order_node;
            other["owner"]["node"] = json!(owner);
            append_rekeyed(&mut package, other)
        };
        Some(append_population(
            &mut package,
            &object,
            descriptor["max"].as_str(),
            descriptor["extra_member"] == true,
        ))
    } else {
        Some(
            package["semantic_graph"]["nodes"][node_index(
                &package,
                descriptor["node_tag"].as_str().expect("tag"),
                descriptor["semantic_form"].as_str().expect("form"),
            )]["node_id"]
                .clone(),
        )
    };
    if let Some(link) = link {
        let ids = super::node_index::fixture_graph_ids(&package);
        let node = &mut package["semantic_graph"]["nodes"][application];
        node["body"]["operation"]["member"]["population"] = link.clone();
        if case["dependencies"] != "without-population" {
            let mut dependencies = node["dependencies"].as_array().expect("deps").clone();
            dependencies.push(link);
            super::node_index::sort_fixture_node_refs_with_ids(&ids, &mut dependencies);
            dependencies.dedup();
            node["dependencies"] = json!(dependencies);
        }
    }
    // The published source-map regions are authoritative. `settle` rebuilds
    // every row with synthetic position regions and loses the declaration
    // correspondence of the QSpec base (including Example.Counter). The
    // application dependency was updated above; normalize only changed
    // owner paths and their exact references.
    fixture_support::refresh_identity(&mut package);
    package
}

#[trace(
    "TC-048",
    "FR-038-AC-269",
    "FR-038-AC-272",
    "FR-038-AC-273",
    "FR-038-AC-274",
    "FR-038-AC-276"
)]
#[test]
fn tc_048_qspec_reaches_population_cases_match_the_reader() {
    let dir = fixtures_dir();
    let adverse = read_json(&dir.join(ADVERSE)).expect("QSpec adverse fixture");
    let rows = mutations(&adverse, "reaches_population_cases").expect("published cases");
    assert_eq!(rows.len(), 7);
    let mut seen = std::collections::BTreeSet::new();
    for row in rows {
        let id = text_of(row, "id").expect("case id");
        assert!(seen.insert(id), "duplicate case {id}");
        let base = text_of(row, "base").expect("base");
        let package = read_json(&dir.join(format!("{base}.json"))).expect("published base");
        let package = candidate(&package, row);
        let got = reading_of(&package).unwrap_or_else(|why| panic!("{id}: {why}"));
        let expected = if row["outcome"] == "admitted" {
            "admitted".to_owned()
        } else if let Some(cause) = row["expected_cause"].as_str() {
            format!(
                "refused:{}/{cause}",
                text_of(row, "expected_code").expect("code")
            )
        } else {
            format!("refused:{}", text_of(row, "expected_code").expect("code"))
        };
        assert_eq!(got, expected, "{id}: {:?}", read_package(&package));
    }
}

/// QSpec FR-322-AC-74's five published rows are read through the production
/// reader with the exact published base source map. The two admitted extents
/// retain distinct structural population identities.
#[trace("TC-233", "FR-322-AC-74")]
#[test]
fn tc_233_qspec_unbounded_population_rows_read_with_exact_sources() {
    let fixtures = fixtures_dir();
    let adverse = read_json(&fixtures.join(ADVERSE)).expect("QSpec adverse fixture");
    let rows = adverse["unbounded_population_cases"]
        .as_array()
        .expect("published unbounded population rows");
    assert_eq!(rows.len(), 5);
    let mut seen = std::collections::BTreeSet::new();
    let mut positive_ids = Vec::new();
    for row in rows {
        let id = row["id"].as_str().expect("published row id");
        assert!(seen.insert(id), "duplicate row {id}");
        let base = row["base"].as_str().expect("published base");
        let published =
            read_json(&fixtures.join(format!("{base}.json"))).expect("published positive base");
        let descriptor = json!({
            "model_population_over": "ix://acme/orders/Order",
            "max": row["max"],
            "extra_member": row["extra_member"] == true,
        });
        let package = candidate(&published, &json!({"population": descriptor}));
        assert_eq!(
            &package["source_map"].as_array().expect("source map")[..published["source_map"]
                .as_array()
                .expect("base source map")
                .len()],
            published["source_map"].as_array().expect("base source map"),
            "{id}: QSpec source correspondence must be retained"
        );
        let position = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("graph nodes")
            .iter()
            .rposition(|node| node["semantic_form"] == "model_population")
            .expect("authored population node");
        let population_id = package["semantic_graph"]["nodes"][position]["node_id"].clone();
        match row["outcome"].as_str() {
            Some("admitted") => {
                let result = read_package(&package).expect("production read");
                let CheckedPackageV2ReadResult::Admitted(admitted) = result else {
                    panic!("{id}: expected admission, got {result:?}");
                };
                assert_eq!(
                    serde_json::to_value(&admitted.graph().nodes[position].node_id)
                        .expect("retained node id"),
                    population_id,
                    "{id}: retained structural identity"
                );
                let application = package["semantic_graph"]["nodes"]
                    .as_array()
                    .expect("graph nodes")
                    .iter()
                    .find(|node| node["body"]["operation"]["identity"] == "quire.op.model.reaches")
                    .expect("reaches application");
                let application_id: CheckedNodeId =
                    serde_json::from_value(application["node_id"].clone())
                        .expect("reaches identity");
                let selector = admitted
                    .reaches_population(&application_id)
                    .expect("admitted reaches population")
                    .expect("population selector");
                let expected_population: CheckedNodeId =
                    serde_json::from_value(population_id.clone()).expect("population identity");
                let expected_object: CheckedNodeId = serde_json::from_value(
                    package["semantic_graph"]["nodes"]
                        .as_array()
                        .expect("graph nodes")
                        .iter()
                        .find(|node| {
                            node["node_tag"] == "model"
                                && node["owner"]["node"] == "ix://acme/orders/Order"
                        })
                        .expect("selected Order declaration")["node_id"]
                        .clone(),
                )
                .expect("selected object identity");
                assert_eq!(selector.binding, expected_population, "{id}: selected node");
                let selected = select_population(PopulationSource::Admitted {
                    package: &admitted,
                    selector: selector.clone(),
                });
                match (row["max"].as_str(), selected) {
                    (None, PopulationOutcome::AdmittedUnbounded(unbounded)) => {
                        assert_eq!(unbounded.package_name.as_ref(), admitted.name(), "{id}");
                        assert_eq!(unbounded.binding, expected_population, "{id}");
                        assert_eq!(unbounded.occurrence, selector.occurrence, "{id}");
                        assert_eq!(unbounded.object_type, expected_object, "{id}");
                        assert!(!unbounded.provenance.is_empty(), "{id}: source provenance");
                    }
                    (Some(expected), PopulationOutcome::Finite(finite)) => {
                        assert_eq!(finite.maximum.as_str(), expected, "{id}");
                        assert_eq!(finite.binding, expected_population, "{id}");
                        assert_eq!(finite.occurrence, selector.occurrence, "{id}");
                        assert_eq!(finite.object_type, expected_object, "{id}");
                        assert!(!finite.provenance.is_empty(), "{id}: source provenance");
                    }
                    (_, other) => panic!("{id}: wrong public selection outcome {other:?}"),
                }
                positive_ids.push(population_id);
            }
            None => {
                let result = read_package(&package).expect("production read");
                let CheckedPackageV2ReadResult::Refused(refusal) = result else {
                    panic!("{id}: expected derived-shape refusal, got {result:?}");
                };
                assert_eq!(refusal.code, Code::InvalidPackage, "{id}");
                assert_eq!(refusal.cause, Some(Cause::InvalidDerivedShape), "{id}");
                assert_eq!(
                    refusal.loci[0].path.as_ref().map(ToString::to_string),
                    Some(format!("/semantic_graph/nodes/{position}/node_id")),
                    "{id}: population identity locus"
                );
            }
            Some(other) => panic!("{id}: unknown QSpec outcome {other}"),
        }
    }
    assert_eq!(positive_ids.len(), 2, "unbounded and finite siblings");
    assert_ne!(positive_ids[0], positive_ids[1]);
}

/// A finite maximum is retained as canonical decimal text beyond u64::MAX.
#[trace("TC-453")]
#[test]
fn tc_453_admitted_population_maximum_exceeds_u64_without_truncation() {
    let fixtures = fixtures_dir();
    let base = read_json(&fixtures.join("positive-operation-identities.json"))
        .expect("QSpec positive base");
    let maximum = "18446744073709551616";
    let package = candidate(
        &base,
        &json!({"population": {
            "model_population_over": "ix://acme/orders/Order", "max": maximum
        }}),
    );
    let result = read_package(&package).expect("production read");
    let CheckedPackageV2ReadResult::Admitted(admitted) = result else {
        panic!("large canonical maximum must admit: {result:?}");
    };
    let application = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("graph nodes")
        .iter()
        .find(|node| node["body"]["operation"]["identity"] == "quire.op.model.reaches")
        .expect("reaches application");
    let request: CheckedNodeId =
        serde_json::from_value(application["node_id"].clone()).expect("reaches identity");
    let selector = admitted
        .reaches_population(&request)
        .expect("admitted population")
        .expect("selected population");
    let PopulationOutcome::Finite(finite) = select_population(PopulationSource::Admitted {
        package: &admitted,
        selector,
    }) else {
        panic!("large admitted maximum must stay finite");
    };
    assert_eq!(finite.maximum.as_str(), maximum);
}
