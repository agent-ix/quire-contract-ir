// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! One allocation oracle shared by the dedicated counter binary and the
//! existing integration binary's global allocation-counter backend.

use quire_contract_model::{
    initialize_operation_catalog, CheckedPackageEvidence, CheckedPackageReadLimits,
    CheckedPackageRefusalCode, CheckedPackageV2, CheckedPackageV2BudgetedReadResult,
    CheckedPackageV2ReadResult, NativeJoinBudget, NativeJoinLocusSource,
};

pub(crate) trait Counter {
    fn measure(read: impl FnOnce()) -> u64;
}
#[derive(Clone, Copy)]
pub(crate) enum Expected {
    Admitted,
    Refused(CheckedPackageRefusalCode),
}

/// One complete public read. Fixtures, budget construction and comparison
/// live outside the counter; the returned outcome remains owned by the caller.
pub(crate) struct Observation {
    pub(crate) result: CheckedPackageV2BudgetedReadResult,
    pub(crate) budget: NativeJoinBudget,
    pub(crate) requested: u64,
}

pub(crate) fn measure_read<C: Counter>(
    bytes: &[u8],
    evidence: &CheckedPackageEvidence,
    limits: CheckedPackageReadLimits,
    limit: u64,
) -> Observation {
    let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
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

pub(crate) fn generous_limits() -> CheckedPackageReadLimits {
    CheckedPackageReadLimits {
        bytes: 1 << 30,
        nodes: 1 << 24,
        edges: 1 << 26,
        occurrences: 1 << 26,
        diagnostics: 1 << 24,
        work: 1 << 50,
    }
}

pub(crate) fn accounted(label: &str, observation: &Observation) {
    let outcome = match &observation.result {
        CheckedPackageV2BudgetedReadResult::Admitted(_) => "admitted",
        CheckedPackageV2BudgetedReadResult::Refused(_) => "refused",
        CheckedPackageV2BudgetedReadResult::Incomplete(_) => "incomplete",
        CheckedPackageV2BudgetedReadResult::BudgetIncomplete(_) => "budget incomplete",
        CheckedPackageV2BudgetedReadResult::Exhausted(_) => "exhausted",
        CheckedPackageV2BudgetedReadResult::CatalogUnavailable(_) => "catalog unavailable",
    };
    assert_eq!(observation.requested,observation.budget.read_storage_consumed(),"{label}: actual during-call allocation requests equal admitted read storage; outcome {outcome}; first unpaid {:?}/{:?}",observation.budget.first_unpaid_read_storage(),observation.budget.first_unpaid_locus());
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

pub(crate) fn assert_read<C: Counter>(
    label: &str,
    bytes: &[u8],
    evidence: &CheckedPackageEvidence,
    expected: Expected,
) {
    initialize_operation_catalog().expect("the specified catalogue gate succeeds");
    let limits = generous_limits();
    // Initialize only the specified catalog gate; do not use an unmeasured
    // public package read to warm per-read owned storage.
    let reference = measure_read::<C>(bytes, evidence, limits, u64::MAX);
    let plain = CheckedPackageV2::read(bytes, limits, evidence);
    match (&plain, &reference.result, expected) {
        (
            CheckedPackageV2ReadResult::Admitted(left),
            CheckedPackageV2BudgetedReadResult::Admitted(right),
            Expected::Admitted,
        ) => assert_eq!(left, right, "{label}: full admitted value"),
        (
            CheckedPackageV2ReadResult::Refused(left),
            CheckedPackageV2BudgetedReadResult::Refused(right),
            Expected::Refused(code),
        ) => {
            assert_eq!(
                left.code, code,
                "{label}: genuine refusal input has its claimed code"
            );
            assert_eq!(left, right, "{label}: complete refusal/first fault");
        }
        (plain, measured, _) => panic!(
            "{label}: fixture outcome differs from its required class: {plain:?}/{measured:?}"
        ),
    }
    accounted(label, &reference);
    let total = reference.budget.read_storage_consumed();
    assert!(total > 0);
    for limit in [total, u64::MAX] {
        let exact = measure_read::<C>(bytes, evidence, limits, limit);
        assert_eq!(
            exact.result, reference.result,
            "{label}: repeated/exact full value"
        );
        assert_eq!(
            exact.budget.consumed(),
            reference.budget.consumed(),
            "{label}: repeat work"
        );
        assert_eq!(
            exact.budget.read_storage_consumed(),
            total,
            "{label}: catalogue and every owned table are charged again"
        );
        accounted(label, &exact);
        drop_without_request::<C>(exact);
    }
    let short = measure_read::<C>(bytes, evidence, limits, total - 1);
    assert!(
        !matches!(
            short.result,
            CheckedPackageV2BudgetedReadResult::Admitted(_)
        ),
        "one short refuses"
    );
    assert!(short.budget.first_unpaid_read_storage().is_some());
    accounted(label, &short);
    drop_without_request::<C>(short);
    drop_without_request::<C>(reference);
}

/// Probe actual requests in runtime order. Stage labels may decrease; no
/// numeric stage-order bisection or phase assumption is used.
pub(crate) fn every_request<C: Counter>(bytes: &[u8], evidence: &CheckedPackageEvidence) {
    initialize_operation_catalog().expect("catalogue gate");
    let limits = generous_limits();
    let reference = measure_read::<C>(bytes, evidence, limits, u64::MAX);
    assert!(
        !matches!(
            reference.result,
            CheckedPackageV2BudgetedReadResult::BudgetIncomplete(_)
                | CheckedPackageV2BudgetedReadResult::Exhausted(_)
        ),
        "ample reference reaches its semantic outcome"
    );
    let total = reference.budget.read_storage_consumed();
    let mut before = 0;
    loop {
        let stopped = measure_read::<C>(bytes, evidence, limits, before);
        accounted("exact preceding request boundary", &stopped);
        if before == total {
            assert_eq!(stopped.result, reference.result);
            drop_without_request::<C>(stopped);
            break;
        }
        let asked = stopped
            .budget
            .first_unpaid_read_storage()
            .expect("first denied storage request");
        let locus = stopped
            .budget
            .first_unpaid_locus()
            .expect("first denied request locus");
        assert!(matches!(
            locus.source,
            NativeJoinLocusSource::PackageRead(..)
        ));
        // No allowance remains at this exact prefix, so a positive-sized
        // diagnostic cannot allocate after the denied request.
        assert_eq!(stopped.budget.read_storage_consumed(), before);
        assert_eq!(
            stopped.requested, before,
            "denied request allocates nothing"
        );
        let next = before
            .checked_add(u64::try_from(asked).expect("bounded request"))
            .expect("bounded total");
        assert!(next > before && next <= total);
        let short = measure_read::<C>(bytes, evidence, limits, next - 1);
        assert_eq!(short.budget.first_unpaid_read_storage(), Some(asked));
        assert_eq!(short.budget.first_unpaid_locus(), Some(locus));
        let diagnostic = match &short.result {
            CheckedPackageV2BudgetedReadResult::BudgetIncomplete(outcome) => outcome
                .path
                .as_ref()
                .map_or(0, |path| path.as_str().len() as u64),
            CheckedPackageV2BudgetedReadResult::Exhausted(_) => 0,
            other => panic!("one below actual request boundary: {other:?}"),
        };
        assert_eq!(
            short.budget.read_storage_consumed(),
            before + diagnostic,
            "only the charged outcome pointer follows a denial"
        );
        accounted("one below actual reservation", &short);
        let again = measure_read::<C>(bytes, evidence, limits, next - 1);
        assert_eq!(
            again.result, short.result,
            "same first fault and full refusal value"
        );
        assert_eq!(again.budget.first_unpaid_locus(), Some(locus));
        assert_eq!(
            again.budget.read_storage_consumed(),
            short.budget.read_storage_consumed()
        );
        accounted("repeated denial", &again);
        drop_without_request::<C>(stopped);
        drop_without_request::<C>(short);
        drop_without_request::<C>(again);
        before = next;
    }
    drop_without_request::<C>(reference);
}
