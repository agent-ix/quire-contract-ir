// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Atomic selected monitor type closure (FR-357-AC-1..4).

use super::reached_model_read::{ReachedModelError, ReachedProducerRows};
use super::selected_closure_walk::{RuntimeWalkError, SelectedGraphWalkErrorCode};
use super::{
    CheckedMonitorClockRead, CheckedMonitorClockReadError, CheckedMonitorClockReadErrorCode,
    CheckedMonitorPlanRef,
};
use crate::checked_package::v2::equality_projection::equality_operator;
use crate::checked_package::v2::native_join_budget::read_charge;
use crate::checked_package::v2::native_join_storage as storage;
use crate::checked_package::v2::{
    AdmittedEqualityReadError, AdmittedEqualityReadErrorCode, ApplicationOperator,
    BoundedDomainForm, CheckedNodeId, CheckedNodeKind, CheckedOccurrence, CheckedOccurrenceRole,
    CheckedTerm, EqualityFactWitness, ExactTypeProjection, ExactTypeProjectionError,
    ExactTypeProjectionErrorCode, NativeJoinBudget, NativeJoinChargeKind, NativeJoinDeniedCharge,
    NativeJoinLocus, NativeJoinLocusSource, ScalarTypeForm, SelectedEqualityApplicationRow,
    SelectedEqualityFactRow, SelectedExactDeclarationRow, SelectedExactFieldRow,
    SelectedExactIdentities, SelectedFactEquality, SelectedFactEqualityId,
    SelectedModelEnumDeclarationRow, SelectedModelEnumErrorCode, SelectedModelEnumMemberRow,
    SelectedModelEnumShapeRow, SelectedSourceCollectionRow, SelectedSourceEnumRow,
    SelectedSourceRecordRow, SelectedSourceTupleRow, SelectedSourceUnionRow, SelectedSourceUnitRow,
    SelectedTupleDeclarationRow, SelectedTypeDeclarations, SelectedTypeDeclarationsError,
};
use quire_exact::{NodeKey, PathSegmentView};
use std::collections::BTreeSet;

/// Producer rows from one final emission, split by their native selected channels.
#[derive(Clone, Copy)]
pub struct SelectedMonitorClosureRows<'a> {
    /// Selected model and declared-unit Exact identities.
    pub declarations: &'a [SelectedExactDeclarationRow],
    /// Selected model record-field identities.
    pub fields: &'a [SelectedExactFieldRow],
    /// Selected model tuple identities.
    pub model_tuples: &'a [SelectedTupleDeclarationRow],
    /// Selected source units.
    pub source_units: &'a [SelectedSourceUnitRow],
    /// Selected model enum declarations.
    pub model_enum_declarations: &'a [SelectedModelEnumDeclarationRow],
    /// Selected model enum members.
    pub model_enum_members: &'a [SelectedModelEnumMemberRow],
    /// Complete selected model enum shapes.
    pub model_enum_shapes: &'a [SelectedModelEnumShapeRow],
    /// Complete selected source records.
    pub source_records: &'a [SelectedSourceRecordRow],
    /// Complete selected source tuples.
    pub source_tuples: &'a [SelectedSourceTupleRow],
    /// Complete selected source sealed unions.
    pub source_unions: &'a [SelectedSourceUnionRow],
    /// Complete selected source enum declarations and cases.
    pub source_enums: &'a [SelectedSourceEnumRow],
    /// Selected structural source collections.
    pub source_collections: &'a [SelectedSourceCollectionRow],
    /// Same-emission applications for reached admitted equality decisions.
    pub equality_applications: &'a [SelectedEqualityApplicationRow],
    /// Same-emission facts for reached admitted equality decisions.
    pub equality_facts: &'a [SelectedEqualityFactRow],
}

/// A complete result for one admitted monitor root, with no executable plan.
#[derive(Clone)]
pub struct SelectedMonitorTypeClosure<'a> {
    plan: CheckedMonitorPlanRef<'a>,
    types: SelectedTypeDeclarations,
    model_enums: Vec<SelectedModelEnumShapeRow>,
    identities: SelectedExactIdentities,
    exact_types: Vec<ExactTypeProjection>,
    clocks: Vec<CheckedMonitorClockRead<'a>>,
    equalities: Vec<SelectedFactEquality>,
    work: u64,
}

impl std::fmt::Debug for SelectedMonitorTypeClosure<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SelectedMonitorTypeClosure")
            .field("root", self.plan.node_id())
            .field("types", &self.types)
            .field("model_enums", &self.model_enums)
            .field("identities", &self.identities)
            .field("exact_types", &self.exact_types)
            .field("clocks", &self.clocks)
            .field("equalities", &self.equalities)
            .field("work", &self.work)
            .finish()
    }
}

