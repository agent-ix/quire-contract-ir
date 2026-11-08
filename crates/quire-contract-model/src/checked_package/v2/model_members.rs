//! FR-322 "Model-owned members" and "Reference conformance" (QSpec STD-100,
//! STD-101, STD-102): a `field` or `operation` member whose declaring node is
//! a model declaration node resolves through the domain package the lock
//! selects, because that node's body is `aggregate{[]}` and names no member.
//!
//! The four steps, as this module implements them:
//!
//! 1. **Selection evidence** ([`admit_selection`]). Each `model_root` owner's
//!    identity selects one document, looked up by identity in the caller's
//!    evidence; its own identity is compared with the selected one
//!    (selections bind by identity; no version is read), and its
//!    declarations read ([`read_semantic_ir`]).
//! 2. **Owner recovery** ([`ModelOwners::recover`]). Every object type, record
//!    value type and relationship declaration of each admitted document has its
//!    `ModelDeclarationNode` key recomputed; a declaring node's owner is the
//!    declaration whose key, tag and form all equal the node's, and the node's
//!    fixed members (`semantic_type` is itself, no `declaration`, no
//!    `recursion_group`, an empty body) must hold.
//! 3. **Resolution** ([`DomainModel::resolve`]), charged to the reader's
//!    `work` limit at the selection's row ([`Budget`]): an operation by
//!    every ancestor edge, member and redefinition pair it walks, a field by
//!    one unit per entry of the field table built once at admission
//!    (`model_fields`). The member name resolves among the declaring type's
//!    exposed effective members, own and inherited, less every member a
//!    redefinition hides and every redefining member of a less derived owner
//!    (the most-derived-redefiner rule).
//! 4. **Member type** ([`DomainModel::slot_type`]). The element-type,
//!    multiplicity and presence tables give a [`MemberType`], whose node key
//!    ([`MemberType::node_key`]) is the structural key QSL FR-092 and FR-094
//!    give that type (an owned declaration key for a record), so an application's
//!    `result_type` or argument type is compared by key.
//!
//! Scope. [`read_semantic_ir`] reads the declarations these steps consult:
//! object types, processes, state machines, persistence interfaces,
//! record value types, variant types, systems interfaces,
//! integer value types and relationship declarations, with their cases,
//! fields, operations, supertypes and member redefinitions. A type of any
//! other FR-208 meaning is recorded as declared,
//! so a `typeRef` naming it resolves to a declaration with no element type,
//! but its own body is not read. Field and operation `redefines` references
//! contribute to the effective-member resolution.

use super::model_fields::{
    DeclarationIndex, DeclaredNode, FieldTables, RecordBinderTypes, Selected,
};
use super::{
    BudgetStop, CheckedNodeOwner, CheckedNodeTag, CheckedSemanticNodeV2, CheckedTerm,
    NativeJoinBudget, NativeJoinChargeKind, NativeJoinLocus, NativeJoinLocusSource,
    PackageReadResource, PackageReadStage, ValidationFailure, WorkMeter,
};
use crate::checked_package::evidence::CheckedPackageEvidence;
use crate::checked_package::shared::{
    CheckedNodeId, CheckedNodePathSegment, CheckedPackageRefusalCause, CheckedPackageRefusalCode,
    JsonPointer,
};
use ix_trace_rs::string_edge;
use serde::Deserialize;

/// The fixed declaration body shape in raw intake and admitted typed nodes.
pub(super) trait EmptyAggregateBody {
    fn is_empty_aggregate(&self) -> bool;
}

impl EmptyAggregateBody for CheckedTerm {
    fn is_empty_aggregate(&self) -> bool {
        matches!(self, Self::Aggregate { members } if members.is_empty())
    }
}

/// The form of the graph-stage requests of the model checks: the tag of the
/// selected package's `model_root` node.
const MODEL: &str = "model";
use std::sync::Arc;

pub(super) type UnitRefs = Arc<SortedMap<Box<str>, CheckedNodeId>>;
pub(super) const DURATION_RESOLVED_PREFIX: &str = "ix://quire/resolved-duration/";

mod charged;
mod derive;
mod document;
mod intake_origin;
mod option_types;
mod view;
pub(super) use charged::{
    arc_bytes, arc_str_bytes, checked_member_type_bytes, field_decl_bytes, node_id_bytes,
    relationship_decl_bytes, SortedMap, SortedSet,
};
use intake_origin::{retained_origin, IntakeDeclarationOrigin, OriginView};
use view::{List, View};

/// Puts `name` into the sorted `seen`; `false` when it was already there.
fn first_visit<'a>(seen: &mut Vec<&'a str>, name: &'a str) -> bool {
    match seen.binary_search(&name) {
        Ok(_) => false,
        Err(at) => {
            seen.insert(at, name);
            true
        }
    }
}

const NATIVE_PREFIX: &str = "ix://quire/native/";

/// FR-208 meaning ids, as the constructs table of a Semantic IR document
/// binds them.
mod meaning {
    /// Every FR-208 meaning accepted by the selected Semantic IR document.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(super) enum Kind {
        SequenceType,
        MapType,
        OptionType,
        ReferenceType,
        ObjectType,
        ValueType,
        RecordValueType,
        TupleValueType,
        VariantType,
        EventType,
        StateMachine,
        Process,
        PersistenceInterface,
        Namespace,
        Population,
        SystemsPart,
        SystemsPort,
        SystemsInterface,
        SystemsConnection,
        SystemsAllocation,
    }

    impl Kind {
        #[ix_trace_rs::string_edge]
        pub(super) fn from_wire(value: &str) -> Option<Self> {
            match value {
                "quire.meaning.model.object-type/v1" => Some(Self::ObjectType),
                "quire.meaning.model.value-type/v1" => Some(Self::ValueType),
                "quire.meaning.model.record-value-type/v1" => Some(Self::RecordValueType),
                "quire.meaning.model.tuple-value-type/v1" => Some(Self::TupleValueType),
                "quire.meaning.model.variant-type/v1" => Some(Self::VariantType),
                "quire.meaning.model.event-type/v1" => Some(Self::EventType),
                "quire.meaning.model.state-machine/v1" => Some(Self::StateMachine),
                "quire.meaning.model.process/v1" => Some(Self::Process),
                "quire.meaning.model.persistence-interface/v1" => Some(Self::PersistenceInterface),
                "quire.meaning.model.namespace/v1" => Some(Self::Namespace),
                "quire.meaning.model.population/v1" => Some(Self::Population),
                "quire.meaning.systems.part/v1" => Some(Self::SystemsPart),
                "quire.meaning.systems.port/v1" => Some(Self::SystemsPort),
                "quire.meaning.systems.interface/v1" => Some(Self::SystemsInterface),
                "quire.meaning.systems.connection/v1" => Some(Self::SystemsConnection),
                "quire.meaning.systems.allocation/v1" => Some(Self::SystemsAllocation),
                _ => None,
            }
        }
    }
}

/// A refusal one of the four steps determined, before the caller locates it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ModelRefusal {
    pub(super) code: CheckedPackageRefusalCode,
    pub(super) cause: CheckedPackageRefusalCause,
}

impl ModelRefusal {
    const fn new(code: CheckedPackageRefusalCode, cause: CheckedPackageRefusalCause) -> Self {
        Self { code, cause }
    }

    /// `ill_typed`/`operator-ineligible`.
    pub(super) const fn ineligible() -> Self {
        Self::new(
            CheckedPackageRefusalCode::IllTyped,
            CheckedPackageRefusalCause::OperatorIneligible,
        )
    }

    /// `missing_declaration`/`missing-selection`.
    pub(super) const fn unselected() -> Self {
        Self::new(
            CheckedPackageRefusalCode::MissingDeclaration,
            CheckedPackageRefusalCause::MissingSelection,
        )
    }

    /// `invalid_model_binding`/`malformed-declaration`.
    pub(super) const fn malformed() -> Self {
        Self::new(
            CheckedPackageRefusalCode::InvalidModelBinding,
            CheckedPackageRefusalCause::MalformedDeclaration,
        )
    }

    /// `missing_declaration`/`missing-name`.
    pub(super) const fn missing_name() -> Self {
        Self::new(
            CheckedPackageRefusalCode::MissingDeclaration,
            CheckedPackageRefusalCause::MissingName,
        )
    }

    /// `ambiguous_declaration`/`ambiguous-name`.
    pub(super) const fn ambiguous() -> Self {
        Self::new(
            CheckedPackageRefusalCode::AmbiguousDeclaration,
            CheckedPackageRefusalCause::AmbiguousName,
        )
    }
}

/// `{lower, upper, ordered, unique}`; `upper` is `None` when unbounded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Multiplicity {
    pub(super) lower: u64,
    pub(super) upper: Option<u64>,
    pub(super) ordered: bool,
    pub(super) unique: bool,
}

/// A declared `typeRef` with its multiplicity: a field, parameter or result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct TypedSlot {
    /// The declared `name`, when the slot's member carries one: an
    /// operation parameter's name, which FR-346 compares with a frame entry's
    /// `parameters`.
    pub(super) name: Option<Box<str>>,
    pub(super) type_ref: Option<SelectedTypeRef>,
    pub(super) multiplicity: Multiplicity,
}

/// The shape of an admitted type name.
enum Admitted<'a> {
    Native,
    Unit(&'a str),
    Declared(&'a str),
    Map(&'a str, &'a str),
}

/// A type reference classified while the selected model document is admitted.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum SelectedTypeRef {
    Native(NativeTypeRef),
    Unit(Box<str>),
    Declared(Box<str>),
    /// A map's value type; retained for timed clock-place resolution.
    Map(Box<str>, Box<str>),
}

/// A core value definition selected by an admitted model document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum SelectedTypeDefinition {
    Sequence(SelectedTypeRef),
    Map(SelectedTypeRef),
    Option(SelectedTypeRef),
    Reference(SelectedTypeRef),
}

#[derive(Clone, Copy)]
enum SelectedValueLayer {
    Sequence,
    Map,
    Option,
}

/// The closed native names accepted by the selected model grammar.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum NativeTypeRef {
    Boolean,
    Integer,
    Rational,
    Decimal,
    Float32,
    Float64,
    Text,
    String,
    Uuid,
    Timestamp,
    Bytes,
    JsonObject,
    Duration,
    Clock,
}

impl SelectedTypeRef {
    #[string_edge]
    fn from_admitted_name(name: &str) -> Option<Self> {
        if let Some(inner) = name.strip_prefix("Map<").and_then(|s| s.strip_suffix('>')) {
            let (key, value) = inner.split_once(", ")?;
            if key.is_empty() || value.is_empty() || value.contains(',') {
                return None;
            }
            return Some(Self::Map(key.into(), value.into()));
        }
        if let Some(unit) = name.strip_prefix("ix://quire/unit/") {
            return (!unit.is_empty()).then(|| Self::Unit(name.into()));
        }
        if let Some(native) = name.strip_prefix(NATIVE_PREFIX) {
            return NativeTypeRef::from_name(native).map(Self::Native);
        }
        if name.is_empty() {
            None
        } else {
            Some(Self::Declared(name.into()))
        }
    }
}

impl SelectedTypeRef {
    /// [`Self::from_admitted_name`] with the owned text requested first.
    fn charged(name: &str, budget: &mut Budget<'_, '_>) -> Result<Option<Self>, ValidationFailure> {
        let owned = match Self::admit(name) {
            None => return Ok(None),
            Some(Admitted::Native) => 0,
            Some(Admitted::Unit(_) | Admitted::Declared(_)) => name.len(),
            Some(Admitted::Map(key, value)) => key.len() + value.len(),
        };
        budget.request(owned, "declarations")?;
        Ok(Self::from_admitted_name(name))
    }

    /// The shape of an admitted type name, without owning any of it.
    fn admit(name: &str) -> Option<Admitted<'_>> {
        if let Some(inner) = name.strip_prefix("Map<").and_then(|s| s.strip_suffix('>')) {
            let (key, value) = inner.split_once(", ")?;
            if key.is_empty() || value.is_empty() || value.contains(',') {
                return None;
            }
            return Some(Admitted::Map(key, value));
        }
        if let Some(unit) = name.strip_prefix("ix://quire/unit/") {
            return (!unit.is_empty()).then_some(Admitted::Unit(name));
        }
        if let Some(native) = name.strip_prefix(NATIVE_PREFIX) {
            return NativeTypeRef::from_name(native).map(|_| Admitted::Native);
        }
        if name.is_empty() {
            None
        } else {
            Some(Admitted::Declared(name))
        }
    }

    /// Whether an admitted type name is a unit reference, which owns nothing.
    fn is_unit_name(name: &str) -> bool {
        name.strip_prefix("Map<")
            .and_then(|s| s.strip_suffix('>'))
            .is_none()
            && name
                .strip_prefix("ix://quire/unit/")
                .is_some_and(|unit| !unit.is_empty())
    }
}

/// Whether `text` is the canonical decimal spelling of `number`.
fn decimal_text_is(number: i64, text: &str) -> bool {
    use std::fmt::Write as _;
    struct Compare<'t>(&'t str);
    impl std::fmt::Write for Compare<'_> {
        fn write_str(&mut self, part: &str) -> std::fmt::Result {
            self.0 = self.0.strip_prefix(part).ok_or(std::fmt::Error)?;
            Ok(())
        }
    }
    let mut remaining = Compare(text);
    write!(remaining, "{number}").is_ok() && remaining.0.is_empty()
}

impl NativeTypeRef {
    #[string_edge]
    fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "Boolean" => Self::Boolean,
            "Integer" => Self::Integer,
            "Rational" => Self::Rational,
            "Decimal" => Self::Decimal,
            "Float32" => Self::Float32,
            "Float64" => Self::Float64,
            "Text" => Self::Text,
            "String" => Self::String,
            "UUID" => Self::Uuid,
            "Timestamp" => Self::Timestamp,
            "Bytes" => Self::Bytes,
            "JsonObject" => Self::JsonObject,
            "Duration" => Self::Duration,
            "Clock" => Self::Clock,
            _ => return None,
        })
    }
}

/// One field member record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct FieldDecl {
    /// IR node identity, `<owner>/<name>`.
    pub(super) identity: Box<str>,
    pub(super) slot: TypedSlot,
    /// Presence `optional`.
    pub(super) optional: bool,
    /// The IR node identity of the member this one redefines.
    pub(super) redefines: Option<Box<str>>,
}

/// One operation member record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct OperationDecl {
    /// IR node identity, `<owner>/<name>`.
    pub(super) identity: Box<str>,
    pub(super) parameters: Vec<TypedSlot>,
    pub(super) result: Option<TypedSlot>,
    /// The admitted selected document's declared frame, before carrier comparison.
    pub(super) frame: SourceOperationFrame,
    /// The IR node identity of the member this one redefines.
    pub(super) redefines: Option<Box<str>>,
}

/// Shallow Semantic IR operation-frame members; absent arrays mean empty effects.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct SourceOperationFrame {
    #[serde(default)]
    pub(super) modifies: Vec<Box<str>>,
    #[serde(default)]
    pub(super) creates: Vec<Box<str>>,
    #[serde(default)]
    pub(super) deletes: Vec<Box<str>>,
    #[serde(default)]
    pub(super) modifies_receiver: Vec<Box<str>>,
    #[serde(default)]
    pub(super) deletes_receiver: bool,
    #[serde(default)]
    pub(super) deletes_parameters: Vec<Box<str>>,
}

/// One object, record value type or systems interface declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct FieldBearingTypeDecl {
    /// Whether it carries `interfaceFeatures` (a systems interface).
    pub(super) interface: bool,
    pub(super) supertypes: Vec<Box<str>>,
    pub(super) fields: Vec<FieldDecl>,
    pub(super) operations: Vec<OperationDecl>,
}

/// One end of a selected relationship, including its authored role.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct RelationshipEnd {
    pub(super) role: Option<Box<str>>,
    pub(super) type_ref: Box<str>,
    pub(super) multiplicity: Multiplicity,
}

/// An FCD relationship nested under its source object type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct RelationshipDecl {
    pub(super) source: RelationshipEnd,
    pub(super) target: RelationshipEnd,
    pub(super) direction: RelationshipDirection,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RelationshipDirection {
    SourceToTarget,
    TargetToSource,
    Bidirectional,
    Undirected,
}

impl RelationshipDirection {
    #[string_edge]
    fn from_wire(text: &str) -> Option<Self> {
        match text {
            "source-to-target" => Some(Self::SourceToTarget),
            "target-to-source" => Some(Self::TargetToSource),
            "bidirectional" => Some(Self::Bidirectional),
            "undirected" => Some(Self::Undirected),
            _ => None,
        }
    }

    pub(super) fn admits(self, forward: bool) -> bool {
        matches!(self, Self::Bidirectional | Self::Undirected)
            || (forward && self == Self::SourceToTarget)
            || (!forward && self == Self::TargetToSource)
    }
}

/// The ends a named role selects on a relationship.
#[derive(Clone, Copy, Debug)]
pub(super) struct SelectedEnds<'a> {
    /// The end carrying the named role.
    pub(super) receiver: &'a RelationshipEnd,
    /// The other end.
    pub(super) destination: &'a RelationshipEnd,
    /// Whether the named role is the source end's.
    pub(super) forward: bool,
}

/// Why a role name selects no single end.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum EndSelectionError {
    /// No end carries the role.
    MissingName,
    /// Both ends carry the role.
    Ambiguous,
}

impl RelationshipDecl {
    /// The `(receiver, destination)` ends of a traversal: forward reads from
    /// the source end to the target end, inverse the other way.
    pub(super) fn ends(&self, forward: bool) -> (&RelationshipEnd, &RelationshipEnd) {
        if forward {
            (&self.source, &self.target)
        } else {
            (&self.target, &self.source)
        }
    }

    /// The one role-to-end rule of admission and of the typed accessor: the end
    /// whose authored role equals `name` is the receiver (forward when it is
    /// the source end) and the other end is the destination.
    pub(super) fn select_ends(&self, name: &str) -> Result<SelectedEnds<'_>, EndSelectionError> {
        let forward = match (
            self.source.role.as_deref() == Some(name),
            self.target.role.as_deref() == Some(name),
        ) {
            (true, false) => true,
            (false, true) => false,
            (false, false) => return Err(EndSelectionError::MissingName),
            (true, true) => return Err(EndSelectionError::Ambiguous),
        };
        let (receiver, destination) = self.ends(forward);
        Ok(SelectedEnds {
            receiver,
            destination,
            forward,
        })
    }

    /// FR-152's single destination-multiplicity derivation for a named end.
    pub(super) fn navigation_type(
        &self,
        forward: bool,
        destination_id: CheckedNodeId,
    ) -> Result<MemberType, ModelRefusal> {
        let multiplicity = self.ends(forward).1.multiplicity;
        let Some(upper) = multiplicity.upper else {
            return Err(ModelRefusal::new(
                CheckedPackageRefusalCode::UnsupportedConstruct,
                CheckedPackageRefusalCause::ExpressionForm,
            ));
        };
        if multiplicity.ordered {
            return Err(ModelRefusal::new(
                CheckedPackageRefusalCode::UnsupportedConstruct,
                CheckedPackageRefusalCause::ExpressionForm,
            ));
        }
        let reference = MemberType::Reference(destination_id);
        match (multiplicity.lower, upper) {
            (1, 1) => Ok(reference),
            (0, 1) => Ok(MemberType::Option(Box::new(reference))),
            (lower, upper) => Ok(MemberType::Collection {
                kind: CollectionKind::of(false, multiplicity.unique),
                element: Box::new(reference),
                bounds: Some((lower, upper)),
            }),
        }
    }
}

/// An integer value type bound as `Int[lo, hi]`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct IntegerBounds {
    pub(super) lower: i128,
    pub(super) upper: i128,
}

/// A selected `Text[min, max; profile]` value type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct TextBounds {
    pub(super) lower: u64,
    pub(super) upper: u64,
    pub(super) profile: Box<str>,
}

/// Exact limits of a selected `Rational[n1, n2; d1, d2]`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RationalBounds {
    pub(super) numerator_min: i128,
    pub(super) numerator_max: i128,
    pub(super) denominator_min: i128,
    pub(super) denominator_max: i128,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum ValueBounds {
    Integer(IntegerBounds),
    Rational(RationalBounds, Option<CheckedNodeId>),
    Text(TextBounds),
}

/// The declarations of one admitted domain package document that FR-322's
/// model-owned member rule consults.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct DomainModel {
    pub(super) identity: Box<str>,
    /// The package containing the anonymous member type nodes read here.
    pub(super) checked_package: Box<str>,
    /// Node position of this package's `model_root`.
    pub(super) root: usize,
    /// The reader's byte limit for declarations selected from this model.
    pub(super) bytes: u64,
    /// Object types and systems interfaces, by IR node identity.
    pub(super) object_types: SortedMap<Box<str>, FieldBearingTypeDecl>,
    /// State machine declarations share field and operation parsing with object types.
    pub(super) state_machines: SortedSet<Box<str>>,
    /// Process declarations share field and operation parsing with object types.
    pub(super) processes: SortedSet<Box<str>>,
    /// Persistence-interface declarations have a selected owner key.
    pub(super) persistence_interfaces: SortedSet<Box<str>>,
    /// Canonical admitted persistence operation declarations, separate from object types.
    pub(super) persistence_operations: SortedMap<Box<str>, FieldBearingTypeDecl>,
    /// Record value types, with their own and inherited fields.
    pub(super) record_types: SortedMap<Box<str>, FieldBearingTypeDecl>,
    /// Event type declarations, by their exact selected identities.
    pub(super) event_types: SortedSet<Box<str>>,
    /// Ordered positions of each selected tuple value type.
    pub(super) tuple_types: SortedMap<Box<str>, Vec<SelectedTypeRef>>,
    /// Value types by IR node identity; `None` for an unmapped value type.
    pub(super) value_types: SortedMap<Box<str>, Option<ValueBounds>>,
    /// Core Semantic IR aliases, resolved before model member type mapping.
    aliases: SortedMap<Box<str>, Box<str>>,
    /// Exact semantic cases, original identities, and authored positions.
    pub(super) variant_types: SortedMap<Box<str>, SortedMap<Box<str>, SelectedVariantCase>>,
    /// Selected FCD dimension declarations by their exact identity.
    pub(super) dimensions: SortedMap<Box<str>, SelectedDimension>,
    /// Selected FCD unit declarations by their exact identity.
    pub(super) units: SortedMap<Box<str>, SelectedUnit>,
    /// Selected sequence, map, option and reference value definitions.
    pub(super) collection_types: SortedMap<Box<str>, SelectedTypeDefinition>,
    /// Checked-package unit `typeRef` to the exact admitted unit node ID.
    pub(super) unit_refs: UnitRefs,
    /// Whether this selection declares its own `s`; such a declaration
    /// prevents a Duration member from falling back to the source unit.
    own_seconds_declared: bool,
    /// Relationship declarations, by IR node identity.
    pub(super) relationships: SortedMap<Box<str>, RelationshipDecl>,
    /// Declared populations, by IR node identity in the selected document.
    pub(super) populations: SortedSet<Box<str>>,
    /// Object type identities covered by each selected population.
    pub(super) population_members: SortedMap<Box<str>, SortedSet<Box<str>>>,
    /// Systems connection declarations that also back relation nodes.
    pub(super) relation_nodes: SortedSet<Box<str>>,
    /// Types of every other FR-208 meaning, by IR node identity.
    other_types: SortedMap<Box<str>, meaning::Kind>,
    /// The field tables of [`Self::object_types`], empty until the lock stage
    /// builds them (`build_field_tables`).
    pub(super) fields: FieldTables,
}

/// One case retained from an admitted selected variant document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct SelectedVariantCase {
    pub(super) identity: Box<str>,
    pub(super) position: usize,
    pub(super) payload: Option<SelectedTypeRef>,
}

/// An admitted FCD dimension, retained for its structural checked-node join.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct SelectedDimension {
    pub(super) name: Box<str>,
    pub(super) base: bool,
    pub(super) terms: Vec<(Box<str>, Box<str>)>,
}

/// An admitted FCD unit, retained for its structural checked-node join.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct SelectedUnit {
    pub(super) name: Box<str>,
    pub(super) dimension: Box<str>,
    pub(super) target: Option<Box<str>>,
    pub(super) scale: Box<str>,
    pub(super) offset: Box<str>,
}

/// A model declaration node's `(node_tag, semantic_form)`.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum DeclarationForm {
    ObjectType,
    StateMachine,
    Process,
    PersistenceInterface,
    RecordValueType,
    TupleValueType,
    VariantType,
    SystemsInterface,
    Relationship,
    Population,
}

impl DeclarationForm {
    pub(super) const fn tag(self) -> &'static str {
        match self {
            Self::ObjectType
            | Self::StateMachine
            | Self::Process
            | Self::PersistenceInterface
            | Self::RecordValueType
            | Self::TupleValueType
            | Self::VariantType
            | Self::SystemsInterface => "model",
            Self::Relationship | Self::Population => "relation",
        }
    }

    pub(super) const fn form(self) -> &'static str {
        match self {
            Self::ObjectType => "object_type",
            Self::StateMachine => "state_machine",
            Self::Process => "process",
            Self::PersistenceInterface => "persistence_interface",
            Self::RecordValueType => "record_value_type",
            Self::TupleValueType => "tuple_value_type",
            Self::VariantType => "variant_type",
            Self::SystemsInterface => "systems_interface",
            Self::Relationship => "relationship",
            Self::Population => "population",
        }
    }
}

/// The structural identity a selected model declaration must carry. The
/// owner join still checks this label against the actual graph node.
pub(super) fn declaration_id(identity: &str, form: DeclarationForm, node: &str) -> CheckedNodeId {
    CheckedNodeId {
        package: identity.into(),
        path: vec![
            CheckedNodePathSegment::Name(form.tag().into()),
            CheckedNodePathSegment::Name(form.form().into()),
            CheckedNodePathSegment::Name(node.into()),
        ],
        ordinal: 0,
    }
}

pub(super) fn declaration_id_in(
    identity: &str,
    form: DeclarationForm,
    node: &str,
    budget: &mut Budget<'_, '_>,
) -> Result<CheckedNodeId, ValidationFailure> {
    let mut path = budget.vec(3, "member_type")?;
    for text in [form.tag(), form.form(), node] {
        path.push(CheckedNodePathSegment::Name(
            budget.text(text, "member_type")?,
        ));
    }
    Ok(CheckedNodeId {
        package: budget.text(identity, "member_type")?,
        path,
        ordinal: 0,
    })
}

/// The last `/` segment of an IR node identity: a member's name.
pub(super) fn member_name(identity: &str) -> &str {
    identity.rsplit('/').next().unwrap_or(identity)
}

/// A collection form of FR-322 step 4's multiplicity table.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CollectionKind {
    Set,
    Bag,
    Sequence,
    OrderedSet,
}

impl CollectionKind {
    pub(super) const fn of(ordered: bool, unique: bool) -> Self {
        match (ordered, unique) {
            (false, true) => Self::Set,
            (false, false) => Self::Bag,
            (true, false) => Self::Sequence,
            (true, true) => Self::OrderedSet,
        }
    }

