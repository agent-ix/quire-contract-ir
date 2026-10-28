// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Catalogue-wide admission differential for the IR-225 member-result rule.

use crate::support::checked_package::{
    add_family_reference_target, application_node_key, canonical, evidence_for, family_key,
    node_id, nominal_fixture_members, nominal_package, rebuild_source_map, refresh_identity,
    settle, structural_key,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageRefusalCause,
    CheckedPackageRefusalCode, CheckedPackageV2, CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

fn node(package: &mut Value, key: &str, tag: &str, form: &str, ty: &str, body: Value) {
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(json!({
            "node_id": node_id(key), "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": tag, "semantic_form": form, "semantic_type": node_id(ty),
            "dependencies": [], "occurrences": [{"role": "expression", "ordinal": 0}], "body": body
        }));
}

struct Fixture {
    package: Value,
    unit: String,
    boolean: String,
    integer: String,
    rational: String,
    text: String,
    left: String,
}

fn fixture() -> Fixture {
    let members = nominal_fixture_members();
    let unit = members
        .iter()
        .find(|(p, _)| p["version"] == "quire.unit-node/v1")
        .expect("unit")
        .1
        .clone();
    let mut package = nominal_package(&members);
    let empty = json!({"term": "aggregate", "members": []});
    let scalar = |form| structural_key("scalar_type", form, None, &empty);
    let boolean = scalar("boolean");
    let integer = scalar("integer");
    let rational = scalar("rational");
    let text = scalar("text");
    for (form, key) in [
        ("boolean", &boolean),
        ("integer", &integer),
        ("rational", &rational),
        ("text", &text),
    ] {
        node(&mut package, key, "scalar_type", form, key, empty.clone());
    }
    let body = json!({"term": "aggregate", "members": [
        {"term": "binding", "name": "name", "value":
            {"term": "literal", "type": node_id(&text), "value_kind": "text", "value": "left"}},
        {"term": "binding", "name": "level", "value":
            {"term": "literal", "type": node_id(&integer), "value_kind": "integer", "value": "0"}}
    ]});
    let left = structural_key("value", "parameter", Some(&unit), &body);
    node(&mut package, &left, "value", "parameter", &unit, body);
    rebuild_source_map(&mut package);
    refresh_identity(&mut package);
    Fixture {
        package,
        unit,
        boolean,
        integer,
        rational,
        text,
        left,
    }
}

fn reference(key: &str) -> Value {
    json!({"term": "reference", "target": node_id(key)})
}
fn integer_literal(key: &str) -> Value {
    json!({"term": "literal", "type": node_id(key), "value_kind": "integer", "value": "1"})
}
fn binder(value: Value) -> Value {
    json!({"term": "binding", "name": "item", "value": value})
}
fn composite(package: &mut Value, form: &str, member: &str) -> String {
    let body = json!({"term": "aggregate", "members": [reference(member)]});
    let key = structural_key("composite_type", form, None, &body);
    node(package, &key, "composite_type", form, &key, body);
    key
}
fn population(f: &mut Fixture, reference_type: &str) -> String {
    let set_body = json!({"term": "aggregate", "members": [reference(reference_type)]});
    let set = structural_key("composite_type", "set", None, &set_body);
    if !f.package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .any(|node| node["node_id"] == node_id(&set))
    {
        node(
            &mut f.package,
            &set,
            "composite_type",
            "set",
            &set,
            set_body,
        );
    }
    let body = json!({"term": "aggregate", "members": []});
    let ty = structural_key("bounded_domain", "model_population", Some(&set), &body);
    node(
        &mut f.package,
        &ty,
        "bounded_domain",
        "model_population",
        &set,
        body,
    );
    parameter(f, &ty, "population_a")
}
fn expression_reference(package: &mut Value, target: &str, ty: &str) -> String {
    let body = reference(target);
    let key = structural_key("expression", "reference", Some(ty), &body);
    node(package, &key, "expression", "reference", ty, body);
    key
}
fn rational_parameter(f: &mut Fixture) -> String {
    let body = json!({"term": "aggregate", "members": [
        {"term": "binding", "name": "name", "value": {"term": "literal",
            "type": node_id(&f.text), "value_kind": "text", "value": "ratio"}},
        {"term": "binding", "name": "level", "value": integer_literal(&f.integer)}]});
    let key = structural_key("value", "parameter", Some(&f.rational), &body);
    node(
        &mut f.package,
        &key,
        "value",
        "parameter",
        &f.rational,
        body,
    );
    key
}
fn scalar(f: &mut Fixture, form: &str) -> String {
    let body = json!({"term": "aggregate", "members": []});
    let key = structural_key("scalar_type", form, None, &body);
    node(&mut f.package, &key, "scalar_type", form, &key, body);
    key
}
fn parameter(f: &mut Fixture, ty: &str, name: &str) -> String {
    let body = json!({"term":"aggregate", "members":[
        {"term":"binding", "name":"name", "value":
            {"term":"literal", "type":node_id(&f.text), "value_kind":"text", "value":name}},
        {"term":"binding", "name":"level", "value":integer_literal(&f.integer)}
    ]});
    let key = structural_key("value", "parameter", Some(ty), &body);
    node(&mut f.package, &key, "value", "parameter", ty, body);
    key
}
fn ieee_law(f: &mut Fixture, position: usize) {
    let definition = json!({"authority":"agent-ix",
        "identity":"quire.value.ieee754-2019-default/v1"});
    f.package["lock"]["definition_selections"]
        .as_array_mut()
        .expect("selections")
        .push(definition.clone());
    f.package["semantic_graph"]["nodes"][position]["body"]["operation"]["laws"] =
        json!([{"role":"ieee_profile", "definition":definition}]);
    settle(&mut f.package);
}
fn numeric_literal(ty: &str, form: &str) -> Value {
    match form {
        "float32" => json!({"term":"literal", "type":node_id(ty),
            "value_kind":"float32_bits", "value":"3f800000"}),
        "float64" => json!({"term":"literal", "type":node_id(ty),
            "value_kind":"float64_bits", "value":"3ff0000000000000"}),
        _ => unreachable!(),
    }
}
fn application(
    f: &mut Fixture,
    operator: &str,
    form: &str,
    identity: &str,
    mode: Value,
    declaration: &str,
    result: &str,
    arguments: Vec<Value>,
) -> usize {
    let member = json!({"kind": "type_argument", "declaration": node_id(declaration)});
    application_with_member(f, operator, form, identity, mode, member, result, arguments)
}
fn application_with_member(
    f: &mut Fixture,
    operator: &str,
    form: &str,
    identity: &str,
    mode: Value,
    member: Value,
    result: &str,
    arguments: Vec<Value>,
) -> usize {
    let operation = json!({"identity": identity, "laws": [], "mode": mode,
        "member": member, "leaves": []});
    let body = json!({"term": "application", "operator": operator,
        "operation": operation, "result_type": node_id(result), "arguments": arguments});
    let key = application_node_key("expression", form, result, &body);
    let position = f.package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len();
    node(&mut f.package, &key, "expression", form, result, body);
    settle(&mut f.package);
    position
}
fn read(package: &Value) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(package),
    )
}
fn read_with_evidence(
    package: &Value,
    evidence: &CheckedPackageEvidence,
) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        evidence,
    )
}

