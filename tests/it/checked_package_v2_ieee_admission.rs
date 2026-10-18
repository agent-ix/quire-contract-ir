// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Reader-boundary witnesses for IEEE literal/type agreement and cross-width
//! conversion results (FR-038-AC-222/223). Every package carries freshly
//! derived node and package IDs.

use crate::support::checked_package::{
    application_node_key, boolean_key, canonical, evidence_for, node_id, pointer,
    rebuild_source_map, refresh_identity, rekey_application_node, rename_node, settle,
    structural_key, v2_all_families,
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

fn with_ieee_float_operands(
    first_kind: &str,
    second_kind: &str,
    declared_family: &str,
) -> (Value, usize) {
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
        "arguments": [literal(first_kind), literal(second_kind)],
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

fn with_ieee_float_operand(kind: &str, declared_family: &str) -> (Value, usize) {
    let matching = if declared_family == "float32" {
        "float32_bits"
    } else {
        "float64_bits"
    };
    with_ieee_float_operands(kind, matching, declared_family)
}

fn familyless_type(package: &Value, tag: &str, form: &str) -> Value {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["node_tag"] == tag && node["semantic_form"] == form)
        .unwrap_or_else(|| panic!("fixture lacks {tag}/{form}"))["node_id"]
        .clone()
}

fn assert_literal_type_refusal(package: &Value, position: usize, suffix: &str) {
    let CheckedPackageV2ReadResult::Refused(refusal) = read(package) else {
        panic!("literal at {position}{suffix} must refuse");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::IllTyped);
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::OperatorIneligible)
    );
    assert_eq!(
        refusal.path,
        Some(pointer(&format!(
            "/semantic_graph/nodes/{position}/body{suffix}/type"
        )))
    );
    assert_eq!(
        serde_json::to_value(refusal.locus).expect("locus"),
        package["semantic_graph"]["nodes"][position]["node_id"]
    );
}

/// all-families fixture is tried as a declared IEEE literal type.
#[trace("TC-048", "FR-038-AC-222")]
#[test]
fn tc_048_ieee_bits_reject_every_familyless_declared_type() {
    let shapes = [
        ("value", "literal"),
        ("value", "parameter"),
        ("model", "namespace"),
        ("model", "process"),
        ("relation", "relationship"),
        ("state", "snapshot"),
        ("state", "frame"),
        ("temporal", "temporal_clause"),
        ("protocol", "protocol_clause"),
        ("claim", "verification_claim"),
        ("correspondence", "source_locus"),
    ];
    for kind in ["float32_bits", "float64_bits"] {
        let own_family = if kind == "float32_bits" {
            "float32"
        } else {
            "float64"
        };
        for (tag, form) in shapes {
            let (mut standalone, position) = with_ieee_literal(kind, own_family);
            let declared = familyless_type(&standalone, tag, form);
            let stale = standalone["semantic_graph"]["nodes"][position]["node_id"]["digest"]
                .as_str()
                .expect("literal digest")
                .to_owned();
            standalone["semantic_graph"]["nodes"][position]["body"]["type"] = declared.clone();
            standalone["semantic_graph"]["nodes"][position]["dependencies"] =
                json!([declared.clone()]);
            let body = &standalone["semantic_graph"]["nodes"][position]["body"];
            let fresh = structural_key("value", "literal", Some(&ieee_type_digest(kind)), body);
            rename_node(&mut standalone, &stale, &fresh);
            rebuild_source_map(&mut standalone);
            refresh_identity(&mut standalone);
            assert_literal_type_refusal(&standalone, position, "");

            for operand in [0, 1] {
                let (mut nested, position) = with_ieee_float_operand(kind, own_family);
                nested["semantic_graph"]["nodes"][position]["body"]["arguments"][operand]["type"] =
                    declared.clone();
                rekey_application_node(&mut nested, position);
                refresh_identity(&mut nested);
                assert_literal_type_refusal(&nested, position, &format!("/arguments/{operand}"));
            }
        }
    }
}

fn ieee_type_digest(kind: &str) -> String {
    structural_key(
        "scalar_type",
        if kind == "float32_bits" {
            "float32"
        } else {
            "float64"
        },
        None,
        &json!({"term": "aggregate", "members": []}),
    )
}

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

