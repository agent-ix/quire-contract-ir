//! Bounded read of producer-captured model dispatch candidates (FR-354 AC-4).

use super::checked_term_data::{
    CheckedDispatchCandidate, CheckedDispatchCandidateTable, CheckedTermDataV2,
};
use super::model_fields::RetainedModels;
use super::{
    ApplicationOperator, CheckedNodeKind, CheckedPackageV2, CheckedTerm,
    ExpressionForm, RawCheckedPackageWireV2, WorkMeter,
};
use crate::checked_package::common::ValidationFailure;
use crate::checked_package::shared::{
    CheckedNodeId, CheckedOccurrence, CheckedPackageRefusalCause, CheckedPackageRefusalCode,
    JsonPointer,
};
use std::collections::{BTreeMap, BTreeSet};

fn invalid(path: JsonPointer) -> ValidationFailure {
    ValidationFailure::refused_because(
        CheckedPackageRefusalCode::InvalidPackage,
        path,
        CheckedPackageRefusalCause::InvalidValue,
    )
}

fn unsupported(path: JsonPointer) -> ValidationFailure {
    ValidationFailure::refused(CheckedPackageRefusalCode::UnsupportedConstruct, path)
}

/// Checks the producer-captured sidecar after graph and source-map admission.
/// Equality and protocol rows await their separate semantic owner slices and
/// therefore refuse before the package can expose any partial sidecar.
pub(super) fn validate_checked_term_data(
    data: &CheckedTermDataV2,
    wire: &RawCheckedPackageWireV2,
    kinds: &[CheckedNodeKind],
    terms: &[CheckedTerm],
    models: &RetainedModels,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let root = JsonPointer::root().key("checked_term_data");
    if !data.equality_decisions.is_empty() {
        let path = root.clone().key("equality_decisions").index(0);
        meter.charge(1, || path.clone())?;
        return Err(unsupported(path));
    }
    if !data.protocol_locations.is_empty() {
        let path = root.clone().key("protocol_locations").index(0);
        meter.charge(1, || path.clone())?;
        return Err(unsupported(path));
    }

    let nodes = &wire.semantic_graph.nodes;
    let index: BTreeMap<CheckedNodeId, usize> = nodes
        .iter()
        .enumerate()
        .map(|(position, node)| (node.node_id.clone(), position))
        .collect();
    let source_occurrences: BTreeSet<_> = wire
        .source_map
        .iter()
        .map(|entry| (&entry.node_id, &entry.role, entry.ordinal))
        .collect();
    let mut previous: Option<(&CheckedNodeId, &CheckedOccurrence)> = None;
    for (row_index, table) in data.dispatch_tables.iter().enumerate() {
        let row = root.clone().key("dispatch_tables").index(row_index);
        meter.charge(1, || row.clone())?;
        let key = (&table.application, &table.occurrence);
        if let Some(prior) = previous {
            if prior == key {
                return Err(ValidationFailure::refused_because(
                    CheckedPackageRefusalCode::InvalidPackage,
                    row,
                    CheckedPackageRefusalCause::ConflictingDefinition,
                ));
            }
            if prior > key {
                return Err(invalid(row));
            }
        }
        previous = Some(key);
        let Some(&application_index) = index.get(&table.application) else {
            return Err(invalid(row.clone().key("application")));
        };
        let Some(application_node) = nodes.get(application_index) else {
            return Err(invalid(row.clone().key("application")));
        };
        if table.application.package.as_ref() != wire.name.as_ref()
            || kinds.get(application_index)
                != Some(&CheckedNodeKind::Expression(ExpressionForm::Call))
            || !application_node.occurrences.contains(&table.occurrence)
            || !source_occurrences.contains(&(
                &table.application,
                &table.occurrence.role,
                table.occurrence.ordinal,
            ))
        {
            return Err(invalid(row.clone().key("application")));
        }
        let Some(CheckedTerm::Application {
            operator: ApplicationOperator::Call,
            operation,
            ..
        }) = terms.get(application_index)
        else {
            return Err(invalid(row.clone().key("application")));
        };
        let member = operation.record().get("member");
        let member_name = member
            .and_then(|member| member.get("name"))
            .and_then(|name| name.as_str());
        if member
            .and_then(|member| member.get("kind"))
            .and_then(|kind| kind.as_str())
            != Some("operation")
            || operation.member_declaration().ok().flatten() != Some(&table.operation.declaring)
            || member_name != Some(table.operation.operation.as_ref())
        {
            return Err(invalid(row.clone().key("operation")));
        }
        if table.context != table.operation.declaring || !index.contains_key(&table.context) {
            return Err(invalid(row.clone().key("context")));
        }
        let mut seen_keys = BTreeSet::new();
        for (candidate_index, candidate) in table.candidates.iter().enumerate() {
            let at = row.clone().key("candidates").index(candidate_index);
            meter.charge(1, || at.clone())?;
            if !seen_keys.insert(&candidate.selection_key) {
                return Err(ValidationFailure::refused_because(
                    CheckedPackageRefusalCode::InvalidPackage,
                    at.clone().key("selection_key"),
                    CheckedPackageRefusalCause::ConflictingDefinition,
                ));
            }
            if candidate.selection_key.owner_effective_type.is_some() {
                return Err(unsupported(at.clone().key("selection_key")));
            }
            let original = &candidate.selection_key.original;
            let selected = models.selected_models().iter().any(|model| {
                model.identity == original.package
                    && model.object_types.contains_key(original.node.as_ref())
            });
            if !selected {
                return Err(invalid(at.clone().key("selection_key")));
            }
            for (member, handle) in [
                ("operation_body", Some(&candidate.operation_body)),
                (
                    "effective_precondition",
                    candidate.effective_precondition.as_ref(),
                ),
            ] {
                if handle.is_some_and(|handle| {
                    handle.package.as_ref() != wire.name.as_ref() || !index.contains_key(handle)
                }) {
                    return Err(invalid(at.clone().key(member)));
                }
            }
        }
    }
    Ok(())
}

