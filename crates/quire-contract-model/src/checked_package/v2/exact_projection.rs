// SPDX-License-Identifier: AGPL-3.0-or-later
//! Read-only projection of admitted checked types into Exact value types.

use super::extension::integer_range_bounds;
use super::model_members::DeclarationForm;
use super::model_members::{CollectionKind as CheckedCollectionKind, MemberType};
use super::structural::{reference_type_target_typed, typed_aggregate};
use super::{
    BoundedDomainForm, CheckedNodeId, CheckedNodeKind, CheckedNodeOwner, CheckedPackageV2,
    CheckedTerm, CompositeTypeForm, ModelForm, ScalarTypeForm,
};
use quire_exact::{
    CardinalityBound, CollectionKind, CollectionType, EffectiveId, FieldDeclaration, Integer,
    IntegerInterval, MemberId, MemberOwner, NodeKey, Presence, UnionMember, UnitId, ValueType,
};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// The Exact identity produced for one selected declaration form.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SelectedExactIdentity {
    /// A selected record declaration's structural key.
    Record(NodeKey),
    /// A selected object declaration's effective identity.
    Object(EffectiveId),
    /// A declaration-only union key; insufficient for complete projection.
    Union(NodeKey),
    /// A selected declared unit's identity.
    DeclaredUnit(UnitId),
}

/// The selected declaration form paired with its Exact identity.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SelectedExactDeclarationForm {
    /// `model/record_value_type`.
    RecordValueType,
    /// `model/object_type`.
    ObjectType,
    /// `composite_type/union`.
    Union,
    /// `scalar_type/unit`.
    DeclaredUnit,
}

impl SelectedExactDeclarationForm {
    fn matches(self, identity: &SelectedExactIdentity) -> bool {
        matches!(
            (self, identity),
            (Self::RecordValueType, SelectedExactIdentity::Record(_))
                | (Self::ObjectType, SelectedExactIdentity::Object(_))
                | (Self::Union, SelectedExactIdentity::Union(_))
                | (
                    Self::DeclaredUnit,
                    SelectedExactIdentity::DeclaredUnit(UnitId::Declared(_))
                )
        )
    }

    fn matches_admitted(self, form: AdmittedDeclarationForm) -> bool {
        matches!(
            (self, form),
            (
                Self::RecordValueType,
                AdmittedDeclarationForm::Model(DeclarationForm::RecordValueType)
            ) | (
                Self::ObjectType,
                AdmittedDeclarationForm::Model(DeclarationForm::ObjectType)
            ) | (Self::DeclaredUnit, AdmittedDeclarationForm::DeclaredUnit)
        )
    }
}

#[derive(Clone, Copy)]
enum AdmittedDeclarationForm {
    Model(DeclarationForm),
    DeclaredUnit,
}

fn admitted_selected_declaration<'a>(
    package: &'a CheckedPackageV2,
    key: &CheckedNodeId,
) -> Option<(&'a str, &'a str, AdmittedDeclarationForm)> {
    if let Some((document, original, form)) = package.models.selected_declaration(key) {
        return Some((document, original, AdmittedDeclarationForm::Model(form)));
    }
    if package.node_kind(key) != Some(CheckedNodeKind::ScalarType(ScalarTypeForm::Unit)) {
        return None;
    }
    let checked = package.node(key)?;
    let CheckedNodeOwner::Model { identity, node } = checked.owner.as_ref()? else {
        return None;
    };
    package
        .models
        .selected_models()
        .iter()
        .any(|model| model.identity.as_ref() == identity.as_ref() && model.units.contains_key(node))
        .then_some((identity, node, AdmittedDeclarationForm::DeclaredUnit))
}

/// One QSL-stage declaration row, keyed by selected original IR identity.
///
/// IR validates this row's structural join. The opaque Exact identity's
/// provenance remains a producer fact; constructing a row in another caller
/// does not make that identity authoritative.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedExactDeclarationRow {
    /// The full structural node identity in the admitted package.
    pub structural_node: CheckedNodeId,
    /// The selected domain document identity.
    pub selected_document: Box<str>,
    /// The selected declaration form.
    pub form: SelectedExactDeclarationForm,
    /// The original IR declaration identity in that document.
    pub original_identity: Box<str>,
    /// The identity issued by the QSL checking/model stage.
    pub exact_identity: SelectedExactIdentity,
}

impl SelectedExactDeclarationRow {
    /// Carry a complete producer row; no bare Exact identity is accepted.
    pub fn new(
        structural_node: CheckedNodeId,
        selected_document: impl Into<Box<str>>,
        form: SelectedExactDeclarationForm,
        original_identity: impl Into<Box<str>>,
        exact_identity: SelectedExactIdentity,
    ) -> Self {
        Self {
            structural_node,
            selected_document: selected_document.into(),
            form,
            original_identity: original_identity.into(),
            exact_identity,
        }
    }
}

/// One QSL-stage record field row, keyed by its selected original IR identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedExactFieldRow {
    /// The selected record's structural node identity.
    pub record_node: CheckedNodeId,
    /// The selected domain document identity.
    pub selected_document: Box<str>,
    /// The original IR record declaration identity.
    pub record_original_identity: Box<str>,
    /// The selected IR field identity, including inherited fields.
    pub field_original_identity: Box<str>,
    /// The field identity issued by the QSL checking/model stage.
    pub member_id: MemberId,
}

impl SelectedExactFieldRow {
    /// Carry a complete producer field row; no bare member ID is accepted.
    pub fn new(
        record_node: CheckedNodeId,
        selected_document: impl Into<Box<str>>,
        record_original_identity: impl Into<Box<str>>,
        field_original_identity: impl Into<Box<str>>,
        member_id: MemberId,
    ) -> Self {
        Self {
            record_node,
            selected_document: selected_document.into(),
            record_original_identity: record_original_identity.into(),
            field_original_identity: field_original_identity.into(),
            member_id,
        }
    }
}

/// One selected model tuple's QSL-stage Exact declaration identity.
///
/// The tuple form is fixed by this row type. Construction carries a producer
/// claim; a later admitted join must validate it before a runtime read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedTupleDeclarationRow {
    /// The tuple declaration's admitted structural node.
    pub structural_node: CheckedNodeId,
    /// The selected domain document identity.
    pub selected_document: Box<str>,
    /// The tuple's original identity in that document.
    pub original_identity: Box<str>,
    /// The tuple key issued by the QSL checking/model stage.
    pub node_key: NodeKey,
}

impl SelectedTupleDeclarationRow {
    /// Carry the complete selected tuple identity claim.
    pub fn new(
        structural_node: CheckedNodeId,
        selected_document: impl Into<Box<str>>,
        original_identity: impl Into<Box<str>>,
        node_key: NodeKey,
    ) -> Self {
        Self {
            structural_node,
            selected_document: selected_document.into(),
            original_identity: original_identity.into(),
            node_key,
        }
    }
}

/// One source-owned declared unit's QSL-stage Exact identity.
///
/// The source-unit form is fixed by this row type. An admitted join must
/// verify the selected source, structural node, original declaration and
/// `UnitId::Declared` domain before exposing a unit view.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedSourceUnitRow {
    /// The declared unit's admitted structural node.
    pub structural_node: CheckedNodeId,
    /// Authority of the selected raw source.
    pub source_authority: Box<str>,
    /// Identity of the selected raw source.
    pub source_identity: Box<str>,
    /// Original qualified declaration segments in that source.
    pub qualified_declaration: Vec<Box<str>>,
    /// The declared unit identity issued by the QSL checking stage.
    pub unit_id: UnitId,
}

