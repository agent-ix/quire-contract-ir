// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-035/TC-047: the single six-member package value reads a source-authored
//! nominal body (enum, dimension, unit) that the generic term grammar lacks.

use crate::support::checked_package::{
    canonical, evidence_for, nominal_fixture_members, nominal_package, structural_fixture_id,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedNodeId, CheckedNodeTag, CheckedPackageReadLimits, CheckedPackageRefusalCode as Code,
    CheckedPackageV2, CheckedPackageV2ReadResult, CompleteLoweringProfileV2, ContractDependencies,
    ContractPackageReadResult, ContractPackageWireV1,
};
use serde_json::Value;

fn profile() -> CompleteLoweringProfileV2 {
    CompleteLoweringProfileV2 {
        reference: quire_contract_model::LoweringProfileRef {
            authority: "quire".into(),
            identity: "complete-v1".into(),
        },
        required_source_selections: Vec::new(),
        supported_tags: [
            CheckedNodeTag::ScalarType,
            CheckedNodeTag::CompositeType,
            CheckedNodeTag::BoundedDomain,
            CheckedNodeTag::Value,
            CheckedNodeTag::Expression,
            CheckedNodeTag::Function,
            CheckedNodeTag::Model,
            CheckedNodeTag::Relation,
            CheckedNodeTag::State,
            CheckedNodeTag::Temporal,
            CheckedNodeTag::Protocol,
            CheckedNodeTag::Claim,
        ]
        .into_iter()
        .collect(),
        require_bounds: true,
        work_limit: u64::MAX,
    }
}

fn lowered_nominal_wire() -> Value {
    let value = nominal_package(&nominal_fixture_members());
    let requested: Vec<CheckedNodeId> = value["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .map(|node| structural_fixture_id(&node["node_id"]))
        .collect();
    let CheckedPackageV2ReadResult::Admitted(package) = CheckedPackageV2::read(
        &canonical(&value),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&value),
    ) else {
        panic!("nominal fixture must admit as V2");
    };
    let output = package.lower(&requested, &profile());
    serde_json::from_slice(output.package.canonical_bytes().expect("package bytes"))
        .expect("canonical package")
}

fn read(wire: &Value) -> ContractPackageReadResult {
    ContractPackageWireV1::read(
        &canonical(wire),
        CheckedPackageReadLimits::bounded(),
        &ContractDependencies::new(),
    )
}

fn body_terms(wire: &Value) -> Vec<String> {
    wire["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .filter_map(|node| node["body"]["term"].as_str().map(str::to_owned))
        .collect()
}

/// A V1 package whose graph holds enum, dimension and unit nominal bodies
/// reads clean, as the V2 reader already admits the same bodies.
#[trace("TC-047", "FR-035-AC-22")]
#[test]
fn v1_reader_admits_nominal_enum_dimension_and_unit_bodies() {
    let wire = lowered_nominal_wire();
    let terms = body_terms(&wire);
    for term in ["enum", "dimension", "unit"] {
        assert!(terms.iter().any(|seen| seen == term), "{term} in {terms:?}");
    }
    match read(&wire) {
        ContractPackageReadResult::Admitted(_) => {}
        other => panic!("nominal bodies must read clean, got {other:?}"),
    }
}

/// A node that is not a nominal declaration keeps the generic grammar: an
/// unknown `term` is still refused at the body, and a nominal `term` on a
/// non-nominal node is too.
#[trace("TC-047", "FR-035-AC-22")]
#[test]
fn v1_reader_still_refuses_bogus_or_misplaced_terms() {
    let wire = lowered_nominal_wire();
    assert!(
        matches!(read(&wire), ContractPackageReadResult::Admitted(_)),
        "the unmodified package must read clean"
    );
    let nodes = wire["semantic_graph"]["nodes"].as_array().expect("nodes");
    let position = nodes
        .iter()
        .position(|node| node["semantic_form"] == "enum_value")
        .expect("enum_value node");
    for term in ["bogus", "enum"] {
        let mut changed = wire.clone();
        changed["semantic_graph"]["nodes"][position]["body"]["term"] = Value::from(term);
        let ContractPackageReadResult::Refused(refusal) = read(&changed) else {
            panic!("term {term} on a non-nominal node must refuse");
        };
        assert_eq!(refusal.code, Code::InvalidSemanticGraph, "{term}");
        assert_eq!(
            refusal.loci[0].path.as_ref().map(|path| path.as_str()),
            Some(format!("/semantic_graph/nodes/{position}/body").as_str()),
            "{term}"
        );
    }
}

/// A malformed body on a real nominal node is refused by the nominal body
/// constructor, not admitted by the new path.
#[trace("TC-047", "FR-035-AC-22")]
#[test]
fn v1_reader_refuses_malformed_nominal_bodies() {
    let wire = lowered_nominal_wire();
    let nodes = wire["semantic_graph"]["nodes"].as_array().expect("nodes");
    let position = nodes
        .iter()
        .position(|node| node["semantic_form"] == "enum")
        .expect("enum node");
    let mut duplicate = wire.clone();
    duplicate["semantic_graph"]["nodes"][position]["body"]["members"] =
        serde_json::json!(["ACTIVE", "ACTIVE"]);
    let mut mismatch = wire.clone();
    mismatch["semantic_graph"]["nodes"][position]["body"]["term"] = Value::from("unit");
    for (name, changed, code, path) in [
        (
            "duplicate",
            duplicate,
            Code::MalformedWire,
            format!("/semantic_graph/nodes/{position}/body/members/1"),
        ),
        (
            "mismatch",
            mismatch,
            Code::InvalidSemanticGraph,
            format!("/semantic_graph/nodes/{position}/body"),
        ),
    ] {
        let ContractPackageReadResult::Refused(refusal) = read(&changed) else {
            panic!("{name} nominal body must refuse");
        };
        assert_eq!(refusal.code, code, "{name}");
        assert_eq!(
            refusal.loci[0].path.as_ref().map(|path| path.as_str()),
            Some(path.as_str()),
            "{name}"
        );
    }
}
