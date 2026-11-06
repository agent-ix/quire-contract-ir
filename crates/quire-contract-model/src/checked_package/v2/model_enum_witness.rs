// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-354: join QSL's model-owned enum identities to admitted selected cases.

use super::model_members::DeclarationForm;
use super::native_join_budget::{read_charge, read_node_storage, read_node_work, NativeReadDenied};
use super::native_join_storage as storage;
use super::{
    CheckedNodeId, CheckedNodeKind, CheckedNodeOwner, CheckedPackageV2, CheckedTerm, LiteralKind,
    ModelForm, ValueForm,
};
use super::{NativeJoinBudget, NativeJoinChargeKind, NativeJoinLocus, NativeJoinLocusSource};
use quire_exact::{EnumShape, NodeKey, VariantId};
use std::collections::{BTreeMap, BTreeSet};

/// One model-owned enum declaration and the Exact key issued by QSL.
///
/// The row carries producer evidence; its constructor does not grant admission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedModelEnumDeclarationRow {
    /// The admitted `model/variant_type` structural node.
    pub structural_node: CheckedNodeId,
    /// The selected domain model document.
    pub selected_document: Box<str>,
    /// The declaration's original IR identity in that document.
    pub original_identity: Box<str>,
    /// The Exact identity issued by the QSL checking stage.
    pub node_key: NodeKey,
}

impl SelectedModelEnumDeclarationRow {
    /// Carry the checking stage's complete enum declaration claim.
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

/// One model-owned enum member and the Exact key issued by QSL.
///
/// The original member identity is selected document data, distinct from the
/// case label and the opaque Exact key.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedModelEnumMemberRow {
    /// The admitted `value/enum_value` structural node.
    pub structural_node: CheckedNodeId,
    /// The admitted declaring `model/variant_type` structural node.
    pub declaration_node: CheckedNodeId,
    /// The selected domain model document.
    pub selected_document: Box<str>,
    /// The enum declaration's original IR identity.
    pub enum_original_identity: Box<str>,
    /// The member's original IR identity.
    pub member_original_identity: Box<str>,
    /// The selected semantic case label.
    pub case: Box<str>,
    /// The Exact member identity issued by QSL.
    pub member_key: NodeKey,
}

/// The checking stage's complete shape of one selected model enum.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedModelEnumShapeRow {
    /// Final declaration node.
    pub structural_node: CheckedNodeId,
    /// QSL-issued declaration key.
    pub node_key: NodeKey,
    /// Selected model document identity.
    pub selected_document: Box<str>,
    /// Original model declaration identity.
    pub original_identity: Box<str>,
    /// QSL-issued ordered flag and canonical variants.
    pub shape: EnumShape,
    /// Complete cases in shape order.
    pub cases: Vec<SelectedModelEnumShapeCaseRow>,
}

/// One checked case in a model enum shape.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedModelEnumShapeCaseRow {
    /// Case label.
    pub case: Box<str>,
    /// Final case node.
    pub structural_node: CheckedNodeId,
    /// QSL-issued member key.
    pub member_key: NodeKey,
    /// Original selected model member identity.
    pub member_original_identity: Box<str>,
    /// QSL-issued variant at this case's canonical rank.
    pub variant: VariantId,
}

impl SelectedModelEnumMemberRow {
    /// Carry the checking stage's complete enum member claim.
    pub fn new(
        structural_node: CheckedNodeId,
        declaration_node: CheckedNodeId,
        selected_document: impl Into<Box<str>>,
        enum_original_identity: impl Into<Box<str>>,
        member_original_identity: impl Into<Box<str>>,
        case: impl Into<Box<str>>,
        member_key: NodeKey,
    ) -> Self {
        Self {
            structural_node,
            declaration_node,
            selected_document: selected_document.into(),
            enum_original_identity: enum_original_identity.into(),
            member_original_identity: member_original_identity.into(),
            case: case.into(),
            member_key,
        }
    }
}

/// Stable reason a model enum stage row cannot join an admitted package.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum SelectedModelEnumErrorCode {
    /// The validated index belongs to another checked package instance.
    #[error("model enum index belongs to another checked package")]
    ForeignPackage,
    /// The selected structural node is absent.
    #[error("unknown enum structural node")]
    UnknownNode,
    /// The structural node has another admitted form or family.
    #[error("wrong enum structural form")]
    WrongForm,
    /// An admitted declaration or member has no producer row.
    #[error("model enum witness is unavailable")]
    UnavailableWitness,
    /// A row disagrees with the selected model or admitted node.
    #[error("model enum witness does not match admission")]
    MismatchedWitness,
    /// Two rows claim one structural, original, or Exact identity.
    #[error("competing model enum witnesses")]
    ConflictingWitness,
    /// The finite work allowance cannot pay the next charge.
    #[error("model enum witness work limit exhausted")]
    WorkLimit,
}

