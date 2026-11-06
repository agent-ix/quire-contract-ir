// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! QSpec's carried-quantity wire controls through the complete V2 reader.

use super::{proposal_dir_of, read_json};
use crate::fixture_support::{
    canonical, evidence_for, index_fixture_node_ids, node_id, nominal_fixture_members,
    nominal_package, resolved_fixture_id, settle,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    BoundedDomainForm, CheckedNodeId, CheckedNodeKind, CheckedNodeTag, CheckedPackageReadLimits,
    CheckedPackageV2, CheckedPackageV2ReadResult, CheckedTerm, CompleteLoweringProfileV2,
    CompleteLoweringRecordV2, LiteralKind, ScalarTypeForm,
};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

fn vectors() -> Value {
    read_json(&proposal_dir_of(&super::fixtures_dir()).join("quantity-carrier-vectors.json"))
        .expect("QSpec quantity carrier vectors")
}

fn bounded_vectors() -> Value {
    read_json(&proposal_dir_of(&super::fixtures_dir()).join("bounded-quantity-vectors.json"))
        .expect("QSpec bounded quantity vectors")
}

fn bound_text(bound: &Value) -> String {
    match bound["kind"].as_str().expect("published bound kind") {
        "integer" => bound["decimal"].as_str().expect("integer bound").to_owned(),
        "rational" => format!(
            "{}/{}",
            bound["numerator"].as_str().expect("numerator"),
            bound["denominator"].as_str().expect("denominator")
        ),
        other => panic!("unsupported published bound kind {other}"),
    }
}

fn assert_published_ids(cases: &[Value], prefix: &str, required: &[&str]) {
    let observed = cases
        .iter()
        .map(|case| case["id"].as_str().expect("published vector ID").to_owned())
        .collect::<BTreeSet<_>>();
    assert_eq!(observed.len(), cases.len(), "duplicate {prefix} vector ID");
    for id in required {
        assert!(
            observed.contains(*id),
            "missing required {prefix} vector {id}"
        );
    }
}

fn assert_carried_bindings(
    admitted: &CheckedPackageV2,
    name: &str,
    all: &Value,
    ids: &BTreeMap<String, Value>,
) {
    let nodes = admitted
        .graph()
        .nodes
        .iter()
        .map(|node| json!({ "node_id": node.node_id }))
        .collect::<Vec<_>>();
    let expected_type = resolved_fixture_id(&ids[name], &nodes);
    let (carried, kind) = admitted
        .graph()
        .nodes
        .iter()
        .zip(admitted.node_kinds())
        .find(|(node, _)| {
            let id = serde_json::to_value(&node.node_id).expect("admitted node ID");
            resolved_fixture_id(&id, &nodes) == expected_type
        })
        .unwrap_or_else(|| panic!("{name}: missing admitted carried type"));
    assert_eq!(
        *kind,
        quire_contract_model::CheckedNodeKind::ScalarType(ScalarTypeForm::CarriedQuantity),
        "{name}"
    );
    let body = serde_json::to_value(&carried.body).expect("typed carried body");
    let members = body["members"].as_array().expect("carried bindings");
    let names = members
        .iter()
        .map(|binding| binding["name"].as_str().expect("binding name"))
        .collect::<Vec<_>>();
    assert_eq!(names, ["unit", "carrier", "rounding"], "{name}");
    for (position, field) in [(0, "unit"), (1, "carrier")] {
        let expected_name = all[name][field].as_str().expect("vector target name");
        let expected = resolved_fixture_id(&ids[expected_name], &nodes);
        let actual = resolved_fixture_id(&members[position]["value"]["target"], &nodes);
        assert_eq!(actual, expected, "{name} {field} binding");
    }
    assert_eq!(
        members[2]["value"]["value"], all[name]["rounding"],
        "{name}"
    );
}

