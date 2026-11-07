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

fn read_at(limit: u64) -> (CheckedPackageV2BudgetedReadResult, NativeJoinBudget) {
    let value = v2_all_families();
    let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
    let result = CheckedPackageV2::read_with_budget(
        &canonical(&value),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&value),
        &mut budget,
    );
    (result, budget)
}

/// The request a read with `limit` bytes of read storage stops at, if any.
fn denial_at(limit: u64) -> Option<Request> {
    let (result, budget) = read_at(limit);
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
fn stage_requests(stage: PackageReadStage) -> Vec<Request> {
    let total = {
        let (result, budget) = read_at(u64::MAX);
        assert!(matches!(
            result,
            CheckedPackageV2BudgetedReadResult::Admitted(_)
        ));
        budget.read_storage_consumed()
    };
    let (mut low, mut high) = (0_u64, total);
    while low < high {
        let middle = low + (high - low) / 2;
        match denial_at(middle) {
            Some(request) if request.stage < stage => low = middle + 1,
            _ => high = middle,
        }
    }
    let mut limit = low;
    let mut found = Vec::new();
    while let Some(mut request) = denial_at(limit) {
        if request.stage > stage {
            break;
        }
        // The limit that admitted everything before this request is the
        // storage consumed before it; walking from the bisected limit the
        // first request's `before` is that limit.
        request.before = limit;
        limit += u64::try_from(request.amount).expect("amount fits");
        found.push(request);
    }
    found
}

fn graph_requests() -> Vec<Request> {
    stage_requests(PackageReadStage::Graph)
}

/// Every graph-stage request is charged before it is made: an allowance
/// exactly covering a request gets past it, one byte short is denied at it
/// with its locus and amount, and both limits are decided by the same
/// request (FR-038-AC-504).
#[trace("TC-472", "FR-038-AC-504")]
#[test]
fn each_graph_request_is_denied_one_byte_short_and_admitted_at_its_exact_allowance() {
    let requests = graph_requests();
    assert!(!requests.is_empty(), "the graph stage makes requests");
    for (position, request) in requests.iter().enumerate() {
        let exact = request.before + u64::try_from(request.amount).unwrap();
        let short = denial_at(exact - 1).expect("one byte short is denied");
        assert_eq!(
            (short.stage, short.index, short.form, short.amount),
            (request.stage, request.index, request.form, request.amount),
            "request {position}"
        );
        match denial_at(exact) {
            None => {}
            Some(next) => assert!(
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
            ),
        }
    }
}

/// The forms the graph stage's requests are charged under, for a coverage
/// view of which validators charge.
fn forms(requests: &[Request]) -> std::collections::BTreeSet<&'static str> {
    requests.iter().map(|request| request.form).collect()
}

#[test]
fn show_graph_forms() {
    let requests = graph_requests();
    eprintln!("{} requests; forms {:?}", requests.len(), forms(&requests));
}
