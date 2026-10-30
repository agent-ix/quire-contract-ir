// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! A structural `dependency_reference` names a directly selected package and
//! one declared function node. Published producer and adverse vectors run in
//! `conformance_qspec`.

use crate::support::checked_package::{
    canonical, evidence_for, read_with_dependencies, v2_all_families,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedNodeTag, CheckedOccurrenceRole, CheckedPackageReadLimits, CheckedPackageRefusalCode,
    CheckedPackageV2, CheckedPackageV2ReadResult, CheckedTerm, CompleteLoweringProfileV2,
    ContractDependencies, ContractDependencyInput, ContractOccurrenceKeyV1,
    ContractPackageReadResult, ContractPackageWireV1, ContractResultViewV1, ContractRootViewV1,
    LoweringProfileRef,
};
use serde_json::{Value, json};

const LIBRARY: &str = "test/geometry";
const CALLER: &str = "test/families";

fn retain_nodes(mut package: Value, positions: &[usize]) -> Value {
    let original = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .clone();
    let retained = positions
        .iter()
        .map(|&position| original[position].clone())
        .collect::<Vec<_>>();
    package["source_map"] = json!(
        package["source_map"]
            .as_array()
            .expect("source map")
            .iter()
            .filter(|entry| retained
                .iter()
                .any(|node| entry["node_id"] == node["node_id"]))
            .cloned()
            .collect::<Vec<_>>()
    );
    package["semantic_graph"]["nodes"] = json!(retained);
    package
}

