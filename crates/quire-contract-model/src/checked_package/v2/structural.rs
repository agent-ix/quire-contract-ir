//! Graph-shape rules for node bodies that carry no nominal declaration: the
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
//!   node), binding "exponent" = a canonical nonzero integer literal typed
//!   at a `scalar_type`/`integer` node]}`, strictly ascending by unit key
//!   (the `quire.value.compound-unit/v1` term order); `dependencies` is
//!   exactly those unit keys in that order. The empty body is the
//!   dimensionless unit.
//! - **Application-node dependency join** (FR-322): a node whose body contains an
//!   `application` term anywhere has `dependencies` exactly the unique,
//!   node-id-ascending reference targets and operation member declarations
//!   of its body. `result_type` and literal `type` annotations are not
//!   dependencies.
//!
//! This stage checks declared structural identities and graph shape. No body
//! derives a node identity.
//!
//! Every violation refuses as `invalid_semantic_graph` with no cause, at the
//! member it breaks (the node's `body` for a body or binding shape, the
//! occurrence's `role` for a role), located at the offending node, at the
//! first such node in ascending structural node-id order.
//! These graph-shape refusals run before the independent derived-shape check.
//! The stage charges no work of its own: every body term it walks was
//! parsed, shape-validated and charged once by the per-node body loop.

use super::graph_storage::GraphIndex;
use super::identity::is_identifier;
use super::model_members::{DomainModel, MemberType, ValueBounds};
use super::native_join_budget::PackageReadStage;
use super::node_index::resolve_id_charged;
use super::stage_storage::{self, Path, Refusal, Site};
use super::term_view::{TermNode, TermView};
use super::{
    ApplicationOperator, CheckedTerm, ExecutionPoint, LiteralKind, OperationMemberKind,
    StateClauseKind, WorkMeter,
};
use super::{
    BoundedDomainForm, CheckedNodeKind, CheckedNodeTag, ClaimForm, CompositeTypeForm,
    CorrespondenceForm, ExpressionForm, FunctionForm, ModelForm, ProtocolForm,
    RawSemanticNodeV2 as CheckedSemanticNodeV2, RelationForm, ScalarTypeForm, StateForm,
    TemporalForm, ValueForm,
};
use crate::checked_package::common::{node_pointer, ValidationFailure};
use crate::checked_package::shared::{
    CheckedNodeId, CheckedNodePathSegment, CheckedOccurrenceRole, CheckedPackageRefusalCode,
};
use quire_exact::TextProfile;
use std::cmp::Ordering;

/// The member of one node a structural defect is about.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Defect {
    SemanticType,
    Dependencies,
    RecursionGroup,
    Occurrences,
    /// The node body itself.
    Body,
    /// A malformed state-clause execution point.
    ClausePoint,
    /// The role of the occurrence at this index.
    Role(usize),
}

impl Defect {
    fn path(self, position: usize) -> Path<'static> {
        let node = Path::root()
            .key("semantic_graph")
            .key("nodes")
            .index(position);
        match self {
            Self::SemanticType => node.key("semantic_type"),
            Self::Dependencies => node.key("dependencies"),
            Self::RecursionGroup => node.key("recursion_group"),
            Self::Occurrences => node.key("occurrences"),
            Self::Body => node.key("body"),
            Self::ClausePoint => node
                .key("body")
                .key("operation")
                .key("member")
                .key("execution_point"),
            Self::Role(occurrence) => node.key("occurrences").index(occurrence).key("role"),
        }
    }
}

/// Prepay the structural diagnostic before constructing its pointer, node
/// identity, location and boxed record. The optional node locus preserves the
/// distinct payloads of structural defects and malformed raw bodies.
fn structural_refusal(
    meter: &mut WorkMeter,
    position: usize,
    form: &'static str,
    code: CheckedPackageRefusalCode,
    defect: Defect,
    locus: Option<&CheckedNodeId>,
) -> ValidationFailure {
    let at = defect.path(position);
    let mut refusal = Refusal::new(code, at);
    if let Some(locus) = locus {
        refusal = refusal.at_node(locus);
    }
    stage_storage::refuse(
        meter,
        &Site::new(PackageReadStage::Graph, position, form, at),
        refusal,
    )
}

/// The graph one stage reads: each node with its decoded kind, by key.
struct Graph<'a> {
    nodes: &'a [CheckedSemanticNodeV2],
    kinds: &'a [CheckedNodeKind],
    index: &'a GraphIndex<'a>,
    models: Option<&'a [super::model_members::DomainModel]>,
}

impl Graph<'_> {
    fn node(&self, id: &CheckedNodeId) -> Option<(&CheckedSemanticNodeV2, CheckedNodeKind)> {
        let position = *self.index.get(id)?;
        Some((self.nodes.get(position)?, *self.kinds.get(position)?))
    }

    /// The node, its kind and its position.
    fn node_at(
        &self,
        id: &CheckedNodeId,
    ) -> Option<(usize, &CheckedSemanticNodeV2, CheckedNodeKind)> {
        let position = *self.index.get(id)?;
        Some((
            position,
            self.nodes.get(position)?,
            *self.kinds.get(position)?,
        ))
    }

    fn kind(&self, id: &CheckedNodeId) -> Option<CheckedNodeKind> {
        self.node(id).map(|(_, kind)| kind)
    }

    fn type_matches(&self, id: &CheckedNodeId, form: ScalarTypeForm) -> bool {
        self.kind(id) == Some(CheckedNodeKind::ScalarType(form))
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
    CarriedQuantity,
    BoundedQuantity,
    Frame,
    OperationAnchor,
    StateClause,
    StateTransition,
    UnionType,
    MapType,
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
            | Self::CarriedQuantity
            | Self::BoundedQuantity
            | Self::UnionType
            | Self::MapType
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
            K::ScalarType(ScalarTypeForm::CarriedQuantity) => Some(Self::CarriedQuantity),
            K::ScalarType(ScalarTypeForm::BoundedQuantity) => Some(Self::BoundedQuantity),
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
            K::CompositeType(CompositeTypeForm::Map) => Some(Self::MapType),
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
                | TemporalForm::Terminal
                | TemporalForm::MonitorPlan,
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
                | ClaimForm::StepRelation
                | ClaimForm::Schedulability
                | ClaimForm::ProbabilisticClaim,
            ) => None,
            K::Correspondence(
                CorrespondenceForm::AbstractionRelation
                | CorrespondenceForm::ImplementationGuarantee
                | CorrespondenceForm::ComponentComposition,
            ) => Some(Self::AbstractionRelation),
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
/// nodes in ascending structural node-id order.
pub(super) fn validate_structural_nodes(
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    terms: &[Option<super::CheckedTerm>],
    index: &GraphIndex<'_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    validate_structural_nodes_with_keys(nodes, kinds, terms, index, &[], None, meter)
}

/// The key a listed dependency ID stands for when the graph holds no such
/// node, with its blocks charged first. An ID that already holds operand keys
/// is its own key; an index-form ID resolves against the graph keys.
fn resolve_unlisted_dependency(
    id: &CheckedNodeId,
    structural_keys: &[std::sync::Arc<CheckedNodeId>],
    meter: &mut WorkMeter,
) -> Result<Option<std::sync::Arc<CheckedNodeId>>, ValidationFailure> {
    let resolved = if id
        .path
        .iter()
        .any(|segment| matches!(segment, CheckedNodePathSegment::Node(_)))
    {
        if id
            .path
            .iter()
            .any(|segment| matches!(segment, CheckedNodePathSegment::Index(_)))
        {
            return Ok(None);
        }
        Some(meter.clone_id(id)?)
    } else {
        resolve_id_charged(id, structural_keys, meter)?
    };
    let Some(resolved) = resolved else {
        return Ok(None);
    };
    // The `Arc` block holds the counts beside the id.
    meter.reserve(
        std::mem::size_of::<CheckedNodeId>().saturating_add(2 * std::mem::size_of::<usize>()),
    )?;
    Ok(Some(std::sync::Arc::new(resolved)))
}

/// The keys the graph-wide order covers: every graph key, then the key of each
/// listed dependency ID absent from the graph, with the address of each listed
/// ID beside its position among the keys.
struct DependencyKeys {
    keys: Vec<std::sync::Arc<CheckedNodeId>>,
    /// `(address of the listed ID, position of its key in keys)`, by address.
    absent: Vec<(*const CheckedNodeId, usize)>,
}

impl DependencyKeys {
    fn new<'a>(
        dependencies: impl IntoIterator<Item = &'a [CheckedNodeId]>,
        index: &GraphIndex<'_>,
        structural_keys: &[std::sync::Arc<CheckedNodeId>],
        meter: &mut WorkMeter,
    ) -> Result<Self, ValidationFailure> {
        let mut keys = meter.vec(structural_keys.len())?;
        let mut absent: Vec<(*const CheckedNodeId, usize)> = Vec::new();
        if structural_keys.is_empty() {
            return Ok(Self { keys, absent });
        }
        keys.extend(structural_keys.iter().cloned());
        for listed in dependencies {
            for id in listed {
                if index.contains_key(id) {
                    continue;
                }
                if let Some(key) = resolve_unlisted_dependency(id, structural_keys, meter)? {
                    meter.push(&mut absent, (std::ptr::from_ref(id), keys.len()))?;
                    meter.push(&mut keys, key)?;
                }
            }
        }
        absent.sort_unstable_by_key(|(address, _)| *address);
        Ok(Self { keys, absent })
    }

    /// The key of a listed dependency ID: the graph's for a node it holds,
    /// the resolved one for an absent ID, none for an unresolvable one.
    fn key_of<'s>(
        &'s self,
        id: &CheckedNodeId,
        index: &GraphIndex<'_>,
        structural_keys: &'s [std::sync::Arc<CheckedNodeId>],
    ) -> Option<&'s std::sync::Arc<CheckedNodeId>> {
        if let Ok(at) = self
            .absent
            .binary_search_by_key(&std::ptr::from_ref(id), |(address, _)| *address)
        {
            return self.keys.get(self.absent[at].1);
        }
        structural_keys.get(*index.get(id)?)
    }
}

/// One graph-wide ordering table for both present and absent dependency IDs,
/// built once through the caller's budget by `StructuralOrder`. Missing IDs are
/// still compared here; their body join owns the typed refusal.
struct DependencyKeyOrder<'k> {
    keys: &'k DependencyKeys,
    order: Option<super::StructuralOrder<'k>>,
}

