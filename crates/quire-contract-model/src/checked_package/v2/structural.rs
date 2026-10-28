//! Graph-shape rules for node bodies that carry no nominal preimage: the
//! `value`/`parameter` body (QSpec FR-341, QSL FR-092), the
//! `scalar_type`/`compound_unit` body QSL emits beyond QSpec FR-322's
//! published form list, the `state`/`operation_anchor` (QSpec FR-342) and
//! `state`/`state_clause` (QSpec FR-341) bodies, the occurrence role the
//! schema's `BodyBindingRules` fix for the frame, anchor, clause and
//! parameter forms, and FR-322's dependency join for every node whose body
//! holds an `application`.
//!
//! - **Parameter** (QSpec FR-341, QSL FR-092 "Parameter nodes"): a value an
//!   enclosing binder binds. Its body is exactly `aggregate{[binding "name"
//!   = an identifier text literal typed at a `scalar_type`/`text` node,
//!   binding "level" = a canonical non-negative integer literal typed at a
//!   `scalar_type`/`integer` node]}`; its `semantic_type` is the binder's
//!   type node (a `scalar_type`, `composite_type`, `bounded_domain`, or
//!   selected model value declaration node, never itself); its body names
//!   no node, so `dependencies` is empty. Every
//!   occurrence has role `expression`.
//! - **Operation anchor** (QSpec FR-342): `aggregate{[binding "context" = a
//!   reference, binding "operation" = an identifier text literal, binding
//!   "frame" = a reference]}`. Every occurrence has role `anchor`.
//! - **State clause** (QSpec FR-341): one `state_clause` application of
//!   `quire.op.state.clause` whose member is exactly `{kind: "state_clause",
//!   clause}` with a closed clause kind, and whose three arguments are an
//!   `aggregate` of one or more `reference` terms, a `reference` and the
//!   condition. Every occurrence has role `claim`.
//! - **Union type and union value** (QSpec FR-440): a `composite_type`/`union`
//!   body is an `aggregate` of one or more `binding`s, each named by an
//!   identifier and valued by an `aggregate` of `reference` terms (its payload
//!   types in position order); a `value`/`union_value` body is an `aggregate`
//!   of exactly one such `binding` over its payload terms. The joins of both
//!   to their union (`duplicate-member`, `type-mismatch`) are the operation
//!   step's.
//! - **Frame** (QSpec FR-340): its body is the frame body the per-node loop
//!   already read; every occurrence has role `generated`.
//! - **Compound unit** (QSL FR-094 "Compound unit"): the anonymous type of
//!   a quantity whose unit is a product, quotient or power. It is its own
//!   semantic type; its body is an `aggregate` of terms, each exactly
//!   `aggregate{[binding "unit" = reference(a root `scalar_type`/`unit`
//!   node), binding "exponent" = a canonical integer literal typed
//!   at a `scalar_type`/`integer` node]}`, strictly ascending by unit key
//!   (the `quire.value.compound-unit/v1` term order); `dependencies` is
//!   exactly those unit keys in that order. The empty body is the
//!   dimensionless unit. Zero exponent is a derived-shape semantic defect
//!   after all structural keys have been checked.
//! - **Application-node dependency join** (FR-322
//!   `application_node_preimage`): a node whose body contains an
//!   `application` term anywhere has `dependencies` exactly the unique,
//!   digest-ascending reference targets and operation member declarations
//!   of its body. `result_type` and literal `type` annotations are not
//!   dependencies.
//!
//! This shape stage does not re-derive keys. A missing target that equals a
//! rekeyed node's canonical key resolves to that node for shape checks; its
//! dependent absence waits for the later key stage. Independent malformed
//! shape still refuses here. `derived_keys` checks owner-free,
//! ungrouped structural nodes against QSL's `quire.structural-node/v1`
//! preimage after application keys and before nominal keys; IR-630 owns the
//! owner-bearing and grouped follow-up (FR-038-AC-123).
//!
//! Every violation refuses as `invalid_semantic_graph` with no cause, at the
//! member it breaks (the node's `body` for a body or binding shape, the
//! occurrence's `role` for a role), located at the offending node, at the
//! first such node in ascending node-id digest order.
//! These are graph-shape refusals, so they run before the stale-key stage.
//! The stage charges no work of its own: every body term it walks was
//! parsed, shape-validated and charged once by the per-node body loop.

use super::derived_keys::DeferredKeyTargets;
use super::identity::is_identifier;
use super::{
    ApplicationOperator, CheckedTerm, CheckedTermView, ExecutionPoint, LiteralKind,
    OperationMemberKind, StateClauseKind,
};
use super::{
    BoundedDomainForm, CheckedNodeKind, CheckedNodeTag, ClaimForm, CompositeTypeForm,
    CorrespondenceForm, ExpressionForm, FunctionForm, ModelForm, NominalIdentityPreimage,
    ProtocolForm, RawSemanticNodeV2 as CheckedSemanticNodeV2, RelationForm, ScalarTypeForm,
    StateForm, TemporalForm, ValueForm,
};
use crate::checked_package::common::{exact_members, is_digest, node_pointer, ValidationFailure};
use crate::checked_package::shared::{
    CheckedNodeId, CheckedOccurrenceRole, CheckedPackageRefusalCode, JsonPointer,
};
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// The member of one node a structural defect is about.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Defect {
    Node,
    SemanticType,
    Dependencies,
    RecursionGroup,
    NominalIdentityPreimage,
    Occurrences,
    /// The node body itself.
    Body,
    /// A malformed state-clause execution point.
    ClausePoint,
    /// One member of the node body's `aggregate`.
    BodyMember(usize),
    /// The role of the occurrence at this index.
    Role(usize),
}