impl SelectedSourceUnitRow {
    /// Carry the complete selected source-unit identity claim.
    pub fn new(
        structural_node: CheckedNodeId,
        source_authority: impl Into<Box<str>>,
        source_identity: impl Into<Box<str>>,
        qualified_declaration: Vec<Box<str>>,
        unit_id: UnitId,
    ) -> Self {
        Self {
            structural_node,
            source_authority: source_authority.into(),
            source_identity: source_identity.into(),
            qualified_declaration,
            unit_id,
        }
    }
}

/// One source-owned enum declaration's QSL-stage Exact identity.
///
/// The enum-declaration form is fixed by this row type. A later admitted join
/// must verify its selected source and original qualified declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedEnumDeclarationRow {
    /// The enum declaration's admitted structural node.
    pub structural_node: CheckedNodeId,
    /// Authority of the selected raw source.
    pub source_authority: Box<str>,
    /// Identity of the selected raw source.
    pub source_identity: Box<str>,
    /// Original qualified declaration segments in that source.
    pub qualified_declaration: Vec<Box<str>>,
    /// The enum key issued by the QSL checking stage.
    pub node_key: NodeKey,
}

impl SelectedEnumDeclarationRow {
    /// Carry the complete selected enum-declaration identity claim.
    pub fn new(
        structural_node: CheckedNodeId,
        source_authority: impl Into<Box<str>>,
        source_identity: impl Into<Box<str>>,
        qualified_declaration: Vec<Box<str>>,
        node_key: NodeKey,
    ) -> Self {
        Self {
            structural_node,
            source_authority: source_authority.into(),
            source_identity: source_identity.into(),
            qualified_declaration,
            node_key,
        }
    }
}

/// One sealed union member binding and its positional payload types.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedUnionMemberRow {
    /// The QSL-issued sealed member identity.
    pub member: UnionMember,
    /// Final-emission IR payload type nodes in declared position order.
    pub payload_nodes: Vec<CheckedNodeId>,
    /// Payload types in declared position order.
    pub payload: Vec<ValueType>,
}

impl SelectedUnionMemberRow {
    /// Carry one sealed member and its complete positional payload types.
    pub fn new(
        member: UnionMember,
        payload_nodes: Vec<CheckedNodeId>,
        payload: Vec<ValueType>,
    ) -> Self {
        Self {
            member,
            payload_nodes,
            payload,
        }
    }
}

/// One source-owned selected union's complete QSL-stage declaration witness.
///
/// IR validates its structural join and member census. Opaque Exact identities
/// and payload types remain producer facts; caller-created rows are not QSL
/// authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedSourceUnionRow {
    /// The union declaration's admitted structural node.
    pub structural_node: CheckedNodeId,
    /// Authority of the selected raw source.
    pub source_authority: Box<str>,
    /// Identity of the selected raw source.
    pub source_identity: Box<str>,
    /// Original qualified declaration segments in that source.
    pub qualified_declaration: Vec<Box<str>>,
    /// QSL's union type handle used by `ValueType::Composite`.
    pub type_handle: NodeKey,
    /// QSL's settled union declaration key used by sealed member bindings.
    pub settled_key: NodeKey,
    /// Every sealed member and ordered payload type, in declaration order.
    pub members: Vec<SelectedUnionMemberRow>,
}

/// One QSL-stage member of a source-owned record, in declaration order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedSourceRecordMemberRow {
    /// Original declared field name in the selected source.
    pub original_member_identity: Box<str>,
    /// Name of the admitted record binding.
    pub admitted_name: Box<str>,
    /// Declared presence, independent of the field's Exact value type.
    pub presence: Presence,
    /// Final emitted type node referenced by the binding.
    pub type_node: CheckedNodeId,
    /// QSL-issued Exact value type after presence handling.
    pub value_type: ValueType,
}

/// One complete QSL-stage source-record declaration and ordered member witness.
///
/// IR checks admitted structural correspondence; caller-created opaque Exact
/// keys and types do not thereby acquire QSL producer authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedSourceRecordRow {
    /// Final admitted record declaration node.
    pub structural_node: CheckedNodeId,
    /// Selected raw source authority.
    pub source_authority: Box<str>,
    /// Selected raw source identity.
    pub source_identity: Box<str>,
    /// Original qualified declaration segments.
    pub qualified_declaration: Vec<Box<str>>,
    /// QSL-issued Exact record type handle.
    pub record_key: NodeKey,
    /// Every declared member in producer order.
    pub members: Vec<SelectedSourceRecordMemberRow>,
}

impl SelectedSourceUnionRow {
    /// Carry the complete selected source-union claim.
    pub fn new(
        structural_node: CheckedNodeId,
        source_authority: impl Into<Box<str>>,
        source_identity: impl Into<Box<str>>,
        qualified_declaration: Vec<Box<str>>,
        type_handle: NodeKey,
        settled_key: NodeKey,
        members: Vec<SelectedUnionMemberRow>,
    ) -> Self {
        Self {
            structural_node,
            source_authority: source_authority.into(),
            source_identity: source_identity.into(),
            qualified_declaration,
            type_handle,
            settled_key,
            members,
        }
    }
}

fn source_union_row_matches(package: &CheckedPackageV2, row: &SelectedSourceUnionRow) -> bool {
    let id = &row.structural_node;
    let Some(node) = package.node(id) else {
        return false;
    };
    let selected_source = package.lock().sources.iter().any(|source| {
        source.authority == row.source_authority && source.identity == row.source_identity
    });
    let matching_owner = matches!(
        &node.owner,
        Some(CheckedNodeOwner::Source { authority, identity })
            if authority == &row.source_authority && identity == &row.source_identity
    );
    let matching_declaration = node
        .declaration
        .as_ref()
        .is_some_and(|declaration| declaration.qualified_name == row.qualified_declaration);
    let admitted_members = package.term(id).and_then(typed_aggregate);
    package.node_kind(id) == Some(CheckedNodeKind::CompositeType(CompositeTypeForm::Union))
        && selected_source
        && matching_owner
        && matching_declaration
        && admitted_members.as_ref().is_some_and(|members| {
            members.len() == row.members.len()
                && members
                    .iter()
                    .zip(&row.members)
                    .all(|(admitted, supplied)| {
                        let CheckedTerm::Binding { name, value } = admitted else {
                            return false;
                        };
                    name.as_ref() == supplied.member.identifier()
                        && typed_aggregate(value).is_some_and(|payload| {
                            payload.len() == supplied.payload.len()
                                && payload.len() == supplied.payload_nodes.len()
                                && payload.iter().zip(&supplied.payload_nodes).all(
                                    |(admitted, claimed)| {
                                        matches!(admitted, CheckedTerm::Reference { target } if target == claimed)
                                    },
                                )
                        })
                            && supplied.member.declaration() == &row.settled_key
                            && supplied.member.variant().owner() == &row.settled_key
                        && supplied.member.variant().case() == name.as_ref()
                    })
        })
}

