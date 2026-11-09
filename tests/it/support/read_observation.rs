// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The allocation observation shared by registry and original-driver proofs.

use quire_contract_model::{
    CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageV2,
    CheckedPackageV2BudgetedReadResult, NativeJoinBudget,
};

pub(crate) trait Counter {
    fn measure(read: impl FnOnce()) -> u64;
}
/// One complete public read. Fixtures, budget construction and comparison
/// live outside the counter; the returned outcome remains owned by the caller.
pub(crate) struct Observation {
    pub(crate) result: CheckedPackageV2BudgetedReadResult,
    pub(crate) budget: NativeJoinBudget,
    pub(crate) requested: u64,
}

pub(crate) fn measure_at<C: Counter>(
    bytes: &[u8],
    evidence: &CheckedPackageEvidence,
    limits: CheckedPackageReadLimits,
    work: u64,
    read_storage: u64,
) -> Observation {
    let mut budget = NativeJoinBudget::new(work, 0).with_read_storage_limit(read_storage);

    let mut result = None;
    let requested = C::measure(|| {
        result = Some(CheckedPackageV2::read_with_budget(
            bytes,
            limits,
            evidence,
            &mut budget,
        ))
    });
    Observation {
        result: result.expect("read ran"),
        budget,
        requested,
    }
}

/// Heap a read may request beyond the read storage its budget admitted, in
/// bytes. The allowance does not depend on the size of the input.
///
/// Measured over the reads that run under `make test`: the dedicated counter
/// binary's roughly twelve thousand observations request no unpaid heap, and
/// the integration binary's caller-budget reads (about fifty-four thousand
/// observations) request none except a constant 1175 bytes in 264 reads, which
/// does not change with the input (three refinement and frame-entry reads
/// request 5101 to 15479 and are real failures of the bound). Four kibibytes is about three and
/// a half times that, and under half a percent of a one-mebibyte input.
pub(crate) const UNPAID_BOUND: u64 = 4_096;

/// Bytes requested during the read that the budget did not admit.
pub(crate) fn unpaid(observation: &Observation) -> u64 {
    observation
        .requested
        .saturating_sub(observation.budget.read_storage_consumed())
}

pub(crate) fn accounted(label: &str, observation: &Observation) {
    assert_eq!(
        observation.budget.consumed().storage,
        0,
        "{label}: a fresh measured read consumes no native logical storage"
    );
    assert!(
        unpaid(observation) <= UNPAID_BOUND,
        "{label}: unpaid read memory {} exceeds the fixed bound {UNPAID_BOUND}",
        unpaid(observation)
    );
}

/// Destruction is measured separately from AC-511's during-call total. A
/// destructor must not hide a further heap request after the read returns.
pub(crate) fn drop_without_request<C: Counter>(observation: Observation) {
    let requested = C::measure(|| drop(observation.result));
    assert_eq!(
        requested, 0,
        "dropping the returned outcome allocates nothing"
    );
}