fn assert_rational_carrier_bounds(
    admitted: &CheckedPackageV2,
    all: &Value,
    ids: &BTreeMap<String, Value>,
) {
    let nodes = admitted
        .graph()
        .nodes
        .iter()
        .map(|node| json!({ "node_id": node.node_id }))
        .collect::<Vec<_>>();
    let expected_id = resolved_fixture_id(&ids["rat_r"], &nodes);
    let (carrier, kind) = admitted
        .graph()
        .nodes
        .iter()
        .zip(admitted.node_kinds())
        .find(|(node, _)| {
            let id = serde_json::to_value(&node.node_id).expect("admitted node ID");
            resolved_fixture_id(&id, &nodes) == expected_id
        })
        .expect("admitted rational carrier");
    assert_eq!(
        *kind,
        quire_contract_model::CheckedNodeKind::BoundedDomain(BoundedDomainForm::RationalRange)
    );
    let CheckedTerm::Aggregate { members } = &carrier.body else {
        panic!("rational carrier body is not a typed aggregate");
    };
    let names = [
        "numerator_min",
        "numerator_max",
        "denominator_min",
        "denominator_max",
    ];
    assert_eq!(members.len(), names.len());
    for (member, name) in members.iter().zip(names) {
        let CheckedTerm::Binding {
            name: actual,
            value,
        } = member
        else {
            panic!("rational carrier member is not a typed binding");
        };
        assert_eq!(actual.as_ref(), name);
        let CheckedTerm::Literal {
            value_kind, value, ..
        } = value.as_ref()
        else {
            panic!("{name} is not a typed literal");
        };
        assert_eq!(*value_kind, LiteralKind::Integer, "{name}");
        let expected = all["rat_r"][name]
            .as_i64()
            .expect("published rational bound")
            .to_string();
        assert_eq!(value.as_str(), Some(expected.as_str()), "{name}");
    }
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
        let tag = row["tag"].as_str().expect("node tag");
        let form = row["form"].as_str().expect("node form");
        if form == "rational_range" {
            self.add("rational", all);
        }
        let id = json!({
            "package": self.wire["name"],
            "path": expand(&row["path"], &self.ids),
            "ordinal": 0,
        });
        let semantic_type = match (tag, form) {
            ("bounded_domain", "integer_range") => self.ids["integer"].clone(),
            ("bounded_domain", "decimal_range") => self.ids["decimal"].clone(),
            ("bounded_domain", "rational_range") => self.ids["rational"].clone(),
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

    fn record(&mut self, name: &str, fields: &[&str]) {
        let id = node_id(&format!("quantity/bounded-record/{name}"));
        let types = fields
            .iter()
            .map(|field| self.ids[*field].clone())
            .collect::<Vec<_>>();
        let members = types
            .iter()
            .enumerate()
            .map(|(index, ty)| {
                json!({"term":"binding","name":format!("field{index}"),
                    "value":{"term":"reference","target":ty}})
            })
            .collect::<Vec<_>>();
        self.wire["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(json!({
                "node_id":id,"schema_version":"quire.checked-semantic-graph/v2",
                "node_tag":"composite_type","semantic_form":"record",
                "semantic_type":id,"dependencies":types,
                "occurrences":[{"role":"type","ordinal":0}],
                "body":{"term":"aggregate","members":members},
            }));
        self.ids.insert(name.to_owned(), id);
    }

    fn parameter(&mut self, name: &str, ty: &str) {
        let id = node_id(&format!("quantity/bounded-parameter/{name}"));
        let body = json!({"term":"aggregate","members":[
            {"term":"binding","name":"name","value":{"term":"literal",
                "type":self.ids["text"],"value_kind":"text","value":name}},
            {"term":"binding","name":"level","value":{"term":"literal",
                "type":self.ids["integer"],"value_kind":"integer","value":"0"}},
        ]});
        self.wire["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(json!({
                "node_id":id,"schema_version":"quire.checked-semantic-graph/v2",
                "node_tag":"value","semantic_form":"parameter",
                "semantic_type":self.ids[ty],"dependencies":[],
                "occurrences":[{"role":"expression","ordinal":0}],"body":body,
            }));
        self.ids.insert(name.to_owned(), id);
    }

    fn compound_unit(&mut self) {
        let id = json!({
            "package": self.wire["name"],
            "path": ["scalar_type", "compound_unit"],
            "ordinal": 0,
        });
        self.wire["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(json!({
                "node_id":id,"schema_version":"quire.checked-semantic-graph/v2",
                "node_tag":"scalar_type","semantic_form":"compound_unit",
                "semantic_type":id,"dependencies":[],
                "occurrences":[{"role":"expression","ordinal":0}],
                "body":{"term":"aggregate","members":[]},
            }));
        self.ids.insert("compound_unit".into(), id);
    }

    fn bounded(&mut self, name: &str, unit: &str, min: &str, max: &str) {
        let rational = self.ids["rational"].clone();
        let unit = self.ids[unit].clone();
        let id = json!({
            "package": self.wire["name"],
            "path": ["scalar_type", "bounded_quantity", unit, "min", rational, min,
                "max", rational, max],
            "ordinal": 0,
        });
        self.wire["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .push(json!({
                "node_id":id,"schema_version":"quire.checked-semantic-graph/v2",
                "node_tag":"scalar_type","semantic_form":"bounded_quantity",
                "semantic_type":id,"dependencies":[unit,rational],
                "occurrences":[{"role":"type","ordinal":0}],
                "body":{"term":"aggregate","members":[
                    {"term":"binding","name":"unit","value":{"term":"reference","target":unit}},
                    {"term":"binding","name":"min","value":{"term":"literal",
                        "type":rational,"value_kind":"rational","value":min}},
                    {"term":"binding","name":"max","value":{"term":"literal",
                        "type":rational,"value_kind":"rational","value":max}}
                ]},
            }));
        self.ids.insert(name.to_owned(), id);
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

