// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The original option-vector document and package preparation adapter.

use crate::support::checked_package as fixture;
use quire_contract_model::CheckedPackageEvidence;
use serde_json::{json, Value};

pub(crate) fn apply(value: &mut Value, patch: &Value) {
    let path = patch["path"].as_str().unwrap();
    let (parent, name) = path.rsplit_once('/').unwrap();
    let target = value.pointer_mut(parent).unwrap();
    match (patch["op"].as_str().unwrap(), target) {
        ("add", Value::Array(items)) if name == "-" => items.push(patch["value"].clone()),
        ("add", Value::Object(members)) => {
            members.insert(name.into(), patch["value"].clone());
        }
        ("replace", Value::Object(members)) => {
            *members.get_mut(name).unwrap() = patch["value"].clone();
        }
        ("replace", Value::Array(items)) => {
            items[name.parse::<usize>().unwrap()] = patch["value"].clone();
        }
        ("remove", Value::Object(members)) => {
            assert!(members.remove(name).is_some());
        }
        _ => panic!("unsupported published patch {patch}"),
    }
}

fn slot(value: &Value, owner: &str, field: bool) -> Value {
    let mut value = value.clone();
    let object = value.as_object_mut().unwrap();
    if let Some(name) = object.get("name").and_then(Value::as_str) {
        object.insert("identity".into(), json!(format!("{owner}/{name}")));
    }
    object.entry("presence").or_insert(json!("required"));
    object.entry("nullable").or_insert(json!(false));
    object.entry("defaultKind").or_insert(json!("none"));
    object
        .entry("multiplicity")
        .or_insert(json!({"lower":1,"upper":1,"ordered":false,"unique":true}));
    for bound in ["lower", "upper"] {
        if let Some(text) = value["multiplicity"][bound].as_str() {
            value["multiplicity"][bound] = json!(text.parse::<u64>().unwrap());
        }
    }
    if field && value["redefines"].is_null() {
        value.as_object_mut().unwrap().remove("redefines");
    }
    value
}

pub(crate) struct Input {
    pub(crate) document: Value,
    /// Map full Semantic IR type rows back to the published projection's rows.
    pub(crate) origins: Vec<String>,
}

pub(crate) fn document(projection: &Value) -> Input {
    let identity = projection["identity"].as_str().unwrap();
    let mut types = Vec::new();
    let mut origins = Vec::new();
    for (group, kind, meaning) in [
        ("option_types", "option", None),
        ("aliases", "alias", None),
        ("sequence_types", "sequence", None),
        ("map_types", "map", None),
        ("reference_types", "reference", None),
        (
            "record_types",
            "record",
            Some("quire.meaning.model.record-value-type/v1"),
        ),
        (
            "tuple_types",
            "tuple",
            Some("quire.meaning.model.tuple-value-type/v1"),
        ),
        (
            "variant_types",
            "variant",
            Some("quire.meaning.model.variant-type/v1"),
        ),
        (
            "namespaces",
            "namespace",
            Some("quire.meaning.model.namespace/v1"),
        ),
        (
            "object_types",
            "entity",
            Some("quire.meaning.model.object-type/v1"),
        ),
    ] {
        for (position, row) in projection[group]
            .as_array()
            .into_iter()
            .flatten()
            .enumerate()
        {
            let owner = row["node"].as_str().unwrap();
            let mut row = row.clone();
            row.as_object_mut().unwrap().remove("node");
            row["identity"] = json!(owner);
            row["kind"] = if meaning.is_some() {
                json!({"module":identity,"name":kind})
            } else {
                json!(kind)
            };
            if meaning.is_some() {
                row["displayName"] = json!(owner.rsplit('/').next().unwrap());
                if row["interfaceFeatures"].is_null() {
                    row.as_object_mut().unwrap().remove("interfaceFeatures");
                }
                for member in ["fields", "operations", "supertypes"] {
                    if matches!(kind, "entity" | "record") && row.get(member).is_none() {
                        row[member] = json!([]);
                    }
                }
                if let Some(fields) = row.get_mut("fields").and_then(Value::as_array_mut) {
                    for field in fields {
                        *field = slot(field, owner, true);
                    }
                }
                if let Some(operations) = row.get_mut("operations").and_then(Value::as_array_mut) {
                    for operation in operations {
                        let name = operation["name"].as_str().unwrap();
                        let key = format!("{owner}/{name}");
                        operation["identity"] = json!(key);
                        if let Some(parameters) = operation["params"].as_array_mut() {
                            for parameter in parameters {
                                *parameter = slot(parameter, &key, false);
                            }
                        }
                        if let Some(result) = operation.get_mut("returns") {
                            *result = slot(result, &key, false);
                        }
                    }
                }
                if kind == "tuple" {
                    if let Some(positions) = row["positions"].as_array_mut() {
                        for (index, position) in positions.iter_mut().enumerate() {
                            position["identity"] = json!(format!("{owner}/p{index}"));
                        }
                    }
                }
                if kind == "variant" {
                    let mut cases = row
                        .as_object_mut()
                        .unwrap()
                        .remove("cases")
                        .unwrap_or(json!([]));
                    for case in cases.as_array_mut().unwrap() {
                        case["identity"] =
                            json!(format!("{owner}/{}", case["name"].as_str().unwrap()));
                        if case["payloadType"].is_null() {
                            case.as_object_mut().unwrap().remove("payloadType");
                        }
                    }
                    row["variants"] = cases;
                }
            }
            origins.push(format!("/{group}/{position}"));
            types.push(row);
        }
    }
    let constructs = [
        ("entity","quire.meaning.model.object-type/v1"),
        ("record","quire.meaning.model.record-value-type/v1"),
        ("tuple","quire.meaning.model.tuple-value-type/v1"),
        ("variant","quire.meaning.model.variant-type/v1"),
        ("namespace","quire.meaning.model.namespace/v1"),
    ].into_iter().map(|(name,meaning)|json!({"kind":{"module":identity,"name":name},"construct":{"meaning":meaning}})).collect::<Vec<_>>();
    let units = projection["units"].as_array().into_iter().flatten().map(|unit| {
        let key = unit["node"].as_str().unwrap();
        json!({"identity":key,"name":key.rsplit('/').next().unwrap(),"dimension":format!("ix://{identity}/dimension/Time"),
            "scale":"1","offset":"0"})
    }).collect::<Vec<_>>();
    Input {
        document: json!({"contractVersion":"2.0.0","package":{"identity":identity,"version":projection["version"]},
        "constructs":constructs,"types":types,"units":units,
        "dimensions":[{"identity":format!("ix://{identity}/dimension/Time"),"name":"Time","base":true,"terms":[]}],
        "constraints":projection.get("constraints").cloned().unwrap_or(json!([]))}),
        origins,
    }
}

