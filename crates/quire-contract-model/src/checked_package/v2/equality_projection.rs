// SPDX-License-Identifier: AGPL-3.0-or-later
//! Structural admission and bounded reads of producer-selected equality decisions.

use super::checked_term_data::{CheckedEqualityOperator, CheckedTermDataV2};
use super::exact_projection::SelectedSourceUnitRow;
use super::extension::integer_range_bounds;
use super::operations::is_type_shaped;
use super::{
    ApplicationOperator, BoundedDomainForm, CheckedNodeKind, CheckedNodeOwner, CheckedPackageV2,
    CheckedResolvedEqualityUnitForm, CheckedTerm, ExpressionForm, RawCheckedPackageWireV2,
    ScalarTypeForm, WorkMeter,
};
use crate::checked_package::common::ValidationFailure;
use crate::checked_package::shared::{
    CheckedNodeId, CheckedOccurrence, CheckedOccurrenceRole, CheckedPackageRefusalCause,
    CheckedPackageRefusalCode, CheckedSourceMapEntry, JsonPointer,
};
use quire_exact::{
    Integer, IntegerInterval, LimitKind, Location, Meter, NodeKey, ScalarLimits, TextProfile,
    TextType, UnitId, ValueType,
};
use quire_semantic_value::declaration::{
    AdmittedEqualityDecision, EqualityDecisionDescriptor, EqualityOperand, EqualityOperator,
    EqualitySchedule,
};
use quire_semantic_value::reader_admission::{
    mint_admitted_equality_decision, EqualityMaterializationError, ReaderSeal,
};
use std::collections::{BTreeMap, BTreeSet};

fn argument_type(
    argument: &CheckedTerm,
    wire: &RawCheckedPackageWireV2,
    kinds: &[CheckedNodeKind],
    index: &BTreeMap<&CheckedNodeId, usize>,
) -> Option<CheckedNodeId> {
    match argument {
        CheckedTerm::Literal { type_id, .. } => Some(type_id.clone()),
        CheckedTerm::Application { result_type, .. } => Some(result_type.clone()),
        CheckedTerm::Reference { target } => {
            let position = *index.get(target)?;
            if is_type_shaped(*kinds.get(position)?) {
                Some(target.clone())
            } else {
                Some(
                    wire.semantic_graph
                        .nodes
                        .get(position)?
                        .semantic_type
                        .clone(),
                )
            }
        }
        _ => None,
    }
}

fn argument_types(
    argument: &CheckedTerm,
    wire: &RawCheckedPackageWireV2,
    kinds: &[CheckedNodeKind],
    terms: &[CheckedTerm],
    index: &BTreeMap<&CheckedNodeId, usize>,
) -> Option<(CheckedNodeId, Option<CheckedNodeId>)> {
    let compared = argument_type(argument, wire, kinds, index)?;
    let CheckedTerm::Reference { target } = argument else {
        return Some((compared, None));
    };
    let position = *index.get(target)?;
    let Some(CheckedTerm::Application {
        operation,
        arguments,
        ..
    }) = terms.get(position)
    else {
        return Some((compared, None));
    };
    if !matches!(
        operation.identity(),
        Some("quire.op.numeric.convert" | "quire.op.quantity.convert")
    ) {
        return Some((compared, None));
    }
    let [source] = arguments.as_slice() else {
        return None;
    };
    Some((argument_type(source, wire, kinds, index)?, Some(compared)))
}

fn invalid(path: JsonPointer) -> ValidationFailure {
    ValidationFailure::refused_because(
        CheckedPackageRefusalCode::InvalidPackage,
        path,
        CheckedPackageRefusalCause::InvalidValue,
    )
}

fn text_bounds(term: &CheckedTerm) -> Option<(&str, &str, &str)> {
    fn bound<'a>(term: &'a CheckedTerm, expected: &str) -> Option<&'a str> {
        let CheckedTerm::Binding { name, value } = term else {
            return None;
        };
        if name.as_ref() != expected {
            return None;
        }
        let CheckedTerm::Literal { value, .. } = value.as_ref() else {
            return None;
        };
        value.as_str()
    }
    let CheckedTerm::Aggregate { members } = term else {
        return None;
    };
    let [min, max, profile] = members.as_slice() else {
        return None;
    };
    Some((
        bound(min, "min")?,
        bound(max, "max")?,
        bound(profile, "text_profile")?,
    ))
}

