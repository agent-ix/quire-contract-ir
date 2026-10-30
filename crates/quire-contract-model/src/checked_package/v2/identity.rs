//! Structural nominal bodies (enum declaration, enum value, dimension,
//! dimension equivalence and unit) and their semantic and cross-field rules.

use super::model_members::DomainModel;
use super::natural::coprime;
use super::{
    BoundedDomainForm, CheckedNodeKind, ClaimForm, CompositeTypeForm, CorrespondenceForm,
    ExpressionForm, FunctionForm, ModelForm, ProtocolForm,
    RawSemanticNodeV2 as CheckedSemanticNodeV2, RelationForm, ScalarTypeForm, StateForm,
    TemporalForm, ValueForm, WorkMeter,
};
use super::{CheckedPackageLockV2, RawCheckedPackageWireV2};
use crate::checked_package::common::{node_pointer, ValidationFailure};
use crate::checked_package::shared::{CheckedNodeId, CheckedPackageRefusalCode, JsonPointer};
use num_bigint::{BigInt, BigUint};
use num_integer::Integer;
use num_traits::{One, Zero};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Reader-owned view assembled from a node's body, owner and declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
enum NominalRecord {
    /// Enum declaration.
    EnumDeclaration(EnumDeclarationRecord),
    /// Enum value.
    EnumMember(EnumMemberRecord),
    /// Dimension.
    Dimension(DimensionRecord),
    /// Dimension equivalence.
    DimensionEquivalence(DimensionEquivalenceRecord),
    /// Unit.
    Unit(UnitRecord),
}

/// Owner subject of a nominal declaration; joins an exact lock selection.
#[derive(Clone, Debug, Eq, PartialEq)]
enum NominalOwner {
    /// A locked raw source, joined by authority and identity.
    Source {
        /// Source authority.
        authority: Box<str>,
        /// Source identity.
        identity: Box<str>,
    },
    /// A declaration in a selected domain package (QSpec `ModelOwner`),
    /// joined by the package identity; `node` is the IR node identity inside
    /// that package.
    Model {
        /// Domain package identity.
        identity: Box<str>,
        /// IR node identity within the domain package.
        node: Box<str>,
    },
}

/// Enum declaration semantic view.
#[derive(Clone, Debug, Eq, PartialEq)]
struct EnumDeclarationRecord {
    /// Declaring owner.
    pub owner: NominalOwner,
    /// ASCII identifier segments.
    pub qualified_declaration: Vec<Box<str>>,
    /// Whether member order is semantic.
    pub ordered: bool,
    /// Member identifiers.
    pub members: Vec<Box<str>>,
}

/// Enum member semantic view.
#[derive(Clone, Debug, Eq, PartialEq)]
struct EnumMemberRecord {
    /// Declaring enum node key.
    pub declaration_node_id: CheckedNodeId,
    /// Exact semantic case name; model variants also admit nonidentifier UTF-8.
    pub case: Box<str>,
}

/// Dimension semantic view.
#[derive(Clone, Debug, Eq, PartialEq)]
struct DimensionRecord {
    /// Declaring owner.
    pub owner: NominalOwner,
    /// ASCII identifier segments.
    pub qualified_declaration: Vec<Box<str>>,
    /// Base-dimension terms; empty for a base dimension.
    pub terms: Vec<DimensionTerm>,
}

/// One base-dimension power in a derived dimension.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DimensionTerm {
    /// Base dimension node key.
    pub dimension_node_id: CheckedNodeId,
    /// Canonical nonzero integer exponent.
    pub exponent: Box<str>,
}

/// Exact equivalence between a structurally ordered pair of dimensions.
#[derive(Clone, Debug, Eq, PartialEq)]
struct DimensionEquivalenceRecord {
    /// Source containing the equivalence declaration.
    pub owner: SourceOwner,
    /// Exactly two distinct dimension IDs, in structural order.
    pub dimension_node_ids: [CheckedNodeId; 2],
    /// Positive reduced ratio from the first canonical root to the second.
    pub root_scale: CheckedRational,
}

/// Source owner of a dimension equivalence.
#[derive(Clone, Debug, Eq, PartialEq)]
struct SourceOwner {
    /// The only admitted owner kind.
    pub kind: SourceOwnerKind,
    /// Source authority selected by the lock.
    pub authority: Box<str>,
    /// Source identity selected by the lock.
    pub identity: Box<str>,
}

/// Closed source-owner discriminant.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SourceOwnerKind {
    /// A locked source.
    Source,
}

/// Declared unit semantic view.
#[derive(Clone, Debug, Eq, PartialEq)]
struct UnitRecord {
    /// Declaring owner.
    pub owner: NominalOwner,
    /// ASCII identifier segments.
    pub qualified_declaration: Vec<Box<str>>,
    /// Dimension node key.
    pub dimension_node_id: CheckedNodeId,
    /// Target unit key; `None` (wire `null`, required member) for the root.
    pub target_unit_node_id: Option<CheckedNodeId>,
    /// Exact reduced scale to the target.
    pub scale: CheckedRational,
    /// Exact reduced offset to the target.
    pub offset: CheckedRational,
}

/// A canonical decimal rational.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedRational {
    /// Canonical integer string.
    pub numerator: Box<str>,
    /// Canonical positive integer string.
    pub denominator: Box<str>,
}

/// The closed semantic body of a source-authored nominal node. The source
/// owner and declared path are carried by the node, outside this body.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "term", rename_all = "snake_case")]
pub enum NominalBody {
    /// Source enum declaration, preserving authored member order.
    Enum {
        /// Whether the enum is ordered.
        ordered: bool,
        /// Declared case names.
        members: Vec<Box<str>>,
    },
    /// Dimension terms over structural dimension IDs.
    Dimension {
        /// Authored base-dimension terms.
        terms: Vec<DimensionTerm>,
    },
    /// Unit conversion to its target in the named dimension.
    Unit {
        /// Structural dimension ID.
        dimension_node_id: CheckedNodeId,
        /// Target unit, or null for the root unit.
        target_unit_node_id: Option<CheckedNodeId>,
        /// Exact scale to the target.
        scale: CheckedRational,
        /// Exact offset to the target.
        offset: CheckedRational,
    },
    /// Exact ratio between two structurally ordered dimensions.
    DimensionEquivalence {
        /// The ordered pair of dimension IDs.
        dimension_node_ids: [CheckedNodeId; 2],
        /// Exact positive ratio of their canonical roots.
        root_scale: CheckedRational,
    },
}