impl Defect {
    fn pointer(self, position: usize) -> JsonPointer {
        let node = node_pointer(position);
        match self {
            Self::Node => node,
            Self::SemanticType => node.key("semantic_type"),
            Self::Dependencies => node.key("dependencies"),
            Self::RecursionGroup => node.key("recursion_group"),
            Self::NominalIdentityPreimage => node.key("nominal_identity_preimage"),
            Self::Occurrences => node.key("occurrences"),
            Self::Body => node.key("body"),
            Self::ClausePoint => node
                .key("body")
                .key("operation")
                .key("member")
                .key("execution_point"),
            Self::BodyMember(member) => node.key("body").key("members").index(member),
            Self::Role(occurrence) => node.key("occurrences").index(occurrence).key("role"),
        }
    }
}

/// The graph one stage reads: each node with its decoded kind, by key.
struct Graph<'a> {
    nodes: &'a [CheckedSemanticNodeV2],
    kinds: &'a [CheckedNodeKind],
    index: &'a BTreeMap<&'a CheckedNodeId, usize>,
    deferred: &'a DeferredKeyTargets,
}

impl Graph<'_> {
    fn node(&self, id: &CheckedNodeId) -> Option<(&CheckedSemanticNodeV2, CheckedNodeKind)> {
        let position = self.deferred.position(id, self.index)?;
        Some((self.nodes.get(position)?, *self.kinds.get(position)?))
    }

    fn kind(&self, id: &CheckedNodeId) -> Option<CheckedNodeKind> {
        self.node(id).map(|(_, kind)| kind)
    }

    fn dependent_target(&self, id: &CheckedNodeId) -> bool {
        self.deferred.ambiguous(id, self.index)
    }

    fn type_matches_or_deferred(&self, id: &CheckedNodeId, form: ScalarTypeForm) -> bool {
        self.kind(id) == Some(CheckedNodeKind::ScalarType(form)) || self.dependent_target(id)
    }
}

/// Which body rule a node's kind selects.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StructuralForm {
    Extension,
    Hybrid,
    TimedModel,
    Parameter,
    CompoundUnit,
    Frame,
    OperationAnchor,
    StateClause,
    StateTransition,
    UnionType,
    UnionValue,
    ProtocolReference,
    AbstractionRelation,
}

impl StructuralForm {
    /// The one role every occurrence of a node of this form carries, when
    /// the schema fixes one.
    const fn role(self) -> Option<CheckedOccurrenceRole> {
        match self {
            Self::Extension => Some(CheckedOccurrenceRole::Declaration),
            Self::Parameter => Some(CheckedOccurrenceRole::Expression),
            Self::Frame => Some(CheckedOccurrenceRole::Generated),
            Self::OperationAnchor => Some(CheckedOccurrenceRole::Anchor),
            Self::StateClause | Self::StateTransition => Some(CheckedOccurrenceRole::Claim),
            Self::AbstractionRelation => Some(CheckedOccurrenceRole::Declaration),
            Self::ProtocolReference => Some(CheckedOccurrenceRole::Expression),
            Self::Hybrid
            | Self::TimedModel
            | Self::CompoundUnit
            | Self::UnionType
            | Self::UnionValue => None,
        }
    }