fn equality_operator(identity: &str) -> Option<CheckedEqualityOperator> {
    // The operation record has already been checked against the closed catalog.
    // All catalogued equality identities end in one of these two segments.
    match identity.rsplit_once('.') {
        Some((_, "eq")) => Some(CheckedEqualityOperator::Equal),
        Some((_, "ne")) => Some(CheckedEqualityOperator::NotEqual),
        _ => None,
    }
}

/// Join each raw decision to the admitted application's own node and occurrence.
pub(super) fn validate(
    data: &CheckedTermDataV2,
    wire: &RawCheckedPackageWireV2,
    kinds: &[CheckedNodeKind],
    terms: &[CheckedTerm],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    if data.equality_decisions.is_empty() {
        return Ok(());
    }
    let nodes = &wire.semantic_graph.nodes;
    let mut index = BTreeMap::new();
    for (position, node) in nodes.iter().enumerate() {
        meter.charge(1, || {
            JsonPointer::root()
                .key("semantic_graph")
                .key("nodes")
                .index(position)
        })?;
        index.insert(&node.node_id, position);
    }
    let mut source_occurrences = BTreeSet::new();
    for (position, entry) in wire.source_map.iter().enumerate() {
        meter.charge(1, || JsonPointer::root().key("source_map").index(position))?;
        source_occurrences.insert((&entry.node_id, &entry.role, entry.ordinal));
    }
    let mut seen = BTreeSet::new();
    for (row_index, decision) in data.equality_decisions.iter().enumerate() {
        let row = JsonPointer::root()
            .key("checked_term_data")
            .key("equality_decisions")
            .index(row_index);
        meter.charge(1, || row.clone())?;
        if !seen.insert((&decision.application, &decision.occurrence)) {
            return Err(ValidationFailure::refused_because(
                CheckedPackageRefusalCode::InvalidPackage,
                row,
                CheckedPackageRefusalCause::ConflictingDefinition,
            ));
        }
        let Some(&position) = index.get(&decision.application) else {
            return Err(invalid(row.key("application")));
        };
        if decision.application.package != wire.name
            || kinds.get(position) != Some(&CheckedNodeKind::Expression(ExpressionForm::Binary))
        {
            return Err(invalid(row.key("application")));
        }
        let Some(node) = nodes.get(position) else {
            return Err(invalid(row.key("application")));
        };
        if !node.occurrences.contains(&decision.occurrence)
            || !source_occurrences.contains(&(
                &decision.application,
                &decision.occurrence.role,
                decision.occurrence.ordinal,
            ))
        {
            return Err(invalid(row.key("occurrence")));
        }
        if !decision.term_path.is_empty() {
            return Err(invalid(row.key("term_path").index(0)));
        }
        let Some(CheckedTerm::Application {
            operator: ApplicationOperator::Binary,
            operation,
            arguments,
            ..
        }) = terms.get(position)
        else {
            return Err(invalid(row.key("application")));
        };
        let Some(actual_operator) = operation.identity().and_then(equality_operator) else {
            return Err(invalid(row.key("application")));
        };
        if actual_operator != decision.operator {
            return Err(invalid(row.key("operator")));
        }
        if arguments.len() != 2 {
            return Err(invalid(row.key("application")));
        }
        for (ordinal, operand) in [&decision.left, &decision.right].into_iter().enumerate() {
            let at = row.clone().key(if ordinal == 0 { "left" } else { "right" });
            meter.charge(1, || at.clone())?;
            if usize::from(operand.operand) != ordinal {
                return Err(invalid(at.key("operand")));
            }
            let Some((source_type, conversion_target)) = arguments
                .get(ordinal)
                .and_then(|argument| argument_types(argument, wire, kinds, terms, &index))
            else {
                return Err(invalid(at.key("source_type")));
            };
            if source_type != operand.source_type {
                return Err(invalid(at.key("source_type")));
            }
            if conversion_target != operand.conversion_target {
                return Err(invalid(at.key("conversion_target")));
            }
        }
    }
    Ok(())
}