impl NominalBody {
    /// Decode without cloning a caller-authored nested NodeId path. The
    /// NodeId decoder itself walks those paths on the heap.
    pub(super) fn from_value(body: &Value, at: JsonPointer) -> Result<Self, ValidationFailure> {
        use crate::checked_package::common::exact_members;
        let malformed =
            |path| ValidationFailure::refused(CheckedPackageRefusalCode::MalformedWire, path);
        let object = body.as_object().ok_or_else(|| malformed(at.clone()))?;
        let term = object
            .get("term")
            .and_then(Value::as_str)
            .ok_or_else(|| malformed(at.clone().key("term")))?;
        let node_ref = |member: &str| {
            object
                .get(member)
                .and_then(CheckedNodeId::from_value)
                .ok_or_else(|| malformed(at.clone().key(member)))
        };
        let rational = |member: &str, positive_numerator: bool| {
            let path = at.clone().key(member);
            let value = object.get(member).ok_or_else(|| malformed(path.clone()))?;
            let fields = value.as_object().ok_or_else(|| malformed(path.clone()))?;
            if !exact_members(fields, &["numerator", "denominator"]) {
                return Err(malformed(path));
            }
            let numerator = fields
                .get("numerator")
                .and_then(Value::as_str)
                .filter(|number| {
                    if positive_numerator {
                        positive_integer(number)
                    } else {
                        integer_magnitude(number).is_some()
                    }
                })
                .ok_or_else(|| malformed(path.clone().key("numerator")))?;
            let denominator = fields
                .get("denominator")
                .and_then(Value::as_str)
                .filter(|number| positive_integer(number))
                .ok_or_else(|| malformed(path.clone().key("denominator")))?;
            Ok(CheckedRational {
                numerator: numerator.into(),
                denominator: denominator.into(),
            })
        };
        match term {
            "enum" if exact_members(object, &["term", "ordered", "members"]) => {
                let ordered = object["ordered"]
                    .as_bool()
                    .ok_or_else(|| malformed(at.clone().key("ordered")))?;
                let values = object["members"]
                    .as_array()
                    .filter(|items| !items.is_empty())
                    .ok_or_else(|| malformed(at.clone().key("members")))?;
                let mut unique = BTreeSet::new();
                let mut members = Vec::with_capacity(values.len());
                for (index, value) in values.iter().enumerate() {
                    let member = value
                        .as_str()
                        .filter(|text| is_identifier(text) && unique.insert(*text))
                        .ok_or_else(|| malformed(at.clone().key("members").index(index)))?;
                    members.push(member.into());
                }
                Ok(Self::Enum { ordered, members })
            }
            "dimension" if exact_members(object, &["term", "terms"]) => {
                let values = object["terms"]
                    .as_array()
                    .ok_or_else(|| malformed(at.clone().key("terms")))?;
                let mut terms = Vec::with_capacity(values.len());
                for (index, value) in values.iter().enumerate() {
                    let path = at.clone().key("terms").index(index);
                    let fields = value.as_object().ok_or_else(|| malformed(path.clone()))?;
                    if !exact_members(fields, &["dimension_node_id", "exponent"]) {
                        return Err(malformed(path));
                    }
                    let dimension_node_id = fields["dimension_node_id"]
                        .as_object()
                        .and_then(|_| CheckedNodeId::from_value(&fields["dimension_node_id"]))
                        .ok_or_else(|| malformed(path.clone().key("dimension_node_id")))?;
                    let exponent = fields["exponent"]
                        .as_str()
                        .filter(|text| integer_magnitude(text).is_some())
                        .ok_or_else(|| malformed(path.clone().key("exponent")))?;
                    terms.push(DimensionTerm {
                        dimension_node_id,
                        exponent: exponent.into(),
                    });
                }
                Ok(Self::Dimension { terms })
            }
            "unit"
                if exact_members(
                    object,
                    &[
                        "term",
                        "dimension_node_id",
                        "target_unit_node_id",
                        "scale",
                        "offset",
                    ],
                ) =>
            {
                let target_unit_node_id = match &object["target_unit_node_id"] {
                    Value::Null => None,
                    value => Some(
                        CheckedNodeId::from_value(value)
                            .ok_or_else(|| malformed(at.clone().key("target_unit_node_id")))?,
                    ),
                };
                Ok(Self::Unit {
                    dimension_node_id: node_ref("dimension_node_id")?,
                    target_unit_node_id,
                    scale: rational("scale", false)?,
                    offset: rational("offset", false)?,
                })
            }
            "dimension_equivalence"
                if exact_members(object, &["term", "dimension_node_ids", "root_scale"]) =>
            {
                let values = object["dimension_node_ids"]
                    .as_array()
                    .filter(|items| items.len() == 2)
                    .ok_or_else(|| malformed(at.clone().key("dimension_node_ids")))?;
                Ok(Self::DimensionEquivalence {
                    dimension_node_ids: [
                        CheckedNodeId::from_value(&values[0]).ok_or_else(|| {
                            malformed(at.clone().key("dimension_node_ids").index(0))
                        })?,
                        CheckedNodeId::from_value(&values[1]).ok_or_else(|| {
                            malformed(at.clone().key("dimension_node_ids").index(1))
                        })?,
                    ],
                    root_scale: rational("root_scale", true)?,
                })
            }
            _ => Err(malformed(at)),
        }
    }

    pub(super) fn replace_node_id(&mut self, old: &CheckedNodeId, new: &CheckedNodeId) {
        let replace = |id: &mut CheckedNodeId| {
            if id == old {
                *id = new.clone();
            }
        };
        match self {
            Self::Enum { .. } => {}
            Self::Dimension { terms } => {
                for term in terms {
                    replace(&mut term.dimension_node_id);
                }
            }
            Self::Unit {
                dimension_node_id,
                target_unit_node_id,
                ..
            } => {
                replace(dimension_node_id);
                if let Some(id) = target_unit_node_id {
                    replace(id);
                }
            }
            Self::DimensionEquivalence {
                dimension_node_ids, ..
            } => {
                for id in dimension_node_ids {
                    replace(id);
                }
            }
        }
    }

    pub(super) fn visit_reference_targets<'a>(
        &'a self,
        visitor: &mut impl FnMut(&'a CheckedNodeId),
    ) {
        match self {
            Self::Enum { .. } => {}
            Self::Dimension { terms } => {
                for term in terms {
                    visitor(&term.dimension_node_id);
                }
            }
            Self::Unit {
                dimension_node_id,
                target_unit_node_id,
                ..
            } => {
                visitor(dimension_node_id);
                if let Some(id) = target_unit_node_id {
                    visitor(id);
                }
            }
            Self::DimensionEquivalence {
                dimension_node_ids, ..
            } => {
                for id in dimension_node_ids {
                    visitor(id);
                }
            }
        }
    }
}

/// Which nominal semantic view a `(node_tag, semantic_form)` pair requires.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NominalKind {
    EnumDeclaration,
    EnumMember,
    Dimension,
    DimensionEquivalence,
    Unit,
}

impl NominalKind {
    /// The nominal semantic view a node of this kind requires, if any. Only the
    /// five nominal forms need one; every other form is decided here too, so
    /// a new form is a compile error until it is.
    fn required_by(kind: CheckedNodeKind) -> Option<Self> {
        use CheckedNodeKind as K;
        match kind {
            K::Extension(_) | K::Hybrid(_) | K::TimedModel(_) => None,
            K::ScalarType(ScalarTypeForm::Enum) => Some(Self::EnumDeclaration),
            K::Value(ValueForm::EnumValue) => Some(Self::EnumMember),
            K::ScalarType(ScalarTypeForm::Dimension) => Some(Self::Dimension),
            K::ScalarType(ScalarTypeForm::DimensionEquivalence) => Some(Self::DimensionEquivalence),
            K::ScalarType(ScalarTypeForm::Unit) => Some(Self::Unit),
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
                | ScalarTypeForm::CompoundUnit,
            ) => None,
            K::CompositeType(
                CompositeTypeForm::Option
                | CompositeTypeForm::Sequence
                | CompositeTypeForm::Set
                | CompositeTypeForm::Bag
                | CompositeTypeForm::OrderedSet
                | CompositeTypeForm::Record
                | CompositeTypeForm::Tuple
                | CompositeTypeForm::Union
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
            K::Value(
                ValueForm::Literal
                | ValueForm::CollectionValue
                | ValueForm::RecordValue
                | ValueForm::TupleValue
                | ValueForm::UnionValue
                | ValueForm::OptionValue
                | ValueForm::Parameter,
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
                | RelationForm::CausalRelation
                | RelationForm::Refinement,
            ) => None,
            K::State(
                StateForm::StateClause
                | StateForm::Frame
                | StateForm::Transition
                | StateForm::OperationAnchor
                | StateForm::Snapshot,
            ) => None,
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
                | ProtocolForm::ProtocolReference
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
                | ClaimForm::SynthesisRequest
                | ClaimForm::StateGraph
                | ClaimForm::StepRelation,
            ) => None,
            K::Correspondence(
                CorrespondenceForm::SourceLocus
                | CorrespondenceForm::ModelCorrespondence
                | CorrespondenceForm::BindingRole
                | CorrespondenceForm::ProfileCorrespondence
                | CorrespondenceForm::AbstractionRelation,
            ) => None,
        }
    }

    fn of(record: &NominalRecord) -> Self {
        match record {
            NominalRecord::EnumDeclaration(_) => Self::EnumDeclaration,
            NominalRecord::EnumMember(_) => Self::EnumMember,
            NominalRecord::Dimension(_) => Self::Dimension,
            NominalRecord::DimensionEquivalence(_) => Self::DimensionEquivalence,
            NominalRecord::Unit(_) => Self::Unit,
        }
    }
}

