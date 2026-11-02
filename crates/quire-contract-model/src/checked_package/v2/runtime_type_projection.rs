// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-354: admitted selected type declarations for a sealed evaluator adapter.

use super::exact_projection::project_record_field_type;
use super::model_members::{DeclarationForm, MemberType};
use super::nominal_reads::{NominalDeclarationRow, NominalIdentity, NominalReadCode};
use super::{
    CheckedNodeId, CheckedNodeKind, CheckedNodeOwner, CheckedNodePathSegment,
    CheckedOperationEffect, CheckedOperationEffectProvenance, CheckedPackageV2, CheckedRational,
    DimensionTerm, ExactTypeProjectionError, ExactTypeProjectionErrorCode, ScalarTypeForm,
    SelectedExactIdentities, SelectedExactIdentity, SelectedRecordView, SelectedSourceUnitRow,
    SelectedTupleDeclarationRow,
};
use quire_exact::{EffectiveId, NodeKey, PathSegmentView, Presence, UnitId, ValueType};
use std::collections::{BTreeMap, BTreeSet};

/// One admitted effective object field in selected declaration order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedTypeObjectField {
    original_identity: Box<str>,
    name: Box<str>,
    value_type: ValueType,
    presence: Presence,
}

impl SelectedTypeObjectField {
    /// The selected document's original field identity.
    pub fn original_identity(&self) -> &str {
        &self.original_identity
    }

    /// The selected field name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The complete Exact field type.
    pub fn value_type(&self) -> &ValueType {
        &self.value_type
    }

    /// The selected field's presence.
    pub fn presence(&self) -> Presence {
        self.presence
    }
}

/// One selected object declaration and its complete effective fields.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedTypeObject {
    original_identity: Box<str>,
    effective_id: EffectiveId,
    supertypes: Vec<EffectiveId>,
    declared_fields: Vec<SelectedTypeObjectField>,
    fields: Vec<SelectedTypeObjectField>,
    operations: Vec<SelectedTypeObjectOperation>,
}

/// A parameter of an object's own selected operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedTypeOperationParameter {
    name: Box<str>,
    value_type: ValueType,
}

impl SelectedTypeOperationParameter {
    /// Original declared parameter name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Exact parameter value type.
    pub fn value_type(&self) -> &ValueType {
        &self.value_type
    }
}

/// An object's own operation, with the admitted producer effect.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedTypeObjectOperation {
    identity: Box<str>,
    name: Box<str>,
    parameters: Vec<SelectedTypeOperationParameter>,
    result: Option<ValueType>,
    effect: CheckedOperationEffect,
    provenance: CheckedOperationEffectProvenance,
}

impl SelectedTypeObjectOperation {
    /// Original selected model member identity.
    pub fn identity(&self) -> &str {
        &self.identity
    }

    /// Original operation name on the declaring object.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Parameters in selected source order.
    pub fn parameters(&self) -> &[SelectedTypeOperationParameter] {
        &self.parameters
    }

    /// Optional exact result type.
    pub fn result(&self) -> Option<&ValueType> {
        self.result.as_ref()
    }

    /// Complete admitted effect, including an intentionally empty effect.
    pub fn effect(&self) -> &CheckedOperationEffect {
        &self.effect
    }

    /// The producer's fixed source origin and original member identity.
    pub fn provenance(&self) -> &CheckedOperationEffectProvenance {
        &self.provenance
    }
}

impl SelectedTypeObject {
    /// The original declaration identity in its selected document.
    pub fn original_identity(&self) -> &str {
        &self.original_identity
    }

    /// The QSL-stage witnessed effective object identity.
    pub fn effective_id(&self) -> &EffectiveId {
        &self.effective_id
    }

    /// Direct witnessed object supertypes in selected declaration order.
    pub fn supertypes(&self) -> &[EffectiveId] {
        &self.supertypes
    }

    /// This declaration's own fields in authored order.
    pub fn declared_fields(&self) -> &[SelectedTypeObjectField] {
        &self.declared_fields
    }

    /// Every effective field in selected declaration order.
    pub fn fields(&self) -> &[SelectedTypeObjectField] {
        &self.fields
    }

    /// This object's own operations in selected declaration order.
    pub fn operations(&self) -> &[SelectedTypeObjectOperation] {
        &self.operations
    }
}

/// One selected record declaration with its complete Exact view and parents.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedTypeRecord {
    original_identity: Box<str>,
    view: SelectedRecordView,
    supertypes: Vec<NodeKey>,
}

/// A selected tuple with its witnessed key and all positions in authored order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedTypeTuple {
    original_identity: Box<str>,
    node_key: NodeKey,
    positions: Vec<ValueType>,
}