    pub(super) const fn form(self) -> &'static str {
        match self {
            Self::Set => "set",
            Self::Bag => "bag",
            Self::Sequence => "sequence",
            Self::OrderedSet => "ordered_set",
        }
    }
}

/// A member type FR-322 step 4 derives.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum MemberType {
    Boolean,
    Integer,
    Float32,
    Float64,
    String,
    Uuid,
    Timestamp,
    Bytes,
    JsonObject,
    /// The timed-model clock marker used by checked clock-place paths.
    Clock,
    /// `Int[lo, hi]`.
    IntRange(IntegerBounds),
    /// `Rational[n1, n2; d1, d2]`.
    RationalRange(RationalBounds),
    /// `Quantity<u, Rational[n1, n2; d1, d2]>`.
    CarriedRational(RationalBounds, CheckedNodeId),
    /// `Text[min, max; profile]`.
    TextBounds(TextBounds),
    /// `Reference<O>`, over `O`'s model declaration node.
    Reference(CheckedNodeId),
    /// A value record, by its owner-bearing declaration node ID.
    Record(CheckedNodeId),
    /// A selected, self-typed model tuple declaration.
    Tuple(CheckedNodeId),
    /// A selected, self-typed model variant declaration.
    Variant(CheckedNodeId),
    /// `Quantity<u>`, by the checked unit's node ID.
    Quantity(CheckedNodeId),
    /// `Option<X>`.
    Option(Box<MemberType>),
    /// `K<E>`, or `K<E>[l, u]` when bounded.
    Collection {
        kind: CollectionKind,
        element: Box<MemberType>,
        bounds: Option<(u64, u64)>,
    },
}

impl MemberType {
    /// Builds the selected member type's structural identity, reserving every
    /// owned path, text, wrapper and shared-node block before construction.
    pub(super) fn node_id_in(
        &self,
        package: &str,
        budget: &mut Budget<'_, '_>,
    ) -> Result<Option<CheckedNodeId>, ValidationFailure> {
        use CheckedNodePathSegment as S;
        fn name(value: &str, b: &mut Budget<'_, '_>) -> Result<S, ValidationFailure> {
            Ok(S::Name(b.text(value, "member_type")?))
        }
        fn copy(
            id: &CheckedNodeId,
            b: &mut Budget<'_, '_>,
        ) -> Result<CheckedNodeId, ValidationFailure> {
            let bytes = id
                .path
                .len()
                .saturating_mul(std::mem::size_of::<S>())
                .saturating_add(id.package.len())
                .saturating_add(
                    id.path
                        .iter()
                        .map(|part| match part {
                            S::Name(text) => text.len(),
                            _ => 0,
                        })
                        .sum::<usize>(),
                );
            b.request(bytes, "member_type")?;
            Ok(id.clone())
        }
        fn node(id: CheckedNodeId, b: &mut Budget<'_, '_>) -> Result<S, ValidationFailure> {
            b.request(
                super::budgeted_read::arc_bytes::<CheckedNodeId>(),
                "member_type",
            )?;
            Ok(S::Node(std::sync::Arc::new(id)))
        }
        fn number(value: i128, b: &mut Budget<'_, '_>) -> Result<S, ValidationFailure> {
            let mut digits = [0_u8; 40];
            let mut at = digits.len();
            let mut magnitude = value.unsigned_abs();
            loop {
                at -= 1;
                digits[at] = b'0' + (magnitude % 10) as u8;
                magnitude /= 10;
                if magnitude == 0 {
                    break;
                }
            }
            if value < 0 {
                at -= 1;
                digits[at] = b'-';
            }
            name(
                std::str::from_utf8(&digits[at..]).expect("decimal ASCII"),
                b,
            )
        }
        fn derived<const N: usize>(
            package: &str,
            tag: &str,
            form: &str,
            extra: [S; N],
            b: &mut Budget<'_, '_>,
        ) -> Result<CheckedNodeId, ValidationFailure> {
            let mut path = b.vec(N + 2, "member_type")?;
            path.push(name(tag, b)?);
            path.push(name(form, b)?);
            path.extend(extra);
            Ok(CheckedNodeId {
                package: b.text(package, "member_type")?,
                path,
                ordinal: 0,
            })
        }
        fn bounded<const N: usize>(
            package: &str,
            form: &str,
            lower: i128,
            upper: i128,
            tail: [S; N],
            b: &mut Budget<'_, '_>,
        ) -> Result<CheckedNodeId, ValidationFailure> {
            let integer = derived(package, "scalar_type", "integer", [], b)?;
            let mut path = b.vec(7 + N, "member_type")?;
            path.push(name("bounded_domain", b)?);
            path.push(name(form, b)?);
            path.push(name("min", b)?);
            path.push(node(integer, b)?);
            path.push(number(lower, b)?);
            path.push(name("max", b)?);
            path.push(number(upper, b)?);
            path.extend(tail);
            Ok(CheckedNodeId {
                package: b.text(package, "member_type")?,
                path,
                ordinal: 0,
            })
        }
        fn rational(
            package: &str,
            bounds: RationalBounds,
            b: &mut Budget<'_, '_>,
        ) -> Result<CheckedNodeId, ValidationFailure> {
            let integer = derived(package, "scalar_type", "integer", [], b)?;
            let extra = [
                name("numerator_min", b)?,
                node(integer, b)?,
                number(bounds.numerator_min, b)?,
                name("numerator_max", b)?,
                number(bounds.numerator_max, b)?,
                name("denominator_min", b)?,
                number(bounds.denominator_min, b)?,
                name("denominator_max", b)?,
                number(bounds.denominator_max, b)?,
            ];
            derived(package, "bounded_domain", "rational_range", extra, b)
        }
        let mut wrappers = Vec::new();
        let mut current = self;
        loop {
            match current {
                Self::Option(inner) => {
                    budget.push(&mut wrappers, current, "member_type")?;
                    current = inner;
                }
                Self::Collection { element, .. } => {
                    budget.push(&mut wrappers, current, "member_type")?;
                    current = element;
                }
                _ => break,
            }
        }
        let mut id = match current {
            Self::Clock => return Ok(None),
            Self::Record(id) | Self::Tuple(id) | Self::Variant(id) | Self::Quantity(id) => {
                copy(id, budget)?
            }
            Self::Reference(id) => {
                let target = copy(id, budget)?;
                let target = node(target, budget)?;
                derived(package, "composite_type", "reference", [target], budget)?
            }
            Self::IntRange(bounds) => bounded(
                package,
                "integer_range",
                bounds.lower,
                bounds.upper,
                [],
                budget,
            )?,
            Self::RationalRange(bounds) => rational(package, *bounds, budget)?,
            Self::CarriedRational(bounds, unit) => {
                let carrier = rational(package, *bounds, budget)?;
                let unit = copy(unit, budget)?;
                let text = derived(package, "scalar_type", "text", [], budget)?;
                let extra = [
                    node(unit, budget)?,
                    node(carrier, budget)?,
                    name("rounding", budget)?,
                    node(text, budget)?,
                    name("exact", budget)?,
                ];
                derived(package, "scalar_type", "carried_quantity", extra, budget)?
            }
            Self::TextBounds(bounds) => {
                let text = derived(package, "scalar_type", "text", [], budget)?;
                let tail = [
                    name("text_profile", budget)?,
                    node(text, budget)?,
                    name(&bounds.profile, budget)?,
                ];
                bounded(
                    package,
                    "text_bounds",
                    bounds.lower.into(),
                    bounds.upper.into(),
                    tail,
                    budget,
                )?
            }
            primitive => {
                let form = match primitive {
                    Self::Boolean => "boolean",
                    Self::Integer => "integer",
                    Self::Float32 => "float32",
                    Self::Float64 => "float64",
                    Self::String => "string",
                    Self::Uuid => "uuid",
                    Self::Timestamp => "timestamp",
                    Self::Bytes => "bytes",
                    Self::JsonObject => "json_object",
                    _ => return Ok(None),
                };
                derived(package, "scalar_type", form, [], budget)?
            }
        };
        while let Some(wrapper) = wrappers.pop() {
            id = match wrapper {
                Self::Option(_) => {
                    let inner = node(id, budget)?;
                    derived(package, "composite_type", "option", [inner], budget)?
                }
                Self::Collection { kind, bounds, .. } => {
                    let inner = node(id, budget)?;
                    let collection =
                        derived(package, "composite_type", kind.form(), [inner], budget)?;
                    if let Some((lower, upper)) = bounds {
                        let tail = [node(collection, budget)?];
                        bounded(
                            package,
                            "collection_bounds",
                            (*lower).into(),
                            (*upper).into(),
                            tail,
                            budget,
                        )?
                    } else {
                        collection
                    }
                }
                _ => unreachable!("only unary member wrappers are stacked"),
            };
        }
        Ok(Some(id))
    }
    /// The exact structural ID a selected member type names. A later graph
    /// join requires this ID to exist before admitting the member use.
    pub(super) fn node_id(&self, package: &str) -> Option<CheckedNodeId> {
        enum Task<'a> {
            Read(&'a MemberType),
            Option,
            Collection(CollectionKind, Option<(u64, u64)>),
        }
        let mut tasks = vec![Task::Read(self)];
        let mut ids = Vec::new();
        while let Some(task) = tasks.pop() {
            match task {
                Task::Read(Self::Clock) => return None,
                Task::Read(Self::Option(inner)) => {
                    tasks.push(Task::Option);
                    tasks.push(Task::Read(inner));
                }
                Task::Read(Self::Collection {
                    kind,
                    element,
                    bounds,
                }) => {
                    tasks.push(Task::Collection(*kind, *bounds));
                    tasks.push(Task::Read(element));
                }
                Task::Read(Self::Reference(target)) => ids.push(derived_id(
                    package,
                    "composite_type",
                    "reference",
                    vec![CheckedNodePathSegment::Node(std::sync::Arc::new(
                        target.clone(),
                    ))],
                )),
                Task::Read(
                    Self::Record(id) | Self::Tuple(id) | Self::Variant(id) | Self::Quantity(id),
                ) => {
                    ids.push(id.clone());
                }
                Task::Read(Self::IntRange(bounds)) => ids.push(bounded_id(
                    package,
                    "integer_range",
                    bounds.lower.to_string(),
                    bounds.upper.to_string(),
                    None,
                )),
                Task::Read(Self::RationalRange(bounds)) => ids.push(derived_id(
                    package,
                    "bounded_domain",
                    "rational_range",
                    vec![
                        CheckedNodePathSegment::Name("numerator_min".into()),
                        CheckedNodePathSegment::Node(std::sync::Arc::new(derived_id(
                            package,
                            "scalar_type",
                            "integer",
                            Vec::new(),
                        ))),
                        CheckedNodePathSegment::Name(bounds.numerator_min.to_string().into()),
                        CheckedNodePathSegment::Name("numerator_max".into()),
                        CheckedNodePathSegment::Name(bounds.numerator_max.to_string().into()),
                        CheckedNodePathSegment::Name("denominator_min".into()),
                        CheckedNodePathSegment::Name(bounds.denominator_min.to_string().into()),
                        CheckedNodePathSegment::Name("denominator_max".into()),
                        CheckedNodePathSegment::Name(bounds.denominator_max.to_string().into()),
                    ],
                )),
                Task::Read(Self::CarriedRational(bounds, unit)) => {
                    let carrier = Self::RationalRange(*bounds).node_id(package)?;
                    ids.push(derived_id(
                        package,
                        "scalar_type",
                        "carried_quantity",
                        vec![
                            CheckedNodePathSegment::Node(std::sync::Arc::new(unit.clone())),
                            CheckedNodePathSegment::Node(std::sync::Arc::new(carrier)),
                            CheckedNodePathSegment::Name("rounding".into()),
                            CheckedNodePathSegment::Node(std::sync::Arc::new(derived_id(
                                package,
                                "scalar_type",
                                "text",
                                Vec::new(),
                            ))),
                            CheckedNodePathSegment::Name("exact".into()),
                        ],
                    ));
                }
                Task::Read(Self::TextBounds(bounds)) => ids.push(bounded_id(
                    package,
                    "text_bounds",
                    bounds.lower.to_string(),
                    bounds.upper.to_string(),
                    Some(BoundTail::Text(&bounds.profile)),
                )),
                Task::Read(primitive) => {
                    let form = match primitive {
                        Self::Boolean => "boolean",
                        Self::Integer => "integer",
                        Self::Float32 => "float32",
                        Self::Float64 => "float64",
                        Self::String => "string",
                        Self::Uuid => "uuid",
                        Self::Timestamp => "timestamp",
                        Self::Bytes => "bytes",
                        Self::JsonObject => "json_object",
                        _ => return None,
                    };
                    ids.push(derived_id(package, "scalar_type", form, Vec::new()));
                }
                Task::Option => {
                    let inner = ids.pop()?;
                    ids.push(derived_id(
                        package,
                        "composite_type",
                        "option",
                        vec![CheckedNodePathSegment::Node(std::sync::Arc::new(inner))],
                    ));
                }
                Task::Collection(kind, bounds) => {
                    let collection = derived_id(
                        package,
                        "composite_type",
                        kind.form(),
                        vec![CheckedNodePathSegment::Node(std::sync::Arc::new(
                            ids.pop()?,
                        ))],
                    );
                    ids.push(match bounds {
                        Some((lower, upper)) => bounded_id(
                            package,
                            "collection_bounds",
                            lower.to_string(),
                            upper.to_string(),
                            Some(BoundTail::Collection(&collection)),
                        ),
                        None => collection,
                    });
                }
            }
        }
        ids.pop().filter(|_| ids.is_empty())
    }
}

fn derived_id(
    package: &str,
    tag: &str,
    form: &str,
    operands: Vec<CheckedNodePathSegment>,
) -> CheckedNodeId {
    let mut path = vec![
        CheckedNodePathSegment::Name(tag.into()),
        CheckedNodePathSegment::Name(form.into()),
    ];
    path.extend(operands);
    CheckedNodeId {
        package: package.into(),
        path,
        ordinal: 0,
    }
}

fn bounded_id(
    package: &str,
    form: &str,
    lower: String,
    upper: String,
    tail: Option<BoundTail<'_>>,
) -> CheckedNodeId {
    let integer = derived_id(package, "scalar_type", "integer", Vec::new());
    let mut operands = vec![
        CheckedNodePathSegment::Name("min".into()),
        CheckedNodePathSegment::Node(std::sync::Arc::new(integer)),
        CheckedNodePathSegment::Name(lower.into()),
        CheckedNodePathSegment::Name("max".into()),
        CheckedNodePathSegment::Name(upper.into()),
    ];
    match tail {
        Some(BoundTail::Text(profile)) => {
            operands.push(CheckedNodePathSegment::Name("text_profile".into()));
            operands.push(CheckedNodePathSegment::Node(std::sync::Arc::new(
                derived_id(package, "scalar_type", "text", Vec::new()),
            )));
            operands.push(CheckedNodePathSegment::Name(profile.into()));
        }
        Some(BoundTail::Collection(target)) => {
            operands.push(CheckedNodePathSegment::Node(std::sync::Arc::new(
                target.clone(),
            )));
        }
        None => {}
    }
    derived_id(package, "bounded_domain", form, operands)
}

enum BoundTail<'a> {
    Text(&'a str),
    Collection(&'a CheckedNodeId),
}

/// The member step 3 resolves.
#[derive(Clone, Copy, Debug)]
pub(super) enum Resolved<'m> {
    Field(&'m FieldDecl),
    Operation(&'m OperationDecl),
}

impl<'m> Resolved<'m> {
    pub(super) fn identity(self) -> &'m str {
        match self {
            Self::Field(field) => &field.identity,
            Self::Operation(operation) => &operation.identity,
        }
    }
}

/// Which member list a `member` of kind `field` or `operation` names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum MemberKind {
    Field,
    Operation,
}

/// Why a model-owned member check stopped: a refusal it determined, or the
/// work limit its charges exhausted.
#[derive(Debug)]
pub(super) enum ModelFailure {
    Refused(ModelRefusal),
    Limit(ValidationFailure),
}

impl From<ModelRefusal> for ModelFailure {
    fn from(refusal: ModelRefusal) -> Self {
        Self::Refused(refusal)
    }
}

impl From<ValidationFailure> for ModelFailure {
    fn from(failure: ValidationFailure) -> Self {
        Self::Limit(failure)
    }
}

/// Work charged to the reader's `work` limit on behalf of one
/// selected package root; an exhausted limit is the read's
/// `incomplete` result at that row.
pub(super) struct Budget<'w, 'b> {
    meter: &'w mut WorkMeter<'b>,
    /// Node position of the selected package's `model_root`.
    root: usize,
    /// The reader's byte limit: the ceiling of every encode the row's
    /// document and declarations reach.
    bytes: u64,
    /// The stage and index heap requests and work are located at.
    stage: PackageReadStage,
    index: usize,
    /// The value a refused request reports as the one whose request failed.
    place: Place,
}

/// Where a refused request of a [`Budget`] points.
#[derive(Clone, Copy)]
enum Place {
    /// The selected package's `model_root` owner.
    Owner,
    /// The entry of the lock array `member` at the budget's index.
    Lock(&'static str),
    /// The body of the graph node at the budget's index.
    Node,
    /// The owner of the graph node at the budget's index.
    NodeOwner,
}

impl Place {
    fn pointer(self, root: usize, index: usize) -> JsonPointer {
        match self {
            Self::Owner => crate::checked_package::common::node_pointer(root).key("owner"),
            Self::Lock(member) => JsonPointer::root().key("lock").key(member).index(index),
            Self::Node => crate::checked_package::common::node_pointer(index).key("body"),
            Self::NodeOwner => crate::checked_package::common::node_pointer(index).key("owner"),
        }
    }
}

#[derive(Debug)]
pub(super) struct RuntimeTypeBudgetDenied;

fn runtime_charge(
    budget: &mut Option<&mut NativeJoinBudget>,
    kind: NativeJoinChargeKind,
    amount: u128,
) -> Result<(), RuntimeTypeBudgetDenied> {
    if budget
        .as_deref_mut()
        .is_some_and(|budget| !budget.charge(kind, amount))
    {
        return Err(RuntimeTypeBudgetDenied);
    }
    Ok(())
}

fn runtime_work(
    budget: &mut Option<&mut NativeJoinBudget>,
    amount: u128,
) -> Result<(), RuntimeTypeBudgetDenied> {
    runtime_charge(budget, NativeJoinChargeKind::Work, amount)
}

fn runtime_storage(
    budget: &mut Option<&mut NativeJoinBudget>,
    amount: u128,
) -> Result<(), RuntimeTypeBudgetDenied> {
    runtime_charge(budget, NativeJoinChargeKind::Storage, amount)
}

fn runtime_text_storage(value: &str) -> u128 {
    super::native_join_storage::text_parts(&[value])
}

fn runtime_declaration_storage(identity: &str, form: DeclarationForm, node: &str) -> u128 {
    runtime_text_storage(identity)
        .saturating_add(runtime_text_storage(form.tag()))
        .saturating_add(runtime_text_storage(form.form()))
        .saturating_add(runtime_text_storage(node))
        .saturating_add(1)
}

impl<'w, 'b> Budget<'w, 'b> {
    /// A budget for the selected package whose `model_root` is the graph node
    /// `root`: the graph stage at that node, until [`Budget::at`] says otherwise.
    pub(super) fn new(meter: &'w mut WorkMeter<'b>, root: usize, bytes: u64) -> Self {
        Self {
            meter,
            root,
            bytes,
            stage: PackageReadStage::Graph,
            index: root,
            place: Place::Owner,
        }
    }

    /// A budget for the lock stage, whose refusals point at the entry of the
    /// lock array `member` that [`Budget::at`] names.
    pub(super) fn lock(meter: &'w mut WorkMeter<'b>, member: &'static str) -> Self {
        Self {
            meter,
            root: 0,
            bytes: 0,
            stage: PackageReadStage::Lock,
            index: 0,
            place: Place::Lock(member),
        }
    }

    /// A budget for the lock stage's tables over the graph: charges are
    /// located at `PackageRead(Lock, node)` and refusals point at the body
    /// (`owner` for `owner == true`) of the graph node at `node`.
    pub(super) fn lock_node(meter: &'w mut WorkMeter<'b>, node: usize, owner: bool) -> Self {
        Self {
            meter,
            root: 0,
            bytes: 0,
            stage: PackageReadStage::Lock,
            index: node,
            place: if owner { Place::NodeOwner } else { Place::Node },
        }
    }

    /// A budget for the graph node at `node`, whose refusals point at the
    /// node's body.
    pub(super) fn node(meter: &'w mut WorkMeter<'b>, node: usize) -> Self {
        Self {
            meter,
            root: 0,
            bytes: 0,
            stage: PackageReadStage::Graph,
            index: node,
            place: Place::Node,
        }
    }

    /// Locates this budget's charges at `stage` and `index`.
    pub(super) fn at(mut self, stage: PackageReadStage, index: usize) -> Self {
        self.stage = stage;
        self.index = index;
        self
    }

    pub(super) fn work(&self) -> u64 {
        self.meter.consumed()
    }

    /// Locates this budget's following charges at `index`.
    pub(super) fn set_index(&mut self, index: usize) {
        self.index = index;
    }

    /// The pointer of the root owner this budget is charged at.
    pub(super) fn row(&self) -> JsonPointer {
        self.place.pointer(self.root, self.index)
    }

    fn locus(&self, form: &'static str) -> NativeJoinLocus {
        NativeJoinLocus {
            source: NativeJoinLocusSource::PackageRead(self.stage, self.index),
            form,
        }
    }

    /// Charges `work` at the `"declarations"` form.
    pub(super) fn charge(&mut self, work: u64) -> Result<(), ValidationFailure> {
        self.charge_form(work, "declarations")
    }

    /// Charges `work` at this budget's locus under `form`.
    pub(super) fn charge_form(
        &mut self,
        work: u64,
        form: &'static str,
    ) -> Result<(), ValidationFailure> {
        let locus = self.locus(form);
        if let Some(budget) = self.meter.budget_mut() {
            budget.set_active_locus(Some(locus));
        }
        let (place, root, index) = (self.place, self.root, self.index);
        self.meter.charge(work, || place.pointer(root, index))
    }

    /// Charges `work` at this budget's locus under `form`, a failed charge
    /// reporting `at`.
    pub(super) fn charge_at(
        &mut self,
        work: u64,
        form: &'static str,
        at: impl FnOnce() -> JsonPointer,
    ) -> Result<(), ValidationFailure> {
        let locus = self.locus(form);
        if let Some(budget) = self.meter.budget_mut() {
            budget.set_active_locus(Some(locus));
        }
        self.meter.charge(work, at)
    }

    /// Admits a request for `bytes` of heap at this budget's locus under
    /// `form`, before the request is made. A budget with a work limit of its
    /// own (lowering, unit tests) holds no read storage and admits it.
    pub(super) fn request(
        &mut self,
        bytes: usize,
        form: &'static str,
    ) -> Result<(), ValidationFailure> {
        if bytes == 0 {
            return Ok(());
        }
        let locus = self.locus(form);
        let (place, root, index) = (self.place, self.root, self.index);
        let Some(budget) = self.meter.budget_mut() else {
            return Ok(());
        };
        budget.set_active_locus(Some(locus));
        if budget.charge_read_storage(bytes as u128) {
            Ok(())
        } else {
            Err(read_storage_denial(
                budget,
                Some(place.pointer(root, index)),
            ))
        }
    }

    /// The caller's budget with its locus set to the reader's reservation of
    /// this stage, when there is one.
    pub(super) fn reader_budget(&mut self, form: &'static str) -> Option<&mut NativeJoinBudget> {
        let locus = self.locus(form);
        let budget = self.meter.budget_mut()?;
        budget.set_active_locus(Some(locus));
        Some(budget)
    }

    /// The stop of a denial the shared reader's reservation just met: no
    /// pointer, as the request belongs to the reader's own heap.
    pub(super) fn reader_denial(&mut self) -> ValidationFailure {
        match self.meter.budget_mut() {
            Some(budget) => read_storage_denial(budget, None),
            None => ValidationFailure::Budget(BudgetStop {
                resource: PackageReadResource::ReadStorage,
                amount: 0,
                path: None,
            }),
        }
    }

    /// An empty list with room for `capacity` items, requested first.
    pub(super) fn vec<T>(
        &mut self,
        capacity: usize,
        form: &'static str,
    ) -> Result<Vec<T>, ValidationFailure> {
        self.request(capacity.saturating_mul(std::mem::size_of::<T>()), form)?;
        Ok(Vec::with_capacity(capacity))
    }

    /// Appends `item`, growing the list to twice its capacity (at least four
    /// items) when it is full, with the whole new capacity requested first.
    pub(super) fn push<T>(
        &mut self,
        list: &mut Vec<T>,
        item: T,
        form: &'static str,
    ) -> Result<(), ValidationFailure> {
        if list.len() == list.capacity() {
            let target = list.capacity().saturating_mul(2).max(4);
            self.request(target.saturating_mul(std::mem::size_of::<T>()), form)?;
            list.reserve_exact(target - list.len());
        }
        list.push(item);
        Ok(())
    }

    pub(super) fn room<T>(
        &mut self,
        items: &mut Vec<T>,
        additional: usize,
        form: &'static str,
    ) -> Result<(), ValidationFailure> {
        let needed = items.len().saturating_add(additional);
        if needed > items.capacity() {
            let target = needed.max(items.capacity().saturating_mul(2)).max(4);
            self.request(target.saturating_mul(std::mem::size_of::<T>()), form)?;
            items.reserve_exact(target - items.len());
        }
        Ok(())
    }
    pub(super) fn boxed<T>(
        &mut self,
        value: T,
        form: &'static str,
    ) -> Result<Box<T>, ValidationFailure> {
        self.request(std::mem::size_of::<T>(), form)?;
        Ok(Box::new(value))
    }
    pub(super) fn concat(
        &mut self,
        parts: &[&str],
        form: &'static str,
    ) -> Result<String, ValidationFailure> {
        let length = parts.iter().map(|part| part.len()).sum();
        self.request(length, form)?;
        let mut text = String::with_capacity(length);
        for part in parts {
            text.push_str(part);
        }
        Ok(text)
    }
    pub(super) fn clone_id(
        &mut self,
        id: &CheckedNodeId,
        form: &'static str,
    ) -> Result<CheckedNodeId, ValidationFailure> {
        let bytes = id
            .package
            .len()
            .saturating_add(
                id.path
                    .len()
                    .saturating_mul(std::mem::size_of::<CheckedNodePathSegment>()),
            )
            .saturating_add(
                id.path
                    .iter()
                    .map(|part| match part {
                        CheckedNodePathSegment::Name(text) => text.len(),
                        _ => 0,
                    })
                    .sum::<usize>(),
            );
        self.request(bytes, form)?;
        Ok(id.clone())
    }

