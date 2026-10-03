//! FR-322 "Model-owned members" and "Reference conformance" (QSpec STD-100,
//! STD-101, STD-102): a `field` or `operation` member whose declaring node is
//! a model declaration node resolves through the domain package the lock
//! selects, because that node's body is `aggregate{[]}` and names no member.
//!
//! The four steps, as this module implements them:
//!
//! 1. **Selection evidence** ([`admit_selection`]). Each `model_selections`
//!    row's document is looked up by digest in the caller's evidence, its
//!    RFC 8785 digest recomputed, its own identity compared with the row's
//!    (selections bind by identity; no version is read), and its
//!    declarations read ([`read_semantic_ir`]).
//! 2. **Owner recovery** ([`ModelOwners::recover`]). Every object type and
//!    relationship declaration of each admitted document has its
//!    `ModelDeclarationNode` key recomputed; a declaring node's owner is the
//!    declaration whose key, tag and form all equal the node's, and the node's
//!    fixed members (`semantic_type` is itself, no `declaration`, no
//!    `recursion_group`, an empty body) must hold.
//! 3. **Resolution** ([`DomainModel::resolve`]), every ancestor edge, member
//!    and redefinition pair of which is charged to the reader's `work` limit
//!    at the selection's row ([`Budget`]). The member name resolves
//!    among the declaring type's exposed effective members, own and
//!    inherited, less every member a redefinition hides and every redefining
//!    member of a less derived owner (the most-derived-redefiner rule).
//! 4. **Member type** ([`DomainModel::slot_type`]). The element-type,
//!    multiplicity and presence tables give a [`MemberType`], whose node key
//!    ([`MemberType::node_key`]) is the anonymous `quire.structural-node/v1`
//!    key QSL FR-092 and FR-094 give that type, so an application's
//!    `result_type` or argument type is compared by key.
//!
//! Scope. [`read_semantic_ir`] reads the declarations these steps consult:
//! object types, systems interfaces, integer value types and relationship
//! declarations, with their fields, operations, supertypes and field
//! redefinitions. A type of any other FR-208 meaning is recorded as declared,
//! so a `typeRef` naming it resolves to a declaration with no element type,
//! but its own body is not read. Semantic IR 2.0.0 operations carry no
//! `redefines`, so every operation a document declares is its own effective
//! member.

use super::{
    member_pointer, CheckedDomainPackageRef, CheckedNodeTag, CheckedSemanticNodeV2,
    ValidationFailure, WorkMeter,
};
use crate::checked_package::common::{pointer_from_steps, Step, NODE_DOMAIN};
use crate::checked_package::evidence::CheckedPackageEvidence;
use crate::checked_package::shared::{
    CheckedPackageLimit, CheckedPackageRefusalCause, CheckedPackageRefusalCode, JsonPointer,
};
use quire_canonical::FixedShape;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

const STRUCTURAL_NODE: &str = "quire.structural-node/v1";
const NATIVE_PREFIX: &str = "ix://quire/native/";
const NATIVE_BOOLEAN: &str = "ix://quire/native/Boolean";
const NATIVE_INTEGER: &str = "ix://quire/native/Integer";
/// The unparameterized native value types a `typeRef` may name.
const NATIVE_NAMES: &[&str] = &[
    "Boolean", "Integer", "Rational", "Decimal", "Float32", "Float64", "Text",
];

/// FR-208 meaning ids, as the constructs table of a Semantic IR document
/// binds them.
mod meaning {
    pub(super) const OBJECT_TYPE: &str = "quire.meaning.model.object-type/v1";
    pub(super) const VALUE_TYPE: &str = "quire.meaning.model.value-type/v1";
    pub(super) const SYSTEMS_INTERFACE: &str = "quire.meaning.systems.interface/v1";
    pub(super) const SYSTEMS_CONNECTION: &str = "quire.meaning.systems.connection/v1";
    /// Every meaning id FR-208 declares.
    pub(super) const ALL: &[&str] = &[
        OBJECT_TYPE,
        VALUE_TYPE,
        "quire.meaning.model.record-value-type/v1",
        "quire.meaning.model.variant-type/v1",
        "quire.meaning.model.event-type/v1",
        "quire.meaning.model.state-machine/v1",
        "quire.meaning.model.process/v1",
        "quire.meaning.model.persistence-interface/v1",
        "quire.meaning.model.namespace/v1",
        "quire.meaning.model.population/v1",
        "quire.meaning.systems.part/v1",
        "quire.meaning.systems.port/v1",
        SYSTEMS_INTERFACE,
        SYSTEMS_CONNECTION,
        "quire.meaning.systems.allocation/v1",
    ];
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
    const fn unselected() -> Self {
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
    pub(super) type_ref: Box<str>,
    pub(super) multiplicity: Multiplicity,
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
    /// The IR node identity of the member this one redefines.
    pub(super) redefines: Option<Box<str>>,
}

/// One object type (or systems interface) declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ObjectTypeDecl {
    /// Whether it carries `interfaceFeatures` (a systems interface).
    pub(super) interface: bool,
    pub(super) supertypes: Vec<Box<str>>,
    pub(super) fields: Vec<FieldDecl>,
    pub(super) operations: Vec<OperationDecl>,
}

