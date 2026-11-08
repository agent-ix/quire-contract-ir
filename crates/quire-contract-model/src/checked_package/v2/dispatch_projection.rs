//! Bounded read of producer-captured model dispatch candidates (FR-354 AC-4).

use super::checked_term_data::{
    CheckedDispatchCandidate, CheckedDispatchCandidateTable, CheckedTermDataV2,
};
use super::model_fields::RetainedModels;
use super::model_members::Budget;
use super::native_join_budget::PackageReadStage;
use super::stage_storage::{self, Path, Refusal, Site};
use super::{
    ApplicationOperator, CheckedNodeKind, CheckedPackageV2, CheckedTerm, ClaimForm, ExpressionForm,
    FunctionForm, ModelForm, RawCheckedPackageWireV2, ScalarTypeForm, WorkMeter,
};
use crate::checked_package::common::ValidationFailure;
use crate::checked_package::shared::{
    CheckedNodeId, CheckedOccurrence, CheckedOccurrenceRole, CheckedPackageRefusalCause,
    CheckedPackageRefusalCode,
};

pub(super) fn data_site(row: usize, at: Path<'static>) -> Site<'static> {
    Site::new(
        PackageReadStage::CheckedTermData,
        row,
        "checked_term_data",
        at,
    )
}

/// A refusal about the value at `at` in sidecar row `row`, its diagnostic
/// charged before it is built.
pub(super) fn fail(
    meter: &mut WorkMeter,
    row: usize,
    code: CheckedPackageRefusalCode,
    cause: Option<CheckedPackageRefusalCause>,
    at: Path<'static>,
) -> ValidationFailure {
    let refusal = Refusal::new(code, at);
    let refusal = match cause {
        Some(cause) => refusal.because(cause),
        None => refusal,
    };
    stage_storage::refuse(meter, &data_site(row, at), refusal)
}

pub(super) fn invalid(meter: &mut WorkMeter, row: usize, at: Path<'static>) -> ValidationFailure {
    fail(
        meter,
        row,
        CheckedPackageRefusalCode::InvalidPackage,
        Some(CheckedPackageRefusalCause::InvalidValue),
        at,
    )
}

pub(super) fn conflicting(
    meter: &mut WorkMeter,
    row: usize,
    at: Path<'static>,
) -> ValidationFailure {
    fail(
        meter,
        row,
        CheckedPackageRefusalCode::InvalidPackage,
        Some(CheckedPackageRefusalCause::ConflictingDefinition),
        at,
    )
}

/// The graph's node ids with their positions, ascending by identity.
fn node_positions<'w>(
    wire: &'w RawCheckedPackageWireV2,
    meter: &mut WorkMeter,
) -> Result<Vec<(&'w CheckedNodeId, usize)>, ValidationFailure> {
    let nodes = &wire.semantic_graph.nodes;
    let mut index = stage_storage::vec(
        meter,
        &data_site(0, Path::root().key("checked_term_data")),
        nodes.len(),
    )?;
    index.extend(
        nodes
            .iter()
            .enumerate()
            .map(|(position, node)| (&node.node_id, position)),
    );
    index.sort_unstable();
    Ok(index)
}

/// The position of `id` among `index`.
pub(super) fn position_of(index: &[(&CheckedNodeId, usize)], id: &CheckedNodeId) -> Option<usize> {
    index
        .binary_search_by(|(held, _)| (*held).cmp(id))
        .ok()
        .map(|found| index[found].1)
}

pub(super) type Occurrence<'a> = (&'a CheckedNodeId, &'a CheckedOccurrenceRole, u64);

/// The source map's occurrences, ascending.
fn source_occurrences<'w>(
    wire: &'w RawCheckedPackageWireV2,
    meter: &mut WorkMeter,
) -> Result<Vec<Occurrence<'w>>, ValidationFailure> {
    let mut held = stage_storage::vec(
        meter,
        &data_site(0, Path::root().key("checked_term_data")),
        wire.source_map.len(),
    )?;
    held.extend(
        wire.source_map
            .iter()
            .map(|entry| (&entry.node_id, &entry.role, entry.ordinal)),
    );
    held.sort_unstable();
    Ok(held)
}

