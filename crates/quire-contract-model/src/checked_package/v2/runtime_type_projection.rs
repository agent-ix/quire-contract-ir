// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-354: admitted selected type declarations for a sealed evaluator adapter.

use super::exact_projection::project_record_field_type;
use super::model_members::{DeclarationForm, MemberType};
use super::native_join_storage;
use super::nominal_reads::{NominalDeclarationRow, NominalIdentity, NominalReadCode};
use super::{
    CheckedFrameModification, CheckedNodeId, CheckedNodeKind, CheckedNodeOwner,
    CheckedNodePathSegment, CheckedOperationEffect, CheckedOperationEffectProvenance,
    CheckedPackageV2, CheckedRational, CheckedScopedDeletion, DimensionTerm,
    ExactTypeProjectionError, ExactTypeProjectionErrorCode, NativeJoinBudget, NativeJoinCharge,
    NativeJoinChargeKind, NativeJoinDeniedCharge, ScalarTypeForm, SelectedExactIdentities,
    SelectedExactIdentity, SelectedRecordView, SelectedSourceUnitRow, SelectedTupleDeclarationRow,
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
#[error("selected runtime type projection refused: {code:?} at {form:?}")]
pub struct SelectedTypeDeclarationsError {
    /// Stable refusal category shared with Exact type projection.
    pub code: ExactTypeProjectionErrorCode,
    /// The first node whose selected type could not be projected.
    pub at: Option<Box<CheckedNodeId>>,
    /// The admitted family/form, or a named selected join.
    pub form: Option<Box<str>>,
    /// Work successfully charged before refusal.
    pub work: u64,
    /// The first unpaid charge when the budget refused construction.
    pub denied: Option<NativeJoinDeniedCharge>,
}

impl SelectedTypeDeclarationsError {
    fn exhausted(budget: &NativeJoinBudget, before: NativeJoinCharge) -> Self {
        let denied = budget.first_unpaid();
        Self {
            code: match denied.map(|charge| charge.kind) {
                Some(NativeJoinChargeKind::Storage) => {
                    ExactTypeProjectionErrorCode::ExhaustedStorage
                }
                _ => ExactTypeProjectionErrorCode::ExhaustedWork,
            },
            at: None,
            form: None,
            work: budget.consumed().work - before.work,
            denied,
        }
    }
}

struct ProjectionBudget<'a> {
    budget: &'a mut NativeJoinBudget,
    before: NativeJoinCharge,
}

