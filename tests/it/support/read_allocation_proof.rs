// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! One allocation oracle shared by the dedicated counter binary and the
//! existing integration binary's global allocation-counter backend.

use quire_contract_model::{
    initialize_operation_catalog, CheckedPackageEvidence, CheckedPackageReadLimits,
    CheckedPackageRefusalCode, CheckedPackageV2, CheckedPackageV2BudgetedReadResult,
    CheckedPackageV2ReadResult, NativeJoinBudget, NativeJoinLocusSource,
};

use crate::qspec_read_proof::observation;
pub(crate) use observation::{accounted, drop_without_request, measure_at, Counter, Observation};

#[derive(Clone, Copy)]
pub(crate) enum Expected {
    Admitted,
    Refused(CheckedPackageRefusalCode),
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
    accounted(label, &reference);
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

/// Count work denials as well as storage denials. The pointer-free diagnostic
/// may cost zero; a positive-sized pointer is denied with exactly its preceding
/// storage allowance and must preserve the original work charge and locus.
/// Returns false only when this input reaches no work charge at all.
pub(crate) fn work_denials<C: Counter>(
    label: &str,
    bytes: &[u8],
    evidence: &CheckedPackageEvidence,
) -> bool {
    initialize_operation_catalog().expect("catalog gate");
    let limits = generous_limits();
    let reference = measure_at::<C>(bytes, evidence, limits, u64::MAX, u64::MAX);
    let work = reference.budget.consumed().work;
    if work == 0 {
        assert!(reference.budget.first_unpaid().is_none());
        drop_without_request::<C>(reference);
        return false;
    }
    let exact = measure_at::<C>(bytes, evidence, limits, work, u64::MAX);
    assert_eq!(
        exact.result, reference.result,
        "{label}: exact work preserves complete outcome"
    );
    let short = measure_at::<C>(bytes, evidence, limits, work - 1, u64::MAX);
    let denied = short
        .budget
        .first_unpaid()
        .expect("actual first unpaid work charge");
    assert_eq!(
        denied.kind,
        quire_contract_model::NativeJoinChargeKind::Work
    );
    let quire_contract_model::CheckedPackageV2BudgetedReadResult::BudgetIncomplete(diagnostic) =
        &short.result
    else {
        panic!("ample storage affords work diagnostic: {:?}", short.result);
    };
    assert_eq!(
        diagnostic.denied.resource,
        quire_contract_model::PackageReadResource::Work
    );
    assert_eq!(diagnostic.denied.amount, denied.amount);
    let repeated = measure_at::<C>(bytes, evidence, limits, work - 1, u64::MAX);
    assert_eq!(
        repeated.result, short.result,
        "{label}: repeated work first fault"
    );
    assert_eq!(repeated.budget.first_unpaid(), Some(denied));
    assert_eq!(
        repeated.budget.first_unpaid_locus(),
        short.budget.first_unpaid_locus()
    );
    let zero_storage = measure_at::<C>(bytes, evidence, limits, work - 1, 0);
    assert_eq!(zero_storage.requested, 0);
    assert_eq!(zero_storage.budget.read_storage_consumed(), 0);
    assert!(zero_storage.budget.first_unpaid_read_storage().is_some());
    // Advance actual work requests until an owned pointer exists. Do not
    // infer runtime order from PackageReadStage discriminants.
    let mut prefix = 0;
    let mut work_probes = 0_u64;
    let pointer_denial = loop {
        work_probes += 1;
        if work_probes.is_multiple_of(64) {
            eprintln!("{label}: work boundary probes={work_probes} prefix={prefix}");
        }
        let observation = measure_at::<C>(bytes, evidence, limits, prefix, u64::MAX);
        if matches!(&observation.result,CheckedPackageV2BudgetedReadResult::BudgetIncomplete(d) if d.path.as_ref().is_some_and(|p|!p.as_str().is_empty()))
        {
            break observation;
        }
        let unpaid = observation
            .budget
            .first_unpaid()
            .expect("work charge before semantic outcome");
        assert_eq!(
            unpaid.kind,
            quire_contract_model::NativeJoinChargeKind::Work
        );
        let next = observation
            .budget
            .consumed()
            .work
            .checked_add(u64::try_from(unpaid.amount).expect("bounded work charge"))
            .expect("bounded work prefix");
        assert!(
            next > prefix && next < work,
            "a genuine owned-pointer work denial must be reached"
        );
        prefix = next;
        drop_without_request::<C>(observation);
    };
    let CheckedPackageV2BudgetedReadResult::BudgetIncomplete(pointer) = &pointer_denial.result
    else {
        unreachable!("selected pointer outcome")
    };
    let bytes_of_pointer =
        u64::try_from(pointer.path.as_ref().expect("owned pointer").as_str().len())
            .expect("bounded pointer");
    assert!(bytes_of_pointer > 0);
    // Discover the storage prefix at the fixed work request from actual
    // unpaid reservations. Do not assume the implementation already charged
    // the pointer correctly: that is precisely what this oracle checks.
    let mut before_pointer = 0;
    let mut storage_probes = 0_u64;
    let exhausted = loop {
        storage_probes += 1;
        if storage_probes.is_multiple_of(64) {
            eprintln!("{label}: storage boundary probes={storage_probes} prefix={before_pointer}");
        }
        let observation = measure_at::<C>(bytes, evidence, limits, prefix, before_pointer);
        if observation
            .budget
            .first_unpaid()
            .is_some_and(|charge| charge.kind == quire_contract_model::NativeJoinChargeKind::Work)
        {
            break observation;
        }
        let asked = observation
            .budget
            .first_unpaid_read_storage()
            .expect("actual storage request before the target work charge");
        let next = before_pointer
            .checked_add(u64::try_from(asked).expect("bounded storage charge"))
            .expect("bounded storage prefix");
        assert!(
            next > before_pointer && next <= pointer_denial.budget.read_storage_consumed(),
            "actual request boundary advances towards the work charge"
        );
        before_pointer = next;
        drop_without_request::<C>(observation);
    };
    eprintln!("{label}: work probes={work_probes}; storage probes={storage_probes}");
    eprintln!("{label}/unaffordable diagnostic probe: requested={} charged={} pointer_bytes={} original_work={:?} locus={:?}",exhausted.requested,exhausted.budget.read_storage_consumed(),bytes_of_pointer,exhausted.budget.first_unpaid(),exhausted.budget.first_unpaid_locus());
    let CheckedPackageV2BudgetedReadResult::Exhausted(outcome) = &exhausted.result else {
        panic!("owned pointer cannot be afforded: {:?}", exhausted.result);
    };
    assert_eq!(
        outcome.denied, pointer.denied,
        "original denied resource/amount survive diagnostic denial"
    );
    assert_eq!(
        outcome.locus,
        pointer_denial
            .budget
            .first_unpaid_locus()
            .expect("original locus")
    );
    assert_eq!(outcome.configured_bound, prefix);
    assert_eq!(
        exhausted.budget.first_unpaid(),
        pointer_denial.budget.first_unpaid()
    );
    assert_eq!(exhausted.budget.read_storage_consumed(), before_pointer);
    for (name, observation) in [
        ("reference", &reference),
        ("exact work", &exact),
        ("work one below", &short),
        ("repeated work denial", &repeated),
        ("zero diagnostic storage", &zero_storage),
        ("owned-pointer work denial", &pointer_denial),
        ("unaffordable pointer", &exhausted),
    ] {
        eprintln!(
            "{label}/{name}: requested={} charged={} first-work={:?} first-storage={:?} locus={:?}",
            observation.requested,
            observation.budget.read_storage_consumed(),
            observation.budget.first_unpaid(),
            observation.budget.first_unpaid_read_storage(),
            observation.budget.first_unpaid_locus()
        );
    }
    // Every combination above has actually executed before the first red
    // allocation comparison; no tolerance can mask another axis' deficit.
    for (name, observation) in [
        ("exact work", exact),
        ("work one below", short),
        ("repeated work denial", repeated),
        ("zero diagnostic storage", zero_storage),
        ("owned-pointer work denial", pointer_denial),
        ("unaffordable pointer", exhausted),
    ] {
        accounted(name, &observation);
        drop_without_request::<C>(observation);
    }
    drop_without_request::<C>(reference);
    true
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
        assert_eq!(
            requested, delta,
            "per-call allocation equals caller-budget delta"
        );
        assert_eq!(C::measure(|| drop(result)), 0);
    }
}
