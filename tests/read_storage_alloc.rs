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
    initialize_operation_catalog, CheckedPackageEvidence, CheckedPackageV2,
    CheckedPackageV2BudgetedReadResult, CheckedPackageV2ReadResult,
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

#[path = "it/support/read_allocation_proof.rs"]
mod read_allocation_proof;
use read_allocation_proof::{
    accounted, drop_without_request, every_request, generous_limits, measure_read, Counter,
    Expected,
};

struct ThreadCounter;
impl Counter for ThreadCounter {
    fn measure(read: impl FnOnce()) -> u64 {
        REQUESTED.with(|counter| {
            assert!(counter.get().is_none());
            counter.set(Some(0));
        });
        read();
        REQUESTED
            .with(Cell::take)
            .expect("counter armed around only the requested operation")
    }
}

fn whole_read(label: &str, bytes: &[u8], evidence: &CheckedPackageEvidence) {
    read_allocation_proof::assert_read::<ThreadCounter>(label, bytes, evidence, Expected::Admitted);
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
    every_request::<ThreadCounter>(&canonical(&value), &evidence_for(&value));
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
        let reference =
            measure_read::<ThreadCounter>(&bytes, &evidence, generous_limits(), u64::MAX);
        let plain = CheckedPackageV2::read(&bytes, generous_limits(), &evidence);
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
        let exact = measure_read::<ThreadCounter>(
            &bytes,
            &evidence,
            generous_limits(),
            reference.budget.read_storage_consumed(),
        );
        assert_eq!(exact.result, reference.result);
        accounted("exact malformed refusal", &exact);
        drop_without_request::<ThreadCounter>(exact);
        drop_without_request::<ThreadCounter>(reference);
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
        let stopped = measure_read::<ThreadCounter>(&bytes, &evidence, generous_limits(), 0);
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
        drop_without_request::<ThreadCounter>(stopped);
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
        let read = measure_read::<ThreadCounter>(bytes, &evidence, generous_limits(), u64::MAX);
        let CheckedPackageV2BudgetedReadResult::Refused(ref refusal) = read.result else {
            panic!("duplicate member must refuse: {:?}", read.result);
        };
        assert_eq!(
            refusal.code,
            quire_contract_model::CheckedPackageRefusalCode::DuplicateMember
        );
        read_allocation_proof::assert_read::<ThreadCounter>(
            "duplicate first fault",
            bytes,
            &evidence,
            Expected::Refused(quire_contract_model::CheckedPackageRefusalCode::DuplicateMember),
        );
        accounted("first byte-stream fault", &read);
        drop_without_request::<ThreadCounter>(read);
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

/// Destructor accounting is independent of the still-red during-call proof.
#[trace("TC-472", "FR-038-AC-511")]
#[test]
fn returned_admitted_refused_and_denied_outcomes_drop_without_allocating() {
    initialize_operation_catalog().expect("catalog gate");
    let value = v2_all_families();
    let bytes = canonical(&value);
    let evidence = evidence_for(&value);
    let admitted = measure_read::<ThreadCounter>(&bytes, &evidence, generous_limits(), u64::MAX);
    assert!(matches!(
        admitted.result,
        CheckedPackageV2BudgetedReadResult::Admitted(_)
    ));
    drop_without_request::<ThreadCounter>(admitted);
    let refused =
        measure_read::<ThreadCounter>(b"not JSON", &evidence, generous_limits(), u64::MAX);
    assert!(matches!(
        refused.result,
        CheckedPackageV2BudgetedReadResult::Refused(_)
    ));
    drop_without_request::<ThreadCounter>(refused);
    let denied = measure_read::<ThreadCounter>(&bytes, &evidence, generous_limits(), 0);
    assert!(denied.budget.first_unpaid_read_storage().is_some());
    drop_without_request::<ThreadCounter>(denied);
}

#[trace("TC-472", "FR-038-AC-511")]
#[test]
fn the_one_short_outcome_itself_has_no_unpaid_allocation() {
    initialize_operation_catalog().expect("catalog gate");
    let value = v2_all_families();
    let bytes = canonical(&value);
    let evidence = evidence_for(&value);
    let reference = measure_read::<ThreadCounter>(&bytes, &evidence, generous_limits(), u64::MAX);
    assert!(matches!(
        reference.result,
        CheckedPackageV2BudgetedReadResult::Admitted(_)
    ));
    let required = reference.budget.read_storage_consumed();
    assert!(required > 0);
    let short = measure_read::<ThreadCounter>(&bytes, &evidence, generous_limits(), required - 1);
    assert!(matches!(
        short.result,
        CheckedPackageV2BudgetedReadResult::BudgetIncomplete(_)
            | CheckedPackageV2BudgetedReadResult::Exhausted(_)
    ));
    assert!(short.budget.first_unpaid_read_storage().is_some());
    accounted("one short whole-read outcome", &short);
    drop_without_request::<ThreadCounter>(short);
    drop_without_request::<ThreadCounter>(reference);
}

#[trace("TC-472", "FR-038-AC-511")]
#[test]
fn counted_exact_and_denied_work_budgets_cover_a_genuine_package() {
    let value = v2_all_families();
    assert!(
        read_allocation_proof::work_denials::<ThreadCounter>(
            "genuine",
            &canonical(&value),
            &evidence_for(&value)
        ),
        "the genuine input reaches work charges"
    );
}

#[trace("TC-472", "FR-038-AC-511")]
#[test]
fn counted_exact_and_denied_work_budgets_cover_a_malformed_body() {
    let mut value = v2_all_families();
    let node = value["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .iter_mut()
        .find(|node| node["node_tag"] == "expression" && node["semantic_form"] == "reference")
        .expect("reference fixture");
    node["body"] = serde_json::json!({"term":"reference"});
    let bytes = canonical(&value);
    let evidence = evidence_for(&value);
    let result = CheckedPackageV2::read(&bytes, generous_limits(), &evidence);
    assert!(
        matches!(result,CheckedPackageV2ReadResult::Refused(ref refusal) if refusal.code==quire_contract_model::CheckedPackageRefusalCode::InvalidSemanticGraph),
        "genuine malformed-body first fault: {result:?}"
    );
    assert!(
        read_allocation_proof::work_denials::<ThreadCounter>("malformed body", &bytes, &evidence),
        "malformed body reaches work charges before refusal"
    );
}

#[trace("TC-472", "FR-038-AC-511")]
#[test]
fn consecutive_reads_charge_each_delta_to_the_same_caller_budget() {
    let value = v2_all_families();
    read_allocation_proof::shared_budget_reads::<ThreadCounter>(
        &canonical(&value),
        &evidence_for(&value),
    );
}