/// Checks the producer-captured sidecar after graph and source-map admission.
pub(super) fn validate_checked_term_data(
    data: &CheckedTermDataV2,
    wire: &RawCheckedPackageWireV2,
    kinds: &[CheckedNodeKind],
    terms: &[CheckedTerm],
    models: &RetainedModels,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let root = Path::root().key("checked_term_data");
    super::equality_projection::validate(data, wire, kinds, terms, meter)?;
    super::protocol_locations::validate(wire, kinds, terms, meter)?;

    let nodes = &wire.semantic_graph.nodes;
    let index = node_positions(wire, meter)?;
    let source_occurrences = source_occurrences(wire, meter)?;
    let tables = root.key("dispatch_tables");
    let repeated_table = stage_storage::first_repeat(
        meter,
        &data_site(0, tables),
        data.dispatch_tables
            .iter()
            .map(|table| (&table.application, &table.occurrence)),
    )?;
    for (row_index, table) in data.dispatch_tables.iter().enumerate() {
        let row = tables.index(row_index);
        stage_storage::work(meter, &data_site(row_index, row), 1)?;
        if repeated_table == Some(row_index) {
            return Err(conflicting(meter, row_index, row));
        }
        let Some(application_index) = position_of(&index, &table.application) else {
            return Err(invalid(meter, row_index, row.key("application")));
        };
        let Some(application_node) = nodes.get(application_index) else {
            return Err(invalid(meter, row_index, row.key("application")));
        };
        if table.application.package.as_ref() != wire.name.as_ref()
            || kinds.get(application_index)
                != Some(&CheckedNodeKind::Expression(ExpressionForm::Call))
        {
            return Err(invalid(meter, row_index, row.key("application")));
        }
        if !application_node.occurrences.contains(&table.occurrence)
            || source_occurrences
                .binary_search(&(
                    &table.application,
                    &table.occurrence.role,
                    table.occurrence.ordinal,
                ))
                .is_err()
        {
            return Err(invalid(meter, row_index, row.key("occurrence")));
        }
        let Some(CheckedTerm::Application {
            operator: ApplicationOperator::Call,
            operation,
            ..
        }) = terms.get(application_index)
        else {
            return Err(invalid(meter, row_index, row.key("application")));
        };
        let member = operation.member_node();
        let member_name = member
            .and_then(|member| member.get("name"))
            .and_then(|name| name.as_str());
        if member
            .and_then(|member| member.get("kind"))
            .and_then(|kind| kind.as_str())
            != Some("operation")
            || member_name != Some(table.operation.operation.as_ref())
        {
            return Err(invalid(meter, row_index, row.key("operation")));
        }
        let context_position = position_of(&index, &table.context);
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
        // The member and receiver context name the call's static type,
        // while the selected operation may belong to an ancestor's family.
        if operation.member_declaration().ok().flatten() != Some(&table.context)
            || !context_is_model
        {
            return Err(invalid(meter, row_index, row.key("context")));
        }
        let Some(context_owner) = models.owner_by_key(&table.context) else {
            return Err(invalid(meter, row_index, row.key("context")));
        };
        let Some(family_owner) = models.owner_by_key(&table.operation.declaring) else {
            return Err(invalid(meter, row_index, row.key("operation")));
        };
        if context_owner.selection != family_owner.selection {
            return Err(invalid(meter, row_index, row.key("operation")));
        }
        let model = context_owner.package;
        let mut budget = Budget::new(meter, model.root, model.bytes);
        if !model.operation_in_family(
            context_owner.node,
            family_owner.node,
            &table.operation.operation,
            &mut budget,
        )? {
            return Err(invalid(meter, row_index, row.key("operation")));
        }
        let candidates = row.key("candidates");
        let repeated_key = stage_storage::first_repeat(
            meter,
            &data_site(row_index, candidates),
            table
                .candidates
                .iter()
                .map(|candidate| &candidate.selection_key),
        )?;
        for (candidate_index, candidate) in table.candidates.iter().enumerate() {
            let at = candidates.index(candidate_index);
            stage_storage::work(meter, &data_site(row_index, at), 1)?;
            if candidate.selected_operation != table.operation {
                return Err(invalid(meter, row_index, at.key("selected_operation")));
            }
            if repeated_key == Some(candidate_index) {
                return Err(conflicting(meter, row_index, at.key("selection_key")));
            }
            if candidate.selection_key.owner_effective_type.is_some() {
                return Err(fail(
                    meter,
                    row_index,
                    CheckedPackageRefusalCode::UnsupportedConstruct,
                    None,
                    at.key("selection_key"),
                ));
            }
            let original = &candidate.selection_key.original;
            if model.identity.as_ref() != original.package.as_ref()
                || !model.object_types.contains_key(original.node.as_ref())
            {
                return Err(invalid(meter, row_index, at.key("selection_key")));
            }
            let mut budget = Budget::new(meter, model.root, model.bytes);
            if !model.operation_in_family(
                &original.node,
                family_owner.node,
                &table.operation.operation,
                &mut budget,
            )? {
                return Err(invalid(meter, row_index, at.key("selection_key")));
            }
            let body_kind = position_of(&index, &candidate.operation_body)
                .and_then(|position| kinds.get(position));
            // Model declarations can belong to a dependency package, so the
            // exact ID's package need not be the package being emitted. The
            // graph index and function kind prove that this handle is a
            // retained function node.
            if !matches!(body_kind, Some(CheckedNodeKind::Function(_))) {
                return Err(invalid(meter, row_index, at.key("operation_body")));
            }
            if let Some(precondition) = candidate.effective_precondition.as_ref() {
                let precondition_position = position_of(&index, precondition);
                let precondition_kind =
                    precondition_position.and_then(|position| kinds.get(position));
                let has_boolean_result = precondition_position
                    .and_then(|position| nodes.get(position))
                    .and_then(|node| position_of(&index, &node.semantic_type))
                    .and_then(|position| kinds.get(position))
                    == Some(&CheckedNodeKind::ScalarType(ScalarTypeForm::Boolean));
                let valid_precondition = match precondition_kind {
                    Some(CheckedNodeKind::Function(FunctionForm::Predicate)) => true,
                    Some(CheckedNodeKind::Function(FunctionForm::PureFunction)) => {
                        has_boolean_result
                    }
                    _ => false,
                };
                if !valid_precondition {
                    return Err(invalid(meter, row_index, at.key("effective_precondition")));
                }
            }
        }
    }
    validate_probabilistic_claims(data, wire, kinds, &index, &source_occurrences, meter)?;
    Ok(())
}

