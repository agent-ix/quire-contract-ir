// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-038-AC-65 through AC-69, at the package: the application operator
//! classes `case`, `temporal_formula` and `temporal_fairness` decode and are
//! refused `unsupported_construct`/`expression-form` at any depth, and an
//! operator outside the closed vocabulary refuses `invalid_semantic_graph`.
//! The catalog read, the root-application checks and the temporal clause shape
//! are unit tests in `crates/quire-contract-model/src/checked_package/v2/`,
//! where the catalog's own entries are read rather than listed.

use crate::support::checked_package::{
    canonical, evidence_for, pointer, refresh_identity, rekey_application_node, v2_all_families,
};
use ix_trace_rs::trace;
use quire_contract_ir::{
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

/// Tracing: TC-048, FR-038-AC-66
#[trace("TC-048", "FR-038-AC-66")]
#[test]
fn tc_048_a_nested_application_of_a_refused_class_refuses_at_its_operator() {
    for (operator, identity) in OPERATORS {
        type Build = Box<dyn Fn(&Value) -> Value>;
        let cases: [(&str, Build, &str); 3] = [
            (
                "an argument element",
                Box::new(move |t| nested(operator, identity, t)),
                "arguments/1/operator",
            ),
            (
                "a binding value",
                Box::new(move |t| {
                    json!({"term": "binding", "name": "b",
                           "value": nested(operator, identity, t)})
                }),
                "arguments/1/value/operator",
            ),
            (
                "inside an aggregate",
                Box::new(
                    move |t| json!({"term": "aggregate", "members": [nested(operator, identity, t)]}),
                ),
                "arguments/1/members/0/operator",
            ),
        ];
        for (name, build, tail) in cases {
            let (package, position) = with_second_argument(build);
            let refusal = refused(&package);
            assert_eq!(
                refusal.code,
                CheckedPackageRefusalCode::UnsupportedConstruct,
                "{operator} {name}"
            );
            assert_eq!(
                refusal.cause,
                Some(CheckedPackageRefusalCause::ExpressionForm),
                "{operator} {name}"
            );
            assert_eq!(
                refusal.path,
                Some(pointer(&format!(
                    "/semantic_graph/nodes/{position}/body/{tail}"
                ))),
                "{operator} {name}"
            );
        }
        // The refusal is of the operator class, whatever identity it names.
        let (package, position) =
            with_second_argument(move |t| nested(operator, "quire.op.boolean.not", t));
        assert_eq!(
            refused(&package).path,
            Some(pointer(&format!(
                "/semantic_graph/nodes/{position}/body/arguments/1/operator"
            )))
        );
    }
}

/// Tracing: TC-048, FR-038-AC-66
#[trace("TC-048", "FR-038-AC-66")]
#[test]
fn tc_048_a_root_application_of_a_refused_class_refuses_in_a_package() {
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
            (
                CheckedPackageRefusalCode::UnsupportedConstruct,
                Some(CheckedPackageRefusalCause::ExpressionForm),
                Some(pointer(&format!(
                    "/semantic_graph/nodes/{position}/body/operator"
                ))),
            ),
            "{operator}"
        );
    }
}

/// Tracing: TC-048, FR-038-AC-65
#[trace("TC-048", "FR-038-AC-65")]
#[test]
fn tc_048_an_operator_outside_the_closed_vocabulary_refuses_at_the_term() {
    let (package, position) = with_second_argument(|t| nested("bogus", "quire.op.boolean.not", t));
    let refusal = refused(&package);
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
    assert_eq!(
        refusal.path,
        Some(pointer(&format!(
            "/semantic_graph/nodes/{position}/body/arguments/1"
        )))
    );
}
