// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Actual adverse inputs for the whole-public-read allocation oracle.
//! All setup, including admission of host-supplied dependencies, happens
//! before the caller arms its allocation counter.

use super::checked_package::{
    canonical, domain_package_document, evidence_for, fixture_source, index_fixture_node_ids,
    node_id, rebuild_source_map, select_model, v2_all_families, v2_reference_chain,
};
use quire_contract_model::{
    CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageV2, CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};

pub struct Input {
    pub bytes: Vec<u8>,
    pub evidence: CheckedPackageEvidence,
}

fn input(mut value: Value) -> Input {
    let evidence = evidence_for(&value);
    index_fixture_node_ids(&mut value);
    Input {
        bytes: canonical(&value),
        evidence,
    }
}

pub fn chain_100000() -> Input {
    let (bytes, _) = v2_reference_chain(100_000);
    Input {
        bytes,
        evidence: evidence_for(&v2_all_families()),
    }
}

fn flat_value() -> Value {
    let mut value = v2_all_families();
    let template = value["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == "expression" && node["semantic_form"] == "reference")
        .expect("real reference fixture")
        .clone();
    let nodes = value["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    for i in 0..10_000 {
        let mut node = template.clone();
        node["node_id"] = node_id(&format!("allocation-flat/{i:05}"));
        nodes.push(node);
    }
    rebuild_source_map(&mut value);
    assert!(value["source_map"].as_array().expect("source map").len() >= 10_000);
    value
}

/// Covers both the 10000-node flat graph and 10000 concrete source-map rows.
pub fn flat_graph_and_source_map_10000() -> Input {
    input(flat_value())
}

pub fn dependencies_10000() -> Input {
    let mut value = flat_value();
    let nodes = value["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    let values = nodes
        .iter()
        .filter(|node| {
            node["node_id"]["path"]
                .to_string()
                .contains("allocation-flat")
        })
        .map(|node| node["node_id"].clone())
        .collect::<Vec<_>>();
    assert_eq!(values.len(), 10_000);
    let template = nodes
        .iter()
        .find(|node| node["node_tag"] == "expression" && node["semantic_form"] == "reference")
        .expect("reference")
        .clone();
    let boolean = template["semantic_type"].clone();
    let tuple = node_id("allocation-tuple/type");
    let mut ty = template.clone();
    ty["node_id"] = tuple.clone();
    ty["node_tag"] = json!("composite_type");
    ty["semantic_form"] = json!("tuple");
    ty["semantic_type"] = tuple.clone();
    ty["dependencies"] = json!([boolean]);
    ty["occurrences"] = json!([{"role":"type","ordinal":0}]);
    ty["body"] = json!({"term":"aggregate","members":(0..10_000).map(|_|json!({"term":"reference","target":boolean})).collect::<Vec<_>>()});
    nodes.push(ty);
    let mut row = template;
    row["node_id"] = node_id("allocation-tuple/value");
    row["node_tag"] = json!("value");
    row["semantic_form"] = json!("tuple_value");
    row["semantic_type"] = tuple;
    row["dependencies"] = json!(values);
    row["body"] = json!({"term":"aggregate","members":values.iter().map(|id|json!({"term":"reference","target":id})).collect::<Vec<_>>()});
    assert_eq!(
        row["dependencies"].as_array().expect("dependencies").len(),
        10_000
    );
    nodes.push(row);
    rebuild_source_map(&mut value);
    input(value)
}

pub fn string_one_mebibyte() -> Input {
    fn replace(value: &mut Value, old: &str, new: &str) {
        match value {
            Value::String(s) if s == old => *s = new.to_owned(),
            Value::Array(rows) => {
                for row in rows {
                    replace(row, old, new)
                }
            }
            Value::Object(fields) => {
                for child in fields.values_mut() {
                    replace(child, old, new)
                }
            }
            _ => {}
        }
    }
    let mut value = v2_all_families();
    let old = value["lock"]["sources"][0]["identity"]
        .as_str()
        .expect("source identity")
        .to_owned();
    let large = "s".repeat(1 << 20);
    replace(&mut value, &old, &large);
    assert_eq!(
        value["lock"]["sources"][0]["identity"]
            .as_str()
            .expect("single string")
            .len(),
        1 << 20
    );
    input(value)
}

pub fn diagnostics_10000() -> Input {
    let mut value = v2_all_families();
    let target = value["semantic_graph"]["nodes"][0]["node_id"].clone();
    let source = fixture_source();
    value["diagnostics"]["entries"]=json!((0..10_000).map(|i|json!({"stage":"package_read","code":"invalid_package","cause_tag":"invalid-value",
        "details":[{"term":"reference","target":target}],"loci":[{"source":source,"start":i,"end":i+1}]})).collect::<Vec<_>>());
    input(value)
}

fn empty_package(name: &str) -> Value {
    let mut value = v2_all_families();
    value["name"] = json!(name);
    value["semantic_graph"]["nodes"] = json!([]);
    value["semantic_graph"]["operand_rows"] = json!([]);
    value["source_map"] = json!([]);
    value["capability_report"] = json!([]);
    value
}

pub fn direct_dependencies_64() -> Input {
    let mut value = empty_package("allocation-root");
    let mut evidence = evidence_for(&value);
    let mut selections = Vec::new();
    for i in 0..64 {
        let identity = format!("allocation-dependency-{i:02}");
        let dependency = empty_package(&identity);
        let bytes = canonical(&dependency);
        let CheckedPackageV2ReadResult::Admitted(package) = CheckedPackageV2::read(
            &bytes,
            CheckedPackageReadLimits::bounded(),
            &evidence_for(&dependency),
        ) else {
            panic!("dependency fixture {identity} must admit");
        };
        evidence
            .insert_dependency_package(identity.clone(), package)
            .expect("unique dependency evidence");
        selections.push(json!({"identity":identity}));
    }
    value["lock"]["dependency_selections"] = json!(selections);
    assert_eq!(
        value["lock"]["dependency_selections"]
            .as_array()
            .expect("selections")
            .len(),
        64
    );
    Input {
        bytes: canonical(&value),
        evidence,
    }
}

pub fn selected_models_two_512k() -> Input {
    let mut value = empty_package("allocation-model-root");
    let mut evidence = evidence_for(&value);
    for identity in ["allocation/model-a", "allocation/model-b"] {
        select_model(&mut value, identity);
        let ty = json!({"identity":format!("ix://{identity}/Large"),"displayName":"",
            "kind":{"module":identity,"name":"entity"},"roles":[],"constraints":[],"extensions":[],"unknownPolicy":"reject","supertypes":[],"fields":[],"operations":[]});
        let mut document = domain_package_document(identity, "1", vec![ty]);
        document["constructs"] = json!([{"kind":{"module":identity,"name":"entity"},"construct":{"meaning":"quire.meaning.model.object-type/v1"}}]);
        let padding = (512 << 10) - canonical(&document).len();
        document["types"][0]["displayName"] = json!("m".repeat(padding));
        let bytes = canonical(&document);
        assert_eq!(bytes.len(), 512 << 10);
        evidence
            .insert_domain_package_document(identity, bytes)
            .expect("distinct models");
    }
    index_fixture_node_ids(&mut value);
    Input {
        bytes: canonical(&value),
        evidence,
    }
}
