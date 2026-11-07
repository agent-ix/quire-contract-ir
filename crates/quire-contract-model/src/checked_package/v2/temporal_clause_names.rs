// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Declared names of temporal-clause source occurrences.

use super::graph_storage::GraphIndex;
use super::{
    identity::is_identifier, CheckedNodeId, CheckedNodeKind, CheckedOccurrenceRole,
    RawSemanticNodeV2, TemporalForm, WorkMeter,
};
use crate::checked_package::common::ValidationFailure;
use crate::checked_package::shared::{
    CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code, JsonPointer,
};
use quire_canonical::{Encode, Error, Sink, Writer};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// One declared name of a temporal-clause claim occurrence.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemporalClauseName {
    /// Local structural identity of the clause node.
    pub declaration: CheckedNodeId,
    /// Ordinal of the named claim occurrence.
    pub occurrence: u64,
    /// Exact source identifier, independent of the clause node identity.
    pub name: Box<str>,
}

impl Encode for TemporalClauseName {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        super::encode::encoded(writer, "declaration", &self.declaration)?;
        super::encode::member(writer, "occurrence", &self.occurrence)?;
        super::encode::member(writer, "name", self.name.as_ref())?;
        writer.end_object()
    }
}

fn root() -> JsonPointer {
    JsonPointer::root().key("temporal_clause_names")
}

fn invalid(at: JsonPointer, cause: Cause, declaration: &CheckedNodeId) -> ValidationFailure {
    ValidationFailure::refused_at(Code::InvalidPackage, at, Some(cause), declaration.clone())
}

pub(super) fn validate(
    rows: Option<&[TemporalClauseName]>,
    nodes: &[RawSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &GraphIndex<'_>,
    structural_keys: &[std::sync::Arc<CheckedNodeId>],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let mut seen = BTreeSet::new();
    let mut previous: Option<(&CheckedNodeId, u64)> = None;
    for (row_index, row) in rows.unwrap_or_default().iter().enumerate() {
        let at = root().index(row_index);
        meter.charge(1, || at.clone())?;
        if !is_identifier(&row.name) {
            return Err(ValidationFailure::refused(
                Code::MalformedWire,
                at.key("name"),
            ));
        }
        let Some(&position) = index.get(&row.declaration).filter(|&&position| {
            kinds[position] == CheckedNodeKind::Temporal(TemporalForm::TemporalClause)
        }) else {
            return Err(invalid(
                at.key("declaration"),
                Cause::InvalidValue,
                &row.declaration,
            ));
        };
        let mut owned = false;
        for occurrence in &nodes[position].occurrences {
            meter.charge(1, || at.clone().key("occurrence"))?;
            if occurrence.role == CheckedOccurrenceRole::Claim
                && occurrence.ordinal == row.occurrence
            {
                owned = true;
                break;
            }
        }
        if !owned {
            return Err(invalid(
                at.key("occurrence"),
                Cause::InvalidValue,
                &row.declaration,
            ));
        }
        if !seen.insert((&row.declaration, row.occurrence)) {
            return Err(invalid(at, Cause::ConflictingDefinition, &row.declaration));
        }
        let key = (structural_keys[position].as_ref(), row.occurrence);
        if previous.is_some_and(|previous| previous > key) {
            return Err(invalid(at, Cause::InvalidValue, &row.declaration));
        }
        previous = Some(key);
    }
    for (position, node) in nodes.iter().enumerate() {
        if kinds[position] != CheckedNodeKind::Temporal(TemporalForm::TemporalClause) {
            continue;
        }
        for occurrence in &node.occurrences {
            meter.charge(1, root)?;
            if occurrence.role == CheckedOccurrenceRole::Claim
                && !seen.contains(&(&node.node_id, occurrence.ordinal))
            {
                return Err(ValidationFailure::refused_because(
                    Code::InvalidPackage,
                    root(),
                    Cause::InvalidValue,
                ));
            }
        }
    }
    Ok(())
}