fn simple_fixture(entry: &Value) -> Option<Value> {
    let identity = entry["identity"].as_str()?;
    let family = if identity.starts_with("quire.op.boolean.") {
        "boolean"
    } else if identity.starts_with("quire.op.integer.") {
        "integer"
    } else if identity.starts_with("quire.op.rational.") && identity != "quire.op.rational.narrow" {
        "rational"
    } else {
        return None;
    };
    if !entry["laws"].as_array()?.is_empty() {
        return None;
    }
    let operator = entry["operator"].as_str()?;
    if operator != "unary" && operator != "binary" {
        return None;
    }
    let mut f = fixture();
    let ty = match family {
        "boolean" => &f.boolean,
        "integer" => &f.integer,
        "rational" => &f.rational,
        _ => unreachable!(),
    }
    .clone();
    let result = if entry["result"] == "boolean" {
        f.boolean.clone()
    } else {
        ty.clone()
    };
    let argument = match family {
        "boolean" => json!({"term":"literal", "type":node_id(&ty),
            "value_kind":"boolean", "value":true}),
        "integer" => integer_literal(&ty),
        "rational" => {
            let parameter = rational_parameter(&mut f);
            reference(&parameter)
        }
        _ => unreachable!(),
    };
    let args = if operator == "unary" {
        vec![argument]
    } else {
        vec![argument.clone(), argument]
    };
    let body = json!({"term":"application", "operator":operator,
        "operation":{"identity":identity, "laws":[], "mode":null,
            "member":null, "leaves":[]},
        "result_type":node_id(&result), "arguments":args});
    let key = application_node_key("expression", operator, &result, &body);
    node(&mut f.package, &key, "expression", operator, &result, body);
    settle(&mut f.package);
    Some(f.package)
}