/// when the first is valid; both wrong must still locate the first defect.
#[trace("TC-048", "FR-038-AC-222")]
#[test]
fn tc_048_ieee_literal_width_checks_both_arguments() {
    for (declared, matching, wrong) in [
        ("float32", "float32_bits", "float64_bits"),
        ("float64", "float64_bits", "float32_bits"),
    ] {
        for (first, second, offending) in [(matching, wrong, 1), (wrong, wrong, 0)] {
            let (package, position) = with_ieee_float_operands(first, second, declared);
            assert_literal_type_refusal(&package, position, &format!("/arguments/{offending}"));
        }
    }
}

fn with_ieee_conversion(
    identity: &str,
    member_family: &str,
    result_family: &str,
) -> (Value, usize) {
    let source_family = if identity.ends_with("to_float32") {
        "float64"
    } else {
        "float32"
    };
    with_ieee_conversion_source(identity, source_family, member_family, result_family)
}

pub(crate) fn with_ieee_conversion_source(
    identity: &str,
    source_family: &str,
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
    let source = if source_family == "float64" {
        float64
    } else {
        float32
    };
    if member_family == "rational" || result_family == "rational" {
        let digest = structural_key(
            "scalar_type",
            "rational",
            None,
            &json!({"term": "aggregate", "members": []}),
        );
        package["semantic_graph"]["nodes"].as_array_mut().expect("nodes").push(json!({
            "node_id": node_id(&digest), "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": "scalar_type", "semantic_form": "rational", "semantic_type": node_id(&digest),
            "dependencies": [], "occurrences": [{"role": "generated", "ordinal": 0}],
            "body": {"term": "aggregate", "members": []}
        }));
    }
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
            "value_kind": if source_family == "float64" {"float64_bits"} else {"float32_bits"},
            "value": if source_family == "float64" {"3ff0000000000000"} else {"3f800000"},
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

fn assert_member_result_refusal(
    package: &Value,
    position: usize,
    suffix: &str,
    code: CheckedPackageRefusalCode,
    cause: CheckedPackageRefusalCause,
) {
    let CheckedPackageV2ReadResult::Refused(refusal) = read(package) else {
        panic!("member result must refuse at {suffix}");
    };
    assert_eq!(refusal.code, code, "{refusal:?}");
    assert_eq!(refusal.cause, Some(cause));
    assert_eq!(
        refusal.path,
        Some(pointer(&format!(
            "/semantic_graph/nodes/{position}/body/{suffix}"
        )))
    );
    assert_eq!(
        serde_json::to_value(refusal.locus).expect("locus"),
        package["semantic_graph"]["nodes"][position]["node_id"]
    );
}

#[trace("TC-048", "FR-038-AC-247", "FR-038-AC-248")]
#[test]
fn tc_048_to_rational_member_result_and_family_for_both_widths() {
    for source in ["float32", "float64"] {
        let (valid, _) = with_ieee_conversion_source(
            "quire.op.ieee.to_rational",
            source,
            "rational",
            "rational",
        );
        assert!(
            matches!(read(&valid), CheckedPackageV2ReadResult::Admitted(_)),
            "{source} rational/rational must admit"
        );
        for (member, result) in [(source, "rational"), ("rational", source), (source, source)] {
            let (adverse, position) =
                with_ieee_conversion_source("quire.op.ieee.to_rational", source, member, result);
            assert_member_result_refusal(
                &adverse,
                position,
                "result_type",
                CheckedPackageRefusalCode::IllTyped,
                CheckedPackageRefusalCause::OperatorIneligible,
            );
        }
    }
}

#[trace("TC-048", "FR-038-AC-249")]
#[test]
fn tc_048_type_argument_member_requires_closed_declaration() {
    let (valid, position) = with_ieee_conversion("quire.op.ieee.to_float32", "float32", "float32");
    for member in [
        json!({"kind": "type_argument"}),
        json!({"kind": "type_argument", "declaration": "bad"}),
        json!({"kind": "type_argument", "declaration": valid["semantic_graph"]["nodes"][position]["body"]["result_type"], "extra": true}),
        json!({"kind": "type_argument", "type_argument": valid["semantic_graph"]["nodes"][position]["body"]["result_type"]}),
    ] {
        let mut adverse = valid.clone();
        adverse["semantic_graph"]["nodes"][position]["body"]["operation"]["member"] = member;
        settle(&mut adverse);
        assert_member_result_refusal(
            &adverse,
            position,
            "operation/member",
            CheckedPackageRefusalCode::InvalidPackage,
            CheckedPackageRefusalCause::OperationMemberMismatch,
        );
    }
}
