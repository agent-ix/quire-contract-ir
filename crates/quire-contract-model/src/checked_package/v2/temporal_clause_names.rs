// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Declared names of temporal-clause source occurrences.

use super::graph_storage::GraphIndex;
use super::stage_storage::{self, Path, Refusal, Site};
use super::{
    identity::is_identifier, CheckedNodeId, CheckedNodeKind, CheckedOccurrenceRole,
    RawSemanticNodeV2, TemporalForm, WorkMeter,
};
use crate::checked_package::common::ValidationFailure;
use crate::checked_package::shared::{
    CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code,
};
use quire_canonical::{Encode, Error, Sink, Writer};
use serde::{Deserialize, Serialize};

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

fn root() -> Path<'static> {
    Path::root().key("temporal_clause_names")
}
fn site(index: usize, path: Path<'_>) -> Site<'_> {
    Site::new(
        super::PackageReadStage::Graph,
        index,
        "temporal_clause_names",
        path,
    )
}
fn invalid(
    at: Path<'_>,
    cause: Cause,
    declaration: &CheckedNodeId,
    index: usize,
    meter: &mut WorkMeter,
) -> ValidationFailure {
    stage_storage::refuse(
        meter,
        &site(index, at),
        Refusal::new(Code::InvalidPackage, at)
            .because(cause)
            .at_node(declaration),
    )
}

pub(super) fn validate(
    rows: Option<&[TemporalClauseName]>,
    nodes: &[RawSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &GraphIndex<'_>,
    structural_keys: &[std::sync::Arc<CheckedNodeId>],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let mut seen = Vec::new();
    let mut previous: Option<(super::graph_storage::ResolvedKey<'_>, u64)> = None;
    for (row_index, row) in rows.unwrap_or_default().iter().enumerate() {
        let at = root().index(row_index);
        stage_storage::work(meter, &site(row_index, at), 1)?;
        if !is_identifier(&row.name) {
            return Err(stage_storage::refuse(
                meter,
                &site(row_index, at),
                Refusal::new(Code::MalformedWire, at.key("name")),
            ));
        }
        let Some(&position) = index.get(&row.declaration).filter(|&&position| {
            kinds[position] == CheckedNodeKind::Temporal(TemporalForm::TemporalClause)
        }) else {
            return Err(invalid(
                at.key("declaration"),
                Cause::InvalidValue,
                &row.declaration,
                row_index,
                meter,
            ));
        };
        let mut owned = false;
        for occurrence in &nodes[position].occurrences {
            stage_storage::work(meter, &site(row_index, at.key("occurrence")), 1)?;
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
                row_index,
                meter,
            ));
        }
        let key = (&row.declaration, row.occurrence);
        match seen.binary_search(&key) {
            Ok(_) => {
                return Err(invalid(
                    at,
                    Cause::ConflictingDefinition,
                    &row.declaration,
                    row_index,
                    meter,
                ))
            }
            Err(position) => {
                if seen.is_empty() {
                    seen = stage_storage::vec(
                        meter,
                        &site(row_index, at),
                        rows.map_or(0, <[TemporalClauseName]>::len),
                    )?;
                }
                seen.insert(position, key);
            }
        }
        let key = (index.resolved_at(position, structural_keys), row.occurrence);
        if previous.is_some_and(|previous| previous > key) {
            return Err(invalid(
                at,
                Cause::InvalidValue,
                &row.declaration,
                row_index,
                meter,
            ));
        }
        previous = Some(key);
    }
    for (position, node) in nodes.iter().enumerate() {
        if kinds[position] != CheckedNodeKind::Temporal(TemporalForm::TemporalClause) {
            continue;
        }
        for occurrence in &node.occurrences {
            stage_storage::work(meter, &site(position, root()), 1)?;
            if occurrence.role == CheckedOccurrenceRole::Claim
                && seen
                    .binary_search(&(&node.node_id, occurrence.ordinal))
                    .is_err()
            {
                return Err(stage_storage::refuse(
                    meter,
                    &site(position, root()),
                    Refusal::new(Code::InvalidPackage, root()).because(Cause::InvalidValue),
                ));
            }
        }
    }
    Ok(())
}
