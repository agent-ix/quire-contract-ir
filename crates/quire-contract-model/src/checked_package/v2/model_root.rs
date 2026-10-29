// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Selected model root shape for FR-065.

use super::{CheckedNodeKind, CheckedTerm, ModelForm, RawSemanticNodeV2};
use crate::checked_package::common::{node_pointer, ValidationFailure};
use crate::checked_package::shared::{
    CheckedOccurrenceRole, CheckedPackageRefusalCode, JsonPointer,
};

/// Checks the root's closed graph and occurrence shape before key derivation.
pub(super) fn validate_shape(
    nodes: &[RawSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    terms: &[Option<CheckedTerm>],
) -> Result<(), ValidationFailure> {
    for (position, node) in nodes.iter().enumerate() {
        if kinds.get(position) != Some(&CheckedNodeKind::Model(ModelForm::ModelRoot)) {
            continue;
        }
        let at = node_pointer(position);
        let invalid = |path: JsonPointer| {
            ValidationFailure::refused(CheckedPackageRefusalCode::InvalidSemanticGraph, path)
        };
        if node.semantic_type != node.node_id {
            return Err(invalid(at.key("semantic_type")));
        }
        if node.recursion_group.is_some() {
            return Err(invalid(at.key("recursion_group")));
        }
        if node.nominal_identity_preimage.is_some() {
            return Err(invalid(at.key("nominal_identity_preimage")));
        }
        if !node.dependencies.is_empty() {
            return Err(invalid(at.key("dependencies")));
        }
        if !matches!(terms.get(position).and_then(Option::as_ref),
            Some(CheckedTerm::Aggregate { members }) if members.is_empty())
        {
            return Err(invalid(at.key("body")));
        }
        if node.occurrences.len() != 1 {
            return Err(ValidationFailure::refused(
                CheckedPackageRefusalCode::InvalidSourceMap,
                at.key("occurrences"),
            ));
        }
        let occurrence = &node.occurrences[0];
        if occurrence.role != CheckedOccurrenceRole::Generated {
            return Err(ValidationFailure::refused(
                CheckedPackageRefusalCode::InvalidSourceMap,
                at.key("occurrences").index(0).key("role"),
            ));
        }
        if occurrence.ordinal != 0 {
            return Err(ValidationFailure::refused(
                CheckedPackageRefusalCode::InvalidSourceMap,
                at.key("occurrences").index(0).key("ordinal"),
            ));
        }
    }
    Ok(())
}
