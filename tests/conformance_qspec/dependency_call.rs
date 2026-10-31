// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! QSpec dependency-call vectors through the public checked-package reader.

use super::{cause_word, code_word, fixtures_dir, proposal_dir_of, read_json, read_package};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedNodeId, CheckedNodeTag, CheckedPackageEvidence, CheckedPackageReadLimits,
    CheckedPackageV2, CheckedPackageV2ReadResult, CheckedTerm, CompleteLoweringProfileV2,
    CompleteLoweringRecordV2, ContractDependencies, ContractDependencyInput,
    ContractPackageReadResult, ContractPackageWireV1, ContractRootViewV1, LoweringProfileRef,
};
use serde_json::Value;
use std::collections::BTreeSet;

fn apply_patch(package: &mut Value, patch: &Value) {
    let path = patch["path"].as_str().expect("published patch path");
    let (parent, member) = path.rsplit_once('/').expect("patch member");
    let target = package.pointer_mut(parent).expect("published patch parent");
    match (patch["op"].as_str().expect("published operation"), target) {
        ("replace", Value::Object(map)) => {
            *map.get_mut(member).expect("replaced member") = patch["value"].clone();
        }
        ("replace", Value::Array(items)) => {
            items[member.parse::<usize>().expect("array index")] = patch["value"].clone();
        }
        ("add", Value::Object(map)) => {
            assert!(map
                .insert(member.to_owned(), patch["value"].clone())
                .is_none());
        }
        (operation, _) => panic!("unsupported published patch {operation}: {path}"),
    }
}

fn without_dependency(package: &Value) -> CheckedPackageV2ReadResult {
    let mut evidence = CheckedPackageEvidence::new();
    for feature in package["lock"]["required_features"]
        .as_array()
        .expect("required features")
    {
        evidence.support_feature(feature.as_str().expect("feature"));
    }
    CheckedPackageV2::read(
        &serde_json::to_vec(package).expect("published package"),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    )
}

#[trace(
    "TC-048",
    "FR-038-AC-35",
    "FR-038-AC-36",
    "FR-038-AC-37",
    "FR-038-AC-38"
)]
#[test]
fn published_dependency_call_vectors_bind_every_reader_outcome_and_locus() {
    let fixtures = fixtures_dir();
    let vectors = read_json(&proposal_dir_of(&fixtures).join("dependency-call-vectors.json"))
        .expect("QSpec dependency call vectors");
    assert_eq!(vectors["base"], "fixtures/positive-dependency-call.json");
    assert_eq!(
        vectors["library"]["package"],
        "fixtures/positive-dependency-library.json"
    );
    let base = read_json(&fixtures.join("positive-dependency-call.json"))
        .expect("published positive dependency call");
    let positive = read_package(&base).expect("published dependency evidence");
    assert!(
        matches!(positive, CheckedPackageV2ReadResult::Admitted(_)),
        "published dependency call must admit: {positive:?}"
    );
    let rows = vectors["mutations"]
        .as_array()
        .expect("published mutations");
    assert_eq!(rows.len(), 11, "all QSpec dependency-call rows must run");
    let mut seen = BTreeSet::new();
    for row in rows {
        let id = row["id"].as_str().expect("row id");
        assert!(seen.insert(id), "duplicate row {id}");
        let mut package = base.clone();
        for patch in row["patch"].as_array().expect("patch list") {
            apply_patch(&mut package, patch);
        }
        let outcome = if row["supplied"] == false {
            without_dependency(&package)
        } else {
            read_package(&package).expect("published evidence")
        };
        let CheckedPackageV2ReadResult::Refused(refusal) = outcome else {
            panic!("{id}: expected refusal, got {outcome:?}")
        };
        let actual = match refusal.cause {
            Some(cause) => format!("refused:{}/{}", code_word(refusal.code), cause_word(cause)),
            None => format!("refused:{}", code_word(refusal.code)),
        };
        assert_eq!(actual, row["outcome"], "{id}: code and cause");
        assert_eq!(
            refusal.loci[0].path.as_ref().map(ToString::to_string),
            row["locus"].as_str().map(str::to_owned),
            "{id}: exact published locus"
        );
    }
}

