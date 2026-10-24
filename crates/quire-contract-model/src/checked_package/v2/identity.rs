//! Closed nominal node identity preimages (enum declaration, enum member,
//! dimension, declared unit) and their semantic and cross-field rules.
//!
//! The node key of a nominal form is SHA-256 of the RFC 8785 canonical bytes
//! of its preimage, computed by `quire-canonical`. Every violation is
//! `invalid_semantic_graph`.

use super::model_members::DomainModel;
use super::natural::coprime;
use super::CheckedPackageLockV2;
use super::{
    BoundedDomainForm, CheckedNodeKind, ClaimForm, CompositeTypeForm, CorrespondenceForm,
    ExpressionForm, FunctionForm, ModelForm, ProtocolForm,
    RawSemanticNodeV2 as CheckedSemanticNodeV2, RelationForm, ScalarTypeForm, StateForm,
    TemporalForm, ValueForm, WorkMeter,
};
use crate::checked_package::common::{
    decoder_pointer, locate_closed_value_failure, node_pointer, ValidationFailure,
};
use crate::checked_package::shared::{CheckedNodeId, CheckedPackageRefusalCode, JsonPointer};
use quire_canonical::FixedShape;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

/// The closed nominal identity preimage carried by a V2 node.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(tag = "version", deny_unknown_fields)]
pub enum NominalIdentityPreimage {
    /// `quire.enum-declaration-node/v1`.
    #[serde(rename = "quire.enum-declaration-node/v1")]
    EnumDeclaration(EnumDeclarationPreimage),
    /// `quire.enum-member-node/v1`.
    #[serde(rename = "quire.enum-member-node/v1")]
    EnumMember(EnumMemberPreimage),
    /// `quire.dimension-node/v1`.
    #[serde(rename = "quire.dimension-node/v1")]
    Dimension(DimensionPreimage),
    /// `quire.unit-node/v1`.
    #[serde(rename = "quire.unit-node/v1")]
    Unit(UnitPreimage),
}