/// One checking-stage correspondence from an emitted equality occurrence to its original key.
///
/// The source-map entry identifies the selected source and its exact admitted
/// occurrence. A caller-created row is still only a claim until joined here.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedEqualityApplicationRow {
    /// Full emitted equality application node.
    pub application: CheckedNodeId,
    /// Exact emitted application occurrence.
    pub occurrence: CheckedOccurrence,
    /// Admitted selected-source correspondence supplied by the producer.
    pub source_map_entry: CheckedSourceMapEntry,
    /// Original producer-issued application key.
    pub node_key: NodeKey,
    /// Original authored Exact location.
    pub location: Location,
}

impl SelectedEqualityApplicationRow {
    /// Carry the producer's full application correspondence.
    pub fn new(
        application: CheckedNodeId,
        occurrence: CheckedOccurrence,
        source_map_entry: CheckedSourceMapEntry,
        node_key: NodeKey,
        location: Location,
    ) -> Self {
        Self {
            application,
            occurrence,
            source_map_entry,
            node_key,
            location,
        }
    }
}

/// A selected identity needed to map an admitted equality into QSV.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EqualityWitnessKind {
    /// Original application key and authored location.
    Application,
    /// Original descriptor of a Text type.
    TextType,
    /// Original identity or descriptor of a Plan operand type.
    PlanType,
    /// Original identity of a quantity type.
    QuantityType,
    /// Original declared dimension identity.
    DimensionDeclaration,
    /// Original base dimension term identity.
    BaseDimensionTerm,
    /// Original ordered conversion edge target identity.
    ConversionEdgeTarget,
    /// Source-declared unit identity.
    SourceUnit,
    /// Original compound unit identity.
    CompoundUnit,
    /// Original enum declaration identity.
    EnumDeclaration,
    /// Original enum member identity.
    EnumMember,
}

/// Stable reason a read did not return one complete admitted equality.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum AdmittedEqualityReadErrorCode {
    /// The requested node belongs to another package.
    #[error("foreign equality application")]
    ForeignApplication,
    /// The requested node is absent.
    #[error("unknown equality application")]
    UnknownApplication,
    /// The node or term is not a binary application.
    #[error("node is not a binary application")]
    WrongApplicationForm,
    /// The binary application is not equality or inequality.
    #[error("application is not equality")]
    NonEqualityApplication,
    /// The occurrence is not owned by the requested application.
    #[error("occurrence does not belong to application")]
    WrongOccurrence,
    /// The producer retained no decision for this application and occurrence.
    #[error("equality decision is unavailable")]
    UnavailableEqualityDecision,
    /// The selected checking-stage identity is absent.
    #[error("equality witness is unavailable")]
    UnavailableEqualityWitness,
    /// A selected checking-stage row disagrees with admission.
    #[error("equality witness does not match admission")]
    MismatchedEqualityWitness,
    /// A checking-stage row belongs to another package.
    #[error("equality witness belongs to another package")]
    ForeignEqualityWitness,
    /// More than one row claims the same application and occurrence.
    #[error("competing equality witnesses")]
    DuplicateEqualityWitness,
    /// A read charge exceeded the finite work limit.
    #[error("equality read work limit exhausted")]
    WorkLimit,
    /// QSV refused a non-work resource charge while materializing admitted facts.
    #[error("equality materialization resource limit exhausted")]
    Resource,
    /// The already joined descriptor could not be materialized by QSV.
    #[error("equality materialization failed")]
    Materialization,
}

/// Refusal at the first structural node or work charge that failed.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("{code} at {at:?}")]
pub struct AdmittedEqualityReadError {
    /// Stable refusal kind.
    pub code: AdmittedEqualityReadErrorCode,
    /// Requested equality application.
    pub application: Box<CheckedNodeId>,
    /// First structural node whose join failed.
    pub at: Box<CheckedNodeId>,
    /// Missing witness kind, when a witness is unavailable.
    pub witness: Option<EqualityWitnessKind>,
    /// Successfully charged read work.
    pub consumed_work: u64,
    /// First refused charge; zero for a semantic refusal.
    pub attempted_work: u64,
    /// QSV's original typed materialization failure and locus.
    pub materialization: Option<Box<EqualityMaterializationError>>,
}

struct ReadWork {
    consumed: u64,
    limit: u64,
}