fn invalid(path: JsonPointer) -> ValidationFailure {
    ValidationFailure::refused(CheckedPackageRefusalCode::InvalidSemanticGraph, path)
}

/// Refuses at the pointer `at` builds unless `condition` holds.
fn require(condition: bool, at: impl FnOnce() -> JsonPointer) -> Result<(), ValidationFailure> {
    if condition {
        Ok(())
    } else {
        Err(invalid(at()))
    }
}

/// One node under validation: the node, its graph position and pointers to
/// the members of its body, owner and declaration.
struct Site<'a> {
    node: &'a CheckedSemanticNodeV2,
    position: usize,
}

impl Site<'_> {
    fn pointer(&self) -> JsonPointer {
        node_pointer(self.position)
    }

    fn member(&self, member: &str) -> JsonPointer {
        node_pointer(self.position).key(member)
    }

    fn nominal_member(&self, members: &[&str]) -> JsonPointer {
        let (base, rest) = match members.split_first() {
            Some((first, rest)) if *first == "owner" => (self.member("owner"), rest),
            Some((first, rest)) if *first == "qualified_declaration" => {
                (self.member("declaration").key("qualified_name"), rest)
            }
            Some((first, rest)) if *first == "case" => (self.member("body").key("value"), rest),
            Some((first, rest)) if *first == "declaration_node_id" => {
                (self.member("semantic_type"), rest)
            }
            _ => (self.member("body"), members),
        };
        rest.iter()
            .fold(base, |pointer, member| pointer.key(member))
    }
}

fn record_from_node(
    node: &CheckedSemanticNodeV2,
    kind: NominalKind,
    position: usize,
) -> Result<NominalRecord, ValidationFailure> {
    let at = node_pointer(position);
    if kind == NominalKind::EnumMember {
        let case = node
            .body
            .as_object()
            .and_then(|body| body.get("value"))
            .and_then(Value::as_str)
            .ok_or_else(|| invalid(at.clone().key("body").key("value")))?;
        return Ok(NominalRecord::EnumMember(EnumMemberRecord {
            declaration_node_id: node.semantic_type.clone(),
            case: case.into(),
        }));
    }
    let source_owner = match node.owner.as_ref() {
        Some(super::CheckedNodeOwner::Source {
            authority,
            identity,
        }) => NominalOwner::Source {
            authority: authority.clone(),
            identity: identity.clone(),
        },
        Some(super::CheckedNodeOwner::Model { identity, node }) => NominalOwner::Model {
            identity: identity.clone(),
            node: node.clone(),
        },
        _ => return Err(invalid(at.key("owner"))),
    };
    let declared = || {
        node.declaration
            .as_ref()
            .map(|declaration| declaration.qualified_name.clone())
            .ok_or_else(|| invalid(at.clone().key("declaration")))
    };
    let body = || NominalBody::from_value(&node.body, at.clone().key("body"));
    match (kind, body()?) {
        (NominalKind::EnumDeclaration, NominalBody::Enum { ordered, members }) => {
            Ok(NominalRecord::EnumDeclaration(EnumDeclarationRecord {
                owner: source_owner,
                qualified_declaration: declared()?,
                ordered,
                members,
            }))
        }
        (NominalKind::Dimension, NominalBody::Dimension { terms }) => {
            Ok(NominalRecord::Dimension(DimensionRecord {
                owner: source_owner,
                qualified_declaration: declared()?,
                terms,
            }))
        }
        (
            NominalKind::Unit,
            NominalBody::Unit {
                dimension_node_id,
                target_unit_node_id,
                scale,
                offset,
            },
        ) => Ok(NominalRecord::Unit(UnitRecord {
            owner: source_owner,
            qualified_declaration: declared()?,
            dimension_node_id,
            target_unit_node_id,
            scale,
            offset,
        })),
        (
            NominalKind::DimensionEquivalence,
            NominalBody::DimensionEquivalence {
                dimension_node_ids,
                root_scale,
            },
        ) => {
            let NominalOwner::Source {
                authority,
                identity,
            } = source_owner
            else {
                return Err(invalid(at.key("owner")));
            };
            Ok(NominalRecord::DimensionEquivalence(
                DimensionEquivalenceRecord {
                    owner: SourceOwner {
                        kind: SourceOwnerKind::Source,
                        authority,
                        identity,
                    },
                    dimension_node_ids,
                    root_scale,
                },
            ))
        }
        _ => Err(invalid(at.key("body"))),
    }
}

/// Validates every node's nominal owner join, semantic rules and cross-field
/// joins. `kinds` holds each node's decoded kind. Equivalence refusals point
/// at their node; other nominal refusals point at the member at fault.
pub(super) fn validate_nominal_nodes(
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &BTreeMap<&CheckedNodeId, usize>,
    lock: &CheckedPackageLockV2,
    models: &[DomainModel],
    meter: &mut WorkMeter,
    bytes: u64,
) -> Result<(), ValidationFailure> {
    let records = nodes
        .iter()
        .zip(kinds)
        .enumerate()
        .map(|(position, (node, kind))| {
            NominalKind::required_by(*kind)
                .map(|required| record_from_node(node, required, position))
                .transpose()
        })
        .collect::<Result<Vec<_>, _>>()?;
    let selected_models = models
        .iter()
        .map(|model| (model.identity.as_ref(), model))
        .collect();
    let graph = NominalGraph {
        nodes,
        kinds,
        index,
        records: &records,
        selected_models,
    };
    let mut roots = BTreeSet::new();
    let mut equivalence_pairs = BTreeSet::new();
    let mut equivalences = Vec::new();
    for (position, (node, kind)) in nodes.iter().zip(kinds).enumerate() {
        let site = Site { node, position };
        let required = NominalKind::required_by(*kind);
        let record = match (required, records[position].as_ref()) {
            (None, None) => continue,
            (Some(kind), Some(record)) if NominalKind::of(record) == kind => record,
            // Required and absent: the node lacks the member.
            (Some(_), None) => return Err(invalid(node_pointer(position))),
            (Some(NominalKind::DimensionEquivalence), Some(_)) => {
                return Err(invalid(site.pointer()))
            }
            _ => return Err(invalid(site.nominal_member(&[]))),
        };
        meter.charge(1, || site.nominal_member(&[]))?;
        match record {
            NominalRecord::EnumDeclaration(declaration) => {
                validate_enum_declaration(declaration, &site, &graph, lock, meter)?;
            }
            NominalRecord::EnumMember(member) => {
                validate_enum_member(&site, member, &graph)?;
            }
            NominalRecord::Dimension(dimension) => {
                validate_dimension(&site, dimension, &graph, lock, meter, bytes)?;
            }
            NominalRecord::DimensionEquivalence(equivalence) => {
                validate_dimension_equivalence(&site, equivalence, &graph, lock, meter)?;
                require(
                    equivalence_pairs.insert(&equivalence.dimension_node_ids),
                    || site.pointer(),
                )?;
                equivalences.push((position, equivalence));
            }
            NominalRecord::Unit(unit) => {
                validate_unit(&site, unit, &graph, lock, meter)?;
                if unit.target_unit_node_id.is_none() {
                    // A dimension has exactly one targetless root unit.
                    require(roots.insert(unit.dimension_node_id.clone()), || {
                        site.nominal_member(&["target_unit_node_id"])
                    })?;
                }
            }
        }
    }
    validate_equivalence_graph(&graph, &equivalences, meter)?;
    Ok(())
}