impl<'k> DependencyKeyOrder<'k> {
    fn new(keys: &'k DependencyKeys, meter: &mut WorkMeter) -> Result<Self, ValidationFailure> {
        let order = if keys.keys.is_empty() {
            None
        } else {
            meter.structural_order(&[keys.keys.as_slice()])?
        };
        Ok(Self { keys, order })
    }

    fn matches(
        &self,
        actual: &[CheckedNodeId],
        expected: &[&CheckedNodeId],
        index: &GraphIndex<'_>,
        structural_keys: &[std::sync::Arc<CheckedNodeId>],
        meter: &mut WorkMeter,
    ) -> Result<bool, ValidationFailure> {
        if structural_keys.is_empty() {
            // No structural keys: the IDs are the document's own flat IDs,
            // whose order allocates nothing.
            let mut expected_keys = meter.vec(expected.len())?;
            expected_keys.extend(
                expected
                    .iter()
                    .map(|id| super::structural_key(id, index, structural_keys)),
            );
            expected_keys.sort_unstable();
            expected_keys.dedup();
            return Ok(actual.len() == expected_keys.len()
                && actual
                    .iter()
                    .map(|id| super::structural_key(id, index, structural_keys))
                    .zip(expected_keys)
                    .all(|(left, right)| left == right));
        }
        let Some(order) = self.order.as_ref() else {
            return Ok(false);
        };
        let mut expected_tokens = meter.vec(expected.len())?;
        for &raw in expected {
            let token = if let Some(&position) = index.get(raw) {
                structural_keys
                    .get(position)
                    .and_then(|key| order.token(key))
            } else {
                // An expected key outside the graph is found by its resolved
                // ID; one that equals no listed key cannot match.
                match resolve_unlisted_dependency(raw, structural_keys, meter)? {
                    Some(resolved) => order.find(&resolved),
                    None => None,
                }
            };
            let Some(token) = token else {
                return Ok(false);
            };
            expected_tokens.push(token);
        }
        expected_tokens.sort_unstable();
        expected_tokens.dedup();
        if actual.len() != expected_tokens.len() {
            return Ok(false);
        }
        let mut previous_rank = None;
        let mut actual_tokens = meter.vec(actual.len())?;
        for id in actual {
            let Some(key) = self.keys.key_of(id, index, structural_keys) else {
                return Ok(false);
            };
            let (Some(token), Some(rank)) = (order.token(key), order.rank(key)) else {
                return Ok(false);
            };
            if previous_rank.is_some_and(|previous| previous >= rank) {
                return Ok(false);
            }
            previous_rank = Some(rank);
            actual_tokens.push(token);
        }
        actual_tokens.sort_unstable();
        actual_tokens.dedup();
        Ok(actual_tokens == expected_tokens)
    }

    fn ordered(
        &self,
        actual: &[CheckedNodeId],
        index: &GraphIndex<'_>,
        structural_keys: &[std::sync::Arc<CheckedNodeId>],
    ) -> bool {
        if structural_keys.is_empty() {
            return actual.windows(2).all(|pair| pair[0] < pair[1]);
        }
        let Some(order) = self.order.as_ref() else {
            return false;
        };
        let rank = |id: &CheckedNodeId| {
            self.keys
                .key_of(id, index, structural_keys)
                .and_then(|key| order.rank(key))
        };
        actual
            .windows(2)
            .all(|pair| match (rank(&pair[0]), rank(&pair[1])) {
                (Some(left), Some(right)) => left < right,
                _ => false,
            })
    }
}

