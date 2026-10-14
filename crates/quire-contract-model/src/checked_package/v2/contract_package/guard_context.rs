// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-050: read-only admitted path-guard context for one checked application occurrence.

use super::{
    ContractDispositionV1, ContractFeatureRecordV1, ContractPackageV1, ContractProvenanceV1,
    ContractRootViewError, ContractRootViewV1, ContractSelectedReferenceV1,
};
use crate::checked_package::shared::{
    CheckedNodeId, CheckedOccurrenceRole, CheckedSemanticId, CheckedSourceMapEntry,
};
use crate::checked_package::v2::{
    CheckedNodeKind, CheckedNodeTag, CheckedSemanticNodeV2, ScalarTypeForm,
};

/// Maximum number of ordered guards validated by one query. This matches
/// QSL's bounded path depth and caps repeated full-graph reached-view work.
pub const CONTRACT_GUARD_CONTEXT_MAX_GUARDS: usize = 64;

/// One QSL-checked path guard mapped to the exact IR source occurrence.
///
/// The caller preserves QSL's checked order and maps each `PathGuard` to this
/// type. IR validates structural admission and polarity; it does not attest
/// that the guard belongs to the supplied application occurrence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractPathGuardV1 {
    /// The checked Boolean-expression occurrence supplied by the caller.
    pub occurrence: super::ContractOccurrenceKeyV1,
    /// Whether this guard holds on the checked path.
    pub holds: bool,
}

/// A public, admitted Boolean-expression reference for one path guard.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractGuardReferenceV1<'a> {
    /// Exact typed source occurrence supplied by the caller.
    pub occurrence: super::ContractOccurrenceKeyV1,
    /// Polarity supplied with that occurrence.
    pub holds: bool,
    /// The represented Boolean expression node from the admitted package.
    pub expression: &'a CheckedSemanticNodeV2,
    /// Exact matching source-map entry from the admitted package.
    pub source: &'a CheckedSourceMapEntry,
    /// Selected external package/key identities reached while validating the
    /// expression body. These remain at their admitted dependency boundary.
    pub selected_references: Vec<ContractSelectedReferenceV1>,
}

/// One lowered application and its ordered, admitted path-guard references.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractLoweredGuardContextV1<'a> {
    /// Exact source package identity and selected lowering profile.
    pub provenance: &'a ContractProvenanceV1,
    /// Canonical identity of the admitted six-field package.
    pub identity: &'a CheckedSemanticId,
    /// The checked application occurrence supplied by the caller.
    pub application: super::ContractOccurrenceKeyV1,
    /// Guard references in the exact supplied order, including repeated keys.
    pub guards: Vec<ContractGuardReferenceV1<'a>>,
}

/// Typed causes for rejecting one supplied path guard.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContractGuardContextFailureCauseV1 {
    /// No feature-accounting record names the supplied guard key.
    AbsentRoot,
    /// The requested guard has a non-lowered FR-035 disposition.
    NonLoweredRoot {
        /// Exact feature-accounting disposition.
        disposition: ContractDispositionV1,
    },
    /// A lowered guard key has no admitted source-map entry.
    MissingSourceMapEntry,
    /// The supplied occurrence role is not `expression`.
    WrongOccurrenceRole,
    /// The source map has this expression key but not the supplied ordinal.
    StaleOccurrenceOrdinal,
    /// The represented root is not an expression node.
    WrongNodeForm,
    /// The represented expression's admitted type is not Boolean.
    NonBooleanExpression,
    /// Reader-backed body or dependency validation failed defensively.
    InvalidBody(ContractRootViewError),
    /// The admitted root record has no represented node.
    UnrepresentedRoot,
}

