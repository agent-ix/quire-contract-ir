// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Authored operation bodies carried by the checked package (TC-987).
//!
//! Implements: FR-322 "Operation-body carrier", FR-322-AC-397.

use super::identity::is_identifier;
use super::model_fields::RetainedModels;
use super::native_join_budget::PackageReadStage;
use super::operation_effects::{
    first_repeated, has_uncovered_operation, join_member, key_of, sorted_keys, Key,
};
use super::stage_storage::{self, Path, Refusal, Site};
use super::{CheckedJson, CheckedSemanticNodeV2, RawCheckedPackageWireV2};
use super::{CheckedNodeId, CheckedNodeKind, CheckedNodeOwner, CheckedTerm, WorkMeter};
use crate::checked_package::common::ValidationFailure;
use crate::checked_package::shared::{
    CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code,
};
use quire_canonical::{Encode, Error, Sink, Writer};
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

/// The authored body of one model operation member, or its stated absence.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OperationBody {
    /// The checked body function node of an operation that has a body.
    Function {
        /// The package `function` node holding the body.
        node: CheckedNodeId,
    },
    /// An operation with no authored body.
    NoBody,
}

impl<'de> Deserialize<'de> for OperationBody {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "snake_case")]
        enum Kind {
            Function,
            NoBody,
        }
        // An internally tagged unit variant ignores a member it does not name,
        // so the closed shape is checked here.
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            kind: Kind,
            #[serde(default, deserialize_with = "present")]
            node: Option<CheckedNodeId>,
        }
        fn present<'de, D: Deserializer<'de>>(
            deserializer: D,
        ) -> Result<Option<CheckedNodeId>, D::Error> {
            CheckedNodeId::deserialize(deserializer).map(Some)
        }
        match Wire::deserialize(deserializer)? {
            Wire {
                kind: Kind::Function,
                node: Some(node),
            } => Ok(Self::Function { node }),
            Wire {
                kind: Kind::NoBody,
                node: None,
            } => Ok(Self::NoBody),
            Wire {
                kind: Kind::Function,
                node: None,
            } => Err(D::Error::missing_field("node")),
            Wire {
                kind: Kind::NoBody,
                node: Some(_),
            } => Err(D::Error::unknown_field("node", &["kind"])),
        }
    }
}

/// A producer-constructible operation body row; admission validates its claims.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationBodyRow {
    /// Selected document identity.
    pub document: Box<str>,
    /// Structural ID of the declaring model object, process or state machine.
    pub declaring: CheckedNodeId,
    /// Original operation name on that declaring node.
    pub operation: Box<str>,
    /// The body function node, or `no_body`; never omitted.
    pub body: OperationBody,
}

impl Encode for OperationBody {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        match self {
            Self::Function { node } => {
                super::encode::member(writer, "kind", "function")?;
                super::encode::encoded(writer, "node", node)?;
            }
            Self::NoBody => super::encode::member(writer, "kind", "no_body")?,
        }
        writer.end_object()
    }
}

impl Encode for OperationBodyRow {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        super::encode::member(writer, "document", self.document.as_ref())?;
        super::encode::encoded(writer, "declaring", &self.declaring)?;
        super::encode::member(writer, "operation", self.operation.as_ref())?;
        super::encode::encoded(writer, "body", &self.body)?;
        writer.end_object()
    }
}

impl super::CheckedPackageV2 {
    /// The admitted body of one operation member, keyed as `operation_effects`
    /// is. Absent when the package carries no `operation_bodies` member or no
    /// row names the key.
    pub fn operation_body(
        &self,
        document: &str,
        declaring: &CheckedNodeId,
        operation: &str,
    ) -> Option<&OperationBody> {
        let rows = self.wire.operation_bodies.as_ref()?;
        let index = rows
            .binary_search_by(|row| {
                key_of(&row.document, &row.declaring, &row.operation)
                    .cmp(&(document, declaring, operation))
            })
            .ok()?;
        rows.get(index).map(|row| &row.body)
    }

    /// Admitted operation body rows for borrowed typed re-encoding.
    ///
    /// Absence means the optional carrier was omitted, so bodies are
    /// unavailable; a present slice, possibly empty, preserves the admitted
    /// key order.
    pub fn operation_body_rows(&self) -> Option<&[OperationBodyRow]> {
        self.wire.operation_bodies.as_deref()
    }
}

const STAGE_FORM: &str = "operation_bodies";

fn rows_at() -> Path<'static> {
    Path::root().key("operation_bodies")
}

fn site(row: usize, at: Path<'static>) -> Site<'static> {
    Site::new(PackageReadStage::OperationBodies, row, STAGE_FORM, at)
}

fn refused(
    meter: &mut WorkMeter,
    row: usize,
    code: Code,
    cause: Option<Cause>,
    at: Path<'static>,
) -> ValidationFailure {
    let refusal = Refusal::new(code, at);
    let refusal = match cause {
        Some(cause) => refusal.because(cause),
        None => refusal,
    };
    stage_storage::refuse(meter, &site(row, at), refusal)
}

fn invalid(meter: &mut WorkMeter, row: usize, at: Path<'static>) -> ValidationFailure {
    refused(
        meter,
        row,
        Code::InvalidPackage,
        Some(Cause::InvalidValue),
        at,
    )
}

/// Whether `term` is a clause function of kind `body`.
fn is_body_clause(term: &CheckedTerm) -> bool {
    super::owner::is_body_clause(term)
}

/// Whether `node` is owned by the operation member `member` of `document`.
fn owned_by(node: &CheckedSemanticNodeV2<CheckedJson>, document: &str, member: &str) -> bool {
    matches!(
        node.owner.as_ref(),
        Some(CheckedNodeOwner::Model { identity, node })
            if identity.as_ref() == document && node.as_ref() == member
    )
}

/// A dispatch candidate's selected operation and its operation body.
type Candidate<'a> = (&'a CheckedNodeId, &'a str, &'a CheckedNodeId);

