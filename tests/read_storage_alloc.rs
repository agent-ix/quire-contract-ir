// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Counting-allocator checks of the budget-charged strict read (FR-038-AC-511,
//! TC-472): each complete public read requests exactly the heap bytes it
//! charged to its caller's read storage.
//!
//! Real request boundaries come from the public first-unpaid amount and locus;
//! stage enumeration order is never treated as runtime order.
//! The counter is per thread and armed only around the read.

#[path = "it/support/checked_package.rs"]
mod checked_package;

use checked_package::{canonical, evidence_for, v2_all_families};
use ix_trace_rs::trace;
use quire_contract_model::{
    initialize_operation_catalog, CheckedPackageEvidence, CheckedPackageReadLimits,
    CheckedPackageV2, CheckedPackageV2BudgetedReadResult, CheckedPackageV2ReadResult,
    NativeJoinBudget, NativeJoinLocusSource,
};
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

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

/// One complete public read. Fixtures, budget construction and comparison
/// live outside the counter; the returned outcome remains owned by the caller.
struct Observation {
    result: CheckedPackageV2BudgetedReadResult,
    budget: NativeJoinBudget,
    requested: u64,
}

fn measure_read(
    bytes: &[u8],
    evidence: &CheckedPackageEvidence,
    limits: CheckedPackageReadLimits,
    limit: u64,
) -> Observation {
    let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
    REQUESTED.with(|counter| {
        assert!(counter.get().is_none());
        counter.set(Some(0));
    });
    let result = CheckedPackageV2::read_with_budget(bytes, limits, evidence, &mut budget);
    let requested = REQUESTED
        .with(Cell::take)
        .expect("armed around only the public read");
    Observation {
        result,
        budget,
        requested,
    }
}

fn generous_limits() -> CheckedPackageReadLimits {
    CheckedPackageReadLimits {
        bytes: 1 << 30,
        nodes: 1 << 24,
        edges: 1 << 26,
        occurrences: 1 << 26,
        diagnostics: 1 << 24,
        work: 1 << 50,
    }
}