/// An integer value type bound as `Int[lo, hi]`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct IntegerBounds {
    pub(super) lower: i128,
    pub(super) upper: i128,
}

/// The declarations of one admitted domain package document that FR-322's
/// model-owned member rule consults.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct DomainModel {
    pub(super) identity: Box<str>,
    /// The reader's byte limit, under which every key derived from this
    /// model's declarations is hashed.
    pub(super) bytes: u64,
    /// Object types and systems interfaces, by IR node identity.
    pub(super) object_types: BTreeMap<Box<str>, ObjectTypeDecl>,
    /// Value types, by IR node identity; `None` when not bound as `Int[lo, hi]`.
    pub(super) value_types: BTreeMap<Box<str>, Option<IntegerBounds>>,
    /// Relationship declarations, by IR node identity.
    pub(super) relationships: BTreeSet<Box<str>>,
    /// Types of every other FR-208 meaning, by IR node identity.
    pub(super) other_types: BTreeSet<Box<str>>,
}

/// A model declaration node's `(node_tag, semantic_form)`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum DeclarationForm {
    ObjectType,
    SystemsInterface,
    Relationship,
}

impl DeclarationForm {
    const fn tag(self) -> &'static str {
        match self {
            Self::ObjectType | Self::SystemsInterface => "model",
            Self::Relationship => "relation",
        }
    }

    const fn form(self) -> &'static str {
        match self {
            Self::ObjectType => "object_type",
            Self::SystemsInterface => "systems_interface",
            Self::Relationship => "relationship",
        }
    }
}

/// The `ModelDeclarationNode` key of one declaration: the JCS SHA-256 of
/// `node-identity-preimage.schema.json`'s closed preimage, computed by
/// `quire-canonical` under `bytes`, the reader's byte limit.
///
/// The owner is the content-only `ModelOwner` (`kind`, `identity`, `node`):
/// it carries no package version, so the key is stable across a
/// version-only change of the domain package (FR-038-AC-45).
///
/// # Errors
///
/// The encoder's refusal when the preimage's canonical bytes exceed `bytes`.
pub(super) fn declaration_key(
    identity: &str,
    form: DeclarationForm,
    node: &str,
    bytes: u64,
) -> Result<String, quire_canonical::Error> {
    structural_key(
        &StructuralPreimage {
            version: STRUCTURAL_NODE,
            node_tag: form.tag(),
            semantic_form: form.form(),
            semantic_type: None,
            declaration: (),
            recursion: (),
            owner: Some(ModelOwnerPreimage {
                kind: "model",
                identity,
                node,
            }),
            body: aggregate(Vec::new()),
        },
        bytes,
    )
}

/// The SHA-256 of `preimage`'s canonical bytes, by `quire-canonical`.
fn structural_key(
    preimage: &StructuralPreimage<'_>,
    bytes: u64,
) -> Result<String, quire_canonical::Error> {
    quire_canonical::sha256(preimage, quire_canonical::Limits::new(bytes))
        .map(|digest| digest.to_string())
}

/// `quire.structural-node/v1`'s closed preimage of one node. Its depth is
/// fixed by its type, so it derives `FixedShape` and holds no `Value`: a member
/// that later grows a recursive or `Value` member stops compiling.
#[derive(Serialize, FixedShape)]
struct StructuralPreimage<'a> {
    version: &'static str,
    node_tag: &'a str,
    semantic_form: &'a str,
    semantic_type: Option<NodeRef<'a>>,
    /// Always `null`: no structural node carries a `declaration`.
    declaration: (),
    /// Always `null`: no structural node is in a recursion group.
    recursion: (),
    /// Present for a model declaration node and absent for an anonymous type.
    #[serde(skip_serializing_if = "Option::is_none")]
    owner: Option<ModelOwnerPreimage<'a>>,
    body: AggregateBody<'a>,
}

/// QSpec's `ModelOwner`: a declaration of a selected domain package.
#[derive(Serialize, FixedShape)]
struct ModelOwnerPreimage<'a> {
    kind: &'static str,
    identity: &'a str,
    node: &'a str,
}

/// A reference to a node by its key.
#[derive(Serialize, FixedShape)]
struct NodeRef<'a> {
    domain: &'static str,
    digest: &'a str,
}

fn node_ref(digest: &str) -> NodeRef<'_> {
    NodeRef {
        domain: NODE_DOMAIN,
        digest,
    }
}

/// A body: an `aggregate` of members.
#[derive(Serialize, FixedShape)]
struct AggregateBody<'a> {
    term: &'static str,
    members: Vec<BodyMember<'a>>,
}

