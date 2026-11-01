// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-352-AC-10: join producer-selected model operation members to admitted anchors.

use super::model_members::{member_name, DeclarationForm};
use super::structural::typed_anchor_body;
use super::{CheckedNodeId, CheckedNodeKind, CheckedPackageV2, ModelForm, StateForm};
use std::collections::BTreeMap;

/// One QSL-stage operation-member witness for an admitted model-owned anchor.
///
/// The selected operation identity is a producer fact. This type validates its
/// correspondence to the admitted selected model member, but cannot establish
/// the provenance of a caller-created identity by inspecting that identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedOperationMemberRow {
    /// Full structural identity of the admitted operation anchor.
    pub anchor: CheckedNodeId,
    /// Identity of the selected domain document that declares the operation.
    pub selected_document: Box<str>,
    /// Closed form of the selected model declaration.
    pub declaring_form: ModelForm,
    /// Original IR identity of the declaring model node.
    pub original_model_identity: Box<str>,
    /// Original IR identity of the model operation member.
    pub original_operation_identity: Box<str>,
    /// Operation identity issued by the QSL checking/model selection stage.
    pub selected_operation_identity: Box<str>,
}

/// Stable reason an operation-member witness or read was refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum SelectedOperationMemberErrorCode {
    /// The read names a different admitted package from the validated index.
    #[error("operation-member index belongs to another checked package")]
    ForeignPackage,
    /// The anchor key does not belong to this admitted package.
    #[error("unknown or foreign anchor")]
    UnknownAnchor,
    /// The node does not carry an admitted operation-anchor body.
    #[error("node is not an operation anchor")]
    WrongAnchorForm,
    /// The anchor context has no selected model operation member.
    #[error("anchor has no selected model operation member")]
    UnavailableModelMember,
    /// A row disagrees with an admitted selected model or operation identity.
    #[error("operation-member witness does not match admission")]
    MismatchedWitness,
    /// More than one row names the same admitted anchor.
    #[error("competing operation-member witnesses")]
    ConflictingWitness,
    /// No producer row was supplied for the requested admitted anchor.
    #[error("operation-member witness is unavailable")]
    UnavailableWitness,
    /// The caller's finite work limit was exhausted before a complete index.
    #[error("operation-member witness work limit exhausted")]
    WorkLimit,
}

/// A typed refusal at the first rejected anchor or work charge.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("{code} at {anchor:?}")]
pub struct SelectedOperationMemberError {
    /// Stable refusal code.
    pub code: SelectedOperationMemberErrorCode,
    /// Anchor being joined or read.
    pub anchor: CheckedNodeId,
    /// Work charged before this outcome.
    pub consumed_work: u64,
    /// Work charge that could not be paid; zero for a non-limit refusal.
    pub attempted_work: u64,
}

/// Complete validated QSL-stage rows for one admitted checked package instance.
///
/// A missing row remains missing: this index never derives a selected
/// operation identity from the anchor's display name.
#[derive(Clone, Debug)]
pub struct SelectedOperationMembers<'a> {
    package: &'a CheckedPackageV2,
    rows: BTreeMap<CheckedNodeId, SelectedOperationMemberRow>,
}

