// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! A sufficient budget returns the unlimited budget's outcome (TC-472).

use crate::support::checked_package::{
    canonical, evidence_for, index_fixture_node_ids, v2_all_families, v2_nominal,
    v2_reference_chain,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    initialize_operation_catalog, CheckedModelPopulationCatalogLimits, CheckedNodeTag,
    CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageRefusalCode, CheckedPackageV2,
    CheckedPackageV2BudgetedReadResult, CheckedPackageV2ReadResult, CompleteLoweringProfileV2,
    LoweringProfileRef, NativeJoinBudget,
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

/// Content equality excludes admission-derived model tables and lowering facts.
/// Exercise their public consumers as well as the retained document content.
fn assert_admitted_views_equal(left: &CheckedPackageV2, right: &CheckedPackageV2, label: &str) {
    assert_eq!(
        left, right,
        "{label}: retained content and selected documents"
    );
    assert_eq!(left.graph(), right.graph(), "{label}: wire graph");
    assert_eq!(
        left.resolved_graph(),
        right.resolved_graph(),
        "{label}: resolved graph"
    );
    assert_eq!(left.source_map(), right.source_map(), "{label}: source map");
    assert_eq!(
        left.diagnostics(),
        right.diagnostics(),
        "{label}: diagnostics"
    );
    assert_eq!(
        left.selected_model_packages().collect::<Vec<_>>(),
        right.selected_model_packages().collect::<Vec<_>>(),
        "{label}: selected models"
    );
    let model_limits = CheckedModelPopulationCatalogLimits {
        work: u64::MAX,
        entries: u64::MAX,
    };
    assert_eq!(
        left.selected_model_population_catalog(model_limits),
        right.selected_model_population_catalog(model_limits),
        "{label}: retained model populations"
    );
    let profile = CompleteLoweringProfileV2 {
        reference: LoweringProfileRef {
            authority: "quire".into(),
            identity: "complete-v1".into(),
        },
        required_source_selections: Vec::new(),
        supported_tags: CheckedNodeTag::ALL.iter().copied().collect(),
        require_bounds: false,
        work_limit: u64::MAX,
    };
    assert!(
        !left.resolved_graph().nodes.is_empty(),
        "{label}: genuine graph"
    );
    for node in &left.resolved_graph().nodes {
        let id = &node.node_id;
        assert_eq!(
            left.node_kind(id),
            right.node_kind(id),
            "{label}: kind {id:?}"
        );
        assert_eq!(left.term(id), right.term(id), "{label}: term {id:?}");
        assert_eq!(
            left.model_record_fields(id),
            right.model_record_fields(id),
            "{label}: model fields {id:?}"
        );
        assert_eq!(
            left.model_object_fields(id),
            right.model_object_fields(id),
            "{label}: object fields {id:?}"
        );
        assert_eq!(
            left.model_operations(id, u64::MAX),
            right.model_operations(id, u64::MAX),
            "{label}: retained model operations {id:?}"
        );
        let request = std::slice::from_ref(id);
        let expected = left.lower(request, &profile);
        let actual = right.lower(request, &profile);
        assert_eq!(
            expected.records.len(),
            1,
            "{label}: one lowering record per node"
        );
        assert_eq!(actual, expected, "{label}: lowering {id:?}");
        assert_eq!(
            actual.package.canonical_bytes(),
            expected.package.canonical_bytes(),
            "{label}: lowered bytes {id:?}"
        );
    }
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