impl<'a> SelectedMonitorTypeClosure<'a> {
    /// The selected admitted root, four type handles, and ordered entry bodies.
    pub fn plan(&self) -> CheckedMonitorPlanRef<'a> {
        self.plan
    }
    /// Complete reached model records, tuples, objects, units and dimensions.
    pub fn model_types(&self) -> &SelectedTypeDeclarations {
        &self.types
    }
    /// Complete reached model enum shapes and original case identities.
    pub fn model_enums(&self) -> &[SelectedModelEnumShapeRow] {
        &self.model_enums
    }
    /// Complete joined source rows and opaque declaration identities.
    pub fn identities(&self) -> &SelectedExactIdentities {
        &self.identities
    }
    /// Reached Exact types in structural identity order, including complete source views.
    pub fn exact_types(&self) -> &[ExactTypeProjection] {
        &self.exact_types
    }
    /// Selected clause clocks in admitted claim-occurrence order.
    pub fn clocks(&self) -> &[CheckedMonitorClockRead<'a>] {
        &self.clocks
    }
    /// Complete reached equality decisions in structural node and occurrence order.
    pub fn equalities(&self) -> &[SelectedFactEquality] {
        &self.equalities
    }
    /// Total work charged to the shared caller allowance for this read.
    pub fn work(&self) -> u64 {
        self.work
    }
}

/// Stable cause of an atomic selected monitor closure refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectedMonitorTypeClosureErrorCode {
    /// A reached local node is unavailable.
    UnknownNode,
    /// A reached dependency belongs to another package.
    ForeignDependency,
    /// A reached term form has no ordinary closure rule.
    UnsupportedTerm,
    /// A reached model or unit dependency is unresolved.
    UnresolvedDependency,
    /// The selected model enum join refused.
    ModelEnum(SelectedModelEnumErrorCode),
    /// Exact identity or type projection refused.
    Exact(ExactTypeProjectionErrorCode),
    /// A reached map key or unary collection key disagrees with the admitted edge.
    CollectionKey,
    /// Runtime declaration projection refused.
    TypeDeclaration(ExactTypeProjectionErrorCode),
    /// Selected clause clock correspondence refused.
    Clock(CheckedMonitorClockReadErrorCode),
    /// A reached admitted equality decision refused.
    Equality(AdmittedEqualityReadErrorCode),
    /// The next work charge exceeds the allowance.
    WorkLimit,
    /// The next owned storage charge exceeds the allowance.
    StorageLimit,
}

/// First typed refusal, without any partial selected closure.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("selected monitor type closure refused: {code:?} at {at:?} ({denied_locus:?})")]
pub struct SelectedMonitorTypeClosureError {
    /// Stable native cause.
    pub code: SelectedMonitorTypeClosureErrorCode,
    /// First native node when its owned diagnostic was admitted.
    pub at: Option<Box<CheckedNodeId>>,
    /// First selected claim occurrence when its diagnostic was admitted.
    pub occurrence: Option<CheckedOccurrence>,
    /// Work already charged to the caller's meter.
    pub work: u64,
    /// First refused charge, if an allowance stopped the read.
    pub denied: Option<NativeJoinDeniedCharge>,
    /// Fixed-size first native row, node, or seed location.
    pub denied_locus: Option<NativeJoinLocus>,
}

fn native_error(budget: &NativeJoinBudget) -> SelectedMonitorTypeClosureError {
    let denied = budget.first_unpaid();
    SelectedMonitorTypeClosureError {
        code: match denied.map(|charge| charge.kind) {
            Some(NativeJoinChargeKind::Storage) => {
                SelectedMonitorTypeClosureErrorCode::StorageLimit
            }
            _ => SelectedMonitorTypeClosureErrorCode::WorkLimit,
        },
        at: None,
        occurrence: None,
        work: budget.consumed().work,
        denied,
        denied_locus: budget.first_unpaid_locus(),
    }
}

fn semantic_error(
    code: SelectedMonitorTypeClosureErrorCode,
    at: Option<Box<CheckedNodeId>>,
    budget: &NativeJoinBudget,
) -> SelectedMonitorTypeClosureError {
    SelectedMonitorTypeClosureError {
        code,
        at,
        occurrence: None,
        work: budget.consumed().work,
        denied: budget.first_unpaid(),
        denied_locus: budget.first_unpaid_locus(),
    }
}

fn exact_error(
    error: ExactTypeProjectionError,
    budget: &NativeJoinBudget,
) -> SelectedMonitorTypeClosureError {
    let at = error.at.or(error.requested);
    semantic_error(
        SelectedMonitorTypeClosureErrorCode::Exact(error.code),
        at,
        budget,
    )
}

