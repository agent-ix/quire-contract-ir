// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Counting-allocator checks of the budget-charged strict read (FR-038-AC-511,
//! TC-472): the heap bytes the read requests inside a stage equal the bytes
//! it charged to the budget's read storage.
//!
//! A stage is isolated by the difference of two reads that stop inside it,
//! each under its own read-storage limit: everything before the first stop
//! is identical in both, so the requests between the stops are the stage's.
//! The counter is per thread and armed only around the read.

#[path = "it/support/mod.rs"]
mod support;

use ix_trace_rs::trace;
use quire_contract_model::{
    initialize_operation_catalog, CheckedPackageEvidence, CheckedPackageReadLimits,
    CheckedPackageV2, CheckedPackageV2BudgetedReadResult, NativeJoinBudget, NativeJoinLocusSource,
    PackageReadStage,
};
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use support::checked_package::{canonical, evidence_for, v2_all_families};

thread_local! {
    /// Bytes requested on this thread while armed.
    static REQUESTED: Cell<Option<u64>> = const { Cell::new(None) };
}

fn note(bytes: usize) {
    let _ = REQUESTED.try_with(|requested| {
        if let Some(total) = requested.get() {
            requested.set(Some(total.saturating_add(bytes as u64)));
        }
    });
}

struct Counting;

