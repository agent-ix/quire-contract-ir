// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! A sufficient budget returns the unlimited budget's outcome (TC-472).

use crate::qspec_read_proof::assert_admitted_views_equal;
use crate::support::checked_package::{
    canonical, evidence_for, index_fixture_node_ids, v2_all_families, v2_nominal,
    v2_reference_chain,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    initialize_operation_catalog, CheckedPackageEvidence, CheckedPackageReadLimits,
    CheckedPackageRefusalCode, CheckedPackageV2, CheckedPackageV2BudgetedReadResult,
    CheckedPackageV2ReadResult, NativeJoinBudget,
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
                assert_admitted_views_equal(left, right, label);
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
            (left, right) => panic!("{label}: mismatched public outcomes: {left:?} / {right:?}"),
        }

        // Limits exactly at, and well above, the measured requirement.
        for (work_limit, storage_limit) in [
            (work, storage),
            (work, u64::MAX),
            (u64::MAX, storage),
            (work.saturating_add(1), storage.saturating_add(1)),
        ] {
            let mut sufficient =
                NativeJoinBudget::new(work_limit, 0).with_read_storage_limit(storage_limit);
            let result = budgeted(input, &mut sufficient);
            assert_eq!(
                result, reference,
                "{label}: limits {work_limit}/{storage_limit}"
            );
            if let (
                CheckedPackageV2BudgetedReadResult::Admitted(expected),
                CheckedPackageV2BudgetedReadResult::Admitted(actual),
            ) = (&reference, &result)
            {
                assert_admitted_views_equal(expected, actual, label);
            }
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
    for input in corpus() {
        let mut measure = NativeJoinBudget::unlimited();
        let reference = budgeted(&input, &mut measure);
        let work = measure.consumed().work;
        let storage = measure.read_storage_consumed();
        for (work_limit, storage_limit) in [
            (work.saturating_sub(1), u64::MAX),
            (u64::MAX, storage.saturating_sub(1)),
            (work.saturating_sub(1), storage.saturating_sub(1)),
        ] {
            // A zero-cost structural refusal (for example Bytes) has no
            // last paid charge to deny, so its full refusal must remain equal.
            let run = || {
                let mut budget =
                    NativeJoinBudget::new(work_limit, 0).with_read_storage_limit(storage_limit);
                let result = budgeted(&input, &mut budget);
                (
                    result,
                    budget.consumed(),
                    budget.read_storage_consumed(),
                    budget.first_unpaid(),
                    budget.first_unpaid_read_storage(),
                    budget.first_unpaid_locus(),
                )
            };
            let first = run();
            assert_eq!(
                first,
                run(),
                "{}: {work_limit}/{storage_limit}",
                input.label
            );
            if first.3.is_some() || first.4.is_some() {
                assert!(
                    first.5.is_some(),
                    "{}: denied charge has a locus",
                    input.label
                );
                assert!(
                    matches!(
                        first.0,
                        CheckedPackageV2BudgetedReadResult::BudgetIncomplete(_)
                            | CheckedPackageV2BudgetedReadResult::Exhausted(_)
                    ),
                    "{}: no package or semantic refusal after budget denial",
                    input.label
                );
            } else {
                assert_eq!(
                    first.0, reference,
                    "{}: structural refusal without budget denial",
                    input.label
                );
            }
        }
    }
}

/// An earlier header fault wins over a second malformed member under every
/// sufficient resource combination; the complete typed refusal is preserved.
#[trace("TC-472", "FR-038-AC-512")]
#[test]
fn sufficient_budgets_preserve_the_first_refusal_with_two_header_faults() {
    let mut wire = v2_all_families();
    // Prepare genuine node identities and evidence before introducing faults:
    // fixture indexing requires the package name that this input removes.
    index_fixture_node_ids(&mut wire);
    let evidence = evidence_for(&wire);
    wire["contract_version"] = Value::String("unknown-version".into());
    wire.as_object_mut().expect("package").remove("name");
    let input = Input {
        label: "version before missing name".into(),
        bytes: canonical(&wire),
        evidence,
        limits: CheckedPackageReadLimits::bounded(),
    };
    let CheckedPackageV2ReadResult::Refused(expected) = plain(&input) else {
        panic!("two header faults must refuse");
    };
    assert_eq!(
        expected.code,
        CheckedPackageRefusalCode::UnknownContractVersion
    );
    let mut measured = NativeJoinBudget::unlimited();
    let CheckedPackageV2BudgetedReadResult::Refused(reference) = budgeted(&input, &mut measured)
    else {
        panic!("the unlimited budget must preserve the semantic refusal");
    };
    assert_eq!(reference, expected);
    for (work, storage) in [
        (measured.consumed().work, measured.read_storage_consumed()),
        (u64::MAX, measured.read_storage_consumed()),
        (measured.consumed().work, u64::MAX),
    ] {
        let mut budget = NativeJoinBudget::new(work, 0).with_read_storage_limit(storage);
        let CheckedPackageV2BudgetedReadResult::Refused(actual) = budgeted(&input, &mut budget)
        else {
            panic!("sufficient {work}/{storage} must preserve the first refusal");
        };
        assert_eq!(actual, expected);
        assert_eq!(budget.first_unpaid(), None);
        assert_eq!(budget.first_unpaid_read_storage(), None);
    }
}