pub(super) fn validate_structural_nodes_with_keys(
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    terms: &[Option<super::CheckedTerm>],
    index: &GraphIndex<'_>,
    structural_keys: &[std::sync::Arc<CheckedNodeId>],
    models: Option<&[super::model_members::DomainModel]>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let graph = Graph {
        nodes,
        kinds,
        index,
        models,
    };
    // Rank once for this graph, including listed IDs absent from its nodes.
    // All later comparisons use shallow tokens and ranks, not deep key walks.
    meter.at_node(0, "dependency_order");
    let dependency_keys = DependencyKeys::new(
        nodes.iter().map(|node| node.dependencies.as_slice()),
        index,
        structural_keys,
        meter,
    )?;
    let dependency_order = DependencyKeyOrder::new(&dependency_keys, meter)?;
    for (&node_id, &position) in index {
        let (Some(node), Some(&kind)) = (nodes.get(position), kinds.get(position)) else {
            continue;
        };
        let node_form = super::graph_storage::node_form(&node.node_tag);
        let refuse = |meter: &mut WorkMeter, defect: Defect| {
            structural_refusal(
                meter,
                position,
                node_form,
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                defect,
                Some(node_id),
            )
        };
        let malformed = |meter: &mut WorkMeter| {
            structural_refusal(
                meter,
                position,
                node_form,
                CheckedPackageRefusalCode::MalformedWire,
                Defect::Body,
                None,
            )
        };
        meter.at_node(position, node_form);
        let form = StructuralForm::of(kind);
        let term = terms.get(position).and_then(Option::as_ref);
        let defect = match form {
            Some(StructuralForm::Extension) => {
                let Some(super::CheckedTerm::ExtensionForm { clauses, .. }) = term else {
                    return Err(malformed(meter));
                };
                if clauses.iter().any(|clause| {
                    !matches!(clause, super::CheckedTerm::Binding { value, .. }
                        if matches!(value.as_ref(), super::CheckedTerm::Reference { .. }))
                }) {
                    return Err(malformed(meter));
                }
                let mut expected: Vec<&CheckedNodeId> = meter.vec(clauses.len())?;
                expected.extend(clauses.iter().filter_map(|clause| match clause {
                    super::CheckedTerm::Binding { value, .. } => match value.as_ref() {
                        super::CheckedTerm::Reference { target } => Some(target),
                        _ => None,
                    },
                    _ => None,
                }));
                if node.semantic_type != node.node_id {
                    Some(Defect::SemanticType)
                } else if node.recursion_group.is_some() {
                    Some(Defect::RecursionGroup)
                } else if !dependency_order.matches(
                    &node.dependencies,
                    &expected,
                    index,
                    structural_keys,
                    meter,
                )? {
                    Some(Defect::Dependencies)
                } else {
                    None
                }
            }
            Some(StructuralForm::Hybrid | StructuralForm::TimedModel) => {
                let Some(term) = term else {
                    return Err(malformed(meter));
                };
                let mut pending = meter.vec(1)?;
                pending.push(term);
                let mut targets: Vec<&CheckedNodeId> = Vec::new();
                while let Some(next) = pending.pop() {
                    match next {
                        super::CheckedTerm::Reference { target } => {
                            meter.push(&mut targets, target)?;
                        }
                        super::CheckedTerm::Aggregate { members } => {
                            meter.room(&mut pending, members.len())?;
                            pending.extend(members);
                        }
                        super::CheckedTerm::Binding { value, .. } => {
                            meter.push(&mut pending, value.as_ref())?;
                        }
                        super::CheckedTerm::Literal { .. } => {}
                        _ => return Err(refuse(meter, Defect::Body)),
                    }
                }
                if node.semantic_type != node.node_id {
                    Some(Defect::SemanticType)
                } else if node.recursion_group.is_some() {
                    Some(Defect::RecursionGroup)
                } else if !dependency_order.matches(
                    &node.dependencies,
                    &targets,
                    index,
                    structural_keys,
                    meter,
                )? {
                    Some(Defect::Dependencies)
                } else {
                    None
                }
            }
            Some(StructuralForm::Parameter) => parameter_defect(node, term, &graph),
            Some(StructuralForm::CompoundUnit) => {
                compound_unit_dependencies_defect(node, term, &graph, meter)?
            }
            Some(StructuralForm::CarriedQuantity) => None,
            Some(StructuralForm::BoundedQuantity) => None,
            Some(StructuralForm::OperationAnchor) => term
                .and_then(typed_anchor_body)
                .is_none()
                .then_some(Defect::Body),
            Some(StructuralForm::StateClause) => {
                if meter
                    .body(position)
                    .and_then(|(body, _)| body.get("operation"))
                    .and_then(|operation| operation.get("member"))
                    .and_then(|member| member.get("execution_point"))
                    .is_some_and(|point| !is_closed_execution_point(point))
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
            Some(StructuralForm::UnionType) => match term {
                Some(term) => typed_union_type_body(term, meter)?.is_none(),
                None => true,
            }
            .then_some(Defect::Body),
            Some(StructuralForm::MapType) => {
                let Some([key, value]) = term.and_then(typed_aggregate) else {
                    return Err(refuse(meter, Defect::Body));
                };
                let Some(key) = typed_binding(key, "key").and_then(typed_reference) else {
                    return Err(refuse(meter, Defect::Body));
                };
                let Some(value) = typed_binding(value, "value").and_then(typed_reference) else {
                    return Err(refuse(meter, Defect::Body));
                };
                (!dependency_order.matches(
                    &node.dependencies,
                    &[key, value],
                    index,
                    structural_keys,
                    meter,
                )?)
                .then_some(Defect::Dependencies)
            }
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
            return Err(refuse(meter, defect));
        }
        if kinds.get(position) == Some(&CheckedNodeKind::Protocol(ProtocolForm::ProtocolClause)) {
            if let Some(owner_model) = node.owner_model.as_ref() {
                let ordered = dependency_order.ordered(&node.dependencies, index, structural_keys);
                if !ordered || !node.dependencies.contains(owner_model) {
                    return Err(refuse(meter, Defect::Dependencies));
                }
                if let Some(mut expected) = match term {
                    Some(_) => application_dependencies_at(position, meter)?,
                    None => None,
                } {
                    meter.insert_id(&mut expected, owner_model)?;
                    let mut listed: Vec<&CheckedNodeId> = meter.vec(expected.len())?;
                    listed.extend(expected.iter());
                    if !dependency_order.matches(
                        &node.dependencies,
                        &listed,
                        index,
                        structural_keys,
                        meter,
                    )? {
                        return Err(refuse(meter, Defect::Dependencies));
                    }
                }
            }
            continue;
        }
        if let Some(expected) = match term {
            Some(_) => application_dependencies_at(position, meter)?,
            None => None,
        } {
            let mut listed: Vec<&CheckedNodeId> = meter.vec(expected.len())?;
            listed.extend(expected.iter());
            if !dependency_order.matches(
                &node.dependencies,
                &listed,
                index,
                structural_keys,
                meter,
            )? {
                return Err(refuse(meter, Defect::Dependencies));
            }
        }
    }
    Ok(())
}

/// Compare canonical nonnegative decimal integers without narrowing them to a machine word.
pub(super) fn canonical_non_negative_cmp(left: &str, right: &str) -> Ordering {
    left.len().cmp(&right.len()).then_with(|| left.cmp(right))
}

fn text_bounds_shape(term: Option<&CheckedTerm>, graph: &Graph<'_>) -> bool {
    fn bound<'a>(member: &'a CheckedTerm, name: &str, graph: &Graph<'_>) -> Option<&'a str> {
        let CheckedTerm::Literal {
            type_id,
            value_kind: LiteralKind::Integer,
            value,
        } = typed_binding(member, name)?
        else {
            return None;
        };
        let text = value.as_str()?;
        (graph.type_matches(type_id, ScalarTypeForm::Integer) && is_non_negative_integer(text))
            .then_some(text)
    }
    let Some([min, max, profile]) = term.and_then(typed_aggregate) else {
        return false;
    };
    let (Some(min), Some(max)) = (bound(min, "min", graph), bound(max, "max", graph)) else {
        return false;
    };
    let Some(CheckedTerm::Literal {
        type_id,
        value_kind: LiteralKind::Text,
        value,
    }) = typed_binding(profile, "text_profile")
    else {
        return false;
    };
    canonical_non_negative_cmp(min, max) != Ordering::Greater
        && graph.type_matches(type_id, ScalarTypeForm::Text)
        && value.as_str().and_then(TextProfile::from_code).is_some()
}

/// FR-038-AC-124/125: validate the closed bodies of the ten derived type
/// forms independently of their producer-assigned structural node IDs.
pub(super) fn validate_derived_type_shapes(
    catalog: &super::operation_catalog::OperationCatalog,
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    terms: &[Option<CheckedTerm>],
    index: &GraphIndex<'_>,
    structural_keys: &[std::sync::Arc<CheckedNodeId>],
    models: &[super::model_members::DomainModel],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let graph = Graph {
        nodes,
        kinds,
        index,
        models: Some(models),
    };
    for (&node_id, &position) in index {
        let (Some(node), Some(&kind)) = (nodes.get(position), kinds.get(position)) else {
            continue;
        };
        meter.at_node(position, super::graph_storage::node_form(&node.node_tag));
        let term = terms.get(position).and_then(Option::as_ref);
        let valid = match kind {
            CheckedNodeKind::ScalarType(ScalarTypeForm::Boolean | ScalarTypeForm::Integer) => {
                node.semantic_type == node.node_id
                    && term.and_then(typed_aggregate).is_some_and(<[_]>::is_empty)
            }
            CheckedNodeKind::CompositeType(CompositeTypeForm::Reference) => {
                node.semantic_type == node.node_id
                    && term.and_then(typed_aggregate).is_some_and(|members| {
                        matches!(members, [CheckedTerm::Reference { target }]
                            if matches!(graph.kind(target),
                                Some(CheckedNodeKind::Model(
                                    ModelForm::ObjectType
                                    | ModelForm::Process
                                    | ModelForm::StateMachine
                                    | ModelForm::SystemsInterface
                                ) | CheckedNodeKind::Relation(RelationForm::Relationship)))
                        )
                    })
            }
            CheckedNodeKind::CompositeType(
                CompositeTypeForm::Option
                | CompositeTypeForm::Set
                | CompositeTypeForm::Bag
                | CompositeTypeForm::Sequence
                | CompositeTypeForm::OrderedSet,
            ) => {
                node.semantic_type == node.node_id
                    && term
                        .and_then(typed_aggregate)
                        .is_some_and(|members| matches!(members, [CheckedTerm::Reference { target }] if target.is_well_formed()))
            }
            CheckedNodeKind::CompositeType(CompositeTypeForm::Map) => {
                let operands = term.and_then(typed_aggregate).and_then(|members| {
                    let [key, value] = members else { return None; };
                    Some((
                        typed_reference(typed_binding(key, "key")?)?,
                        typed_reference(typed_binding(value, "value")?)?,
                    ))
                });
                let shaped = node.semantic_type == node.node_id
                    && operands.is_some_and(|(key, value)| {
                        matches!(graph.kind(key), Some(CheckedNodeKind::ScalarType(
                            ScalarTypeForm::Boolean | ScalarTypeForm::Integer
                            | ScalarTypeForm::Rational | ScalarTypeForm::Decimal
                            | ScalarTypeForm::Float32 | ScalarTypeForm::Float64
                            | ScalarTypeForm::Text | ScalarTypeForm::String
                            | ScalarTypeForm::Uuid | ScalarTypeForm::Timestamp
                            | ScalarTypeForm::Unit | ScalarTypeForm::Enum
                            | ScalarTypeForm::CompoundUnit | ScalarTypeForm::CarriedQuantity
                            | ScalarTypeForm::BoundedQuantity
                        )))
                            && matches!(graph.kind(value),
                                Some(CheckedNodeKind::ScalarType(_)
                                    | CheckedNodeKind::CompositeType(_)
                                    | CheckedNodeKind::BoundedDomain(_)
                                    | CheckedNodeKind::Model(
                                        ModelForm::RecordValueType
                                            | ModelForm::TupleValueType
                                            | ModelForm::VariantType
                                    )))
                            && !matches!(graph.kind(value), Some(CheckedNodeKind::ScalarType(
                                ScalarTypeForm::Dimension | ScalarTypeForm::DimensionEquivalence
                            )))
                    });
                // The key's identity holds the key and value nodes it names.
                // Comparing them is charged: a rebuilt key nests as deep as
                // its operand chain.
                match (shaped, operands, structural_keys.get(position)) {
                    (true, Some((key, value)), Some(id)) => match id.path.as_slice() {
                        [CheckedNodePathSegment::Name(tag), CheckedNodePathSegment::Name(form), CheckedNodePathSegment::Node(key_id), CheckedNodePathSegment::Node(value_id)]
                            if tag.as_ref() == "composite_type" && form.as_ref() == "map" =>
                        {
                            meter.compare_ids(
                                key_id,
                                super::structural_key(key, index, structural_keys),
                            )? == std::cmp::Ordering::Equal
                                && meter.compare_ids(
                                    value_id,
                                    super::structural_key(value, index, structural_keys),
                                )? == std::cmp::Ordering::Equal
                        }
                        _ => false,
                    },
                    _ => false,
                }
            }
            CheckedNodeKind::BoundedDomain(BoundedDomainForm::TextBounds) => {
                // A bound's form selects its own base, regardless of its authored identity.
                if !graph.type_matches(&node.semantic_type, ScalarTypeForm::Text) {
                    return Err(ValidationFailure::refused_at(
                        CheckedPackageRefusalCode::InvalidPackage,
                        node_pointer(position).key("body"),
                        None,
                        node_id.clone(),
                    ));
                }
                if !text_bounds_shape(term, &graph) {
                    return Err(ValidationFailure::refused_at(
                        CheckedPackageRefusalCode::InvalidPackage,
                        node_pointer(position),
                        Some(super::CheckedPackageRefusalCause::InvalidDerivedShape),
                        node_id.clone(),
                    ));
                }
                continue;
            }
            CheckedNodeKind::BoundedDomain(
                BoundedDomainForm::IntegerRange | BoundedDomainForm::CollectionBounds,
            ) => {
                let base_valid = match kind {
                    CheckedNodeKind::BoundedDomain(BoundedDomainForm::IntegerRange) => {
                        graph.type_matches(&node.semantic_type, ScalarTypeForm::Integer)
                    }
                    _ => matches!(graph.kind(&node.semantic_type),
                        Some(CheckedNodeKind::CompositeType(
                            CompositeTypeForm::Set | CompositeTypeForm::Bag
                            | CompositeTypeForm::Sequence | CompositeTypeForm::OrderedSet
                            | CompositeTypeForm::Map))),
                };
                let natural = kind == CheckedNodeKind::BoundedDomain(BoundedDomainForm::CollectionBounds);
                base_valid && term.and_then(typed_aggregate).is_some_and(|members| {
                    let [min, max] = members else { return false; };
                    [(min, "min"), (max, "max")].into_iter().all(|(member, name)| {
                        let Some(CheckedTerm::Literal { type_id, value_kind: LiteralKind::Integer, value }) =
                            typed_binding(member, name) else { return false; };
                        let Some(text) = value.as_str() else { return false; };
                        graph.type_matches(type_id, ScalarTypeForm::Integer)
                            && (if natural { is_non_negative_integer(text) } else { is_integer_bound(text) })
                    })
                })
            }
            CheckedNodeKind::BoundedDomain(BoundedDomainForm::ModelPopulation) => {
                let set_of_reference = graph.kind(&node.semantic_type)
                    == Some(CheckedNodeKind::CompositeType(CompositeTypeForm::Set))
                    && index
                    .get(&node.semantic_type)
                    .and_then(|&position| terms.get(position).and_then(Option::as_ref))
                    .and_then(typed_aggregate)
                    .is_some_and(|members| {
                        matches!(members, [CheckedTerm::Reference { target }]
                            if graph.kind(target) == Some(CheckedNodeKind::CompositeType(CompositeTypeForm::Reference)))
                    });
                set_of_reference && term.and_then(typed_aggregate).is_some_and(|members| {
                    match members {
                        [] => true,
                        [member] => {
                            let Some(CheckedTerm::Literal { type_id, value_kind: LiteralKind::Integer, value }) =
                                typed_binding(member, "max") else { return false; };
                            value.as_str().is_some_and(is_non_negative_integer)
                                && graph.type_matches(type_id, ScalarTypeForm::Integer)
                        }
                        _ => false,
                    }
                })
            }
            CheckedNodeKind::Relation(RelationForm::Population) => {
                node.semantic_type == node.node_id
                    && node.dependencies.is_empty()
                    && node.declaration.is_none()
                    && node.recursion_group.is_none()
                    && term.and_then(typed_aggregate).is_some_and(<[_]>::is_empty)
            }
            CheckedNodeKind::ScalarType(ScalarTypeForm::CompoundUnit) => {
                node.semantic_type == node.node_id
                    && node.recursion_group.is_none()
                    && match term.and_then(typed_aggregate) {
                        Some(members) => compound_units_ascend(members, &graph, meter)?,
                        None => false,
                    }
            }
            CheckedNodeKind::ScalarType(ScalarTypeForm::CarriedQuantity) => {
                carried_quantity_shape(catalog, node, term, &graph, terms, structural_keys, meter)?
            }
            CheckedNodeKind::ScalarType(ScalarTypeForm::BoundedQuantity) => {
                bounded_quantity_shape(node, term, &graph, structural_keys, meter)?
            }
            _ => continue,
        };
        if !valid {
            let at = if kind == CheckedNodeKind::ScalarType(ScalarTypeForm::BoundedQuantity) {
                noncanonical_bounded_endpoint(term, meter)?
                    .map(|member| {
                        node_pointer(position)
                            .key("body")
                            .key("members")
                            .index(member)
                            .key("value")
                    })
                    .unwrap_or_else(|| node_pointer(position))
            } else if kind == CheckedNodeKind::CompositeType(CompositeTypeForm::Map) {
                node_pointer(position)
            } else {
                node_pointer(position).key("node_id")
            };
            return Err(ValidationFailure::refused_at(
                CheckedPackageRefusalCode::InvalidPackage,
                at,
                Some(super::CheckedPackageRefusalCause::InvalidDerivedShape),
                node_id.clone(),
            ));
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum CheckedRoundingMode {
    Exact,
    CataloguedNonExact,
}

impl CheckedRoundingMode {
    #[ix_trace_rs::string_edge]
    fn from_literal(
        catalog: &super::operation_catalog::OperationCatalog,
        value: &str,
    ) -> Option<Self> {
        if !catalog.mode_value_admitted(super::OperationModeKind::Rounding, value) {
            return None;
        }
        Some(if value == "exact" {
            Self::Exact
        } else {
            Self::CataloguedNonExact
        })
    }
}

fn carried_quantity_shape(
    catalog: &super::operation_catalog::OperationCatalog,
    node: &CheckedSemanticNodeV2,
    term: Option<&CheckedTerm>,
    graph: &Graph<'_>,
    terms: &[Option<CheckedTerm>],
    structural_keys: &[std::sync::Arc<CheckedNodeId>],
    meter: &mut WorkMeter,
) -> Result<bool, ValidationFailure> {
    let Some([unit_term, carrier_term, rounding_term]) = term
        .and_then(typed_aggregate)
        .and_then(|members| <&[CheckedTerm; 3]>::try_from(members).ok())
    else {
        return Ok(false);
    };
    let Some(unit) = typed_binding(unit_term, "unit").and_then(typed_reference) else {
        return Ok(false);
    };
    let Some(carrier) = typed_binding(carrier_term, "carrier").and_then(typed_reference) else {
        return Ok(false);
    };
    let Some(CheckedTerm::Literal {
        type_id: text_type,
        value_kind: LiteralKind::Text,
        value,
    }) = typed_binding(rounding_term, "rounding")
    else {
        return Ok(false);
    };
    let Some(rounding) = value.as_str() else {
        return Ok(false);
    };
    let Some(rounding_mode) = CheckedRoundingMode::from_literal(catalog, rounding) else {
        return Ok(false);
    };
    let Some(carrier_kind) = graph.kind(carrier) else {
        return Ok(false);
    };
    let carrier_valid = match carrier_kind {
        CheckedNodeKind::BoundedDomain(BoundedDomainForm::IntegerRange) => true,
        CheckedNodeKind::BoundedDomain(BoundedDomainForm::RationalRange) => {
            rounding_mode == CheckedRoundingMode::Exact
        }
        CheckedNodeKind::BoundedDomain(BoundedDomainForm::DecimalRange) => graph
            .index
            .get(carrier)
            .and_then(|&position| terms.get(position))
            .and_then(Option::as_ref)
            .and_then(typed_aggregate)
            .is_some_and(|members| {
                members.iter().any(|member| {
                    matches!(typed_binding(member, "rounding"), Some(CheckedTerm::Literal { value, .. })
                        if value.as_str() == Some(rounding))
                })
            }),
        _ => false,
    };
    if node.semantic_type != node.node_id
        || node.declaration.is_some()
        || node.owner.is_some()
        || node.recursion_group.is_some()
        || graph.kind(unit) != Some(CheckedNodeKind::ScalarType(ScalarTypeForm::Unit))
        || !carrier_valid
        || !graph.type_matches(text_type, ScalarTypeForm::Text)
    {
        return Ok(false);
    }
    let (Some(unit), Some(carrier), Some(text_type), Some(actual)) = (
        resolve_id_charged(unit, structural_keys, meter)?,
        resolve_id_charged(carrier, structural_keys, meter)?,
        resolve_id_charged(text_type, structural_keys, meter)?,
        resolve_id_charged(&node.node_id, structural_keys, meter)?,
    ) else {
        return Ok(false);
    };
    let expected = if meter.compare_ids(&unit, &carrier)? != std::cmp::Ordering::Greater {
        [&unit, &carrier]
    } else {
        [&carrier, &unit]
    };
    // Each listed dependency is resolved and compared in turn, not collected.
    let mut dependencies = node.dependencies.iter();
    for expected in expected {
        let Some(dependency) = dependencies.next() else {
            return Ok(false);
        };
        let Some(listed) = resolve_id_charged(dependency, structural_keys, meter)? else {
            return Ok(false);
        };
        if meter.compare_ids(&listed, expected)? != std::cmp::Ordering::Equal {
            return Ok(false);
        }
    }
    if dependencies.next().is_some() {
        return Ok(false);
    }
    let names = |at: usize, name: &str| matches!(actual.path.get(at), Some(CheckedNodePathSegment::Name(found)) if found.as_ref() == name);
    let named = actual.path.len() == 7
        && names(0, "scalar_type")
        && names(1, "carried_quantity")
        && names(4, "rounding")
        && names(6, rounding);
    Ok(named
        && names_node(&actual, 2, &unit, meter)?
        && names_node(&actual, 3, &carrier, meter)?
        && names_node(&actual, 5, &text_type, meter)?)
}

/// Whether the operand at `at` of `actual` is the node `id`, compared through
/// the meter: both sides are rebuilt keys that may nest deeply.
fn names_node(
    actual: &CheckedNodeId,
    at: usize,
    id: &CheckedNodeId,
    meter: &mut WorkMeter,
) -> Result<bool, ValidationFailure> {
    match actual.path.get(at) {
        Some(CheckedNodePathSegment::Node(found)) => {
            Ok(meter.compare_ids(found, id)? == std::cmp::Ordering::Equal)
        }
        _ => Ok(false),
    }
}

fn noncanonical_bounded_endpoint(
    term: Option<&CheckedTerm>,
    meter: &mut WorkMeter,
) -> Result<Option<usize>, ValidationFailure> {
    let Some(members) = term.and_then(typed_aggregate) else {
        return Ok(None);
    };
    for (position, name) in [(1, "min"), (2, "max")] {
        let Some(CheckedTerm::Literal { value, .. }) = members
            .get(position)
            .and_then(|member| typed_binding(member, name))
        else {
            continue;
        };
        let canonical = match value.as_str() {
            Some(text) => meter.canonical_rational(text)?.is_some(),
            None => false,
        };
        if !canonical {
            return Ok(Some(position));
        }
    }
    Ok(None)
}

fn rational_bound<'a>(
    term: &'a CheckedTerm,
    name: &str,
    graph: &Graph<'_>,
) -> Option<(&'a CheckedNodeId, &'a str)> {
    let CheckedTerm::Literal {
        type_id,
        value_kind: LiteralKind::Rational,
        value,
    } = typed_binding(term, name)?
    else {
        return None;
    };
    graph
        .type_matches(type_id, ScalarTypeForm::Rational)
        .then(|| value.as_str().map(|value| (type_id, value)))
        .flatten()
}

fn bounded_quantity_shape(
    node: &CheckedSemanticNodeV2,
    term: Option<&CheckedTerm>,
    graph: &Graph<'_>,
    structural_keys: &[std::sync::Arc<CheckedNodeId>],
    meter: &mut WorkMeter,
) -> Result<bool, ValidationFailure> {
    let Some([unit_term, min_term, max_term]) = term
        .and_then(typed_aggregate)
        .and_then(|members| <&[CheckedTerm; 3]>::try_from(members).ok())
    else {
        return Ok(false);
    };
    let Some(unit) = typed_binding(unit_term, "unit").and_then(typed_reference) else {
        return Ok(false);
    };
    let (Some((rational_type, min)), Some((max_type, max))) = (
        rational_bound(min_term, "min", graph),
        rational_bound(max_term, "max", graph),
    ) else {
        return Ok(false);
    };
    let (Some((min_num, min_den)), Some((max_num, max_den))) = (
        meter.canonical_rational(min)?,
        meter.canonical_rational(max)?,
    ) else {
        return Ok(false);
    };
    let low = meter.exact_mul(&min_num, &max_den)?;
    let high = meter.exact_mul(&max_num, &min_den)?;
    if low > high
        || rational_type != max_type
        || node.semantic_type != node.node_id
        || node.declaration.is_some()
        || node.owner.is_some()
        || node.recursion_group.is_some()
        || !graph.type_matches(unit, ScalarTypeForm::Unit)
    {
        return Ok(false);
    }
    let (Some(unit), Some(rational_type), Some(actual)) = (
        resolve_id_charged(unit, structural_keys, meter)?,
        resolve_id_charged(rational_type, structural_keys, meter)?,
        resolve_id_charged(&node.node_id, structural_keys, meter)?,
    ) else {
        return Ok(false);
    };
    let (first, second) =
        if meter.compare_ids(&unit, &rational_type)? != std::cmp::Ordering::Greater {
            (&unit, &rational_type)
        } else {
            (&rational_type, &unit)
        };
    let mut dependencies = node.dependencies.iter();
    let mut listed = true;
    for expected in [first, second] {
        let Some(dependency) = dependencies.next() else {
            listed = false;
            break;
        };
        let Some(held) = resolve_id_charged(dependency, structural_keys, meter)? else {
            listed = false;
            break;
        };
        if meter.compare_ids(&held, expected)? != std::cmp::Ordering::Equal {
            listed = false;
            break;
        }
    }
    listed = listed && dependencies.next().is_none();
    let names = |at: usize, name: &str| matches!(actual.path.get(at), Some(CheckedNodePathSegment::Name(found)) if found.as_ref() == name);
    let named = listed
        && actual.path.len() == 9
        && names(0, "scalar_type")
        && names(1, "bounded_quantity")
        && names(3, "min")
        && names(5, min)
        && names(6, "max")
        && names(8, max);
    Ok(named
        && names_node(&actual, 2, &unit, meter)?
        && names_node(&actual, 4, &rational_type, meter)?
        && names_node(&actual, 7, &rational_type, meter)?)
}

/// Join an admitted bounded type's actual members to the selected Semantic IR
/// declaration. A structural node ID names the type but does not attest to
/// its body: keeping the ID while changing a bound must not change the type
/// selected by a model field.
pub(super) fn validate_selected_type_bounds(
    models: &[DomainModel],
    terms: &[Option<CheckedTerm>],
    structural_keys: &[std::sync::Arc<CheckedNodeId>],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let mut selected = Vec::new();
    for model in models {
        for bounds in model.value_types.values().flatten() {
            let member_type = match bounds {
                ValueBounds::Integer(expected) => MemberType::IntRange(*expected),
                ValueBounds::Rational(expected, _) => MemberType::RationalRange(*expected),
                ValueBounds::Text(_) => continue,
            };
            let Some(id) = member_type.node_id_in(
                &model.checked_package,
                &mut super::model_members::Budget::new(meter, model.root, model.bytes),
            )?
            else {
                continue;
            };
            meter.reserve(super::budgeted_read::arc_bytes::<CheckedNodeId>())?;
            meter.push(&mut selected, (bounds, std::sync::Arc::new(id)))?;
        }
    }
    if selected.is_empty() {
        return Ok(());
    }

    // Tokens compare one path segment at a time. Comparing every rebuilt key
    // structurally would repeatedly descend a long operand chain.
    let mut selected_keys = meter.vec(selected.len())?;
    selected_keys.extend(selected.iter().map(|(_, id)| std::sync::Arc::clone(id)));
    // The order wants every operand of a listed key listed too: the selected
    // keys' own operands follow them, each allocation once.
    let mut listed: Vec<*const CheckedNodeId> = Vec::new();
    let mut pending: Vec<std::sync::Arc<CheckedNodeId>> = Vec::new();
    for initial in 0..selected.len() {
        let key = std::sync::Arc::clone(&selected_keys[initial]);
        meter.push(&mut pending, key)?;
        while let Some(key) = pending.pop() {
            if !meter.insert_sorted(&mut listed, std::sync::Arc::as_ptr(&key))? {
                continue;
            }
            for segment in &key.path {
                if let CheckedNodePathSegment::Node(operand) = segment {
                    meter.push(&mut pending, std::sync::Arc::clone(operand))?;
                    meter.push(&mut selected_keys, std::sync::Arc::clone(operand))?;
                }
            }
        }
    }
    let order = meter
        .structural_order(&[structural_keys, selected_keys.as_slice()])?
        .expect("reader-rebuilt keys and selected member keys contain only resolved operands");
    // A token is at most the number of keys the order was built over.
    let mut index = meter.filled(structural_keys.len() + selected_keys.len(), None)?;
    for (position, key) in structural_keys.iter().enumerate() {
        let token = order.token(key).expect("graph keys are in the order");
        index[token] = Some(position);
    }
    let mut selected_tokens = meter.vec(selected.len())?;
    for key in selected_keys.iter().take(selected.len()) {
        selected_tokens.push(order.token(key).expect("selected keys are in the order"));
    }
    for ((expected, id), token) in selected.into_iter().zip(selected_tokens) {
        let Some(&Some(position)) = index.get(token) else {
            continue;
        };
        let actual = terms
            .get(position)
            .and_then(Option::as_ref)
            .and_then(typed_aggregate)
            .and_then(|members| {
                let bound = |term, name| match typed_binding(term, name)? {
                    CheckedTerm::Literal {
                        value_kind: LiteralKind::Integer,
                        value,
                        ..
                    } => value.as_str()?.parse::<i128>().ok(),
                    _ => None,
                };
                match (&expected, members) {
                    (ValueBounds::Integer(_), [minimum, maximum]) => {
                        Some(([bound(minimum, "min")?, bound(maximum, "max")?, 0, 0], 2))
                    }
                    (ValueBounds::Rational(..), [nmin, nmax, dmin, dmax]) => Some((
                        [
                            bound(nmin, "numerator_min")?,
                            bound(nmax, "numerator_max")?,
                            bound(dmin, "denominator_min")?,
                            bound(dmax, "denominator_max")?,
                        ],
                        4,
                    )),
                    _ => None,
                }
            });
        let expected_values = match expected {
            ValueBounds::Integer(bounds) => ([bounds.lower, bounds.upper, 0, 0], 2),
            ValueBounds::Rational(bounds, _) => (
                [
                    bounds.numerator_min,
                    bounds.numerator_max,
                    bounds.denominator_min,
                    bounds.denominator_max,
                ],
                4,
            ),
            ValueBounds::Text(_) => continue,
        };
        if actual != Some(expected_values) {
            return Err(ValidationFailure::refused_at(
                CheckedPackageRefusalCode::InvalidPackage,
                node_pointer(position).key("node_id"),
                Some(super::CheckedPackageRefusalCause::InvalidDerivedShape),
                id.as_ref().clone(),
            ));
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
    if !target.is_well_formed() {
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
        && graph
            .node(&node.semantic_type)
            .is_none_or(|(type_node, kind)| {
                // An absent target is a dependent graph edge checked after
                // owner selection, not an independent parameter-shape defect.
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
            });
    if !is_binder_type {
        return Some(Defect::SemanticType);
    }
    if term
        .and_then(|term| {
            typed_parameter_binding_with(term, &|id, form| graph.type_matches(id, form))
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
        super::CheckedTerm::Reference { target } if target.is_well_formed() => Some(target),
        _ => None,
    }
}

/// Borrow a reference from the strict reader's authenticated term table.
/// Its node ID was already checked by the retained-document decoder. Unlike
/// `typed_reference`, this must not be used for arbitrary caller-built terms.
pub(super) fn typed_admitted_reference(term: &super::CheckedTerm) -> Option<&CheckedNodeId> {
    match term {
        super::CheckedTerm::Reference { target } => Some(target),
        _ => None,
    }
}

fn typed_literal<'a>(
    term: &'a super::CheckedTerm,
    value_kind: LiteralKind,
    type_form: ScalarTypeForm,
    type_matches: &impl Fn(&CheckedNodeId, ScalarTypeForm) -> bool,
) -> Option<&'a super::CheckedJson> {
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
        .and_then(super::CheckedJson::as_str)
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
        .and_then(super::CheckedJson::as_str)
        .filter(|level| is_non_negative_integer(level))?;
    Some((name, level))
}

/// Graph-shape dependency control for compound units. Their semantic body
/// shape is checked at the later derived-shape stage.
fn compound_unit_dependencies_defect(
    node: &CheckedSemanticNodeV2,
    term: Option<&super::CheckedTerm>,
    graph: &Graph<'_>,
    meter: &mut WorkMeter,
) -> Result<Option<Defect>, ValidationFailure> {
    let Some(terms) = term.and_then(typed_aggregate) else {
        return Ok(None);
    };
    let mut units: Vec<&CheckedNodeId> = meter.vec(terms.len())?;
    for term in terms {
        match typed_compound_unit_term(term, graph, meter)? {
            Some(unit) => units.push(unit),
            None => return Ok(None),
        }
    }
    let mut same = node.dependencies.len() == units.len();
    for (listed, unit) in node.dependencies.iter().zip(&units) {
        if !same {
            break;
        }
        same = meter.compare_ids(listed, unit)?.is_eq();
    }
    Ok((!same).then_some(Defect::Dependencies))
}

/// Whether the members of a compound unit name root units in strictly
/// ascending order of their IDs.
fn compound_units_ascend(
    members: &[super::CheckedTerm],
    graph: &Graph<'_>,
    meter: &mut WorkMeter,
) -> Result<bool, ValidationFailure> {
    let mut previous: Option<&CheckedNodeId> = None;
    for member in members {
        let Some(unit) = typed_compound_unit_term(member, graph, meter)? else {
            return Ok(false);
        };
        if let Some(last) = previous {
            if !meter.compare_ids(last, unit)?.is_lt() {
                return Ok(false);
            }
        }
        previous = Some(unit);
    }
    Ok(true)
}

/// The unit a compound-unit member names, when the member is a unit of
/// exponent other than zero whose dimension has it as the canonical root.
fn typed_compound_unit_term<'t>(
    term: &'t super::CheckedTerm,
    graph: &Graph<'_>,
    meter: &mut WorkMeter,
) -> Result<Option<&'t CheckedNodeId>, ValidationFailure> {
    let Some([unit, exponent]) = typed_aggregate(term) else {
        return Ok(None);
    };
    let Some(unit) = typed_binding(unit, "unit").and_then(typed_reference) else {
        return Ok(None);
    };
    // A dimension's canonical root has no target. Source units carry that
    // fact in their nominal body; model units carry it in the selected model.
    let is_root_unit = match graph.node_at(unit) {
        Some((position, node, CheckedNodeKind::ScalarType(ScalarTypeForm::Unit))) => {
            match node.owner.as_ref() {
                Some(super::CheckedNodeOwner::Source { .. }) => {
                    super::identity::nominal_body_if_closed(position, meter)?.is_some_and(|body| {
                        matches!(
                            body.kind(),
                            super::identity::NominalBodyKind::Unit {
                                target_unit_node_id: None,
                                ..
                            }
                        )
                    })
                }
                Some(super::CheckedNodeOwner::Model {
                    identity,
                    node: original,
                }) => graph.models.is_some_and(|models| {
                    models.iter().any(|model| {
                        model.identity == *identity
                            && model
                                .units
                                .get(original)
                                .is_some_and(|unit| unit.target.is_none())
                    })
                }),
                _ => false,
            }
        }
        _ => false,
    };
    let exponent_ok = typed_binding(exponent, "exponent")
        .and_then(|value| {
            typed_literal(
                value,
                LiteralKind::Integer,
                ScalarTypeForm::Integer,
                &|id, form| graph.type_matches(id, form),
            )
        })
        .and_then(super::CheckedJson::as_str)
        .is_some_and(is_nonzero_integer);
    Ok((is_root_unit && exponent_ok).then_some(unit))
}

/// Whether a state clause's `execution_point` is a closed point: a `kind`
/// and the one nonempty name its variant carries, and nothing else.
fn is_closed_execution_point(point: TermNode<'_>) -> bool {
    let Some(kind) = point.get("kind").and_then(TermNode::as_str) else {
        return false;
    };
    let field = match kind {
        "initialization" | "handler" => "name",
        "pre" | "post" => "operation",
        _ => return false,
    };
    point.exact_members(&["kind", field])
        && point
            .get(field)
            .and_then(TermNode::as_str)
            .is_some_and(|name| !name.is_empty())
}

/// A `state`/`operation_anchor` body (QSpec FR-342), borrowed from its term.
pub(super) struct AnchorBody<'a> {
    /// The object type declaring the operation.
    pub(super) context: &'a CheckedNodeId,
    /// The operation's name.
    pub(super) operation: &'a str,
    /// The operation's frame node.
    pub(super) frame: &'a CheckedNodeId,
    /// Optional point, retained separately from the three expression terms.
    pub(super) execution_point: Option<&'a ExecutionPoint>,
}

/// The admitted anchor body, or `None` for any other typed shape.
pub(super) fn typed_anchor_body(body: &super::CheckedTerm) -> Option<AnchorBody<'_>> {
    let (members, execution_point) = match body {
        CheckedTerm::OperationAnchorBody(anchor) => {
            (&anchor.members[..], anchor.execution_point.as_ref())
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
        context: typed_binding(context, "context").and_then(typed_reference)?,
        operation: value.as_str().filter(|name| is_identifier(name))?,
        frame: typed_binding(frame, "frame").and_then(typed_reference)?,
        execution_point,
    })
}

/// The catalogued identity of the state clause operation.
/// A `state`/`state_clause` body (QSpec FR-341), borrowed from its term.
pub(super) struct ClauseBody<'a> {
    /// The clause kind its `state_clause` member names.
    pub(super) clause: StateClauseKind,
    /// The parameter aggregate, every member a reference.
    parameters: &'a [super::CheckedTerm],
    /// The anchor reference's target.
    pub(super) anchor: &'a CheckedNodeId,
    /// Optional closed point of the clause application member.
    pub(super) execution_point: Option<&'a ExecutionPoint>,
}