    /// Every form is listed, so a new form is a compile error here until
    /// its body rule is decided.
    fn of(kind: CheckedNodeKind) -> Option<Self> {
        use CheckedNodeKind as K;
        match kind {
            K::Extension(_) => Some(Self::Extension),
            K::Hybrid(_) => Some(Self::Hybrid),
            K::TimedModel(_) => Some(Self::TimedModel),
            K::Value(ValueForm::Parameter) => Some(Self::Parameter),
            K::ScalarType(ScalarTypeForm::CompoundUnit) => Some(Self::CompoundUnit),
            K::ScalarType(
                ScalarTypeForm::Boolean
                | ScalarTypeForm::Integer
                | ScalarTypeForm::Rational
                | ScalarTypeForm::Decimal
                | ScalarTypeForm::Float32
                | ScalarTypeForm::Float64
                | ScalarTypeForm::Text
                | ScalarTypeForm::String
                | ScalarTypeForm::Uuid
                | ScalarTypeForm::Timestamp
                | ScalarTypeForm::Dimension
                | ScalarTypeForm::DimensionEquivalence
                | ScalarTypeForm::Unit
                | ScalarTypeForm::Enum,
            ) => None,
            K::CompositeType(CompositeTypeForm::Union) => Some(Self::UnionType),
            K::CompositeType(
                CompositeTypeForm::Option
                | CompositeTypeForm::Sequence
                | CompositeTypeForm::Set
                | CompositeTypeForm::Bag
                | CompositeTypeForm::OrderedSet
                | CompositeTypeForm::Record
                | CompositeTypeForm::Tuple
                | CompositeTypeForm::Alias
                | CompositeTypeForm::Reference,
            ) => None,
            K::BoundedDomain(
                BoundedDomainForm::IntegerRange
                | BoundedDomainForm::RationalRange
                | BoundedDomainForm::DecimalRange
                | BoundedDomainForm::FloatRounding
                | BoundedDomainForm::TextBounds
                | BoundedDomainForm::CollectionBounds
                | BoundedDomainForm::ModelPopulation,
            ) => None,
            K::Value(ValueForm::UnionValue) => Some(Self::UnionValue),
            K::Value(
                ValueForm::Literal
                | ValueForm::EnumValue
                | ValueForm::CollectionValue
                | ValueForm::RecordValue
                | ValueForm::TupleValue
                | ValueForm::OptionValue,
            ) => None,
            K::Expression(
                ExpressionForm::Reference
                | ExpressionForm::Call
                | ExpressionForm::Unary
                | ExpressionForm::Binary
                | ExpressionForm::Conditional
                | ExpressionForm::Case
                | ExpressionForm::Let
                | ExpressionForm::Quantify
                | ExpressionForm::Collection
                | ExpressionForm::Conversion
                | ExpressionForm::Query
                | ExpressionForm::PreRead
                | ExpressionForm::PresenceRead
                | ExpressionForm::ValueRead
                | ExpressionForm::Deref
                | ExpressionForm::Reachability,
            ) => None,
            K::Function(
                FunctionForm::PureFunction
                | FunctionForm::Predicate
                | FunctionForm::RecursiveFunction,
            ) => None,
            K::Model(
                ModelForm::ModelRoot
                | ModelForm::ModelImport
                | ModelForm::ObjectType
                | ModelForm::ValueType
                | ModelForm::VariantType
                | ModelForm::TupleValueType
                | ModelForm::RecordValueType
                | ModelForm::EventType
                | ModelForm::StateMachine
                | ModelForm::Process
                | ModelForm::PersistenceInterface
                | ModelForm::Namespace
                | ModelForm::SystemsInterface
                | ModelForm::SystemsPart
                | ModelForm::SystemsPort
                | ModelForm::SystemsConnection
                | ModelForm::SystemsAllocation,
            ) => None,
            K::Relation(
                RelationForm::Relationship
                | RelationForm::Population
                | RelationForm::Membership
                | RelationForm::CausalRelation,
            ) => None,
            K::Relation(RelationForm::Refinement) => None,
            K::State(StateForm::Frame) => Some(Self::Frame),
            K::State(StateForm::OperationAnchor) => Some(Self::OperationAnchor),
            K::State(StateForm::StateClause) => Some(Self::StateClause),
            K::State(StateForm::Transition) => Some(Self::StateTransition),
            K::State(StateForm::Snapshot) => None,
            K::Temporal(
                TemporalForm::TemporalClause
                | TemporalForm::Formula
                | TemporalForm::Fairness
                | TemporalForm::Clock
                | TemporalForm::Window
                | TemporalForm::Activation
                | TemporalForm::Deadline
                | TemporalForm::Terminal,
            ) => None,
            K::Protocol(
                ProtocolForm::ProtocolClause
                | ProtocolForm::Role
                | ProtocolForm::Channel
                | ProtocolForm::Queue
                | ProtocolForm::Control
                | ProtocolForm::Obligation
                | ProtocolForm::Compensation,
            ) => None,
            K::Protocol(ProtocolForm::ProtocolReference) => Some(Self::ProtocolReference),
            K::Claim(
                ClaimForm::VerificationClaim
                | ClaimForm::AnalysisClaim
                | ClaimForm::Hyperproperty
                | ClaimForm::SynthesisRequest
                | ClaimForm::StateGraph
                | ClaimForm::StepRelation,
            ) => None,
            K::Correspondence(CorrespondenceForm::AbstractionRelation) => {
                Some(Self::AbstractionRelation)
            }
            K::Correspondence(
                CorrespondenceForm::SourceLocus
                | CorrespondenceForm::ModelCorrespondence
                | CorrespondenceForm::BindingRole
                | CorrespondenceForm::ProfileCorrespondence,
            ) => None,
        }
    }
}

