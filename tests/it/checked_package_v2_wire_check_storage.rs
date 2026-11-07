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

/// The least read-storage limit at which the read is past runtime phase
/// `done`: it is admitted, or the stage and form of its first unpaid request
/// fall in a later phase. Phases run in order, so bisection finds it.
fn limit_past(done: u8) -> u64 {
    let past = |stage: PackageReadStage, form: &str| phase(stage, form) > done;
    let reached = |limit: u64| {
        let outcome = read_under(limit);
        outcome.admitted
            || outcome
                .denial
                .and_then(|(locus, _)| stage_of(locus).map(|stage| past(stage, locus.form)))
                .unwrap_or(false)
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
    assert!(reached(high), "the whole requirement reaches every stage");
    while low < high {
        let middle = low + (high - low) / 2;
        if reached(middle) {
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

/// The wire checks open with the owner-schema stage's worklist at graph node 0
/// and `"owner_schema"`; once the owner schema is paid, the next is the decoded
/// kind table, one entry per graph node, at graph node 0 and `"flat_wire"`.
#[trace("TC-472", "FR-038-AC-499")]
#[test]
fn the_wire_checks_open_with_the_owner_schema_then_the_kind_table() {
    let before = limit_past(DECODE);
    let (opening, _) = denial_after(before);
    assert_eq!(
        opening,
        NativeJoinLocus {
            source: NativeJoinLocusSource::PackageRead(PackageReadStage::WireChecks, 0),
            form: "owner_schema",
        }
    );
    let owner_paid = limit_past(OWNER_SCHEMA);
    let (locus, asked) = denial_after(owner_paid);
    assert_eq!(
        locus,
        NativeJoinLocus {
            source: NativeJoinLocusSource::PackageRead(PackageReadStage::WireChecks, 0),
            form: "flat_wire",
        }
    );
    let nodes = graph_node_count() as u128;
    assert!(asked > 0 && asked % nodes == 0, "one entry per node");
    // Paying the table moves the first unpaid request on within the stage.
    let after_table = denial_after(owner_paid + u64::try_from(asked).expect("small"));
    assert_eq!(stage_of(after_table.0), Some(PackageReadStage::WireChecks));
    assert_ne!(after_table, (locus, asked));
    assert_ne!(after_table.0.form, "owner_schema");
}

/// The wire checks' last request paid, the node-index stage's first is the
/// next one denied, at `"node_index"`; the stage's last request is the key
/// table, one `Arc` per graph node, at graph node 0, and the allowance that
/// pays it gets past the stage while one byte less is denied exactly there.
#[trace("TC-472", "FR-038-AC-500", "FR-038-AC-501")]
#[test]
fn the_node_index_stage_is_charged_to_the_byte_and_ends_with_the_key_table() {
    let before = limit_past(FLAT_WIRE);
    let (first, _) = denial_after(before);
    assert_eq!(stage_of(first), Some(PackageReadStage::NodeIndex));
    assert_eq!(first.form, "node_index");

    let end = limit_past(NODE_INDEX);
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
                .and_then(|(locus, _)| stage_of(locus).map(|stage| phase(stage, locus.form)))
                .is_some_and(|denied| denied > NODE_INDEX)
    );
}