fn source_record_row_matches(package: &CheckedPackageV2, row: &SelectedSourceRecordRow) -> bool {
    let Some(node) = package.node(&row.structural_node) else {
        return false;
    };
    let selected_source = package.lock().sources.iter().any(|source| {
        source.authority == row.source_authority && source.identity == row.source_identity
    });
    let matching_owner = matches!(
        &node.owner,
        Some(CheckedNodeOwner::Source { authority, identity })
            if authority == &row.source_authority && identity == &row.source_identity
    );
    let matching_declaration = node
        .declaration
        .as_ref()
        .is_some_and(|declaration| declaration.qualified_name == row.qualified_declaration);
    let Some(bindings) = package.term(&row.structural_node).and_then(typed_aggregate) else {
        return false;
    };
    let mut original_names = BTreeSet::new();
    package.node_kind(&row.structural_node)
        == Some(CheckedNodeKind::CompositeType(CompositeTypeForm::Record))
        && selected_source
        && matching_owner
        && matching_declaration
        && bindings.len() == row.members.len()
        && bindings
            .iter()
            .zip(&row.members)
            .all(|(binding, supplied)| {
                let CheckedTerm::Binding { name, value } = binding else {
                    return false;
                };
                if supplied.original_member_identity.is_empty()
                    || !original_names.insert(supplied.original_member_identity.as_ref())
                    || name.as_ref() != supplied.admitted_name.as_ref()
                {
                    return false;
                }
                let (optional, target) = match value.as_ref() {
                    CheckedTerm::Reference { target } => (false, target),
                    CheckedTerm::Aggregate { members } => {
                        let [CheckedTerm::Binding { name, value }] = members.as_slice() else {
                            return false;
                        };
                        if name.internal_kind() != super::TermBindingName::Optional {
                            return false;
                        }
                        let CheckedTerm::Reference { target } = value.as_ref() else {
                            return false;
                        };
                        (true, target)
                    }
                    _ => return false,
                };
                target == &supplied.type_node
                    && package.node(target).is_some()
                    && optional == (supplied.presence == Presence::Optional)
            })
}

/// One source-owned enum case's QSL-stage Exact member identity.
///
/// The enum-member form is fixed by this row type. A later admitted join must
/// verify the parent declaration, its selected source and this exact case.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedEnumMemberRow {
    /// The enum member's admitted structural node.
    pub structural_node: CheckedNodeId,
    /// Authority of the selected raw source declaring its enum.
    pub source_authority: Box<str>,
    /// Identity of the selected raw source declaring its enum.
    pub source_identity: Box<str>,
    /// The selected enum declaration's admitted structural node.
    pub declaration_node: CheckedNodeId,
    /// Original qualified identity of the selected enum declaration.
    pub declaration_qualified_name: Vec<Box<str>>,
    /// The original member case in that enum declaration.
    pub case: Box<str>,
    /// The member key issued by the QSL checking stage.
    pub member_key: NodeKey,
}

impl SelectedEnumMemberRow {
    /// Carry the complete selected enum-member identity claim.
    pub fn new(
        structural_node: CheckedNodeId,
        source_authority: impl Into<Box<str>>,
        source_identity: impl Into<Box<str>>,
        declaration_node: CheckedNodeId,
        declaration_qualified_name: Vec<Box<str>>,
        case: impl Into<Box<str>>,
        member_key: NodeKey,
    ) -> Self {
        Self {
            structural_node,
            source_authority: source_authority.into(),
            source_identity: source_identity.into(),
            declaration_node,
            declaration_qualified_name,
            case: case.into(),
            member_key,
        }
    }
}

/// Selected identities supplied by the checking stage alongside a package.
///
/// An empty selection supports scalar reads. Identity-bearing reads refuse
/// until the selected stage supplies a complete identity witness.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SelectedExactIdentities {
    declarations: Arc<BTreeMap<CheckedNodeId, SelectedExactDeclarationRow>>,
    fields: Arc<Vec<SelectedExactFieldRow>>,
    admitted_tuples: BTreeMap<CheckedNodeId, NodeKey>,
    admitted_source_units: BTreeMap<CheckedNodeId, UnitId>,
    source_unions: Arc<BTreeMap<CheckedNodeId, SelectedSourceUnionRow>>,
    source_records: Arc<BTreeMap<CheckedNodeId, Arc<SelectedSourceRecordRow>>>,
}

impl SelectedExactIdentities {
    /// An empty selection for reads of types without declaration identities.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Validate the stage's selected rows against one admitted package.
    ///
    /// This checks structural, selected-document, form and original-identity
    /// joins. IR cannot authenticate the opaque Exact payload. Production
    /// callers obtain these rows from the QSL checking/model identity stage;
    /// caller-created rows alone grant no authority.
    pub fn from_stage_rows(
        package: &CheckedPackageV2,
        declarations: impl IntoIterator<Item = SelectedExactDeclarationRow>,
        fields: impl IntoIterator<Item = SelectedExactFieldRow>,
    ) -> Result<Self, ExactTypeProjectionError> {
        let mut selected = Self::empty();
        let mut owners = BTreeSet::new();
        let mut node_identities = BTreeSet::new();
        let mut object_identities = BTreeSet::new();
        for row in declarations {
            let node = &row.structural_node;
            let Some((document, original, form)) = admitted_selected_declaration(package, node)
            else {
                return Err(selected_declaration_error(package, node));
            };
            if document != row.selected_document.as_ref()
                || original != row.original_identity.as_ref()
                || !row.form.matches_admitted(form)
                || !row.form.matches(&row.exact_identity)
            {
                return Err(witness_error(
                    node,
                    ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                ));
            }
            let unique_exact = match &row.exact_identity {
                SelectedExactIdentity::Record(key)
                | SelectedExactIdentity::Union(key)
                | SelectedExactIdentity::DeclaredUnit(UnitId::Declared(key)) => {
                    node_identities.insert(key.clone())
                }
                SelectedExactIdentity::Object(effective) => {
                    object_identities.insert(effective.clone())
                }
                SelectedExactIdentity::DeclaredUnit(UnitId::Compound(_)) => false,
            };
            if !unique_exact
                || !owners.insert((
                    row.selected_document.clone(),
                    row.form,
                    row.original_identity.clone(),
                ))
                || selected.declarations.contains_key(node)
            {
                return Err(witness_error(
                    node,
                    ExactTypeProjectionErrorCode::ConflictingIdentityWitness,
                ));
            }
            Arc::make_mut(&mut selected.declarations).insert(node.clone(), row);
        }
        let mut seen_fields = BTreeSet::new();
        let mut seen_members = BTreeSet::new();
        for row in fields {
            let node = &row.record_node;
            let Some((document, original, form)) = package.models.selected_declaration(node) else {
                return Err(selected_declaration_error(package, node));
            };
            if document != row.selected_document.as_ref()
                || original != row.record_original_identity.as_ref()
                || form != DeclarationForm::RecordValueType
                || !package
                    .models
                    .selected_record_field(node, &row.field_original_identity)
            {
                return Err(witness_error(
                    node,
                    ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                ));
            }
            let Some(declaration) = selected.declarations.get(node) else {
                return Err(witness_error(
                    node,
                    ExactTypeProjectionErrorCode::UnavailableIdentityWitness,
                ));
            };
            let SelectedExactIdentity::Record(record_key) = &declaration.exact_identity else {
                return Err(witness_error(
                    node,
                    ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                ));
            };
            if row.member_id.owner() != &MemberOwner::Node(record_key.clone()) {
                return Err(witness_error(
                    node,
                    ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                ));
            }
            if !seen_fields.insert((node.clone(), row.field_original_identity.clone()))
                || !seen_members.insert(row.member_id.clone())
            {
                return Err(witness_error(
                    node,
                    ExactTypeProjectionErrorCode::ConflictingIdentityWitness,
                ));
            }
            Arc::make_mut(&mut selected.fields).push(row);
        }
        Ok(selected)
    }