/// A typed refusal at the first rejected structural node or work charge.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("{code} at {node:?}")]
pub struct SelectedModelEnumError {
    /// Stable refusal code.
    pub code: SelectedModelEnumErrorCode,
    /// Structural node being joined or read.
    pub node: CheckedNodeId,
    /// Work charged before this outcome.
    pub consumed_work: u64,
    /// Work charge that could not be paid; zero for other refusals.
    pub attempted_work: u64,
}

/// Validated model enum stage rows for one admitted checked package instance.
///
/// A caller-created Exact key remains a producer claim. This index verifies
/// correspondence to selected model and checked node facts, not key provenance.
#[derive(Clone, Debug)]
pub struct SelectedModelEnums<'a> {
    package: &'a CheckedPackageV2,
    declarations: BTreeMap<CheckedNodeId, SelectedModelEnumDeclarationRow>,
    members: BTreeMap<CheckedNodeId, SelectedModelEnumMemberRow>,
    joined_work: u64,
}

#[derive(Debug)]
pub(super) enum RuntimeEnumError {
    Semantic(SelectedModelEnumError),
    Native(NativeReadDenied),
}
impl From<NativeReadDenied> for RuntimeEnumError {
    fn from(error: NativeReadDenied) -> Self {
        Self::Native(error)
    }
}
impl RuntimeEnumError {
    fn legacy(self) -> SelectedModelEnumError {
        match self {
            Self::Semantic(error) => error,
            Self::Native(_) => unreachable!("the numeric enum reader has no native allowance"),
        }
    }
}
struct EnumMeter<'b> {
    native: Option<&'b mut NativeJoinBudget>,
    consumed: u64,
    limit: u64,
}
impl EnumMeter<'_> {
    fn row(&mut self, index: usize, form: &'static str) {
        if let Some(budget) = &mut self.native {
            budget.set_active_locus(Some(NativeJoinLocus {
                source: NativeJoinLocusSource::ReachedRow(index),
                form,
            }));
        }
    }
    fn storage(&mut self, amount: u128) -> Result<(), RuntimeEnumError> {
        if let Some(budget) = &mut self.native {
            read_charge(budget, NativeJoinChargeKind::Storage, amount)?;
        }
        Ok(())
    }
    fn inspect(&mut self, node: &CheckedNodeId) -> Result<(), RuntimeEnumError> {
        if let Some(budget) = &mut self.native {
            read_node_work(budget, node)?;
        }
        Ok(())
    }
    fn text(&mut self, text: &str) -> Result<(), RuntimeEnumError> {
        if let Some(budget) = &mut self.native {
            read_charge(budget, NativeJoinChargeKind::Work, text.len() as u128)?;
        }
        Ok(())
    }
    fn charge(&mut self, node: &CheckedNodeId) -> Result<(), RuntimeEnumError> {
        self.inspect(node)?;
        if self.consumed >= self.limit {
            let mut error =
                self.refusal(SelectedModelEnumErrorCode::WorkLimit, node, self.consumed);
            if let RuntimeEnumError::Semantic(ref mut error) = error {
                error.attempted_work = 1;
            }
            return Err(error);
        }
        self.consumed += 1;
        Ok(())
    }
    fn refusal(
        &mut self,
        code: SelectedModelEnumErrorCode,
        node: &CheckedNodeId,
        consumed: u64,
    ) -> RuntimeEnumError {
        if let Some(budget) = &mut self.native {
            if let Err(error) = read_node_storage(budget, node) {
                return error.into();
            }
        }
        RuntimeEnumError::Semantic(refusal(code, node, consumed))
    }
}

impl<'a> SelectedModelEnums<'a> {
    /// Join producer rows to an admitted package under a finite work limit.
    ///
    /// A failure returns no index. Empty input is valid and later reads refuse
    /// missing witnesses. Each row and selected member lookup charges work.
    pub fn from_stage_rows(
        package: &'a CheckedPackageV2,
        declarations: impl IntoIterator<Item = SelectedModelEnumDeclarationRow>,
        members: impl IntoIterator<Item = SelectedModelEnumMemberRow>,
        work_limit: u64,
    ) -> Result<Self, SelectedModelEnumError> {
        Self::from_stage_rows_inner(
            package,
            declarations,
            members,
            &mut EnumMeter {
                native: None,
                consumed: 0,
                limit: work_limit,
            },
        )
        .map_err(RuntimeEnumError::legacy)
    }

