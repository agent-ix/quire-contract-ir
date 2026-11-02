// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-977: QSpec's operation-effect vectors against the production reader.

use super::{
    cause_word, code_word, fixture_support as fixture, fixtures_dir, proposal_dir_of, read_json,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedOperationEffectRow, CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageV2,
    CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};
use std::collections::BTreeSet;

fn vectors() -> Value {
    read_json(&proposal_dir_of(&fixtures_dir()).join("operation-effect-vectors.json"))
        .expect("current QSpec TC-977 vectors")
}

fn patch(value: &mut Value, patch: &Value) {
    let path = patch["path"].as_str().expect("patch path");
    let (parent, member) = path.rsplit_once('/').expect("patch member");
    let target = value.pointer_mut(parent).expect("published patch parent");
    match (patch["op"].as_str().expect("patch operation"), target) {
        ("add", Value::Array(items)) if member == "-" => items.push(patch["value"].clone()),
        ("add", Value::Array(items)) => {
            items.insert(member.parse().unwrap(), patch["value"].clone())
        }
        ("add", Value::Object(items)) => {
            items.insert(member.into(), patch["value"].clone());
        }
        ("replace", Value::Array(items)) => {
            items[member.parse::<usize>().unwrap()] = patch["value"].clone()
        }
        ("replace", Value::Object(items)) => {
            *items.get_mut(member).expect("replace target") = patch["value"].clone();
        }
        ("remove", Value::Array(items)) => {
            items.remove(member.parse::<usize>().unwrap());
        }
        ("remove", Value::Object(items)) => {
            assert!(items.remove(member).is_some());
        }
        _ => panic!("unsupported published patch: {patch}"),
    }
}

fn slot(identity: &str, name: &str, type_ref: &str) -> Value {
    json!({"identity":format!("{identity}/{name}"),"name":name,"typeRef":type_ref,
        "presence":"required","nullable":false,"defaultKind":"none",
        "multiplicity":{"lower":1,"upper":1,"ordered":false,"unique":true}})
}

fn expand_document(sketch: &Value) -> Value {
    let identity = sketch["identity"].as_str().expect("document identity");
    let mut types = Vec::new();
    for ty in sketch["types"].as_array().expect("sketch types") {
        let name = ty["name"].as_str().expect("type name");
        let owner = format!("ix://{identity}/{name}");
        let form = ty["form"].as_str().expect("type form");
        let kind = if form == "persistence_interface" {
            "store"
        } else {
            "entity"
        };
        let fields = ty["fields"]
            .as_array()
            .expect("fields")
            .iter()
            .map(|field| {
                slot(
                    &owner,
                    field.as_str().expect("field name"),
                    "ix://quire/native/Integer",
                )
            })
            .collect::<Vec<_>>();
        let operations = ty["operations"]
            .as_array()
            .expect("operations")
            .iter()
            .map(|operation| {
                let op_name = operation["name"].as_str().expect("operation name");
                let op_id = format!("{owner}/{op_name}");
                let params = operation["params"]
                    .as_array()
                    .expect("params")
                    .iter()
                    .map(|param| {
                        let name = param["name"].as_str().expect("parameter name");
                        let type_ref = param["type"].as_str().expect("parameter type");
                        let type_ref = if type_ref.starts_with("Reference<") {
                            format!(
                                "ix://{identity}/{}",
                                type_ref
                                    .trim_start_matches("Reference<")
                                    .trim_end_matches('>')
                            )
                        } else {
                            type_ref.to_owned()
                        };
                        slot(&op_id, name, &type_ref)
                    })
                    .collect::<Vec<_>>();
                let mut value = json!({"identity":op_id,"name":op_name,"params":params,
                "pre":[],"post":[]});
                if form != "persistence_interface" {
                    value["frame"] = operation["frame"].clone();
                }
                value
            })
            .collect::<Vec<_>>();
        let mut value = json!({"identity":owner,"displayName":name,
            "kind":{"module":identity,"name":kind},"roles":[],"constraints":[],
            "extensions":[],"unknownPolicy":"reject","supertypes":[],
            "fields":fields,"operations":operations});
        if form == "persistence_interface" {
            value["interfaceFeatures"] = json!([]);
            value["persists"] = json!([format!("ix://{identity}/Order")]);
        }
        if name == "Order"
            && sketch["relationships"]
                .as_array()
                .is_some_and(|rows| !rows.is_empty())
        {
            value["relationships"] = json!([{
                "identity":format!("ix://{identity}/relationship/Placed"),
                "category":"structural","composite":false,"direction":"source-to-target",
                "origin":{"source":{"path":"order.md","sourceIdentity":format!("ix://{identity}/spec"),"startLine":1,"startColumn":1}},
                "sourceEnd":{"type":owner,"role":"placed","multiplicity":{"lower":0,"upper":1,"ordered":false,"unique":true}},
                "targetEnd":{"type":owner,"multiplicity":{"lower":0,"upper":1,"ordered":false,"unique":true}}
            }]);
        }
        types.push(value);
    }
    json!({"contractVersion":"2.0.0","package":{"identity":identity,"version":"1.0.0"},
        "constructs":[
            {"kind":{"module":identity,"name":"entity"},"construct":{"meaning":"quire.meaning.model.object-type/v1"}},
            {"kind":{"module":identity,"name":"store"},"construct":{"meaning":"quire.meaning.model.persistence-interface/v1"}}
        ],"types":types})
}

