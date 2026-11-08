// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The budget-charged strict V2 read: entries, deltas, the read-storage
//! counter's API and the envelope stages' exact charging (TC-472).

use crate::support::checked_package::{
    all_families_read_work, canonical, evidence_for, v2_all_families,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    initialize_operation_catalog, read_checked_package_with_budget,
    CheckedPackageBudgetedDispatchResult, CheckedPackageLimit, CheckedPackageReadLimits,
    CheckedPackageV2, CheckedPackageV2BudgetedReadResult, CheckedPackageV2ReadResult,
    NativeJoinBudget, NativeJoinChargeKind, NativeJoinLocus, NativeJoinLocusSource,
    PackageReadResource, PackageReadStage,
};

fn bytes() -> Vec<u8> {
    canonical(&v2_all_families())
}

fn read_with(budget: &mut NativeJoinBudget) -> CheckedPackageV2BudgetedReadResult {
    let value = v2_all_families();
    CheckedPackageV2::read_with_budget(
        &canonical(&value),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&value),
        budget,
    )
}

fn envelope_locus(stage: PackageReadStage, form: &'static str) -> NativeJoinLocus {
    NativeJoinLocus {
        source: NativeJoinLocusSource::PackageRead(stage, 0),
        form,
    }
}

/// The budgeted read admits what the unbudgeted read admits, with the work
/// delta the unbudgeted work boundary names and no logical storage.
#[trace("TC-472", "FR-038-AC-495", "FR-038-AC-496", "FR-038-AC-508")]
#[test]
fn budgeted_read_matches_the_unbudgeted_read_and_records_its_deltas() {
    let value = v2_all_families();
    let evidence = evidence_for(&value);
    let CheckedPackageV2ReadResult::Admitted(plain) =
        CheckedPackageV2::read(&bytes(), CheckedPackageReadLimits::bounded(), &evidence)
    else {
        panic!("the fixture admits");
    };
    let mut budget = NativeJoinBudget::unlimited();
    let CheckedPackageV2BudgetedReadResult::Admitted(budgeted) = read_with(&mut budget) else {
        panic!("the fixture admits under an unlimited budget");
    };
    assert_eq!(*plain, *budgeted);
    let delta = budget
        .last_success_delta()
        .expect("admission records a delta");
    assert_eq!(delta.work, all_families_read_work());
    assert_eq!(delta.storage, 0);
    assert!(budget.last_success_read_storage_delta().expect("delta") > 0);
    assert_eq!(budget.first_unpaid_read_storage(), None);
}

/// One work unit below the requirement stops at the same value the unbudgeted
/// read reports, as a budget denial.
#[trace("TC-472", "FR-038-AC-495", "FR-038-AC-510")]
#[test]
fn one_below_the_work_requirement_is_a_work_budget_incomplete_matching_the_plain_read() {
    let value = v2_all_families();
    let evidence = evidence_for(&value);
    let work = all_families_read_work();
    let mut limits = CheckedPackageReadLimits::bounded();
    limits.work = work - 1;
    let CheckedPackageV2ReadResult::Incomplete(plain) =
        CheckedPackageV2::read(&bytes(), limits, &evidence)
    else {
        panic!("one below the requirement is incomplete");
    };
    assert_eq!(plain.limit_kind, CheckedPackageLimit::Work);
    let mut budget = NativeJoinBudget::new(work - 1, 0).with_read_storage_limit(u64::MAX);
    let CheckedPackageV2BudgetedReadResult::BudgetIncomplete(denied) = read_with(&mut budget)
    else {
        panic!("one below the requirement is a budget denial");
    };
    assert_eq!(denied.denied.resource, PackageReadResource::Work);
    assert_eq!(denied.limit, work - 1);
    assert_eq!(denied.consumed, plain.consumed);
    assert_eq!(denied.path, plain.path);
    assert_eq!(budget.last_success_delta(), None);
    assert_eq!(budget.last_success_read_storage_delta(), None);
}

/// The charged stages' reservations are exact: the measured requirement
/// admits, one byte below is denied at the last request (today the node-index
/// key table), at its locus, with the bytes asked.
#[trace("TC-472", "FR-038-AC-497", "FR-038-AC-498", "FR-358-AC-13")]
#[test]
fn read_storage_one_below_the_requirement_is_denied_at_the_last_request() {
    let mut measure = NativeJoinBudget::unlimited();
    assert!(matches!(
        read_with(&mut measure),
        CheckedPackageV2BudgetedReadResult::Admitted(_)
    ));
    let required = measure.read_storage_consumed();
    let mut exact = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(required);
    assert!(matches!(
        read_with(&mut exact),
        CheckedPackageV2BudgetedReadResult::Admitted(_)
    ));
    assert_eq!(exact.read_storage_consumed(), required);

    let mut short = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(required - 1);
    let result = read_with(&mut short);
    let asked = short
        .first_unpaid_read_storage()
        .expect("a denial is recorded");
    assert!(asked > 0);
    assert_eq!(short.first_unpaid(), None);
    assert_eq!(
        short.first_unpaid_locus(),
        Some(envelope_locus(PackageReadStage::NodeIndex, "node_index"))
    );
    assert_eq!(short.read_storage_limit(), required - 1);
    assert_eq!(short.consumed().storage, 0);
    let CheckedPackageV2BudgetedReadResult::BudgetIncomplete(incomplete) = result else {
        panic!("a denial with no pointer to own is affordable: {result:?}");
    };
    assert_eq!(incomplete.denied.resource, PackageReadResource::ReadStorage);
    assert_eq!(incomplete.denied.amount, asked);
    assert_eq!(incomplete.limit, required - 1);
    assert!(short.read_storage_consumed() < required);
}