fn validate_probabilistic_claims(
    data: &CheckedTermDataV2,
    wire: &RawCheckedPackageWireV2,
    kinds: &[CheckedNodeKind],
    index: &[(&CheckedNodeId, usize)],
    source_occurrences: &[Occurrence<'_>],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let root = Path::root()
        .key("checked_term_data")
        .key("probabilistic_claims");
    let nodes = &wire.semantic_graph.nodes;
    let claims = &data.probabilistic_claims;
    // The declared claims, ascending, to find a repeat and to find a claim node
    // no row declares.
    let mut declared: Vec<(&CheckedNodeId, usize)> =
        stage_storage::vec(meter, &data_site(0, root), claims.len())?;
    declared.extend(
        claims
            .iter()
            .enumerate()
            .map(|(position, claim)| (&claim.declaration, position)),
    );
    declared.sort_unstable();
    let repeated = declared
        .windows(2)
        .filter(|pair| pair[0].0 == pair[1].0)
        .map(|pair| pair[1].1)
        .min();
    let mut previous = None;
    for (row_index, claim) in claims.iter().enumerate() {
        let row = root.index(row_index);
        stage_storage::work(meter, &data_site(row_index, row), 1)?;
        if repeated == Some(row_index) {
            return Err(conflicting(meter, row_index, row));
        }
        if previous.is_some_and(|prior| prior > &claim.declaration) {
            return Err(fail(
                meter,
                row_index,
                CheckedPackageRefusalCode::InvalidPackage,
                Some(CheckedPackageRefusalCause::InvalidSemanticGraph),
                row,
            ));
        }
        previous = Some(&claim.declaration);
        let Some(position) = position_of(index, &claim.declaration) else {
            return Err(invalid(meter, row_index, row.key("declaration")));
        };
        let node = &nodes[position];
        if claim.declaration.package != wire.name
            || kinds.get(position) != Some(&CheckedNodeKind::Claim(ClaimForm::ProbabilisticClaim))
        {
            return Err(invalid(meter, row_index, row.key("declaration")));
        }
        if claim.occurrence.role != CheckedOccurrenceRole::Claim
            || !node.occurrences.contains(&claim.occurrence)
            || source_occurrences
                .binary_search(&(
                    &claim.declaration,
                    &claim.occurrence.role,
                    claim.occurrence.ordinal,
                ))
                .is_err()
        {
            return Err(invalid(meter, row_index, row.key("occurrence")));
        }
    }
    for node in nodes.iter().zip(kinds).filter_map(|(node, kind)| {
        (*kind == CheckedNodeKind::Claim(ClaimForm::ProbabilisticClaim)).then_some(node)
    }) {
        if declared
            .binary_search_by(|(held, _)| (*held).cmp(&node.node_id))
            .is_err()
        {
            return Err(invalid(meter, 0, root));
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
