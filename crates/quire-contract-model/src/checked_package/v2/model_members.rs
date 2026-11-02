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
    CheckedNodeOwner, CheckedNodeTag, CheckedSemanticNodeV2, CheckedTerm, ValidationFailure,
    WorkMeter,
};
use crate::checked_package::common::{pointer_from_steps, Step};
use crate::checked_package::evidence::CheckedPackageEvidence;
use crate::checked_package::shared::{
    CheckedNodeId, CheckedNodePathSegment, CheckedPackageLimit, CheckedPackageRefusalCause,
    CheckedPackageRefusalCode, JsonPointer,
};
use ix_trace_rs::string_edge;
use serde::Deserialize;
use serde_json::{json, Value};

/// The fixed declaration body shape in raw intake and admitted typed nodes.
pub(super) trait EmptyAggregateBody {
    fn is_empty_aggregate(&self) -> bool;
}

impl EmptyAggregateBody for Value {
    fn is_empty_aggregate(&self) -> bool {
        *self == json!({"term": "aggregate", "members": []})
    }
}

impl EmptyAggregateBody for CheckedTerm {
    fn is_empty_aggregate(&self) -> bool {
        matches!(self, Self::Aggregate { members } if members.is_empty())
    }
}
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

pub(super) type UnitRefs = Arc<BTreeMap<Box<str>, CheckedNodeId>>;
pub(super) const DURATION_RESOLVED_PREFIX: &str = "ix://quire/resolved-duration/";

