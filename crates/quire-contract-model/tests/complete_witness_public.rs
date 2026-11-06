// SPDX-License-Identifier: AGPL-3.0-or-later
//! Public native acquisition and rebinding without optional model features.

use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageV2, CheckedPackageV2ReadResult,
    ExactTypeProjectionErrorCode, NativeJoinBudget, SelectedExactIdentities,
};
use serde_json::json;

fn admitted() -> CheckedPackageV2 {
    let id = json!({"package":"complete-native", "path":["scalar_type","boolean"], "ordinal":0});
    let source = json!({"authority":"test","identity":"complete-native"});
    let wire = json!({
        "contract_version":"quire.checked-package/v2", "name":"complete-native",
        "lock":{"sources":[source],"edition":{"role":"edition","definition":{"authority":"test","identity":"edition"}},
            "profile_selections":[],"definition_selections":[],"required_features":[],"dependency_selections":[]},
        "semantic_graph":{"graph_version":"quire.checked-semantic-graph/v2","nodes":[{
            "node_id":id,"schema_version":"quire.checked-semantic-graph/v2","node_tag":"scalar_type","semantic_form":"boolean",
            "semantic_type":id,"dependencies":[],"occurrences":[{"role":"type","ordinal":0}],"body":{"term":"aggregate","members":[]}
        }]},
        "source_map":[{"node_id":id,"role":"type","ordinal":0,"regions":[{"source":source,"start":0,"end":1}]}],
        "capability_report":[],"diagnostics":{"catalog":{"authority":"test","identity":"diagnostics"},"entries":[]}
    });
    match CheckedPackageV2::read(
        &serde_json::to_vec(&wire).unwrap(),
        CheckedPackageReadLimits::bounded(),
        &CheckedPackageEvidence::new(),
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("strict obligation-free package admits: {other:?}"),
    }
}

#[trace("TC-467", "FR-352-AC-22", "FR-352-AC-28")]
#[test]
fn complete_witness_public_default_feature_consumer_uses_the_shared_budget() {
    let package = admitted();
    let mut budget = NativeJoinBudget::new(10_000, 10_000);
    let stage = SelectedExactIdentities::from_stage_rows(&package, [], [], &mut budget).unwrap();
    assert_eq!(budget.consumed().storage, 24,
        "nine owned containers plus nine Arc handles, five uniqueness containers, one binding handle");
    let selected = stage
        .with_admitted_type_rows(&package, &[], &[], &mut budget)
        .unwrap()
        .with_source_union_rows(&package, [], &mut budget)
        .unwrap()
        .with_source_record_rows(&package, [], &mut budget)
        .unwrap()
        .with_source_tuple_rows(&package, [], &mut budget)
        .unwrap()
        .with_source_enum_rows(&package, [], &mut budget)
        .unwrap()
        .with_source_collection_rows(&package, [], &mut budget)
        .unwrap();
    let before = budget.consumed();
    let rebound = selected.validate_and_rebind(&package, &mut budget).unwrap();
    assert_eq!(rebound, selected);
    assert!(budget.consumed().work > before.work && budget.consumed().storage > before.storage);
    assert_eq!(
        selected.validate_and_rebind(&package.clone(), &mut budget),
        Ok(selected.clone())
    );
    assert_eq!(
        selected
            .validate_and_rebind(&admitted(), &mut budget)
            .unwrap_err()
            .code,
        ExactTypeProjectionErrorCode::ForeignNode
    );
}