    /// An owned copy of `text`, requested first.
    pub(super) fn text(
        &mut self,
        text: &str,
        form: &'static str,
    ) -> Result<Box<str>, ValidationFailure> {
        self.request(text.len(), form)?;
        Ok(Box::from(text))
    }
}

/// The stop a denied read-storage charge ends the read with.
fn read_storage_denial(budget: &NativeJoinBudget, path: Option<JsonPointer>) -> ValidationFailure {
    let (resource, amount) = budget
        .first_unpaid_resource()
        .unwrap_or((PackageReadResource::ReadStorage, 0));
    ValidationFailure::Budget(BudgetStop {
        resource,
        amount,
        path,
    })
}

pub(super) fn units(len: usize) -> u64 {
    u64::try_from(len).unwrap_or(u64::MAX)
}

/// Test probes: how many ancestor walks ran on this thread.
#[cfg(test)]
pub(super) mod probe {
    use std::cell::Cell;

    thread_local! {
        static WALKS: Cell<u64> = const { Cell::new(0) };
    }

    pub(super) fn walked() {
        WALKS.with(|walks| walks.set(walks.get().saturating_add(1)));
    }

    /// The ancestor walks run on this thread so far.
    pub(in crate::checked_package::v2) fn ancestor_walks() -> u64 {
        WALKS.with(Cell::get)
    }
}

impl DomainModel {
    /// One admitted alias edge, borrowed from the selected document's retained index.
    pub(super) fn population_alias_target(&self, name: &str) -> Option<&str> {
        self.aliases.get(name).map(AsRef::as_ref)
    }

    /// Number of retained alias edges, bounding a population-member walk.
    pub(super) fn population_alias_count(&self) -> usize {
        self.aliases.len()
    }

    /// Whether this identity names any retained selected declaration, even
    /// when that declaration is not an object type.
    pub(super) fn population_member_declaration_exists(&self, name: &str) -> bool {
        self.object_types.contains_key(name)
            || self.state_machines.contains(name)
            || self.processes.contains(name)
            || self.persistence_interfaces.contains(name)
            || self.persistence_operations.contains_key(name)
            || self.record_types.contains_key(name)
            || self.event_types.contains(name)
            || self.tuple_types.contains_key(name)
            || self.value_types.contains_key(name)
            || self.variant_types.contains_key(name)
            || self.collection_types.contains_key(name)
            || self.dimensions.contains_key(name)
            || self.units.contains_key(name)
            || self.relationships.contains_key(name)
            || self.populations.contains(name)
            || self.relation_nodes.contains(name)
            || self.other_types.contains_key(name)
    }

    /// Whether the admitted selected declaration has the systems Port meaning.
    pub(super) fn is_systems_port(&self, node: &str) -> bool {
        self.other_types.get(node) == Some(&meaning::Kind::SystemsPort)
    }

    /// The name an alias chain starting at `name` ends at; `None` when the
    /// chain loops. Allocates nothing: a chain without a loop is at most as
    /// long as the alias table.
    fn resolve_alias<'a>(&'a self, name: &'a str) -> Option<&'a str> {
        let mut current = name;
        let mut followed = 0_usize;
        while let Some(target) = self.aliases.get(current) {
            followed += 1;
            if followed > self.aliases.len() {
                return None;
            }
            current = target;
        }
        Some(current)
    }

    /// `name` resolved through the aliases and admitted as a type reference,
    /// its owned text requested first.
    fn selected_type_ref(
        &self,
        name: &str,
        budget: &mut Budget<'_, '_>,
    ) -> Result<Option<SelectedTypeRef>, ValidationFailure> {
        let Some(resolved) = self.resolve_alias(name) else {
            return Ok(None);
        };
        SelectedTypeRef::charged(resolved, budget)
    }
    /// The selected object or interface form of a relationship end.
    pub(super) fn end_declaration_form(&self, type_ref: &str) -> Option<DeclarationForm> {
        if self.state_machines.contains(type_ref) || self.processes.contains(type_ref) {
            return None;
        }
        let declared = self.object_types.get(type_ref)?;
        Some(if declared.interface {
            DeclarationForm::SystemsInterface
        } else {
            DeclarationForm::ObjectType
        })
    }

    /// A field-bearing declaration, whether it is an object or a value record.
    pub(super) fn field_owner(&self, node: &str) -> Option<&FieldBearingTypeDecl> {
        self.object_types
            .get(node)
            .or_else(|| self.record_types.get(node))
    }
    /// The proper ancestors of `node` along its declared supertypes; one
    /// work unit per supertype edge followed.
    fn ancestors(
        &self,
        node: &str,
        budget: &mut Budget<'_, '_>,
    ) -> Result<SortedSet<&str>, ValidationFailure> {
        #[cfg(test)]
        probe::walked();
        let mut reached = SortedSet::new();
        let mut pending: Vec<&str> = match self.field_owner(node) {
            Some(declared) => {
                let mut pending = budget.vec(declared.supertypes.len(), MODEL)?;
                pending.extend(declared.supertypes.iter().map(AsRef::as_ref));
                pending
            }
            None => Vec::new(),
        };
        while let Some(supertype) = pending.pop() {
            budget.charge(1)?;
            if reached.insert(budget, supertype, MODEL)? {
                if let Some(declared) = self.field_owner(supertype) {
                    for next in &declared.supertypes {
                        budget.push(&mut pending, next.as_ref(), MODEL)?;
                    }
                }
            }
        }
        Ok(reached)
    }

    /// Whether `a` and `b` are one type or a chain of declared supertypes
    /// leads from one to the other (FR-151's conformance relation).
    pub(super) fn conforms(
        &self,
        a: &str,
        b: &str,
        budget: &mut Budget<'_, '_>,
    ) -> Result<bool, ValidationFailure> {
        Ok(a == b
            || self.ancestors(a, budget)?.contains(b)
            || self.ancestors(b, budget)?.contains(a))
    }

    /// Whether `sub` is `sup` or a chain of declared supertypes leads from
    /// `sub` to `sup` (FR-151's conformance relation, one way).
    pub(super) fn conforms_to(
        &self,
        sub: &str,
        sup: &str,
        budget: &mut Budget<'_, '_>,
    ) -> Result<bool, ValidationFailure> {
        Ok(sub == sup || self.ancestors(sub, budget)?.contains(sup))
    }

    /// Whether the context's unique effective operation reaches the exact
    /// declared family anchor through retained, ancestry-valid operation links.
    pub(super) fn operation_in_family(
        &self,
        context: &str,
        declaring: &str,
        name: &str,
        budget: &mut Budget<'_, '_>,
    ) -> Result<bool, ValidationFailure> {
        if !self.conforms_to(context, declaring, budget)? {
            return Ok(false);
        }
        let Some(declared) = self.object_types.get(declaring) else {
            return Ok(false);
        };
        let mut anchor = None;
        for operation in &declared.operations {
            budget.charge(1)?;
            if member_name(&operation.identity) == name && anchor.replace(operation).is_some() {
                return Ok(false);
            }
        }
        let Some(anchor) = anchor else {
            return Ok(false);
        };
        let effective = match self.resolve(context, MemberKind::Operation, name, budget) {
            Ok(Resolved::Operation(operation)) => operation,
            Ok(Resolved::Field(_)) | Err(ModelFailure::Refused(_)) => return Ok(false),
            Err(ModelFailure::Limit(failure)) => return Err(failure),
        };
        // Every operation by identity; a repeated identity names its last.
        let total: usize = self
            .object_types
            .values()
            .map(|declared| declared.operations.len())
            .sum();
        let mut operations: Vec<(&str, usize, &str, &OperationDecl)> = budget.vec(total, MODEL)?;
        for (owner, declared) in &self.object_types {
            budget.charge(1)?;
            for operation in &declared.operations {
                budget.charge(1)?;
                operations.push((
                    operation.identity.as_ref(),
                    operations.len(),
                    owner.as_ref(),
                    operation,
                ));
            }
        }
        operations.sort_unstable_by(|a, b| a.0.cmp(b.0).then(a.1.cmp(&b.1)));
        let find = |identity: &str| {
            let after = operations.partition_point(|row| row.0 <= identity);
            let row = operations.get(after.checked_sub(1)?)?;
            (row.0 == identity).then_some((row.2, row.3))
        };
        let mut current = effective.identity.as_ref();
        let mut seen: SortedSet<&str> = SortedSet::new();
        let mut reaches_anchor = false;
        loop {
            budget.charge(1)?;
            if !seen.insert(budget, current, MODEL)? {
                return Ok(false);
            }
            let Some((owner, operation)) = find(current) else {
                return Ok(false);
            };
            reaches_anchor |= operation.identity == anchor.identity;
            let Some(target) = operation.redefines.as_deref() else {
                return Ok(reaches_anchor);
            };
            budget.charge(1)?;
            let Some((target_owner, _)) = find(target) else {
                return Ok(false);
            };
            if owner == target_owner || !self.conforms_to(owner, target_owner, budget)? {
                return Ok(false);
            }
            current = target;
        }
    }

    /// FR-322 step 3: the member of `kind` named `name` among the exposed
    /// effective members of the object or record type `node`. A field is selected from
    /// the field table built at admission, one work unit per exposed or hidden
    /// entry of that table; an operation walks the ancestors, and every
    /// ancestor edge, member and redefinition pair visited is one work unit.
    pub(super) fn resolve(
        &self,
        node: &str,
        kind: MemberKind,
        name: &str,
        budget: &mut Budget<'_, '_>,
    ) -> Result<Resolved<'_>, ModelFailure> {
        let Some((owner_node, declared)) = self
            .object_types
            .get_key_value(node)
            .or_else(|| self.record_types.get_key_value(node))
        else {
            return Err(ModelRefusal::ineligible().into());
        };
        if kind == MemberKind::Operation && self.record_types.contains_key(node) {
            return Err(ModelRefusal::ineligible().into());
        }
        if kind == MemberKind::Field {
            let Some(table) = self.fields.of(node) else {
                return Err(ModelRefusal::ineligible().into());
            };
            budget.charge(units(table.entries()))?;
            return match table.select(name) {
                Selected::One(field) => Ok(Resolved::Field(field)),
                Selected::Ambiguous => Err(ModelRefusal::ambiguous().into()),
                Selected::Absent => Err(ModelRefusal::ineligible().into()),
            };
        }
        let ancestors = self.ancestors(node, budget)?;
        let owners = std::iter::once((owner_node.as_ref(), declared)).chain(
            ancestors
                .iter()
                .filter_map(|ancestor| Some((*ancestor, self.field_owner(ancestor)?))),
        );
        let mut members: SortedMap<&str, Resolved<'_>> = SortedMap::new();
        let mut hidden: SortedSet<&str> = SortedSet::new();
        // (redefined target, order met, redefining member, redefining owner)
        let mut redefiners: Vec<(&str, usize, &str, &str)> = Vec::new();
        for (owner, declared) in owners {
            budget.charge(units(
                declared
                    .fields
                    .len()
                    .saturating_add(declared.operations.len()),
            ))?;
            let fields = declared.fields.iter().map(|field| {
                (
                    Resolved::Field(field),
                    field.redefines.as_deref(),
                    MemberKind::Field,
                )
            });
            let operations = declared.operations.iter().map(|operation| {
                (
                    Resolved::Operation(operation),
                    operation.redefines.as_deref(),
                    MemberKind::Operation,
                )
            });
            for (member, redefines, member_kind) in fields.chain(operations) {
                let identity = member.identity();
                if let Some(target) = redefines {
                    hidden.insert(budget, target, MODEL)?;
                    let met = redefiners.len();
                    budget.push(&mut redefiners, (target, met, identity, owner), MODEL)?;
                }
                if member_kind == kind {
                    members.insert(budget, identity, member, MODEL)?;
                }
            }
        }
        // The most-derived-redefiner rule: a redefiner of a target that a
        // redefiner in a more derived owner also redefines is hidden.
        redefiners.sort_unstable();
        let mut ancestry: SortedMap<&str, SortedSet<&str>> = SortedMap::new();
        let mut start = 0;
        while let Some(&(target, ..)) = redefiners.get(start) {
            let end = start
                + redefiners
                    .get(start..)
                    .unwrap_or_default()
                    .iter()
                    .take_while(|row| row.0 == target)
                    .count();
            let group = redefiners.get(start..end).unwrap_or_default();
            for (_, _, identity, owner) in group {
                for (_, _, _, other_owner) in group {
                    budget.charge(1)?;
                    if !ancestry.contains_key(other_owner) {
                        let of_other = self.ancestors(other_owner, budget)?;
                        ancestry.insert(budget, other_owner, of_other, MODEL)?;
                    }
                    if ancestry
                        .get(other_owner)
                        .is_some_and(|of_other| of_other.contains(owner))
                    {
                        hidden.insert(budget, identity, MODEL)?;
                    }
                }
            }
            start = end;
        }
        let mut matches = members
            .iter()
            .filter(|(identity, _)| !hidden.contains(**identity) && member_name(identity) == name)
            .map(|(_, member)| *member);
        match (matches.next(), matches.next()) {
            (Some(member), None) => Ok(member),
            (Some(_), Some(_)) => Err(ModelRefusal::ambiguous().into()),
            (None, _) => Err(ModelRefusal::ineligible().into()),
        }
    }

    /// Resolve a member type using byte reservations from the active reader.
    pub(super) fn element_type_in(
        &self,
        type_ref: &SelectedTypeRef,
        budget: &mut Budget<'_, '_>,
    ) -> Result<Option<MemberType>, ValidationFailure> {
        let mut current = type_ref;
        let mut seen: Vec<&str> = Vec::new();
        let mut layers = Vec::new();
        while let SelectedTypeRef::Declared(name) = current {
            budget.charge(1)?;
            let Some(definition) = self.collection_types.get(name) else {
                break;
            };
            match seen.binary_search(&name.as_ref()) {
                Ok(_) => return Ok(None),
                Err(at) => {
                    budget.room(&mut seen, 1, "member_type")?;
                    seen.insert(at, name);
                }
            }
            match definition {
                SelectedTypeDefinition::Sequence(inner) => {
                    budget.push(&mut layers, SelectedValueLayer::Sequence, "member_type")?;
                    current = inner;
                }
                SelectedTypeDefinition::Option(inner) => {
                    budget.push(&mut layers, SelectedValueLayer::Option, "member_type")?;
                    current = inner;
                }
                SelectedTypeDefinition::Reference(inner) => current = inner,
                SelectedTypeDefinition::Map(_) => return Ok(None),
            }
        }
        let Some(mut element) = self.element_type_leaf_in(current, budget)? else {
            return Ok(None);
        };
        while let Some(layer) = layers.pop() {
            element = match layer {
                SelectedValueLayer::Option => {
                    MemberType::Option(budget.boxed(element, "member_type")?)
                }
                SelectedValueLayer::Sequence => MemberType::Collection {
                    kind: CollectionKind::Sequence,
                    element: budget.boxed(element, "member_type")?,
                    bounds: None,
                },
                SelectedValueLayer::Map => return Ok(None),
            };
        }
        Ok(Some(element))
    }

    fn element_type_leaf_in(
        &self,
        type_ref: &SelectedTypeRef,
        budget: &mut Budget<'_, '_>,
    ) -> Result<Option<MemberType>, ValidationFailure> {
        budget.charge(1)?;
        let declared = match type_ref {
            SelectedTypeRef::Map(_, _) => return Ok(None),
            SelectedTypeRef::Unit(name) => {
                return self
                    .unit_refs
                    .get(name)
                    .map(|id| budget.clone_id(id, "member_type").map(MemberType::Quantity))
                    .transpose()
            }
            SelectedTypeRef::Native(native) => {
                return Ok(match native {
                    NativeTypeRef::Boolean => Some(MemberType::Boolean),
                    NativeTypeRef::Integer => Some(MemberType::Integer),
                    NativeTypeRef::Float32 => Some(MemberType::Float32),
                    NativeTypeRef::Float64 => Some(MemberType::Float64),
                    NativeTypeRef::String => Some(MemberType::String),
                    NativeTypeRef::Uuid => Some(MemberType::Uuid),
                    NativeTypeRef::Timestamp => Some(MemberType::Timestamp),
                    NativeTypeRef::Bytes => Some(MemberType::Bytes),
                    NativeTypeRef::JsonObject => Some(MemberType::JsonObject),
                    NativeTypeRef::Clock => Some(MemberType::Clock),
                    NativeTypeRef::Duration => {
                        let parts = if self.own_seconds_declared {
                            [
                                DURATION_RESOLVED_PREFIX,
                                "ix://quire/unit/",
                                self.identity.as_ref(),
                                "/s",
                            ]
                        } else {
                            [DURATION_RESOLVED_PREFIX, "ix://quire/unit/", "", "s"]
                        };
                        let key = budget.concat(&parts, "member_type")?;
                        self.unit_refs
                            .get(key.as_str())
                            .map(|id| budget.clone_id(id, "member_type").map(MemberType::Quantity))
                            .transpose()?
                    }
                    NativeTypeRef::Rational | NativeTypeRef::Decimal | NativeTypeRef::Text => None,
                })
            }
            SelectedTypeRef::Declared(name) => name.as_ref(),
        };
        if let Some(bounds) = self.value_types.get(declared) {
            return Ok(match bounds {
                None => None,
                Some(ValueBounds::Integer(bounds)) => Some(MemberType::IntRange(*bounds)),
                Some(ValueBounds::Rational(bounds, None)) => {
                    Some(MemberType::RationalRange(*bounds))
                }
                Some(ValueBounds::Rational(bounds, Some(unit))) => Some(
                    MemberType::CarriedRational(*bounds, budget.clone_id(unit, "member_type")?),
                ),
                Some(ValueBounds::Text(bounds)) => Some(MemberType::TextBounds(TextBounds {
                    lower: bounds.lower,
                    upper: bounds.upper,
                    profile: budget.text(&bounds.profile, "member_type")?,
                })),
            });
        }
        let (form, kind) = if self.record_types.contains_key(declared) {
            (DeclarationForm::RecordValueType, 0)
        } else if self.tuple_types.contains_key(declared) {
            (DeclarationForm::TupleValueType, 1)
        } else if self.variant_types.contains_key(declared) {
            (DeclarationForm::VariantType, 2)
        } else {
            let Some(object) = self.object_types.get(declared) else {
                return Ok(None);
            };
            if object.interface {
                return Ok(None);
            }
            (
                if self.state_machines.contains(declared) {
                    DeclarationForm::StateMachine
                } else if self.processes.contains(declared) {
                    DeclarationForm::Process
                } else {
                    DeclarationForm::ObjectType
                },
                3,
            )
        };
        let id = declaration_id_in(&self.identity, form, declared, budget)?;
        Ok(Some(match kind {
            0 => MemberType::Record(id),
            1 => MemberType::Tuple(id),
            2 => MemberType::Variant(id),
            _ => MemberType::Reference(id),
        }))
    }

    pub(super) fn slot_type_in(
        &self,
        slot: &TypedSlot,
        budget: &mut Budget<'_, '_>,
    ) -> Result<Option<MemberType>, ValidationFailure> {
        let Some(type_ref) = slot.type_ref.as_ref() else {
            return Ok(None);
        };
        let Some(element) = self.element_type_in(type_ref, budget)? else {
            return Ok(None);
        };
        let multiplicity = slot.multiplicity;
        if (multiplicity.lower, multiplicity.upper) == (1, Some(1)) {
            return Ok(Some(element));
        }
        let bounds = match (multiplicity.lower, multiplicity.upper) {
            (0, None) => None,
            (_, None) => return Ok(None),
            (lower, Some(upper)) => Some((lower, upper)),
        };
        Ok(Some(MemberType::Collection {
            kind: CollectionKind::of(multiplicity.ordered, multiplicity.unique),
            element: budget.boxed(element, "member_type")?,
            bounds,
        }))
    }

    pub(super) fn field_type_in(
        &self,
        field: &FieldDecl,
        budget: &mut Budget<'_, '_>,
    ) -> Result<Option<MemberType>, ValidationFailure> {
        let Some(slot) = self.slot_type_in(&field.slot, budget)? else {
            return Ok(None);
        };
        Ok(Some(
            match (field.optional, field.slot.type_ref.as_ref(), slot) {
                (
                    true,
                    Some(SelectedTypeRef::Native(NativeTypeRef::Clock)),
                    MemberType::Collection {
                        kind,
                        element,
                        bounds,
                    },
                ) => MemberType::Collection {
                    kind,
                    element: budget.boxed(MemberType::Option(element), "member_type")?,
                    bounds,
                },
                (true, _, slot) => MemberType::Option(budget.boxed(slot, "member_type")?),
                (false, _, slot) => slot,
            },
        ))
    }
    /// FR-322 step 4's element type of a declared `typeRef`.
    pub(super) fn element_type(&self, type_ref: &SelectedTypeRef) -> Option<MemberType> {
        self.element_type_runtime_inner(type_ref, None)
            .ok()
            .flatten()
    }

    pub(super) fn element_type_runtime(
        &self,
        type_ref: &SelectedTypeRef,
        budget: &mut NativeJoinBudget,
    ) -> Result<Option<MemberType>, RuntimeTypeBudgetDenied> {
        self.element_type_runtime_inner(type_ref, Some(budget))
    }

    fn element_type_runtime_inner(
        &self,
        type_ref: &SelectedTypeRef,
        mut budget: Option<&mut NativeJoinBudget>,
    ) -> Result<Option<MemberType>, RuntimeTypeBudgetDenied> {
        runtime_storage(&mut budget, 2)?;
        let mut current = type_ref;
        let mut seen: Vec<&str> = Vec::new();
        let mut layers = Vec::new();
        while let SelectedTypeRef::Declared(name) = current {
            runtime_work(&mut budget, 1)?;
            let Some(definition) = self.collection_types.get(name) else {
                break;
            };
            runtime_storage(&mut budget, runtime_text_storage(name).saturating_add(1))?;
            if !first_visit(&mut seen, name) {
                return Ok(None);
            }
            match definition {
                SelectedTypeDefinition::Sequence(items) => {
                    runtime_storage(&mut budget, 1)?;
                    layers.push(SelectedValueLayer::Sequence);
                    current = items;
                }
                SelectedTypeDefinition::Option(inner) => {
                    runtime_storage(&mut budget, 1)?;
                    layers.push(SelectedValueLayer::Option);
                    current = inner;
                }
                SelectedTypeDefinition::Reference(target) => {
                    current = target;
                }
                SelectedTypeDefinition::Map(_) => return Ok(None),
            }
        }
        let Some(mut element) = self.element_type_leaf_runtime(current, &mut budget)? else {
            return Ok(None);
        };
        for layer in layers.into_iter().rev() {
            element = match layer {
                SelectedValueLayer::Option => {
                    runtime_storage(&mut budget, 2)?;
                    MemberType::Option(Box::new(element))
                }
                SelectedValueLayer::Sequence => {
                    // Charge the wrapper and its owned child before construction.
                    runtime_storage(&mut budget, 2)?;
                    MemberType::Collection {
                        kind: CollectionKind::Sequence,
                        element: Box::new(element),
                        bounds: None,
                    }
                }
                SelectedValueLayer::Map => return Ok(None),
            };
        }
        Ok(Some(element))
    }

    /// A Duration member has no type when its chosen owner declares no `s`
    /// node. Distinguish that missing name from a present but invalid `s`.
    pub(super) fn duration_unit_absent(
        &self,
        type_ref: &SelectedTypeRef,
        budget: &mut Budget<'_, '_>,
    ) -> Result<bool, ValidationFailure> {
        let mut current = type_ref;
        let mut seen: Vec<&str> = Vec::new();
        while let SelectedTypeRef::Declared(name) = current {
            if seen.contains(&name.as_ref()) {
                return Ok(false);
            }
            budget.push(&mut seen, name.as_ref(), "member_type")?;
            match self.collection_types.get(name) {
                Some(
                    SelectedTypeDefinition::Sequence(items) | SelectedTypeDefinition::Option(items),
                ) => current = items,
                _ => break,
            }
        }
        if !matches!(current, SelectedTypeRef::Native(NativeTypeRef::Duration)) {
            return Ok(false);
        }
        let unit_name = if self.own_seconds_declared {
            budget.concat(&["ix://quire/unit/", &self.identity, "/s"], "member_type")?
        } else {
            budget.concat(&["ix://quire/unit/s"], "member_type")?
        };
        Ok(!self.unit_refs.contains_key(unit_name.as_str()))
    }

    fn duration_unit_node(
        &self,
        budget: &mut Option<&mut NativeJoinBudget>,
    ) -> Result<Option<&CheckedNodeId>, RuntimeTypeBudgetDenied> {
        const UNIT_PREFIX: &str = "ix://quire/unit/";
        let parts = if self.own_seconds_declared {
            [
                DURATION_RESOLVED_PREFIX,
                UNIT_PREFIX,
                self.identity.as_ref(),
                "/s",
            ]
        } else {
            [DURATION_RESOLVED_PREFIX, UNIT_PREFIX, "", "s"]
        };
        // Compute the single buffer's footprint from borrowed parts. Both its
        // inspection and ownership are payable before construction or lookup.
        let key_storage = super::native_join_storage::text_parts(&parts);
        runtime_work(budget, key_storage)?;
        runtime_storage(budget, key_storage)?;
        let key = parts.concat();
        Ok(self.unit_refs.get(key.as_str()))
    }