/// Validate a present carrier against the selected documents and the package
/// graph before exposing any borrowed row.
pub(super) fn validate(
    rows: Option<&[OperationBodyRow]>,
    wire: &RawCheckedPackageWireV2,
    kinds: &[CheckedNodeKind],
    terms: &[CheckedTerm],
    models: &RetainedModels,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let Some(rows) = rows else {
        return Ok(());
    };
    let nodes = &wire.semantic_graph.nodes;
    // Graph positions by node ID, to resolve a body reference.
    let mut by_id: Vec<usize> = stage_storage::vec(meter, &site(0, rows_at()), nodes.len())?;
    by_id.extend(0..nodes.len());
    by_id.sort_unstable_by(|&left, &right| nodes[left].node_id.cmp(&nodes[right].node_id));
    // Dispatch candidates by selected operation, to join a row to its candidates.
    let tables = wire
        .checked_term_data
        .as_ref()
        .map_or(&[][..], |data| data.dispatch_tables.as_slice());
    let count = tables.iter().map(|table| table.candidates.len()).sum();
    let mut candidates: Vec<Candidate<'_>> = stage_storage::vec(meter, &site(0, rows_at()), count)?;
    candidates.extend(tables.iter().flat_map(|table| {
        table.candidates.iter().map(|candidate| {
            (
                &candidate.selected_operation.declaring,
                candidate.selected_operation.operation.as_ref(),
                &candidate.operation_body,
            )
        })
    }));
    candidates.sort_unstable();
    let covered = sorted_keys(
        meter,
        &site(0, rows_at()),
        rows.iter()
            .map(|row| key_of(&row.document, &row.declaring, &row.operation)),
    )?;
    let repeated = first_repeated(&covered);
    // The (document, member) pairs that own a body clause function.
    let mut owned: Vec<(&str, &str)> = stage_storage::vec(meter, &site(0, rows_at()), nodes.len())?;
    owned.extend(nodes.iter().enumerate().filter_map(|(position, node)| {
        match (
            node.owner.as_ref(),
            kinds.get(position),
            terms.get(position),
        ) {
            (
                Some(CheckedNodeOwner::Model {
                    identity,
                    node: member,
                }),
                Some(kind),
                Some(term),
            ) if matches!(kind, CheckedNodeKind::Function(_)) && is_body_clause(term) => {
                Some((identity.as_ref(), member.as_ref()))
            }
            _ => None,
        }
    }));
    owned.sort_unstable();
    let mut previous: Option<Key<'_>> = None;
    for (index, row) in rows.iter().enumerate() {
        let at = rows_at().index(index);
        stage_storage::work(meter, &site(index, at), 1)?;
        if row.document.is_empty() {
            return Err(refused(
                meter,
                index,
                Code::MalformedWire,
                None,
                at.key("document"),
            ));
        }
        if !is_identifier(&row.operation) {
            return Err(refused(
                meter,
                index,
                Code::MalformedWire,
                None,
                at.key("operation"),
            ));
        }
        let (_, operation) = join_member(
            models,
            &row.document,
            &row.declaring,
            &row.operation,
            at,
            meter,
            &mut |meter, code, cause, locus| refused(meter, index, code, Some(cause), locus),
        )?;
        let has_body = owned
            .binary_search(&(row.document.as_ref(), operation.identity.as_ref()))
            .is_ok();
        let body_at = at.key("body");
        let node = match &row.body {
            OperationBody::Function { node } if has_body => Some(node),
            OperationBody::NoBody if !has_body => None,
            _ => return Err(invalid(meter, index, body_at)),
        };
        if let Some(node) = node {
            let node_at = body_at.key("node");
            let Ok(found) = by_id.binary_search_by(|&position| nodes[position].node_id.cmp(node))
            else {
                return Err(refused(
                    meter,
                    index,
                    Code::MissingDeclaration,
                    Some(Cause::MissingName),
                    node_at,
                ));
            };
            let position = by_id[found];
            if !matches!(kinds.get(position), Some(CheckedNodeKind::Function(_))) {
                return Err(refused(
                    meter,
                    index,
                    Code::IllTyped,
                    Some(Cause::OperatorIneligible),
                    node_at,
                ));
            }
            let own = terms.get(position).is_some_and(is_body_clause)
                && owned_by(&nodes[position], &row.document, &operation.identity);
            if !own {
                return Err(invalid(meter, index, node_at));
            }
        }
        let start =
            candidates.partition_point(|held| (held.0, held.1) < (&row.declaring, &*row.operation));
        let agrees = candidates[start..]
            .iter()
            .take_while(|held| (held.0, held.1) == (&row.declaring, &*row.operation))
            .all(|held| node == Some(held.2));
        if !agrees {
            return Err(invalid(meter, index, body_at));
        }
        if repeated == Some(index) {
            return Err(refused(
                meter,
                index,
                Code::InvalidPackage,
                Some(Cause::ConflictingDefinition),
                at,
            ));
        }
        let key = key_of(&row.document, &row.declaring, &row.operation);
        if previous.is_some_and(|earlier| earlier >= key) {
            return Err(invalid(meter, index, at));
        }
        previous = Some(key);
    }
    if has_uncovered_operation(models, &covered) {
        return Err(invalid(meter, 0, rows_at()));
    }
    Ok(())
}