impl ReadWork {
    fn charge(
        &mut self,
        application: &CheckedNodeId,
        at: &CheckedNodeId,
    ) -> Result<(), AdmittedEqualityReadError> {
        self.charge_amount(application, at, 1)
    }

    fn charge_amount(
        &mut self,
        application: &CheckedNodeId,
        at: &CheckedNodeId,
        amount: u64,
    ) -> Result<(), AdmittedEqualityReadError> {
        let Some(attempted) = self.consumed.checked_add(amount) else {
            return Err(read_error(
                AdmittedEqualityReadErrorCode::WorkLimit,
                application,
                at,
                self.consumed,
                amount,
            ));
        };
        if attempted > self.limit {
            return Err(read_error(
                AdmittedEqualityReadErrorCode::WorkLimit,
                application,
                at,
                self.consumed,
                amount,
            ));
        }
        self.consumed = attempted;
        Ok(())
    }
}

fn read_error(
    code: AdmittedEqualityReadErrorCode,
    application: &CheckedNodeId,
    at: &CheckedNodeId,
    consumed_work: u64,
    attempted_work: u64,
) -> AdmittedEqualityReadError {
    AdmittedEqualityReadError {
        code,
        application: Box::new(application.clone()),
        at: Box::new(at.clone()),
        witness: None,
        consumed_work,
        attempted_work,
        materialization: None,
    }
}

fn missing_witness(
    application: &CheckedNodeId,
    at: &CheckedNodeId,
    kind: EqualityWitnessKind,
    work: &ReadWork,
) -> AdmittedEqualityReadError {
    let mut error = read_error(
        AdmittedEqualityReadErrorCode::UnavailableEqualityWitness,
        application,
        at,
        work.consumed,
        0,
    );
    error.witness = Some(kind);
    error
}

fn exact_type(
    package: &CheckedPackageV2,
    application: &CheckedNodeId,
    type_id: &CheckedNodeId,
    work: &mut ReadWork,
) -> Result<ValueType, AdmittedEqualityReadError> {
    work.charge(application, type_id)?;
    let kind = package.node_kind(type_id).ok_or_else(|| {
        missing_witness(application, type_id, EqualityWitnessKind::PlanType, work)
    })?;
    match kind {
        CheckedNodeKind::ScalarType(ScalarTypeForm::Boolean) => Ok(ValueType::Boolean),
        CheckedNodeKind::ScalarType(ScalarTypeForm::Integer) => Ok(ValueType::Integer),
        CheckedNodeKind::ScalarType(ScalarTypeForm::String) => Ok(ValueType::String),
        CheckedNodeKind::ScalarType(ScalarTypeForm::Uuid) => Ok(ValueType::Uuid),
        CheckedNodeKind::ScalarType(ScalarTypeForm::Timestamp) => Ok(ValueType::Timestamp),
        CheckedNodeKind::BoundedDomain(BoundedDomainForm::IntegerRange) => {
            let (lower, upper) = package
                .term(type_id)
                .and_then(integer_range_bounds)
                .ok_or_else(|| {
                    missing_witness(application, type_id, EqualityWitnessKind::PlanType, work)
                })?;
            let bytes = lower.len().saturating_add(upper.len());
            work.charge_amount(
                application,
                type_id,
                u64::try_from(bytes).unwrap_or(u64::MAX),
            )?;
            let lower: Integer = lower.parse().map_err(|_| {
                missing_witness(application, type_id, EqualityWitnessKind::PlanType, work)
            })?;
            let upper: Integer = upper.parse().map_err(|_| {
                missing_witness(application, type_id, EqualityWitnessKind::PlanType, work)
            })?;
            let interval = IntegerInterval::new(lower, upper).map_err(|_| {
                missing_witness(application, type_id, EqualityWitnessKind::PlanType, work)
            })?;
            Ok(ValueType::Int(interval))
        }
        CheckedNodeKind::BoundedDomain(BoundedDomainForm::TextBounds) => {
            let (min, max, profile) =
                package.term(type_id).and_then(text_bounds).ok_or_else(|| {
                    missing_witness(application, type_id, EqualityWitnessKind::TextType, work)
                })?;
            let bytes = min
                .len()
                .saturating_add(max.len())
                .saturating_add(profile.len());
            work.charge_amount(
                application,
                type_id,
                u64::try_from(bytes).unwrap_or(u64::MAX),
            )?;
            let min = min.parse().map_err(|_| {
                missing_witness(application, type_id, EqualityWitnessKind::TextType, work)
            })?;
            let max = max.parse().map_err(|_| {
                missing_witness(application, type_id, EqualityWitnessKind::TextType, work)
            })?;
            let profile = TextProfile::from_code(profile).ok_or_else(|| {
                missing_witness(application, type_id, EqualityWitnessKind::TextType, work)
            })?;
            let text = TextType::new(min, max, profile).map_err(|_| {
                missing_witness(application, type_id, EqualityWitnessKind::TextType, work)
            })?;
            Ok(ValueType::Text(text))
        }
        other => {
            let witness = match other {
                CheckedNodeKind::ScalarType(ScalarTypeForm::Text) => EqualityWitnessKind::TextType,
                CheckedNodeKind::ScalarType(ScalarTypeForm::Unit) => {
                    EqualityWitnessKind::QuantityType
                }
                CheckedNodeKind::ScalarType(ScalarTypeForm::CompoundUnit) => {
                    EqualityWitnessKind::CompoundUnit
                }
                CheckedNodeKind::ScalarType(ScalarTypeForm::Enum) => {
                    EqualityWitnessKind::EnumDeclaration
                }
                _ => EqualityWitnessKind::PlanType,
            };
            Err(missing_witness(application, type_id, witness, work))
        }
    }
}