/// Validates every parameter and compound-unit node's body, type and
/// dependencies, and every application node's dependency join, visiting
/// nodes in ascending node-id digest order.
pub(super) fn validate_structural_nodes(
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    terms: &[Option<super::CheckedTerm>],
    index: &BTreeMap<&CheckedNodeId, usize>,
    deferred: &DeferredKeyTargets,
) -> Result<(), ValidationFailure> {
    let graph = Graph {
        nodes,
        kinds,
        index,
        deferred,
    };
    for (&node_id, &position) in index {
        let (Some(node), Some(&kind)) = (nodes.get(position), kinds.get(position)) else {
            continue;
        };
        let refuse = |defect: Defect| {
            ValidationFailure::refused_at(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                defect.pointer(position),
                None,
                node_id.clone(),
            )
        };
        let form = StructuralForm::of(kind);
        let term = terms.get(position).and_then(Option::as_ref);
        let defect = match form {
            Some(StructuralForm::Extension) => {
                let Some(super::CheckedTerm::ExtensionForm { clauses, .. }) = term else {
                    return Err(ValidationFailure::refused(
                        CheckedPackageRefusalCode::MalformedWire,
                        node_pointer(position).key("body"),
                    ));
                };
                if clauses.iter().any(|clause| {
                    !matches!(clause, super::CheckedTerm::Binding { value, .. }
                        if matches!(value.as_ref(), super::CheckedTerm::Reference { .. }))
                }) {
                    return Err(ValidationFailure::refused(
                        CheckedPackageRefusalCode::MalformedWire,
                        node_pointer(position).key("body"),
                    ));
                }
                let expected = clauses
                    .iter()
                    .filter_map(|clause| match clause {
                        super::CheckedTerm::Binding { value, .. } => match value.as_ref() {
                            super::CheckedTerm::Reference { target } => Some(target),
                            _ => None,
                        },
                        _ => None,
                    })
                    .collect::<BTreeSet<_>>();
                if node.semantic_type != node.node_id {
                    Some(Defect::SemanticType)
                } else if node.recursion_group.is_some() {
                    Some(Defect::RecursionGroup)
                } else if node.nominal_identity_preimage.is_some() {
                    Some(Defect::NominalIdentityPreimage)
                } else if !node.dependencies.iter().eq(expected) {
                    Some(Defect::Dependencies)
                } else {
                    None
                }
            }
            Some(StructuralForm::Hybrid | StructuralForm::TimedModel) => {
                let Some(term) = term else {
                    return Err(ValidationFailure::refused(
                        CheckedPackageRefusalCode::MalformedWire,
                        node_pointer(position).key("body"),
                    ));
                };
                let mut pending = vec![term];
                let mut targets = BTreeSet::new();
                while let Some(next) = pending.pop() {
                    match next {
                        super::CheckedTerm::Reference { target } => {
                            targets.insert(target);
                        }
                        super::CheckedTerm::Aggregate { members } => pending.extend(members),
                        super::CheckedTerm::Binding { value, .. } => pending.push(value),
                        super::CheckedTerm::Literal { .. } => {}
                        _ => return Err(refuse(Defect::Body)),
                    }
                }
                if node.semantic_type != node.node_id {
                    Some(Defect::SemanticType)
                } else if node.recursion_group.is_some() {
                    Some(Defect::RecursionGroup)
                } else if node.nominal_identity_preimage.is_some() {
                    Some(Defect::NominalIdentityPreimage)
                } else if !node.dependencies.iter().eq(targets) {
                    Some(Defect::Dependencies)
                } else {
                    None
                }
            }
            Some(StructuralForm::Parameter) => parameter_defect(node, term, &graph),
            Some(StructuralForm::CompoundUnit) => compound_unit_defect(node, term, &graph),
            Some(StructuralForm::OperationAnchor) => term
                .and_then(typed_anchor_body)
                .is_none()
                .then_some(Defect::Body),
            Some(StructuralForm::StateClause) => {
                if node
                    .body
                    .pointer("/operation/member/execution_point")
                    .is_some_and(|point| ExecutionPoint::from_wire(point).is_none())
                {
                    Some(Defect::ClausePoint)
                } else {
                    term.and_then(typed_clause_body)
                        .is_none()
                        .then_some(Defect::Body)
                }
            }
            Some(StructuralForm::StateTransition) => term
                .and_then(typed_transition_body)
                .is_none()
                .then_some(Defect::Body),
            Some(StructuralForm::UnionType) => term
                .and_then(typed_union_type_body)
                .is_none()
                .then_some(Defect::Body),
            Some(StructuralForm::UnionValue) => term
                .and_then(typed_union_value_body)
                .is_none()
                .then_some(Defect::Body),
            Some(StructuralForm::ProtocolReference) => protocol_reference_defect(node, term),
            // The per-node body loop already read the frame body; the
            // abstraction step (FR-346) reads its own, so only the role is
            // this stage's.
            Some(StructuralForm::Frame | StructuralForm::AbstractionRelation) | None => None,
        };
        let role = form.and_then(StructuralForm::role).and_then(|role| {
            node.occurrences
                .iter()
                .position(|occurrence| occurrence.role != role)
                .map(Defect::Role)
        });
        if let Some(defect) = defect.or(role) {
            return Err(refuse(defect));
        }
        if kinds.get(position) == Some(&CheckedNodeKind::Protocol(ProtocolForm::ProtocolClause)) {
            if let Some(owner_model) = node.owner_model.as_ref() {
                let ordered = node.dependencies.windows(2).all(|pair| pair[0] < pair[1]);
                if !ordered || !node.dependencies.contains(owner_model) {
                    return Err(refuse(Defect::Dependencies));
                }
                if let Some(mut expected) = term.and_then(application_dependencies) {
                    expected.insert(owner_model.clone());
                    if !node.dependencies.iter().eq(expected.iter()) {
                        return Err(refuse(Defect::Dependencies));
                    }
                }
            }
            continue;
        }
        if let Some(expected) = term.and_then(application_dependencies) {
            if !node.dependencies.iter().eq(expected.iter()) {
                return Err(refuse(Defect::Dependencies));
            }
        }
    }
    Ok(())
}

/// Carrier-only members and the local edge shape, checked before keys or target resolution.
fn protocol_reference_defect(
    node: &CheckedSemanticNodeV2,
    term: Option<&super::CheckedTerm>,
) -> Option<Defect> {
    if node.occurrences.is_empty() {
        return Some(Defect::Occurrences);
    }
    let Some(super::CheckedTerm::Reference { target }) = term else {
        return Some(Defect::Body);
    };
    if !super::is_node_domain(&target.domain) || !is_digest(&target.digest) {
        return Some(Defect::Body);
    }
    if node.dependencies.as_slice() != std::slice::from_ref(target) {
        return Some(Defect::Dependencies);
    }
    None
}

/// The first rule a `value`/`parameter` node breaks, as the member it breaks
/// it at.
fn parameter_defect(
    node: &CheckedSemanticNodeV2,
    term: Option<&super::CheckedTerm>,
    graph: &Graph<'_>,
) -> Option<Defect> {
    let is_binder_type = node.semantic_type != node.node_id
        && (graph.dependent_target(&node.semantic_type)
            || graph.node(&node.semantic_type).is_none()
            || graph
                .node(&node.semantic_type)
                .is_some_and(|(type_node, kind)| {
                    // A binder may also be typed at a selected model value declaration.
                    kind == CheckedNodeKind::Model(ModelForm::RecordValueType)
                        || kind == CheckedNodeKind::Model(ModelForm::TupleValueType)
                        || kind == CheckedNodeKind::Hybrid(super::HybridForm::Model)
                        || (kind == CheckedNodeKind::Model(ModelForm::VariantType)
                            && matches!(
                                type_node.owner.as_ref(),
                                Some(super::CheckedNodeOwner::Model { .. })
                            ))
                        || matches!(
                            kind.tag(),
                            CheckedNodeTag::ScalarType
                                | CheckedNodeTag::CompositeType
                                | CheckedNodeTag::BoundedDomain
                        )
                }));
    if !is_binder_type {
        return Some(Defect::SemanticType);
    }
    if term
        .and_then(|term| {
            typed_parameter_binding_with(term, &|id, form| graph.type_matches_or_deferred(id, form))
        })
        .is_none()
    {
        return Some(Defect::Body);
    }
    if !node.dependencies.is_empty() {
        return Some(Defect::Dependencies);
    }
    None
}

