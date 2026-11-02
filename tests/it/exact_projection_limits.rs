// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Public admitted type reads with finite work and deep or cyclic type graphs.

use crate::support::checked_package::{canonical, evidence_for, node_id, nominal_package, settle};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedNodeId, CheckedPackageReadLimits, CheckedPackageV2, CheckedPackageV2ReadResult,
    ExactTypeProjectionErrorCode, SelectedExactIdentities,
};
use quire_exact::{Integer, IntegerInterval, ValueType};
use serde_json::{json, Value};

fn node(name: &str, tag: &str, form: &str, ty: &str, dependencies: &[&str], body: Value) -> Value {
    json!({"node_id": node_id(name), "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": tag, "semantic_form": form, "semantic_type": node_id(ty),
        "dependencies": dependencies.iter().map(|name| node_id(name)).collect::<Vec<_>>(),
        "occurrences": [{"role": "type", "ordinal": 0}], "body": body})
}

fn package(nodes: Vec<Value>) -> Value {
    let mut wire = nominal_package(&[]);
    wire["semantic_graph"]["nodes"] = Value::Array(nodes);
    settle(&mut wire);
    wire
}

fn read(wire: &Value) -> CheckedPackageV2ReadResult {
    let bytes = canonical(wire);
    CheckedPackageV2::read(
        &bytes,
        CheckedPackageReadLimits {
            bytes: u64::try_from(bytes.len()).expect("fixture length fits the reader budget"),
            ..CheckedPackageReadLimits::bounded()
        },
        &evidence_for(wire),
    )
}

fn admitted(wire: &Value) -> Box<CheckedPackageV2> {
    match read(wire) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("independent type graph must admit: {other:?}"),
    }
}

#[trace("FR-352-AC-7")]
#[test]
fn admitted_projection_bounds_deep_and_cyclic_types_on_a_small_stack() {
    std::thread::Builder::new().stack_size(256 * 1024).spawn(|| {
        let empty = || json!({"term": "aggregate", "members": []});
        let mut nodes = vec![node("integer", "scalar_type", "integer", "integer", &[], empty()), node("text", "scalar_type", "text", "text", &[], empty())];
        nodes.push(node("digit", "bounded_domain", "integer_range", "integer", &["integer"], json!({"term": "aggregate", "members": [
            {"term": "binding", "name": "min", "value": {"term": "literal", "type": node_id("integer"), "value_kind": "integer", "value": "0"}},
            {"term": "binding", "name": "max", "value": {"term": "literal", "type": node_id("integer"), "value_kind": "integer", "value": "9"}}
        ]})));
        let mut parameter = node("parameter", "value", "parameter", "digit", &[], json!({"term": "aggregate", "members": [
            {"term": "binding", "name": "name", "value": {"term": "literal", "type": node_id("text"), "value_kind": "text", "value": "digit_parameter"}},
            {"term": "binding", "name": "level", "value": {"term": "literal", "type": node_id("integer"), "value_kind": "integer", "value": "0"}}
        ]}));
        parameter["occurrences"][0]["role"] = json!("expression");
        nodes.push(parameter);
        let mut target = "integer".to_owned();
        for index in 0..4096 {
            let name = format!("deep-{index}");
            nodes.push(node(&name, "composite_type", "option", &name, &[&target], json!({"term": "aggregate", "members": [{"term": "reference", "target": node_id(&target)}]})));
            target = name;
        }
        let mut cyclic = node("cyclic", "composite_type", "option", "cyclic", &["cyclic"], json!({"term": "aggregate", "members": [{"term": "reference", "target": node_id("cyclic") }]}));
        cyclic["recursion_group"] = json!("recursive-type");
        nodes.push(cyclic);
        let wire = package(nodes);
        let mut invalid = wire.clone();
        invalid["semantic_graph"]["nodes"][0]["semantic_type"] = node_id("missing-type");
        assert!(matches!(read(&invalid), CheckedPackageV2ReadResult::Refused(_)));
        let admitted = admitted(&wire);
        drop(invalid);
        drop(wire);
        let clone = admitted.clone();
        let identities = SelectedExactIdentities::empty();
        for (name, work, expected) in [
            ("integer", 2, Some(ValueType::Integer)),
            ("parameter", 4, Some(ValueType::Int(IntegerInterval::new(Integer::from(0_i64), Integer::from(9_i64)).unwrap()))),
            (target.as_str(), 2, None), ("cyclic", 2, None),
        ] {
            let id: CheckedNodeId = serde_json::from_value(node_id(name)).unwrap();
            let result = admitted.project_exact_type(&id, &identities, work);
            assert_eq!(result, clone.project_exact_type(&id, &identities, work));
            assert_eq!(result, admitted.project_exact_type(&id, &identities, work));
            match expected {
                Some(expected) => {
                    let projection = result.expect("complete Exact type at its exact work limit");
                    assert_eq!(projection.value_type(), &expected);
                    assert_eq!(projection.work(), work);
                }
                None => {
                    let error = result.expect_err("unsupported nested type has no partial view");
                    assert_eq!(error.code, ExactTypeProjectionErrorCode::UnsupportedExactType);
                    assert_eq!(error.at.as_deref(), Some(&id));
                    assert_eq!(error.form.as_ref(), "composite_type/option");
                    assert_eq!(error.work, work);
                }
            }
            let error = admitted.project_exact_type(&id, &identities, work-1).expect_err("one below required work must exhaust");
            assert_eq!(error, clone.project_exact_type(&id, &identities, work-1).unwrap_err());
            assert_eq!(error.code, ExactTypeProjectionErrorCode::ExhaustedWork);
            assert_eq!(error.requested.as_ref(), &id);
            let locus: CheckedNodeId = serde_json::from_value(node_id(if name == "parameter" { "digit" } else { name })).unwrap();
            assert_eq!(error.at.as_deref(), Some(&locus));
            assert_eq!(error.work, if name == "parameter" { 2 } else { 1 });
        }
    }).expect("spawn small-stack reader").join().expect("bounded public reads must not grow the stack with type depth");
}
