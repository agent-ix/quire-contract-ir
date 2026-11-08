// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Default reader controls for the distinct closed monitor-plan form.

use crate::support::checked_package::{
    canonical, evidence_for, rebuild_complete_source_map, refresh_identity, v2_all_families,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedNodeKind, CheckedPackageReadLimits, CheckedPackageRefusalCode,
    CheckedPackageV2ReadResult, TemporalForm,
};
use serde_json::{json, Value};

fn read(wire: &Value) -> CheckedPackageV2ReadResult {
    crate::read_and_account(
        &canonical(wire),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(wire),
    )
}
#[trace("TC-048", "FR-038-AC-426", "FR-019-AC-23")]
#[test]
fn monitor_plan_absence_is_an_empty_exact_size_iterator() {
    let wire = v2_all_families();
    let CheckedPackageV2ReadResult::Admitted(package) = read(&wire) else {
        panic!("ordinary supporting package admits")
    };
    assert_eq!(package.monitor_plans().len(), 0);
    assert!(package.monitor_plans().next().is_none());
    assert_eq!(
        CheckedNodeKind::decode(
            quire_contract_model::CheckedNodeTag::Temporal,
            "monitor_plan"
        ),
        Some(CheckedNodeKind::Temporal(TemporalForm::MonitorPlan))
    );
}
#[trace("TC-048", "FR-038-AC-422")]
#[test]
fn monitor_plan_empty_body_refuses_at_first_required_binding() {
    let mut wire = v2_all_families();
    let boolean = wire["semantic_graph"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["semantic_form"] == "boolean")
        .unwrap()["node_id"]
        .clone();
    let root = wire["semantic_graph"]["nodes"].as_array().unwrap().len();
    let id = json!({"package":wire["name"],"path":["temporal","monitor_plan","empty"],"ordinal":0});
    wire["semantic_graph"]["nodes"].as_array_mut().unwrap().push(json!({
        "schema_version":"quire.checked-semantic-graph/v2","node_id":id,"node_tag":"temporal","semantic_form":"monitor_plan",
        "semantic_type":boolean,"body":{"term":"aggregate","members":[]},"dependencies":[boolean],"occurrences":[{"role":"generated","ordinal":0}]
    }));
    rebuild_complete_source_map(&mut wire);
    refresh_identity(&mut wire);
    let CheckedPackageV2ReadResult::Refused(refused) = read(&wire) else {
        panic!("empty plan must refuse")
    };
    assert_eq!(
        refused.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
    assert_eq!(refused.cause, None);
    assert_eq!(
        refused.loci[0].path.as_ref().unwrap().as_str(),
        format!("/semantic_graph/nodes/{root}/body/members/0")
    );
    assert_eq!(
        refused.locus,
        Some(
            serde_json::from_value(wire["semantic_graph"]["nodes"][root]["node_id"].clone())
                .unwrap()
        )
    );
}

#[trace("TC-048", "FR-038-AC-425")]
#[test]
fn anonymous_entry_support_keeps_model_and_source_owner_requirements() {
    for source in [false, true] {
        let mut wire = v2_all_families();
        let node = wire["semantic_graph"]["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .position(|n| n["node_tag"] == "function" && n["owner"]["kind"] == "source")
            .expect("existing declared function");
        if !source {
            wire["semantic_graph"]["nodes"][node]
                .as_object_mut()
                .unwrap()
                .remove("declaration");
            wire["semantic_graph"]["nodes"][node]["occurrences"][0]["role"] = json!("generated");
        }
        wire["semantic_graph"]["nodes"][node]
            .as_object_mut()
            .unwrap()
            .remove("owner");
        let CheckedPackageV2ReadResult::Refused(refused) = read(&wire) else {
            panic!("ordinary owner-bearing function cannot become an anonymous entry")
        };
        assert_eq!(refused.code, CheckedPackageRefusalCode::MalformedWire);
        assert_eq!(
            refused.loci[0].path.as_ref().unwrap().as_str(),
            format!("/semantic_graph/nodes/{node}")
        );
    }
}

/// The graph helper used by monitor admission pays its denial pointer once;
/// final outcome conversion must neither reallocate nor charge it again.
#[trace("TC-472", "FR-038-AC-510")]
#[test]
fn graph_storage_denial_pointer_is_paid_once_through_the_public_reader() {
    use quire_contract_model::{
        CheckedPackageV2BudgetedReadResult, NativeJoinBudget, NativeJoinLocusSource,
        PackageReadResource, PackageReadStage,
    };
    let wire = v2_all_families();
    let bytes = canonical(&wire);
    let evidence = evidence_for(&wire);
    let read = |limit| {
        let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
        let result = CheckedPackageV2::read_with_budget(
            &bytes,
            CheckedPackageReadLimits::bounded(),
            &evidence,
            &mut budget,
        );
        (result, budget)
    };
    // Stop at the first suitable node-owned request, rather than sweeping
    // graph or later-stage boundaries. Each jump is one exact request.
    let mut before = 0_u64;
    loop {
        let (outcome, budget) = read(before);
        assert!(
            !matches!(outcome, CheckedPackageV2BudgetedReadResult::Admitted(_)),
            "the supporting graph must make a node-owned request"
        );
        let locus = budget.first_unpaid_locus().expect("a denied request");
        let amount = budget.first_unpaid_read_storage().expect("storage request");
        if let NativeJoinLocusSource::PackageRead(PackageReadStage::Graph, position) = locus.source
        {
            let expected = format!("/semantic_graph/nodes/{position}");
            let pointer_bytes = u64::try_from(expected.len()).expect("small pointer");
            if locus.form == "state" && amount > u128::from(pointer_bytes) {
                let (fits, paid) = read(before + pointer_bytes);
                // Typed decoders and libraries can share the node tag while
                // intentionally carrying no node pointer. Select the actual
                // graph helper's node-owned pointer, and require one exists.
                if !matches!(&fits, CheckedPackageV2BudgetedReadResult::BudgetIncomplete(value)
                    if value.path.as_ref().map(|path| path.as_str()) == Some(expected.as_str()))
                {
                    before = before
                        .checked_add(u64::try_from(amount).expect("fixture request fits"))
                        .expect("fixture total fits");
                    continue;
                }
                let CheckedPackageV2BudgetedReadResult::BudgetIncomplete(incomplete) = fits else {
                    panic!("the denial pointer fits without a second charge: {fits:?}");
                };
                assert_eq!(incomplete.denied.resource, PackageReadResource::ReadStorage);
                assert_eq!(incomplete.denied.amount, amount);
                assert_eq!(
                    incomplete.path.as_ref().expect("paid pointer").as_str(),
                    expected
                );
                assert_eq!(paid.read_storage_consumed(), before + pointer_bytes);
                assert_eq!(paid.first_unpaid_locus(), Some(locus));
                let (short, unpaid) = read(before + pointer_bytes - 1);
                let CheckedPackageV2BudgetedReadResult::Exhausted(exhausted) = short else {
                    panic!("an unaffordable pointer exhausts: {short:?}");
                };
                assert_eq!(exhausted.denied.resource, PackageReadResource::ReadStorage);
                assert_eq!(exhausted.denied.amount, amount);
                assert_eq!(exhausted.locus, locus);
                assert_eq!(unpaid.first_unpaid_read_storage(), Some(amount));
                assert_eq!(unpaid.read_storage_consumed(), before);
                break;
            }
        }
        before = before
            .checked_add(u64::try_from(amount).expect("fixture request fits"))
            .expect("fixture total fits");
    }
}