    /// The validated declaration rows, keyed by structural node identity.
    pub fn declaration_rows(&self) -> impl Iterator<Item = &SelectedExactDeclarationRow> {
        self.declarations.values()
    }

    /// The validated effective-field rows in producer order.
    pub fn field_rows(&self) -> &[SelectedExactFieldRow] {
        &self.fields
    }

    /// Join complete source-owned union rows to this package's selected lock.
    ///
    /// The source stage supplies opaque keys, sealed members and payload types.
    /// This check establishes only their structural correspondence with the
    /// admitted package; it does not authenticate a caller as that stage.
    pub fn with_source_union_rows(
        &self,
        package: &CheckedPackageV2,
        rows: impl IntoIterator<Item = SelectedSourceUnionRow>,
    ) -> Result<Self, ExactTypeProjectionError> {
        let mut selected = self.clone();
        let mut handles = self
            .declarations
            .values()
            .filter_map(|row| match &row.exact_identity {
                SelectedExactIdentity::Record(key)
                | SelectedExactIdentity::Union(key)
                | SelectedExactIdentity::DeclaredUnit(UnitId::Declared(key)) => Some(key.clone()),
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        let mut settled_keys = selected
            .source_unions
            .values()
            .map(|row| row.settled_key.clone())
            .collect::<BTreeSet<_>>();
        handles.extend(
            selected
                .source_unions
                .values()
                .map(|row| row.type_handle.clone()),
        );
        handles.extend(
            selected
                .source_records
                .values()
                .map(|row| row.record_key.clone()),
        );
        for row in rows {
            let id = &row.structural_node;
            if package.node(id).is_none() {
                return Err(selected_declaration_error(package, id));
            }
            if !source_union_row_matches(package, &row) {
                return Err(witness_error(
                    id,
                    ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                ));
            }
            if selected.source_unions.contains_key(id)
                || !handles.insert(row.type_handle.clone())
                || !settled_keys.insert(row.settled_key.clone())
            {
                return Err(witness_error(
                    id,
                    ExactTypeProjectionErrorCode::ConflictingIdentityWitness,
                ));
            }
            Arc::make_mut(&mut selected.source_unions).insert(id.clone(), row);
        }
        Ok(selected)
    }

    /// Validated source union rows, keyed by the full admitted structural node.
    pub fn source_union_rows(&self) -> impl Iterator<Item = &SelectedSourceUnionRow> {
        self.source_unions.values()
    }

    /// Join complete source-owned record rows to this package under a finite allowance.
    ///
    /// The join checks admitted structural facts. It does not authenticate
    /// caller-created opaque Exact keys or member value types as QSL output.
    pub fn with_source_record_rows(
        &self,
        package: &CheckedPackageV2,
        rows: impl IntoIterator<Item = SelectedSourceRecordRow>,
        work_limit: u64,
    ) -> Result<Self, ExactTypeProjectionError> {
        let mut selected = self.clone();
        let mut work = 0;
        for row in rows {
            let id = &row.structural_node;
            charge_work(id, id, "source_record_row", &mut work, work_limit)?;
            for _ in &package.lock().sources {
                charge_work(id, id, "selected_source", &mut work, work_limit)?;
            }
            for segment in &row.qualified_declaration {
                charge_work_amount(
                    id,
                    id,
                    "record_declaration",
                    &mut work,
                    work_limit,
                    u64::try_from(segment.len()).unwrap_or(u64::MAX),
                )?;
            }
            for member in &row.members {
                charge_work(
                    id,
                    &member.type_node,
                    "source_record_member",
                    &mut work,
                    work_limit,
                )?;
                charge_work_amount(
                    id,
                    &member.type_node,
                    "source_record_member_identity",
                    &mut work,
                    work_limit,
                    u64::try_from(
                        member
                            .original_member_identity
                            .len()
                            .saturating_add(member.admitted_name.len()),
                    )
                    .unwrap_or(u64::MAX),
                )?;
            }
            if package.node(id).is_none() {
                let mut error = selected_declaration_error(package, id);
                error.work = work;
                return Err(error);
            }
            if !source_record_row_matches(package, &row) {
                let mut error =
                    witness_error(id, ExactTypeProjectionErrorCode::MismatchedIdentityWitness);
                error.work = work;
                return Err(error);
            }
            let mut conflicting_handle = false;
            for declaration in selected.declarations.values() {
                charge_work(
                    id,
                    &declaration.structural_node,
                    "record_handle_join",
                    &mut work,
                    work_limit,
                )?;
                conflicting_handle |= matches!(
                    &declaration.exact_identity,
                    SelectedExactIdentity::Record(key)
                        | SelectedExactIdentity::Union(key)
                        | SelectedExactIdentity::DeclaredUnit(UnitId::Declared(key))
                        if key == &row.record_key
                );
            }
            for union in selected.source_unions.values() {
                charge_work(
                    id,
                    &union.structural_node,
                    "record_handle_join",
                    &mut work,
                    work_limit,
                )?;
                conflicting_handle |= union.type_handle == row.record_key;
            }
            for record in selected.source_records.values() {
                charge_work(
                    id,
                    &record.structural_node,
                    "record_handle_join",
                    &mut work,
                    work_limit,
                )?;
                conflicting_handle |= record.record_key == row.record_key;
            }
            if selected.source_records.contains_key(id) || conflicting_handle {
                let mut error =
                    witness_error(id, ExactTypeProjectionErrorCode::ConflictingIdentityWitness);
                error.work = work;
                return Err(error);
            }
            Arc::make_mut(&mut selected.source_records).insert(id.clone(), Arc::new(row));
        }
        Ok(selected)
    }

    /// Validated source record rows, keyed by the final admitted node.
    pub fn source_record_rows(&self) -> impl Iterator<Item = &SelectedSourceRecordRow> {
        self.source_records.values().map(Arc::as_ref)
    }

    pub(super) fn with_admitted_type_rows(
        &self,
        tuples: &BTreeMap<CheckedNodeId, &SelectedTupleDeclarationRow>,
        source_units: &BTreeMap<CheckedNodeId, &SelectedSourceUnitRow>,
    ) -> Self {
        let mut selected = self.clone();
        selected.admitted_tuples = tuples
            .iter()
            .map(|(id, row)| (id.clone(), row.node_key.clone()))
            .collect();
        selected.admitted_source_units = source_units
            .iter()
            .map(|(id, row)| (id.clone(), row.unit_id.clone()))
            .collect();
        selected
    }

    pub(super) fn declaration_identity(
        &self,
        package: &CheckedPackageV2,
        key: &CheckedNodeId,
    ) -> Result<&SelectedExactIdentity, ExactTypeProjectionErrorCode> {
        let (document, original, form) = admitted_selected_declaration(package, key)
            .ok_or(ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration)?;
        let row = self
            .declarations
            .get(key)
            .ok_or(ExactTypeProjectionErrorCode::UnavailableIdentityWitness)?;
        if document != row.selected_document.as_ref()
            || original != row.original_identity.as_ref()
            || !row.form.matches_admitted(form)
            || !row.form.matches(&row.exact_identity)
        {
            return Err(ExactTypeProjectionErrorCode::MismatchedIdentityWitness);
        }
        Ok(&row.exact_identity)
    }
}

/// One effective field of an admitted selected record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedRecordFieldView {
    ir_identity: Box<str>,
    name: Box<str>,
    member_id: MemberId,
    value_type: ValueType,
    presence: Presence,
}

impl SelectedRecordFieldView {
    /// The selected original IR field identity.
    pub fn ir_identity(&self) -> &str {
        &self.ir_identity
    }

