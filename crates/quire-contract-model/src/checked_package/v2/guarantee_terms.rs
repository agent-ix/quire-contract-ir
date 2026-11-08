// SPDX-License-Identifier: AGPL-3.0-or-later
//! The typed bodies of the `correspondence`/`implementation_guarantee` and
//! `correspondence`/`component_composition` nodes (QSpec FR-490, FR-491).
//!
//! Each body is the closed flat shape the QSpec text gives: node keys,
//! identifiers, Rust paths and fixed spellings, with no application,
//! aggregate, binding or literal term. [`CheckedTerm`](super::CheckedTerm)
//! carries them as its `ImplementationGuarantee` and `ComponentComposition`
//! terms; the strict read of their meaning is [`super::guarantee_composition`].
//!
//! Implements: FR-038 "Implementation guarantee and component composition bodies".

use super::{CheckedFairnessGranularity, CheckedFairnessKind};
use crate::checked_package::shared::CheckedNodeId;
use quire_canonical::{Encode, Error, Sink, Writer};
use serde::{Deserialize, Serialize};

/// How one guarantee operation's stutter steps are bounded.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum CheckedGuaranteeMeasure {
    /// A well-founded measure every stutter step strictly decreases: a Rust
    /// path.
    Decreases(#[serde(deserialize_with = "super::document_de::exact_vec")] Vec<Box<str>>),
    /// A bound on consecutive stutter steps.
    Bound(u64),
}

/// One `progress` entry: the no-divergence premise of an operation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedGuaranteeProgress {
    /// The operation's context node.
    pub context: CheckedNodeId,
    /// The operation's name.
    pub operation: Box<str>,
    /// The divergence measure.
    pub measure: CheckedGuaranteeMeasure,
}

/// One scheduling constraint the environment or composition supplies.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedFairnessSupply {
    /// Weak or strong.
    pub kind: CheckedFairnessKind,
    /// Whole or each.
    pub granularity: CheckedFairnessGranularity,
    /// The operation's context node.
    pub context: CheckedNodeId,
    /// The operation's name.
    pub operation: Box<str>,
}

/// What one `environment` premise supplies.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum CheckedEnvironmentSupplies {
    /// A scheduling premise.
    Fairness(CheckedFairnessSupply),
    /// An assumption premise: a `predicate` node.
    Assumption(CheckedNodeId),
}

/// One `environment` entry.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedEnvironmentEntry {
    /// The premise's name.
    pub name: Box<str>,
    /// Who supplies it.
    pub supplier: Box<str>,
    /// What it supplies.
    pub supplies: CheckedEnvironmentSupplies,
}

/// One `components` entry of a composition.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedCompositionComponent {
    /// The component's name.
    pub name: Box<str>,
    /// The imported checked package's identity.
    pub package: Box<str>,
    /// The component's subject node.
    pub subject: CheckedNodeId,
}

/// A temporal clause of one component.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedComponentClause {
    /// The component's name.
    pub component: Box<str>,
    /// The clause node.
    pub clause: CheckedNodeId,
}

/// A `relies` or `liveness_dependencies` premise: `guarantee` relies on, or
/// depends on the progress of, `on`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedCompositionPremise {
    /// The guarantee that holds the premise.
    pub guarantee: CheckedComponentClause,
    /// The peer guarantee it rests on.
    pub on: CheckedComponentClause,
}

pub(super) fn kind_spelling(kind: CheckedFairnessKind) -> &'static str {
    match kind {
        CheckedFairnessKind::Weak => "weak",
        CheckedFairnessKind::Strong => "strong",
    }
}

pub(super) fn granularity_spelling(granularity: CheckedFairnessGranularity) -> &'static str {
    match granularity {
        CheckedFairnessGranularity::Whole => "whole",
        CheckedFairnessGranularity::Each => "each",
    }
}

pub(super) fn encode_all<T: Encode, S: Sink + ?Sized>(
    rows: &[T],
    writer: &mut Writer<'_, S>,
) -> Result<(), Error> {
    super::refinement::encode_rows(rows, writer)
}

impl Encode for CheckedGuaranteeMeasure {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        match self {
            Self::Decreases(path) => {
                writer.name("decreases")?;
                writer.serialize(path)?;
            }
            Self::Bound(count) => {
                writer.name("bound")?;
                writer.serialize(count)?;
            }
        }
        writer.end_object()
    }
}