impl SelectedTypeTuple {
    /// Original identity in the selected document.
    pub fn original_identity(&self) -> &str {
        &self.original_identity
    }
    /// Producer-witnessed Exact composite key.
    pub fn node_key(&self) -> &NodeKey {
        &self.node_key
    }
    /// Exact types of every position in admitted author order.
    pub fn positions(&self) -> &[ValueType] {
        &self.positions
    }
}

/// A source-owned unit and its admitted conversion and producer identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedTypeUnit {
    /// Admitted structural unit node.
    pub structural_node: CheckedNodeId,
    /// Selected source authority.
    pub source_authority: Box<str>,
    /// Selected source identity.
    pub source_identity: Box<str>,
    /// Original qualified declaration in the selected source.
    pub qualified_declaration: Vec<Box<str>>,
    /// Producer-witnessed declared unit identity.
    pub unit_id: UnitId,
    /// Admitted dimension node.
    pub dimension_node: CheckedNodeId,
    /// Canonical dimension powers, empty for a base dimension.
    pub dimension_terms: Vec<DimensionTerm>,
    /// Admitted target unit, absent for a root unit.
    pub target_node: Option<CheckedNodeId>,
    /// Exact conversion scale.
    pub scale: CheckedRational,
    /// Exact conversion offset.
    pub offset: CheckedRational,
}

/// A dimension in the complete selected source-unit dependency closure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedTypeDimension {
    /// Admitted dimension node.
    pub structural_node: CheckedNodeId,
    /// Canonical dimension powers.
    pub terms: Vec<DimensionTerm>,
}

impl SelectedTypeRecord {
    /// The original declaration identity in its selected document.
    pub fn original_identity(&self) -> &str {
        &self.original_identity
    }

    /// Complete effective fields and the witnessed record key.
    pub fn view(&self) -> &SelectedRecordView {
        &self.view
    }

    /// Direct witnessed record supertypes in selected declaration order.
    pub fn supertypes(&self) -> &[NodeKey] {
        &self.supertypes
    }
}

/// Complete selected types for a package with supported declarations.
///
/// The view contains no partial tables. Unsupported enum and model-unit
/// selections refuse rather than reconstructing missing producer authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedTypeDeclarations {
    records: Vec<SelectedTypeRecord>,
    tuples: Vec<SelectedTypeTuple>,
    objects: Vec<SelectedTypeObject>,
    units: Vec<SelectedTypeUnit>,
    dimensions: Vec<SelectedTypeDimension>,
    work: u64,
}

impl SelectedTypeDeclarations {
    /// Selected records in structural identity order.
    pub fn records(&self) -> &[SelectedTypeRecord] {
        &self.records
    }

    /// Selected tuples in structural identity order.
    pub fn tuples(&self) -> &[SelectedTypeTuple] {
        &self.tuples
    }

    /// Selected objects in structural identity order.
    pub fn objects(&self) -> &[SelectedTypeObject] {
        &self.objects
    }

    /// Selected source units in structural identity order.
    pub fn units(&self) -> &[SelectedTypeUnit] {
        &self.units
    }

    /// Complete dimension dependency closure of selected source units.
    pub fn dimensions(&self) -> &[SelectedTypeDimension] {
        &self.dimensions
    }

    /// Work units charged to form the complete view.
    pub fn work(&self) -> u64 {
        self.work
    }
}

/// A typed refusal with the first failing admitted node and charged work.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("selected runtime type projection refused: {code:?} at {form}")]
pub struct SelectedTypeDeclarationsError {
    /// Stable refusal category shared with Exact type projection.
    pub code: ExactTypeProjectionErrorCode,
    /// The first node whose selected type could not be projected.
    pub at: Box<CheckedNodeId>,
    /// The admitted family/form, or a named selected join.
    pub form: Box<str>,
    /// Work successfully charged before refusal.
    pub work: u64,
}

impl SelectedTypeDeclarationsError {
    fn at(
        code: ExactTypeProjectionErrorCode,
        at: &CheckedNodeId,
        form: impl Into<Box<str>>,
        work: u64,
    ) -> Self {
        Self {
            code,
            at: Box::new(at.clone()),
            form: form.into(),
            work,
        }
    }

    fn from_exact(error: ExactTypeProjectionError, prior_work: u64) -> Self {
        Self {
            code: error.code,
            at: error.at.unwrap_or(error.requested),
            form: error.form,
            work: prior_work.saturating_add(error.work),
        }
    }
}

fn charge(
    work: &mut u64,
    limit: u64,
    at: &CheckedNodeId,
    form: &str,
) -> Result<(), SelectedTypeDeclarationsError> {
    if *work >= limit {
        return Err(SelectedTypeDeclarationsError::at(
            ExactTypeProjectionErrorCode::ExhaustedWork,
            at,
            form,
            *work,
        ));
    }
    *work += 1;
    Ok(())
}