impl<'a> ClauseBody<'a> {
    /// The number of parameters.
    pub(super) fn parameter_count(&self) -> usize {
        self.parameters.len()
    }

    /// The parameter aggregate's reference targets, in order.
    pub(super) fn parameters(&self) -> impl Iterator<Item = &'a CheckedNodeId> + '_ {
        self.parameters
            .iter()
            .filter_map(|term| typed_reference(term))
    }
}

/// The five identity-bearing references of a Boolean state transition,
/// borrowed from its term.
pub(super) struct TransitionBody<'a> {
    pub(super) anchor: &'a CheckedNodeId,
    pub(super) subject: &'a CheckedNodeId,
    inputs: &'a [super::CheckedTerm],
    pub(super) pre: Option<&'a CheckedNodeId>,
    pub(super) post: &'a CheckedNodeId,
}

impl<'a> TransitionBody<'a> {
    /// The number of inputs.
    pub(super) fn input_count(&self) -> usize {
        self.inputs.len()
    }

    /// The input reference targets, in order.
    pub(super) fn inputs(&self) -> impl Iterator<Item = &'a CheckedNodeId> + '_ {
        self.inputs.iter().filter_map(|term| typed_reference(term))
    }
}

/// Decode the closed transition argument shape without doing its semantic joins.
pub(super) fn typed_transition_body(body: &super::CheckedTerm) -> Option<TransitionBody<'_>> {
    let super::CheckedTerm::Application {
        operator: ApplicationOperator::StateTransition,
        operation,
        arguments,
        ..
    } = body
    else {
        return None;
    };
    if super::KnownOperation::from_wire(operation.identity()?)
        != Some(super::KnownOperation::StateTransition)
    {
        return None;
    }
    let [anchor, subject, inputs, pre, post] = arguments.as_slice() else {
        return None;
    };
    let anchor = typed_binding(anchor, "anchor").and_then(typed_reference)?;
    let subject = typed_binding(subject, "subject").and_then(typed_reference)?;
    let inputs = typed_aggregate(typed_binding(inputs, "inputs")?)?;
    if !inputs.iter().all(|term| typed_reference(term).is_some()) {
        return None;
    }
    let pre = typed_aggregate(typed_binding(pre, "pre")?)?;
    let pre = match pre {
        [] => None,
        [one] => Some(typed_reference(one)?),
        _ => return None,
    };
    let post = typed_binding(post, "post").and_then(typed_reference)?;
    Some(TransitionBody {
        anchor,
        subject,
        inputs,
        pre,
        post,
    })
}

