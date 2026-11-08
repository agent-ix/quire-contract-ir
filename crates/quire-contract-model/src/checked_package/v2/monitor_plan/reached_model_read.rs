// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Reached model and unit facts for the selected monitor graph walk.

use super::selected_closure_walk::RuntimeWalkError;
use super::CheckedMonitorPlanRef;
use crate::checked_package::v2::model_enum_witness::RuntimeEnumError;
use crate::checked_package::v2::model_members::{DeclarationForm, MemberType};
use crate::checked_package::v2::native_join_budget::{
    read_charge, read_node_storage, read_node_work, NativeReadDenied,
};
use crate::checked_package::v2::native_join_storage as storage;
use crate::checked_package::v2::{
    CheckedNodeId, CheckedNodeKind, CheckedNodeOwner, CheckedTerm, ExactTypeProjectionError,
    NativeJoinBudget, ScalarTypeForm, SelectedExactDeclarationRow, SelectedExactFieldRow,
    SelectedExactIdentities, SelectedModelEnumDeclarationRow, SelectedModelEnumError,
    SelectedModelEnumErrorCode, SelectedModelEnumMemberRow, SelectedModelEnumShapeRow,
    SelectedModelEnums, SelectedSourceUnitRow, SelectedTupleDeclarationRow,
    SelectedTypeDeclarations, SelectedTypeDeclarationsError,
};
use crate::checked_package::v2::{NativeJoinChargeKind, NativeJoinLocus, NativeJoinLocusSource};
use std::collections::BTreeSet;

pub(super) struct ReachedProducerRows<'a> {
    pub(super) declarations: &'a [SelectedExactDeclarationRow],
    pub(super) fields: &'a [SelectedExactFieldRow],
    pub(super) tuples: &'a [SelectedTupleDeclarationRow],
    pub(super) source_units: &'a [SelectedSourceUnitRow],
    pub(super) enum_declarations: &'a [SelectedModelEnumDeclarationRow],
    pub(super) enum_members: &'a [SelectedModelEnumMemberRow],
    pub(super) enum_shapes: &'a [SelectedModelEnumShapeRow],
}

#[derive(Debug)]
pub(super) struct ReachedModelRead {
    pub(super) types: SelectedTypeDeclarations,
    pub(super) enums: Vec<SelectedModelEnumShapeRow>,
    pub(super) identities: SelectedExactIdentities,
    pub(super) reached: BTreeSet<CheckedNodeId>,
    pub(super) work: u64,
}

#[derive(Debug)]
pub(super) enum ReachedModelError {
    Walk(RuntimeWalkError),
    Exact {
        prior_work: u64,
        error: ExactTypeProjectionError,
    },
    Types(SelectedTypeDeclarationsError),
    Enum {
        prior_work: u64,
        error: SelectedModelEnumError,
    },
    Native(NativeReadDenied),
    UnresolvedDependency {
        at: CheckedNodeId,
        work: u64,
    },
}

impl From<NativeReadDenied> for ReachedModelError {
    fn from(error: NativeReadDenied) -> Self {
        Self::Native(error)
    }
}
fn charge(
    work: &mut u64,
    budget: &mut NativeJoinBudget,
    at: &CheckedNodeId,
) -> Result<(), ReachedModelError> {
    read_node_work(budget, at)?;
    *work += 1;
    Ok(())
}
fn retain(budget: &mut NativeJoinBudget, amount: u128) -> Result<(), ReachedModelError> {
    read_charge(budget, NativeJoinChargeKind::Storage, amount)?;
    Ok(())
}
fn unresolved(budget: &mut NativeJoinBudget, at: &CheckedNodeId, work: u64) -> ReachedModelError {
    if let Err(error) = read_node_storage(budget, at) {
        return error.into();
    }
    ReachedModelError::UnresolvedDependency {
        at: at.clone(),
        work,
    }
}
fn enum_error(error: RuntimeEnumError, work: u64) -> ReachedModelError {
    match error {
        RuntimeEnumError::Semantic(error) => ReachedModelError::Enum {
            prior_work: work,
            error,
        },
        RuntimeEnumError::Native(error) => error.into(),
    }
}

