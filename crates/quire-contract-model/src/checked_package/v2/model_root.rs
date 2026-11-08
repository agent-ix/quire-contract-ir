// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Selected model root shape for FR-065.

use super::native_join_budget::PackageReadStage;
use super::stage_storage::{self, Path, Refusal, Site};
use super::{CheckedNodeKind, CheckedSemanticNodeV2, CheckedTerm, ModelForm, WorkMeter};
use crate::checked_package::common::ValidationFailure;
use crate::checked_package::shared::{CheckedOccurrenceRole, CheckedPackageRefusalCode};

fn refuse(
    meter: &mut WorkMeter,
    position: usize,
    code: CheckedPackageRefusalCode,
    at: Path<'_>,
) -> ValidationFailure {
    stage_storage::refuse(
        meter,
        &Site::new(PackageReadStage::Graph, position, "model", at),
        Refusal::new(code, at),
    )
}

/// Checks the root's closed graph and occurrence shape before key derivation.
pub(super) fn validate_shape<Body>(
    nodes: &[CheckedSemanticNodeV2<Body>],
    kinds: &[CheckedNodeKind],
    terms: &[Option<CheckedTerm>],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    use CheckedPackageRefusalCode::{InvalidSemanticGraph, InvalidSourceMap};
    for (position, node) in nodes.iter().enumerate() {
        if kinds.get(position) != Some(&CheckedNodeKind::Model(ModelForm::ModelRoot)) {
            continue;
        }
        let at = Path::root()
            .key("semantic_graph")
            .key("nodes")
            .index(position);
        if node.semantic_type != node.node_id {
            return Err(refuse(
                meter,
                position,
                InvalidSemanticGraph,
                at.key("semantic_type"),
            ));
        }
        if node.recursion_group.is_some() {
            return Err(refuse(
                meter,
                position,
                InvalidSemanticGraph,
                at.key("recursion_group"),
            ));
        }
        if !node.dependencies.is_empty() {
            return Err(refuse(
                meter,
                position,
                InvalidSemanticGraph,
                at.key("dependencies"),
            ));
        }
        if !matches!(terms.get(position).and_then(Option::as_ref),
            Some(CheckedTerm::Aggregate { members }) if members.is_empty())
        {
            return Err(refuse(
                meter,
                position,
                InvalidSemanticGraph,
                at.key("body"),
            ));
        }
        if node.occurrences.len() != 1 {
            return Err(refuse(
                meter,
                position,
                InvalidSourceMap,
                at.key("occurrences"),
            ));
        }
        let occurrence = &node.occurrences[0];
        if occurrence.role != CheckedOccurrenceRole::Generated {
            return Err(refuse(
                meter,
                position,
                InvalidSourceMap,
                at.key("occurrences").index(0).key("role"),
            ));
        }
        if occurrence.ordinal != 0 {
            return Err(refuse(
                meter,
                position,
                InvalidSourceMap,
                at.key("occurrences").index(0).key("ordinal"),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::native_join_budget::{NativeJoinBudget, NativeJoinLocusSource};
    use super::super::{test_support::node_id, CheckedJson};
    use super::*;
    use ix_trace_rs::trace;
    use serde_json::json;

    fn root() -> CheckedSemanticNodeV2<CheckedJson> {
        let id = node_id('a');
        serde_json::from_value(json!({
            "node_id":id, "schema_version":"quire.checked-semantic-graph/v2",
            "node_tag":"model", "semantic_form":"model_root", "semantic_type":id,
            "dependencies":[], "occurrences":[{"role":"generated", "ordinal":0}],
            "body":{"term":"aggregate", "members":[]}
        }))
        .expect("retained document metadata")
    }

    /// A healthy retained document node uses metadata and its typed term,
    /// without constructing a diagnostic pointer or an owned body.
    #[trace("TC-472", "FR-038-AC-511")]
    #[test]
    fn healthy_document_model_root_validation_allocates_nothing() {
        let node = root();
        let terms = [Some(CheckedTerm::Aggregate {
            members: Vec::new(),
        })];
        let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(0);
        let mut result = None;
        let observed = allocation_counter::measure(|| {
            result = Some(validate_shape(
                std::slice::from_ref(&node),
                &[CheckedNodeKind::Model(ModelForm::ModelRoot)],
                &terms,
                &mut WorkMeter::over(&mut budget),
            ));
        });
        assert!(result.expect("validation ran").is_ok());
        assert_eq!(observed.bytes_total, 0);
        assert_eq!(budget.read_storage_consumed(), 0);
    }

    /// The first shape fault retains its original code and path; its exact
    /// diagnostic is payable before construction through the caller meter.
    #[trace("TC-472", "FR-038-AC-510")]
    #[test]
    fn document_model_root_fault_keeps_precedence_and_prepays_its_diagnostic() {
        let mut node = root();
        node.semantic_type = node_id('b');
        node.recursion_group = Some("later-fault".into());
        let terms = [Some(CheckedTerm::Aggregate {
            members: Vec::new(),
        })];
        let run = |limit| {
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut result = None;
            let observed = allocation_counter::measure(|| {
                result = Some(validate_shape(
                    std::slice::from_ref(&node),
                    &[CheckedNodeKind::Model(ModelForm::ModelRoot)],
                    &terms,
                    &mut WorkMeter::over(&mut budget),
                ));
            });
            (
                result.expect("validation ran"),
                observed.bytes_total,
                budget,
            )
        };
        let check = |result| {
            let Err(ValidationFailure::Refused(refusal)) = result else {
                panic!("the first shape fault must refuse: {result:?}");
            };
            assert_eq!(
                refusal.code,
                CheckedPackageRefusalCode::InvalidSemanticGraph
            );
            assert_eq!(refusal.cause, None);
            assert_eq!(refusal.locus, None);
            assert_eq!(
                refusal.loci[0].path.as_ref().expect("path").as_str(),
                "/semantic_graph/nodes/0/semantic_type"
            );
        };
        let (result, allocated, budget) = run(u64::MAX);
        check(result);
        let required = budget.read_storage_consumed();
        assert!(required > 0);
        assert_eq!(allocated, required);
        check(run(required).0);
        let (short, _, budget) = run(required - 1);
        assert!(matches!(short, Err(ValidationFailure::Budget(_))));
        let locus = budget.first_unpaid_locus().expect("denial locus");
        assert_eq!(
            locus.source,
            NativeJoinLocusSource::PackageRead(PackageReadStage::Graph, 0)
        );
        assert_eq!(locus.form, "diagnostic");
        let (zero, allocated, _) = run(0);
        assert!(matches!(zero, Err(ValidationFailure::Budget(_))));
        assert_eq!(allocated, 0, "denied diagnostic allocates nothing");
    }
}