/// Trace: QSpec FR-322-AC-37 and IR FR-349-AC-1. The published importer
/// retains a typed external callee through lowering and canonical readback.
#[trace("TC-233", "FR-322-AC-37", "FR-349-AC-1")]
#[test]
fn published_dependency_call_lowers_and_reads_back_with_selected_callee() {
    let fixtures = fixtures_dir();
    let importer_wire =
        read_json(&fixtures.join("positive-dependency-call.json")).expect("published importer");
    let library_wire =
        read_json(&fixtures.join("positive-dependency-library.json")).expect("published library");
    let CheckedPackageV2ReadResult::Admitted(importer) =
        read_package(&importer_wire).expect("importer production read")
    else {
        panic!("published importer must admit");
    };
    let CheckedPackageV2ReadResult::Admitted(library) =
        read_package(&library_wire).expect("library production read")
    else {
        panic!("published library must admit");
    };
    let call = importer_wire["semantic_graph"]["nodes"]
        .as_array()
        .expect("published graph")
        .iter()
        .find(|node| node["body"]["arguments"][0]["term"] == "dependency_reference")
        .expect("published dependency call");
    let request: CheckedNodeId =
        serde_json::from_value(call["node_id"].clone()).expect("call identity");
    let callee: CheckedNodeId =
        serde_json::from_value(call["body"]["arguments"][0]["node"].clone())
            .expect("selected callee identity");
    let library_name = call["body"]["arguments"][0]["package"]
        .as_str()
        .expect("selected library name");
    assert_eq!(library_name, library.name());
    let Some(CheckedTerm::Application { arguments, .. }) = importer.term(&request) else {
        panic!("published call must retain typed application body");
    };
    let [CheckedTerm::DependencyReference { package, node }, CheckedTerm::Literal { .. }] =
        arguments.as_slice()
    else {
        panic!("published call must retain typed external callee and literal argument");
    };
    assert_eq!(package.as_ref(), library_name);
    assert_eq!(node, &callee);
    assert!(call["dependencies"]
        .as_array()
        .expect("local dependencies")
        .iter()
        .all(|id| *id != call["body"]["arguments"][0]["node"]));

    let profile = CompleteLoweringProfileV2 {
        reference: LoweringProfileRef {
            authority: "quire".into(),
            identity: "complete-v1".into(),
        },
        required_source_selections: Vec::new(),
        supported_tags: CheckedNodeTag::ALL.iter().copied().collect(),
        require_bounds: false,
        work_limit: u64::MAX,
    };
    let result = importer.lower(std::slice::from_ref(&request), &profile);
    assert!(
        matches!(
            result.records.as_slice(),
            [CompleteLoweringRecordV2::Lowered { .. }]
        ),
        "published call must lower: {:?}",
        result.records
    );
    let bytes = result
        .package
        .canonical_bytes()
        .expect("lowered canonical bytes");
    let mut selected = ContractDependencies::new();
    selected.insert(
        library_name,
        ContractDependencyInput {
            identity: library_name,
            package: &library,
        },
    );
    let read_back_outcome =
        ContractPackageWireV1::read(bytes, CheckedPackageReadLimits::bounded(), &selected);
    let ContractPackageReadResult::Admitted(read_back) = read_back_outcome else {
        panic!(
            "published external call must read back with selected library: {read_back_outcome:?}"
        );
    };
    let ContractRootViewV1::Lowered(view) = read_back
        .reached_closure(&request)
        .expect("admitted reached closure")
    else {
        panic!("published call must remain lowered after readback");
    };
    assert_eq!(view.selected_references.len(), 1);
    assert_eq!(view.selected_references[0].package.as_ref(), library_name);
    assert_eq!(view.selected_references[0].node, callee);
    assert!(view.nodes.iter().all(|entry| entry.node.node_id != callee));
}
