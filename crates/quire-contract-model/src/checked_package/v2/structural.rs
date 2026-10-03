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
//!   type node (a `scalar_type`, `composite_type` or `bounded_domain` node,
//!   never itself); its body names no node, so `dependencies` is empty. Every
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
//!   node), binding "exponent" = a canonical nonzero integer literal typed
//!   at a `scalar_type`/`integer` node]}`, strictly ascending by unit key
//!   (the `quire.value.compound-unit/v1` term order); `dependencies` is
//!   exactly those unit keys in that order. The empty body is the
//!   dimensionless unit.
//! - **Application-node dependency join** (FR-322
//!   `application_node_preimage`): a node whose body contains an
//!   `application` term anywhere has `dependencies` exactly the unique,
//!   digest-ascending reference targets and operation member declarations
//!   of its body. `result_type` and literal `type` annotations are not
//!   dependencies.
//!
//! Neither structural form's node key is re-derived here: QSL keys both by
//! its proposed `quire.structural-node/v1` preimage, which QSpec does not
//! publish, and this reader re-derives only the published nominal and
//! application preimages.
//!
//! Every violation refuses as `invalid_semantic_graph` with no cause, at the
//! member it breaks (the node's `body` for a body or binding shape, the
//! occurrence's `role` for a role), located at the offending node, at the
//! first such node in ascending node-id digest order.
//! These are graph-shape refusals, so they run before the stale-key stage.
//! The stage charges no work of its own: every body term it walks was
//! parsed, shape-validated and charged once by the per-node body loop.

use super::identity::is_identifier;
use super::{ApplicationOperator, BodyTerm, LiteralKind, OperationMemberKind, StateClauseKind};
use super::{
    BoundedDomainForm, CheckedNodeKind, CheckedNodeTag, CheckedSemanticNodeV2, ClaimForm,
    CompositeTypeForm, CorrespondenceForm, ExpressionForm, FunctionForm, ModelForm,
    NominalIdentityPreimage, ProtocolForm, RelationForm, ScalarTypeForm, StateForm, TemporalForm,
    ValueForm,
};
use crate::checked_package::common::{
    application_operator, body_term, exact_members, literal_kind, node_pointer, ValidationFailure,
};
use crate::checked_package::shared::{
    CheckedNodeId, CheckedOccurrenceRole, CheckedPackageRefusalCode, JsonPointer,
};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};

/// The member of one node a structural defect is about.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Defect {
    SemanticType,
    Dependencies,
    /// The node body itself.
    Body,
    /// One member of the node body's `aggregate`.
    BodyMember(usize),
    /// The role of the occurrence at this index.
    Role(usize),
}

impl Defect {
    fn pointer(self, position: usize) -> JsonPointer {
        let node = node_pointer(position);
        match self {
            Self::SemanticType => node.key("semantic_type"),
            Self::Dependencies => node.key("dependencies"),
            Self::Body => node.key("body"),
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
}

impl Graph<'_> {
    fn node(&self, id: &CheckedNodeId) -> Option<(&CheckedSemanticNodeV2, CheckedNodeKind)> {
        let position = *self.index.get(id)?;
        Some((self.nodes.get(position)?, *self.kinds.get(position)?))
    }

    fn kind(&self, id: &CheckedNodeId) -> Option<CheckedNodeKind> {
        self.node(id).map(|(_, kind)| kind)
    }
}

/// Which body rule a node's kind selects.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StructuralForm {
    Parameter,
    CompoundUnit,
    Frame,
    OperationAnchor,
    StateClause,
    UnionType,
    UnionValue,
}

impl StructuralForm {
    /// The one role every occurrence of a node of this form carries, when
    /// the schema fixes one.
    const fn role(self) -> Option<CheckedOccurrenceRole> {
        match self {
            Self::Parameter => Some(CheckedOccurrenceRole::Expression),
            Self::Frame => Some(CheckedOccurrenceRole::Generated),
            Self::OperationAnchor => Some(CheckedOccurrenceRole::Anchor),
            Self::StateClause => Some(CheckedOccurrenceRole::Claim),
            Self::CompoundUnit | Self::UnionType | Self::UnionValue => None,
        }
    }