impl ProjectionBudget<'_> {
    fn work(&self) -> u64 {
        self.budget.consumed().work - self.before.work
    }

    fn charge(
        &mut self,
        _at: &CheckedNodeId,
        _form: &str,
    ) -> Result<(), SelectedTypeDeclarationsError> {
        self.charge_amount(NativeJoinChargeKind::Work, 1)
    }

    fn charge_amount(
        &mut self,
        kind: NativeJoinChargeKind,
        amount: u128,
    ) -> Result<(), SelectedTypeDeclarationsError> {
        if self.budget.charge(kind, amount) {
            Ok(())
        } else {
            Err(SelectedTypeDeclarationsError::exhausted(
                self.budget,
                self.before,
            ))
        }
    }

    fn charge_work(
        &mut self,
        _at: &CheckedNodeId,
        _form: &str,
        amount: usize,
    ) -> Result<(), SelectedTypeDeclarationsError> {
        self.charge_amount(
            NativeJoinChargeKind::Work,
            u128::try_from(amount).unwrap_or(u128::MAX),
        )
    }

    fn charge_storage(&mut self, amount: u128) -> Result<(), SelectedTypeDeclarationsError> {
        self.charge_amount(NativeJoinChargeKind::Storage, amount)
    }

    fn storage_text(&mut self, value: &str) -> Result<(), SelectedTypeDeclarationsError> {
        self.charge_storage(1_u128.saturating_add(value.len() as u128))
    }

    fn storage_node(&mut self, value: &CheckedNodeId) -> Result<(), SelectedTypeDeclarationsError> {
        self.charge_storage(native_join_storage::node(value))
    }

    fn storage_value_type(
        &mut self,
        value: &ValueType,
    ) -> Result<(), SelectedTypeDeclarationsError> {
        self.charge_storage(native_join_storage::value_type(value))
    }

    fn storage_record_view(
        &mut self,
        view: &SelectedRecordView,
    ) -> Result<(), SelectedTypeDeclarationsError> {
        self.charge_storage(2)?;
        for field in view.fields() {
            self.charge_storage(1)?;
            self.storage_text(field.ir_identity())?;
            self.storage_text(field.name())?;
            self.charge_storage(native_join_storage::member_identity(field.member_id()))?;
            self.storage_value_type(field.value_type())?;
        }
        Ok(())
    }

    fn storage_effect(
        &mut self,
        effect: &CheckedOperationEffect,
    ) -> Result<(), SelectedTypeDeclarationsError> {
        self.charge_storage(4)?;
        for modification in &effect.modifies {
            self.charge_storage(1)?;
            match modification {
                CheckedFrameModification::Relationship { declaration } => {
                    self.storage_node(declaration)?
                }
                CheckedFrameModification::Field {
                    declaration, name, ..
                } => {
                    self.storage_node(declaration)?;
                    self.storage_text(name)?;
                }
            }
        }
        for node in effect.creates.iter().chain(&effect.deletes) {
            self.charge_storage(1)?;
            self.storage_node(node)?;
        }
        for deletion in &effect.deletes_scoped {
            self.charge_storage(1)?;
            if let CheckedScopedDeletion::Parameter { name } = deletion {
                self.storage_text(name)?;
            }
        }
        Ok(())
    }

    fn storage_dimension_terms(
        &mut self,
        terms: &[DimensionTerm],
    ) -> Result<(), SelectedTypeDeclarationsError> {
        self.charge_storage(1)?;
        for term in terms {
            self.charge_storage(1)?;
            self.storage_node(&term.dimension_node_id)?;
            self.storage_text(&term.exponent)?;
        }
        Ok(())
    }

    fn storage_rational(
        &mut self,
        value: &CheckedRational,
    ) -> Result<(), SelectedTypeDeclarationsError> {
        self.storage_text(&value.numerator)?;
        self.storage_text(&value.denominator)
    }

    fn error(
        &mut self,
        code: ExactTypeProjectionErrorCode,
        at: &CheckedNodeId,
        form: &str,
    ) -> SelectedTypeDeclarationsError {
        // The diagnostic is built only after each owned component is payable.
        if let Err(error) = self.charge_structural_id(at, "runtime_diagnostic") {
            return error;
        }
        let node_storage = native_join_storage::node(at);
        if self.charge_storage(node_storage).is_err() {
            return SelectedTypeDeclarationsError::exhausted(self.budget, self.before);
        }
        if self
            .charge_storage(1_u128.saturating_add(form.len() as u128))
            .is_err()
        {
            return SelectedTypeDeclarationsError::exhausted(self.budget, self.before);
        }
        SelectedTypeDeclarationsError {
            code,
            at: Some(Box::new(at.clone())),
            form: Some(form.into()),
            work: self.work(),
            denied: None,
        }
    }

    fn error_pair(
        &mut self,
        code: ExactTypeProjectionErrorCode,
        at: &CheckedNodeId,
        tag: &str,
        semantic: &str,
    ) -> SelectedTypeDeclarationsError {
        if let Err(error) = self.charge_structural_id(at, "runtime_diagnostic") {
            return error;
        }
        if self.charge_storage(native_join_storage::node(at)).is_err() {
            return SelectedTypeDeclarationsError::exhausted(self.budget, self.before);
        }
        let form_len = 2_u128
            .saturating_add(tag.len() as u128)
            .saturating_add(semantic.len() as u128);
        if self.charge_storage(form_len).is_err() {
            return SelectedTypeDeclarationsError::exhausted(self.budget, self.before);
        }
        SelectedTypeDeclarationsError {
            code,
            at: Some(Box::new(at.clone())),
            form: Some(format!("{tag}/{semantic}").into()),
            work: self.work(),
            denied: None,
        }
    }

    fn charge_structural_id(
        &mut self,
        at: &CheckedNodeId,
        form: &str,
    ) -> Result<(), SelectedTypeDeclarationsError> {
        self.charge_storage(2)?;
        let mut pending = vec![at];
        while let Some(id) = pending.pop() {
            self.charge_work(at, form, id.package.len())?;
            self.charge_work(at, form, 8)?;
            for segment in &id.path {
                self.charge(at, form)?;
                match segment {
                    CheckedNodePathSegment::Name(name) => self.charge_work(at, form, name.len())?,
                    CheckedNodePathSegment::Index(_) => self.charge_work(at, form, 8)?,
                    CheckedNodePathSegment::Node(child) => {
                        self.charge_storage(1)?;
                        pending.push(child);
                    }
                }
            }
        }
        Ok(())
    }

    fn charge_node_key(
        &mut self,
        at: &CheckedNodeId,
        form: &str,
        key: &NodeKey,
    ) -> Result<(), SelectedTypeDeclarationsError> {
        self.charge_storage(2)?;
        let mut pending = vec![key.view()];
        while let Some(view) = pending.pop() {
            self.charge_work(at, form, view.package().len())?;
            self.charge_work(at, form, 8)?;
            for segment in view.path() {
                self.charge(at, form)?;
                match segment {
                    PathSegmentView::Text(text) => self.charge_work(at, form, text.len())?,
                    PathSegmentView::Node(child) => {
                        self.charge_storage(1)?;
                        pending.push(child);
                    }
                }
            }
        }
        Ok(())
    }

    fn from_exact(&mut self, error: ExactTypeProjectionError) -> SelectedTypeDeclarationsError {
        if error.denied.is_some() {
            return SelectedTypeDeclarationsError::exhausted(self.budget, self.before);
        }
        // Moving the Exact locus and form adds no owned payload.
        SelectedTypeDeclarationsError {
            code: error.code,
            at: error.at.or(error.requested),
            form: error.form,
            work: self.work(),
            denied: None,
        }
    }

    fn from_exact_with_form(
        &mut self,
        error: ExactTypeProjectionError,
        form: &str,
    ) -> SelectedTypeDeclarationsError {
        if error.denied.is_some() {
            return SelectedTypeDeclarationsError::exhausted(self.budget, self.before);
        }
        if self.storage_text(form).is_err() {
            return SelectedTypeDeclarationsError::exhausted(self.budget, self.before);
        }
        SelectedTypeDeclarationsError {
            code: error.code,
            at: error.at.or(error.requested),
            form: Some(form.into()),
            work: self.work(),
            denied: None,
        }
    }
}

