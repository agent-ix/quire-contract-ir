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

/// General bound for non-refusal-diagnostic bytes requested beyond the
/// read-storage amount admitted by the budget.
pub(crate) const UNPAID_BOUND: u64 = 4_096;

/// The independent bound for unpaid refusal-diagnostic allocations exercised
/// by semantic refusal reads. It leaves headroom above the largest measured
/// diagnostic while remaining below the general non-diagnostic bound.
pub(crate) const REFUSAL_DIAGNOSTIC_UNPAID_BOUND: u64 = 2_048;

/// Bytes requested during the read that the budget did not admit.
pub(crate) fn unpaid(observation: &Observation) -> u64 {
    observation
        .requested
        .saturating_sub(observation.budget.read_storage_consumed())
}

pub(crate) fn accounted(label: &str, observation: &Observation) {
    accounted_with_bound(label, observation, UNPAID_BOUND, "unpaid read memory");
}

/// Account a read whose entire unpaid remainder is the refusal diagnostic.
pub(crate) fn accounted_refusal_diagnostic(label: &str, observation: &Observation) {
    accounted_with_bound(
        label,
        observation,
        REFUSAL_DIAGNOSTIC_UNPAID_BOUND,
        "unpaid refusal-diagnostic memory",
    );
}

fn accounted_with_bound(label: &str, observation: &Observation, bound: u64, description: &str) {
    assert_eq!(
        observation.budget.consumed().storage,
        0,
        "{label}: a fresh measured read consumes no native logical storage"
    );
    assert!(
        unpaid(observation) <= bound,
        "{label}: {description} {} exceeds its independent bound {bound}",
        unpaid(observation),
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
