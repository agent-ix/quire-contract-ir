// SPDX-License-Identifier: AGPL-3.0-or-later
//! Structural admission and bounded reads of producer-selected equality decisions.

use super::checked_term_data::{CheckedEqualityOperator, CheckedTermDataV2};
use super::exact_projection::SelectedSourceUnitRow;
use super::extension::integer_range_bounds;
use super::operations::is_type_shaped;
use super::{
    ApplicationOperator, BoundedDomainForm, CheckedNodeKind, CheckedNodeOwner, CheckedPackageV2,
    CheckedResolvedEqualityUnitForm, CheckedTerm, ExpressionForm, ModelForm,
    RawCheckedPackageWireV2, ScalarTypeForm, ValueForm, WorkMeter,
};
use crate::checked_package::common::ValidationFailure;
use crate::checked_package::shared::{
    CheckedNodeId, CheckedOccurrence, CheckedOccurrenceRole, CheckedPackageRefusalCause,
    CheckedPackageRefusalCode, CheckedSourceMapEntry, JsonPointer,
};
use quire_exact::{
    Integer, IntegerInterval, LimitKind, Location, Meter, NodeKey, Rational, ScalarLimits,
    TextProfile, TextType, UnitId, ValueType, VariantId,
};
use quire_semantic_value::declaration::{
    AdmittedEqualityDecision, EqualityDecisionDescriptor, EqualityEnumDeclarationDescriptor,
    EqualityEnumMemberDescriptor, EqualityOperand, EqualityOperator, EqualitySchedule,
    EqualityUnitDescriptor, EqualityUnitEdgeDescriptor, EqualityUnitForm,
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

/// One QSL-selected equality fact joined to its emitted structural node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedEqualityFactRow {
    /// Exact application identity retained by the producer.
    pub application: NodeKey,
    /// Emitted application node.
    pub checked_application: CheckedNodeId,
    /// Emitted application occurrence.
    pub occurrence: CheckedOccurrence,
    /// Source-map correspondence for that occurrence.
    pub source_map_entry: CheckedSourceMapEntry,
    /// Original key of this fact's structural node.
    pub structural_key: NodeKey,
    /// Emitted structural node of this fact.
    pub structural_node: CheckedNodeId,
    /// Owner of the emitted structural node, if present.
    pub owner: Option<CheckedNodeOwner>,
    /// Checker-held Exact type for a typed operand or comparison fact.
    pub value_type: Option<ValueType>,
    /// The producer's selected fact and role.
    pub fact: EqualityFactWitness,
}

/// A QSL-selected fact used by an admitted equality decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EqualityFactWitness {
    /// Source or conversion-target operand type.
    OperandType {
        position: usize,
        role: EqualityOperandTypeRole,
    },
    /// Resolved unit and its quantity type.
    Unit {
        unit_id: UnitId,
        quantity_type: CheckedNodeId,
        operand_position: usize,
        role: EqualityUnitRole,
    },
    /// A dimension declaration in a resolved unit's identity closure.
    DimensionDeclaration {
        unit_id: UnitId,
        operand_position: usize,
        role: EqualityDimensionRole,
        ordinal: usize,
    },
    /// An ordered direct term of a declared dimension.
    DimensionTerm {
        unit_id: UnitId,
        operand_position: usize,
        ordinal: usize,
        base: NodeKey,
        base_node: CheckedNodeId,
        exponent: Box<str>,
    },
    /// An ordered conversion edge to another declared unit.
    ConversionEdge {
        unit_id: UnitId,
        operand_position: usize,
        ordinal: usize,
        target: NodeKey,
        target_node: CheckedNodeId,
        scale: super::CheckedRational,
        offset: super::CheckedRational,
    },
    /// Compared Text or Plan type.
    TextPlanType {
        position: usize,
        role: EqualityOperandTypeRole,
    },
    /// One selected enum member and its exact source and lowered identities.
    EnumMember {
        source_declaration: NodeKey,
        source_declaration_node: CheckedNodeId,
        declaration: NodeKey,
        declaration_node: CheckedNodeId,
        member: NodeKey,
        member_node: CheckedNodeId,
        case: Box<str>,
        ordinal: usize,
        ordered: bool,
    },
}

/// Source or selected conversion-target role for an equality type fact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EqualityOperandTypeRole {
    Source,
    ConversionTarget,
}

/// Source or conversion-target role for a selected resolved unit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EqualityUnitRole {
    Source,
    ConversionTarget,
}

fn unit_role(role: EqualityOperandTypeRole) -> EqualityUnitRole {
    match role {
        EqualityOperandTypeRole::Source => EqualityUnitRole::Source,
        EqualityOperandTypeRole::ConversionTarget => EqualityUnitRole::ConversionTarget,
    }
}

struct OperandFactSelection {
    position: usize,
    role: EqualityOperandTypeRole,
}

/// Role of a dimension declaration retained for a resolved unit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EqualityDimensionRole {
    UnitDimension,
    ConversionDimension,
    CompoundBase,
    BaseDimension,
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