fn aggregate(members: Vec<BodyMember<'_>>) -> AggregateBody<'_> {
    AggregateBody {
        term: "aggregate",
        members,
    }
}

/// A member of a structural body: a named integer bound or a reference.
#[derive(Serialize, FixedShape)]
#[serde(untagged)]
enum BodyMember<'a> {
    Binding(Binding<'a>),
    Reference(Reference<'a>),
}

/// A `binding` term of a bound.
#[derive(Serialize, FixedShape)]
struct Binding<'a> {
    term: &'static str,
    name: &'static str,
    value: IntegerLiteral<'a>,
}

/// An `Integer`-typed `literal` term.
#[derive(Serialize, FixedShape)]
struct IntegerLiteral<'a> {
    term: &'static str,
    #[serde(rename = "type")]
    type_node: NodeRef<'a>,
    value: &'a str,
    value_kind: &'static str,
}

/// A `reference` term.
#[derive(Serialize, FixedShape)]
struct Reference<'a> {
    term: &'static str,
    target: NodeRef<'a>,
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
    const fn of(ordered: bool, unique: bool) -> Self {
        match (ordered, unique) {
            (false, true) => Self::Set,
            (false, false) => Self::Bag,
            (true, false) => Self::Sequence,
            (true, true) => Self::OrderedSet,
        }
    }

    const fn form(self) -> &'static str {
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
    /// `Int[lo, hi]`.
    IntRange(IntegerBounds),
    /// `Reference<O>`, by the digest of `O`'s model declaration node.
    Reference(Box<str>),
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
    /// The anonymous structural node key of this type, exactly as QSL FR-092
    /// and FR-094 key it (`quire.structural-node/v1`, no `declaration`, no
    /// `owner`, self-typed composite and scalar nodes, bounded domains typed
    /// at the node they bound).
    ///
    /// Every key is hashed by `quire-canonical` under `bytes`, the reader's
    /// byte limit.
    ///
    /// # Errors
    ///
    /// The encoder's refusal when a preimage's canonical bytes exceed `bytes`.
    pub(super) fn node_key(&self, bytes: u64) -> Result<String, quire_canonical::Error> {
        match self {
            Self::Boolean => {
                anonymous("scalar_type", "boolean", None, aggregate(Vec::new()), bytes)
            }
            Self::Integer => {
                anonymous("scalar_type", "integer", None, aggregate(Vec::new()), bytes)
            }
            Self::IntRange(range) => {
                let integer = Self::Integer.node_key(bytes)?;
                anonymous(
                    "bounded_domain",
                    "integer_range",
                    Some(&integer),
                    bounds(&integer, &range.lower.to_string(), &range.upper.to_string()),
                    bytes,
                )
            }
            Self::Reference(target) => {
                anonymous("composite_type", "reference", None, over(target), bytes)
            }
            Self::Option(inner) => {
                let inner = inner.node_key(bytes)?;
                anonymous("composite_type", "option", None, over(&inner), bytes)
            }
            Self::Collection {
                kind,
                element,
                bounds: collection_bounds,
            } => {
                let element = element.node_key(bytes)?;
                let collection =
                    anonymous("composite_type", kind.form(), None, over(&element), bytes)?;
                match collection_bounds {
                    None => Ok(collection),
                    Some((lower, upper)) => {
                        let integer = Self::Integer.node_key(bytes)?;
                        anonymous(
                            "bounded_domain",
                            "collection_bounds",
                            Some(&collection),
                            bounds(&integer, &lower.to_string(), &upper.to_string()),
                            bytes,
                        )
                    }
                }
            }
        }
    }
}

/// The key of an anonymous structural node: no `declaration`, no `owner`.
fn anonymous(
    node_tag: &str,
    semantic_form: &str,
    semantic_type: Option<&str>,
    body: AggregateBody<'_>,
    bytes: u64,
) -> Result<String, quire_canonical::Error> {
    structural_key(
        &StructuralPreimage {
            version: STRUCTURAL_NODE,
            node_tag,
            semantic_form,
            semantic_type: semantic_type.map(node_ref),
            declaration: (),
            recursion: (),
            owner: None,
            body,
        },
        bytes,
    )
}

/// An `aggregate` of one `reference` to `target`.
fn over(target: &str) -> AggregateBody<'_> {
    aggregate(vec![BodyMember::Reference(Reference {
        term: "reference",
        target: node_ref(target),
    })])
}

/// An `aggregate` of the `min` and `max` bindings, each an `Integer` literal
/// typed at `integer`.
fn bounds<'a>(integer: &'a str, lower: &'a str, upper: &'a str) -> AggregateBody<'a> {
    let binding = |name: &'static str, value: &'a str| {
        BodyMember::Binding(Binding {
            term: "binding",
            name,
            value: IntegerLiteral {
                term: "literal",
                type_node: node_ref(integer),
                value,
                value_kind: "integer",
            },
        })
    };
    aggregate(vec![binding("min", lower), binding("max", upper)])
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
/// `lock.model_selections` row; an exhausted limit is the read's
/// `incomplete` result at that row.
pub(super) struct Budget<'w> {
    meter: &'w mut WorkMeter,
    selection: usize,
    /// The reader's byte limit: the ceiling of every encode the row's
    /// document and declarations reach.
    bytes: u64,
}

impl<'w> Budget<'w> {
    pub(super) fn new(meter: &'w mut WorkMeter, selection: usize, bytes: u64) -> Self {
        Self {
            meter,
            selection,
            bytes,
        }
    }

