// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! IR-482: an enum's operand family is `ordered_enum` when its nominal
//! preimage is ordered and `enum` otherwise (QSpec FR-322), so
//! `quire.op.enum.lt/le/gt/ge` (catalog family `ordered_enum`) admit over an
//! ordered enum and refuse over an unordered one (QSpec FR-141-AC-5), while
//! `quire.op.enum.eq/ne` (`enum_kind`) admit over both.
//!
//! Every package is built here, node by node, in the shapes QSL's emitter
//! writes: an enum `scalar_type` carrying its `enum-declaration-node/v1`
//! preimage, its member `value` nodes, and a `binary` application of the
//! operation over two operands of that enum.

use crate::support::checked_package::{
    canonical, evidence_for, node_id, nominal_package, refusal_at_id, settle, ExpectedRefusal,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedPackageReadLimits, CheckedPackageRefusal, CheckedPackageRefusalCause,
    CheckedPackageRefusalCode, CheckedPackageV2, CheckedPackageV2ReadResult,
};
use serde_json::{json, Value};

/// How the application's operands are formed.
#[derive(Clone, Copy)]
enum Operands {
    /// Two member literals of the enum.
    Members,
    /// Two `value`/`parameter` nodes typed at the enum.
    Parameters,
}

fn owner() -> Value {
    json!({"kind": "source", "authority": "agent-ix", "identity": "quire.fixture.source/v1"})
}

/// `(preimage, key)` of an enum declaration.
fn declaration(name: &str, ordered: bool) -> (Value, String) {
    // The reader requires an unordered enum's members sorted, so the
    // `ordered` flag alone separates the two when the members are sorted.
    let members = if ordered {
        ["READY", "DONE"]
    } else {
        ["DONE", "READY"]
    };
    declaration_with(name, ordered, members)
}

/// An enum declaration with its members in the given order.
fn declaration_with(name: &str, ordered: bool, members: [&str; 2]) -> (Value, String) {
    let preimage = json!({
        "version": "quire.enum-declaration-node/v1",
        "owner": owner(),
        "qualified_declaration": [name],
        "ordered": ordered,
        "members": members,
    });
    let key = format!("enum/{name}");
    (preimage, key)
}

/// `(preimage, key)` of one member of the enum keyed `declaration`.
fn member(declaration: &str, case: &str) -> (Value, String) {
    let preimage = json!({
        "version": "quire.enum-member-node/v1",
        "declaration_node_id": node_id(declaration),
        "case": case,
    });
    let key = format!("enum/member/{declaration}/{case}");
    (preimage, key)
}

fn plain_node(
    key: &str,
    tag: &str,
    form: &str,
    semantic_type: &str,
    role: &str,
    body: Value,
) -> Value {
    json!({
        "node_id": node_id(key),
        "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": tag,
        "semantic_form": form,
        "semantic_type": node_id(semantic_type),
        "dependencies": [],
        "occurrences": [{"role": role, "ordinal": 0}],
        "body": body,
    })
}

fn reference(key: &str) -> Value {
    json!({"term": "reference", "target": node_id(key)})
}

/// A package applying `operation` to two operands typed at the enum
/// `left`, the right operand typed at `right` when it names another enum.
fn package(
    operation: &str,
    left: &(Value, String),
    right: &(Value, String),
    form: Operands,
) -> Value {
    let mut members = vec![left.clone()];
    if right.1 != left.1 {
        members.push(right.clone());
    }
    let mut operands = Vec::new();
    let mut extra = Vec::new();
    // A declaration's package holds every member it declares.
    for declaration in [&left.1, &right.1] {
        for case in ["READY", "DONE"] {
            let (preimage, member_key) = member(declaration, case);
            if !members.iter().any(|(_, held)| *held == member_key) {
                members.push((preimage, member_key));
            }
        }
    }
    for (position, (declaration, case)) in [(&left.1, "READY"), (&right.1, "DONE")]
        .into_iter()
        .enumerate()
    {
        match form {
            Operands::Members => operands.push(member(declaration, case).1),
            Operands::Parameters => extra.push((declaration.clone(), position)),
        }
    }
    let mut package = nominal_package(&members);
    let boolean_body = json!({"term": "aggregate", "members": []});
    let boolean = "scalar_type/boolean".to_owned();
    let mut nodes = vec![plain_node(
        &boolean,
        "scalar_type",
        "boolean",
        &boolean,
        "type",
        boolean_body,
    )];
    if let Operands::Parameters = form {
        let text_body = json!({"term": "aggregate", "members": []});
        let text = "scalar_type/text".to_owned();
        let integer = "scalar_type/integer".to_owned();
        nodes.push(plain_node(
            &text,
            "scalar_type",
            "text",
            &text,
            "type",
            text_body.clone(),
        ));
        nodes.push(plain_node(
            &integer,
            "scalar_type",
            "integer",
            &integer,
            "type",
            text_body,
        ));
        for (declaration, position) in extra {
            let body = json!({"term": "aggregate", "members": [
                {"term": "binding", "name": "name", "value": {"term": "literal",
                    "type": node_id(&text), "value_kind": "text", "value": format!("p{position}")}},
                {"term": "binding", "name": "level", "value": {"term": "literal",
                    "type": node_id(&integer), "value_kind": "integer", "value": "0"}},
            ]});
            let parameter = format!("value/parameter/{position}");
            nodes.push(plain_node(
                &parameter,
                "value",
                "parameter",
                &declaration,
                "expression",
                body,
            ));
            operands.push(parameter);
        }
    }
    let body = json!({
        "term": "application",
        "operator": "binary",
        "operation": {"identity": operation, "laws": [], "mode": null, "member": null,
            "leaves": []},
        "result_type": node_id(&boolean),
        "arguments": [reference(&operands[0]), reference(&operands[1])],
    });
    let application = "expression/enum-operation".to_owned();
    let mut application_node = plain_node(
        &application,
        "expression",
        "binary",
        &boolean,
        "expression",
        body,
    );
    let mut dependencies = [operands[0].as_str(), operands[1].as_str()];
    dependencies.sort_unstable();
    application_node["dependencies"] =
        Value::Array(dependencies.iter().map(|key| node_id(key)).collect());
    nodes.push(application_node);
    package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend(nodes);
    settle(&mut package);
    package
}

