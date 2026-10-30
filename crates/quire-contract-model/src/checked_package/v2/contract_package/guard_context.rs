// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-050: read-only admitted path-guard context for one checked application occurrence.

use super::{
    ContractDispositionV1, ContractFeatureRecordV1, ContractPackageTraversalIndex,
    ContractPackageV1, ContractProvenanceV1, ContractRootViewError, ContractRootViewV1,
    ContractSelectedReferenceV1, GuardContextWork,
};
use crate::checked_package::shared::{CheckedNodeId, CheckedOccurrenceRole, CheckedSourceMapEntry};
use crate::checked_package::v2::{
    CheckedNodeKind, CheckedNodeTag, RawSemanticNodeV2 as CheckedSemanticNodeV2, ScalarTypeForm,
};

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
    /// query shares one package index and work budget across body/dependency
    /// validation and never infers guard membership. Each unique guard node's
    /// reached view is validated once per query; repeated occurrences reuse
    /// that validation while retaining their individual order, polarity and
    /// source correspondence.
    pub fn guard_context(
        &self,
        expression: &CheckedNodeId,
        application: &super::ContractOccurrenceKeyV1,
        guards: &[ContractPathGuardV1],
    ) -> Result<ContractGuardContextViewV1<'_>, ContractGuardContextErrorV1> {
        let mut work = GuardContextWork::bounded_query();
        self.guard_context_with_work_mut(expression, application, guards, &mut work)
    }

    #[cfg(feature = "fault-injection")]
    #[doc(hidden)]
    pub fn guard_context_with_work_for_test(
        &self,
        expression: &CheckedNodeId,
        application: &super::ContractOccurrenceKeyV1,
        guards: &[ContractPathGuardV1],
        work_limit: u64,
    ) -> (
        Result<ContractGuardContextViewV1<'_>, ContractGuardContextErrorV1>,
        u64,
    ) {
        let mut work = GuardContextWork::with_limit(work_limit);
        let result = self.guard_context_with_work_mut(expression, application, guards, &mut work);
        (result, work.consumed)
    }

    fn guard_context_with_work_mut(
        &self,
        expression: &CheckedNodeId,
        application: &super::ContractOccurrenceKeyV1,
        guards: &[ContractPathGuardV1],
        query_work: &mut GuardContextWork,
    ) -> Result<ContractGuardContextViewV1<'_>, ContractGuardContextErrorV1> {
        let index = ContractPackageTraversalIndex::build(self, query_work).map_err(|error| {
            ContractGuardContextErrorV1::ApplicationView(ContractRootViewError::Incomplete(error))
        })?;
        let Some(application_record) = index.accounting.get(expression).copied() else {
            return Ok(ContractGuardContextViewV1::Absent);
        };
        if application_record.disposition() != ContractDispositionV1::Lowered {
            return Ok(ContractGuardContextViewV1::NonLowered(application_record));
        }
        let _application_reached =
            match self.reached_closure_indexed(expression, &index, query_work) {
                Ok(ContractRootViewV1::Absent) => return Ok(ContractGuardContextViewV1::Absent),
                Ok(ContractRootViewV1::NonLowered(record)) => {
                    return Ok(ContractGuardContextViewV1::NonLowered(record));
                }
                Ok(ContractRootViewV1::Lowered(reached)) => reached,
                Err(error) => return Err(ContractGuardContextErrorV1::ApplicationView(error)),
            };
        let application_source = index.sources.get(&application.node_id).and_then(|sources| {
            sources.iter().find(|entry| {
                entry.node_id == application.node_id
                    && entry.role == application.role
                    && entry.ordinal == application.ordinal
            })
        });
        query_work.charge(1).map_err(|error| {
            ContractGuardContextErrorV1::ApplicationView(ContractRootViewError::Incomplete(error))
        })?;
        if application.node_id != *expression
            || application.role != CheckedOccurrenceRole::Expression
            || application_source.is_none()
        {
            return Err(ContractGuardContextErrorV1::InvalidApplicationOccurrence {
                application: application.clone(),
            });
        }

        let mut admitted_guards = Vec::new();
        let mut validated_roots = std::collections::BTreeMap::new();
        for (position, guard) in guards.iter().enumerate() {
            let occurrence = &guard.occurrence;
            if let Err(error) = query_work.charge(1) {
                return Err(ContractGuardContextErrorV1::Guard(Box::new(
                    ContractGuardContextFailureV1 {
                        position,
                        occurrence: occurrence.clone(),
                        cause: ContractGuardContextFailureCauseV1::InvalidBody(
                            ContractRootViewError::Incomplete(error),
                        ),
                        source: None,
                    },
                )));
            }
            let source_entries = index.sources.get(&occurrence.node_id).copied();
            let source = source_entries.and_then(|sources| {
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
            if let Err(error) = query_work.charge(
                1_u64.saturating_add(
                    source_entries
                        .and_then(|entries| u64::try_from(entries.len()).ok())
                        .unwrap_or(0),
                ),
            ) {
                return Err(failure(
                    ContractGuardContextFailureCauseV1::InvalidBody(
                        ContractRootViewError::Incomplete(error),
                    ),
                    None,
                ));
            }

            if occurrence.role != CheckedOccurrenceRole::Expression {
                return Err(failure(
                    ContractGuardContextFailureCauseV1::WrongOccurrenceRole,
                    source.or_else(|| source_entries.and_then(|sources| sources.first())),
                ));
            }
            let Some(record) = index.accounting.get(&occurrence.node_id).copied() else {
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

            let Some(sources) = source_entries else {
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

            if !validated_roots.contains_key(&occurrence.node_id) {
                let reached =
                    match self.reached_closure_indexed(&occurrence.node_id, &index, query_work) {
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
                validated_roots.insert(
                    occurrence.node_id.clone(),
                    (expression_node, reached.selected_references),
                );
            }
            let Some((expression_node, selected_references)) =
                validated_roots.get(&occurrence.node_id)
            else {
                return Err(failure(
                    ContractGuardContextFailureCauseV1::InvalidBody(
                        ContractRootViewError::Inconsistent,
                    ),
                    Some(source),
                ));
            };
            if let Err(error) = query_work.charge(
                1_u64.saturating_add(u64::try_from(selected_references.len()).unwrap_or(u64::MAX)),
            ) {
                return Err(failure(
                    ContractGuardContextFailureCauseV1::InvalidBody(
                        ContractRootViewError::Incomplete(error),
                    ),
                    Some(source),
                ));
            }
            admitted_guards.push(ContractGuardReferenceV1 {
                occurrence: occurrence.clone(),
                holds: guard.holds,
                expression: expression_node,
                source,
                selected_references: selected_references.clone(),
            });
        }

        Ok(ContractGuardContextViewV1::Lowered(Box::new(
            ContractLoweredGuardContextV1 {
                provenance: &self.wire.package,
                application: application.clone(),
                guards: admitted_guards,
            },
        )))
    }
}