fn reached_error(
    error: ReachedModelError,
    budget: &NativeJoinBudget,
) -> SelectedMonitorTypeClosureError {
    match error {
        ReachedModelError::Walk(RuntimeWalkError::Semantic(error)) => semantic_error(
            match error.code {
                SelectedGraphWalkErrorCode::UnknownNode => {
                    SelectedMonitorTypeClosureErrorCode::UnknownNode
                }
                SelectedGraphWalkErrorCode::ForeignDependency => {
                    SelectedMonitorTypeClosureErrorCode::ForeignDependency
                }
                SelectedGraphWalkErrorCode::UnsupportedTerm => {
                    SelectedMonitorTypeClosureErrorCode::UnsupportedTerm
                }
            },
            Some(Box::new(error.at)),
            budget,
        ),
        ReachedModelError::Walk(RuntimeWalkError::Native(_)) | ReachedModelError::Native(_) => {
            native_error(budget)
        }
        ReachedModelError::Exact { error, .. } => exact_error(error, budget),
        ReachedModelError::Types(error) => runtime_error(error, budget),
        ReachedModelError::Enum { error, .. } => semantic_error(
            SelectedMonitorTypeClosureErrorCode::ModelEnum(error.code),
            Some(Box::new(error.node)),
            budget,
        ),
        ReachedModelError::UnresolvedDependency { at, .. } => semantic_error(
            SelectedMonitorTypeClosureErrorCode::UnresolvedDependency,
            Some(Box::new(at)),
            budget,
        ),
    }
}

fn runtime_error(
    error: SelectedTypeDeclarationsError,
    budget: &NativeJoinBudget,
) -> SelectedMonitorTypeClosureError {
    semantic_error(
        SelectedMonitorTypeClosureErrorCode::TypeDeclaration(error.code),
        error.at,
        budget,
    )
}

fn clock_error(
    error: CheckedMonitorClockReadError,
    budget: &NativeJoinBudget,
) -> SelectedMonitorTypeClosureError {
    SelectedMonitorTypeClosureError {
        code: SelectedMonitorTypeClosureErrorCode::Clock(error.code),
        at: error.at,
        occurrence: error.occurrence,
        work: budget.consumed().work,
        denied: error.denied.or_else(|| budget.first_unpaid()),
        denied_locus: budget.first_unpaid_locus(),
    }
}

fn equality_error(
    error: AdmittedEqualityReadError,
    occurrence: &CheckedOccurrence,
    budget: &NativeJoinBudget,
) -> SelectedMonitorTypeClosureError {
    SelectedMonitorTypeClosureError {
        code: SelectedMonitorTypeClosureErrorCode::Equality(error.code),
        at: error.at.or(error.application),
        occurrence: Some(occurrence.clone()),
        work: budget.consumed().work,
        denied: budget.first_unpaid(),
        denied_locus: budget.first_unpaid_locus(),
    }
}

fn source_map_storage(entry: &crate::CheckedSourceMapEntry) -> u128 {
    entry.regions.iter().fold(
        storage::node(&entry.node_id).saturating_add(1),
        |total, region| {
            total
                .saturating_add(region.source.authority.len() as u128)
                .saturating_add(region.source.identity.len() as u128)
                .saturating_add(3)
        },
    )
}

// The two opaque keys are charged by preflight_equality_application before this
// fixed row and source-map footprint is calculated.
fn equality_application_storage(row: &SelectedEqualityApplicationRow) -> u128 {
    storage::node(&row.application)
        .saturating_add(source_map_storage(&row.source_map_entry))
        .saturating_add(row.location.occurrence().role().as_str().len() as u128)
        .saturating_add(4)
}

fn equality_fact_storage(row: &SelectedEqualityFactRow) -> u128 {
    let owner = match row.owner.as_ref() {
        Some(crate::CheckedNodeOwner::ModelRoot { identity }) => identity.len() as u128,
        Some(crate::CheckedNodeOwner::Source {
            authority,
            identity,
        }) => authority.len().saturating_add(identity.len()) as u128,
        Some(crate::CheckedNodeOwner::Model { identity, node }) => {
            identity.len().saturating_add(node.len()) as u128
        }
        None => 0,
    };
    let fact = match &row.fact {
        EqualityFactWitness::OperandType { .. } | EqualityFactWitness::TextPlanType { .. } => 1,
        EqualityFactWitness::Unit {
            unit_id,
            quantity_type,
            ..
        } => storage::unit(unit_id).saturating_add(storage::node(quantity_type)),
        EqualityFactWitness::DimensionDeclaration { unit_id, .. } => storage::unit(unit_id),
        EqualityFactWitness::DimensionTerm {
            unit_id,
            base_node,
            exponent,
            ..
        } => storage::unit(unit_id)
            .saturating_add(storage::node(base_node))
            .saturating_add(exponent.len() as u128),
        EqualityFactWitness::ConversionEdge {
            unit_id,
            target_node,
            scale,
            offset,
            ..
        } => storage::unit(unit_id)
            .saturating_add(storage::node(target_node))
            .saturating_add(scale.numerator.len() as u128)
            .saturating_add(scale.denominator.len() as u128)
            .saturating_add(offset.numerator.len() as u128)
            .saturating_add(offset.denominator.len() as u128),
        EqualityFactWitness::EnumMember {
            source_declaration_node,
            declaration_node,
            member_node,
            case,
            ..
        } => storage::node(source_declaration_node)
            .saturating_add(storage::node(declaration_node))
            .saturating_add(storage::node(member_node))
            .saturating_add(case.len() as u128),
    };
    storage::node(&row.checked_application)
        .saturating_add(storage::node(&row.structural_node))
        .saturating_add(source_map_storage(&row.source_map_entry))
        .saturating_add(row.value_type.as_ref().map_or(0, storage::value_type))
        .saturating_add(owner)
        .saturating_add(fact)
        .saturating_add(5)
}