fn charge_amount(
    work: &mut u64,
    limit: u64,
    at: &CheckedNodeId,
    form: &str,
    amount: usize,
) -> Result<(), SelectedTypeDeclarationsError> {
    let Some(next) = u64::try_from(amount)
        .ok()
        .and_then(|amount| work.checked_add(amount))
    else {
        return Err(SelectedTypeDeclarationsError::at(
            ExactTypeProjectionErrorCode::ExhaustedWork,
            at,
            form,
            *work,
        ));
    };
    if next > limit {
        return Err(SelectedTypeDeclarationsError::at(
            ExactTypeProjectionErrorCode::ExhaustedWork,
            at,
            form,
            *work,
        ));
    }
    *work = next;
    Ok(())
}

fn charge_structural_id(
    work: &mut u64,
    limit: u64,
    at: &CheckedNodeId,
    form: &str,
) -> Result<(), SelectedTypeDeclarationsError> {
    let mut pending = vec![at];
    while let Some(id) = pending.pop() {
        charge_amount(work, limit, at, form, id.package.len())?;
        charge_amount(work, limit, at, form, 8)?;
        for segment in &id.path {
            charge(work, limit, at, form)?;
            match segment {
                CheckedNodePathSegment::Name(name) => {
                    charge_amount(work, limit, at, form, name.len())?
                }
                CheckedNodePathSegment::Index(_) => charge_amount(work, limit, at, form, 8)?,
                CheckedNodePathSegment::Node(child) => pending.push(child),
            }
        }
    }
    Ok(())
}

fn charge_node_key(
    work: &mut u64,
    limit: u64,
    at: &CheckedNodeId,
    form: &str,
    key: &NodeKey,
) -> Result<(), SelectedTypeDeclarationsError> {
    let mut pending = vec![key.view()];
    while let Some(view) = pending.pop() {
        charge_amount(work, limit, at, form, view.package().len())?;
        charge_amount(work, limit, at, form, 8)?;
        for segment in view.path() {
            charge(work, limit, at, form)?;
            match segment {
                PathSegmentView::Text(text) => charge_amount(work, limit, at, form, text.len())?,
                PathSegmentView::Node(child) => pending.push(child),
            }
        }
    }
    Ok(())
}

fn projected_field_type(
    package: &CheckedPackageV2,
    identities: &SelectedExactIdentities,
    member_type: Option<&MemberType>,
    optional: bool,
    at: &CheckedNodeId,
    form: &str,
    work: u64,
) -> Result<ValueType, SelectedTypeDeclarationsError> {
    let member_type = member_type.ok_or_else(|| {
        SelectedTypeDeclarationsError::at(
            ExactTypeProjectionErrorCode::UnsupportedExactType,
            at,
            form,
            work,
        )
    })?;
    let member_type = if optional {
        match member_type {
            MemberType::Option(inner) => inner.as_ref(),
            _ => {
                return Err(SelectedTypeDeclarationsError::at(
                    ExactTypeProjectionErrorCode::UnsupportedExactType,
                    at,
                    form,
                    work,
                ));
            }
        }
    } else {
        member_type
    };
    project_record_field_type(package, identities, member_type).map_err(|(code, target)| {
        SelectedTypeDeclarationsError::at(code, target.as_ref().unwrap_or(at), form, work)
    })
}