fn checked_enum_error(
    budget: &mut NativeJoinBudget,
    code: SelectedModelEnumErrorCode,
    id: &CheckedNodeId,
    work: u64,
) -> ReachedModelError {
    if let Err(error) = read_node_storage(budget, id) {
        return error.into();
    }
    ReachedModelError::Enum {
        prior_work: work,
        error: SelectedModelEnumError {
            code,
            node: id.clone(),
            consumed_work: 0,
            attempted_work: 0,
        },
    }
}
fn text_work(budget: &mut NativeJoinBudget, value: &str) -> Result<(), ReachedModelError> {
    read_charge(budget, NativeJoinChargeKind::Work, value.len() as u128)?;
    Ok(())
}
fn preflight_declarations(
    row: &SelectedExactDeclarationRow,
    budget: &mut NativeJoinBudget,
) -> Result<(), ReachedModelError> {
    read_node_work(budget, &row.structural_node)?;
    text_work(budget, &row.selected_document)?;
    text_work(budget, &row.original_identity)?;
    match &row.exact_identity {
        crate::checked_package::v2::SelectedExactIdentity::Object(id) => {
            for original in id.owner_chain() {
                text_work(budget, original.package())?;
                text_work(budget, original.node())?;
            }
        }
        crate::checked_package::v2::SelectedExactIdentity::DeclaredUnit(id) => {
            storage::preflight_unit(id, budget, &row.structural_node).map_err(|error| {
                ReachedModelError::Exact {
                    prior_work: 0,
                    error,
                }
            })?;
        }
        _ => {}
    }
    Ok(())
}
fn preflight_fields(
    row: &SelectedExactFieldRow,
    budget: &mut NativeJoinBudget,
) -> Result<(), ReachedModelError> {
    read_node_work(budget, &row.record_node)?;
    text_work(budget, &row.selected_document)?;
    text_work(budget, &row.record_original_identity)?;
    text_work(budget, &row.field_original_identity)?;
    text_work(budget, row.member_id.member())?;
    Ok(())
}
fn preflight_tuples(
    row: &SelectedTupleDeclarationRow,
    budget: &mut NativeJoinBudget,
) -> Result<(), ReachedModelError> {
    read_node_work(budget, &row.structural_node)?;
    text_work(budget, &row.selected_document)?;
    text_work(budget, &row.original_identity)?;
    Ok(())
}
fn preflight_source_units(
    row: &SelectedSourceUnitRow,
    budget: &mut NativeJoinBudget,
) -> Result<(), ReachedModelError> {
    read_node_work(budget, &row.structural_node)?;
    text_work(budget, &row.source_authority)?;
    text_work(budget, &row.source_identity)?;
    for segment in &row.qualified_declaration {
        text_work(budget, segment)?;
    }
    Ok(())
}
fn preflight_enum_declarations(
    row: &SelectedModelEnumDeclarationRow,
    budget: &mut NativeJoinBudget,
) -> Result<(), ReachedModelError> {
    read_node_work(budget, &row.structural_node)?;
    text_work(budget, &row.selected_document)?;
    text_work(budget, &row.original_identity)?;
    Ok(())
}
fn preflight_enum_members(
    row: &SelectedModelEnumMemberRow,
    budget: &mut NativeJoinBudget,
) -> Result<(), ReachedModelError> {
    read_node_work(budget, &row.structural_node)?;
    text_work(budget, &row.selected_document)?;
    text_work(budget, &row.enum_original_identity)?;
    text_work(budget, &row.member_original_identity)?;
    text_work(budget, &row.case)?;
    read_node_work(budget, &row.declaration_node)?;
    Ok(())
}
fn preflight_enum_shapes(
    row: &SelectedModelEnumShapeRow,
    budget: &mut NativeJoinBudget,
) -> Result<(), ReachedModelError> {
    read_node_work(budget, &row.structural_node)?;
    text_work(budget, &row.selected_document)?;
    text_work(budget, &row.original_identity)?;
    for variant in row.shape.variants() {
        text_work(budget, variant.case())?;
    }
    for case in &row.cases {
        read_node_work(budget, &case.structural_node)?;
        text_work(budget, &case.case)?;
        text_work(budget, &case.member_original_identity)?;
        text_work(budget, case.variant.case())?;
    }
    Ok(())
}
impl CheckedMonitorPlanRef<'_> {
    fn follow_member_type(
        &self,
        member: &MemberType,
        pending: &mut Vec<(CheckedNodeId, NativeJoinLocusSource)>,
        work: &mut u64,
        budget: &mut NativeJoinBudget,
        at: &CheckedNodeId,
    ) -> Result<(), ReachedModelError> {
        retain(budget, 2)?;
        let mut types = vec![member];
        while let Some(member) = types.pop() {
            charge(work, budget, at)?;
            match member {
                MemberType::Reference(id)
                | MemberType::Record(id)
                | MemberType::Tuple(id)
                | MemberType::Variant(id)
                | MemberType::Quantity(id)
                | MemberType::CarriedRational(_, id) => {
                    read_node_storage(budget, id)?;
                    retain(budget, 1)?;
                    pending.push((
                        id.clone(),
                        budget
                            .active_locus()
                            .expect("reached parent context")
                            .source,
                    ));
                }
                MemberType::Option(inner) => {
                    retain(budget, 1)?;
                    types.push(inner);
                }
                MemberType::Collection { element, .. } => {
                    retain(budget, 1)?;
                    types.push(element);
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn follow_fields<'a>(
        &self,
        mut fields: impl Iterator<
            Item = crate::checked_package::v2::model_fields::SelectedRecordField<'a>,
        >,
        pending: &mut Vec<(CheckedNodeId, NativeJoinLocusSource)>,
        work: &mut u64,
        budget: &mut NativeJoinBudget,
        at: &CheckedNodeId,
    ) -> Result<(), ReachedModelError> {
        loop {
            budget.set_locus_form("reached_field");
            charge(work, budget, at)?;
            let Some(field) = fields.next() else {
                break;
            };
            if let Some(member) = field.member_type {
                self.follow_member_type(member, pending, work, budget, at)?;
            }
        }
        Ok(())
    }

    fn followed_model_nodes(
        &self,
        reached: &mut BTreeSet<CheckedNodeId>,
        work: &mut u64,
        budget: &mut NativeJoinBudget,
    ) -> Result<(), ReachedModelError> {
        retain(budget, 2)?;
        let mut pending = Vec::new();
        for (index, id) in reached.iter().enumerate() {
            let source = NativeJoinLocusSource::ReachedRow(index);
            budget.set_active_locus(Some(NativeJoinLocus {
                source,
                form: "reached_pending",
            }));
            read_node_storage(budget, id)?;
            retain(budget, 1)?;
            pending.push((id.clone(), source));
        }
        let mut visited = BTreeSet::new();
        while let Some((id, source)) = pending.pop() {
            budget.set_active_locus(Some(NativeJoinLocus {
                source,
                form: "reached_model_node",
            }));
            charge(work, budget, &id)?;
            let graph_position = self
                .package
                .node_with_position(&id)
                .map(|(position, _)| position)
                .ok_or_else(|| unresolved(budget, &id, *work))?;
            budget.set_locus_source(NativeJoinLocusSource::GraphNode(graph_position));
            read_node_storage(budget, &id)?;
            retain(budget, 1)?;
            if !visited.insert(id.clone()) {
                continue;
            }
            budget.set_locus_source(NativeJoinLocusSource::GraphNode(graph_position));
            charge(work, budget, &id)?;
            self.package
                .node(&id)
                .ok_or_else(|| unresolved(budget, &id, *work))?;
            read_node_storage(budget, &id)?;
            retain(budget, 1)?;
            reached.insert(id.clone());
            if let Some(CheckedTerm::NominalBody(body)) = self.package.term(&id) {
                if let Some((dimension, target, _, _)) = body.unit_details() {
                    budget.set_locus_source(NativeJoinLocusSource::GraphNode(graph_position));
                    charge(work, budget, &id)?;
                    read_node_storage(budget, dimension)?;
                    retain(budget, 1)?;
                    pending.push((
                        dimension.clone(),
                        budget
                            .active_locus()
                            .expect("reached parent context")
                            .source,
                    ));
                    if let Some(target) = target {
                        budget.set_locus_source(NativeJoinLocusSource::GraphNode(graph_position));
                        charge(work, budget, &id)?;
                        read_node_storage(budget, target)?;
                        retain(budget, 1)?;
                        pending.push((
                            target.clone(),
                            budget
                                .active_locus()
                                .expect("reached parent context")
                                .source,
                        ));
                    }
                }
                if let Some(terms) = body.dimension_details() {
                    for term in terms {
                        budget.set_locus_source(NativeJoinLocusSource::GraphNode(graph_position));
                        charge(work, budget, &id)?;
                        read_node_storage(budget, &term.dimension_node_id)?;
                        retain(budget, 1)?;
                        pending.push((
                            term.dimension_node_id.clone(),
                            budget
                                .active_locus()
                                .expect("reached parent context")
                                .source,
                        ));
                    }
                }
            }
            let Some(declaration) = self.package.models.selected_declaration_entry(&id) else {
                continue;
            };
            let model = self
                .package
                .models
                .selected_models()
                .get(declaration.selection)
                .ok_or_else(|| unresolved(budget, &id, *work))?;
            if declaration.form == DeclarationForm::TupleValueType {
                let positions = model
                    .tuple_types
                    .get(&declaration.node)
                    .ok_or_else(|| unresolved(budget, &id, *work))?;
                for position in positions {
                    budget.set_locus_source(NativeJoinLocusSource::GraphNode(graph_position));
                    charge(work, budget, &id)?;
                    let member = model
                        .element_type_runtime(position, budget)
                        .map_err(|_| ReachedModelError::Native(NativeReadDenied))?
                        .ok_or_else(|| unresolved(budget, &id, *work))?;
                    self.follow_member_type(&member, &mut pending, work, budget, &id)?;
                }
            }
            if declaration.form == DeclarationForm::VariantType {
                let cases = model
                    .variant_types
                    .get(&declaration.node)
                    .ok_or_else(|| unresolved(budget, &id, *work))?;
                for case in cases.values() {
                    budget.set_locus_source(NativeJoinLocusSource::GraphNode(graph_position));
                    charge(work, budget, &id)?;
                    if let Some(payload) = &case.payload {
                        let member = model
                            .element_type_runtime(payload, budget)
                            .map_err(|_| ReachedModelError::Native(NativeReadDenied))?
                            .ok_or_else(|| unresolved(budget, &id, *work))?;
                        self.follow_member_type(&member, &mut pending, work, budget, &id)?;
                    }
                }
            }
            let bearing = match declaration.form {
                DeclarationForm::RecordValueType => model.record_types.get(&declaration.node),
                DeclarationForm::ObjectType => model.object_types.get(&declaration.node),
                _ => None,
            };
            if let Some(bearing) = bearing {
                for original in &bearing.supertypes {
                    budget.set_locus_source(NativeJoinLocusSource::GraphNode(graph_position));
                    charge(work, budget, &id)?;
                    text_work(budget, original)?;
                    retain(budget, storage::text_parts(&[original]))?;
                    let target = self
                        .package
                        .models
                        .selected_declaration_by_original(declaration.selection, original)
                        .ok_or_else(|| unresolved(budget, &id, *work))?;
                    read_node_storage(budget, target)?;
                    retain(budget, 1)?;
                    pending.push((
                        target.clone(),
                        budget
                            .active_locus()
                            .expect("reached parent context")
                            .source,
                    ));
                }
                budget.set_locus_source(NativeJoinLocusSource::GraphNode(graph_position));
                if declaration.form == DeclarationForm::RecordValueType {
                    let (_, fields) = self
                        .package
                        .models
                        .selected_record_fields(&id)
                        .ok_or_else(|| unresolved(budget, &id, *work))?;
                    self.follow_fields(fields, &mut pending, work, budget, &id)?;
                } else {
                    let (_, fields) = self
                        .package
                        .models
                        .selected_object_fields(&id)
                        .ok_or_else(|| unresolved(budget, &id, *work))?;
                    self.follow_fields(fields, &mut pending, work, budget, &id)?;
                }
                if declaration.form == DeclarationForm::ObjectType {
                    for operation in &bearing.operations {
                        budget.set_locus_source(NativeJoinLocusSource::GraphNode(graph_position));
                        charge(work, budget, &id)?;
                        for parameter in &operation.parameters {
                            if let Some(member) = model
                                .slot_type_runtime(parameter, budget)
                                .map_err(|_| ReachedModelError::Native(NativeReadDenied))?
                            {
                                self.follow_member_type(&member, &mut pending, work, budget, &id)?;
                            }
                        }
                        if let Some(result) = &operation.result {
                            if let Some(member) = model
                                .slot_type_runtime(result, budget)
                                .map_err(|_| ReachedModelError::Native(NativeReadDenied))?
                            {
                                self.follow_member_type(&member, &mut pending, work, budget, &id)?;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// The result is a prerequisite fact read, not a complete monitor type closure.
    pub(super) fn read_reached_model_facts(
        &self,
        rows: ReachedProducerRows<'_>,
        budget: &mut NativeJoinBudget,
    ) -> Result<ReachedModelRead, ReachedModelError> {
        let before = budget.begin();
        let previous_locus = budget.active_locus();
        let previous_mode = budget.exact_diagnostics();
        budget.set_exact_diagnostics(true);
        budget.set_active_locus(Some(NativeJoinLocus {
            source: NativeJoinLocusSource::MonitorSeed(0),
            form: "reached_model",
        }));
        let result = (|| {
            let graph = self
                .selected_graph_walk(budget)
                .map_err(ReachedModelError::Walk)?;
            retain(budget, 1)?;
            let mut reached = BTreeSet::new();
            for (index, id) in graph.nodes.into_iter().enumerate() {
                budget.set_active_locus(Some(NativeJoinLocus {
                    source: NativeJoinLocusSource::ReachedRow(index),
                    form: "reached_graph_set",
                }));
                charge(&mut 0, budget, &id)?;
                retain(budget, 1)?;
                reached.insert(id);
            }
            let mut work = graph.work;
            self.followed_model_nodes(&mut reached, &mut work, budget)?;
            retain(budget, 4)?;
            let mut model_declarations = BTreeSet::new();
            let mut model_records = BTreeSet::new();
            let mut model_enums = BTreeSet::new();
            let mut source_unit_declarations = BTreeSet::new();
            for (reached_index, id) in reached.iter().enumerate() {
                budget.set_active_locus(Some(NativeJoinLocus {
                    source: NativeJoinLocusSource::ReachedRow(reached_index),
                    form: "reached_selection",
                }));
                charge(&mut work, budget, id)?;
                if let Some((position, _)) = self.package.node_with_position(id) {
                    budget.set_locus_source(NativeJoinLocusSource::GraphNode(position));
                }
                if let Some((document, original, form)) =
                    self.package.models.selected_declaration(id)
                {
                    let identity = (document, original);
                    retain(budget, 1)?;
                    model_declarations.insert(identity);
                    if form == DeclarationForm::RecordValueType {
                        retain(budget, 1)?;
                        model_records.insert(identity);
                    }
                    if form == DeclarationForm::VariantType {
                        retain(budget, 1)?;
                        model_enums.insert(identity);
                    }
                }
                let Some(node) = self.package.node(id) else {
                    continue;
                };
                if self.package.node_kind(id)
                    == Some(CheckedNodeKind::ScalarType(ScalarTypeForm::Unit))
                {
                    match node.owner.as_ref() {
                        Some(CheckedNodeOwner::Model { identity, node }) => {
                            retain(budget, 1)?;
                            model_declarations.insert((identity.as_ref(), node.as_ref()));
                        }
                        Some(CheckedNodeOwner::Source {
                            authority,
                            identity,
                        }) => {
                            if let Some(declaration) = &node.declaration {
                                retain(budget, 1)?;
                                source_unit_declarations.insert((
                                    authority.as_ref(),
                                    identity.as_ref(),
                                    declaration.qualified_name.as_slice(),
                                ));
                            }
                        }
                        _ => {}
                    }
                }
            }
            retain(budget, 7)?;
            let mut declarations = Vec::new();
            let mut fields = Vec::new();
            let mut tuples = Vec::new();
            let mut source_units = Vec::new();
            let mut enum_declarations = Vec::new();
            let mut enum_members = Vec::new();
            let mut enum_shapes = Vec::new();
            for (index, row) in rows.declarations.iter().enumerate() {
                budget.set_active_locus(Some(NativeJoinLocus {
                    source: NativeJoinLocusSource::ReachedRow(index),
                    form: "reached_declarations",
                }));
                charge(&mut work, budget, &row.structural_node)?;
                preflight_declarations(row, budget)?;
                if reached.contains(&row.structural_node)
                    || model_declarations.contains(&(
                        row.selected_document.as_ref(),
                        row.original_identity.as_ref(),
                    ))
                {
                    retain(budget, storage::stage_declaration(row).saturating_add(1))?;
                    declarations.push(row.clone());
                }
            }
            for (index, row) in rows.fields.iter().enumerate() {
                budget.set_active_locus(Some(NativeJoinLocus {
                    source: NativeJoinLocusSource::ReachedRow(index),
                    form: "reached_fields",
                }));
                charge(&mut work, budget, &row.record_node)?;
                preflight_fields(row, budget)?;
                if reached.contains(&row.record_node)
                    || model_records.contains(&(
                        row.selected_document.as_ref(),
                        row.record_original_identity.as_ref(),
                    ))
                {
                    retain(budget, storage::stage_field(row).saturating_add(1))?;
                    fields.push(row.clone());
                }
            }
            for (index, row) in rows.tuples.iter().enumerate() {
                budget.set_active_locus(Some(NativeJoinLocus {
                    source: NativeJoinLocusSource::ReachedRow(index),
                    form: "reached_tuples",
                }));
                charge(&mut work, budget, &row.structural_node)?;
                preflight_tuples(row, budget)?;
                if reached.contains(&row.structural_node) {
                    retain(budget, storage::runtime_tuple(row).saturating_add(1))?;
                    tuples.push(row.clone());
                }
            }
            for (index, row) in rows.source_units.iter().enumerate() {
                budget.set_active_locus(Some(NativeJoinLocus {
                    source: NativeJoinLocusSource::ReachedRow(index),
                    form: "reached_source_units",
                }));
                charge(&mut work, budget, &row.structural_node)?;
                preflight_source_units(row, budget)?;
                if reached.contains(&row.structural_node)
                    || source_unit_declarations.contains(&(
                        row.source_authority.as_ref(),
                        row.source_identity.as_ref(),
                        row.qualified_declaration.as_slice(),
                    ))
                {
                    retain(budget, storage::runtime_source_unit(row).saturating_add(1))?;
                    source_units.push(row.clone());
                }
            }
            for (index, row) in rows.enum_declarations.iter().enumerate() {
                budget.set_active_locus(Some(NativeJoinLocus {
                    source: NativeJoinLocusSource::ReachedRow(index),
                    form: "reached_enum_declarations",
                }));
                charge(&mut work, budget, &row.structural_node)?;
                preflight_enum_declarations(row, budget)?;
                if reached.contains(&row.structural_node)
                    || model_enums.contains(&(
                        row.selected_document.as_ref(),
                        row.original_identity.as_ref(),
                    ))
                {
                    retain(
                        budget,
                        storage::model_enum_declaration(row).saturating_add(1),
                    )?;
                    enum_declarations.push(row.clone());
                }
            }
            for (index, row) in rows.enum_members.iter().enumerate() {
                budget.set_active_locus(Some(NativeJoinLocus {
                    source: NativeJoinLocusSource::ReachedRow(index),
                    form: "reached_enum_members",
                }));
                charge(&mut work, budget, &row.declaration_node)?;
                preflight_enum_members(row, budget)?;
                if reached.contains(&row.declaration_node)
                    || model_enums.contains(&(
                        row.selected_document.as_ref(),
                        row.enum_original_identity.as_ref(),
                    ))
                {
                    retain(budget, storage::model_enum_member(row).saturating_add(1))?;
                    enum_members.push(row.clone());
                }
            }
            for (index, row) in rows.enum_shapes.iter().enumerate() {
                budget.set_active_locus(Some(NativeJoinLocus {
                    source: NativeJoinLocusSource::ReachedRow(index),
                    form: "reached_enum_shapes",
                }));
                charge(&mut work, budget, &row.structural_node)?;
                preflight_enum_shapes(row, budget)?;
                if reached.contains(&row.structural_node)
                    || model_enums.contains(&(
                        row.selected_document.as_ref(),
                        row.original_identity.as_ref(),
                    ))
                {
                    retain(budget, storage::model_enum_shape(row).saturating_add(1))?;
                    enum_shapes.push(row.clone());
                }
            }
            let identities = SelectedExactIdentities::from_stage_rows(
                self.package,
                declarations,
                fields,
                budget,
            )
            .map_err(|error| ReachedModelError::Exact {
                prior_work: work,
                error,
            })?;
            work = budget.consumed().work - before.work;
            let enums = SelectedModelEnums::from_stage_rows_runtime(
                self.package,
                enum_declarations,
                enum_members,
                budget,
            )
            .map_err(|error| enum_error(error, work))?;
            work = budget.consumed().work - before.work;
            retain(budget, 1)?;
            let mut selected_enums = Vec::new();
            for (index, id) in reached.iter().enumerate() {
                budget.set_active_locus(Some(NativeJoinLocus {
                    source: NativeJoinLocusSource::ReachedRow(index),
                    form: "reached_enum",
                }));
                charge(&mut work, budget, id)?;
                if let Some((position, _)) = self.package.node_with_position(id) {
                    budget.set_locus_source(NativeJoinLocusSource::GraphNode(position));
                }
                if self.package.node_kind(id)
                    != Some(crate::checked_package::v2::CheckedNodeKind::Model(
                        crate::checked_package::v2::ModelForm::VariantType,
                    ))
                {
                    continue;
                }
                let mut selected_shape = None;
                for row in &enum_shapes {
                    charge(&mut work, budget, &row.structural_node)?;
                    if &row.structural_node != id {
                        continue;
                    }
                    if selected_shape.replace(row).is_some() {
                        return Err(checked_enum_error(
                            budget,
                            SelectedModelEnumErrorCode::ConflictingWitness,
                            id,
                            work,
                        ));
                    }
                }
                let Some(row) = selected_shape else {
                    return Err(checked_enum_error(
                        budget,
                        SelectedModelEnumErrorCode::UnavailableWitness,
                        id,
                        work,
                    ));
                };
                enums
                    .read_shape_runtime(self.package, row, budget)
                    .map_err(|error| enum_error(error, work))?;
                charge(&mut work, budget, id)?;
                preflight_enum_shapes(row, budget)?;
                retain(budget, storage::model_enum_shape(row).saturating_add(1))?;
                selected_enums.push(row.clone());
            }
            let identities = identities
                .with_model_enum_shapes(&selected_enums, budget)
                .map_err(|error| ReachedModelError::Exact {
                    prior_work: work,
                    error,
                })?;
            let prior_projection_work = budget.consumed().work - before.work;
            let types = self
                .package
                .project_type_declarations_for_nodes(
                    &identities,
                    &tuples,
                    &source_units,
                    budget,
                    Some(&reached),
                    prior_projection_work,
                )
                .map_err(ReachedModelError::Types)?;
            work = budget.consumed().work - before.work;
            Ok(ReachedModelRead {
                types,
                enums: selected_enums,
                identities,
                reached,
                work,
            })
        })();
        budget.set_active_locus(previous_locus);
        budget.set_exact_diagnostics(previous_mode);
        if result.is_ok() {
            budget.finish(before);
        } else {
            budget.abort();
        }
        result
    }
}

#[cfg(test)]
#[allow(unused_imports)]
#[path = "../../../../../../tests/it/support/checked_package.rs"]
pub(in crate::checked_package::v2) mod fixture_support;

#[cfg(test)]
fn fixture_read(
    bytes: &[u8],
    limits: crate::CheckedPackageReadLimits,
    evidence: &crate::CheckedPackageEvidence,
) -> crate::CheckedPackageV2ReadResult {
    crate::CheckedPackageV2::read(bytes, limits, evidence)
}

#[cfg(test)]
fn fixtures_dir() -> std::path::PathBuf {
    std::path::PathBuf::new()
}
#[cfg(test)]
fn proposal_dir_of(path: &std::path::Path) -> std::path::PathBuf {
    path.to_path_buf()
}
#[cfg(test)]
fn read_json(_path: &std::path::Path) -> Result<serde_json::Value, String> {
    Err("unused".into())
}

#[cfg(test)]
#[path = "../../../../../../tests/it/support/monitor_plan.rs"]
mod monitor_plan_fixtures;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checked_package::v2::{
        CheckedPackageReadLimits, CheckedPackageV2, CheckedPackageV2ReadResult,
    };
    use crate::checked_package::v2::{
        ExactTypeProjectionErrorCode, SelectedExactDeclarationForm, SelectedExactIdentity,
    };
    use ix_trace_rs::trace;
    use quire_exact::{
        CollectionKind, CollectionType, EffectiveId, Location, MemberId, MemberOwner, NodeKey,
        Origin, OriginalDeclarationKey, PathSegment, Presence, Role, UnionMember, UnitId,
        ValueType,
    };
    use quire_semantic_value::declaration::{EqualityOperator, EqualitySchedule};

    fn admitted_monitor() -> CheckedPackageV2 {
        admitted_monitor_with_variant(false)
    }

    fn admitted_monitor_for_closure() -> CheckedPackageV2 {
        admitted_monitor_for_closure_with_status("integer")
    }

    fn admitted_monitor_for_closure_with_status(status: &str) -> CheckedPackageV2 {
        admitted_monitor_for_closure_with(
            status,
            serde_json::json!([{"role":"expression","ordinal":0}]),
            false,
            false,
        )
    }

    fn admitted_monitor_for_closure_with_maps(status: &str, maps: bool) -> CheckedPackageV2 {
        admitted_monitor_for_closure_with(
            status,
            serde_json::json!([{"role":"expression","ordinal":0}]),
            false,
            maps,
        )
    }

    fn admitted_monitor_for_closure_with(
        status: &str,
        equality_occurrences: serde_json::Value,
        bounded_sequence: bool,
        maps: bool,
    ) -> CheckedPackageV2 {
        let (mut wire, document) =
            monitor_plan_fixtures::model_fact_monitor_with_status(false, status);
        let boolean = monitor_plan_fixtures::find(&wire, "boolean");
        let equality = monitor_plan_fixtures::append(
            &mut wire,
            "expression",
            "binary",
            Some(&boolean),
            serde_json::json!({"term":"application","operator":"binary",
                "operation":{"identity":"quire.op.boolean.eq","laws":[],"leaves":[],
                    "member":null,"mode":null},"result_type":boolean,
                "arguments":[
                    {"term":"literal","type":boolean,"value_kind":"boolean","value":true},
                    {"term":"literal","type":boolean,"value_kind":"boolean","value":true}]}),
            "monitor_equality",
        );
        let equality_position = monitor_plan_fixtures::position(&wire, &equality);
        wire["semantic_graph"]["nodes"][equality_position]["occurrences"] = equality_occurrences;
        let conditional_position = wire["semantic_graph"]["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .position(|node| {
                node["semantic_form"] == "conditional" && node["node_id"]["path"][2] == "on_gap"
            })
            .expect("on_gap conditional");
        wire["semantic_graph"]["nodes"][conditional_position]["body"]["arguments"][0] =
            monitor_plan_fixtures::reference(&equality);
        wire["semantic_graph"]["nodes"][conditional_position]["dependencies"]
            .as_array_mut()
            .unwrap()
            .push(equality.clone());
        monitor_plan_fixtures::rekey(&mut wire, conditional_position);
        if wire["checked_term_data"].is_null() {
            wire["checked_term_data"] = serde_json::json!({
                "equality_decisions":[],"protocol_locations":[],
                "dispatch_tables":[],"probabilistic_claims":[]});
        }
        let decided_occurrence = wire["semantic_graph"]["nodes"][equality_position]["occurrences"]
            .as_array()
            .unwrap()
            .iter()
            .find(|occurrence| occurrence["role"] == "expression")
            .unwrap_or(&wire["semantic_graph"]["nodes"][equality_position]["occurrences"][0])
            .clone();
        wire["checked_term_data"]["equality_decisions"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({"application":equality,
                "occurrence":decided_occurrence,"term_path":[],
                "operator":"equal","schedule":"plan",
                "left":{"operand":0,"source_type":boolean,"conversion_target":null},
                "right":{"operand":1,"source_type":boolean,"conversion_target":null},
                "units":[],"enum_members":[]}));
        if bounded_sequence {
            let integer = monitor_plan_fixtures::find(&wire, "integer");
            let plan_position = wire["semantic_graph"]["nodes"]
                .as_array()
                .unwrap()
                .iter()
                .position(|node| node["semantic_form"] == "monitor_plan")
                .expect("monitor plan");
            let scope = wire["semantic_graph"]["nodes"][plan_position]["body"]["members"]
                .as_array()
                .unwrap()
                .iter()
                .find(|member| member["name"] == "scope")
                .unwrap()["value"]["target"]
                .clone();
            let state_position = monitor_plan_fixtures::position(&wire, &scope);
            let base = monitor_plan_fixtures::type_node(
                &mut wire,
                "sequence",
                monitor_plan_fixtures::aggregate(vec![monitor_plan_fixtures::reference(&integer)]),
                "bounded_base",
            );
            let integer_at = monitor_plan_fixtures::position(&wire, &integer);
            let base_at = monitor_plan_fixtures::position(&wire, &base);
            let bound = |name: &str, value: &str| {
                serde_json::json!({"term":"binding","name":name,"value":{
                    "term":"literal","type":integer,"value_kind":"integer","value":value}})
            };
            let wrapper = serde_json::json!({"package":wire["name"],"path":[
                "bounded_domain","collection_bounds","min",{"node":integer_at},"0",
                "max",{"node":integer_at},"3",{"node":base_at}],"ordinal":0});
            wire["semantic_graph"]["nodes"]
                .as_array_mut()
                .unwrap()
                .push(serde_json::json!({
                "schema_version":"quire.checked-semantic-graph/v2","node_id":wrapper,
                "node_tag":"bounded_domain","semantic_form":"collection_bounds",
                "semantic_type":base,"dependencies":[base],
                "occurrences":[{"role":"type","ordinal":0}],
                "body":monitor_plan_fixtures::aggregate(vec![bound("min","0"),bound("max","3")])}));
            let state = &mut wire["semantic_graph"]["nodes"][state_position];
            state["body"]["members"]
                .as_array_mut()
                .unwrap()
                .push(monitor_plan_fixtures::binding(
                    "bounded",
                    monitor_plan_fixtures::reference(&wrapper),
                ));
            state["dependencies"].as_array_mut().unwrap().push(wrapper);
            monitor_plan_fixtures::rekey(&mut wire, state_position);
        }
        let source = wire["lock"]["sources"][0].clone();
        for node in wire["semantic_graph"]["nodes"].as_array_mut().unwrap() {
            if node["node_tag"] != "composite_type"
                || !matches!(
                    node["semantic_form"].as_str(),
                    Some("record" | "tuple" | "union")
                )
            {
                continue;
            }
            let Some(name) = node["node_id"]["path"][2].as_str().map(str::to_owned) else {
                continue;
            };
            node["owner"] = fixture_support::source_owner(&source);
            node["declaration"] = serde_json::json!({"qualified_name":["Example",name]});
            node["occurrences"] = serde_json::json!([{"role":"declaration","ordinal":0}]);
        }
        if maps {
            let text = monitor_plan_fixtures::find(&wire, "text");
            let target = |node: &serde_json::Value, name: &str| {
                node["body"]["members"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|binding| binding["name"] == name)
                    .unwrap_or_else(|| panic!("existing {name} binding"))["value"]["target"]
                    .clone()
            };
            let graph = wire["semantic_graph"]["nodes"].as_array().unwrap();
            let root = graph
                .iter()
                .find(|node| node["semantic_form"] == "monitor_plan")
                .unwrap();
            let inputs_id = target(root, "inputs");
            let state_id = target(root, "state");
            let state = graph
                .iter()
                .find(|node| node["node_id"] == state_id)
                .unwrap();
            let activations_id = target(state, "activations");
            let activations = graph
                .iter()
                .find(|node| node["node_id"] == activations_id)
                .unwrap();
            let activation = activations["body"]["members"][0]["target"].clone();
            assert!(
                graph.iter().any(|node| node["node_id"] == activation
                    && node["semantic_form"] == "record"
                    && node["declaration"].is_object()),
                "the state sequence names an existing declared activation record"
            );
            let boolean = monitor_plan_fixtures::find(&wire, "boolean");
            let append_map = |wire: &mut serde_json::Value, value: &serde_json::Value| {
                let key_position = monitor_plan_fixtures::position(wire, &text);
                let value_position = monitor_plan_fixtures::position(wire, value);
                let id = serde_json::json!({"package":wire["name"],
                    "path":["composite_type","map",{"node":key_position},{"node":value_position}],
                    "ordinal":0});
                wire["semantic_graph"]["nodes"].as_array_mut().unwrap().push(serde_json::json!({
                    "schema_version":"quire.checked-semantic-graph/v2", "node_id":id,
                    "node_tag":"composite_type", "semantic_form":"map", "semantic_type":id,
                    "dependencies":[text,value], "occurrences":[{"role":"type","ordinal":0}],
                    "body":{"term":"aggregate","members":[
                        {"term":"binding","name":"key","value":{"term":"reference","target":text}},
                        {"term":"binding","name":"value","value":{"term":"reference","target":value}}
                    ]}}));
                id
            };
            let map = append_map(&mut wire, &activation);
            let _unrelated = append_map(&mut wire, &boolean);
            let inputs = wire["semantic_graph"]["nodes"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|node| node["node_id"] == inputs_id)
                .expect("monitor's actual Inputs record");
            inputs["body"]["members"]
                .as_array_mut()
                .unwrap()
                .push(monitor_plan_fixtures::binding(
                    "by_key",
                    monitor_plan_fixtures::reference(&map),
                ));
            inputs["dependencies"].as_array_mut().unwrap().push(map);
        }
        fixture_support::rebuild_complete_source_map(&mut wire);
        fixture_support::refresh_identity(&mut wire);
        for node in wire["semantic_graph"]["nodes"].as_array_mut().unwrap() {
            node["dependencies"]
                .as_array_mut()
                .unwrap()
                .sort_by(|a, b| {
                    let a: CheckedNodeId = serde_json::from_value(a.clone()).unwrap();
                    let b: CheckedNodeId = serde_json::from_value(b.clone()).unwrap();
                    a.cmp(&b)
                });
        }
        admit_wire(&wire, &document)
    }

    fn equality_rows(package: &CheckedPackageV2) -> Vec<crate::SelectedEqualityApplicationRow> {
        let application = package.resolved_graph().nodes.iter().find(|node| {
            matches!(node.node_id.path.get(2),
                Some(crate::CheckedNodePathSegment::Name(name)) if name.as_ref() == "monitor_equality")
        }).expect("reached monitor equality").node_id.clone();
        let source_map_entry = package
            .source_map()
            .iter()
            .find(|row| {
                row.node_id == application && row.role == crate::CheckedOccurrenceRole::Expression
            })
            .or_else(|| {
                package
                    .source_map()
                    .iter()
                    .find(|row| row.node_id == application)
            })
            .expect("equality source occurrence")
            .clone();
        let occurrence = crate::CheckedOccurrence {
            role: source_map_entry.role.clone(),
            ordinal: source_map_entry.ordinal,
        };
        let key = NodeKey::new(
            package.name(),
            vec![PathSegment::Text("monitor-equality".into())],
            0,
        )
        .unwrap();
        vec![crate::SelectedEqualityApplicationRow::new(
            application,
            occurrence,
            source_map_entry,
            key.clone(),
            Location::new(key, Origin::new(Role::from("expression"), 0)),
        )]
    }

    /// Project `package` with the fixture's source rows plus `wrappers`, and
    /// return the reached equality occurrences and the projected Exact type nodes.
    fn project_closure(
        package: &CheckedPackageV2,
        wrappers: Vec<crate::SelectedSourceCollectionRow>,
    ) -> Result<
        (Vec<crate::CheckedOccurrence>, Vec<ValueType>),
        super::super::selected_type_closure::SelectedMonitorTypeClosureError,
    > {
        let equality_apps = equality_rows(package);
        let (declarations, fields, source_units) = producer_rows(package);
        let (source_records, source_tuples, source_unions, mut source_collections) =
            source_rows(package);
        source_collections.retain(|row| {
            !wrappers
                .iter()
                .any(|wrapper| wrapper.base_node == row.structural_node)
        });
        source_collections.extend(wrappers);
        let rows = super::super::selected_type_closure::SelectedMonitorClosureRows {
            declarations: &declarations,
            fields: &fields,
            model_tuples: &[],
            source_units: &source_units,
            model_enum_declarations: &[],
            model_enum_members: &[],
            model_enum_shapes: &[],
            source_records: &source_records,
            source_tuples: &source_tuples,
            source_unions: &source_unions,
            source_enums: &[],
            source_collections: &source_collections,
            equality_applications: &equality_apps,
            equality_facts: &[],
        };
        let plan = package.monitor_plans().next().unwrap();
        let closure = plan
            .project_selected_type_closure(rows, &mut NativeJoinBudget::new(u64::MAX, u64::MAX))?;
        Ok((
            closure
                .equalities()
                .iter()
                .map(|equality| equality.id.occurrence.clone())
                .collect(),
            closure
                .exact_types()
                .iter()
                .map(|projection| projection.value_type().clone())
                .collect(),
        ))
    }

    #[trace("TC-470", "FR-357-AC-21")]
    #[test]
    fn a_claim_registration_of_a_reached_equality_needs_no_decision() {
        let both = admitted_monitor_for_closure_with(
            "integer",
            serde_json::json!([{"role":"claim","ordinal":0},{"role":"expression","ordinal":0}]),
            false,
            false,
        );
        let (occurrences, _) = project_closure(&both, Vec::new()).expect("expression site decided");
        assert_eq!(
            occurrences,
            vec![crate::CheckedOccurrence {
                role: crate::CheckedOccurrenceRole::Expression,
                ordinal: 0,
            }]
        );
        let claim_only = admitted_monitor_for_closure_with(
            "integer",
            serde_json::json!([{"role":"claim","ordinal":0}]),
            false,
            false,
        );
        let error = project_closure(&claim_only, Vec::new()).expect_err("no evaluation site");
        assert_eq!(
            error.code,
            crate::SelectedMonitorTypeClosureErrorCode::Equality(
                crate::AdmittedEqualityReadErrorCode::UnavailableEqualityDecision
            )
        );
    }

    #[trace("TC-470", "FR-357-AC-22")]
    #[test]
    fn a_bounded_collection_row_projects_its_base_with_the_wrapper() {
        let package = admitted_monitor_for_closure_with(
            "integer",
            serde_json::json!([{"role":"expression","ordinal":0}]),
            true,
            false,
        );
        let wrapper = package
            .resolved_graph()
            .nodes
            .iter()
            .find(|node| node.semantic_form.as_ref() == "collection_bounds")
            .expect("bounded wrapper");
        let base = wrapper.semantic_type.clone();
        let element = package
            .resolved_graph()
            .nodes
            .iter()
            .find(|node| node.semantic_form.as_ref() == "integer")
            .expect("integer")
            .node_id
            .clone();
        let row = crate::SelectedSourceCollectionRow {
            structural_node: wrapper.node_id.clone(),
            base_node: base.clone(),
            element_node: element,
            key_node: None,
            value_type: ValueType::collection(CollectionType::new(
                CollectionKind::Sequence,
                ValueType::Integer,
                Some(quire_exact::CardinalityBound::new(0, 3).unwrap()),
            )),
        };
        let bounded = row.value_type.clone();
        let (_, projected) = project_closure(&package, vec![row]).expect("wrapper row alone");
        assert!(projected.contains(&bounded));
    }

    #[trace("TC-470", "FR-357-AC-1")]
    #[test]
    fn source_owned_monitor_closure_fixture_admits() {
        let package = admitted_monitor_for_closure();
        assert_eq!(package.monitor_plans().len(), 1);
    }

    fn admitted_monitor_with_variant(reach_variant: bool) -> CheckedPackageV2 {
        admitted_monitor_with_selected_model(reach_variant, false)
    }

    fn admitted_monitor_with_selected_model(
        reach_variant: bool,
        unrelated_model: bool,
    ) -> CheckedPackageV2 {
        let (mut wire, document) =
            monitor_plan_fixtures::model_fact_monitor_with_variant(reach_variant);
        if unrelated_model {
            fixture_support::select_model(&mut wire, "aaa/other");
            fixture_support::refresh_identity(&mut wire);
        }
        admit_wire(&wire, &document)
    }

    fn admit_wire(wire: &serde_json::Value, document: &serde_json::Value) -> CheckedPackageV2 {
        let evidence = fixture_support::evidence_with_document(wire, "test/families", document);
        let result = CheckedPackageV2::read(
            &fixture_support::canonical(wire),
            CheckedPackageReadLimits::bounded(),
            &evidence,
        );
        let CheckedPackageV2ReadResult::Admitted(package) = result else {
            panic!("self-contained monitor admits: {result:?}")
        };
        *package
    }

    fn with_unemitted_record(document: &mut serde_json::Value) {
        let mut record = document["types"]
            .as_array()
            .unwrap()
            .iter()
            .find(|ty| ty["identity"] == "ix://test/families/Receipt")
            .unwrap()
            .clone();
        record["identity"] = serde_json::json!("ix://test/families/Unemitted");
        record["displayName"] = serde_json::json!("Unemitted");
        record["fields"] = serde_json::json!([]);
        document["types"].as_array_mut().unwrap().push(record);
    }

    fn admitted_document_change(change: impl FnOnce(&mut serde_json::Value)) -> CheckedPackageV2 {
        let (wire, mut document) = monitor_plan_fixtures::model_fact_monitor_with_variant(false);
        change(&mut document);
        admit_wire(&wire, &document)
    }

    fn read<'a>(
        package: &'a CheckedPackageV2,
        declarations: &'a [SelectedExactDeclarationRow],
        fields: &'a [SelectedExactFieldRow],
        units: &'a [SelectedSourceUnitRow],
        limit: u64,
    ) -> Result<ReachedModelRead, ReachedModelError> {
        package
            .monitor_plans()
            .next()
            .unwrap()
            .read_reached_model_facts(
                ReachedProducerRows {
                    declarations,
                    fields,
                    tuples: &[],
                    source_units: units,
                    enum_declarations: &[],
                    enum_members: &[],
                    enum_shapes: &[],
                },
                &mut NativeJoinBudget::new(limit, u64::MAX),
            )
    }

    fn key(name: &str) -> NodeKey {
        NodeKey::new("producer-stage", vec![PathSegment::Text(name.into())], 0).unwrap()
    }

    fn selected_node(package: &CheckedPackageV2, original: &str) -> CheckedNodeId {
        package
            .resolved_graph()
            .nodes
            .iter()
            .find(|node| {
                matches!(&node.owner,
            Some(CheckedNodeOwner::Model {node: selected, ..}) if selected.as_ref() == original)
            })
            .expect("selected model node")
            .node_id
            .clone()
    }

    fn source_unit(package: &CheckedPackageV2, name: &str) -> SelectedSourceUnitRow {
        let node = package
            .resolved_graph()
            .nodes
            .iter()
            .find(|node| {
                node.semantic_form.as_ref() == "unit"
                    && node.declaration.as_ref().is_some_and(|declaration| {
                        declaration
                            .qualified_name
                            .last()
                            .is_some_and(|part| part.as_ref() == name)
                    })
            })
            .expect("source unit node");
        let CheckedNodeOwner::Source {
            authority,
            identity,
        } = node.owner.as_ref().unwrap()
        else {
            panic!("source owner")
        };
        SelectedSourceUnitRow::new(
            node.node_id.clone(),
            authority.clone(),
            identity.clone(),
            node.declaration.as_ref().unwrap().qualified_name.clone(),
            UnitId::Declared(key(name)),
        )
    }

    fn producer_rows(
        package: &CheckedPackageV2,
    ) -> (
        Vec<SelectedExactDeclarationRow>,
        Vec<SelectedExactFieldRow>,
        Vec<SelectedSourceUnitRow>,
    ) {
        let model = "test/families";
        let record_original = "ix://test/families/Receipt";
        let object_original = "ix://test/families/Account";
        let unit_original = "ix://test/families/unit/Foot";
        let derived_original = "ix://test/families/unit/Inch";
        let record = selected_node(package, record_original);
        let object = selected_node(package, object_original);
        let unit = selected_node(package, unit_original);
        let derived = selected_node(package, derived_original);
        let record_key = key("Receipt");
        let declarations = vec![
            SelectedExactDeclarationRow::new(
                record.clone(),
                model,
                SelectedExactDeclarationForm::RecordValueType,
                record_original,
                SelectedExactIdentity::Record(record_key.clone()),
            ),
            SelectedExactDeclarationRow::new(
                object.clone(),
                model,
                SelectedExactDeclarationForm::ObjectType,
                object_original,
                SelectedExactIdentity::Object(EffectiveId::new(
                    None,
                    OriginalDeclarationKey::new(model, object_original),
                )),
            ),
            SelectedExactDeclarationRow::new(
                unit,
                model,
                SelectedExactDeclarationForm::DeclaredUnit,
                unit_original,
                SelectedExactIdentity::DeclaredUnit(UnitId::Declared(key("Foot"))),
            ),
            SelectedExactDeclarationRow::new(
                derived,
                model,
                SelectedExactDeclarationForm::DeclaredUnit,
                derived_original,
                SelectedExactIdentity::DeclaredUnit(UnitId::Declared(key("Inch"))),
            ),
        ];
        let fields = std::iter::once((record.clone(), record_original, record_key, "amount"))
            .map(|(node, owner, key, name)| {
                SelectedExactFieldRow::new(
                    node,
                    model,
                    owner,
                    format!("{owner}/{name}"),
                    MemberId::new(MemberOwner::Node(key), name).unwrap(),
                )
            })
            .collect();
        (
            declarations,
            fields,
            vec![
                source_unit(package, "Metre"),
                source_unit(package, "Second"),
            ],
        )
    }

    fn source_rows(
        package: &CheckedPackageV2,
    ) -> (
        Vec<crate::SelectedSourceRecordRow>,
        Vec<crate::SelectedSourceTupleRow>,
        Vec<crate::SelectedSourceUnionRow>,
        Vec<crate::SelectedSourceCollectionRow>,
    ) {
        use crate::{
            SelectedSourceCollectionRow, SelectedSourceRecordMemberRow, SelectedSourceRecordRow,
            SelectedSourceTupleRow, SelectedSourceUnionRow, SelectedTuplePositionRow,
            SelectedUnionMemberRow,
        };
        let source = &package.lock().sources[0];
        let mut records = Vec::new();
        let mut tuples = Vec::new();
        let mut unions = Vec::new();
        let mut collections = Vec::new();
        for node in &package.resolved_graph().nodes {
            let Some(CheckedTerm::Aggregate { members }) = package.term(&node.node_id) else {
                continue;
            };
            let qualified = node
                .declaration
                .as_ref()
                .map(|declaration| declaration.qualified_name.clone());
            match (package.node_kind(&node.node_id), qualified) {
                (
                    Some(CheckedNodeKind::CompositeType(super::super::CompositeTypeForm::Record)),
                    Some(qualified_declaration),
                ) => {
                    let members = members
                        .iter()
                        .map(|term| {
                            let CheckedTerm::Binding { name, value } = term else {
                                panic!("record field")
                            };
                            let (presence, target) = match value.as_ref() {
                                CheckedTerm::Reference { target } => {
                                    (Presence::Required, target.clone())
                                }
                                CheckedTerm::Aggregate { members } => {
                                    let [CheckedTerm::Binding { value, .. }] = members.as_slice()
                                    else {
                                        panic!("optional field")
                                    };
                                    let CheckedTerm::Reference { target } = value.as_ref() else {
                                        panic!("optional target")
                                    };
                                    (Presence::Optional, target.clone())
                                }
                                _ => panic!("record field reference"),
                            };
                            SelectedSourceRecordMemberRow {
                                original_member_identity: name.to_string().into(),
                                admitted_name: name.to_string().into(),
                                presence,
                                type_node: target,
                                value_type: ValueType::Integer,
                            }
                        })
                        .collect();
                    records.push(SelectedSourceRecordRow {
                        structural_node: node.node_id.clone(),
                        source_authority: source.authority.clone(),
                        source_identity: source.identity.clone(),
                        qualified_declaration,
                        record_key: key(&format!("fixture-record-{}", records.len())),
                        members,
                    });
                }
                (
                    Some(CheckedNodeKind::CompositeType(super::super::CompositeTypeForm::Tuple)),
                    Some(qualified_declaration),
                ) => {
                    let positions = members
                        .iter()
                        .enumerate()
                        .map(|(ordinal, term)| {
                            let CheckedTerm::Reference { target } = term else {
                                panic!("tuple position")
                            };
                            SelectedTuplePositionRow {
                                ordinal: ordinal as u64,
                                type_node: target.clone(),
                                value_type: ValueType::Integer,
                            }
                        })
                        .collect();
                    tuples.push(SelectedSourceTupleRow {
                        structural_node: node.node_id.clone(),
                        source_authority: source.authority.clone(),
                        source_identity: source.identity.clone(),
                        qualified_declaration,
                        type_handle: key(&format!("fixture-tuple-{}", tuples.len())),
                        positions,
                    });
                }
                (
                    Some(CheckedNodeKind::CompositeType(super::super::CompositeTypeForm::Union)),
                    Some(qualified_declaration),
                ) => {
                    let settled = key("outcome-settled");
                    let members = members
                        .iter()
                        .map(|term| {
                            let CheckedTerm::Binding { name, value } = term else {
                                panic!("union case")
                            };
                            let CheckedTerm::Aggregate { members } = value.as_ref() else {
                                panic!("union payload")
                            };
                            let payload_nodes = members
                                .iter()
                                .map(|item| {
                                    let CheckedTerm::Reference { target } = item else {
                                        panic!("union payload type")
                                    };
                                    target.clone()
                                })
                                .collect::<Vec<_>>();
                            SelectedUnionMemberRow::new(
                                UnionMember::from_admitted(settled.clone(), name.as_ref()).unwrap(),
                                payload_nodes.clone(),
                                vec![ValueType::Integer; payload_nodes.len()],
                            )
                        })
                        .collect();
                    unions.push(SelectedSourceUnionRow::new(
                        node.node_id.clone(),
                        source.authority.clone(),
                        source.identity.clone(),
                        qualified_declaration,
                        key("outcome-handle"),
                        settled,
                        members,
                    ));
                }
                (
                    Some(CheckedNodeKind::CompositeType(super::super::CompositeTypeForm::Sequence)),
                    None,
                ) => {
                    let [CheckedTerm::Reference { target }] = members.as_slice() else {
                        continue;
                    };
                    collections.push(SelectedSourceCollectionRow {
                        structural_node: node.node_id.clone(),
                        base_node: node.node_id.clone(),
                        key_node: None,
                        element_node: target.clone(),
                        value_type: ValueType::collection(CollectionType::new(
                            CollectionKind::Sequence,
                            ValueType::Integer,
                            None,
                        )),
                    });
                }
                (
                    Some(CheckedNodeKind::CompositeType(super::super::CompositeTypeForm::Map)),
                    None,
                ) => {
                    let [CheckedTerm::Binding { value: key, .. }, CheckedTerm::Binding { value, .. }] =
                        members.as_slice()
                    else {
                        panic!("map operands");
                    };
                    let (
                        CheckedTerm::Reference { target: key },
                        CheckedTerm::Reference { target: value },
                    ) = (key.as_ref(), value.as_ref())
                    else {
                        panic!("map reference operands");
                    };
                    collections.push(SelectedSourceCollectionRow {
                        structural_node: node.node_id.clone(),
                        base_node: node.node_id.clone(),
                        key_node: Some(key.clone()),
                        element_node: value.clone(),
                        value_type: ValueType::map(quire_exact::MapType::new(
                            ValueType::Integer,
                            None,
                        )),
                    });
                }
                _ => {}
            }
        }
        for row in &mut collections {
            let element = &row.element_node;
            let value = records
                .iter()
                .find(|record| &record.structural_node == element)
                .map(|record| ValueType::Composite(record.record_key.clone()))
                .or_else(|| {
                    tuples
                        .iter()
                        .find(|tuple| &tuple.structural_node == element)
                        .map(|tuple| ValueType::Composite(tuple.type_handle.clone()))
                })
                .or_else(|| {
                    unions
                        .iter()
                        .find(|union| &union.structural_node == element)
                        .map(|union| ValueType::Composite(union.type_handle.clone()))
                })
                .unwrap_or_else(|| {
                    if package.node_kind(element)
                        == Some(CheckedNodeKind::ScalarType(ScalarTypeForm::Boolean))
                    {
                        ValueType::Boolean
                    } else {
                        ValueType::Integer
                    }
                });
            row.value_type = if row.key_node.is_some() {
                ValueType::map(quire_exact::MapType::new(value, None))
            } else {
                ValueType::collection(CollectionType::new(CollectionKind::Sequence, value, None))
            };
        }
        for record in &mut records {
            for member in &mut record.members {
                if let Some(map) = collections
                    .iter()
                    .find(|row| row.key_node.is_some() && row.structural_node == member.type_node)
                {
                    member.value_type = map.value_type.clone();
                }
            }
        }
        (records, tuples, unions, collections)
    }

    #[trace("TC-470", "FR-357-AC-1", "FR-357-AC-3")]
    #[test]
    fn selected_monitor_closure_reuses_one_meter_for_reached_model_facts() {
        let package = admitted_monitor_for_closure();
        let equality_apps = equality_rows(&package);
        let (declarations, fields, source_units) = producer_rows(&package);
        let (source_records, source_tuples, source_unions, source_collections) =
            source_rows(&package);
        let rows = super::super::selected_type_closure::SelectedMonitorClosureRows {
            declarations: &declarations,
            fields: &fields,
            model_tuples: &[],
            source_units: &source_units,
            model_enum_declarations: &[],
            model_enum_members: &[],
            model_enum_shapes: &[],
            source_records: &source_records,
            source_tuples: &source_tuples,
            source_unions: &source_unions,
            source_enums: &[],
            source_collections: &source_collections,
            equality_applications: &equality_apps,
            equality_facts: &[],
        };
        let plan = package.monitor_plans().next().unwrap();
        let mut budget = NativeJoinBudget::new(u64::MAX, u64::MAX);
        let closure = plan
            .project_selected_type_closure(rows, &mut budget)
            .unwrap();
        assert_eq!(closure.model_types().records().len(), 1);
        assert_eq!(closure.model_types().objects().len(), 1);
        assert_eq!(closure.plan().state(), plan.state());
        assert_eq!(closure.plan().init().node_id(), plan.init().node_id());
        assert_eq!(closure.plan().on_event().body(), plan.on_event().body());
        assert_eq!(
            closure.plan().on_watermark().result_type(),
            plan.on_watermark().result_type()
        );
        assert_eq!(closure.plan().on_gap().parameter_types().len(), 2);
        assert!(!closure.clocks().is_empty());
        assert_eq!(closure.clocks()[0].clause(), plan.clause());
        assert_eq!(closure.equalities().len(), 1);
        let equality = &closure.equalities()[0];
        assert_eq!(equality.id.application, equality_apps[0].application);
        assert_eq!(equality.id.occurrence, equality_apps[0].occurrence);
        assert_eq!(equality.decision.application(), &equality_apps[0].node_key);
        assert_eq!(equality.decision.operator(), EqualityOperator::Equal);
        assert_eq!(equality.decision.schedule(), EqualitySchedule::Plan);
        assert_eq!(equality.decision.left().source(), &ValueType::Boolean);
        assert_eq!(equality.decision.right().source(), &ValueType::Boolean);
        let union = closure
            .identities()
            .source_union_rows()
            .next()
            .expect("reached union");
        assert_eq!(union.members, source_unions[0].members);
        assert_eq!(union.type_handle, source_unions[0].type_handle);
        let projected_union = closure
            .exact_types()
            .iter()
            .find(|projection| projection.type_node() == &union.structural_node)
            .expect("reached union Exact type");
        assert_eq!(
            projected_union.value_type(),
            &ValueType::Composite(union.type_handle.clone())
        );
        let record = closure
            .identities()
            .source_record_rows()
            .find(|row| {
                row.qualified_declaration
                    .last()
                    .is_some_and(|name| name.as_ref() == "Result")
            })
            .expect("reached result record");
        assert_eq!(record.members.len(), 2);
        assert_eq!(record.members[0].admitted_name.as_ref(), "instance");
        assert_eq!(record.members[1].admitted_name.as_ref(), "outcome");
        assert_eq!(record.members[1].value_type, ValueType::Integer);
        let used = budget.consumed();
        assert_eq!(closure.work(), used.work);
        let mut model_budget = NativeJoinBudget::new(u64::MAX, u64::MAX);
        plan.read_reached_model_facts(
            ReachedProducerRows {
                declarations: &declarations,
                fields: &fields,
                tuples: &[],
                source_units: &source_units,
                enum_declarations: &[],
                enum_members: &[],
                enum_shapes: &[],
            },
            &mut model_budget,
        )
        .expect("individually affordable reached read");
        let mut source_budget = NativeJoinBudget::new(u64::MAX, u64::MAX);
        SelectedExactIdentities::empty()
            .with_source_record_rows(&package, source_records.clone(), &mut source_budget)
            .expect("individually affordable source join");
        let mut equality_budget = NativeJoinBudget::new(u64::MAX, u64::MAX);
        let native_equality = package
            .admitted_equality(
                &equality_apps[0].application,
                &equality_apps[0].occurrence,
                &equality_apps,
                &[],
                &source_units,
                &mut equality_budget,
            )
            .expect("individually affordable equality read");
        assert_eq!(native_equality, equality.decision);
        let combined_short = model_budget
            .consumed()
            .work
            .max(source_budget.consumed().work)
            .max(equality_budget.consumed().work);
        assert!(combined_short < used.work);
        let mut combined_budget = NativeJoinBudget::new(combined_short, used.storage);
        assert_eq!(
            plan.project_selected_type_closure(rows, &mut combined_budget)
                .expect_err("combined charge refuses")
                .code,
            super::super::selected_type_closure::SelectedMonitorTypeClosureErrorCode::WorkLimit
        );
        let mut exact = NativeJoinBudget::new(used.work, used.storage);
        assert_eq!(
            plan.project_selected_type_closure(rows, &mut exact)
                .unwrap()
                .exact_types()
                .len(),
            closure.exact_types().len()
        );
        let mut short = NativeJoinBudget::new(used.work - 1, used.storage);
        assert_eq!(
            plan.project_selected_type_closure(rows, &mut short)
                .expect_err("work refusal")
                .code,
            super::super::selected_type_closure::SelectedMonitorTypeClosureErrorCode::WorkLimit
        );
        assert!(short.first_unpaid_locus().is_some());
        let mut short_storage = NativeJoinBudget::new(used.work, used.storage - 1);
        assert_eq!(
            plan.project_selected_type_closure(rows, &mut short_storage)
                .expect_err("storage refusal")
                .code,
            super::super::selected_type_closure::SelectedMonitorTypeClosureErrorCode::StorageLimit
        );
        assert!(short_storage.first_unpaid_locus().is_some());
        let mut zero_storage = NativeJoinBudget::new(used.work, 0);
        let compact = plan
            .project_selected_type_closure(rows, &mut zero_storage)
            .expect_err("zero storage refusal");
        assert_eq!(
            compact.code,
            super::super::selected_type_closure::SelectedMonitorTypeClosureErrorCode::StorageLimit
        );
        assert_eq!(compact.at, None);
        assert_eq!(compact.denied.unwrap().kind, NativeJoinChargeKind::Storage);
        let cloned = package.clone();
        let mut cloned_budget = NativeJoinBudget::new(used.work, used.storage);
        let cloned_closure = cloned
            .monitor_plans()
            .next()
            .unwrap()
            .project_selected_type_closure(rows, &mut cloned_budget)
            .unwrap();
        assert_eq!(cloned_budget.consumed(), used);
        assert_eq!(cloned_closure.model_types(), closure.model_types());
        assert_eq!(cloned_closure.identities(), closure.identities());
        assert_eq!(cloned_closure.exact_types(), closure.exact_types());
        assert_eq!(cloned_closure.equalities(), closure.equalities());
    }

    #[trace("TC-470", "FR-357-AC-3")]
    #[test]
    fn selected_monitor_closure_precharges_variable_equality_key_and_location() {
        use super::super::selected_type_closure::SelectedMonitorClosureRows;

        let package = admitted_monitor_for_closure();
        let applications = equality_rows(&package);
        let (declarations, fields, source_units) = producer_rows(&package);
        let (source_records, source_tuples, source_unions, source_collections) =
            source_rows(&package);
        let plan = package.monitor_plans().next().unwrap();
        let read = |apps: &[crate::SelectedEqualityApplicationRow],
                    budget: &mut NativeJoinBudget| {
            plan.project_selected_type_closure(
                SelectedMonitorClosureRows {
                    declarations: &declarations,
                    fields: &fields,
                    model_tuples: &[],
                    source_units: &source_units,
                    model_enum_declarations: &[],
                    model_enum_members: &[],
                    model_enum_shapes: &[],
                    source_records: &source_records,
                    source_tuples: &source_tuples,
                    source_unions: &source_unions,
                    source_enums: &[],
                    source_collections: &source_collections,
                    equality_applications: apps,
                    equality_facts: &[],
                },
                budget,
            )
        };
        let mut short_budget = NativeJoinBudget::new(u64::MAX, u64::MAX);
        read(&applications, &mut short_budget).expect("short key closure");
        let short_charge = short_budget.consumed();
        let width = usize::try_from(short_charge.storage).unwrap() + 1024;
        let long_key = NodeKey::new(
            package.name(),
            vec![PathSegment::Text("x".repeat(width))],
            0,
        )
        .unwrap();
        let mut long_applications = applications.clone();
        long_applications[0].node_key = long_key.clone();
        long_applications[0].location =
            Location::new(long_key.clone(), Origin::new(Role::from("expression"), 0));
        let mut ample = NativeJoinBudget::new(u64::MAX, u64::MAX);
        let closure = read(&long_applications, &mut ample).expect("long key closure");
        let exact_charge = ample.consumed();
        assert_eq!(closure.equalities()[0].decision.application(), &long_key);
        assert!(exact_charge.storage >= short_charge.storage + (width as u64 - 16) * 2);
        assert!(exact_charge.work > short_charge.work);
        let mut exact = NativeJoinBudget::new(exact_charge.work, exact_charge.storage);
        assert_eq!(
            read(&long_applications, &mut exact).unwrap().equalities(),
            closure.equalities()
        );

        let mut probe = NativeJoinBudget::new(exact_charge.work, short_charge.storage);
        let refusal =
            read(&long_applications, &mut probe).expect_err("long key exceeds short allowance");
        assert_eq!(
            refusal.code,
            super::super::selected_type_closure::SelectedMonitorTypeClosureErrorCode::StorageLimit
        );
        let denied = refusal.denied.expect("first unpaid key bytes");
        assert_eq!(denied.kind, NativeJoinChargeKind::Storage);
        assert_eq!(denied.amount, width as u128 + 1);
        assert_eq!(
            refusal.denied_locus.unwrap().source,
            NativeJoinLocusSource::ReachedRow(0)
        );
        assert_eq!(
            refusal.denied_locus.unwrap().form,
            "equality_application_selection"
        );
        assert_eq!(refusal.at, None);
        let one_below = probe.consumed().storage + u64::try_from(denied.amount).unwrap() - 1;
        let mut one_below_budget = NativeJoinBudget::new(exact_charge.work, one_below);
        let one_below_refusal = read(&long_applications, &mut one_below_budget)
            .expect_err("one below key charge stops before selected-row clone");
        assert_eq!(one_below_refusal.denied, Some(denied));
        assert_eq!(one_below_refusal.denied_locus, refusal.denied_locus);
        assert_eq!(
            one_below_budget.consumed().storage,
            probe.consumed().storage
        );
    }

    #[trace("TC-470", "FR-357-AC-2", "FR-357-AC-3")]
    #[test]
    fn selected_monitor_closure_refuses_reached_rows_and_ignores_unreached_rows() {
        use super::super::selected_type_closure::{
            SelectedMonitorClosureRows, SelectedMonitorTypeClosureErrorCode as Code,
        };
        let package = admitted_monitor_for_closure();
        let equality_apps = equality_rows(&package);
        let (declarations, fields, source_units) = producer_rows(&package);
        let (source_records, source_tuples, source_unions, source_collections) =
            source_rows(&package);
        let plan = package.monitor_plans().next().unwrap();
        let read = |records: &[crate::SelectedSourceRecordRow]| {
            let rows = SelectedMonitorClosureRows {
                declarations: &declarations,
                fields: &fields,
                model_tuples: &[],
                source_units: &source_units,
                model_enum_declarations: &[],
                model_enum_members: &[],
                model_enum_shapes: &[],
                source_records: records,
                source_tuples: &source_tuples,
                source_unions: &source_unions,
                source_enums: &[],
                source_collections: &source_collections,
                equality_applications: &equality_apps,
                equality_facts: &[],
            };
            plan.project_selected_type_closure(rows, &mut NativeJoinBudget::new(u64::MAX, u64::MAX))
                .map(|closure| (closure.work(), closure.identities().clone()))
        };
        let (baseline_work, baseline) = read(&source_records).expect("complete fixture");
        let read_equalities = |applications: &[crate::SelectedEqualityApplicationRow]| {
            let rows = SelectedMonitorClosureRows {
                declarations: &declarations,
                fields: &fields,
                model_tuples: &[],
                source_units: &source_units,
                model_enum_declarations: &[],
                model_enum_members: &[],
                model_enum_shapes: &[],
                source_records: &source_records,
                source_tuples: &source_tuples,
                source_unions: &source_unions,
                source_enums: &[],
                source_collections: &source_collections,
                equality_applications: applications,
                equality_facts: &[],
            };
            plan.project_selected_type_closure(rows, &mut NativeJoinBudget::new(u64::MAX, u64::MAX))
                .map(|closure| (closure.work(), closure.equalities().to_vec()))
        };
        let (equality_work, equality_values) =
            read_equalities(&equality_apps).expect("reached equality");
        assert_eq!(equality_values.len(), 1);
        let missing_equality = read_equalities(&[]).expect_err("reached equality requires row");
        assert_eq!(
            missing_equality.code,
            Code::Equality(crate::AdmittedEqualityReadErrorCode::UnavailableEqualityWitness)
        );
        assert_eq!(
            missing_equality.at.as_deref(),
            Some(&equality_apps[0].application)
        );
        assert_eq!(
            missing_equality.occurrence,
            Some(equality_apps[0].occurrence.clone())
        );
        let mut stale_equality = equality_apps.clone();
        stale_equality[0].node_key = NodeKey::new(
            package.name(),
            vec![PathSegment::Text("stale-equality".into())],
            0,
        )
        .unwrap();
        let stale_error = read_equalities(&stale_equality).expect_err("stale equality row");
        assert_eq!(
            stale_error.code,
            Code::Equality(crate::AdmittedEqualityReadErrorCode::MismatchedEqualityWitness)
        );
        let mut with_unreached_equality = equality_apps.clone();
        let mut irrelevant_equality = equality_apps[0].clone();
        irrelevant_equality.application.package = "foreign/package".into();
        with_unreached_equality.push(irrelevant_equality);
        let (extra_work, same_equalities) = read_equalities(&with_unreached_equality)
            .expect("unreached unsupported equality row ignored");
        assert_eq!(same_equalities, equality_values);
        assert!(extra_work > equality_work);
        let mut missing = source_records.clone();
        missing.retain(|row| {
            row.qualified_declaration
                .last()
                .is_none_or(|name| name.as_ref() != "Result")
        });
        let error = read(&missing).expect_err("reached record requires row");
        assert_eq!(
            error.code,
            Code::Exact(ExactTypeProjectionErrorCode::UnavailableIdentityWitness)
        );
        assert!(error.at.is_some());
        let missing_rows = SelectedMonitorClosureRows {
            declarations: &declarations,
            fields: &fields,
            model_tuples: &[],
            source_units: &source_units,
            model_enum_declarations: &[],
            model_enum_members: &[],
            model_enum_shapes: &[],
            source_records: &missing,
            source_tuples: &source_tuples,
            source_unions: &source_unions,
            source_enums: &[],
            source_collections: &source_collections,
            equality_applications: &equality_apps,
            equality_facts: &[],
        };
        let mut diagnostic_budget = NativeJoinBudget::new(u64::MAX, u64::MAX);
        let semantic = plan
            .project_selected_type_closure(missing_rows, &mut diagnostic_budget)
            .expect_err("complete diagnostic");
        assert_eq!(
            semantic.code,
            Code::Exact(ExactTypeProjectionErrorCode::UnavailableIdentityWitness)
        );
        let mut short_diagnostic =
            NativeJoinBudget::new(u64::MAX, diagnostic_budget.consumed().storage - 1);
        let compact = plan
            .project_selected_type_closure(missing_rows, &mut short_diagnostic)
            .expect_err("diagnostic storage refusal");
        assert_eq!(
            compact.code,
            Code::Exact(ExactTypeProjectionErrorCode::ExhaustedStorage)
        );
        assert_eq!(compact.at, None);
        assert!(compact.denied_locus.is_some());
        let mut foreign = source_records.clone();
        foreign[0].source_identity = "other/source".into();
        let error = read(&foreign).expect_err("foreign selected source refuses");
        assert_eq!(
            error.code,
            Code::Exact(ExactTypeProjectionErrorCode::MismatchedIdentityWitness)
        );
        assert_eq!(error.at.as_deref(), Some(&foreign[0].structural_node));
        let mut stale = source_records.clone();
        stale[0].members[0].type_node = plan.state().clone();
        let error = read(&stale).expect_err("stale member type refuses");
        assert_eq!(
            error.code,
            Code::Exact(ExactTypeProjectionErrorCode::MismatchedIdentityWitness)
        );
        let mut with_unreached = source_records.clone();
        let mut irrelevant = source_records[0].clone();
        irrelevant.structural_node = CheckedNodeId {
            package: "foreign/package".into(),
            path: vec![crate::CheckedNodePathSegment::Name("unsupported".into())],
            ordinal: 0,
        };
        irrelevant.source_identity = "unsupported/source".into();
        with_unreached.push(irrelevant);
        let (work, selected) = read(&with_unreached).expect("unreached row ignored");
        assert_eq!(selected, baseline);
        assert!(
            work > baseline_work,
            "unreached row lookup still charges inspection"
        );
        let unrepresentable = admitted_monitor_for_closure_with_status("text");
        let unrepresentable_equalities = equality_rows(&unrepresentable);
        let (unrepresentable_declarations, unrepresentable_fields, unrepresentable_units) =
            producer_rows(&unrepresentable);
        let (
            unrepresentable_records,
            unrepresentable_tuples,
            unrepresentable_unions,
            unrepresentable_collections,
        ) = source_rows(&unrepresentable);
        let error = unrepresentable
            .monitor_plans()
            .next()
            .unwrap()
            .project_selected_type_closure(
                SelectedMonitorClosureRows {
                    declarations: &unrepresentable_declarations,
                    fields: &unrepresentable_fields,
                    model_tuples: &[],
                    source_units: &unrepresentable_units,
                    model_enum_declarations: &[],
                    model_enum_members: &[],
                    model_enum_shapes: &[],
                    source_records: &unrepresentable_records,
                    source_tuples: &unrepresentable_tuples,
                    source_unions: &unrepresentable_unions,
                    source_enums: &[],
                    source_collections: &unrepresentable_collections,
                    equality_applications: &unrepresentable_equalities,
                    equality_facts: &[],
                },
                &mut NativeJoinBudget::new(u64::MAX, u64::MAX),
            )
            .expect_err("unrepresentable reached type");
        assert_eq!(
            error.code,
            Code::Exact(ExactTypeProjectionErrorCode::UnsupportedExactType),
            "first native locus: {:?}",
            error.at
        );
    }

    /// Trace: FR-357, FR-357-AC-13. The admitted root reaches exactly the
    /// record, object and unit facts asserted below.
    #[trace("TC-469", "FR-357", "FR-357-AC-13", "FR-357-AC-14")]
    #[test]
    fn self_contained_model_fact_monitor_admits() {
        let package = admitted_monitor();
        assert_eq!(package.monitor_plans().len(), 1);
        let (declarations, fields, source_units) = producer_rows(&package);
        let result = read(&package, &declarations, &fields, &source_units, u64::MAX)
            .expect("reached model read");
        assert_eq!(result.types.records().len(), 1);
        assert_eq!(
            result.types.records()[0].original_identity(),
            "ix://test/families/Receipt"
        );
        assert_eq!(result.types.records()[0].view().fields().len(), 1);
        assert_eq!(result.types.objects().len(), 1);
        assert_eq!(
            result.types.objects()[0].original_identity(),
            "ix://test/families/Account"
        );
        assert_eq!(result.types.objects()[0].fields().len(), 2);
        assert_eq!(result.types.model_units().len(), 2);
        assert_eq!(
            result.types.model_units()[0].original_identity.as_ref(),
            "ix://test/families/unit/Foot"
        );
        assert_eq!(result.types.units().len(), 1);
        assert_eq!(
            result.types.units()[0]
                .qualified_declaration
                .last()
                .unwrap()
                .as_ref(),
            "Metre"
        );
        assert_eq!(result.types.objects()[0].operations().len(), 2);
        assert_eq!(result.types.objects()[0].operations()[0].name(), "deposit");
        assert_eq!(
            result.types.objects()[0].operations()[0].parameters().len(),
            1
        );
        assert_eq!(
            result.types.objects()[0].operations()[0].parameters()[0].name(),
            "delta"
        );
        assert!(result.types.objects()[0].operations()[0].result().is_some());
        assert_eq!(result.types.objects()[0].operations()[1].name(), "withdraw");
        assert_eq!(
            result.types.model_units()[0].unit_id,
            UnitId::Declared(key("Foot"))
        );
        assert_eq!(result.types.model_units()[0].scale.numerator.as_ref(), "1");
        assert_eq!(
            result.types.model_units()[0].scale.denominator.as_ref(),
            "1"
        );
        assert_eq!(
            result.types.model_units()[1].original_identity.as_ref(),
            "ix://test/families/unit/Inch"
        );
        assert_eq!(
            result.types.model_units()[1].unit_id,
            UnitId::Declared(key("Inch"))
        );
        assert_eq!(result.types.model_units()[1].scale.numerator.as_ref(), "1");
        assert_eq!(
            result.types.model_units()[1].scale.denominator.as_ref(),
            "12"
        );
        assert_eq!(
            result.types.model_units()[1].target_node.as_ref(),
            Some(&result.types.model_units()[0].structural_node)
        );
        assert_eq!(
            result.types.units()[0].unit_id,
            UnitId::Declared(key("Metre"))
        );
        assert!(matches!(
            package
                .node(&result.types.units()[0].structural_node)
                .unwrap()
                .owner,
            Some(CheckedNodeOwner::Source { .. })
        ));
        assert!(matches!(
            package
                .node(&result.types.model_units()[0].structural_node)
                .unwrap()
                .owner,
            Some(CheckedNodeOwner::Model { .. })
        ));
        assert_eq!(result.types.units()[0].scale.numerator.as_ref(), "1");
        assert_eq!(result.types.units()[0].offset.numerator.as_ref(), "0");
        assert_eq!(result.types.model_units()[0].offset.numerator.as_ref(), "0");
        assert!(result.enums.is_empty());
        assert_eq!(result.work, result.types.work());
        let mut with_unrelated = declarations.clone();
        with_unrelated.push(SelectedExactDeclarationRow::new(
            selected_node(&package, "ix://test/families/Flow"),
            "test/families",
            SelectedExactDeclarationForm::ObjectType,
            "ix://test/families/Flow",
            SelectedExactIdentity::Object(EffectiveId::new(
                None,
                OriginalDeclarationKey::new("test/families", "ix://test/families/Flow"),
            )),
        ));
        let unrelated = read(&package, &with_unrelated, &fields, &source_units, u64::MAX)
            .expect("unreached selected declaration is not projected");
        assert_eq!(unrelated.types.records(), result.types.records());
        assert_eq!(unrelated.types.objects(), result.types.objects());
        assert!(unrelated.work > result.work);
        assert_eq!(
            read(&package, &declarations, &fields, &source_units, result.work)
                .unwrap()
                .types,
            result.types
        );
        match read(
            &package,
            &declarations,
            &fields,
            &source_units,
            result.work - 1,
        ) {
            Err(ReachedModelError::Native(_))
            | Err(ReachedModelError::Walk(RuntimeWalkError::Native(_))) => {}
            Err(ReachedModelError::Exact { error, .. }) => assert_eq!(
                error.code,
                crate::ExactTypeProjectionErrorCode::ExhaustedWork
            ),
            Err(ReachedModelError::Types(error)) => assert_eq!(
                error.code,
                crate::ExactTypeProjectionErrorCode::ExhaustedWork
            ),
            other => panic!("one below shared work limit refuses: {other:?}"),
        }
        let cloned = package.clone();
        assert_eq!(
            read(&cloned, &declarations, &fields, &source_units, result.work)
                .unwrap()
                .types,
            result.types
        );
    }

    #[ix_trace_rs::trace("TC-469", "FR-357-AC-14")]
    #[test]
    fn unrelated_selected_model_does_not_change_reached_work() {
        let baseline = admitted_monitor();
        let with_unrelated = admitted_monitor_with_selected_model(false, true);
        let (declarations, fields, units) = producer_rows(&baseline);
        let expected = read(&baseline, &declarations, &fields, &units, u64::MAX)
            .expect("baseline reached facts");
        let actual = read(
            &with_unrelated,
            &declarations,
            &fields,
            &units,
            expected.work,
        )
        .expect("unrelated selected model uses the same exact allowance");
        assert_eq!(actual.work, expected.work);
        assert_eq!(actual.types, expected.types);
    }

    #[ix_trace_rs::trace("TC-469", "FR-357-AC-14")]
    #[test]
    fn reached_rows_refuse_missing_foreign_and_duplicate_claims() {
        let package = admitted_monitor();
        let (declarations, fields, units) = producer_rows(&package);
        let reached_record = selected_node(&package, "ix://test/families/Receipt");
        let reached_unit = selected_node(&package, "ix://test/families/unit/Foot");
        let missing_field = read(&package, &declarations, &[], &units, u64::MAX).unwrap_err();
        assert!(
            matches!(missing_field,ReachedModelError::Types(ref error)
            if error.code == crate::ExactTypeProjectionErrorCode::UnavailableIdentityWitness && error.at.as_deref() == Some(&reached_record)),
            "{missing_field:?}"
        );
        let without_model_unit = declarations[..2].to_vec();
        let missing_unit =
            read(&package, &without_model_unit, &fields, &units, u64::MAX).unwrap_err();
        assert!(
            matches!(missing_unit,ReachedModelError::Types(ref error)
            if error.code == crate::ExactTypeProjectionErrorCode::UnavailableIdentityWitness && error.at.as_deref() == Some(&reached_unit)),
            "{missing_unit:?}"
        );
        let mut foreign_units = units.clone();
        foreign_units[0].source_authority = "foreign".into();
        let foreign = read(&package, &declarations, &fields, &foreign_units, u64::MAX).unwrap_err();
        assert!(
            matches!(foreign,ReachedModelError::Types(ref error)
            if error.code == crate::ExactTypeProjectionErrorCode::MismatchedIdentityWitness),
            "{foreign:?}"
        );
        let mut duplicate = declarations.clone();
        duplicate.push(declarations[0].clone());
        let duplicate_error = read(&package, &duplicate, &fields, &units, u64::MAX).unwrap_err();
        assert!(
            matches!(duplicate_error,ReachedModelError::Exact {ref error,..}
            if error.code == crate::ExactTypeProjectionErrorCode::ConflictingIdentityWitness),
            "{duplicate_error:?}"
        );
    }

    #[ix_trace_rs::trace("TC-469", "FR-357-AC-14")]
    #[test]
    fn changed_owner_original_and_stale_reached_rows_refuse() {
        use crate::ExactTypeProjectionErrorCode as Code;
        let package = admitted_monitor();
        let (declarations, fields, units) = producer_rows(&package);
        let record = selected_node(&package, "ix://test/families/Receipt");

        let mut changed_owner = declarations.clone();
        changed_owner[0].selected_document = "aaa/other".into();
        let owner_error = read(&package, &changed_owner, &fields, &units, u64::MAX).unwrap_err();
        assert!(
            matches!(owner_error,ReachedModelError::Exact {ref error,..}
            if error.code == Code::MismatchedIdentityWitness && error.at.as_deref() == Some(&record)),
            "{owner_error:?}"
        );

        let mut changed_original = declarations.clone();
        changed_original[0].original_identity = "ix://test/families/Other".into();
        let original_error =
            read(&package, &changed_original, &fields, &units, u64::MAX).unwrap_err();
        assert!(
            matches!(original_error,ReachedModelError::Exact {ref error,..}
            if error.code == Code::MismatchedIdentityWitness && error.at.as_deref() == Some(&record)),
            "{original_error:?}"
        );

        let mut stale_fields = fields.clone();
        stale_fields[0].record_node.package = "foreign".into();
        let stale_field_node = stale_fields[0].record_node.clone();
        let field_error =
            read(&package, &declarations, &stale_fields, &units, u64::MAX).unwrap_err();
        assert!(
            matches!(field_error,ReachedModelError::Exact {ref error,..}
            if error.code == Code::ForeignNode && error.at.as_deref() == Some(&stale_field_node)),
            "{field_error:?}"
        );

        let mut stale_model_units = declarations.clone();
        stale_model_units[2].structural_node.package = "foreign".into();
        let stale_model_node = stale_model_units[2].structural_node.clone();
        let model_error =
            read(&package, &stale_model_units, &fields, &units, u64::MAX).unwrap_err();
        assert!(
            matches!(model_error,ReachedModelError::Exact {ref error,..}
            if error.code == Code::ForeignNode && error.at.as_deref() == Some(&stale_model_node)),
            "{model_error:?}"
        );

        let mut stale_source_units = units.clone();
        stale_source_units[0].structural_node.package = "foreign".into();
        let stale_source_node = stale_source_units[0].structural_node.clone();
        let source_error = read(
            &package,
            &declarations,
            &fields,
            &stale_source_units,
            u64::MAX,
        )
        .unwrap_err();
        assert!(
            matches!(source_error,ReachedModelError::Types(ref error)
            if error.code == Code::ForeignNode && error.at.as_deref() == Some(&stale_source_node)),
            "{source_error:?}"
        );
    }

    #[ix_trace_rs::trace("TC-469", "FR-357-AC-14")]
    #[test]
    fn reached_field_operation_ancestry_and_unit_dependencies_refuse_when_unemitted() {
        use crate::ExactTypeProjectionErrorCode as Code;
        let missing_record = || -> CheckedNodeId {
            serde_json::from_value(fixture_support::structural_id(
                "test/families",
                vec![
                    serde_json::json!("model"),
                    serde_json::json!("record_value_type"),
                    serde_json::json!("ix://test/families/Unemitted"),
                ],
                0,
            ))
            .unwrap()
        };
        let field_package = admitted_document_change(|document| {
            with_unemitted_record(document);
            let receipt = document["types"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|ty| ty["identity"] == "ix://test/families/Receipt")
                .unwrap();
            receipt["fields"][0]["typeRef"] = serde_json::json!("ix://test/families/Unemitted");
        });
        let (decl, fields, units) = producer_rows(&field_package);
        let field_error = read(&field_package, &decl, &fields, &units, u64::MAX).unwrap_err();
        assert!(
            matches!(field_error,ReachedModelError::UnresolvedDependency {ref at,..}
            if at == &missing_record()),
            "{field_error:?}"
        );

        let operation_package = admitted_document_change(|document| {
            with_unemitted_record(document);
            let account = document["types"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|ty| ty["identity"] == "ix://test/families/Account")
                .unwrap();
            account["operations"][0]["params"][0]["typeRef"] =
                serde_json::json!("ix://test/families/Unemitted");
        });
        let (decl, fields, units) = producer_rows(&operation_package);
        let operation_error =
            read(&operation_package, &decl, &fields, &units, u64::MAX).unwrap_err();
        assert!(
            matches!(operation_error,ReachedModelError::UnresolvedDependency {ref at,..}
            if at == &missing_record()),
            "{operation_error:?}"
        );

        let ancestry_package = admitted_document_change(|document| {
            document["types"]
                .as_array_mut()
                .unwrap()
                .push(serde_json::json!({
                    "identity":"ix://test/families/Base","displayName":"Base",
                    "kind":{"module":"test/families","name":"entity"},
                    "roles":[],"constraints":[],"extensions":[],"unknownPolicy":"reject",
                    "supertypes":[],"fields":[],"operations":[]
                }));
            let account = document["types"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|ty| ty["identity"] == "ix://test/families/Account")
                .unwrap();
            account["supertypes"] = serde_json::json!(["ix://test/families/Base"]);
        });
        let (decl, fields, units) = producer_rows(&ancestry_package);
        let account = selected_node(&ancestry_package, "ix://test/families/Account");
        let ancestry_error = read(&ancestry_package, &decl, &fields, &units, u64::MAX).unwrap_err();
        assert!(
            matches!(ancestry_error,ReachedModelError::UnresolvedDependency {ref at,..}
            if at == &account),
            "{ancestry_error:?}"
        );

        let unit_package = admitted_monitor();
        let (mut decl, fields, units) = producer_rows(&unit_package);
        decl.remove(2); // Foot is the reached conversion target of Inch.
        let foot = selected_node(&unit_package, "ix://test/families/unit/Foot");
        let unit_error = read(&unit_package, &decl, &fields, &units, u64::MAX).unwrap_err();
        assert!(
            matches!(unit_error,ReachedModelError::Types(ref error)
            if error.code == Code::UnavailableIdentityWitness && error.at.as_deref() == Some(&foot)),
            "{unit_error:?}"
        );
    }

    #[ix_trace_rs::trace("TC-469", "FR-357-AC-14")]
    #[test]
    fn reached_unrepresentable_member_refuses_without_facts() {
        let package = admitted_document_change(|document| {
            let receipt = document["types"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|ty| ty["identity"] == "ix://test/families/Receipt")
                .unwrap();
            receipt["fields"][0]["typeRef"] = serde_json::json!("ix://quire/native/Float32");
        });
        let (decl, fields, units) = producer_rows(&package);
        let record = selected_node(&package, "ix://test/families/Receipt");
        let error = read(&package, &decl, &fields, &units, u64::MAX).unwrap_err();
        assert!(
            matches!(error,ReachedModelError::Types(ref refusal)
            if refusal.code == crate::ExactTypeProjectionErrorCode::UnsupportedExactType
                && refusal.at.as_deref() == Some(&record)),
            "{error:?}"
        );
    }

    #[ix_trace_rs::trace("TC-469", "FR-357-AC-14")]
    #[test]
    fn reached_variant_needs_qsl_issued_shape() {
        let package = admitted_monitor_with_variant(true);
        let (declarations, fields, units) = producer_rows(&package);
        let variant = selected_node(&package, "ix://test/families/Tier");
        let error = read(&package, &declarations, &fields, &units, u64::MAX).unwrap_err();
        assert!(
            matches!(error,ReachedModelError::Enum {ref error,..}
            if error.code == SelectedModelEnumErrorCode::UnavailableWitness && error.node == variant),
            "{error:?}"
        );
    }

    #[trace("TC-470", "FR-360-AC-1", "FR-360-AC-2", "FR-360-AC-3", "FR-360-AC-4")]
    #[test]
    fn reached_read_shares_exact_work_storage_and_restores_caller_context() {
        let package = admitted_monitor();
        let (declarations, fields, source_units) = producer_rows(&package);
        let plan = package.monitor_plans().next().unwrap();
        let read = |budget: &mut NativeJoinBudget| {
            plan.read_reached_model_facts(
                ReachedProducerRows {
                    declarations: &declarations,
                    fields: &fields,
                    tuples: &[],
                    source_units: &source_units,
                    enum_declarations: &[],
                    enum_members: &[],
                    enum_shapes: &[],
                },
                budget,
            )
        };
        let caller = Some(NativeJoinLocus {
            source: NativeJoinLocusSource::TupleRow(91),
            form: "caller",
        });
        let mut generous = NativeJoinBudget::new(u64::MAX, u64::MAX);
        generous.set_active_locus(caller);
        let complete = read(&mut generous).unwrap();
        let used = generous.consumed();
        assert_eq!(complete.work, used.work);
        assert_eq!(generous.last_success_delta(), Some(used));
        assert_eq!(generous.active_locus(), caller);
        assert!(!generous.exact_diagnostics());
        let mut exact = NativeJoinBudget::new(used.work, used.storage);
        assert_eq!(read(&mut exact).unwrap().types, complete.types);
        assert_eq!(exact.consumed(), used);
        for (work, storage, kind) in [
            (0, u64::MAX, NativeJoinChargeKind::Work),
            (u64::MAX, 0, NativeJoinChargeKind::Storage),
            (used.work - 1, u64::MAX, NativeJoinChargeKind::Work),
            (u64::MAX, used.storage - 1, NativeJoinChargeKind::Storage),
        ] {
            let mut budget = NativeJoinBudget::new(work, storage);
            budget.set_active_locus(caller);
            assert!(read(&mut budget).is_err());
            assert_eq!(budget.first_unpaid().unwrap().kind, kind);
            assert!(budget.first_unpaid_locus().is_some());
            assert_eq!(budget.active_locus(), caller);
            assert!(!budget.exact_diagnostics());
            assert_eq!(budget.last_success_delta(), None);
            if work == 0 || storage == 0 {
                assert_eq!(budget.consumed().work, 0);
                assert_eq!(
                    budget.first_unpaid_locus().unwrap().source,
                    NativeJoinLocusSource::MonitorSeed(0)
                );
                assert_eq!(
                    budget.first_unpaid_locus().unwrap().form,
                    if work == 0 {
                        "monitor_graph_inspection"
                    } else {
                        "monitor_graph_storage"
                    }
                );
            }
        }
        let mut shared = NativeJoinBudget::new(used.work * 2 - 1, u64::MAX);
        read(&mut shared).unwrap();
        let first = shared.consumed();
        assert!(read(&mut shared).is_err());
        assert!(shared.consumed().work >= first.work);
        assert_eq!(
            shared.first_unpaid().unwrap().kind,
            NativeJoinChargeKind::Work
        );
        assert_eq!(shared.last_success_delta(), None);
    }

    #[trace("TC-470", "FR-360-AC-2", "FR-360-AC-3", "FR-360-AC-5")]
    #[test]
    fn reached_enum_shape_uses_the_same_budget_as_model_and_unit_reads() {
        use crate::SelectedModelEnumShapeCaseRow;
        use quire_exact::{EnumShape, VariantId};
        let package = admitted_monitor_with_variant(true);
        let (declarations, fields, source_units) = producer_rows(&package);
        let original = "ix://test/families/Tier";
        let enum_node = selected_node(&package, original);
        let member_node = package
            .resolved_graph()
            .nodes
            .iter()
            .find(|node| {
                node.semantic_form.as_ref() == "enum_value" && node.semantic_type == enum_node
            })
            .unwrap()
            .node_id
            .clone();
        let declaration = SelectedModelEnumDeclarationRow::new(
            enum_node.clone(),
            "test/families",
            original,
            key("Tier"),
        );
        let member = SelectedModelEnumMemberRow::new(
            member_node.clone(),
            enum_node.clone(),
            "test/families",
            original,
            "ix://test/families/Tier/Gold",
            "Gold",
            key("Gold"),
        );
        let variant = VariantId::new(declaration.node_key.clone(), "Gold").unwrap();
        let shape = SelectedModelEnumShapeRow {
            structural_node: enum_node,
            node_key: declaration.node_key.clone(),
            selected_document: declaration.selected_document.clone(),
            original_identity: declaration.original_identity.clone(),
            shape: EnumShape::new(false, [variant.clone()]),
            cases: vec![SelectedModelEnumShapeCaseRow {
                structural_node: member_node,
                case: member.case.clone(),
                member_key: member.member_key.clone(),
                member_original_identity: member.member_original_identity.clone(),
                variant,
            }],
        };
        let enum_declarations = [declaration];
        let enum_members = [member];
        let enum_shapes = [shape];
        let plan = package.monitor_plans().next().unwrap();
        let read = |budget: &mut NativeJoinBudget| {
            plan.read_reached_model_facts(
                ReachedProducerRows {
                    declarations: &declarations,
                    fields: &fields,
                    tuples: &[],
                    source_units: &source_units,
                    enum_declarations: &enum_declarations,
                    enum_members: &enum_members,
                    enum_shapes: &enum_shapes,
                },
                budget,
            )
        };
        let mut budget = NativeJoinBudget::new(u64::MAX, u64::MAX);
        let complete = read(&mut budget).unwrap();
        assert_eq!(complete.enums, enum_shapes);
        assert!(!complete.types.objects().is_empty());
        assert!(!complete.types.model_units().is_empty());
        let used = budget.consumed();
        let mut exact = NativeJoinBudget::new(used.work, used.storage);
        assert_eq!(read(&mut exact).unwrap().types, complete.types);
        let mut exhausted = NativeJoinBudget::new(used.work, used.storage - 1);
        assert!(read(&mut exhausted).is_err());
        assert_eq!(
            exhausted.first_unpaid().unwrap().kind,
            NativeJoinChargeKind::Storage
        );
        assert!(exhausted.first_unpaid_locus().is_some());
    }
    #[trace("TC-470", "FR-357-AC-13", "FR-357-AC-14", "FR-360-AC-2")]
    #[test]
    fn emitted_unreached_declaration_does_not_change_exact_shared_allowance() {
        let (wire, mut document) = monitor_plan_fixtures::model_fact_monitor_with_variant(false);
        with_unemitted_record(&mut document);
        let baseline = admit_wire(&wire, &document);
        let mut emitted = wire;
        let original = "ix://test/families/Unemitted";
        let id = fixture_support::structural_id(
            "test/families",
            vec![
                serde_json::json!("model"),
                serde_json::json!("record_value_type"),
                serde_json::json!(original),
            ],
            0,
        );
        emitted["semantic_graph"]["nodes"].as_array_mut().unwrap().push(serde_json::json!({
            "node_id": id, "schema_version":"quire.checked-semantic-graph/v2", "node_tag":"model", "semantic_form":"record_value_type", "semantic_type":id, "dependencies":[], "occurrences":[{"role":"generated","ordinal":0}], "owner":{"kind":"model","identity":"test/families","node":original}, "body": {"term":"aggregate", "members":[]}
        }));
        fixture_support::rebuild_complete_source_map(&mut emitted);
        fixture_support::refresh_identity(&mut emitted);
        let extra = admit_wire(&emitted, &document);
        assert!(extra
            .models
            .selected_declaration_entry(&selected_node(&extra, original))
            .is_some());
        let (declarations, fields, units) = producer_rows(&baseline);
        let read = |package: &CheckedPackageV2, budget: &mut NativeJoinBudget| {
            package
                .monitor_plans()
                .next()
                .unwrap()
                .read_reached_model_facts(
                    ReachedProducerRows {
                        declarations: &declarations,
                        fields: &fields,
                        tuples: &[],
                        source_units: &units,
                        enum_declarations: &[],
                        enum_members: &[],
                        enum_shapes: &[],
                    },
                    budget,
                )
        };
        let mut generous = NativeJoinBudget::new(u64::MAX, u64::MAX);
        let expected = read(&baseline, &mut generous).unwrap();
        let used = generous.consumed();
        let mut exact = NativeJoinBudget::new(used.work, used.storage);
        let actual = read(&extra, &mut exact).expect(
            "an emitted unreachable declaration must not enter reached lookup or its allowance",
        );
        assert_eq!(actual.types, expected.types);
        assert_eq!(exact.consumed(), used);
    }
    #[trace("TC-470", "FR-360-AC-2", "FR-360-AC-3")]
    #[test]
    fn runtime_field_stream_stops_before_first_unpaid_field_inspection() {
        use std::cell::Cell;
        let package = admitted_monitor();
        let record = selected_node(&package, "ix://test/families/Receipt");
        let inspected = Cell::new(0);
        let (_, fields) = package.models.selected_record_fields(&record).unwrap();
        assert!(fields.len() > 0);
        let fields = fields.inspect(|_| inspected.set(inspected.get() + 1));
        let mut budget = NativeJoinBudget::new(0, u64::MAX);
        let (position, _) = package.node_with_position(&record).unwrap();
        budget.set_active_locus(Some(NativeJoinLocus {
            source: NativeJoinLocusSource::GraphNode(position),
            form: "parent",
        }));
        let mut pending = Vec::new();
        let result = package.monitor_plans().next().unwrap().follow_fields(
            fields,
            &mut pending,
            &mut 0,
            &mut budget,
            &record,
        );
        assert!(matches!(result, Err(ReachedModelError::Native(_))));
        assert_eq!(inspected.get(), 0);
        assert!(pending.is_empty());
        assert_eq!(
            budget.consumed(),
            crate::NativeJoinCharge {
                work: 0,
                storage: 0
            }
        );
        assert_eq!(
            budget.first_unpaid_locus(),
            Some(NativeJoinLocus {
                source: NativeJoinLocusSource::GraphNode(position),
                form: "reached_field"
            })
        );
    }
    #[trace("TC-470", "FR-357-AC-12", "FR-360-AC-2", "FR-360-AC-3")]
    #[test]
    fn repeated_runtime_reference_adds_one_term_charge_and_no_edge_copy() {
        let mut single = admitted_monitor();
        let plan = single.monitor_plans().next().unwrap();
        let entry = plan.init().node_id().clone();
        let target = plan.state().clone();
        let (position, _) = single.node_with_position(&entry).unwrap();
        std::sync::Arc::make_mut(&mut single.wire)
            .semantic_graph
            .nodes[position]
            .body = CheckedTerm::aggregate(vec![CheckedTerm::reference(target.clone())]);
        let mut repeated = single.clone();
        std::sync::Arc::make_mut(&mut repeated.wire)
            .semantic_graph
            .nodes[position]
            .body = CheckedTerm::aggregate(vec![
            CheckedTerm::reference(target.clone()),
            CheckedTerm::reference(target),
        ]);
        let mut once_budget = NativeJoinBudget::new(u64::MAX, u64::MAX);
        let mut twice_budget = NativeJoinBudget::new(u64::MAX, u64::MAX);
        let once = single
            .monitor_plans()
            .next()
            .unwrap()
            .selected_graph_walk(&mut once_budget)
            .unwrap();
        let twice = repeated
            .monitor_plans()
            .next()
            .unwrap()
            .selected_graph_walk(&mut twice_budget)
            .unwrap();
        assert_eq!(twice.nodes, once.nodes);
        assert_eq!(
            twice.work,
            once.work + 1,
            "only the additional term inspection is work; the edge is already admitted"
        );
        assert_eq!(
            twice_budget.consumed().work,
            once_budget.consumed().work + 1
        );
        assert_eq!(
            twice_budget.consumed().storage,
            once_budget.consumed().storage + 2,
            "only the extra borrowed visitor continuation and child frames are retained"
        );
    }
    mod reached_map_rows {
        use super::super::super::selected_type_closure::{
            SelectedMonitorClosureRows, SelectedMonitorTypeClosureErrorCode,
        };
        use super::*;
        use crate::{
            SelectedEqualityApplicationRow, SelectedSourceCollectionRow, SelectedSourceRecordRow,
            SelectedSourceTupleRow, SelectedSourceUnionRow,
        };

        struct Rows {
            declarations: Vec<SelectedExactDeclarationRow>,
            fields: Vec<SelectedExactFieldRow>,
            units: Vec<SelectedSourceUnitRow>,
            records: Vec<SelectedSourceRecordRow>,
            tuples: Vec<SelectedSourceTupleRow>,
            unions: Vec<SelectedSourceUnionRow>,
            collections: Vec<SelectedSourceCollectionRow>,
            equalities: Vec<SelectedEqualityApplicationRow>,
        }
        impl Rows {
            fn new(package: &CheckedPackageV2) -> Self {
                let (declarations, fields, units) = producer_rows(package);
                let (records, tuples, unions, mut collections) = source_rows(package);
                let unrelated = collections
                    .iter()
                    .position(|row| {
                        row.key_node.is_some()
                            && row.value_type
                                == ValueType::map(quire_exact::MapType::new(
                                    ValueType::Boolean,
                                    None,
                                ))
                    })
                    .expect("unrelated boolean map has no opaque record payload");
                let unrelated = collections.remove(unrelated);
                collections.insert(0, unrelated);
                let selected_map = collections.iter().position(|row| row.key_node.is_some()
                    && matches!(&row.value_type, ValueType::Map(map) if matches!(map.values(), ValueType::Composite(_))))
                    .expect("selected map");
                let selected_map = collections.remove(selected_map);
                collections.insert(1, selected_map);
                Self {
                    declarations,
                    fields,
                    units,
                    records,
                    tuples,
                    unions,
                    collections,
                    equalities: equality_rows(package),
                }
            }
            fn view(&self) -> SelectedMonitorClosureRows<'_> {
                SelectedMonitorClosureRows {
                    declarations: &self.declarations,
                    fields: &self.fields,
                    model_tuples: &[],
                    source_units: &self.units,
                    model_enum_declarations: &[],
                    model_enum_members: &[],
                    model_enum_shapes: &[],
                    source_records: &self.records,
                    source_tuples: &self.tuples,
                    source_unions: &self.unions,
                    source_enums: &[],
                    source_collections: &self.collections,
                    equality_applications: &self.equalities,
                    equality_facts: &[],
                }
            }
            fn reached_map(&self) -> &SelectedSourceCollectionRow {
                self.collections.iter().find(|row| row.key_node.is_some()
                    && matches!(&row.value_type, ValueType::Map(map) if matches!(map.values(), ValueType::Composite(_))))
                    .expect("map of authentic source record")
            }
        }

        #[trace("TC-473", "FR-357-AC-19")]
        #[trace("TC-473", "FR-357-AC-20")]
        #[test]
        fn selected_closure_retains_only_reached_map_and_composes_exact_budget() {
            let package = admitted_monitor_for_closure_with_maps("integer", true);
            let rows = Rows::new(&package);
            let row = rows.reached_map();
            let plan = package.monitor_plans().next().expect("plan");
            let mut ample = NativeJoinBudget::new(u64::MAX, u64::MAX);
            let closure = plan
                .project_selected_type_closure(rows.view(), &mut ample)
                .expect("reached map closure");
            let maps: Vec<_> = closure
                .identities()
                .source_collection_rows()
                .filter(|row| row.key_node.is_some())
                .collect();
            assert_eq!(maps, vec![row]);
            let member = closure
                .identities()
                .source_record_rows()
                .flat_map(|record| record.members.iter())
                .find(|member| member.type_node == row.structural_node)
                .expect("the retained Inputs member names the reached map");
            assert_eq!(member.admitted_name.as_ref(), "by_key");
            assert_eq!(member.value_type, row.value_type);

            assert_eq!(
                package.node_kind(row.key_node.as_ref().expect("key")),
                Some(CheckedNodeKind::ScalarType(ScalarTypeForm::Text))
            );
            let record = closure
                .identities()
                .source_record_rows()
                .find(|record| record.structural_node == row.element_node)
                .expect("map value record reached");
            assert_eq!(
                row.value_type,
                ValueType::map(quire_exact::MapType::new(
                    ValueType::Composite(record.record_key.clone()),
                    None
                ))
            );
            let projected = closure
                .exact_types()
                .iter()
                .find(|projection| projection.type_node() == &row.structural_node)
                .expect("map exact projection");
            assert_eq!(projected.value_type(), &row.value_type);
            let used = ample.consumed();
            let mut exact = NativeJoinBudget::new(used.work, used.storage);
            let exact_closure = plan
                .project_selected_type_closure(rows.view(), &mut exact)
                .expect("exact map closure");
            assert_eq!(exact_closure.identities(), closure.identities());
            assert_eq!(exact_closure.exact_types(), closure.exact_types());
            assert_eq!(exact.consumed(), used);
            for kind in [NativeJoinChargeKind::Work, NativeJoinChargeKind::Storage] {
                let limits = match kind {
                    NativeJoinChargeKind::Work => (used.work - 1, used.storage),
                    NativeJoinChargeKind::Storage => (used.work, used.storage - 1),
                };
                let mut short = NativeJoinBudget::new(limits.0, limits.1);
                let error = plan
                    .project_selected_type_closure(rows.view(), &mut short)
                    .expect_err("one under map closure");
                let denied = short.first_unpaid().expect("first unpaid");
                assert_eq!(denied.kind, kind);
                assert_eq!(error.denied, Some(denied));
                assert_eq!(error.denied_locus, short.first_unpaid_locus());
                assert!(short.consumed().work > 0 && short.consumed().storage > 0);
                assert_eq!(short.last_success_delta(), None);
                assert_eq!((short.work_limit(), short.storage_limit()), limits);
            }
        }

        #[trace("TC-473", "FR-357-AC-19")]
        #[test]
        fn selected_map_key_refusal_is_typed_and_unreached_bad_key_is_ignored() {
            let package = admitted_monitor_for_closure_with_maps("integer", true);
            let plan = package.monitor_plans().next().expect("plan");
            for mismatch in [false, true] {
                let mut rows = Rows::new(&package);
                let row = rows.reached_map().structural_node.clone();
                let map = rows
                    .collections
                    .iter_mut()
                    .find(|item| item.structural_node == row)
                    .unwrap();
                map.key_node = mismatch.then(|| map.element_node.clone());
                let mut budget = NativeJoinBudget::new(u64::MAX, u64::MAX);
                let error = plan
                    .project_selected_type_closure(rows.view(), &mut budget)
                    .expect_err("malformed reached key");
                assert_eq!(
                    error.code,
                    SelectedMonitorTypeClosureErrorCode::CollectionKey
                );
                assert_eq!(error.at.as_deref(), Some(&row));
                assert!(error.work > 0);
                assert_eq!(error.denied, None);
                assert_eq!(budget.last_success_delta(), None);
            }
            let mut rows = Rows::new(&package);
            rows.collections[0].key_node = None;
            let mut budget = NativeJoinBudget::new(u64::MAX, u64::MAX);
            let closure = plan
                .project_selected_type_closure(rows.view(), &mut budget)
                .expect("unreached malformed map does not refuse");
            assert!(!closure
                .identities()
                .source_collection_rows()
                .any(|row| row.structural_node == rows.collections[0].structural_node));
            let mut rows = Rows::new(&package);
            let key = rows.reached_map().key_node.clone();
            let unary = rows
                .collections
                .iter_mut()
                .find(|row| row.key_node.is_none())
                .expect("ordinary sequence");
            let node = unary.structural_node.clone();
            unary.key_node = key;
            let error = plan
                .project_selected_type_closure(
                    rows.view(),
                    &mut NativeJoinBudget::new(u64::MAX, u64::MAX),
                )
                .expect_err("unary collection cannot carry map key");
            assert_eq!(
                error.code,
                SelectedMonitorTypeClosureErrorCode::CollectionKey
            );
            assert_eq!(error.at.as_deref(), Some(&node));
        }
        #[trace("TC-473", "FR-357-AC-20")]
        #[test]
        fn selected_map_row_keeps_original_index_and_precharges_its_actual_copy() {
            let package = admitted_monitor_for_closure_with_maps("integer", true);
            let plan = package.monitor_plans().next().expect("plan");
            let rows = Rows::new(&package);
            let index = rows
                .collections
                .iter()
                .position(|row| row.structural_node == rows.reached_map().structural_node)
                .unwrap();
            assert!(
                index > 0,
                "an unrelated earlier row precedes this selected map"
            );
            let mut malformed = Rows::new(&package);
            malformed.collections[index].key_node = None;
            let mut diagnostic = NativeJoinBudget::new(u64::MAX, u64::MAX);
            let error = plan
                .project_selected_type_closure(malformed.view(), &mut diagnostic)
                .unwrap_err();
            assert_eq!(
                error.code,
                SelectedMonitorTypeClosureErrorCode::CollectionKey
            );
            let mut diagnostic_short =
                NativeJoinBudget::new(u64::MAX, diagnostic.consumed().storage - 1);
            plan.project_selected_type_closure(malformed.view(), &mut diagnostic_short)
                .unwrap_err();
            let prefix = diagnostic_short.consumed().storage;
            let mut at_copy = NativeJoinBudget::new(u64::MAX, prefix);
            plan.project_selected_type_closure(rows.view(), &mut at_copy)
                .unwrap_err();
            let copy_charge = at_copy.first_unpaid().expect("map row retention charge");
            assert_eq!(copy_charge.kind, NativeJoinChargeKind::Storage);
            assert_eq!(
                at_copy.first_unpaid_locus(),
                Some(NativeJoinLocus {
                    source: NativeJoinLocusSource::ReachedRow(index),
                    form: "source_collection_selection",
                })
            );
            let exact_copy = prefix + u64::try_from(copy_charge.amount).unwrap();
            let mut short = NativeJoinBudget::new(u64::MAX, exact_copy - 1);
            let mut result = None;
            let unpaid = allocation_counter::measure(|| {
                result = Some(plan.project_selected_type_closure(rows.view(), &mut short));
            });
            let error = result.unwrap().expect_err("one below selected map copy");
            assert_eq!(error.denied, Some(copy_charge));
            assert_eq!(
                error.denied_locus,
                Some(NativeJoinLocus {
                    source: NativeJoinLocusSource::ReachedRow(index),
                    form: "source_collection_selection",
                })
            );
            assert_eq!(short.consumed().storage, prefix);
            assert_eq!(short.storage_limit(), exact_copy - 1);
            assert_eq!(short.last_success_delta(), None);
            let mut paid = NativeJoinBudget::new(u64::MAX, exact_copy);
            let mut paid_result = None;
            let admitted = allocation_counter::measure(|| {
                paid_result = Some(plan.project_selected_type_closure(rows.view(), &mut paid));
            });
            paid_result.unwrap().expect_err("later join remains unpaid");
            assert!(
                admitted.count_total > unpaid.count_total,
                "the selected map row is actually cloned only after its storage admission"
            );
            assert!(unpaid.count_total > 0, "earlier paid work allocated");
            assert_eq!(paid.consumed().storage, exact_copy);
            let mut zero = NativeJoinBudget::new(u64::MAX, 0);
            let mut zero_result = None;
            let zero_allocations = allocation_counter::measure(|| {
                zero_result = Some(plan.project_selected_type_closure(rows.view(), &mut zero));
            });
            let error = zero_result.unwrap().expect_err("zero storage");
            assert_eq!(zero_allocations.count_total, 0);
            assert_eq!(
                error.code,
                SelectedMonitorTypeClosureErrorCode::StorageLimit
            );
            assert_eq!(error.at, None);
            assert_eq!(zero.consumed().storage, 0);
        }
    }
}