// SAFETY: every method forwards to `System` with the layout and pointer it was
// given; the counter only reads sizes and touches no allocation.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        note(layout.size());
        // SAFETY: the caller's contract is `System`'s.
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        note(layout.size());
        // SAFETY: the caller's contract is `System`'s.
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        note(new_size);
        // SAFETY: the caller's contract is `System`'s.
        unsafe { System.realloc(pointer, layout, new_size) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: the caller's contract is `System`'s.
        unsafe { System.dealloc(pointer, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

/// Where a request falls in the read's runtime order, by the stage and form
/// of its locus. The stage enum is not that order: the interval-bound checks
/// of `WireChecks` run after the node-index stage.
fn phase(stage: PackageReadStage, form: &str) -> u8 {
    match (stage, form) {
        (PackageReadStage::Envelope, _) => 0,
        (PackageReadStage::Canonical, _) => 1,
        (PackageReadStage::WireDecode, _) => 2,
        (PackageReadStage::WireChecks, "owner_schema") => 3,
        (PackageReadStage::WireChecks, "interval_bounds") => 6,
        (PackageReadStage::WireChecks, _) => 4,
        (PackageReadStage::NodeIndex, _) => 5,
        _ => 7,
    }
}

const OWNER_SCHEMA: u8 = 3;
const FLAT_WIRE: u8 = 4;
const NODE_INDEX: u8 = 5;
const DECODE: u8 = 2;

/// The bytes one read requested and the bytes it charged, under `limit`, and
/// the stage of its first unpaid request.
fn read_under(limit: u64) -> (u64, u64, Option<(PackageReadStage, &'static str)>, String) {
    let value = v2_all_families();
    let bytes = canonical(&value);
    let evidence = evidence_for(&value);
    let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
    REQUESTED.with(|requested| requested.set(Some(0)));
    let result = CheckedPackageV2::read_with_budget(
        &bytes,
        CheckedPackageReadLimits::bounded(),
        &evidence,
        &mut budget,
    );
    let requested = REQUESTED
        .with(Cell::take)
        .expect("the counter was armed for the read");
    let outcome = format!("{result:?}");
    drop(result);
    let stage = budget
        .first_unpaid_locus()
        .and_then(|locus| match locus.source {
            NativeJoinLocusSource::PackageRead(stage, _) => Some((stage, locus.form)),
            _ => None,
        });
    (requested, budget.read_storage_consumed(), stage, outcome)
}

/// The least limit at which the read is admitted or its first unpaid request
/// falls in a runtime phase after `done`.
fn limit_past(done: u8) -> u64 {
    let past = |stage: PackageReadStage, form: &str| phase(stage, form) > done;
    let (_, whole, _, _) = read_under(u64::MAX);
    let (mut low, mut high) = (0, whole);
    while low < high {
        let middle = low + (high - low) / 2;
        let (_, _, denied, _) = read_under(middle);
        if denied.is_none_or(|(stage, form)| past(stage, form)) {
            high = middle;
        } else {
            low = middle + 1;
        }
    }
    low
}

/// Every heap request of the node-index stage is charged: between a read that
/// stops at the stage's first request and one that stops at its last, the
/// bytes requested equal the bytes charged.
#[trace("TC-472", "FR-038-AC-500", "FR-038-AC-511")]
#[test]
fn node_index_requests_exactly_what_it_charges() {
    let start = limit_past(FLAT_WIRE);
    let end = limit_past(NODE_INDEX) - 1;
    let (first_requested, first_charged, first_stage, first_outcome) = read_under(start);
    let (last_requested, last_charged, last_stage, last_outcome) = read_under(end);
    assert_eq!(
        first_stage.map(|(stage, _)| stage),
        Some(PackageReadStage::NodeIndex),
        "first outcome: {first_outcome}"
    );
    assert_eq!(
        last_stage.map(|(stage, _)| stage),
        Some(PackageReadStage::NodeIndex),
        "last outcome: {last_outcome}"
    );
    assert!(last_charged > first_charged, "the stage charges something");
    assert_eq!(
        last_requested - first_requested,
        last_charged - first_charged
    );
}

/// Every heap request of the slot resolution and the owner-schema check is
/// charged: between a read that stops at their first request and one that
/// stops at their last, the bytes requested equal the bytes charged.
#[trace("TC-472", "FR-038-AC-499", "FR-038-AC-511")]
#[test]
fn owner_schema_requests_exactly_what_it_charges() {
    let start = limit_past(DECODE);
    let end = limit_past(OWNER_SCHEMA) - 1;
    let (first_requested, first_charged, first_stage, first_outcome) = read_under(start);
    let (last_requested, last_charged, last_stage, last_outcome) = read_under(end);
    assert_eq!(
        first_stage,
        Some((PackageReadStage::WireChecks, "owner_schema")),
        "first outcome: {first_outcome}"
    );
    assert_eq!(
        last_stage,
        Some((PackageReadStage::WireChecks, "owner_schema")),
        "last outcome: {last_outcome}"
    );
    assert!(last_charged > first_charged, "the check charges something");
    assert_eq!(
        last_requested - first_requested,
        last_charged - first_charged
    );
}

/// The bytes one whole read requested, the read storage it charged, and
/// whether it admitted, under `limit`. The admitted package is held until the
/// counter is disarmed, so only the read's own requests are counted.
fn whole_read(
    bytes: &[u8],
    evidence: &CheckedPackageEvidence,
    limit: u64,
) -> (u64, u64, bool, String) {
    let limits = CheckedPackageReadLimits {
        bytes: 1 << 30,
        nodes: 1 << 24,
        edges: 1 << 24,
        occurrences: 1 << 24,
        diagnostics: 1 << 24,
        work: 1 << 40,
    };
    let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
    REQUESTED.with(|requested| requested.set(Some(0)));
    let result = CheckedPackageV2::read_with_budget(bytes, limits, evidence, &mut budget);
    let requested = REQUESTED
        .with(Cell::take)
        .expect("the counter was armed for the read");
    let outcome = format!("{result:?}");
    let admitted = matches!(result, CheckedPackageV2BudgetedReadResult::Admitted(_));
    drop(result);
    (requested, budget.read_storage_consumed(), admitted, outcome)
}

/// The whole-read accounting of one input: under an unlimited budget the
/// bytes requested equal the bytes charged; the exact limit admits; one byte
/// short refuses and requests no more than it charged.
fn assert_whole_read_accounted(label: &str, bytes: &[u8], evidence: &CheckedPackageEvidence) {
    assert!(
        initialize_operation_catalog().is_ok(),
        "the operation catalog reads"
    );
    let (requested, charged, admitted, outcome) = whole_read(bytes, evidence, u64::MAX);
    assert!(admitted, "{label}: the input admits; outcome: {outcome}");
    assert_eq!(
        requested, charged,
        "{label}: requested bytes == charged bytes"
    );
    let (_, _, exact, exact_outcome) = whole_read(bytes, evidence, charged);
    assert!(
        exact,
        "{label}: the exact limit admits; outcome: {exact_outcome}"
    );
    let (short_requested, short_charged, short, short_outcome) =
        whole_read(bytes, evidence, charged - 1);
    assert!(
        !short,
        "{label}: one byte short refuses; outcome: {short_outcome}"
    );
    assert!(short_charged < charged, "{label}: the refusal stops early");
    assert!(
        short_requested <= short_charged + charged / 16 + 4096,
        "{label}: a refusal requests only its charged bytes and its diagnostic"
    );
}

/// A genuine package: every request is charged.
#[trace("TC-472", "FR-038-AC-511")]
#[test]
fn a_genuine_package_requests_exactly_what_it_charges() {
    let value = v2_all_families();
    assert_whole_read_accounted("all families", &canonical(&value), &evidence_for(&value));
}

/// A package whose graph sequence exceeds 1 MiB.
#[trace("TC-472", "FR-038-AC-511")]
#[test]
fn a_package_with_a_sequence_over_one_mebibyte_requests_exactly_what_it_charges() {
    let value = v2_all_families();
    let (bytes, _) = support::checked_package::v2_reference_chain(4000);
    assert!(bytes.len() > 1 << 20, "the chain exceeds 1 MiB");
    assert_whole_read_accounted("reference chain", &bytes, &evidence_for(&value));
}