    fn element_type_leaf_runtime(
        &self,
        type_ref: &SelectedTypeRef,
        budget: &mut Option<&mut NativeJoinBudget>,
    ) -> Result<Option<MemberType>, RuntimeTypeBudgetDenied> {
        runtime_work(budget, 1)?;
        let type_ref = match type_ref {
            SelectedTypeRef::Map(_, _) => return Ok(None),
            SelectedTypeRef::Unit(name) => {
                let Some(node) = self.unit_refs.get(name) else {
                    return Ok(None);
                };
                runtime_storage(
                    budget,
                    super::native_join_storage::node(node).saturating_add(1),
                )?;
                return Ok(Some(MemberType::Quantity(node.clone())));
            }
            SelectedTypeRef::Native(native) => {
                let value = match native {
                    NativeTypeRef::Boolean => Some(MemberType::Boolean),
                    NativeTypeRef::Integer => Some(MemberType::Integer),
                    NativeTypeRef::Float32 => Some(MemberType::Float32),
                    NativeTypeRef::Float64 => Some(MemberType::Float64),
                    NativeTypeRef::String => Some(MemberType::String),
                    NativeTypeRef::Uuid => Some(MemberType::Uuid),
                    NativeTypeRef::Timestamp => Some(MemberType::Timestamp),
                    NativeTypeRef::Bytes => Some(MemberType::Bytes),
                    NativeTypeRef::JsonObject => Some(MemberType::JsonObject),
                    NativeTypeRef::Clock => Some(MemberType::Clock),
                    NativeTypeRef::Duration => {
                        let Some(node) = self.duration_unit_node(budget)? else {
                            return Ok(None);
                        };
                        runtime_storage(
                            budget,
                            super::native_join_storage::node(node).saturating_add(1),
                        )?;
                        Some(MemberType::Quantity(node.clone()))
                    }
                    NativeTypeRef::Rational | NativeTypeRef::Decimal | NativeTypeRef::Text => None,
                };
                if value.is_some() && !matches!(native, NativeTypeRef::Duration) {
                    runtime_storage(budget, 1)?;
                }
                return Ok(value);
            }
            SelectedTypeRef::Declared(name) => name.as_ref(),
        };
        if let Some(bounds) = self.value_types.get(type_ref) {
            let Some(bounds) = bounds.as_ref() else {
                return Ok(None);
            };
            return Ok(Some(match bounds {
                ValueBounds::Integer(range) => {
                    runtime_storage(budget, 1)?;
                    MemberType::IntRange(*range)
                }
                ValueBounds::Rational(bounds, None) => {
                    runtime_storage(budget, 1)?;
                    MemberType::RationalRange(*bounds)
                }
                ValueBounds::Rational(bounds, Some(unit)) => {
                    runtime_storage(
                        budget,
                        super::native_join_storage::node(unit).saturating_add(1),
                    )?;
                    MemberType::CarriedRational(*bounds, unit.clone())
                }
                ValueBounds::Text(bounds) => {
                    runtime_storage(
                        budget,
                        runtime_text_storage(&bounds.profile).saturating_add(1),
                    )?;
                    MemberType::TextBounds(bounds.clone())
                }
            }));
        }
        if self.record_types.contains_key(type_ref) {
            runtime_storage(
                budget,
                runtime_declaration_storage(
                    &self.identity,
                    DeclarationForm::RecordValueType,
                    type_ref,
                ),
            )?;
            return Ok(Some(MemberType::Record(declaration_id(
                &self.identity,
                DeclarationForm::RecordValueType,
                type_ref,
            ))));
        }
        if self.tuple_types.contains_key(type_ref) {
            runtime_storage(
                budget,
                runtime_declaration_storage(
                    &self.identity,
                    DeclarationForm::TupleValueType,
                    type_ref,
                ),
            )?;
            return Ok(Some(MemberType::Tuple(declaration_id(
                &self.identity,
                DeclarationForm::TupleValueType,
                type_ref,
            ))));
        }
        if self.variant_types.contains_key(type_ref) {
            runtime_storage(
                budget,
                runtime_declaration_storage(&self.identity, DeclarationForm::VariantType, type_ref),
            )?;
            return Ok(Some(MemberType::Variant(declaration_id(
                &self.identity,
                DeclarationForm::VariantType,
                type_ref,
            ))));
        }
        let Some(object) = self.object_types.get(type_ref) else {
            return Ok(None);
        };
        if object.interface {
            return Ok(None);
        }
        // The field tables of the lock stage reach this before
        // `ModelOwners::new` does, so an encoder refusal here is `None`, not a
        // refusal. The same key (or a longer one for an interface) is derived
        // there under the same limit, and the graph stage refuses the read at
        // the selection row when the encoder refuses it.
        let form = if self.state_machines.contains(type_ref) {
            DeclarationForm::StateMachine
        } else if self.processes.contains(type_ref) {
            DeclarationForm::Process
        } else {
            DeclarationForm::ObjectType
        };
        runtime_storage(
            budget,
            runtime_declaration_storage(&self.identity, form, type_ref),
        )?;
        Ok(Some(MemberType::Reference(declaration_id(
            &self.identity,
            form,
            type_ref,
        ))))
    }

    /// Render the selected field type used by a timed clock-place path.
    pub(super) fn clock_type_name_in(
        &self,
        type_ref: &SelectedTypeRef,
        budget: &mut Budget<'_, '_>,
    ) -> Result<Option<String>, ValidationFailure> {
        let mut current = type_ref;
        let mut seen: Vec<&str> = Vec::new();
        let mut layers = Vec::new();
        while let SelectedTypeRef::Declared(name) = current {
            let Some(definition) = self.collection_types.get(name) else {
                break;
            };
            if seen.contains(&name.as_ref()) {
                return Ok(None);
            }
            budget.push(&mut seen, name.as_ref(), "clock_type")?;
            match definition {
                SelectedTypeDefinition::Sequence(items) => {
                    budget.push(&mut layers, SelectedValueLayer::Sequence, "clock_type")?;
                    current = items;
                }
                SelectedTypeDefinition::Map(values) => {
                    budget.push(&mut layers, SelectedValueLayer::Map, "clock_type")?;
                    current = values;
                }
                SelectedTypeDefinition::Option(_) | SelectedTypeDefinition::Reference(_) => {
                    return Ok(None);
                }
            }
        }
        let Some(mut rendered) = (match current {
            SelectedTypeRef::Native(NativeTypeRef::Clock) => {
                Some(budget.concat(&["ix://quire/native/Clock"], "clock_type")?)
            }
            SelectedTypeRef::Native(NativeTypeRef::Text) => {
                Some(budget.concat(&["ix://quire/native/Text"], "clock_type")?)
            }
            SelectedTypeRef::Native(_) => None,
            SelectedTypeRef::Unit(name) => Some(budget.concat(&[name], "clock_type")?),
            SelectedTypeRef::Map(key, value) => {
                Some(budget.concat(&["Map<", key, ", ", value, ">"], "clock_type")?)
            }
            SelectedTypeRef::Declared(name) => Some(budget.concat(&[name], "clock_type")?),
        }) else {
            return Ok(None);
        };
        for layer in layers.into_iter().rev() {
            rendered = match layer {
                SelectedValueLayer::Sequence => {
                    budget.concat(&["Sequence<", &rendered, ">"], "clock_type")?
                }
                SelectedValueLayer::Map => {
                    budget.concat(&["Map<Text, ", &rendered, ">"], "clock_type")?
                }
                SelectedValueLayer::Option => return Ok(None),
            };
        }
        Ok(Some(rendered))
    }

    /// FR-322 step 4 over one declared `typeRef` and multiplicity; `None`
    /// when the tables give the slot no type.
    pub(super) fn slot_type(&self, slot: &TypedSlot) -> Option<MemberType> {
        self.slot_type_inner(slot, None).ok().flatten()
    }

    fn slot_type_inner(
        &self,
        slot: &TypedSlot,
        mut budget: Option<&mut NativeJoinBudget>,
    ) -> Result<Option<MemberType>, RuntimeTypeBudgetDenied> {
        let Some(type_ref) = slot.type_ref.as_ref() else {
            return Ok(None);
        };
        let Some(element) = self.element_type_runtime_inner(type_ref, budget.as_deref_mut())?
        else {
            return Ok(None);
        };
        let multiplicity = slot.multiplicity;
        if (multiplicity.lower, multiplicity.upper) == (1, Some(1)) {
            return Ok(Some(element));
        }
        let kind = CollectionKind::of(multiplicity.ordered, multiplicity.unique);
        let bounds = match (multiplicity.lower, multiplicity.upper) {
            (0, None) => None,
            (_, None) => return Ok(None),
            (lower, Some(upper)) => Some((lower, upper)),
        };
        runtime_storage(&mut budget, 2)?;
        Ok(Some(MemberType::Collection {
            kind,
            element: Box::new(element),
            bounds,
        }))
    }

    /// A field's type. Timed clock collections retain optionality on each
    /// selected clock element; other optional fields wrap the whole slot.
    pub(super) fn field_type(&self, field: &FieldDecl) -> Option<MemberType> {
        let slot = self.slot_type(&field.slot)?;
        Some(match (field.optional, field.slot.type_ref.as_ref(), slot) {
            (
                true,
                Some(SelectedTypeRef::Native(NativeTypeRef::Clock)),
                MemberType::Collection {
                    kind,
                    element,
                    bounds,
                },
            ) => MemberType::Collection {
                kind,
                element: Box::new(MemberType::Option(element)),
                bounds,
            },
            (true, _, slot) => MemberType::Option(Box::new(slot)),
            (false, _, slot) => slot,
        })
    }

    pub(super) fn slot_type_runtime(
        &self,
        slot: &TypedSlot,
        budget: &mut NativeJoinBudget,
    ) -> Result<Option<MemberType>, RuntimeTypeBudgetDenied> {
        self.slot_type_inner(slot, Some(budget))
    }

    /// Checked binder types of one record in effective field order.
    pub(super) fn record_binder_types(&self, record: &str) -> Option<RecordBinderTypes> {
        self.record_types.get(record)?;
        self.fields.of(record)?.binder_types(&self.checked_package)
    }

    /// Checked receiver, parameter and optional result binder types of the
    /// effective operation on the named object type, including inheritance.
    pub(super) fn effective_operation_binder_types(
        &self,
        object: &str,
        name: &str,
        budget: &mut Budget<'_, '_>,
    ) -> Result<Option<Vec<CheckedNodeId>>, ValidationFailure> {
        if self.state_machines.contains(object) {
            return Ok(None);
        }
        self.model_operation_binder_types(object, name, budget)
    }

    /// Checked receiver and member signature of an effective model operation.
    pub(super) fn model_operation_binder_types(
        &self,
        object: &str,
        name: &str,
        budget: &mut Budget<'_, '_>,
    ) -> Result<Option<Vec<CheckedNodeId>>, ValidationFailure> {
        let Some(declaration) = self.object_types.get(object) else {
            return Ok(None);
        };
        let operation = match self.resolve(object, MemberKind::Operation, name, budget) {
            Ok(Resolved::Operation(operation)) => operation,
            Ok(Resolved::Field(_)) | Err(ModelFailure::Refused(_)) => return Ok(None),
            Err(ModelFailure::Limit(failure)) => return Err(failure),
        };
        let form = if self.state_machines.contains(object) {
            DeclarationForm::StateMachine
        } else if self.processes.contains(object) {
            DeclarationForm::Process
        } else if declaration.interface {
            DeclarationForm::SystemsInterface
        } else {
            DeclarationForm::ObjectType
        };
        let object_id = declaration_id_in(&self.identity, form, object, budget)?;
        let Some(receiver) =
            MemberType::Reference(object_id).node_id_in(&self.checked_package, budget)?
        else {
            return Ok(None);
        };
        let mut types = budget.vec(
            1 + operation.parameters.len() + usize::from(operation.result.is_some()),
            "operation_binders",
        )?;
        types.push(receiver);
        for parameter in &operation.parameters {
            let Some(parameter_type) = self.slot_type_in(parameter, budget)? else {
                return Ok(None);
            };
            let Some(parameter_type) = parameter_type.node_id_in(&self.checked_package, budget)?
            else {
                return Ok(None);
            };
            types.push(parameter_type);
        }
        if let Some(result) = &operation.result {
            let Some(result_type) = self.slot_type_in(result, budget)? else {
                return Ok(None);
            };
            let Some(result_type) = result_type.node_id_in(&self.checked_package, budget)? else {
                return Ok(None);
            };
            types.push(result_type);
        }
        Ok(Some(types))
    }

    /// Candidate names, including ambiguous names, before effective-member
    /// resolution, in ascending order without repeats.
    pub(super) fn operation_candidate_names(
        &self,
        object: &str,
        budget: &mut Budget<'_, '_>,
    ) -> Result<Vec<Box<str>>, ValidationFailure> {
        let ancestors = self.ancestors(object, budget)?;
        let declared_by = |node: &str| self.object_types.get(node);
        let total: usize = std::iter::once(object)
            .chain(ancestors.iter().copied())
            .filter_map(declared_by)
            .map(|declared| declared.operations.len())
            .sum();
        // The names are borrowed from the declarations until the owned list
        // is made at the exact size the distinct ones need.
        let mut names: Vec<&str> = budget.vec(total, MODEL)?;
        for node in std::iter::once(object).chain(ancestors.iter().copied()) {
            if let Some(declared) = declared_by(node) {
                budget.charge(units(declared.operations.len()))?;
                names.extend(
                    declared
                        .operations
                        .iter()
                        .map(|operation| member_name(&operation.identity)),
                );
            }
        }
        names.sort_unstable();
        names.dedup();
        let mut owned = budget.vec(names.len(), MODEL)?;
        for name in names {
            owned.push(budget.text(name, MODEL)?);
        }
        Ok(owned)
    }

    /// Names of the effective operations reachable from one model type.
    pub(super) fn effective_operation_names(
        &self,
        object: &str,
        budget: &mut Budget<'_, '_>,
    ) -> Result<Vec<Box<str>>, ValidationFailure> {
        let mut names = self.operation_candidate_names(object, budget)?;
        let mut kept = 0;
        for at in 0..names.len() {
            let Some(name) = names.get(at) else {
                break;
            };
            match self.resolve(object, MemberKind::Operation, name, budget) {
                Ok(Resolved::Operation(_)) => {
                    names.swap(kept, at);
                    kept += 1;
                }
                Ok(Resolved::Field(_)) | Err(ModelFailure::Refused(_)) => {}
                Err(ModelFailure::Limit(failure)) => return Err(failure),
            }
        }
        names.truncate(kept);
        Ok(names)
    }
}

/// FR-322 step 2's owner of one model declaration node.
#[derive(Clone, Copy, Debug)]
pub(super) struct Owner<'m> {
    pub(super) package: &'m DomainModel,
    pub(super) form: DeclarationForm,
    /// The declaration's IR node identity.
    pub(super) node: &'m str,
    /// The index of `package` among the selected packages.
    pub(super) selection: usize,
}

/// One effective operation of a selected model declaration.
pub(super) struct ModelOperationClass {
    pub(super) object: Box<str>,
    pub(super) operation: Box<str>,
    pub(super) node: CheckedNodeId,
    pub(super) binder_types: Option<Vec<CheckedNodeId>>,
}

/// The authored structural node of one selected declaration, and who owns it.
#[derive(Debug)]
struct DeclaredRow<'m> {
    key: CheckedNodeId,
    owner: Owner<'m>,
    /// The graph position of the authored node.
    position: usize,
}

/// Every selected declaration's model declaration node key: the rows once,
/// and two sorted tables of row numbers, one by key and one by the selected
/// package, declaration and form.
#[derive(Debug, Default)]
pub(super) struct ModelOwners<'m> {
    rows: Vec<DeclaredRow<'m>>,
    by_key: Vec<usize>,
    by_declaration: Vec<usize>,
    /// The identity of every selected domain package, sorted.
    identities: Vec<&'m str>,
}

impl<'m> ModelOwners<'m> {
    /// The selected variant's family, determined by its retained payload types.
    pub(super) fn variant_family(&self, key: &CheckedNodeId) -> Option<&'static str> {
        let owner = self.owner_by_key(key)?;
        if owner.form != DeclarationForm::VariantType {
            return None;
        }
        let cases = owner.package.variant_types.get(owner.node)?;
        Some(if cases.values().any(|case| case.payload.is_some()) {
            "union"
        } else {
            "enum"
        })
    }

    /// Whether a domain package of `identity` is selected.
    pub(super) fn selects(&self, identity: &str) -> bool {
        self.identities.binary_search(&identity).is_ok()
    }

    /// The row whose key is `key`.
    fn row_by_key(&self, key: &CheckedNodeId) -> Option<&DeclaredRow<'m>> {
        let after = self.by_key.partition_point(|&at| {
            self.rows
                .get(at)
                .is_some_and(|row| row.key.cmp(key) != std::cmp::Ordering::Greater)
        });
        let row = self.rows.get(*self.by_key.get(after.checked_sub(1)?)?)?;
        (row.key.cmp(key) == std::cmp::Ordering::Equal).then_some(row)
    }

    pub(super) fn owner_by_key(&self, key: &CheckedNodeId) -> Option<Owner<'m>> {
        self.row_by_key(key).map(|row| row.owner)
    }

    /// The exact admitted node for a selected model declaration.
    pub(super) fn declaration_id(
        &self,
        identity: &str,
        node: &str,
        form: DeclarationForm,
    ) -> Option<&CheckedNodeId> {
        let order = |row: &DeclaredRow<'_>| {
            (
                row.owner.package.identity.as_ref(),
                row.owner.node,
                row.owner.form,
            )
                .cmp(&(identity, node, form))
        };
        let after = self.by_declaration.partition_point(|&at| {
            self.rows
                .get(at)
                .is_some_and(|row| order(row) != std::cmp::Ordering::Greater)
        });
        let row = self
            .rows
            .get(*self.by_declaration.get(after.checked_sub(1)?)?)?;
        (order(row) == std::cmp::Ordering::Equal).then_some(&row.key)
    }

    /// Effective operation names, declaration nodes and checked binder records
    /// for one selected domain package. A model member has no node of its own.
    pub(super) fn operation_classes(
        &self,
        identity: &str,
        meter: &mut super::WorkMeter,
    ) -> Result<Vec<ModelOperationClass>, ValidationFailure> {
        let mut classes = Vec::new();
        for row in self.by_key.iter().filter_map(|&at| self.rows.get(at)) {
            let (id, owner) = (&row.key, &row.owner);
            if owner.package.identity.as_ref() != identity
                || !matches!(
                    owner.form,
                    DeclarationForm::ObjectType
                        | DeclarationForm::StateMachine
                        | DeclarationForm::Process
                        | DeclarationForm::SystemsInterface
                )
            {
                continue;
            }
            let mut budget = Budget::new(meter, owner.package.root, owner.package.bytes);
            let names = owner
                .package
                .effective_operation_names(owner.node, &mut budget)?;
            for name in names {
                let binder =
                    owner
                        .package
                        .model_operation_binder_types(owner.node, &name, &mut budget)?;
                let object = budget.text(member_name(owner.node), "operation_class")?;
                let node = budget.clone_id(id, "operation_class")?;
                budget.push(
                    &mut classes,
                    ModelOperationClass {
                        object,
                        operation: name,
                        node,
                        binder_types: binder,
                    },
                    "operation_class",
                )?;
            }
        }
        Ok(classes)
    }
    pub(super) fn refinement_object_fields(
        &self,
        key: &crate::checked_package::shared::CheckedNodeId,
    ) -> Option<&[Box<str>]> {
        let owner = &self.row_by_key(key)?.owner;
        if owner.form != DeclarationForm::ObjectType {
            return None;
        }
        owner.package.fields.exposed_names(owner.node)
    }
    #[cfg(test)]
    /// Recomputes the key of every selected model declaration
    /// of each admitted document; `charge` is called once per key. Nothing
    /// is charged to a read-storage budget.
    pub(super) fn new<Body>(
        models: &'m [DomainModel],
        nodes: &[CheckedSemanticNodeV2<Body>],
        mut charge: impl FnMut(usize) -> Result<(), ValidationFailure>,
    ) -> Result<Self, ValidationFailure> {
        let mut meter = WorkMeter::new(u64::MAX);
        let mut budget = Budget::node(&mut meter, 0);
        Self::build(models, nodes, &mut budget, |_, root| charge(root))
    }

    /// [`Self::new`], with every allocation requested of `budget` at
    /// `PackageRead(Graph, node)` before it is made: the tables at the
    /// first node, each owned key at the node it names. `visit` is called
    /// once per key with the selected package's root.
    pub(super) fn build<Body>(
        models: &'m [DomainModel],
        nodes: &[CheckedSemanticNodeV2<Body>],
        budget: &mut Budget<'_, '_>,
        mut visit: impl FnMut(&mut Budget<'_, '_>, usize) -> Result<(), ValidationFailure>,
    ) -> Result<Self, ValidationFailure> {
        const F: &str = "model";
        let owned = |candidate: &CheckedSemanticNodeV2<Body>| {
            matches!(candidate.owner, Some(CheckedNodeOwner::Model { .. }))
        };
        budget.set_index(0);
        // The authored nodes by (package, declaration) and graph position.
        let mut authored: Vec<(&str, &str, usize)> =
            budget.vec(nodes.iter().filter(|candidate| owned(candidate)).count(), F)?;
        for (position, candidate) in nodes.iter().enumerate() {
            if let Some(CheckedNodeOwner::Model { identity, node }) = &candidate.owner {
                authored.push((identity.as_ref(), node.as_ref(), position));
            }
        }
        authored.sort_unstable();
        let declared: usize = models
            .iter()
            .map(|package| {
                package.object_types.len()
                    + package.relationships.len()
                    + package.relation_nodes.len()
                    + package.record_types.len()
                    + package.tuple_types.len()
                    + package.variant_types.len()
                    + package.populations.len()
                    + package.persistence_interfaces.len()
            })
            .sum();
        let mut identities: Vec<&str> = budget.vec(models.len(), F)?;
        let mut rows: Vec<DeclaredRow<'m>> = budget.vec(declared, F)?;
        for (index, package) in models.iter().enumerate() {
            identities.push(package.identity.as_ref());
            let objects = package.object_types.iter().map(|(node, declared)| {
                let form = if package.state_machines.contains(node.as_ref()) {
                    DeclarationForm::StateMachine
                } else if package.processes.contains(node.as_ref()) {
                    DeclarationForm::Process
                } else if declared.interface {
                    DeclarationForm::SystemsInterface
                } else {
                    DeclarationForm::ObjectType
                };
                (node, form)
            });
            let relationships = package
                .relationships
                .keys()
                .chain(&package.relation_nodes)
                .map(|node| (node, DeclarationForm::Relationship));
            let records = package
                .record_types
                .keys()
                .map(|node| (node, DeclarationForm::RecordValueType));
            let tuples = package
                .tuple_types
                .keys()
                .map(|node| (node, DeclarationForm::TupleValueType));
            let variants = package
                .variant_types
                .keys()
                .map(|node| (node, DeclarationForm::VariantType));
            let populations = package
                .populations
                .iter()
                .map(|node| (node, DeclarationForm::Population));
            let persistence_interfaces = package
                .persistence_interfaces
                .iter()
                .map(|node| (node, DeclarationForm::PersistenceInterface));
            for (node, form) in objects
                .chain(records)
                .chain(tuples)
                .chain(variants)
                .chain(relationships)
                .chain(populations)
                .chain(persistence_interfaces)
            {
                budget.set_index(package.root);
                visit(budget, package.root)?;
                let key = (package.identity.as_ref(), node.as_ref());
                let first = authored.partition_point(|row| (row.0, row.1) < key);
                for &(_, _, position) in authored
                    .get(first..)
                    .unwrap_or_default()
                    .iter()
                    .take_while(|row| (row.0, row.1) == key)
                {
                    let Some(candidate) = nodes.get(position) else {
                        continue;
                    };
                    if candidate.node_tag.as_ref() == form.tag()
                        && candidate.semantic_form.as_ref() == form.form()
                    {
                        budget.set_index(position);
                        budget.request(node_id_bytes(&candidate.node_id), form.tag())?;
                        rows.push(DeclaredRow {
                            key: candidate.node_id.clone(),
                            owner: Owner {
                                package,
                                form,
                                node,
                                selection: index,
                            },
                            position,
                        });
                    }
                }
            }
        }
        budget.set_index(0);
        identities.sort_unstable();
        identities.dedup();
        let mut by_key: Vec<usize> = budget.vec(rows.len(), F)?;
        by_key.extend(0..rows.len());
        by_key.sort_unstable_by(|&a, &b| match (rows.get(a), rows.get(b)) {
            (Some(a_row), Some(b_row)) => a_row.key.cmp(&b_row.key).then(a.cmp(&b)),
            _ => a.cmp(&b),
        });
        // A key held twice names its last row.
        let mut kept = 0;
        for at in 0..by_key.len() {
            let repeated = by_key
                .get(at + 1)
                .zip(by_key.get(at))
                .and_then(|(&next, &this)| Some(rows.get(next)?.key == rows.get(this)?.key))
                .unwrap_or(false);
            if !repeated {
                by_key.swap(kept, at);
                kept += 1;
            }
        }
        by_key.truncate(kept);
        let mut by_declaration: Vec<usize> = budget.vec(rows.len(), F)?;
        by_declaration.extend(0..rows.len());
        let declaration = |at: usize| {
            rows.get(at).map(|row| {
                (
                    row.owner.package.identity.as_ref(),
                    row.owner.node,
                    row.owner.form,
                )
            })
        };
        by_declaration
            .sort_unstable_by(|&a, &b| declaration(a).cmp(&declaration(b)).then(a.cmp(&b)));
        Ok(Self {
            rows,
            by_key,
            by_declaration,
            identities,
        })
    }

    /// Whether `node` is a model declaration node: a `model` or `relation`
    /// node that carries no `declaration`, or whose key is a selected
    /// declaration's model declaration node key.
    pub(super) fn is_model_declaration_node<Body>(
        &self,
        node: &CheckedSemanticNodeV2<Body>,
        tag: CheckedNodeTag,
    ) -> bool {
        matches!(tag, CheckedNodeTag::Model | CheckedNodeTag::Relation)
            && (node.declaration.is_none() || self.row_by_key(&node.node_id).is_some())
    }

    /// FR-322 step 2 over one wire node.
    pub(super) fn recover<Body: EmptyAggregateBody>(
        &self,
        node: &CheckedSemanticNodeV2<Body>,
    ) -> Result<Owner<'m>, ModelRefusal> {
        let owner = self
            .owner_by_key(&node.node_id)
            .ok_or_else(ModelRefusal::unselected)?;
        check_declaration_node(owner.form, node, node.body.is_empty_aggregate())?;
        Ok(owner)
    }

    /// Implements: FR-347. the record value type whose model declaration node key is
    /// `key`, `None` when `key` names no selected record.
    pub(super) fn record_by_key(&self, key: &CheckedNodeId) -> Option<Owner<'m>> {
        self.owner_by_key(key)
            .filter(|owner| owner.form == DeclarationForm::RecordValueType)
    }

    #[cfg(test)]
    /// An owned copy of the key index, which an admitted package retains
    /// after the documents it borrows are gone. Nothing is charged to a
    /// read-storage budget.
    pub(super) fn index(&self) -> DeclarationIndex {
        let mut meter = WorkMeter::new(u64::MAX);
        let mut budget = Budget::node(&mut meter, 0);
        self.index_charged(&mut budget)
            .unwrap_or_else(|_| unreachable!("an unmetered budget admits every request"))
    }

    /// [`Self::index`], each owned key and node text requested at
    /// `PackageRead(Graph, node)` before it is made.
    pub(super) fn index_charged(
        &self,
        budget: &mut Budget<'_, '_>,
    ) -> Result<DeclarationIndex, ValidationFailure> {
        const F: &str = "model";
        budget.set_index(0);
        let mut entries = budget.vec(self.by_key.len(), F)?;
        for row in self.by_key.iter().filter_map(|&at| self.rows.get(at)) {
            budget.set_index(row.position);
            let form = row.owner.form.tag();
            budget.request(
                node_id_bytes(&row.key).saturating_add(row.owner.node.len()),
                form,
            )?;
            entries.push((
                row.key.clone(),
                DeclaredNode {
                    form: row.owner.form,
                    selection: row.owner.selection,
                    node: row.owner.node.into(),
                },
            ));
        }
        Ok(DeclarationIndex(SortedMap::from_sorted(entries)))
    }
}