    pub(super) fn from_stage_rows_runtime(
        package: &'a CheckedPackageV2,
        declarations: impl IntoIterator<Item = SelectedModelEnumDeclarationRow>,
        members: impl IntoIterator<Item = SelectedModelEnumMemberRow>,
        budget: &mut NativeJoinBudget,
    ) -> Result<Self, RuntimeEnumError> {
        let previous_locus = budget.active_locus();
        budget.set_active_locus(Some(NativeJoinLocus {
            source: NativeJoinLocusSource::Package,
            form: "model_enum_join_storage",
        }));
        let result = Self::from_stage_rows_inner(
            package,
            declarations,
            members,
            &mut EnumMeter {
                native: Some(budget),
                consumed: 0,
                limit: u64::MAX,
            },
        );
        budget.set_active_locus(previous_locus);
        result
    }

    fn from_stage_rows_inner(
        package: &'a CheckedPackageV2,
        declarations: impl IntoIterator<Item = SelectedModelEnumDeclarationRow>,
        members: impl IntoIterator<Item = SelectedModelEnumMemberRow>,
        meter: &mut EnumMeter<'_>,
    ) -> Result<Self, RuntimeEnumError> {
        meter.storage(5)?;
        let mut selected = Self {
            package,
            declarations: BTreeMap::new(),
            members: BTreeMap::new(),
            joined_work: 0,
        };
        let mut original_declarations = BTreeSet::new();
        let mut exact_keys = BTreeSet::new();
        for (index, row) in declarations.into_iter().enumerate() {
            meter.row(index, "model_enum_declaration");
            let node = &row.structural_node;
            meter.charge(node)?;
            meter.text(&row.selected_document)?;
            meter.text(&row.original_identity)?;
            if package.node(node).is_none() {
                return Err(meter.refusal(
                    SelectedModelEnumErrorCode::UnknownNode,
                    node,
                    meter.consumed,
                ));
            }
            if package.node_kind(node) != Some(CheckedNodeKind::Model(ModelForm::VariantType)) {
                return Err(meter.refusal(
                    SelectedModelEnumErrorCode::WrongForm,
                    node,
                    meter.consumed,
                ));
            }
            let Some((document, original, form)) = package.models.selected_declaration(node) else {
                return Err(meter.refusal(
                    SelectedModelEnumErrorCode::UnavailableWitness,
                    node,
                    meter.consumed,
                ));
            };
            if form != DeclarationForm::VariantType
                || package.models.variant_family(node) != Some("enum")
            {
                return Err(meter.refusal(
                    SelectedModelEnumErrorCode::WrongForm,
                    node,
                    meter.consumed,
                ));
            }
            let Some(checked) = package.node(node) else {
                return Err(meter.refusal(
                    SelectedModelEnumErrorCode::UnknownNode,
                    node,
                    meter.consumed,
                ));
            };
            if document != row.selected_document.as_ref()
                || original != row.original_identity.as_ref()
                || !matches!(&checked.owner, Some(CheckedNodeOwner::Model { identity, node })
                    if identity.as_ref() == document && node.as_ref() == original)
            {
                return Err(meter.refusal(
                    SelectedModelEnumErrorCode::MismatchedWitness,
                    node,
                    meter.consumed,
                ));
            }
            meter.text(&row.selected_document)?;
            meter.text(&row.original_identity)?;
            meter.storage(
                storage::text_parts(&[&row.selected_document])
                    .saturating_add(storage::text_parts(&[&row.original_identity]))
                    .saturating_add(storage::node(node))
                    .saturating_add(4),
            )?;
            if selected.declarations.contains_key(node)
                || !original_declarations
                    .insert((row.selected_document.clone(), row.original_identity.clone()))
                || !exact_keys.insert(row.node_key.clone())
            {
                return Err(meter.refusal(
                    SelectedModelEnumErrorCode::ConflictingWitness,
                    node,
                    meter.consumed,
                ));
            }
            selected.declarations.insert(node.clone(), row);
        }
        let mut original_members = BTreeSet::new();
        for (index, row) in members.into_iter().enumerate() {
            meter.row(index, "model_enum_member");
            let node = &row.structural_node;
            meter.charge(node)?;
            meter.inspect(&row.declaration_node)?;
            meter.text(&row.selected_document)?;
            meter.text(&row.enum_original_identity)?;
            meter.text(&row.member_original_identity)?;
            meter.text(&row.case)?;
            let Some(checked) = package.node(node) else {
                return Err(meter.refusal(
                    SelectedModelEnumErrorCode::UnknownNode,
                    node,
                    meter.consumed,
                ));
            };
            if package.node_kind(node) != Some(CheckedNodeKind::Value(ValueForm::EnumValue)) {
                return Err(meter.refusal(
                    SelectedModelEnumErrorCode::WrongForm,
                    node,
                    meter.consumed,
                ));
            }
            meter.inspect(&row.declaration_node)?;
            let Some(declaration) = selected.declarations.get(&row.declaration_node) else {
                return Err(meter.refusal(
                    SelectedModelEnumErrorCode::UnavailableWitness,
                    node,
                    meter.consumed,
                ));
            };
            for dependency in &checked.dependencies {
                meter.inspect(dependency)?;
            }
            if declaration.selected_document != row.selected_document
                || declaration.original_identity != row.enum_original_identity
                || checked.semantic_type != row.declaration_node
                || !checked.dependencies.contains(&row.declaration_node)
            {
                return Err(meter.refusal(
                    SelectedModelEnumErrorCode::MismatchedWitness,
                    node,
                    meter.consumed,
                ));
            }
            meter.text(&row.case)?;
            let Some(case) = package
                .models
                .selected_variant_case(&row.declaration_node, &row.case)
            else {
                return Err(meter.refusal(
                    SelectedModelEnumErrorCode::MismatchedWitness,
                    node,
                    meter.consumed,
                ));
            };
            meter.charge(node)?;
            if case.payload.is_some()
                || case.identity != row.member_original_identity
                || !matches!(package.term(node), Some(CheckedTerm::Literal {
                    type_id, value_kind: LiteralKind::Enum, value,
                }) if type_id == &row.declaration_node && value.as_str() == Some(row.case.as_ref()))
            {
                return Err(meter.refusal(
                    SelectedModelEnumErrorCode::MismatchedWitness,
                    node,
                    meter.consumed,
                ));
            }
            meter.text(&row.selected_document)?;
            meter.text(&row.member_original_identity)?;
            meter.storage(
                storage::text_parts(&[&row.selected_document])
                    .saturating_add(storage::text_parts(&[&row.member_original_identity]))
                    .saturating_add(storage::node(node))
                    .saturating_add(4),
            )?;
            if selected.members.contains_key(node)
                || !original_members.insert((
                    row.selected_document.clone(),
                    row.member_original_identity.clone(),
                ))
                || !exact_keys.insert(row.member_key.clone())
            {
                return Err(meter.refusal(
                    SelectedModelEnumErrorCode::ConflictingWitness,
                    node,
                    meter.consumed,
                ));
            }
            selected.members.insert(node.clone(), row);
        }
        selected.joined_work = meter.consumed;
        Ok(selected)
    }