/// Owner subject of a nominal declaration; joins an exact lock selection.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum NominalOwner {
    /// A locked raw source, joined by authority and identity.
    Source {
        /// Source authority.
        authority: Box<str>,
        /// Source identity.
        identity: Box<str>,
    },
    /// A selected definition, joined by authority and identity.
    Definition {
        /// Definition authority.
        authority: Box<str>,
        /// Definition identity.
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

/// Enum declaration identity preimage.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(deny_unknown_fields)]
pub struct EnumDeclarationPreimage {
    /// Declaring owner.
    pub owner: NominalOwner,
    /// ASCII identifier segments.
    pub qualified_declaration: Vec<Box<str>>,
    /// Whether member order is semantic.
    pub ordered: bool,
    /// Member identifiers.
    pub members: Vec<Box<str>>,
}

/// Enum member identity preimage.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(deny_unknown_fields)]
pub struct EnumMemberPreimage {
    /// Declaring enum node key.
    pub declaration_node_id: CheckedNodeId,
    /// Exact semantic case name; model variants also admit nonidentifier UTF-8.
    pub case: Box<str>,
}

/// Dimension identity preimage.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(deny_unknown_fields)]
pub struct DimensionPreimage {
    /// Declaring owner.
    pub owner: NominalOwner,
    /// ASCII identifier segments.
    pub qualified_declaration: Vec<Box<str>>,
    /// Base-dimension terms; empty for a base dimension.
    pub terms: Vec<DimensionTerm>,
}

/// One base-dimension power in a derived dimension.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(deny_unknown_fields)]
pub struct DimensionTerm {
    /// Base dimension node key.
    pub dimension_node_id: CheckedNodeId,
    /// Canonical nonzero integer exponent.
    pub exponent: Box<str>,
}

/// Declared unit identity preimage.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(deny_unknown_fields)]
pub struct UnitPreimage {
    /// Declaring owner.
    pub owner: NominalOwner,
    /// ASCII identifier segments.
    pub qualified_declaration: Vec<Box<str>>,
    /// Dimension node key.
    pub dimension_node_id: CheckedNodeId,
    /// Target unit key; `None` (wire `null`, required member) for the root.
    #[serde(deserialize_with = "super::required_nullable")]
    pub target_unit_node_id: Option<CheckedNodeId>,
    /// Exact reduced scale to the target.
    pub scale: CheckedRational,
    /// Exact reduced offset to the target.
    pub offset: CheckedRational,
}

/// A canonical decimal rational.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(deny_unknown_fields)]
pub struct CheckedRational {
    /// Canonical integer string.
    pub numerator: Box<str>,
    /// Canonical positive integer string.
    pub denominator: Box<str>,
}

/// Locates a decode failure the wire decoder could place no deeper than a
/// nominal preimage: an internally tagged object whose members serde reads
/// from a buffer it does not track. Decodes the variant the preimage's own
/// `version` selects, with the tag removed, and then the internally tagged
/// `owner` the same way, so the pointer names the member at fault. `at` is
/// the preimage's pointer and `preimage` its value. The boolean records a
/// rejected unknown member rather than interpreting Serde's error text.
pub(in crate::checked_package) fn locate_preimage_failure(
    at: JsonPointer,
    preimage: &Value,
) -> (JsonPointer, bool) {
    let Some(object) = preimage.as_object() else {
        return (at, false);
    };
    let version = match object.get("version") {
        Some(Value::String(version)) => version.as_str(),
        Some(_) => return (at.key("version"), false),
        None => return (at, false),
    };
    let mut members = object.clone();
    members.remove("version");
    let members = Value::Object(members);
    let failure = match NominalKind::from_version(version) {
        Some(NominalKind::EnumDeclaration) => {
            locate_closed_value_failure::<EnumDeclarationPreimage>(&members)
        }
        Some(NominalKind::EnumMember) => {
            locate_closed_value_failure::<EnumMemberPreimage>(&members)
        }
        Some(NominalKind::Dimension) => locate_closed_value_failure::<DimensionPreimage>(&members),
        Some(NominalKind::Unit) => locate_closed_value_failure::<UnitPreimage>(&members),
        None => return (at.key("version"), false),
    };
    let Some((path, unknown_member)) = failure else {
        return (at, false);
    };
    let located = decoder_pointer(at.clone(), &path);
    if located == at.clone().key("owner") {
        if let Some(owner) = object.get("owner") {
            return locate_owner_failure(located, owner);
        }
    }
    (located, unknown_member)
}

/// Locates a decode failure inside an internally tagged [`NominalOwner`] at
/// `at`, checking members in the order the decoder reads them: the first
/// member outside the `kind`'s closed set or not a string, else the owner
/// itself for a missing member.
fn locate_owner_failure(at: JsonPointer, owner: &Value) -> (JsonPointer, bool) {
    let Some(object) = owner.as_object() else {
        return (at, false);
    };
    let members: &[&str] = match object.get("kind") {
        Some(Value::String(kind)) => match NominalOwnerKind::from_wire(kind) {
            Some(NominalOwnerKind::Source | NominalOwnerKind::Definition) => {
                &["kind", "authority", "identity"]
            }
            Some(NominalOwnerKind::Model) => &["kind", "identity", "node"],
            None => return (at.key("kind"), false),
        },
        Some(_) => return (at.key("kind"), false),
        None => return (at, false),
    };
    match object
        .iter()
        .find(|(key, value)| !members.contains(&key.as_str()) || !value.is_string())
    {
        Some((key, _)) => (at.key(key), !members.contains(&key.as_str())),
        None => (at, false),
    }
}

/// Which nominal preimage a `(node_tag, semantic_form)` pair requires.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NominalKind {
    EnumDeclaration,
    EnumMember,
    Dimension,
    Unit,
}

impl NominalKind {
    #[ix_trace_rs::string_edge]
    fn from_version(version: &str) -> Option<Self> {
        match version {
            "quire.enum-declaration-node/v1" => Some(Self::EnumDeclaration),
            "quire.enum-member-node/v1" => Some(Self::EnumMember),
            "quire.dimension-node/v1" => Some(Self::Dimension),
            "quire.unit-node/v1" => Some(Self::Unit),
            _ => None,
        }
    }
}

enum NominalOwnerKind {
    Source,
    Definition,
    Model,
}

impl NominalOwnerKind {
    #[ix_trace_rs::string_edge]
    fn from_wire(kind: &str) -> Option<Self> {
        match kind {
            "source" => Some(Self::Source),
            "definition" => Some(Self::Definition),
            "model" => Some(Self::Model),
            _ => None,
        }
    }
}

