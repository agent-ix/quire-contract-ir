// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The source-map, equivalence, extension, capability, diagnostic,
//! operation-effect and checked-term-data stages charge every heap request to
//! the caller's budget before it is made (TC-472).

use crate::support::checked_package::{
    canonical, evidence_for, node_id, nominal_fixture_members, nominal_package, v2_all_families,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedPackagePendingOutcome, CheckedPackageReadLimits, CheckedPackageRefusalCode,
    CheckedPackageV2, CheckedPackageV2BudgetedReadResult, NativeJoinBudget, NativeJoinLocusSource,
    PackageReadResource, PackageReadStage,
};
use serde_json::{json, Value};

/// One heap request of the read, in the order it was made.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Request {
    pub stage: PackageReadStage,
    pub index: usize,
    pub form: &'static str,
    pub amount: u128,
    /// Read storage admitted before this request.
    pub before: u64,
}

type Reader<'a> = &'a dyn Fn(&mut NativeJoinBudget) -> CheckedPackageV2BudgetedReadResult;

fn reader(wire: &Value) -> impl Fn(&mut NativeJoinBudget) -> CheckedPackageV2BudgetedReadResult {
    let bytes = canonical(wire);
    let evidence = evidence_for(wire);
    move |budget| {
        CheckedPackageV2::read_with_budget(
            &bytes,
            CheckedPackageReadLimits::bounded(),
            &evidence,
            budget,
        )
    }
}

fn read_at(read: Reader<'_>, limit: u64) -> (CheckedPackageV2BudgetedReadResult, NativeJoinBudget) {
    let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
    let result = read(&mut budget);
    (result, budget)
}

/// The request a read with `limit` bytes of read storage stops at, if any.
fn denial_at(read: Reader<'_>, limit: u64) -> Option<Request> {
    let (result, budget) = read_at(read, limit);
    if matches!(result, CheckedPackageV2BudgetedReadResult::Admitted(_)) {
        return None;
    }
    let locus = budget.first_unpaid_locus().expect("a denial has a locus");
    let NativeJoinLocusSource::PackageRead(stage, index) = locus.source else {
        panic!("a read denial is at a package read locus: {locus:?}");
    };
    Some(Request {
        stage,
        index,
        form: locus.form,
        amount: budget
            .first_unpaid_read_storage()
            .expect("read storage denial"),
        before: limit,
    })
}

/// Every request the read makes while it is in `stage`, found by raising the
/// limit to exactly admit each request in turn. The first request of the stage
/// is found by bisection, since the stage a read reaches never falls as the
/// limit rises.
fn stage_requests(read: Reader<'_>, stage: PackageReadStage) -> Vec<Request> {
    let total = {
        let (result, budget) = read_at(read, u64::MAX);
        assert!(
            matches!(result, CheckedPackageV2BudgetedReadResult::Admitted(_)),
            "{result:?}"
        );
        budget.read_storage_consumed()
    };
    let (mut low, mut high) = (0_u64, total);
    while low < high {
        let middle = low + (high - low) / 2;
        match denial_at(read, middle) {
            Some(request) if request.stage < stage => low = middle + 1,
            _ => high = middle,
        }
    }
    let mut limit = low;
    let mut found = Vec::new();
    while let Some(mut request) = denial_at(read, limit) {
        if request.stage > stage {
            break;
        }
        request.before = limit;
        limit += u64::try_from(request.amount).expect("amount fits");
        found.push(request);
    }
    found
}

/// Each request of `requests` is charged before it is made: the allowance
/// exactly covering it gets past it, and one byte short is denied at it with
/// its locus and amount.
fn assert_exact_and_one_short(read: Reader<'_>, requests: &[Request]) {
    assert!(!requests.is_empty(), "the stage makes requests");
    for (position, request) in requests.iter().enumerate() {
        let exact = request.before + u64::try_from(request.amount).unwrap();
        let short = denial_at(read, exact - 1).expect("one byte short is denied");
        assert_eq!(
            (short.stage, short.index, short.form, short.amount),
            (request.stage, request.index, request.form, request.amount),
            "request {position}"
        );
        if let Some(next) = denial_at(read, exact) {
            assert!(
                next.stage > request.stage
                    || requests.get(position + 1).is_some_and(|following| {
                        (
                            following.stage,
                            following.index,
                            following.form,
                            following.amount,
                        ) == (next.stage, next.index, next.form, next.amount)
                    }),
                "request {position} got past at its allowance"
            );
        }
    }
}

fn check_stage(wire: &Value, stage: PackageReadStage, form: &str) {
    let read = reader(wire);
    let requests = stage_requests(&read, stage);
    assert!(
        requests.iter().all(|request| request.form == form),
        "{requests:?}"
    );
    assert_exact_and_one_short(&read, &requests);
}

#[trace("TC-472", "FR-038-AC-505")]
#[test]
fn source_map_tables_are_charged_one_byte_short_and_admitted_at_the_exact_allowance() {
    let wire = v2_all_families();
    check_stage(&wire, PackageReadStage::SourceMap, "source_map");
    let requests = stage_requests(&reader(&wire), PackageReadStage::SourceMap);
    assert!(
        requests.iter().any(|request| request.index > 0),
        "the per-entry region table is charged at its entry"
    );
}