    /// The pointer of the row this budget is charged at.
    pub(super) fn row(&self) -> JsonPointer {
        member_pointer(&["lock", "model_selections"]).index(self.selection)
    }

    pub(super) fn charge(&mut self, work: u64) -> Result<(), ValidationFailure> {
        let row = self.row();
        self.meter.charge(work, || row)
    }
}

fn units(len: usize) -> u64 {
    u64::try_from(len).unwrap_or(u64::MAX)
}

impl DomainModel {
    /// The proper ancestors of `node` along its declared supertypes; one
    /// work unit per supertype edge followed.
    fn ancestors(
        &self,
        node: &str,
        budget: &mut Budget<'_>,
    ) -> Result<BTreeSet<&str>, ValidationFailure> {
        let mut reached = BTreeSet::new();
        let mut pending: Vec<&str> = self
            .object_types
            .get(node)
            .map(|declared| declared.supertypes.iter().map(AsRef::as_ref).collect())
            .unwrap_or_default();
        while let Some(supertype) = pending.pop() {
            budget.charge(1)?;
            if reached.insert(supertype) {
                if let Some(declared) = self.object_types.get(supertype) {
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
    /// effective members of the object type `node`. Every ancestor edge,
    /// member and redefinition pair visited is one work unit.
    pub(super) fn resolve(
        &self,
        node: &str,
        kind: MemberKind,
        name: &str,
        budget: &mut Budget<'_>,
    ) -> Result<Resolved<'_>, ModelFailure> {
        let Some((owner_node, declared)) = self.object_types.get_key_value(node) else {
            return Err(ModelRefusal::ineligible().into());
        };
        let ancestors = self.ancestors(node, budget)?;
        let owners = std::iter::once((owner_node.as_ref(), declared)).chain(
            ancestors
                .iter()
                .filter_map(|ancestor| Some((*ancestor, self.object_types.get(*ancestor)?))),
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
    // Reads a selected domain package document's native type reference, decoding it to a member type.
    fn element_type(&self, type_ref: &str) -> Option<MemberType> {
        match type_ref {
            NATIVE_BOOLEAN => return Some(MemberType::Boolean),
            NATIVE_INTEGER => return Some(MemberType::Integer),
            _ => {}
        }
        if let Some(bounds) = self.value_types.get(type_ref) {
            return bounds.map(MemberType::IntRange);
        }
        let object = self.object_types.get(type_ref)?;
        if object.interface {
            return None;
        }
        // `ModelOwners::new` derives this same key (or a longer one for an
        // interface) under the same limit and refuses the read when the
        // encoder refuses it, so no refusal reaches this point.
        declaration_key(
            &self.identity,
            DeclarationForm::ObjectType,
            type_ref,
            self.bytes,
        )
        .ok()
        .map(|key| MemberType::Reference(key.into()))
    }

    /// FR-322 step 4 over one declared `typeRef` and multiplicity; `None`
    /// when the tables give the slot no type.
    pub(super) fn slot_type(&self, slot: &TypedSlot) -> Option<MemberType> {
        let element = self.element_type(&slot.type_ref)?;
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

    /// A field's type: its slot type, wrapped in `Option` when its presence
    /// is `optional`.
    pub(super) fn field_type(&self, field: &FieldDecl) -> Option<MemberType> {
        let slot = self.slot_type(&field.slot)?;
        Some(if field.optional {
            MemberType::Option(Box::new(slot))
        } else {
            slot
        })
    }
}

/// FR-322 step 2's owner of one model declaration node.
#[derive(Clone, Copy, Debug)]
pub(super) struct Owner<'m> {
    pub(super) package: &'m DomainModel,
    pub(super) form: DeclarationForm,
    /// The declaration's IR node identity.
    pub(super) node: &'m str,
    /// The `lock.model_selections` row of `package`.
    pub(super) selection: usize,
}

/// Every selected declaration's model declaration node key.
#[derive(Debug, Default)]
pub(super) struct ModelOwners<'m> {
    by_key: BTreeMap<String, Owner<'m>>,
}

impl<'m> ModelOwners<'m> {
    /// Recomputes the key of every object type and relationship declaration
    /// of each admitted document; `charge` is called once per key.
    pub(super) fn new(
        models: &'m [DomainModel],
        mut charge: impl FnMut(usize) -> Result<(), ValidationFailure>,
    ) -> Result<Self, ValidationFailure> {
        let mut by_key = BTreeMap::new();
        for (index, package) in models.iter().enumerate() {
            let objects = package.object_types.iter().map(|(node, declared)| {
                let form = if declared.interface {
                    DeclarationForm::SystemsInterface
                } else {
                    DeclarationForm::ObjectType
                };
                (node, form)
            });
            let relationships = package
                .relationships
                .iter()
                .map(|node| (node, DeclarationForm::Relationship));
            for (node, form) in objects.chain(relationships) {
                charge(index)?;
                // A key the encoder refuses (a preimage past the byte limit)
                // refuses the read as `invalid_semantic_graph` at the
                // selection row the declaration belongs to.
                let key = declaration_key(&package.identity, form, node, package.bytes).map_err(
                    |_| {
                        ValidationFailure::refused(
                            CheckedPackageRefusalCode::InvalidSemanticGraph,
                            member_pointer(&["lock", "model_selections"]).index(index),
                        )
                    },
                )?;
                by_key.insert(
                    key,
                    Owner {
                        package,
                        form,
                        node,
                        selection: index,
                    },
                );
            }
        }
        Ok(Self { by_key })
    }

    /// Whether `node` is a model declaration node: a `model` or `relation`
    /// node that carries no `declaration`, or whose key is a selected
    /// declaration's model declaration node key.
    pub(super) fn is_model_declaration_node(
        &self,
        node: &CheckedSemanticNodeV2,
        tag: CheckedNodeTag,
    ) -> bool {
        matches!(tag, CheckedNodeTag::Model | CheckedNodeTag::Relation)
            && (node.declaration.is_none()
                || self.by_key.contains_key(node.node_id.digest.as_ref()))
    }

    /// FR-322 step 2 over one wire node.
    pub(super) fn recover(&self, node: &CheckedSemanticNodeV2) -> Result<Owner<'m>, ModelRefusal> {
        let owner = self
            .by_key
            .get(node.node_id.digest.as_ref())
            .filter(|owner| {
                owner.form.tag() == node.node_tag.as_ref()
                    && owner.form.form() == node.semantic_form.as_ref()
            })
            .copied()
            .ok_or_else(ModelRefusal::unselected)?;
        let fixed = node.semantic_type == node.node_id
            && node.declaration.is_none()
            && node.recursion_group.is_none()
            && node.body == json!({"term": "aggregate", "members": []});
        if fixed {
            Ok(owner)
        } else {
            Err(ModelRefusal::new(
                CheckedPackageRefusalCode::InvalidPackage,
                CheckedPackageRefusalCause::StaleNodeKey,
            ))
        }
    }
}

impl<'m> ModelOwners<'m> {
    /// FR-322 steps 2 and 3 for the member of `kind` named `name` on the
    /// model declaration node `node`: the recovered owner and the member,
    /// or the refusal the first failing step determined. Step 3's charges
    /// go to `meter` at the owner's selection row; an exhausted limit is the
    /// outer error.
    pub(super) fn resolve_member(
        &self,
        node: &CheckedSemanticNodeV2,
        kind: MemberKind,
        name: &str,
        meter: &mut WorkMeter,
    ) -> Result<Result<(Owner<'m>, Resolved<'m>), ModelRefusal>, ValidationFailure> {
        let owner = match self.recover(node) {
            Ok(owner) => owner,
            Err(refusal) => return Ok(Err(refusal)),
        };
        let mut budget = Budget::new(meter, owner.selection, owner.package.bytes);
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
    pub(super) fn object_type(&self) -> Option<&ObjectTypeDecl> {
        self.package.object_types.get(self.node)
    }
}

/// How many bytes of a domain package document one work unit covers.
/// Informational tuning: nothing but the reader's own `work` limit reads it.
const DOCUMENT_BYTES_PER_WORK: usize = 1024;

/// A step 1 refusal and the member of the selection row it is about
/// (`None`: the row itself).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct SelectionRefusal {
    pub(super) refusal: ModelRefusal,
    pub(super) member: Option<&'static str>,
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
        }
    }
}

/// Why FR-322 step 1 stopped at one `model_selections` row: a refusal, or a
/// reader limit exhausted while reading the row's document.
#[derive(Debug)]
pub(super) enum SelectionFailure {
    Refused(SelectionRefusal),
    Limit(ValidationFailure),
    /// The document holds a number with no exact RFC 8785 spelling:
    /// `noncanonical_wire` at the row's `digest`, with the pointer of the
    /// first such number in the document and the cause `inexact-integer` or
    /// `inexact-number`.
    InexactNumber {
        document_pointer: JsonPointer,
        cause: CheckedPackageRefusalCause,
    },
}

impl From<SelectionRefusal> for SelectionFailure {
    fn from(refusal: SelectionRefusal) -> Self {
        Self::Refused(refusal)
    }
}

impl From<ValidationFailure> for SelectionFailure {
    fn from(failure: ValidationFailure) -> Self {
        Self::Limit(failure)
    }
}

impl From<ModelFailure> for SelectionFailure {
    fn from(failure: ModelFailure) -> Self {
        match failure {
            ModelFailure::Refused(refusal) => Self::Refused(SelectionRefusal {
                refusal,
                member: None,
            }),
            ModelFailure::Limit(failure) => Self::Limit(failure),
        }
    }
}

/// FR-322 step 1's four admission checks, in FR-154's order, over the
/// document supplied under the row's digest. `identity_of` reads the
/// document's own identity (its `package.version`, if any, is never read).
/// Parsing is charged to the reader's `work` limit before it starts, one unit
/// per [`DOCUMENT_BYTES_PER_WORK`] bytes, so a document too large for the
/// limit is `incomplete` at its row (`/lock/model_selections/<i>`) and is
/// never parsed. The reader recomputes
/// the RFC 8785 digest from the supplied bytes and compares it with the
/// digest the lock selected, so the model actually parsed is the one the
/// package was compiled against: a content identity check, not a comparison
/// of the lock with itself. Returns the parsed document.
// Intake: checks the fixed digest domain of a selected domain package document.
pub(super) fn admit_document(
    selection: &CheckedDomainPackageRef,
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
            Some("digest"),
        )
        .into());
    };
    budget.charge(units(bytes.len().div_ceil(DOCUMENT_BYTES_PER_WORK)))?;
    let mismatch = SelectionRefusal::at(
        Code::StaleDependency,
        Cause::ByteDigestMismatch,
        Some("digest"),
    );
    // The document is read once, by `quire-canonical`, which keeps each
    // number's text. Bytes that are not strict JSON have no RFC 8785 form, so
    // no digest of theirs equals the selected one; bytes past the reader's
    // limit are the read's `incomplete` for `bytes`.
    let reading = match quire_canonical::read(bytes, budget.bytes) {
        Ok(reading) => reading,
        Err(quire_canonical::ReadError::Limit(limit)) => {
            return Err(bytes_exceeded(limit).into());
        }
        Err(_) => return Err(mismatch.into()),
    };
    // Decided on the text of each number, before any rounding to a double and
    // before any digest, so two documents that differ in such a number never
    // share one.
    if let Some((document_pointer, cause)) = first_inexact_number(&reading) {
        return Err(SelectionFailure::InexactNumber {
            document_pointer,
            cause,
        });
    }
    let digest = match quire_canonical::sha256(&reading, quire_canonical::Limits::new(budget.bytes))
    {
        Ok(digest) => digest,
        Err(quire_canonical::Error::Limit(limit)) => return Err(bytes_exceeded(limit).into()),
        Err(_) => return Err(mismatch.into()),
    };
    if digest.to_string() != selection.digest.as_ref() {
        return Err(mismatch.into());
    }
    // The declarations are read from the one reading of the document, as a
    // `Value` view of it: the document is not parsed a second time, so the
    // number decision above, the digest and the declarations are about the
    // same reading.
    let document = document_value(&reading);
    let wrong = |member| {
        SelectionFailure::from(SelectionRefusal::at(
            Code::InvalidModelBinding,
            Cause::WrongModelSelection,
            Some(member),
        ))
    };
    let refusal = match identity_of(&document) {
        Some(identity) if identity == selection.identity.as_ref() => return Ok(document),
        Some(_) | None => wrong("identity"),
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

/// The cause of a model-document number's refusal (FR-038-AC-109 and
/// FR-038-AC-110), or `None` when the number is admitted. `text` is the
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
    // Writing one finite double cannot fail; were it to, the number has no
    // RFC 8785 text to be exact against, so it is refused.
    let mut written = Vec::new();
    let mut writer = quire_canonical::Writer::new(&mut written, quire_canonical::Limits::new(64));
    if writer.number(value).is_err() {
        return Some(Cause::InexactNumber);
    }
    let written = String::from_utf8_lossy(&written);
    (spelled != Spelling::of(&written)).then_some(Cause::InexactNumber)
}

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
/// identity and content digest alone.
pub(super) fn semantic_ir_identity(document: &Value) -> Option<&str> {
    document.get("package")?.get("identity")?.as_str()
}