fn preflight_work(
    budget: &mut NativeJoinBudget,
    count: usize,
) -> Result<(), SelectedMonitorTypeClosureError> {
    read_charge(budget, NativeJoinChargeKind::Work, count as u128).map_err(|_| native_error(budget))
}

fn charge_opaque_key(
    key: &NodeKey,
    budget: &mut NativeJoinBudget,
) -> Result<(), SelectedMonitorTypeClosureError> {
    read_charge(budget, NativeJoinChargeKind::Storage, 1).map_err(|_| native_error(budget))?;
    let mut pending = vec![key.view()];
    while let Some(view) = pending.pop() {
        read_charge(budget, NativeJoinChargeKind::Work, 1).map_err(|_| native_error(budget))?;
        read_charge(
            budget,
            NativeJoinChargeKind::Storage,
            (view.package().len() as u128).saturating_add(2),
        )
        .map_err(|_| native_error(budget))?;
        for segment in view.path() {
            read_charge(budget, NativeJoinChargeKind::Work, 1).map_err(|_| native_error(budget))?;
            match segment {
                PathSegmentView::Text(text) => {
                    let bytes = text.len() as u128;
                    read_charge(budget, NativeJoinChargeKind::Work, bytes)
                        .map_err(|_| native_error(budget))?;
                    read_charge(
                        budget,
                        NativeJoinChargeKind::Storage,
                        bytes.saturating_add(1),
                    )
                    .map_err(|_| native_error(budget))?;
                }
                PathSegmentView::Node(child) => {
                    read_charge(budget, NativeJoinChargeKind::Storage, 1)
                        .map_err(|_| native_error(budget))?;
                    pending.push(child);
                }
            }
        }
    }
    Ok(())
}

fn preflight_equality_application(
    row: &SelectedEqualityApplicationRow,
    budget: &mut NativeJoinBudget,
) -> Result<(), SelectedMonitorTypeClosureError> {
    preflight_work(budget, row.source_map_entry.regions.len())?;
    charge_opaque_key(&row.node_key, budget)?;
    charge_opaque_key(row.location.node(), budget)
}

fn preflight_source_record(
    row: &SelectedSourceRecordRow,
    budget: &mut NativeJoinBudget,
) -> Result<(), SelectedMonitorTypeClosureError> {
    preflight_work(
        budget,
        row.qualified_declaration
            .len()
            .saturating_add(row.members.len()),
    )?;
    for member in &row.members {
        storage::preflight_value_type(&member.value_type, budget, &member.type_node)
            .map_err(|error| exact_error(error, budget))?;
    }
    Ok(())
}

fn preflight_source_tuple(
    row: &SelectedSourceTupleRow,
    budget: &mut NativeJoinBudget,
) -> Result<(), SelectedMonitorTypeClosureError> {
    preflight_work(
        budget,
        row.qualified_declaration
            .len()
            .saturating_add(row.positions.len()),
    )?;
    for position in &row.positions {
        storage::preflight_value_type(&position.value_type, budget, &position.type_node)
            .map_err(|error| exact_error(error, budget))?;
    }
    Ok(())
}

fn preflight_source_union(
    row: &SelectedSourceUnionRow,
    budget: &mut NativeJoinBudget,
) -> Result<(), SelectedMonitorTypeClosureError> {
    preflight_work(
        budget,
        row.qualified_declaration
            .len()
            .saturating_add(row.members.len()),
    )?;
    for member in &row.members {
        preflight_work(
            budget,
            member
                .payload_nodes
                .len()
                .saturating_add(member.payload.len()),
        )?;
        for value in &member.payload {
            storage::preflight_value_type(value, budget, &row.structural_node)
                .map_err(|error| exact_error(error, budget))?;
        }
    }
    Ok(())
}

fn preflight_source_enum(
    row: &SelectedSourceEnumRow,
    budget: &mut NativeJoinBudget,
) -> Result<(), SelectedMonitorTypeClosureError> {
    preflight_work(
        budget,
        row.declaration
            .qualified_declaration
            .len()
            .saturating_add(row.members.len()),
    )?;
    storage::preflight_enum_shape(
        &row.declaration.shape,
        budget,
        &row.declaration.structural_node,
    )
    .map_err(|error| exact_error(error, budget))?;
    for member in &row.members {
        preflight_work(budget, member.declaration_qualified_name.len())?;
    }
    Ok(())
}

