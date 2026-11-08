// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Public retained and admission-derived consumers shared by read-budget proofs.
//! Every assertion runs after the measured reader call has returned.

use quire_contract_model::{
    CheckedModelPopulationCatalogLimits, CheckedNodeTag, CheckedPackageV2,
    CheckedPackageV2BudgetedReadResult, CompleteLoweringProfileV2, LoweringProfileRef,
};

/// Content equality excludes admission-derived model tables and lowering facts.
/// Exercise their public consumers as well as the retained document content.
pub(crate) fn assert_admitted_views_equal(
    left: &CheckedPackageV2,
    right: &CheckedPackageV2,
    label: &str,
) {
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

pub(crate) fn assert_budgeted_results_equal(
    left: &CheckedPackageV2BudgetedReadResult,
    right: &CheckedPackageV2BudgetedReadResult,
    label: &str,
) {
    assert_eq!(left, right, "{label}: complete typed outcome");
    if let (
        CheckedPackageV2BudgetedReadResult::Admitted(left),
        CheckedPackageV2BudgetedReadResult::Admitted(right),
    ) = (left, right)
    {
        assert_admitted_views_equal(left, right, label);
    }
}