    /// Work spent joining the selected producer identity rows.
    pub fn joined_work(&self) -> u64 {
        self.joined_work
    }

    /// Return the validated declaration row for this admitted package instance.
    pub fn read_declaration(
        &self,
        package: &CheckedPackageV2,
        node: &CheckedNodeId,
    ) -> Result<&SelectedModelEnumDeclarationRow, SelectedModelEnumError> {
        self.read_declaration_inner(
            package,
            node,
            &mut EnumMeter {
                native: None,
                consumed: 0,
                limit: u64::MAX,
            },
        )
        .map_err(RuntimeEnumError::legacy)
    }

    fn read_declaration_inner(
        &self,
        package: &CheckedPackageV2,
        node: &CheckedNodeId,
        meter: &mut EnumMeter<'_>,
    ) -> Result<&SelectedModelEnumDeclarationRow, RuntimeEnumError> {
        self.check_read(package, node, meter)?;
        self.declarations
            .get(node)
            .ok_or_else(|| meter.refusal(SelectedModelEnumErrorCode::UnavailableWitness, node, 0))
    }

    /// Return the validated member row for this admitted package instance.
    pub fn read_member(
        &self,
        package: &CheckedPackageV2,
        node: &CheckedNodeId,
    ) -> Result<&SelectedModelEnumMemberRow, SelectedModelEnumError> {
        self.read_member_inner(
            package,
            node,
            &mut EnumMeter {
                native: None,
                consumed: 0,
                limit: u64::MAX,
            },
        )
        .map_err(RuntimeEnumError::legacy)
    }