/// A zero read-storage limit refuses at the first allocation of the shared
/// reader with the fixed-size outcome; an input over the byte limit is still
/// `incomplete` for `bytes`.
#[trace("TC-472", "FR-038-AC-510", "FR-358-AC-14")]
#[test]
fn zero_read_storage_is_denied_at_the_first_request_and_bytes_precede_it() {
    let mut zero = NativeJoinBudget::new(u64::MAX, u64::MAX);
    assert_eq!(zero.read_storage_limit(), 0);
    // No allocation is owned by the outcome, so its diagnostic costs nothing.
    let CheckedPackageV2BudgetedReadResult::BudgetIncomplete(denied) = read_with(&mut zero) else {
        panic!("an envelope denial with no pointer owns no diagnostic");
    };
    assert_eq!(denied.limit, 0);
    assert_eq!(denied.path, None);
    assert_eq!(
        zero.first_unpaid_locus(),
        Some(envelope_locus(PackageReadStage::Envelope, "package_bytes"))
    );
    assert_eq!(zero.read_storage_consumed(), 0);

    let value = v2_all_families();
    let mut limits = CheckedPackageReadLimits::bounded();
    limits.bytes = 8;
    let mut budget = NativeJoinBudget::new(u64::MAX, 0);
    let CheckedPackageV2BudgetedReadResult::Incomplete(bytes) = CheckedPackageV2::read_with_budget(
        &canonical(&value),
        limits,
        &evidence_for(&value),
        &mut budget,
    ) else {
        panic!("over the byte limit is incomplete");
    };
    assert_eq!(bytes.limit_kind, CheckedPackageLimit::Bytes);
    assert_eq!(bytes.path, None);
    assert_eq!(budget.first_unpaid_read_storage(), None);
}

/// The read-storage counter's API: default zero limit, builder, unlimited
/// saturation, and the logical counter untouched by a read.
#[trace("TC-472", "FR-358-AC-14", "FR-038-AC-515")]
#[test]
fn read_storage_counter_api_and_catalog_gate() {
    assert!(initialize_operation_catalog().is_ok());
    let fresh = NativeJoinBudget::new(5, 6);
    assert_eq!(fresh.read_storage_limit(), 0);
    assert_eq!(fresh.read_storage_consumed(), 0);
    assert_eq!(fresh.last_success_read_storage_delta(), None);
    assert_eq!(fresh.with_read_storage_limit(9).read_storage_limit(), 9);

    let mut unlimited = NativeJoinBudget::unlimited();
    let locus = envelope_locus(PackageReadStage::Graph, "x");
    for _ in 0..2 {
        assert!(unlimited
            .accounted_charge(NativeJoinChargeKind::Work, u128::from(u64::MAX), locus)
            .is_ok());
    }
    assert_eq!(unlimited.consumed().work, u64::MAX);
    assert_eq!(unlimited.first_unpaid(), None);

    let value = v2_all_families();
    let mut budget = NativeJoinBudget::new(u64::MAX, 7).with_read_storage_limit(u64::MAX);
    let CheckedPackageBudgetedDispatchResult::AdmittedV2(_) = read_checked_package_with_budget(
        &canonical(&value),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&value),
        &mut budget,
    ) else {
        panic!("the dispatch admits");
    };
    assert_eq!(budget.consumed().storage, 0);
    assert_eq!(budget.storage_limit(), 7);
}

/// Two reads, each affordable alone, composed on one budget below their sum:
/// the first admits and the second stops at its first unpaid charge.
#[trace("TC-472", "FR-038-AC-496")]
#[test]
fn composed_reads_stop_the_second_at_its_first_unpaid_charge() {
    let mut measure = NativeJoinBudget::unlimited();
    assert!(matches!(
        read_with(&mut measure),
        CheckedPackageV2BudgetedReadResult::Admitted(_)
    ));
    let one = measure.read_storage_consumed();
    let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(one + one / 2);
    assert!(matches!(
        read_with(&mut budget),
        CheckedPackageV2BudgetedReadResult::Admitted(_)
    ));
    assert_eq!(budget.read_storage_consumed(), one);
    let second = read_with(&mut budget);
    assert!(!matches!(
        second,
        CheckedPackageV2BudgetedReadResult::Admitted(_)
    ));
    assert_eq!(budget.last_success_read_storage_delta(), None);
    assert!(budget.first_unpaid_read_storage().is_some());
}