fn selected_fact<'a>(
    package: &CheckedPackageV2,
    application: &CheckedNodeId,
    application_row: &SelectedEqualityApplicationRow,
    rows: &'a [SelectedEqualityFactRow],
    at: &CheckedNodeId,
    kind: EqualityWitnessKind,
    predicate: impl Fn(&EqualityFactWitness) -> bool,
    work: &mut ReadWork,
) -> Result<&'a SelectedEqualityFactRow, AdmittedEqualityReadError> {
    let mut found = None;
    for row in rows {
        work.charge(application, &row.structural_node)?;
        if row.checked_application != *application || row.occurrence != application_row.occurrence {
            continue;
        }
        if row.structural_node.package != package.wire.name {
            return Err(read_error(
                AdmittedEqualityReadErrorCode::ForeignEqualityWitness,
                application,
                &row.structural_node,
                work.consumed,
                0,
            ));
        }
        if row.application != application_row.node_key
            || row.source_map_entry != application_row.source_map_entry
            || package
                .node(&row.structural_node)
                .is_none_or(|node| node.owner != row.owner)
        {
            return Err(read_error(
                AdmittedEqualityReadErrorCode::MismatchedEqualityWitness,
                application,
                &row.structural_node,
                work.consumed,
                0,
            ));
        }
        if predicate(&row.fact) && row.structural_node == *at {
            if found.is_some() {
                return Err(read_error(
                    AdmittedEqualityReadErrorCode::DuplicateEqualityWitness,
                    application,
                    at,
                    work.consumed,
                    0,
                ));
            }
            found = Some(row);
        }
    }
    found.ok_or_else(|| missing_witness(application, at, kind, work))
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

fn operand_type(
    package: &CheckedPackageV2,
    application: &CheckedNodeId,
    application_row: &SelectedEqualityApplicationRow,
    decision: &super::CheckedEqualityDecision,
    rows: &[SelectedEqualityFactRow],
    type_id: &CheckedNodeId,
    selection: OperandFactSelection,
    work: &mut ReadWork,
) -> Result<ValueType, AdmittedEqualityReadError> {
    let witness = match decision.schedule {
        super::CheckedEqualitySchedule::Text => EqualityWitnessKind::TextType,
        super::CheckedEqualitySchedule::Enum => EqualityWitnessKind::EnumDeclaration,
        super::CheckedEqualitySchedule::Quantity => EqualityWitnessKind::QuantityType,
        super::CheckedEqualitySchedule::Plan => EqualityWitnessKind::PlanType,
    };
    let primitive_plan = if decision.schedule == super::CheckedEqualitySchedule::Plan {
        match exact_type(package, application, type_id, work) {
            Ok(value) => Some(value),
            Err(error)
                if error.code == AdmittedEqualityReadErrorCode::UnavailableEqualityWitness =>
            {
                None
            }
            Err(error) => return Err(error),
        }
    } else {
        None
    };
    let OperandFactSelection { position, role } = selection;
    let mut has_fact = false;
    for row in rows {
        work.charge(application, &row.structural_node)?;
        if row.checked_application == *application
            && row.structural_node == *type_id
            && matches!(row.fact, EqualityFactWitness::OperandType { position: p, role: r }
            if p == position && r == role)
        {
            has_fact = true;
        }
    }
    if !has_fact {
        if let Some(projected) = primitive_plan.as_ref() {
            return Ok(projected.clone());
        }
    }
    let operand_row = selected_fact(
        package,
        application,
        application_row,
        rows,
        type_id,
        witness,
        |fact| {
            matches!(fact, EqualityFactWitness::OperandType { position: p, role: r }
            if *p == position && *r == role)
        },
        work,
    )?;
    let value = operand_row
        .value_type
        .as_ref()
        .ok_or_else(|| missing_witness(application, type_id, witness, work))?;
    if decision.schedule == super::CheckedEqualitySchedule::Text
        && (!matches!(value, ValueType::Text(_))
            || !matches!(
                package.node_kind(type_id),
                Some(
                    CheckedNodeKind::ScalarType(ScalarTypeForm::Text)
                        | CheckedNodeKind::BoundedDomain(BoundedDomainForm::TextBounds)
                )
            ))
    {
        return Err(read_error(
            AdmittedEqualityReadErrorCode::MismatchedEqualityWitness,
            application,
            type_id,
            work.consumed,
            0,
        ));
    }
    if decision.schedule == super::CheckedEqualitySchedule::Text
        && package.node_kind(type_id)
            == Some(CheckedNodeKind::BoundedDomain(
                BoundedDomainForm::TextBounds,
            ))
        && exact_type(package, application, type_id, work)? != *value
    {
        return Err(read_error(
            AdmittedEqualityReadErrorCode::MismatchedEqualityWitness,
            application,
            type_id,
            work.consumed,
            0,
        ));
    }
    if let Some(projected) = primitive_plan {
        if projected != *value {
            return Err(read_error(
                AdmittedEqualityReadErrorCode::MismatchedEqualityWitness,
                application,
                type_id,
                work.consumed,
                0,
            ));
        }
    }
    if matches!(
        decision.schedule,
        super::CheckedEqualitySchedule::Text | super::CheckedEqualitySchedule::Plan
    ) && (role == EqualityOperandTypeRole::ConversionTarget
        || [&decision.left, &decision.right][position]
            .conversion_target
            .is_none())
    {
        let type_row = selected_fact(
            package,
            application,
            application_row,
            rows,
            type_id,
            witness,
            |fact| {
                matches!(fact, EqualityFactWitness::TextPlanType { position: p, role: r }
                if *p == position && *r == role)
            },
            work,
        )?;
        if type_row.structural_key != operand_row.structural_key
            || type_row.value_type.as_ref() != Some(value)
        {
            return Err(read_error(
                AdmittedEqualityReadErrorCode::MismatchedEqualityWitness,
                application,
                type_id,
                work.consumed,
                0,
            ));
        }
    }
    if decision.schedule == super::CheckedEqualitySchedule::Quantity {
        let unit_row = selected_fact(
            package,
            application,
            application_row,
            rows,
            type_id,
            EqualityWitnessKind::QuantityType,
            |fact| {
                matches!(fact, EqualityFactWitness::Unit { quantity_type, operand_position, role: r, .. }
                if quantity_type == type_id && *operand_position == position && *r == unit_role(role))
            },
            work,
        )?;
        let EqualityFactWitness::Unit { unit_id, .. } = &unit_row.fact else {
            return Err(read_error(
                AdmittedEqualityReadErrorCode::MismatchedEqualityWitness,
                application,
                type_id,
                work.consumed,
                0,
            ));
        };
        let mut selected_unit = false;
        for unit in &decision.units {
            work.charge(application, &unit.unit)?;
            if unit.quantity_type == *type_id && unit.unit == unit_row.structural_node {
                selected_unit = true;
            }
        }
        if value != &ValueType::Quantity(unit_id.clone()) || !selected_unit {
            return Err(read_error(
                AdmittedEqualityReadErrorCode::MismatchedEqualityWitness,
                application,
                type_id,
                work.consumed,
                0,
            ));
        }
    }
    if let ValueType::Composite(key) = value {
        if &operand_row.structural_key != key
            || !matches!(
                package.node_kind(type_id),
                Some(
                    CheckedNodeKind::CompositeType(_)
                        | CheckedNodeKind::Model(
                            ModelForm::RecordValueType
                                | ModelForm::TupleValueType
                                | ModelForm::VariantType
                        )
                )
            )
        {
            return Err(read_error(
                AdmittedEqualityReadErrorCode::MismatchedEqualityWitness,
                application,
                type_id,
                work.consumed,
                0,
            ));
        }
    }
    Ok(value.clone())
}

