// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The wire-check and node-index stages of the budget-charged strict read
//! charge their read storage exactly, at the loci FR-038 "Budget-charged
//! strict read" names (TC-472).

use crate::support::checked_package::{canonical, evidence_for, v2_all_families};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedPackageReadLimits, CheckedPackageV2, CheckedPackageV2BudgetedReadResult,
    NativeJoinBudget, NativeJoinLocus, NativeJoinLocusSource, PackageReadStage,
};

/// What one read under a read-storage limit ended at.
struct Outcome {
    /// The read admitted the package.
    admitted: bool,
    /// The stage and locus of the first unpaid request, when there was one.
    denial: Option<(NativeJoinLocus, u128)>,
}

fn read_under(limit: u64) -> Outcome {
    let value = v2_all_families();
    let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
    let result = CheckedPackageV2::read_with_budget(
        &canonical(&value),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&value),
        &mut budget,
    );
    Outcome {
        admitted: matches!(result, CheckedPackageV2BudgetedReadResult::Admitted(_)),
        denial: budget
            .first_unpaid_locus()
            .zip(budget.first_unpaid_read_storage()),
    }
}

fn stage_of(locus: NativeJoinLocus) -> Option<PackageReadStage> {
    match locus.source {
        NativeJoinLocusSource::PackageRead(stage, _) => Some(stage),
        _ => None,
    }
}

/// The least read-storage limit at which the read gets past `stage`: it is
/// admitted, or its first unpaid request is of a later stage. The stages run
/// in order, so the answer is found by bisection.
fn limit_past(stage: PackageReadStage) -> u64 {
    let past = |limit: u64| {
        let outcome = read_under(limit);
        outcome.admitted
            || outcome
                .denial
                .and_then(|(locus, _)| stage_of(locus))
                .is_some_and(|denied| denied > stage)
    };
    let mut measure = NativeJoinBudget::unlimited();
    let value = v2_all_families();
    let _ = CheckedPackageV2::read_with_budget(
        &canonical(&value),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&value),
        &mut measure,
    );
    let (mut low, mut high) = (0, measure.read_storage_consumed());
    assert!(past(high), "the whole requirement gets past every stage");
    while low < high {
        let middle = low + (high - low) / 2;
        if past(middle) {
            high = middle;
        } else {
            low = middle + 1;
        }
    }
    low
}

fn graph_node_count() -> usize {
    v2_all_families()["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .len()
}

fn denial_after(limit: u64) -> (NativeJoinLocus, u128) {
    read_under(limit)
        .denial
        .expect("a read one request short is denied")
}

/// The first request of the wire checks is the decoded kind table, one entry
/// per graph node, at graph node 0 with the form `"flat_wire"`; the allowance
/// that pays everything before it admits no more.
#[trace("TC-472", "FR-038-AC-499")]
#[test]
fn the_wire_checks_open_with_the_kind_table_at_flat_wire() {
    let before = limit_past(PackageReadStage::WireDecode);
    let (locus, asked) = denial_after(before);
    assert_eq!(
        locus,
        NativeJoinLocus {
            source: NativeJoinLocusSource::PackageRead(PackageReadStage::WireChecks, 0),
            form: "flat_wire",
        }
    );
    let nodes = graph_node_count() as u128;
    assert!(asked > 0 && asked % nodes == 0, "one entry per node");
    // Paying the table moves the first unpaid request on within the stage;
    // one byte under the allowance stops before the stage.
    let after_table = denial_after(before + u64::try_from(asked).expect("small"));
    assert_eq!(
        stage_of(after_table.0),
        Some(PackageReadStage::WireChecks)
    );
    assert_ne!(after_table.0.form, "flat_wire");
    let short = stage_of(denial_after(before - 1).0).expect("a package-read stage");
    assert!(short < PackageReadStage::WireChecks);
}

/// The wire checks' last request paid, the node-index stage's first is the
/// next one denied, at `"node_index"`; the stage's last request is the key
/// table, one `Arc` per graph node, at graph node 0, and the allowance that
/// pays it gets past the stage while one byte less is denied exactly there.
#[trace("TC-472", "FR-038-AC-500", "FR-038-AC-501")]
#[test]
fn the_node_index_stage_is_charged_to_the_byte_and_ends_with_the_key_table() {
    let before = limit_past(PackageReadStage::WireChecks);
    let (first, _) = denial_after(before);
    assert_eq!(stage_of(first), Some(PackageReadStage::NodeIndex));
    assert_eq!(first.form, "node_index");

    let end = limit_past(PackageReadStage::NodeIndex);
    let (last, asked) = denial_after(end - 1);
    assert_eq!(
        last,
        NativeJoinLocus {
            source: NativeJoinLocusSource::PackageRead(PackageReadStage::NodeIndex, 0),
            form: "node_index",
        }
    );
    assert_eq!(
        asked,
        (graph_node_count() * std::mem::size_of::<usize>()) as u128
    );
    let past = read_under(end);
    assert!(
        past.admitted
            || past
                .denial
                .and_then(|(locus, _)| stage_of(locus))
                .is_some_and(|stage| stage > PackageReadStage::NodeIndex)
    );
}