    /// The selected field's display name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The witnessed Exact member identity.
    pub fn member_id(&self) -> &MemberId {
        &self.member_id
    }

    /// The complete Exact field type.
    pub fn value_type(&self) -> &ValueType {
        &self.value_type
    }

    /// Whether this field is required or optional.
    pub fn presence(&self) -> Presence {
        self.presence
    }
}

/// A complete admitted selected record in effective declaration order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedRecordView {
    record_key: NodeKey,
    fields: Vec<SelectedRecordFieldView>,
}

impl SelectedRecordView {
    /// The witnessed Exact record declaration identity.
    pub fn record_key(&self) -> &NodeKey {
        &self.record_key
    }

    /// Every effective field, including inherited and unread fields.
    pub fn fields(&self) -> &[SelectedRecordFieldView] {
        &self.fields
    }

    /// Complete Exact field declarations for supplied-record checking.
    pub fn field_declarations(&self) -> Vec<FieldDeclaration> {
        self.fields
            .iter()
            .map(|field| {
                FieldDeclaration::new(
                    field.member_id.clone(),
                    field.name.as_ref(),
                    field.value_type.clone(),
                    field.presence,
                )
            })
            .collect()
    }
}

/// One complete Exact value type read from an admitted package.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExactTypeProjection {
    node: CheckedNodeId,
    type_node: CheckedNodeId,
    value_type: ValueType,
    record_view: Option<SelectedRecordView>,
    source_record_view: Option<Arc<SelectedSourceRecordRow>>,
    work: u64,
}

impl ExactTypeProjection {
    /// The requested structural node.
    pub fn node(&self) -> &CheckedNodeId {
        &self.node
    }

    /// The semantic type node resolved from the admitted graph.
    pub fn type_node(&self) -> &CheckedNodeId {
        &self.type_node
    }

    /// The complete Exact type.
    pub fn value_type(&self) -> &ValueType {
        &self.value_type
    }

    /// The complete selected record view, when this projection is a record.
    pub fn record_view(&self) -> Option<&SelectedRecordView> {
        self.record_view.as_ref()
    }

    /// Complete structurally joined source record row, when this type is a source record.
    ///
    /// Its opaque key and member types remain QSL producer claims.
    pub fn source_record_view(&self) -> Option<&SelectedSourceRecordRow> {
        self.source_record_view.as_deref()
    }

    /// Work units charged by this complete projection.
    pub fn work(&self) -> u64 {
        self.work
    }
}

/// Stable reason a checked type cannot be projected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExactTypeProjectionErrorCode {
    /// The request does not resolve in this admitted package.
    UnknownNode,
    /// The node belongs to a different package.
    ForeignNode,
    /// A selected declaration has no authoritative identity row.
    UnavailableIdentityWitness,
    /// A selected declaration row disagrees with admission-retained facts.
    MismatchedIdentityWitness,
    /// Competing rows claim one selected declaration or structural node.
    ConflictingIdentityWitness,
    /// The selected declaration is not resolved in the admitted package.
    UnresolvedSelectedDeclaration,
    /// The requested type has no faithful Exact representation yet.
    UnsupportedExactType,
    /// The caller's finite projection work limit was consumed.
    ExhaustedWork,
}

impl ExactTypeProjectionErrorCode {
    /// Stable machine-readable spelling of the refusal.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnknownNode => "unknown_node",
            Self::ForeignNode => "foreign_node",
            Self::UnavailableIdentityWitness => "unavailable_identity_witness",
            Self::MismatchedIdentityWitness => "mismatched_identity_witness",
            Self::ConflictingIdentityWitness => "conflicting_identity_witness",
            Self::UnresolvedSelectedDeclaration => "unresolved_selected_declaration",
            Self::UnsupportedExactType => "unsupported_exact_type",
            Self::ExhaustedWork => "exhausted_work",
        }
    }

    /// Every refusal code exposed by this projection.
    pub const fn all() -> &'static [Self] {
        &[
            Self::UnknownNode,
            Self::ForeignNode,
            Self::UnavailableIdentityWitness,
            Self::MismatchedIdentityWitness,
            Self::ConflictingIdentityWitness,
            Self::UnresolvedSelectedDeclaration,
            Self::UnsupportedExactType,
            Self::ExhaustedWork,
        ]
    }

    /// Decodes an exact stable refusal spelling.
    pub fn from_code(code: &str) -> Option<Self> {
        Self::all()
            .iter()
            .copied()
            .find(|candidate| candidate.as_str() == code)
    }
}

/// A typed refusal or incomplete read; no partial value type is returned.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("Exact type projection refused: {code:?} at {form}")]
pub struct ExactTypeProjectionError {
    /// Stable refusal category.
    pub code: ExactTypeProjectionErrorCode,
    /// The structural node the caller requested.
    pub requested: Box<CheckedNodeId>,
    /// First type node whose projection failed, if it resolved.
    pub at: Option<Box<CheckedNodeId>>,
    /// Admitted form at the failure locus, or `unknown_node`.
    pub form: Box<str>,
    /// Work successfully charged before the refusal.
    pub work: u64,
}

fn witness_error(
    node: &CheckedNodeId,
    code: ExactTypeProjectionErrorCode,
) -> ExactTypeProjectionError {
    ExactTypeProjectionError {
        code,
        requested: Box::new(node.clone()),
        at: Some(Box::new(node.clone())),
        form: "selected_identity".into(),
        work: 0,
    }
}

fn selected_declaration_error(
    package: &CheckedPackageV2,
    node: &CheckedNodeId,
) -> ExactTypeProjectionError {
    let code = if package.node(node).is_some() {
        ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration
    } else if node.package == package.wire.name {
        ExactTypeProjectionErrorCode::UnknownNode
    } else {
        ExactTypeProjectionErrorCode::ForeignNode
    };
    witness_error(node, code)
}

fn charge_work(
    requested: &CheckedNodeId,
    at: &CheckedNodeId,
    form: &str,
    work: &mut u64,
    limit: u64,
) -> Result<(), ExactTypeProjectionError> {
    charge_work_amount(requested, at, form, work, limit, 1)
}

fn charge_work_amount(
    requested: &CheckedNodeId,
    at: &CheckedNodeId,
    form: &str,
    work: &mut u64,
    limit: u64,
    amount: u64,
) -> Result<(), ExactTypeProjectionError> {
    if amount > limit.saturating_sub(*work) {
        return Err(ExactTypeProjectionError {
            code: ExactTypeProjectionErrorCode::ExhaustedWork,
            requested: Box::new(requested.clone()),
            at: Some(Box::new(at.clone())),
            form: form.into(),
            work: *work,
        });
    }
    *work += amount;
    Ok(())
}

fn exact_integer_interval(
    lower: &str,
    upper: &str,
) -> Result<IntegerInterval, ExactTypeProjectionErrorCode> {
    let lower = lower
        .parse::<Integer>()
        .map_err(|_| ExactTypeProjectionErrorCode::UnsupportedExactType)?;
    let upper = upper
        .parse::<Integer>()
        .map_err(|_| ExactTypeProjectionErrorCode::UnsupportedExactType)?;
    IntegerInterval::new(lower, upper)
        .map_err(|_| ExactTypeProjectionErrorCode::UnsupportedExactType)
}