fn operand(
    package: &CheckedPackageV2,
    application: &CheckedNodeId,
    selected: &super::CheckedEqualityOperandDecision,
    work: &mut ReadWork,
) -> Result<EqualityOperand, AdmittedEqualityReadError> {
    let source = exact_type(package, application, &selected.source_type, work)?;
    match &selected.conversion_target {
        Some(target) => Ok(EqualityOperand::converted(
            source,
            exact_type(package, application, target, work)?,
        )),
        None => Ok(EqualityOperand::typed(source)),
    }
}

fn exact_meter(work_limit: u64) -> Meter {
    Meter::new(ScalarLimits {
        integer_bits: u64::MAX,
        decimal_digits: u64::MAX,
        scale_expansion: u64::MAX,
        text_input_bytes: u64::MAX,
        text_scalars: u64::MAX,
        normalized_scalars: u64::MAX,
        unit_edges: u64::MAX,
        value_occurrences: u64::MAX,
        work_units: work_limit,
        result_units: u64::MAX,
    })
}

fn materialization_refusal(
    application: &CheckedNodeId,
    ir_work: u64,
    exact_work: u64,
    cause: EqualityMaterializationError,
) -> AdmittedEqualityReadError {
    let (code, attempted_work) = match &cause {
        EqualityMaterializationError::Resource { cause, .. } => (
            if cause.limit_kind == LimitKind::WorkUnits {
                AdmittedEqualityReadErrorCode::WorkLimit
            } else {
                AdmittedEqualityReadErrorCode::Resource
            },
            // The nested QSV Incomplete retains the mathematical integer if
            // a denied charge cannot be represented by this u64 summary.
            cause.next_charge.to_u64().unwrap_or(u64::MAX),
        ),
        _ => (AdmittedEqualityReadErrorCode::Materialization, 0),
    };
    let mut error = read_error(
        code,
        application,
        application,
        ir_work.saturating_add(exact_work),
        attempted_work,
    );
    error.materialization = Some(Box::new(cause));
    error
}