    /// Every form is listed, so a new form is a compile error here until
    /// its body rule is decided.
    fn of(kind: CheckedNodeKind) -> Option<Self> {
        use CheckedNodeKind as K;
        match kind {
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
                | ScalarTypeForm::Dimension
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
                ModelForm::ModelImport
                | ModelForm::ObjectType
                | ModelForm::ValueType
                | ModelForm::VariantType
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
            K::State(StateForm::Frame) => Some(Self::Frame),
            K::State(StateForm::OperationAnchor) => Some(Self::OperationAnchor),
            K::State(StateForm::StateClause) => Some(Self::StateClause),
            K::State(StateForm::Transition | StateForm::Snapshot) => None,
            K::Temporal(
                TemporalForm::TemporalClause
                | TemporalForm::Formula
                | TemporalForm::Fairness
                | TemporalForm::Clock
                | TemporalForm::Window
                | TemporalForm::Activation
                | TemporalForm::Deadline,
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
            K::Claim(
                ClaimForm::VerificationClaim
                | ClaimForm::AnalysisClaim
                | ClaimForm::Hyperproperty
                | ClaimForm::SynthesisRequest,
            ) => None,
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
    index: &BTreeMap<&CheckedNodeId, usize>,
) -> Result<(), ValidationFailure> {
    let graph = Graph {
        nodes,
        kinds,
        index,
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
        let defect = match form {
            Some(StructuralForm::Parameter) => parameter_defect(node, &graph),
            Some(StructuralForm::CompoundUnit) => compound_unit_defect(node, &graph),
            Some(StructuralForm::OperationAnchor) => {
                anchor_body(&node.body).is_none().then_some(Defect::Body)
            }
            Some(StructuralForm::StateClause) => {
                clause_body(&node.body).is_none().then_some(Defect::Body)
            }
            Some(StructuralForm::UnionType) => union_type_body(&node.body)
                .is_none()
                .then_some(Defect::Body),
            Some(StructuralForm::UnionValue) => union_value_body(&node.body)
                .is_none()
                .then_some(Defect::Body),
            // The per-node body loop already read the frame body.
            Some(StructuralForm::Frame) | None => None,
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
        if let Some(expected) = application_dependencies(&node.body) {
            if !node.dependencies.iter().eq(expected.iter()) {
                return Err(refuse(Defect::Dependencies));
            }
        }
    }
    Ok(())
}

/// The first rule a `value`/`parameter` node breaks, as the member it breaks
/// it at.
fn parameter_defect(node: &CheckedSemanticNodeV2, graph: &Graph<'_>) -> Option<Defect> {
    let is_binder_type = node.semantic_type != node.node_id
        && graph.kind(&node.semantic_type).is_some_and(|kind| {
            matches!(
                kind.tag(),
                CheckedNodeTag::ScalarType
                    | CheckedNodeTag::CompositeType
                    | CheckedNodeTag::BoundedDomain
            )
        });
    if !is_binder_type {
        return Some(Defect::SemanticType);
    }
    let Some([name, level]) = aggregate_members(&node.body) else {
        return Some(Defect::Body);
    };
    let name_ok = binding(name, "name")
        .and_then(|value| literal(value, LiteralKind::Text, ScalarTypeForm::Text, graph))
        .and_then(Value::as_str)
        .is_some_and(is_identifier);
    let level_ok = binding(level, "level")
        .and_then(|value| literal(value, LiteralKind::Integer, ScalarTypeForm::Integer, graph))
        .and_then(Value::as_str)
        .is_some_and(is_non_negative_integer);
    if !(name_ok && level_ok) {
        return Some(Defect::Body);
    }
    if !node.dependencies.is_empty() {
        return Some(Defect::Dependencies);
    }
    None
}

/// The first rule a `scalar_type`/`compound_unit` node breaks, as the member
/// it breaks it at.
fn compound_unit_defect(node: &CheckedSemanticNodeV2, graph: &Graph<'_>) -> Option<Defect> {
    if node.semantic_type != node.node_id {
        return Some(Defect::SemanticType);
    }
    let Some(terms) = aggregate_members(&node.body) else {
        return Some(Defect::Body);
    };
    let mut units: Vec<CheckedNodeId> = Vec::with_capacity(terms.len());
    for (index, term) in terms.iter().enumerate() {
        let Some(unit) = compound_unit_term(term, graph) else {
            return Some(Defect::BodyMember(index));
        };
        // Strictly ascending: canonical order and no repeated unit.
        if units.last().is_some_and(|previous| *previous >= unit) {
            return Some(Defect::BodyMember(index));
        }
        units.push(unit);
    }
    if node.dependencies != units {
        return Some(Defect::Dependencies);
    }
    None
}

/// One compound-unit term's root unit key, when the term has exactly the
/// closed term shape.
fn compound_unit_term(term: &Value, graph: &Graph<'_>) -> Option<CheckedNodeId> {
    let Some([unit, exponent]) = aggregate_members(term) else {
        return None;
    };
    let unit = binding(unit, "unit").and_then(reference_target)?;
    // QSL requires each term to name its dimension's canonical root unit.
    // A unit whose preimage has no target unit is that root: the nominal
    // stage admits exactly one per dimension.
    let is_root_unit = graph.node(&unit).is_some_and(|(node, kind)| {
        kind == CheckedNodeKind::ScalarType(ScalarTypeForm::Unit)
            && matches!(
                &node.nominal_identity_preimage,
                Some(NominalIdentityPreimage::Unit(preimage))
                    if preimage.target_unit_node_id.is_none()
            )
    });
    let exponent_ok = binding(exponent, "exponent")
        .and_then(|value| literal(value, LiteralKind::Integer, ScalarTypeForm::Integer, graph))
        .and_then(Value::as_str)
        .is_some_and(is_nonzero_integer);
    (is_root_unit && exponent_ok).then_some(unit)
}

/// A `state`/`operation_anchor` body (QSpec FR-342).
pub(super) struct AnchorBody<'a> {
    /// The object type declaring the operation.
    pub(super) context: CheckedNodeId,
    /// The operation's name.
    pub(super) operation: &'a str,
    /// The operation's frame node.
    pub(super) frame: CheckedNodeId,
}

/// The anchor body `body` holds, or `None` for any other shape.
pub(super) fn anchor_body(body: &Value) -> Option<AnchorBody<'_>> {
    let Some([context, operation, frame]) = aggregate_members(body) else {
        return None;
    };
    Some(AnchorBody {
        context: binding(context, "context").and_then(reference_target)?,
        operation: binding(operation, "operation")
            .filter(|value| literal_kind(value) == Some(LiteralKind::Text))
            .and_then(|value| value.get("value"))
            .and_then(Value::as_str)
            .filter(|name| is_identifier(name))?,
        frame: binding(frame, "frame").and_then(reference_target)?,
    })
}

/// The catalogued identity of the state clause operation.
const STATE_CLAUSE_OPERATION: &str = "quire.op.state.clause";

/// A `state`/`state_clause` body (QSpec FR-341).
pub(super) struct ClauseBody {
    /// The clause kind its `state_clause` member names.
    pub(super) clause: StateClauseKind,
    /// The parameter aggregate's reference targets, in order.
    pub(super) parameters: Vec<CheckedNodeId>,
    /// The anchor reference's target.
    pub(super) anchor: CheckedNodeId,
}

/// Whether `term` is an application of `quire.op.state.clause`.
// Reads an application's operation identity.
pub(super) fn is_state_clause_application(term: &Value) -> bool {
    body_term(term) == Some(BodyTerm::Application)
        && term
            .get("operation")
            .and_then(|operation| operation.get("identity"))
            .and_then(Value::as_str)
            .is_some_and(|identity| identity == STATE_CLAUSE_OPERATION)
}

/// The state clause body `body` holds, or `None` for any other shape: an
/// application of `quire.op.state.clause` with operator class
/// `state_clause`, member exactly `{kind: "state_clause", clause}`, and
/// arguments an `aggregate` of one or more `reference` terms, a `reference`
/// and the condition.
// Decodes a state clause member's kind and clause.
pub(super) fn clause_body(body: &Value) -> Option<ClauseBody> {
    if !is_state_clause_application(body)
        || application_operator(body) != Some(ApplicationOperator::StateClause)
    {
        return None;
    }
    let member = body.get("operation")?.get("member")?.as_object()?;
    let kind = member
        .get("kind")
        .and_then(Value::as_str)
        .and_then(OperationMemberKind::from_wire);
    if !exact_members(member, &["kind", "clause"]) || kind != Some(OperationMemberKind::StateClause)
    {
        return None;
    }
    let clause = StateClauseKind::from_wire(member.get("clause")?.as_str()?)?;
    let [parameters, anchor, _condition] = body.get("arguments")?.as_array()?.as_slice() else {
        return None;
    };
    let parameters = aggregate_members(parameters)?
        .iter()
        .map(reference_target)
        .collect::<Option<Vec<_>>>()
        .filter(|parameters| !parameters.is_empty())?;
    Some(ClauseBody {
        clause,
        parameters,
        anchor: reference_target(anchor)?,
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
pub(super) fn union_type_body(body: &Value) -> Option<Vec<UnionMember<'_>>> {
    let members = aggregate_members(body)?
        .iter()
        .map(|member| {
            let name = member
                .get("name")
                .and_then(Value::as_str)
                .filter(|name| is_identifier(name))?;
            let payload = aggregate_members(binding(member, name)?)?
                .iter()
                .map(reference_target)
                .collect::<Option<Vec<_>>>()?;
            Some(UnionMember { name, payload })
        })
        .collect::<Option<Vec<_>>>()?;
    (!members.is_empty()).then_some(members)
}

/// The active member and payload terms a `value`/`union_value` body holds, or
/// `None` for any other shape: an `aggregate` of exactly one `binding`, named
/// by an identifier and valued by an `aggregate` (QSpec's `UnionValueBody`).
pub(super) fn union_value_body(body: &Value) -> Option<(&str, &[Value])> {
    let [member] = aggregate_members(body)? else {
        return None;
    };
    let name = member
        .get("name")
        .and_then(Value::as_str)
        .filter(|name| is_identifier(name))?;
    Some((name, aggregate_members(binding(member, name)?)?))
}

/// FR-322's dependency join for a body that contains an `application` term
/// at any depth: `None` for a body holding none, else the unique
/// digest-ascending reference targets and member declarations. A member
/// `declaration` that is not a node key is skipped here: the operation stage
/// owns that refusal and its path.
fn application_dependencies(body: &Value) -> Option<BTreeSet<CheckedNodeId>> {
    let mut contains_application = false;
    let mut join = BTreeSet::new();
    let mut pending = vec![body];
    while let Some(term) = pending.pop() {
        let Some(object) = term.as_object() else {
            continue;
        };
        match body_term(term) {
            Some(BodyTerm::Reference) => {
                if let Some(target) = reference_target(term) {
                    join.insert(target);
                }
            }
            Some(BodyTerm::Application) => {
                contains_application = true;
                let declaration = object
                    .get("operation")
                    .and_then(|operation| operation.get("member"))
                    .and_then(|member| member.get("declaration"))
                    .and_then(|declaration| {
                        serde_json::from_value::<CheckedNodeId>(declaration.clone()).ok()
                    });
                join.extend(declaration);
                pending.extend(terms(object, "arguments"));
            }
            Some(BodyTerm::Aggregate) => pending.extend(terms(object, "members")),
            Some(BodyTerm::Binding) => pending.extend(object.get("value")),
            // A literal names only its type annotation, which is not a
            // dependency; a `dependency_reference` names a node of another
            // package and is never listed in `dependencies`; a frame is
            // never a nested term; a tag outside the vocabulary is refused
            // before this stage.
            Some(BodyTerm::Literal | BodyTerm::DependencyReference | BodyTerm::Frame) | None => {}
        }
    }
    contains_application.then_some(join)
}

fn terms<'a>(object: &'a Map<String, Value>, key: &str) -> impl Iterator<Item = &'a Value> {
    object
        .get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
}

/// An `aggregate` term's members.
pub(super) fn aggregate_members(term: &Value) -> Option<&[Value]> {
    let object = term.as_object()?;
    (body_term(term) == Some(BodyTerm::Aggregate))
        .then(|| object.get("members").and_then(Value::as_array))
        .flatten()
        .map(Vec::as_slice)
}

/// A `binding` term's value, when the binding is named `name`.
pub(super) fn binding<'a>(term: &'a Value, name: &str) -> Option<&'a Value> {
    let object = term.as_object()?;
    (body_term(term) == Some(BodyTerm::Binding)
        && object.get("name").and_then(Value::as_str) == Some(name))
    .then(|| object.get("value"))
    .flatten()
}