pub(super) fn typed_aggregate(term: &super::CheckedTerm) -> Option<&[super::CheckedTerm]> {
    match term {
        super::CheckedTerm::Aggregate { members } => Some(members),
        _ => None,
    }
}

pub(super) fn typed_binding<'a>(
    term: &'a super::CheckedTerm,
    name: &str,
) -> Option<&'a super::CheckedTerm> {
    match term {
        super::CheckedTerm::Binding { name: found, value } if found.as_ref() == name => Some(value),
        _ => None,
    }
}

pub(super) fn typed_reference(term: &super::CheckedTerm) -> Option<&CheckedNodeId> {
    match term {
        super::CheckedTerm::Reference { target }
            if super::is_node_domain(&target.domain) && is_digest(&target.digest) =>
        {
            Some(target)
        }
        _ => None,
    }
}

fn typed_literal<'a>(
    term: &'a super::CheckedTerm,
    value_kind: LiteralKind,
    type_form: ScalarTypeForm,
    type_matches: &impl Fn(&CheckedNodeId, ScalarTypeForm) -> bool,
) -> Option<&'a Value> {
    match term {
        super::CheckedTerm::Literal {
            type_id,
            value_kind: found,
            value,
        } if *found == value_kind && type_matches(type_id, type_form) => Some(value),
        _ => None,
    }
}

pub(super) fn typed_parameter_binding<'a>(
    body: &'a super::CheckedTerm,
    kind_of: &impl Fn(&CheckedNodeId) -> Option<CheckedNodeKind>,
) -> Option<(&'a str, &'a str)> {
    typed_parameter_binding_with(body, &|id, form| {
        kind_of(id) == Some(CheckedNodeKind::ScalarType(form))
    })
}

fn typed_parameter_binding_with<'a>(
    body: &'a super::CheckedTerm,
    type_matches: &impl Fn(&CheckedNodeId, ScalarTypeForm) -> bool,
) -> Option<(&'a str, &'a str)> {
    let [name, level] = typed_aggregate(body)? else {
        return None;
    };
    let name = typed_binding(name, "name")
        .and_then(|value| {
            typed_literal(value, LiteralKind::Text, ScalarTypeForm::Text, type_matches)
        })
        .and_then(Value::as_str)
        .filter(|name| is_identifier(name))?;
    let level = typed_binding(level, "level")
        .and_then(|value| {
            typed_literal(
                value,
                LiteralKind::Integer,
                ScalarTypeForm::Integer,
                type_matches,
            )
        })
        .and_then(Value::as_str)
        .filter(|level| is_non_negative_integer(level))?;
    Some((name, level))
}

/// The first rule a `scalar_type`/`compound_unit` node breaks, as the member
/// it breaks it at.
fn compound_unit_defect(
    node: &CheckedSemanticNodeV2,
    term: Option<&super::CheckedTerm>,
    graph: &Graph<'_>,
) -> Option<Defect> {
    if node.semantic_type != node.node_id {
        return Some(Defect::SemanticType);
    }
    let Some(terms) = term.and_then(typed_aggregate) else {
        return Some(Defect::Body);
    };
    let mut units: Vec<CheckedNodeId> = Vec::with_capacity(terms.len());
    for (index, term) in terms.iter().enumerate() {
        if let Some(unit) = typed_aggregate(term)
            .and_then(|members| members.first())
            .and_then(|member| typed_binding(member, "unit"))
            .and_then(typed_reference)
        {
            if graph.node(unit).is_none() && !graph.dependent_target(unit) {
                return Some(Defect::Node);
            }
        }
        let Some(unit) = typed_compound_unit_term(term, graph) else {
            return Some(Defect::BodyMember(index));
        };
        units.push(unit);
    }
    // Dependency membership is checked here; the order and uniqueness of
    // body terms are semantic shape rules, after structural keys.
    units.sort();
    units.dedup();
    if node.dependencies != units {
        return Some(Defect::Dependencies);
    }
    None
}

fn typed_compound_unit_term(term: &super::CheckedTerm, graph: &Graph<'_>) -> Option<CheckedNodeId> {
    let [unit, exponent] = typed_aggregate(term)? else {
        return None;
    };
    let unit = typed_binding(unit, "unit").and_then(typed_reference)?;
    // A dimension's canonical root unit has no target unit in its nominal
    // preimage; the nominal stage admits exactly one such unit.
    let is_root_unit = graph.dependent_target(unit)
        || graph.node(unit).is_some_and(|(node, kind)| {
            kind == CheckedNodeKind::ScalarType(ScalarTypeForm::Unit)
                && matches!(
                    &node.nominal_identity_preimage,
                    Some(NominalIdentityPreimage::Unit(preimage))
                        if preimage.target_unit_node_id.is_none()
                )
        });
    let exponent_ok = typed_binding(exponent, "exponent")
        .and_then(|value| {
            typed_literal(
                value,
                LiteralKind::Integer,
                ScalarTypeForm::Integer,
                &|id, form| graph.type_matches_or_deferred(id, form),
            )
        })
        .and_then(Value::as_str)
        .is_some_and(is_integer_bound);
    (is_root_unit && exponent_ok).then(|| unit.clone())
}