    fn read_member_inner(
        &self,
        package: &CheckedPackageV2,
        node: &CheckedNodeId,
        meter: &mut EnumMeter<'_>,
    ) -> Result<&SelectedModelEnumMemberRow, RuntimeEnumError> {
        self.check_read(package, node, meter)?;
        self.members
            .get(node)
            .ok_or_else(|| meter.refusal(SelectedModelEnumErrorCode::UnavailableWitness, node, 0))
    }

    /// Join one QSL-issued shape to the already admitted declaration and case identities.
    pub fn read_shape(
        &self,
        package: &CheckedPackageV2,
        row: &SelectedModelEnumShapeRow,
        limit: u64,
    ) -> Result<u64, SelectedModelEnumError> {
        self.read_shape_inner(
            package,
            row,
            &mut EnumMeter {
                native: None,
                consumed: 0,
                limit,
            },
        )
        .map_err(RuntimeEnumError::legacy)
    }
    pub(super) fn read_shape_runtime(
        &self,
        package: &CheckedPackageV2,
        row: &SelectedModelEnumShapeRow,
        budget: &mut NativeJoinBudget,
    ) -> Result<u64, RuntimeEnumError> {
        let previous_locus = budget.active_locus();
        let result = self.read_shape_inner(
            package,
            row,
            &mut EnumMeter {
                native: Some(budget),
                consumed: 0,
                limit: u64::MAX,
            },
        );
        budget.set_active_locus(previous_locus);
        result
    }

    fn read_shape_inner(
        &self,
        package: &CheckedPackageV2,
        row: &SelectedModelEnumShapeRow,
        meter: &mut EnumMeter<'_>,
    ) -> Result<u64, RuntimeEnumError> {
        let node = &row.structural_node;
        meter.text(&row.selected_document)?;
        meter.text(&row.original_identity)?;
        let declaration = self.read_declaration_inner(package, node, meter)?;
        meter.charge(node)?;
        if declaration.node_key != row.node_key
            || declaration.selected_document != row.selected_document
            || declaration.original_identity != row.original_identity
        {
            return Err(meter.refusal(
                SelectedModelEnumErrorCode::MismatchedWitness,
                node,
                meter.consumed,
            ));
        }
        for variant in row.shape.variants() {
            meter.text(variant.case())?;
        }
        if row.shape.variants().count() != row.cases.len() {
            return Err(meter.refusal(
                SelectedModelEnumErrorCode::MismatchedWitness,
                node,
                meter.consumed,
            ));
        }
        meter.storage(1)?;
        let mut cases = BTreeSet::new();
        for (index, (case, variant)) in row.cases.iter().zip(row.shape.variants()).enumerate() {
            meter.row(index, "model_enum_shape_case");
            meter.charge(&case.structural_node)?;
            meter.text(&case.member_original_identity)?;
            meter.text(case.variant.case())?;
            let member = self.read_member_inner(package, &case.structural_node, meter)?;
            meter.text(&case.case)?;
            meter.storage(1)?;
            if member.declaration_node != *node
                || member.case != case.case
                || member.member_key != case.member_key
                || member.member_original_identity != case.member_original_identity
                || &case.variant != variant
                || !cases.insert(&case.case)
            {
                return Err(meter.refusal(
                    SelectedModelEnumErrorCode::MismatchedWitness,
                    &case.structural_node,
                    meter.consumed,
                ));
            }
        }
        let mut member_count = 0;
        for (index, member) in self.members.values().enumerate() {
            meter.row(index, "model_enum_shape_member_count");
            meter.charge(&member.structural_node)?;
            meter.inspect(&member.declaration_node)?;
            if member.declaration_node == *node {
                member_count += 1;
            }
        }
        if member_count != row.cases.len() {
            return Err(meter.refusal(
                SelectedModelEnumErrorCode::MismatchedWitness,
                node,
                meter.consumed,
            ));
        }
        Ok(meter.consumed)
    }

    fn check_read(
        &self,
        package: &CheckedPackageV2,
        node: &CheckedNodeId,
        meter: &mut EnumMeter<'_>,
    ) -> Result<(), RuntimeEnumError> {
        meter.inspect(node)?;
        if !std::ptr::eq(self.package, package) {
            return Err(meter.refusal(SelectedModelEnumErrorCode::ForeignPackage, node, 0));
        }
        if self.package.node(node).is_none() {
            return Err(meter.refusal(SelectedModelEnumErrorCode::UnknownNode, node, 0));
        }
        Ok(())
    }
}

fn refusal(
    code: SelectedModelEnumErrorCode,
    node: &CheckedNodeId,
    consumed_work: u64,
) -> SelectedModelEnumError {
    SelectedModelEnumError {
        code,
        node: node.clone(),
        consumed_work,
        attempted_work: 0,
    }
}