/// Trace: FR-038-AC-482, FR-038-AC-483, FR-019-AC-26; QSpec FR-322-AC-384.
#[trace(
    "TC-048",
    "TC-050",
    "TC-058",
    "FR-038-AC-482",
    "FR-038-AC-483",
    "FR-019-AC-26"
)]
#[test]
fn published_bounded_quantity_shape_and_bounds_policy() {
    let published = bounded_vectors();
    let forms = published["forms"]
        .as_array()
        .expect("published bounded forms");
    assert_published_ids(forms, "BQ", &["BQ-01", "BQ-04", "BQ-05", "BQ-06", "BQ-07"]);
    let case = |id| forms.iter().find(|form| form["id"] == id).expect("BQ case");
    assert_eq!(case("BQ-07")["expected"]["cause"], "requires-bound");
    for (id, admitted) in [
        ("BQ-01", true),
        ("BQ-04", true),
        ("BQ-05", false),
        ("BQ-06", false),
    ] {
        let form = case(id);
        let mut package = Package::new();
        let all = &vectors()["nodes"];
        for name in ["rational", "text", "integer"] {
            package.add(name, all);
        }
        let min = bound_text(&form["min"]);
        let max = bound_text(&form["max"]);
        package.bounded("bounded", "unit_m", &min, &max);
        if admitted {
            package.parameter("bounded_parameter", "bounded");
            package.parameter("bare_parameter", "unit_m");
            package.compound_unit();
            package.parameter("compound_parameter", "compound_unit");
            let ids = package.ids.clone();
            let CheckedPackageV2ReadResult::Admitted(model) = package.read() else {
                panic!("{id} must admit");
            };
            let bounded = model
                .graph()
                .nodes
                .iter()
                .zip(model.node_kinds())
                .find(|(_, kind)| {
                    **kind == CheckedNodeKind::ScalarType(ScalarTypeForm::BoundedQuantity)
                })
                .expect("admitted bounded node")
                .0;
            let CheckedTerm::Aggregate { members } = &bounded.body else {
                panic!("{id}: bounded body is not aggregate");
            };
            assert_eq!(members.len(), 3);
            let actual = serde_json::to_value(&bounded.body).expect("typed body");
            assert_eq!(actual["members"][1]["value"]["value"], min, "{id} min");
            assert_eq!(actual["members"][2]["value"]["value"], max, "{id} max");
            let profile = CompleteLoweringProfileV2 {
                reference: quire_contract_model::LoweringProfileRef {
                    authority: "quire".into(),
                    identity: "complete-v1".into(),
                },
                required_source_selections: Vec::new(),
                supported_tags: CheckedNodeTag::ALL.iter().copied().collect(),
                require_bounds: true,
                work_limit: 100_000,
            };
            let graph_ids = model
                .graph()
                .nodes
                .iter()
                .map(|node| json!({"node_id": node.node_id}))
                .collect::<Vec<_>>();
            let request = resolved_fixture_id(&ids["bounded_parameter"], &graph_ids);
            let bare_request = resolved_fixture_id(&ids["bare_parameter"], &graph_ids);
            let compound_request = resolved_fixture_id(&ids["compound_parameter"], &graph_ids);
            let records = model
                .lower(&[request, bare_request, compound_request], &profile)
                .records;
            assert!(
                matches!(records[0], CompleteLoweringRecordV2::Lowered { .. }),
                "{id} bounded parameter: {:?}",
                records[0]
            );
            let CompleteLoweringRecordV2::RequiresBound { unbounded_type, .. } = &records[1] else {
                panic!(
                    "BQ-07 bare unit parameter must require a bound: {:?}",
                    records[1]
                );
            };
            assert_eq!(
                *unbounded_type,
                resolved_fixture_id(&ids["unit_m"], &graph_ids)
            );
            let CompleteLoweringRecordV2::RequiresBound { unbounded_type, .. } = &records[2] else {
                panic!(
                    "BQ-07 compound-unit parameter must require a bound: {:?}",
                    records[2]
                );
            };
            assert_eq!(
                *unbounded_type,
                resolved_fixture_id(&ids["compound_unit"], &graph_ids)
            );
        } else {
            let CheckedPackageV2ReadResult::Refused(refusal) = package.read() else {
                panic!("{id} must refuse");
            };
            assert_eq!(
                refusal.code,
                quire_contract_model::CheckedPackageRefusalCode::InvalidPackage,
                "{id}"
            );
            assert_eq!(
                refusal.cause,
                Some(quire_contract_model::CheckedPackageRefusalCause::InvalidDerivedShape),
                "{id}"
            );
            if id == "BQ-06" {
                assert!(
                    refusal.loci[0]
                        .path
                        .as_ref()
                        .is_some_and(|path| path.to_string().ends_with("/body/members/1/value")),
                    "{id} endpoint pointer: {:?}",
                    refusal.loci[0].path
                );
            }
        }
    }
}