/// A complete admitted dispatch table, borrowing candidates in producer order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CheckedDispatchView<'a> {
    /// Exact application and occurrence selected by the producer.
    pub table: &'a CheckedDispatchCandidateTable,
}

impl CheckedDispatchView<'_> {
    /// Candidate rows in the producer's order, including an authoritative empty set.
    pub fn candidates(&self) -> &[CheckedDispatchCandidate] {
        &self.table.candidates
    }
}

/// Why a bounded dispatch read returned no complete table.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum CheckedDispatchReadError {
    /// The producer retained no checked-term sidecar.
    #[error("checked-term dispatch data is unavailable")]
    Unavailable,
    /// No admitted table names the requested application and occurrence.
    #[error("no dispatch table names the requested occurrence")]
    Missing,
    /// The first uncharged row or candidate exceeded the caller's work limit.
    #[error("dispatch read exhausted its work limit")]
    WorkExhausted {
        /// Caller-supplied work limit.
        limit: u64,
        /// Work attempted, including the rejected charge.
        consumed: u64,
        /// Sidecar row at which the charge failed.
        row: usize,
        /// Candidate position when the rejected charge was for a candidate.
        candidate: Option<usize>,
    },
}

impl CheckedPackageV2 {
    /// Returns one complete producer-captured dispatch table under a finite work limit.
    ///
    /// The read does not choose a candidate, evaluate a precondition, or derive
    /// a table from the graph. `Missing` differs from a captured empty set.
    pub fn dispatch_candidates(
        &self,
        application: &CheckedNodeId,
        occurrence: &CheckedOccurrence,
        work_limit: u64,
    ) -> Result<CheckedDispatchView<'_>, CheckedDispatchReadError> {
        let Some(data) = self.wire.checked_term_data.as_ref() else {
            return Err(CheckedDispatchReadError::Unavailable);
        };
        let mut consumed = 0_u64;
        for (row_index, table) in data.dispatch_tables.iter().enumerate() {
            consumed = consumed.saturating_add(1);
            if consumed > work_limit {
                return Err(CheckedDispatchReadError::WorkExhausted {
                    limit: work_limit,
                    consumed,
                    row: row_index,
                    candidate: None,
                });
            }
            if &table.application != application || &table.occurrence != occurrence {
                continue;
            }
            for candidate_index in 0..table.candidates.len() {
                consumed = consumed.saturating_add(1);
                if consumed > work_limit {
                    return Err(CheckedDispatchReadError::WorkExhausted {
                        limit: work_limit,
                        consumed,
                        row: row_index,
                        candidate: Some(candidate_index),
                    });
                }
            }
            return Ok(CheckedDispatchView { table });
        }
        Err(CheckedDispatchReadError::Missing)
    }
}
