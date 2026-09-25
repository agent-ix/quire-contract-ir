//! FR-322 "Model-owned members" and "Reference conformance" (QSpec STD-100,
//! STD-101, STD-102): a `field` or `operation` member whose declaring node is
//! a model declaration node resolves through the domain package the lock
//! selects, because that node's body is `aggregate{[]}` and names no member.
//!
//! The four steps, as this module implements them:
//!
//! 1. **Selection evidence** ([`admit_selection`]). Each `model_selections`
//!    row's document is looked up by digest in the caller's evidence, its
//!    RFC 8785 digest recomputed, its own identity and version compared with
//!    the row, and its declarations read ([`read_semantic_ir`]).
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
    ValidationFailure, WorkMeter, DOMAIN_PACKAGE_DIGEST,
};
use crate::checked_package::common::{digest_json, strict_json_value, NODE_DOMAIN};
use crate::checked_package::evidence::CheckedPackageEvidence;
use crate::checked_package::shared::{
    CheckedPackageRefusalCause, CheckedPackageRefusalCode, JsonPointer,
};
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

    const fn malformed() -> Self {
        Self::new(
            CheckedPackageRefusalCode::InvalidModelBinding,
            CheckedPackageRefusalCause::MalformedDeclaration,
        )
    }

    const fn missing_name() -> Self {
        Self::new(
            CheckedPackageRefusalCode::MissingDeclaration,
            CheckedPackageRefusalCause::MissingName,
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
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct DomainModel {
    pub(super) identity: Box<str>,
    pub(super) version: Box<str>,
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
/// `node-identity-preimage.schema.json`'s closed preimage.
pub(super) fn declaration_key(
    identity: &str,
    version: &str,
    form: DeclarationForm,
    node: &str,
) -> String {
    structural_key(&json!({
        "version": STRUCTURAL_NODE,
        "node_tag": form.tag(),
        "semantic_form": form.form(),
        "semantic_type": null,
        "declaration": null,
        "recursion": null,
        "owner": {"kind": "model", "identity": identity, "version": version, "node": node},
        "body": {"term": "aggregate", "members": []},
    }))
}

fn structural_key(preimage: &Value) -> String {
    // A `json!` value always serializes.
    digest_json(preimage).unwrap_or_default()
}

fn node_ref(digest: &str) -> Value {
    json!({"domain": NODE_DOMAIN, "digest": digest})
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
    pub(super) fn node_key(&self) -> String {
        let integer_literal = |value: &str| {
            json!({
                "term": "literal",
                "type": node_ref(&Self::Integer.node_key()),
                "value": value,
                "value_kind": "integer",
            })
        };
        let bounds = |lower: &str, upper: &str| {
            json!({"term": "aggregate", "members": [
                {"term": "binding", "name": "min", "value": integer_literal(lower)},
                {"term": "binding", "name": "max", "value": integer_literal(upper)},
            ]})
        };
        let over = |inner: &str| json!({"term": "aggregate", "members": [{"term": "reference", "target": node_ref(inner)}]});
        let node = |tag: &str, form: &str, semantic_type: Value, body: Value| {
            structural_key(&json!({
                "version": STRUCTURAL_NODE,
                "node_tag": tag,
                "semantic_form": form,
                "semantic_type": semantic_type,
                "declaration": null,
                "recursion": null,
                "body": body,
            }))
        };
        let empty = json!({"term": "aggregate", "members": []});
        match self {
            Self::Boolean => node("scalar_type", "boolean", Value::Null, empty),
            Self::Integer => node("scalar_type", "integer", Value::Null, empty),
            Self::IntRange(range) => node(
                "bounded_domain",
                "integer_range",
                node_ref(&Self::Integer.node_key()),
                bounds(&range.lower.to_string(), &range.upper.to_string()),
            ),
            Self::Reference(target) => {
                node("composite_type", "reference", Value::Null, over(target))
            }
            Self::Option(inner) => node(
                "composite_type",
                "option",
                Value::Null,
                over(&inner.node_key()),
            ),
            Self::Collection {
                kind,
                element,
                bounds: collection_bounds,
            } => {
                let collection = node(
                    "composite_type",
                    kind.form(),
                    Value::Null,
                    over(&element.node_key()),
                );
                match collection_bounds {
                    None => collection,
                    Some((lower, upper)) => node(
                        "bounded_domain",
                        "collection_bounds",
                        node_ref(&collection),
                        bounds(&lower.to_string(), &upper.to_string()),
                    ),
                }
            }
        }
    }
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
}

impl<'w> Budget<'w> {
    pub(super) fn new(meter: &'w mut WorkMeter, selection: usize) -> Self {
        Self { meter, selection }
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
            (Some(_), Some(_)) => Err(ModelRefusal::new(
                CheckedPackageRefusalCode::AmbiguousDeclaration,
                CheckedPackageRefusalCause::AmbiguousName,
            )
            .into()),
            (None, _) => Err(ModelRefusal::ineligible().into()),
        }
    }

    /// FR-322 step 4's element type of a declared `typeRef`.
    // string-edge: reads a selected domain package document's native type reference, decoding it to a member type.
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
        (!object.interface).then(|| {
            MemberType::Reference(
                declaration_key(
                    &self.identity,
                    &self.version,
                    DeclarationForm::ObjectType,
                    type_ref,
                )
                .into(),
            )
        })
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
    pub(super) fn new<E>(
        models: &'m [DomainModel],
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Self, E> {
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
                by_key.insert(
                    declaration_key(&package.identity, &package.version, form, node),
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
/// document's own identity and version. Parsing is charged to the reader's
/// `work` limit before it starts, one unit per [`DOCUMENT_BYTES_PER_WORK`]
/// bytes, so a document too large for the limit is `incomplete` at its row
/// (`/lock/model_selections/<i>`) and is never parsed. Returns the parsed
/// document.
// string-edge: intake: checks the fixed digest domain of a selected domain package document.
pub(super) fn admit_document(
    selection: &CheckedDomainPackageRef,
    supplied: Option<&[u8]>,
    identity_of: fn(&Value) -> Option<(&str, &str)>,
    budget: &mut Budget<'_>,
) -> Result<Value, SelectionFailure> {
    use CheckedPackageRefusalCause as Cause;
    use CheckedPackageRefusalCode as Code;
    if selection.digest_domain.as_ref() != DOMAIN_PACKAGE_DIGEST {
        return Err(SelectionRefusal::at(
            Code::StaleDependency,
            Cause::DigestDomainMismatch,
            Some("digest_domain"),
        )
        .into());
    }
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
    // Bytes that are not strict JSON have no RFC 8785 form, so no digest of
    // theirs equals the selected one.
    let document = strict_json_value(bytes).map_err(|_| mismatch)?;
    if digest_json(&document).ok().as_deref() != Some(selection.digest.as_ref()) {
        return Err(mismatch.into());
    }
    let wrong = |member| {
        SelectionFailure::from(SelectionRefusal::at(
            Code::InvalidModelBinding,
            Cause::WrongModelSelection,
            Some(member),
        ))
    };
    match identity_of(&document) {
        Some((identity, _)) if identity != selection.identity.as_ref() => Err(wrong("identity")),
        Some((_, version)) if version != selection.version.as_ref() => Err(wrong("version")),
        Some(_) => Ok(document),
        None => Err(wrong("identity")),
    }
}

/// A Semantic IR 2.0.0 document's own `package.identity` and
/// `package.version`.
pub(super) fn semantic_ir_identity(document: &Value) -> Option<(&str, &str)> {
    let package = document.get("package")?;
    Some((
        package.get("identity")?.as_str()?,
        package.get("version")?.as_str()?,
    ))
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
    Ok(read_semantic_ir(&document, budget)?)
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
// string-edge: reads a selected domain package document's member identity text.
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
// string-edge: reads a selected domain package document's value type.
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
// string-edge: reads a selected domain package document's node id prefix.
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
    let (identity, version) = semantic_ir_identity(document).ok_or_else(ModelRefusal::malformed)?;
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
        version: version.into(),
        ..DomainModel::default()
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
    // string-edge: reads a selected domain package document's type reference prefix.
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
// string-edge: reads a selected domain package document's supertype references.
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