/// Trace: FR-038-AC-482, FR-019-AC-26; QSpec FR-322-AC-384.
#[trace("TC-048", "TC-058", "FR-038-AC-482", "FR-019-AC-26")]
#[test]
fn published_bounded_quantity_unit_and_alias_identity() {
    let published = bounded_vectors();
    let forms = published["forms"].as_array().expect("BQ forms");
    assert_published_ids(forms, "BQ", &["BQ-01", "BQ-02", "BQ-03"]);
    let case = |id| forms.iter().find(|form| form["id"] == id).expect("BQ case");
    let metre = case("BQ-01");
    let centimetre = case("BQ-02");
    let alias = case("BQ-03");
    assert_eq!(metre["unit"], alias["unit"]);
    assert_eq!(metre["min"], alias["min"]);
    assert_eq!(metre["max"], alias["max"]);
    assert_ne!(metre["unit"], centimetre["unit"]);
    let mut package = Package::new();
    let all = &vectors()["nodes"];
    for name in ["rational", "text", "integer"] {
        package.add(name, all);
    }
    package.bounded(
        "metre",
        "unit_m",
        &bound_text(&metre["min"]),
        &bound_text(&metre["max"]),
    );
    package.bounded(
        "centimetre",
        "unit_cm",
        &bound_text(&centimetre["min"]),
        &bound_text(&centimetre["max"]),
    );
    let metre_id = package.ids["metre"].clone();
    package.ids.insert("aliased_metre".into(), metre_id);
    package.parameter("direct", "metre");
    package.parameter("through_alias", "aliased_metre");
    let ids = package.ids.clone();
    let CheckedPackageV2ReadResult::Admitted(model) = package.read() else {
        panic!("BQ-01 through BQ-03 must admit");
    };
    let graph_ids = model
        .graph()
        .nodes
        .iter()
        .map(|node| json!({"node_id":node.node_id}))
        .collect::<Vec<_>>();
    let metre_id = resolved_fixture_id(&ids["metre"], &graph_ids);
    let centimetre_id = resolved_fixture_id(&ids["centimetre"], &graph_ids);
    assert_ne!(metre_id, centimetre_id, "BQ-02 changes structural identity");
    let direct_id = resolved_fixture_id(&ids["direct"], &graph_ids);
    let alias_id = resolved_fixture_id(&ids["through_alias"], &graph_ids);
    let direct = model
        .graph()
        .nodes
        .iter()
        .find(|node| node.node_id == direct_id)
        .expect("direct parameter");
    let through_alias = model
        .graph()
        .nodes
        .iter()
        .find(|node| node.node_id == alias_id)
        .expect("alias parameter");
    let direct_type = serde_json::to_value(&direct.semantic_type).expect("direct type");
    let alias_type = serde_json::to_value(&through_alias.semantic_type).expect("alias type");
    assert_eq!(resolved_fixture_id(&direct_type, &graph_ids), metre_id);
    assert_eq!(
        resolved_fixture_id(&alias_type, &graph_ids),
        metre_id,
        "BQ-03 forwards to one node"
    );
    let bounded_nodes = model
        .graph()
        .nodes
        .iter()
        .zip(model.node_kinds())
        .filter(|(_, kind)| **kind == CheckedNodeKind::ScalarType(ScalarTypeForm::BoundedQuantity))
        .count();
    assert_eq!(bounded_nodes, 2, "BQ-03 adds no alias node");
}

