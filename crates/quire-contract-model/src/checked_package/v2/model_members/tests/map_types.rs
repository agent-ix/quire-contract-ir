// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! A package map type declares its key type and its entry-count bound
//! (FR-154 "Map types", FR-357). The owner's published `package_map_types`
//! cases are replayed through the selected-document reader; only the adapter
//! from the published type definitions to Semantic IR rows lives here.

use super::*;
use crate::checked_package::shared::JsonPointer;
use ix_trace_rs::trace;
use std::path::PathBuf;

const SPECIFICATION_DIR: &str = "QUIRE_SPECIFICATION_DIR";
const VECTORS: &str = "proposals/checked-package-v2/map-type-vectors.json";
const MAP: &str = "ix://acme/maps/M";

fn vectors() -> Value {
    let root = std::env::var(SPECIFICATION_DIR)
        .unwrap_or_else(|_| panic!("{SPECIFICATION_DIR} names a quire-specification checkout"));
    let path = PathBuf::from(root).join(VECTORS);
    let bytes = std::fs::read(&path).unwrap_or_else(|why| panic!("{}: {why}", path.display()));
    serde_json::from_slice::<Value>(&bytes).expect("published map vectors")["package_map_types"]
        .clone()
}

fn apply(document: &mut Value, patch: &Value) {
    let path = patch["path"].as_str().expect("patch path");
    let (parent, name) = path.rsplit_once('/').expect("patch member");
    let target = document.pointer_mut(parent).expect("patch parent");
    match (patch["op"].as_str().expect("op"), target) {
        ("add", Value::Object(members)) => {
            members.insert(name.into(), patch["value"].clone());
        }
        ("replace", Value::Object(members)) => {
            *members.get_mut(name).expect("member") = patch["value"].clone();
        }
        ("replace", Value::Array(items)) => {
            items[name.parse::<usize>().expect("index")] = patch["value"].clone();
        }
        ("remove", Value::Object(members)) => {
            members.remove(name).expect("member");
        }
        (op, _) => panic!("unsupported published patch {op}"),
    }
}

/// One published type definition as a Semantic IR row.
fn row(identity: &str, definition: &Value) -> Value {
    let kind = definition["kind"].as_str().expect("kind");
    let module = |name: &str| json!({"module": "acme/maps", "name": name});
    let slot = |owner: &str, member: &Value| {
        field(
            owner,
            member["name"].as_str().expect("name"),
            member["typeRef"].as_str().expect("typeRef"),
        )
    };
    match kind {
        "record" | "object" => {
            let fields: Vec<Value> = definition["fields"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|member| slot(identity, member))
                .collect();
            let mut declared = object_type(identity, &[], fields);
            declared["kind"] = module(if kind == "object" { "entity" } else { "record" });
            declared
        }
        "value" => {
            let mut constraints = Vec::new();
            for keyword in ["min", "max"] {
                if let Some(bound) = definition.get(keyword) {
                    constraints.push(json!({"keyword": keyword, "operands": {"value": bound}}));
                }
            }
            json!({"identity": identity, "displayName": identity, "kind": module("value"),
                "roles": [], "extensions": [], "unknownPolicy": "reject",
                "scalar": definition["scalar"], "constraints": constraints})
        }
        "variant" => {
            let cases: Vec<Value> = definition["variants"]
                .as_array()
                .expect("variants")
                .iter()
                .map(|case| {
                    let mut declared = json!({"identity": format!("{identity}/{}", case["name"].as_str().expect("case")),
                        "name": case["name"]});
                    if let Some(payload) = case.get("payloadType") {
                        declared["payloadType"] = payload.clone();
                    }
                    declared
                })
                .collect();
            json!({"identity": identity, "kind": module("variant"), "variants": cases})
        }
        "tuple" => {
            let positions: Vec<Value> = definition["positions"]
                .as_array()
                .expect("positions")
                .iter()
                .enumerate()
                .map(|(at, position)| {
                    json!({"identity": format!("{identity}/p{at}"), "typeRef": position["typeRef"]})
                })
                .collect();
            json!({"identity": identity, "kind": module("tuple"), "positions": positions})
        }
        "namespace" => json!({"identity": identity, "kind": module("namespace"),
            "members": [], "vocabulary": []}),
        "alias" | "sequence" | "option" | "reference" | "map" => {
            let mut declared = definition.clone();
            declared
                .as_object_mut()
                .expect("definition")
                .remove("identity");
            declared["identity"] = json!(identity);
            declared
        }
        other => panic!("new published kind needs its adapter: {other}"),
    }
}

fn selected(package: &Value) -> Value {
    let types: Vec<Value> = package["types"]
        .as_array()
        .expect("types")
        .iter()
        .map(|definition| {
            row(
                definition["identity"].as_str().expect("identity"),
                definition,
            )
        })
        .collect();
    let construct = |name: &str, meaning: &str| {
        json!({"kind": {"module": "acme/maps", "name": name},
            "construct": {"meaning": format!("quire.meaning.model.{meaning}/v1")}})
    };
    json!({
        "contractVersion": "2.0.0",
        "package": {"identity": "acme/maps", "version": "1.0.0"},
        "constructs": [
            construct("entity", "object-type"),
            construct("record", "record-value-type"),
            construct("value", "value-type"),
            construct("variant", "variant-type"),
            construct("tuple", "tuple-value-type"),
            construct("namespace", "namespace"),
        ],
        "types": types,
    })
}