fn join_source_unit(
    package: &CheckedPackageV2,
    application: &CheckedNodeId,
    unit: &super::CheckedResolvedEqualityUnit,
    rows: &[SelectedSourceUnitRow],
    work: &mut ReadWork,
) -> Result<(), AdmittedEqualityReadError> {
    work.charge(application, &unit.unit)?;
    let Some(node) = package.node(&unit.unit) else {
        return Err(missing_witness(
            application,
            &unit.unit,
            EqualityWitnessKind::SourceUnit,
            work,
        ));
    };
    let CheckedNodeOwner::Source {
        authority,
        identity,
    } = node.owner.as_ref().ok_or_else(|| {
        missing_witness(
            application,
            &unit.unit,
            EqualityWitnessKind::QuantityType,
            work,
        )
    })?
    else {
        return Err(missing_witness(
            application,
            &unit.unit,
            EqualityWitnessKind::QuantityType,
            work,
        ));
    };
    let mut found = None;
    for row in rows {
        work.charge(application, &row.structural_node)?;
        if row.structural_node.package != package.wire.name {
            return Err(read_error(
                AdmittedEqualityReadErrorCode::ForeignEqualityWitness,
                application,
                &row.structural_node,
                work.consumed,
                0,
            ));
        }
        if row.structural_node == unit.unit {
            if found.is_some() {
                return Err(read_error(
                    AdmittedEqualityReadErrorCode::DuplicateEqualityWitness,
                    application,
                    &unit.unit,
                    work.consumed,
                    0,
                ));
            }
            found = Some(row);
        }
    }
    let row = found.ok_or_else(|| {
        missing_witness(
            application,
            &unit.unit,
            EqualityWitnessKind::SourceUnit,
            work,
        )
    })?;
    let mut selected_source = false;
    for source in &package.lock().sources {
        work.charge(application, &unit.unit)?;
        if source.authority == *authority && source.identity == *identity {
            selected_source = true;
        }
    }
    for _ in &row.qualified_declaration {
        work.charge(application, &unit.unit)?;
    }
    if package.node_kind(&unit.unit) != Some(CheckedNodeKind::ScalarType(ScalarTypeForm::Unit))
        || &row.source_authority != authority
        || &row.source_identity != identity
        || !selected_source
        || node
            .declaration
            .as_ref()
            .is_none_or(|declaration| declaration.qualified_name != row.qualified_declaration)
        || !matches!(row.unit_id, UnitId::Declared(_))
        || !matches!(unit.form, CheckedResolvedEqualityUnitForm::Declared { .. })
    {
        return Err(read_error(
            AdmittedEqualityReadErrorCode::MismatchedEqualityWitness,
            application,
            &unit.unit,
            work.consumed,
            0,
        ));
    }
    Ok(())
}

