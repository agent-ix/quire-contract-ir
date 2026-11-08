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
    let observation = measure_at_unchecked::<C>(bytes, evidence, limits, work, read_storage);
    accounted("checked measured public reader", &observation);
    observation
}

/// Diagnostic primitive, deliberately not a trusted census boundary. Any
/// direct use adds a production reader entry without a checked measure_at
/// entry and therefore fails normative compiler-census arithmetic.
pub(crate) fn measure_at_unchecked<C: Counter>(
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

pub(crate) fn accounted(label: &str, observation: &Observation) {
    assert_eq!(
        observation.budget.consumed().storage,
        0,
        "{label}: a fresh measured read consumes no native logical storage"
    );
    let outcome = match &observation.result {
        CheckedPackageV2BudgetedReadResult::Admitted(_) => "admitted",
        CheckedPackageV2BudgetedReadResult::Refused(_) => "refused",
        CheckedPackageV2BudgetedReadResult::Incomplete(_) => "incomplete",
        CheckedPackageV2BudgetedReadResult::BudgetIncomplete(_) => "budget incomplete",
        CheckedPackageV2BudgetedReadResult::Exhausted(_) => "exhausted",
        CheckedPackageV2BudgetedReadResult::CatalogUnavailable(_) => "catalog unavailable",
    };
    let refusal_context = match &observation.result {
        CheckedPackageV2BudgetedReadResult::Refused(refusal) => Some((
            &refusal.code,
            &refusal.cause,
            refusal
                .loci
                .first()
                .and_then(|locus| locus.path.as_ref().map(|path| path.as_str())),
        )),
        _ => None,
    };
    assert_eq!(observation.requested,observation.budget.read_storage_consumed(),"{label}: actual during-call allocation requests equal admitted read storage; outcome {outcome}; first unpaid {:?}/{:?}; refusal context {refusal_context:?}",observation.budget.first_unpaid_read_storage(),observation.budget.first_unpaid_locus());
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
