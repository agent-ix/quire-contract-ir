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
    NativeJoinChargeKind, NativeJoinDeniedCharge, NativeJoinLocus, NativeJoinLocusSource,
    ScalarTypeForm, SelectedExactIdentities, SelectedExactIdentity, SelectedRecordView,
    SelectedSourceUnitRow, SelectedTupleDeclarationRow,
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

/// A selected model-owned unit and its admitted exact conversion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedTypeModelUnit {
    /// Final structural unit node.
    pub structural_node: CheckedNodeId,
    /// Selected model document identity.
    pub selected_document: Box<str>,
    /// Original unit identity in the selected model.
    pub original_identity: Box<str>,
    /// QSL-issued declared unit identity.
    pub unit_id: UnitId,
    /// Admitted dimension node.
    pub dimension_node: CheckedNodeId,
    /// Admitted dimension terms.
    pub dimension_terms: Vec<DimensionTerm>,
    /// Conversion target, if this unit is derived.
    pub target_node: Option<CheckedNodeId>,
    /// Exact admitted scale.
    pub scale: CheckedRational,
    /// Exact admitted offset.
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
    model_units: Vec<SelectedTypeModelUnit>,
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

    /// Reached model-owned units in structural identity order.
    pub fn model_units(&self) -> &[SelectedTypeModelUnit] {
        &self.model_units
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
#[error("selected runtime type projection refused: {code:?} at {form:?} (native locus {denied_locus:?})")]
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
    /// Fixed-size native location resolved against the original package and rows.
    pub denied_locus: Option<NativeJoinLocus>,
}

impl SelectedTypeDeclarationsError {
    /// Resolve a denied node using the package and row slices supplied to projection.
    /// Package allocations and unresolved declaration request coordinates have no
    /// resolved node. Their fixed input coordinate remains in `denied_locus`;
    /// this resolver never searches for an identity after a denied charge.
    pub fn denied_node<'a>(
        &self,
        package: &'a CheckedPackageV2,
        tuple_rows: &'a [SelectedTupleDeclarationRow],
        source_unit_rows: &'a [SelectedSourceUnitRow],
    ) -> Option<&'a CheckedNodeId> {
        match self.denied_locus?.source {
            NativeJoinLocusSource::Package
            | NativeJoinLocusSource::MonitorSeed(_)
            | NativeJoinLocusSource::ReachedRow(_) => None,
            NativeJoinLocusSource::GraphNode(index) => {
                package.resolved_graph().nodes.get(index).map(|node| &node.node_id)
            }
            NativeJoinLocusSource::SelectedDeclaration(_)
            | NativeJoinLocusSource::IdentityDeclaration(_) => None,
            NativeJoinLocusSource::TupleRow(index) => {
                tuple_rows.get(index).map(|row| &row.structural_node)
            }
            NativeJoinLocusSource::SourceUnitRow(index) => {
                source_unit_rows.get(index).map(|row| &row.structural_node)
            }
        }
    }

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
            denied_locus: budget.first_unpaid_locus(),
        }
    }
}

struct ProjectionBudget<'a> {
    budget: &'a mut NativeJoinBudget,
    before: NativeJoinCharge,
}