#[trace("TC-472", "FR-038-AC-505")]
#[test]
fn capability_table_is_charged_at_the_capability_report() {
    check_stage(
        &v2_all_families(),
        PackageReadStage::Capabilities,
        "capability_report",
    );
}

#[trace("TC-472", "FR-038-AC-505")]
#[test]
fn equivalence_tables_are_charged_at_the_equivalence_stage() {
    check_stage(
        &crate::checked_package_v2_dimension_equivalence::package(),
        PackageReadStage::Equivalence,
        "equivalence",
    );
}

fn with_diagnostic() -> Value {
    let mut wire = v2_all_families();
    let node = wire["semantic_graph"]["nodes"][0]["node_id"].clone();
    let source = wire["lock"]["sources"][0].clone();
    wire["diagnostics"]["entries"] = json!([{
        "stage": "package_read", "code": "invalid_package", "cause_tag": "invalid-value",
        "details": [{"term": "reference", "target": node}],
        "loci": [{"source": source, "start": 0, "end": 1}]
    }]);
    wire
}

#[trace("TC-472", "FR-038-AC-506")]
#[test]
fn diagnostic_node_table_is_charged_at_the_diagnostic_entries() {
    check_stage(
        &with_diagnostic(),
        PackageReadStage::Diagnostics,
        "diagnostics.entries",
    );
}

fn effect_row(operation: &str) -> Value {
    json!({
        "document": "test/families",
        "declaring": node_id("fixture/1515"),
        "operation": operation,
        "effect": {"modifies": [], "creates": [], "deletes": [], "deletes_scoped": []},
        "provenance": {
            "origin": "model_document_operation_frame",
            "member": format!("ix://test/families/Account/{operation}")
        }
    })
}

fn with_effects() -> Value {
    let mut wire = v2_all_families();
    wire["operation_effects"] = json!([effect_row("deposit"), effect_row("withdraw")]);
    wire
}

#[trace("TC-472", "FR-038-AC-506")]
#[test]
fn operation_effect_tables_are_charged_at_their_rows() {
    let wire = with_effects();
    check_stage(
        &wire,
        PackageReadStage::OperationEffects,
        "operation_effects",
    );
}

#[trace("TC-472", "FR-038-AC-506")]
#[test]
fn checked_term_data_tables_are_charged_at_the_checked_term_data() {
    let mut wire = v2_all_families();
    wire["checked_term_data"] = json!({
        "equality_decisions": [], "protocol_locations": [], "dispatch_tables": [],
        "probabilistic_claims": []
    });
    check_stage(
        &wire,
        PackageReadStage::CheckedTermData,
        "checked_term_data",
    );
}

/// The smallest work limit under which `wire` admits.
fn exact_work(wire: &Value) -> u64 {
    let read = reader(wire);
    let (mut low, mut high) = (0_u64, u64::MAX / 2);
    while low + 1 < high {
        let middle = low + (high - low) / 2;
        let mut budget = NativeJoinBudget::new(middle, 0).with_read_storage_limit(u64::MAX);
        match read(&mut budget) {
            CheckedPackageV2BudgetedReadResult::Admitted(_) => high = middle,
            _ => low = middle,
        }
    }
    high
}

/// A work denial in a stage owns a pointer: it is charged to read storage at
/// the denial's locus with form `diagnostic` before it is built, and the
/// outcome is `exhausted` when that charge is denied (FR-038-AC-510).
#[trace("TC-472", "FR-038-AC-510")]
#[test]
fn work_denial_pointer_is_charged_before_it_is_built() {
    let wire = with_effects();
    let read = reader(&wire);
    let mut denied =
        NativeJoinBudget::new(exact_work(&wire) - 1, 0).with_read_storage_limit(u64::MAX);
    let CheckedPackageV2BudgetedReadResult::BudgetIncomplete(incomplete) = read(&mut denied) else {
        panic!("one unit of work short is a budget outcome");
    };
    assert_eq!(incomplete.denied.resource, PackageReadResource::Work);
    let path = incomplete.path.expect("the value whose charge failed");
    let full = denied.read_storage_consumed();

    // The pointer was paid for within the read's read storage: with exactly
    // that much the outcome is affordable, with one byte less it is
    // exhausted at the work denial's own locus.
    let mut exact = NativeJoinBudget::new(exact_work(&wire) - 1, 0).with_read_storage_limit(full);
    assert!(matches!(
        read(&mut exact),
        CheckedPackageV2BudgetedReadResult::BudgetIncomplete(_)
    ));
    let mut short =
        NativeJoinBudget::new(exact_work(&wire) - 1, 0).with_read_storage_limit(full - 1);
    let CheckedPackageV2BudgetedReadResult::Exhausted(exhausted) = read(&mut short) else {
        panic!("a pointer one byte short is exhausted");
    };
    assert_eq!(exhausted.denied.resource, PackageReadResource::Work);
    assert_eq!(exhausted.locus.form, "operation_effects");
    let NativeJoinLocusSource::PackageRead(stage, _) = exhausted.locus.source else {
        panic!("a package read locus");
    };
    assert!(
        stage == PackageReadStage::OperationEffects,
        "{stage:?} for {path}"
    );
}