/// The admitted state clause body, or `None` for any other shape: an
/// application of `quire.op.state.clause` with operator class
/// `state_clause`, member exactly `{kind: "state_clause", clause}`, and
/// arguments an `aggregate` of one or more `reference` terms, a `reference`
/// and the condition.
// Decodes a state clause member's kind and clause.
pub(super) fn typed_clause_body(body: &super::CheckedTerm) -> Option<ClauseBody<'_>> {
    let super::CheckedTerm::Application {
        operator: ApplicationOperator::StateClause,
        operation,
        arguments,
        ..
    } = body
    else {
        return None;
    };
    if super::KnownOperation::from_wire(operation.identity()?)
        != Some(super::KnownOperation::StateClause)
    {
        return None;
    }
    let member = operation.member_node()?;
    if !(member.exact_members(&["kind", "clause"])
        || member.exact_members(&["kind", "clause", "execution_point"]))
    {
        return None;
    }
    if member
        .get("kind")
        .and_then(super::TermNode::as_str)
        .and_then(OperationMemberKind::from_wire)
        != Some(OperationMemberKind::StateClause)
    {
        return None;
    }
    let execution_point = operation.execution_point();
    let clause = StateClauseKind::from_wire(member.get("clause")?.as_str()?)?;
    let [parameters, anchor, _condition] = arguments.as_slice() else {
        return None;
    };
    let parameters = typed_aggregate(parameters)?;
    if parameters.is_empty()
        || !parameters
            .iter()
            .all(|term| typed_reference(term).is_some())
    {
        return None;
    }
    Some(ClauseBody {
        clause,
        parameters,
        anchor: typed_reference(anchor)?,
        execution_point,
    })
}