pub(super) fn project_record_field_type(
    package: &CheckedPackageV2,
    identities: &SelectedExactIdentities,
    member_type: &MemberType,
) -> Result<ValueType, (ExactTypeProjectionErrorCode, Option<CheckedNodeId>)> {
    match member_type {
        MemberType::Boolean => Ok(ValueType::Boolean),
        MemberType::Integer => Ok(ValueType::Integer),
        MemberType::String => Ok(ValueType::String),
        MemberType::Uuid => Ok(ValueType::Uuid),
        MemberType::Timestamp => Ok(ValueType::Timestamp),
        MemberType::IntRange(bounds) => {
            exact_integer_interval(&bounds.lower.to_string(), &bounds.upper.to_string())
                .map(ValueType::Int)
                .map_err(|code| (code, None))
        }
        MemberType::Reference(target) => {
            if package.node_kind(target) != Some(CheckedNodeKind::Model(ModelForm::ObjectType)) {
                return Err((
                    ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration,
                    Some(target.clone()),
                ));
            }
            match identities
                .declaration_identity(package, target)
                .map_err(|code| (code, Some(target.clone())))?
            {
                SelectedExactIdentity::Object(effective) => {
                    Ok(ValueType::Reference(effective.clone()))
                }
                _ => Err((
                    ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                    Some(target.clone()),
                )),
            }
        }
        MemberType::Record(target) => {
            if package.node_kind(target) != Some(CheckedNodeKind::Model(ModelForm::RecordValueType))
            {
                return Err((
                    ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration,
                    Some(target.clone()),
                ));
            }
            match identities
                .declaration_identity(package, target)
                .map_err(|code| (code, Some(target.clone())))?
            {
                SelectedExactIdentity::Record(key) => Ok(ValueType::Composite(key.clone())),
                _ => Err((
                    ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                    Some(target.clone()),
                )),
            }
        }
        MemberType::Quantity(target) => {
            if package.node_kind(target) != Some(CheckedNodeKind::ScalarType(ScalarTypeForm::Unit))
            {
                return Err((
                    ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration,
                    Some(target.clone()),
                ));
            }
            if let Some(unit) = identities.admitted_source_units.get(target) {
                return Ok(ValueType::Quantity(unit.clone()));
            }
            match identities
                .declaration_identity(package, target)
                .map_err(|code| (code, Some(target.clone())))?
            {
                SelectedExactIdentity::DeclaredUnit(unit @ UnitId::Declared(_)) => {
                    Ok(ValueType::Quantity(unit.clone()))
                }
                _ => Err((
                    ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                    Some(target.clone()),
                )),
            }
        }
        MemberType::Tuple(target) => identities
            .admitted_tuples
            .get(target)
            .cloned()
            .map(ValueType::Composite)
            .ok_or((
                ExactTypeProjectionErrorCode::UnavailableIdentityWitness,
                Some(target.clone()),
            )),
        MemberType::Float32
        | MemberType::Float64
        | MemberType::Bytes
        | MemberType::JsonObject
        | MemberType::Clock
        | MemberType::RationalRange(_)
        | MemberType::CarriedRational(..)
        | MemberType::TextBounds(_)
        | MemberType::Variant(_)
        | MemberType::Option(_)
        | MemberType::Collection { .. } => {
            Err((ExactTypeProjectionErrorCode::UnsupportedExactType, None))
        }
    }
}

enum FieldWrapper {
    Option,
    Collection(CheckedCollectionKind, Option<(u64, u64)>),
}

fn project_wrapped_record_field_type(
    package: &CheckedPackageV2,
    identities: &SelectedExactIdentities,
    requested: &CheckedNodeId,
    record: &CheckedNodeId,
    member_type: &MemberType,
    work: &mut u64,
    work_limit: u64,
) -> Result<ValueType, ExactTypeProjectionError> {
    let mut wrappers = Vec::new();
    let mut leaf = member_type;
    loop {
        match leaf {
            MemberType::Option(inner) => {
                charge_work(requested, record, "record_field_option", work, work_limit)?;
                wrappers.push(FieldWrapper::Option);
                leaf = inner;
            }
            MemberType::Collection {
                kind,
                element,
                bounds,
            } => {
                charge_work(
                    requested,
                    record,
                    "record_field_collection",
                    work,
                    work_limit,
                )?;
                wrappers.push(FieldWrapper::Collection(*kind, *bounds));
                leaf = element;
            }
            _ => break,
        }
    }
    if let MemberType::Reference(target)
    | MemberType::Record(target)
    | MemberType::Quantity(target) = leaf
    {
        charge_work(requested, target, "record_field_target", work, work_limit)?;
    }
    let mut value =
        project_record_field_type(package, identities, leaf).map_err(|(code, target)| {
            let mut error = ExactTypeProjectionError {
                code,
                requested: Box::new(requested.clone()),
                at: Some(Box::new(record.clone())),
                form: "model/record_value_type".into(),
                work: *work,
            };
            if let Some(target) = target {
                error.form = package.node(&target).map_or_else(
                    || "unresolved_selected_declaration".into(),
                    |node| format!("{}/{}", node.node_tag, node.semantic_form).into(),
                );
                error.at = Some(Box::new(target));
            }
            error
        })?;
    for wrapper in wrappers.into_iter().rev() {
        value = match wrapper {
            FieldWrapper::Option => ValueType::option(value),
            FieldWrapper::Collection(kind, bounds) => {
                let kind = match kind {
                    CheckedCollectionKind::Sequence => CollectionKind::Sequence,
                    CheckedCollectionKind::Set => CollectionKind::Set,
                    CheckedCollectionKind::Bag => CollectionKind::Bag,
                    CheckedCollectionKind::OrderedSet => CollectionKind::OrderedSet,
                };
                let bound = bounds
                    .map(|(lower, upper)| CardinalityBound::new(lower, upper))
                    .transpose()
                    .map_err(|_| ExactTypeProjectionError {
                        code: ExactTypeProjectionErrorCode::UnsupportedExactType,
                        requested: Box::new(requested.clone()),
                        at: Some(Box::new(record.clone())),
                        form: "record_field_collection".into(),
                        work: *work,
                    })?;
                ValueType::collection(CollectionType::new(kind, value, bound))
            }
        };
    }
    Ok(value)
}

