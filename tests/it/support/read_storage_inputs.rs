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
    CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageV2ReadResult,
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
    value
}

fn named_package(name: &str) -> Value {
    let mut value = v2_all_families();
    value["name"] = json!(name);
    index_fixture_node_ids(&mut value);
    value
}

pub fn direct_dependencies_64() -> Input {
    let mut value = named_package("allocation-root");
    let mut evidence = evidence_for(&value);
    let mut selections = Vec::new();
    for i in 0..64 {
        let identity = format!("allocation-dependency-{i:02}");
        let dependency = named_package(&identity);
        let bytes = canonical(&dependency);
        let result = crate::read_and_account(
            &bytes,
            CheckedPackageReadLimits::bounded(),
            &evidence_for(&dependency),
        );
        let CheckedPackageV2ReadResult::Admitted(package) = result else {
            panic!("dependency fixture {identity} must admit: {result:?}");
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

/// Native tick source built around the existing admitted state/operation
/// fixture. The period is a positive coprime rational with 130-bit operands.
pub fn timed_magnitude_from_state_fixture(mut wire: Value, period: &str) -> Input {
    use super::checked_package::{
        node_id, nominal_fixture_members, nominal_package, reassign_node_id,
        rebuild_complete_source_map, structural_id,
    };
    let nodes = wire["semantic_graph"]["nodes"]
        .as_array()
        .expect("state nodes");
    let anchor = nodes
        .iter()
        .find(|node| node["semantic_form"] == "operation_anchor")
        .expect("original operation anchor")["node_id"]
        .clone();
    let model = nodes
        .iter()
        .find(|node| node["node_id"] == anchor)
        .expect("anchor")["semantic_type"]
        .clone();
    let text = nodes
        .iter()
        .find(|node| node["semantic_form"] == "text")
        .expect("text")["node_id"]
        .clone();
    let mut nominal = nominal_package(&nominal_fixture_members());
    let time = node_id("qualified:Example/Time");
    reassign_node_id(&mut nominal, &node_id("qualified:Example/Length"), &time);
    for node in nominal["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nominal nodes")
    {
        if node["semantic_form"] == "dimension" {
            node["declaration"]["qualified_name"] = json!(["Example", "Time"]);
        }
    }
    let unit = nominal["semantic_graph"]["nodes"]
        .as_array()
        .expect("nominal nodes")
        .iter()
        .find(|node| node["semantic_form"] == "unit")
        .expect("unit")["node_id"]
        .clone();
    let name = wire["name"].as_str().expect("name");
    let rational = structural_id(name, vec![json!("scalar_type"), json!("rational")], 0);
    let source = structural_id(
        name,
        vec![
            json!("timed_model"),
            json!("time_source"),
            json!("numeric-period"),
        ],
        0,
    );
    let root = structural_id(name, vec![json!("Numeric"), json!("Timed")], 0);
    let binding = |name: &str, value: Value| json!({"term":"binding","name":name,"value":value});
    let reference = |target: &Value| json!({"term":"reference","target":target});
    let empty = || json!({"term":"aggregate","members":[]});
    let mut source_deps = vec![unit.clone(), model.clone(), anchor.clone()];
    source_deps.sort_by_key(|value| canonical(value));
    source_deps.dedup();
    let mut root_deps = vec![model.clone(), time.clone(), source.clone()];
    root_deps.sort_by_key(|value| canonical(value));
    root_deps.dedup();
    let source_body = json!({"term":"aggregate","members":[
        binding("kind",json!({"term":"literal","type":text,"value_kind":"text","value":"tick"})),
        binding("unit",reference(&unit)),binding("model",reference(&model)),binding("operation",reference(&anchor)),
        binding("period",json!({"term":"literal","type":rational,"value_kind":"rational","value":period}))
    ]});
    let root_body = json!({"term":"aggregate","members":[binding("model",reference(&model)),binding("time_dimension",reference(&time)),binding("source",reference(&source)),binding("clock_places",empty()),binding("resets",empty()),binding("guards",empty()),binding("invariants",empty()),binding("urgencies",empty()),binding("constraints",empty())]});
    let source_owner = wire["lock"]["sources"][0].clone();
    let additions = nominal["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .filter(|node| matches!(node["semantic_form"].as_str(), Some("dimension" | "unit")))
        .cloned()
        .collect::<Vec<_>>();
    let nodes = wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    nodes.extend(additions);
    nodes.extend([
        json!({"node_id":rational,"schema_version":"quire.checked-semantic-graph/v2","node_tag":"scalar_type","semantic_form":"rational","semantic_type":rational,"dependencies":[],"occurrences":[{"role":"type","ordinal":0}],"body":empty()}),
        json!({"node_id":source,"schema_version":"quire.checked-semantic-graph/v2","node_tag":"timed_model","semantic_form":"time_source","semantic_type":source,"dependencies":source_deps,"occurrences":[{"role":"expression","ordinal":0}],"body":source_body}),
        json!({"node_id":root,"schema_version":"quire.checked-semantic-graph/v2","node_tag":"timed_model","semantic_form":"timed_model","semantic_type":root,"dependencies":root_deps,"occurrences":[{"role":"declaration","ordinal":0}],"declaration":{"qualified_name":["Numeric","Timed"]},"owner":{"kind":"source","authority":source_owner["authority"],"identity":source_owner["identity"]},"body":root_body})
    ]);
    rebuild_complete_source_map(&mut wire);
    index_fixture_node_ids(&mut wire);
    Input {
        bytes: canonical(&wire),
        evidence: evidence_for(&wire),
    }
}

/// The original native StatePackage constructor and its pure vocabulary helpers.
/// StatePackage::new delegates here; the dedicated allocator imports this same
/// module instead of a copy of the fixture or the original test module.
pub(crate) mod state_fixture {
    use super::super::checked_package::{
        family_key, node_id as fixture_node_id, settle, structural_id, v2_all_families,
        BOOLEAN_KEY, INTEGER_KEY,
    };
    use serde_json::{json, Value};
    use std::sync::LazyLock;
    pub(crate) const INTEGER: &str = INTEGER_KEY;
    pub(crate) fn node_id(label: &str) -> Value {
        if label == "model-scalar:integer" {
            return structural_id(
                "test/families",
                vec![json!("scalar_type"), json!("integer")],
                0,
            );
        }
        if let Some(model) = label.strip_prefix("model:") {
            let mut parts = model.splitn(3, ':');
            let form = parts.next().expect("model form");
            let identity = parts.next().expect("model package");
            let declaration = parts.next().expect("model declaration");
            return structural_id(
                identity,
                vec![json!("model"), json!(form), json!(declaration)],
                0,
            );
        }
        fixture_node_id(label)
    }

    pub(crate) fn nodes(value: &Value) -> &Vec<Value> {
        value["semantic_graph"]["nodes"].as_array().expect("nodes")
    }

    pub(crate) fn position(value: &Value, tag: &str, form: &str) -> usize {
        nodes(value)
            .iter()
            .position(|node| node["node_tag"] == tag && node["semantic_form"] == form)
            .unwrap_or_else(|| panic!("a {tag}/{form} node"))
    }

    pub(crate) fn digest(value: &Value, position: usize) -> String {
        nodes(value)[position]["node_id"]["path"][0]
            .as_str()
            .expect("digest")
            .to_owned()
    }

    pub(crate) fn object_key() -> &'static str {
        "fixture/1515"
    }

    pub(crate) fn text_key() -> &'static str {
        static KEY: LazyLock<String> = LazyLock::new(|| family_key("a1a1"));
        &KEY
    }

    pub(crate) fn reference_key(target: &str) -> String {
        format!("fixture/frame/reference/{target}")
    }

    pub(crate) fn plain(
        digest: &str,
        tag: &str,
        form: &str,
        ty: &str,
        deps: &[&str],
        role: &str,
        body: Value,
    ) -> Value {
        json!({
            "node_id": node_id(digest),
            "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": tag,
            "semantic_form": form,
            "semantic_type": node_id(ty),
            "dependencies": deps.iter().map(|d| node_id(d)).collect::<Vec<_>>(),
            "occurrences": [{"role": role, "ordinal": 0}],
            "body": body,
        })
    }

    pub(crate) fn self_key() -> &'static str {
        "fixture/frame/self-parameter"
    }

    pub(crate) fn anchor_key() -> &'static str {
        "fixture/frame/deposit-anchor"
    }

    pub(crate) fn reference(digest: &str) -> Value {
        json!({"term": "reference", "target": node_id(digest)})
    }

    pub(crate) fn binding(name: &str, value: Value) -> Value {
        json!({"term": "binding", "name": name, "value": value})
    }

    pub(crate) fn text(value: &str) -> Value {
        json!({"term": "literal", "type": node_id(text_key()), "value_kind": "text", "value": value})
    }

    pub(crate) fn anchor_body(context: &str, operation: Value, frame: &str) -> Value {
        json!({"term": "aggregate", "members": [
            binding("context", reference(context)),
            binding("operation", operation),
            binding("frame", reference(frame)),
        ]})
    }

    pub(crate) fn clause_body(
        clause: &str,
        parameters: &[&str],
        anchor: &str,
        condition: &str,
    ) -> Value {
        json!({
            "term": "application",
            "operator": "state_clause",
            "operation": {"identity": "quire.op.state.clause", "laws": [], "mode": null,
                "member": {"kind": "state_clause", "clause": clause}, "leaves": []},
            "result_type": node_id(&boolean()),
            "arguments": [
                {"term": "aggregate",
                    "members": parameters.iter().map(|p| reference(p)).collect::<Vec<_>>()},
                reference(anchor),
                reference(condition),
            ],
        })
    }

    pub(crate) fn boolean() -> String {
        BOOLEAN_KEY.to_owned()
    }

    pub(crate) fn condition() -> String {
        family_key("dddd")
    }

    pub(crate) fn clause_dependencies(parameters: &[&str], anchor: &str) -> Vec<String> {
        let mut deps: Vec<String> = parameters
            .iter()
            .map(|p| (*p).to_owned())
            .chain([anchor.to_owned(), condition()])
            .collect();
        deps.sort();
        deps.dedup();
        deps
    }

    pub(crate) fn clause_node(clause: &str, parameters: &[&str], anchor: &str) -> Value {
        let deps = clause_dependencies(parameters, anchor);
        let deps: Vec<&str> = deps.iter().map(String::as_str).collect();
        let placeholder = match clause {
            "invariant" => "c1",
            "precondition" => "c2",
            "constraint" => "c7",
            _ => "c3",
        };
        plain(
            &placeholder.repeat(32),
            "state",
            "state_clause",
            &boolean(),
            &deps,
            "claim",
            clause_body(clause, parameters, anchor, &condition()),
        )
    }
    pub(crate) fn package() -> Value {
        let mut value = v2_all_families();
        let frame = position(&value, "state", "frame");
        let frame_key = digest(&value, frame);
        let reference_type = reference_key(object_key());
        let added = [
            plain(
                &reference_type,
                "composite_type",
                "reference",
                &reference_type,
                &[object_key()],
                "type",
                json!({"term": "aggregate", "members": [reference(object_key())]}),
            ),
            plain(
                self_key(),
                "value",
                "parameter",
                &reference_type,
                &[],
                "expression",
                json!({"term": "aggregate", "members": [
                    binding("name", text("self")),
                    binding("level", json!({"term": "literal", "type": node_id(INTEGER),
                        "value_kind": "integer", "value": "0"})),
                ]}),
            ),
            plain(
                anchor_key(),
                "state",
                "operation_anchor",
                object_key(),
                &[object_key(), &frame_key],
                "anchor",
                anchor_body(object_key(), text("deposit"), &frame_key),
            ),
            clause_node("invariant", &[self_key()], object_key()),
            clause_node("precondition", &[self_key()], anchor_key()),
        ];
        value["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("nodes")
            .extend(added);
        settle(&mut value);
        value
    }
}

pub fn timed_magnitude_multilimb() -> Input {
    timed_magnitude_from_state_fixture(
        state_fixture::package(),
        "680564733841876926926749214863536422913/680564733841876926926749214863536422915",
    )
}
