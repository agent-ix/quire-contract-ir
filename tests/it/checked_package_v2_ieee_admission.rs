// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Reader-boundary witnesses for IEEE literal/type agreement and cross-width
//! conversion results (FR-038-AC-222/223). Every package carries freshly
//! derived node and package IDs.

use crate::support::checked_package::{
    application_node_key, boolean_key, canonical, evidence_for, node_id, pointer,
    rebuild_source_map, refresh_identity, rekey_application_node, structural_key, v2_all_families,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedPackageReadLimits, CheckedPackageRefusalCause, CheckedPackageRefusalCode,
    CheckedPackageV2, CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};

fn read(package: &Value) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(package),
    )
}

fn with_ieee_literal(kind: &str, declared_family: &str) -> (Value, usize) {
    let mut package = v2_all_families();
    let empty = json!({"term": "aggregate", "members": []});
    let type_node = |family: &str| {
        let digest = structural_key("scalar_type", family, None, &empty);
        json!({
            "node_id": node_id(&digest),
            "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": "scalar_type", "semantic_form": family,
            "semantic_type": node_id(&digest), "dependencies": [],
            "occurrences": [{"role": "generated", "ordinal": 0}],
            "body": empty,
        })
    };
    let float32 = type_node("float32");
    let float64 = type_node("float64");
    let float32_id = float32["node_id"].clone();
    let float64_id = float64["node_id"].clone();
    let semantic_type = if kind == "float32_bits" {
        float32_id.clone()
    } else {
        float64_id.clone()
    };
    let declared = if declared_family == "float32" {
        float32_id
    } else {
        float64_id
    };
    let body = json!({
        "term": "literal", "type": declared.clone(), "value_kind": kind,
        "value": if kind == "float32_bits" { "3f800000" } else { "3ff0000000000000" },
    });
    let literal_digest = structural_key(
        "value",
        "literal",
        Some(semantic_type["digest"].as_str().expect("type digest")),
        &body,
    );
    let literal = json!({
        "node_id": node_id(&literal_digest),
        "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": "value", "semantic_form": "literal",
        "semantic_type": semantic_type,
        "dependencies": [declared],
        "occurrences": [{"role": "generated", "ordinal": 0}],
        "body": body,
    });
    let nodes = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    nodes.push(float32);
    nodes.push(float64);
    let position = nodes.len();
    nodes.push(literal);
    rebuild_source_map(&mut package);
    refresh_identity(&mut package);
    (package, position)
}

/// Trace: TC-048, FR-038-AC-222.
#[trace("TC-048", "FR-038-AC-222")]
#[test]
fn tc_048_ieee_bits_literal_requires_its_own_declared_family() {
    for (kind, correct, wrong) in [
        ("float32_bits", "float32", "float64"),
        ("float64_bits", "float64", "float32"),
    ] {
        let (positive, _) = with_ieee_literal(kind, correct);
        match read(&positive) {
            CheckedPackageV2ReadResult::Admitted(_) => {}
            other => panic!("{kind} with its own declared family must admit, got {other:?}"),
        }
        let (adverse, position) = with_ieee_literal(kind, wrong);
        let CheckedPackageV2ReadResult::Refused(refusal) = read(&adverse) else {
            panic!("{kind} with the opposite declared family must refuse");
        };
        assert_eq!(refusal.code, CheckedPackageRefusalCode::IllTyped);
        assert_eq!(
            refusal.cause,
            Some(CheckedPackageRefusalCause::OperatorIneligible)
        );
        assert_eq!(
            refusal.path,
            Some(pointer(&format!(
                "/semantic_graph/nodes/{position}/body/type"
            )))
        );
        assert_eq!(
            serde_json::to_value(refusal.locus).expect("locus"),
            adverse["semantic_graph"]["nodes"][position]["node_id"]
        );
    }
}