/// Zero exponents and noncanonical unit order are well-formed terms whose
/// semantic shape is refused only after every structural key has been checked.
pub(super) fn compound_unit_has_invalid_shape(term: Option<&super::CheckedTerm>) -> bool {
    term.and_then(typed_aggregate).is_some_and(|terms| {
        let mut previous: Option<&CheckedNodeId> = None;
        for term in terms {
            let Some([unit, exponent]) = typed_aggregate(term) else {
                continue;
            };
            let unit = typed_binding(unit, "unit").and_then(typed_reference);
            if unit.is_some_and(|unit| previous.is_some_and(|previous| previous >= unit)) {
                return true;
            }
            previous = unit;
            if typed_binding(exponent, "exponent").and_then(|value| match value {
                super::CheckedTerm::Literal { value, .. } => value.as_str(),
                _ => None,
            }) == Some("0")
            {
                return true;
            }
        }
        false
    })
}

/// A `state`/`operation_anchor` body (QSpec FR-342).
pub(super) struct AnchorBody<'a> {
    /// The object type declaring the operation.
    pub(super) context: CheckedNodeId,
    /// The operation's name.
    pub(super) operation: &'a str,
    /// The operation's frame node.
    pub(super) frame: CheckedNodeId,
    /// Optional point, retained separately from the three expression terms.
    pub(super) execution_point: Option<ExecutionPoint>,
}

/// The admitted anchor body, or `None` for any other typed shape.
pub(super) fn typed_anchor_body(body: &super::CheckedTerm) -> Option<AnchorBody<'_>> {
    let (members, execution_point) = match body {
        CheckedTerm::OperationAnchorBody(anchor) => {
            (&anchor.members[..], anchor.execution_point.clone())
        }
        _ => (typed_aggregate(body)?, None),
    };
    let [context, operation, frame] = members else {
        return None;
    };
    let operation = typed_binding(operation, "operation")?;
    let super::CheckedTerm::Literal {
        value_kind: LiteralKind::Text,
        value,
        ..
    } = operation
    else {
        return None;
    };
    Some(AnchorBody {
        context: typed_binding(context, "context")
            .and_then(typed_reference)?
            .clone(),
        operation: value.as_str().filter(|name| is_identifier(name))?,
        frame: typed_binding(frame, "frame")
            .and_then(typed_reference)?
            .clone(),
        execution_point,
    })
}

/// The catalogued identity of the state clause operation.
/// A `state`/`state_clause` body (QSpec FR-341).
pub(super) struct ClauseBody {
    /// The clause kind its `state_clause` member names.
    pub(super) clause: StateClauseKind,
    /// The parameter aggregate's reference targets, in order.
    pub(super) parameters: Vec<CheckedNodeId>,
    /// The anchor reference's target.
    pub(super) anchor: CheckedNodeId,
    /// Optional closed point of the clause application member.
    pub(super) execution_point: Option<ExecutionPoint>,
}

/// The five identity-bearing references of a Boolean state transition.
pub(super) struct TransitionBody {
    pub(super) anchor: CheckedNodeId,
    pub(super) subject: CheckedNodeId,
    pub(super) inputs: Vec<CheckedNodeId>,
    pub(super) pre: Option<CheckedNodeId>,
    pub(super) post: CheckedNodeId,
}

/// Whether a raw term is the catalogued state-transition application.
pub(super) fn is_state_transition_application(term: &Value) -> bool {
    matches!(CheckedTerm::view(term), Some(CheckedTermView::Application { operation, .. })
        if operation.get("identity").and_then(Value::as_str).and_then(|identity| super::operation_catalog::operation_catalog().known(identity)) == Some(super::KnownOperation::StateTransition))
}

/// Decode the closed transition argument shape without doing its semantic joins.
pub(super) fn typed_transition_body(body: &super::CheckedTerm) -> Option<TransitionBody> {
    let super::CheckedTerm::Application {
        operator: ApplicationOperator::StateTransition,
        operation,
        arguments,
        ..
    } = body
    else {
        return None;
    };
    if super::operation_catalog::operation_catalog().known(operation.get("identity")?.as_str()?)
        != Some(super::KnownOperation::StateTransition)
    {
        return None;
    }
    let [anchor, subject, inputs, pre, post] = arguments.as_slice() else {
        return None;
    };
    let anchor = typed_binding(anchor, "anchor")
        .and_then(typed_reference)?
        .clone();
    let subject = typed_binding(subject, "subject")
        .and_then(typed_reference)?
        .clone();
    let inputs = typed_aggregate(typed_binding(inputs, "inputs")?)?
        .iter()
        .map(|term| typed_reference(term).cloned())
        .collect::<Option<Vec<_>>>()?;
    let pre = typed_aggregate(typed_binding(pre, "pre")?)?;
    let pre = match pre {
        [] => None,
        [one] => Some(typed_reference(one)?.clone()),
        _ => return None,
    };
    let post = typed_binding(post, "post")
        .and_then(typed_reference)?
        .clone();
    Some(TransitionBody {
        anchor,
        subject,
        inputs,
        pre,
        post,
    })
}

/// Whether `term` is an application of `quire.op.state.clause`.
// Reads an application's operation identity.
pub(super) fn is_state_clause_application(term: &Value) -> bool {
    matches!(CheckedTerm::view(term), Some(CheckedTermView::Application { operation, .. })
        if operation.get("identity").and_then(Value::as_str).and_then(|identity| super::operation_catalog::operation_catalog().known(identity)) == Some(super::KnownOperation::StateClause))
}