fn validate_dimension_equivalence(
    site: &Site<'_>,
    equivalence: &DimensionEquivalenceRecord,
    graph: &NominalGraph<'_>,
    lock: &CheckedPackageLockV2,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let ids = &equivalence.dimension_node_ids;
    require(ids[0] < ids[1], || site.pointer())?;
    for (position, id) in ids.iter().enumerate() {
        meter.charge(1, || {
            site.nominal_member(&["dimension_node_ids"]).index(position)
        })?;
        require(
            matches!(graph.nominal(id), Some(NominalRecord::Dimension(_))),
            || site.pointer(),
        )?;
    }
    meter.charge(
        u64::try_from(lock.sources.len()).unwrap_or(u64::MAX),
        || site.nominal_member(&["owner"]),
    )?;
    require(
        lock.sources.iter().any(|source| {
            source.authority == equivalence.owner.authority
                && source.identity == equivalence.owner.identity
        }),
        || site.pointer(),
    )?;
    validate_rational(&equivalence.root_scale, meter, &|| site.pointer())?;
    require(site.node.semantic_type == site.node.node_id, || {
        site.pointer()
    })?;
    require(site.node.dependencies.as_slice() == ids, || site.pointer())?;
    require(
        matches!(site.node.occurrences.as_slice(), [occurrence]
            if occurrence.role == crate::checked_package::shared::CheckedOccurrenceRole::Anchor),
        || site.pointer(),
    )?;
    Ok(())
}

/// A positive exact ratio. Every operation charges the work meter before
/// allocating or multiplying integers from the untrusted package.
#[derive(Clone, Debug, Eq, PartialEq)]
struct ExactRatio {
    numerator: BigInt,
    denominator: BigInt,
}

impl ExactRatio {
    fn zero() -> Self {
        Self {
            numerator: BigInt::zero(),
            denominator: BigInt::one(),
        }
    }

    fn one() -> Self {
        Self {
            numerator: BigInt::one(),
            denominator: BigInt::one(),
        }
    }

    fn parse(
        value: &CheckedRational,
        meter: &mut WorkMeter,
        at: &dyn Fn() -> JsonPointer,
    ) -> Result<Self, ValidationFailure> {
        meter.charge(
            u64::try_from(
                value
                    .numerator
                    .len()
                    .saturating_add(value.denominator.len()),
            )
            .unwrap_or(u64::MAX),
            at,
        )?;
        let numerator = BigInt::parse_bytes(value.numerator.as_bytes(), 10)
            .ok_or_else(|| invalid(at().key("numerator")))?;
        let denominator = BigInt::parse_bytes(value.denominator.as_bytes(), 10)
            .ok_or_else(|| invalid(at().key("denominator")))?;
        if denominator.is_zero() {
            return Err(invalid(at().key("denominator")));
        }
        Ok(Self {
            numerator,
            denominator,
        })
    }

    fn inverse(&self) -> Self {
        Self {
            numerator: self.denominator.clone(),
            denominator: self.numerator.clone(),
        }
    }

    fn multiply(
        &self,
        other: &Self,
        meter: &mut WorkMeter,
        at: &dyn Fn() -> JsonPointer,
    ) -> Result<Self, ValidationFailure> {
        let digits =
            |value: &BigInt| u64::try_from(value.to_bytes_be().1.len()).unwrap_or(u64::MAX);
        let size = digits(&self.numerator)
            .saturating_add(digits(&self.denominator))
            .saturating_add(digits(&other.numerator))
            .saturating_add(digits(&other.denominator));
        meter.charge(size.saturating_mul(size).saturating_mul(size).max(1), at)?;
        let numerator = &self.numerator * &other.numerator;
        let denominator = &self.denominator * &other.denominator;
        let common = numerator.gcd(&denominator);
        Ok(Self {
            numerator: numerator / &common,
            denominator: denominator / common,
        })
    }

    fn add(
        &self,
        other: &Self,
        meter: &mut WorkMeter,
        at: &dyn Fn() -> JsonPointer,
    ) -> Result<Self, ValidationFailure> {
        let size = self
            .numerator
            .to_bytes_be()
            .1
            .len()
            .saturating_add(self.denominator.to_bytes_be().1.len())
            .saturating_add(other.numerator.to_bytes_be().1.len())
            .saturating_add(other.denominator.to_bytes_be().1.len());
        let size = u64::try_from(size).unwrap_or(u64::MAX);
        meter.charge(size.saturating_mul(size).saturating_mul(size).max(1), at)?;
        let numerator = &self.numerator * &other.denominator + &other.numerator * &self.denominator;
        let denominator = &self.denominator * &other.denominator;
        let common = numerator.gcd(&denominator);
        Ok(Self {
            numerator: numerator / &common,
            denominator: denominator / common,
        })
    }

    fn divide(
        &self,
        other: &Self,
        meter: &mut WorkMeter,
        at: &dyn Fn() -> JsonPointer,
    ) -> Result<Self, ValidationFailure> {
        self.multiply(&other.inverse(), meter, at)
    }

    fn power(
        &self,
        exponent: &str,
        meter: &mut WorkMeter,
        at: &dyn Fn() -> JsonPointer,
    ) -> Result<Self, ValidationFailure> {
        let (negative, magnitude) = match exponent.strip_prefix('-') {
            Some(magnitude) => (true, magnitude),
            None => (false, exponent),
        };
        meter.charge(u64::try_from(magnitude.len()).unwrap_or(u64::MAX), at)?;
        let mut exponent =
            BigUint::parse_bytes(magnitude.as_bytes(), 10).ok_or_else(|| invalid(at()))?;
        let mut power = self.clone();
        let mut result = Self::one();
        while !exponent.is_zero() {
            meter.charge(
                u64::try_from(exponent.to_bytes_be().len())
                    .unwrap_or(u64::MAX)
                    .max(1),
                at,
            )?;
            if exponent.bit(0) {
                result = result.multiply(&power, meter, at)?;
            }
            exponent >>= 1;
            if !exponent.is_zero() {
                power = power.multiply(&power, meter, at)?;
            }
        }
        Ok(if negative { result.inverse() } else { result })
    }
}

