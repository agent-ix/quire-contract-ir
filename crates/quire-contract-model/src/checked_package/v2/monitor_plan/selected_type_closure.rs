// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Atomic selected monitor type closure (FR-357-AC-1..4).

use super::reached_model_read::{ReachedModelError, ReachedProducerRows};
use super::selected_closure_walk::{RuntimeWalkError, SelectedGraphWalkErrorCode};
use super::{
    CheckedMonitorClockRead, CheckedMonitorClockReadError, CheckedMonitorClockReadErrorCode,
    CheckedMonitorPlanRef,
};
use crate::checked_package::v2::native_join_budget::{read_charge, NativeReadDenied};
use crate::checked_package::v2::native_join_storage as storage;
use crate::checked_package::v2::{
    CheckedNodeId, CheckedNodeKind, CheckedOccurrence, CheckedOccurrenceRole, ExactTypeProjection,
    ExactTypeProjectionError, ExactTypeProjectionErrorCode, NativeJoinBudget, NativeJoinChargeKind,
    NativeJoinDeniedCharge, NativeJoinLocus, NativeJoinLocusSource, ScalarTypeForm,
    SelectedExactDeclarationRow, SelectedExactFieldRow, SelectedExactIdentities,
    SelectedModelEnumDeclarationRow, SelectedModelEnumErrorCode, SelectedModelEnumMemberRow,
    SelectedModelEnumShapeRow, SelectedSourceCollectionRow, SelectedSourceEnumRow,
    SelectedSourceRecordRow, SelectedSourceTupleRow, SelectedSourceUnionRow, SelectedSourceUnitRow,
    SelectedTupleDeclarationRow, SelectedTypeDeclarations, SelectedTypeDeclarationsError,
};
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
    /// Runtime declaration projection refused.
    Runtime(ExactTypeProjectionErrorCode),
    /// Selected clause clock correspondence refused.
    Clock(CheckedMonitorClockReadErrorCode),
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
        SelectedMonitorTypeClosureErrorCode::Runtime(error.code),
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

fn select_rows<T: Clone>(
    rows: &[T],
    reached: &BTreeSet<CheckedNodeId>,
    budget: &mut NativeJoinBudget,
    form: &'static str,
    node: impl Fn(&T) -> &CheckedNodeId,
    storage_units: impl Fn(&T) -> u128,
) -> Result<Vec<T>, NativeReadDenied> {
    budget.set_locus_form(form);
    read_charge(budget, NativeJoinChargeKind::Storage, 1)?;
    let mut selected = Vec::new();
    for (index, row) in rows.iter().enumerate() {
        budget.set_active_locus(Some(NativeJoinLocus {
            source: NativeJoinLocusSource::ReachedRow(index),
            form,
        }));
        read_charge(budget, NativeJoinChargeKind::Work, 1)?;
        let id = node(row);
        if reached.contains(id) {
            read_charge(budget, NativeJoinChargeKind::Storage, storage_units(row))?;
            selected.push(row.clone());
        }
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
            let source_records = select_rows(
                rows.source_records,
                &reached.reached,
                budget,
                "source_record_selection",
                |row| &row.structural_node,
                storage::source_record,
            )
            .map_err(|_| native_error(budget))?;
            let source_tuples = select_rows(
                rows.source_tuples,
                &reached.reached,
                budget,
                "source_tuple_selection",
                |row| &row.structural_node,
                storage::source_tuple,
            )
            .map_err(|_| native_error(budget))?;
            let source_unions = select_rows(
                rows.source_unions,
                &reached.reached,
                budget,
                "source_union_selection",
                |row| &row.structural_node,
                storage::source_union,
            )
            .map_err(|_| native_error(budget))?;
            let source_enums = select_rows(
                rows.source_enums,
                &reached.reached,
                budget,
                "source_enum_selection",
                |row| &row.declaration.structural_node,
                storage::source_enum,
            )
            .map_err(|_| native_error(budget))?;
            let source_collections = select_rows(
                rows.source_collections,
                &reached.reached,
                budget,
                "source_collection_selection",
                |row| &row.structural_node,
                storage::source_collection,
            )
            .map_err(|_| native_error(budget))?;
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
                .with_source_collection_rows(self.package, source_collections, budget)
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
                    _ => false,
                };
                if !is_exact_type {
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