fn unit_refs(package: &Value) -> UnitRefs {
    let node = |name: &str| CheckedNodeId {
        package: "acme/maps".into(),
        path: vec![
            CheckedNodePathSegment::Name("scalar_type".into()),
            CheckedNodePathSegment::Name("unit".into()),
            CheckedNodePathSegment::Name(name.into()),
        ],
        ordinal: 0,
    };
    unit_refs_of(
        package["units"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|unit| {
                let name = unit.as_str().expect("unit");
                (
                    Box::from(name),
                    node(name.rsplit('/').next().expect("name")),
                )
            })
            .collect(),
    )
}

/// What reading one package yields: the model, or the located refusal.
fn read_package(package: &Value) -> Result<DomainModel, Box<SelectionRefusal>> {
    let mut meter = WorkMeter::new(u64::MAX);
    match read_semantic_ir(
        view_of(&selected(package)),
        "checked",
        &mut Budget::new(&mut meter, 0, BYTES),
        &unit_refs(package),
    ) {
        Ok(model) => Ok(model),
        Err(SelectionFailure::Refused(refusal)) => Err(refusal),
        Err(_) => panic!("no limit is reached"),
    }
}

fn pointer(position: usize, member: &str) -> JsonPointer {
    JsonPointer::root().key("types").index(position).key(member)
}

fn word(code: CheckedPackageRefusalCode) -> &'static str {
    match code {
        CheckedPackageRefusalCode::IllTyped => "ill_typed",
        CheckedPackageRefusalCode::InvalidModelBinding => "invalid_model_binding",
        CheckedPackageRefusalCode::MissingDeclaration => "missing_declaration",
        other => panic!("unexpected code {other:?}"),
    }
}

fn cause_word(cause: CheckedPackageRefusalCause) -> &'static str {
    match cause {
        CheckedPackageRefusalCause::OperatorIneligible => "operator-ineligible",
        CheckedPackageRefusalCause::MalformedDeclaration => "malformed-declaration",
        CheckedPackageRefusalCause::MissingName => "missing-name",
        CheckedPackageRefusalCause::UnpreservedModelMeaning => "unpreserved-model-meaning",
        other => panic!("unexpected cause {other:?}"),
    }
}

/// The FR-322 row the published case describes for a field typed `M`.
fn reads(model: &DomainModel) -> Value {
    let type_ref = SelectedTypeRef::Declared(MAP.into());
    let mut meter = WorkMeter::new(u64::MAX);
    let rendered = model
        .clock_type_name_in(&type_ref, &mut Budget::new(&mut meter, 0, BYTES))
        .expect("no limit is reached")
        .expect("a rendered map");
    let bound = model.map_bound(&type_ref).expect("a map bound");
    json!({"rendered": rendered, "min": bound.lower.to_string(),
        "max": bound.upper.map_or_else(|| "unbounded".to_owned(), |upper| upper.to_string())})
}

/// Every published case: the admitted rows' key rendering and bound, and each
/// refusal's exact code, cause, declaration and member.
#[trace("TC-473", "FR-357-AC-23", "FR-357-AC-24")]
#[test]
fn published_package_map_types_match_the_reader() {
    let vectors = vectors();
    let cases = vectors["cases"].as_array().expect("cases");
    assert_eq!(cases.len(), 34);
    for case in cases {
        let id = case["id"].as_str().expect("id");
        let mut package = vectors["base"].clone();
        for patch in case["patch"].as_array().expect("patch") {
            apply(&mut package, patch);
        }
        let expected = &case["expected"];
        let result = read_package(&package);
        if let Some(refused) = expected.get("refused") {
            let refusal = *result.err().unwrap_or_else(|| panic!("{id}: must refuse"));
            let at = refused["at"].as_str().expect("at");
            let member = refused["path"]
                .as_str()
                .and_then(|path| path.strip_prefix("TypeDefinition."))
                .expect("member");
            let position = package["types"]
                .as_array()
                .expect("types")
                .iter()
                .position(|definition| definition["identity"] == at)
                .expect("declaration");
            assert_eq!(word(refusal.refusal.code), refused["code"], "{id}");
            assert_eq!(cause_word(refusal.refusal.cause), refused["cause"], "{id}");
            assert_eq!(
                refusal.declaration_identity.as_deref(),
                Some(at),
                "{id}: declaration"
            );
            assert_eq!(
                refusal.document_pointer,
                Some(pointer(position, member)),
                "{id}: member"
            );
        } else {
            let model = result.unwrap_or_else(|why| panic!("{id}: must admit: {why:?}"));
            assert_eq!(reads(&model), expected["reads"], "{id}");
        }
    }
}

/// The key type a package map retains is the resolved key, and the key
/// spelling the rendering uses is the authored reference.
#[trace("TC-473", "FR-357-AC-23")]
#[test]
fn package_map_retains_its_key_type_and_bound() {
    let vectors = vectors();
    let model = read_package(&vectors["base"]).expect("the base package");
    let Some(SelectedTypeDefinition::Map(map)) = model.collection_types.get(MAP) else {
        panic!("a map definition");
    };
    assert_eq!(map.keys, SelectedTypeRef::Native(NativeTypeRef::Text));
    assert_eq!(
        map.values,
        SelectedTypeRef::Declared("ix://acme/maps/Timer".into())
    );
    assert_eq!(
        map.bound,
        MapBound {
            lower: 0,
            upper: Some(2)
        }
    );
    let mut aliased = vectors["base"].clone();
    apply(
        &mut aliased,
        &json!({"op": "replace", "path": "/types/13/keys", "value": "ix://acme/maps/Key"}),
    );
    let model = read_package(&aliased).expect("an alias of Text");
    let Some(SelectedTypeDefinition::Map(map)) = model.collection_types.get(MAP) else {
        panic!("a map definition");
    };
    assert_eq!(map.keys, SelectedTypeRef::Native(NativeTypeRef::Text));
    assert_eq!(map.keys_name.as_ref(), "ix://acme/maps/Key");
}