/// Whether the named unit is its owner's one-second unit of its own base
/// Time dimension. The graph's nominal checks subsequently authenticate the
/// body, owner, and path; this projection only resolves the member's type.
#[ix_trace_rs::string_edge]
pub(super) fn duration_unit_key(
    nodes: &[CheckedSemanticNodeV2],
    name: &str,
    meter: &mut WorkMeter,
) -> Result<Option<CheckedNodeId>, ValidationFailure> {
    meter.charge(u64::try_from(nodes.len()).unwrap_or(u64::MAX), || {
        JsonPointer::root().key("semantic_graph").key("nodes")
    })?;
    let Some((position, node)) = nodes.iter().enumerate().find(|(_, node)| {
        node.declaration.as_ref().is_some_and(|declaration| {
            let declared = declaration
                .qualified_name
                .iter()
                .map(AsRef::as_ref)
                .collect::<Vec<&str>>()
                .join("/");
            format!("ix://quire/unit/{declared}") == name
        }) && node.node_tag.as_ref() == "scalar_type"
            && node.semantic_form.as_ref() == "unit"
    }) else {
        return Ok(None);
    };
    let Ok(NominalBody::Unit {
        dimension_node_id,
        target_unit_node_id,
        scale: unit_scale,
        offset: unit_offset,
    }) = NominalBody::from_value(&node.body, node_pointer(position).key("body"))
    else {
        return Ok(None);
    };
    let owner_matches = match node.owner.as_ref() {
        Some(super::CheckedNodeOwner::Source { .. }) => name == "ix://quire/unit/s",
        Some(super::CheckedNodeOwner::Model { identity, .. }) => {
            name == format!("ix://quire/unit/{identity}/s")
        }
        _ => false,
    };
    if !owner_matches {
        return Ok(None);
    }
    let at = || node_pointer(position);
    meter.charge(u64::try_from(nodes.len()).unwrap_or(u64::MAX), at)?;
    let Some(dimension) = nodes
        .iter()
        .find(|candidate| candidate.node_id == dimension_node_id)
    else {
        return Ok(None);
    };
    let Ok(NominalBody::Dimension { terms }) =
        NominalBody::from_value(&dimension.body, node_pointer(position).key("body"))
    else {
        return Ok(None);
    };
    if dimension.owner != node.owner
        || dimension
            .declaration
            .as_ref()
            .map(|value| &value.qualified_name)
            .into_iter()
            .flatten()
            .last()
            .is_none_or(|name| name.as_ref() != "Time")
        || !terms.is_empty()
    {
        return Ok(None);
    }
    let mut scale = ExactRatio::one();
    let mut offset = ExactRatio::zero();
    let mut current = (
        dimension_node_id,
        target_unit_node_id,
        unit_scale,
        unit_offset,
    );
    let mut seen = BTreeSet::new();
    loop {
        meter.charge(u64::try_from(nodes.len()).unwrap_or(u64::MAX), at)?;
        let edge_scale = ExactRatio::parse(&current.2, meter, &at)?;
        let edge_offset = ExactRatio::parse(&current.3, meter, &at)?;
        offset = offset
            .multiply(&edge_scale, meter, &at)?
            .add(&edge_offset, meter, &at)?;
        scale = scale.multiply(&edge_scale, meter, &at)?;
        let Some(target) = current.1.clone() else {
            break;
        };
        if !seen.insert(target.clone()) {
            return Ok(None);
        }
        let Some(next) = nodes.iter().find(|candidate| candidate.node_id == target) else {
            return Ok(None);
        };
        let Ok(NominalBody::Unit {
            dimension_node_id: next_dimension,
            target_unit_node_id: next_target,
            scale: next_scale,
            offset: next_offset,
        }) = NominalBody::from_value(&next.body, at().key("body"))
        else {
            return Ok(None);
        };
        if next.owner != node.owner || next_dimension != current.0 {
            return Ok(None);
        }
        current = (next_dimension, next_target, next_scale, next_offset);
    }
    Ok((scale == ExactRatio::one() && offset == ExactRatio::zero()).then(|| node.node_id.clone()))
}

/// A dimension's canonical base component and exact root scale within it.
type BasePotential = (CheckedNodeId, ExactRatio);

/// Derives one potential per connected component and checks each edge against
/// the first path already established to its target. The first structural ID
/// in order starts each component with ratio one, so the result is deterministic.
fn equivalence_potentials(
    graph: &NominalGraph<'_>,
    edges: &[(usize, &DimensionEquivalenceRecord)],
    base_only: bool,
    meter: &mut WorkMeter,
) -> Result<BTreeMap<CheckedNodeId, BasePotential>, ValidationFailure> {
    meter.charge(u64::try_from(graph.nodes.len()).unwrap_or(u64::MAX), || {
        JsonPointer::root().key("semantic_graph").key("nodes")
    })?;
    let mut adjacency: BTreeMap<CheckedNodeId, Vec<(CheckedNodeId, ExactRatio, usize)>> =
        BTreeMap::new();
    for (position, edge) in edges {
        let ids = &edge.dimension_node_ids;
        let is_base = |id: &CheckedNodeId| matches!(graph.nominal(id), Some(NominalRecord::Dimension(dim)) if dim.terms.is_empty());
        if base_only && !(is_base(&ids[0]) && is_base(&ids[1])) {
            continue;
        }
        let at = || node_pointer(*position);
        let ratio = ExactRatio::parse(&edge.root_scale, meter, &at)?;
        adjacency.entry(ids[0].clone()).or_default().push((
            ids[1].clone(),
            ratio.inverse(),
            *position,
        ));
        adjacency
            .entry(ids[1].clone())
            .or_default()
            .push((ids[0].clone(), ratio, *position));
    }
    let mut potentials = BTreeMap::new();
    for id in graph.index.keys().copied() {
        if !matches!(graph.nominal(id), Some(NominalRecord::Dimension(_)))
            || potentials.contains_key(id)
        {
            continue;
        }
        let mut pending = VecDeque::from([id.clone()]);
        potentials.insert(id.clone(), (id.clone(), ExactRatio::one()));
        while let Some(current) = pending.pop_front() {
            let Some((component, current_ratio)) = potentials.get(&current).cloned() else {
                continue;
            };
            for (neighbor, multiplier, position) in adjacency.get(&current).into_iter().flatten() {
                let at = || node_pointer(*position);
                let candidate = current_ratio.multiply(multiplier, meter, &at)?;
                match potentials.get(neighbor) {
                    Some((known_component, known_ratio)) => {
                        require(
                            *known_component == component && *known_ratio == candidate,
                            at,
                        )?;
                    }
                    None => {
                        potentials.insert(neighbor.clone(), (component.clone(), candidate));
                        pending.push_back(neighbor.clone());
                    }
                }
            }
        }
    }
    Ok(potentials)
}

fn normalized_dimension(
    id: &CheckedNodeId,
    graph: &NominalGraph<'_>,
    bases: &BTreeMap<CheckedNodeId, BasePotential>,
    meter: &mut WorkMeter,
    at: &dyn Fn() -> JsonPointer,
) -> Result<(BTreeMap<CheckedNodeId, BigInt>, ExactRatio), ValidationFailure> {
    let Some(NominalRecord::Dimension(dimension)) = graph.nominal(id) else {
        return Err(invalid(at()));
    };
    if dimension.terms.is_empty() {
        let Some((component, scale)) = bases.get(id) else {
            return Err(invalid(at()));
        };
        return Ok((
            BTreeMap::from([(component.clone(), BigInt::one())]),
            scale.clone(),
        ));
    }
    let mut map: BTreeMap<CheckedNodeId, BigInt> = BTreeMap::new();
    let mut scale = ExactRatio::one();
    for term in &dimension.terms {
        meter.charge(u64::try_from(term.exponent.len()).unwrap_or(u64::MAX), at)?;
        let exponent = term.exponent.parse::<BigInt>().map_err(|_| invalid(at()))?;
        let Some((component, base_scale)) = bases.get(&term.dimension_node_id) else {
            return Err(invalid(at()));
        };
        *map.entry(component.clone()).or_default() += exponent;
        scale = scale.multiply(&base_scale.power(&term.exponent, meter, at)?, meter, at)?;
    }
    map.retain(|_, exponent| !exponent.is_zero());
    Ok((map, scale))
}

