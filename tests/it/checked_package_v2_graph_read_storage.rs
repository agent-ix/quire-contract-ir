// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The graph stage's read-storage charging: each family validator's heap
//! requests are charged to the caller's budget at `PackageRead(Graph, i)`
//! before they are made (TC-472).

use crate::support::checked_package::{canonical, evidence_for, v2_all_families};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedPackageReadLimits, CheckedPackageV2, CheckedPackageV2BudgetedReadResult,
    NativeJoinBudget, NativeJoinLocusSource, PackageReadStage,
};

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

/// How a fixture is read under a budget.
pub type Reader<'a> = &'a dyn Fn(&mut NativeJoinBudget) -> CheckedPackageV2BudgetedReadResult;

fn all_families(budget: &mut NativeJoinBudget) -> CheckedPackageV2BudgetedReadResult {
    let value = v2_all_families();
    CheckedPackageV2::read_with_budget(
        &canonical(&value),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&value),
        budget,
    )
}

fn read_at(read: Reader<'_>, limit: u64) -> (CheckedPackageV2BudgetedReadResult, NativeJoinBudget) {
    let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
    let result = read(&mut budget);
    (result, budget)
}

/// The request a read with `limit` bytes of read storage stops at, if any.
pub fn denial_at(read: Reader<'_>, limit: u64) -> Option<Request> {
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
/// limit to exactly admit each request in turn. The first request of the
/// stage is found by bisection, since the stage a read reaches never falls as
/// the limit rises.
pub fn stage_requests(read: Reader<'_>, stage: PackageReadStage) -> Vec<Request> {
    let total = {
        let (result, budget) = read_at(read, u64::MAX);
        assert!(matches!(
            result,
            CheckedPackageV2BudgetedReadResult::Admitted(_)
        ));
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
pub fn assert_exact_and_one_short(read: Reader<'_>, requests: &[Request]) {
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

/// The graph-stage requests of the all-families fixture.
fn graph_requests() -> Vec<Request> {
    stage_requests(&all_families, PackageReadStage::Graph)
}

/// Every graph-stage request is charged before it is made (FR-038-AC-504).
#[trace("TC-472", "FR-038-AC-504")]
#[test]
fn each_graph_request_is_denied_one_byte_short_and_admitted_at_its_exact_allowance() {
    let requests = graph_requests();
    assert!(!requests.is_empty(), "the graph stage makes requests");
    assert_exact_and_one_short(&all_families, &requests);
}

/// The graph index, the per-node tables and the state step's walk and anchor
/// table are each charged at their own node and form.
#[trace("TC-472", "FR-038-AC-504")]
#[test]
fn the_graph_index_state_tables_and_walks_are_charged_at_graph_loci() {
    let requests = graph_requests();
    let at = |form: &str| {
        requests
            .iter()
            .filter(|request| request.form == form)
            .count()
    };
    assert!(at("walk") > 0, "a term walk grows its lists on the budget");
    assert!(
        at("state") > 0,
        "the state step builds its tables on the budget"
    );
    let first = requests.first().expect("requests");
    assert_eq!(first.index, 0, "the index table is charged at node 0");
    assert!(requests
        .iter()
        .all(|request| request.stage == PackageReadStage::Graph));
}