/// A `reference` term's target.
pub(super) fn reference_target(term: &Value) -> Option<CheckedNodeId> {
    let object = term.as_object()?;
    if body_term(term) != Some(BodyTerm::Reference) {
        return None;
    }
    serde_json::from_value(object.get("target")?.clone()).ok()
}

/// A `literal` term's value, when its `value_kind` is `value_kind` and its
/// `type` names a `scalar_type` node of form `type_form`.
fn literal<'a>(
    term: &'a Value,
    value_kind: LiteralKind,
    type_form: ScalarTypeForm,
    graph: &Graph<'_>,
) -> Option<&'a Value> {
    let object = term.as_object()?;
    if body_term(term) != Some(BodyTerm::Literal) || literal_kind(term) != Some(value_kind) {
        return None;
    }
    let ty: CheckedNodeId = serde_json::from_value(object.get("type")?.clone()).ok()?;
    (graph.kind(&ty)? == CheckedNodeKind::ScalarType(type_form))
        .then(|| object.get("value"))
        .flatten()
}

/// `^(0|[1-9][0-9]*)$`.
fn is_non_negative_integer(value: &str) -> bool {
    value == "0" || is_positive_integer(value)
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
    use serde_json::json;

    /// Tracing: TC-048, FR-038-AC-22
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

    /// Tracing: TC-048, FR-038-AC-23
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
        let join = application_dependencies(&body).expect("the body holds an application");
        let expected: BTreeSet<CheckedNodeId> = ['a', 'b', 'c']
            .into_iter()
            .map(|fill| serde_json::from_value(id(fill)).expect("node id"))
            .collect();
        assert_eq!(join, expected);
        assert_eq!(
            application_dependencies(&json!({"term": "reference", "target": id('a')})),
            None
        );
        // An unparseable member declaration is left to the operation stage.
        let mut malformed = body.clone();
        malformed["operation"]["member"]["declaration"] = json!("not a node key");
        let expected: BTreeSet<CheckedNodeId> = ['a', 'b']
            .into_iter()
            .map(|fill| serde_json::from_value(id(fill)).expect("node id"))
            .collect();
        assert_eq!(application_dependencies(&malformed), Some(expected));
    }
}
