// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Selected model operation effects carried by the checked package (TC-977).

use super::{CheckedFrameModification, CheckedNodeId, CheckedScopedDeletion};
use quire_canonical::{Encode, Error, FixedShape, Sink, Writer};
use serde::{Deserialize, Deserializer, Serialize};

/// The four complete grants of one selected model operation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedOperationEffect {
    /// Field and relationship grants in canonical order.
    pub modifies: Vec<CheckedFrameModification>,
    /// Object types whose instances the operation may create.
    pub creates: Vec<CheckedNodeId>,
    /// Object types whose instances the operation may delete.
    pub deletes: Vec<CheckedNodeId>,
    /// Receiver and parameter scoped deletion grants.
    pub deletes_scoped: Vec<CheckedScopedDeletion>,
}

/// The sole admitted source of a model operation's declared effect.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(rename_all = "snake_case")]
pub enum CheckedOperationEffectOrigin {
    /// The selected document's operation frame.
    ModelDocumentOperationFrame,
}

/// Source member identity and origin supplied with an effect row.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(deny_unknown_fields)]
pub struct CheckedOperationEffectProvenance {
    /// Fixed selected-document origin.
    pub origin: CheckedOperationEffectOrigin,
    /// Original identity of the operation member.
    pub member: Box<str>,
}

/// A producer-constructible operation effect row; admission validates its claims.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedOperationEffectRow {
    /// Selected document identity.
    pub document: Box<str>,
    /// Structural ID of the declaring model object, process or state machine.
    pub declaring: CheckedNodeId,
    /// Original operation name on that declaring node.
    pub operation: Box<str>,
    /// Complete frame lift, including intentionally empty effects.
    pub effect: CheckedOperationEffect,
    /// Selected member identity and fixed source origin.
    pub provenance: CheckedOperationEffectProvenance,
}

/// A borrowed row from a fully admitted checked package.
#[derive(Clone, Copy, Debug)]
pub struct CheckedOperationEffectRef<'a> {
    row: &'a CheckedOperationEffectRow,
}

impl<'a> CheckedOperationEffectRef<'a> {
    pub(super) fn admitted(row: &'a CheckedOperationEffectRow) -> Self {
        Self { row }
    }

    /// Selected document identity.
    pub fn document(self) -> &'a str {
        &self.row.document
    }

    /// Structural declaring node.
    pub fn declaring(self) -> &'a CheckedNodeId {
        &self.row.declaring
    }

    /// Original operation name.
    pub fn operation(self) -> &'a str {
        &self.row.operation
    }

    /// All four admitted frame grants in their canonical order.
    pub fn effect(self) -> &'a CheckedOperationEffect {
        &self.row.effect
    }

    /// Fixed origin and original member identity.
    pub fn provenance(self) -> &'a CheckedOperationEffectProvenance {
        &self.row.provenance
    }
}

// A defaulted optional envelope field must reject explicit JSON null. Deserializing
// the present value as Vec rather than Option preserves that distinction.
pub(super) fn present_rows<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(deserializer).map(Some)
}

impl Encode for CheckedOperationEffect {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        super::encode::elements(writer, "modifies", &self.modifies)?;
        super::encode::elements(writer, "creates", &self.creates)?;
        super::encode::elements(writer, "deletes", &self.deletes)?;
        super::encode::member(writer, "deletes_scoped", &self.deletes_scoped)?;
        writer.end_object()
    }
}

impl Encode for CheckedOperationEffectRow {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        super::encode::member(writer, "document", self.document.as_ref())?;
        super::encode::encoded(writer, "declaring", &self.declaring)?;
        super::encode::member(writer, "operation", self.operation.as_ref())?;
        super::encode::encoded(writer, "effect", &self.effect)?;
        super::encode::member(writer, "provenance", &self.provenance)?;
        writer.end_object()
    }
}
