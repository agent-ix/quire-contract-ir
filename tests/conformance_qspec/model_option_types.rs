// SPDX-License-Identifier: AGPL-3.0-or-later
//! OP vectors are read from QSpec; only the projection-to-document adapter lives here.

use super::{fixture_support as fixture, fixtures_dir, proposal_dir_of, read_json};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedCollectionKind, CheckedMemberType, CheckedNodeId, CheckedPackageReadLimits,
    CheckedPackageV2, CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

fn vectors() -> Value {
    read_json(&proposal_dir_of(&fixtures_dir()).join("model-option-type-vectors.json"))
        .expect("the current QSpec option corpus")
}

fn apply(value: &mut Value, patch: &Value) {
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

struct Input {
    document: Value,
    /// Map full Semantic IR type rows back to the published projection's rows.
    origins: Vec<String>,
}

fn document(projection: &Value) -> Input {
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
        "dimensions":[{"identity":format!("ix://{identity}/dimension/Time"),"name":"Time","terms":[]}],
        "constraints":projection.get("constraints").cloned().unwrap_or(json!([]))}),
        origins,
    }
}

fn model_id(identity: &str, node: &str, form: &str) -> Value {
    fixture::structural_id(identity, vec![json!("model"), json!(form), json!(node)], 0)
}

fn read(input: &Input) -> CheckedPackageV2ReadResult {
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
    CheckedPackageV2::read(
        &fixture::canonical(&package),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    )
}

fn projected_pointer(input: &Input, pointer: &str) -> String {
    if let Some(rest) = pointer.strip_prefix("/types/") {
        let (index, suffix) = rest
            .split_once('/')
            .map_or((rest, ""), |(index, suffix)| (index, suffix));
        let prefix = &input.origins[index.parse::<usize>().unwrap()];
        let suffix = if prefix.starts_with("/variant_types/") {
            suffix.replacen("variants", "cases", 1)
        } else {
            suffix.to_owned()
        };
        if suffix.is_empty() {
            prefix.clone()
        } else {
            format!("{prefix}/{suffix}")
        }
    } else {
        pointer.to_owned()
    }
}

fn expected(value: &Value, unit: &CheckedNodeId, identity: &str) -> CheckedMemberType {
    if let Some(inner) = value.get("option") {
        return CheckedMemberType::Option(Box::new(expected(inner, unit, identity)));
    }
    if let Some(inner) = value.get("sequence") {
        return CheckedMemberType::Collection {
            kind: CheckedCollectionKind::Sequence,
            element: Box::new(expected(inner, unit, identity)),
            bounds: Some((
                value["min"].as_str().unwrap().parse().unwrap(),
                value["max"].as_str().unwrap().parse().unwrap(),
            )),
        };
    }
    if let Some(name) = value["declaration"].as_str() {
        return CheckedMemberType::Record(
            serde_json::from_value(model_id(
                identity,
                &format!("ix://{identity}/{name}"),
                "record_value_type",
            ))
            .unwrap(),
        );
    }
    match value["type"].as_str().unwrap() {
        "Integer" => CheckedMemberType::Integer,
        "Quantity<ms>" => CheckedMemberType::Quantity(unit.clone()),
        other => panic!("new published expected type needs its independent decoder: {other}"),
    }
}

#[trace("TC-465", "FR-356-AC-1", "FR-356-AC-2", "FR-356-AC-5")]
#[test]
fn published_option_field_types_match_canonical_domains() {
    let vectors = vectors();
    let input = document(&vectors["domain_package"]);
    let admitted = match read(&input) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("base option projection must admit: {other:?}"),
    };
    let identity = input.document["package"]["identity"].as_str().unwrap();
    let holder: CheckedNodeId = serde_json::from_value(model_id(
        identity,
        &format!("ix://{identity}/Holder"),
        "object_type",
    ))
    .unwrap();
    let fields = admitted.model_object_fields(&holder).unwrap();
    let unit = admitted
        .graph()
        .nodes
        .iter()
        .find(|node| {
            node.declaration.as_ref().is_some_and(|declaration| {
                declaration
                    .qualified_name
                    .last()
                    .is_some_and(|name| name.as_ref() == "ms")
            })
        })
        .unwrap()
        .node_id
        .clone();
    let mut seen = BTreeSet::new();
    for case in vectors["field_type_cases"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        assert!(seen.insert(name));
        if let Some(field) = case["field"].as_str() {
            let actual = fields.field(field).unwrap().member_type().unwrap();
            assert_eq!(
                actual,
                &expected(&case["expected"], &unit, identity),
                "{name}"
            );
            if let Some(other) = case["same_node_as"].as_str() {
                assert_eq!(
                    Some(actual),
                    fields.field(other).unwrap().member_type(),
                    "{name}: canonical option identity"
                );
            }
        } else {
            let census = admitted.model_operations(&holder, 100_000).unwrap();
            let operation = census
                .operations()
                .iter()
                .find(|op| Some(op.name()) == case["operation"].as_str())
                .unwrap();
            let parameters = case["expected"]["parameters"].as_array().unwrap();
            assert_eq!(operation.parameters().len(), parameters.len(), "{name}");
            for (actual, want) in operation.parameters().iter().zip(parameters) {
                assert_eq!(
                    actual.member_type(),
                    Some(&expected(want, &unit, identity)),
                    "{name}"
                );
            }
            assert_eq!(
                operation.result(),
                Some(Some(&expected(
                    &case["expected"]["result"],
                    &unit,
                    identity
                ))),
                "{name}"
            );
        }
        eprintln!("{name}: PASS");
    }
    for i in 1..=8 {
        assert!(seen.contains(format!("OP-{i:02}").as_str()));
    }
}

#[trace("TC-465", "FR-356-AC-1", "FR-356-AC-3", "FR-356-AC-4", "FR-356-AC-5")]
#[test]
fn published_option_intake_rows_match_exact_refusals() {
    let vectors = vectors();
    let mut seen = BTreeSet::new();
    let mut mismatches = BTreeMap::new();
    for case in vectors["intake_cases"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        assert!(seen.insert(name));
        let mut projection = vectors["domain_package"].clone();
        for patch in case["patch"].as_array().unwrap() {
            apply(&mut projection, patch);
        }
        let input = document(&projection);
        let result = read(&input);
        if case["expected"]["admitted"] == true {
            if !matches!(result, CheckedPackageV2ReadResult::Admitted(_)) {
                mismatches.insert(name, format!("expected admission, got {result:?}"));
            }
        } else if let CheckedPackageV2ReadResult::Refused(refusal) = result {
            let expected = &case["expected"]["refused"];
            let code = super::code_word(refusal.code);
            let cause = refusal.cause.map(super::cause_word);
            let at = refusal
                .document_pointer
                .as_ref()
                .map(|pointer| projected_pointer(&input, pointer.as_str()));
            if Some(code) != expected["code"].as_str()
                || cause != expected["cause"].as_str()
                || at.as_deref() != expected["at"].as_str()
            {
                mismatches.insert(
                    name,
                    format!("expected {expected}, got {code}/{cause:?} at {at:?} ({refusal:?})"),
                );
            }
        } else {
            mismatches.insert(name, format!("expected refusal, got {result:?}"));
        }
        eprintln!(
            "{name}: {}",
            if mismatches.contains_key(name) {
                "MISMATCH"
            } else {
                "PASS"
            }
        );
    }
    for i in 9..=41 {
        assert!(seen.contains(format!("OP-{i:02}").as_str()));
    }
    assert!(
        mismatches.is_empty(),
        "option cross-reader mismatches: {mismatches:#?}"
    );
}