/// FR-322 step 1 for one `model_selections` row: admission, then the
/// document's declarations. The document's parse and read are charged to
/// the reader's `work` limit at the row.
pub(super) fn admit_selection(
    selection: &CheckedDomainPackageRef,
    evidence: &CheckedPackageEvidence,
    budget: &mut Budget<'_>,
) -> Result<DomainModel, SelectionFailure> {
    let document = admit_document(
        selection,
        evidence.domain_package_document(&selection.digest),
        semantic_ir_identity,
        budget,
    )?;
    let model = read_semantic_ir(&document, budget);
    quire_canonical::drop_value(document);
    Ok(model?)
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

/// An FCD value type bound as `Int[lo, hi]`: `scalar` `integer` with one
/// `min` and one `max` constraint whose bounds read as integers, `lo <= hi`.
// Reads a selected domain package document's value type.
fn semantic_ir_value_type(value: &Value) -> Option<IntegerBounds> {
    if value.get("scalar").and_then(Value::as_str) != Some("integer") {
        return None;
    }
    let bound = |keyword: &str| -> Option<i128> {
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
        match constraint.get("operands")?.get("value")? {
            Value::Number(number) => number.as_i64().map(i128::from),
            Value::String(text) => text.parse().ok(),
            _ => None,
        }
    };
    let (lower, upper) = (bound("min")?, bound("max")?);
    (lower <= upper).then_some(IntegerBounds { lower, upper })
}

/// FR-154's declaration-refusal table rows, in table order: the order one
/// node's failures report in.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Row {
    /// The node is not valid for its meaning, including a reference that
    /// names a node of the wrong meaning.
    Meaning,
    /// A reference names no node or member of the package.
    Reference,
    /// A multiplicity has `lower > upper`.
    Multiplicity,
}

