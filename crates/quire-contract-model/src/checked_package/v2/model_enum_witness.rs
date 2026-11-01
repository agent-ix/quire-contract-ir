// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-354: join QSL's model-owned enum identities to admitted selected cases.

use super::model_members::DeclarationForm;
use super::{
    CheckedNodeId, CheckedNodeKind, CheckedNodeOwner, CheckedPackageV2, CheckedTerm, LiteralKind,
    ModelForm, ValueForm,
};
use quire_exact::NodeKey;
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
        let mut selected = Self {
            package,
            declarations: BTreeMap::new(),
            members: BTreeMap::new(),
        };
        let mut consumed = 0_u64;
        let mut original_declarations = BTreeSet::new();
        let mut declaration_keys = BTreeSet::new();
        for row in declarations {
            let node = &row.structural_node;
            charge(&mut consumed, work_limit, node)?;
            if package.node(node).is_none() {
                return Err(refusal(
                    SelectedModelEnumErrorCode::UnknownNode,
                    node,
                    consumed,
                ));
            }
            if package.node_kind(node) != Some(CheckedNodeKind::Model(ModelForm::VariantType)) {
                return Err(refusal(
                    SelectedModelEnumErrorCode::WrongForm,
                    node,
                    consumed,
                ));
            }
            let Some((document, original, form)) = package.models.selected_declaration(node) else {
                return Err(refusal(
                    SelectedModelEnumErrorCode::UnavailableWitness,
                    node,
                    consumed,
                ));
            };
            if form != DeclarationForm::VariantType
                || package.models.variant_family(node) != Some("enum")
            {
                return Err(refusal(
                    SelectedModelEnumErrorCode::WrongForm,
                    node,
                    consumed,
                ));
            }
            let Some(checked) = package.node(node) else {
                return Err(refusal(
                    SelectedModelEnumErrorCode::UnknownNode,
                    node,
                    consumed,
                ));
            };
            if document != row.selected_document.as_ref()
                || original != row.original_identity.as_ref()
                || !matches!(&checked.owner, Some(CheckedNodeOwner::Model { identity, node })
                    if identity.as_ref() == document && node.as_ref() == original)
            {
                return Err(refusal(
                    SelectedModelEnumErrorCode::MismatchedWitness,
                    node,
                    consumed,
                ));
            }
            if selected.declarations.contains_key(node)
                || !original_declarations
                    .insert((row.selected_document.clone(), row.original_identity.clone()))
                || !declaration_keys.insert(row.node_key.clone())
            {
                return Err(refusal(
                    SelectedModelEnumErrorCode::ConflictingWitness,
                    node,
                    consumed,
                ));
            }
            selected.declarations.insert(node.clone(), row);
        }
        let mut original_members = BTreeSet::new();
        let mut member_keys = BTreeSet::new();
        for row in members {
            let node = &row.structural_node;
            charge(&mut consumed, work_limit, node)?;
            let Some(checked) = package.node(node) else {
                return Err(refusal(
                    SelectedModelEnumErrorCode::UnknownNode,
                    node,
                    consumed,
                ));
            };
            if package.node_kind(node) != Some(CheckedNodeKind::Value(ValueForm::EnumValue)) {
                return Err(refusal(
                    SelectedModelEnumErrorCode::WrongForm,
                    node,
                    consumed,
                ));
            }
            let Some(declaration) = selected.declarations.get(&row.declaration_node) else {
                return Err(refusal(
                    SelectedModelEnumErrorCode::UnavailableWitness,
                    node,
                    consumed,
                ));
            };
            if declaration.selected_document != row.selected_document
                || declaration.original_identity != row.enum_original_identity
                || checked.semantic_type != row.declaration_node
                || !checked.dependencies.contains(&row.declaration_node)
            {
                return Err(refusal(
                    SelectedModelEnumErrorCode::MismatchedWitness,
                    node,
                    consumed,
                ));
            }
            let Some(model) = package
                .models
                .selected_models()
                .iter()
                .find(|model| model.identity == row.selected_document)
            else {
                return Err(refusal(
                    SelectedModelEnumErrorCode::UnavailableWitness,
                    node,
                    consumed,
                ));
            };
            let Some(cases) = model.variant_types.get(&row.enum_original_identity) else {
                return Err(refusal(
                    SelectedModelEnumErrorCode::UnavailableWitness,
                    node,
                    consumed,
                ));
            };
            let Some(case) = cases.get(&row.case) else {
                return Err(refusal(
                    SelectedModelEnumErrorCode::MismatchedWitness,
                    node,
                    consumed,
                ));
            };
            charge(&mut consumed, work_limit, node)?;
            if case.payload.is_some()
                || case.identity != row.member_original_identity
                || !matches!(package.term(node), Some(CheckedTerm::Literal {
                    type_id, value_kind: LiteralKind::Enum, value,
                }) if type_id == &row.declaration_node && value.as_str() == Some(row.case.as_ref()))
            {
                return Err(refusal(
                    SelectedModelEnumErrorCode::MismatchedWitness,
                    node,
                    consumed,
                ));
            }
            if selected.members.contains_key(node)
                || !original_members.insert((
                    row.selected_document.clone(),
                    row.member_original_identity.clone(),
                ))
                || !member_keys.insert(row.member_key.clone())
            {
                return Err(refusal(
                    SelectedModelEnumErrorCode::ConflictingWitness,
                    node,
                    consumed,
                ));
            }
            selected.members.insert(node.clone(), row);
        }
        Ok(selected)
    }

    /// Return the validated declaration row for this admitted package instance.
    pub fn read_declaration(
        &self,
        package: &CheckedPackageV2,
        node: &CheckedNodeId,
    ) -> Result<&SelectedModelEnumDeclarationRow, SelectedModelEnumError> {
        self.check_read(package, node)?;
        self.declarations
            .get(node)
            .ok_or_else(|| refusal(SelectedModelEnumErrorCode::UnavailableWitness, node, 0))
    }

    /// Return the validated member row for this admitted package instance.
    pub fn read_member(
        &self,
        package: &CheckedPackageV2,
        node: &CheckedNodeId,
    ) -> Result<&SelectedModelEnumMemberRow, SelectedModelEnumError> {
        self.check_read(package, node)?;
        self.members
            .get(node)
            .ok_or_else(|| refusal(SelectedModelEnumErrorCode::UnavailableWitness, node, 0))
    }

    fn check_read(
        &self,
        package: &CheckedPackageV2,
        node: &CheckedNodeId,
    ) -> Result<(), SelectedModelEnumError> {
        if !std::ptr::eq(self.package, package) {
            return Err(refusal(SelectedModelEnumErrorCode::ForeignPackage, node, 0));
        }
        if self.package.node(node).is_none() {
            return Err(refusal(SelectedModelEnumErrorCode::UnknownNode, node, 0));
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

fn charge(
    consumed: &mut u64,
    limit: u64,
    node: &CheckedNodeId,
) -> Result<(), SelectedModelEnumError> {
    if *consumed >= limit {
        return Err(SelectedModelEnumError {
            code: SelectedModelEnumErrorCode::WorkLimit,
            node: node.clone(),
            consumed_work: *consumed,
            attempted_work: 1,
        });
    }
    *consumed += 1;
    Ok(())
}