fn validate_equivalence_graph(
    graph: &NominalGraph<'_>,
    edges: &[(usize, &DimensionEquivalenceRecord)],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    if edges.is_empty() {
        return Ok(());
    }
    let bases = equivalence_potentials(graph, edges, true, meter)?;
    for (position, edge) in edges {
        let ids = &edge.dimension_node_ids;
        let both_base = ids.iter().all(|id| {
            matches!(graph.nominal(id), Some(NominalRecord::Dimension(dim)) if dim.terms.is_empty())
        });
        if both_base {
            continue;
        }
        let at = || node_pointer(*position);
        let (left_map, left_scale) = normalized_dimension(&ids[0], graph, &bases, meter, &at)?;
        let (right_map, right_scale) = normalized_dimension(&ids[1], graph, &bases, meter, &at)?;
        require(left_map == right_map, at)?;
        let expected = left_scale.divide(&right_scale, meter, &at)?;
        let actual = ExactRatio::parse(&edge.root_scale, meter, &at)?;
        require(actual == expected, at)?;
    }
    // A derived edge may complete a cycle through other derived edges, so
    // check the whole equivalence graph after base-root consistency.
    equivalence_potentials(graph, edges, false, meter)?;
    Ok(())
}

/// Source selected by the lock for an equivalence declaration.
type EquivalenceSource<'a> = (&'a str, &'a str);

/// Source order, graph position and occurrence ordinal of one anchor.
type EquivalenceAnchor = (u64, usize, u64);

/// Anchors grouped by their declaring source.
type SourceEquivalenceAnchors<'a> = BTreeMap<EquivalenceSource<'a>, Vec<EquivalenceAnchor>>;

/// Anchor ordinals count equivalence declarations in source byte order, not
/// graph order. Source-map admission already established the exact node and
/// occurrence joins before this check runs.
pub(super) fn validate_equivalence_occurrences(
    wire: &RawCheckedPackageWireV2,
    kinds: &[CheckedNodeKind],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    if !kinds.contains(&CheckedNodeKind::ScalarType(
        ScalarTypeForm::DimensionEquivalence,
    )) {
        return Ok(());
    }
    meter.charge(
        u64::try_from(wire.source_map.len()).unwrap_or(u64::MAX),
        || JsonPointer::root().key("source_map"),
    )?;
    let anchors = wire
        .source_map
        .iter()
        .filter(|entry| entry.role == crate::checked_package::shared::CheckedOccurrenceRole::Anchor)
        .map(|entry| (&entry.node_id, entry))
        .collect::<BTreeMap<_, _>>();
    let mut by_source: SourceEquivalenceAnchors<'_> = BTreeMap::new();
    for (position, (node, kind)) in wire.semantic_graph.nodes.iter().zip(kinds).enumerate() {
        if *kind != CheckedNodeKind::ScalarType(ScalarTypeForm::DimensionEquivalence) {
            continue;
        }
        let Some(super::CheckedNodeOwner::Source {
            authority,
            identity,
        }) = node.owner.as_ref()
        else {
            return Err(invalid(node_pointer(position).key("owner")));
        };
        meter.charge(1, || node_pointer(position).key("occurrences"))?;
        let Some(entry) = anchors.get(&node.node_id) else {
            return Err(invalid(node_pointer(position).key("occurrences")));
        };
        let Some(start) = entry
            .regions
            .iter()
            .filter(|region| {
                region.source.authority == *authority && region.source.identity == *identity
            })
            .map(|region| region.start)
            .min()
        else {
            return Err(invalid(node_pointer(position).key("occurrences")));
        };
        by_source
            .entry((authority, identity))
            .or_default()
            .push((start, position, entry.ordinal));
    }
    for mut declarations in by_source.into_values() {
        declarations.sort();
        for (expected, (_, position, actual)) in declarations.into_iter().enumerate() {
            require(
                actual == u64::try_from(expected).unwrap_or(u64::MAX),
                || node_pointer(position).key("occurrences"),
            )?;
        }
    }
    Ok(())
}

struct NominalGraph<'a> {
    nodes: &'a [CheckedSemanticNodeV2],
    kinds: &'a [CheckedNodeKind],
    index: &'a BTreeMap<&'a CheckedNodeId, usize>,
    records: &'a [Option<NominalRecord>],
    selected_models: BTreeMap<&'a str, &'a DomainModel>,
}

impl NominalGraph<'_> {
    fn nominal(&self, id: &CheckedNodeId) -> Option<&NominalRecord> {
        self.index
            .get(id)
            .and_then(|position| self.records.get(*position))
            .and_then(Option::as_ref)
    }

    fn model_variant_cases(
        &self,
        id: &CheckedNodeId,
    ) -> Option<&BTreeMap<Box<str>, Option<super::model_members::SelectedTypeRef>>> {
        let position = *self.index.get(id)?;
        if self.kinds.get(position) != Some(&CheckedNodeKind::Model(super::ModelForm::VariantType))
        {
            return None;
        }
        let node = self.nodes.get(position)?;
        let Some(super::CheckedNodeOwner::Model {
            identity,
            node: owner_node,
        }) = &node.owner
        else {
            return None;
        };
        self.selected_models
            .get(identity.as_ref())
            .and_then(|model| model.variant_types.get(owner_node))
    }

    fn is_model_variant(&self, id: &CheckedNodeId) -> bool {
        let Some(&position) = self.index.get(id) else {
            return false;
        };
        self.kinds.get(position) == Some(&CheckedNodeKind::Model(super::ModelForm::VariantType))
            && self.nodes.get(position).is_some_and(|node| {
                matches!(
                    node.owner.as_ref(),
                    Some(super::CheckedNodeOwner::Model { .. })
                )
            })
    }
}

fn validate_owner(
    owner: &NominalOwner,
    lock: &CheckedPackageLockV2,
    graph: &NominalGraph<'_>,
    at: impl FnOnce() -> JsonPointer,
) -> Result<(), ValidationFailure> {
    let joined = match owner {
        NominalOwner::Source {
            authority,
            identity,
        } => lock
            .sources
            .iter()
            .any(|source| source.authority == *authority && source.identity == *identity),
        // A selected domain package is named by its `model_root` owner; the
        // node is not a lock member, so the join is by package identity, and
        // the owner's identity and node must each be nonempty (FR-038-AC-45).
        // Each identity has at most one root, so joining by identity alone is
        // sound.
        NominalOwner::Model { identity, node } => {
            !node.is_empty()
                && !identity.is_empty()
                && graph.selected_models.contains_key(identity.as_ref())
        }
    };
    require(joined, at)
}

