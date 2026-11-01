//! Bounded read of producer-captured model dispatch candidates (FR-354 AC-4).

use super::checked_term_data::{
    CheckedDispatchCandidate, CheckedDispatchCandidateTable, CheckedTermDataV2,
};
use super::model_fields::RetainedModels;
use super::model_members::Budget;
use super::{
    ApplicationOperator, CheckedNodeKind, CheckedPackageV2, CheckedTerm, ExpressionForm,
    FunctionForm, ModelForm, RawCheckedPackageWireV2, WorkMeter,
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
/// Equality rows await their separate semantic owner slice and therefore
/// refuse before the package can expose any partial sidecar.
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
        if !data.equality_decisions[0].term_path.is_empty() {
            let segment = path.clone().key("term_path").index(0);
            meter.charge(1, || segment.clone())?;
            return Err(invalid(segment));
        }
        return Err(unsupported(path));
    }
    super::protocol_locations::validate(wire, kinds, terms, meter)?;

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
    let mut seen_tables = BTreeSet::new();
    for (row_index, table) in data.dispatch_tables.iter().enumerate() {
        let row = root.clone().key("dispatch_tables").index(row_index);
        meter.charge(1, || row.clone())?;
        if !seen_tables.insert((&table.application, &table.occurrence)) {
            return Err(ValidationFailure::refused_because(
                CheckedPackageRefusalCode::InvalidPackage,
                row,
                CheckedPackageRefusalCause::ConflictingDefinition,
            ));
        }
        let Some(&application_index) = index.get(&table.application) else {
            return Err(invalid(row.clone().key("application")));
        };
        let Some(application_node) = nodes.get(application_index) else {
            return Err(invalid(row.clone().key("application")));
        };
        if table.application.package.as_ref() != wire.name.as_ref()
            || kinds.get(application_index)
                != Some(&CheckedNodeKind::Expression(ExpressionForm::Call))
        {
            return Err(invalid(row.clone().key("application")));
        }
        if !application_node.occurrences.contains(&table.occurrence)
            || !source_occurrences.contains(&(
                &table.application,
                &table.occurrence.role,
                table.occurrence.ordinal,
            ))
        {
            return Err(invalid(row.clone().key("occurrence")));
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
        let context_position = index.get(&table.context).copied();
        let context_is_model = context_position.is_some_and(|position| {
            matches!(
                kinds.get(position),
                Some(CheckedNodeKind::Model(
                    ModelForm::ObjectType
                        | ModelForm::Process
                        | ModelForm::StateMachine
                        | ModelForm::PersistenceInterface
                ))
            )
        });
        // Operation admission already requires argument zero to be Reference
        // of the selected member's declaring model node.
        if table.context != table.operation.declaring || !context_is_model {
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
            let Some(context_owner) = models.owner_by_key(&table.context) else {
                return Err(invalid(row.clone().key("context")));
            };
            let model = context_owner.package;
            if model.identity.as_ref() != original.package.as_ref()
                || !model.object_types.contains_key(original.node.as_ref())
            {
                return Err(invalid(at.clone().key("selection_key")));
            }
            let mut budget = Budget::new(meter, model.root, model.bytes);
            if !model.conforms_to(&original.node, context_owner.node, &mut budget)?
                || !model
                    .effective_operation_names(&original.node, &mut budget)?
                    .contains(&table.operation.operation)
            {
                return Err(invalid(at.clone().key("selection_key")));
            }
            let body_kind = index
                .get(&candidate.operation_body)
                .and_then(|position| kinds.get(*position));
            if candidate.operation_body.package.as_ref() != wire.name.as_ref()
                || !matches!(body_kind, Some(CheckedNodeKind::Function(_)))
            {
                return Err(invalid(at.clone().key("operation_body")));
            }
            if let Some(precondition) = candidate.effective_precondition.as_ref() {
                let precondition_kind = index
                    .get(precondition)
                    .and_then(|position| kinds.get(*position));
                if precondition.package.as_ref() != wire.name.as_ref()
                    || precondition_kind
                        != Some(&CheckedNodeKind::Function(FunctionForm::Predicate))
                {
                    return Err(invalid(at.clone().key("effective_precondition")));
                }
            }
        }
        if !table.candidates.is_empty() {
            // The wire has no producer witness binding each candidate's
            // effective type and selected operation to its function bodies.
            return Err(unsupported(row.key("candidates").index(0)));
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
    /// The requested application belongs to another checked package.
    #[error("dispatch application belongs to another checked package")]
    ForeignApplication,
    /// The requested application does not name an admitted graph node.
    #[error("dispatch application is not admitted")]
    UnknownApplication,
    /// The requested admitted node is not a call application.
    #[error("dispatch application is not a call")]
    WrongApplication,
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
        if application.package != self.wire.name {
            return Err(CheckedDispatchReadError::ForeignApplication);
        }
        let Some(&position) = self.node_index.get(application) else {
            return Err(CheckedDispatchReadError::UnknownApplication);
        };
        if self.kinds.get(position) != Some(&CheckedNodeKind::Expression(ExpressionForm::Call)) {
            return Err(CheckedDispatchReadError::WrongApplication);
        }
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