/// A refusal a stage builds is an owned diagnostic: charged at the stage's
/// locus with form `diagnostic`, and `exhausted` with the refusal code pending
/// when that charge is denied (FR-038-AC-510).
#[trace("TC-472", "FR-038-AC-510")]
#[test]
fn stage_refusal_diagnostic_is_charged_and_exhausts_when_denied() {
    let mut wire = v2_all_families();
    wire["source_map"][3]["regions"] = json!([]);
    let read = reader(&wire);
    let mut unlimited = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(u64::MAX);
    let CheckedPackageV2BudgetedReadResult::Refused(refusal) = read(&mut unlimited) else {
        panic!("an entry without regions refuses");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::InvalidSourceMap);
    let total = unlimited.read_storage_consumed();

    let mut exact = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(total);
    assert!(matches!(
        read(&mut exact),
        CheckedPackageV2BudgetedReadResult::Refused(_)
    ));
    let mut short = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(total - 1);
    let CheckedPackageV2BudgetedReadResult::Exhausted(exhausted) = read(&mut short) else {
        panic!("a refusal diagnostic one byte short is exhausted");
    };
    assert_eq!(
        exhausted.pending,
        Some(CheckedPackagePendingOutcome::Refused(
            CheckedPackageRefusalCode::InvalidSourceMap
        ))
    );
    assert_eq!(
        (exhausted.locus.source, exhausted.locus.form),
        (
            NativeJoinLocusSource::PackageRead(PackageReadStage::SourceMap, 3),
            "diagnostic"
        )
    );
}

/// A package whose checked-term data holds one admitted equality decision.
fn with_equality_decision() -> Value {
    let members = nominal_fixture_members()
        .into_iter()
        .skip(2)
        .collect::<Vec<_>>();
    let mut wire = nominal_package(&members);
    let boolean = json!({"package":wire["name"],"path":["scalar_type","boolean"],"ordinal":0});
    let application = node_id("equality-application");
    let unit = node_id("qualified:Example/Metre");
    let dimension = node_id("qualified:Example/Length");
    let source = wire["lock"]["sources"][0].clone();
    let graph_nodes = wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    graph_nodes.push(json!({
        "node_id":boolean,"schema_version":"quire.checked-semantic-graph/v2",
        "node_tag":"scalar_type","semantic_form":"boolean","semantic_type":boolean,
        "dependencies":[],"occurrences":[{"role":"type","ordinal":0}],
        "body":{"term":"aggregate","members":[]}
    }));
    graph_nodes.push(json!({
        "node_id":application,"schema_version":"quire.checked-semantic-graph/v2",
        "node_tag":"expression","semantic_form":"binary","semantic_type":boolean,
        "dependencies":[unit],"occurrences":[{"role":"expression","ordinal":0}],
        "body":{"term":"application","operator":"binary",
            "operation":{"identity":"quire.op.quantity.eq","laws":[],"mode":null,
                "member":null,"leaves":[]},
            "result_type":boolean,
            "arguments":[{"term":"reference","target":unit},{"term":"reference","target":unit}]}
    }));
    let source_map = wire["source_map"].as_array_mut().expect("source map");
    source_map.push(json!({"node_id":boolean,"role":"type","ordinal":0,
        "regions":[{"source":source,"start":0,"end":1}]}));
    source_map.push(
        json!({"node_id":application,"role":"expression","ordinal":0,
        "regions":[{"source":source,"start":1,"end":2}]}),
    );
    wire["checked_term_data"] = json!({"equality_decisions":[{
        "application":application,"occurrence":{"role":"expression","ordinal":0},
        "term_path":[],"operator":"equal","schedule":"quantity",
        "left":{"operand":0,"source_type":unit,"conversion_target":null},
        "right":{"operand":1,"source_type":unit,"conversion_target":null},
        "units":[{"quantity_type":unit,"unit":unit,"form":{
            "kind":"declared","dimension_node":dimension,"dimension_terms":[],
            "root_unit":unit,"path":[]}}],"enum_members":[]
    }],"protocol_locations":[],"dispatch_tables":[],"probabilistic_claims":[]});
    wire
}

#[trace("TC-472", "FR-038-AC-506")]
#[test]
fn equality_decision_tables_are_charged_one_byte_short_at_the_checked_term_data() {
    let wire = with_equality_decision();
    check_stage(
        &wire,
        PackageReadStage::CheckedTermData,
        "checked_term_data",
    );
    let requests = stage_requests(&reader(&wire), PackageReadStage::CheckedTermData);
    // The node table, the source-map table and the decision table.
    assert!(requests.len() >= 3, "{requests:?}");
}