/// ASCII identifier grammar shared with the closed `Declaration` member:
/// the schema's `Identifier`, `^[A-Za-z_][A-Za-z0-9_]*$`.
pub(super) fn is_identifier(value: &str) -> bool {
    let mut bytes = value.bytes();
    bytes
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

/// A nonempty qualified name whose every segment is an ASCII identifier.
/// Shared by the three declared nominal kinds and by
/// `super::validate_declaration`'s `Declaration.qualified_name` check, so the
/// grammar is checked in exactly one place regardless of which member holds
/// the name. `at` names the array; an empty name refuses there and a bad
/// segment at that segment.
pub(super) fn validate_qualified_name(
    name: &[Box<str>],
    at: &dyn Fn() -> JsonPointer,
) -> Result<(), ValidationFailure> {
    require(!name.is_empty(), at)?;
    match name.iter().position(|segment| !is_identifier(segment)) {
        Some(segment) => Err(invalid(at().index(segment))),
        None => Ok(()),
    }
}

/// `^(0|-?[1-9][0-9]*)$`; returns the unsigned magnitude.
#[derive(Debug, Eq, PartialEq)]
struct IntegerMagnitude<'a> {
    digits: &'a str,
    zero: bool,
}

#[ix_trace_rs::string_edge]
fn integer_magnitude(value: &str) -> Option<IntegerMagnitude<'_>> {
    if value == "0" {
        return Some(IntegerMagnitude {
            digits: value,
            zero: true,
        });
    }
    let magnitude = value.strip_prefix('-').unwrap_or(value);
    positive_integer(magnitude).then_some(IntegerMagnitude {
        digits: magnitude,
        zero: false,
    })
}

/// `^[1-9][0-9]*$`.
fn positive_integer(value: &str) -> bool {
    let mut bytes = value.bytes();
    bytes
        .next()
        .is_some_and(|first| (b'1'..=b'9').contains(&first))
        && bytes.all(|byte| byte.is_ascii_digit())
}

/// Validates the rational at `at`: a canonical numerator, a positive
/// denominator, and the pair reduced (its GCD work charged at `at`).
fn validate_rational(
    rational: &CheckedRational,
    meter: &mut WorkMeter,
    at: &dyn Fn() -> JsonPointer,
) -> Result<(), ValidationFailure> {
    let numerator =
        integer_magnitude(&rational.numerator).ok_or_else(|| invalid(at().key("numerator")))?;
    require(positive_integer(&rational.denominator), || {
        at().key("denominator")
    })?;
    require(
        coprime(numerator.digits, &rational.denominator, meter, at)?,
        at,
    )
}

fn validate_enum_declaration(
    declaration: &EnumDeclarationRecord,
    site: &Site<'_>,
    graph: &NominalGraph<'_>,
    lock: &CheckedPackageLockV2,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    validate_owner(&declaration.owner, lock, graph, || {
        site.nominal_member(&["owner"])
    })?;
    validate_qualified_name(&declaration.qualified_declaration, &|| {
        site.nominal_member(&["qualified_declaration"])
    })?;
    require(!declaration.members.is_empty(), || {
        site.nominal_member(&["members"])
    })?;
    let member_at = |index: usize| site.nominal_member(&["members"]).index(index);
    let mut seen = BTreeSet::new();
    for (index, member) in declaration.members.iter().enumerate() {
        meter.charge(1, || member_at(index))?;
        require(
            is_identifier(member) && seen.insert(member.as_ref()),
            || member_at(index),
        )?;
    }
    if !declaration.ordered {
        // Identifiers are escape-free ASCII, so JCS key order is byte order.
        // The later member of the first out-of-order pair is at fault.
        if let Some(pair) = declaration
            .members
            .windows(2)
            .position(|pair| !matches!(pair, [left, right] if left < right))
        {
            return Err(invalid(member_at(pair.saturating_add(1))));
        }
    }
    Ok(())
}

fn validate_enum_member(
    site: &Site<'_>,
    member: &EnumMemberRecord,
    graph: &NominalGraph<'_>,
) -> Result<(), ValidationFailure> {
    let node = site.node;
    if graph.is_model_variant(&member.declaration_node_id) {
        require(!member.case.is_empty(), || site.pointer())?;
        if let Some(cases) = graph.model_variant_cases(&member.declaration_node_id) {
            require(cases.values().all(Option::is_none), || site.pointer())?;
            require(cases.contains_key(&member.case), || site.pointer())?;
        }
    } else {
        require(is_identifier(&member.case), || {
            site.nominal_member(&["case"])
        })?;
        let Some(NominalRecord::EnumDeclaration(declaration)) =
            graph.nominal(&member.declaration_node_id)
        else {
            return Err(invalid(site.nominal_member(&["declaration_node_id"])));
        };
        require(declaration.members.contains(&member.case), || {
            site.nominal_member(&["case"])
        })?;
    }
    require(node.semantic_type == member.declaration_node_id, || {
        site.member("semantic_type")
    })?;
    require(
        node.dependencies.as_slice() == std::slice::from_ref(&member.declaration_node_id),
        || site.member("dependencies"),
    )?;
    let declared_type = serde_json::to_value(&member.declaration_node_id)
        .map_err(|_| invalid(site.nominal_member(&["declaration_node_id"])))?;
    require(
        node.body
            == json!({
                "term": "literal",
                "type": declared_type,
                "value_kind": "enum",
                "value": member.case.as_ref(),
            }),
        || site.member("body"),
    )
}

fn validate_dimension(
    site: &Site<'_>,
    dimension: &DimensionRecord,
    graph: &NominalGraph<'_>,
    lock: &CheckedPackageLockV2,
    meter: &mut WorkMeter,
    _bytes: u64,
) -> Result<(), ValidationFailure> {
    let node = site.node;
    validate_owner(&dimension.owner, lock, graph, || {
        site.nominal_member(&["owner"])
    })?;
    validate_qualified_name(&dimension.qualified_declaration, &|| {
        site.nominal_member(&["qualified_declaration"])
    })?;
    require(node.semantic_type == node.node_id, || {
        site.member("semantic_type")
    })?;
    let term_at = |index: usize| site.nominal_member(&["terms"]).index(index);
    let mut bases = BTreeSet::new();
    for (index, term) in dimension.terms.iter().enumerate() {
        meter.charge(1, || term_at(index))?;
        let exponent_at = || term_at(index).key("exponent");
        let magnitude = integer_magnitude(&term.exponent).ok_or_else(|| invalid(exponent_at()))?;
        require(!magnitude.zero, exponent_at)?;
        let base_at = || term_at(index).key("dimension_node_id");
        require(bases.insert(&term.dimension_node_id), base_at)?;
        let Some(NominalRecord::Dimension(base)) = graph.nominal(&term.dimension_node_id) else {
            return Err(invalid(base_at()));
        };
        require(base.terms.is_empty(), base_at)?;
    }
    // The later term of the first out-of-order node ID is at fault.
    if let Some(pair) = dimension.terms.windows(2).position(
        |pair| !matches!(pair, [left, right] if left.dimension_node_id < right.dimension_node_id),
    ) {
        return Err(invalid(term_at(pair.saturating_add(1))));
    }
    let mut expected = dimension
        .terms
        .iter()
        .map(|term| &term.dimension_node_id)
        .collect::<Vec<_>>();
    let mut actual = node.dependencies.iter().collect::<Vec<_>>();
    expected.sort();
    actual.sort();
    require(expected == actual, || site.member("dependencies"))
}

