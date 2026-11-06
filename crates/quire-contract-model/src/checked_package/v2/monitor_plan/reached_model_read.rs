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
                    read_node_storage(budget, &dimension)?;
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
                        read_node_storage(budget, &target)?;
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
                    read_node_storage(budget, &target)?;
                    retain(budget, 1)?;
                    pending.push((
                        target.clone(),
                        budget
                            .active_locus()
                            .expect("reached parent context")
                            .source,
                    ));
                }
                retain(budget, 1)?;
                let fields = if declaration.form == DeclarationForm::RecordValueType {
                    self.package
                        .models
                        .selected_record_fields(&id)
                        .map(|(_, mut fields)| {
                            fields.try_fold(
                                Vec::new(),
                                |mut retained, field| -> Result<_, ReachedModelError> {
                                    retain(budget, 1)?;
                                    retained.push(field);
                                    Ok(retained)
                                },
                            )
                        })
                        .transpose()?
                } else {
                    self.package
                        .models
                        .selected_object_fields(&id)
                        .map(|(_, mut fields)| {
                            fields.try_fold(
                                Vec::new(),
                                |mut retained, field| -> Result<_, ReachedModelError> {
                                    retain(budget, 1)?;
                                    retained.push(field);
                                    Ok(retained)
                                },
                            )
                        })
                        .transpose()?
                };
                for field in fields.ok_or_else(|| unresolved(budget, &id, *work))? {
                    budget.set_locus_source(NativeJoinLocusSource::GraphNode(graph_position));
                    charge(work, budget, &id)?;
                    if let Some(member) = field.member_type {
                        self.follow_member_type(member, &mut pending, work, budget, &id)?;
                    }
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
            for id in graph.nodes {
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
                let mut matching = enum_shapes.iter().filter(|row| &row.structural_node == id);
                let row = match matching.next() {
                    Some(row) => row,
                    None => {
                        return Err(checked_enum_error(
                            budget,
                            SelectedModelEnumErrorCode::UnavailableWitness,
                            id,
                            work,
                        ))
                    }
                };
                if matching.next().is_some() {
                    return Err(checked_enum_error(
                        budget,
                        SelectedModelEnumErrorCode::ConflictingWitness,
                        id,
                        work,
                    ));
                }
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
                )
                .map_err(ReachedModelError::Types)?;
            work = budget.consumed().work - before.work;
            Ok(ReachedModelRead {
                types,
                enums: selected_enums,
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
mod fixture_support;

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
    use crate::checked_package::v2::{SelectedExactDeclarationForm, SelectedExactIdentity};
    use ix_trace_rs::trace;
    use quire_exact::{
        EffectiveId, MemberId, MemberOwner, NodeKey, OriginalDeclarationKey, PathSegment, UnitId,
    };

    fn admitted_monitor() -> CheckedPackageV2 {
        admitted_monitor_with_variant(false)
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
            .graph()
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
            .graph()
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
}