impl CheckedPackageV2 {
    /// Project every selected record, tuple, object and source unit as one bounded view.
    ///
    /// Enum and model-unit selections return a typed refusal before a view is
    /// exposed. Identity rows must originate from
    /// the stage selecting the same documents; IR checks their structural join
    /// but cannot authenticate an opaque Exact key supplied by another caller.
    pub fn project_selected_type_declarations(
        &self,
        identities: &SelectedExactIdentities,
        tuple_rows: &[SelectedTupleDeclarationRow],
        source_unit_rows: &[SelectedSourceUnitRow],
        work_limit: u64,
    ) -> Result<SelectedTypeDeclarations, SelectedTypeDeclarationsError> {
        let mut work = 0;
        let mut selected_source_units = Vec::new();
        for (node, kind) in self.graph().nodes.iter().zip(self.node_kinds()) {
            charge(&mut work, work_limit, &node.node_id, "graph_node")?;
            if *kind == CheckedNodeKind::ScalarType(ScalarTypeForm::Unit)
                && matches!(node.owner, Some(CheckedNodeOwner::Source { .. }))
            {
                charge_structural_id(&mut work, work_limit, &node.node_id, "selected_source_unit")?;
                selected_source_units.push(node.node_id.clone());
            } else if matches!(
                kind,
                CheckedNodeKind::ScalarType(
                    ScalarTypeForm::Unit | ScalarTypeForm::Enum | ScalarTypeForm::CompoundUnit
                )
            ) {
                return Err(SelectedTypeDeclarationsError::at(
                    ExactTypeProjectionErrorCode::UnavailableIdentityWitness,
                    &node.node_id,
                    format!("{}/{}", node.node_tag, node.semantic_form),
                    work,
                ));
            }
        }

        let mut by_original = BTreeMap::new();
        for (id, declaration) in self.models.selected_declarations() {
            charge(&mut work, work_limit, id, "selected_declaration")?;
            if by_original
                .insert(
                    (declaration.selection, declaration.node.as_ref()),
                    (id, declaration.form),
                )
                .is_some()
            {
                return Err(SelectedTypeDeclarationsError::at(
                    ExactTypeProjectionErrorCode::ConflictingIdentityWitness,
                    id,
                    "selected_declaration",
                    work,
                ));
            }
            if matches!(declaration.form, DeclarationForm::VariantType) {
                return Err(SelectedTypeDeclarationsError::at(
                    ExactTypeProjectionErrorCode::UnavailableIdentityWitness,
                    id,
                    declaration.form.form(),
                    work,
                ));
            }
        }

        // An admitted selected document can declare types that no graph node
        // currently names. Such a document cannot yield a complete runtime
        // environment from this graph and must refuse as a whole.
        for (selection, model) in self.models.selected_models().iter().enumerate() {
            let root = &self
                .graph()
                .nodes
                .get(model.root)
                .expect("admitted selected model root is a graph node")
                .node_id;
            for original in model.record_types.keys().chain(model.object_types.keys()) {
                if model.end_declaration_form(original) == Some(DeclarationForm::SystemsInterface)
                    || model.state_machines.contains(original)
                    || model.processes.contains(original)
                {
                    continue;
                }
                charge(&mut work, work_limit, root, "selected_document_type")?;
                if !by_original.contains_key(&(selection, original.as_ref())) {
                    return Err(SelectedTypeDeclarationsError::at(
                        ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration,
                        root,
                        "selected_document_type",
                        work,
                    ));
                }
            }
            for original in model.tuple_types.keys() {
                charge(&mut work, work_limit, root, "selected_document_tuple")?;
                if !by_original.contains_key(&(selection, original.as_ref())) {
                    return Err(SelectedTypeDeclarationsError::at(
                        ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration,
                        root,
                        "selected_document_tuple",
                        work,
                    ));
                }
            }
            if let Some(original) = model.variant_types.keys().next() {
                charge(&mut work, work_limit, root, "selected_document_type")?;
                return Err(SelectedTypeDeclarationsError::at(
                    ExactTypeProjectionErrorCode::UnavailableIdentityWitness,
                    by_original
                        .get(&(selection, original.as_ref()))
                        .map_or(root, |(id, _)| id),
                    "selected_variant",
                    work,
                ));
            }
            if !model.units.is_empty() {
                charge(&mut work, work_limit, root, "selected_unit")?;
                return Err(SelectedTypeDeclarationsError::at(
                    ExactTypeProjectionErrorCode::UnavailableIdentityWitness,
                    root,
                    "selected_unit",
                    work,
                ));
            }
        }

        let mut tuple_by_node = BTreeMap::new();
        let mut source_unit_by_node = BTreeMap::new();
        let mut exact_keys = BTreeSet::new();
        for row in identities.declaration_rows() {
            charge(
                &mut work,
                work_limit,
                &row.structural_node,
                "selected_exact_key",
            )?;
            let key = match &row.exact_identity {
                SelectedExactIdentity::Record(key)
                | SelectedExactIdentity::Union(key)
                | SelectedExactIdentity::DeclaredUnit(UnitId::Declared(key)) => Some(key),
                SelectedExactIdentity::Object(_)
                | SelectedExactIdentity::DeclaredUnit(UnitId::Compound(_)) => None,
            };
            if let Some(key) = key {
                charge_node_key(
                    &mut work,
                    work_limit,
                    &row.structural_node,
                    "selected_exact_key",
                    key,
                )?;
                exact_keys.insert(key.clone());
            }
        }
        for row in tuple_rows {
            charge(
                &mut work,
                work_limit,
                &row.structural_node,
                "selected_tuple_row",
            )?;
            charge_structural_id(
                &mut work,
                work_limit,
                &row.structural_node,
                "selected_tuple_row",
            )?;
            charge_amount(
                &mut work,
                work_limit,
                &row.structural_node,
                "selected_tuple_row",
                row.selected_document.len(),
            )?;
            charge_amount(
                &mut work,
                work_limit,
                &row.structural_node,
                "selected_tuple_row",
                row.original_identity.len(),
            )?;
            charge_node_key(
                &mut work,
                work_limit,
                &row.structural_node,
                "selected_tuple_row",
                &row.node_key,
            )?;
            let Some((document, original, form)) =
                self.models.selected_declaration(&row.structural_node)
            else {
                let code = if row.structural_node.package.as_ref() != self.name() {
                    ExactTypeProjectionErrorCode::ForeignNode
                } else if self.node(&row.structural_node).is_some() {
                    ExactTypeProjectionErrorCode::MismatchedIdentityWitness
                } else {
                    ExactTypeProjectionErrorCode::UnknownNode
                };
                return Err(SelectedTypeDeclarationsError::at(
                    code,
                    &row.structural_node,
                    "selected_tuple_row",
                    work,
                ));
            };
            if document != row.selected_document.as_ref()
                || original != row.original_identity.as_ref()
                || form != DeclarationForm::TupleValueType
            {
                return Err(SelectedTypeDeclarationsError::at(
                    ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                    &row.structural_node,
                    "selected_tuple_row",
                    work,
                ));
            }
            if tuple_by_node
                .insert(row.structural_node.clone(), row)
                .is_some()
                || !exact_keys.insert(row.node_key.clone())
            {
                return Err(SelectedTypeDeclarationsError::at(
                    ExactTypeProjectionErrorCode::ConflictingIdentityWitness,
                    &row.structural_node,
                    "selected_tuple_row",
                    work,
                ));
            }
        }
        for row in source_unit_rows {
            charge(
                &mut work,
                work_limit,
                &row.structural_node,
                "selected_source_unit_row",
            )?;
            charge_structural_id(
                &mut work,
                work_limit,
                &row.structural_node,
                "selected_source_unit_row",
            )?;
            charge_amount(
                &mut work,
                work_limit,
                &row.structural_node,
                "selected_source_unit_row",
                row.source_authority.len(),
            )?;
            charge_amount(
                &mut work,
                work_limit,
                &row.structural_node,
                "selected_source_unit_row",
                row.source_identity.len(),
            )?;
            for segment in &row.qualified_declaration {
                charge(
                    &mut work,
                    work_limit,
                    &row.structural_node,
                    "selected_source_unit_row",
                )?;
                charge_amount(
                    &mut work,
                    work_limit,
                    &row.structural_node,
                    "selected_source_unit_row",
                    segment.len(),
                )?;
            }
            if let UnitId::Declared(key) = &row.unit_id {
                charge_node_key(
                    &mut work,
                    work_limit,
                    &row.structural_node,
                    "selected_source_unit_row",
                    key,
                )?;
            }
            let Some(node) = self.node(&row.structural_node) else {
                let code = if row.structural_node.package.as_ref() != self.name() {
                    ExactTypeProjectionErrorCode::ForeignNode
                } else {
                    ExactTypeProjectionErrorCode::UnknownNode
                };
                return Err(SelectedTypeDeclarationsError::at(
                    code,
                    &row.structural_node,
                    "selected_source_unit_row",
                    work,
                ));
            };
            let selected_source = self.lock().sources.iter().any(|source| {
                source.authority == row.source_authority && source.identity == row.source_identity
            });
            let matching_owner = matches!(&node.owner, Some(CheckedNodeOwner::Source { authority, identity }) if authority == &row.source_authority && identity == &row.source_identity);
            let matching_declaration = node
                .declaration
                .as_ref()
                .is_some_and(|declaration| declaration.qualified_name == row.qualified_declaration);
            if self.node_kind(&row.structural_node)
                != Some(CheckedNodeKind::ScalarType(ScalarTypeForm::Unit))
                || !selected_source
                || !matching_owner
                || !matching_declaration
                || !matches!(row.unit_id, UnitId::Declared(_))
            {
                return Err(SelectedTypeDeclarationsError::at(
                    ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                    &row.structural_node,
                    "selected_source_unit_row",
                    work,
                ));
            }
            let UnitId::Declared(key) = &row.unit_id else {
                unreachable!("checked declared domain")
            };
            if source_unit_by_node
                .insert(row.structural_node.clone(), row)
                .is_some()
                || !exact_keys.insert(key.clone())
            {
                return Err(SelectedTypeDeclarationsError::at(
                    ExactTypeProjectionErrorCode::ConflictingIdentityWitness,
                    &row.structural_node,
                    "selected_source_unit_row",
                    work,
                ));
            }
        }
        for id in &selected_source_units {
            charge(&mut work, work_limit, id, "selected_source_unit")?;
            if !source_unit_by_node.contains_key(id) {
                return Err(SelectedTypeDeclarationsError::at(
                    ExactTypeProjectionErrorCode::UnavailableIdentityWitness,
                    id,
                    "selected_source_unit",
                    work,
                ));
            }
        }

        let admitted_identities =
            identities.with_admitted_type_rows(&tuple_by_node, &source_unit_by_node);
        let identities = &admitted_identities;
        let mut records = Vec::new();
        let mut tuples = Vec::new();
        let mut objects = Vec::new();
        for (id, declaration) in self.models.selected_declarations() {
            let Some(model) = self.models.selected_models().get(declaration.selection) else {
                return Err(SelectedTypeDeclarationsError::at(
                    ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration,
                    id,
                    "selected_model",
                    work,
                ));
            };
            let (supertypes, expected_form) = match declaration.form {
                DeclarationForm::RecordValueType => (
                    model
                        .record_types
                        .get(&declaration.node)
                        .map(|value| value.supertypes.as_slice()),
                    DeclarationForm::RecordValueType,
                ),
                DeclarationForm::ObjectType => (
                    model
                        .object_types
                        .get(&declaration.node)
                        .map(|value| value.supertypes.as_slice()),
                    DeclarationForm::ObjectType,
                ),
                DeclarationForm::TupleValueType => (None, DeclarationForm::TupleValueType),
                _ => continue,
            };
            let supertypes = if declaration.form == DeclarationForm::TupleValueType {
                &[][..]
            } else {
                supertypes.ok_or_else(|| {
                    SelectedTypeDeclarationsError::at(
                        ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration,
                        id,
                        declaration.form.form(),
                        work,
                    )
                })?
            };
            let mut record_parents = Vec::new();
            let mut object_parents = Vec::new();
            for original in supertypes {
                charge(&mut work, work_limit, id, "selected_supertype")?;
                let Some((parent_id, parent_form)) =
                    by_original.get(&(declaration.selection, original.as_ref()))
                else {
                    return Err(SelectedTypeDeclarationsError::at(
                        ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration,
                        id,
                        "selected_supertype",
                        work,
                    ));
                };
                if *parent_form != expected_form {
                    return Err(SelectedTypeDeclarationsError::at(
                        ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                        parent_id,
                        "selected_supertype",
                        work,
                    ));
                }
                let identity =
                    identities
                        .declaration_identity(self, parent_id)
                        .map_err(|code| {
                            SelectedTypeDeclarationsError::at(
                                code,
                                parent_id,
                                "selected_supertype",
                                work,
                            )
                        })?;
                match (declaration.form, identity) {
                    (DeclarationForm::RecordValueType, SelectedExactIdentity::Record(key)) => {
                        record_parents.push(key.clone());
                    }
                    (DeclarationForm::ObjectType, SelectedExactIdentity::Object(key)) => {
                        object_parents.push(key.clone());
                    }
                    _ => {
                        return Err(SelectedTypeDeclarationsError::at(
                            ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                            parent_id,
                            "selected_supertype",
                            work,
                        ));
                    }
                }
            }
            match declaration.form {
                DeclarationForm::TupleValueType => {
                    let Some(row) = tuple_by_node.get(id) else {
                        return Err(SelectedTypeDeclarationsError::at(
                            ExactTypeProjectionErrorCode::UnavailableIdentityWitness,
                            id,
                            "tuple_value_type",
                            work,
                        ));
                    };
                    let Some(positions) = model.tuple_types.get(&declaration.node) else {
                        return Err(SelectedTypeDeclarationsError::at(
                            ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration,
                            id,
                            "tuple_value_type",
                            work,
                        ));
                    };
                    let mut projected = Vec::with_capacity(positions.len());
                    for position in positions {
                        charge(&mut work, work_limit, id, "tuple_position")?;
                        let member_type = model.element_type(position).ok_or_else(|| {
                            SelectedTypeDeclarationsError::at(
                                ExactTypeProjectionErrorCode::UnsupportedExactType,
                                id,
                                "tuple_position",
                                work,
                            )
                        })?;
                        projected.push(projected_field_type(
                            self,
                            identities,
                            Some(&member_type),
                            false,
                            id,
                            "tuple_position",
                            work,
                        )?);
                    }
                    tuples.push(SelectedTypeTuple {
                        original_identity: declaration.node.clone(),
                        node_key: row.node_key.clone(),
                        positions: projected,
                    });
                }
                DeclarationForm::RecordValueType => {
                    let remaining = work_limit.saturating_sub(work);
                    let projection = self
                        .project_exact_type(id, identities, remaining)
                        .map_err(|error| SelectedTypeDeclarationsError::from_exact(error, work))?;
                    work = work.saturating_add(projection.work());
                    let view = projection.record_view().ok_or_else(|| {
                        SelectedTypeDeclarationsError::at(
                            ExactTypeProjectionErrorCode::UnsupportedExactType,
                            id,
                            "record_value_type",
                            work,
                        )
                    })?;
                    records.push(SelectedTypeRecord {
                        original_identity: declaration.node.clone(),
                        view: view.clone(),
                        supertypes: record_parents,
                    });
                }
                DeclarationForm::ObjectType => {
                    charge(&mut work, work_limit, id, "object_type")?;
                    let declared = model.object_types.get(&declaration.node).ok_or_else(|| {
                        SelectedTypeDeclarationsError::at(
                            ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration,
                            id,
                            "object_type",
                            work,
                        )
                    })?;
                    let mut projected_operations = Vec::with_capacity(declared.operations.len());
                    for operation in &declared.operations {
                        charge(&mut work, work_limit, id, "object_operation")?;
                        let name = super::model_members::member_name(&operation.identity);
                        let row = self
                            .selected_operation_effect(&model.identity, id, name)
                            .ok_or_else(|| {
                                SelectedTypeDeclarationsError::at(
                                    ExactTypeProjectionErrorCode::UnavailableIdentityWitness,
                                    id,
                                    "object_operation_effect",
                                    work,
                                )
                            })?;
                        let mut parameters = Vec::with_capacity(operation.parameters.len());
                        for parameter in &operation.parameters {
                            charge(&mut work, work_limit, id, "object_operation_parameter")?;
                            let name = parameter.name.as_ref().ok_or_else(|| {
                                SelectedTypeDeclarationsError::at(
                                    ExactTypeProjectionErrorCode::UnsupportedExactType,
                                    id,
                                    "object_operation_parameter_name",
                                    work,
                                )
                            })?;
                            let member_type = model.slot_type(parameter);
                            let value_type = projected_field_type(
                                self,
                                identities,
                                member_type.as_ref(),
                                false,
                                id,
                                "object_operation_parameter_type",
                                work,
                            )?;
                            parameters.push(SelectedTypeOperationParameter {
                                name: name.clone(),
                                value_type,
                            });
                        }
                        let result = operation
                            .result
                            .as_ref()
                            .map(|slot| {
                                charge(&mut work, work_limit, id, "object_operation_result")?;
                                let member_type = model.slot_type(slot);
                                projected_field_type(
                                    self,
                                    identities,
                                    member_type.as_ref(),
                                    false,
                                    id,
                                    "object_operation_result_type",
                                    work,
                                )
                            })
                            .transpose()?;
                        let effect = &row.effect;
                        for _ in &effect.modifies {
                            charge(&mut work, work_limit, id, "object_operation_effect")?;
                        }
                        for _ in &effect.creates {
                            charge(&mut work, work_limit, id, "object_operation_effect")?;
                        }
                        for _ in &effect.deletes {
                            charge(&mut work, work_limit, id, "object_operation_effect")?;
                        }
                        for _ in &effect.deletes_scoped {
                            charge(&mut work, work_limit, id, "object_operation_effect")?;
                        }
                        projected_operations.push(SelectedTypeObjectOperation {
                            identity: operation.identity.clone(),
                            name: name.into(),
                            parameters,
                            result,
                            effect: effect.clone(),
                            provenance: row.provenance.clone(),
                        });
                    }
                    for field in &declared.fields {
                        charge(&mut work, work_limit, id, "declared_object_field")?;
                        if field.redefines.is_some() {
                            return Err(SelectedTypeDeclarationsError::at(
                                ExactTypeProjectionErrorCode::UnsupportedExactType,
                                id,
                                "object_field_redefinition",
                                work,
                            ));
                        }
                    }
                    let effective_id =
                        match identities.declaration_identity(self, id).map_err(|code| {
                            SelectedTypeDeclarationsError::at(code, id, "object_type", work)
                        })? {
                            SelectedExactIdentity::Object(key) => key.clone(),
                            _ => {
                                return Err(SelectedTypeDeclarationsError::at(
                                    ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                                    id,
                                    "object_type",
                                    work,
                                ));
                            }
                        };
                    let (ambiguous, fields) =
                        self.models.selected_object_fields(id).ok_or_else(|| {
                            SelectedTypeDeclarationsError::at(
                                ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration,
                                id,
                                "object_type_fields",
                                work,
                            )
                        })?;
                    if ambiguous.is_some() {
                        return Err(SelectedTypeDeclarationsError::at(
                            ExactTypeProjectionErrorCode::UnsupportedExactType,
                            id,
                            "ambiguous_object_field",
                            work,
                        ));
                    }
                    let mut projected_fields = Vec::new();
                    for field in fields {
                        charge(&mut work, work_limit, id, "object_field")?;
                        let value_type = projected_field_type(
                            self,
                            identities,
                            field.member_type,
                            field.optional,
                            id,
                            "object_field_type",
                            work,
                        )?;
                        projected_fields.push(SelectedTypeObjectField {
                            original_identity: field.identity.into(),
                            name: field.name.into(),
                            value_type,
                            presence: if field.optional {
                                Presence::Optional
                            } else {
                                Presence::Required
                            },
                        });
                    }
                    let mut by_identity = BTreeMap::new();
                    for field in &projected_fields {
                        charge(&mut work, work_limit, id, "object_field_join")?;
                        if by_identity
                            .insert(field.original_identity(), field)
                            .is_some()
                        {
                            return Err(SelectedTypeDeclarationsError::at(
                                ExactTypeProjectionErrorCode::ConflictingIdentityWitness,
                                id,
                                "object_field_join",
                                work,
                            ));
                        }
                    }
                    let mut declared_fields = Vec::new();
                    for field in &declared.fields {
                        charge(&mut work, work_limit, id, "declared_object_field")?;
                        let projected =
                            by_identity.get(field.identity.as_ref()).ok_or_else(|| {
                                SelectedTypeDeclarationsError::at(
                                    ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration,
                                    id,
                                    "declared_object_field",
                                    work,
                                )
                            })?;
                        declared_fields.push((*projected).clone());
                    }
                    objects.push(SelectedTypeObject {
                        original_identity: declaration.node.clone(),
                        effective_id,
                        supertypes: object_parents,
                        declared_fields,
                        fields: projected_fields,
                        operations: projected_operations,
                    });
                }
                _ => unreachable!("selected runtime declaration form was checked"),
            }
        }
        selected_source_units.sort();
        let nominal_rows = selected_source_units
            .iter()
            .map(|id| {
                let row = source_unit_by_node
                    .get(id)
                    .expect("checked complete source-unit rows");
                let node = self
                    .node(id)
                    .expect("admitted source unit remains in graph");
                NominalDeclarationRow {
                    node: id,
                    owner: node.owner.as_ref().expect("admitted source owner"),
                    declaration: node
                        .declaration
                        .as_ref()
                        .expect("admitted source declaration"),
                    identity: NominalIdentity::Unit(&row.unit_id),
                }
            })
            .collect::<Vec<_>>();
        let unit_read = self
            .read_unit_definition(
                &selected_source_units,
                &nominal_rows,
                work_limit.saturating_sub(work),
            )
            .map_err(|error| {
                let code = match error.code {
                    NominalReadCode::MissingNode => ExactTypeProjectionErrorCode::UnknownNode,
                    NominalReadCode::MissingWitness => {
                        ExactTypeProjectionErrorCode::UnavailableIdentityWitness
                    }
                    NominalReadCode::ForeignWitness => {
                        ExactTypeProjectionErrorCode::MismatchedIdentityWitness
                    }
                    NominalReadCode::ConflictingWitness => {
                        ExactTypeProjectionErrorCode::ConflictingIdentityWitness
                    }
                    NominalReadCode::UnsupportedForm | NominalReadCode::UnresolvedConversion => {
                        ExactTypeProjectionErrorCode::UnsupportedExactType
                    }
                    NominalReadCode::LimitReached => ExactTypeProjectionErrorCode::ExhaustedWork,
                };
                SelectedTypeDeclarationsError::at(
                    code,
                    error.node,
                    "selected_source_unit",
                    work.saturating_add(error.work),
                )
            })?;
        work = work.saturating_add(unit_read.work);
        let units = unit_read
            .units
            .iter()
            .map(|unit| {
                let row = source_unit_by_node
                    .get(&unit.node.node_id)
                    .expect("read selected source unit");
                SelectedTypeUnit {
                    structural_node: unit.node.node_id.clone(),
                    source_authority: row.source_authority.clone(),
                    source_identity: row.source_identity.clone(),
                    qualified_declaration: row.qualified_declaration.clone(),
                    unit_id: unit.identity.clone(),
                    dimension_node: unit.dimension.node_id.clone(),
                    dimension_terms: unit.dimension_terms.to_vec(),
                    target_node: unit.target.map(|node| node.node_id.clone()),
                    scale: unit.scale.clone(),
                    offset: unit.offset.clone(),
                }
            })
            .collect();
        let dimensions = unit_read
            .dimensions
            .iter()
            .map(|dimension| SelectedTypeDimension {
                structural_node: dimension.node.node_id.clone(),
                terms: dimension.terms.to_vec(),
            })
            .collect();
        Ok(SelectedTypeDeclarations {
            records,
            tuples,
            objects,
            units,
            dimensions,
            work,
        })
    }
}
