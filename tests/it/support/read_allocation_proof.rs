// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! One allocation oracle shared by the dedicated counter binary and the
//! existing integration binary's global allocation-counter backend.

use quire_contract_model::NativeJoinLocusSource;
use quire_contract_model::{
    initialize_operation_catalog, CheckedPackageEvidence, CheckedPackageReadLimits,
    CheckedPackageRefusalCode, CheckedPackageV2, CheckedPackageV2BudgetedReadResult,
    CheckedPackageV2ReadResult, NativeJoinBudget,
};

use crate::qspec_read_proof::observation;
pub(crate) use observation::{
    accounted, accounted_refusal_diagnostic, drop_without_request, measure_at, unpaid, Counter,
    Observation, REFUSAL_DIAGNOSTIC_UNPAID_BOUND, UNPAID_BOUND,
};

#[derive(Clone, Copy)]
pub(crate) enum Expected {
    Admitted,
    Refused(CheckedPackageRefusalCode),
    #[allow(dead_code)] // The whole-read binary constructs it; the allocation binary does not.
    OriginalOracle(fn(&CheckedPackageV2ReadResult) -> bool),
}

pub(crate) fn measure_read<C: Counter>(
    bytes: &[u8],
    evidence: &CheckedPackageEvidence,
    limits: CheckedPackageReadLimits,
    limit: u64,
) -> Observation {
    measure_at::<C>(bytes, evidence, limits, u64::MAX, limit)
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

pub(crate) fn assert_read<C: Counter>(
    label: &str,
    bytes: &[u8],
    evidence: &CheckedPackageEvidence,
    expected: Expected,
) {
    initialize_operation_catalog().expect("the specified catalogue gate succeeds");
    let limits = generous_limits();
    let account = if matches!(expected, Expected::Refused(_)) {
        accounted_refusal_diagnostic
    } else {
        accounted
    };
    // Initialize only the specified catalog gate; do not use an unmeasured
    // public package read to warm per-read owned storage.
    let reference = measure_read::<C>(bytes, evidence, limits, u64::MAX);
    let plain = crate::read_and_account(bytes, limits, evidence);
    match (&plain, &reference.result, expected) {
        (
            CheckedPackageV2ReadResult::Admitted(left),
            CheckedPackageV2BudgetedReadResult::Admitted(right),
            Expected::Admitted,
        ) => crate::qspec_read_proof::assert_admitted_views_equal(left, right, label),
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
        (plain, measured, Expected::OriginalOracle(predicate)) => {
            assert!(predicate(plain), "{label}: original driver predicate");
            match (plain, measured) {
                (
                    CheckedPackageV2ReadResult::Admitted(a),
                    CheckedPackageV2BudgetedReadResult::Admitted(b),
                ) => crate::qspec_read_proof::assert_admitted_views_equal(a, b, label),
                (
                    CheckedPackageV2ReadResult::Refused(a),
                    CheckedPackageV2BudgetedReadResult::Refused(b),
                ) => assert_eq!(a, b, "{label}: original refused full value"),
                (
                    CheckedPackageV2ReadResult::Incomplete(a),
                    CheckedPackageV2BudgetedReadResult::Incomplete(b),
                ) => assert_eq!(a, b, "{label}: original incomplete full value"),
                (plain, measured) => {
                    panic!("{label}: original driver outcome differs: {plain:?}/{measured:?}")
                }
            }
        }
        (plain, measured, _) => panic!(
            "{label}: fixture outcome differs from its required class: {plain:?}/{measured:?}"
        ),
    }
    account(label, &reference);
    let total = reference.budget.read_storage_consumed();
    assert!(total > 0);
    let work = reference.budget.consumed().work;
    for (work_limit, storage_limit) in [
        (work, total),
        (u64::MAX, total),
        (work, u64::MAX),
        (u64::MAX, u64::MAX),
    ] {
        let exact = measure_at::<C>(bytes, evidence, limits, work_limit, storage_limit);
        crate::qspec_read_proof::assert_budgeted_results_equal(
            &exact.result,
            &reference.result,
            label,
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
        account(label, &exact);
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
    account(label, &short);
    drop_without_request::<C>(short);
    drop_without_request::<C>(reference);
}

/// Observe every actual storage request once, then verify the final exact and
/// one-below request boundaries. The observer avoids replaying the full read
/// once for every reservation.
pub(crate) fn every_request<C: Counter>(bytes: &[u8], evidence: &CheckedPackageEvidence) -> u64 {
    #[cfg(feature = "test-support")]
    {
        every_request_from_observer::<C>(bytes, evidence)
    }
    #[cfg(not(feature = "test-support"))]
    {
        every_request_by_replay::<C>(bytes, evidence)
    }
}

#[cfg(feature = "test-support")]
fn every_request_from_observer<C: Counter>(bytes: &[u8], evidence: &CheckedPackageEvidence) -> u64 {
    initialize_operation_catalog().expect("catalogue gate");
    let limits = generous_limits();
    let (reference, events) =
        crate::qspec_read_proof::observe_native::<C>(bytes, evidence, limits, u64::MAX, u64::MAX);
    assert!(
        !matches!(
            reference.result,
            CheckedPackageV2BudgetedReadResult::BudgetIncomplete(_)
                | CheckedPackageV2BudgetedReadResult::Exhausted(_)
        ),
        "ample reference reaches its semantic outcome"
    );
    let is_refusal = matches!(
        &reference.result,
        CheckedPackageV2BudgetedReadResult::Refused(_)
    );
    let account = if is_refusal {
        accounted_refusal_diagnostic
    } else {
        accounted
    };
    account("ample reference", &reference);

    let mut consumed = 0_u64;
    let mut last_storage_event = None;
    let mut storage_requests = 0_usize;
    for event in &events {
        if event.resource != quire_contract_model::NativeBudgetResource::ReadStorage {
            continue;
        }
        assert_eq!(
            event.consumed_before, consumed,
            "observer records storage requests in budget order"
        );
        storage_requests += 1;
        if event.amount == 0 {
            continue;
        }
        assert!(event.admitted, "unlimited storage request is admitted");
        assert!(
            event.locus.is_some_and(|locus| matches!(
                locus.source,
                NativeJoinLocusSource::PackageRead(..)
            )),
            "every observed storage request has its package-read locus"
        );
        consumed = consumed
            .checked_add(u64::try_from(event.amount).expect("bounded request"))
            .expect("bounded total read storage");
        last_storage_event = Some(*event);
    }
    assert!(storage_requests > 0, "the read makes storage requests");
    assert_eq!(
        consumed,
        reference.budget.read_storage_consumed(),
        "observed requests equal the admitted storage delta"
    );
    let final_request = last_storage_event.expect("at least one storage request");
    let final_amount = u64::try_from(final_request.amount).expect("bounded final request");
    assert_eq!(
        final_request.consumed_before.checked_add(final_amount),
        Some(consumed),
        "the final observed request reaches the exact read-storage delta"
    );

    let exact = measure_read::<C>(bytes, evidence, limits, consumed);
    crate::qspec_read_proof::assert_budgeted_results_equal(
        &exact.result,
        &reference.result,
        "final exact storage boundary",
    );
    account("final exact storage boundary", &exact);
    let short_limit = consumed
        .checked_sub(1)
        .expect("positive read-storage total");
    let short = measure_read::<C>(bytes, evidence, limits, short_limit);
    assert_eq!(
        short.budget.first_unpaid_read_storage(),
        Some(u128::from(final_amount)),
        "one below the final boundary denies its observed request"
    );
    assert_eq!(
        short.budget.first_unpaid_locus(),
        final_request.locus,
        "one below the final boundary retains its observed locus"
    );
    assert_eq!(
        short.budget.read_storage_consumed(),
        final_request.consumed_before,
        "a denied request does not advance admitted storage"
    );
    assert!(
        !matches!(
            short.result,
            CheckedPackageV2BudgetedReadResult::Admitted(_)
        ),
        "one below the final request cannot admit the package"
    );
    account("one below final storage boundary", &short);
    let maximum_unpaid = unpaid(&reference).max(unpaid(&exact)).max(unpaid(&short));
    drop_without_request::<C>(exact);
    drop_without_request::<C>(short);
    drop_without_request::<C>(reference);
    eprintln!(
        "Observed {storage_requests} actual storage reservations; checked final exact and one-below boundaries"
    );
    maximum_unpaid
}

#[cfg(not(feature = "test-support"))]
fn every_request_by_replay<C: Counter>(bytes: &[u8], evidence: &CheckedPackageEvidence) -> u64 {
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
    let is_refusal = matches!(
        &reference.result,
        CheckedPackageV2BudgetedReadResult::Refused(_)
    );
    let account = if is_refusal {
        accounted_refusal_diagnostic
    } else {
        accounted
    };
    account("ample reference", &reference);
    let mut maximum_unpaid = unpaid(&reference);
    let total = reference.budget.read_storage_consumed();
    let mut before = 0;
    loop {
        let stopped = measure_read::<C>(bytes, evidence, limits, before);
        account("exact preceding request boundary", &stopped);
        maximum_unpaid = maximum_unpaid.max(unpaid(&stopped));
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
        // No allowance remains at this exact prefix. A refusal may still
        // build its uncharged semantic diagnostic; its separate bound was
        // applied above. Other outcomes must not grow unpaid memory.
        assert_eq!(stopped.budget.read_storage_consumed(), before);
        if !is_refusal {
            assert!(
                unpaid(&stopped) <= unpaid(&reference),
                "a denied request allocates nothing beyond the ample read's own unpaid memory: asked={asked}, locus={locus:?}, requested={}, charged={}, denied_result={:?}, ample_unpaid={}",
                stopped.requested,
                stopped.budget.read_storage_consumed(),
                stopped.result,
                unpaid(&reference),
            );
        }
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
        account("one below actual reservation", &short);
        maximum_unpaid = maximum_unpaid.max(unpaid(&short));
        drop_without_request::<C>(stopped);
        drop_without_request::<C>(short);
        before = next;
    }
    drop_without_request::<C>(reference);
    maximum_unpaid
}

/// Repeated public calls compose in one caller-owned budget; every counter
/// interval is compared with that call's admitted storage delta.
pub(crate) fn shared_budget_reads<C: Counter>(bytes: &[u8], evidence: &CheckedPackageEvidence) {
    initialize_operation_catalog().expect("catalogue gate");
    let limits = generous_limits();
    let reference = measure_read::<C>(bytes, evidence, limits, u64::MAX);
    assert!(matches!(
        reference.result,
        CheckedPackageV2BudgetedReadResult::Admitted(_)
    ));
    let storage = reference.budget.read_storage_consumed();
    let work = reference.budget.consumed().work;
    let mut observations = Vec::new();
    for limit in [
        storage.checked_mul(2).unwrap(),
        storage.checked_mul(2).unwrap() - 1,
    ] {
        let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
        for call in 0..2 {
            let before_storage = budget.read_storage_consumed();
            let before_work = budget.consumed();
            let mut result = None;
            let requested = C::measure(|| {
                result = Some(CheckedPackageV2::read_with_budget(
                    bytes,
                    limits,
                    evidence,
                    &mut budget,
                ));
            });
            let result = result.expect("read ran");
            let delta = budget.read_storage_consumed() - before_storage;
            eprintln!("shared budget limit={limit} call={call} requested={requested} storage_delta={delta} work_before={before_work:?} work_after={:?}", budget.consumed());
            if call == 0 || limit == storage * 2 {
                assert_eq!(result, reference.result);
                assert_eq!(delta, storage);
                assert_eq!(budget.consumed().work - before_work.work, work);
            } else {
                assert!(!matches!(
                    result,
                    CheckedPackageV2BudgetedReadResult::Admitted(_)
                ));
                assert!(budget.first_unpaid_read_storage().is_some());
            }
            assert_eq!(budget.consumed().storage, 0, "logical storage remains zero");
            observations.push((requested, delta, result));
        }
    }
    for (requested, delta, result) in observations {
        assert!(
            requested.saturating_sub(delta) <= UNPAID_BOUND,
            "per-call allocation exceeds the caller-budget delta by no more than the fixed bound"
        );
        assert_eq!(C::measure(|| drop(result)), 0);
    }
}