/// A `composite_type`/`reference` type node's target, borrowed from its typed
/// body: exactly `aggregate{[reference(target)]}`.
pub(super) fn reference_type_target_ref(term: &super::CheckedTerm) -> Option<&CheckedNodeId> {
    let super::CheckedTerm::Aggregate { members } = term else {
        return None;
    };
    let [one] = members.as_slice() else {
        return None;
    };
    typed_reference(one)
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
pub(super) fn typed_union_type_body<'a>(
    body: &'a super::CheckedTerm,
    meter: &mut WorkMeter,
) -> Result<Option<Vec<UnionMember<'a>>>, ValidationFailure> {
    let Some(members) = typed_aggregate(body).filter(|members| !members.is_empty()) else {
        return Ok(None);
    };
    // Establish the closed shape without allocating partial output.
    for member in members {
        let super::CheckedTerm::Binding { name, value } = member else {
            return Ok(None);
        };
        if !is_identifier(name) {
            return Ok(None);
        }
        let Some(payload) = typed_aggregate(value) else {
            return Ok(None);
        };
        if payload
            .iter()
            .any(|term| typed_admitted_reference(term).is_none())
        {
            return Ok(None);
        }
    }
    let mut output = meter.vec(members.len())?;
    for member in members {
        let super::CheckedTerm::Binding { name, value } = member else {
            unreachable!("shape checked above")
        };
        let entries = typed_aggregate(value).expect("shape checked above");
        let mut payload = meter.vec(entries.len())?;
        for entry in entries {
            let id = typed_admitted_reference(entry).expect("shape checked above");
            payload.push(meter.clone_id(id)?);
        }
        output.push(UnionMember { name, payload });
    }
    Ok(Some(output))
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
/// structural-id-ascending reference targets and member declarations, plus the member
/// `population` of a reaches or navigate application. A member `declaration` or
/// `population` that is not a node key is skipped here: the operation stage
/// owns that refusal and its path.
#[cfg(test)]
pub(super) fn application_dependencies(
    body: &super::CheckedTerm,
    meter: &mut WorkMeter,
) -> Result<Option<Vec<CheckedNodeId>>, ValidationFailure> {
    let mut join = Vec::new();
    Ok(collect_application_dependencies(body, &mut join, meter)?.then_some(join))
}