fn preflight_source_collection(
    package: &super::super::CheckedPackageV2,
    row: &SelectedSourceCollectionRow,
    budget: &mut NativeJoinBudget,
) -> Result<(), SelectedMonitorTypeClosureError> {
    read_charge(budget, NativeJoinChargeKind::Work, 1).map_err(|_| native_error(budget))?;
    let map = package.node_kind(&row.base_node)
        == Some(super::super::CheckedNodeKind::CompositeType(
            super::super::CompositeTypeForm::Map,
        ));
    if (map && !super::super::exact_projection::source_map_key_matches(package, row))
        || (!map && row.key_node.is_some())
    {
        read_charge(
            budget,
            NativeJoinChargeKind::Storage,
            storage::node(&row.structural_node),
        )
        .map_err(|_| native_error(budget))?;
        return Err(semantic_error(
            SelectedMonitorTypeClosureErrorCode::CollectionKey,
            Some(Box::new(row.structural_node.clone())),
            budget,
        ));
    }
    storage::preflight_value_type(&row.value_type, budget, &row.structural_node)
        .map_err(|error| exact_error(error, budget))
}

fn preflight_source_unit(
    row: &SelectedSourceUnitRow,
    budget: &mut NativeJoinBudget,
) -> Result<(), SelectedMonitorTypeClosureError> {
    preflight_work(budget, row.qualified_declaration.len())?;
    storage::preflight_unit(&row.unit_id, budget, &row.structural_node)
        .map_err(|error| exact_error(error, budget))
}

fn select_rows<T: Clone>(
    rows: &[T],
    reached: &BTreeSet<CheckedNodeId>,
    budget: &mut NativeJoinBudget,
    form: &'static str,
    node: impl Fn(&T) -> &CheckedNodeId,
    storage_units: impl Fn(&T) -> u128,
    preflight: impl Fn(&T, &mut NativeJoinBudget) -> Result<(), SelectedMonitorTypeClosureError>,
) -> Result<Vec<T>, SelectedMonitorTypeClosureError> {
    budget.set_locus_form(form);
    read_charge(budget, NativeJoinChargeKind::Storage, 1).map_err(|_| native_error(budget))?;
    let mut selected = Vec::new();
    for (index, row) in rows.iter().enumerate() {
        budget.set_active_locus(Some(NativeJoinLocus {
            source: NativeJoinLocusSource::ReachedRow(index),
            form,
        }));
        read_charge(budget, NativeJoinChargeKind::Work, 1).map_err(|_| native_error(budget))?;
        let id = node(row);
        if reached.contains(id) {
            preflight(row, budget)?;
            read_charge(budget, NativeJoinChargeKind::Storage, storage_units(row))
                .map_err(|_| native_error(budget))?;
            selected.push(row.clone());
        }
    }
    Ok(selected)
}

fn select_source_collections(
    package: &super::super::CheckedPackageV2,
    rows: &[SelectedSourceCollectionRow],
    reached: &BTreeSet<CheckedNodeId>,
    budget: &mut NativeJoinBudget,
) -> Result<Vec<(usize, SelectedSourceCollectionRow)>, SelectedMonitorTypeClosureError> {
    budget.set_locus_form("source_collection_selection");
    read_charge(budget, NativeJoinChargeKind::Storage, 1).map_err(|_| native_error(budget))?;
    let mut selected = Vec::new();
    for (index, row) in rows.iter().enumerate() {
        budget.set_active_locus(Some(NativeJoinLocus {
            source: NativeJoinLocusSource::ReachedRow(index),
            form: "source_collection_selection",
        }));
        read_charge(budget, NativeJoinChargeKind::Work, 1).map_err(|_| native_error(budget))?;
        if reached.contains(&row.structural_node) {
            preflight_source_collection(package, row, budget)?;
            budget.set_locus_form("source_collection_selection");
            read_charge(
                budget,
                NativeJoinChargeKind::Storage,
                storage::source_collection(row).saturating_add(1),
            )
            .map_err(|_| native_error(budget))?;
            selected.push((index, row.clone()));
        }
    }
    Ok(selected)
}