impl NominalKind {
    /// The nominal identity a node of this kind must carry, if any. Only the
    /// four nominal forms carry one; every other form is decided here too, so
    /// a new form is a compile error until it is.
    fn required_by(kind: CheckedNodeKind) -> Option<Self> {
        use CheckedNodeKind as K;
        match kind {
            K::Extension(_) | K::Hybrid(_) | K::TimedModel(_) => None,
            K::ScalarType(ScalarTypeForm::Enum) => Some(Self::EnumDeclaration),
            K::Value(ValueForm::EnumValue) => Some(Self::EnumMember),
            K::ScalarType(ScalarTypeForm::Dimension) => Some(Self::Dimension),
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
                | TemporalForm::Deadline,
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
                | ClaimForm::SynthesisRequest,
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

    fn of(preimage: &NominalIdentityPreimage) -> Self {
        match preimage {
            NominalIdentityPreimage::EnumDeclaration(_) => Self::EnumDeclaration,
            NominalIdentityPreimage::EnumMember(_) => Self::EnumMember,
            NominalIdentityPreimage::Dimension(_) => Self::Dimension,
            NominalIdentityPreimage::Unit(_) => Self::Unit,
        }
    }
}

impl NominalIdentityPreimage {
    /// The declaring owner, for the forms that carry one; an enum member is
    /// owned through its declaration.
    pub(crate) fn owner(&self) -> Option<&NominalOwner> {
        match self {
            Self::EnumDeclaration(declaration) => Some(&declaration.owner),
            Self::Dimension(dimension) => Some(&dimension.owner),
            Self::Unit(unit) => Some(&unit.owner),
            Self::EnumMember(_) => None,
        }
    }

    /// The declared name this preimage fixes, for the forms that carry one:
    /// an enum declaration, a dimension or a declared unit. An enum member
    /// is named through its declaration, not by a name of its own.
    pub(crate) fn qualified_declaration(&self) -> Option<&[Box<str>]> {
        match self {
            Self::EnumDeclaration(declaration) => Some(&declaration.qualified_declaration),
            Self::Dimension(dimension) => Some(&dimension.qualified_declaration),
            Self::Unit(unit) => Some(&unit.qualified_declaration),
            Self::EnumMember(_) => None,
        }
    }

    /// Lowercase SHA-256 of the canonical preimage bytes, computed by
    /// `quire-canonical` under `limit_bytes`, the caller's byte limit. A
    /// preimage whose canonical bytes are longer than `limit_bytes` returns the
    /// encoder's byte-limit refusal and no digest; a length equal to it
    /// digests.
    ///
    /// # Errors
    ///
    /// The `quire-canonical` refusal: [`quire_canonical::Error::Limit`] when
    /// the canonical bytes exceed `limit_bytes`.
    pub fn digest(&self, limit_bytes: u64) -> Result<String, quire_canonical::Error> {
        quire_canonical::sha256(self, quire_canonical::Limits::new(limit_bytes))
            .map(|digest| digest.to_string())
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
/// its members and to members of its nominal preimage.
struct Site<'a> {
    node: &'a CheckedSemanticNodeV2,
    position: usize,
}

impl Site<'_> {
    fn member(&self, member: &str) -> JsonPointer {
        node_pointer(self.position).key(member)
    }

    fn preimage(&self, members: &[&str]) -> JsonPointer {
        members.iter().fold(
            self.member("nominal_identity_preimage"),
            |pointer, member| pointer.key(member),
        )
    }
}

/// Validates every node's nominal binding, key re-derivation, owner join,
/// semantic rules and cross-field joins. `kinds` holds each node's decoded
/// kind. Each refusal points at the member it is about. Every digest and term
/// order is computed under `bytes`, the reader's byte limit; a preimage the
/// encoder refuses is refused at the node's `node_id` (a key) or at the term
/// (an order), as `invalid_semantic_graph`.
pub(super) fn validate_nominal_nodes(
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &BTreeMap<&CheckedNodeId, usize>,
    lock: &CheckedPackageLockV2,
    models: &[DomainModel],
    meter: &mut WorkMeter,
    bytes: u64,
) -> Result<(), ValidationFailure> {
    let selected_models = models
        .iter()
        .map(|model| (model.identity.as_ref(), model))
        .collect();
    let graph = NominalGraph {
        nodes,
        kinds,
        index,
        selected_models,
    };
    let mut roots = BTreeSet::new();
    for (position, (node, kind)) in nodes.iter().zip(kinds).enumerate() {
        let site = Site { node, position };
        let required = NominalKind::required_by(*kind);
        let preimage = match (required, &node.nominal_identity_preimage) {
            (None, None) => continue,
            (Some(kind), Some(preimage)) if NominalKind::of(preimage) == kind => preimage,
            // Required and absent: the node lacks the member.
            (Some(_), None) => return Err(invalid(node_pointer(position))),
            _ => return Err(invalid(site.preimage(&[]))),
        };
        meter.charge(1, || site.preimage(&[]))?;
        require(
            preimage
                .digest(bytes)
                .is_ok_and(|digest| digest == node.node_id.digest.as_ref()),
            || site.member("node_id"),
        )?;
        match preimage {
            NominalIdentityPreimage::EnumDeclaration(declaration) => {
                validate_enum_declaration(declaration, &site, lock, meter)?;
            }
            NominalIdentityPreimage::EnumMember(member) => {
                validate_enum_member(&site, member, &graph)?;
            }
            NominalIdentityPreimage::Dimension(dimension) => {
                validate_dimension(&site, dimension, &graph, lock, meter, bytes)?;
            }
            NominalIdentityPreimage::Unit(unit) => {
                validate_unit(&site, unit, &graph, lock, meter)?;
                if unit.target_unit_node_id.is_none() {
                    // A dimension has exactly one targetless root unit.
                    require(roots.insert(unit.dimension_node_id.clone()), || {
                        site.preimage(&["target_unit_node_id"])
                    })?;
                }
            }
        }
    }
    Ok(())
}

struct NominalGraph<'a> {
    nodes: &'a [CheckedSemanticNodeV2],
    kinds: &'a [CheckedNodeKind],
    index: &'a BTreeMap<&'a CheckedNodeId, usize>,
    selected_models: BTreeMap<&'a str, &'a DomainModel>,
}

impl NominalGraph<'_> {
    fn preimage(&self, id: &CheckedNodeId) -> Option<&NominalIdentityPreimage> {
        self.index
            .get(id)
            .and_then(|position| self.nodes.get(*position))
            .and_then(|node| node.nominal_identity_preimage.as_ref())
    }