mod intake_origin;
mod option_types;
use intake_origin::{retained_origin, IntakeDeclarationOrigin, OriginView};

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
        if name.starts_with("ix://quire/unit/") {
            return Some(Self::Unit(name.into()));
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

/// Shallow Semantic IR operation-frame members; the first three are required
/// when `frame` is present, while an absent frame is intentionally empty.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct SourceOperationFrame {
    pub(super) modifies: Vec<Box<str>>,
    pub(super) creates: Vec<Box<str>>,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum ValueBounds {
    Integer(IntegerBounds),
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
    pub(super) object_types: BTreeMap<Box<str>, FieldBearingTypeDecl>,
    /// State machine declarations share field and operation parsing with object types.
    pub(super) state_machines: BTreeSet<Box<str>>,
    /// Process declarations share field and operation parsing with object types.
    pub(super) processes: BTreeSet<Box<str>>,
    /// Persistence-interface declarations have a selected owner key.
    pub(super) persistence_interfaces: BTreeSet<Box<str>>,
    /// Canonical admitted persistence operation declarations, separate from object types.
    pub(super) persistence_operations: BTreeMap<Box<str>, FieldBearingTypeDecl>,
    /// Record value types, with their own and inherited fields.
    pub(super) record_types: BTreeMap<Box<str>, FieldBearingTypeDecl>,
    /// Ordered positions of each selected tuple value type.
    pub(super) tuple_types: BTreeMap<Box<str>, Vec<SelectedTypeRef>>,
    /// Value types, by IR node identity; `None` when not bound as `Int[lo, hi]`.
    pub(super) value_types: BTreeMap<Box<str>, Option<ValueBounds>>,
    /// Core Semantic IR aliases, resolved before model member type mapping.
    aliases: BTreeMap<Box<str>, Box<str>>,
    /// Exact semantic cases, original identities, and authored positions.
    pub(super) variant_types: BTreeMap<Box<str>, BTreeMap<Box<str>, SelectedVariantCase>>,
    /// Selected FCD dimension declarations by their exact identity.
    pub(super) dimensions: BTreeMap<Box<str>, SelectedDimension>,
    /// Selected FCD unit declarations by their exact identity.
    pub(super) units: BTreeMap<Box<str>, SelectedUnit>,
    /// Selected sequence, map, option and reference value definitions.
    pub(super) collection_types: BTreeMap<Box<str>, SelectedTypeDefinition>,
    /// Checked-package unit `typeRef` to the exact admitted unit node ID.
    pub(super) unit_refs: UnitRefs,
    /// Whether this selection declares its own `s`; such a declaration
    /// prevents a Duration member from falling back to the source unit.
    own_seconds_declared: bool,
    /// Relationship declarations, by IR node identity.
    pub(super) relationships: BTreeMap<Box<str>, RelationshipDecl>,
    /// Declared populations, by IR node identity in the selected document.
    pub(super) populations: BTreeSet<Box<str>>,
    /// Object type identities covered by each selected population.
    pub(super) population_members: BTreeMap<Box<str>, BTreeSet<Box<str>>>,
    /// Systems connection declarations that also back relation nodes.
    pub(super) relation_nodes: BTreeSet<Box<str>>,
    /// Types of every other FR-208 meaning, by IR node identity.
    pub(super) other_types: BTreeSet<Box<str>>,
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
pub(super) struct Budget<'w> {
    meter: &'w mut WorkMeter,
    /// Node position of the selected package's `model_root`.
    root: usize,
    /// The reader's byte limit: the ceiling of every encode the row's
    /// document and declarations reach.
    bytes: u64,
}

impl<'w> Budget<'w> {
    pub(super) fn new(meter: &'w mut WorkMeter, root: usize, bytes: u64) -> Self {
        Self { meter, root, bytes }
    }

    pub(super) const fn work(&self) -> u64 {
        self.meter.consumed()
    }

    /// The pointer of the root owner this budget is charged at.
    pub(super) fn row(&self) -> JsonPointer {
        crate::checked_package::common::node_pointer(self.root).key("owner")
    }

    pub(super) fn charge(&mut self, work: u64) -> Result<(), ValidationFailure> {
        let row = self.row();
        self.meter.charge(work, || row)
    }
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
    fn selected_type_ref(&self, name: &str) -> Option<SelectedTypeRef> {
        let mut current: Box<str> = name.into();
        let mut seen = BTreeSet::new();
        while let Some(target) = self.aliases.get(current.as_ref()) {
            if !seen.insert(current.clone()) {
                return None;
            }
            current = target.clone();
        }
        SelectedTypeRef::from_admitted_name(&current)
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
        budget: &mut Budget<'_>,
    ) -> Result<BTreeSet<&str>, ValidationFailure> {
        #[cfg(test)]
        probe::walked();
        let mut reached = BTreeSet::new();
        let mut pending: Vec<&str> = self
            .field_owner(node)
            .map(|declared| declared.supertypes.iter().map(AsRef::as_ref).collect())
            .unwrap_or_default();
        while let Some(supertype) = pending.pop() {
            budget.charge(1)?;
            if reached.insert(supertype) {
                if let Some(declared) = self.field_owner(supertype) {
                    pending.extend(declared.supertypes.iter().map(AsRef::as_ref));
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
        budget: &mut Budget<'_>,
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
        budget: &mut Budget<'_>,
    ) -> Result<bool, ValidationFailure> {
        Ok(sub == sup || self.ancestors(sub, budget)?.contains(sup))
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
        budget: &mut Budget<'_>,
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
        let mut members: BTreeMap<&str, Resolved<'_>> = BTreeMap::new();
        let mut hidden: BTreeSet<&str> = BTreeSet::new();
        // redefined target -> (redefining member, redefining owner)
        let mut redefiners: BTreeMap<&str, Vec<(&str, &str)>> = BTreeMap::new();
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
                    hidden.insert(target);
                    redefiners
                        .entry(target)
                        .or_default()
                        .push((identity, owner));
                }
                if member_kind == kind {
                    members.insert(identity, member);
                }
            }
        }
        // The most-derived-redefiner rule: a redefiner of a target that a
        // redefiner in a more derived owner also redefines is hidden.
        let mut ancestry: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
        for group in redefiners.values() {
            for (identity, owner) in group {
                for (_, other_owner) in group {
                    budget.charge(1)?;
                    if !ancestry.contains_key(other_owner) {
                        ancestry.insert(other_owner, self.ancestors(other_owner, budget)?);
                    }
                    if ancestry
                        .get(other_owner)
                        .is_some_and(|of_other| of_other.contains(owner))
                    {
                        hidden.insert(identity);
                    }
                }
            }
        }
        let mut matches = members
            .into_iter()
            .filter(|(identity, _)| !hidden.contains(identity) && member_name(identity) == name)
            .map(|(_, member)| member);
        match (matches.next(), matches.next()) {
            (Some(member), None) => Ok(member),
            (Some(_), Some(_)) => Err(ModelRefusal::ambiguous().into()),
            (None, _) => Err(ModelRefusal::ineligible().into()),
        }
    }

    /// FR-322 step 4's element type of a declared `typeRef`.
    pub(super) fn element_type(&self, type_ref: &SelectedTypeRef) -> Option<MemberType> {
        let mut current = type_ref;
        let mut seen = BTreeSet::new();
        let mut layers = Vec::new();
        while let SelectedTypeRef::Declared(name) = current {
            let Some(definition) = self.collection_types.get(name) else {
                break;
            };
            if !seen.insert(name.clone()) {
                return None;
            }
            match definition {
                SelectedTypeDefinition::Sequence(items) => {
                    layers.push(SelectedValueLayer::Sequence);
                    current = items;
                }
                SelectedTypeDefinition::Option(inner) => {
                    layers.push(SelectedValueLayer::Option);
                    current = inner;
                }
                SelectedTypeDefinition::Reference(target) => {
                    current = target;
                }
                SelectedTypeDefinition::Map(_) => return None,
            }
        }
        let mut element = self.element_type_leaf(current)?;
        for layer in layers.into_iter().rev() {
            element = match layer {
                SelectedValueLayer::Option => MemberType::Option(Box::new(element)),
                SelectedValueLayer::Sequence => MemberType::Collection {
                    kind: CollectionKind::Sequence,
                    element: Box::new(element),
                    bounds: None,
                },
                SelectedValueLayer::Map => return None,
            };
        }
        Some(element)
    }

    /// A Duration member has no type when its chosen owner declares no `s`
    /// node. Distinguish that missing name from a present but invalid `s`.
    pub(super) fn duration_unit_absent(&self, type_ref: &SelectedTypeRef) -> bool {
        let mut current = type_ref;
        let mut seen = BTreeSet::new();
        while let SelectedTypeRef::Declared(name) = current {
            if !seen.insert(name) {
                return false;
            }
            match self.collection_types.get(name) {
                Some(
                    SelectedTypeDefinition::Sequence(items) | SelectedTypeDefinition::Option(items),
                ) => current = items,
                _ => break,
            }
        }
        if !matches!(current, SelectedTypeRef::Native(NativeTypeRef::Duration)) {
            return false;
        }
        let unit_name = if self.own_seconds_declared {
            format!("ix://quire/unit/{}/s", self.identity)
        } else {
            "ix://quire/unit/s".to_owned()
        };
        !self.unit_refs.contains_key(unit_name.as_str())
    }

    fn element_type_leaf(&self, type_ref: &SelectedTypeRef) -> Option<MemberType> {
        let type_ref = match type_ref {
            SelectedTypeRef::Map(_, _) => return None,
            SelectedTypeRef::Unit(name) => {
                return self.unit_refs.get(name).cloned().map(MemberType::Quantity);
            }
            SelectedTypeRef::Native(native) => {
                return match native {
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
                        let selected = format!("ix://quire/unit/{}/s", self.identity);
                        let owner_unit = if self.own_seconds_declared {
                            selected
                        } else {
                            "ix://quire/unit/s".to_owned()
                        };
                        self.unit_refs
                            .get(format!("{DURATION_RESOLVED_PREFIX}{owner_unit}").as_str())
                            .cloned()
                            .map(MemberType::Quantity)
                    }
                    NativeTypeRef::Rational | NativeTypeRef::Decimal | NativeTypeRef::Text => None,
                };
            }
            SelectedTypeRef::Declared(name) => name.as_ref(),
        };
        if let Some(bounds) = self.value_types.get(type_ref) {
            return bounds.as_ref().map(|bounds| match bounds {
                ValueBounds::Integer(range) => MemberType::IntRange(*range),
                ValueBounds::Text(bounds) => MemberType::TextBounds(bounds.clone()),
            });
        }
        if self.record_types.contains_key(type_ref) {
            return Some(MemberType::Record(declaration_id(
                &self.identity,
                DeclarationForm::RecordValueType,
                type_ref,
            )));
        }
        if self.tuple_types.contains_key(type_ref) {
            return Some(MemberType::Tuple(declaration_id(
                &self.identity,
                DeclarationForm::TupleValueType,
                type_ref,
            )));
        }
        if self.variant_types.contains_key(type_ref) {
            return Some(MemberType::Variant(declaration_id(
                &self.identity,
                DeclarationForm::VariantType,
                type_ref,
            )));
        }
        let object = self.object_types.get(type_ref)?;
        if object.interface {
            return None;
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
        Some(MemberType::Reference(declaration_id(
            &self.identity,
            form,
            type_ref,
        )))
    }

    /// Render the selected field type used by a timed clock-place path.
    pub(super) fn clock_type_name(&self, type_ref: &SelectedTypeRef) -> Option<String> {
        let mut current = type_ref;
        let mut seen = BTreeSet::new();
        let mut layers = Vec::new();
        while let SelectedTypeRef::Declared(name) = current {
            let Some(definition) = self.collection_types.get(name) else {
                break;
            };
            if !seen.insert(name.clone()) {
                return None;
            }
            match definition {
                SelectedTypeDefinition::Sequence(items) => {
                    layers.push(SelectedValueLayer::Sequence);
                    current = items;
                }
                SelectedTypeDefinition::Map(values) => {
                    layers.push(SelectedValueLayer::Map);
                    current = values;
                }
                SelectedTypeDefinition::Option(_) | SelectedTypeDefinition::Reference(_) => {
                    return None
                }
            }
        }
        let mut rendered = match current {
            SelectedTypeRef::Native(NativeTypeRef::Clock) => {
                Some("ix://quire/native/Clock".to_owned())
            }
            SelectedTypeRef::Native(NativeTypeRef::Text) => {
                Some("ix://quire/native/Text".to_owned())
            }
            SelectedTypeRef::Native(_) => None,
            SelectedTypeRef::Unit(name) => Some(name.to_string()),
            SelectedTypeRef::Map(key, value) => Some(format!("Map<{key}, {value}>")),
            SelectedTypeRef::Declared(name) => Some(name.to_string()),
        }?;
        for layer in layers.into_iter().rev() {
            rendered = match layer {
                SelectedValueLayer::Sequence => format!("Sequence<{rendered}>"),
                SelectedValueLayer::Map => format!("Map<Text, {rendered}>"),
                SelectedValueLayer::Option => return None,
            };
        }
        Some(rendered)
    }

    /// FR-322 step 4 over one declared `typeRef` and multiplicity; `None`
    /// when the tables give the slot no type.
    pub(super) fn slot_type(&self, slot: &TypedSlot) -> Option<MemberType> {
        let element = self.element_type(slot.type_ref.as_ref()?)?;
        let multiplicity = slot.multiplicity;
        if (multiplicity.lower, multiplicity.upper) == (1, Some(1)) {
            return Some(element);
        }
        let kind = CollectionKind::of(multiplicity.ordered, multiplicity.unique);
        let bounds = match (multiplicity.lower, multiplicity.upper) {
            (0, None) => None,
            (_, None) => return None,
            (lower, Some(upper)) => Some((lower, upper)),
        };
        Some(MemberType::Collection {
            kind,
            element: Box::new(element),
            bounds,
        })
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
        budget: &mut Budget<'_>,
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
        budget: &mut Budget<'_>,
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
        let object_id = declaration_id(&self.identity, form, object);
        let Some(receiver) = self.type_node_id(&MemberType::Reference(object_id)) else {
            return Ok(None);
        };
        let mut types = vec![receiver];
        for parameter in &operation.parameters {
            let Some(parameter_type) = self
                .slot_type(parameter)
                .and_then(|ty| self.type_node_id(&ty))
            else {
                return Ok(None);
            };
            types.push(parameter_type);
        }
        if let Some(result) = &operation.result {
            let Some(result_type) = self.slot_type(result).and_then(|ty| self.type_node_id(&ty))
            else {
                return Ok(None);
            };
            types.push(result_type);
        }
        Ok(Some(types))
    }

    /// Candidate names, including ambiguous names, before effective-member resolution.
    pub(super) fn operation_candidate_names(
        &self,
        object: &str,
        budget: &mut Budget<'_>,
    ) -> Result<BTreeSet<Box<str>>, ValidationFailure> {
        let mut candidates = BTreeSet::new();
        let ancestors = self.ancestors(object, budget)?;
        for node in std::iter::once(object).chain(ancestors.iter().copied()) {
            if let Some(declared) = self.object_types.get(node) {
                budget.charge(units(declared.operations.len()))?;
                candidates.extend(
                    declared
                        .operations
                        .iter()
                        .map(|operation| Box::from(member_name(&operation.identity))),
                );
            }
        }
        Ok(candidates)
    }

    /// Names of the effective operations reachable from one model type.
    pub(super) fn effective_operation_names(
        &self,
        object: &str,
        budget: &mut Budget<'_>,
    ) -> Result<BTreeSet<Box<str>>, ValidationFailure> {
        let candidates = self.operation_candidate_names(object, budget)?;
        let mut names = BTreeSet::new();
        for name in candidates {
            match self.resolve(object, MemberKind::Operation, &name, budget) {
                Ok(Resolved::Operation(_)) => {
                    names.insert(name);
                }
                Ok(Resolved::Field(_)) | Err(ModelFailure::Refused(_)) => {}
                Err(ModelFailure::Limit(failure)) => return Err(failure),
            }
        }
        Ok(names)
    }

    fn type_node_id(&self, member_type: &MemberType) -> Option<CheckedNodeId> {
        member_type.node_id(&self.checked_package)
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

/// Selected package, declaration, and form to the authored structural node.
type DeclarationOwners =
    BTreeMap<Box<str>, BTreeMap<Box<str>, BTreeMap<DeclarationForm, CheckedNodeId>>>;

/// Every selected declaration's model declaration node key.
#[derive(Debug, Default)]
pub(super) struct ModelOwners<'m> {
    by_key: BTreeMap<CheckedNodeId, Owner<'m>>,
    by_declaration: DeclarationOwners,
    /// The identity of every selected domain package.
    identities: BTreeSet<&'m str>,
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
        self.identities.contains(identity)
    }

    pub(super) fn owner_by_key(&self, key: &CheckedNodeId) -> Option<Owner<'m>> {
        self.by_key.get(key).copied()
    }

    /// The exact admitted node for a selected model declaration.
    pub(super) fn declaration_id(
        &self,
        identity: &str,
        node: &str,
        form: DeclarationForm,
    ) -> Option<&CheckedNodeId> {
        self.by_declaration.get(identity)?.get(node)?.get(&form)
    }

    /// Effective operation names, declaration nodes and checked binder records
    /// for one selected domain package. A model member has no node of its own.
    pub(super) fn operation_classes(
        &self,
        identity: &str,
        meter: &mut super::WorkMeter,
    ) -> Result<Vec<ModelOperationClass>, ValidationFailure> {
        let mut classes = Vec::new();
        for (id, owner) in &self.by_key {
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
                classes.push(ModelOperationClass {
                    object: member_name(owner.node).into(),
                    operation: name,
                    node: id.clone(),
                    binder_types: binder,
                });
            }
        }
        Ok(classes)
    }
    pub(super) fn refinement_object_fields(
        &self,
        key: &crate::checked_package::shared::CheckedNodeId,
    ) -> Option<BTreeSet<Box<str>>> {
        let owner = self.by_key.get(key)?;
        if owner.form != DeclarationForm::ObjectType {
            return None;
        }
        owner.package.fields.exposed_names(owner.node)
    }
    /// Recomputes the key of every selected model declaration
    /// of each admitted document; `charge` is called once per key.
    pub(super) fn new<Body>(
        models: &'m [DomainModel],
        nodes: &[CheckedSemanticNodeV2<Body>],
        mut charge: impl FnMut(usize) -> Result<(), ValidationFailure>,
    ) -> Result<Self, ValidationFailure> {
        let mut by_key = BTreeMap::new();
        let mut by_declaration = DeclarationOwners::new();
        let mut identities = BTreeSet::new();
        let mut authored = BTreeMap::<(&str, &str), Vec<&CheckedSemanticNodeV2<Body>>>::new();
        for candidate in nodes {
            if let Some(CheckedNodeOwner::Model { identity, node }) = &candidate.owner {
                authored
                    .entry((identity.as_ref(), node.as_ref()))
                    .or_default()
                    .push(candidate);
            }
        }
        for (index, package) in models.iter().enumerate() {
            identities.insert(package.identity.as_ref());
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
                charge(package.root)?;
                if let Some(candidates) = authored.get(&(package.identity.as_ref(), node.as_ref()))
                {
                    for candidate in candidates {
                        if candidate.node_tag.as_ref() == form.tag()
                            && candidate.semantic_form.as_ref() == form.form()
                        {
                            by_declaration
                                .entry(package.identity.clone())
                                .or_default()
                                .entry(node.clone())
                                .or_default()
                                .insert(form, candidate.node_id.clone());
                            by_key.insert(
                                candidate.node_id.clone(),
                                Owner {
                                    package,
                                    form,
                                    node,
                                    selection: index,
                                },
                            );
                        }
                    }
                }
            }
        }
        Ok(Self {
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
            && (node.declaration.is_none() || self.by_key.contains_key(&node.node_id))
    }

    /// FR-322 step 2 over one wire node.
    pub(super) fn recover<Body: EmptyAggregateBody>(
        &self,
        node: &CheckedSemanticNodeV2<Body>,
    ) -> Result<Owner<'m>, ModelRefusal> {
        let owner = self
            .by_key
            .get(&node.node_id)
            .copied()
            .ok_or_else(ModelRefusal::unselected)?;
        check_declaration_node(owner.form, node, node.body.is_empty_aggregate())?;
        Ok(owner)
    }

    /// Implements: FR-347. the record value type whose model declaration node key is
    /// `key`, `None` when `key` names no selected record.
    pub(super) fn record_by_key(&self, key: &CheckedNodeId) -> Option<Owner<'m>> {
        self.by_key
            .get(key)
            .copied()
            .filter(|owner| owner.form == DeclarationForm::RecordValueType)
    }

    /// An owned copy of the key index, which an admitted package retains
    /// after the documents it borrows are gone.
    pub(super) fn index(&self) -> DeclarationIndex {
        DeclarationIndex(
            self.by_key
                .iter()
                .map(|(key, owner)| {
                    (
                        key.clone(),
                        DeclaredNode {
                            form: owner.form,
                            selection: owner.selection,
                            node: owner.node.into(),
                        },
                    )
                })
                .collect(),
        )
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
    pub(super) declaration_path: Option<Vec<Segment>>,
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

    fn located(
        refusal: ModelRefusal,
        node: &Value,
        budget: &mut Budget<'_>,
    ) -> Result<Self, ValidationFailure> {
        let identity = node.get("identity").and_then(Value::as_str);
        budget.charge(1_u64.saturating_add(identity.map_or(0, |value| units(value.len()))))?;
        Ok(Self {
            refusal,
            member: None,
            declaration_identity: identity.map(Box::from),
            declaration_origin: retained_origin(node.get("origin"), budget)?,
            declaration_path: None,
            document_pointer: None,
        })
    }

    fn group(
        refusal: ModelRefusal,
        nodes: &[&Value],
        budget: &mut Budget<'_>,
    ) -> Result<Self, ValidationFailure> {
        let Some(first) = nodes.first() else {
            return Ok(Self::at(refusal.code, refusal.cause, None));
        };
        let mut retained = Self::located(refusal, first, budget)?;
        let mut agrees = retained.declaration_origin.is_some();
        for node in nodes.iter().skip(1) {
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
    case: &Value,
    type_index: usize,
    case_index: usize,
    budget: &mut Budget<'_>,
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
    node: &Value,
    type_index: usize,
    position_index: Option<usize>,
    budget: &mut Budget<'_>,
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

/// FR-322 step 1's admission checks over the document supplied under the
/// selected identity: it parses, holds no number without an exact RFC 8785
/// spelling, and carries the selected identity as its own `package.identity`.
/// Parsing is charged to the reader's `work` limit before it starts, one unit
/// per [`DOCUMENT_BYTES_PER_WORK`] bytes, so a document too large for the
/// limit is `incomplete` at its root and is never parsed. Returns the parsed
/// document.
pub(super) fn admit_document(
    identity: &str,
    supplied: Option<&[u8]>,
    identity_of: fn(&Value) -> Option<&str>,
    budget: &mut Budget<'_>,
) -> Result<Value, SelectionFailure> {
    use CheckedPackageRefusalCause as Cause;
    use CheckedPackageRefusalCode as Code;
    let Some(bytes) = supplied else {
        return Err(SelectionRefusal::at(
            Code::MissingImport,
            Cause::MissingSelection,
            Some("identity"),
        )
        .into());
    };
    budget.charge(units(bytes.len().div_ceil(DOCUMENT_BYTES_PER_WORK)))?;
    // The document is read once, by `quire-canonical`, which keeps each
    // number's text. Bytes past the reader's limit are the read's
    // `incomplete` for `bytes`; a number past the double range has no exact
    // RFC 8785 spelling, so the read names it by pointer and text and it is
    // refused as any other inexact number is.
    let reading = match quire_canonical::read(bytes, budget.bytes) {
        Ok(reading) => reading,
        Err(quire_canonical::ReadError::Limit(limit)) => {
            return Err(bytes_exceeded(limit).into());
        }
        Err(quire_canonical::ReadError::NumberOutOfRange {
            pointer, lexeme, ..
        }) => {
            // The reader's pointer is RFC 6901 by its contract, so it is
            // taken as the escaped text it is.
            let document_pointer = JsonPointer::from_escaped(pointer);
            let cause = if Spelling::of(&lexeme).is_whole_past_2_pow_53() {
                Cause::InexactInteger
            } else {
                Cause::InexactNumber
            };
            return Err(SelectionFailure::InexactNumber {
                document_pointer,
                cause,
            });
        }
        Err(_) => {
            return Err(SelectionRefusal::at(
                Code::MalformedWire,
                Cause::MalformedJson,
                Some("identity"),
            )
            .into());
        }
    };
    // Decided on the text of each number, before any rounding to a double.
    if let Some((document_pointer, cause)) = first_inexact_number(&reading) {
        return Err(SelectionFailure::InexactNumber {
            document_pointer,
            cause,
        });
    }
    // The declarations are read from the one reading of the document, as a
    // `Value` view of it: the document is not parsed a second time.
    let document = document_value(&reading);
    let refusal = match identity_of(&document) {
        Some(found) if found == identity => return Ok(document),
        Some(_) | None => SelectionFailure::from(SelectionRefusal::at(
            Code::InvalidModelBinding,
            Cause::WrongModelSelection,
            Some("identity"),
        )),
    };
    quire_canonical::drop_value(document);
    Err(refusal)
}

/// The `Value` view of one reading of a document, built from an explicit heap
/// stack so a document of any depth converts on any thread stack. Each number
/// is the integer its text spells when it spells one that fits an `i64` or
/// `u64`, and the double its text denotes otherwise. The caller drops the
/// result through `quire_canonical::drop_value`, as `Value`'s own drop recurses.
fn document_value(document: &quire_canonical::Document) -> Value {
    use quire_canonical::{Items, Members, Node};
    enum Open<'d> {
        Array(Vec<Value>, Items<'d>),
        Object(serde_json::Map<String, Value>, Members<'d>, String),
    }
    let mut open: Vec<Open<'_>> = Vec::new();
    let mut pending = Some(document.root());
    loop {
        let mut finished = None;
        if let Some(node) = pending.take() {
            match node.node() {
                Node::Null => finished = Some(Value::Null),
                Node::Bool(flag) => finished = Some(Value::Bool(flag)),
                Node::Number(number) => {
                    let text = number.text();
                    let value = text
                        .parse::<i64>()
                        .map(serde_json::Number::from)
                        .or_else(|_| text.parse::<u64>().map(serde_json::Number::from))
                        .ok()
                        .or_else(|| serde_json::Number::from_f64(number.value()));
                    finished = Some(value.map_or(Value::Null, Value::Number));
                }
                Node::String(text) => finished = Some(Value::String(text.to_owned())),
                Node::Array(items) => open.push(Open::Array(Vec::new(), items)),
                Node::Object(members) => {
                    open.push(Open::Object(serde_json::Map::new(), members, String::new()));
                }
            }
        }
        // Hand each finished value to its container, then move to the next
        // child, closing every container with none left.
        loop {
            if let Some(value) = finished.take() {
                match open.last_mut() {
                    None => return value,
                    Some(Open::Array(elements, _)) => elements.push(value),
                    Some(Open::Object(members, _, name)) => {
                        members.insert(std::mem::take(name), value);
                    }
                }
            }
            match open.last_mut() {
                None => return Value::Null,
                Some(Open::Array(_, items)) => match items.next() {
                    Some(item) => {
                        pending = Some(item);
                        break;
                    }
                    None => {
                        if let Some(Open::Array(elements, _)) = open.pop() {
                            finished = Some(Value::Array(elements));
                        }
                    }
                },
                Some(Open::Object(_, members, name)) => match members.next() {
                    Some((key, member)) => {
                        key.clone_into(name);
                        pending = Some(member);
                        break;
                    }
                    None => {
                        if let Some(Open::Object(members, _, _)) = open.pop() {
                            finished = Some(Value::Object(members));
                        }
                    }
                },
            }
        }
    }
}

/// The reader's `incomplete` for `bytes`: the document's length as consumed
/// and no pointer, as every byte-limit outcome is (FR-038-AC-26).
fn bytes_exceeded(limit: quire_canonical::LimitExceeded) -> ValidationFailure {
    ValidationFailure::incomplete(
        CheckedPackageLimit::Bytes,
        limit.bound,
        limit.required,
        None,
    )
}

/// The RFC 6901 pointer and cause of the first number, in document order,
/// that has no exact RFC 8785 spelling (see [`inexact_cause`]); `None` when
/// there is none. Walks from an explicit heap stack, so a document of any
/// depth is walked on any thread stack.
fn first_inexact_number(
    document: &quire_canonical::Document,
) -> Option<(JsonPointer, CheckedPackageRefusalCause)> {
    use quire_canonical::{Items, Members, Node};
    enum Open<'d> {
        Array(Items<'d>, usize),
        Object(Members<'d>),
    }
    let mut path: Vec<Step<'_>> = Vec::new();
    let mut open: Vec<Open<'_>> = Vec::new();
    let mut next = Some(document.root());
    loop {
        if let Some(value) = next.take() {
            match value.node() {
                Node::Number(number) => {
                    if let Some(cause) = inexact_cause(number.text(), number.value()) {
                        return Some((pointer_from_steps(path.iter().copied()), cause));
                    }
                }
                Node::Array(items) => open.push(Open::Array(items, 0)),
                Node::Object(members) => open.push(Open::Object(members)),
                Node::Null | Node::Bool(_) | Node::String(_) => {}
            }
        }
        // The path holds one step for each open container but the innermost
        // child being entered, so it is cut back to the open containers
        // before the next child's step is pushed.
        let depth = open.len().saturating_sub(1);
        match open.last_mut() {
            None => return None,
            Some(Open::Array(items, index)) => match items.next() {
                Some(item) => {
                    path.truncate(depth);
                    path.push(Step::Index(*index));
                    *index = index.saturating_add(1);
                    next = Some(item);
                }
                None => {
                    open.pop();
                }
            },
            Some(Open::Object(members)) => match members.next() {
                Some((name, member)) => {
                    path.truncate(depth);
                    path.push(Step::Key(name));
                    next = Some(member);
                }
                None => {
                    open.pop();
                }
            },
        }
    }
}

/// The cause of a model-document number's refusal, or `None` when the number
/// is admitted. `text` is the
/// number as spelled in the document and `value` the double `quire-canonical`
/// read from it. The decision is made on the text, never on a double:
///
/// - `inexact-integer` when the text denotes a whole value past 2^53,
///   however spelled;
/// - `inexact-number` when the digits and scale of the text differ from those
///   of the text `quire-canonical` writes for `value`, so the exact value of
///   the number is not the one its RFC 8785 encoding carries.
fn inexact_cause(text: &str, value: f64) -> Option<CheckedPackageRefusalCause> {
    use CheckedPackageRefusalCause as Cause;
    let spelled = Spelling::of(text);
    if spelled.is_whole_past_2_pow_53() {
        return Some(Cause::InexactInteger);
    }
    // Writing one finite double cannot fail, and its text is ASCII; were
    // either to fail, the number has no RFC 8785 text to be exact against, so
    // it is refused.
    let mut written = Vec::new();
    let mut writer = quire_canonical::Writer::new(
        &mut written,
        quire_canonical::Limits::new(DOUBLE_TEXT_BYTES),
    );
    let wrote = writer.number(value).is_ok();
    let exact = wrote
        && std::str::from_utf8(&written).is_ok_and(|written| spelled == Spelling::of(written));
    (!exact).then_some(Cause::InexactNumber)
}

/// The bytes one double's RFC 8785 text can take (at most 25), with room to
/// spare: the room one number's text is written in, not a cap on a document.
const DOUBLE_TEXT_BYTES: u64 = 64;

/// The decimal digits of `2^53`, 9007199254740992.
const MAXIMUM_INTEGER_DIGITS: &str = "9007199254740992";

/// A JSON number's value as `digits` x 10^`scale`, with neither a leading nor
/// a trailing zero in `digits`, so two texts spell one value exactly when
/// their spellings are equal. Zero has no digits and no sign.
#[derive(Debug, Eq, PartialEq)]
struct Spelling {
    negative: bool,
    digits: String,
    scale: i64,
}

impl Spelling {
    /// The spelling of `text`, a number `quire-canonical` read or wrote, so
    /// `-? int frac? exp?`.
    fn of(text: &str) -> Self {
        let (negative, unsigned) = match text.strip_prefix('-') {
            Some(unsigned) => (true, unsigned),
            None => (false, text),
        };
        let (mantissa, exponent) = unsigned.split_once(['e', 'E']).unwrap_or((unsigned, "0"));
        let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
        let digits = [whole, fraction].concat();
        let trimmed = digits.trim_start_matches('0');
        let significant = trimmed.trim_end_matches('0');
        if significant.is_empty() {
            return Self {
                negative: false,
                digits: String::new(),
                scale: 0,
            };
        }
        let trailing = trimmed.len().saturating_sub(significant.len());
        let scale = parse_exponent(exponent)
            .saturating_sub(i64::try_from(fraction.len()).unwrap_or(i64::MAX))
            .saturating_add(i64::try_from(trailing).unwrap_or(i64::MAX));
        Self {
            negative,
            digits: significant.to_owned(),
            scale,
        }
    }

    /// Whether the value is a whole number of magnitude greater than 2^53.
    /// `digits` has no trailing zero, so a negative `scale` leaves a nonzero
    /// fraction: the value is not whole.
    fn is_whole_past_2_pow_53(&self) -> bool {
        if self.digits.is_empty() || self.scale < 0 {
            return false;
        }
        let length = i64::try_from(self.digits.len()).unwrap_or(i64::MAX);
        let maximum = i64::try_from(MAXIMUM_INTEGER_DIGITS.len()).unwrap_or(i64::MAX);
        match length.saturating_add(self.scale).cmp(&maximum) {
            std::cmp::Ordering::Greater => true,
            std::cmp::Ordering::Less => false,
            // Sixteen integer digits: `digits` padded with `scale` zeros.
            std::cmp::Ordering::Equal => {
                let padding = usize::try_from(self.scale).unwrap_or(0);
                [self.digits.as_str(), &"0".repeat(padding)]
                    .concat()
                    .as_str()
                    .cmp(MAXIMUM_INTEGER_DIGITS)
                    .is_gt()
            }
        }
    }
}

/// A JSON exponent, `[+-]?digits`, saturated to an `i64`.
fn parse_exponent(text: &str) -> i64 {
    let (negative, digits) = match text.strip_prefix('-') {
        Some(digits) => (true, digits),
        None => (false, text.strip_prefix('+').unwrap_or(text)),
    };
    let magnitude = digits.bytes().fold(0_i64, |total, byte| {
        total
            .saturating_mul(10)
            .saturating_add(i64::from(byte.saturating_sub(b'0')))
    });
    if negative {
        -magnitude
    } else {
        magnitude
    }
}

/// A Semantic IR 2.0.0 document's own `package.identity`. The document's
/// `package.version` is neither required nor read: a selection binds by
/// identity alone.
pub(super) fn semantic_ir_identity(document: &Value) -> Option<&str> {
    document.get("package")?.get("identity")?.as_str()
}

/// FR-322 step 1 for one selected identity: admission, then the
/// document's declarations. The document's parse and read are charged to
/// the reader's `work` limit at the identity's root.
pub(super) fn admit_selection(
    identity: &str,
    checked_package: &str,
    evidence: &CheckedPackageEvidence,
    budget: &mut Budget<'_>,
    unit_refs: &UnitRefs,
) -> Result<DomainModel, SelectionFailure> {
    let document = admit_document(
        identity,
        evidence.domain_package_document(identity),
        semantic_ir_identity,
        budget,
    )?;
    let model = read_semantic_ir(&document, checked_package, budget, unit_refs);
    quire_canonical::drop_value(document);
    model
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
fn list<'v>(value: &'v Value, member: &str) -> Result<&'v [Value], ModelRefusal> {
    match value.get(member) {
        None => Ok(&[]),
        Some(items) => items
            .as_array()
            .map(Vec::as_slice)
            .ok_or_else(ModelRefusal::malformed),
    }
}

fn text<'v>(value: &'v Value, member: &str) -> Result<&'v str, ModelRefusal> {
    value
        .get(member)
        .and_then(Value::as_str)
        .ok_or_else(ModelRefusal::malformed)
}

/// A Semantic IR multiplicity's shape, `upper` absent when unbounded.
fn semantic_ir_multiplicity(value: &Value) -> Result<Multiplicity, ModelRefusal> {
    let multiplicity = value
        .get("multiplicity")
        .ok_or_else(ModelRefusal::malformed)?;
    let lower = multiplicity
        .get("lower")
        .and_then(Value::as_u64)
        .ok_or_else(ModelRefusal::malformed)?;
    let upper = match multiplicity.get("upper") {
        None => None,
        Some(upper) => Some(upper.as_u64().ok_or_else(ModelRefusal::malformed)?),
    };
    let flag = |member: &str| {
        multiplicity
            .get(member)
            .and_then(Value::as_bool)
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
fn member_identity(member: &Value, owner: &str) -> Result<Box<str>, ModelRefusal> {
    let identity = text(member, "identity")?;
    let name = identity
        .strip_prefix(owner)
        .and_then(|rest| rest.strip_prefix('/'))
        .ok_or_else(ModelRefusal::malformed)?;
    if name.is_empty() || name.contains('/') {
        return Err(ModelRefusal::malformed());
    }
    Ok(identity.into())
}

/// An FCD value type bound as `Int[lo, hi]` or `Text[min, max; profile]`.
// Reads a selected domain package document's value type.
#[ix_trace_rs::string_edge]
fn semantic_ir_value_type(value: &Value) -> Option<ValueBounds> {
    let constraint = |keyword: &str| -> Option<&Value> {
        let mut found = list(value, "constraints")
            .ok()?
            .iter()
            .filter(|constraint| {
                constraint.get("keyword").and_then(Value::as_str) == Some(keyword)
            });
        let constraint = found.next()?;
        if found.next().is_some() {
            return None;
        }
        Some(constraint)
    };
    let integer = |keyword| -> Option<i128> {
        match constraint(keyword)?.get("operands")?.get("value")? {
            Value::Number(number) => number.as_i64().map(i128::from),
            Value::String(text) => text.parse().ok(),
            _ => None,
        }
    };
    match value.get("scalar").and_then(Value::as_str)? {
        "integer" => {
            let (lower, upper) = (integer("min")?, integer("max")?);
            (lower <= upper).then_some(ValueBounds::Integer(IntegerBounds { lower, upper }))
        }
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

/// One failure inside one node, ordered by table row and then member path.
struct Defect<'v> {
    row: Row,
    path: Vec<Segment>,
    refusal: ModelRefusal,
    declaration: &'v Value,
}

/// The failures of one node.
struct Defects<'v> {
    entries: Vec<Defect<'v>>,
    declaration: &'v Value,
}

impl<'v> Defects<'v> {
    fn new(declaration: &'v Value) -> Self {
        Self {
            entries: Vec::new(),
            declaration,
        }
    }
    fn context(&mut self, declaration: &'v Value) {
        self.declaration = declaration;
    }
    fn push(&mut self, row: Row, path: &[Segment], refusal: ModelRefusal) {
        self.entries.push(Defect {
            row,
            path: path.to_vec(),
            refusal,
            declaration: self.declaration,
        });
    }

    /// The failure FR-154 reports for the node: first by table row, then by
    /// member path.
    fn first(self) -> Option<Defect<'v>> {
        self.entries
            .into_iter()
            .min_by(|a, b| (a.row, &a.path).cmp(&(b.row, &b.path)))
    }
}

fn joined(base: &[Segment], tail: Segment) -> Vec<Segment> {
    let mut path = base.to_vec();
    path.push(tail);
    path
}

/// Require a non-empty meaning member without accepting an absent or malformed list.
fn required_items<'v>(
    value: &'v Value,
    member: &'static str,
    defects: &mut Defects<'v>,
) -> &'v [Value] {
    match list(value, member) {
        Ok(items) if !items.is_empty() => items,
        _ => {
            defects.context(value);
            defects.push(
                Row::Meaning,
                &[Segment::Name(member)],
                ModelRefusal::malformed(),
            );
            &[]
        }
    }
}

fn meaning_reference<'v>(
    value: &'v Value,
    expected: &BTreeSet<&str>,
    declared: &BTreeSet<&str>,
    path: &[Segment],
    defects: &mut Defects<'v>,
) {
    defects.context(value);
    match value.as_str() {
        Some(name) if expected.contains(name) => {}
        Some(name) if declared.contains(name) => {
            defects.push(Row::Meaning, path, ModelRefusal::malformed());
        }
        Some(_) => defects.push(Row::Reference, path, ModelRefusal::missing_name()),
        None if value.is_null() => {
            defects.push(Row::Reference, path, ModelRefusal::missing_name());
        }
        None => defects.push(Row::Meaning, path, ModelRefusal::malformed()),
    }
}