fn normalize_relationship(value: &mut Value) {
    match value {
        Value::String(text) if text == "ix://acme/orders/Placed" => {
            *text = "ix://acme/orders/relationship/Placed".into();
        }
        Value::Array(items) => items.iter_mut().for_each(normalize_relationship),
        Value::Object(items) => items.values_mut().for_each(normalize_relationship),
        _ => {}
    }
}

fn read(case: &Value) -> CheckedPackageV2ReadResult {
    let mut case = case.clone();
    // QSpec sketches use a short relationship identifier; the selected FCD's
    // relationship grammar requires its /relationship/ segment.
    normalize_relationship(&mut case);
    let mut package = fixture::nominal_package(&[]);
    package["operation_effects"] = case["operation_effects"].clone();
    let mut evidence = CheckedPackageEvidence::new();
    for sketch in case["documents"].as_array().expect("sketch documents") {
        let identity = sketch["identity"].as_str().expect("document identity");
        let document = expand_document(sketch);
        for ty in sketch["types"].as_array().expect("types") {
            let name = ty["name"].as_str().expect("name");
            let form = ty["form"].as_str().expect("form");
            let member = format!("ix://{identity}/{name}");
            let key = fixture::structural_id(
                identity,
                vec![json!("model"), json!(form), json!(member)],
                0,
            );
            package["semantic_graph"]["nodes"].as_array_mut().unwrap().push(json!({
                "node_id":key,"schema_version":"quire.checked-semantic-graph/v2","node_tag":"model",
                "semantic_form":form,"semantic_type":key,"dependencies":[],
                "occurrences":[{"role":"type","ordinal":0}],
                "owner":{"kind":"model","identity":identity,"node":member},
                "body":{"term":"aggregate","members":[]}
            }));
        }
        if sketch["relationships"]
            .as_array()
            .is_some_and(|rows| !rows.is_empty())
        {
            let member = format!("ix://{identity}/relationship/Placed");
            let key = fixture::structural_id(
                identity,
                vec![json!("relation"), json!("relationship"), json!(member)],
                0,
            );
            package["semantic_graph"]["nodes"].as_array_mut().unwrap().push(json!({
                "node_id":key,"schema_version":"quire.checked-semantic-graph/v2","node_tag":"relation",
                "semantic_form":"relationship","semantic_type":key,"dependencies":[],
                "occurrences":[{"role":"type","ordinal":0}],
                "owner":{"kind":"model","identity":identity,"node":member},
                "body":{"term":"aggregate","members":[]}
            }));
        }
        fixture::select_model(&mut package, identity);
        evidence
            .insert_domain_package_document(identity, fixture::canonical(&document))
            .expect("one document per identity");
    }
    evidence.support_feature("quire.value.complete/v1");
    fixture::settle(&mut package);
    CheckedPackageV2::read(
        &fixture::canonical(&package),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    )
}

fn case(vectors: &Value, id: &str) -> Value {
    let base = vectors["positive"]
        .as_array()
        .expect("positive vectors")
        .iter()
        .find(|row| row["id"] == id)
        .expect("published base");
    let mut result = base.clone();
    result["operation_effects"] = result["rows"].clone();
    result
}

#[trace(
    "TC-977",
    "FR-322-AC-303",
    "FR-038-AC-406",
    "FR-038-AC-407",
    "FR-038-AC-408"
)]
#[test]
fn tc_977_published_operation_effect_vectors() {
    let vectors = vectors();
    let mut seen = BTreeSet::new();
    for positive in vectors["positive"].as_array().expect("positive vectors") {
        let id = positive["id"].as_str().expect("vector id");
        assert!(seen.insert(id), "duplicate vector {id}");
        let mut input = case(&vectors, id);
        let actual = read(&input);
        let CheckedPackageV2ReadResult::Admitted(admitted) = actual else {
            panic!("{id}: {actual:?}");
        };
        normalize_relationship(&mut input);
        let expected: Vec<CheckedOperationEffectRow> =
            serde_json::from_value(input["operation_effects"].clone()).expect("published rows");
        let observed = admitted
            .operation_effects()
            .expect("present admitted rows")
            .collect::<Vec<_>>();
        assert_eq!(observed.len(), expected.len(), "{id}");
        for (actual, expected) in observed.iter().zip(expected.iter()) {
            assert_eq!(actual.document(), expected.document.as_ref(), "{id}");
            assert_eq!(actual.declaring(), &expected.declaring, "{id}");
            assert_eq!(actual.operation(), expected.operation.as_ref(), "{id}");
            assert_eq!(actual.effect(), &expected.effect, "{id}");
            assert_eq!(actual.provenance(), &expected.provenance, "{id}");
        }
    }
    for group in ["schema_refusals", "reader_refusals"] {
        for row in vectors[group].as_array().expect("refusal vectors") {
            let id = row["id"].as_str().expect("vector id");
            assert!(seen.insert(id), "duplicate vector {id}");
            let mut input = case(&vectors, row["base"].as_str().expect("base id"));
            for mutation in row["patch"].as_array().expect("patches") {
                patch(&mut input, mutation);
            }
            let actual = read(&input);
            let CheckedPackageV2ReadResult::Refused(refusal) = actual else {
                panic!("{id}: unexpectedly admitted");
            };
            assert_eq!(
                code_word(refusal.code),
                row["expected_code"].as_str().unwrap(),
                "{id}"
            );
            if let Some(cause) = row["expected_cause"].as_str() {
                assert_eq!(refusal.cause.map(cause_word), Some(cause), "{id}");
            }
            assert_eq!(
                refusal.loci[0].path.as_ref().expect("path").to_string(),
                row["locus"],
                "{id}"
            );
        }
    }
}