impl<'a> SelectedOperationMembers<'a> {
    /// Validate producer rows against the admitted package under a finite work limit.
    ///
    /// Charges one unit for each row and each selected operation inspected. No
    /// partially validated index escapes on refusal or exhaustion. An empty
    /// index is valid and makes every later read return `UnavailableWitness`.
    pub fn from_stage_rows(
        package: &'a CheckedPackageV2,
        rows: impl IntoIterator<Item = SelectedOperationMemberRow>,
        work_limit: u64,
    ) -> Result<Self, SelectedOperationMemberError> {
        let mut selected = Self {
            package,
            rows: BTreeMap::new(),
        };
        let mut consumed = 0_u64;
        for row in rows {
            let anchor = &row.anchor;
            charge(&mut consumed, work_limit, anchor)?;
            if selected.rows.contains_key(anchor) {
                return Err(refusal(
                    SelectedOperationMemberErrorCode::ConflictingWitness,
                    anchor,
                    consumed,
                ));
            }
            if package.node(anchor).is_none() {
                return Err(refusal(
                    SelectedOperationMemberErrorCode::UnknownAnchor,
                    anchor,
                    consumed,
                ));
            }
            if package.node_kind(anchor) != Some(CheckedNodeKind::State(StateForm::OperationAnchor))
            {
                return Err(refusal(
                    SelectedOperationMemberErrorCode::WrongAnchorForm,
                    anchor,
                    consumed,
                ));
            }
            let Some(body) = package.term(anchor).and_then(typed_anchor_body) else {
                return Err(refusal(
                    SelectedOperationMemberErrorCode::WrongAnchorForm,
                    anchor,
                    consumed,
                ));
            };
            let Some(owner) = package.models.owner_by_key(&body.context) else {
                return Err(refusal(
                    SelectedOperationMemberErrorCode::UnavailableWitness,
                    anchor,
                    consumed,
                ));
            };
            let form = match owner.form {
                DeclarationForm::ObjectType => ModelForm::ObjectType,
                DeclarationForm::StateMachine => ModelForm::StateMachine,
                DeclarationForm::Process => ModelForm::Process,
                DeclarationForm::SystemsInterface => ModelForm::SystemsInterface,
                _ => {
                    return Err(refusal(
                        SelectedOperationMemberErrorCode::UnavailableModelMember,
                        anchor,
                        consumed,
                    ));
                }
            };
            if package.node_kind(&body.context) != Some(CheckedNodeKind::Model(form)) {
                return Err(refusal(
                    SelectedOperationMemberErrorCode::UnavailableWitness,
                    anchor,
                    consumed,
                ));
            }
            if row.selected_document.as_ref() != owner.package.identity.as_ref()
                || row.declaring_form != form
                || row.original_model_identity.as_ref() != owner.node
            {
                return Err(refusal(
                    SelectedOperationMemberErrorCode::MismatchedWitness,
                    anchor,
                    consumed,
                ));
            }
            let Some(declaration) = owner.object_type() else {
                return Err(refusal(
                    SelectedOperationMemberErrorCode::UnavailableModelMember,
                    anchor,
                    consumed,
                ));
            };
            let mut matched = false;
            for operation in &declaration.operations {
                charge(&mut consumed, work_limit, anchor)?;
                if operation.identity == row.original_operation_identity
                    && member_name(&operation.identity) == body.operation
                {
                    matched = true;
                    break;
                }
            }
            if !matched {
                return Err(refusal(
                    SelectedOperationMemberErrorCode::MismatchedWitness,
                    anchor,
                    consumed,
                ));
            }
            selected.rows.insert(anchor.clone(), row);
        }
        Ok(selected)
    }

    /// Read the validated producer row for an admitted anchor.
    ///
    /// Returns a typed missing-witness outcome; it never finds an operation by
    /// name or treats a row from a different admission as selected here. A
    /// cloned package needs its own index joined from the producer rows.
    pub fn read(
        &self,
        package: &CheckedPackageV2,
        anchor: &CheckedNodeId,
    ) -> Result<&SelectedOperationMemberRow, SelectedOperationMemberError> {
        if !std::ptr::eq(self.package, package) {
            return Err(refusal(
                SelectedOperationMemberErrorCode::ForeignPackage,
                anchor,
                0,
            ));
        }
        if self.package.node(anchor).is_none() {
            return Err(refusal(
                SelectedOperationMemberErrorCode::UnknownAnchor,
                anchor,
                0,
            ));
        }
        self.rows.get(anchor).ok_or_else(|| {
            refusal(
                SelectedOperationMemberErrorCode::UnavailableWitness,
                anchor,
                0,
            )
        })
    }
}

fn refusal(
    code: SelectedOperationMemberErrorCode,
    anchor: &CheckedNodeId,
    consumed_work: u64,
) -> SelectedOperationMemberError {
    SelectedOperationMemberError {
        code,
        anchor: anchor.clone(),
        consumed_work,
        attempted_work: 0,
    }
}

fn charge(
    consumed: &mut u64,
    limit: u64,
    anchor: &CheckedNodeId,
) -> Result<(), SelectedOperationMemberError> {
    if *consumed >= limit {
        return Err(SelectedOperationMemberError {
            code: SelectedOperationMemberErrorCode::WorkLimit,
            anchor: anchor.clone(),
            consumed_work: *consumed,
            attempted_work: 1,
        });
    }
    *consumed += 1;
    Ok(())
}
