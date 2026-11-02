// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! QSpec's carried-quantity wire controls through the complete V2 reader.

use super::{proposal_dir_of, read_json};
use crate::fixture_support::{
    canonical, evidence_for, index_fixture_node_ids, node_id, nominal_fixture_members,
    nominal_package, settle,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedPackageReadLimits, CheckedPackageV2, CheckedPackageV2ReadResult, ScalarTypeForm,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;

fn vectors() -> Value {
    read_json(&proposal_dir_of(&super::fixtures_dir()).join("quantity-carrier-vectors.json"))
        .expect("QSpec quantity carrier vectors")
}

fn expand(value: &Value, ids: &BTreeMap<String, Value>) -> Value {
    match value {
        Value::Object(object) if object.len() == 1 && object.contains_key("node") => {
            ids[object["node"].as_str().expect("vector node name")].clone()
        }
        Value::Array(array) => Value::Array(array.iter().map(|item| expand(item, ids)).collect()),
        Value::Object(object) => Value::Object(
            object
                .iter()
                .map(|(name, value)| (name.clone(), expand(value, ids)))
                .collect(),
        ),
        other => other.clone(),
    }
}

fn referenced_names(value: &Value, names: &mut Vec<String>) {
    match value {
        Value::Object(object) if object.len() == 1 && object.contains_key("node") => {
            names.push(
                object["node"]
                    .as_str()
                    .expect("vector node name")
                    .to_owned(),
            );
        }
        Value::Array(array) => {
            for item in array {
                referenced_names(item, names);
            }
        }
        Value::Object(object) => {
            for value in object.values() {
                referenced_names(value, names);
            }
        }
        _ => {}
    }
}

struct Package {
    wire: Value,
    ids: BTreeMap<String, Value>,
}

impl Package {
    fn new() -> Self {
        let mut members = nominal_fixture_members();
        let dimension = members
            .iter()
            .find(|(value, _)| value["version"] == "quire.dimension-node/v1")
            .expect("length dimension");
        let length = node_id(&dimension.1);
        let mut time = dimension.0.clone();
        time["qualified_declaration"] = json!(["Example", "Time"]);
        members.push((time, "qualified:Example/Time".into()));
        let unit = members
            .iter()
            .find(|(value, _)| value["version"] == "quire.unit-node/v1")
            .expect("length unit");
        let metre = node_id(&unit.1);
        let unit = unit.0.clone();
        for (name, key, numerator, denominator) in [
            ("Centimetre", "qualified:Example/Centimetre", "1", "100"),
            ("Inch", "qualified:Example/Inch", "127", "5000"),
        ] {
            let mut scaled = unit.clone();
            scaled["qualified_declaration"] = json!(["Example", name]);
            scaled["target_unit_node_id"] = metre.clone();
            scaled["scale"] = json!({"numerator":numerator,"denominator":denominator});
            members.push((scaled, key.into()));
        }
        let mut second = unit;
        second["qualified_declaration"] = json!(["Example", "Second"]);
        second["dimension_node_id"] = node_id("qualified:Example/Time");
        members.push((second, "qualified:Example/Second".into()));
        let wire = nominal_package(&members);
        let ids = BTreeMap::from([
            ("dim_L".into(), length),
            ("dim_T".into(), node_id("qualified:Example/Time")),
            ("unit_cm".into(), node_id("qualified:Example/Centimetre")),
            ("unit_in".into(), node_id("qualified:Example/Inch")),
            ("unit_m".into(), metre),
            ("unit_s".into(), node_id("qualified:Example/Second")),
        ]);
        Self { wire, ids }
    }

    fn add(&mut self, name: &str, all: &Value) {
        if self.ids.contains_key(name) {
            return;
        }
        let row = &all[name];
        assert!(row.is_object(), "published node {name}");
        let mut names = Vec::new();
        referenced_names(&row["path"], &mut names);
        referenced_names(&row["body"], &mut names);
        if let Some(base) = row["base"].as_str() {
            names.push(base.to_owned());
        }
        for dependency in names {
            self.add(&dependency, all);
        }
        let id = json!({
            "package": self.wire["name"],
            "path": expand(&row["path"], &self.ids),
            "ordinal": 0,
        });
        let tag = row["tag"].as_str().expect("node tag");
        let form = row["form"].as_str().expect("node form");
        let semantic_type = match (tag, form) {
            ("bounded_domain", "integer_range") => self.ids["integer"].clone(),
            ("bounded_domain", "decimal_range") => self.ids["decimal"].clone(),
            _ => id.clone(),
        };
        let mut dependencies = Vec::new();
        if form == "carried_quantity" {
            dependencies.push(self.ids[row["unit"].as_str().expect("unit")].clone());
            dependencies.push(self.ids[row["carrier"].as_str().expect("carrier")].clone());
        }
        self.wire["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(json!({
                "node_id": id,
                "schema_version": "quire.checked-semantic-graph/v2",
                "node_tag": tag,
                "semantic_form": form,
                "semantic_type": semantic_type,
                "dependencies": dependencies,
                "occurrences": [{"role":"expression","ordinal":0}],
                "body": expand(&row["body"], &self.ids),
            }));
        self.ids.insert(name.to_owned(), id);
    }

    fn application(&mut self, case: &Value, all: &Value) -> usize {
        let application = &case["application"];
        let source_name = application["operand_types"][0]
            .as_str()
            .expect("source type");
        let target_name = application["result_type"].as_str().expect("result type");
        self.add(source_name, all);
        self.add(target_name, all);
        self.add("text", all);
        self.add("integer", all);
        let source = self.ids[source_name].clone();
        let target = self.ids[target_name].clone();
        let parameter = node_id("quantity/vector-source");
        let body = json!({"term":"aggregate", "members":[
            {"term":"binding","name":"name","value":
                {"term":"literal","type":self.ids["text"],"value_kind":"text","value":"source"}},
            {"term":"binding","name":"level","value":
                {"term":"literal","type":self.ids["integer"],"value_kind":"integer","value":"0"}}
        ]});
        self.wire["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(json!({
                "node_id": parameter, "schema_version":"quire.checked-semantic-graph/v2",
                "node_tag":"value", "semantic_form":"parameter", "semantic_type":source,
                "dependencies":[], "occurrences":[{"role":"expression","ordinal":0}],
                "body":body,
            }));
        let at = self.wire["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .len();
        let identity = application["operation"].as_str().expect("operation");
        let member = if identity == "quire.op.quantity.convert" {
            json!({"kind":"type_argument","declaration":target})
        } else {
            Value::Null
        };
        let operator = if identity == "quire.op.quantity.convert" {
            "convert"
        } else {
            "binary"
        };
        let form = if identity == "quire.op.quantity.convert" {
            "conversion"
        } else {
            "binary"
        };
        self.wire["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(json!({
                "node_id":node_id(&format!("quantity/vector-application/{at}")),
                "schema_version":"quire.checked-semantic-graph/v2",
                "node_tag":"expression", "semantic_form":form,
                "semantic_type":target, "dependencies":[],
                "occurrences":[{"role":"expression","ordinal":0}],
                "body":{"term":"application","operator":operator,
                    "operation":{"identity":identity,"laws":[],"mode":application["mode"],
                        "member":member,"leaves":[]},
                    "result_type":target,
                    "arguments": if identity == "quire.op.quantity.convert" {
                        vec![json!({"term":"reference","target":parameter})]
                    } else {
                        vec![json!({"term":"reference","target":parameter}),
                             json!({"term":"reference","target":parameter})]
                    },
                },
            }));
        at
    }

    fn read(mut self) -> CheckedPackageV2ReadResult {
        settle(&mut self.wire);
        index_fixture_node_ids(&mut self.wire);
        let source = self.wire["lock"]["sources"][0].clone();
        let entries = self.wire["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .enumerate()
            .flat_map(|(position, node)| {
                node["occurrences"]
                    .as_array()
                    .expect("occurrences")
                    .iter()
                    .map(|occurrence| {
                        json!({
                            "node_id":node["node_id"],
                            "role":occurrence["role"],
                            "ordinal":occurrence["ordinal"],
                            "regions":[{"source":source,"start":position,"end":position+1}],
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        self.wire["source_map"] = json!(entries);
        CheckedPackageV2::read(
            &canonical(&self.wire),
            CheckedPackageReadLimits::bounded(),
            &evidence_for(&self.wire),
        )
    }
}

/// Trace: FR-038-AC-401 through FR-038-AC-405; FR-019-AC-20.
#[trace(
    "TC-048",
    "TC-058",
    "FR-038-AC-401",
    "FR-038-AC-402",
    "FR-038-AC-403",
    "FR-038-AC-404",
    "FR-038-AC-405",
    "FR-019-AC-20"
)]
#[test]
fn published_carried_quantity_vectors_read_as_wire() {
    let vectors = vectors();
    let all = &vectors["nodes"];
    for case in vectors["wire"].as_array().expect("wire vectors") {
        let id = case["id"].as_str().expect("vector id");
        let mut package = Package::new();
        let at = if let Some(name) = case["node"].as_str() {
            package.add(name, all);
            package.wire["semantic_graph"]["nodes"]
                .as_array()
                .expect("nodes")
                .len()
                - 1
        } else if case["expected"] == "distinct" {
            for name in case["nodes"].as_array().expect("distinct nodes") {
                package.add(name.as_str().expect("node name"), all);
            }
            0
        } else {
            package.application(case, all)
        };
        let result = package.read();
        if case["expected"] == "admitted" || case["expected"] == "distinct" {
            let CheckedPackageV2ReadResult::Admitted(admitted) = result else {
                panic!("{id}: {result:?}");
            };
            if case["expected"] == "distinct" {
                let nodes = &admitted.graph().nodes;
                let carried = nodes
                    .iter()
                    .filter(|node| node.semantic_form.as_ref() == "carried_quantity")
                    .collect::<Vec<_>>();
                assert_eq!(carried.len(), 2, "{id}");
                assert_ne!(carried[0].node_id, carried[1].node_id, "{id}");
            } else if case["node"].is_string() {
                let carried = admitted
                    .graph()
                    .nodes
                    .iter()
                    .zip(admitted.node_kinds())
                    .find(|(_, kind)| {
                        **kind
                            == quire_contract_model::CheckedNodeKind::ScalarType(
                                ScalarTypeForm::CarriedQuantity,
                            )
                    })
                    .map(|(node, _)| node)
                    .unwrap_or_else(|| panic!("{id}: missing carried type"));
                let body = serde_json::to_value(&carried.body).expect("typed body");
                let names = body["members"]
                    .as_array()
                    .expect("bindings")
                    .iter()
                    .map(|binding| binding["name"].as_str().expect("name"))
                    .collect::<Vec<_>>();
                assert_eq!(names, ["unit", "carrier", "rounding"], "{id}");
                let name = case["node"].as_str().expect("node name");
                assert_eq!(
                    body["members"][2]["value"]["value"], all[name]["rounding"],
                    "{id}"
                );
            }
        } else {
            let CheckedPackageV2ReadResult::Refused(refusal) = result else {
                panic!("{id}: expected refusal, got {result:?}");
            };
            assert_eq!(
                serde_json::to_value(refusal.code).expect("code"),
                case["code"],
                "{id}"
            );
            assert_eq!(
                serde_json::to_value(refusal.cause).expect("cause"),
                case["cause"],
                "{id}"
            );
            let expected = if case["node"].is_string() {
                format!("/semantic_graph/nodes/{at}/node_id")
            } else {
                format!("/semantic_graph/nodes/{at}")
            };
            assert_eq!(
                refusal.loci[0].path.as_ref().map(ToString::to_string),
                Some(expected),
                "{id}"
            );
        }
    }
    for case in vectors["evaluation"]
        .as_array()
        .expect("evaluation controls")
    {
        let target = case["target"].as_str().expect("target");
        let unit = case["source"]["unit"].as_str().expect("source unit");
        let mut package = Package::new();
        let application = json!({
            "application":{"operation":"quire.op.quantity.convert",
                "operand_types":[format!("unit_{unit}")],"result_type":target,
                "mode":{"kind":"rounding","value":case["mode"]}}
        });
        package.application(&application, all);
        let result = package.read();
        assert!(
            matches!(result, CheckedPackageV2ReadResult::Admitted(_)),
            "{}: {result:?}",
            case["id"]
        );
    }
}

/// Trace: FR-038-AC-402.
#[trace("TC-048", "FR-038-AC-402")]
#[test]
fn carried_quantity_binding_and_path_mutations_refuse_at_the_type() {
    let vectors = vectors();
    for mutation in ["missing", "extra", "reordered", "path"] {
        let mut package = Package::new();
        package.add("q_dec", &vectors["nodes"]);
        let at = package.wire["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .len()
            - 1;
        let node = &mut package.wire["semantic_graph"]["nodes"][at];
        match mutation {
            "missing" => {
                node["body"]["members"]
                    .as_array_mut()
                    .expect("members")
                    .pop();
            }
            "extra" => {
                let extra = node["body"]["members"][2].clone();
                node["body"]["members"]
                    .as_array_mut()
                    .expect("members")
                    .push(extra);
            }
            "reordered" => {
                node["body"]["members"]
                    .as_array_mut()
                    .expect("members")
                    .swap(0, 1);
            }
            "path" => {
                node["node_id"]["path"][6] = json!("toward-zero");
                node["semantic_type"] = node["node_id"].clone();
            }
            _ => unreachable!("test cases are closed"),
        }
        let result = package.read();
        let CheckedPackageV2ReadResult::Refused(refusal) = result else {
            panic!("{mutation}: expected refusal, got {result:?}");
        };
        assert_eq!(
            serde_json::to_value(refusal.code).expect("code"),
            "invalid_package",
            "{mutation}"
        );
        assert_eq!(
            serde_json::to_value(refusal.cause).expect("cause"),
            "invalid-derived-shape",
            "{mutation}"
        );
        assert_eq!(
            refusal.loci[0].path.as_ref().map(ToString::to_string),
            Some(format!("/semantic_graph/nodes/{at}/node_id")),
            "{mutation}"
        );
    }
}

/// Trace: FR-038-AC-403.
#[trace("TC-048", "FR-038-AC-403")]
#[test]
fn conversion_into_a_carried_quantity_requires_a_unit_or_carried_source() {
    let vectors = vectors();
    let mut package = Package::new();
    let compound = json!({
        "package":package.wire["name"],
        "path":["scalar_type","compound_unit"],
        "ordinal":0,
    });
    package.wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(json!({
            "node_id":compound,
            "schema_version":"quire.checked-semantic-graph/v2",
            "node_tag":"scalar_type",
            "semantic_form":"compound_unit",
            "semantic_type":compound,
            "dependencies":[],
            "occurrences":[{"role":"expression","ordinal":0}],
            "body":{"term":"aggregate","members":[]},
        }));
    package.ids.insert("compound_unit".into(), compound);
    let at = package.application(
        &json!({"application":{
            "operation":"quire.op.quantity.convert",
            "operand_types":["compound_unit"],
            "result_type":"q_dec",
            "mode":{"kind":"rounding","value":"exact"},
        }}),
        &vectors["nodes"],
    );
    let result = package.read();
    let CheckedPackageV2ReadResult::Refused(refusal) = result else {
        panic!("expected ineligible source refusal, got {result:?}");
    };
    assert_eq!(
        serde_json::to_value(refusal.code).expect("code"),
        "ill_typed"
    );
    assert_eq!(
        serde_json::to_value(refusal.cause).expect("cause"),
        "operator-ineligible"
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some(format!("/semantic_graph/nodes/{at}"))
    );
}
