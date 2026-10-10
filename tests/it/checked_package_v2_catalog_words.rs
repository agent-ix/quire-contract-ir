// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-038-AC-65 and FR-038-AC-100, at the package: the application operator
//! classes `case`, `temporal_formula` and `temporal_fairness` decode, stand
//! only at the body root of their node form, and refuse
//! `ill_typed`/`operator-ineligible` at the node that holds them anywhere
//! else, and at the application's `operator` in a diagnostic detail; an
//! operator outside the closed vocabulary refuses `invalid_semantic_graph`.
//! The catalog read, the root-application checks and the temporal clause shape
//! are unit tests in `crates/quire-contract-model/src/checked_package/v2/`,
//! where the catalog's own entries are read rather than listed; the placement
//! of nodes and references and the rest of the temporal step are
//! `checked_package_v2_temporal.rs`.

use crate::support::checked_package::{
    canonical, evidence_for, pointer, refresh_identity, rekey_application_node, v2_all_families,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedPackageReadLimits, CheckedPackageRefusal, CheckedPackageRefusalCause,
    CheckedPackageRefusalCode, CheckedPackageV2, CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};

const OPERATORS: [(&str, &str); 3] = [
    ("case", "quire.op.control.case"),
    ("temporal_formula", "quire.op.temporal.holds"),
    ("temporal_fairness", "quire.op.temporal.fair"),
];

fn refused(package: &Value) -> CheckedPackageRefusal {
    match CheckedPackageV2::read(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(package),
    ) {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        other => panic!("expected a refusal, read {other:?}"),
    }
}

/// The position of the `function` family's call node, whose `arguments` the
/// cases below extend.
fn call_position(package: &Value) -> usize {
    package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_tag"] == "function" && node["body"]["term"] == "application")
        .expect("the fixture's function call node")
}

fn nested(operator: &str, identity: &str, result_type: &Value) -> Value {
    json!({
        "term": "application",
        "operator": operator,
        "operation": {
            "identity": identity, "laws": [], "mode": null, "member": null, "leaves": [],
        },
        "result_type": result_type,
        "arguments": [],
    })
}

fn with_second_argument(second: impl Fn(&Value) -> Value) -> (Value, usize) {
    let mut package = v2_all_families();
    let position = call_position(&package);
    let call = &mut package["semantic_graph"]["nodes"][position];
    let result_type = call["body"]["result_type"].clone();
    call["body"]["arguments"]
        .as_array_mut()
        .expect("arguments")
        .push(second(&result_type));
    rekey_application_node(&mut package, position);
    refresh_identity(&mut package);
    (package, position)
}

/// `ill_typed`/`operator-ineligible` at `path`.
fn ineligible(
    path: &str,
) -> (
    CheckedPackageRefusalCode,
    Option<CheckedPackageRefusalCause>,
    Option<quire_contract_model::JsonPointer>,
) {
    (
        CheckedPackageRefusalCode::IllTyped,
        Some(CheckedPackageRefusalCause::OperatorIneligible),
        Some(pointer(path)),
    )
}

/// A nested application is outside the body grammar, whatever its class: a
/// `case` one refuses `ill_typed`/`operator-ineligible` at its own `operator`
/// with the holder as locus, and one of the temporal classes `malformed_wire`
/// at the nested application with no locus, at strict wire validation and so
/// whatever the identities of the package are (FR-038-AC-114, FR-038-AC-115).
///
/// Tracing: TC-048, FR-038-AC-100, FR-038-AC-114, FR-038-AC-115
#[trace("TC-048", "FR-038-AC-100", "FR-038-AC-114", "FR-038-AC-115")]
#[test]
fn tc_048_a_nested_application_of_a_placed_class_refuses_at_the_nested_application() {
    for (operator, identity) in OPERATORS {
        type Build = Box<dyn Fn(&Value) -> Value>;
        // Where the nested application sits, below the holder's body.
        let cases: [(&str, Build, &str); 3] = [
            (
                "an argument element",
                Box::new(move |t| nested(operator, identity, t)),
                "arguments/1",
            ),
            (
                "a binding value",
                Box::new(move |t| {
                    json!({"term": "binding", "name": "b",
                           "value": nested(operator, identity, t)})
                }),
                "arguments/1/value",
            ),
            (
                "inside an aggregate",
                Box::new(
                    move |t| json!({"term": "aggregate", "members": [nested(operator, identity, t)]}),
                ),
                "arguments/1/members/0",
            ),
        ];
        // A nested `case` is refused at its own `operator`, the temporal classes
        // at the nested application itself.
        let expected = |position: usize, below: &str| {
            if operator == "case" {
                ineligible(&format!(
                    "/semantic_graph/nodes/{position}/body/{below}/operator"
                ))
            } else {
                (
                    CheckedPackageRefusalCode::MalformedWire,
                    None,
                    Some(pointer(&format!(
                        "/semantic_graph/nodes/{position}/body/{below}"
                    ))),
                )
            }
        };
        let holder = |package: &Value, position: usize| {
            if operator == "case" {
                package["semantic_graph"]["nodes"][position]["node_id"].clone()
            } else {
                Value::Null
            }
        };
        for (name, build, below) in cases {
            let (package, position) = with_second_argument(build);
            let refusal = refused(&package);
            assert_eq!(
                (refusal.code, refusal.cause, refusal.path),
                expected(position, below),
                "{operator} {name}"
            );
            assert_eq!(
                serde_json::to_value(refusal.locus).expect("locus"),
                holder(&package, position),
                "{operator} {name}: the locus"
            );
        }
        // The refusal is of the operator class, whatever identity it names.
        let (package, position) =
            with_second_argument(move |t| nested(operator, "quire.op.boolean.not", t));
        assert_eq!(refused(&package).path, expected(position, "arguments/1").2);
    }
}