/// A typed failure for one guard, including its position and source locus.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractGuardContextFailureV1 {
    /// Zero-based position in the caller's ordered guard list.
    pub position: usize,
    /// Exact supplied occurrence that failed.
    pub occurrence: super::ContractOccurrenceKeyV1,
    /// Closed reason for the refusal.
    pub cause: ContractGuardContextFailureCauseV1,
    /// Matching source-map occurrence, when one exists.
    pub source: Option<CheckedSourceMapEntry>,
}

/// Errors that prevent an admitted guard-context query from returning a view.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContractGuardContextErrorV1 {
    /// The application occurrence does not match its admitted source entry.
    InvalidApplicationOccurrence {
        /// Exact supplied application occurrence.
        application: super::ContractOccurrenceKeyV1,
    },
    /// Defensive reader-backed validation of the application failed.
    ApplicationView(ContractRootViewError),
    /// The requested path has more guards than one bounded query admits.
    GuardLimitExceeded {
        /// Number of supplied guards.
        requested: usize,
        /// Maximum guards admitted by one query.
        limit: usize,
    },
    /// The first invalid guard in caller order.
    Guard(Box<ContractGuardContextFailureV1>),
}

/// Exact requested-key disposition and occurrence-scoped guard context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContractGuardContextViewV1<'a> {
    /// A lowered application and all validated guards.
    Lowered(Box<ContractLoweredGuardContextV1<'a>>),
    /// The exact non-lowered FR-035 application record.
    NonLowered(&'a ContractFeatureRecordV1),
    /// No feature-accounting record names the application expression key.
    Absent,
}

impl ContractPackageV1 {
    /// Queries the ordered guards supplied for one checked application
    /// occurrence. Every guard must be an explicitly lowered root with an
    /// exact admitted `expression` source occurrence and Boolean type. The
    /// query uses the existing reached-view API for body/dependency validation
    /// and never infers guard membership. One query accepts at most
    /// [`CONTRACT_GUARD_CONTEXT_MAX_GUARDS`] guards so repeated reached-view
    /// validation has a fixed work ceiling.
    pub fn guard_context(
        &self,
        expression: &CheckedNodeId,
        application: &super::ContractOccurrenceKeyV1,
        guards: &[ContractPathGuardV1],
    ) -> Result<ContractGuardContextViewV1<'_>, ContractGuardContextErrorV1> {
        let _application_reached = match self.reached_closure(expression) {
            Ok(ContractRootViewV1::Absent) => return Ok(ContractGuardContextViewV1::Absent),
            Ok(ContractRootViewV1::NonLowered(record)) => {
                return Ok(ContractGuardContextViewV1::NonLowered(record));
            }
            Ok(ContractRootViewV1::Lowered(reached)) => reached,
            Err(error) => return Err(ContractGuardContextErrorV1::ApplicationView(error)),
        };
        let application_source = self.sources(&application.node_id).and_then(|sources| {
            sources.iter().find(|entry| {
                entry.node_id == application.node_id
                    && entry.role == application.role
                    && entry.ordinal == application.ordinal
            })
        });
        if application.node_id != *expression
            || application.role != CheckedOccurrenceRole::Expression
            || application_source.is_none()
        {
            return Err(ContractGuardContextErrorV1::InvalidApplicationOccurrence {
                application: application.clone(),
            });
        }

        if guards.len() > CONTRACT_GUARD_CONTEXT_MAX_GUARDS {
            return Err(ContractGuardContextErrorV1::GuardLimitExceeded {
                requested: guards.len(),
                limit: CONTRACT_GUARD_CONTEXT_MAX_GUARDS,
            });
        }

        let mut admitted_guards = Vec::with_capacity(guards.len());
        for (position, guard) in guards.iter().enumerate() {
            let occurrence = &guard.occurrence;
            let source = self.sources(&occurrence.node_id).and_then(|sources| {
                sources.iter().find(|entry| {
                    entry.node_id == occurrence.node_id
                        && entry.role == occurrence.role
                        && entry.ordinal == occurrence.ordinal
                })
            });
            let failure = |cause, source: Option<&CheckedSourceMapEntry>| {
                ContractGuardContextErrorV1::Guard(Box::new(ContractGuardContextFailureV1 {
                    position,
                    occurrence: occurrence.clone(),
                    cause,
                    source: source.cloned(),
                }))
            };

            if occurrence.role != CheckedOccurrenceRole::Expression {
                return Err(failure(
                    ContractGuardContextFailureCauseV1::WrongOccurrenceRole,
                    source.or_else(|| {
                        self.sources(&occurrence.node_id)
                            .and_then(|sources| sources.first())
                    }),
                ));
            }
            let Some(record) = self
                .wire
                .feature_accounting
                .iter()
                .find(|record| record.node_id() == &occurrence.node_id)
            else {
                return Err(failure(
                    ContractGuardContextFailureCauseV1::AbsentRoot,
                    source,
                ));
            };
            if record.disposition() != ContractDispositionV1::Lowered {
                return Err(failure(
                    ContractGuardContextFailureCauseV1::NonLoweredRoot {
                        disposition: record.disposition(),
                    },
                    source,
                ));
            }

            let Some(sources) = self.sources(&occurrence.node_id) else {
                return Err(failure(
                    ContractGuardContextFailureCauseV1::MissingSourceMapEntry,
                    None,
                ));
            };
            let Some(source) = source else {
                let cause = if sources
                    .iter()
                    .any(|entry| entry.role == CheckedOccurrenceRole::Expression)
                {
                    ContractGuardContextFailureCauseV1::StaleOccurrenceOrdinal
                } else {
                    ContractGuardContextFailureCauseV1::WrongOccurrenceRole
                };
                return Err(failure(cause, sources.first()));
            };

            let reached = match self.reached_closure(&occurrence.node_id) {
                Ok(ContractRootViewV1::Lowered(reached)) => reached,
                Ok(ContractRootViewV1::Absent | ContractRootViewV1::NonLowered(_)) => {
                    return Err(failure(
                        ContractGuardContextFailureCauseV1::UnrepresentedRoot,
                        Some(source),
                    ));
                }
                Err(error) => {
                    return Err(failure(
                        ContractGuardContextFailureCauseV1::InvalidBody(error),
                        Some(source),
                    ));
                }
            };
            let expression_node = reached.root;
            let kind = CheckedNodeTag::from_wire(&expression_node.node_tag)
                .and_then(|tag| CheckedNodeKind::decode(tag, &expression_node.semantic_form));
            if !matches!(kind, Some(CheckedNodeKind::Expression(_))) {
                return Err(failure(
                    ContractGuardContextFailureCauseV1::WrongNodeForm,
                    Some(source),
                ));
            }
            let Some(type_node) = reached
                .nodes
                .iter()
                .find(|entry| entry.node.node_id == expression_node.semantic_type)
                .map(|entry| entry.node)
            else {
                return Err(failure(
                    ContractGuardContextFailureCauseV1::InvalidBody(
                        ContractRootViewError::Inconsistent,
                    ),
                    Some(source),
                ));
            };
            let type_kind = CheckedNodeTag::from_wire(&type_node.node_tag)
                .and_then(|tag| CheckedNodeKind::decode(tag, &type_node.semantic_form));
            if type_kind != Some(CheckedNodeKind::ScalarType(ScalarTypeForm::Boolean)) {
                return Err(failure(
                    ContractGuardContextFailureCauseV1::NonBooleanExpression,
                    Some(source),
                ));
            }
            admitted_guards.push(ContractGuardReferenceV1 {
                occurrence: occurrence.clone(),
                holds: guard.holds,
                expression: expression_node,
                source,
                selected_references: reached.selected_references,
            });
        }

        Ok(ContractGuardContextViewV1::Lowered(Box::new(
            ContractLoweredGuardContextV1 {
                provenance: &self.wire.package,
                identity: &self.wire.identity,
                application: application.clone(),
                guards: admitted_guards,
            },
        )))
    }
}