fn with_ieee_float_operand(kind: &str, declared_family: &str) -> (Value, usize) {
    let (mut package, _) = with_ieee_literal("float32_bits", "float32");
    let declared = node_id(&structural_key(
        "scalar_type",
        declared_family,
        None,
        &json!({"term": "aggregate", "members": []}),
    ));
    let law = json!({
        "authority": "agent-ix", "identity": "quire.value.ieee754-2019-default/v1",
    });
    package["lock"]["definition_selections"] = json!([law.clone()]);
    let literal = |value_kind: &str| {
        json!({
            "term": "literal", "type": declared.clone(),
            "value_kind": value_kind,
            "value": if value_kind == "float32_bits" {"3f800000"} else {"3ff0000000000000"},
        })
    };
    let body = json!({
        "term": "application", "operator": "call",
        "operation": {
            "identity": "quire.op.ieee.numeric_equal",
            "laws": [{"role": "ieee_profile", "definition": law}],
            "mode": null, "member": null, "leaves": [],
        },
        "result_type": node_id(&boolean_key()),
        "arguments": [literal(kind), literal(if declared_family == "float32" {
            "float32_bits"
        } else {
            "float64_bits"
        })],
    });
    let digest = application_node_key("expression", "call", &boolean_key(), &body);
    let application = json!({
        "node_id": node_id(&digest),
        "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": "expression", "semantic_form": "call",
        "semantic_type": node_id(&boolean_key()), "dependencies": [],
        "occurrences": [{"role": "generated", "ordinal": 0}],
        "body": body,
    });
    let nodes = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    let position = nodes.len();
    nodes.push(application);
    rebuild_source_map(&mut package);
    refresh_identity(&mut package);
    (package, position)
}

/// Trace: TC-048, FR-038-AC-222. `ieee_float` accepts either declared
/// operand family, so only the literal's own width rule separates the pair.
#[trace("TC-048", "FR-038-AC-222")]
#[test]
fn tc_048_ieee_literal_width_refuses_before_an_accepting_operand_family() {
    for (kind, declared_family, matching_kind) in [
        ("float32_bits", "float64", "float64_bits"),
        ("float64_bits", "float32", "float32_bits"),
    ] {
        let (positive, _) = with_ieee_float_operand(matching_kind, declared_family);
        match read(&positive) {
            CheckedPackageV2ReadResult::Admitted(_) => {}
            other => panic!("{matching_kind} control must admit, got {other:?}"),
        }
        let (adverse, position) = with_ieee_float_operand(kind, declared_family);
        let CheckedPackageV2ReadResult::Refused(refusal) = read(&adverse) else {
            panic!("{kind} at {declared_family} must refuse at its own type");
        };
        assert_eq!(refusal.code, CheckedPackageRefusalCode::IllTyped);
        assert_eq!(
            refusal.cause,
            Some(CheckedPackageRefusalCause::OperatorIneligible)
        );
        assert_eq!(
            refusal.path,
            Some(pointer(&format!(
                "/semantic_graph/nodes/{position}/body/arguments/0/type"
            )))
        );
        assert_eq!(
            serde_json::to_value(refusal.locus).expect("locus"),
            adverse["semantic_graph"]["nodes"][position]["node_id"]
        );
    }
}