/// Trace: FR-038-AC-483; QSpec FR-322-AC-384.
#[trace("TC-048", "FR-038-AC-483")]
#[test]
fn published_bounded_quantity_add_convert_and_lt_admit() {
    let bounded = bounded_vectors();
    let forms = bounded["forms"].as_array().expect("BQ forms");
    let add_convert = forms
        .iter()
        .find(|form| form["id"] == "BQ-08")
        .expect("published operation case");
    assert_eq!(add_convert["expected"], "admitted");
    let operations = add_convert["operations"].as_array().expect("operations");
    let mut all = vectors()["nodes"].clone();
    all["boolean"] = json!({"tag":"scalar_type","form":"boolean",
        "path":["scalar_type","boolean"],"body":{"term":"aggregate","members":[]}});
    for (identity, source, result, mode) in [
        ("quire.op.quantity.add", "bounded", "unit_m", Value::Null),
        (
            "quire.op.quantity.convert",
            "unit_m",
            "bounded",
            json!({"kind":"rounding","value":"exact"}),
        ),
        ("quire.op.quantity.lt", "bounded", "boolean", Value::Null),
    ] {
        assert!(
            operations.iter().any(|item| item == identity),
            "published {identity}"
        );
        let mut package = Package::new();
        for name in ["rational", "text", "integer"] {
            package.add(name, &all);
        }
        package.bounded("bounded", "unit_m", "0", "20");
        if identity == "quire.op.quantity.lt" {
            package.add("boolean", &all);
        }
        package.application(
            &json!({"application":{
                "operation": identity,
                "operand_types": [source],
                "result_type": result,
                "mode": mode
            }}),
            &all,
        );
        let CheckedPackageV2ReadResult::Admitted(model) = package.read() else {
            panic!("{identity} over bounded quantity must admit");
        };
        assert!(model.node_kinds().contains(&CheckedNodeKind::ScalarType(
            ScalarTypeForm::BoundedQuantity
        )));
    }
}