/// FR-322 step 2 over one wire node whose key names a declaration of `form`:
/// the node's tag and form are the declaration's (`missing_declaration`/
/// `missing-selection` otherwise), and its fixed members hold, `semantic_type`
/// itself, no `declaration`, no `recursion_group`, an empty body
/// (`invalid_package`/`invalid-derived-shape` otherwise). The one statement of the
/// rule: [`ModelOwners::recover`] at admission and the field accessor of an
/// admitted package both apply it.
pub(super) fn check_declaration_node<Body>(
    form: DeclarationForm,
    node: &CheckedSemanticNodeV2<Body>,
    body_is_empty_aggregate: bool,
) -> Result<(), ModelRefusal> {
    if form.tag() != node.node_tag.as_ref() || form.form() != node.semantic_form.as_ref() {
        return Err(ModelRefusal::unselected());
    }
    let exact_owner_id = matches!(
        &node.owner,
        Some(CheckedNodeOwner::Model { identity, node: declaration })
            if node.node_id.package.as_ref() == identity.as_ref()
                && matches!(node.node_id.path.as_slice(),
                    [CheckedNodePathSegment::Name(tag),
                     CheckedNodePathSegment::Name(semantic_form),
                     CheckedNodePathSegment::Name(path)]
                        if tag.as_ref() == form.tag()
                            && semantic_form.as_ref() == form.form()
                            && path.as_ref() == declaration.as_ref())
    );
    let fixed = exact_owner_id
        && node.semantic_type == node.node_id
        && node.declaration.is_none()
        && node.recursion_group.is_none()
        && (form != DeclarationForm::Population || node.dependencies.is_empty())
        && body_is_empty_aggregate;
    if fixed {
        Ok(())
    } else {
        Err(ModelRefusal::new(
            CheckedPackageRefusalCode::InvalidPackage,
            CheckedPackageRefusalCause::InvalidDerivedShape,
        ))
    }
}

impl<'m> ModelOwners<'m> {
    /// FR-322 steps 2 and 3 for the member of `kind` named `name` on the
    /// model declaration node `node`: the recovered owner and the member,
    /// or the refusal the first failing step determined. Step 3's charges
    /// go to `meter` at the owner's selection row; an exhausted limit is the
    /// outer error.
    pub(super) fn resolve_member<Body: EmptyAggregateBody>(
        &self,
        node: &CheckedSemanticNodeV2<Body>,
        kind: MemberKind,
        name: &str,
        meter: &mut WorkMeter,
    ) -> Result<Result<(Owner<'m>, Resolved<'m>), ModelRefusal>, ValidationFailure> {
        let owner = match self.recover(node) {
            Ok(owner) => owner,
            Err(refusal) => return Ok(Err(refusal)),
        };
        if owner.form == DeclarationForm::Population {
            return Ok(Err(ModelRefusal::ineligible()));
        }
        let mut budget = Budget::new(meter, owner.package.root, owner.package.bytes);
        match owner.package.resolve(owner.node, kind, name, &mut budget) {
            Ok(member) => Ok(Ok((owner, member))),
            Err(ModelFailure::Refused(refusal)) => Ok(Err(refusal)),
            Err(ModelFailure::Limit(failure)) => Err(failure),
        }
    }
}

impl Owner<'_> {
    /// The object type (or systems interface) this owner declares, `None`
    /// for a relationship.
    pub(super) fn object_type(&self) -> Option<&FieldBearingTypeDecl> {
        self.package.object_types.get(self.node)
    }
}

/// How many bytes of a domain package document one work unit covers.
/// Informational tuning: nothing but the reader's own `work` limit reads it.
const DOCUMENT_BYTES_PER_WORK: usize = 1024;

/// A step 1 refusal and the member of the selection row it is about
/// (`None`: the row itself).
#[derive(Debug, Eq, PartialEq)]
pub(super) struct SelectionRefusal {
    pub(super) refusal: ModelRefusal,
    pub(super) member: Option<&'static str>,
    pub(super) declaration_identity: Option<Box<str>>,
    pub(super) declaration_origin: Option<IntakeDeclarationOrigin>,
    /// Member path within the selected type definition, retained privately.
    pub(super) declaration_path: Option<SegPath>,
    /// A selected-document pointer that the public refusal contract locates.
    pub(super) document_pointer: Option<JsonPointer>,
}

impl SelectionRefusal {
    const fn at(
        code: CheckedPackageRefusalCode,
        cause: CheckedPackageRefusalCause,
        member: Option<&'static str>,
    ) -> Self {
        Self {
            refusal: ModelRefusal::new(code, cause),
            member,
            declaration_identity: None,
            declaration_origin: None,
            declaration_path: None,
            document_pointer: None,
        }
    }

    fn located<'n>(
        refusal: ModelRefusal,
        node: impl Into<Option<View<'n>>>,
        budget: &mut Budget<'_, '_>,
    ) -> Result<Self, ValidationFailure> {
        let node = node.into();
        let identity = node
            .and_then(|node| node.get("identity"))
            .and_then(View::as_str);
        budget.charge(1_u64.saturating_add(identity.map_or(0, |value| units(value.len()))))?;
        let declaration_identity = identity
            .map(|identity| budget.text(identity, "declarations"))
            .transpose()?;
        Ok(Self {
            refusal,
            member: None,
            declaration_identity,
            declaration_origin: retained_origin(node.and_then(|node| node.get("origin")), budget)?,
            declaration_path: None,
            document_pointer: None,
        })
    }

    fn group(
        refusal: ModelRefusal,
        nodes: &[(&str, usize, View<'_>)],
        budget: &mut Budget<'_, '_>,
    ) -> Result<Self, ValidationFailure> {
        let Some((_, _, first)) = nodes.first() else {
            return Ok(Self::at(refusal.code, refusal.cause, None));
        };
        let mut retained = Self::located(refusal, *first, budget)?;
        let mut agrees = retained.declaration_origin.is_some();
        for (_, _, node) in nodes.iter().skip(1) {
            budget.charge(1)?;
            let origin = retained_origin(node.get("origin"), budget)?;
            agrees &= origin.is_some() && origin == retained.declaration_origin;
        }
        if !agrees {
            retained.declaration_origin = None;
        }
        Ok(retained)
    }
}

/// FR-322 selected variant payload refusals name both the selection row and
/// the exact payload member in the selected Semantic IR document.
fn unsupported_variant_payload(
    case: View<'_>,
    type_index: usize,
    case_index: usize,
    budget: &mut Budget<'_, '_>,
) -> Result<SelectionRefusal, ValidationFailure> {
    let mut refusal = SelectionRefusal::located(
        ModelRefusal::new(
            CheckedPackageRefusalCode::UnsupportedConstruct,
            CheckedPackageRefusalCause::DeclarationForm,
        ),
        case,
        budget,
    )?;
    refusal.document_pointer = Some(
        JsonPointer::root()
            .key("types")
            .index(type_index)
            .key("variants")
            .index(case_index)
            .key("payloadType"),
    );
    Ok(refusal)
}

/// FR-069: a selected tuple refusal names the offending type or position in
/// the full Semantic IR document as well as the package's model selection row.
fn selected_tuple_refusal(
    refusal: ModelRefusal,
    node: View<'_>,
    type_index: usize,
    position_index: Option<usize>,
    budget: &mut Budget<'_, '_>,
) -> Result<SelectionRefusal, ValidationFailure> {
    let mut selected = SelectionRefusal::located(refusal, node, budget)?;
    let mut pointer = JsonPointer::root().key("types").index(type_index);
    if let Some(position_index) = position_index {
        pointer = pointer.key("positions").index(position_index);
    }
    selected.document_pointer = Some(pointer);
    Ok(selected)
}

/// Why FR-322 step 1 stopped at one selected package: a refusal, or a
/// reader limit exhausted while reading its document.
#[derive(Debug)]
pub(super) enum SelectionFailure {
    Refused(Box<SelectionRefusal>),
    Limit(ValidationFailure),
    /// The document holds a number with no exact RFC 8785 spelling:
    /// `noncanonical_wire` at the root owner's `identity`, with the pointer of the
    /// first such number in the document and the cause `inexact-integer` or
    /// `inexact-number`.
    InexactNumber {
        document_pointer: JsonPointer,
        cause: CheckedPackageRefusalCause,
    },
}

impl From<SelectionRefusal> for SelectionFailure {
    fn from(refusal: SelectionRefusal) -> Self {
        Self::Refused(Box::new(refusal))
    }
}

impl From<ValidationFailure> for SelectionFailure {
    fn from(failure: ValidationFailure) -> Self {
        Self::Limit(failure)
    }
}

impl From<ModelRefusal> for SelectionFailure {
    fn from(refusal: ModelRefusal) -> Self {
        Self::Refused(Box::new(SelectionRefusal::at(
            refusal.code,
            refusal.cause,
            None,
        )))
    }
}

/// A Semantic IR 2.0.0 document's own `package.identity`. The document's
/// `package.version` is neither required nor read: a selection binds by
/// identity alone.
pub(super) fn admit_selection(
    identity: &str,
    checked_package: &str,
    evidence: &CheckedPackageEvidence,
    budget: &mut Budget<'_, '_>,
    unit_refs: &UnitRefs,
) -> Result<(DomainModel, Arc<quire_canonical::Document>), SelectionFailure> {
    let selected =
        document::admit_document(identity, evidence.domain_package_document(identity), budget)?;
    let model = read_semantic_ir(View::root(&selected), checked_package, budget, unit_refs)?;
    Ok((model, selected))
}

/// Whether `text` is an FR-154 object id, `^[A-Za-z][A-Za-z0-9_]*$`.
fn is_object_id(text: &str) -> bool {
    let mut bytes = text.bytes();
    bytes
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic())
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

/// An optional array member: absent reads as empty.
fn list<'v>(value: View<'v>, member: &str) -> Result<List<'v>, ModelRefusal> {
    match value.get(member) {
        None => Ok(List::empty()),
        Some(items) => items.as_array().ok_or_else(ModelRefusal::malformed),
    }
}

fn text<'v>(value: View<'v>, member: &str) -> Result<&'v str, ModelRefusal> {
    value
        .get(member)
        .and_then(View::as_str)
        .ok_or_else(ModelRefusal::malformed)
}

/// A Semantic IR multiplicity's shape, `upper` absent when unbounded.
fn semantic_ir_multiplicity(value: View<'_>) -> Result<Multiplicity, ModelRefusal> {
    let multiplicity = value
        .get("multiplicity")
        .ok_or_else(ModelRefusal::malformed)?;
    let lower = multiplicity
        .get("lower")
        .and_then(View::as_u64)
        .ok_or_else(ModelRefusal::malformed)?;
    let upper = match multiplicity.get("upper") {
        None => None,
        Some(upper) => Some(upper.as_u64().ok_or_else(ModelRefusal::malformed)?),
    };
    let flag = |member: &str| {
        multiplicity
            .get(member)
            .and_then(View::as_bool)
            .ok_or_else(ModelRefusal::malformed)
    };
    Ok(Multiplicity {
        lower,
        upper,
        ordered: flag("ordered")?,
        unique: flag("unique")?,
    })
}

/// A member's IR node identity, which FR-154 requires to be
/// `<owner>/<name>`.
// Reads a selected domain package document's member identity text.
fn member_identity<'v>(member: View<'v>, owner: &str) -> Result<&'v str, ModelRefusal> {
    let identity = text(member, "identity")?;
    let name = identity
        .strip_prefix(owner)
        .and_then(|rest| rest.strip_prefix('/'))
        .ok_or_else(ModelRefusal::malformed)?;
    if name.is_empty() || name.contains('/') {
        return Err(ModelRefusal::malformed());
    }
    Ok(identity)
}

type RationalValueError = (ModelRefusal, RationalPath);

/// Where in a Rational value type a refusal points: a member of the type, or
/// a member of one of its constraints.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RationalPath {
    head: &'static str,
    constraint: Option<(usize, &'static str)>,
}

impl RationalPath {
    const fn member(name: &'static str) -> Self {
        Self {
            head: name,
            constraint: None,
        }
    }

    const fn constraint(index: usize, name: &'static str) -> Self {
        Self {
            head: "constraints",
            constraint: Some((index, name)),
        }
    }

    fn segments(self) -> SegPath {
        let path = SegPath::from_slice(&[Segment::Name(self.head)]);
        match self.constraint {
            Some((index, name)) => path.then(Segment::Index(index)).then(Segment::Name(name)),
            None => path,
        }
    }
}

/// FR-154's ordered Rational value-type grammar and selected-unit resolution.
fn rational_value_type(
    value: View<'_>,
    unit_refs: &UnitRefs,
) -> Result<Option<ValueBounds>, RationalValueError> {
    let malformed = |path: RationalPath| (ModelRefusal::malformed(), path);
    let unsupported = |path: RationalPath| {
        (
            ModelRefusal::new(
                CheckedPackageRefusalCode::UnsupportedConstruct,
                CheckedPackageRefusalCause::DeclarationForm,
            ),
            path,
        )
    };
    let constraints =
        list(value, "constraints").map_err(|_| malformed(RationalPath::member("constraints")))?;
    let identity =
        text(value, "identity").map_err(|_| malformed(RationalPath::member("identity")))?;
    let keywords = [
        "minNumerator",
        "maxNumerator",
        "minDenominator",
        "maxDenominator",
    ];
    // The constraint of each keyword, with its position among the constraints.
    let mut found: [Option<(usize, View<'_>)>; 4] = [None; 4];
    for (index, constraint) in constraints.iter().enumerate() {
        let keyword = constraint
            .get("keyword")
            .and_then(View::as_str)
            .unwrap_or("");
        let Some(slot) = keywords.iter().position(|candidate| *candidate == keyword) else {
            return Err(unsupported(RationalPath::constraint(index, "keyword")));
        };
        if constraint.get("appliesTo").and_then(View::as_str) != Some(identity) {
            return Err(malformed(RationalPath::constraint(index, "appliesTo")));
        }
        if found
            .get_mut(slot)
            .is_none_or(|entry| entry.replace((index, constraint)).is_some())
        {
            return Err(malformed(RationalPath::constraint(index, "keyword")));
        }
    }
    if !constraints.is_empty() && found.iter().any(Option::is_none) {
        return Err(malformed(RationalPath::member("constraints")));
    }
    let mut numbers = [0_i128; 4];
    let mut read = 0;
    for entry in found.iter().flatten() {
        let (index, constraint) = entry;
        let operands = constraint.get("operands").and_then(View::as_object);
        let number = operands
            .filter(|operands| operands.len() == 1)
            .and_then(|operands| operands.get("value"))
            .and_then(|value| match value.node() {
                quire_canonical::Node::Number(_) => value
                    .as_i64()
                    .filter(|number| number.unsigned_abs() <= (1_u64 << 53)),
                quire_canonical::Node::String(text) => text
                    .parse::<i64>()
                    .ok()
                    .filter(|number| decimal_text_is(*number, text)),
                _ => None,
            })
            .ok_or_else(|| malformed(RationalPath::constraint(*index, "operands")))?;
        if let Some(slot) = numbers.get_mut(read) {
            *slot = i128::from(number);
        }
        read += 1;
    }
    let bounds = if read == 0 {
        None
    } else {
        let [numerator_min, numerator_max, denominator_min, denominator_max] = numbers;
        let bounds = RationalBounds {
            numerator_min,
            numerator_max,
            denominator_min,
            denominator_max,
        };
        if bounds.numerator_min > bounds.numerator_max
            || bounds.denominator_min <= 0
            || bounds.denominator_min > bounds.denominator_max
        {
            return Err(unsupported(RationalPath::member("constraints")));
        }
        Some(bounds)
    };
    let unit = match value.get("unit") {
        None => None,
        Some(unit) => match unit.as_str() {
            Some(name) if SelectedTypeRef::is_unit_name(name) => Some(
                unit_refs
                    .get(name)
                    .cloned()
                    .ok_or_else(|| (ModelRefusal::missing_name(), RationalPath::member("unit")))?,
            ),
            _ => return Err(malformed(RationalPath::member("unit"))),
        },
    };
    Ok(bounds.map(|bounds| ValueBounds::Rational(bounds, unit)))
}

/// An FCD value type bound as `Int[lo, hi]`, Rational, or `Text[min, max; profile]`.
// Reads a selected domain package document's value type.
#[ix_trace_rs::string_edge]
fn semantic_ir_value_type(value: View<'_>, unit_refs: &UnitRefs) -> Option<ValueBounds> {
    let constraint = |keyword: &str| -> Option<View<'_>> {
        let mut found = list(value, "constraints")
            .ok()?
            .iter()
            .filter(|constraint| constraint.get("keyword").and_then(View::as_str) == Some(keyword));
        let constraint = found.next()?;
        if found.next().is_some() {
            return None;
        }
        Some(constraint)
    };
    let integer = |keyword| -> Option<i128> {
        let value = constraint(keyword)?.get("operands")?.get("value")?;
        match value.node() {
            quire_canonical::Node::Number(_) => value.as_i64().map(i128::from),
            quire_canonical::Node::String(text) => text.parse().ok(),
            _ => None,
        }
    };
    let scalar = value.get("scalar").and_then(View::as_str)?;
    if scalar != "rational" && value.get("unit").is_some() {
        return None;
    }
    match scalar {
        "integer" => {
            let (lower, upper) = (integer("min")?, integer("max")?);
            (lower <= upper).then_some(ValueBounds::Integer(IntegerBounds { lower, upper }))
        }
        "rational" => rational_value_type(value, unit_refs).ok().flatten(),
        "string" => {
            let (lower, upper) = (
                u64::try_from(integer("minLength")?).ok()?,
                u64::try_from(integer("maxLength")?).ok()?,
            );
            let name = constraint("format")?
                .get("operands")?
                .get("name")?
                .as_str()?;
            let profile = name.strip_prefix("text-profile:")?;
            if lower > upper
                || !matches!(
                    profile,
                    "unicode-scalars" | "nfc" | "nfd" | "nfkc" | "nfkd" | "binary-utf8"
                )
            {
                return None;
            }
            Some(ValueBounds::Text(TextBounds {
                lower,
                upper,
                profile: profile.into(),
            }))
        }
        _ => None,
    }
}

/// FR-154's declaration-refusal order for one node. Parameter collisions and
/// names precede reference checks within an operation (FR-154-AC-14); other
/// duplicate declarations retain the table's later row.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Row {
    /// The node is not valid for its meaning, including a reference that
    /// names a node of the wrong meaning.
    Meaning,
    /// A repeated operation parameter identity suppresses checks of both names.
    ParameterDuplicate,
    /// An operation parameter's declared name is invalid for its identity.
    ParameterName,
    /// A reference names no node or member of the package.
    Reference,
    /// A relationship identity names a declaration already read in this document.
    Duplicate,
    /// A multiplicity has `lower > upper`.
    Multiplicity,
}

/// A member path segment: a member name, or a list index. A name orders
/// before an index and names by their UTF-8 bytes (FR-154).
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum Segment {
    Name(&'static str),
    Index(usize),
}

/// The deepest member path of a declaration: a list item's member's item's member.
const SEGMENT_DEPTH: usize = 8;

/// A member path held inline: no allocation, ordered as the list of its
/// segments is.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct SegPath {
    len: usize,
    segments: [Segment; SEGMENT_DEPTH],
}

impl SegPath {
    pub(super) fn from_slice(segments: &[Segment]) -> Self {
        let mut path = Self {
            len: 0,
            segments: [Segment::Index(0); SEGMENT_DEPTH],
        };
        for segment in segments {
            path = path.then(*segment);
        }
        path
    }

    /// This path followed by `tail`.
    pub(super) fn then(mut self, tail: Segment) -> Self {
        if let Some(slot) = self.segments.get_mut(self.len) {
            *slot = tail;
            self.len += 1;
        }
        self
    }
}

impl std::ops::Deref for SegPath {
    type Target = [Segment];

    fn deref(&self) -> &[Segment] {
        self.segments.get(..self.len).unwrap_or_default()
    }
}

impl Ord for SegPath {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (**self).cmp(&**other)
    }
}

impl PartialOrd for SegPath {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// One failure inside one node, ordered by table row and then member path.
struct Defect<'v> {
    row: Row,
    path: SegPath,
    refusal: ModelRefusal,
    /// The value the failure is about; `None` for an absent member, which
    /// reads as `null`.
    declaration: Option<View<'v>>,
}

/// The failures of one node: only the one FR-154 reports is kept, first by
/// table row, then by member path, so recording failures allocates nothing.
struct Defects<'v> {
    first: Option<Defect<'v>>,
    declaration: Option<View<'v>>,
}

impl<'v> Defects<'v> {
    fn new(declaration: View<'v>) -> Self {
        Self {
            first: None,
            declaration: Some(declaration),
        }
    }
    fn context(&mut self, declaration: View<'v>) {
        self.declaration = Some(declaration);
    }
    /// The context of a member the declaration lacks.
    fn context_absent(&mut self) {
        self.declaration = None;
    }
    fn push(&mut self, row: Row, path: &[Segment], refusal: ModelRefusal) {
        let path = SegPath::from_slice(path);
        if self
            .first
            .as_ref()
            .is_none_or(|first| (row, path) < (first.row, first.path))
        {
            self.first = Some(Defect {
                row,
                path,
                refusal,
                declaration: self.declaration,
            });
        }
    }

    /// The failure FR-154 reports for the node: first by table row, then by
    /// member path.
    fn first(self) -> Option<Defect<'v>> {
        self.first
    }
}

fn joined(base: &[Segment], tail: Segment) -> SegPath {
    SegPath::from_slice(base).then(tail)
}

/// Require a non-empty meaning member without accepting an absent or malformed list.
fn required_items<'v>(
    value: View<'v>,
    member: &'static str,
    defects: &mut Defects<'v>,
) -> List<'v> {
    match list(value, member) {
        Ok(items) if !items.is_empty() => items,
        _ => {
            defects.context(value);
            defects.push(
                Row::Meaning,
                &[Segment::Name(member)],
                ModelRefusal::malformed(),
            );
            List::empty()
        }
    }
}

fn meaning_reference<'v>(
    value: Option<View<'v>>,
    expected: &SortedSet<&str>,
    declared: &SortedSet<&str>,
    path: &[Segment],
    defects: &mut Defects<'v>,
) {
    match value {
        Some(value) => defects.context(value),
        None => defects.context_absent(),
    }
    match value.and_then(View::as_str) {
        Some(name) if expected.contains(name) => {}
        Some(name) if declared.contains(name) => {
            defects.push(Row::Meaning, path, ModelRefusal::malformed());
        }
        Some(_) => defects.push(Row::Reference, path, ModelRefusal::missing_name()),
        None if value.is_none_or(View::is_null) => {
            defects.push(Row::Reference, path, ModelRefusal::missing_name());
        }
        None => defects.push(Row::Meaning, path, ModelRefusal::malformed()),
    }
}

/// The identities of `items` that carry the string member `member`, in order.
fn identity_set<'v>(
    items: &List<'v>,
    member: &str,
    budget: &mut Budget<'_, '_>,
) -> Result<SortedSet<&'v str>, ValidationFailure> {
    let mut set = SortedSet::with_capacity(budget, items.len(), SET_FORM)?;
    for item in items.iter() {
        if let Some(identity) = item.get(member).and_then(View::as_str) {
            set.insert(budget, identity, SET_FORM)?;
        }
    }
    Ok(set)
}

/// The locus form of the declaration reader's tables.
const SET_FORM: &str = "declarations";

/// FR-208's required members are checked before a selected declaration can
/// contribute an authenticated Process, StateMachine or PersistenceInterface key.
fn check_selected_meaning<'v>(
    value: View<'v>,
    meaning: meaning::Kind,
    scope: &Scope<'_>,
    defects: &mut Defects<'v>,
    budget: &mut Budget<'_, '_>,
) -> Result<(), ValidationFailure> {
    if !matches!(
        meaning,
        meaning::Kind::Process | meaning::Kind::StateMachine | meaning::Kind::PersistenceInterface
    ) {
        return Ok(());
    }
    let declared = &scope.all_types;
    match meaning {
        meaning::Kind::Process => {
            let fields = list(value, "fields").unwrap_or_default();
            let identities = identity_set(&fields, "identity", budget)?;
            for (index, field) in required_items(value, "identityFields", defects)
                .iter()
                .enumerate()
            {
                if !field
                    .as_str()
                    .is_some_and(|identity| identities.contains(identity))
                {
                    defects.context(value);
                    defects.push(
                        Row::Meaning,
                        &[Segment::Name("identityFields"), Segment::Index(index)],
                        ModelRefusal::malformed(),
                    );
                }
            }
            let steps = required_items(value, "steps", defects);
            let mut names = SortedSet::with_capacity(budget, steps.len(), SET_FORM)?;
            for (index, step) in steps.iter().enumerate() {
                let base = [Segment::Name("steps"), Segment::Index(index)];
                defects.context(step);
                let name = step.get("name").and_then(View::as_str);
                if let Some(name) = name.filter(|name| is_object_id(name)) {
                    if !names.insert(budget, name, SET_FORM)? {
                        defects.push(
                            Row::Meaning,
                            &base,
                            ModelRefusal::new(
                                CheckedPackageRefusalCode::InvalidModelBinding,
                                CheckedPackageRefusalCause::ConflictingBinding,
                            ),
                        );
                    }
                } else {
                    defects.push(Row::Meaning, &base, ModelRefusal::malformed());
                }
                if !matches!(
                    step.get("stepKind").and_then(View::as_str),
                    Some("command" | "event" | "decision" | "compensation" | "wait")
                ) {
                    defects.push(
                        Row::Meaning,
                        &joined(&base, Segment::Name("stepKind")),
                        ModelRefusal::malformed(),
                    );
                }
                check_event_lists(
                    step,
                    &base,
                    &["consumes", "emits"],
                    scope,
                    declared,
                    defects,
                );
            }
        }
        meaning::Kind::StateMachine => {
            required_items(value, "operations", defects);
            let listed = required_items(value, "states", defects);
            let mut states = SortedSet::with_capacity(budget, listed.len(), SET_FORM)?;
            for (index, state) in listed.iter().enumerate() {
                let path = [Segment::Name("states"), Segment::Index(index)];
                defects.context(state);
                let identity = state.get("identity").and_then(View::as_str);
                let repeated = match identity {
                    Some(identity) => !states.insert(budget, identity, SET_FORM)?,
                    None => false,
                };
                if repeated {
                    defects.push(
                        Row::Meaning,
                        &path,
                        ModelRefusal::new(
                            CheckedPackageRefusalCode::InvalidModelBinding,
                            CheckedPackageRefusalCause::ConflictingBinding,
                        ),
                    );
                } else if identity.is_none() {
                    defects.push(Row::Meaning, &path, ModelRefusal::malformed());
                }
            }
            let operations = identity_set(
                &list(value, "operations").unwrap_or_default(),
                "identity",
                budget,
            )?;
            let authored = list(value, "clauses").unwrap_or_default();
            let mut clauses: SortedMap<&str, View<'_>> =
                SortedMap::with_capacity(budget, authored.len(), SET_FORM)?;
            for clause in authored.iter() {
                if let Some(id) = clause.get("clauseId").and_then(View::as_str) {
                    clauses.insert(budget, id, clause, SET_FORM)?;
                }
            }
            for (index, transition) in required_items(value, "transitions", defects)
                .iter()
                .enumerate()
            {
                let base = [Segment::Name("transitions"), Segment::Index(index)];
                for (member, expected) in
                    [("from", &states), ("to", &states), ("trigger", &operations)]
                {
                    let path = joined(&base, Segment::Name(member));
                    meaning_reference(transition.get(member), expected, declared, &path, defects);
                }
                if let Some(guard) = transition.get("guard") {
                    let path = joined(&base, Segment::Name("guard"));
                    defects.context(transition);
                    let clause_id = guard.get("clauseId").and_then(View::as_str);
                    let clause = clause_id.and_then(|id| clauses.get(id).copied());
                    match clause {
                        _ if guard.get("language").and_then(View::as_str) != Some("quire") => {
                            defects.push(Row::Meaning, &path, ModelRefusal::malformed());
                        }
                        None => defects.push(Row::Reference, &path, ModelRefusal::missing_name()),
                        Some(clause)
                            if clause.get("language").and_then(View::as_str) != Some("quire") =>
                        {
                            defects.push(Row::Meaning, &path, ModelRefusal::malformed());
                        }
                        Some(_) => {}
                    }
                }
                check_event_lists(transition, &base, &["emits"], scope, declared, defects);
            }
        }
        meaning::Kind::PersistenceInterface => {
            if !list(value, "fields").unwrap_or_default().is_empty() {
                defects.context(value);
                defects.push(
                    Row::Meaning,
                    &[Segment::Name("fields")],
                    ModelRefusal::malformed(),
                );
            }
            required_items(value, "operations", defects);
            if !list(value, "supertypes").unwrap_or_default().is_empty() {
                defects.context(value);
                defects.push(
                    Row::Meaning,
                    &[Segment::Name("supertypes")],
                    ModelRefusal::new(
                        CheckedPackageRefusalCode::UnsupportedConstruct,
                        CheckedPackageRefusalCause::DeclarationForm,
                    ),
                );
            }
            if !list(value, "clauses").unwrap_or_default().is_empty() {
                defects.context(value);
                defects.push(
                    Row::Meaning,
                    &[Segment::Name("clauses")],
                    ModelRefusal::new(
                        CheckedPackageRefusalCode::UnsupportedConstruct,
                        CheckedPackageRefusalCause::DeclarationForm,
                    ),
                );
            }
            for (index, operation) in list(value, "operations")
                .unwrap_or_default()
                .iter()
                .enumerate()
            {
                if operation
                    .get("pre")
                    .is_some_and(|pre| !pre.is_empty_array())
                    || operation
                        .get("post")
                        .is_some_and(|post| !post.is_empty_array())
                {
                    defects.context(operation);
                    defects.push(
                        Row::Meaning,
                        &[Segment::Name("operations"), Segment::Index(index)],
                        ModelRefusal::new(
                            CheckedPackageRefusalCode::UnsupportedConstruct,
                            CheckedPackageRefusalCause::DeclarationForm,
                        ),
                    );
                }
            }
            for (index, persisted) in required_items(value, "persists", defects)
                .iter()
                .enumerate()
            {
                meaning_reference(
                    Some(persisted),
                    &scope.object_types,
                    declared,
                    &[Segment::Name("persists"), Segment::Index(index)],
                    defects,
                );
            }
        }
        _ => {}
    }
    Ok(())
}