impl CheckedPackageV2 {
    /// Read a complete retained equality using the producer's original application row.
    ///
    /// Types requiring further QSL-stage identities refuse with their missing
    /// structural witness. The read never projects a type from an unrelated
    /// declaration row, reconstructs an Exact key, or selects a schedule.
    pub fn admitted_equality(
        &self,
        application: &CheckedNodeId,
        occurrence: &CheckedOccurrence,
        application_rows: &[SelectedEqualityApplicationRow],
        source_unit_rows: &[SelectedSourceUnitRow],
        work_limit: u64,
    ) -> Result<AdmittedEqualityDecision, AdmittedEqualityReadError> {
        let mut work = ReadWork {
            consumed: 0,
            limit: work_limit,
        };
        work.charge(application, application)?;
        if application.package != self.wire.name {
            return Err(read_error(
                AdmittedEqualityReadErrorCode::ForeignApplication,
                application,
                application,
                work.consumed,
                0,
            ));
        }
        let Some(&position) = self.node_index.get(application) else {
            return Err(read_error(
                AdmittedEqualityReadErrorCode::UnknownApplication,
                application,
                application,
                work.consumed,
                0,
            ));
        };
        if self.kinds.get(position) != Some(&CheckedNodeKind::Expression(ExpressionForm::Binary)) {
            return Err(read_error(
                AdmittedEqualityReadErrorCode::WrongApplicationForm,
                application,
                application,
                work.consumed,
                0,
            ));
        }
        let Some(CheckedTerm::Application {
            operator: ApplicationOperator::Binary,
            operation,
            ..
        }) = self.terms.get(position)
        else {
            return Err(read_error(
                AdmittedEqualityReadErrorCode::WrongApplicationForm,
                application,
                application,
                work.consumed,
                0,
            ));
        };
        if operation.identity().and_then(equality_operator).is_none() {
            return Err(read_error(
                AdmittedEqualityReadErrorCode::NonEqualityApplication,
                application,
                application,
                work.consumed,
                0,
            ));
        }
        let source_position = self.source_index.get(&(
            application.clone(),
            occurrence.role.clone(),
            occurrence.ordinal,
        ));
        if !self.wire.semantic_graph.nodes[position]
            .occurrences
            .contains(occurrence)
            || source_position.is_none()
        {
            return Err(read_error(
                AdmittedEqualityReadErrorCode::WrongOccurrence,
                application,
                application,
                work.consumed,
                0,
            ));
        }
        let Some(data) = self.wire.checked_term_data.as_ref() else {
            return Err(read_error(
                AdmittedEqualityReadErrorCode::UnavailableEqualityDecision,
                application,
                application,
                work.consumed,
                0,
            ));
        };
        let mut selected = None;
        for decision in &data.equality_decisions {
            work.charge(application, &decision.application)?;
            if &decision.application == application && &decision.occurrence == occurrence {
                selected = Some(decision);
                break;
            }
        }
        let Some(decision) = selected else {
            return Err(read_error(
                AdmittedEqualityReadErrorCode::UnavailableEqualityDecision,
                application,
                application,
                work.consumed,
                0,
            ));
        };
        let mut row = None;
        let mut seen_rows = BTreeSet::new();
        for candidate in application_rows {
            work.charge(application, &candidate.application)?;
            if candidate.application.package != self.wire.name {
                return Err(read_error(
                    AdmittedEqualityReadErrorCode::ForeignEqualityWitness,
                    application,
                    &candidate.application,
                    work.consumed,
                    0,
                ));
            }
            if !seen_rows.insert((&candidate.application, &candidate.occurrence)) {
                return Err(read_error(
                    AdmittedEqualityReadErrorCode::DuplicateEqualityWitness,
                    application,
                    &candidate.application,
                    work.consumed,
                    0,
                ));
            }
            let candidate_source = self.source_index.get(&(
                candidate.application.clone(),
                candidate.occurrence.role.clone(),
                candidate.occurrence.ordinal,
            ));
            for _ in &candidate.source_map_entry.regions {
                work.charge(application, &candidate.application)?;
            }
            let candidate_valid =
                self.node_index
                    .get(&candidate.application)
                    .is_some_and(|position| {
                        self.kinds.get(*position)
                            == Some(&CheckedNodeKind::Expression(ExpressionForm::Binary))
                            && matches!(self.terms.get(*position), Some(CheckedTerm::Application {
                        operator: ApplicationOperator::Binary, operation, ..
                    }) if operation.identity().and_then(equality_operator).is_some())
                    })
                    && candidate_source.and_then(|position| self.wire.source_map.get(*position))
                        == Some(&candidate.source_map_entry)
                    && *candidate.location.node() == candidate.node_key
                    && candidate.location.occurrence().ordinal() == candidate.occurrence.ordinal
                    && candidate.location.occurrence().role().as_str()
                        == occurrence_role(&candidate.occurrence.role);
            if !candidate_valid {
                return Err(read_error(
                    AdmittedEqualityReadErrorCode::MismatchedEqualityWitness,
                    application,
                    &candidate.application,
                    work.consumed,
                    0,
                ));
            }
            if &candidate.application == application && &candidate.occurrence == occurrence {
                row = Some(candidate);
            }
        }
        let Some(row) = row else {
            return Err(missing_witness(
                application,
                application,
                EqualityWitnessKind::Application,
                &work,
            ));
        };
        if let Some(unit) = decision.units.first() {
            if matches!(
                self.node(&unit.unit).and_then(|node| node.owner.as_ref()),
                Some(CheckedNodeOwner::Source { .. })
            ) {
                join_source_unit(self, application, unit, source_unit_rows, &mut work)?;
            }
            let (first_missing, witness) = match &unit.form {
                CheckedResolvedEqualityUnitForm::Declared { dimension_node, .. } => {
                    (dimension_node, EqualityWitnessKind::DimensionDeclaration)
                }
                CheckedResolvedEqualityUnitForm::Compound {
                    dimension_terms,
                    terms,
                } => {
                    if let Some(term) = dimension_terms.first() {
                        (&term.dimension, EqualityWitnessKind::BaseDimensionTerm)
                    } else if let Some(term) = terms.first() {
                        (&term.root_unit, EqualityWitnessKind::CompoundUnit)
                    } else {
                        (&unit.unit, EqualityWitnessKind::CompoundUnit)
                    }
                }
            };
            return Err(missing_witness(application, first_missing, witness, &work));
        }
        if let Some(member) = decision.enum_members.first() {
            return Err(missing_witness(
                application,
                &member.member,
                EqualityWitnessKind::EnumMember,
                &work,
            ));
        }
        work.charge(application, &decision.left.source_type)?;
        let left = operand(self, application, &decision.left, &mut work)?;
        work.charge(application, &decision.right.source_type)?;
        let right = operand(self, application, &decision.right, &mut work)?;
        let schedule = match decision.schedule {
            super::CheckedEqualitySchedule::Text => EqualitySchedule::Text,
            super::CheckedEqualitySchedule::Enum => EqualitySchedule::Enum,
            super::CheckedEqualitySchedule::Quantity => EqualitySchedule::Quantity,
            super::CheckedEqualitySchedule::Plan => EqualitySchedule::Plan,
        };
        if matches!(
            schedule,
            EqualitySchedule::Text | EqualitySchedule::Enum | EqualitySchedule::Quantity
        ) {
            return Err(missing_witness(
                application,
                &decision.left.source_type,
                match schedule {
                    EqualitySchedule::Text => EqualityWitnessKind::TextType,
                    EqualitySchedule::Enum => EqualityWitnessKind::EnumDeclaration,
                    EqualitySchedule::Quantity => EqualityWitnessKind::QuantityType,
                    EqualitySchedule::Plan => EqualityWitnessKind::PlanType,
                },
                &work,
            ));
        }
        let descriptor = EqualityDecisionDescriptor {
            location: row.location.clone(),
            operator: match decision.operator {
                CheckedEqualityOperator::Equal => EqualityOperator::Equal,
                CheckedEqualityOperator::NotEqual => EqualityOperator::NotEqual,
            },
            left,
            right,
            schedule,
            units: Vec::new(),
            enum_declarations: Vec::new(),
            enum_members: Vec::new(),
        };
        let mut meter = exact_meter(work.limit - work.consumed);
        mint_admitted_equality_decision(ReaderSeal, descriptor, &mut meter).map_err(|cause| {
            materialization_refusal(
                application,
                work.consumed,
                meter.consumed(LimitKind::WorkUnits),
                cause,
            )
        })
    }
}