/// A member path segment: a member name, or a list index. A name orders
/// before an index and names by their UTF-8 bytes (FR-154).
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Segment {
    Name(&'static str),
    Index(usize),
}

/// One failure inside one node, ordered by table row and then member path.
struct Defect {
    row: Row,
    path: Vec<Segment>,
    refusal: ModelRefusal,
}

/// The failures of one node.
#[derive(Default)]
struct Defects(Vec<Defect>);

impl Defects {
    fn push(&mut self, row: Row, path: &[Segment], refusal: ModelRefusal) {
        self.0.push(Defect {
            row,
            path: path.to_vec(),
            refusal,
        });
    }

    /// The failure FR-154 reports for the node: first by table row, then by
    /// member path.
    fn first(self) -> Option<ModelRefusal> {
        self.0
            .into_iter()
            .min_by(|a, b| (a.row, &a.path).cmp(&(b.row, &b.path)))
            .map(|defect| defect.refusal)
    }
}

fn joined(base: &[Segment], tail: Segment) -> Vec<Segment> {
    let mut path = base.to_vec();
    path.push(tail);
    path
}

/// One declared type node: its declaration and meaning, or the refusal it draws.
type ClassifiedType<'v> = Result<(&'v Value, &'v str), ModelRefusal>;

/// The document-wide sets a node's references are checked against. All are
/// known before any node's body is read, so no reference outcome depends on
/// the order the nodes are read in.
struct Scope<'v> {
    /// Object types and systems interfaces.
    objects: BTreeSet<&'v str>,
    /// Type nodes whose own declaration is refused; a reference to one is
    /// not reported again, its own refusal stands for it (FR-154).
    refused: BTreeSet<&'v str>,
    /// Field and operation members of the object types.
    members: BTreeSet<Box<str>>,
}