fn check_event_lists<'v>(
    value: View<'v>,
    base: &[Segment],
    members: &[&'static str],
    scope: &Scope<'_>,
    declared: &SortedSet<&str>,
    defects: &mut Defects<'v>,
) {
    for &member in members {
        if let Ok(events) = list(value, member) {
            for (index, event) in events.iter().enumerate() {
                let path = joined(&joined(base, Segment::Name(member)), Segment::Index(index));
                meaning_reference(Some(event), &scope.events, declared, &path, defects);
            }
        } else {
            defects.context(value);
            defects.push(
                Row::Meaning,
                &joined(base, Segment::Name(member)),
                ModelRefusal::malformed(),
            );
        }
    }
}

/// One declared type node: its declaration and meaning, or the refusal it draws.
type ClassifiedType<'v> = Result<(View<'v>, meaning::Kind), ModelRefusal>;

/// The document-wide sets a node's references are checked against. All are
/// known before any node's body is read, so no reference outcome depends on
/// the order the nodes are read in.
struct Scope<'v> {
    /// Every classified type identity, for wrong-kind versus absent references.
    all_types: SortedSet<&'v str>,
    /// Named option definitions, known before any member body is read.
    option_types: SortedSet<&'v str>,
    /// Declared value-definition forms, including yet-unread option/collection bodies.
    value_definitions: SortedSet<&'v str>,
    /// Object-identity forms accepted as declared field types.
    objects: SortedSet<&'v str>,
    /// Object and systems-interface supertypes, excluding process/state forms.
    nominal_objects: SortedSet<&'v str>,
    /// Object types alone may occur in a persistence interface's `persists` list.
    object_types: SortedSet<&'v str>,
    /// Event types alone may be consumed or emitted by steps and transitions.
    events: SortedSet<&'v str>,
    /// Process and state-machine supertypes must stay within their meanings.
    processes: SortedSet<&'v str>,
    state_machines: SortedSet<&'v str>,
    /// Record value types, which carry value fields without object identity.
    records: SortedSet<&'v str>,
    /// Meanings forbidden as a record field's declared type.
    record_field_forbidden: SortedSet<&'v str>,
    /// Type nodes whose own declaration is refused; a reference to one is
    /// not reported again, its own refusal stands for it (FR-154).
    refused: SortedSet<&'v str>,
    /// Field and operation members of selected field-bearing declarations.
    members: SortedSet<&'v str>,
    /// Relationship identities are collected before any declaration body is read.
    relationships: SortedSet<&'v str>,
}

/// How many declarations of each kind a document holds: the capacity every
/// table the reader builds is requested at.
#[derive(Default)]
struct Counts {
    classified: usize,
    refused: usize,
    value_definitions: usize,
    options: usize,
    objects: usize,
    nominal_objects: usize,
    object_types: usize,
    events: usize,
    processes: usize,
    state_machines: usize,
    records: usize,
    record_field_forbidden: usize,
    persistence_interfaces: usize,
    value_types: usize,
    tuple_types: usize,
    variant_types: usize,
    collection_types: usize,
    relation_nodes: usize,
    other_types: usize,
    members: usize,
    relationships: usize,
}

impl Counts {
    fn of(classified: &[(&str, ClassifiedType<'_>)]) -> Self {
        let mut counts = Self {
            classified: classified.len(),
            ..Self::default()
        };
        for (_, entry) in classified {
            let Ok((declared, meaning)) = entry else {
                counts.refused += 1;
                continue;
            };
            let members = |name| list(*declared, name).map_or(0, |items| items.len());
            match meaning {
                meaning::Kind::SequenceType
                | meaning::Kind::MapType
                | meaning::Kind::ReferenceType => {
                    counts.value_definitions += 1;
                    counts.collection_types += 1;
                }
                meaning::Kind::OptionType => {
                    counts.value_definitions += 1;
                    counts.options += 1;
                    counts.collection_types += 1;
                }
                meaning::Kind::ObjectType
                | meaning::Kind::StateMachine
                | meaning::Kind::Process
                | meaning::Kind::SystemsInterface
                | meaning::Kind::RecordValueType => {
                    if *meaning == meaning::Kind::RecordValueType {
                        counts.records += 1;
                    } else {
                        counts.objects += 1;
                        match meaning {
                            meaning::Kind::Process => counts.processes += 1,
                            meaning::Kind::StateMachine => counts.state_machines += 1,
                            _ => {
                                counts.nominal_objects += 1;
                                if *meaning == meaning::Kind::ObjectType {
                                    counts.object_types += 1;
                                }
                            }
                        }
                    }
                    if matches!(
                        meaning,
                        meaning::Kind::Process | meaning::Kind::StateMachine
                    ) {
                        counts.record_field_forbidden += 1;
                    }
                    counts.members += members("fields") + members("operations");
                    counts.relationships += members("relationships");
                }
                meaning::Kind::ValueType => counts.value_types += 1,
                meaning::Kind::TupleValueType => counts.tuple_types += 1,
                meaning::Kind::VariantType => counts.variant_types += 1,
                meaning::Kind::SystemsConnection => {
                    counts.relation_nodes += 1;
                    counts.relationships += 1;
                }
                meaning::Kind::EventType
                | meaning::Kind::PersistenceInterface
                | meaning::Kind::Namespace
                | meaning::Kind::Population
                | meaning::Kind::SystemsPart
                | meaning::Kind::SystemsPort
                | meaning::Kind::SystemsAllocation => {
                    counts.other_types += 1;
                    if matches!(
                        meaning,
                        meaning::Kind::EventType
                            | meaning::Kind::PersistenceInterface
                            | meaning::Kind::Namespace
                    ) {
                        counts.record_field_forbidden += 1;
                    }
                    if *meaning == meaning::Kind::EventType {
                        counts.events += 1;
                    }
                    if *meaning == meaning::Kind::PersistenceInterface {
                        counts.persistence_interfaces += 1;
                    }
                }
            }
        }
        counts
    }
}

impl<'v> Scope<'v> {
    fn new(counts: &Counts, budget: &mut Budget<'_, '_>) -> Result<Self, ValidationFailure> {
        let mut set = |capacity| SortedSet::with_capacity(budget, capacity, SET_FORM);
        Ok(Self {
            all_types: set(counts.classified - counts.refused)?,
            option_types: set(counts.options)?,
            value_definitions: set(counts.value_definitions)?,
            objects: set(counts.objects)?,
            nominal_objects: set(counts.nominal_objects)?,
            object_types: set(counts.object_types)?,
            events: set(counts.events)?,
            processes: set(counts.processes)?,
            state_machines: set(counts.state_machines)?,
            records: set(counts.records)?,
            record_field_forbidden: set(counts.record_field_forbidden)?,
            refused: set(counts.refused + counts.relationships)?,
            members: set(counts.members)?,
            relationships: set(counts.relationships)?,
        })
    }
}

/// A node's own refusal, from its identity, its kind and whether another
/// node shares its identity, in FR-154's table order.
// Reads a selected domain package document's node id prefix.
#[ix_trace_rs::string_edge]
fn classify<'v>(
    prefix: &str,
    meanings: &SortedMap<(&str, &str), &'v str>,
    declared: &[(&'v str, usize, View<'v>)],
    node: &str,
) -> ClassifiedType<'v> {
    let one = |declared: View<'v>| -> ClassifiedType<'v> {
        if !node.strip_prefix(prefix).is_some_and(is_object_id) {
            return Err(ModelRefusal::malformed());
        }
        let meaning = match declared.get("kind") {
            Some(kind) if kind.as_str() == Some("sequence") => Some(meaning::Kind::SequenceType),
            Some(kind) if kind.as_str() == Some("map") => Some(meaning::Kind::MapType),
            Some(kind) if kind.as_str() == Some("option") => Some(meaning::Kind::OptionType),
            Some(kind) if kind.as_str() == Some("reference") => Some(meaning::Kind::ReferenceType),
            Some(kind) if kind.is_object() => {
                let module = kind.get("module").and_then(View::as_str);
                let name = kind.get("name").and_then(View::as_str);
                module
                    .zip(name)
                    .and_then(|(module, name)| meanings.get(&(module, name)).copied())
                    .and_then(meaning::Kind::from_wire)
            }
            _ => None,
        };
        meaning
            .map(|meaning| (declared, meaning))
            .ok_or_else(ModelRefusal::malformed)
    };
    let mut single = None;
    for (_, _, declared) in declared {
        single = Some(one(*declared)?);
    }
    match (declared.len(), single) {
        (1, Some(single)) => Ok(single),
        _ => Err(ModelRefusal::new(
            CheckedPackageRefusalCode::InvalidModelBinding,
            CheckedPackageRefusalCause::ConflictingBinding,
        )),
    }
}

/// The work of reading one type node: itself and every member list entry.
fn node_size(value: View<'_>) -> u64 {
    let len = |value: View<'_>, member: &str| {
        value
            .get(member)
            .and_then(View::as_array)
            .map_or(0, |items| items.len())
    };
    let mut total = 1_usize;
    for member in [
        "fields",
        "identityFields",
        "operations",
        "clauses",
        "states",
        "transitions",
        "steps",
        "persists",
        "supertypes",
        "relationships",
        "positions",
        "variants",
    ] {
        total = total.saturating_add(len(value, member));
    }
    if matches!(
        value.get("kind").and_then(View::as_str),
        Some("sequence" | "map" | "option" | "reference")
    ) {
        total = total.saturating_add(1);
    }
    let members = |name: &str| value.get(name).and_then(View::as_array).unwrap_or_default();
    for operation in members("operations") {
        total = total.saturating_add(len(operation, "params"));
    }
    for step in members("steps") {
        total = total
            .saturating_add(len(step, "consumes"))
            .saturating_add(len(step, "emits"));
    }
    for transition in members("transitions") {
        total = total.saturating_add(len(transition, "emits"));
    }
    units(total)
}

/// The type nodes of a document in identity order, each declaration with its
/// position in the document.
struct Declared<'v> {
    rows: Vec<(&'v str, usize, View<'v>)>,
}

impl<'v> Declared<'v> {
    /// Every declaration of `node`, in document order.
    fn group(&self, node: &str) -> &[(&'v str, usize, View<'v>)] {
        let start = self.rows.partition_point(|row| row.0 < node);
        let rest = self.rows.get(start..).unwrap_or_default();
        let len = rest.partition_point(|row| row.0 == node);
        rest.get(..len).unwrap_or_default()
    }

    /// The document position of the first declaration of `node`.
    fn position(&self, node: &str) -> Option<usize> {
        self.group(node).first().map(|row| row.1)
    }
}

/// [`read_semantic_ir`] over a retained document, for a test outside this
/// module.
#[cfg(test)]
pub(super) fn read_semantic_ir_of(
    document: &quire_canonical::Document,
    checked_package: &str,
    budget: &mut Budget<'_, '_>,
    unit_refs: &UnitRefs,
) -> Result<DomainModel, SelectionFailure> {
    read_semantic_ir(View::root(document), checked_package, budget, unit_refs)
}