/// Trace: FR-038-AC-483; QSpec FR-322-AC-384.
#[trace("TC-048", "FR-038-AC-483")]
#[test]
fn bounded_quantity_add_requires_an_ordinary_unit_result() {
    let published = bounded_vectors();
    let case = published["forms"]
        .as_array()
        .expect("BQ forms")
        .iter()
        .find(|form| form["id"] == "BQ-08")
        .expect("BQ-08");
    assert!(case["operations"]
        .as_array()
        .expect("BQ operations")
        .iter()
        .any(|operation| operation == "quire.op.quantity.add"));
    let mut package = Package::new();
    let all = &vectors()["nodes"];
    for name in ["rational", "text", "integer"] {
        package.add(name, all);
    }
    package.bounded("bounded", "unit_m", "0", "20");
    let at = package.application(
        &json!({"application":{
            "operation":"quire.op.quantity.add", "operand_types":["bounded"],
            "result_type":"unit_m", "mode":null
        }}),
        all,
    );
    let bounded = package.ids["bounded"].clone();
    let node = &mut package.wire["semantic_graph"]["nodes"][at];
    node["semantic_type"] = bounded.clone();
    node["body"]["result_type"] = bounded;
    let CheckedPackageV2ReadResult::Refused(refusal) = package.read() else {
        panic!("quantity.add must refuse a bounded result type");
    };
    assert_eq!(
        refusal.code,
        quire_contract_model::CheckedPackageRefusalCode::IllTyped
    );
    assert_eq!(
        refusal.cause,
        Some(quire_contract_model::CheckedPackageRefusalCause::OperatorIneligible)
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some(format!("/semantic_graph/nodes/{at}"))
    );
}

/// Trace: FR-038-AC-483; QSpec FR-322-AC-384.
#[trace("TC-048", "FR-038-AC-483")]
#[test]
fn published_bounded_quantity_lt_rejects_other_dimension() {
    let bounded = bounded_vectors();
    let case = bounded["forms"]
        .as_array()
        .expect("BQ forms")
        .iter()
        .find(|form| form["id"] == "BQ-10")
        .expect("BQ-10");
    assert_eq!(case["expected"]["cause"], "operator-ineligible");
    let mut all = vectors()["nodes"].clone();
    all["boolean"] = json!({"tag":"scalar_type","form":"boolean",
        "path":["scalar_type","boolean"],"body":{"term":"aggregate","members":[]}});
    let mut package = Package::new();
    for name in ["rational", "text", "integer", "boolean"] {
        package.add(name, &all);
    }
    package.bounded("length", "unit_m", "0", "20");
    package.bounded("time", "unit_s", "0", "20");
    package.parameter("time_parameter", "time");
    let at = package.application(
        &json!({"application":{
            "operation":"quire.op.quantity.lt", "operand_types":["length"],
            "result_type":"boolean", "mode":null
        }}),
        &all,
    );
    let time_parameter = package.ids["time_parameter"].clone();
    package.wire["semantic_graph"]["nodes"][at]["body"]["arguments"][1]["target"] = time_parameter;
    let CheckedPackageV2ReadResult::Refused(refusal) = package.read() else {
        panic!("BQ-10 must refuse dimension mismatch");
    };
    assert_eq!(
        refusal.code,
        quire_contract_model::CheckedPackageRefusalCode::IllTyped
    );
    assert_eq!(
        refusal.cause,
        Some(quire_contract_model::CheckedPackageRefusalCause::OperatorIneligible)
    );
}

