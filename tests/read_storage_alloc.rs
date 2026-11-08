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

use checked_package::{
    canonical, evidence_for, family_key, node_id, settle, v2_all_families, v2_nominal,
};
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
    for (index, bytes) in [
        br#"{"name":"first","name":"duplicate"}"#.as_slice(),
        br#"{"name":"first","name":"duplicate","later":1e999}"#.as_slice(),
    ]
    .into_iter()
    .enumerate()
    {
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
            if index == 0 {
                Expected::Refused(quire_contract_model::CheckedPackageRefusalCode::DuplicateMember)
            } else {
                Expected::OriginalOracle(
                    |outcome| matches!(outcome, CheckedPackageV2ReadResult::Refused(refusal) if refusal.code == quire_contract_model::CheckedPackageRefusalCode::DuplicateMember),
                )
            },
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
    // A large graph of shallow reference terms is not a single deep term
    // walk. Keep its normal stage/class endpoint proof; it does not designate
    // every per-node initial walker reservation as repeated stack growth.
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

#[trace("TC-472", "FR-038-AC-511", "FR-038-AC-512")]
#[test]
fn exact_and_above_native_budgets_preserve_the_original_model_views_and_lowering() {
    let wire = v2_all_families();
    let bytes = canonical(&wire);
    let evidence = evidence_for(&wire);
    let limits = generous_limits();
    let reference = read_allocation_proof::measure_at::<ThreadCounter>(
        &bytes,
        &evidence,
        limits,
        u64::MAX,
        u64::MAX,
    );
    let CheckedPackageV2BudgetedReadResult::Admitted(package) = &reference.result else {
        panic!("the original all-families fixture must admit");
    };
    assert!(
        package.selected_model_packages().next().is_some(),
        "genuine selected model views execute"
    );
    let work = reference.budget.consumed().work;
    let storage = reference.budget.read_storage_consumed();
    for (work, storage) in [
        (work, storage),
        (
            work.checked_add(1).expect("above work"),
            storage.checked_add(1).expect("above read storage"),
        ),
    ] {
        let sufficient = read_allocation_proof::measure_at::<ThreadCounter>(
            &bytes, &evidence, limits, work, storage,
        );
        // Shared oracle executes every node's kind/term/model/operation views,
        // population projections, lowering result and canonical output bytes.
        qspec_read_proof::assert_budgeted_results_equal(
            &reference.result,
            &sufficient.result,
            "original all-families exact/above views",
        );
        assert_eq!(reference.requested, sufficient.requested);
        drop_without_request::<ThreadCounter>(sufficient);
    }
    drop_without_request::<ThreadCounter>(reference);
}

#[cfg(feature = "test-support")]
#[trace("TC-472", "FR-038-AC-511")]
#[test]
fn full_native_boundaries_cover_seeded_work_on_the_original_genuine_package() {
    use quire_contract_model::{
        NativeJoinBudget, NativeJoinChargeKind, NativeJoinLocus, NativeJoinLocusSource,
    };
    let value = v2_all_families();
    let bytes = canonical(&value);
    let evidence = evidence_for(&value);
    let limits = generous_limits();
    let mut seed = NativeJoinBudget::new(u64::MAX, 2).with_read_storage_limit(u64::MAX);
    let locus = NativeJoinLocus {
        source: NativeJoinLocusSource::Package,
        form: "prior_caller_charge",
    };
    seed.accounted_charge(NativeJoinChargeKind::Work, 3, locus)
        .expect("prior work");
    seed.accounted_charge(NativeJoinChargeKind::Storage, 2, locus)
        .expect("prior logical storage");
    let reference =
        qspec_read_proof::measure_seeded(&bytes, &evidence, limits, seed.read_proof_snapshot());
    assert!(
        matches!(
            reference.result,
            CheckedPackageV2BudgetedReadResult::Admitted(_)
        ),
        "the original genuine package must admit"
    );
    let (_, results, complete) = qspec_read_proof::full_native_boundaries(
        &bytes,
        limits,
        &evidence,
        &reference,
        &[],
        Some(&seed),
    );
    assert_eq!(complete["prior_work"], 3);
    assert!(complete["work"].as_u64().expect("actual work") > 3);
    assert!(
        results.iter().any(|row| row["resource"] == "Work"),
        "real selected Work denials executed"
    );
    assert!(results
        .iter()
        .filter(|row| row["resource"] == "Work")
        .all(|row| row["denial"]["resource"] == "Work"
            && row["denial"]["entire_repeat_budget_equal"] == true));
    assert_eq!(seed.consumed().work, 3);
    assert_eq!(seed.consumed().storage, 2);
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

#[path = "it/support/qspec_read_proof.rs"]
mod qspec_read_proof;
use qspec_read_proof::read_and_account;
use qspec_read_proof::read_and_account as fixture_read;

fn measure_qspec_read_requests(read: impl FnOnce()) -> u64 {
    if let Some(before) = REQUESTED.with(Cell::get) {
        read();
        REQUESTED
            .with(Cell::get)
            .expect("outer counter remains armed")
            .checked_sub(before)
            .expect("requested-byte counter is monotone")
    } else {
        <ThreadCounter as Counter>::measure(read)
    }
}

fn without_qspec_instrumentation<T>(operation: impl FnOnce() -> T) -> T {
    let previous = REQUESTED.with(|counter| counter.replace(None));
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(operation));
    REQUESTED.with(|counter| counter.set(previous));
    match result {
        Ok(value) => value,
        Err(panic) => std::panic::resume_unwind(panic),
    }
}

#[trace("TC-472", "FR-038-AC-511")]
#[test]
fn original_dedicated_counter_observes_only_its_original_read() {
    let wire = v2_all_families();
    let bytes = canonical(&wire);
    let evidence = evidence_for(&wire);
    let limits = quire_contract_model::CheckedPackageReadLimits::bounded();
    let mut wrapped = None;
    let measured = <ThreadCounter as Counter>::measure(|| {
        wrapped = Some(read_and_account(&bytes, limits, &evidence));
    });
    let mut original = None;
    let expected = <ThreadCounter as Counter>::measure(|| {
        original = Some(CheckedPackageV2::read(&bytes, limits, &evidence));
    });
    assert_eq!(wrapped, original);
    assert_eq!(
        measured, expected,
        "proof instrumentation does not alter the original allocator oracle"
    );
    let with_drop = <ThreadCounter as Counter>::measure(|| {
        drop(read_and_account(&bytes, limits, &evidence));
    });
    let original_drop = <ThreadCounter as Counter>::measure(|| {
        drop(CheckedPackageV2::read(&bytes, limits, &evidence));
    });
    assert_eq!(
        with_drop, original_drop,
        "original requested bytes preserved including result drop"
    );
}

#[cfg(feature = "test-support")]
#[path = "it/support/canonical_class_denial.rs"]
mod canonical_class_denial;

#[trace("TC-472", "FR-038-AC-511")]
#[test]
fn nominal_validation_work_denial_prepays_its_original_body_pointer() {
    use quire_contract_model::{
        NativeJoinChargeKind, NativeJoinLocus, NativeJoinLocusSource, PackageReadResource,
        PackageReadStage,
    };
    let wire = v2_nominal();
    let bytes = canonical(&wire);
    let evidence = evidence_for(&wire);
    let limits = generous_limits();
    let ample =
        read_allocation_proof::measure_at::<ThreadCounter>(&bytes, &evidence, limits, 13, u64::MAX);
    let CheckedPackageV2BudgetedReadResult::BudgetIncomplete(outcome) = &ample.result else {
        panic!(
            "the original Work13 control must return its affordable diagnostic: {:?}",
            ample.result
        );
    };
    assert_eq!(outcome.denied.resource, PackageReadResource::Work);
    assert_eq!(outcome.denied.amount, 1);
    assert_eq!(outcome.limit, 13);
    assert_eq!(outcome.consumed, 14);
    let path = outcome.path.as_ref().expect("original body pointer");
    assert_eq!(path.as_str(), "/semantic_graph/nodes/2/body");
    let locus = NativeJoinLocus {
        source: NativeJoinLocusSource::PackageRead(PackageReadStage::Graph, 0),
        form: "dependency_order",
    };
    assert_eq!(
        ample.budget.first_unpaid_locus(),
        Some(locus),
        "preserve the caller's inherited first-denial locus"
    );
    let denied = ample.budget.first_unpaid().expect("first unpaid Work");
    assert_eq!(denied.kind, NativeJoinChargeKind::Work);
    assert_eq!(denied.amount, 1);
    assert!(ample.budget.first_unpaid_read_storage().is_none());
    let paid = ample.budget.read_storage_consumed();
    let pointer_bytes = u64::try_from(path.as_str().len()).expect("bounded pointer");
    let prefix = paid
        .checked_sub(pointer_bytes)
        .expect("actual prefix before the only outcome pointer");
    let exact =
        read_allocation_proof::measure_at::<ThreadCounter>(&bytes, &evidence, limits, 13, paid);
    assert_eq!(exact.result, ample.result);
    assert_eq!(exact.requested, ample.requested);
    let short =
        read_allocation_proof::measure_at::<ThreadCounter>(&bytes, &evidence, limits, 13, paid - 1);
    let CheckedPackageV2BudgetedReadResult::Exhausted(exhausted) = &short.result else {
        panic!(
            "one below the actual diagnostic request must exhaust: {:?}",
            short.result
        );
    };
    assert_eq!(exhausted.denied, outcome.denied);
    assert_eq!(exhausted.configured_bound, 13);
    assert_eq!(exhausted.locus, locus);
    assert!(exhausted.pending.is_none());
    assert_eq!(short.budget.first_unpaid(), Some(denied));
    assert!(
        short.budget.first_unpaid_read_storage().is_none(),
        "the original Work denial remains first when its diagnostic storage is denied"
    );
    assert_eq!(
        short.requested, prefix,
        "the denied pointer makes no allocation request"
    );
    assert_eq!(short.budget.read_storage_consumed(), prefix);
    let repeated =
        read_allocation_proof::measure_at::<ThreadCounter>(&bytes, &evidence, limits, 13, paid - 1);
    assert_eq!(repeated.result, short.result);
    assert_eq!(repeated.budget, short.budget);
    assert_eq!(repeated.requested, short.requested);
    for observation in [ample, exact, short, repeated] {
        drop_without_request::<ThreadCounter>(observation);
    }
}

#[path = "it/support/dependency_selection_inputs.rs"]
mod dependency_selection_inputs;

#[trace("TC-472", "FR-038-AC-511")]
#[test]
fn imported_nominals_work_denial_prepays_original_graph_pointer() {
    use quire_contract_model::{
        NativeJoinChargeKind, NativeJoinLocus, NativeJoinLocusSource, PackageReadResource,
        PackageReadStage,
    };
    let dependency_wire =
        dependency_selection_inputs::dependency_selection_package("test/geometry");
    let (_, dependency) = checked_package::admitted_dependency(&dependency_wire);
    let mut importer = dependency_selection_inputs::dependency_selection_package("test/importer");
    importer["lock"]["dependency_selections"] = serde_json::json!([{"identity":"test/geometry"}]);
    let bytes = canonical(&importer);
    let mut evidence = evidence_for(&importer);
    evidence
        .insert_dependency_package("test/geometry", dependency)
        .expect("dependency");
    let limits = generous_limits();
    let ample =
        read_allocation_proof::measure_at::<ThreadCounter>(&bytes, &evidence, limits, 5, u64::MAX);
    let CheckedPackageV2BudgetedReadResult::BudgetIncomplete(outcome) = &ample.result else {
        panic!(
            "the original Work5 control must return its affordable diagnostic: {:?}",
            ample.result
        );
    };
    assert_eq!(outcome.denied.resource, PackageReadResource::Work);
    assert_eq!(outcome.denied.amount, 1);
    assert_eq!(outcome.limit, 5);
    assert_eq!(outcome.consumed, 6);
    let path = outcome.path.as_ref().expect("original body pointer");
    assert_eq!(path.as_str(), "/semantic_graph/nodes");
    let locus = NativeJoinLocus {
        source: NativeJoinLocusSource::PackageRead(PackageReadStage::Graph, 0),
        form: "dependency_order",
    };
    assert_eq!(
        ample.budget.first_unpaid_locus(),
        Some(locus),
        "preserve the caller's inherited first-denial locus"
    );
    let denied = ample.budget.first_unpaid().expect("first unpaid Work");
    assert_eq!(denied.kind, NativeJoinChargeKind::Work);
    assert_eq!(denied.amount, 1);
    assert!(ample.budget.first_unpaid_read_storage().is_none());
    let paid = ample.budget.read_storage_consumed();
    let pointer_bytes = u64::try_from(path.as_str().len()).expect("bounded pointer");
    let prefix = paid
        .checked_sub(pointer_bytes)
        .expect("actual prefix before the only outcome pointer");
    let exact =
        read_allocation_proof::measure_at::<ThreadCounter>(&bytes, &evidence, limits, 5, paid);
    assert_eq!(exact.result, ample.result);
    assert_eq!(exact.requested, ample.requested);
    let short =
        read_allocation_proof::measure_at::<ThreadCounter>(&bytes, &evidence, limits, 5, paid - 1);
    let CheckedPackageV2BudgetedReadResult::Exhausted(exhausted) = &short.result else {
        panic!(
            "one below the actual diagnostic request must exhaust: {:?}",
            short.result
        );
    };
    assert_eq!(exhausted.denied, outcome.denied);
    assert_eq!(exhausted.configured_bound, 5);
    assert_eq!(exhausted.locus, locus);
    assert!(exhausted.pending.is_none());
    assert_eq!(short.budget.first_unpaid(), Some(denied));
    assert!(
        short.budget.first_unpaid_read_storage().is_none(),
        "the original Work denial remains first when its diagnostic storage is denied"
    );
    assert_eq!(
        short.requested, prefix,
        "the denied pointer makes no allocation request"
    );
    assert_eq!(short.budget.read_storage_consumed(), prefix);
    let repeated =
        read_allocation_proof::measure_at::<ThreadCounter>(&bytes, &evidence, limits, 5, paid - 1);
    assert_eq!(repeated.result, short.result);
    assert_eq!(repeated.budget, short.budget);
    assert_eq!(repeated.requested, short.requested);
    for observation in [ample, exact, short, repeated] {
        drop_without_request::<ThreadCounter>(observation);
    }
    let zero = read_allocation_proof::measure_at::<ThreadCounter>(&bytes, &evidence, limits, 5, 0);
    let CheckedPackageV2BudgetedReadResult::BudgetIncomplete(zero_outcome) = &zero.result else {
        panic!(
            "zero storage denies the earlier pointer-free catalog reservation: {:?}",
            zero.result
        );
    };
    assert_eq!(
        zero_outcome.denied.resource,
        PackageReadResource::ReadStorage
    );
    assert_eq!(zero_outcome.limit, 0);
    assert!(zero_outcome.path.is_none());
    let first_storage = zero
        .budget
        .first_unpaid_read_storage()
        .expect("first catalog storage request");
    assert_eq!(zero_outcome.denied.amount, first_storage);
    assert_eq!(
        zero_outcome.consumed,
        u64::try_from(first_storage).expect("bounded catalog reservation")
    );
    assert_eq!(zero.budget.read_storage_consumed(), 0);
    assert_eq!(zero.requested, 0);
    let zero_repeat =
        read_allocation_proof::measure_at::<ThreadCounter>(&bytes, &evidence, limits, 5, 0);
    assert_eq!(zero_repeat.result, zero.result);
    assert_eq!(zero_repeat.budget, zero.budget);
    assert_eq!(zero_repeat.requested, zero.requested);
    drop_without_request::<ThreadCounter>(zero);
    drop_without_request::<ThreadCounter>(zero_repeat);
}

#[trace("TC-472", "FR-038-AC-511")]
#[test]
fn dependency_closure_work_denial_prepays_original_lock_pointer() {
    use quire_contract_model::{
        NativeJoinChargeKind, NativeJoinLocus, NativeJoinLocusSource, PackageReadResource,
        PackageReadStage,
    };
    let dependency_wire =
        dependency_selection_inputs::dependency_selection_package("test/geometry");
    let (_, dependency) = checked_package::admitted_dependency(&dependency_wire);
    let mut importer = dependency_selection_inputs::dependency_selection_package("test/importer");
    importer["lock"]["dependency_selections"] = serde_json::json!([{"identity":"test/geometry"}]);
    let bytes = canonical(&importer);
    let mut evidence = evidence_for(&importer);
    evidence
        .insert_dependency_package("test/geometry", dependency)
        .expect("dependency");
    let limits = generous_limits();
    let ample =
        read_allocation_proof::measure_at::<ThreadCounter>(&bytes, &evidence, limits, 1, u64::MAX);
    let CheckedPackageV2BudgetedReadResult::BudgetIncomplete(outcome) = &ample.result else {
        panic!(
            "the original Work1 control must return its affordable diagnostic: {:?}",
            ample.result
        );
    };
    assert_eq!(outcome.denied.resource, PackageReadResource::Work);
    assert_eq!(outcome.denied.amount, 1);
    assert_eq!(outcome.limit, 1);
    assert_eq!(outcome.consumed, 2);
    let path = outcome.path.as_ref().expect("original body pointer");
    assert_eq!(path.as_str(), "/lock/dependency_selections/0");
    let locus = NativeJoinLocus {
        source: NativeJoinLocusSource::PackageRead(PackageReadStage::Lock, 0),
        form: "dependencies",
    };
    assert_eq!(
        ample.budget.first_unpaid_locus(),
        Some(locus),
        "preserve the caller's inherited first-denial locus"
    );
    let denied = ample.budget.first_unpaid().expect("first unpaid Work");
    assert_eq!(denied.kind, NativeJoinChargeKind::Work);
    assert_eq!(denied.amount, 1);
    assert!(ample.budget.first_unpaid_read_storage().is_none());
    let paid = ample.budget.read_storage_consumed();
    let pointer_bytes = u64::try_from(path.as_str().len()).expect("bounded pointer");
    let prefix = paid
        .checked_sub(pointer_bytes)
        .expect("actual prefix before the only outcome pointer");
    let exact =
        read_allocation_proof::measure_at::<ThreadCounter>(&bytes, &evidence, limits, 1, paid);
    assert_eq!(exact.result, ample.result);
    assert_eq!(exact.requested, ample.requested);
    let short =
        read_allocation_proof::measure_at::<ThreadCounter>(&bytes, &evidence, limits, 1, paid - 1);
    let CheckedPackageV2BudgetedReadResult::Exhausted(exhausted) = &short.result else {
        panic!(
            "one below the actual diagnostic request must exhaust: {:?}",
            short.result
        );
    };
    assert_eq!(exhausted.denied, outcome.denied);
    assert_eq!(exhausted.configured_bound, 1);
    assert_eq!(exhausted.locus, locus);
    assert!(exhausted.pending.is_none());
    assert_eq!(short.budget.first_unpaid(), Some(denied));
    assert!(
        short.budget.first_unpaid_read_storage().is_none(),
        "the original Work denial remains first when its diagnostic storage is denied"
    );
    assert_eq!(
        short.requested, prefix,
        "the denied pointer makes no allocation request"
    );
    assert_eq!(short.budget.read_storage_consumed(), prefix);
    let repeated =
        read_allocation_proof::measure_at::<ThreadCounter>(&bytes, &evidence, limits, 1, paid - 1);
    assert_eq!(repeated.result, short.result);
    assert_eq!(repeated.budget, short.budget);
    assert_eq!(repeated.requested, short.requested);
    for observation in [ample, exact, short, repeated] {
        drop_without_request::<ThreadCounter>(observation);
    }
    let zero = read_allocation_proof::measure_at::<ThreadCounter>(&bytes, &evidence, limits, 1, 0);
    let CheckedPackageV2BudgetedReadResult::BudgetIncomplete(zero_outcome) = &zero.result else {
        panic!(
            "zero storage denies the earlier pointer-free catalog reservation: {:?}",
            zero.result
        );
    };
    assert_eq!(
        zero_outcome.denied.resource,
        PackageReadResource::ReadStorage
    );
    assert_eq!(zero_outcome.limit, 0);
    assert!(zero_outcome.path.is_none());
    let first_storage = zero
        .budget
        .first_unpaid_read_storage()
        .expect("first catalog storage request");
    assert_eq!(zero_outcome.denied.amount, first_storage);
    assert_eq!(
        zero_outcome.consumed,
        u64::try_from(first_storage).expect("bounded catalog reservation")
    );
    assert_eq!(zero.budget.read_storage_consumed(), 0);
    assert_eq!(zero.requested, 0);
    let zero_repeat =
        read_allocation_proof::measure_at::<ThreadCounter>(&bytes, &evidence, limits, 1, 0);
    assert_eq!(zero_repeat.result, zero.result);
    assert_eq!(zero_repeat.budget, zero.budget);
    assert_eq!(zero_repeat.requested, zero.requested);
    drop_without_request::<ThreadCounter>(zero);
    drop_without_request::<ThreadCounter>(zero_repeat);
}

#[trace("TC-472", "FR-038-AC-511")]
#[test]
fn nonidentifier_enum_member_refusal_is_paid_before_construction() {
    let mut wire = v2_nominal();
    wire["semantic_graph"]["nodes"][0]["body"]["value"] = serde_json::json!("Gold Plus");
    let bytes = canonical(&wire);
    let evidence = evidence_for(&wire);
    let observation = read_allocation_proof::measure_at::<ThreadCounter>(
        &bytes,
        &evidence,
        generous_limits(),
        u64::MAX,
        u64::MAX,
    );
    let CheckedPackageV2BudgetedReadResult::Refused(refusal) = &observation.result else {
        panic!(
            "nonidentifier scalar enum must refuse: {:?}",
            observation.result
        );
    };
    assert_eq!(
        refusal.code,
        quire_contract_model::CheckedPackageRefusalCode::InvalidSemanticGraph
    );
    assert!(refusal.cause.is_none());
    assert_eq!(refusal.loci.len(), 1);
    assert_eq!(
        refusal.loci[0].path.as_ref().map(|path| path.as_str()),
        Some("/semantic_graph/nodes/0/body/value")
    );
    assert!(refusal.locus.is_none());
    assert!(refusal.loci[0].source_region.is_none());
    assert!(refusal.contract_version.is_none());
    assert!(refusal.document_pointer.is_none());
    assert!(refusal.extension_detail.is_none());
    assert!(refusal.protocol_detail.is_none());
    assert!(refusal.terminal_duplicate.is_none());
    let paid = observation.budget.read_storage_consumed();
    let exact = read_allocation_proof::measure_at::<ThreadCounter>(
        &bytes,
        &evidence,
        generous_limits(),
        u64::MAX,
        paid,
    );
    assert_eq!(exact.result, observation.result);
    assert_eq!(exact.requested, observation.requested);
    let short = read_allocation_proof::measure_at::<ThreadCounter>(
        &bytes,
        &evidence,
        generous_limits(),
        u64::MAX,
        paid - 1,
    );
    let CheckedPackageV2BudgetedReadResult::Exhausted(exhausted) = &short.result else {
        panic!(
            "the complete refusal must be denied before construction: {:?}",
            short.result
        );
    };
    assert_eq!(
        exhausted.denied.resource,
        quire_contract_model::PackageReadResource::ReadStorage
    );
    assert_eq!(exhausted.configured_bound, paid - 1);
    assert_eq!(
        exhausted.pending,
        Some(quire_contract_model::CheckedPackagePendingOutcome::Refused(
            refusal.code
        ))
    );
    let request = short
        .budget
        .first_unpaid_read_storage()
        .expect("actual complete refusal request");
    assert_eq!(exhausted.denied.amount, request);
    assert_eq!(u128::from(short.requested) + request, u128::from(paid));
    assert_eq!(
        exhausted.locus.source,
        quire_contract_model::NativeJoinLocusSource::PackageRead(
            quire_contract_model::PackageReadStage::Graph,
            0
        )
    );
    assert_eq!(exhausted.locus.form, "diagnostic");
    let prefix = short.requested;
    // No remaining diagnostic storage: the same semantic refusal remains pending,
    // and neither its pointer nor record/location is constructed.
    let zero_diagnostic = read_allocation_proof::measure_at::<ThreadCounter>(
        &bytes,
        &evidence,
        generous_limits(),
        u64::MAX,
        prefix,
    );
    let CheckedPackageV2BudgetedReadResult::Exhausted(zero_outcome) = &zero_diagnostic.result
    else {
        panic!("zero diagnostic storage must exhaust");
    };
    assert_eq!(zero_outcome.pending, exhausted.pending);
    assert_eq!(zero_outcome.denied, exhausted.denied);
    assert_eq!(zero_outcome.configured_bound, prefix);
    assert_eq!(zero_outcome.locus, exhausted.locus);
    assert_eq!(zero_diagnostic.requested, prefix);
    let repeated = read_allocation_proof::measure_at::<ThreadCounter>(
        &bytes,
        &evidence,
        generous_limits(),
        u64::MAX,
        paid - 1,
    );
    assert_eq!(repeated.result, short.result);
    assert_eq!(repeated.budget, short.budget);
    assert_eq!(repeated.requested, short.requested);
    for observed in [observation, exact, short, zero_diagnostic, repeated] {
        drop_without_request::<ThreadCounter>(observed);
    }
}

/// A call whose callee function binds `parameters` pays for the declared
/// parameter types it checks the call's arguments against (FR-347).
#[trace("TC-472", "FR-038-AC-511")]
#[test]
fn function_call_parameter_check_has_no_unpaid_allocation() {
    initialize_operation_catalog().expect("catalogue gate");
    let reference = |key: &str| serde_json::json!({"term": "reference", "target": node_id(key)});
    let literal = |type_key: &str, kind: &str, value: &str| serde_json::json!({"term": "literal", "type": node_id(type_key), "value_kind": kind, "value": value});
    let mut package = v2_all_families();
    let nodes = package["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    let function = nodes
        .iter_mut()
        .find(|node| node["node_id"] == node_id(&family_key("8080")))
        .expect("the fixture's function node");
    function["body"] = serde_json::json!({"term": "aggregate", "members": [{
        "term": "binding", "name": "parameters",
        "value": {"term": "aggregate", "members": [reference("parameter/0")]}
    }]});
    nodes.push(serde_json::json!({
        "node_id": node_id("parameter/0"),
        "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": "value",
        "semantic_form": "parameter",
        "semantic_type": node_id(&family_key("aaaa")),
        "dependencies": [],
        "occurrences": [{"role": "expression", "ordinal": 0}],
        "body": {"term": "aggregate", "members": [
            {"term": "binding", "name": "name", "value": literal(&family_key("a1a1"), "text", "x")},
            {"term": "binding", "name": "level", "value": literal(&family_key("a3a3"), "integer", "0")},
        ]},
    }));
    let call = nodes
        .iter_mut()
        .find(|node| node["node_tag"] == "function" && node["body"]["term"] == "application")
        .expect("the fixture's function call node");
    call["body"]["arguments"]
        .as_array_mut()
        .expect("arguments")
        .push(reference("parameter/0"));
    settle(&mut package);
    let observation = read_allocation_proof::measure_at::<ThreadCounter>(
        &canonical(&package),
        &evidence_for(&package),
        generous_limits(),
        u64::MAX,
        u64::MAX,
    );
    assert!(
        matches!(
            observation.result,
            CheckedPackageV2BudgetedReadResult::Admitted(_)
        ),
        "the parameter-bound call admits: {:?}",
        observation.result
    );
    drop_without_request::<ThreadCounter>(observation);
}