fn rename_package(value: &mut Value, from: &str, to: &str) {
    match value {
        Value::Object(members) => {
            for (name, member) in members {
                if name == "package" && member == from {
                    *member = json!(to);
                } else {
                    rename_package(member, from, to);
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                rename_package(item, from, to);
            }
        }
        _ => {}
    }
}

fn library_document() -> Value {
    let mut document = retain_nodes(v2_all_families(), &[0, 13]);
    document["name"] = json!(LIBRARY);
    rename_package(&mut document, CALLER, LIBRARY);
    document
}

fn admitted_library(document: &Value) -> Box<CheckedPackageV2> {
    match CheckedPackageV2::read(
        &canonical(document),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(document),
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("library must admit: {other:?}"),
    }
}

fn caller_document(library: &Value) -> Value {
    let mut document = retain_nodes(v2_all_families(), &[0, 5]);
    let call = &mut document["semantic_graph"]["nodes"][1];
    let old_id = call["node_id"].clone();
    let call_id = json!({"package": CALLER, "path": ["function", "call", "library"], "ordinal": 0});
    call["node_id"] = call_id.clone();
    call["body"]["arguments"][0] = json!({
        "term": "dependency_reference",
        "package": LIBRARY,
        "node": library["semantic_graph"]["nodes"][1]["node_id"],
    });
    call["dependencies"] = json!([]);
    let source = document["source_map"]
        .as_array_mut()
        .expect("source map")
        .iter_mut()
        .find(|entry| entry["node_id"] == old_id)
        .expect("call source");
    source["node_id"] = call_id;
    document["lock"]["dependency_selections"] = json!([{"identity": LIBRARY}]);
    document
}

fn read(caller: &Value, library: &CheckedPackageV2) -> CheckedPackageV2ReadResult {
    read_with_dependencies(caller, &[(LIBRARY, library)])
}

fn refusal(
    caller: &Value,
    library: &CheckedPackageV2,
) -> quire_contract_model::CheckedPackageRefusal {
    let CheckedPackageV2ReadResult::Refused(refusal) = read(caller, library) else {
        panic!("expected refusal, got {:?}", read(caller, library));
    };
    refusal
}

#[trace("TC-048", "FR-349-AC-1")]
#[test]
fn external_callee_is_retained_and_lowered_without_a_graph_dependency() {
    let library_wire = library_document();
    let library = admitted_library(&library_wire);
    let caller_wire = caller_document(&library_wire);
    let CheckedPackageV2ReadResult::Admitted(caller) = read(&caller_wire, &library) else {
        panic!(
            "external call must admit: {:?}",
            read(&caller_wire, &library)
        );
    };
    let call = &caller.graph().nodes[1];
    assert!(call.dependencies.is_empty());
    assert_eq!(
        serde_json::to_value(&call.body).expect("typed body")["arguments"][0],
        caller_wire["semantic_graph"]["nodes"][1]["body"]["arguments"][0]
    );
    assert!(matches!(
        caller.term(&call.node_id),
        Some(CheckedTerm::Application { arguments, .. })
            if matches!(arguments.first(), Some(CheckedTerm::DependencyReference { package, node })
                if package.as_ref() == LIBRARY && node == &library.graph().nodes[1].node_id)
    ));

    let root = call.node_id.clone();
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
    let lowered = caller.lower(std::slice::from_ref(&root), &profile);
    let wire = lowered.package.wire().expect("lowered wire");
    let ContractRootViewV1::Lowered(reached) =
        wire.reached_closure(&root).expect("reached closure")
    else {
        panic!("external call must lower");
    };
    assert_eq!(reached.selected_references.len(), 1);
    assert_eq!(reached.selected_references[0].package.as_ref(), LIBRARY);
    assert_eq!(
        reached.selected_references[0].node,
        library.graph().nodes[1].node_id
    );
    assert!(
        !reached
            .nodes
            .iter()
            .any(|node| node.node.node_id == library.graph().nodes[1].node_id)
    );

    let mut selected = ContractDependencies::new();
    selected.insert(
        LIBRARY,
        ContractDependencyInput {
            identity: LIBRARY,
            package: &library,
        },
    );
    let read_back_result = ContractPackageWireV1::read(
        lowered.package.canonical_bytes().expect("lowered bytes"),
        CheckedPackageReadLimits::bounded(),
        &selected,
    );
    let ContractPackageReadResult::Admitted(read_back) = read_back_result else {
        panic!("lowered call must read back: {read_back_result:?}");
    };
    assert_eq!(
        read_back.reached_closure(&root),
        Ok(ContractRootViewV1::Lowered(reached.clone()))
    );
    let occurrence = ContractOccurrenceKeyV1 {
        node_id: root.clone(),
        role: CheckedOccurrenceRole::Generated,
        ordinal: 0,
    };
    assert!(matches!(
        read_back.result_context(
            &root,
            &occurrence,
            None,
            quire_contract_model::BoundedDomainForm::IntegerRange
        ),
        Ok(ContractResultViewV1::Lowered(_))
    ));
}

#[trace("TC-048")]
#[test]
fn dependency_reference_uses_direct_selection_and_exact_node() {
    let library_wire = library_document();
    let library = admitted_library(&library_wire);
    let caller = caller_document(&library_wire);
    assert!(matches!(
        read(&caller, &library),
        CheckedPackageV2ReadResult::Admitted(_)
    ));

    let mut missing_selection = caller.clone();
    missing_selection["lock"]["dependency_selections"] = json!([]);
    let result = refusal(&missing_selection, &library);
    assert_eq!(result.code, CheckedPackageRefusalCode::MissingDeclaration);
    assert_eq!(
        result.cause,
        Some(quire_contract_model::CheckedPackageRefusalCause::MissingSelection)
    );
    assert_eq!(
        result.loci[0].path.as_ref().map(ToString::to_string),
        Some("/semantic_graph/nodes/1/body/arguments/0/package".to_owned())
    );

    let mut missing_name = caller;
    missing_name["semantic_graph"]["nodes"][1]["body"]["arguments"][0]["node"]["path"] =
        json!(["Absent"]);
    let result = refusal(&missing_name, &library);
    assert_eq!(result.code, CheckedPackageRefusalCode::MissingDeclaration);
    assert_eq!(
        result.cause,
        Some(quire_contract_model::CheckedPackageRefusalCause::MissingName)
    );
    assert_eq!(
        result.loci[0].path.as_ref().map(ToString::to_string),
        Some("/semantic_graph/nodes/1/body/arguments/0/node".to_owned())
    );
}

#[trace("TC-048")]
#[test]
fn wrong_callee_position_refuses_as_operator_ineligible() {
    let library_wire = library_document();
    let library = admitted_library(&library_wire);
    let mut caller = caller_document(&library_wire);
    caller["semantic_graph"]["nodes"][1]["body"]["arguments"].as_array_mut().expect("arguments").push(
        json!({"term": "dependency_reference", "package": LIBRARY, "node": library_wire["semantic_graph"]["nodes"][1]["node_id"]})
    );
    let result = refusal(&caller, &library);
    assert_eq!(result.code, CheckedPackageRefusalCode::IllTyped);
    assert_eq!(
        result.cause,
        Some(quire_contract_model::CheckedPackageRefusalCause::OperatorIneligible)
    );
    assert_eq!(
        result.loci[0].path.as_ref().map(ToString::to_string),
        Some("/semantic_graph/nodes/1/body/arguments/1".to_owned())
    );
}