impl CheckedPackageV2 {
    /// Read one admitted node's semantic type without reparsing its source.
    ///
    /// The finite `work_limit` charges the requested node and every type node
    /// inspected. The selected identities are required for declaration-bearing
    /// types. A refusal never exposes a partial Exact type.
    pub fn project_exact_type(
        &self,
        requested: &CheckedNodeId,
        identities: &SelectedExactIdentities,
        work_limit: u64,
    ) -> Result<ExactTypeProjection, ExactTypeProjectionError> {
        let mut work = 0;
        charge_work(requested, requested, "node", &mut work, work_limit)?;
        let node = self
            .node(requested)
            .ok_or_else(|| ExactTypeProjectionError {
                code: if requested.package == self.wire.name {
                    ExactTypeProjectionErrorCode::UnknownNode
                } else {
                    ExactTypeProjectionErrorCode::ForeignNode
                },
                requested: Box::new(requested.clone()),
                at: None,
                form: "unknown_node".into(),
                work,
            })?;
        let type_id = &node.semantic_type;
        charge_work(requested, type_id, "semantic_type", &mut work, work_limit)?;
        let type_node = self.node(type_id).ok_or_else(|| ExactTypeProjectionError {
            code: ExactTypeProjectionErrorCode::UnknownNode,
            requested: Box::new(requested.clone()),
            at: Some(Box::new(type_id.clone())),
            form: "unknown_node".into(),
            work,
        })?;
        let kind = self
            .node_kind(type_id)
            .ok_or_else(|| ExactTypeProjectionError {
                code: ExactTypeProjectionErrorCode::UnknownNode,
                requested: Box::new(requested.clone()),
                at: Some(Box::new(type_id.clone())),
                form: "unknown_node".into(),
                work,
            })?;
        let range_bounds = (kind
            == CheckedNodeKind::BoundedDomain(BoundedDomainForm::IntegerRange))
        .then(|| self.term(type_id).and_then(integer_range_bounds))
        .flatten();
        if let Some((lower, upper)) = range_bounds {
            let bytes = lower.len().saturating_add(upper.len());
            charge_work_amount(
                requested,
                type_id,
                "bounded_domain/integer_range",
                &mut work,
                work_limit,
                u64::try_from(bytes).unwrap_or(u64::MAX),
            )?;
        }
        let refuse = |code, charged| ExactTypeProjectionError {
            code,
            requested: Box::new(requested.clone()),
            at: Some(Box::new(type_id.clone())),
            form: format!("{}/{}", type_node.node_tag, type_node.semantic_form).into(),
            work: charged,
        };
        let mut record_view = None;
        let mut source_record_view = None;
        let value_type = match kind {
            CheckedNodeKind::ScalarType(ScalarTypeForm::Boolean) => ValueType::Boolean,
            CheckedNodeKind::ScalarType(ScalarTypeForm::Integer) => ValueType::Integer,
            CheckedNodeKind::ScalarType(ScalarTypeForm::String) => ValueType::String,
            CheckedNodeKind::ScalarType(ScalarTypeForm::Uuid) => ValueType::Uuid,
            CheckedNodeKind::ScalarType(ScalarTypeForm::Timestamp) => ValueType::Timestamp,
            CheckedNodeKind::BoundedDomain(BoundedDomainForm::IntegerRange) => {
                let (lower, upper) = range_bounds.ok_or_else(|| {
                    refuse(ExactTypeProjectionErrorCode::UnsupportedExactType, work)
                })?;
                let interval =
                    exact_integer_interval(lower, upper).map_err(|code| refuse(code, work))?;
                ValueType::Int(interval)
            }
            CheckedNodeKind::CompositeType(CompositeTypeForm::Reference) => {
                let target = reference_type_target_typed(type_node).ok_or_else(|| {
                    refuse(
                        ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration,
                        work,
                    )
                })?;
                charge_work(
                    requested,
                    &target,
                    "reference_target",
                    &mut work,
                    work_limit,
                )?;
                let refuse_target = |code, charged| ExactTypeProjectionError {
                    code,
                    requested: Box::new(requested.clone()),
                    at: Some(Box::new(target.clone())),
                    form: self.node(&target).map_or_else(
                        || Box::<str>::from("unresolved_selected_declaration"),
                        |node| format!("{}/{}", node.node_tag, node.semantic_form).into(),
                    ),
                    work: charged,
                };
                if self.node_kind(&target) != Some(CheckedNodeKind::Model(ModelForm::ObjectType)) {
                    return Err(refuse_target(
                        ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration,
                        work,
                    ));
                }
                match identities
                    .declaration_identity(self, &target)
                    .map_err(|code| refuse_target(code, work))?
                {
                    SelectedExactIdentity::Object(effective) => {
                        ValueType::Reference(effective.clone())
                    }
                    _ => {
                        return Err(refuse_target(
                            ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                            work,
                        ))
                    }
                }
            }
            CheckedNodeKind::Model(ModelForm::RecordValueType) => {
                let identity = identities
                    .declaration_identity(self, type_id)
                    .map_err(|code| refuse(code, work))?;
                let SelectedExactIdentity::Record(record_key) = identity else {
                    return Err(refuse(
                        ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                        work,
                    ));
                };
                let (ambiguous, fields) =
                    self.models.selected_record_fields(type_id).ok_or_else(|| {
                        refuse(
                            ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration,
                            work,
                        )
                    })?;
                if ambiguous.is_some() {
                    return Err(refuse(
                        ExactTypeProjectionErrorCode::UnsupportedExactType,
                        work,
                    ));
                }
                let mut member_rows = BTreeMap::new();
                for row in identities.fields.iter() {
                    charge_work(
                        requested,
                        type_id,
                        "record_field_witness",
                        &mut work,
                        work_limit,
                    )?;
                    if row.record_node == *type_id {
                        member_rows.insert(row.field_original_identity.as_ref(), &row.member_id);
                    }
                }
                let mut projected_fields = Vec::new();
                for field in fields {
                    charge_work(requested, type_id, "record_field", &mut work, work_limit)?;
                    let member_id = member_rows.get(field.identity).ok_or_else(|| {
                        refuse(
                            ExactTypeProjectionErrorCode::UnavailableIdentityWitness,
                            work,
                        )
                    })?;
                    let member_type = field.member_type.ok_or_else(|| {
                        refuse(ExactTypeProjectionErrorCode::UnsupportedExactType, work)
                    })?;
                    let member_type = if field.optional {
                        match member_type {
                            MemberType::Option(inner) => inner.as_ref(),
                            _ => {
                                return Err(refuse(
                                    ExactTypeProjectionErrorCode::UnsupportedExactType,
                                    work,
                                ))
                            }
                        }
                    } else {
                        member_type
                    };
                    let value_type = project_wrapped_record_field_type(
                        self,
                        identities,
                        requested,
                        type_id,
                        member_type,
                        &mut work,
                        work_limit,
                    )?;
                    projected_fields.push(SelectedRecordFieldView {
                        ir_identity: field.identity.into(),
                        name: field.name.into(),
                        member_id: (*member_id).clone(),
                        value_type,
                        presence: if field.optional {
                            Presence::Optional
                        } else {
                            Presence::Required
                        },
                    });
                }
                record_view = Some(SelectedRecordView {
                    record_key: record_key.clone(),
                    fields: projected_fields,
                });
                ValueType::Composite(record_key.clone())
            }
            CheckedNodeKind::CompositeType(CompositeTypeForm::Record) => {
                let row = identities.source_records.get(type_id).ok_or_else(|| {
                    refuse(
                        ExactTypeProjectionErrorCode::UnavailableIdentityWitness,
                        work,
                    )
                })?;
                charge_work(requested, type_id, "source_record", &mut work, work_limit)?;
                for _ in &self.lock().sources {
                    charge_work(requested, type_id, "selected_source", &mut work, work_limit)?;
                }
                for segment in &row.qualified_declaration {
                    charge_work_amount(
                        requested,
                        type_id,
                        "record_declaration",
                        &mut work,
                        work_limit,
                        u64::try_from(segment.len()).unwrap_or(u64::MAX),
                    )?;
                }
                for member in &row.members {
                    charge_work(
                        requested,
                        &member.type_node,
                        "source_record_member",
                        &mut work,
                        work_limit,
                    )?;
                    charge_work_amount(
                        requested,
                        &member.type_node,
                        "source_record_member_identity",
                        &mut work,
                        work_limit,
                        u64::try_from(
                            member
                                .original_member_identity
                                .len()
                                .saturating_add(member.admitted_name.len()),
                        )
                        .unwrap_or(u64::MAX),
                    )?;
                }
                if !source_record_row_matches(self, row) {
                    return Err(refuse(
                        ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                        work,
                    ));
                }
                source_record_view = Some(Arc::clone(row));
                ValueType::Composite(row.record_key.clone())
            }
            CheckedNodeKind::CompositeType(CompositeTypeForm::Union) => {
                let row = identities.source_unions.get(type_id).ok_or_else(|| {
                    refuse(
                        ExactTypeProjectionErrorCode::UnavailableIdentityWitness,
                        work,
                    )
                })?;
                for member in &row.members {
                    charge_work(requested, type_id, "union_member", &mut work, work_limit)?;
                    for _ in &member.payload_nodes {
                        charge_work(requested, type_id, "union_payload", &mut work, work_limit)?;
                    }
                }
                if !source_union_row_matches(self, row) {
                    return Err(refuse(
                        ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                        work,
                    ));
                }
                ValueType::Composite(row.type_handle.clone())
            }
            CheckedNodeKind::ScalarType(ScalarTypeForm::Unit) => {
                match identities
                    .declaration_identity(self, type_id)
                    .map_err(|code| refuse(code, work))?
                {
                    SelectedExactIdentity::DeclaredUnit(unit @ UnitId::Declared(_)) => {
                        ValueType::Quantity(unit.clone())
                    }
                    _ => {
                        return Err(refuse(
                            ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                            work,
                        ));
                    }
                }
            }
            _ => {
                return Err(refuse(
                    ExactTypeProjectionErrorCode::UnsupportedExactType,
                    work,
                ))
            }
        };
        Ok(ExactTypeProjection {
            node: requested.clone(),
            type_node: type_id.clone(),
            value_type,
            record_view,
            source_record_view,
            work,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checked_package::v2::{CheckedPackageWireV2, RetainedModels};
    use serde_json::{json, Value};

    fn id(path: &str) -> Value {
        json!({"package":"exact-test","path":[path],"ordinal":0})
    }

    fn node(path: &str, tag: &str, form: &str, type_path: &str, body: Value) -> Value {
        json!({
            "node_id":id(path),
            "schema_version":"quire.checked-semantic-graph/v2",
            "node_tag":tag,"semantic_form":form,"semantic_type":id(type_path),
            "dependencies":[],"occurrences":[],"body":body,
        })
    }

    fn fixture() -> CheckedPackageV2 {
        let empty = json!({"term":"aggregate","members":[]});
        let range = |path: &str, lower: &str, upper: &str| {
            node(
                path,
                "bounded_domain",
                "integer_range",
                "integer",
                json!({
                    "term":"aggregate","members":[
                        {"term":"binding","name":"min","value":{"term":"literal","type":id("integer"),"value_kind":"integer","value":lower}},
                        {"term":"binding","name":"max","value":{"term":"literal","type":id("integer"),"value_kind":"integer","value":upper}},
                    ]
                }),
            )
        };
        let wire: CheckedPackageWireV2 = super::super::test_support::private_wire(json!({
            "contract_version":"quire.checked-package/v2","name":"exact-test",
            "lock":{"sources":[],"edition":{"role":"edition","definition":{"authority":"test","identity":"edition"}},
                "profile_selections":[],"definition_selections":[],"required_features":[],"dependency_selections":[]},
            "semantic_graph":{"graph_version":"quire.checked-semantic-graph/v2","nodes":[
                node("boolean","scalar_type","boolean","boolean",empty.clone()),
                node("integer","scalar_type","integer","integer",empty.clone()),
                node("string","scalar_type","string","string",empty.clone()),
                node("uuid","scalar_type","uuid","uuid",empty.clone()),
                node("timestamp","scalar_type","timestamp","timestamp",empty.clone()),
                range("range1000","0","1000"),
                range("range3","0","3"),
                range("range9","0","9"),
                node("total","value","parameter","range1000",empty.clone()),
                node("tally","value","parameter","range3",empty.clone()),
                node("count","value","parameter","range9",empty.clone()),
                node("receipt","model","record_value_type","receipt",empty.clone()),
                node("alias","composite_type","alias","alias",empty),
            ]},
            "source_map":[],"capability_report":[],
            "diagnostics":{"catalog":{"authority":"test","identity":"diagnostics"},"entries":[]},
        }));
        let terms = wire
            .semantic_graph
            .nodes
            .iter()
            .map(|node| node.body.clone())
            .collect();
        let kinds = wire
            .semantic_graph
            .nodes
            .iter()
            .map(|node| {
                CheckedNodeKind::decode(
                    super::super::CheckedNodeTag::from_wire(&node.node_tag).expect("known tag"),
                    &node.semantic_form,
                )
                .expect("known form")
            })
            .collect();
        CheckedPackageV2 {
            schedulability_claim_index: Vec::new(),
            monitor_plan_index: Vec::new(),
            node_index: super::super::retained_node_index(&wire),
            structural_keys: Vec::new(),
            source_index: super::super::retained_source_index(&wire),
            lowering_facts: Vec::new(),
            wire,
            terms,
            kinds,
            bytes: 1 << 20,
            models: RetainedModels::default(),
            selected_documents: Vec::new(),
            direct_dependencies: Vec::new(),
            closure_index: Default::default(),
        }
    }

    fn checked_id(path: &str) -> CheckedNodeId {
        serde_json::from_value(id(path)).expect("fixture identity")
    }

    #[test]
    fn scalar_and_inclusive_range_projection() {
        let package = fixture();
        let identities = SelectedExactIdentities::empty();
        for (path, expected) in [
            ("boolean", ValueType::Boolean),
            ("integer", ValueType::Integer),
            ("string", ValueType::String),
            ("uuid", ValueType::Uuid),
            ("timestamp", ValueType::Timestamp),
        ] {
            let projected = package
                .project_exact_type(&checked_id(path), &identities, 2)
                .expect("scalar");
            assert_eq!(projected.value_type(), &expected, "{path}");
        }
        for (path, upper) in [("total", 1000_i64), ("tally", 3), ("count", 9)] {
            let projected = package
                .project_exact_type(&checked_id(path), &identities, 12)
                .expect("range");
            let expected = ValueType::Int(
                IntegerInterval::new(Integer::from(0_i64), Integer::from(upper)).expect("ordered"),
            );
            assert_eq!(projected.value_type(), &expected, "{path}");
        }
    }

    #[test]
    fn unknown_foreign_unsupported_and_exhausted_are_distinct() {
        let package = fixture();
        let identities = SelectedExactIdentities::empty();
        let refusal = |id: CheckedNodeId, limit| {
            package
                .project_exact_type(&id, &identities, limit)
                .expect_err("refused")
        };
        let mut changed = checked_id("boolean");
        changed.ordinal = 1;
        assert_eq!(
            refusal(changed, 2).code,
            ExactTypeProjectionErrorCode::UnknownNode
        );
        assert_eq!(
            refusal(checked_id("different-path"), 2).code,
            ExactTypeProjectionErrorCode::UnknownNode
        );
        let mut foreign = checked_id("boolean");
        foreign.package = "another-package".into();
        assert_eq!(
            refusal(foreign, 2).code,
            ExactTypeProjectionErrorCode::ForeignNode
        );
        assert_eq!(
            refusal(checked_id("alias"), 2).code,
            ExactTypeProjectionErrorCode::UnsupportedExactType
        );
        assert_eq!(
            refusal(checked_id("receipt"), 2).code,
            ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration
        );
        let exhausted = refusal(checked_id("total"), 1);
        assert_eq!(exhausted.code, ExactTypeProjectionErrorCode::ExhaustedWork);
        assert_eq!(exhausted.work, 1);
        assert_eq!(exhausted.at, Some(Box::new(checked_id("range1000"))));
        assert_eq!(
            package.project_exact_type(&checked_id("total"), &identities, 12),
            package
                .clone()
                .project_exact_type(&checked_id("total"), &identities, 12)
        );
    }
}