/// The admitted state clause body, or `None` for any other shape: an
/// application of `quire.op.state.clause` with operator class
/// `state_clause`, member exactly `{kind: "state_clause", clause}`, and
/// arguments an `aggregate` of one or more `reference` terms, a `reference`
/// and the condition.
// Decodes a state clause member's kind and clause.
pub(super) fn typed_clause_body(body: &super::CheckedTerm) -> Option<ClauseBody> {
    let super::CheckedTerm::Application {
        operator: ApplicationOperator::StateClause,
        operation,
        arguments,
        ..
    } = body
    else {
        return None;
    };
    if super::operation_catalog::operation_catalog().known(operation.get("identity")?.as_str()?)
        != Some(super::KnownOperation::StateClause)
    {
        return None;
    }
    let member = operation.get("member")?.as_object()?;
    if !exact_members(member, &["kind", "clause"]) {
        return None;
    }
    if member
        .get("kind")
        .and_then(Value::as_str)
        .and_then(OperationMemberKind::from_wire)
        != Some(OperationMemberKind::StateClause)
    {
        return None;
    }
    let execution_point = operation.execution_point().cloned();
    let clause = StateClauseKind::from_wire(member.get("clause")?.as_str()?)?;
    let [parameters, anchor, _condition] = arguments.as_slice() else {
        return None;
    };
    let parameters = typed_aggregate(parameters)?
        .iter()
        .map(typed_reference)
        .collect::<Option<Vec<_>>>()?
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();
    if parameters.is_empty() {
        return None;
    }
    Some(ClauseBody {
        clause,
        parameters,
        anchor: typed_reference(anchor)?.clone(),
        execution_point,
    })
}

/// A `composite_type`/`reference` type node's target: its body is exactly
/// `aggregate{[reference(target)]}`.
pub(super) fn reference_type_target(node: &CheckedSemanticNodeV2) -> Option<CheckedNodeId> {
    let Some([target]) = aggregate_members(&node.body) else {
        return None;
    };
    reference_target(target)
}

pub(super) fn reference_type_target_typed(
    node: &super::CheckedSemanticNodeV2,
) -> Option<CheckedNodeId> {
    let super::CheckedTerm::Aggregate { members } = &node.body else {
        return None;
    };
    let [super::CheckedTerm::Reference { target }] = members.as_slice() else {
        return None;
    };
    Some(target.clone())
}

/// One member of a `composite_type`/`union` node (QSpec FR-440): its name and
/// its payload type nodes in position order.
pub(super) struct UnionMember<'a> {
    pub(super) name: &'a str,
    pub(super) payload: Vec<CheckedNodeId>,
}

/// The members a `composite_type`/`union` body holds, in declaration order, or
/// `None` for any other shape: an `aggregate` of one or more `binding`s, each
/// named by an identifier and valued by an `aggregate` of `reference` terms
/// (QSpec's `UnionTypeBody`). A repeated name is not a shape defect here: it
/// is the operation step's `duplicate-member`.
pub(super) fn typed_union_type_body(body: &super::CheckedTerm) -> Option<Vec<UnionMember<'_>>> {
    let members = typed_aggregate(body)?
        .iter()
        .map(|member| {
            let super::CheckedTerm::Binding { name, value } = member else {
                return None;
            };
            if !is_identifier(name) {
                return None;
            }
            let payload = typed_aggregate(value)?
                .iter()
                .map(typed_reference)
                .collect::<Option<Vec<_>>>()?
                .into_iter()
                .cloned()
                .collect();
            Some(UnionMember { name, payload })
        })
        .collect::<Option<Vec<_>>>()?;
    (!members.is_empty()).then_some(members)
}

/// The active member and payload terms a `value`/`union_value` body holds, or
/// `None` for any other shape: an `aggregate` of exactly one `binding`, named
/// by an identifier and valued by an `aggregate` (QSpec's `UnionValueBody`).
pub(super) fn typed_union_value_body(
    body: &super::CheckedTerm,
) -> Option<(&str, &[super::CheckedTerm])> {
    let [member] = typed_aggregate(body)? else {
        return None;
    };
    let super::CheckedTerm::Binding { name, value } = member else {
        return None;
    };
    if !is_identifier(name) {
        return None;
    }
    Some((name, typed_aggregate(value)?))
}

/// FR-322's dependency join for a body that contains an `application` term
/// at any depth: `None` for a body holding none, else the unique
/// digest-ascending reference targets and member declarations, plus the member
/// `population` of a reaches or navigate application. A member `declaration` or
/// `population` that is not a node key is skipped here: the operation stage
/// owns that refusal and its path.
pub(super) fn application_dependencies(
    body: &super::CheckedTerm,
) -> Option<BTreeSet<CheckedNodeId>> {
    use super::CheckedTerm;
    let mut contains_application = false;
    let mut join = BTreeSet::new();
    let mut pending = vec![body];
    while let Some(term) = pending.pop() {
        match term {
            CheckedTerm::Reference { target } => {
                join.insert(target.clone());
            }
            CheckedTerm::Application {
                operation,
                arguments,
                ..
            } => {
                contains_application = true;
                pending.extend(arguments);
                let member = operation.get("member");
                let member_id = |key: &str| {
                    member.and_then(|member| member.get(key)).and_then(|value| {
                        serde_json::from_value::<CheckedNodeId>(value.clone()).ok()
                    })
                };
                join.extend(member_id("declaration"));
                // Only a relationship application names a population.
                let relationship = operation
                    .get("identity")
                    .and_then(Value::as_str)
                    .is_some_and(|identity| {
                        matches!(
                            super::operation_catalog::operation_catalog().known(identity),
                            Some(
                                super::KnownOperation::ModelReaches
                                    | super::KnownOperation::ModelNavigate
                            )
                        )
                    });
                if relationship {
                    join.extend(member_id("population"));
                }
            }
            CheckedTerm::Aggregate { members } => pending.extend(members),
            CheckedTerm::OperationAnchorBody(body) => pending.extend(&body.members),
            CheckedTerm::Binding { value, .. } => pending.push(value),
            // These bodies do not contribute local application dependencies.
            CheckedTerm::Literal { .. }
            | CheckedTerm::DependencyReference { .. }
            | CheckedTerm::Frame { .. }
            | CheckedTerm::AbstractionRelation { .. }
            | CheckedTerm::Refinement { .. }
            | CheckedTerm::ExtensionForm { .. } => {}
        }
    }
    contains_application.then_some(join)
}