pub(crate) fn model_id(identity: &str, node: &str, form: &str) -> Value {
    fixture::structural_id(identity, vec![json!("model"), json!(form), json!(node)], 0)
}

pub(crate) fn prepare(input: &Input) -> (Value, CheckedPackageEvidence) {
    // The vector projection assumes an available ms unit. Supply a genuine source-owned
    // unit declaration with that name; the option reader receives its admitted unit handle.
    let members = fixture::nominal_fixture_members();
    let mut unit = members[2].0.clone();
    unit["qualified_declaration"] = json!(["ms"]);
    let mut dimension = members[3].0.clone();
    dimension["qualified_declaration"] = json!(["Time"]);
    unit["dimension_node_id"] = fixture::node_id("qualified:Time");
    let members = vec![
        (unit, "qualified:ms".to_owned()),
        (dimension, "qualified:Time".to_owned()),
    ];
    let mut package = fixture::nominal_package(&members);
    let identity = input.document["package"]["identity"].as_str().unwrap();
    for ty in input.document["types"].as_array().unwrap() {
        let form = match ty["kind"]["name"].as_str() {
            Some("entity") => "object_type",
            Some("record") => "record_value_type",
            Some("tuple") => "tuple_value_type",
            Some("variant") => "variant_type",
            _ => continue,
        };
        let key = model_id(identity, ty["identity"].as_str().unwrap(), form);
        package["semantic_graph"]["nodes"].as_array_mut().unwrap().push(json!({
            "node_id":key,"schema_version":"quire.checked-semantic-graph/v2","node_tag":"model","semantic_form":form,
            "semantic_type":key,"dependencies":[],"occurrences":[{"role":"type","ordinal":0}],
            "owner":{"kind":"model","identity":identity,"node":ty["identity"]},"body":{"term":"aggregate","members":[]}
        }));
    }
    fixture::select_model(&mut package, identity);
    fixture::settle(&mut package);
    let mut evidence = fixture::evidence_with_document(&package, identity, &input.document);
    evidence.support_feature("quire.value.complete/v1");
    (package, evidence)
}

/// The original field queries share one admitted projection. These rows are
/// consumed after that read; none describes an independent package mutation.
pub(crate) fn field_query_inputs(vectors: &Value) -> (Input, &[Value]) {
    (
        document(&vectors["domain_package"]),
        vectors["field_type_cases"]
            .as_array()
            .expect("published option field queries"),
    )
}
