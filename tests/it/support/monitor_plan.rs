// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Complete monitor fixtures assembled from selected QSpec facts by reference.
#![allow(dead_code)] // Public-consumer and conformance binaries use different subsets.

use super::{fixture_support, fixtures_dir, proposal_dir_of, read_json};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub(super) fn recipes() -> Value {
    read_json(&proposal_dir_of(&fixtures_dir()).join("monitor-plan-vectors.json"))
        .expect("selected QSpec monitor recipes")
}
pub(super) fn aggregate(members: Vec<Value>) -> Value {
    json!({"term":"aggregate","members":members})
}
pub(super) fn binding(name: &str, value: Value) -> Value {
    json!({"term":"binding","name":name,"value":value})
}
pub(super) fn reference(id: &Value) -> Value {
    json!({"term":"reference","target":id})
}
pub(super) fn literal(id: &Value, kind: &str, value: Value) -> Value {
    json!({"term":"literal","type":id,"value_kind":kind,"value":value})
}
pub(super) fn find(wire: &Value, form: &str) -> Value {
    wire["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["semantic_form"] == form)
        .unwrap()["node_id"]
        .clone()
}
pub(super) fn position(wire: &Value, id: &Value) -> usize {
    wire["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .position(|n| n["node_id"] == *id)
        .unwrap()
}
pub(super) fn operands(term: &Value, wire: &Value, out: &mut Vec<Value>, labels: bool) {
    match term["term"].as_str().expect("fixture term") {
        "reference" => out.push(json!({"node":position(wire,&term["target"])})),
        "literal" => out.push(
            term["value"]
                .as_str()
                .map_or_else(|| json!(term["value"].to_string()), |s| json!(s)),
        ),
        "aggregate" => {
            for member in term["members"].as_array().unwrap() {
                operands(member, wire, out, labels);
            }
        }
        "binding" => {
            if labels {
                out.push(term["name"].clone());
            }
            operands(&term["value"], wire, out, labels);
        }
        "application" => {
            out.push(term["operator"].clone());
            out.push(term["operation"]["identity"].clone());
            out.push(json!({"node":position(wire,&term["result_type"])}));
            for argument in term["arguments"].as_array().unwrap() {
                operands(argument, wire, out, labels);
            }
        }
        other => panic!("fixture content term {other}"),
    }
}
pub(super) fn function_path(wire: &Value, form: &str, body: &Value) -> Vec<Value> {
    let members = body["members"]
        .as_array()
        .expect("ordinary function bindings");
    assert!(matches!(members.len(), 2 | 3));
    assert_eq!(members[0]["name"], "parameters");
    assert_eq!(members[1]["name"], "body");
    let parameters = members[0]["value"]["members"]
        .as_array()
        .expect("parameter references");
    let mut path = vec![
        json!("function"),
        json!(form),
        json!(parameters.len().to_string()),
    ];
    for parameter in parameters {
        path.push(json!({"node":position(wire,&parameter["target"])}));
    }
    path.push(json!({"node":position(wire,&members[1]["value"]["target"])}));
    if let Some(measure) = members.get(2) {
        assert_eq!(measure["name"], "measure");
        path.push(json!({"node":position(wire,&measure["value"]["target"])}));
    }
    path
}

pub(super) fn rekey(wire: &mut Value, position: usize) {
    let node = &wire["semantic_graph"]["nodes"][position];
    let old = node["node_id"].clone();
    let path = if node["node_tag"] == "function"
        && node.get("declaration").is_none()
        && node.get("owner").is_none()
    {
        function_path(wire, node["semantic_form"].as_str().unwrap(), &node["body"])
    } else {
        let mut path = vec![node["node_tag"].clone(), node["semantic_form"].clone()];
        operands(
            &node["body"],
            wire,
            &mut path,
            node["semantic_form"] != "monitor_plan",
        );
        path
    };
    let fresh = json!({"package":old["package"],"path":path,"ordinal":old["ordinal"]});
    fn replace(value: &mut Value, old: &Value, fresh: &Value) {
        if value == old {
            *value = fresh.clone();
            return;
        }
        match value {
            Value::Array(a) => {
                for v in a {
                    replace(v, old, fresh);
                }
            }
            Value::Object(o) => {
                for v in o.values_mut() {
                    replace(v, old, fresh);
                }
            }
            _ => {}
        }
    }
    replace(wire, &old, &fresh);
}

pub(super) fn append(
    wire: &mut Value,
    tag: &str,
    form: &str,
    ty: Option<&Value>,
    body: Value,
    suffix: &str,
) -> Value {
    let n = wire["semantic_graph"]["nodes"].as_array().unwrap().len();
    let mut path = vec![json!(tag), json!(form), json!(suffix), json!(n.to_string())];
    if tag == "function" {
        path = function_path(wire, form, &body);
    } else if tag == "expression" {
        operands(&body, wire, &mut path, true);
    }
    let id = json!({"package":wire["name"],"path":path,"ordinal":0});
    wire["semantic_graph"]["nodes"].as_array_mut().unwrap().push(json!({
        "schema_version":"quire.checked-semantic-graph/v2","node_id":id,"node_tag":tag,"semantic_form":form,
        "semantic_type":ty.unwrap_or(&id),"body":body,"dependencies":[],"occurrences":[{"role":if form == "parameter" { "expression" } else { "generated" },"ordinal":0}]
    }));
    id
}
pub(super) fn type_node(wire: &mut Value, form: &str, body: Value, name: &str) -> Value {
    append(wire, "composite_type", form, None, body, name)
}
pub(super) fn record(wire: &mut Value, fields: Vec<(&str, Value)>, name: &str) -> Value {
    type_node(
        wire,
        "record",
        aggregate(
            fields
                .into_iter()
                .map(|(n, t)| binding(n, reference(&t)))
                .collect(),
        ),
        name,
    )
}
pub(super) fn constant(wire: &mut Value, ty: &Value, name: &str) -> Value {
    let type_node = wire["semantic_graph"]["nodes"][position(wire, ty)].clone();
    let form = type_node["semantic_form"].as_str().unwrap();
    let (value_form, body) = match form {
        "integer" => ("literal", literal(ty, "integer", json!("0"))),
        "text" => ("literal", literal(ty, "text", json!(""))),
        "string" => ("literal", literal(ty, "string", json!(""))),
        "sequence" => ("collection_value", aggregate(vec![])),
        "record" | "tuple" => {
            let mut values = vec![];
            for (i, field) in type_node["body"]["members"]
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
            {
                let (field_name, field_type) = if field["term"] == "binding" {
                    (field["name"].as_str(), &field["value"]["target"])
                } else {
                    (None, &field["target"])
                };
                let value = constant(wire, field_type, &format!("{name}_{i}"));
                values.push(
                    field_name.map_or_else(|| reference(&value), |n| binding(n, reference(&value))),
                );
            }
            (
                if form == "record" {
                    "record_value"
                } else {
                    "tuple_value"
                },
                aggregate(values),
            )
        }
        other => panic!("unsupported constant fixture type {other}"),
    };
    append(wire, "value", value_form, Some(ty), body, name)
}

pub(super) fn patch(facts: &mut Value, edits: &Value) {
    for edit in edits.as_array().expect("published edits") {
        let path = format!("/{}", edit[0].as_str().unwrap().replace('.', "/"));
        *facts
            .pointer_mut(&path)
            .unwrap_or_else(|| panic!("published fact path {path}")) = edit[1].clone();
    }
}
pub(super) fn finish(wire: &mut Value) {
    // References, including nested buffer entries, determine graph dependencies.
    let nodes = wire["semantic_graph"]["nodes"].as_array().unwrap().clone();
    let ids = wire["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["node_id"].clone())
        .collect::<Vec<_>>();
    fn targets(term: &Value, out: &mut Vec<Value>) {
        match term["term"].as_str() {
            Some("reference") => out.push(term["target"].clone()),
            Some("aggregate") => {
                for member in term["members"].as_array().unwrap() {
                    targets(member, out);
                }
            }
            Some("binding") => targets(&term["value"], out),
            Some("application") => {
                for arg in term["arguments"].as_array().unwrap() {
                    targets(arg, out);
                }
            }
            _ => {}
        }
    }
    for node in wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .skip(53)
    {
        if node["node_tag"] == "scalar_type"
            && matches!(node["semantic_form"].as_str(), Some("unit" | "dimension"))
        {
            continue;
        }
        let mut deps = vec![];
        targets(&node["body"], &mut deps);
        if node["semantic_form"] == "monitor_plan" {
            deps.push(node["semantic_type"].clone());
        }
        let mut ordered = deps
            .into_iter()
            .map(|id| {
                let decoded = fixture_support::resolved_fixture_id(&id, &nodes);
                (decoded, id)
            })
            .collect::<BTreeMap<_, _>>();
        // The added types have no indexed operands yet; indexed root references
        // are ordered below against the production package's supporting keys.
        node["dependencies"] = Value::Array(ordered.values_mut().map(|id| id.clone()).collect());
    }
    let _ = ids;
    fixture_support::rebuild_complete_source_map(wire);
    fixture_support::refresh_identity(wire);
}
pub(super) fn materialize(facts: &Value) -> (Value, usize) {
    let mut wire = read_json(&fixtures_dir().join("positive-tick-timed-model-nodes.json"))
        .expect("selected complete supporting fixture");
    assert_eq!(
        wire["semantic_graph"]["nodes"].as_array().unwrap().len(),
        53,
        "fixture construction boundary"
    );
    let integer = find(&wire, "integer");
    let text = find(&wire, "text");
    let rational = find(&wire, "rational");
    let string = append(
        &mut wire,
        "scalar_type",
        "string",
        None,
        aggregate(vec![]),
        "String",
    );
    let sequence = type_node(
        &mut wire,
        "sequence",
        aggregate(vec![reference(&integer)]),
        "buffer",
    );
    let activation_fields = facts["state"]["activation_fields"]
        .as_array()
        .unwrap()
        .iter()
        .map(|name| {
            let n = name.as_str().unwrap();
            (
                n,
                if n == "status" {
                    string.clone()
                } else {
                    integer.clone()
                },
            )
        })
        .collect();
    let activation = record(&mut wire, activation_fields, "Activation");
    let activations = type_node(
        &mut wire,
        "sequence",
        aggregate(vec![reference(&activation)]),
        "Activations",
    );
    let fields = facts["state"]["fields"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            let ty = match row[1].as_str().unwrap() {
                "integer" => integer.clone(),
                "text" => text.clone(),
                "sequence" => {
                    if row[0] == "activations" {
                        activations.clone()
                    } else {
                        sequence.clone()
                    }
                }
                other => panic!("state type {other}"),
            };
            (row[0].as_str().unwrap(), ty)
        })
        .collect();
    let state = if facts["state"]["form"] == "record" {
        record(&mut wire, fields, "State")
    } else {
        type_node(
            &mut wire,
            "tuple",
            aggregate(vec![reference(&integer)]),
            "State",
        )
    };
    let inputs = record(&mut wire, vec![], "Inputs");
    let scope = record(&mut wire, vec![], "Scope");
    let mut payloads = BTreeMap::new();
    for name in ["LimitReached", "ReadingGapExceeded"] {
        let fields = facts["payload_fields"][name]
            .as_array()
            .unwrap()
            .iter()
            .map(|field| {
                let n = field.as_str().unwrap();
                (
                    n,
                    if matches!(n, "limit" | "setting") {
                        string.clone()
                    } else {
                        integer.clone()
                    },
                )
            })
            .collect();
        payloads.insert(name, record(&mut wire, fields, name));
    }
    let cases = facts["outcome"]["cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            binding(
                row[0].as_str().unwrap(),
                aggregate(
                    row[1]
                        .as_str()
                        .map(|name| reference(&payloads[name]))
                        .into_iter()
                        .collect(),
                ),
            )
        })
        .collect();
    let outcome = type_node(&mut wire, "union", aggregate(cases), "Outcome");
    let instance = record(
        &mut wire,
        vec![
            ("subject", string.clone()),
            ("subformula", integer.clone()),
            ("origin", integer.clone()),
        ],
        "Instance",
    );
    let result = record(
        &mut wire,
        vec![("instance", instance), ("outcome", outcome)],
        "Result",
    );
    let results = type_node(
        &mut wire,
        "sequence",
        aggregate(vec![reference(&result)]),
        "Results",
    );
    let step = type_node(
        &mut wire,
        "tuple",
        aggregate(vec![reference(&state), reference(&results)]),
        "Step",
    );
    let types = BTreeMap::from([
        ("State", state.clone()),
        ("Inputs", inputs.clone()),
        ("Scope", scope.clone()),
        ("Step", step.clone()),
        ("integer", integer.clone()),
        ("text", text.clone()),
    ]);
    let mut entries = BTreeMap::new();
    for name in ["init", "on_event", "on_watermark", "on_gap"] {
        let fact = &facts["entries"][name];
        let mut parameters = vec![];
        for (i, ty) in fact["params"].as_array().unwrap().iter().enumerate() {
            let ty = &types[ty.as_str().unwrap()];
            let body = aggregate(vec![
                binding("name", literal(&text, "text", json!(format!("p{i}")))),
                binding("level", literal(&integer, "integer", json!(i.to_string()))),
            ]);
            parameters.push(append(
                &mut wire,
                "value",
                "parameter",
                Some(ty),
                body,
                &format!("{name}{i}"),
            ));
        }
        let result = &types[fact["result"].as_str().unwrap()];
        // A constant typed value and ordinary reference body are enough for
        // reader qualification; transition semantics are the producer's tests.
        let constant = constant(&mut wire, result, name);
        let boolean = find(&wire, "boolean");
        let expression = append(
            &mut wire,
            "expression",
            "conditional",
            Some(result),
            json!({"term":"application", "operator":"conditional", "operation":{
                "identity":"quire.op.control.if", "laws":[], "leaves":[], "member":null, "mode":null},
                "result_type":result, "arguments":[literal(&boolean,"boolean",json!(true)),reference(&constant),reference(&constant)]}),
            name,
        );
        let body = aggregate(vec![
            binding(
                "parameters",
                aggregate(parameters.iter().map(reference).collect()),
            ),
            binding("body", reference(&expression)),
        ]);
        let mut entry = append(
            &mut wire,
            "function",
            fact["form"].as_str().unwrap(),
            Some(result),
            body,
            &format!("{name}{}", fact["index"]),
        );
        if let Some(discriminator) = fact["index"].as_u64() {
            // A selected different node may share a producer path. Ordinals
            // are producer-assigned and distinguish those exact identities.
            let n = position(&wire, &entry);
            wire["semantic_graph"]["nodes"][n]["node_id"]["ordinal"] = json!(discriminator);
            entry = wire["semantic_graph"]["nodes"][n]["node_id"].clone();
        }
        if fact["declared"] == true {
            let n = position(&wire, &entry);
            wire["semantic_graph"]["nodes"][n]["owner"] = wire["semantic_graph"]["nodes"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["owner"]["kind"] == "source")
                .unwrap()["owner"]
                .clone();
            wire["semantic_graph"]["nodes"][n]["declaration"] =
                json!({"qualified_name":["declaredEntry"]});
            wire["semantic_graph"]["nodes"][n]["occurrences"] =
                json!([{"role":"declaration","ordinal":0}]);
            wire["semantic_graph"]["nodes"][n]["node_id"]["path"] = json!(["declaredEntry"]);
            entries.insert(name, wire["semantic_graph"]["nodes"][n]["node_id"].clone());
        } else {
            entries.insert(name, entry);
        }
    }
    let formulas = wire["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["semantic_form"] == "formula")
        .map(|n| n["node_id"].clone())
        .collect::<Vec<_>>();
    let mut ordered_formulas = formulas.clone();
    ordered_formulas.sort_by_key(|id| {
        fixture_support::resolved_fixture_id(
            id,
            wire["semantic_graph"]["nodes"].as_array().unwrap(),
        )
    });
    // These two published formulas are both reached from the clause.
    let boolean = find(&wire, "boolean");
    let unreached_body = wire["semantic_graph"]["nodes"][28]["body"].clone();
    let unreached = append(
        &mut wire,
        "temporal",
        "formula",
        Some(&boolean),
        unreached_body,
        "unreached",
    );
    let clause = if facts["clause_form"] == "temporal_clause" {
        find(&wire, "temporal_clause")
    } else {
        formulas[0].clone()
    };
    let buffers = aggregate(
        facts["buffers"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| {
                let subformula = if b["reached"] == false {
                    unreached.clone()
                } else {
                    ordered_formulas
                        [usize::try_from(b["subformula"].as_u64().unwrap()).unwrap() - 1]
                        .clone()
                };
                aggregate(vec![
                    binding("subformula", reference(&subformula)),
                    binding(
                        "capacity",
                        literal(&integer, "integer", json!(b["capacity"].to_string())),
                    ),
                ])
            })
            .collect(),
    );
    let mut unit = find(&wire, "unit");
    if facts["tick_unit"]["form"] != "unit" {
        unit = inputs.clone();
    } else if facts["tick_unit"]["dimension"] != "Time" || facts["tick_unit"]["index"] != 7 {
        let old = position(&wire, &unit);
        let mut new = wire["semantic_graph"]["nodes"][old].clone();
        let mut dimension = find(&wire, "dimension");
        if facts["tick_unit"]["dimension"] != "Time" {
            let old_dim = position(&wire, &dimension);
            let mut n = wire["semantic_graph"]["nodes"][old_dim].clone();
            n["declaration"]["qualified_name"] = json!(["Example", "Length"]);
            n["node_id"]["path"] = json!(["Example", "Length"]);
            n["semantic_type"] = n["node_id"].clone();
            dimension = n["node_id"].clone();
            wire["semantic_graph"]["nodes"]
                .as_array_mut()
                .unwrap()
                .push(n);
        }
        new["node_id"]["path"] = json!(["Example", "otherTick"]);
        new["declaration"]["qualified_name"] = json!(["Example", "otherTick"]);
        new["body"]["dimension_node_id"] = dimension.clone();
        new["semantic_type"] = dimension.clone();
        new["dependencies"] = json!([dimension]);
        if facts["tick_unit"]["dimension"] == "Time" {
            new["body"]["target_unit_node_id"] = unit.clone();
            new["dependencies"]
                .as_array_mut()
                .unwrap()
                .push(unit.clone());
        }
        unit = new["node_id"].clone();
        wire["semantic_graph"]["nodes"]
            .as_array_mut()
            .unwrap()
            .push(new);
    }
    let mut values = BTreeMap::from([
        ("clause", reference(&clause)),
        ("state", reference(&state)),
        ("inputs", reference(&inputs)),
        ("scope", reference(&scope)),
        ("results", reference(&results)),
        ("buffers", buffers),
        ("tick_unit", reference(&unit)),
        ("extra", aggregate(vec![])),
    ]);
    for name in ["init", "on_event", "on_watermark", "on_gap"] {
        values.insert(name, reference(&entries[name]));
    }
    for name in [
        "counter_width_bits",
        "max_reading_gap",
        "max_event_rate",
        "counter",
        "uncertainty",
        "activation_capacity",
    ] {
        let v = &facts["premises"][name];
        let (kind, ty) = match name {
            "counter_width_bits" | "max_reading_gap" | "activation_capacity" => {
                if v.is_number() {
                    ("integer", &integer)
                } else {
                    ("text", &text)
                }
            }
            "max_event_rate" | "uncertainty" => {
                if v.is_string() {
                    ("rational", &rational)
                } else {
                    ("integer", &integer)
                }
            }
            _ => {
                if v.is_string() {
                    ("text", &text)
                } else {
                    ("integer", &integer)
                }
            }
        };
        let value = if v.is_string() {
            v.clone()
        } else {
            json!(v.to_string())
        };
        values.insert(name, literal(ty, kind, value));
    }
    let body = aggregate(
        facts["bindings"]
            .as_array()
            .unwrap()
            .iter()
            .map(|name| {
                binding(
                    name.as_str().unwrap(),
                    values[name.as_str().unwrap()].clone(),
                )
            })
            .collect(),
    );
    let root = append(
        &mut wire,
        "temporal",
        "monitor_plan",
        Some(&state),
        body,
        "plan",
    );
    let n = position(&wire, &root);
    rekey(&mut wire, n);
    finish(&mut wire);
    (wire, n)
}