fn unavailable_class(identity: &str) -> &'static str {
    match identity.split('.').nth(2).expect("operation family") {
        "control" | "temporal" | "state" | "claim" | "protocol" => {
            "requires control, temporal, state, claim, or protocol context not built here"
        }
        "ieee" => "requires an IEEE profile and typed operands not built here",
        "model" => "requires selected model declarations not built here",
        _ => "requires typed operands or member fixtures not built here",
    }
}

// A fresh package is built for each identity. These are the entries for which
// the existing nominal/structural fixture helpers produce a valid source.
pub(super) fn admitted_fixture(identity: &str) -> Option<Value> {
    if identity == "quire.op.numeric.narrow" {
        return Some(crate::admitted_result_context::fixture());
    }
    if identity == "quire.op.collection.sum.quantity" {
        return Some(crate::checked_package_v2_operand_families::quantity_sum_control().0);
    }
    if matches!(
        identity,
        "quire.op.ieee.to_float32" | "quire.op.ieee.to_float64" | "quire.op.ieee.to_rational"
    ) {
        let (source, destination) = match identity {
            "quire.op.ieee.to_float32" => ("float64", "float32"),
            "quire.op.ieee.to_float64" => ("float32", "float64"),
            "quire.op.ieee.to_rational" => ("float32", "rational"),
            _ => unreachable!(),
        };
        return Some(
            crate::checked_package_v2_ieee_admission::with_ieee_conversion_source(
                identity,
                source,
                destination,
                destination,
            )
            .0,
        );
    }
    let mut f = fixture();
    let i = f.integer.clone();
    let r = f.rational.clone();
    let u = f.unit.clone();
    let b = f.boolean.clone();
    let mut set_i = || composite(&mut f.package, "set", &i);
    match identity {
        "quire.op.ieee.from_exact" | "quire.op.numeric.convert_rounding" => {
            let family = if identity == "quire.op.ieee.from_exact" {
                "float32"
            } else {
                "decimal"
            };
            let destination = scalar(&mut f, family);
            let at = application(
                &mut f,
                "convert",
                "conversion",
                identity,
                json!({"kind":"rounding", "value":"exact"}),
                &destination,
                &destination,
                vec![integer_literal(&i)],
            );
            if identity == "quire.op.ieee.from_exact" {
                ieee_law(&mut f, at);
            }
        }
        "quire.op.collection.sum.decimal"
        | "quire.op.collection.sum.float32"
        | "quire.op.collection.sum.float64" => {
            let family = identity.rsplit('.').next().expect("family");
            let destination = scalar(&mut f, family);
            let set = composite(&mut f.package, "set", &destination);
            let bound = if family == "decimal" {
                reference(&parameter(&mut f, &destination, "decimal"))
            } else {
                numeric_literal(&destination, family)
            };
            let at = application(
                &mut f,
                "collection",
                "collection",
                identity,
                json!({"kind":"rounding", "value":"exact"}),
                &destination,
                &destination,
                vec![reference(&set), binder(bound)],
            );
            if family.starts_with("float") {
                ieee_law(&mut f, at);
            }
        }
        "quire.op.collection.fold" => {
            let set = set_i();
            application(
                &mut f,
                "collection",
                "collection",
                identity,
                Value::Null,
                &i,
                &i,
                vec![
                    reference(&set),
                    binder(integer_literal(&i)),
                    integer_literal(&i),
                ],
            );
        }
        "quire.op.collection.reduce" | "quire.op.collection.sum.integer" => {
            let set = set_i();
            application(
                &mut f,
                "collection",
                "collection",
                identity,
                Value::Null,
                &i,
                &i,
                vec![reference(&set), binder(integer_literal(&i))],
            );
        }
        "quire.op.collection.count" => {
            let set = set_i();
            application(
                &mut f,
                "collection",
                "collection",
                identity,
                Value::Null,
                &i,
                &i,
                vec![
                    reference(&set),
                    binder(json!({"term":"literal", "type":node_id(&b),
                    "value_kind":"boolean", "value":true})),
                ],
            );
        }
        "quire.op.collection.sum.rational" => {
            let set = composite(&mut f.package, "set", &r);
            let parameter = rational_parameter(&mut f);
            application(
                &mut f,
                "collection",
                "collection",
                identity,
                Value::Null,
                &r,
                &r,
                vec![reference(&set), binder(reference(&parameter))],
            );
        }
        "quire.op.collection.size" => {
            let set = set_i();
            application(
                &mut f,
                "collection",
                "collection",
                identity,
                Value::Null,
                &i,
                &i,
                vec![reference(&set)],
            );
        }
        "quire.op.collection.convert" => {
            let set = set_i();
            let sequence = composite(&mut f.package, "sequence", &i);
            application(
                &mut f,
                "convert",
                "conversion",
                identity,
                Value::Null,
                &sequence,
                &sequence,
                vec![reference(&set)],
            );
        }
        "quire.op.numeric.convert" => {
            application(
                &mut f,
                "convert",
                "conversion",
                identity,
                Value::Null,
                &r,
                &r,
                vec![integer_literal(&i)],
            );
        }
        "quire.op.quantity.convert" => {
            let left = f.left.clone();
            application(
                &mut f,
                "convert",
                "conversion",
                identity,
                json!({"kind":"rounding", "value":"exact"}),
                &u,
                &u,
                vec![reference(&left)],
            );
        }
        "quire.op.rational.narrow" => {
            let parameter = rational_parameter(&mut f);
            application(
                &mut f,
                "convert",
                "conversion",
                identity,
                Value::Null,
                &r,
                &r,
                vec![reference(&parameter)],
            );
        }
        "quire.op.reference.upcast" => {
            let target = add_family_reference_target(&mut f.package);
            let reference_type = composite(&mut f.package, "reference", &target);
            let expression = expression_reference(&mut f.package, &f.left.clone(), &reference_type);
            application(
                &mut f,
                "convert",
                "conversion",
                identity,
                Value::Null,
                &reference_type,
                &reference_type,
                vec![reference(&expression)],
            );
        }
        "quire.op.model.all_instances" => {
            let target = add_family_reference_target(&mut f.package);
            let reference_type = composite(&mut f.package, "reference", &target);
            let set = composite(&mut f.package, "set", &reference_type);
            let pop = population(&mut f, &reference_type);
            application(
                &mut f,
                "query",
                "query",
                identity,
                Value::Null,
                &u,
                &set,
                vec![reference(&pop)],
            );
        }
        "quire.op.model.lookup" => {
            let target = add_family_reference_target(&mut f.package);
            let reference_type = composite(&mut f.package, "reference", &target);
            let option = composite(&mut f.package, "option", &u);
            let pop = population(&mut f, &reference_type);
            let expression = expression_reference(&mut f.package, &f.left.clone(), &reference_type);
            application(
                &mut f,
                "query",
                "query",
                identity,
                json!({"kind":"absence", "value":"option"}),
                &u,
                &option,
                vec![reference(&pop), reference(&expression)],
            );
        }
        "quire.op.tuple.project" => {
            let tuple = composite(&mut f.package, "tuple", &i);
            application_with_member(
                &mut f,
                "query",
                "query",
                identity,
                Value::Null,
                json!({"kind":"position", "declaration":node_id(&tuple), "index":0}),
                &i,
                vec![reference(&tuple)],
            );
        }
        "quire.op.collection.index" => {
            let set = set_i();
            application_with_member(
                &mut f,
                "query",
                "query",
                identity,
                Value::Null,
                json!({"kind":"element", "declaration":node_id(&set)}),
                &i,
                vec![
                    reference(&set),
                    json!({"term":"literal", "type":node_id(&i),
                    "value_kind":"integer", "value":"0"}),
                ],
            );
        }
        _ => return None,
    }
    Some(f.package)
}