fn with_ieee_conversion(
    identity: &str,
    member_family: &str,
    result_family: &str,
) -> (Value, usize) {
    let (mut package, _) = with_ieee_literal("float32_bits", "float32");
    let type_id = |family: &str| {
        node_id(&structural_key(
            "scalar_type",
            family,
            None,
            &json!({"term": "aggregate", "members": []}),
        ))
    };
    let float32 = type_id("float32");
    let float64 = type_id("float64");
    let source = if identity.ends_with("to_float32") {
        float64.clone()
    } else {
        float32.clone()
    };
    let destination = type_id(member_family);
    let result = type_id(result_family);
    let ieee_definition = json!({
        "authority": "agent-ix", "identity": "quire.value.ieee754-2019-default/v1",
    });
    package["lock"]["definition_selections"] = json!([ieee_definition.clone()]);
    let operation = json!({
        "identity": identity,
        "laws": [{"role": "ieee_profile", "definition": ieee_definition}],
        "mode": if identity.ends_with("to_float32") {
            json!({"kind": "rounding", "value": "exact"})
        } else {
            Value::Null
        },
        "member": {"kind": "type_argument", "declaration": destination.clone()},
        "leaves": [],
    });
    let body = json!({
        "term": "application", "operator": "convert", "operation": operation,
        "result_type": result.clone(),
        "arguments": [{
            "term": "literal", "type": source,
            "value_kind": if identity.ends_with("to_float32") {"float64_bits"} else {"float32_bits"},
            "value": if identity.ends_with("to_float32") {"3ff0000000000000"} else {"3f800000"},
        }],
    });
    let digest = application_node_key(
        "expression",
        "call",
        result["digest"].as_str().expect("result digest"),
        &body,
    );
    let application = json!({
        "node_id": node_id(&digest),
        "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": "expression", "semantic_form": "call",
        "semantic_type": result,
        "dependencies": [destination],
        "occurrences": [{"role": "generated", "ordinal": 0}],
        "body": body,
    });
    let nodes = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    let position = nodes.len();
    nodes.push(application);
    rebuild_source_map(&mut package);
    refresh_identity(&mut package);
    (package, position)
}

/// Trace: TC-048, FR-038-AC-223.
#[trace("TC-048", "FR-038-AC-223")]
#[test]
fn tc_048_cross_width_ieee_result_is_the_type_argument_of_the_catalogued_family() {
    for (identity, destination, other) in [
        ("quire.op.ieee.to_float32", "float32", "float64"),
        ("quire.op.ieee.to_float64", "float64", "float32"),
    ] {
        let (positive, _) = with_ieee_conversion(identity, destination, destination);
        match read(&positive) {
            CheckedPackageV2ReadResult::Admitted(_) => {}
            other => panic!("{identity} must admit its advertised result type, got {other:?}"),
        }
        for (member, result) in [(destination, other), (other, other)] {
            let (adverse, position) = with_ieee_conversion(identity, member, result);
            let CheckedPackageV2ReadResult::Refused(refusal) = read(&adverse) else {
                panic!("{identity} must refuse member {member}, result {result}");
            };
            assert_eq!(refusal.code, CheckedPackageRefusalCode::IllTyped);
            assert_eq!(
                refusal.cause,
                Some(CheckedPackageRefusalCause::OperatorIneligible)
            );
            assert_eq!(
                refusal.path,
                Some(pointer(&format!(
                    "/semantic_graph/nodes/{position}/body/result_type"
                )))
            );
            assert_eq!(
                serde_json::to_value(refusal.locus).expect("locus"),
                adverse["semantic_graph"]["nodes"][position]["node_id"]
            );
        }
        let (mut same_width, position) = with_ieee_conversion(identity, destination, destination);
        let width = node_id(&structural_key(
            "scalar_type",
            destination,
            None,
            &json!({"term": "aggregate", "members": []}),
        ));
        same_width["semantic_graph"]["nodes"][position]["body"]["arguments"][0] = json!({
            "term": "literal", "type": width,
            "value_kind": if destination == "float32" {"float32_bits"} else {"float64_bits"},
            "value": if destination == "float32" {"3f800000"} else {"3ff0000000000000"},
        });
        rekey_application_node(&mut same_width, position);
        refresh_identity(&mut same_width);
        let CheckedPackageV2ReadResult::Refused(refusal) = read(&same_width) else {
            panic!("{identity} must not admit a same-width operand");
        };
        assert_eq!(refusal.code, CheckedPackageRefusalCode::IllTyped);
        assert_eq!(
            refusal.cause,
            Some(CheckedPackageRefusalCause::OperatorIneligible)
        );
        assert_eq!(
            refusal.path,
            Some(pointer(&format!(
                "/semantic_graph/nodes/{position}/body/arguments/0"
            )))
        );
        assert_eq!(
            serde_json::to_value(refusal.locus).expect("locus"),
            same_width["semantic_graph"]["nodes"][position]["node_id"]
        );
    }
}