fn operand(
    package: &CheckedPackageV2,
    application: &CheckedNodeId,
    application_row: &SelectedEqualityApplicationRow,
    decision: &super::CheckedEqualityDecision,
    rows: &[SelectedEqualityFactRow],
    selected: &super::CheckedEqualityOperandDecision,
    position: usize,
    work: &mut ReadWork,
) -> Result<EqualityOperand, AdmittedEqualityReadError> {
    let source = operand_type(
        package,
        application,
        application_row,
        decision,
        rows,
        &selected.source_type,
        OperandFactSelection {
            position,
            role: EqualityOperandTypeRole::Source,
        },
        work,
    )?;
    match &selected.conversion_target {
        Some(target) => Ok(EqualityOperand::converted(
            source,
            operand_type(
                package,
                application,
                application_row,
                decision,
                rows,
                target,
                OperandFactSelection {
                    position,
                    role: EqualityOperandTypeRole::ConversionTarget,
                },
                work,
            )?,
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
    unit_node: &CheckedNodeId,
    expected_id: &UnitId,
    rows: &[SelectedSourceUnitRow],
    work: &mut ReadWork,
) -> Result<(), AdmittedEqualityReadError> {
    work.charge(application, unit_node)?;
    let Some(node) = package.node(unit_node) else {
        return Err(missing_witness(
            application,
            unit_node,
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
            unit_node,
            EqualityWitnessKind::QuantityType,
            work,
        )
    })?
    else {
        return Err(missing_witness(
            application,
            unit_node,
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
        if row.structural_node == *unit_node {
            if found.is_some() {
                return Err(read_error(
                    AdmittedEqualityReadErrorCode::DuplicateEqualityWitness,
                    application,
                    unit_node,
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
            unit_node,
            EqualityWitnessKind::SourceUnit,
            work,
        )
    })?;
    let mut selected_source = false;
    for source in &package.lock().sources {
        work.charge(application, unit_node)?;
        if source.authority == *authority && source.identity == *identity {
            selected_source = true;
        }
    }
    for segment in &row.qualified_declaration {
        work.charge_amount(
            application,
            unit_node,
            u64::try_from(segment.len()).unwrap_or(u64::MAX),
        )?;
    }
    if package.node_kind(unit_node) != Some(CheckedNodeKind::ScalarType(ScalarTypeForm::Unit))
        || &row.source_authority != authority
        || &row.source_identity != identity
        || !selected_source
        || node
            .declaration
            .as_ref()
            .is_none_or(|declaration| declaration.qualified_name != row.qualified_declaration)
        || row.unit_id != *expected_id
        || !matches!(expected_id, UnitId::Declared(_))
    {
        return Err(read_error(
            AdmittedEqualityReadErrorCode::MismatchedEqualityWitness,
            application,
            unit_node,
            work.consumed,
            0,
        ));
    }
    Ok(())
}

fn checked_rational(
    value: &super::CheckedRational,
    application: &CheckedNodeId,
    at: &CheckedNodeId,
    work: &mut ReadWork,
) -> Result<Rational, AdmittedEqualityReadError> {
    work.charge_amount(
        application,
        at,
        u64::try_from(
            value
                .numerator
                .len()
                .saturating_add(value.denominator.len()),
        )
        .unwrap_or(u64::MAX),
    )?;
    let numerator: Integer = value.numerator.parse().map_err(|_| {
        read_error(
            AdmittedEqualityReadErrorCode::MismatchedEqualityWitness,
            application,
            at,
            work.consumed,
            0,
        )
    })?;
    let denominator: Integer = value.denominator.parse().map_err(|_| {
        read_error(
            AdmittedEqualityReadErrorCode::MismatchedEqualityWitness,
            application,
            at,
            work.consumed,
            0,
        )
    })?;
    Rational::new(numerator, denominator).map_err(|_| {
        read_error(
            AdmittedEqualityReadErrorCode::MismatchedEqualityWitness,
            application,
            at,
            work.consumed,
            0,
        )
    })
}

fn validate_dimension_terms(
    package: &CheckedPackageV2,
    application: &CheckedNodeId,
    application_row: &SelectedEqualityApplicationRow,
    unit_id: &UnitId,
    operand_position: usize,
    rows: &[SelectedEqualityFactRow],
    work: &mut ReadWork,
) -> Result<(), AdmittedEqualityReadError> {
    let mut seen = BTreeSet::new();
    for declaration in rows {
        work.charge(application, &declaration.structural_node)?;
        if declaration.checked_application != *application
            || !matches!(&declaration.fact,
                EqualityFactWitness::DimensionDeclaration { unit_id: id, operand_position: position, .. }
                if id == unit_id && *position == operand_position)
            || !seen.insert(declaration.structural_node.clone())
        {
            continue;
        }
        let body = match package.term(&declaration.structural_node) {
            Some(CheckedTerm::NominalBody(body)) => body,
            Some(_)
                if matches!(
                    package
                        .node(&declaration.structural_node)
                        .and_then(|node| node.owner.as_ref()),
                    Some(CheckedNodeOwner::Model { .. })
                ) =>
            {
                continue
            }
            _ => {
                return Err(read_error(
                    AdmittedEqualityReadErrorCode::MismatchedEqualityWitness,
                    application,
                    &declaration.structural_node,
                    work.consumed,
                    0,
                ))
            }
        };
        let Some(terms) = body.dimension_details() else {
            return Err(read_error(
                AdmittedEqualityReadErrorCode::MismatchedEqualityWitness,
                application,
                &declaration.structural_node,
                work.consumed,
                0,
            ));
        };
        for (ordinal, term) in terms.iter().enumerate() {
            work.charge(application, &term.dimension_node_id)?;
            work.charge_amount(
                application,
                &declaration.structural_node,
                u64::try_from(term.exponent.len()).unwrap_or(u64::MAX),
            )?;
            let fact = selected_fact(
                package,
                application,
                application_row,
                rows,
                &declaration.structural_node,
                EqualityWitnessKind::BaseDimensionTerm,
                |fact| {
                    matches!(fact, EqualityFactWitness::DimensionTerm {
                    unit_id: id, operand_position: position, ordinal: n, .. }
                    if id == unit_id && *position == operand_position && *n == ordinal)
                },
                work,
            )?;
            let EqualityFactWitness::DimensionTerm {
                base,
                base_node,
                exponent,
                ..
            } = &fact.fact
            else {
                return Err(read_error(
                    AdmittedEqualityReadErrorCode::MismatchedEqualityWitness,
                    application,
                    &declaration.structural_node,
                    work.consumed,
                    0,
                ));
            };
            if fact.structural_key != declaration.structural_key
                || *base_node != term.dimension_node_id
                || exponent != &term.exponent
            {
                return Err(read_error(
                    AdmittedEqualityReadErrorCode::MismatchedEqualityWitness,
                    application,
                    &declaration.structural_node,
                    work.consumed,
                    0,
                ));
            }
            let base_row = selected_fact(
                package,
                application,
                application_row,
                rows,
                base_node,
                EqualityWitnessKind::BaseDimensionTerm,
                |fact| {
                    matches!(fact, EqualityFactWitness::DimensionDeclaration {
                    unit_id: id, operand_position: position, role: EqualityDimensionRole::BaseDimension, .. }
                    if id == unit_id && *position == operand_position)
                },
                work,
            )?;
            if &base_row.structural_key != base {
                return Err(read_error(
                    AdmittedEqualityReadErrorCode::MismatchedEqualityWitness,
                    application,
                    base_node,
                    work.consumed,
                    0,
                ));
            }
        }
    }
    Ok(())
}

fn quantity_unit(
    package: &CheckedPackageV2,
    application: &CheckedNodeId,
    application_row: &SelectedEqualityApplicationRow,
    decision: &super::CheckedEqualityDecision,
    unit: &super::CheckedResolvedEqualityUnit,
    rows: &[SelectedEqualityFactRow],
    source_rows: &[SelectedSourceUnitRow],
    work: &mut ReadWork,
) -> Result<EqualityUnitDescriptor, AdmittedEqualityReadError> {
    let roles = [
        (0, EqualityUnitRole::Source, &decision.left.source_type),
        (1, EqualityUnitRole::Source, &decision.right.source_type),
    ];
    let targets = [
        decision.left.conversion_target.as_ref(),
        decision.right.conversion_target.as_ref(),
    ];
    let binding = roles
        .iter()
        .find(|(_, _, id)| **id == unit.quantity_type)
        .map(|(position, role, _)| (*position, *role))
        .or_else(|| {
            targets
                .iter()
                .enumerate()
                .find(|(_, id)| id.is_some_and(|id| *id == unit.quantity_type))
                .map(|(position, _)| (position, EqualityUnitRole::ConversionTarget))
        })
        .ok_or_else(|| {
            missing_witness(
                application,
                &unit.quantity_type,
                EqualityWitnessKind::QuantityType,
                work,
            )
        })?;
    let row = selected_fact(
        package,
        application,
        application_row,
        rows,
        &unit.unit,
        EqualityWitnessKind::QuantityType,
        |fact| {
            matches!(fact, EqualityFactWitness::Unit { quantity_type, operand_position, role, .. }
            if *quantity_type == unit.quantity_type && *operand_position == binding.0 && *role == binding.1)
        },
        work,
    )?;
    let EqualityFactWitness::Unit { unit_id, .. } = &row.fact else {
        return Err(read_error(
            AdmittedEqualityReadErrorCode::MismatchedEqualityWitness,
            application,
            &unit.unit,
            work.consumed,
            0,
        ));
    };
    let expected_kind = match &unit.form {
        CheckedResolvedEqualityUnitForm::Declared { .. } => {
            CheckedNodeKind::ScalarType(ScalarTypeForm::Unit)
        }
        CheckedResolvedEqualityUnitForm::Compound { .. } => {
            CheckedNodeKind::ScalarType(ScalarTypeForm::CompoundUnit)
        }
    };
    if package.node_kind(&unit.unit) != Some(expected_kind) {
        return Err(read_error(
            AdmittedEqualityReadErrorCode::MismatchedEqualityWitness,
            application,
            &unit.unit,
            work.consumed,
            0,
        ));
    }
    if matches!(
        package
            .node(&unit.unit)
            .and_then(|node| node.owner.as_ref()),
        Some(CheckedNodeOwner::Source { .. })
    ) {
        join_source_unit(package, application, &unit.unit, unit_id, source_rows, work)?;
    }
    let dimensions = |terms: &[super::CheckedUnitDimensionTerm], role, work: &mut ReadWork| {
        terms.iter().map(|term| {
            let fact = selected_fact(package, application, application_row, rows,
                &term.dimension, EqualityWitnessKind::BaseDimensionTerm,
                |fact| matches!(fact, EqualityFactWitness::DimensionDeclaration { unit_id: id, operand_position, role: r, .. }
                    if id == unit_id && *operand_position == binding.0 && *r == role), work)?;
            work.charge_amount(application, &term.dimension,
                u64::try_from(term.exponent.len()).unwrap_or(u64::MAX))?;
            let exponent = term.exponent.parse::<Integer>().map_err(|_| read_error(
                AdmittedEqualityReadErrorCode::MismatchedEqualityWitness, application,
                &term.dimension, work.consumed, 0))?;
            Ok((fact.structural_key.clone(), exponent))
        }).collect::<Result<Vec<_>, AdmittedEqualityReadError>>()
    };
    let form = match &unit.form {
        CheckedResolvedEqualityUnitForm::Declared {
            dimension_node,
            dimension_terms,
            root_unit,
            path,
        } => {
            if unit_id.declared_key() != Some(&row.structural_key) {
                return Err(read_error(
                    AdmittedEqualityReadErrorCode::MismatchedEqualityWitness,
                    application,
                    &unit.unit,
                    work.consumed,
                    0,
                ));
            }
            let dimension = selected_fact(
                package,
                application,
                application_row,
                rows,
                dimension_node,
                EqualityWitnessKind::DimensionDeclaration,
                |fact| {
                    matches!(fact, EqualityFactWitness::DimensionDeclaration { unit_id: id, operand_position, role: EqualityDimensionRole::UnitDimension, .. }
                    if id == unit_id && *operand_position == binding.0)
                },
                work,
            )?;
            let dimension_key = dimension.structural_key.clone();
            let dimension_terms =
                dimensions(dimension_terms, EqualityDimensionRole::BaseDimension, work)?;
            let mut edges = Vec::new();
            let mut source = unit.unit.clone();
            for (ordinal, edge) in path.iter().enumerate() {
                let fact = selected_fact(
                    package,
                    application,
                    application_row,
                    rows,
                    &source,
                    EqualityWitnessKind::ConversionEdgeTarget,
                    |fact| {
                        matches!(fact, EqualityFactWitness::ConversionEdge { unit_id: id, operand_position, ordinal: n, .. }
                        if id == unit_id && *operand_position == binding.0 && *n == ordinal)
                    },
                    work,
                )?;
                let EqualityFactWitness::ConversionEdge {
                    target,
                    target_node,
                    scale,
                    offset,
                    ..
                } = &fact.fact
                else {
                    return Err(read_error(
                        AdmittedEqualityReadErrorCode::MismatchedEqualityWitness,
                        application,
                        &fact.structural_node,
                        work.consumed,
                        0,
                    ));
                };
                for rational in [&edge.scale, &edge.offset] {
                    work.charge_amount(
                        application,
                        &fact.structural_node,
                        u64::try_from(
                            rational
                                .numerator
                                .len()
                                .saturating_add(rational.denominator.len()),
                        )
                        .unwrap_or(u64::MAX),
                    )?;
                }
                if target_node != &edge.target || scale != &edge.scale || offset != &edge.offset {
                    return Err(read_error(
                        AdmittedEqualityReadErrorCode::MismatchedEqualityWitness,
                        application,
                        &fact.structural_node,
                        work.consumed,
                        0,
                    ));
                }
                if matches!(
                    package.node(&source).and_then(|node| node.owner.as_ref()),
                    Some(CheckedNodeOwner::Source { .. })
                ) {
                    let Some(CheckedTerm::NominalBody(body)) = package.term(&source) else {
                        return Err(read_error(
                            AdmittedEqualityReadErrorCode::MismatchedEqualityWitness,
                            application,
                            &source,
                            work.consumed,
                            0,
                        ));
                    };
                    let Some((_, selected_target, selected_scale, selected_offset)) =
                        body.unit_details()
                    else {
                        return Err(read_error(
                            AdmittedEqualityReadErrorCode::MismatchedEqualityWitness,
                            application,
                            &source,
                            work.consumed,
                            0,
                        ));
                    };
                    if selected_target != Some(&edge.target)
                        || selected_scale != &edge.scale
                        || selected_offset != &edge.offset
                    {
                        return Err(read_error(
                            AdmittedEqualityReadErrorCode::MismatchedEqualityWitness,
                            application,
                            &source,
                            work.consumed,
                            0,
                        ));
                    }
                }
                if matches!(
                    package
                        .node(&edge.target)
                        .and_then(|node| node.owner.as_ref()),
                    Some(CheckedNodeOwner::Source { .. })
                ) {
                    let Some(CheckedTerm::NominalBody(body)) = package.term(&edge.target) else {
                        return Err(read_error(
                            AdmittedEqualityReadErrorCode::MismatchedEqualityWitness,
                            application,
                            &edge.target,
                            work.consumed,
                            0,
                        ));
                    };
                    let Some((target_dimension, _, _, _)) = body.unit_details() else {
                        return Err(read_error(
                            AdmittedEqualityReadErrorCode::MismatchedEqualityWitness,
                            application,
                            &edge.target,
                            work.consumed,
                            0,
                        ));
                    };
                    selected_fact(
                        package,
                        application,
                        application_row,
                        rows,
                        target_dimension,
                        EqualityWitnessKind::DimensionDeclaration,
                        |fact| {
                            matches!(fact, EqualityFactWitness::DimensionDeclaration {
                            unit_id: id, operand_position, role: EqualityDimensionRole::ConversionDimension, .. }
                            if id == unit_id && *operand_position == binding.0)
                        },
                        work,
                    )?;
                }
                edges.push(EqualityUnitEdgeDescriptor {
                    target: target.clone(),
                    scale: checked_rational(scale, application, &fact.structural_node, work)?,
                    offset: checked_rational(offset, application, &fact.structural_node, work)?,
                });
                source = edge.target.clone();
            }
            if source != *root_unit {
                return Err(read_error(
                    AdmittedEqualityReadErrorCode::MismatchedEqualityWitness,
                    application,
                    root_unit,
                    work.consumed,
                    0,
                ));
            }
            EqualityUnitForm::Declared {
                dimension_node: dimension_key,
                dimension_terms,
                path: edges,
            }
        }
        CheckedResolvedEqualityUnitForm::Compound {
            dimension_terms,
            terms,
        } => {
            let Some(root_terms) = unit_id.compound_terms() else {
                return Err(read_error(
                    AdmittedEqualityReadErrorCode::MismatchedEqualityWitness,
                    application,
                    &unit.unit,
                    work.consumed,
                    0,
                ));
            };
            if root_terms.len() != terms.len() {
                return Err(read_error(
                    AdmittedEqualityReadErrorCode::MismatchedEqualityWitness,
                    application,
                    &unit.unit,
                    work.consumed,
                    0,
                ));
            }
            for ((root_key, exponent), term) in root_terms.iter().zip(terms) {
                join_source_unit(
                    package,
                    application,
                    &term.root_unit,
                    &UnitId::declared(root_key.clone()),
                    source_rows,
                    work,
                )?;
                work.charge_amount(
                    application,
                    &term.root_unit,
                    u64::try_from(term.exponent.len()).unwrap_or(u64::MAX),
                )?;
                if &term.exponent.parse::<Integer>().map_err(|_| {
                    read_error(
                        AdmittedEqualityReadErrorCode::MismatchedEqualityWitness,
                        application,
                        &term.root_unit,
                        work.consumed,
                        0,
                    )
                })? != exponent
                {
                    return Err(read_error(
                        AdmittedEqualityReadErrorCode::MismatchedEqualityWitness,
                        application,
                        &term.root_unit,
                        work.consumed,
                        0,
                    ));
                }
            }
            EqualityUnitForm::Compound {
                dimension_terms: dimensions(
                    dimension_terms,
                    EqualityDimensionRole::CompoundBase,
                    work,
                )?,
            }
        }
    };
    validate_dimension_terms(
        package,
        application,
        application_row,
        unit_id,
        binding.0,
        rows,
        work,
    )?;
    Ok(EqualityUnitDescriptor {
        id: unit_id.clone(),
        form,
    })
}

fn enum_descriptors(
    package: &CheckedPackageV2,
    application: &CheckedNodeId,
    application_row: &SelectedEqualityApplicationRow,
    decision: &super::CheckedEqualityDecision,
    compared_type: &ValueType,
    rows: &[SelectedEqualityFactRow],
    work: &mut ReadWork,
) -> Result<
    (
        Vec<EqualityEnumDeclarationDescriptor>,
        Vec<EqualityEnumMemberDescriptor>,
    ),
    AdmittedEqualityReadError,
> {
    let mut declarations: Vec<EqualityEnumDeclarationDescriptor> = Vec::new();
    let mut members = Vec::new();
    let ValueType::Enum(expected_shape) = compared_type else {
        return Err(missing_witness(
            application,
            &decision.left.source_type,
            EqualityWitnessKind::EnumDeclaration,
            work,
        ));
    };
    let mut expected_variants = Vec::new();
    for variant in expected_shape.variants() {
        work.charge(application, &decision.left.source_type)?;
        expected_variants.push(variant);
    }
    if decision.enum_members.len() != expected_variants.len() {
        return Err(missing_witness(
            application,
            &decision.left.source_type,
            EqualityWitnessKind::EnumMember,
            work,
        ));
    }
    let compared_node = decision
        .left
        .conversion_target
        .as_ref()
        .unwrap_or(&decision.left.source_type);
    for wire in &decision.enum_members {
        let row = selected_fact(
            package,
            application,
            application_row,
            rows,
            &wire.member,
            EqualityWitnessKind::EnumMember,
            |fact| {
                matches!(fact, EqualityFactWitness::EnumMember { member_node, .. }
                if *member_node == wire.member)
            },
            work,
        )?;
        let EqualityFactWitness::EnumMember {
            source_declaration,
            source_declaration_node,
            declaration,
            declaration_node,
            member,
            member_node,
            case,
            ordinal,
            ordered,
        } = &row.fact
        else {
            return Err(read_error(
                AdmittedEqualityReadErrorCode::MismatchedEqualityWitness,
                application,
                &wire.member,
                work.consumed,
                0,
            ));
        };
        work.charge_amount(
            application,
            &wire.member,
            u64::try_from(case.len()).unwrap_or(u64::MAX),
        )?;
        if &wire.enum_type != compared_node
            || source_declaration_node != &wire.enum_type
            || member_node != &wire.member
            || member != &row.structural_key
            || case != &wire.case
            || u64::try_from(*ordinal).ok() != Some(wire.ordinal)
            || ordered != &wire.ordered
            || !matches!(
                package.node_kind(source_declaration_node),
                Some(
                    CheckedNodeKind::ScalarType(ScalarTypeForm::Enum)
                        | CheckedNodeKind::Model(ModelForm::VariantType)
                )
            )
            || !matches!(
                package.node_kind(declaration_node),
                Some(
                    CheckedNodeKind::ScalarType(ScalarTypeForm::Enum)
                        | CheckedNodeKind::Model(ModelForm::VariantType)
                )
            )
            || package.node_kind(member_node) != Some(CheckedNodeKind::Value(ValueForm::EnumValue))
            || package
                .node(member_node)
                .is_none_or(|node| node.semantic_type != *declaration_node)
        {
            return Err(read_error(
                AdmittedEqualityReadErrorCode::MismatchedEqualityWitness,
                application,
                &wire.member,
                work.consumed,
                0,
            ));
        }
        let Some(ValueType::Enum(shape)) = row.value_type.as_ref() else {
            return Err(missing_witness(
                application,
                &wire.member,
                EqualityWitnessKind::EnumDeclaration,
                work,
            ));
        };
        let variant =
            VariantId::new(source_declaration.clone(), case.to_string()).map_err(|_| {
                read_error(
                    AdmittedEqualityReadErrorCode::MismatchedEqualityWitness,
                    application,
                    &wire.member,
                    work.consumed,
                    0,
                )
            })?;
        work.charge_amount(
            application,
            &wire.member,
            u64::try_from(expected_variants.len()).unwrap_or(u64::MAX),
        )?;
        if shape != expected_shape
            || shape.is_ordered() != *ordered
            || expected_variants
                .get(*ordinal)
                .is_none_or(|expected| *expected != &variant)
        {
            return Err(read_error(
                AdmittedEqualityReadErrorCode::MismatchedEqualityWitness,
                application,
                &wire.member,
                work.consumed,
                0,
            ));
        }
        work.charge_amount(
            application,
            &wire.member,
            u64::try_from(declarations.len()).unwrap_or(u64::MAX),
        )?;
        let declaration_index = declarations.iter().position(|row| row.key == *declaration);
        if let Some(index) = declaration_index {
            let declaration_row = &mut declarations[index];
            if declaration_row.ordered != *ordered || declaration_row.members.len() != *ordinal {
                return Err(read_error(
                    AdmittedEqualityReadErrorCode::MismatchedEqualityWitness,
                    application,
                    &wire.member,
                    work.consumed,
                    0,
                ));
            }
            declaration_row.members.push(case.to_string());
        } else {
            if *ordinal != 0 {
                return Err(read_error(
                    AdmittedEqualityReadErrorCode::MismatchedEqualityWitness,
                    application,
                    &wire.member,
                    work.consumed,
                    0,
                ));
            }
            declarations.push(EqualityEnumDeclarationDescriptor {
                key: declaration.clone(),
                ordered: *ordered,
                members: vec![case.to_string()],
            });
        }
        members.push(EqualityEnumMemberDescriptor {
            variant,
            declaration: declaration.clone(),
            member: member.clone(),
            ordered: *ordered,
            position: *ordinal,
            case: case.clone(),
        });
    }
    Ok((declarations, members))
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
        fact_rows: &[SelectedEqualityFactRow],
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
        let units = decision
            .units
            .iter()
            .map(|unit| {
                quantity_unit(
                    self,
                    application,
                    row,
                    decision,
                    unit,
                    fact_rows,
                    source_unit_rows,
                    &mut work,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        work.charge(application, &decision.left.source_type)?;
        let left = operand(
            self,
            application,
            row,
            decision,
            fact_rows,
            &decision.left,
            0,
            &mut work,
        )?;
        work.charge(application, &decision.right.source_type)?;
        let right = operand(
            self,
            application,
            row,
            decision,
            fact_rows,
            &decision.right,
            1,
            &mut work,
        )?;
        let schedule = match decision.schedule {
            super::CheckedEqualitySchedule::Text => EqualitySchedule::Text,
            super::CheckedEqualitySchedule::Enum => EqualitySchedule::Enum,
            super::CheckedEqualitySchedule::Quantity => EqualitySchedule::Quantity,
            super::CheckedEqualitySchedule::Plan => EqualitySchedule::Plan,
        };
        let (enum_declarations, enum_members) = if schedule == EqualitySchedule::Enum {
            let compared = left.target().unwrap_or_else(|| left.source());
            if right.target().unwrap_or_else(|| right.source()) != compared {
                return Err(read_error(
                    AdmittedEqualityReadErrorCode::MismatchedEqualityWitness,
                    application,
                    &decision.right.source_type,
                    work.consumed,
                    0,
                ));
            }
            enum_descriptors(
                self,
                application,
                row,
                decision,
                compared,
                fact_rows,
                &mut work,
            )?
        } else {
            (Vec::new(), Vec::new())
        };
        let descriptor = EqualityDecisionDescriptor {
            location: row.location.clone(),
            operator: match decision.operator {
                CheckedEqualityOperator::Equal => EqualityOperator::Equal,
                CheckedEqualityOperator::NotEqual => EqualityOperator::NotEqual,
            },
            left,
            right,
            schedule,
            units,
            enum_declarations,
            enum_members,
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