fn accounted(label: &str, observation: &Observation) {
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
fn drop_without_request(observation: Observation) {
    REQUESTED.with(|counter| counter.set(Some(0)));
    drop(observation.result);
    let requested = REQUESTED.with(Cell::take).expect("drop counter armed");
    assert_eq!(
        requested, 0,
        "dropping the returned outcome allocates nothing"
    );
}

fn whole_read(label: &str, bytes: &[u8], evidence: &CheckedPackageEvidence) {
    initialize_operation_catalog().expect("the specified catalogue gate succeeds");
    let limits = generous_limits();
    // Initialize only the specified catalog gate; do not use an unmeasured
    // public package read to warm per-read owned storage.
    let reference = measure_read(bytes, evidence, limits, u64::MAX);
    let plain = CheckedPackageV2::read(bytes, limits, evidence);
    let CheckedPackageV2ReadResult::Admitted(ref expected) = plain else {
        panic!("{label}: real fixture must admit: {plain:?}");
    };
    let CheckedPackageV2BudgetedReadResult::Admitted(ref package) = reference.result else {
        panic!("{label}: budgeted admission {:?}", reference.result);
    };
    assert_eq!(
        package, expected,
        "{label}: full admitted value equals the unbudgeted reader"
    );
    accounted(label, &reference);
    let total = reference.budget.read_storage_consumed();
    assert!(total > 0);
    for limit in [total, u64::MAX] {
        let exact = measure_read(bytes, evidence, limits, limit);
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
        drop_without_request(exact);
    }
    let short = measure_read(bytes, evidence, limits, total - 1);
    assert!(
        !matches!(
            short.result,
            CheckedPackageV2BudgetedReadResult::Admitted(_)
        ),
        "one short refuses"
    );
    assert!(short.budget.first_unpaid_read_storage().is_some());
    accounted(label, &short);
    drop_without_request(short);
    drop_without_request(reference);
}

/// Probe actual requests in runtime order. Stage labels may decrease; no
/// numeric stage-order bisection or phase assumption is used.
fn every_request(bytes: &[u8], evidence: &CheckedPackageEvidence) {
    initialize_operation_catalog().expect("catalogue gate");
    let limits = generous_limits();
    let reference = measure_read(bytes, evidence, limits, u64::MAX);
    assert!(matches!(
        reference.result,
        CheckedPackageV2BudgetedReadResult::Admitted(_)
    ));
    let total = reference.budget.read_storage_consumed();
    let mut before = 0;
    loop {
        let stopped = measure_read(bytes, evidence, limits, before);
        accounted("exact preceding request boundary", &stopped);
        if before == total {
            assert_eq!(stopped.result, reference.result);
            drop_without_request(stopped);
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
        let short = measure_read(bytes, evidence, limits, next - 1);
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
        let again = measure_read(bytes, evidence, limits, next - 1);
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
        drop_without_request(stopped);
        drop_without_request(short);
        drop_without_request(again);
        before = next;
    }
    drop_without_request(reference);
}

#[trace("TC-472", "FR-038-AC-511")]
#[test]
fn whole_read_small_package_has_no_unpaid_allocation() {
    let value = v2_all_families();
    whole_read("all families", &canonical(&value), &evidence_for(&value));
}

#[trace("TC-472", "FR-038-AC-511")]
#[test]
fn whole_read_over_one_mebibyte_has_no_unpaid_allocation() {
    let value = v2_all_families();
    let (bytes, _) = checked_package::v2_reference_chain(4000);
    assert!(bytes.len() > 1 << 20);
    whole_read("over1MiB sequence", &bytes, &evidence_for(&value));
}

#[trace("TC-472", "FR-038-AC-511")]
#[test]
fn whole_read_each_actual_request_is_prepaid() {
    let value = v2_all_families();
    every_request(&canonical(&value), &evidence_for(&value));
}

#[trace("TC-472", "FR-038-AC-512")]
#[test]
fn malformed_first_fault_and_exact_budget_match_the_public_reader() {
    initialize_operation_catalog().expect("catalogue gate");
    let base = v2_all_families();
    let evidence = evidence_for(&base);
    let mut unknown = base.clone();
    unknown["unexpected"] = serde_json::json!(true);
    unknown["diagnostics"] = serde_json::json!(null);
    let mut missing = base.clone();
    missing.as_object_mut().expect("object").remove("name");
    missing["diagnostics"] = serde_json::json!(null);
    for bytes in [
        b"{\"name\":\"first\",\"name\":\"duplicate\"}".to_vec(),
        b"not json".to_vec(),
        canonical(&unknown),
        canonical(&missing),
    ] {
        let plain = CheckedPackageV2::read(&bytes, generous_limits(), &evidence);
        let reference = measure_read(&bytes, &evidence, generous_limits(), u64::MAX);
        let (
            CheckedPackageV2ReadResult::Refused(expected),
            CheckedPackageV2BudgetedReadResult::Refused(actual),
        ) = (&plain, &reference.result)
        else {
            panic!(
                "malformed input must refuse: {plain:?}/{:?}",
                reference.result
            );
        };
        assert_eq!(
            actual, expected,
            "code/cause/ordered loci/node/document pointer preserve first fault"
        );
        assert!(
            !actual.loci.is_empty(),
            "refusal retains ordered primary locations"
        );
        accounted("malformed/refused", &reference);
        let exact = measure_read(
            &bytes,
            &evidence,
            generous_limits(),
            reference.budget.read_storage_consumed(),
        );
        assert_eq!(exact.result, reference.result);
        accounted("exact malformed refusal", &exact);
        drop_without_request(exact);
        drop_without_request(reference);
    }
}

#[trace("TC-472", "FR-038-AC-511")]
#[test]
fn even_a_warm_catalogs_first_owned_request_is_denied_before_allocation() {
    initialize_operation_catalog().expect("catalogue gate");
    let value = v2_all_families();
    let bytes = canonical(&value);
    let evidence = evidence_for(&value);
    for _ in 0..2 {
        let stopped = measure_read(&bytes, &evidence, generous_limits(), 0);
        assert_eq!(stopped.requested, 0);
        assert_eq!(stopped.budget.read_storage_consumed(), 0);
        let CheckedPackageV2BudgetedReadResult::BudgetIncomplete(outcome) = &stopped.result else {
            panic!(
                "zero-cost pointer-free denial is affordable: {:?}",
                stopped.result
            );
        };
        assert_eq!(
            outcome.denied.resource,
            quire_contract_model::PackageReadResource::ReadStorage
        );
        assert!(outcome.denied.amount > 0);
        assert_eq!(outcome.limit, 0);
        assert_eq!(
            outcome.consumed,
            u64::try_from(outcome.denied.amount).expect("catalog request")
        );
        assert!(outcome.path.is_none());
        assert_eq!(
            stopped
                .budget
                .first_unpaid_locus()
                .expect("denial locus")
                .form,
            "operation_catalog"
        );
        drop_without_request(stopped);
    }
}

#[trace("TC-472", "FR-038-AC-512")]
#[test]
fn duplicate_member_precedes_a_later_out_of_range_number() {
    initialize_operation_catalog().expect("catalogue gate");
    let evidence = CheckedPackageEvidence::new();
    for bytes in [
        br#"{"name":"first","name":"duplicate"}"#.as_slice(),
        br#"{"name":"first","name":"duplicate","later":1e999}"#.as_slice(),
    ] {
        let read = measure_read(bytes, &evidence, generous_limits(), u64::MAX);
        let CheckedPackageV2BudgetedReadResult::Refused(ref refusal) = read.result else {
            panic!("duplicate member must refuse: {:?}", read.result);
        };
        assert_eq!(
            refusal.code,
            quire_contract_model::CheckedPackageRefusalCode::DuplicateMember
        );
        accounted("first byte-stream fault", &read);
        drop_without_request(read);
    }
}

#[path = "it/support/read_storage_inputs.rs"]
mod read_storage_inputs;

#[trace("TC-472", "FR-038-AC-511")]
#[test]
fn whole_read_chain_100000() {
    let input = read_storage_inputs::chain_100000();
    whole_read("chain_100000", &input.bytes, &input.evidence);
}

#[trace("TC-472", "FR-038-AC-511")]
#[test]
fn whole_read_flat_graph_and_source_map_10000() {
    let input = read_storage_inputs::flat_graph_and_source_map_10000();
    whole_read(
        "flat_graph_and_source_map_10000",
        &input.bytes,
        &input.evidence,
    );
}

#[trace("TC-472", "FR-038-AC-511")]
#[test]
fn whole_read_node_with_10000_dependencies() {
    let input = read_storage_inputs::dependencies_10000();
    whole_read("dependencies_10000", &input.bytes, &input.evidence);
}

#[trace("TC-472", "FR-038-AC-511")]
#[test]
fn whole_read_single_string_one_mebibyte() {
    let input = read_storage_inputs::string_one_mebibyte();
    whole_read("string_one_mebibyte", &input.bytes, &input.evidence);
}

#[trace("TC-472", "FR-038-AC-511")]
#[test]
fn whole_read_diagnostics_10000_with_details() {
    let input = read_storage_inputs::diagnostics_10000();
    whole_read("diagnostics_10000", &input.bytes, &input.evidence);
}

#[trace("TC-472", "FR-038-AC-511")]
#[test]
fn whole_read_direct_dependencies_64() {
    let input = read_storage_inputs::direct_dependencies_64();
    whole_read("direct_dependencies_64", &input.bytes, &input.evidence);
}

#[trace("TC-472", "FR-038-AC-511")]
#[test]
fn whole_read_two_selected_documents_512k() {
    let input = read_storage_inputs::selected_models_two_512k();
    whole_read("selected_models_two_512k", &input.bytes, &input.evidence);
}