/// Adds the join of [`application_dependencies`] to the sorted `join` and
/// says whether `body` holds an application. Every table is charged before
/// it is made; an ID read out of an operation member is charged as soon as it
/// is parsed (the parse itself is the document-value bridge).
pub(super) fn collect_application_dependencies(
    body: &super::CheckedTerm,
    join: &mut Vec<CheckedNodeId>,
    meter: &mut WorkMeter,
) -> Result<bool, ValidationFailure> {
    use super::CheckedTerm;
    let mut contains_application = false;
    let mut pending: Vec<&super::CheckedTerm> = meter.vec(1)?;
    pending.push(body);
    while let Some(term) = pending.pop() {
        match term {
            CheckedTerm::Reference { target } => {
                meter.insert_id(join, target)?;
            }
            CheckedTerm::Application {
                operation,
                arguments,
                ..
            } => {
                contains_application = true;
                meter.room(&mut pending, arguments.len())?;
                pending.extend(arguments);
                let member_id = |key: &str| operation.member_key(key);
                if let Some(id) = member_id("declaration") {
                    meter.insert_id(join, id)?;
                }
                if operation.member_kind() == Some(OperationMemberKind::Fairness) {
                    let ids = &operation.record().member_ids;
                    for id in [
                        ids.target_declaration.as_ref(),
                        ids.target_node.as_ref(),
                        ids.target_parallel.as_ref(),
                    ]
                    .into_iter()
                    .flatten()
                    {
                        meter.insert_id(join, id)?;
                    }
                }
                // Only a relationship application names a population.
                let relationship = operation.identity().is_some_and(|identity| {
                    matches!(
                        super::KnownOperation::from_wire(identity),
                        Some(
                            super::KnownOperation::ModelReaches
                                | super::KnownOperation::ModelNavigate
                        )
                    )
                });
                if relationship {
                    if let Some(id) = member_id("population") {
                        meter.insert_id(join, id)?;
                    }
                }
            }
            CheckedTerm::Aggregate { members } => {
                meter.room(&mut pending, members.len())?;
                pending.extend(members);
            }
            CheckedTerm::OperationAnchorBody(body) => {
                meter.room(&mut pending, body.members.len())?;
                pending.extend(&body.members);
            }
            CheckedTerm::Binding { value, .. } => meter.push(&mut pending, value.as_ref())?,
            // These bodies do not contribute local application dependencies.
            CheckedTerm::Literal { .. }
            | CheckedTerm::DependencyReference { .. }
            | CheckedTerm::Frame { .. }
            | CheckedTerm::AbstractionRelation { .. }
            | CheckedTerm::ImplementationGuarantee { .. }
            | CheckedTerm::ComponentComposition { .. }
            | CheckedTerm::Refinement { .. }
            | CheckedTerm::ExtensionForm { .. }
            | CheckedTerm::NominalBody(_) => {}
        }
    }
    Ok(contains_application)
}

/// [`application_dependencies`] for the body of graph node `position`, read
/// from the retained document: every ID is charged by the node view before
/// it is allocated, with the node's operand slots read through its row.
pub(super) fn application_dependencies_at(
    position: usize,
    meter: &mut WorkMeter,
) -> Result<Option<Vec<CheckedNodeId>>, ValidationFailure> {
    let Some((body, row)) = meter.body(position) else {
        return Ok(None);
    };
    let mut join = Vec::new();
    let mut pending: Vec<TermNode<'_>> = meter.vec(1)?;
    pending.push(body);
    let mut contains_application = false;
    while let Some(term) = pending.pop() {
        match term.view() {
            Some(TermView::Reference { target }) => {
                if let Some(id) = meter.read_id(target, row)?.valid() {
                    meter.insert_owned_id(&mut join, id)?;
                }
            }
            Some(TermView::Application {
                operation,
                arguments,
                ..
            }) => {
                contains_application = true;
                meter.room(&mut pending, arguments.len())?;
                pending.extend(arguments);
                let member = operation.get("member");
                if let Some(id) = member_id(member, "declaration", row, meter)? {
                    meter.insert_owned_id(&mut join, id)?;
                }
                if member
                    .and_then(|member| member.get("kind"))
                    .and_then(TermNode::as_str)
                    == Some("fairness")
                {
                    if let Some(target) = member.and_then(|member| member.get("target")) {
                        for field in ["declaration", "node", "parallel"] {
                            if let Some(id) = member_id(Some(target), field, row, meter)? {
                                meter.insert_owned_id(&mut join, id)?;
                            }
                        }
                    }
                }
                // Only a relationship application names a population.
                let relationship = operation
                    .get("identity")
                    .and_then(TermNode::as_str)
                    .is_some_and(|identity| {
                        matches!(
                            super::KnownOperation::from_wire(identity),
                            Some(
                                super::KnownOperation::ModelReaches
                                    | super::KnownOperation::ModelNavigate
                            )
                        )
                    });
                if relationship {
                    if let Some(id) = member_id(member, "population", row, meter)? {
                        meter.insert_owned_id(&mut join, id)?;
                    }
                }
            }
            Some(TermView::Aggregate { members }) => {
                meter.room(&mut pending, members.len())?;
                pending.extend(members);
            }
            Some(TermView::Binding { value, .. }) => meter.push(&mut pending, value)?,
            Some(TermView::Literal { .. }) | None => {}
        }
    }
    Ok(contains_application.then_some(join))
}