fn select_equality_facts(
    rows: &[SelectedEqualityFactRow],
    reached: &BTreeSet<CheckedNodeId>,
    budget: &mut NativeJoinBudget,
) -> Result<Vec<SelectedEqualityFactRow>, SelectedMonitorTypeClosureError> {
    budget.set_locus_form("equality_fact_selection");
    read_charge(budget, NativeJoinChargeKind::Storage, 1).map_err(|_| native_error(budget))?;
    let mut selected = Vec::new();
    for (index, row) in rows.iter().enumerate() {
        budget.set_active_locus(Some(NativeJoinLocus {
            source: NativeJoinLocusSource::ReachedRow(index),
            form: "equality_fact_selection",
        }));
        read_charge(budget, NativeJoinChargeKind::Work, 1).map_err(|_| native_error(budget))?;
        if !reached.contains(&row.checked_application) {
            continue;
        }
        preflight_work(budget, row.source_map_entry.regions.len())?;
        if let Some(value) = &row.value_type {
            storage::preflight_value_type(value, budget, &row.structural_node)
                .map_err(|error| exact_error(error, budget))?;
        }
        let unit_id = match &row.fact {
            EqualityFactWitness::Unit { unit_id, .. }
            | EqualityFactWitness::DimensionDeclaration { unit_id, .. }
            | EqualityFactWitness::DimensionTerm { unit_id, .. }
            | EqualityFactWitness::ConversionEdge { unit_id, .. } => Some(unit_id),
            _ => None,
        };
        if let Some(unit_id) = unit_id {
            storage::preflight_unit(unit_id, budget, &row.structural_node)
                .map_err(|error| exact_error(error, budget))?;
        }
        read_charge(
            budget,
            NativeJoinChargeKind::Storage,
            equality_fact_storage(row),
        )
        .map_err(|_| native_error(budget))?;
        selected.push(row.clone());
    }
    Ok(selected)
}

