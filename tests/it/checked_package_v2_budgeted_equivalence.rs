// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! A sufficient budget returns the unlimited budget's outcome, and the
//! retained package reports the size its read charged (TC-472).

use crate::support::checked_package::{
    canonical, evidence_for, index_fixture_node_ids, v2_all_families, v2_nominal,
    v2_reference_chain,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    initialize_operation_catalog, CheckedPackageEvidence, CheckedPackageReadLimits,
    CheckedPackageV2, CheckedPackageV2BudgetedReadResult, CheckedPackageV2ReadResult,
    NativeJoinBudget,
};
use serde_json::Value;

struct Input {
    label: String,
    bytes: Vec<u8>,
    evidence: CheckedPackageEvidence,
    limits: CheckedPackageReadLimits,
}

fn input(label: &str, value: &Value) -> Input {
    let mut wire = value.clone();
    index_fixture_node_ids(&mut wire);
    Input {
        label: label.to_owned(),
        bytes: canonical(&wire),
        evidence: evidence_for(&wire),
        limits: CheckedPackageReadLimits::bounded(),
    }
}

/// The TC-048 vectors this repository builds (the shared fixtures and every
/// admitted catalogue control), then refused and incomplete variants of them.
fn corpus() -> Vec<Input> {
    let mut inputs = vec![
        input("all_families", &v2_all_families()),
        input("nominal", &v2_nominal()),
    ];
    let catalog: Value = serde_json::from_str(
        quire_verification_contracts::operation_catalog::CHECKED_OPERATION_CATALOG_V1,
    )
    .expect("the reader's catalogue is JSON");
    for entry in catalog["operations"].as_array().expect("operations") {
        let identity = entry["identity"].as_str().expect("identity");
        if let Some(package) =
            crate::checked_package_v2_catalogue_differential::admitted_fixture(identity)
        {
            inputs.push(input(identity, &package));
        }
    }
    let (chain, _) = v2_reference_chain(64);
    let mut chain_limits = CheckedPackageReadLimits::bounded();
    chain_limits.nodes = chain_limits.nodes.max(10_000);
    inputs.push(Input {
        label: "reference_chain".to_owned(),
        evidence: evidence_for(&v2_all_families()),
        bytes: chain,
        limits: chain_limits,
    });

    let base = input("base", &v2_all_families());
    let mut refused = Vec::new();
    refused.push(Input {
        label: "truncated".to_owned(),
        bytes: base.bytes[..base.bytes.len() / 2].to_vec(),
        evidence: evidence_for(&v2_all_families()),
        limits: base.limits,
    });
    refused.push(Input {
        label: "not_json".to_owned(),
        bytes: b"not json".to_vec(),
        evidence: evidence_for(&v2_all_families()),
        limits: base.limits,
    });
    let mut missing_name = v2_all_families();
    missing_name.as_object_mut().expect("object").remove("name");
    refused.push(Input {
        label: "missing_name".to_owned(),
        bytes: canonical(&missing_name),
        evidence: evidence_for(&v2_all_families()),
        limits: base.limits,
    });
    let mut over_bytes = input("over_bytes", &v2_all_families());
    over_bytes.limits.bytes = 16;
    refused.push(over_bytes);
    inputs.extend(refused);
    inputs
}

fn plain(input: &Input) -> CheckedPackageV2ReadResult {
    CheckedPackageV2::read(&input.bytes, input.limits, &input.evidence)
}

fn budgeted(input: &Input, budget: &mut NativeJoinBudget) -> CheckedPackageV2BudgetedReadResult {
    CheckedPackageV2::read_with_budget(&input.bytes, input.limits, &input.evidence, budget)
}

/// The shape of a result, for outcomes the unbudgeted and budgeted entries
/// both express: variant, code, cause, loci and pointers all print.
fn shape(result: &impl std::fmt::Debug) -> String {
    format!("{result:?}")
}