#[trace("TC-048", "FR-038-AC-251")]
#[test]
fn tc_048_every_catalogue_entry_retains_its_constructible_admission() {
    let catalog: Value = serde_json::from_str(
        quire_verification_contracts::operation_catalog::CHECKED_OPERATION_CATALOG_V1,
    )
    .expect("reader's catalogue is JSON");
    let entries = catalog["operations"].as_array().expect("operations");
    let mut admitted = BTreeSet::new();
    let mut unavailable = BTreeSet::new();
    let mut unavailable_classes = BTreeMap::new();
    for entry in entries {
        let identity = entry["identity"].as_str().expect("identity");
        let external = match identity {
            "quire.op.record.project" => {
                Some(crate::checked_package_v2_model_fields::record_project_control())
            }
            "quire.op.model.navigate" | "quire.op.model.dispatch_call" => {
                Some(crate::checked_package_v2_model_members::model_member_control(identity))
            }
            _ => None,
        };
        if let Some((package, evidence)) = external {
            let result = read_with_evidence(&package, &evidence);
            assert!(
                matches!(result, CheckedPackageV2ReadResult::Admitted(_)),
                "{identity}: {result:?}"
            );
            admitted.insert(identity.to_owned());
        } else if let Some(package) = admitted_fixture(identity).or_else(|| simple_fixture(entry)) {
            assert!(
                matches!(read(&package), CheckedPackageV2ReadResult::Admitted(_)),
                "{identity}: {:?}",
                read(&package)
            );
            admitted.insert(identity.to_owned());
        } else {
            assert!(
                entry["result"] != "member" && entry["member"] != "type_argument",
                "member-result or type-argument lookalike {identity} needs an admitted control"
            );
            unavailable.insert(identity.to_owned());
            *unavailable_classes
                .entry(unavailable_class(identity))
                .or_insert(0usize) += 1;
        }
    }
    assert_eq!(entries.len(), 160, "catalogue entry count changed");
    assert_eq!(
        admitted.len(),
        56,
        "56 of 160 entries have behavioral controls"
    );
    assert_eq!(unavailable.len(), 104, "unavailable fixture count changed");
    assert_eq!(
        unavailable_classes,
        BTreeMap::from([
            (
                "requires control, temporal, state, claim, or protocol context not built here",
                25
            ),
            (
                "requires an IEEE profile and typed operands not built here",
                15
            ),
            ("requires selected model declarations not built here", 3),
            (
                "requires typed operands or member fixtures not built here",
                61
            ),
        ])
    );
    assert_eq!(
        unavailable,
        expected_unavailable(),
        "an entry needs an admitted fixture or an explicit gap reason"
    );

    // Each selected entry has an admitted witness and a mismatched-result
    // refusal. This catches selection and family regressions across the catalogue.
    for entry in entries
        .iter()
        .filter(|entry| entry["result"] == "member" && entry["member"] == "type_argument")
    {
        let identity = entry["identity"].as_str().expect("identity");
        let mut package = admitted_fixture(identity).expect("admitted control");
        let position = package["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .len()
            - 1;
        let other = match identity {
            "quire.op.ieee.to_float32" => "float64",
            "quire.op.ieee.to_float64" | "quire.op.ieee.to_rational" => "float32",
            "quire.op.collection.fold"
            | "quire.op.collection.reduce"
            | "quire.op.collection.count"
            | "quire.op.collection.sum.integer"
            | "quire.op.collection.size" => "rational",
            _ => "integer",
        };
        let key = if identity == "quire.op.numeric.narrow" {
            family_key("a3a3")
        } else {
            structural_key(
                "scalar_type",
                other,
                None,
                &json!({"term":"aggregate", "members":[]}),
            )
        };
        package["semantic_graph"]["nodes"][position]["body"]["result_type"] = node_id(&key);
        package["semantic_graph"]["nodes"][position]["semantic_type"] = node_id(&key);
        settle(&mut package);
        assert_result_refusal(&package, position, identity);
    }
    for source in ["float32", "float64"] {
        for (member, result) in [(source, "rational"), ("rational", source), (source, source)] {
            let (package, position) =
                crate::checked_package_v2_ieee_admission::with_ieee_conversion_source(
                    "quire.op.ieee.to_rational",
                    source,
                    member,
                    result,
                );
            assert_result_refusal(&package, position, "quire.op.ieee.to_rational");
        }
    }
}

fn assert_result_refusal(package: &Value, position: usize, identity: &str) {
    let CheckedPackageV2ReadResult::Refused(refusal) = read(package) else {
        panic!(
            "{identity}: expected result refusal, got {:?}",
            read(package)
        );
    };
    assert_eq!(
        (
            refusal.code,
            refusal.cause,
            refusal.path.as_ref().map(ToString::to_string),
            serde_json::to_value(refusal.locus).expect("locus")
        ),
        (
            CheckedPackageRefusalCode::IllTyped,
            Some(CheckedPackageRefusalCause::OperatorIneligible),
            Some(format!("/semantic_graph/nodes/{position}/body/result_type")),
            package["semantic_graph"]["nodes"][position]["node_id"].clone()
        ),
        "{identity}"
    );
}

#[trace("TC-048", "FR-038-AC-69", "FR-038-AC-251")]
#[test]
fn tc_048_member_kind_mismatch_precedes_member_result_check() {
    let mut package = admitted_fixture("quire.op.collection.fold").expect("fold control");
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
        - 1;
    package["semantic_graph"]["nodes"][position]["body"]["operation"]["member"] = json!({"kind":"position", "declaration":node_id(&structural_key(
            "scalar_type", "integer", None, &json!({"term":"aggregate", "members":[]}))),
            "index":0});
    let wrong = structural_key(
        "scalar_type",
        "rational",
        None,
        &json!({"term":"aggregate", "members":[]}),
    );
    package["semantic_graph"]["nodes"][position]["body"]["result_type"] = node_id(&wrong);
    package["semantic_graph"]["nodes"][position]["semantic_type"] = node_id(&wrong);
    settle(&mut package);
    let CheckedPackageV2ReadResult::Refused(refusal) = read(&package) else {
        panic!("wrong member kind must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidPackage);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::OperationMemberMismatch)
    );
    assert_eq!(
        refusal.path.as_ref().map(ToString::to_string),
        Some(format!(
            "/semantic_graph/nodes/{position}/body/operation/member"
        ))
    );
    assert_eq!(
        serde_json::to_value(refusal.locus).expect("locus"),
        package["semantic_graph"]["nodes"][position]["node_id"]
    );
}

fn expected_unavailable() -> BTreeSet<String> {
    // Every identity here is outside the member-result selector. The exact
    // inventory ensures a new or newly selected entry needs an admission control.
    [
        "quire.op.control.case",
        "quire.op.string.eq",
        "quire.op.string.ne",
        "quire.op.temporal.always",
        "quire.op.temporal.and",
        "quire.op.temporal.eventually",
        "quire.op.temporal.fair",
        "quire.op.temporal.false",
        "quire.op.temporal.historically",
        "quire.op.temporal.holds",
        "quire.op.temporal.implies",
        "quire.op.temporal.not",
        "quire.op.temporal.once",
        "quire.op.temporal.or",
        "quire.op.temporal.release",
        "quire.op.temporal.since",
        "quire.op.temporal.triggered",
        "quire.op.temporal.true",
        "quire.op.temporal.until",
        "quire.op.timestamp.eq",
        "quire.op.timestamp.ne",
        "quire.op.uuid.eq",
        "quire.op.uuid.ne",
        "quire.op.claim.clause",
        "quire.op.collection.bag",
        "quire.op.collection.contains",
        "quire.op.collection.exists",
        "quire.op.collection.filter",
        "quire.op.collection.flat_map",
        "quire.op.collection.flatten",
        "quire.op.collection.forall",
        "quire.op.collection.map",
        "quire.op.collection.ordered_set",
        "quire.op.collection.sequence",
        "quire.op.collection.set",
        "quire.op.control.if",
        "quire.op.control.let",
        "quire.op.decimal.add",
        "quire.op.decimal.div",
        "quire.op.decimal.eq",
        "quire.op.decimal.ge",
        "quire.op.decimal.gt",
        "quire.op.decimal.le",
        "quire.op.decimal.lt",
        "quire.op.decimal.mul",
        "quire.op.decimal.ne",
        "quire.op.decimal.negate",
        "quire.op.decimal.sub",
        "quire.op.enum.eq",
        "quire.op.enum.ge",
        "quire.op.enum.gt",
        "quire.op.enum.le",
        "quire.op.enum.lt",
        "quire.op.enum.ne",
        "quire.op.function.call",
        "quire.op.ieee.bit_identical",
        "quire.op.ieee.float32.add",
        "quire.op.ieee.float32.div",
        "quire.op.ieee.float32.fma",
        "quire.op.ieee.float32.mul",
        "quire.op.ieee.float32.sqrt",
        "quire.op.ieee.float32.sub",
        "quire.op.ieee.float64.add",
        "quire.op.ieee.float64.div",
        "quire.op.ieee.float64.fma",
        "quire.op.ieee.float64.mul",
        "quire.op.ieee.float64.sqrt",
        "quire.op.ieee.float64.sub",
        "quire.op.ieee.numeric_equal",
        "quire.op.ieee.total_order",
        "quire.op.integer.div",
        "quire.op.integer.rem",
        "quire.op.model.deref",
        "quire.op.model.reaches",
        "quire.op.model.reaches_field",
        "quire.op.option.present",
        "quire.op.option.value",
        "quire.op.protocol.control",
        "quire.op.quantity.add",
        "quire.op.quantity.div",
        "quire.op.quantity.eq",
        "quire.op.quantity.ge",
        "quire.op.quantity.gt",
        "quire.op.quantity.le",
        "quire.op.quantity.lt",
        "quire.op.quantity.mul",
        "quire.op.quantity.ne",
        "quire.op.quantity.pow",
        "quire.op.quantity.sub",
        "quire.op.reference.eq",
        "quire.op.reference.ne",
        "quire.op.state.clause",
        "quire.op.state.pre",
        "quire.op.state.transition",
        "quire.op.structural.eq",
        "quire.op.structural.ne",
        "quire.op.temporal.clause",
        "quire.op.text.eq",
        "quire.op.text.ge",
        "quire.op.text.gt",
        "quire.op.text.le",
        "quire.op.text.lt",
        "quire.op.text.ne",
        "quire.op.text.size",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}