/// Trace: FR-038-AC-73; TC-050.
#[trace("TC-050", "FR-038-AC-73")]
#[test]
fn published_carried_quantity_positions_lower_without_covering_a_bare_unit() {
    let vectors = vectors();
    let all = &vectors["nodes"];
    let mut package = Package::new();
    for name in ["q_int", "q_dec", "q_rat", "text", "integer"] {
        package.add(name, all);
    }
    package.record("int_record", &["q_int"]);
    package.parameter("dec_parameter", "q_dec");
    package.record("rat_record", &["q_rat"]);
    package.record("mixed_record", &["unit_m", "q_int"]);
    let conversion = vectors["wire"]
        .as_array()
        .expect("published wire vectors")
        .iter()
        .find(|case| case["id"] == "CQ-W15")
        .expect("published carried-source conversion");
    assert_eq!(conversion["expected"], "admitted");
    assert_eq!(conversion["application"]["operand_types"][0], "q_pm2");
    assert_eq!(conversion["application"]["result_type"], "q_dec");
    let application = package.application(conversion, all);
    let nodes = package.wire["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    package.ids.insert(
        "source_parameter".into(),
        nodes[application - 1]["node_id"].clone(),
    );
    package.ids.insert(
        "quantity_application".into(),
        nodes[application]["node_id"].clone(),
    );
    let fixture_ids = package.ids.clone();
    let CheckedPackageV2ReadResult::Admitted(admitted) = package.read() else {
        panic!("published carried quantity forms must admit through the public reader");
    };
    let graph_ids = admitted
        .graph()
        .nodes
        .iter()
        .map(|node| json!({"node_id":node.node_id}))
        .collect::<Vec<_>>();
    let resolved = |name: &str| resolved_fixture_id(&fixture_ids[name], &graph_ids);
    let same_id = |actual: &CheckedNodeId, name: &str| {
        let wire = serde_json::to_value(actual).expect("encoded admitted node ID");
        resolved_fixture_id(&wire, &graph_ids) == resolved(name)
    };
    let source_parameter = admitted
        .graph()
        .nodes
        .iter()
        .find(|node| same_id(&node.node_id, "source_parameter"))
        .expect("admitted conversion source parameter");
    assert!(same_id(&source_parameter.semantic_type, "q_pm2"));
    let requests = [
        "int_record",
        "dec_parameter",
        "rat_record",
        "mixed_record",
        "quantity_application",
    ];
    let request_ids = requests
        .iter()
        .map(|name| resolved(name))
        .collect::<Vec<_>>();
    let profile = CompleteLoweringProfileV2 {
        reference: quire_contract_model::LoweringProfileRef {
            authority: "quire".into(),
            identity: "complete-v1".into(),
        },
        required_source_selections: Vec::new(),
        supported_tags: CheckedNodeTag::ALL.iter().copied().collect(),
        require_bounds: true,
        work_limit: 100_000,
    };
    let records = admitted.lower(&request_ids, &profile).records;
    assert_eq!(records.len(), requests.len());
    for (index, (request, carried, carrier, form)) in [
        (
            "int_record",
            "q_int",
            "int_pm2",
            BoundedDomainForm::IntegerRange,
        ),
        (
            "dec_parameter",
            "q_dec",
            "dec_cents",
            BoundedDomainForm::DecimalRange,
        ),
        (
            "rat_record",
            "q_rat",
            "rat_r",
            BoundedDomainForm::RationalRange,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(
            all[carried]["carrier"], carrier,
            "{carried} carrier identity"
        );
        let carrier_kind = admitted
            .graph()
            .nodes
            .iter()
            .zip(admitted.node_kinds())
            .find(|(node, _)| same_id(&node.node_id, carrier))
            .map(|(_, kind)| *kind)
            .expect("admitted carrier node");
        assert_eq!(
            carrier_kind,
            CheckedNodeKind::BoundedDomain(form),
            "{carried}"
        );
        let unit = all[carried]["unit"].as_str().expect("published unit");
        match &records[index] {
            CompleteLoweringRecordV2::Lowered { node } => {
                assert!(same_id(&node.node.node_id, request), "{request}");
                assert!(
                    node.bounds.iter().any(|id| same_id(id, carrier)),
                    "{request}: carrier bound missing"
                );
                assert!(
                    node.dependencies.iter().any(|id| same_id(id, unit)),
                    "{request}: unit dependency missing"
                );
            }
            other => panic!("{request} should lower at its carried type: {other:?}"),
        }
    }
    match &records[3] {
        CompleteLoweringRecordV2::RequiresBound {
            node_id,
            unbounded_type,
        } => {
            assert!(same_id(node_id, "mixed_record"));
            assert!(same_id(unbounded_type, "unit_m"));
        }
        other => panic!("bare unit beside a carried field must refuse: {other:?}"),
    }
    match &records[4] {
        CompleteLoweringRecordV2::Lowered { node } => {
            assert!(same_id(&node.node.node_id, "quantity_application"));
            assert!(same_id(&node.semantic_type, "q_dec"));
            assert!(
                node.dependencies
                    .iter()
                    .any(|id| same_id(id, "source_parameter")),
                "carried-typed source parameter must be in the application closure"
            );
        }
        other => panic!("carried-source quantity-result application must lower: {other:?}"),
    }
}

/// Trace: FR-038-AC-401 through FR-038-AC-405, FR-038-AC-450 through FR-038-AC-452; FR-019-AC-20.
#[trace(
    "TC-048",
    "TC-058",
    "FR-038-AC-401",
    "FR-038-AC-402",
    "FR-038-AC-403",
    "FR-038-AC-404",
    "FR-038-AC-405",
    "FR-038-AC-450",
    "FR-038-AC-451",
    "FR-038-AC-452",
    "FR-019-AC-20"
)]
#[test]
fn published_carried_quantity_vectors_read_as_wire() {
    let vectors = vectors();
    let all = &vectors["nodes"];
    let wire = vectors["wire"].as_array().expect("wire vectors");
    let evaluation = vectors["evaluation"]
        .as_array()
        .expect("evaluation controls");
    assert_published_ids(wire, "CQ-W", &["CQ-W16", "CQ-W17", "CQ-W18", "CQ-W19"]);
    assert_published_ids(
        evaluation,
        "CQ-E",
        &["CQ-E21", "CQ-E22", "CQ-E23", "CQ-E24", "CQ-E25", "CQ-E26"],
    );
    for case in wire {
        let id = case["id"].as_str().expect("vector id");
        match id {
            "CQ-W16" | "CQ-W18" => assert_eq!(case["expected"], "admitted", "{id}"),
            "CQ-W17" => {
                assert_eq!(
                    case["expected"], "invalid_package/invalid-derived-shape",
                    "{id}"
                );
                assert_eq!(case["code"], "invalid_package", "{id}");
                assert_eq!(case["cause"], "invalid-derived-shape", "{id}");
            }
            "CQ-W19" => {
                assert_eq!(
                    case["expected"], "invalid_package/operation-mode-type-mismatch",
                    "{id}"
                );
                assert_eq!(case["code"], "invalid_package", "{id}");
                assert_eq!(case["cause"], "operation-mode-type-mismatch", "{id}");
            }
            _ => {}
        }
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
        let expected_ids = package.ids.clone();
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
                for name in case["nodes"].as_array().expect("distinct nodes") {
                    assert_carried_bindings(
                        &admitted,
                        name.as_str().expect("node name"),
                        all,
                        &expected_ids,
                    );
                }
            } else if case["node"].is_string() {
                let name = case["node"].as_str().expect("node name");
                assert_carried_bindings(&admitted, name, all, &expected_ids);
            }
            if matches!(id, "CQ-W16" | "CQ-W18") {
                assert_rational_carrier_bounds(&admitted, all, &expected_ids);
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
    for case in evaluation {
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
    package.compound_unit();
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