/// A node's own refusal, from its identity, its kind and whether another
/// node shares its identity, in FR-154's table order.
// Reads a selected domain package document's node id prefix.
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
        declared
            .get("kind")
            .filter(|kind| kind.is_object())
            .and_then(|kind| {
                let module = kind.get("module")?.as_str()?;
                let name = kind.get("name")?.as_str()?;
                meanings.get(&(module, name)).copied()
            })
            .filter(|meaning| meaning::ALL.contains(meaning))
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
    for member in ["fields", "operations", "supertypes", "relationships"] {
        total = total.saturating_add(len(value, member));
    }
    for operation in value
        .get("operations")
        .and_then(Value::as_array)
        .map_or(&[][..], Vec::as_slice)
    {
        total = total.saturating_add(len(operation, "params"));
    }
    units(total)
}

/// Reads an admitted Semantic IR 2.0.0 document's declarations (FR-154),
/// node by node in ascending IR node identity, returning the first refusal
/// in that order; each node's own failures report in FR-154's table order.
/// Every node and every member read is charged to `budget`.
pub(super) fn read_semantic_ir(
    document: &Value,
    budget: &mut Budget<'_>,
) -> Result<DomainModel, ModelFailure> {
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
    for declared in list(document, "types")? {
        // A node with no identity has none to share and is malformed on its
        // own; the empty identity orders before every other.
        let Some(node) = declared.get("identity").and_then(Value::as_str) else {
            return Err(ModelRefusal::malformed().into());
        };
        types.entry(node).or_default().push(declared);
    }
    // Each node's meaning, or the refusal its own identity, uniqueness or
    // kind draws; read before any body so a reference check sees every
    // declared node.
    let classified: Vec<(&str, ClassifiedType)> = types
        .iter()
        .map(|(node, declared)| (*node, classify(&prefix, &meanings, declared, node)))
        .collect();
    let mut model = DomainModel {
        identity: identity.into(),
        bytes: budget.bytes,
        object_types: BTreeMap::new(),
        value_types: BTreeMap::new(),
        relationships: BTreeSet::new(),
        other_types: BTreeSet::new(),
    };
    let mut scope = Scope {
        objects: BTreeSet::new(),
        refused: BTreeSet::new(),
        members: BTreeSet::new(),
    };
    for (node, classified) in &classified {
        let (declared, meaning) = match classified {
            Ok(classified) => *classified,
            Err(_) => {
                scope.refused.insert(node);
                continue;
            }
        };
        match meaning {
            meaning::OBJECT_TYPE | meaning::SYSTEMS_INTERFACE => {
                scope.objects.insert(node);
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
                    if let Ok(identity) = member_identity(item, node) {
                        model.relationships.insert(identity);
                    }
                }
            }
            meaning::VALUE_TYPE => {
                model.value_types.insert((*node).into(), None);
            }
            meaning::SYSTEMS_CONNECTION => {
                model.relationships.insert((*node).into());
            }
            _ => {
                model.other_types.insert((*node).into());
            }
        }
    }
    let mut objects = Vec::new();
    for (node, classified) in classified {
        let (declared, meaning) = classified?;
        budget.charge(node_size(declared))?;
        match meaning {
            meaning::OBJECT_TYPE | meaning::SYSTEMS_INTERFACE => {
                let mut defects = Defects::default();
                let object = semantic_ir_object_type(
                    declared,
                    node,
                    meaning == meaning::SYSTEMS_INTERFACE,
                    &References {
                        model: &model,
                        scope: &scope,
                    },
                    &mut defects,
                );
                if let Some(refusal) = defects.first() {
                    return Err(refusal.into());
                }
                objects.push((node, object));
            }
            meaning::VALUE_TYPE => {
                model
                    .value_types
                    .insert(node.into(), semantic_ir_value_type(declared));
            }
            _ => {}
        }
    }
    model.object_types.extend(
        objects
            .into_iter()
            .map(|(node, object)| (Box::from(node), object)),
    );
    Ok(model)
}