/// Reads an admitted Semantic IR 2.0.0 document's declarations (FR-154),
/// with tuple declarations in document order before variants, then the
/// remaining nodes in ascending IR identity; each node's own failures report
/// in FR-154's table order.
/// Every node and every member read is charged to `budget`.
pub(super) fn read_semantic_ir(
    document: View<'_>,
    checked_package: &str,
    budget: &mut Budget<'_, '_>,
    unit_refs: &UnitRefs,
) -> Result<DomainModel, SelectionFailure> {
    const F: &str = SET_FORM;
    let identity = document::identity_of(document).ok_or_else(ModelRefusal::malformed)?;
    let constructs = list(document, "constructs")?;
    let mut meanings: SortedMap<(&str, &str), &str> =
        SortedMap::with_capacity(budget, constructs.len(), F)?;
    for construct in constructs {
        let kind = construct.get("kind").ok_or_else(ModelRefusal::malformed)?;
        let meaning = construct
            .get("construct")
            .map_or(Err(ModelRefusal::malformed()), |inner| {
                text(inner, "meaning")
            })?;
        meanings.insert(
            budget,
            (text(kind, "module")?, text(kind, "name")?),
            meaning,
            F,
        )?;
    }
    let prefix_len = "ix://".len() + identity.len() + 1;
    budget.request(prefix_len, F)?;
    let mut prefix = String::with_capacity(prefix_len);
    prefix.push_str("ix://");
    prefix.push_str(identity);
    prefix.push('/');
    let declared_types = list(document, "types")?;
    let alias_count = declared_types
        .iter()
        .filter(|declared| declared.get("kind").and_then(View::as_str) == Some("alias"))
        .count();
    let mut aliases: SortedMap<Box<str>, Box<str>> =
        SortedMap::with_capacity(budget, alias_count, F)?;
    let mut seen: SortedSet<&str> =
        SortedSet::with_capacity(budget, declared_types.len() - alias_count, F)?;
    let mut declared: Declared<'_> = Declared {
        rows: budget.vec(declared_types.len() - alias_count, F)?,
    };
    for (position, row) in declared_types.iter().enumerate() {
        // A node with no identity has none to share and is malformed on its
        // own; the empty identity orders before every other.
        let Some(node) = row.get("identity").and_then(View::as_str) else {
            return Err(SelectionRefusal::located(ModelRefusal::malformed(), row, budget)?.into());
        };
        if row.get("kind").and_then(View::as_str) == Some("alias") {
            budget.charge(node_size(row))?;
            let target = text(row, "target")?;
            let key = budget.text(node, F)?;
            let value = budget.text(target, F)?;
            if aliases.insert(budget, key, value, F)?.is_some() || seen.contains(node) {
                return Err(
                    SelectionRefusal::located(ModelRefusal::malformed(), row, budget)?.into(),
                );
            }
            continue;
        }
        if aliases.contains_key(node) {
            return Err(SelectionRefusal::located(ModelRefusal::malformed(), row, budget)?.into());
        }
        seen.insert(budget, node, F)?;
        declared.rows.push((node, position, row));
    }
    drop(seen);
    // Rows are ordered by identity, then by document position.
    declared
        .rows
        .sort_unstable_by(|a, b| a.0.cmp(b.0).then(a.1.cmp(&b.1)));
    // Each node's meaning, or the refusal its own identity, uniqueness or
    // kind draws; read before any body so a reference check sees every
    // declared node.
    let distinct = declared
        .rows
        .iter()
        .enumerate()
        .filter(|(at, row)| {
            *at == 0
                || declared
                    .rows
                    .get(at - 1)
                    .is_none_or(|prior| prior.0 != row.0)
        })
        .count();
    let mut classified: Vec<(&str, ClassifiedType)> = budget.vec(distinct, F)?;
    let mut at = 0;
    while let Some(first) = declared.rows.get(at) {
        let node = first.0;
        let group = declared.group(node);
        classified.push((node, classify(&prefix, &meanings, group, node)));
        at += group.len();
    }
    classified.sort_unstable_by_key(|(node, classified)| match classified {
        Ok((_, meaning::Kind::TupleValueType)) => (0, declared.position(node).unwrap_or(0), *node),
        Ok((_, meaning::Kind::VariantType)) => (1, declared.position(node).unwrap_or(0), *node),
        _ => (2, 0, *node),
    });
    let counts = Counts::of(&classified);
    let dimension_rows = list(document, "dimensions")?;
    let mut dimensions: SortedMap<Box<str>, SelectedDimension> =
        SortedMap::with_capacity(budget, dimension_rows.len(), F)?;
    for dimension in dimension_rows {
        budget.charge(node_size(dimension))?;
        let identity = text(dimension, "identity")?;
        let name = text(dimension, "name")?;
        let base = dimension
            .get("base")
            .and_then(View::as_bool)
            .ok_or_else(ModelRefusal::malformed)?;
        let term_rows = list(dimension, "terms")?;
        let mut terms = budget.vec(term_rows.len(), F)?;
        for term in term_rows {
            let pair = (
                budget.text(text(term, "dimension")?, F)?,
                budget.text(text(term, "exponent")?, F)?,
            );
            terms.push(pair);
        }
        if base && !terms.is_empty() {
            return Err(ModelRefusal::malformed().into());
        }
        let key = budget.text(identity, F)?;
        let selected = SelectedDimension {
            name: budget.text(name, F)?,
            base,
            terms,
        };
        if dimensions.insert(budget, key, selected, F)?.is_some() {
            return Err(ModelRefusal::malformed().into());
        }
    }
    let unit_rows = list(document, "units")?;
    let mut units: SortedMap<Box<str>, SelectedUnit> =
        SortedMap::with_capacity(budget, unit_rows.len(), F)?;
    for unit in unit_rows.iter() {
        budget.charge(node_size(unit))?;
        let identity = text(unit, "identity")?;
        let target = unit
            .get("target")
            .map(|_| text(unit, "target"))
            .transpose()?;
        let target = target.map(|target| budget.text(target, F)).transpose()?;
        let selected = SelectedUnit {
            name: budget.text(text(unit, "name")?, F)?,
            dimension: budget.text(text(unit, "dimension")?, F)?,
            target,
            scale: budget.text(text(unit, "scale")?, F)?,
            offset: budget.text(text(unit, "offset")?, F)?,
        };
        let key = budget.text(identity, F)?;
        if units.insert(budget, key, selected, F)?.is_some() {
            return Err(ModelRefusal::malformed().into());
        }
    }
    let population_rows = list(document, "populations")?;
    let mut model = DomainModel {
        identity: budget.text(identity, F)?,
        checked_package: budget.text(checked_package, F)?,
        root: budget.root,
        bytes: budget.bytes,
        object_types: SortedMap::with_capacity(budget, counts.objects, F)?,
        state_machines: SortedSet::with_capacity(budget, counts.state_machines, F)?,
        processes: SortedSet::with_capacity(budget, counts.processes, F)?,
        persistence_interfaces: SortedSet::with_capacity(budget, counts.persistence_interfaces, F)?,
        persistence_operations: SortedMap::with_capacity(budget, counts.persistence_interfaces, F)?,
        record_types: SortedMap::with_capacity(budget, counts.records, F)?,
        event_types: SortedSet::with_capacity(budget, counts.events, F)?,
        tuple_types: SortedMap::with_capacity(budget, counts.tuple_types, F)?,
        value_types: SortedMap::with_capacity(budget, counts.value_types, F)?,
        aliases,
        variant_types: SortedMap::with_capacity(budget, counts.variant_types, F)?,
        dimensions,
        units,
        collection_types: SortedMap::with_capacity(budget, counts.collection_types, F)?,
        unit_refs: unit_refs.clone(),
        own_seconds_declared: list(document, "units")?
            .iter()
            .any(|unit| unit.get("name").and_then(View::as_str) == Some("s")),
        relationships: SortedMap::with_capacity(budget, counts.relationships, F)?,
        populations: SortedSet::with_capacity(budget, population_rows.len(), F)?,
        population_members: SortedMap::with_capacity(budget, population_rows.len(), F)?,
        relation_nodes: SortedSet::with_capacity(budget, counts.relation_nodes, F)?,
        other_types: SortedMap::with_capacity(budget, counts.other_types, F)?,
        fields: FieldTables::default(),
    };
    for population in population_rows {
        budget.charge(node_size(population))?;
        let node = text(population, "identity")?;
        if population.get("node").is_some() {
            return Err(ModelRefusal::malformed().into());
        }
        let group = [(node, 0, population)];
        let (_, meaning) = classify(&prefix, &meanings, &group, node)?;
        let key = budget.text(node, F)?;
        if meaning != meaning::Kind::Population
            || !model.populations.insert(budget, key.clone(), F)?
        {
            return Err(ModelRefusal::malformed().into());
        }
        let member_rows = list(population, "members")?;
        let mut members = SortedSet::with_capacity(budget, member_rows.len(), F)?;
        for member in member_rows {
            let member = member.as_str().ok_or_else(ModelRefusal::malformed)?;
            let member = budget.text(member, F)?;
            members.insert(budget, member, F)?;
        }
        model.population_members.insert(budget, key, members, F)?;
    }
    let mut scope = Scope::new(&counts, budget)?;
    for (node, classified) in &classified {
        let (declared_node, meaning) = match classified {
            Ok(classified) => *classified,
            Err(_) => {
                scope.refused.insert(budget, node, F)?;
                continue;
            }
        };
        scope.all_types.insert(budget, node, F)?;
        match meaning {
            meaning::Kind::SequenceType | meaning::Kind::MapType | meaning::Kind::ReferenceType => {
                scope.value_definitions.insert(budget, node, F)?;
            }
            meaning::Kind::OptionType => {
                scope.value_definitions.insert(budget, node, F)?;
                scope.option_types.insert(budget, node, F)?;
            }
            meaning::Kind::ObjectType
            | meaning::Kind::StateMachine
            | meaning::Kind::Process
            | meaning::Kind::SystemsInterface
            | meaning::Kind::RecordValueType => {
                if meaning == meaning::Kind::RecordValueType {
                    scope.records.insert(budget, node, F)?;
                } else {
                    scope.objects.insert(budget, node, F)?;
                    if meaning == meaning::Kind::Process {
                        scope.processes.insert(budget, node, F)?;
                    } else if meaning == meaning::Kind::StateMachine {
                        scope.state_machines.insert(budget, node, F)?;
                    } else {
                        scope.nominal_objects.insert(budget, node, F)?;
                        if meaning == meaning::Kind::ObjectType {
                            scope.object_types.insert(budget, node, F)?;
                        }
                    }
                }
                if matches!(
                    meaning,
                    meaning::Kind::Process | meaning::Kind::StateMachine
                ) {
                    scope.record_field_forbidden.insert(budget, node, F)?;
                }
                // Members and relationships are collected from every object
                // type before any is read: a reference to one resolves the
                // same whichever node declares it first.
                for member in ["fields", "operations"] {
                    for item in list(declared_node, member).unwrap_or_default() {
                        if let Ok(identity) = member_identity(item, node) {
                            scope.members.insert(budget, identity, F)?;
                        }
                    }
                }
                for item in list(declared_node, "relationships").unwrap_or_default() {
                    if let Some(identity) = item.get("identity").and_then(View::as_str) {
                        if !valid_relationship_identity(&model.identity, identity) {
                            scope.refused.insert(budget, identity, F)?;
                        }
                        scope.relationships.insert(budget, identity, F)?;
                    }
                }
            }
            meaning::Kind::ValueType => {
                if declared_node.get("unit").is_some()
                    && declared_node.get("scalar").and_then(View::as_str) != Some("rational")
                {
                    let mut selected = SelectionRefusal::located(
                        ModelRefusal::malformed(),
                        declared_node,
                        budget,
                    )?;
                    let type_index = declared
                        .position(node)
                        .ok_or_else(ModelRefusal::malformed)?;
                    selected.document_pointer = Some(
                        JsonPointer::root()
                            .key("types")
                            .index(type_index)
                            .key("unit"),
                    );
                    return Err(selected.into());
                }
                let bounds = if declared_node.get("scalar").and_then(View::as_str)
                    == Some("rational")
                {
                    match rational_value_type(declared_node, &model.unit_refs) {
                        Ok(bounds) => bounds,
                        Err((refusal, path)) => {
                            let mut selected =
                                SelectionRefusal::located(refusal, declared_node, budget)?;
                            let type_index = declared
                                .position(node)
                                .ok_or_else(ModelRefusal::malformed)?;
                            let mut pointer = JsonPointer::root().key("types").index(type_index);
                            for segment in path.segments().iter() {
                                pointer = match segment {
                                    Segment::Name(name) => pointer.key(name),
                                    Segment::Index(index) => pointer.index(*index),
                                };
                            }
                            selected.document_pointer = Some(pointer);
                            return Err(selected.into());
                        }
                    }
                } else {
                    semantic_ir_value_type(declared_node, &model.unit_refs)
                };
                let key = budget.text(node, F)?;
                model.value_types.insert(budget, key, bounds, F)?;
            }
            meaning::Kind::TupleValueType => {
                let key = budget.text(node, F)?;
                model.tuple_types.insert(budget, key, Vec::new(), F)?;
            }
            meaning::Kind::VariantType => {
                let key = budget.text(node, F)?;
                model
                    .variant_types
                    .insert(budget, key, SortedMap::new(), F)?;
            }
            meaning::Kind::SystemsConnection => {
                let key = budget.text(node, F)?;
                model.relation_nodes.insert(budget, key, F)?;
                scope.relationships.insert(budget, node, F)?;
            }
            meaning::Kind::EventType
            | meaning::Kind::PersistenceInterface
            | meaning::Kind::Namespace
            | meaning::Kind::Population
            | meaning::Kind::SystemsPart
            | meaning::Kind::SystemsPort
            | meaning::Kind::SystemsAllocation => {
                let key = budget.text(node, F)?;
                model.other_types.insert(budget, key, meaning, F)?;
                if matches!(
                    meaning,
                    meaning::Kind::EventType
                        | meaning::Kind::PersistenceInterface
                        | meaning::Kind::Namespace
                ) {
                    scope.record_field_forbidden.insert(budget, node, F)?;
                }
                if meaning == meaning::Kind::EventType {
                    let key = budget.text(node, F)?;
                    model.event_types.insert(budget, key, F)?;
                    scope.events.insert(budget, node, F)?;
                }
                if meaning == meaning::Kind::PersistenceInterface {
                    let key = budget.text(node, F)?;
                    model.persistence_interfaces.insert(budget, key, F)?;
                }
            }
        }
    }
    let mut objects: Vec<(&str, FieldBearingTypeDecl)> = budget.vec(counts.objects, F)?;
    let mut records: Vec<(&str, FieldBearingTypeDecl)> = budget.vec(counts.records, F)?;
    let mut relationships: SortedMap<Box<str>, RelationshipDecl> =
        SortedMap::with_capacity(budget, counts.relationships, F)?;
    for (node, classified) in classified {
        let (declared_node, meaning) = match classified {
            Ok(value) => value,
            Err(refusal) => {
                let candidates = declared.group(node);
                return Err(SelectionRefusal::group(refusal, candidates, budget)?.into());
            }
        };
        budget.charge(node_size(declared_node))?;
        match meaning {
            meaning::Kind::OptionType | meaning::Kind::ReferenceType => {
                let type_index = declared
                    .position(node)
                    .ok_or_else(ModelRefusal::malformed)?;
                let definition =
                    option_types::read(declared_node, meaning, type_index, &model, &scope, budget)?;
                let key = budget.text(node, F)?;
                model.collection_types.insert(budget, key, definition, F)?;
            }
            meaning::Kind::SequenceType | meaning::Kind::MapType => {
                let member = if meaning == meaning::Kind::SequenceType {
                    "items"
                } else {
                    "values"
                };
                let reference = text(declared_node, member)?;
                let selected = model
                    .selected_type_ref(reference, budget)?
                    .ok_or_else(ModelRefusal::malformed)?;
                let refusal = match &selected {
                    SelectedTypeRef::Map(..) => Some(ModelRefusal::malformed()),
                    SelectedTypeRef::Unit(name) if !model.unit_refs.contains_key(name) => {
                        Some(ModelRefusal::missing_name())
                    }
                    SelectedTypeRef::Declared(name)
                        if scope.relationships.contains(name.as_ref()) =>
                    {
                        Some(ModelRefusal::malformed())
                    }
                    SelectedTypeRef::Declared(name) if !scope.all_types.contains(name.as_ref()) => {
                        Some(ModelRefusal::missing_name())
                    }
                    _ => None,
                };
                if let Some(refusal) = refusal {
                    return Err(SelectionRefusal::located(refusal, declared_node, budget)?.into());
                }
                let definition = if meaning == meaning::Kind::SequenceType {
                    SelectedTypeDefinition::Sequence(selected)
                } else {
                    SelectedTypeDefinition::Map(selected)
                };
                let key = budget.text(node, F)?;
                model.collection_types.insert(budget, key, definition, F)?;
            }
            meaning::Kind::ObjectType
            | meaning::Kind::StateMachine
            | meaning::Kind::Process
            | meaning::Kind::SystemsInterface
            | meaning::Kind::RecordValueType => {
                let mut defects = Defects::new(declared_node);
                check_selected_meaning(declared_node, meaning, &scope, &mut defects, budget)?;
                let (object, declared_relationships) = semantic_ir_object_type(
                    declared_node,
                    node,
                    meaning == meaning::Kind::SystemsInterface,
                    meaning == meaning::Kind::RecordValueType,
                    meaning,
                    &References {
                        model: &model,
                        scope: &scope,
                    },
                    &relationships,
                    &mut defects,
                    budget,
                )?;
                if let Some(refusal) = defects.first() {
                    let mut selected =
                        SelectionRefusal::located(refusal.refusal, refusal.declaration, budget)?;
                    let mut pointer = JsonPointer::root().key("types").index(
                        declared
                            .position(node)
                            .ok_or_else(ModelRefusal::malformed)?,
                    );
                    for step in refusal.path.iter() {
                        pointer = match step {
                            Segment::Name(name) => pointer.key(name),
                            Segment::Index(index) => pointer.index(*index),
                        };
                    }
                    selected.document_pointer = Some(pointer);
                    selected.declaration_path = Some(refusal.path);
                    if refusal.row == Row::ParameterDuplicate {
                        selected.declaration_origin = None;
                    }
                    return Err(selected.into());
                }
                if meaning == meaning::Kind::RecordValueType {
                    records.push((node, object));
                } else {
                    if meaning == meaning::Kind::StateMachine {
                        let key = budget.text(node, F)?;
                        model.state_machines.insert(budget, key, F)?;
                    } else if meaning == meaning::Kind::Process {
                        let key = budget.text(node, F)?;
                        model.processes.insert(budget, key, F)?;
                    }
                    objects.push((node, object));
                }
                for (identity, relationship) in declared_relationships {
                    // Unreachable for an admitted object: semantic_ir_object_type
                    // records every known/local duplicate at the later actual
                    // relationship before defects.first returns. AC-196 tests
                    // pin both same-owner and cross-owner first-refusal contexts.
                    // This defensive insertion check must not replace that pass.
                    if relationships
                        .insert(budget, identity, relationship, F)?
                        .is_some()
                    {
                        return Err(ModelRefusal::new(
                            CheckedPackageRefusalCode::InvalidModelBinding,
                            CheckedPackageRefusalCause::ConflictingBinding,
                        )
                        .into());
                    }
                }
            }
            meaning::Kind::ValueType => {
                // Bounds were read with the classified declarations, before
                // variant payload references are checked.
            }
            meaning::Kind::TupleValueType => {
                let Some(type_index) = declared.position(node) else {
                    return Err(ModelRefusal::malformed().into());
                };
                let positions = match list(declared_node, "positions") {
                    Ok(positions) => positions,
                    Err(refusal) => {
                        return Err(selected_tuple_refusal(
                            refusal,
                            declared_node,
                            type_index,
                            None,
                            budget,
                        )?
                        .into());
                    }
                };
                let mut types = budget.vec(positions.len(), F)?;
                let mut seen: SortedSet<&str> =
                    SortedSet::with_capacity(budget, positions.len(), F)?;
                for (ordinal, position) in positions.iter().enumerate() {
                    let identity = match text(position, "identity") {
                        Ok(identity) => identity,
                        Err(refusal) => {
                            return Err(selected_tuple_refusal(
                                refusal,
                                position,
                                type_index,
                                Some(ordinal),
                                budget,
                            )?
                            .into());
                        }
                    };
                    if !seen.insert(budget, identity, F)? {
                        return Err(selected_tuple_refusal(
                            ModelRefusal::new(
                                CheckedPackageRefusalCode::InvalidModelBinding,
                                CheckedPackageRefusalCause::ConflictingBinding,
                            ),
                            position,
                            type_index,
                            Some(ordinal),
                            budget,
                        )?
                        .into());
                    }
                }
                drop(seen);
                if [
                    "fields",
                    "identityFields",
                    "operations",
                    "supertypes",
                    "clauses",
                ]
                .iter()
                .any(|member| declared_node.get(member).is_some())
                {
                    return Err(selected_tuple_refusal(
                        ModelRefusal::malformed(),
                        declared_node,
                        type_index,
                        None,
                        budget,
                    )?
                    .into());
                }
                if positions.is_empty() {
                    return Err(selected_tuple_refusal(
                        ModelRefusal::malformed(),
                        declared_node,
                        type_index,
                        None,
                        budget,
                    )?
                    .into());
                }
                for (ordinal, position) in positions.iter().enumerate() {
                    let identity = text(position, "identity")?;
                    if !is_position_identity(identity, node, ordinal) {
                        return Err(selected_tuple_refusal(
                            ModelRefusal::malformed(),
                            position,
                            type_index,
                            Some(ordinal),
                            budget,
                        )?
                        .into());
                    }
                    let type_ref = match text(position, "typeRef") {
                        Ok(type_ref) => type_ref,
                        Err(refusal) => {
                            return Err(selected_tuple_refusal(
                                refusal,
                                position,
                                type_index,
                                Some(ordinal),
                                budget,
                            )?
                            .into());
                        }
                    };
                    let Some(admitted) = model.selected_type_ref(type_ref, budget)? else {
                        return Err(selected_tuple_refusal(
                            ModelRefusal::malformed(),
                            position,
                            type_index,
                            Some(ordinal),
                            budget,
                        )?
                        .into());
                    };
                    let mapped = match &admitted {
                        SelectedTypeRef::Native(
                            NativeTypeRef::Clock
                            | NativeTypeRef::Rational
                            | NativeTypeRef::Decimal
                            | NativeTypeRef::Text,
                        )
                        | SelectedTypeRef::Map(..) => false,
                        SelectedTypeRef::Declared(name)
                            if model.value_types.get(name).is_some_and(Option::is_none) =>
                        {
                            false
                        }
                        SelectedTypeRef::Declared(name)
                            if scope.all_types.contains(name.as_ref())
                                && !scope.records.contains(name.as_ref())
                                && !model.tuple_types.contains_key(name)
                                && !model.variant_types.contains_key(name)
                                && !model.value_types.contains_key(name)
                                && !scope.option_types.contains(name.as_ref()) =>
                        {
                            false
                        }
                        _ => true,
                    };
                    if !mapped {
                        return Err(selected_tuple_refusal(
                            ModelRefusal::malformed(),
                            position,
                            type_index,
                            Some(ordinal),
                            budget,
                        )?
                        .into());
                    }
                    types.push(admitted);
                }
                for (ordinal, (position, admitted)) in positions.iter().zip(&types).enumerate() {
                    let resolved = match admitted {
                        SelectedTypeRef::Native(_) => true,
                        SelectedTypeRef::Unit(name) => model.unit_refs.contains_key(name),
                        SelectedTypeRef::Declared(name) => {
                            model.value_types.contains_key(name)
                                || scope.records.contains(name.as_ref())
                                || model.tuple_types.contains_key(name)
                                || model.variant_types.contains_key(name)
                                || scope.option_types.contains(name.as_ref())
                        }
                        SelectedTypeRef::Map(..) => false,
                    };
                    if !resolved {
                        return Err(selected_tuple_refusal(
                            ModelRefusal::missing_name(),
                            position,
                            type_index,
                            Some(ordinal),
                            budget,
                        )?
                        .into());
                    }
                }
                let key = budget.text(node, F)?;
                model.tuple_types.insert(budget, key, types, F)?;
            }
            meaning::Kind::PersistenceInterface => {
                let mut defects = Defects::new(declared_node);
                check_selected_meaning(declared_node, meaning, &scope, &mut defects, budget)?;
                let (operations, _) = semantic_ir_object_type(
                    declared_node,
                    node,
                    true,
                    false,
                    meaning,
                    &References {
                        model: &model,
                        scope: &scope,
                    },
                    &relationships,
                    &mut defects,
                    budget,
                )?;
                if let Some(refusal) = defects.first() {
                    let mut selected =
                        SelectionRefusal::located(refusal.refusal, refusal.declaration, budget)?;
                    let mut pointer = JsonPointer::root().key("types").index(
                        declared
                            .position(node)
                            .ok_or_else(ModelRefusal::malformed)?,
                    );
                    for step in refusal.path.iter() {
                        pointer = match step {
                            Segment::Name(name) => pointer.key(name),
                            Segment::Index(index) => pointer.index(*index),
                        };
                    }
                    selected.document_pointer = Some(pointer);
                    selected.declaration_path = Some(refusal.path);
                    return Err(selected.into());
                }
                let key = budget.text(node, F)?;
                model
                    .persistence_operations
                    .insert(budget, key, operations, F)?;
            }
            meaning::Kind::VariantType => {
                let Some(type_index) = declared.position(node) else {
                    return Err(ModelRefusal::malformed().into());
                };
                let cases = match list(declared_node, "variants") {
                    Ok(cases) => cases,
                    Err(refusal) => {
                        return Err(
                            SelectionRefusal::located(refusal, declared_node, budget)?.into()
                        );
                    }
                };
                let mut names: SortedMap<Box<str>, SelectedVariantCase> =
                    SortedMap::with_capacity(budget, cases.len(), F)?;
                for (case_index, case) in cases.iter().enumerate() {
                    let name = match text(case, "name") {
                        Ok(name) => name,
                        Err(refusal) => {
                            return Err(
                                SelectionRefusal::located(refusal, declared_node, budget)?.into()
                            );
                        }
                    };
                    if name.is_empty() || names.contains_key(name) {
                        return Err(SelectionRefusal::located(
                            ModelRefusal::malformed(),
                            declared_node,
                            budget,
                        )?
                        .into());
                    }
                    let identity = match text(case, "identity") {
                        Ok(identity) if !identity.is_empty() => identity,
                        _ => {
                            return Err(SelectionRefusal::located(
                                ModelRefusal::malformed(),
                                case,
                                budget,
                            )?
                            .into());
                        }
                    };
                    let payload = match case.get("payloadType") {
                        None => None,
                        Some(payload) if payload.is_null() => None,
                        Some(payload) => match payload.as_str() {
                            Some(reference) => {
                                let Some(resolved) = model.selected_type_ref(reference, budget)?
                                else {
                                    return Err(unsupported_variant_payload(
                                        case, type_index, case_index, budget,
                                    )?
                                    .into());
                                };
                                Some(resolved)
                            }
                            None => {
                                return Err(SelectionRefusal::located(
                                    ModelRefusal::malformed(),
                                    case,
                                    budget,
                                )?
                                .into());
                            }
                        },
                    };
                    if let Some(reference) = &payload {
                        let valid = match reference {
                            SelectedTypeRef::Native(
                                NativeTypeRef::Boolean
                                | NativeTypeRef::Integer
                                | NativeTypeRef::Float32
                                | NativeTypeRef::Float64
                                | NativeTypeRef::String
                                | NativeTypeRef::Uuid
                                | NativeTypeRef::Timestamp
                                | NativeTypeRef::Bytes
                                | NativeTypeRef::JsonObject
                                | NativeTypeRef::Duration,
                            ) => true,
                            SelectedTypeRef::Native(
                                NativeTypeRef::Rational
                                | NativeTypeRef::Decimal
                                | NativeTypeRef::Text
                                | NativeTypeRef::Clock,
                            )
                            | SelectedTypeRef::Map(..) => false,
                            SelectedTypeRef::Unit(name) => model.unit_refs.contains_key(name),
                            SelectedTypeRef::Declared(name) => {
                                model.value_types.get(name).is_some_and(Option::is_some)
                                    || scope.records.contains(name.as_ref())
                                    || model.tuple_types.contains_key(name)
                                    || model.variant_types.contains_key(name)
                                    || scope.option_types.contains(name.as_ref())
                            }
                        };
                        if !valid {
                            return Err(unsupported_variant_payload(
                                case, type_index, case_index, budget,
                            )?
                            .into());
                        }
                    }
                    let key = budget.text(name, F)?;
                    let selected = SelectedVariantCase {
                        identity: budget.text(identity, F)?,
                        position: case_index,
                        payload,
                    };
                    names.insert(budget, key, selected, F)?;
                }
                if names.is_empty() {
                    return Err(SelectionRefusal::located(
                        ModelRefusal::malformed(),
                        declared_node,
                        budget,
                    )?
                    .into());
                }
                let key = budget.text(node, F)?;
                model.variant_types.insert(budget, key, names, F)?;
            }
            meaning::Kind::EventType
            | meaning::Kind::Namespace
            | meaning::Kind::Population
            | meaning::Kind::SystemsPart
            | meaning::Kind::SystemsPort
            | meaning::Kind::SystemsConnection
            | meaning::Kind::SystemsAllocation => {}
        }
    }
    for (node, object) in objects {
        let key = budget.text(node, F)?;
        model.object_types.insert(budget, key, object, F)?;
    }
    for (node, record) in records {
        let key = budget.text(node, F)?;
        model.record_types.insert(budget, key, record, F)?;
    }
    model.relationships = relationships;
    option_types::check_constraints(document, &model, &scope, budget)?;
    Ok(model)
}

/// Whether `identity` is the identity of position `ordinal` of tuple `node`,
/// `<node>/p<ordinal>`.
fn is_position_identity(identity: &str, node: &str, ordinal: usize) -> bool {
    use std::fmt::Write as _;
    struct Expect<'t>(&'t str);
    impl std::fmt::Write for Expect<'_> {
        fn write_str(&mut self, part: &str) -> std::fmt::Result {
            self.0 = self.0.strip_prefix(part).ok_or(std::fmt::Error)?;
            Ok(())
        }
    }
    let mut remaining = Expect(identity);
    write!(remaining, "{node}/p{ordinal}").is_ok() && remaining.0.is_empty()
}

/// What a node's references are checked against.
struct References<'a> {
    model: &'a DomainModel,
    scope: &'a Scope<'a>,
}

enum CheckedTypeRefInput<'a> {
    MapValue(&'a str),
    Unit,
    Native,
    Declared,
    Malformed,
}

impl<'a> CheckedTypeRefInput<'a> {
    #[ix_trace_rs::string_edge]
    fn from_wire(value: &'a str) -> Self {
        match SelectedTypeRef::admit(value) {
            Some(Admitted::Map("Text", value)) => Self::MapValue(value),
            Some(Admitted::Map(..)) => Self::Malformed,
            Some(Admitted::Unit(_)) => Self::Unit,
            Some(Admitted::Native) => Self::Native,
            Some(Admitted::Declared(_)) => Self::Declared,
            None if value.starts_with(NATIVE_PREFIX)
                || value
                    .strip_prefix("Map<")
                    .and_then(|tail| tail.strip_suffix('>'))
                    .is_some() =>
            {
                Self::Malformed
            }
            None => Self::Declared,
        }
    }
}

impl References<'_> {
    fn object_ref(&self, type_ref: &str) -> Result<(), (Row, ModelRefusal)> {
        if self.scope.objects.contains(type_ref) || self.scope.refused.contains(type_ref) {
            Ok(())
        } else if self.declared_elsewhere(type_ref) {
            Err((Row::Meaning, ModelRefusal::malformed()))
        } else {
            Err((Row::Reference, ModelRefusal::missing_name()))
        }
    }
    /// Whether `node` is declared by the document as something other than an
    /// object type: a value type, a type of another meaning, or a
    /// relationship.
    fn declared_elsewhere(&self, node: &str) -> bool {
        self.model.value_types.contains_key(node)
            || self.model.variant_types.contains_key(node)
            || self.model.tuple_types.contains_key(node)
            || self.model.record_types.contains_key(node)
            || self.model.collection_types.contains_key(node)
            || self.scope.records.contains(node)
            || self.model.other_types.contains_key(node)
            || self.scope.relationships.contains(node)
    }

    /// A field type or parameter or result type (FR-154 rows 3 and 4): a
    /// native value type, or a type of the document that is not a
    /// relationship.
    // Reads a selected domain package document's type reference prefix.
    fn check_type_ref(&self, type_ref: &str, path: &[Segment], defects: &mut Defects) {
        let path = joined(path, Segment::Name("typeRef"));
        let resolved = self
            .model
            .resolve_alias(type_ref)
            .and_then(SelectedTypeRef::admit);
        let resolved_name = match &resolved {
            Some(Admitted::Declared(name) | Admitted::Unit(name)) => name,
            _ => type_ref,
        };
        let input = match resolved {
            Some(Admitted::Native) => CheckedTypeRefInput::Native,
            _ => CheckedTypeRefInput::from_wire(resolved_name),
        };
        match input {
            CheckedTypeRefInput::MapValue(value) => self.check_type_ref(value, &[], defects),
            CheckedTypeRefInput::Unit if self.model.unit_refs.contains_key(resolved_name) => {}
            CheckedTypeRefInput::Unit => {
                defects.push(Row::Reference, &path, ModelRefusal::missing_name());
            }
            CheckedTypeRefInput::Native => {}
            CheckedTypeRefInput::Malformed => {
                defects.push(Row::Meaning, &path, ModelRefusal::malformed());
            }
            CheckedTypeRefInput::Declared if self.scope.relationships.contains(resolved_name) => {
                defects.push(Row::Meaning, &path, ModelRefusal::malformed());
            }
            CheckedTypeRefInput::Declared if self.scope.all_types.contains(resolved_name) => {}
            CheckedTypeRefInput::Declared if self.scope.refused.contains(resolved_name) => {}
            CheckedTypeRefInput::Declared => {
                defects.push(Row::Reference, &path, ModelRefusal::missing_name());
            }
        }
    }

    /// A supertype names an object type of the document.
    fn check_supertype(
        &self,
        supertype: &str,
        meaning: meaning::Kind,
        path: &[Segment],
        defects: &mut Defects,
    ) {
        let eligible = match meaning {
            meaning::Kind::RecordValueType => &self.scope.records,
            meaning::Kind::Process => &self.scope.processes,
            meaning::Kind::StateMachine => &self.scope.state_machines,
            _ => &self.scope.nominal_objects,
        };
        if eligible.contains(supertype) || self.scope.refused.contains(supertype) {
            return;
        }
        if self.declared_elsewhere(supertype)
            || self.scope.objects.contains(supertype)
            || self.scope.records.contains(supertype)
        {
            defects.push(Row::Meaning, path, ModelRefusal::malformed());
        } else {
            defects.push(Row::Reference, path, ModelRefusal::missing_name());
        }
    }

    /// A `redefines` names a member of the document (a member of a refused
    /// type is left to that type's own refusal).
    fn check_redefines(&self, target: &str, path: &[Segment], defects: &mut Defects) {
        let of_refused = target
            .rsplit_once('/')
            .is_some_and(|(owner, _)| self.scope.refused.contains(owner));
        if !self.scope.members.contains(target) && !of_refused {
            defects.push(Row::Reference, path, ModelRefusal::missing_name());
        }
    }

    /// Reads a field or operation's optional redefinition reference.
    fn redefinition(
        &self,
        member: View<'_>,
        base: &[Segment],
        defects: &mut Defects,
        budget: &mut Budget<'_, '_>,
    ) -> Result<Option<Box<str>>, ValidationFailure> {
        let path = joined(base, Segment::Name("redefines"));
        match member.get("redefines").map(View::as_str) {
            None => Ok(None),
            Some(Some(target)) => {
                self.check_redefines(target, &path, defects);
                Ok(Some(budget.text(target, "declarations")?))
            }
            Some(None) => {
                defects.push(Row::Meaning, &path, ModelRefusal::malformed());
                Ok(None)
            }
        }
    }

    /// A `typeRef` with a multiplicity: a field, parameter or result.
    fn slot(
        &self,
        value: View<'_>,
        base: &[Segment],
        defects: &mut Defects,
        budget: &mut Budget<'_, '_>,
    ) -> Result<TypedSlot, ValidationFailure> {
        let type_ref = match text(value, "typeRef") {
            Ok(type_ref) => {
                self.check_type_ref(type_ref, base, defects);
                type_ref
            }
            Err(refusal) => {
                defects.push(
                    Row::Meaning,
                    &joined(base, Segment::Name("typeRef")),
                    refusal,
                );
                ""
            }
        };
        let path = joined(base, Segment::Name("multiplicity"));
        let multiplicity = match semantic_ir_multiplicity(value) {
            Ok(multiplicity) => {
                if multiplicity
                    .upper
                    .is_some_and(|upper| multiplicity.lower > upper)
                {
                    defects.push(
                        Row::Multiplicity,
                        &path,
                        ModelRefusal::new(
                            CheckedPackageRefusalCode::InvalidModelBinding,
                            CheckedPackageRefusalCause::UnpreservedModelMeaning,
                        ),
                    );
                }
                multiplicity
            }
            Err(refusal) => {
                defects.push(Row::Meaning, &path, refusal);
                Multiplicity {
                    lower: 1,
                    upper: Some(1),
                    ordered: false,
                    unique: true,
                }
            }
        };
        let name = value
            .get("name")
            .and_then(View::as_str)
            .map(|name| budget.text(name, "declarations"))
            .transpose()?;
        Ok(TypedSlot {
            name,
            type_ref: self.model.selected_type_ref(type_ref, budget)?,
            multiplicity,
        })
    }
}

#[derive(Clone, Copy)]
enum RelationshipMember {
    Identity,
    Category,
    Composite,
    Direction,
    Origin,
    SourceEnd,
    TargetEnd,
}

