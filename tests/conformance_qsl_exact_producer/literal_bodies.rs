// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Fresh U12 square emission through the public QSL and strict IR readers.

use qsl_foundation::SourceIdentity;
use qsl_package::{emit_checked, CheckedPackage};
use qsl_semantics::{check::PackageDeclarations, model::intake::SelectedModels};
use quire_contract_model::{
    CheckedNodeId, CheckedNodeTag, CheckedPackageEvidence, CheckedPackageReadLimits,
    CheckedPackageRefusalCode, CheckedPackageV2, CheckedPackageV2ReadResult, CheckedTerm,
    CompleteLoweringProfileV2, CompleteLoweringRecordV2, LiteralKind, LoweringProfileRef,
};
use quire_semantic_value::checking::CheckingLimits;
use serde_json::{json, Value};

fn main() {
    let source = "language \"ix:native\" edition \"1-draft\";\n\
        profile v = \"quire.value.complete/v1\";\n\
        dimension Distance;\n\
        unit m : Distance = rational(1, 1);\n\
        function square using v(): Boolean pure { 2 m ^ 2 = 2 m ^ 2 }\n";
    let parsed = qsl_cst::parse(
        SourceIdentity::new("agent-ix", "test:literal-body-admission"),
        "square.native",
        source.as_bytes(),
        qsl_cst::Limits::default(),
    )
    .expect("public source parser");
    assert_eq!(parsed.diagnostics(), []);
    let forms = qsl_forms::build_unit(&parsed).expect("public forms builder");
    let checked = CheckedPackage::link(
        PackageDeclarations::assemble(
            parsed.source().reference().clone(),
            "agent-ix/literal-body-admission".to_owned(),
            forms,
            SelectedModels::default(),
            Vec::new(),
        )
        .expect("public assembly")
        .check(CheckingLimits::default())
        .expect("public checking"),
    );
    let emission = emit_checked(&checked).expect("public checked V2 emission");
    assert_eq!(emission.omitted(), []);
    let wire: Value = serde_json::from_slice(emission.package().bytes()).expect("emitted V2");
    let mut evidence = CheckedPackageEvidence::new();
    for feature in wire["lock"]["required_features"]
        .as_array()
        .expect("features")
    {
        evidence.support_feature(feature.as_str().expect("feature name"));
    }
    let package = match CheckedPackageV2::read(
        emission.package().bytes(),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("genuine U12 square must admit: {other:?}"),
    };
    let nodes = wire["semantic_graph"]["nodes"].as_array().expect("nodes");
    let square_position = nodes
        .iter()
        .position(|node| node["body"]["operation"]["identity"] == "quire.op.quantity.pow")
        .expect("square application");
    let exponent_id: CheckedNodeId =
        serde_json::from_value(nodes[square_position]["body"]["arguments"][1]["target"].clone())
            .expect("referenced exponent");
    let exponent_position = nodes
        .iter()
        .position(|node| {
            node["node_id"] == serde_json::to_value(&exponent_id).expect("exponent id")
        })
        .expect("exponent node");
    let exponent = &package.graph().nodes[exponent_position];
    assert!(exponent.dependencies.is_empty());
    let CheckedTerm::Literal {
        type_id,
        value_kind,
        value,
    } = &exponent.body
    else {
        panic!("Integer literal exponent")
    };
    assert_eq!(*value_kind, LiteralKind::Integer);
    assert_eq!(*value, json!("2"));
    assert_eq!(*type_id, exponent.semantic_type);
    let integer = package
        .graph()
        .nodes
        .iter()
        .find(|node| node.node_id == *type_id)
        .expect("Integer type");
    assert_eq!(integer.semantic_form.as_ref(), "integer");
    assert_eq!(integer.node_tag, CheckedNodeTag::ScalarType);
    let request = package.graph().nodes[square_position].node_id.clone();
    let result = package.lower(
        std::slice::from_ref(&request),
        &CompleteLoweringProfileV2 {
            reference: LoweringProfileRef {
                authority: "quire".into(),
                identity: "literal-admission".into(),
            },
            required_source_selections: Vec::new(),
            supported_tags: CheckedNodeTag::ALL.iter().copied().collect(),
            require_bounds: true,
            work_limit: 100_000,
        },
    );
    let [CompleteLoweringRecordV2::Lowered { node }] = result.records.as_slice() else {
        panic!(
            "genuine square lowers with literal exponent: {:?}",
            result.records
        )
    };
    assert!(node.dependencies.contains(&exponent_id));
    assert!(node.dependencies.contains(type_id));
    for body in [
        json!({"term": "reference", "target": nodes[square_position]["body"]["arguments"][0]["target"]}),
        json!({"term": "aggregate", "members": []}),
    ] {
        let mut invalid = wire.clone();
        invalid["semantic_graph"]["nodes"][exponent_position]["body"] = body;
        // Preserve every other producer member. Strict body refusal must win
        // without rekeying, repairing the envelope, or reaching lowering.
        let bytes = serde_json::to_vec(&invalid).expect("canonical mutation");
        match CheckedPackageV2::read(&bytes, CheckedPackageReadLimits::bounded(), &evidence) {
            CheckedPackageV2ReadResult::Refused(refusal) => {
                assert_eq!(refusal.code, CheckedPackageRefusalCode::MalformedWire);
                assert_eq!(refusal.cause, None);
                assert_eq!(refusal.locus.as_ref(), Some(&exponent_id));
                assert_eq!(
                    refusal.loci[0]
                        .path
                        .as_ref()
                        .expect("body pointer")
                        .as_str(),
                    format!("/semantic_graph/nodes/{exponent_position}/body")
                );
            }
            other => panic!("nonliteral exponent must refuse before bounds lowering: {other:?}"),
        }
    }
    println!(
        "FR-038-AC-472 genuine U12 literal admitted; body-only reference and aggregate refused"
    );
}