/// What a node's references are checked against.
struct References<'a> {
    model: &'a DomainModel,
    scope: &'a Scope<'a>,
}

impl References<'_> {
    /// Whether `node` is declared by the document as something other than an
    /// object type: a value type, a type of another meaning, or a
    /// relationship.
    fn declared_elsewhere(&self, node: &str) -> bool {
        self.model.value_types.contains_key(node)
            || self.model.other_types.contains(node)
            || self.model.relationships.contains(node)
    }

    /// A field type or parameter or result type (FR-154 rows 3 and 4): a
    /// native value type, or a type of the document that is not a
    /// relationship.
    // Reads a selected domain package document's type reference prefix.
    fn check_type_ref(&self, type_ref: &str, path: &[Segment], defects: &mut Defects) {
        let path = joined(path, Segment::Name("typeRef"));
        match type_ref.strip_prefix(NATIVE_PREFIX) {
            Some(name) if NATIVE_NAMES.contains(&name) => {}
            Some(_) => defects.push(Row::Meaning, &path, ModelRefusal::malformed()),
            None if self.scope.objects.contains(type_ref)
                || self.model.value_types.contains_key(type_ref)
                || self.model.other_types.contains(type_ref) => {}
            None if self.model.relationships.contains(type_ref) => {
                defects.push(Row::Meaning, &path, ModelRefusal::malformed());
            }
            None if self.scope.refused.contains(type_ref) => {}
            None => defects.push(Row::Reference, &path, ModelRefusal::missing_name()),
        }
    }

    /// A supertype names an object type of the document.
    fn check_supertype(&self, supertype: &str, path: &[Segment], defects: &mut Defects) {
        if self.scope.objects.contains(supertype) || self.scope.refused.contains(supertype) {
            return;
        }
        if self.declared_elsewhere(supertype) {
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
            type_ref: type_ref.into(),
            multiplicity,
        }
    }
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
fn semantic_ir_object_type(
    value: &Value,
    node: &str,
    interface: bool,
    references: &References<'_>,
    defects: &mut Defects,
) -> ObjectTypeDecl {
    let mut fields = Vec::new();
    for (index, field) in items(value, "fields", defects).iter().enumerate() {
        let base = [Segment::Name("fields"), Segment::Index(index)];
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
        let redefines_path = joined(&base, Segment::Name("redefines"));
        let redefines = match field.get("redefines").map(Value::as_str) {
            None => None,
            Some(Some(target)) => {
                references.check_redefines(target, &redefines_path, defects);
                Some(Box::from(target))
            }
            Some(None) => {
                defects.push(Row::Meaning, &redefines_path, ModelRefusal::malformed());
                None
            }
        };
        fields.push(FieldDecl {
            identity: identity_of(field, node, &base, defects),
            slot: references.slot(field, &base, defects),
            optional,
            redefines,
        });
    }
    let mut operations = Vec::new();
    for (index, operation) in items(value, "operations", defects).iter().enumerate() {
        let base = [Segment::Name("operations"), Segment::Index(index)];
        let params_path = joined(&base, Segment::Name("params"));
        let mut parameters = Vec::new();
        let declared = items_at(operation, "params", &params_path, defects);
        for (position, parameter) in declared.iter().enumerate() {
            let at = joined(&params_path, Segment::Index(position));
            parameters.push(references.slot(parameter, &at, defects));
        }
        let result = operation.get("returns").map(|returns| {
            references.slot(returns, &joined(&base, Segment::Name("returns")), defects)
        });
        operations.push(OperationDecl {
            identity: identity_of(operation, node, &base, defects),
            parameters,
            result,
            redefines: None,
        });
    }
    for (index, relationship) in items(value, "relationships", defects).iter().enumerate() {
        let base = [Segment::Name("relationships"), Segment::Index(index)];
        identity_of(relationship, node, &base, defects);
    }
    let mut supertypes = Vec::new();
    for (index, supertype) in items(value, "supertypes", defects).iter().enumerate() {
        let path = [Segment::Name("supertypes"), Segment::Index(index)];
        match supertype.as_str() {
            Some(supertype) => {
                references.check_supertype(supertype, &path, defects);
                supertypes.push(Box::from(supertype));
            }
            None => defects.push(Row::Meaning, &path, ModelRefusal::malformed()),
        }
    }
    ObjectTypeDecl {
        interface,
        supertypes,
        fields,
        operations,
    }
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