fn read(package: &Value) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(
        &canonical(package),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(package),
    )
}

fn admits(package: &Value) -> bool {
    matches!(read(package), CheckedPackageV2ReadResult::Admitted(_))
}

/// The refusal an operation over operands the catalog family rejects gets:
/// `ill_typed`/`operator_ineligible` at the argument that fails, located at
/// the application node.
fn ineligible(package: &Value, argument: usize) -> ExpectedRefusal {
    let nodes = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes");
    let position = nodes.len() - 1;
    refusal_at_id(
        CheckedPackageRefusalCode::IllTyped,
        &format!("/semantic_graph/nodes/{position}/body/arguments/{argument}"),
        Some(CheckedPackageRefusalCause::OperatorIneligible),
        &nodes[position]["node_id"],
    )
}

fn refused(package: &Value) -> CheckedPackageRefusal {
    match read(package) {
        CheckedPackageV2ReadResult::Refused(refusal) => refusal,
        other => panic!("expected V2 refusal, got {other:?}"),
    }
}

const ORDERING: [&str; 4] = [
    "quire.op.enum.lt",
    "quire.op.enum.le",
    "quire.op.enum.gt",
    "quire.op.enum.ge",
];
const EQUALITY: [&str; 2] = ["quire.op.enum.eq", "quire.op.enum.ne"];
const FORMS: [Operands; 2] = [Operands::Members, Operands::Parameters];

#[trace("FR-038-AC-238")]
#[test]
fn ordinary_scalar_enum_still_refuses_nonidentifier_cases() {
    let declaration = declaration_with("Status", false, ["Gold Plus", "READY"]);
    let package = package(
        "quire.op.enum.eq",
        &declaration,
        &declaration,
        Operands::Members,
    );
    let refusal = refused(&package);
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
}

#[trace("TC-048", "FR-038-AC-42")]
#[test]
fn tc_048_ordering_over_an_ordered_enum_admits() {
    // The second declaration is ordered although its members are sorted, so
    // ordering cannot be inferred from member order.
    let declarations = [
        declaration("Status", true),
        declaration_with("Status", true, ["DONE", "READY"]),
    ];
    for ordered in &declarations {
        for form in FORMS {
            for operation in ORDERING {
                let package = package(operation, ordered, ordered, form);
                assert!(admits(&package), "{operation}: {:?}", read(&package));
            }
        }
    }
}

#[trace("TC-048", "FR-038-AC-42")]
#[test]
fn tc_048_ordering_over_an_unordered_enum_refuses() {
    let unordered = declaration("Status", false);
    for form in FORMS {
        for operation in ORDERING {
            let package = package(operation, &unordered, &unordered, form);
            assert_eq!(refused(&package), ineligible(&package, 0), "{operation}");
        }
    }
}

#[trace("TC-048", "FR-038-AC-42")]
#[test]
fn tc_048_equality_over_an_enum_admits_ordered_or_not() {
    for ordered in [true, false] {
        let declaration = declaration("Status", ordered);
        for form in FORMS {
            for operation in EQUALITY {
                let package = package(operation, &declaration, &declaration, form);
                assert!(
                    admits(&package),
                    "{operation} ordered={ordered}: {:?}",
                    read(&package)
                );
            }
        }
    }
}

#[trace("TC-048", "FR-038-AC-42")]
#[test]
fn tc_048_ordering_across_two_enums_refuses() {
    let left = declaration("Status", true);
    let right = declaration("Phase", true);
    for form in FORMS {
        let package = package("quire.op.enum.lt", &left, &right, form);
        assert_eq!(refused(&package), ineligible(&package, 1));
    }
}