    fn model_variant_cases(&self, id: &CheckedNodeId) -> Option<&BTreeSet<Box<str>>> {
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
        NominalOwner::Definition {
            authority,
            identity,
        } => lock.definition_selections.iter().any(|definition| {
            definition.authority == *authority && definition.identity == *identity
        }),
        // The lock selects whole domain packages; the node is not a lock
        // member, so the join is by package identity, and the owner's
        // identity and node must each be nonempty (FR-038-AC-45). The owner
        // is content-only: it carries no version, and neither does the lock's
        // selection row, which binds by identity and digest. `validate_lock` refuses a
        // `model_selections` array holding two entries with the same
        // identity (a repeat or a different digest) before this join ever
        // runs, so the lock guarantees at most one selection per model
        // identity — that single-selection invariant is what makes joining
        // by identity alone sound.
        NominalOwner::Model { identity, node } => {
            !node.is_empty()
                && !identity.is_empty()
                && lock
                    .model_selections
                    .iter()
                    .any(|model| model.identity == *identity)
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
/// Shared by the three nominal preimage kinds that carry one and by
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
    declaration: &EnumDeclarationPreimage,
    site: &Site<'_>,
    lock: &CheckedPackageLockV2,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    validate_owner(&declaration.owner, lock, || site.preimage(&["owner"]))?;
    validate_qualified_name(&declaration.qualified_declaration, &|| {
        site.preimage(&["qualified_declaration"])
    })?;
    require(!declaration.members.is_empty(), || {
        site.preimage(&["members"])
    })?;
    let member_at = |index: usize| site.preimage(&["members"]).index(index);
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
    member: &EnumMemberPreimage,
    graph: &NominalGraph<'_>,
) -> Result<(), ValidationFailure> {
    let node = site.node;
    if graph.is_model_variant(&member.declaration_node_id) {
        require(!member.case.is_empty(), || site.preimage(&["case"]))?;
        if let Some(cases) = graph.model_variant_cases(&member.declaration_node_id) {
            require(cases.contains(&member.case), || site.preimage(&["case"]))?;
        }
    } else {
        require(is_identifier(&member.case), || site.preimage(&["case"]))?;
        let Some(NominalIdentityPreimage::EnumDeclaration(declaration)) =
            graph.preimage(&member.declaration_node_id)
        else {
            return Err(invalid(site.preimage(&["declaration_node_id"])));
        };
        require(declaration.members.contains(&member.case), || {
            site.preimage(&["case"])
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
        .map_err(|_| invalid(site.preimage(&["declaration_node_id"])))?;
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
    dimension: &DimensionPreimage,
    graph: &NominalGraph<'_>,
    lock: &CheckedPackageLockV2,
    meter: &mut WorkMeter,
    bytes: u64,
) -> Result<(), ValidationFailure> {
    let node = site.node;
    validate_owner(&dimension.owner, lock, || site.preimage(&["owner"]))?;
    validate_qualified_name(&dimension.qualified_declaration, &|| {
        site.preimage(&["qualified_declaration"])
    })?;
    require(node.semantic_type == node.node_id, || {
        site.member("semantic_type")
    })?;
    let term_at = |index: usize| site.preimage(&["terms"]).index(index);
    let mut keys = Vec::with_capacity(dimension.terms.len());
    let mut bases = BTreeSet::new();
    for (index, term) in dimension.terms.iter().enumerate() {
        meter.charge(1, || term_at(index))?;
        let exponent_at = || term_at(index).key("exponent");
        let magnitude = integer_magnitude(&term.exponent).ok_or_else(|| invalid(exponent_at()))?;
        require(!magnitude.zero, exponent_at)?;
        let base_at = || term_at(index).key("dimension_node_id");
        require(bases.insert(&term.dimension_node_id), base_at)?;
        let Some(NominalIdentityPreimage::Dimension(base)) =
            graph.preimage(&term.dimension_node_id)
        else {
            return Err(invalid(base_at()));
        };
        require(base.terms.is_empty(), base_at)?;
        // A term's place in the order is its canonical bytes under
        // `quire-canonical`, not its `serde_json` bytes.
        keys.push(
            quire_canonical::to_vec(term, quire_canonical::Limits::new(bytes))
                .map_err(|_| invalid(term_at(index)))?,
        );
    }
    // The later term of the first out-of-order pair is at fault.
    if let Some(pair) = keys
        .windows(2)
        .position(|pair| !matches!(pair, [left, right] if left < right))
    {
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
    unit: &UnitPreimage,
    graph: &NominalGraph<'_>,
    lock: &CheckedPackageLockV2,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let node = site.node;
    validate_owner(&unit.owner, lock, || site.preimage(&["owner"]))?;
    validate_qualified_name(&unit.qualified_declaration, &|| {
        site.preimage(&["qualified_declaration"])
    })?;
    require(
        matches!(
            graph.preimage(&unit.dimension_node_id),
            Some(NominalIdentityPreimage::Dimension(_))
        ),
        || site.preimage(&["dimension_node_id"]),
    )?;
    validate_rational(&unit.scale, meter, &|| site.preimage(&["scale"]))?;
    validate_rational(&unit.offset, meter, &|| site.preimage(&["offset"]))?;
    require(node.semantic_type == unit.dimension_node_id, || {
        site.member("semantic_type")
    })?;
    let mut expected = vec![&unit.dimension_node_id];
    match &unit.target_unit_node_id {
        None => {
            require(is_rational(&unit.scale, "1", "1"), || {
                site.preimage(&["scale"])
            })?;
            require(is_rational(&unit.offset, "0", "1"), || {
                site.preimage(&["offset"])
            })?;
        }
        Some(target) => {
            require(
                integer_magnitude(&unit.scale.numerator).is_none_or(|m| !m.zero),
                || site.preimage(&["scale", "numerator"]),
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
    unit: &UnitPreimage,
    graph: &NominalGraph<'_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let link_at = |position: usize| {
        node_pointer(position)
            .key("nominal_identity_preimage")
            .key("target_unit_node_id")
    };
    let mut visited = BTreeSet::from([&site.node.node_id]);
    let mut from = site.position;
    let mut next = unit.target_unit_node_id.as_ref();
    while let Some(target) = next {
        meter.charge(1, || link_at(from))?;
        require(visited.insert(target), || link_at(from))?;
        let Some(NominalIdentityPreimage::Unit(target_unit)) = graph.preimage(target) else {
            return Err(invalid(link_at(from)));
        };
        require(
            target_unit.dimension_node_id == unit.dimension_node_id,
            || link_at(from),
        )?;
        // `graph.preimage` resolved `target`, so it is indexed.
        from = graph.index.get(target).copied().unwrap_or(from);
        next = target_unit.target_unit_node_id.as_ref();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ix_trace_rs::trace;

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
