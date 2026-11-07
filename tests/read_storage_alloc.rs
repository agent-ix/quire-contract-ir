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
    CheckedPackageReadLimits, CheckedPackageV2, NativeJoinBudget, NativeJoinLocusSource,
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

/// The bytes one read requested and the bytes it charged, under `limit`, and
/// the stage of its first unpaid request.
fn read_under(limit: u64) -> (u64, u64, Option<PackageReadStage>) {
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
    drop(result);
    let stage = budget
        .first_unpaid_locus()
        .and_then(|locus| match locus.source {
            NativeJoinLocusSource::PackageRead(stage, _) => Some(stage),
            _ => None,
        });
    (requested, budget.read_storage_consumed(), stage)
}

/// The least limit at which the read's first unpaid request is of a stage
/// after `stage`, or it is admitted.
fn limit_past(stage: PackageReadStage) -> u64 {
    let (_, whole, _) = read_under(u64::MAX);
    let (mut low, mut high) = (0, whole);
    while low < high {
        let middle = low + (high - low) / 2;
        let (_, _, denied) = read_under(middle);
        if denied.is_none_or(|denied| denied > stage) {
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
    let start = limit_past(PackageReadStage::WireChecks);
    let end = limit_past(PackageReadStage::NodeIndex) - 1;
    let (first_requested, first_charged, first_stage) = read_under(start);
    let (last_requested, last_charged, last_stage) = read_under(end);
    assert_eq!(first_stage, Some(PackageReadStage::NodeIndex));
    assert_eq!(last_stage, Some(PackageReadStage::NodeIndex));
    assert!(last_charged > first_charged, "the stage charges something");
    assert_eq!(
        last_requested - first_requested,
        last_charged - first_charged
    );
}