fn validate_unit(
    site: &Site<'_>,
    unit: &UnitRecord,
    graph: &NominalGraph<'_>,
    lock: &CheckedPackageLockV2,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let node = site.node;
    validate_owner(&unit.owner, lock, graph, || site.nominal_member(&["owner"]))?;
    validate_qualified_name(&unit.qualified_declaration, &|| {
        site.nominal_member(&["qualified_declaration"])
    })?;
    require(
        matches!(
            graph.nominal(&unit.dimension_node_id),
            Some(NominalRecord::Dimension(_))
        ),
        || site.nominal_member(&["dimension_node_id"]),
    )?;
    validate_rational(&unit.scale, meter, &|| site.nominal_member(&["scale"]))?;
    validate_rational(&unit.offset, meter, &|| site.nominal_member(&["offset"]))?;
    require(node.semantic_type == unit.dimension_node_id, || {
        site.member("semantic_type")
    })?;
    let mut expected = vec![&unit.dimension_node_id];
    match &unit.target_unit_node_id {
        None => {
            require(is_rational(&unit.scale, "1", "1"), || {
                site.nominal_member(&["scale"])
            })?;
            require(is_rational(&unit.offset, "0", "1"), || {
                site.nominal_member(&["offset"])
            })?;
        }
        Some(target) => {
            require(
                integer_magnitude(&unit.scale.numerator).is_none_or(|m| !m.zero),
                || site.nominal_member(&["scale", "numerator"]),
            )?;
            expected.push(target);
            validate_unit_path(site, unit, graph, meter)?;
        }
    }
    let mut actual = node.dependencies.iter().collect::<Vec<_>>();
    expected.sort();
    actual.sort();
    require(expected == actual, || site.member("dependencies"))
}

fn is_rational(rational: &CheckedRational, numerator: &str, denominator: &str) -> bool {
    rational.numerator.as_ref() == numerator && rational.denominator.as_ref() == denominator
}

/// Follows targets to a root; every hop is a same-dimension unit, acyclic.
/// Each hop is charged, and refused, at the `target_unit_node_id` that names
/// it — this node's own for the first hop, the previous unit's after that.
fn validate_unit_path(
    site: &Site<'_>,
    unit: &UnitRecord,
    graph: &NominalGraph<'_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let link_at = |position: usize| {
        node_pointer(position)
            .key("body")
            .key("target_unit_node_id")
    };
    let mut visited = BTreeSet::from([&site.node.node_id]);
    let mut from = site.position;
    let mut next = unit.target_unit_node_id.as_ref();
    while let Some(target) = next {
        meter.charge(1, || link_at(from))?;
        require(visited.insert(target), || link_at(from))?;
        let Some(NominalRecord::Unit(target_unit)) = graph.nominal(target) else {
            return Err(invalid(link_at(from)));
        };
        require(
            target_unit.dimension_node_id == unit.dimension_node_id,
            || link_at(from),
        )?;
        // `graph.nominal` resolved `target`, so it is indexed.
        from = graph.index.get(target).copied().unwrap_or(from);
        next = target_unit.target_unit_node_id.as_ref();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ix_trace_rs::trace;
    use serde_json::json;

    #[trace("TC-048", "FR-038-AC-5", "FR-063-AC-3")]
    #[test]
    fn merged_qspec_structural_nominal_fixture_reads() {
        let Some(root) = std::env::var_os("QUIRE_SPECIFICATION_DIR") else {
            return;
        };
        let path = std::path::Path::new(&root)
            .join("proposals/checked-package-v2/fixtures/positive-nominal-identities.json");
        let source = std::fs::read(path).expect("authoritative QSpec nominal fixture");
        let fixture: Value = serde_json::from_slice(&source).expect("QSpec JSON");
        let bytes = serde_json::to_vec(&fixture).expect("canonical test JSON");
        let mut evidence = crate::CheckedPackageEvidence::new();
        evidence.support_feature("quire.value.complete/v1");
        let result = super::super::CheckedPackageV2::read(
            &bytes,
            crate::CheckedPackageReadLimits::bounded(),
            &evidence,
        );
        assert!(
            matches!(
                result,
                super::super::CheckedPackageV2ReadResult::Admitted(_)
            ),
            "{result:?}"
        );
    }

    #[trace("TC-048", "FR-038-AC-5", "FR-063-AC-3")]
    #[test]
    fn nominal_body_keeps_closed_structural_references() {
        let first = json!({"package":"acme/clock","path":["Time"],"ordinal":0});
        let second = json!({"package":"acme/flow","path":["Time"],"ordinal":0});
        let body = json!({"term":"dimension_equivalence",
            "dimension_node_ids":[first,second],
            "root_scale":{"numerator":"3","denominator":"2"}});
        let parsed = NominalBody::from_value(&body, JsonPointer::root()).expect("closed body");
        let NominalBody::DimensionEquivalence {
            dimension_node_ids,
            root_scale,
        } = parsed
        else {
            panic!("equivalence body");
        };
        assert_eq!(dimension_node_ids[0].package.as_ref(), "acme/clock");
        assert_eq!(dimension_node_ids[1].package.as_ref(), "acme/flow");
        assert_eq!(
            (&*root_scale.numerator, &*root_scale.denominator),
            ("3", "2")
        );
        let malformed = json!({"term":"dimension_equivalence",
            "dimension_node_ids":[first,second],
            "root_scale":{"numerator":"0","denominator":"1"}});
        assert!(matches!(
            NominalBody::from_value(&malformed, JsonPointer::root()),
            Err(ValidationFailure::Refused(refusal))
                if refusal.code == CheckedPackageRefusalCode::MalformedWire
                    && refusal.loci[0].path.as_ref().is_some_and(|path| path.as_str() == "/root_scale/numerator")
        ));
    }

    #[trace("TC-048", "FR-038-AC-117")]
    #[test]
    fn nominal_body_node_ref_has_no_fixed_path_depth() {
        let mut id = json!({"package":"example","path":["base"],"ordinal":0});
        for _ in 0..10_000 {
            let mut wrapper = serde_json::Map::new();
            wrapper.insert("package".into(), Value::String("example".into()));
            wrapper.insert("path".into(), Value::Array(vec![id]));
            wrapper.insert("ordinal".into(), Value::from(0));
            id = Value::Object(wrapper);
        }
        let mut term = serde_json::Map::new();
        term.insert("dimension_node_id".into(), id);
        term.insert("exponent".into(), Value::String("1".into()));
        let mut body_members = serde_json::Map::new();
        body_members.insert("term".into(), Value::String("dimension".into()));
        body_members.insert("terms".into(), Value::Array(vec![Value::Object(term)]));
        let body = Value::Object(body_members);
        let parsed = NominalBody::from_value(&body, JsonPointer::root()).expect("deep reference");
        let term = super::super::CheckedTerm::NominalBody(Box::new(parsed));
        let encoded = quire_canonical::to_vec(&term, quire_canonical::Limits::new(1 << 22))
            .expect("iterative encoding");
        assert!(encoded.len() > 100_000);
        quire_canonical::drop_value(body);
    }

    #[trace("TC-048")]
    #[test]
    fn tc_048_integer_and_identifier_grammars_are_exact() {
        assert_eq!(
            integer_magnitude("0").map(|m| (m.digits, m.zero)),
            Some(("0", true))
        );
        assert_eq!(
            integer_magnitude("-12").map(|m| (m.digits, m.zero)),
            Some(("12", false))
        );
        assert_eq!(integer_magnitude("-0"), None);
        assert_eq!(integer_magnitude("012"), None);
        assert_eq!(integer_magnitude(""), None);
        assert!(!positive_integer("0"));
        assert!(is_identifier("_a1"));
        assert!(!is_identifier("1a"));
        assert!(!is_identifier(""));
    }
}