/// Core accounting performs no derived lowering; explicit AC512 comparisons
/// keep every public-consumer check even without a full-proof environment.
#[trace("TC-472", "FR-038-AC-512")]
#[test]
fn core_outcome_comparison_does_not_lower_but_explicit_view_proof_visits_every_node() {
    use crate::qspec_read_proof::equivalence::{take_oracle_visits, OracleVisits, ReadViewScope};
    let input = input("original nominal scope control", &v2_nominal());
    let left = plain(&input);
    let right = budgeted(&input, &mut NativeJoinBudget::unlimited());
    let (
        CheckedPackageV2ReadResult::Admitted(left),
        CheckedPackageV2BudgetedReadResult::Admitted(right_package),
    ) = (&left, &right)
    else {
        panic!("the genuine nominal control must admit");
    };
    let nodes = left.resolved_graph().nodes.len();
    assert!(nodes > 0);
    assert_eq!(
        left.graph(),
        right_package.graph(),
        "same genuine retained graph"
    );
    take_oracle_visits();
    ReadViewScope::Core.admitted(left, right_package, "core original nominal result");
    assert_eq!(take_oracle_visits(), OracleVisits::default());
    assert_admitted_views_equal(left, right_package, "explicit original nominal views");
    let full = take_oracle_visits();
    assert_eq!(full.comparisons, 1);
    assert_eq!(full.nodes, nodes);
    assert_eq!(full.lower_calls, 2 * nodes);
}

#[cfg(feature = "test-support")]
#[trace("TC-472", "FR-038-AC-511", "FR-038-AC-512")]
#[test]
fn core_observer_keeps_reader_values_and_full_observer_checks_every_node() {
    use crate::qspec_read_proof::{equivalence::take_oracle_visits, Counter, ReadViewScope};
    struct ScopeCounter;
    impl Counter for ScopeCounter {
        fn measure(read: impl FnOnce()) -> u64 {
            crate::measure_qspec_read_requests(read)
        }
    }
    let input = input("original nominal observer scope", &v2_nominal());
    take_oracle_visits();
    let (core, core_events) = crate::qspec_read_proof::observe_native_with_scope::<ScopeCounter>(
        &input.bytes,
        &input.evidence,
        input.limits,
        u64::MAX,
        u64::MAX,
        ReadViewScope::Core,
    );
    let CheckedPackageV2BudgetedReadResult::Admitted(package) = &core.result else {
        panic!("genuine nominal control must admit");
    };
    let nodes = package.resolved_graph().nodes.len();
    assert!(nodes > 0);
    assert!(!core_events.is_empty());
    assert_eq!(take_oracle_visits().lower_calls, 0);
    let (full, full_events) = crate::qspec_read_proof::observe_native_with_scope::<ScopeCounter>(
        &input.bytes,
        &input.evidence,
        input.limits,
        u64::MAX,
        u64::MAX,
        ReadViewScope::Full,
    );
    assert_eq!(core.result, full.result);
    assert_eq!(core.requested, full.requested);
    assert_eq!(core.budget, full.budget);
    assert_eq!(core_events, full_events);
    let full_visits = take_oracle_visits();
    assert_eq!(
        full_visits.comparisons, 2,
        "OFF/count and OFF/record full comparisons"
    );
    assert_eq!(full_visits.nodes, 2 * nodes);
    assert_eq!(full_visits.lower_calls, 4 * nodes);
}

#[cfg(feature = "test-support")]
#[trace("TC-472", "FR-038-AC-511", "FR-038-AC-512")]
#[test]
fn direct_full_native_proof_checks_admitted_views_without_a_mode_environment() {
    use crate::qspec_read_proof::{equivalence::take_oracle_visits, Counter};
    struct ScopeCounter;
    impl Counter for ScopeCounter {
        fn measure(read: impl FnOnce()) -> u64 {
            crate::measure_qspec_read_requests(read)
        }
    }
    let input = input("original nominal direct full proof", &v2_nominal());
    let reference = crate::qspec_read_proof::measure_at::<ScopeCounter>(
        &input.bytes,
        &input.evidence,
        input.limits,
        u64::MAX,
        u64::MAX,
    );
    let CheckedPackageV2BudgetedReadResult::Admitted(package) = &reference.result else {
        panic!("genuine nominal control must admit");
    };
    let nodes = package.resolved_graph().nodes.len();
    take_oracle_visits();
    let (events, results, complete) = crate::qspec_read_proof::full_native_boundaries(
        &input.bytes,
        input.limits,
        &input.evidence,
        &reference,
        &[],
        None,
    );
    assert_eq!(complete["outcome"], "admitted");
    assert_eq!(
        complete["public_view_equivalence"],
        "derived-views-and-per-node-lowering"
    );
    assert!(!events.is_empty());
    assert!(!results.is_empty());
    let visits = take_oracle_visits();
    assert!(visits.comparisons > 0);
    assert_eq!(visits.nodes, visits.comparisons * nodes);
    assert_eq!(visits.lower_calls, 2 * visits.nodes);
}