impl Encode for CheckedGuaranteeProgress {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        writer.name("context")?;
        self.context.encode_into(writer)?;
        writer.name("operation")?;
        writer.serialize(&self.operation)?;
        writer.name("measure")?;
        self.measure.encode_into(writer)?;
        writer.end_object()
    }
}

impl Encode for CheckedFairnessSupply {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        writer.name("kind")?;
        writer.string(kind_spelling(self.kind))?;
        writer.name("granularity")?;
        writer.string(granularity_spelling(self.granularity))?;
        writer.name("context")?;
        self.context.encode_into(writer)?;
        writer.name("operation")?;
        writer.serialize(&self.operation)?;
        writer.end_object()
    }
}

impl Encode for CheckedEnvironmentEntry {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        writer.name("name")?;
        writer.serialize(&self.name)?;
        writer.name("supplier")?;
        writer.serialize(&self.supplier)?;
        writer.name("supplies")?;
        writer.begin_object()?;
        match &self.supplies {
            CheckedEnvironmentSupplies::Fairness(supply) => {
                writer.name("fairness")?;
                supply.encode_into(writer)?;
            }
            CheckedEnvironmentSupplies::Assumption(node) => {
                writer.name("assumption")?;
                node.encode_into(writer)?;
            }
        }
        writer.end_object()?;
        writer.end_object()
    }
}

impl Encode for CheckedCompositionComponent {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        writer.name("name")?;
        writer.serialize(&self.name)?;
        writer.name("package")?;
        writer.serialize(&self.package)?;
        writer.name("subject")?;
        self.subject.encode_into(writer)?;
        writer.end_object()
    }
}

impl Encode for CheckedComponentClause {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        writer.name("component")?;
        writer.serialize(&self.component)?;
        writer.name("clause")?;
        self.clause.encode_into(writer)?;
        writer.end_object()
    }
}

impl Encode for CheckedCompositionPremise {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        writer.name("guarantee")?;
        self.guarantee.encode_into(writer)?;
        writer.name("on")?;
        self.on.encode_into(writer)?;
        writer.end_object()
    }
}

/// Every node id a guarantee or composition term holds, in body order, or
/// none for any other term.
pub(super) fn ids_mut(term: &mut super::CheckedTerm) -> Vec<&mut CheckedNodeId> {
    use super::CheckedTerm as T;
    fn environment_ids<'a>(
        entries: &'a mut [CheckedEnvironmentEntry],
        ids: &mut Vec<&'a mut CheckedNodeId>,
    ) {
        for entry in entries {
            match &mut entry.supplies {
                CheckedEnvironmentSupplies::Fairness(supply) => ids.push(&mut supply.context),
                CheckedEnvironmentSupplies::Assumption(node) => ids.push(node),
            }
        }
    }
    let mut ids = Vec::new();
    match term {
        T::ImplementationGuarantee {
            clause,
            relation,
            progress,
            environment,
            ..
        } => {
            ids.push(clause);
            ids.push(relation);
            ids.extend(progress.iter_mut().map(|entry| &mut entry.context));
            environment_ids(environment, &mut ids);
        }
        T::ComponentComposition {
            components,
            guarantees,
            relies,
            liveness_dependencies,
            supplies,
            environment,
            ..
        } => {
            ids.extend(components.iter_mut().map(|entry| &mut entry.subject));
            ids.extend(guarantees.iter_mut().map(|entry| &mut entry.clause));
            for premise in relies.iter_mut().chain(liveness_dependencies.iter_mut()) {
                ids.push(&mut premise.guarantee.clause);
                ids.push(&mut premise.on.clause);
            }
            ids.extend(supplies.iter_mut().map(|entry| &mut entry.context));
            environment_ids(environment, &mut ids);
        }
        _ => {}
    }
    ids
}

/// Replaces every held node id equal to `old` with `new`.
pub(super) fn replace_node_ids(
    term: &mut super::CheckedTerm,
    old: &CheckedNodeId,
    new: &CheckedNodeId,
) {
    for id in ids_mut(term) {
        if id == old {
            *id = new.clone();
        }
    }
}