/// FR-208's required members are checked before a selected declaration can
/// contribute an authenticated Process, StateMachine or PersistenceInterface key.
fn check_selected_meaning<'v>(
    value: &'v Value,
    meaning: meaning::Kind,
    scope: &Scope<'_>,
    defects: &mut Defects<'v>,
) {
    if !matches!(
        meaning,
        meaning::Kind::Process | meaning::Kind::StateMachine | meaning::Kind::PersistenceInterface
    ) {
        return;
    }
    let declared = &scope.all_types;
    match meaning {
        meaning::Kind::Process => {
            let fields = list(value, "fields").unwrap_or_default();
            let identities: BTreeSet<&str> = fields
                .iter()
                .filter_map(|field| field.get("identity").and_then(Value::as_str))
                .collect();
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
            let mut names = BTreeSet::new();
            for (index, step) in required_items(value, "steps", defects).iter().enumerate() {
                let base = [Segment::Name("steps"), Segment::Index(index)];
                defects.context(step);
                let name = step.get("name").and_then(Value::as_str);
                if let Some(name) = name.filter(|name| is_object_id(name)) {
                    if !names.insert(name) {
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
                    step.get("stepKind").and_then(Value::as_str),
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
            let mut states = BTreeSet::new();
            for (index, state) in required_items(value, "states", defects).iter().enumerate() {
                let path = [Segment::Name("states"), Segment::Index(index)];
                defects.context(state);
                let identity = state.get("identity").and_then(Value::as_str);
                if identity.is_some_and(|identity| !states.insert(identity)) {
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
            let operations: BTreeSet<&str> = list(value, "operations")
                .unwrap_or_default()
                .iter()
                .filter_map(|operation| operation.get("identity").and_then(Value::as_str))
                .collect();
            let clauses: BTreeMap<&str, &Value> = list(value, "clauses")
                .unwrap_or_default()
                .iter()
                .filter_map(|clause| Some((clause.get("clauseId")?.as_str()?, clause)))
                .collect();
            for (index, transition) in required_items(value, "transitions", defects)
                .iter()
                .enumerate()
            {
                let base = [Segment::Name("transitions"), Segment::Index(index)];
                for (member, expected) in
                    [("from", &states), ("to", &states), ("trigger", &operations)]
                {
                    let path = joined(&base, Segment::Name(member));
                    meaning_reference(&transition[member], expected, declared, &path, defects);
                }
                if let Some(guard) = transition.get("guard") {
                    let path = joined(&base, Segment::Name("guard"));
                    defects.context(transition);
                    let clause_id = guard.get("clauseId").and_then(Value::as_str);
                    let clause = clause_id.and_then(|id| clauses.get(id).copied());
                    match clause {
                        _ if guard.get("language").and_then(Value::as_str) != Some("quire") => {
                            defects.push(Row::Meaning, &path, ModelRefusal::malformed());
                        }
                        None => defects.push(Row::Reference, &path, ModelRefusal::missing_name()),
                        Some(clause)
                            if clause.get("language").and_then(Value::as_str) != Some("quire") =>
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
                    .is_some_and(|pre| !pre.as_array().is_some_and(Vec::is_empty))
                    || operation
                        .get("post")
                        .is_some_and(|post| !post.as_array().is_some_and(Vec::is_empty))
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
                    persisted,
                    &scope.object_types,
                    declared,
                    &[Segment::Name("persists"), Segment::Index(index)],
                    defects,
                );
            }
        }
        _ => {}
    }
}

fn check_event_lists<'v>(
    value: &'v Value,
    base: &[Segment],
    members: &[&'static str],
    scope: &Scope<'_>,
    declared: &BTreeSet<&str>,
    defects: &mut Defects<'v>,
) {
    for &member in members {
        if let Ok(events) = list(value, member) {
            for (index, event) in events.iter().enumerate() {
                let path = joined(&joined(base, Segment::Name(member)), Segment::Index(index));
                meaning_reference(event, &scope.events, declared, &path, defects);
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
type ClassifiedType<'v> = Result<(&'v Value, meaning::Kind), ModelRefusal>;

/// The document-wide sets a node's references are checked against. All are
/// known before any node's body is read, so no reference outcome depends on
/// the order the nodes are read in.
struct Scope<'v> {
    /// Every classified type identity, for wrong-kind versus absent references.
    all_types: BTreeSet<&'v str>,
    /// Named option definitions, known before any member body is read.
    option_types: BTreeSet<&'v str>,
    /// Declared value-definition forms, including yet-unread option/collection bodies.
    value_definitions: BTreeSet<&'v str>,
    /// Object-identity forms accepted as declared field types.
    objects: BTreeSet<&'v str>,
    /// Object and systems-interface supertypes, excluding process/state forms.
    nominal_objects: BTreeSet<&'v str>,
    /// Object types alone may occur in a persistence interface's `persists` list.
    object_types: BTreeSet<&'v str>,
    /// Event types alone may be consumed or emitted by steps and transitions.
    events: BTreeSet<&'v str>,
    /// Process and state-machine supertypes must stay within their meanings.
    processes: BTreeSet<&'v str>,
    state_machines: BTreeSet<&'v str>,
    /// Record value types, which carry value fields without object identity.
    records: BTreeSet<&'v str>,
    /// Meanings forbidden as a record field's declared type.
    record_field_forbidden: BTreeSet<&'v str>,
    /// Type nodes whose own declaration is refused; a reference to one is
    /// not reported again, its own refusal stands for it (FR-154).
    refused: BTreeSet<&'v str>,
    /// Field and operation members of selected field-bearing declarations.
    members: BTreeSet<Box<str>>,
    /// Relationship identities are collected before any declaration body is read.
    relationships: BTreeSet<&'v str>,
}

/// A node's own refusal, from its identity, its kind and whether another
/// node shares its identity, in FR-154's table order.
// Reads a selected domain package document's node id prefix.
#[ix_trace_rs::string_edge]
fn classify<'v>(
    prefix: &str,
    meanings: &BTreeMap<(&str, &str), &'v str>,
    declared: &[&'v Value],
    node: &str,
) -> ClassifiedType<'v> {
    let one = |declared: &&'v Value| -> ClassifiedType<'v> {
        if !node.strip_prefix(prefix).is_some_and(is_object_id) {
            return Err(ModelRefusal::malformed());
        }
        let meaning = match declared.get("kind") {
            Some(Value::String(kind)) if kind == "sequence" => Some(meaning::Kind::SequenceType),
            Some(Value::String(kind)) if kind == "map" => Some(meaning::Kind::MapType),
            Some(Value::String(kind)) if kind == "option" => Some(meaning::Kind::OptionType),
            Some(Value::String(kind)) if kind == "reference" => Some(meaning::Kind::ReferenceType),
            Some(kind) if kind.is_object() => {
                let module = kind.get("module").and_then(Value::as_str);
                let name = kind.get("name").and_then(Value::as_str);
                module
                    .zip(name)
                    .and_then(|(module, name)| meanings.get(&(module, name)).copied())
                    .and_then(meaning::Kind::from_wire)
            }
            _ => None,
        };
        meaning
            .map(|meaning| (*declared, meaning))
            .ok_or_else(ModelRefusal::malformed)
    };
    let checked = declared.iter().map(one).collect::<Result<Vec<_>, _>>()?;
    match checked.as_slice() {
        [single] => Ok(*single),
        _ => Err(ModelRefusal::new(
            CheckedPackageRefusalCode::InvalidModelBinding,
            CheckedPackageRefusalCause::ConflictingBinding,
        )),
    }
}

/// The work of reading one type node: itself and every member list entry.
fn node_size(value: &Value) -> u64 {
    let len = |value: &Value, member: &str| {
        value
            .get(member)
            .and_then(Value::as_array)
            .map_or(0, Vec::len)
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
        value.get("kind").and_then(Value::as_str),
        Some("sequence" | "map" | "option" | "reference")
    ) {
        total = total.saturating_add(1);
    }
    for operation in value
        .get("operations")
        .and_then(Value::as_array)
        .map_or(&[][..], Vec::as_slice)
    {
        total = total.saturating_add(len(operation, "params"));
    }
    for step in value
        .get("steps")
        .and_then(Value::as_array)
        .map_or(&[][..], Vec::as_slice)
    {
        total = total
            .saturating_add(len(step, "consumes"))
            .saturating_add(len(step, "emits"));
    }
    for transition in value
        .get("transitions")
        .and_then(Value::as_array)
        .map_or(&[][..], Vec::as_slice)
    {
        total = total.saturating_add(len(transition, "emits"));
    }
    units(total)
}

/// Reads an admitted Semantic IR 2.0.0 document's declarations (FR-154),
/// with tuple declarations in document order before variants, then the
/// remaining nodes in ascending IR identity; each node's own failures report
/// in FR-154's table order.
/// Every node and every member read is charged to `budget`.
pub(super) fn read_semantic_ir(
    document: &Value,
    checked_package: &str,
    budget: &mut Budget<'_>,
    unit_refs: &UnitRefs,
) -> Result<DomainModel, SelectionFailure> {
    let identity = semantic_ir_identity(document).ok_or_else(ModelRefusal::malformed)?;
    let mut meanings: BTreeMap<(&str, &str), &str> = BTreeMap::new();
    for construct in list(document, "constructs")? {
        let kind = construct.get("kind").ok_or_else(ModelRefusal::malformed)?;
        let meaning = construct
            .get("construct")
            .map_or(Err(ModelRefusal::malformed()), |inner| {
                text(inner, "meaning")
            })?;
        meanings.insert((text(kind, "module")?, text(kind, "name")?), meaning);
    }
    let prefix = format!("ix://{identity}/");
    let mut types: BTreeMap<&str, Vec<&Value>> = BTreeMap::new();
    let mut aliases: BTreeMap<Box<str>, Box<str>> = BTreeMap::new();
    let mut document_positions = BTreeMap::new();
    for (position, declared) in list(document, "types")?.iter().enumerate() {
        // A node with no identity has none to share and is malformed on its
        // own; the empty identity orders before every other.
        let Some(node) = declared.get("identity").and_then(Value::as_str) else {
            return Err(
                SelectionRefusal::located(ModelRefusal::malformed(), declared, budget)?.into(),
            );
        };
        if declared.get("kind").and_then(Value::as_str) == Some("alias") {
            budget.charge(node_size(declared))?;
            let target = text(declared, "target")?;
            if aliases.insert(node.into(), target.into()).is_some() || types.contains_key(node) {
                return Err(SelectionRefusal::located(
                    ModelRefusal::malformed(),
                    declared,
                    budget,
                )?
                .into());
            }
            continue;
        }
        if aliases.contains_key(node) {
            return Err(
                SelectionRefusal::located(ModelRefusal::malformed(), declared, budget)?.into(),
            );
        }
        types.entry(node).or_default().push(declared);
        document_positions.entry(node).or_insert(position);
    }
    // Each node's meaning, or the refusal its own identity, uniqueness or
    // kind draws; read before any body so a reference check sees every
    // declared node.
    let mut classified: Vec<(&str, ClassifiedType)> = types
        .iter()
        .map(|(node, declared)| (*node, classify(&prefix, &meanings, declared, node)))
        .collect();
    classified.sort_by_key(|(node, classified)| match classified {
        Ok((_, meaning::Kind::TupleValueType)) => {
            (0, document_positions.get(node).copied().unwrap_or(0))
        }
        Ok((_, meaning::Kind::VariantType)) => {
            (1, document_positions.get(node).copied().unwrap_or(0))
        }
        _ => (2, 0),
    });
    let mut dimensions = BTreeMap::new();
    for dimension in list(document, "dimensions")? {
        budget.charge(node_size(dimension))?;
        let identity = text(dimension, "identity")?;
        let name = text(dimension, "name")?;
        let terms = list(dimension, "terms")?
            .iter()
            .map(|term| {
                Ok((
                    text(term, "dimension")?.into(),
                    text(term, "exponent")?.into(),
                ))
            })
            .collect::<Result<Vec<_>, ModelRefusal>>()?;
        if dimensions
            .insert(
                identity.into(),
                SelectedDimension {
                    name: name.into(),
                    terms,
                },
            )
            .is_some()
        {
            return Err(ModelRefusal::malformed().into());
        }
    }
    let mut units = BTreeMap::new();
    for unit in list(document, "units")? {
        budget.charge(node_size(unit))?;
        let identity = text(unit, "identity")?;
        let target = unit
            .get("target")
            .map(|_| text(unit, "target").map(Box::<str>::from))
            .transpose()?;
        let selected = SelectedUnit {
            name: text(unit, "name")?.into(),
            dimension: text(unit, "dimension")?.into(),
            target,
            scale: text(unit, "scale")?.into(),
            offset: text(unit, "offset")?.into(),
        };
        if units.insert(identity.into(), selected).is_some() {
            return Err(ModelRefusal::malformed().into());
        }
    }
    let mut model = DomainModel {
        identity: identity.into(),
        checked_package: checked_package.into(),
        root: budget.root,
        bytes: budget.bytes,
        object_types: BTreeMap::new(),
        state_machines: BTreeSet::new(),
        processes: BTreeSet::new(),
        persistence_interfaces: BTreeSet::new(),
        persistence_operations: BTreeMap::new(),
        record_types: BTreeMap::new(),
        tuple_types: BTreeMap::new(),
        value_types: BTreeMap::new(),
        aliases,
        variant_types: BTreeMap::new(),
        dimensions,
        units,
        collection_types: BTreeMap::new(),
        unit_refs: unit_refs.clone(),
        own_seconds_declared: list(document, "units")?
            .iter()
            .any(|unit| unit.get("name").and_then(Value::as_str) == Some("s")),
        relationships: BTreeMap::new(),
        populations: BTreeSet::new(),
        population_members: BTreeMap::new(),
        relation_nodes: BTreeSet::new(),
        other_types: BTreeSet::new(),
        fields: FieldTables::default(),
    };
    for population in list(document, "populations")? {
        budget.charge(node_size(population))?;
        let node = text(population, "identity")?;
        if population.get("node").is_some() {
            return Err(ModelRefusal::malformed().into());
        }
        let (_, meaning) = classify(&prefix, &meanings, &[population], node)?;
        if meaning != meaning::Kind::Population || !model.populations.insert(node.into()) {
            return Err(ModelRefusal::malformed().into());
        }
        let members = list(population, "members")?
            .iter()
            .map(|member| {
                member
                    .as_str()
                    .map(Box::<str>::from)
                    .ok_or_else(ModelRefusal::malformed)
            })
            .collect::<Result<BTreeSet<_>, _>>()?;
        model.population_members.insert(node.into(), members);
    }
    let mut scope = Scope {
        all_types: BTreeSet::new(),
        option_types: BTreeSet::new(),
        value_definitions: BTreeSet::new(),
        objects: BTreeSet::new(),
        nominal_objects: BTreeSet::new(),
        object_types: BTreeSet::new(),
        events: BTreeSet::new(),
        processes: BTreeSet::new(),
        state_machines: BTreeSet::new(),
        records: BTreeSet::new(),
        record_field_forbidden: BTreeSet::new(),
        refused: BTreeSet::new(),
        members: BTreeSet::new(),
        relationships: BTreeSet::new(),
    };
    for (node, classified) in &classified {
        let (declared, meaning) = match classified {
            Ok(classified) => *classified,
            Err(_) => {
                scope.refused.insert(node);
                continue;
            }
        };
        scope.all_types.insert(node);
        match meaning {
            meaning::Kind::SequenceType | meaning::Kind::MapType | meaning::Kind::ReferenceType => {
                scope.value_definitions.insert(node);
            }
            meaning::Kind::OptionType => {
                scope.value_definitions.insert(node);
                scope.option_types.insert(node);
            }
            meaning::Kind::ObjectType
            | meaning::Kind::StateMachine
            | meaning::Kind::Process
            | meaning::Kind::SystemsInterface
            | meaning::Kind::RecordValueType => {
                if meaning == meaning::Kind::RecordValueType {
                    scope.records.insert(node);
                } else {
                    scope.objects.insert(node);
                    if meaning == meaning::Kind::Process {
                        scope.processes.insert(node);
                    } else if meaning == meaning::Kind::StateMachine {
                        scope.state_machines.insert(node);
                    } else {
                        scope.nominal_objects.insert(node);
                        if meaning == meaning::Kind::ObjectType {
                            scope.object_types.insert(node);
                        }
                    }
                }
                if matches!(
                    meaning,
                    meaning::Kind::Process | meaning::Kind::StateMachine
                ) {
                    scope.record_field_forbidden.insert(node);
                }
                // Members and relationships are collected from every object
                // type before any is read: a reference to one resolves the
                // same whichever node declares it first.
                for member in ["fields", "operations"] {
                    for item in list(declared, member).unwrap_or_default() {
                        if let Ok(identity) = member_identity(item, node) {
                            scope.members.insert(identity);
                        }
                    }
                }
                for item in list(declared, "relationships").unwrap_or_default() {
                    if let Some(identity) = item.get("identity").and_then(Value::as_str) {
                        if !valid_relationship_identity(&model.identity, identity) {
                            scope.refused.insert(identity);
                        }
                        scope.relationships.insert(identity);
                    }
                }
            }
            meaning::Kind::ValueType => {
                model
                    .value_types
                    .insert((*node).into(), semantic_ir_value_type(declared));
            }
            meaning::Kind::TupleValueType => {
                model.tuple_types.insert((*node).into(), Vec::new());
            }
            meaning::Kind::VariantType => {
                model.variant_types.insert((*node).into(), BTreeMap::new());
            }
            meaning::Kind::SystemsConnection => {
                model.relation_nodes.insert((*node).into());
                scope.relationships.insert(node);
            }
            meaning::Kind::EventType
            | meaning::Kind::PersistenceInterface
            | meaning::Kind::Namespace
            | meaning::Kind::Population
            | meaning::Kind::SystemsPart
            | meaning::Kind::SystemsPort
            | meaning::Kind::SystemsAllocation => {
                model.other_types.insert((*node).into());
                if matches!(
                    meaning,
                    meaning::Kind::EventType
                        | meaning::Kind::PersistenceInterface
                        | meaning::Kind::Namespace
                ) {
                    scope.record_field_forbidden.insert(node);
                }
                if meaning == meaning::Kind::EventType {
                    scope.events.insert(node);
                }
                if meaning == meaning::Kind::PersistenceInterface {
                    model.persistence_interfaces.insert((*node).into());
                }
            }
        }
    }
    let mut objects = Vec::new();
    let mut records = Vec::new();
    let mut relationships = BTreeMap::new();
    for (node, classified) in classified {
        let (declared, meaning) = match classified {
            Ok(value) => value,
            Err(refusal) => {
                let candidates = types.get(node).map_or(&[][..], Vec::as_slice);
                return Err(SelectionRefusal::group(refusal, candidates, budget)?.into());
            }
        };
        budget.charge(node_size(declared))?;
        match meaning {
            meaning::Kind::OptionType | meaning::Kind::ReferenceType => {
                let type_index = *document_positions
                    .get(node)
                    .ok_or_else(ModelRefusal::malformed)?;
                let definition =
                    option_types::read(declared, meaning, type_index, &model, &scope, budget)?;
                model.collection_types.insert(node.into(), definition);
            }
            meaning::Kind::SequenceType | meaning::Kind::MapType => {
                let member = if meaning == meaning::Kind::SequenceType {
                    "items"
                } else {
                    "values"
                };
                let reference = text(declared, member)?;
                let selected = model
                    .selected_type_ref(reference)
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
                    return Err(SelectionRefusal::located(refusal, declared, budget)?.into());
                }
                let definition = if meaning == meaning::Kind::SequenceType {
                    SelectedTypeDefinition::Sequence(selected)
                } else {
                    SelectedTypeDefinition::Map(selected)
                };
                model.collection_types.insert(node.into(), definition);
            }
            meaning::Kind::ObjectType
            | meaning::Kind::StateMachine
            | meaning::Kind::Process
            | meaning::Kind::SystemsInterface
            | meaning::Kind::RecordValueType => {
                let mut defects = Defects::new(declared);
                check_selected_meaning(declared, meaning, &scope, &mut defects);
                let (object, declared_relationships) = semantic_ir_object_type(
                    declared,
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
                );
                if let Some(refusal) = defects.first() {
                    let mut selected =
                        SelectionRefusal::located(refusal.refusal, refusal.declaration, budget)?;
                    let mut pointer = JsonPointer::root().key("types").index(
                        *document_positions
                            .get(node)
                            .ok_or_else(ModelRefusal::malformed)?,
                    );
                    for step in &refusal.path {
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
                        model.state_machines.insert(node.into());
                    } else if meaning == meaning::Kind::Process {
                        model.processes.insert(node.into());
                    }
                    objects.push((node, object));
                }
                for (identity, relationship) in declared_relationships {
                    // Unreachable for an admitted object: semantic_ir_object_type
                    // records every known/local duplicate at the later actual
                    // relationship before defects.first returns. AC-196 tests
                    // pin both same-owner and cross-owner first-refusal contexts.
                    // This defensive insertion check must not replace that pass.
                    if relationships.insert(identity, relationship).is_some() {
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
                let Some(&type_index) = document_positions.get(node) else {
                    return Err(ModelRefusal::malformed().into());
                };
                let positions = match list(declared, "positions") {
                    Ok(positions) => positions,
                    Err(refusal) => {
                        return Err(selected_tuple_refusal(
                            refusal, declared, type_index, None, budget,
                        )?
                        .into())
                    }
                };
                let mut types = Vec::with_capacity(positions.len());
                let mut seen = BTreeSet::new();
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
                            .into())
                        }
                    };
                    if !seen.insert(identity) {
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
                if [
                    "fields",
                    "identityFields",
                    "operations",
                    "supertypes",
                    "clauses",
                ]
                .iter()
                .any(|member| declared.get(*member).is_some())
                {
                    return Err(selected_tuple_refusal(
                        ModelRefusal::malformed(),
                        declared,
                        type_index,
                        None,
                        budget,
                    )?
                    .into());
                }
                if positions.is_empty() {
                    return Err(selected_tuple_refusal(
                        ModelRefusal::malformed(),
                        declared,
                        type_index,
                        None,
                        budget,
                    )?
                    .into());
                }
                for (ordinal, position) in positions.iter().enumerate() {
                    let expected = format!("{node}/p{ordinal}");
                    let identity = text(position, "identity")?;
                    if identity != expected {
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
                            .into())
                        }
                    };
                    let Some(admitted) = model.selected_type_ref(type_ref) else {
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
                model.tuple_types.insert(node.into(), types);
            }
            meaning::Kind::PersistenceInterface => {
                let mut defects = Defects::new(declared);
                check_selected_meaning(declared, meaning, &scope, &mut defects);
                let (operations, _) = semantic_ir_object_type(
                    declared,
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
                );
                if let Some(refusal) = defects.first() {
                    let mut selected =
                        SelectionRefusal::located(refusal.refusal, refusal.declaration, budget)?;
                    let mut pointer = JsonPointer::root().key("types").index(
                        *document_positions
                            .get(node)
                            .ok_or_else(ModelRefusal::malformed)?,
                    );
                    for step in &refusal.path {
                        pointer = match step {
                            Segment::Name(name) => pointer.key(name),
                            Segment::Index(index) => pointer.index(*index),
                        };
                    }
                    selected.document_pointer = Some(pointer);
                    selected.declaration_path = Some(refusal.path);
                    return Err(selected.into());
                }
                model.persistence_operations.insert(node.into(), operations);
            }
            meaning::Kind::VariantType => {
                let Some(&type_index) = document_positions.get(node) else {
                    return Err(ModelRefusal::malformed().into());
                };
                let cases = match list(declared, "variants") {
                    Ok(cases) => cases,
                    Err(refusal) => {
                        return Err(SelectionRefusal::located(refusal, declared, budget)?.into());
                    }
                };
                let mut names = BTreeMap::new();
                for (case_index, case) in cases.iter().enumerate() {
                    let name = match text(case, "name") {
                        Ok(name) => name,
                        Err(refusal) => {
                            return Err(
                                SelectionRefusal::located(refusal, declared, budget)?.into()
                            );
                        }
                    };
                    if name.is_empty() || names.contains_key(name) {
                        return Err(SelectionRefusal::located(
                            ModelRefusal::malformed(),
                            declared,
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
                        None | Some(Value::Null) => None,
                        Some(Value::String(reference)) => {
                            let Some(resolved) = model.selected_type_ref(reference) else {
                                return Err(unsupported_variant_payload(
                                    case, type_index, case_index, budget,
                                )?
                                .into());
                            };
                            Some(resolved)
                        }
                        Some(_) => {
                            return Err(SelectionRefusal::located(
                                ModelRefusal::malformed(),
                                case,
                                budget,
                            )?
                            .into())
                        }
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
                    names.insert(
                        name.into(),
                        SelectedVariantCase {
                            identity: identity.into(),
                            position: case_index,
                            payload,
                        },
                    );
                }
                if names.is_empty() {
                    return Err(SelectionRefusal::located(
                        ModelRefusal::malformed(),
                        declared,
                        budget,
                    )?
                    .into());
                }
                model.variant_types.insert(node.into(), names);
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
    model.object_types.extend(
        objects
            .into_iter()
            .map(|(node, object)| (Box::from(node), object)),
    );
    model.record_types.extend(
        records
            .into_iter()
            .map(|(node, record)| (Box::from(node), record)),
    );
    model.relationships = relationships;
    option_types::check_constraints(document, &model, &scope, budget)?;
    Ok(model)
}

/// What a node's references are checked against.
struct References<'a> {
    model: &'a DomainModel,
    scope: &'a Scope<'a>,
}

enum CheckedTypeRefInput {
    MapValue(Box<str>),
    Unit,
    Native,
    Declared,
    Malformed,
}

impl CheckedTypeRefInput {
    #[ix_trace_rs::string_edge]
    fn from_wire(value: &str) -> Self {
        match SelectedTypeRef::from_admitted_name(value) {
            Some(SelectedTypeRef::Map(key, value)) if key.as_ref() == "Text" => {
                Self::MapValue(value)
            }
            Some(SelectedTypeRef::Map(..)) => Self::Malformed,
            Some(SelectedTypeRef::Unit(_)) => Self::Unit,
            Some(SelectedTypeRef::Native(_)) => Self::Native,
            Some(SelectedTypeRef::Declared(_)) => Self::Declared,
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
            || self.model.other_types.contains(node)
            || self.scope.relationships.contains(node)
    }

    /// A field type or parameter or result type (FR-154 rows 3 and 4): a
    /// native value type, or a type of the document that is not a
    /// relationship.
    // Reads a selected domain package document's type reference prefix.
    fn check_type_ref(&self, type_ref: &str, path: &[Segment], defects: &mut Defects) {
        let path = joined(path, Segment::Name("typeRef"));
        let resolved = self.model.selected_type_ref(type_ref);
        let resolved_name = match &resolved {
            Some(SelectedTypeRef::Declared(name) | SelectedTypeRef::Unit(name)) => name.as_ref(),
            _ => type_ref,
        };
        let input = match resolved {
            Some(SelectedTypeRef::Native(_)) => CheckedTypeRefInput::Native,
            _ => CheckedTypeRefInput::from_wire(resolved_name),
        };
        match input {
            CheckedTypeRefInput::MapValue(value) => self.check_type_ref(&value, &[], defects),
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
        member: &Value,
        base: &[Segment],
        defects: &mut Defects,
    ) -> Option<Box<str>> {
        let path = joined(base, Segment::Name("redefines"));
        match member.get("redefines").map(Value::as_str) {
            None => None,
            Some(Some(target)) => {
                self.check_redefines(target, &path, defects);
                Some(Box::from(target))
            }
            Some(None) => {
                defects.push(Row::Meaning, &path, ModelRefusal::malformed());
                None
            }
        }
    }

    /// A `typeRef` with a multiplicity: a field, parameter or result.
    fn slot(&self, value: &Value, base: &[Segment], defects: &mut Defects) -> TypedSlot {
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
        TypedSlot {
            name: value.get("name").and_then(Value::as_str).map(Box::from),
            type_ref: self.model.selected_type_ref(type_ref),
            multiplicity,
        }
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
    value: &Value,
    owner: &str,
    base: &[Segment],
    references: &References<'_>,
    defects: &mut Defects,
) -> Option<(Box<str>, RelationshipDecl)> {
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
            .and_then(Value::as_str)
            .and_then(RelationshipCategory::from_wire)
            .is_none()
            || value.get("composite").and_then(Value::as_bool).is_none()
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
        return None;
    };
    for (end, member) in [(&source, "sourceEnd"), (&target, "targetEnd")] {
        if let Err((row, refusal)) = references.object_ref(&end.type_ref) {
            defects.push(row, &joined(base, Segment::Name(member)), refusal);
            return None;
        }
    }
    if source.type_ref.as_ref() != owner {
        defects.push(Row::Meaning, base, ModelRefusal::malformed());
        return None;
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
            return None;
        }
    }
    Some((
        identity.into(),
        RelationshipDecl {
            source,
            target,
            direction,
        },
    ))
}

fn valid_relationship_identity(package: &str, identity: &str) -> bool {
    let prefix = format!("ix://{package}/relationship/");
    identity.strip_prefix(&prefix).is_some_and(|name| {
        !name.is_empty() && !name.contains('/') && valid_semantic_identity(identity)
    })
}

fn relationship_end(value: &Value, required_role: bool) -> Option<RelationshipEnd> {
    let members = value.as_object()?;
    if !members
        .keys()
        .all(|key| matches!(key.as_str(), "type" | "role" | "multiplicity"))
    {
        return None;
    }
    let role = match members.get("role") {
        Some(Value::String(role)) if !role.is_empty() => Some(role.as_str().into()),
        None if !required_role => None,
        _ => return None,
    };
    let multiplicity = value.get("multiplicity")?.as_object()?;
    if !multiplicity
        .keys()
        .all(|key| matches!(key.as_str(), "lower" | "upper" | "ordered" | "unique"))
    {
        return None;
    }
    Some(RelationshipEnd {
        role,
        type_ref: value
            .get("type")?
            .as_str()
            .filter(|id| valid_semantic_identity(id))?
            .into(),
        multiplicity: semantic_ir_multiplicity(value).ok()?,
    })
}

fn valid_relationship_origin(value: &Value) -> bool {
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
fn items<'v>(value: &'v Value, member: &'static str, defects: &mut Defects) -> &'v [Value] {
    items_at(value, member, &[Segment::Name(member)], defects)
}

/// One member identity, or the failure of one that is not `<owner>/<name>`.
fn identity_of(member: &Value, owner: &str, path: &[Segment], defects: &mut Defects) -> Box<str> {
    member_identity(member, owner).unwrap_or_else(|refusal| {
        defects.push(
            Row::Meaning,
            &joined(path, Segment::Name("identity")),
            refusal,
        );
        Box::from("")
    })
}

/// One object type or systems interface; every failure of the node is
/// recorded in `defects`, and the declaration returned is meaningful only
/// when there is none.
// Reads a selected domain package document's supertype references.
fn semantic_ir_object_type<'v>(
    value: &'v Value,
    node: &str,
    interface: bool,
    record: bool,
    meaning: meaning::Kind,
    references: &References<'_>,
    known_relationships: &BTreeMap<Box<str>, RelationshipDecl>,
    defects: &mut Defects<'v>,
) -> (FieldBearingTypeDecl, Vec<(Box<str>, RelationshipDecl)>) {
    let mut fields = Vec::new();
    for (index, field) in items(value, "fields", defects).iter().enumerate() {
        defects.context(field);
        let base = [Segment::Name("fields"), Segment::Index(index)];
        if record
            && field
                .get("typeRef")
                .and_then(Value::as_str)
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
        let redefines = references.redefinition(field, &base, defects);
        let slot = references.slot(field, &base, defects);
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
            identity: identity_of(field, node, &base, defects),
            slot,
            optional,
            redefines,
        });
    }
    defects.context(value);
    let mut operations = Vec::new();
    for (index, operation) in items(value, "operations", defects).iter().enumerate() {
        defects.context(operation);
        let base = [Segment::Name("operations"), Segment::Index(index)];
        let operation_identity = identity_of(operation, node, &base, defects);
        let params_path = joined(&base, Segment::Name("params"));
        let mut parameters = Vec::new();
        let declared = items_at(operation, "params", &params_path, defects);
        let mut seen = BTreeMap::new();
        let mut colliding = BTreeSet::new();
        for (position, parameter) in declared.iter().enumerate() {
            if let Some(identity) = parameter.get("identity").and_then(Value::as_str) {
                if let Some(&first) = seen.get(identity) {
                    colliding.insert(first);
                    colliding.insert(position);
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
                    seen.insert(identity, position);
                }
            }
        }
        for (position, parameter) in declared.iter().enumerate() {
            defects.context(parameter);
            let at = joined(&params_path, Segment::Index(position));
            if !colliding.contains(&position) && parameter.get("identity").is_some() {
                identity_of(parameter, &operation_identity, &at, defects);
            }
            if !colliding.contains(&position) {
                let name = parameter.get("name").and_then(Value::as_str);
                let identity_name = parameter
                    .get("identity")
                    .and_then(Value::as_str)
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
            parameters.push(references.slot(parameter, &at, defects));
        }
        defects.context(operation);
        let result = operation.get("returns").map(|returns| {
            references.slot(returns, &joined(&base, Segment::Name("returns")), defects)
        });
        let frame = match operation.get("frame") {
            Some(raw) => match serde_json::from_value(raw.clone()) {
                Ok(frame) => frame,
                Err(_) => {
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
            redefines: references.redefinition(operation, &base, defects),
        });
    }
    defects.context(value);
    let mut relationships = Vec::new();
    let mut local_relationships = BTreeSet::new();
    for (index, relationship) in items(value, "relationships", defects).iter().enumerate() {
        defects.context(relationship);
        let base = [Segment::Name("relationships"), Segment::Index(index)];
        if let Some(identity) = relationship
            .get("identity")
            .and_then(Value::as_str)
            .filter(|identity| valid_relationship_identity(&references.model.identity, identity))
        {
            if known_relationships.contains_key(identity) || !local_relationships.insert(identity) {
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
            semantic_ir_relationship(relationship, node, &base, references, defects)
        {
            relationships.push(declared);
        }
    }
    defects.context(value);
    let mut supertypes = Vec::new();
    for (index, supertype) in items(value, "supertypes", defects).iter().enumerate() {
        let path = [Segment::Name("supertypes"), Segment::Index(index)];
        match supertype.as_str() {
            Some(supertype) => {
                references.check_supertype(supertype, meaning, &path, defects);
                supertypes.push(Box::from(supertype));
            }
            None => defects.push(Row::Meaning, &path, ModelRefusal::malformed()),
        }
    }
    (
        FieldBearingTypeDecl {
            interface,
            supertypes,
            fields,
            operations,
        },
        relationships,
    )
}

/// A member list nested at `path`, or the failure of one that is not a list.
fn items_at<'v>(
    value: &'v Value,
    member: &'static str,
    path: &[Segment],
    defects: &mut Defects,
) -> &'v [Value] {
    list(value, member).unwrap_or_else(|refusal| {
        defects.push(Row::Meaning, path, refusal);
        &[]
    })
}

#[cfg(test)]
pub(super) mod tests;

#[cfg(test)]
mod structural_member_type_tests {
    use super::{declaration_id, CollectionKind, DeclarationForm, IntegerBounds, MemberType};
    use serde_json::json;

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
}
