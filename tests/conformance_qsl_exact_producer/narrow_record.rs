// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Fresh producer control for the closed narrow-field projection exception.
use qsl_foundation::SourceIdentity;
use qsl_replay::{
    compile_package_with_execution_points, spine::DependencyInput, ReplayLimits, StageLimits,
};
use quire_contract_model::{
    CheckedCompositeOperandDomain, CheckedOccurrence, CheckedPackageEvidence,
    CheckedPackageReadLimits, CheckedPackageV2, CheckedPackageV2ReadResult,
    CheckedScalarOperandChild,
};
use serde_json::Value;

fn main() {
    for (name, operator) in [("equal", "="), ("unequal", "!=")] {
        let source = format!("language \"ix:native\" edition \"1-draft\";\nprofile v = \"quire.value.complete/v1\";\nrecord Point {{ x: Int[0,9]; }}\nfunction {name} using v(): Boolean pure {{ Point {{ x: 3 }} {operator} Point {{ x: 3 }} }}\n");
        let compiled = compile_package_with_execution_points(
            SourceIdentity::new("agent-ix", "test:narrow-record"),
            "narrow-record".to_owned(),
            "narrow-record.native",
            source.as_bytes(),
            std::iter::empty::<&[u8]>(),
            &DependencyInput::new(Vec::new()).unwrap(),
            &Default::default(),
            StageLimits::default(),
            ReplayLimits::default(),
        )
        .expect("fresh bounded record checking and emission");
        let wire: Value = serde_json::from_slice(compiled.bytes()).unwrap();
        let mut evidence = CheckedPackageEvidence::new();
        for feature in wire["lock"]["required_features"].as_array().unwrap() {
            evidence.support_feature(feature.as_str().unwrap());
        }
        let CheckedPackageV2ReadResult::Admitted(package) = CheckedPackageV2::read(
            compiled.bytes(),
            CheckedPackageReadLimits::bounded(),
            &evidence,
        ) else {
            panic!("fresh bounded record must admit")
        };
        let expected = if name == "equal" {
            "quire.op.structural.eq"
        } else {
            "quire.op.structural.ne"
        };
        let application = package
            .graph()
            .nodes
            .iter()
            .find(|node| {
                serde_json::to_value(&node.body)
                    .unwrap()
                    .pointer("/operation/identity")
                    .and_then(Value::as_str)
                    == Some(expected)
            })
            .expect("genuine structural comparison");
        let occurrence = application.occurrences.first().unwrap();
        let result = package
            .composite_application_operands(
                &application.node_id,
                &CheckedOccurrence {
                    role: occurrence.role.clone(),
                    ordinal: occurrence.ordinal,
                },
                100_000,
            )
            .expect("closed producer-authored narrow record");
        assert_eq!(result.operands.len(), 2);
        for operand in result.operands {
            assert_eq!(operand.domain, CheckedCompositeOperandDomain::Literal);
            let CheckedScalarOperandChild::GraphChild(child) = operand.child else {
                panic!("original graph child")
            };
            let node = package
                .graph()
                .nodes
                .iter()
                .find(|node| node.node_id == child)
                .unwrap();
            assert_eq!(node.semantic_form.as_ref(), "record_value");
            assert_eq!(operand.semantic_type, node.semantic_type);
            let body = serde_json::to_value(&node.body).unwrap();
            let target = &body["members"][0]["value"]["target"];
            let narrow = package
                .graph()
                .nodes
                .iter()
                .find(|node| serde_json::to_value(&node.node_id).unwrap() == *target)
                .unwrap();
            let body = serde_json::to_value(&narrow.body).unwrap();
            assert_eq!(body["operation"]["identity"], "quire.op.numeric.narrow");
            assert_eq!(
                body["result_type"],
                body["operation"]["member"]["declaration"]
            );
            assert_eq!(
                body["result_type"],
                serde_json::to_value(&narrow.semantic_type).unwrap()
            );
            let range = package
                .graph()
                .nodes
                .iter()
                .find(|node| node.node_id == narrow.semantic_type)
                .expect("authentic bounded result");
            assert_eq!(range.semantic_form.as_ref(), "integer_range");
            let range_body = serde_json::to_value(&range.body).unwrap();
            assert_eq!(range_body["members"][0]["name"], "min");
            assert_eq!(range_body["members"][0]["value"]["value"], "0");
            assert_eq!(range_body["members"][1]["name"], "max");
            assert_eq!(range_body["members"][1]["value"]["value"], "9");
            let literal = package
                .graph()
                .nodes
                .iter()
                .find(|node| {
                    serde_json::to_value(&node.node_id).unwrap() == body["arguments"][0]["target"]
                })
                .unwrap();
            let literal = serde_json::to_value(&literal.body).unwrap();
            assert_eq!(literal["value_kind"], "integer");
            assert_eq!(literal["value"], "3");
        }
    }
    println!("fresh bounded-record eq and ne preserve closed authentic operands");
}