/// Over the corpus, every budget at least as large as the read's measured
/// requirement returns the unlimited budget's outcome, equal to the
/// unbudgeted read's; a repeated read charges the same and stops alike.
#[trace("TC-472", "FR-038-AC-512")]
#[test]
fn a_sufficient_budget_returns_the_unlimited_budgets_outcome_over_the_corpus() {
    initialize_operation_catalog().expect("catalogue reads");
    let inputs = corpus();
    assert!(inputs.len() > 10, "the corpus is not trivial");
    let mut admitted = 0;
    for input in &inputs {
        let label = &input.label;
        let mut unlimited = NativeJoinBudget::unlimited();
        let reference = budgeted(input, &mut unlimited);
        let work = unlimited.consumed().work;
        let storage = unlimited.read_storage_consumed();

        // The unbudgeted entry maps onto the same outcome.
        let unbudgeted = plain(input);
        match (&unbudgeted, &reference) {
            (
                CheckedPackageV2ReadResult::Admitted(left),
                CheckedPackageV2BudgetedReadResult::Admitted(right),
            ) => {
                assert_eq!(left, right, "{label}");
                admitted += 1;
            }
            (
                CheckedPackageV2ReadResult::Refused(left),
                CheckedPackageV2BudgetedReadResult::Refused(right),
            ) => {
                assert_eq!(left, right, "{label}");
            }
            (
                CheckedPackageV2ReadResult::Incomplete(left),
                CheckedPackageV2BudgetedReadResult::Incomplete(right),
            ) => assert_eq!(left, right, "{label}"),
            (
                CheckedPackageV2ReadResult::Incomplete(left),
                CheckedPackageV2BudgetedReadResult::BudgetIncomplete(right),
            ) => {
                assert_eq!(left.consumed, right.consumed, "{label}");
                assert_eq!(left.path, right.path, "{label}");
            }
            (left, right) => assert_eq!(shape(left), shape(right), "{label}"),
        }

        // Limits exactly at, and well above, the measured requirement.
        for (work_limit, storage_limit) in [(work, storage), (u64::MAX, u64::MAX)] {
            let mut sufficient =
                NativeJoinBudget::new(work_limit, 0).with_read_storage_limit(storage_limit);
            let result = budgeted(input, &mut sufficient);
            assert_eq!(
                result, reference,
                "{label}: limits {work_limit}/{storage_limit}"
            );
            assert_eq!(shape(&result), shape(&reference), "{label}");
            assert_eq!(sufficient.consumed().work, work, "{label}");
            assert_eq!(sufficient.read_storage_consumed(), storage, "{label}");
        }

        // Repeating the read charges equally and stops at the same place.
        let mut again = NativeJoinBudget::unlimited();
        assert_eq!(budgeted(input, &mut again), reference, "{label}");
        assert_eq!(again.consumed(), unlimited.consumed(), "{label}");
        assert_eq!(again.read_storage_consumed(), storage, "{label}");
        assert_eq!(again.first_unpaid(), unlimited.first_unpaid(), "{label}");
        assert_eq!(
            again.first_unpaid_locus(),
            unlimited.first_unpaid_locus(),
            "{label}"
        );
    }
    assert!(admitted >= 2, "the corpus admits genuine packages");
}

/// Reading the same bytes twice under one-below limits stops at the same first
/// unpaid charge and locus.
#[trace("TC-472", "FR-038-AC-512")]
#[test]
fn a_repeated_insufficient_read_stops_at_the_same_first_unpaid_charge() {
    initialize_operation_catalog().expect("catalogue reads");
    let input = input("all_families", &v2_all_families());
    let mut measure = NativeJoinBudget::unlimited();
    budgeted(&input, &mut measure);
    let storage = measure.read_storage_consumed();
    let run = || {
        let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(storage - 1);
        let result = budgeted(&input, &mut budget);
        (
            shape(&result),
            budget.read_storage_consumed(),
            budget.first_unpaid_read_storage(),
            budget.first_unpaid_locus(),
        )
    };
    assert_eq!(run(), run());
}

/// Each retained field of an admitted package reports its charged size, and
/// the retained total equals the Retained stage's charge of the read.
///
/// Specification only: `CheckedPackageV2` has no retained-size inspection yet.
/// It needs, from the retained-structures slice (F1), a public or
/// `#[doc(hidden)]` accessor returning one `(field name, charged bytes)` pair
/// per retained field and the sum of those, to compare with the read-storage
/// charged between the Retained stage's first and last request.
#[trace("TC-472", "FR-038-AC-513")]
#[test]
#[ignore = "waits on F1 retained-size API (luna)"]
fn the_retained_package_reports_each_field_charge_and_their_total() {
    initialize_operation_catalog().expect("catalogue reads");
    let input = input("all_families", &v2_all_families());
    let mut budget = NativeJoinBudget::unlimited();
    let CheckedPackageV2BudgetedReadResult::Admitted(_package) = budgeted(&input, &mut budget)
    else {
        panic!("the fixture admits");
    };
    unimplemented!(
        "compare package retained-size inspection (per-field charge, total) \
         with the Retained stage's read-storage charge"
    );
}