/// Tracing: TC-048, FR-038-AC-100
#[trace("TC-048", "FR-038-AC-100")]
#[test]
fn tc_048_a_root_application_of_a_placed_class_in_another_form_refuses_in_a_package() {
    for (operator, identity) in OPERATORS {
        let mut package = v2_all_families();
        let position = call_position(&package);
        let call = &mut package["semantic_graph"]["nodes"][position];
        call["body"]["operator"] = json!(operator);
        call["body"]["operation"]["identity"] = json!(identity);
        call["body"]["arguments"] = json!([]);
        // No reference remains, so the application join leaves no dependency.
        call["dependencies"] = json!([]);
        rekey_application_node(&mut package, position);
        refresh_identity(&mut package);
        let refusal = refused(&package);
        assert_eq!(
            (refusal.code, refusal.cause, refusal.path),
            ineligible(&format!("/semantic_graph/nodes/{position}")),
            "{operator}"
        );
    }
}

/// A diagnostic detail is a semantic term read by the same walk, and no
/// node's body, so an application of a placed class as its root, or nested in
/// it, refuses at its own `operator`.
///
/// Tracing: TC-048, FR-038-AC-100
#[trace("TC-048", "FR-038-AC-100")]
#[test]
fn tc_048_a_diagnostic_detail_of_a_placed_class_refuses_at_its_operator() {
    for (operator, identity) in OPERATORS {
        let base = v2_all_families();
        let result_type = base["semantic_graph"]["nodes"][0]["node_id"].clone();
        let cases = [
            (
                "a detail root",
                nested(operator, identity, &result_type),
                "operator",
            ),
            (
                "inside a detail aggregate",
                json!({"term": "aggregate",
                       "members": [nested(operator, identity, &result_type)]}),
                "members/0/operator",
            ),
            (
                "a detail binding value",
                json!({"term": "binding", "name": "b",
                       "value": nested(operator, identity, &result_type)}),
                "value/operator",
            ),
        ];
        for (name, detail, tail) in cases {
            let mut package = base.clone();
            package["diagnostics"]["entries"] = json!([{
                "stage": "type_checking", "code": "ill_typed", "cause_tag": "invalid-value",
                "details": [detail], "loci": [],
            }]);
            let refusal = refused(&package);
            assert_eq!(
                (refusal.code, refusal.cause, refusal.path),
                ineligible(&format!("/diagnostics/entries/0/details/0/{tail}")),
                "{operator} {name}"
            );
        }
    }
}

/// Tracing: TC-048, FR-038-AC-65
#[trace("TC-048", "FR-038-AC-65")]
#[test]
fn tc_048_an_operator_outside_the_closed_vocabulary_refuses_at_the_term() {
    let mut package = v2_all_families();
    let position = call_position(&package);
    package["semantic_graph"]["nodes"][position]["body"]["operator"] = json!("bogus");
    rekey_application_node(&mut package, position);
    refresh_identity(&mut package);
    let refusal = refused(&package);
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
    assert_eq!(
        refusal.path,
        Some(pointer(&format!("/semantic_graph/nodes/{position}/body")))
    );
}

/// An application stands only at a body root, so one nested in another term is
/// outside the grammar whatever it names, an operator outside the closed
/// vocabulary included (FR-038-AC-114).
///
/// Tracing: TC-048, FR-038-AC-114
#[trace("TC-048", "FR-038-AC-114")]
#[test]
fn tc_048_a_nested_application_with_an_unknown_operator_is_outside_the_grammar() {
    let (package, position) = with_second_argument(|t| nested("bogus", "quire.op.boolean.not", t));
    let refusal = refused(&package);
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MalformedWire);
    assert_eq!(
        refusal.path,
        Some(pointer(&format!(
            "/semantic_graph/nodes/{position}/body/arguments/1"
        )))
    );
}