/// An `aggregate` term's members.
pub(super) fn aggregate_members(term: &Value) -> Option<&[Value]> {
    match CheckedTerm::view(term)? {
        CheckedTermView::Aggregate { members } => Some(members),
        _ => None,
    }
}

/// A `binding` term's value, when the binding is named `name`.
pub(super) fn binding<'a>(term: &'a Value, name: &str) -> Option<&'a Value> {
    match CheckedTerm::view(term)? {
        CheckedTermView::Binding {
            name: actual,
            value,
        } if actual == name => Some(value),
        _ => None,
    }
}

/// A `reference` term's target.
pub(super) fn reference_target(term: &Value) -> Option<CheckedNodeId> {
    let target = reference_selector(term)?;
    (super::is_node_domain(&target.domain) && is_digest(&target.digest)).then_some(target)
}

/// A typed draft `reference` target before its provisional selector is replaced.
/// The reader still requires the final digest through [`reference_target`].
pub(super) fn reference_selector(term: &Value) -> Option<CheckedNodeId> {
    match CheckedTerm::view(term)? {
        CheckedTermView::Reference { target } => CheckedNodeId::deserialize(target).ok(),
        _ => None,
    }
}

/// `^(0|[1-9][0-9]*)$`.
#[ix_trace_rs::string_edge]
pub(super) fn is_non_negative_integer(value: &str) -> bool {
    value == "0" || is_positive_integer(value)
}

/// `^(0|-?[1-9][0-9]*)$`: an `integer_range` bound. No leading zero, no `+`,
/// and `-0` is refused.
#[ix_trace_rs::string_edge]
pub(super) fn is_integer_bound(value: &str) -> bool {
    value == "0" || is_nonzero_integer(value)
}

/// `^-?[1-9][0-9]*$`.
fn is_nonzero_integer(value: &str) -> bool {
    is_positive_integer(value.strip_prefix('-').unwrap_or(value))
}

/// `^[1-9][0-9]*$`.
fn is_positive_integer(value: &str) -> bool {
    let mut bytes = value.bytes();
    bytes
        .next()
        .is_some_and(|first| (b'1'..=b'9').contains(&first))
        && bytes.all(|byte| byte.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ix_trace_rs::trace;
    use serde_json::json;

    #[trace("TC-048", "FR-038-AC-22")]
    #[test]
    fn tc_048_structural_integer_grammars_are_exact() {
        assert!(is_non_negative_integer("0"));
        assert!(is_non_negative_integer("12"));
        assert!(!is_non_negative_integer("-1"));
        assert!(!is_non_negative_integer("01"));
        assert!(!is_non_negative_integer(""));
        assert!(is_nonzero_integer("-3"));
        assert!(is_nonzero_integer("2"));
        assert!(!is_nonzero_integer("0"));
        assert!(!is_nonzero_integer("-0"));
        assert!(!is_nonzero_integer("--1"));
    }

    fn id(fill: char) -> Value {
        json!({"domain": "quire.checked-semantic-node/v1", "digest": fill.to_string().repeat(64)})
    }

    #[trace("TC-048", "FR-038-AC-22")]
    #[test]
    fn tc_048_structural_reference_rejects_an_extra_wire_member() {
        let target = id('a');
        let closed = json!({"term": "reference", "target": target});
        assert_eq!(
            reference_selector(&closed),
            serde_json::from_value(target).ok()
        );

        let extra = json!({"term": "reference", "target": id('a'), "members": []});
        assert_eq!(reference_selector(&extra), None);
    }

    #[trace("TC-048", "FR-038-AC-23")]
    #[test]
    fn tc_048_application_join_collects_targets_and_member_declarations_only() {
        let body = json!({
            "term": "application",
            "operator": "query",
            "operation": {"identity": "x", "laws": [], "mode": null,
                "member": {"kind": "field", "declaration": id('c'), "name": "f"}, "leaves": []},
            "result_type": id('d'),
            "arguments": [
                {"term": "reference", "target": id('b')},
                {"term": "literal", "type": id('e'), "value_kind": "integer", "value": "1"},
                {"term": "aggregate", "members": [
                    {"term": "binding", "name": "x", "value": {"term": "reference", "target": id('a')}},
                    {"term": "reference", "target": id('b')},
                ]},
            ],
        });
        let term = super::super::CheckedTerm::decode_admitted(&body).expect("closed application");
        let join = application_dependencies(&term).expect("the body holds an application");
        let expected: BTreeSet<CheckedNodeId> = ['a', 'b', 'c']
            .into_iter()
            .map(|fill| serde_json::from_value(id(fill)).expect("node id"))
            .collect();
        assert_eq!(join, expected);
        assert_eq!(
            application_dependencies(
                &super::super::CheckedTerm::decode_admitted(
                    &json!({"term": "reference", "target": id('a')}),
                )
                .expect("closed reference"),
            ),
            None
        );
        // An unparseable member declaration is left to the operation stage.
        let mut malformed = body.clone();
        malformed["operation"]["member"]["declaration"] = json!("not a node key");
        let expected: BTreeSet<CheckedNodeId> = ['a', 'b']
            .into_iter()
            .map(|fill| serde_json::from_value(id(fill)).expect("node id"))
            .collect();
        let malformed =
            super::super::CheckedTerm::decode_admitted(&malformed).expect("closed application");
        assert_eq!(application_dependencies(&malformed), Some(expected));
    }
}