impl RelationshipMember {
    #[string_edge]
    fn from_wire(text: &str) -> Option<Self> {
        match text {
            "identity" => Some(Self::Identity),
            "category" => Some(Self::Category),
            "composite" => Some(Self::Composite),
            "direction" => Some(Self::Direction),
            "origin" => Some(Self::Origin),
            "sourceEnd" => Some(Self::SourceEnd),
            "targetEnd" => Some(Self::TargetEnd),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
enum RelationshipCategory {
    Structural,
    Behavioral,
    Dataflow,
    Dependency,
    Realization,
    Governance,
    Traceability,
}

impl RelationshipCategory {
    #[string_edge]
    fn from_wire(text: &str) -> Option<Self> {
        match text {
            "structural" => Some(Self::Structural),
            "behavioral" => Some(Self::Behavioral),
            "dataflow" => Some(Self::Dataflow),
            "dependency" => Some(Self::Dependency),
            "realization" => Some(Self::Realization),
            "governance" => Some(Self::Governance),
            "traceability" => Some(Self::Traceability),
            _ => None,
        }
    }
}

/// Closed authored relationship metadata and ends, read at the selection row.
fn semantic_ir_relationship(
    value: View<'_>,
    owner: &str,
    base: &[Segment],
    references: &References<'_>,
    defects: &mut Defects,
    budget: &mut Budget<'_, '_>,
) -> Result<Option<(Box<str>, RelationshipDecl)>, ValidationFailure> {
    let parsed = (|| {
        let members = value.as_object()?;
        if !members
            .keys()
            .all(|key| RelationshipMember::from_wire(key).is_some())
        {
            return None;
        }
        let identity = value.get("identity")?.as_str()?;
        if !valid_relationship_identity(&references.model.identity, identity) {
            return None;
        }
        let direction = RelationshipDirection::from_wire(value.get("direction")?.as_str()?)?;
        if value
            .get("category")
            .and_then(View::as_str)
            .and_then(RelationshipCategory::from_wire)
            .is_none()
            || value.get("composite").and_then(View::as_bool).is_none()
            || !valid_relationship_origin(value.get("origin")?)
        {
            return None;
        }
        let source = relationship_end(value.get("sourceEnd")?, true)?;
        let target = relationship_end(value.get("targetEnd")?, false)?;
        Some((identity, source, target, direction))
    })();
    let Some((identity, source, target, direction)) = parsed else {
        defects.push(Row::Meaning, base, ModelRefusal::malformed());
        return Ok(None);
    };
    for (end, member) in [(&source, "sourceEnd"), (&target, "targetEnd")] {
        if let Err((row, refusal)) = references.object_ref(end.type_ref) {
            defects.push(row, &joined(base, Segment::Name(member)), refusal);
            return Ok(None);
        }
    }
    if source.type_ref != owner {
        defects.push(Row::Meaning, base, ModelRefusal::malformed());
        return Ok(None);
    }
    for (end, member) in [(&source, "sourceEnd"), (&target, "targetEnd")] {
        if end
            .multiplicity
            .upper
            .is_some_and(|upper| end.multiplicity.lower > upper)
        {
            defects.push(
                Row::Multiplicity,
                &joined(base, Segment::Name(member)),
                ModelRefusal::new(
                    CheckedPackageRefusalCode::InvalidModelBinding,
                    CheckedPackageRefusalCause::UnpreservedModelMeaning,
                ),
            );
            return Ok(None);
        }
    }
    let form = "declarations";
    let key = budget.text(identity, form)?;
    let mut own = |end: RelationshipEndView<'_>| -> Result<RelationshipEnd, ValidationFailure> {
        Ok(RelationshipEnd {
            role: end.role.map(|role| budget.text(role, form)).transpose()?,
            type_ref: budget.text(end.type_ref, form)?,
            multiplicity: end.multiplicity,
        })
    };
    let source = own(source)?;
    let target = own(target)?;
    Ok(Some((
        key,
        RelationshipDecl {
            source,
            target,
            direction,
        },
    )))
}

fn valid_relationship_identity(package: &str, identity: &str) -> bool {
    identity
        .strip_prefix("ix://")
        .and_then(|rest| rest.strip_prefix(package))
        .and_then(|rest| rest.strip_prefix("/relationship/"))
        .is_some_and(|name| {
            !name.is_empty() && !name.contains('/') && valid_semantic_identity(identity)
        })
}

/// One end of a relationship as the document spells it.
struct RelationshipEndView<'v> {
    role: Option<&'v str>,
    type_ref: &'v str,
    multiplicity: Multiplicity,
}

fn relationship_end(value: View<'_>, required_role: bool) -> Option<RelationshipEndView<'_>> {
    let members = value.as_object()?;
    if !members
        .keys()
        .all(|key| matches!(key, "type" | "role" | "multiplicity"))
    {
        return None;
    }
    let role = match members.get("role") {
        Some(role) => match role.as_str() {
            Some(role) if !role.is_empty() => Some(role),
            _ => return None,
        },
        None if !required_role => None,
        None => return None,
    };
    let multiplicity = value.get("multiplicity")?.as_object()?;
    if !multiplicity
        .keys()
        .all(|key| matches!(key, "lower" | "upper" | "ordered" | "unique"))
    {
        return None;
    }
    Some(RelationshipEndView {
        role,
        type_ref: value
            .get("type")?
            .as_str()
            .filter(|id| valid_semantic_identity(id))?,
        multiplicity: semantic_ir_multiplicity(value).ok()?,
    })
}

fn valid_relationship_origin(value: View<'_>) -> bool {
    OriginView::decode(value).is_some()
}

#[ix_trace_rs::string_edge]
fn valid_semantic_identity(identity: &str) -> bool {
    let Some((org, path)) = identity
        .strip_prefix("ix://")
        .and_then(|rest| rest.split_once('/'))
    else {
        return false;
    };
    org.as_bytes()
        .first()
        .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && org.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
        && path
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphanumeric)
        && path.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'~' | b':' | b'/' | b'-')
        })
}

#[ix_trace_rs::string_edge]
fn valid_source_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.contains('\\')
        && !path.contains('\0')
        && !(path.as_bytes().first().is_some_and(u8::is_ascii_alphabetic)
            && path.as_bytes().get(1) == Some(&b':'))
        && !path.split('/').any(|segment| segment == "..")
}

fn valid_semver(version: &str) -> bool {
    let (before_build, build) = version
        .split_once('+')
        .map_or((version, None), |(core, suffix)| (core, Some(suffix)));
    let (core, pre) = before_build
        .split_once('-')
        .map_or((before_build, None), |(core, suffix)| (core, Some(suffix)));
    let parts = core.split('.').collect::<Vec<_>>();
    parts.len() == 3
        && parts.iter().all(|part| {
            !part.is_empty()
                && (part.len() == 1 || !part.starts_with('0'))
                && part.bytes().all(|byte| byte.is_ascii_digit())
        })
        && [pre, build].into_iter().flatten().all(|suffix| {
            !suffix.is_empty()
                && suffix
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-'))
        })
}

/// One member list of the node, or the failure of a member that is not a list.
fn items<'v>(value: View<'v>, member: &'static str, defects: &mut Defects) -> List<'v> {
    items_at(value, member, &[Segment::Name(member)], defects)
}

/// One member identity, or the failure of one that is not `<owner>/<name>`.
fn identity_of(
    member: View<'_>,
    owner: &str,
    path: &[Segment],
    defects: &mut Defects,
    budget: &mut Budget<'_, '_>,
) -> Result<Box<str>, ValidationFailure> {
    match member_identity(member, owner) {
        Ok(identity) => budget.text(identity, "declarations"),
        Err(refusal) => {
            defects.push(
                Row::Meaning,
                &joined(path, Segment::Name("identity")),
                refusal,
            );
            Ok(Box::from(""))
        }
    }
}

/// The frame an operation declares: the closed members of
/// [`SourceOperationFrame`], each absent one empty. `None` when a member is
/// unknown or of the wrong kind.
fn operation_frame(
    raw: View<'_>,
    budget: &mut Budget<'_, '_>,
) -> Result<Option<SourceOperationFrame>, ValidationFailure> {
    let Some(members) = raw.as_object() else {
        return Ok(None);
    };
    if !members.keys().all(|key| {
        matches!(
            key,
            "modifies"
                | "creates"
                | "deletes"
                | "modifies_receiver"
                | "deletes_receiver"
                | "deletes_parameters"
        )
    }) {
        return Ok(None);
    }
    let mut names = |member: &str| -> Result<Option<Vec<Box<str>>>, ValidationFailure> {
        let Some(value) = members.get(member) else {
            return Ok(Some(Vec::new()));
        };
        let Some(items) = value.as_array() else {
            return Ok(None);
        };
        let mut owned = budget.vec(items.len(), "declarations")?;
        for item in items {
            let Some(text) = item.as_str() else {
                return Ok(None);
            };
            owned.push(budget.text(text, "declarations")?);
        }
        Ok(Some(owned))
    };
    let (Some(modifies), Some(creates), Some(deletes), Some(modifies_receiver)) = (
        names("modifies")?,
        names("creates")?,
        names("deletes")?,
        names("modifies_receiver")?,
    ) else {
        return Ok(None);
    };
    let Some(deletes_parameters) = names("deletes_parameters")? else {
        return Ok(None);
    };
    let deletes_receiver = match members.get("deletes_receiver") {
        None => false,
        Some(flag) => match flag.as_bool() {
            Some(flag) => flag,
            None => return Ok(None),
        },
    };
    Ok(Some(SourceOperationFrame {
        modifies,
        creates,
        deletes,
        modifies_receiver,
        deletes_receiver,
        deletes_parameters,
    }))
}

/// One object type or systems interface; every failure of the node is
/// recorded in `defects`, and the declaration returned is meaningful only
/// when there is none.
// Reads a selected domain package document's supertype references.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn semantic_ir_object_type<'v>(
    value: View<'v>,
    node: &str,
    interface: bool,
    record: bool,
    meaning: meaning::Kind,
    references: &References<'_>,
    known_relationships: &SortedMap<Box<str>, RelationshipDecl>,
    defects: &mut Defects<'v>,
    budget: &mut Budget<'_, '_>,
) -> Result<(FieldBearingTypeDecl, Vec<(Box<str>, RelationshipDecl)>), ValidationFailure> {
    const F: &str = SET_FORM;
    let field_rows = items(value, "fields", defects);
    let mut fields = budget.vec(field_rows.len(), F)?;
    for (index, field) in field_rows.iter().enumerate() {
        defects.context(field);
        let base = [Segment::Name("fields"), Segment::Index(index)];
        if record
            && field
                .get("typeRef")
                .and_then(View::as_str)
                .is_some_and(|name| references.scope.record_field_forbidden.contains(name))
        {
            defects.push(
                Row::Meaning,
                &joined(&base, Segment::Name("typeRef")),
                ModelRefusal::malformed(),
            );
        }
        let optional = match text(field, "presence") {
            Ok("required") => false,
            Ok("optional") => true,
            Ok(_) => {
                defects.push(
                    Row::Meaning,
                    &joined(&base, Segment::Name("presence")),
                    ModelRefusal::malformed(),
                );
                false
            }
            Err(refusal) => {
                defects.push(
                    Row::Meaning,
                    &joined(&base, Segment::Name("presence")),
                    refusal,
                );
                false
            }
        };
        let redefines = references.redefinition(field, &base, defects, budget)?;
        let slot = references.slot(field, &base, defects, budget)?;
        if optional
            && (slot.multiplicity.lower, slot.multiplicity.upper) == (1, Some(1))
            && matches!(&slot.type_ref, Some(SelectedTypeRef::Declared(name)) if references.scope.option_types.contains(name.as_ref()))
        {
            defects.push(
                Row::Meaning,
                &joined(&base, Segment::Name("presence")),
                ModelRefusal::malformed(),
            );
        }
        fields.push(FieldDecl {
            identity: identity_of(field, node, &base, defects, budget)?,
            slot,
            optional,
            redefines,
        });
    }
    defects.context(value);
    let operation_rows = items(value, "operations", defects);
    let mut operations = budget.vec(operation_rows.len(), F)?;
    for (index, operation) in operation_rows.iter().enumerate() {
        defects.context(operation);
        let base = [Segment::Name("operations"), Segment::Index(index)];
        let operation_identity = identity_of(operation, node, &base, defects, budget)?;
        let params_path = joined(&base, Segment::Name("params"));
        let declared = items_at(operation, "params", &params_path, defects);
        let mut parameters = budget.vec(declared.len(), F)?;
        // The positions of the parameters that share an identity with an
        // earlier one, and that earlier one.
        let mut seen: SortedMap<&str, usize> = SortedMap::with_capacity(budget, declared.len(), F)?;
        let mut colliding: SortedSet<usize> = SortedSet::with_capacity(budget, declared.len(), F)?;
        for (position, parameter) in declared.iter().enumerate() {
            if let Some(identity) = parameter.get("identity").and_then(View::as_str) {
                if let Some(&first) = seen.get(identity) {
                    colliding.insert(budget, first, F)?;
                    colliding.insert(budget, position, F)?;
                    defects.context(parameter);
                    defects.push(
                        Row::ParameterDuplicate,
                        &joined(&params_path, Segment::Index(position)),
                        ModelRefusal::new(
                            CheckedPackageRefusalCode::InvalidModelBinding,
                            CheckedPackageRefusalCause::ConflictingBinding,
                        ),
                    );
                } else {
                    seen.insert(budget, identity, position, F)?;
                }
            }
        }
        drop(seen);
        for (position, parameter) in declared.iter().enumerate() {
            defects.context(parameter);
            let at = joined(&params_path, Segment::Index(position));
            if !colliding.contains(&position) && parameter.get("identity").is_some() {
                identity_of(parameter, &operation_identity, &at, defects, budget)?;
            }
            if !colliding.contains(&position) {
                let name = parameter.get("name").and_then(View::as_str);
                let identity_name = parameter
                    .get("identity")
                    .and_then(View::as_str)
                    .and_then(|identity| identity.strip_prefix(operation_identity.as_ref()))
                    .and_then(|suffix| suffix.strip_prefix('/'));
                if identity_name.is_some()
                    && !name.is_some_and(|name| is_object_id(name) && identity_name == Some(name))
                {
                    defects.context(operation);
                    defects.push(
                        Row::ParameterName,
                        &joined(&at, Segment::Name("name")),
                        ModelRefusal::malformed(),
                    );
                }
            }
            parameters.push(references.slot(parameter, &at, defects, budget)?);
        }
        drop(colliding);
        defects.context(operation);
        let result = match operation.get("returns") {
            Some(returns) => Some(references.slot(
                returns,
                &joined(&base, Segment::Name("returns")),
                defects,
                budget,
            )?),
            None => None,
        };
        let frame = match operation.get("frame") {
            Some(raw) => match operation_frame(raw, budget)? {
                Some(frame) => frame,
                None => {
                    defects.push(
                        Row::Meaning,
                        &joined(&base, Segment::Name("frame")),
                        ModelRefusal::malformed(),
                    );
                    SourceOperationFrame::default()
                }
            },
            None => SourceOperationFrame::default(),
        };
        operations.push(OperationDecl {
            identity: operation_identity,
            parameters,
            result,
            frame,
            redefines: references.redefinition(operation, &base, defects, budget)?,
        });
    }
    defects.context(value);
    let relationship_rows = items(value, "relationships", defects);
    let mut relationships = budget.vec(relationship_rows.len(), F)?;
    let mut local_relationships: SortedSet<&str> =
        SortedSet::with_capacity(budget, relationship_rows.len(), F)?;
    for (index, relationship) in relationship_rows.iter().enumerate() {
        defects.context(relationship);
        let base = [Segment::Name("relationships"), Segment::Index(index)];
        if let Some(identity) = relationship
            .get("identity")
            .and_then(View::as_str)
            .filter(|identity| valid_relationship_identity(&references.model.identity, identity))
        {
            if known_relationships.contains_key(identity)
                || !local_relationships.insert(budget, identity, F)?
            {
                defects.push(
                    Row::Duplicate,
                    &joined(&base, Segment::Name("identity")),
                    ModelRefusal::new(
                        CheckedPackageRefusalCode::InvalidModelBinding,
                        CheckedPackageRefusalCause::ConflictingBinding,
                    ),
                );
            }
        }
        if let Some(declared) =
            semantic_ir_relationship(relationship, node, &base, references, defects, budget)?
        {
            relationships.push(declared);
        }
    }
    drop(local_relationships);
    defects.context(value);
    let supertype_rows = items(value, "supertypes", defects);
    let mut supertypes = budget.vec(supertype_rows.len(), F)?;
    for (index, supertype) in supertype_rows.iter().enumerate() {
        let path = [Segment::Name("supertypes"), Segment::Index(index)];
        match supertype.as_str() {
            Some(supertype) => {
                references.check_supertype(supertype, meaning, &path, defects);
                supertypes.push(budget.text(supertype, F)?);
            }
            None => defects.push(Row::Meaning, &path, ModelRefusal::malformed()),
        }
    }
    Ok((
        FieldBearingTypeDecl {
            interface,
            supertypes,
            fields,
            operations,
        },
        relationships,
    ))
}

/// A member list nested at `path`, or the failure of one that is not a list.
fn items_at<'v>(
    value: View<'v>,
    member: &'static str,
    path: &[Segment],
    defects: &mut Defects,
) -> List<'v> {
    list(value, member).unwrap_or_else(|refusal| {
        defects.push(Row::Meaning, path, refusal);
        List::empty()
    })
}

/// A fixture value as the view the reader takes, kept alive for the test run.
#[cfg(test)]
pub(in crate::checked_package::v2) fn view_of(value: &serde_json::Value) -> View<'static> {
    let bytes = serde_json::to_vec(value).expect("encode fixture");
    let document = quire_canonical::read(&bytes, 1 << 30).expect("read fixture");
    View::root(Box::leak(Box::new(document)))
}

#[cfg(test)]
pub(super) mod tests;

#[cfg(test)]
mod structural_member_type_tests {
    use super::{declaration_id, CollectionKind, DeclarationForm, IntegerBounds, MemberType};
    use serde_json::json;

    #[ix_trace_rs::trace("TC-472", "FR-038-AC-504")]
    #[test]
    fn member_type_identity_allocates_only_reserved_storage() {
        use super::{Budget, RationalBounds, TextBounds};
        use crate::checked_package::v2::{native_join_budget::NativeJoinBudget, WorkMeter};
        let target = declaration_id("model", DeclarationForm::ObjectType, "Thing");
        let bounds = RationalBounds {
            numerator_min: i128::MIN,
            numerator_max: i128::MAX,
            denominator_min: 1,
            denominator_max: 99,
        };
        let cases = [
            MemberType::Integer,
            MemberType::Reference(target.clone()),
            MemberType::Record(target.clone()),
            MemberType::IntRange(IntegerBounds {
                lower: i128::MIN,
                upper: i128::MAX,
            }),
            MemberType::RationalRange(bounds),
            MemberType::CarriedRational(bounds, target),
            MemberType::TextBounds(TextBounds {
                lower: 0,
                upper: u64::MAX,
                profile: "nfc".into(),
            }),
            MemberType::Option(Box::new(MemberType::Collection {
                kind: CollectionKind::Set,
                element: Box::new(MemberType::Boolean),
                bounds: Some((0, u64::MAX)),
            })),
        ];
        for member in cases {
            let expected = member.node_id("checked");
            let run = |limit| {
                let mut native = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
                let mut result = None;
                let measured = allocation_counter::measure(|| {
                    let mut meter = WorkMeter::over(&mut native);
                    let mut budget = Budget::new(&mut meter, 0, u64::MAX);
                    result = Some(member.node_id_in("checked", &mut budget));
                });
                (
                    result.expect("ran"),
                    measured.bytes_total,
                    native.read_storage_consumed(),
                )
            };
            let (actual, allocated, charged) = run(u64::MAX);
            assert_eq!(actual.expect("unlimited"), expected);
            assert!(charged > 0);
            assert_eq!(allocated, charged, "{member:?}");
            assert_eq!(run(charged).0.expect("exact limit"), expected);
            assert!(run(charged - 1).0.is_err(), "one short: {member:?}");
        }
    }

    /// Trace: FR-038-AC-364, FR-322-AC-62.
    #[test]
    fn selected_member_types_use_structural_paths() {
        let object = declaration_id(
            "acme/orders",
            DeclarationForm::ObjectType,
            "ix://acme/orders/Order",
        );
        let reference = MemberType::Reference(object.clone())
            .node_id("example")
            .expect("reference type");
        assert_eq!(
            serde_json::to_value(&reference).expect("encode reference"),
            json!({"package":"example","path":["composite_type","reference",
                {"package":"acme/orders","path":["model","object_type","ix://acme/orders/Order"],"ordinal":0}
            ],"ordinal":0})
        );
        let bounded = MemberType::Collection {
            kind: CollectionKind::Sequence,
            element: Box::new(MemberType::Reference(object)),
            bounds: Some((0, 3)),
        }
        .node_id("example")
        .expect("bounded sequence");
        assert_eq!(bounded.path.len(), 8);
        assert_eq!(
            bounded.path[0],
            super::CheckedNodePathSegment::Name("bounded_domain".into())
        );
        let integer_range = MemberType::IntRange(IntegerBounds {
            lower: -9,
            upper: 9,
        })
        .node_id("example")
        .expect("integer range");
        assert_eq!(
            serde_json::to_value(&integer_range).expect("encode range")["path"],
            json!(["bounded_domain","integer_range","min",
                {"package":"example","path":["scalar_type","integer"],"ordinal":0},
                "-9","max","9"])
        );
    }

    /// Trace: FR-069-AC-8, FR-322-AC-380.
    #[test]
    fn selected_rational_bounds_require_four_integer_carriers_and_name_the_rational_shape() {
        let declared = |value: serde_json::Value| {
            json!({"identity":"ix://example/Ratio",
            "scalar":"rational","constraints":value})
        };
        let constraints = |nmin: serde_json::Value| {
            json!([
                {"keyword":"minNumerator","appliesTo":"ix://example/Ratio","operands":{"value":nmin}},
                {"keyword":"maxNumerator","appliesTo":"ix://example/Ratio","operands":{"value":7}},
                {"keyword":"minDenominator","appliesTo":"ix://example/Ratio","operands":{"value":1}},
                {"keyword":"maxDenominator","appliesTo":"ix://example/Ratio","operands":{"value":12}}
            ])
        };
        let unit_refs = crate::checked_package::v2::model_members::UnitRefs::default();
        let value = super::semantic_ir_value_type(
            super::view_of(&declared(constraints(json!(-3)))),
            &unit_refs,
        )
        .expect("bounded Rational");
        let super::ValueBounds::Rational(bounds, None) = value else {
            panic!("Rational value type must retain Rational bounds");
        };
        let id = MemberType::RationalRange(bounds)
            .node_id("example")
            .expect("rational range");
        assert_eq!(
            serde_json::to_value(&id).expect("encode rational range")["path"],
            json!(["bounded_domain","rational_range","numerator_min",
                {"package":"example","path":["scalar_type","integer"],"ordinal":0},
                "-3","numerator_max","7","denominator_min","1","denominator_max","12"])
        );
        let unit = super::derived_id("example", "scalar_type", "unit", Vec::new());
        let mut refs = std::collections::BTreeMap::new();
        refs.insert("ix://quire/unit/m".into(), unit.clone());
        let refs = super::tests::unit_refs_of(refs);
        let mut with_unit = declared(constraints(json!(-3)));
        with_unit["unit"] = json!("ix://quire/unit/m");
        let super::ValueBounds::Rational(carried_bounds, Some(actual_unit)) =
            super::semantic_ir_value_type(super::view_of(&with_unit), &refs)
                .expect("unit-bearing Rational")
        else {
            panic!("unit-bearing Rational must retain its unit");
        };
        assert_eq!(actual_unit, unit);
        let carried = MemberType::CarriedRational(carried_bounds, actual_unit)
            .node_id("example")
            .expect("carried Rational");
        assert_eq!(
            serde_json::to_value(carried).expect("encode carried rational")["path"],
            json!(["scalar_type", "carried_quantity", unit, id, "rounding",
                {"package":"example","path":["scalar_type","text"],"ordinal":0}, "exact"])
        );
        for bad in [json!(-3.5), json!(null)] {
            assert!(
                super::semantic_ir_value_type(
                    super::view_of(&declared(constraints(bad.clone()))),
                    &unit_refs
                )
                .is_none(),
                "wrong bound carrier must not become a bounded type"
            );
            let (refusal, path) =
                super::rational_value_type(super::view_of(&declared(constraints(bad))), &unit_refs)
                    .expect_err("wrong carrier must refuse before tuple or payload admission");
            assert_eq!(refusal, super::ModelRefusal::malformed());
            assert_eq!(path, super::RationalPath::constraint(0, "operands"));
        }
        assert!(
            super::semantic_ir_value_type(super::view_of(&declared(json!([]))), &unit_refs)
                .is_none(),
            "unbounded Rational must remain unmapped"
        );
    }

    /// Trace: FR-038-AC-487, FR-322-AC-380, TC-981 (RV-01..RV-03).
    #[test]
    fn selected_rational_bounds_preserve_canonical_signed_64_values() {
        use super::{
            document::admit_document, Budget, CheckedPackageRefusalCause, ModelRefusal,
            RationalValueError, SelectionFailure, UnitRefs, ValueBounds, View, WorkMeter,
        };

        let selected = |lower: serde_json::Value, upper: serde_json::Value| {
            json!({
                "package":{"identity":"test/orders"},
                "types":[{"identity":"ix://test/orders/Ratio","scalar":"rational",
                    "constraints":[
                        {"keyword":"minNumerator","appliesTo":"ix://test/orders/Ratio","operands":{"value":lower}},
                        {"keyword":"maxNumerator","appliesTo":"ix://test/orders/Ratio","operands":{"value":upper}},
                        {"keyword":"minDenominator","appliesTo":"ix://test/orders/Ratio","operands":{"value":1}},
                        {"keyword":"maxDenominator","appliesTo":"ix://test/orders/Ratio","operands":{"value":12}}
                    ]}]
            })
        };
        let read = |document: &serde_json::Value| {
            let bytes = serde_json::to_vec(document).expect("selected document");
            let mut meter = WorkMeter::new(u64::MAX);
            let mut budget = Budget::new(&mut meter, 0, u64::MAX);
            admit_document("test/orders", Some(&bytes), &mut budget)
        };
        let unit_refs = UnitRefs::default();
        for (lower, upper, expected_lower, expected_upper) in [
            (
                json!(-9_007_199_254_740_992_i64),
                json!(9_007_199_254_740_992_i64),
                -9_007_199_254_740_992_i128,
                9_007_199_254_740_992_i128,
            ),
            (
                json!("-9223372036854775808"),
                json!("9223372036854775807"),
                i128::from(i64::MIN),
                i128::from(i64::MAX),
            ),
            (json!("-3"), json!("7"), -3, 7),
        ] {
            let document = read(&selected(lower, upper)).expect("canonical selected document");
            let Some(ValueBounds::Rational(bounds, None)) = super::rational_value_type(
                View::root(&document)
                    .get("types")
                    .and_then(View::as_array)
                    .and_then(|types| types.iter().next())
                    .expect("first type"),
                &unit_refs,
            )
            .expect("selected Rational bounds") else {
                panic!("selected Rational bounds must remain exact");
            };
            assert_eq!(
                (bounds.numerator_min, bounds.numerator_max),
                (expected_lower, expected_upper)
            );
            let id = super::MemberType::RationalRange(bounds)
                .node_id("test/orders")
                .expect("rational range node");
            let path = serde_json::to_value(id).expect("structural node key")["path"]
                .as_array()
                .expect("rational range path")
                .clone();
            assert_eq!(path[4], json!(expected_lower.to_string()));
            assert_eq!(path[6], json!(expected_upper.to_string()));
        }

        for bad in [
            "01",
            "+1",
            "-0",
            "3/4",
            "9223372036854775808",
            "-9223372036854775809",
        ] {
            let document =
                read(&selected(json!(-3), json!(bad))).expect("well-shaped selected document");
            let (refusal, path): RationalValueError = super::rational_value_type(
                View::root(&document)
                    .get("types")
                    .and_then(View::as_array)
                    .and_then(|types| types.iter().next())
                    .expect("first type"),
                &unit_refs,
            )
            .expect_err("non-canonical or out-of-range decimal refuses");
            assert_eq!(refusal, ModelRefusal::malformed(), "{bad}");
            assert_eq!(
                path,
                super::RationalPath::constraint(1, "operands"),
                "{bad}"
            );
        }
        for (constraint, unsafe_integer) in [
            (1, 9_007_199_254_740_993_i64),
            (1, 9_007_199_254_740_994_i64),
            (0, -9_007_199_254_740_994_i64),
        ] {
            let (lower, upper) = if constraint == 0 {
                (json!(unsafe_integer), json!(7))
            } else {
                (json!(-3), json!(unsafe_integer))
            };
            let result = read(&selected(lower, upper));
            let Err(SelectionFailure::InexactNumber {
                document_pointer,
                cause,
            }) = result
            else {
                panic!("unsafe JSON integer must refuse before selected-model typing: {result:?}");
            };
            assert_eq!(cause, CheckedPackageRefusalCause::InexactInteger);
            assert_eq!(
                document_pointer.to_string(),
                format!("/types/0/constraints/{constraint}/operands/value")
            );
        }
    }
}