fn occurrence_role(role: &CheckedOccurrenceRole) -> &'static str {
    match role {
        CheckedOccurrenceRole::Declaration => "declaration",
        CheckedOccurrenceRole::Type => "type",
        CheckedOccurrenceRole::Expression => "expression",
        CheckedOccurrenceRole::Anchor => "anchor",
        CheckedOccurrenceRole::Claim => "claim",
        CheckedOccurrenceRole::Generated => "generated",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quire_exact::{ChargePoint, Incomplete};
    use quire_semantic_value::reader_admission::EqualityMaterializationLocus;

    /// Trace: FR-353-AC-4.
    /// Provenance: IR-1209 QSV resource translation at the admitted boundary.
    #[test]
    fn qsv_work_denial_keeps_exact_charge_and_locus() {
        let application = super::super::test_support::node_id('a');
        let cause = EqualityMaterializationError::Resource {
            locus: EqualityMaterializationLocus::Unit { row: 0 },
            cause: Incomplete {
                limit_kind: LimitKind::WorkUnits,
                limit: 3,
                consumed: 3,
                next_charge: Integer::from(2_u64),
                charge_point: ChargePoint::UnitIdentityRead,
            },
        };
        let refused = materialization_refusal(&application, 5, 3, cause.clone());
        assert_eq!(refused.code, AdmittedEqualityReadErrorCode::WorkLimit);
        assert_eq!(refused.consumed_work, 8);
        assert_eq!(refused.attempted_work, 2);
        assert_eq!(refused.materialization.as_deref(), Some(&cause));

        let resource = EqualityMaterializationError::Resource {
            locus: EqualityMaterializationLocus::EnumMember { row: 1 },
            cause: Incomplete {
                limit_kind: LimitKind::TextInputBytes,
                limit: 4,
                consumed: 4,
                next_charge: Integer::from(7_u64),
                charge_point: ChargePoint::EnumIdentityRead,
            },
        };
        let refused = materialization_refusal(&application, 5, 2, resource.clone());
        assert_eq!(refused.code, AdmittedEqualityReadErrorCode::Resource);
        assert_eq!(refused.consumed_work, 7);
        assert_eq!(refused.attempted_work, 7);
        assert_eq!(refused.materialization.as_deref(), Some(&resource));
    }
}