fn projected_field_type(
    package: &CheckedPackageV2,
    identities: &SelectedExactIdentities,
    member_type: Option<&MemberType>,
    optional: bool,
    at: &CheckedNodeId,
    form: &str,
    ctx: &mut ProjectionBudget<'_>,
) -> Result<ValueType, SelectedTypeDeclarationsError> {
    let member_type = member_type
        .ok_or_else(|| ctx.error(ExactTypeProjectionErrorCode::UnsupportedExactType, at, form))?;
    let member_type = if optional {
        match member_type {
            MemberType::Option(inner) => inner.as_ref(),
            _ => {
                return Err(ctx.error(
                    ExactTypeProjectionErrorCode::UnsupportedExactType,
                    at,
                    form,
                ));
            }
        }
    } else {
        member_type
    };
    project_record_field_type(package, identities, member_type, at, ctx.budget)
        .map_err(|error| ctx.from_exact_with_form(error, form))
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
        budget: &mut NativeJoinBudget,
    ) -> Result<SelectedTypeDeclarations, SelectedTypeDeclarationsError> {
        let before = budget.begin();
        let mut ctx = ProjectionBudget { budget, before };
        ctx.charge_storage(1)?;
        let mut selected_source_units = Vec::new();
        for (node, kind) in self.graph().nodes.iter().zip(self.node_kinds()) {
            ctx.charge(&node.node_id, "graph_node")?;
            if *kind == CheckedNodeKind::ScalarType(ScalarTypeForm::Unit)
                && matches!(node.owner, Some(CheckedNodeOwner::Source { .. }))
            {
                ctx.charge_structural_id(&node.node_id, "selected_source_unit")?;
                ctx.storage_node(&node.node_id)?;
                selected_source_units.push(node.node_id.clone());
            } else if matches!(
                kind,
                CheckedNodeKind::ScalarType(
                    ScalarTypeForm::Unit | ScalarTypeForm::Enum | ScalarTypeForm::CompoundUnit
                )
            ) {
                return Err(ctx.error_pair(
                    ExactTypeProjectionErrorCode::UnavailableIdentityWitness,
                    &node.node_id,
                    &node.node_tag,
                    &node.semantic_form,
                ));
            }
        }

        ctx.charge_storage(1)?;
        let mut by_original = BTreeMap::new();
        for (id, declaration) in self.models.selected_declarations() {
            ctx.charge(id, "selected_declaration")?;
            ctx.charge_storage(1)?;
            if by_original
                .insert(
                    (declaration.selection, declaration.node.as_ref()),
                    (id, declaration.form),
                )
                .is_some()
            {
                return Err(ctx.error(
                    ExactTypeProjectionErrorCode::ConflictingIdentityWitness,
                    id,
                    "selected_declaration",
                ));
            }
            if matches!(declaration.form, DeclarationForm::VariantType) {
                return Err(ctx.error(
                    ExactTypeProjectionErrorCode::UnavailableIdentityWitness,
                    id,
                    declaration.form.form(),
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
                ctx.charge(root, "selected_document_type")?;
                if !by_original.contains_key(&(selection, original.as_ref())) {
                    return Err(ctx.error(
                        ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration,
                        root,
                        "selected_document_type",
                    ));
                }
            }
            for original in model.tuple_types.keys() {
                ctx.charge(root, "selected_document_tuple")?;
                if !by_original.contains_key(&(selection, original.as_ref())) {
                    return Err(ctx.error(
                        ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration,
                        root,
                        "selected_document_tuple",
                    ));
                }
            }
            if let Some(original) = model.variant_types.keys().next() {
                ctx.charge(root, "selected_document_type")?;
                return Err(ctx.error(
                    ExactTypeProjectionErrorCode::UnavailableIdentityWitness,
                    by_original
                        .get(&(selection, original.as_ref()))
                        .map_or(root, |(id, _)| id),
                    "selected_variant",
                ));
            }
            if !model.units.is_empty() {
                ctx.charge(root, "selected_unit")?;
                return Err(ctx.error(
                    ExactTypeProjectionErrorCode::UnavailableIdentityWitness,
                    root,
                    "selected_unit",
                ));
            }
        }

        ctx.charge_storage(3)?;
        let mut tuple_by_node = BTreeMap::new();
        let mut source_unit_by_node = BTreeMap::new();
        let mut exact_keys = BTreeSet::new();
        for row in identities.declaration_rows() {
            ctx.charge(&row.structural_node, "selected_exact_key")?;
            let key = match &row.exact_identity {
                SelectedExactIdentity::Record(key)
                | SelectedExactIdentity::Union(key)
                | SelectedExactIdentity::DeclaredUnit(UnitId::Declared(key)) => Some(key),
                SelectedExactIdentity::Object(_)
                | SelectedExactIdentity::DeclaredUnit(UnitId::Compound(_)) => None,
            };
            if let Some(key) = key {
                ctx.charge_node_key(&row.structural_node, "selected_exact_key", key)?;
                ctx.charge_storage(2)?;
                exact_keys.insert(key.clone());
            }
        }
        for row in tuple_rows {
            ctx.charge(&row.structural_node, "selected_tuple_row")?;
            ctx.charge_structural_id(&row.structural_node, "selected_tuple_row")?;
            ctx.charge_work(
                &row.structural_node,
                "selected_tuple_row",
                row.selected_document.len(),
            )?;
            ctx.charge_work(
                &row.structural_node,
                "selected_tuple_row",
                row.original_identity.len(),
            )?;
            ctx.charge_node_key(&row.structural_node, "selected_tuple_row", &row.node_key)?;
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
                return Err(ctx.error(code, &row.structural_node, "selected_tuple_row"));
            };
            if document != row.selected_document.as_ref()
                || original != row.original_identity.as_ref()
                || form != DeclarationForm::TupleValueType
            {
                return Err(ctx.error(
                    ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                    &row.structural_node,
                    "selected_tuple_row",
                ));
            }
            ctx.storage_node(&row.structural_node)?;
            ctx.charge_storage(2)?;
            if tuple_by_node
                .insert(row.structural_node.clone(), row)
                .is_some()
            {
                return Err(ctx.error(
                    ExactTypeProjectionErrorCode::ConflictingIdentityWitness,
                    &row.structural_node,
                    "selected_tuple_row",
                ));
            }
            ctx.charge_storage(2)?;
            if !exact_keys.insert(row.node_key.clone()) {
                return Err(ctx.error(
                    ExactTypeProjectionErrorCode::ConflictingIdentityWitness,
                    &row.structural_node,
                    "selected_tuple_row",
                ));
            }
        }
        for row in source_unit_rows {
            ctx.charge(&row.structural_node, "selected_source_unit_row")?;
            ctx.charge_structural_id(&row.structural_node, "selected_source_unit_row")?;
            ctx.charge_work(
                &row.structural_node,
                "selected_source_unit_row",
                row.source_authority.len(),
            )?;
            ctx.charge_work(
                &row.structural_node,
                "selected_source_unit_row",
                row.source_identity.len(),
            )?;
            for segment in &row.qualified_declaration {
                ctx.charge(&row.structural_node, "selected_source_unit_row")?;
                ctx.charge_work(
                    &row.structural_node,
                    "selected_source_unit_row",
                    segment.len(),
                )?;
            }
            if let UnitId::Declared(key) = &row.unit_id {
                ctx.charge_node_key(&row.structural_node, "selected_source_unit_row", key)?;
            }
            let Some(node) = self.node(&row.structural_node) else {
                let code = if row.structural_node.package.as_ref() != self.name() {
                    ExactTypeProjectionErrorCode::ForeignNode
                } else {
                    ExactTypeProjectionErrorCode::UnknownNode
                };
                return Err(ctx.error(code, &row.structural_node, "selected_source_unit_row"));
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
                return Err(ctx.error(
                    ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                    &row.structural_node,
                    "selected_source_unit_row",
                ));
            }
            let UnitId::Declared(key) = &row.unit_id else {
                unreachable!("checked declared domain")
            };
            ctx.storage_node(&row.structural_node)?;
            ctx.charge_storage(2)?;
            if source_unit_by_node
                .insert(row.structural_node.clone(), row)
                .is_some()
            {
                return Err(ctx.error(
                    ExactTypeProjectionErrorCode::ConflictingIdentityWitness,
                    &row.structural_node,
                    "selected_source_unit_row",
                ));
            }
            ctx.charge_storage(2)?;
            if !exact_keys.insert(key.clone()) {
                return Err(ctx.error(
                    ExactTypeProjectionErrorCode::ConflictingIdentityWitness,
                    &row.structural_node,
                    "selected_source_unit_row",
                ));
            }
        }
        for id in &selected_source_units {
            ctx.charge(id, "selected_source_unit")?;
            if !source_unit_by_node.contains_key(id) {
                return Err(ctx.error(
                    ExactTypeProjectionErrorCode::UnavailableIdentityWitness,
                    id,
                    "selected_source_unit",
                ));
            }
        }

        let admitted_identities = identities
            .with_admitted_type_rows(&tuple_by_node, &source_unit_by_node, ctx.budget)
            .map_err(|error| ctx.from_exact(error))?;
        let identities = &admitted_identities;
        ctx.charge_storage(3)?;
        let mut records = Vec::new();
        let mut tuples = Vec::new();
        let mut objects = Vec::new();
        for (id, declaration) in self.models.selected_declarations() {
            let Some(model) = self.models.selected_models().get(declaration.selection) else {
                return Err(ctx.error(
                    ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration,
                    id,
                    "selected_model",
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
                    ctx.error(
                        ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration,
                        id,
                        declaration.form.form(),
                    )
                })?
            };
            ctx.charge_storage(2)?;
            let mut record_parents = Vec::new();
            let mut object_parents = Vec::new();
            for original in supertypes {
                ctx.charge(id, "selected_supertype")?;
                let Some((parent_id, parent_form)) =
                    by_original.get(&(declaration.selection, original.as_ref()))
                else {
                    return Err(ctx.error(
                        ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration,
                        id,
                        "selected_supertype",
                    ));
                };
                if *parent_form != expected_form {
                    return Err(ctx.error(
                        ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                        parent_id,
                        "selected_supertype",
                    ));
                }
                let identity = identities
                    .declaration_identity(self, parent_id)
                    .map_err(|code| ctx.error(code, parent_id, "selected_supertype"))?;
                match (declaration.form, identity) {
                    (DeclarationForm::RecordValueType, SelectedExactIdentity::Record(key)) => {
                        ctx.charge_storage(1)?;
                        record_parents.push(key.clone());
                    }
                    (DeclarationForm::ObjectType, SelectedExactIdentity::Object(key)) => {
                        ctx.charge_storage(
                            1_u128.saturating_add(native_join_storage::effective_identity(&key)),
                        )?;
                        object_parents.push(key.clone());
                    }
                    _ => {
                        return Err(ctx.error(
                            ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                            parent_id,
                            "selected_supertype",
                        ));
                    }
                }
            }
            match declaration.form {
                DeclarationForm::TupleValueType => {
                    let Some(row) = tuple_by_node.get(id) else {
                        return Err(ctx.error(
                            ExactTypeProjectionErrorCode::UnavailableIdentityWitness,
                            id,
                            "tuple_value_type",
                        ));
                    };
                    let Some(positions) = model.tuple_types.get(&declaration.node) else {
                        return Err(ctx.error(
                            ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration,
                            id,
                            "tuple_value_type",
                        ));
                    };
                    ctx.charge_storage(1)?;
                    let mut projected = Vec::with_capacity(positions.len());
                    for position in positions {
                        ctx.charge(id, "tuple_position")?;
                        let member_type = model.element_type(position).ok_or_else(|| {
                            ctx.error(
                                ExactTypeProjectionErrorCode::UnsupportedExactType,
                                id,
                                "tuple_position",
                            )
                        })?;
                        let value_type = projected_field_type(
                            self,
                            identities,
                            Some(&member_type),
                            false,
                            id,
                            "tuple_position",
                            &mut ctx,
                        )?;
                        ctx.charge_storage(1)?;
                        projected.push(value_type);
                    }
                    ctx.charge_storage(2)?;
                    ctx.storage_text(&declaration.node)?;
                    tuples.push(SelectedTypeTuple {
                        original_identity: declaration.node.clone(),
                        node_key: row.node_key.clone(),
                        positions: projected,
                    });
                }
                DeclarationForm::RecordValueType => {
                    let projection = self
                        .project_exact_type(id, identities, ctx.budget)
                        .map_err(|error| ctx.from_exact(error))?;
                    let view = projection.record_view().ok_or_else(|| {
                        ctx.error(
                            ExactTypeProjectionErrorCode::UnsupportedExactType,
                            id,
                            "record_value_type",
                        )
                    })?;
                    ctx.storage_record_view(view)?;
                    ctx.storage_text(&declaration.node)?;
                    ctx.charge_storage(1)?;
                    records.push(SelectedTypeRecord {
                        original_identity: declaration.node.clone(),
                        view: view.clone(),
                        supertypes: record_parents,
                    });
                }
                DeclarationForm::ObjectType => {
                    ctx.charge(id, "object_type")?;
                    let declared = model.object_types.get(&declaration.node).ok_or_else(|| {
                        ctx.error(
                            ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration,
                            id,
                            "object_type",
                        )
                    })?;
                    ctx.charge_storage(1)?;
                    let mut projected_operations = Vec::with_capacity(declared.operations.len());
                    for operation in &declared.operations {
                        ctx.charge(id, "object_operation")?;
                        let name = super::model_members::member_name(&operation.identity);
                        let row = self
                            .selected_operation_effect(&model.identity, id, name)
                            .ok_or_else(|| {
                                ctx.error(
                                    ExactTypeProjectionErrorCode::UnavailableIdentityWitness,
                                    id,
                                    "object_operation_effect",
                                )
                            })?;
                        ctx.charge_storage(1)?;
                        let mut parameters = Vec::with_capacity(operation.parameters.len());
                        for parameter in &operation.parameters {
                            ctx.charge(id, "object_operation_parameter")?;
                            let name = parameter.name.as_ref().ok_or_else(|| {
                                ctx.error(
                                    ExactTypeProjectionErrorCode::UnsupportedExactType,
                                    id,
                                    "object_operation_parameter_name",
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
                                &mut ctx,
                            )?;
                            ctx.charge_storage(1)?;
                            ctx.storage_text(name)?;
                            parameters.push(SelectedTypeOperationParameter {
                                name: name.clone(),
                                value_type,
                            });
                        }
                        let result = operation
                            .result
                            .as_ref()
                            .map(|slot| {
                                ctx.charge(id, "object_operation_result")?;
                                let member_type = model.slot_type(slot);
                                projected_field_type(
                                    self,
                                    identities,
                                    member_type.as_ref(),
                                    false,
                                    id,
                                    "object_operation_result_type",
                                    &mut ctx,
                                )
                            })
                            .transpose()?;
                        let effect = &row.effect;
                        for _ in &effect.modifies {
                            ctx.charge(id, "object_operation_effect")?;
                        }
                        for _ in &effect.creates {
                            ctx.charge(id, "object_operation_effect")?;
                        }
                        for _ in &effect.deletes {
                            ctx.charge(id, "object_operation_effect")?;
                        }
                        for _ in &effect.deletes_scoped {
                            ctx.charge(id, "object_operation_effect")?;
                        }
                        ctx.charge_storage(1)?;
                        ctx.storage_text(&operation.identity)?;
                        ctx.storage_text(name)?;
                        ctx.storage_effect(effect)?;
                        ctx.storage_text(&row.provenance.member)?;
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
                        ctx.charge(id, "declared_object_field")?;
                        if field.redefines.is_some() {
                            return Err(ctx.error(
                                ExactTypeProjectionErrorCode::UnsupportedExactType,
                                id,
                                "object_field_redefinition",
                            ));
                        }
                    }
                    let effective_id = match identities
                        .declaration_identity(self, id)
                        .map_err(|code| ctx.error(code, id, "object_type"))?
                    {
                        SelectedExactIdentity::Object(key) => {
                            ctx.charge_storage(native_join_storage::effective_identity(&key))?;
                            key.clone()
                        }
                        _ => {
                            return Err(ctx.error(
                                ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                                id,
                                "object_type",
                            ));
                        }
                    };
                    let (ambiguous, fields) =
                        self.models.selected_object_fields(id).ok_or_else(|| {
                            ctx.error(
                                ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration,
                                id,
                                "object_type_fields",
                            )
                        })?;
                    if ambiguous.is_some() {
                        return Err(ctx.error(
                            ExactTypeProjectionErrorCode::UnsupportedExactType,
                            id,
                            "ambiguous_object_field",
                        ));
                    }
                    ctx.charge_storage(1)?;
                    let mut projected_fields = Vec::new();
                    for field in fields {
                        ctx.charge(id, "object_field")?;
                        let value_type = projected_field_type(
                            self,
                            identities,
                            field.member_type,
                            field.optional,
                            id,
                            "object_field_type",
                            &mut ctx,
                        )?;
                        ctx.charge_storage(1)?;
                        ctx.storage_text(field.identity)?;
                        ctx.storage_text(field.name)?;
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
                    ctx.charge_storage(1)?;
                    let mut by_identity = BTreeMap::new();
                    for field in &projected_fields {
                        ctx.charge(id, "object_field_join")?;
                        ctx.charge_storage(1)?;
                        if by_identity
                            .insert(field.original_identity(), field)
                            .is_some()
                        {
                            return Err(ctx.error(
                                ExactTypeProjectionErrorCode::ConflictingIdentityWitness,
                                id,
                                "object_field_join",
                            ));
                        }
                    }
                    ctx.charge_storage(1)?;
                    let mut declared_fields = Vec::new();
                    for field in &declared.fields {
                        ctx.charge(id, "declared_object_field")?;
                        let projected =
                            by_identity.get(field.identity.as_ref()).ok_or_else(|| {
                                ctx.error(
                                    ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration,
                                    id,
                                    "declared_object_field",
                                )
                            })?;
                        ctx.charge_storage(1)?;
                        ctx.storage_text(projected.original_identity())?;
                        ctx.storage_text(projected.name())?;
                        ctx.storage_value_type(projected.value_type())?;
                        declared_fields.push((*projected).clone());
                    }
                    ctx.charge_storage(1)?;
                    ctx.storage_text(&declaration.node)?;
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
        ctx.charge_storage(1)?;
        let mut nominal_rows = Vec::new();
        for id in &selected_source_units {
            ctx.charge_storage(1)?;
            let row = source_unit_by_node
                .get(id)
                .expect("checked complete source-unit rows");
            let node = self
                .node(id)
                .expect("admitted source unit remains in graph");
            nominal_rows.push(NominalDeclarationRow {
                node: id,
                owner: node.owner.as_ref().expect("admitted source owner"),
                declaration: node
                    .declaration
                    .as_ref()
                    .expect("admitted source declaration"),
                identity: NominalIdentity::Unit(&row.unit_id),
            });
        }
        let unit_read = self
            .read_unit_definition(&selected_source_units, &nominal_rows, ctx.budget)
            .map_err(|error| {
                if ctx.budget.first_unpaid().is_some() {
                    return SelectedTypeDeclarationsError::exhausted(ctx.budget, before);
                }
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
                ctx.error(code, error.node, "selected_source_unit")
            })?;
        ctx.charge_storage(2)?;
        let mut units = Vec::new();
        for unit in &unit_read.units {
            let row = source_unit_by_node
                .get(&unit.node.node_id)
                .expect("read selected source unit");
            ctx.charge_storage(1)?;
            ctx.storage_node(&unit.node.node_id)?;
            ctx.storage_text(&row.source_authority)?;
            ctx.storage_text(&row.source_identity)?;
            ctx.charge_storage(1)?;
            for segment in &row.qualified_declaration {
                ctx.storage_text(segment)?;
            }
            ctx.charge_storage(1)?;
            ctx.storage_node(&unit.dimension.node_id)?;
            ctx.storage_dimension_terms(unit.dimension_terms)?;
            if let Some(target) = unit.target {
                ctx.storage_node(&target.node_id)?;
            }
            ctx.storage_rational(unit.scale)?;
            ctx.storage_rational(unit.offset)?;
            units.push(SelectedTypeUnit {
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
            });
        }
        let mut dimensions = Vec::new();
        for dimension in &unit_read.dimensions {
            ctx.charge_storage(1)?;
            ctx.storage_node(&dimension.node.node_id)?;
            ctx.storage_dimension_terms(dimension.terms)?;
            dimensions.push(SelectedTypeDimension {
                structural_node: dimension.node.node_id.clone(),
                terms: dimension.terms.to_vec(),
            });
        }
        ctx.budget.finish(before);
        Ok(SelectedTypeDeclarations {
            records,
            tuples,
            objects,
            units,
            dimensions,
            work: ctx.work(),
        })
    }
}