impl ProjectionBudget<'_> {
    fn package_locus(&mut self, form: &'static str) {
        self.budget.set_active_locus(Some(NativeJoinLocus {
            source: NativeJoinLocusSource::Package,
            form,
        }));
    }

    fn activate(&mut self, source: NativeJoinLocusSource, form: &'static str) {
        self.budget
            .set_active_locus(Some(NativeJoinLocus { source, form }));
    }

    fn form(&mut self, form: &'static str) {
        self.budget.set_locus_form(form);
    }

    fn work(&self) -> u64 {
        self.budget.consumed().work - self.before.work
    }

    fn charge(
        &mut self,
        _at: &CheckedNodeId,
        form: &'static str,
    ) -> Result<(), SelectedTypeDeclarationsError> {
        self.form(form);
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
        form: &'static str,
        amount: usize,
    ) -> Result<(), SelectedTypeDeclarationsError> {
        self.form(form);
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
        form: &'static str,
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
            denied_locus: None,
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
            denied_locus: None,
        }
    }

    fn charge_structural_id(
        &mut self,
        at: &CheckedNodeId,
        form: &'static str,
    ) -> Result<(), SelectedTypeDeclarationsError> {
        self.form(form);
        self.charge_storage(2)?;
        let mut pending = vec![at];
        while let Some(id) = pending.pop() {
            self.charge_work(at, form, id.package.len())?;
            self.charge_work(at, form, 8)?;
            for segment in &id.path {
                self.charge(at, form)?;
                match segment {
                    CheckedNodePathSegment::Name(name) => self.charge_work(at, form, name.len())?,
                    CheckedNodePathSegment::Index(_) | CheckedNodePathSegment::Operand(_) => {
                        self.charge_work(at, form, 8)?
                    }
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
        form: &'static str,
        key: &NodeKey,
    ) -> Result<(), SelectedTypeDeclarationsError> {
        self.form(form);
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

    fn exact_read<T>(
        &mut self,
        read: impl FnOnce(&mut NativeJoinBudget) -> Result<T, ExactTypeProjectionError>,
    ) -> Result<T, ExactTypeProjectionError> {
        let previous = self.budget.exact_diagnostics();
        let previous_locus = self.budget.active_locus();
        self.budget.set_exact_diagnostics(true);
        let result = read(self.budget);
        self.budget.set_exact_diagnostics(previous);
        self.budget.set_active_locus(previous_locus);
        result
    }

    fn exact_error(&mut self, error: ExactTypeProjectionError) -> SelectedTypeDeclarationsError {
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
            denied_locus: None,
        }
    }

    fn exact_error_with_form(
        &mut self,
        error: ExactTypeProjectionError,
        form: &'static str,
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
            denied_locus: None,
        }
    }
}

fn projected_field_type(
    package: &CheckedPackageV2,
    identities: &SelectedExactIdentities,
    member_type: Option<&MemberType>,
    optional: bool,
    at: &CheckedNodeId,
    form: &'static str,
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
    ctx.exact_read(|budget| project_record_field_type(package, identities, member_type, at, budget))
        .map_err(|error| ctx.exact_error_with_form(error, form))
}

impl CheckedPackageV2 {
    /// Project every selected record, tuple, object and source unit as one bounded view.
    ///
    /// Enum and model-unit selections return a typed refusal before a view is
    /// exposed. Identity rows must originate from
    /// the stage selecting the same documents; IR checks their structural join
    /// but cannot authenticate an opaque Exact key supplied by another caller.
    /// The caller budget charges cumulative traversal work and owned storage;
    /// refusal preserves admitted charges and leaves no successful delta.
    pub fn project_selected_type_declarations(
        &self,
        identities: &SelectedExactIdentities,
        tuple_rows: &[SelectedTupleDeclarationRow],
        source_unit_rows: &[SelectedSourceUnitRow],
        budget: &mut NativeJoinBudget,
    ) -> Result<SelectedTypeDeclarations, SelectedTypeDeclarationsError> {
        self.project_type_declarations_for_nodes(
            identities,
            tuple_rows,
            source_unit_rows,
            budget,
            None,
            0,
        )
    }

    pub(super) fn project_type_declarations_for_nodes(
        &self,
        identities: &SelectedExactIdentities,
        tuple_rows: &[SelectedTupleDeclarationRow],
        source_unit_rows: &[SelectedSourceUnitRow],
        budget: &mut NativeJoinBudget,
        reached: Option<&BTreeSet<CheckedNodeId>>,
        prior_work: u64,
    ) -> Result<SelectedTypeDeclarations, SelectedTypeDeclarationsError> {
        let before = budget.begin();
        let previous_locus = budget.active_locus();
        budget.set_active_locus(Some(NativeJoinLocus {
            source: NativeJoinLocusSource::Package,
            form: "runtime_projection",
        }));
        let result = (|| {
            let mut ctx = ProjectionBudget { budget, before };
            ctx.charge_storage(1)?;
            let mut selected_source_units = Vec::new();
            ctx.charge_storage(1)?;
            let mut selected_model_units = Vec::new();
            ctx.charge_storage(1)?;
            let mut selected_declarations = Vec::new();
            let mut reached_nodes = Vec::new();
            if let Some(reached) = reached {
                ctx.charge_storage(1)?;
                for (request_index, id) in reached.iter().enumerate() {
                    ctx.activate(
                        NativeJoinLocusSource::ReachedRow(request_index),
                        "reached_graph_node",
                    );
                    ctx.charge_structural_id(id, "reached_graph_node")?;
                    let Some((position, node)) = self.node_with_position(id) else {
                        return Err(ctx.error(
                            ExactTypeProjectionErrorCode::UnknownNode,
                            id,
                            "graph_node",
                        ));
                    };
                    ctx.activate(
                        NativeJoinLocusSource::GraphNode(position),
                        "reached_graph_node",
                    );
                    ctx.charge_storage(1)?;
                    reached_nodes.push((position, (node, &self.node_kinds()[position])));
                }
            }
            for (index, (node, kind)) in self
                .resolved_graph()
                .nodes
                .iter()
                .zip(self.node_kinds())
                .enumerate()
                .take(if reached.is_none() {
                    self.resolved_graph().nodes.len()
                } else {
                    0
                })
                .chain(reached_nodes)
            {
                ctx.activate(NativeJoinLocusSource::GraphNode(index), "graph_node");
                ctx.charge(&node.node_id, "graph_node")?;
                if reached.is_some() {
                    ctx.charge_structural_id(&node.node_id, "reached_declaration_lookup")?;
                    if let Some(declaration) = self.models.selected_declaration_entry(&node.node_id)
                    {
                        ctx.charge_storage(1)?;
                        selected_declarations.push((
                            &node.node_id,
                            declaration,
                            NativeJoinLocusSource::GraphNode(index),
                        ));
                    }
                }
                if *kind == CheckedNodeKind::ScalarType(ScalarTypeForm::Unit)
                    && matches!(node.owner, Some(CheckedNodeOwner::Source { .. }))
                {
                    ctx.charge_structural_id(&node.node_id, "selected_source_unit")?;
                    ctx.storage_node(&node.node_id)?;
                    ctx.charge_storage(1)?;
                    selected_source_units.push((index, node.node_id.clone()));
                } else if reached.is_some()
                    && *kind == CheckedNodeKind::ScalarType(ScalarTypeForm::Unit)
                    && matches!(node.owner, Some(CheckedNodeOwner::Model { .. }))
                {
                    ctx.charge_structural_id(&node.node_id, "selected_model_unit")?;
                    ctx.storage_node(&node.node_id)?;
                    ctx.charge_storage(1)?;
                    selected_model_units.push((index, node.node_id.clone()));
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

            if reached.is_none() {
                for (index, (id, declaration)) in self.models.selected_declarations().enumerate() {
                    let source = NativeJoinLocusSource::SelectedDeclaration(index);
                    ctx.activate(source, "selected_declaration_index");
                    ctx.charge(id, "selected_declaration_index")?;
                    ctx.charge_storage(1)?;
                    selected_declarations.push((id, declaration, source));
                }
            }
            ctx.package_locus("selected_declaration_index");
            ctx.charge_storage(1)?;
            let mut by_original = BTreeMap::new();
            for &(id, declaration, source) in &selected_declarations {
                ctx.activate(source, "selected_declaration");
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
                if reached.is_none() && matches!(declaration.form, DeclarationForm::VariantType) {
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
                if reached.is_some() {
                    continue;
                }
                ctx.activate(
                    NativeJoinLocusSource::GraphNode(model.root),
                    "selected_document_type",
                );
                let root = &self
                    .resolved_graph()
                    .nodes
                    .get(model.root)
                    .expect("admitted selected model root is a graph node")
                    .node_id;
                for original in model.record_types.keys().chain(model.object_types.keys()) {
                    if model.end_declaration_form(original)
                        == Some(DeclarationForm::SystemsInterface)
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

            ctx.package_locus("runtime_identity_indices");
            ctx.charge_storage(4)?;
            let mut model_unit_by_node = BTreeMap::new();
            let mut tuple_by_node = BTreeMap::new();
            let mut source_unit_by_node = BTreeMap::new();
            let mut exact_keys = BTreeSet::new();
            for (index, row) in identities.declaration_rows().enumerate() {
                ctx.activate(
                    NativeJoinLocusSource::IdentityDeclaration(index),
                    "selected_exact_key",
                );
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
                if row.form == super::SelectedExactDeclarationForm::DeclaredUnit {
                    ctx.storage_node(&row.structural_node)?;
                    ctx.charge_storage(1)?;
                    model_unit_by_node.insert(row.structural_node.clone(), row);
                }
            }
            for (index, row) in tuple_rows.iter().enumerate() {
                ctx.activate(NativeJoinLocusSource::TupleRow(index), "selected_tuple_row");
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
            }
            for (index, row) in source_unit_rows.iter().enumerate() {
                ctx.activate(
                    NativeJoinLocusSource::SourceUnitRow(index),
                    "selected_source_unit_row",
                );
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
            }
            // The checked join remains the sole semantic tuple/unit authority.
            let admitted_identities = ctx
                .exact_read(|budget| {
                    identities.with_admitted_type_rows(self, tuple_rows, source_unit_rows, budget)
                })
                .map_err(|error| ctx.exact_error(error))?;
            for (index, row) in tuple_rows.iter().enumerate() {
                ctx.activate(
                    NativeJoinLocusSource::TupleRow(index),
                    "selected_tuple_index",
                );
                ctx.storage_node(&row.structural_node)?;
                ctx.charge_storage(4)?;
                if tuple_by_node
                    .insert(row.structural_node.clone(), row)
                    .is_some()
                    || !exact_keys.insert(row.node_key.clone())
                {
                    return Err(ctx.error(
                        ExactTypeProjectionErrorCode::ConflictingIdentityWitness,
                        &row.structural_node,
                        "selected_tuple_row",
                    ));
                }
            }
            for (index, row) in source_unit_rows.iter().enumerate() {
                ctx.activate(
                    NativeJoinLocusSource::SourceUnitRow(index),
                    "selected_source_unit_index",
                );
                let UnitId::Declared(key) = &row.unit_id else {
                    unreachable!("checked declared domain")
                };
                ctx.storage_node(&row.structural_node)?;
                ctx.charge_storage(4)?;
                if source_unit_by_node
                    .insert(row.structural_node.clone(), row)
                    .is_some()
                    || !exact_keys.insert(key.clone())
                {
                    return Err(ctx.error(
                        ExactTypeProjectionErrorCode::ConflictingIdentityWitness,
                        &row.structural_node,
                        "selected_source_unit_row",
                    ));
                }
            }
            for (position, id) in &selected_source_units {
                ctx.activate(
                    NativeJoinLocusSource::GraphNode(*position),
                    "selected_source_unit",
                );
                ctx.charge(id, "selected_source_unit")?;
                if !source_unit_by_node.contains_key(id) {
                    return Err(ctx.error(
                        ExactTypeProjectionErrorCode::UnavailableIdentityWitness,
                        id,
                        "selected_source_unit",
                    ));
                }
            }

            for (position, id) in &selected_model_units {
                ctx.activate(
                    NativeJoinLocusSource::GraphNode(*position),
                    "selected_model_unit",
                );
                ctx.charge_structural_id(id, "selected_model_unit")?;
                if !model_unit_by_node.contains_key(id) {
                    return Err(ctx.error(
                        ExactTypeProjectionErrorCode::UnavailableIdentityWitness,
                        id,
                        "selected_model_unit",
                    ));
                }
            }

            let identities = &admitted_identities;
            ctx.package_locus("runtime_declarations");
            ctx.charge_storage(3)?;
            let mut records = Vec::new();
            let mut tuples = Vec::new();
            let mut objects = Vec::new();
            for &(id, declaration, source) in &selected_declarations {
                ctx.activate(source, "selected_declaration");
                ctx.charge(id, "selected_declaration")?;
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
                                1_u128.saturating_add(native_join_storage::effective_identity(key)),
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
                        let mut projected = Vec::new();
                        for position in positions {
                            ctx.charge(id, "tuple_position")?;
                            let member_type = model
                                .element_type_runtime(position, ctx.budget)
                                .map_err(|_| {
                                    SelectedTypeDeclarationsError::exhausted(ctx.budget, ctx.before)
                                })?
                                .ok_or_else(|| {
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
                            .map_err(|error| ctx.exact_error(error))?;
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
                        let declared =
                            model.object_types.get(&declaration.node).ok_or_else(|| {
                                ctx.error(
                                    ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration,
                                    id,
                                    "object_type",
                                )
                            })?;
                        ctx.charge_storage(1)?;
                        let mut projected_operations = Vec::new();
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
                            let mut parameters = Vec::new();
                            for parameter in &operation.parameters {
                                ctx.charge(id, "object_operation_parameter")?;
                                let name = parameter.name.as_ref().ok_or_else(|| {
                                    ctx.error(
                                        ExactTypeProjectionErrorCode::UnsupportedExactType,
                                        id,
                                        "object_operation_parameter_name",
                                    )
                                })?;
                                let member_type = model
                                    .slot_type_runtime(parameter, ctx.budget)
                                    .map_err(|_| {
                                        SelectedTypeDeclarationsError::exhausted(
                                            ctx.budget, ctx.before,
                                        )
                                    })?;
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
                                    let member_type = model
                                        .slot_type_runtime(slot, ctx.budget)
                                        .map_err(|_| {
                                            SelectedTypeDeclarationsError::exhausted(
                                                ctx.budget, ctx.before,
                                            )
                                        })?;
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
                                ctx.charge_storage(native_join_storage::effective_identity(key))?;
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
            selected_source_units.sort_by(|(_, left), (_, right)| left.cmp(right));
            ctx.package_locus("runtime_nominal_rows");
            ctx.charge_storage(4)?;
            let mut requested_units = Vec::new();
            let mut requested_positions = Vec::new();
            let mut dimension_positions = Vec::new();
            let mut nominal_rows = Vec::new();
            for (position, id) in selected_source_units {
                ctx.activate(
                    NativeJoinLocusSource::GraphNode(position),
                    "selected_source_unit",
                );
                ctx.charge_storage(2)?;
                requested_units.push(id);
                requested_positions.push(position);
            }
            for (index, id) in requested_units.iter().enumerate() {
                let position = requested_positions[index];
                ctx.activate(
                    NativeJoinLocusSource::GraphNode(position),
                    "selected_source_unit",
                );
                ctx.charge(id, "selected_source_unit")?;
                let row = source_unit_by_node
                    .get(id)
                    .expect("checked complete source-unit rows");
                let node = &self.resolved_graph().nodes[position];
                ctx.charge_storage(1)?;
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
                .read_unit_definition_runtime(
                    &requested_units,
                    &nominal_rows,
                    &requested_positions,
                    &mut dimension_positions,
                    ctx.budget,
                )
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
                        NominalReadCode::UnsupportedForm
                        | NominalReadCode::UnresolvedConversion => {
                            ExactTypeProjectionErrorCode::UnsupportedExactType
                        }
                        NominalReadCode::LimitReached => {
                            ExactTypeProjectionErrorCode::ExhaustedWork
                        }
                    };
                    ctx.error(code, error.node, "selected_source_unit")
                })?;
            ctx.package_locus("runtime_unit_results");
            ctx.charge_storage(2)?;
            let mut units = Vec::new();
            for (index, unit) in unit_read.units.iter().enumerate() {
                ctx.activate(
                    NativeJoinLocusSource::ReachedRow(index),
                    "selected_source_unit_result",
                );
                ctx.charge_structural_id(&unit.node.node_id, "selected_source_unit_result")?;
                let (position, _) = self
                    .node_with_position(&unit.node.node_id)
                    .expect("read admitted source unit");
                ctx.activate(
                    NativeJoinLocusSource::GraphNode(position),
                    "selected_source_unit",
                );
                ctx.charge(&unit.node.node_id, "selected_source_unit")?;
                ctx.charge_structural_id(&unit.node.node_id, "selected_source_unit")?;
                let Some(row) = source_unit_by_node.get(&unit.node.node_id) else {
                    continue;
                };
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
            for (index, dimension) in unit_read.dimensions.iter().enumerate() {
                ctx.activate(
                    NativeJoinLocusSource::GraphNode(dimension_positions[index]),
                    "selected_dimension",
                );
                ctx.charge_storage(1)?;
                ctx.storage_node(&dimension.node.node_id)?;
                ctx.storage_dimension_terms(dimension.terms)?;
                dimensions.push(SelectedTypeDimension {
                    structural_node: dimension.node.node_id.clone(),
                    terms: dimension.terms.to_vec(),
                });
            }
            ctx.package_locus("runtime_model_units");
            ctx.charge_storage(1)?;
            let mut model_units = Vec::new();
            if let Some(reached) = reached {
                let find_model_node =
                    |model_identity: &str,
                     original: &str,
                     form: ScalarTypeForm,
                     ctx: &mut ProjectionBudget<'_>,
                     at: &CheckedNodeId|
                     -> Result<CheckedNodeId, SelectedTypeDeclarationsError> {
                        for candidate in reached {
                            ctx.charge_structural_id(candidate, "model_unit_dependency")?;
                            let Some(node) = self.node(candidate) else {
                                continue;
                            };
                            if self.node_kind(candidate) == Some(CheckedNodeKind::ScalarType(form))
                                && matches!(&node.owner, Some(CheckedNodeOwner::Model { identity, node })
                                if identity.as_ref() == model_identity && node.as_ref() == original)
                            {
                                ctx.storage_node(candidate)?;
                                return Ok(candidate.clone());
                            }
                        }
                        Err(ctx.error(
                            ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration,
                            at,
                            "model_unit_dependency",
                        ))
                    };
                for (position, id) in &selected_model_units {
                    ctx.activate(NativeJoinLocusSource::GraphNode(*position), "model_unit");
                    ctx.charge(id, "model_unit")?;
                    let row = model_unit_by_node[id];
                    let Some(CheckedNodeOwner::Model {
                        identity,
                        node: original,
                    }) = self.node(id).and_then(|node| node.owner.as_ref())
                    else {
                        return Err(ctx.error(
                            ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                            id,
                            "model_unit_owner",
                        ));
                    };
                    ctx.charge(id, "model_unit_selection")?;
                    let model = self.models.selected_model(identity).ok_or_else(|| {
                        ctx.error(
                            ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration,
                            id,
                            "model_unit_selection",
                        )
                    })?;
                    let unit = model.units.get(original).ok_or_else(|| {
                        ctx.error(
                            ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration,
                            id,
                            "model_unit",
                        )
                    })?;
                    let dimension_node = find_model_node(
                        identity,
                        &unit.dimension,
                        ScalarTypeForm::Dimension,
                        &mut ctx,
                        id,
                    )?;
                    ctx.charge_storage(4)?;
                    let mut pending = vec![unit.dimension.as_ref()];
                    let mut seen_dimensions = BTreeSet::new();
                    let mut unit_terms = Vec::new();
                    while let Some(current) = pending.pop() {
                        ctx.charge(id, "model_dimension")?;
                        ctx.charge_work(id, "model_dimension", current.len())?;
                        ctx.charge_storage(1)?;
                        if !seen_dimensions.insert(current) {
                            continue;
                        }
                        let current_node = find_model_node(
                            identity,
                            current,
                            ScalarTypeForm::Dimension,
                            &mut ctx,
                            id,
                        )?;
                        let current_dimension = model.dimensions.get(current).ok_or_else(|| {
                            ctx.error(
                                ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration,
                                id,
                                "model_unit_dimension",
                            )
                        })?;
                        ctx.charge_storage(1)?;
                        let mut terms = Vec::new();
                        for (original, exponent) in &current_dimension.terms {
                            ctx.charge(id, "model_dimension_term")?;
                            ctx.charge_storage(2)?;
                            ctx.storage_text(exponent)?;
                            terms.push(DimensionTerm {
                                dimension_node_id: find_model_node(
                                    identity,
                                    original,
                                    ScalarTypeForm::Dimension,
                                    &mut ctx,
                                    id,
                                )?,
                                exponent: exponent.clone(),
                            });
                            pending.push(original);
                        }
                        if current == unit.dimension.as_ref() {
                            ctx.storage_dimension_terms(&terms)?;
                            unit_terms = terms.clone();
                        }
                        ctx.charge_work(id, "model_dimension_join", dimensions.len())?;
                        if !dimensions
                            .iter()
                            .any(|entry| entry.structural_node == current_node)
                        {
                            ctx.charge_storage(1)?;
                            dimensions.push(SelectedTypeDimension {
                                structural_node: current_node,
                                terms,
                            });
                        }
                    }
                    let target_node = unit
                        .target
                        .as_ref()
                        .map(|target| {
                            find_model_node(identity, target, ScalarTypeForm::Unit, &mut ctx, id)
                        })
                        .transpose()?;
                    let ratio =
                    |raw: &str,
                     ctx: &mut ProjectionBudget<'_>|
                     -> Result<CheckedRational, SelectedTypeDeclarationsError> {
                        ctx.charge_work(id, "model_unit_ratio", raw.len())?;
                        let (numerator, denominator) = raw.split_once('/').unwrap_or((raw, "1"));
                        ctx.storage_text(numerator)?;
                        ctx.storage_text(denominator)?;
                        if numerator.is_empty()
                            || denominator.is_empty()
                            || numerator.parse::<num_bigint::BigInt>().is_err()
                            || denominator
                                .parse::<num_bigint::BigInt>()
                                .ok()
                                .is_none_or(|value| value <= 0.into())
                        {
                            return Err(ctx.error(
                                ExactTypeProjectionErrorCode::UnsupportedExactType,
                                id,
                                "model_unit_ratio",
                            ));
                        }
                        ctx.storage_text(numerator)?;
                        ctx.storage_text(denominator)?;
                        Ok(CheckedRational {
                            numerator: numerator.into(),
                            denominator: denominator.into(),
                        })
                    };
                    let SelectedExactIdentity::DeclaredUnit(unit_id) = &row.exact_identity else {
                        unreachable!("joined model unit")
                    };
                    ctx.charge_storage(1)?;
                    ctx.storage_node(id)?;
                    ctx.storage_text(&row.selected_document)?;
                    ctx.storage_text(&row.original_identity)?;
                    ctx.charge_storage(native_join_storage::unit(unit_id))?;
                    model_units.push(SelectedTypeModelUnit {
                        structural_node: id.clone(),
                        selected_document: row.selected_document.clone(),
                        original_identity: row.original_identity.clone(),
                        unit_id: unit_id.clone(),
                        dimension_node,
                        dimension_terms: unit_terms,
                        target_node,
                        scale: ratio(&unit.scale, &mut ctx)?,
                        offset: ratio(&unit.offset, &mut ctx)?,
                    });
                }
            }

            ctx.budget.finish(before);
            Ok(SelectedTypeDeclarations {
                records,
                tuples,
                objects,
                units,
                model_units,
                dimensions,
                work: ctx.work().saturating_add(prior_work),
            })
        })();
        budget.set_active_locus(previous_locus);
        if result.is_err() {
            budget.abort();
        }
        result
    }
}