impl<'a> CheckedMonitorPlanRef<'a> {
    /// Project one complete selected monitor type closure under one finite caller budget.
    ///
    /// All source rows are structural claims over admitted facts. Their opaque Exact
    /// values are carried without deriving replacement identities.
    pub fn project_selected_type_closure(
        &self,
        rows: SelectedMonitorClosureRows<'_>,
        budget: &mut NativeJoinBudget,
    ) -> Result<SelectedMonitorTypeClosure<'a>, SelectedMonitorTypeClosureError> {
        let before = budget.begin();
        let result = (|| {
            let reached = self
                .read_reached_model_facts(
                    ReachedProducerRows {
                        declarations: rows.declarations,
                        fields: rows.fields,
                        tuples: rows.model_tuples,
                        source_units: rows.source_units,
                        enum_declarations: rows.model_enum_declarations,
                        enum_members: rows.model_enum_members,
                        enum_shapes: rows.model_enum_shapes,
                    },
                    budget,
                )
                .map_err(|error| reached_error(error, budget))?;
            budget.set_active_locus(Some(NativeJoinLocus {
                source: NativeJoinLocusSource::MonitorSeed(0),
                form: "selected_clock_output",
            }));
            read_charge(budget, NativeJoinChargeKind::Storage, 1)
                .map_err(|_| native_error(budget))?;
            let mut clocks = Vec::new();
            let clause = self.package.node(self.clause()).expect("admitted clause");
            for (index, occurrence) in clause.occurrences.iter().enumerate() {
                budget.set_active_locus(Some(NativeJoinLocus {
                    source: NativeJoinLocusSource::MonitorSeed(index),
                    form: "selected_claim_occurrence",
                }));
                read_charge(budget, NativeJoinChargeKind::Work, 1)
                    .map_err(|_| native_error(budget))?;
                if occurrence.role != CheckedOccurrenceRole::Claim {
                    continue;
                }
                let clock = self
                    .clock_for_claim(self.clause(), occurrence, budget)
                    .map_err(|error| clock_error(error, budget))?;
                read_charge(budget, NativeJoinChargeKind::Storage, 1)
                    .map_err(|_| native_error(budget))?;
                clocks.push(clock);
            }
            budget.set_active_locus(Some(NativeJoinLocus {
                source: NativeJoinLocusSource::MonitorSeed(0),
                form: "selected_equality_output",
            }));
            read_charge(budget, NativeJoinChargeKind::Storage, 2)
                .map_err(|_| native_error(budget))?;
            let mut equality_ids = BTreeSet::new();
            let mut equality_occurrences = Vec::new();
            for id in &reached.reached {
                let Some(node) = self.package.node(id) else {
                    continue;
                };
                budget.set_locus_form("reached_equality_node");
                read_charge(budget, NativeJoinChargeKind::Work, 1)
                    .map_err(|_| native_error(budget))?;
                if self.package.node_kind(id)
                    != Some(CheckedNodeKind::Expression(crate::ExpressionForm::Binary))
                {
                    continue;
                }
                let Some(CheckedTerm::Application {
                    operator: ApplicationOperator::Binary,
                    operation,
                    ..
                }) = self.package.term(id)
                else {
                    continue;
                };
                if !operation
                    .identity()
                    .is_some_and(|name| equality_operator(name).is_some())
                {
                    continue;
                }
                read_charge(budget, NativeJoinChargeKind::Storage, storage::node(id))
                    .map_err(|_| native_error(budget))?;
                equality_ids.insert(id.clone());
                // A decision belongs to an evaluation site of the application.
                // The clause's claim registration of an interned application is
                // the same computation and carries no decision of its own.
                let mut has_site = false;
                for occurrence in &node.occurrences {
                    budget.set_locus_form("reached_equality_occurrence");
                    read_charge(budget, NativeJoinChargeKind::Work, 1)
                        .map_err(|_| native_error(budget))?;
                    if occurrence.role != CheckedOccurrenceRole::Expression {
                        continue;
                    }
                    read_charge(budget, NativeJoinChargeKind::Storage, 1)
                        .map_err(|_| native_error(budget))?;
                    has_site = true;
                    equality_occurrences.push((id, occurrence));
                }
                if !has_site {
                    read_charge(budget, NativeJoinChargeKind::Storage, storage::node(id))
                        .map_err(|_| native_error(budget))?;
                    return Err(semantic_error(
                        SelectedMonitorTypeClosureErrorCode::Equality(
                            AdmittedEqualityReadErrorCode::UnavailableEqualityDecision,
                        ),
                        Some(Box::new(id.clone())),
                        budget,
                    ));
                }
            }
            let equality_applications = select_rows(
                rows.equality_applications,
                &equality_ids,
                budget,
                "equality_application_selection",
                |row| &row.application,
                equality_application_storage,
                preflight_equality_application,
            )?;
            let equality_facts = select_equality_facts(rows.equality_facts, &equality_ids, budget)?;
            let source_units = select_rows(
                rows.source_units,
                &reached.reached,
                budget,
                "equality_source_unit_selection",
                |row| &row.structural_node,
                storage::runtime_source_unit,
                preflight_source_unit,
            )?;
            let mut equalities = Vec::new();
            for (id, occurrence) in equality_occurrences {
                let decision = self
                    .package
                    .admitted_equality(
                        id,
                        occurrence,
                        &equality_applications,
                        &equality_facts,
                        &source_units,
                        budget,
                    )
                    .map_err(|error| equality_error(error, occurrence, budget))?;
                read_charge(
                    budget,
                    NativeJoinChargeKind::Storage,
                    storage::node(id).saturating_add(2),
                )
                .map_err(|_| native_error(budget))?;
                equalities.push(SelectedFactEquality {
                    id: SelectedFactEqualityId {
                        application: id.clone(),
                        occurrence: occurrence.clone(),
                    },
                    decision,
                });
            }
            let source_records = select_rows(
                rows.source_records,
                &reached.reached,
                budget,
                "source_record_selection",
                |row| &row.structural_node,
                storage::source_record,
                preflight_source_record,
            )?;
            let source_tuples = select_rows(
                rows.source_tuples,
                &reached.reached,
                budget,
                "source_tuple_selection",
                |row| &row.structural_node,
                storage::source_tuple,
                preflight_source_tuple,
            )?;
            let source_unions = select_rows(
                rows.source_unions,
                &reached.reached,
                budget,
                "source_union_selection",
                |row| &row.structural_node,
                storage::source_union,
                preflight_source_union,
            )?;
            let source_enums = select_rows(
                rows.source_enums,
                &reached.reached,
                budget,
                "source_enum_selection",
                |row| &row.declaration.structural_node,
                storage::source_enum,
                preflight_source_enum,
            )?;
            let source_collections = select_source_collections(
                self.package,
                rows.source_collections,
                &reached.reached,
                budget,
            )?;
            let identities = reached
                .identities
                .with_source_record_rows(self.package, source_records, budget)
                .map_err(|error| exact_error(error, budget))?
                .with_source_tuple_rows(self.package, source_tuples, budget)
                .map_err(|error| exact_error(error, budget))?
                .with_source_union_rows(self.package, source_unions, budget)
                .map_err(|error| exact_error(error, budget))?
                .with_source_enum_rows(self.package, source_enums, budget)
                .map_err(|error| exact_error(error, budget))?
                .with_indexed_source_collection_rows(self.package, source_collections, budget)
                .map_err(|error| exact_error(error, budget))?;
            // A parameter's name literal uses QSpec's unbounded Text marker.
            // It is metadata, not the parameter's ordinary value type. A Text
            // node used by an ordinary declaration or value remains required
            // and reaches the native unsupported-type refusal below.
            let mut required_types = BTreeSet::new();
            for id in &reached.reached {
                let Some(node) = self.package.node(id) else {
                    continue;
                };
                budget.set_locus_form("ordinary_type_selection");
                read_charge(budget, NativeJoinChargeKind::Work, 1)
                    .map_err(|_| native_error(budget))?;
                if node.semantic_type != *id {
                    read_charge(budget, NativeJoinChargeKind::Storage, 1)
                        .map_err(|_| native_error(budget))?;
                    required_types.insert(&node.semantic_type);
                }
            }
            for row in identities.source_record_rows() {
                for member in &row.members {
                    read_charge(budget, NativeJoinChargeKind::Work, 1)
                        .map_err(|_| native_error(budget))?;
                    read_charge(budget, NativeJoinChargeKind::Storage, 1)
                        .map_err(|_| native_error(budget))?;
                    required_types.insert(&member.type_node);
                }
            }
            for row in identities.source_tuple_rows() {
                for position in &row.positions {
                    read_charge(budget, NativeJoinChargeKind::Work, 1)
                        .map_err(|_| native_error(budget))?;
                    read_charge(budget, NativeJoinChargeKind::Storage, 1)
                        .map_err(|_| native_error(budget))?;
                    required_types.insert(&position.type_node);
                }
            }
            for row in identities.source_union_rows() {
                for member in &row.members {
                    for node in &member.payload_nodes {
                        read_charge(budget, NativeJoinChargeKind::Work, 1)
                            .map_err(|_| native_error(budget))?;
                        read_charge(budget, NativeJoinChargeKind::Storage, 1)
                            .map_err(|_| native_error(budget))?;
                        required_types.insert(node);
                    }
                }
            }
            for row in identities.source_collection_rows() {
                read_charge(budget, NativeJoinChargeKind::Work, 1)
                    .map_err(|_| native_error(budget))?;
                read_charge(budget, NativeJoinChargeKind::Storage, 1)
                    .map_err(|_| native_error(budget))?;
                required_types.insert(&row.element_node);
            }
            // A bounded collection row projects its unbounded base node as part
            // of the wrapper. The base is projected on its own only when some
            // other reached node is typed by it, and then it needs its own row.
            let mut row_nodes = BTreeSet::new();
            for row in identities.source_collection_rows() {
                read_charge(budget, NativeJoinChargeKind::Work, 1)
                    .map_err(|_| native_error(budget))?;
                read_charge(budget, NativeJoinChargeKind::Storage, 1)
                    .map_err(|_| native_error(budget))?;
                row_nodes.insert(&row.structural_node);
            }
            let mut wrapped_bases = BTreeSet::new();
            for row in identities.source_collection_rows() {
                read_charge(budget, NativeJoinChargeKind::Work, 1)
                    .map_err(|_| native_error(budget))?;
                if row.structural_node != row.base_node && !row_nodes.contains(&row.base_node) {
                    read_charge(budget, NativeJoinChargeKind::Storage, 1)
                        .map_err(|_| native_error(budget))?;
                    wrapped_bases.insert(&row.base_node);
                }
            }
            for id in &reached.reached {
                let Some(node) = self.package.node(id) else {
                    continue;
                };
                read_charge(budget, NativeJoinChargeKind::Work, 1)
                    .map_err(|_| native_error(budget))?;
                if node.semantic_type != *id && !row_nodes.contains(id) {
                    wrapped_bases.remove(&node.semantic_type);
                }
            }
            budget.set_active_locus(Some(NativeJoinLocus {
                source: NativeJoinLocusSource::MonitorSeed(0),
                form: "selected_type_output",
            }));
            read_charge(budget, NativeJoinChargeKind::Storage, 1)
                .map_err(|_| native_error(budget))?;
            let mut exact_types = Vec::new();
            for id in &reached.reached {
                let Some((position, _)) = self.package.node_with_position(id) else {
                    continue;
                };
                budget.set_active_locus(Some(NativeJoinLocus {
                    source: NativeJoinLocusSource::GraphNode(position),
                    form: "selected_exact_type",
                }));
                read_charge(budget, NativeJoinChargeKind::Work, 1)
                    .map_err(|_| native_error(budget))?;
                let is_exact_type = match self.package.node_kind(id) {
                    Some(CheckedNodeKind::ScalarType(
                        ScalarTypeForm::Dimension
                        | ScalarTypeForm::DimensionEquivalence
                        | ScalarTypeForm::Unit
                        | ScalarTypeForm::CompoundUnit,
                    )) => false,
                    Some(CheckedNodeKind::ScalarType(ScalarTypeForm::Text)) => {
                        required_types.contains(id)
                    }
                    Some(CheckedNodeKind::ScalarType(_) | CheckedNodeKind::CompositeType(_)) => {
                        true
                    }
                    Some(CheckedNodeKind::BoundedDomain(BoundedDomainForm::CollectionBounds)) => {
                        row_nodes.contains(id)
                    }
                    _ => false,
                };
                if !is_exact_type || wrapped_bases.contains(id) {
                    continue;
                }
                let projection = self
                    .package
                    .project_exact_type(id, &identities, budget)
                    .map_err(|error| exact_error(error, budget))?;
                read_charge(budget, NativeJoinChargeKind::Storage, 1)
                    .map_err(|_| native_error(budget))?;
                exact_types.push(projection);
            }
            Ok(SelectedMonitorTypeClosure {
                plan: *self,
                types: reached.types,
                model_enums: reached.enums,
                identities,
                exact_types,
                clocks,
                equalities,
                work: budget.consumed().work - before.work,
            })
        })();
        if result.is_ok() {
            budget.finish(before);
        } else {
            budget.abort();
        }
        result
    }
}