/// The valid node ID in member `key` of `holder`, charged by the node view.
fn member_id(
    holder: Option<TermNode<'_>>,
    key: &str,
    row: &[u64],
    meter: &mut WorkMeter,
) -> Result<Option<CheckedNodeId>, ValidationFailure> {
    match holder.and_then(|holder| holder.get(key)) {
        Some(value) => Ok(meter.read_id(value, row)?.valid()),
        None => Ok(None),
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
    use crate::checked_package::shared::checked_node_comparisons;
    use ix_trace_rs::trace;
    use serde_json::{json, Value};
    use std::collections::BTreeSet;

    #[trace("TC-472", "FR-038-AC-511", "FR-038-AC-512")]
    #[test]
    fn structural_defects_prepay_their_exact_payload_and_preserve_loci() {
        use crate::checked_package::v2::budgeted_read::{
            CheckedPackagePendingOutcome, PackageReadResource,
        };
        use crate::{NativeJoinBudget, NativeJoinLocusSource, PackageReadStage};
        let id: CheckedNodeId = serde_json::from_value(super::super::test_support::id('a'))
            .expect("the original fixture node identity");
        for (defect, expected) in [
            (
                Defect::SemanticType,
                "/semantic_graph/nodes/3/semantic_type",
            ),
            (Defect::Dependencies, "/semantic_graph/nodes/3/dependencies"),
            (
                Defect::RecursionGroup,
                "/semantic_graph/nodes/3/recursion_group",
            ),
            (Defect::Occurrences, "/semantic_graph/nodes/3/occurrences"),
            (Defect::Body, "/semantic_graph/nodes/3/body"),
            (
                Defect::ClausePoint,
                "/semantic_graph/nodes/3/body/operation/member/execution_point",
            ),
            (
                Defect::Role(12),
                "/semantic_graph/nodes/3/occurrences/12/role",
            ),
        ] {
            let observe = |limit| {
                let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
                let mut result = None;
                let requested = allocation_counter::measure(|| {
                    let mut meter = WorkMeter::over(&mut budget);
                    result = Some(structural_refusal(
                        &mut meter,
                        3,
                        "expression",
                        CheckedPackageRefusalCode::InvalidSemanticGraph,
                        defect,
                        Some(&id),
                    ));
                })
                .bytes_total;
                (
                    result.expect("structural diagnostic ran"),
                    budget,
                    requested,
                )
            };
            let (ample, budget, requested) = observe(u64::MAX);
            assert_eq!(
                requested,
                budget.read_storage_consumed(),
                "{defect:?}: entire diagnostic prepaid"
            );
            let ValidationFailure::Refused(refusal) = &ample else {
                panic!("{defect:?}: semantic refusal: {ample:?}");
            };
            assert_eq!(
                refusal.code,
                CheckedPackageRefusalCode::InvalidSemanticGraph
            );
            assert_eq!(refusal.cause, None);
            assert_eq!(
                refusal.loci[0].path.as_ref().map(|path| path.as_str()),
                Some(expected)
            );
            assert_eq!(refusal.locus.as_ref(), Some(&id));
            let (exact, exact_budget, exact_requested) = observe(requested);
            assert_eq!(exact, ample);
            assert_eq!(exact_requested, requested);
            assert_eq!(exact_budget.read_storage_consumed(), requested);
            let (short, mut short_budget, short_requested) = observe(requested - 1);
            let (repeat, repeat_budget, repeat_requested) = observe(requested - 1);
            assert_eq!(short, repeat);
            assert_eq!(short_budget, repeat_budget);
            assert_eq!(short_requested, repeat_requested);
            assert_eq!(short_requested, short_budget.read_storage_consumed());
            let ValidationFailure::Budget(stop) = short else {
                panic!("{defect:?}: denied diagnostic: {short:?}");
            };
            assert_eq!(stop.resource, PackageReadResource::ReadStorage);
            assert!(
                stop.path.is_none(),
                "the unpaid diagnostic builds no pointer"
            );
            let locus = short_budget.first_unpaid_locus().expect("diagnostic locus");
            assert_eq!(
                (locus.source, locus.form),
                (
                    NativeJoinLocusSource::PackageRead(PackageReadStage::Graph, 3),
                    "diagnostic"
                )
            );
            assert_eq!(
                short_budget.take_pending(),
                Some(CheckedPackagePendingOutcome::Refused(
                    CheckedPackageRefusalCode::InvalidSemanticGraph
                ))
            );
        }
    }

    #[trace("TC-472", "FR-038-AC-511")]
    #[test]
    fn union_member_projection_prepays_exact_tables_and_ids() {
        let id = super::super::test_support::id('a');
        let body = super::super::CheckedTerm::decode_admitted(&json!({
            "term":"aggregate", "members":[
                {"term":"binding","name":"One","value":{"term":"aggregate","members":[
                    {"term":"reference","target":id}
                ]}},
                {"term":"binding","name":"Empty","value":{"term":"aggregate","members":[]}}
            ]
        }))
        .expect("closed union body");
        let mut body = body;
        let mut target = CheckedNodeId {
            package: "deep-union".into(),
            path: vec![CheckedNodePathSegment::Name("base".into())],
            ordinal: 0,
        };
        for _ in 0..512 {
            target = CheckedNodeId {
                package: "deep-union".into(),
                path: vec![CheckedNodePathSegment::Node(std::sync::Arc::new(target))],
                ordinal: 0,
            };
        }
        assert!(target.is_well_formed());
        let super::super::CheckedTerm::Aggregate { members } = &mut body else {
            unreachable!()
        };
        let super::super::CheckedTerm::Binding { value, .. } = &mut members[0] else {
            unreachable!()
        };
        let super::super::CheckedTerm::Aggregate { members } = value.as_mut() else {
            unreachable!()
        };
        members[0] = super::super::CheckedTerm::Reference { target };
        let run = |limit| {
            let mut budget =
                crate::NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut result = None;
            let measured = allocation_counter::measure(|| {
                let mut meter = WorkMeter::over(&mut budget);
                meter.at_node(3, "union");
                result = Some(typed_union_type_body(&body, &mut meter));
            });
            (
                result.expect("ran"),
                measured.bytes_total,
                budget.read_storage_consumed(),
            )
        };
        let (result, allocated, charged) = run(u64::MAX);
        assert_eq!(allocated, charged);
        let members = result.expect("ample").expect("closed union");
        assert_eq!(
            members.iter().map(|member| member.name).collect::<Vec<_>>(),
            ["One", "Empty"]
        );
        assert_eq!(members[0].payload.len(), 1);
        assert!(members[1].payload.is_empty());
        let (exact, allocated, consumed) = run(charged);
        assert_eq!(allocated, consumed);
        assert!(exact.expect("exact").is_some());
        let (short, allocated, consumed) = run(charged - 1);
        assert_eq!(allocated, consumed);
        assert!(short.is_err());
    }

    #[trace("TC-048", "FR-038-AC-380")]
    #[test]
    fn absent_dependency_uses_shallow_order_on_a_deep_index_chain() {
        const LENGTH: usize = 4_096;
        let base = CheckedNodeId {
            package: "structural-test".into(),
            path: vec![CheckedNodePathSegment::Name("base".into())],
            ordinal: 0,
        };
        let mut wire = Vec::with_capacity(LENGTH);
        let mut structural = Vec::with_capacity(LENGTH);
        wire.push(base.clone());
        structural.push(std::sync::Arc::new(base));
        for position in 1..LENGTH {
            wire.push(CheckedNodeId {
                package: "structural-test".into(),
                path: vec![
                    CheckedNodePathSegment::Name("derived".into()),
                    CheckedNodePathSegment::Index(u64::try_from(position - 1).expect("index")),
                ],
                ordinal: 0,
            });
            structural.push(std::sync::Arc::new(CheckedNodeId {
                package: "structural-test".into(),
                path: vec![
                    CheckedNodePathSegment::Name("derived".into()),
                    CheckedNodePathSegment::Node(std::sync::Arc::clone(&structural[position - 1])),
                ],
                ordinal: 0,
            }));
        }
        let index = wire
            .iter()
            .enumerate()
            .map(|(position, id)| (id, position))
            .collect();
        let absent = CheckedNodeId {
            package: "structural-test".into(),
            path: vec![
                CheckedNodePathSegment::Name("missing".into()),
                CheckedNodePathSegment::Index(u64::try_from(LENGTH - 1).expect("index")),
            ],
            ordinal: 0,
        };
        let mut actual = wire.clone();
        actual.push(absent.clone());
        let expected: Vec<&CheckedNodeId> = wire.iter().chain(std::iter::once(&absent)).collect();
        checked_node_comparisons::reset();
        let mut meter = WorkMeter::new(u64::MAX);
        let keys = DependencyKeys::new(
            std::iter::once(actual.as_slice()),
            &index,
            &structural,
            &mut meter,
        )
        .expect("keys build");
        let order = DependencyKeyOrder::new(&keys, &mut meter).expect("order builds");
        assert_eq!(
            order
                .matches(&actual, &expected, &index, &structural, &mut meter)
                .ok(),
            Some(true)
        );
        assert!(
            checked_node_comparisons::count() < 64 * LENGTH as u64,
            "one missing target must not compare the shared chain repeatedly: {}",
            checked_node_comparisons::count()
        );
        assert_eq!(
            order
                .matches(
                    &actual,
                    &wire.iter().collect::<Vec<_>>(),
                    &index,
                    &structural,
                    &mut meter
                )
                .ok(),
            Some(false)
        );
        let wrong = CheckedNodeId {
            package: "structural-test".into(),
            path: vec![
                CheckedNodePathSegment::Name("other".into()),
                CheckedNodePathSegment::Index(u64::try_from(LENGTH - 1).expect("index")),
            ],
            ordinal: 0,
        };
        assert_eq!(
            order
                .matches(
                    &actual,
                    &wire
                        .iter()
                        .chain(std::iter::once(&wrong))
                        .collect::<Vec<_>>(),
                    &index,
                    &structural,
                    &mut meter,
                )
                .ok(),
            Some(false)
        );
    }

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
        json!({"package": "structural-test", "path": [fill.to_string()], "ordinal": 0})
    }

    fn reference_selector(term: &Value) -> Option<CheckedNodeId> {
        match CheckedTerm::decode_admitted(term).ok()? {
            CheckedTerm::Reference { target } => Some(target),
            _ => None,
        }
    }

    #[trace("TC-048", "FR-038-AC-22")]
    #[test]
    fn tc_048_structural_reference_rejects_an_extra_wire_member() {
        let target = id('a');
        let closed = json!({"term": "reference", "target": target});
        assert_eq!(
            reference_selector(&closed),
            CheckedNodeId::from_value(&target)
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
        let mut meter = WorkMeter::new(u64::MAX);
        let join = application_dependencies(&term, &mut meter)
            .ok()
            .flatten()
            .expect("the body holds an application");
        let expected: Vec<CheckedNodeId> = ['a', 'b', 'c']
            .into_iter()
            .map(|fill| CheckedNodeId::from_value(&id(fill)).expect("node id"))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        assert_eq!(join, expected);
        // The join's blocks are requested before they are made: the measured
        // requirement admits and one byte below is denied.
        let measure = |limit: u64| {
            let mut budget =
                crate::NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut meter = WorkMeter::over(&mut budget);
            let joined = application_dependencies(&term, &mut meter);
            (joined.ok(), budget.read_storage_consumed())
        };
        let (found, spent) = measure(u64::MAX);
        assert_eq!(found, Some(Some(expected.clone())));
        assert!(spent > 0);
        assert_eq!(measure(spent).0, found, "the measured limit admits");
        assert_eq!(measure(spent - 1).0, None, "one byte short is denied");
        assert_eq!(
            application_dependencies(
                &super::super::CheckedTerm::decode_admitted(
                    &json!({"term": "reference", "target": id('a')}),
                )
                .expect("closed reference"),
                &mut meter,
            )
            .ok(),
            Some(None)
        );
        // An unparseable member declaration is left to the operation stage.
        let mut malformed = body.clone();
        malformed["operation"]["member"]["declaration"] = json!("not a node key");
        let expected: Vec<CheckedNodeId> = ['a', 'b']
            .into_iter()
            .map(|fill| CheckedNodeId::from_value(&id(fill)).expect("node id"))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let malformed =
            super::super::CheckedTerm::decode_admitted(&malformed).expect("closed application");
        assert_eq!(
            application_dependencies(&malformed, &mut meter).ok(),
            Some(Some(expected))
        );
    }

    /// The join of a node body, read from the retained document, is the join
    /// the typed term gives on the same body with its operand slots resolved,
    /// and is charged to the byte: the allocator is asked for what the
    /// budget is charged, the measured limit admits and one byte below is
    /// denied (FR-038-AC-504).
    #[trace("TC-472", "FR-038-AC-504")]
    #[test]
    fn application_join_reads_the_document_and_allocates_what_it_charges() {
        let slot = |k: u64, fill: char| json!({"package": "structural-test", "path": [fill.to_string(), {"operand": k}], "ordinal": 0});
        let node = |index: u64, fill: char| json!({"package": "structural-test", "path": [fill.to_string(), {"node": index}], "ordinal": 0});
        let body = |slotted: bool| {
            let at = |k: u64, fill: char| {
                if slotted {
                    slot(k, fill)
                } else {
                    node(10 + k, fill)
                }
            };
            json!({
                "term": "application",
                "operator": "query",
                "operation": {"identity": "quire.op.model.reaches", "laws": [], "mode": null,
                    "member": {"kind": "field", "declaration": at(0, 'c'), "name": "f",
                        "population": id('p')}, "leaves": []},
                "result_type": id('d'),
                "arguments": [
                    {"term": "reference", "target": at(1, 'b')},
                    {"term": "aggregate", "members": [
                        {"term": "binding", "name": "x", "value": {"term": "reference", "target": id('a')}},
                        {"term": "reference", "target": at(1, 'b')},
                    ]},
                ],
            })
        };
        let slotted = body(true);
        let resolved = body(false);
        let table = super::super::graph_storage::OwnedBodies::of_values(
            std::slice::from_ref(&slotted),
            vec![vec![10, 11]],
        );
        let typed =
            super::super::CheckedTerm::decode_admitted(&resolved).expect("closed application");
        let expected = application_dependencies(&typed, &mut WorkMeter::new(u64::MAX))
            .ok()
            .flatten()
            .expect("the body holds an application");
        let run = |limit: u64| {
            let mut budget =
                crate::NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut joined = None;
            let measured = allocation_counter::measure(|| {
                let mut meter = WorkMeter::over(&mut budget);
                table.attach(&mut meter);
                meter.at_node(0, "scalar_type");
                joined = Some(application_dependencies_at(0, &mut meter));
            });
            (
                joined.expect("ran").ok(),
                measured.bytes_total,
                budget.read_storage_consumed(),
            )
        };
        let (found, allocated, charged) = run(u64::MAX);
        assert_eq!(found, Some(Some(expected)));
        assert!(charged > 0);
        assert_eq!(allocated, charged, "allocated == requested");
        assert_eq!(run(charged).0, found, "the measured limit admits");
        assert_eq!(run(charged - 1).0, None, "one byte short is denied");
    }

    /// A state clause's execution point is closed for the node view exactly
    /// when the typed decoder admits it.
    #[trace("TC-472", "FR-038-AC-504")]
    #[test]
    fn closed_execution_point_agrees_with_the_decoder() {
        for point in [
            json!({"kind": "initialization", "name": "boot"}),
            json!({"kind": "handler", "name": ""}),
            json!({"kind": "pre", "operation": "run"}),
            json!({"kind": "post", "operation": "run", "name": "x"}),
            json!({"kind": "pre", "name": "run"}),
            json!({"kind": "after", "operation": "run"}),
            json!({"kind": 3, "operation": "run"}),
            json!({"operation": "run"}),
            json!({"kind": "post", "operation": 4}),
            json!([]),
            json!("pre"),
        ] {
            let handle = super::super::CheckedJson::from_serializable(&point).expect("canonical");
            assert_eq!(
                is_closed_execution_point(TermNode::of(&handle)),
                ExecutionPoint::from_wire(&point).is_some(),
                "{point}"
            );
        }
    }
}
