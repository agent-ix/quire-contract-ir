// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-049: read-only result context for one QSL-checked application occurrence.

use super::{
    ContractDispositionV1, ContractFeatureRecordV1, ContractPackageV1, ContractProvenanceV1,
    ContractReachedClosureV1, ContractRootViewError, ContractRootViewV1,
};
use crate::checked_package::shared::{
    CheckedNodeId, CheckedOccurrenceRole, CheckedSemanticId, CheckedSourceMapEntry,
};
use crate::checked_package::v2::{
    BodyTerm, BoundedDomainForm, CheckedNodeKind, CheckedNodeTag, CheckedSemanticNodeV2,
    ExpressionForm,
};
use serde::Deserialize;

/// A node key with the role and ordinal of one admitted source occurrence.
///
/// QSL's checked `OccurrenceKey` is mapped to this type at the IR boundary;
/// source regions remain owned by the admitted package's source map.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractOccurrenceKeyV1 {
    /// The semantic node named by this occurrence.
    pub node_id: CheckedNodeId,
    /// The source-map occurrence role.
    pub role: CheckedOccurrenceRole,
    /// The source-map occurrence ordinal.
    pub ordinal: u64,
}

impl ContractOccurrenceKeyV1 {
    fn matches(&self, entry: &CheckedSourceMapEntry) -> bool {
        self.node_id == entry.node_id && self.role == entry.role && self.ordinal == entry.ordinal
    }
}

/// One represented result-bound node and all its admitted source occurrences.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractResultBoundV1<'a> {
    /// The exact bounded-domain node, never an operand or substitute key.
    pub node: &'a CheckedSemanticNodeV2,
    /// Source occurrences from the admitted FR-330 source map.
    pub sources: &'a [CheckedSourceMapEntry],
}

/// The typed result of applying one QSL-checked conversion choice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContractResultContextV1<'a> {
    /// QSL supplied no conversion and the expression owns a readable bound.
    DirectBounded { bound: ContractResultBoundV1<'a> },
    /// QSL supplied no conversion and the expression owns an unbounded scalar.
    Unnarrowed,
    /// The selected direct narrow has this represented target bound.
    Narrowed {
        /// The exact selected conversion node and its source occurrences.
        conversion: &'a CheckedSemanticNodeV2,
        /// Exact selected conversion source occurrences.
        conversion_sources: &'a [CheckedSourceMapEntry],
        /// Its result-bound node and source occurrences.
        bound: ContractResultBoundV1<'a>,
    },
    /// The selected own type or narrow target has no readable bound of the
    /// requested form over the expression's scalar type.
    MissingBound { type_key: CheckedNodeId },
    /// The application occurrence or selected conversion is not represented
    /// as the required direct narrow in this admitted package.
    IncompleteContext,
}

/// One lowered expression and its occurrence-scoped result context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractLoweredResultV1<'a> {
    /// Exact source package identity and selected lowering profile.
    pub provenance: &'a ContractProvenanceV1,
    /// Canonical identity of the admitted six-field package.
    pub identity: &'a CheckedSemanticId,
    /// The QSL-checked application occurrence supplied by the caller.
    pub application: ContractOccurrenceKeyV1,
    /// The QSL-checked optional conversion occurrence supplied by the caller.
    pub conversion: Option<ContractOccurrenceKeyV1>,
    /// The expression's unchanged forward reached closure.
    pub reached: ContractReachedClosureV1<'a>,
    /// Result context selected only by that QSL occurrence pair.
    pub context: ContractResultContextV1<'a>,
}

/// Exact requested-key disposition with occurrence-scoped result context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContractResultViewV1<'a> {
    /// A lowered request with a typed result-context outcome.
    Lowered(Box<ContractLoweredResultV1<'a>>),
    /// The exact non-lowered feature-accounting record.
    NonLowered(&'a ContractFeatureRecordV1),
    /// No feature-accounting record names this requested expression key.
    Absent,
}

impl ContractPackageV1 {
    /// Queries one lowered expression for the result bound selected by a
    /// QSL-checked requirement occurrence. The caller passes QSL's exact
    /// optional conversion occurrence; this method never discovers one by a
    /// reverse graph walk. The admitted six-field wire is unchanged.
    pub fn result_context(
        &self,
        expression: &CheckedNodeId,
        application: &ContractOccurrenceKeyV1,
        conversion: Option<&ContractOccurrenceKeyV1>,
        expected: BoundedDomainForm,
    ) -> Result<ContractResultViewV1<'_>, ContractRootViewError> {
        let reached = match self.reached_closure(expression)? {
            ContractRootViewV1::Absent => return Ok(ContractResultViewV1::Absent),
            ContractRootViewV1::NonLowered(record) => {
                return Ok(ContractResultViewV1::NonLowered(record));
            }
            ContractRootViewV1::Lowered(reached) => reached,
        };

        let context = self.context_for(&reached, expression, application, conversion, expected)?;
        Ok(ContractResultViewV1::Lowered(Box::new(
            ContractLoweredResultV1 {
                provenance: &self.wire.package,
                identity: &self.wire.identity,
                application: application.clone(),
                conversion: conversion.cloned(),
                reached,
                context,
            },
        )))
    }

    fn context_for<'a>(
        &'a self,
        reached: &ContractReachedClosureV1<'a>,
        expression: &CheckedNodeId,
        application: &ContractOccurrenceKeyV1,
        conversion: Option<&ContractOccurrenceKeyV1>,
        expected: BoundedDomainForm,
    ) -> Result<ContractResultContextV1<'a>, ContractRootViewError> {
        let Some(expression_sources) = self.sources(expression) else {
            return Err(ContractRootViewError::Inconsistent);
        };
        if application.node_id != *expression
            || application.role != CheckedOccurrenceRole::Expression
            || !expression_sources
                .iter()
                .any(|entry| application.matches(entry))
        {
            return Ok(ContractResultContextV1::IncompleteContext);
        }

        let expression_type = &reached.root.semantic_type;
        let scalar_type = self
            .node(expression_type)
            .and_then(|node| match node_kind(node) {
                Some(CheckedNodeKind::BoundedDomain(_)) => Some(&node.semantic_type),
                Some(CheckedNodeKind::ScalarType(_)) => Some(expression_type),
                _ => None,
            });

        let selected_type = if let Some(selected) = conversion {
            if selected.role != CheckedOccurrenceRole::Expression
                || !self.has_lowered(&selected.node_id)
                || !self
                    .sources(&selected.node_id)
                    .is_some_and(|sources| sources.iter().any(|entry| selected.matches(entry)))
            {
                return Ok(ContractResultContextV1::IncompleteContext);
            }
            // A context root is separately lowered. This validates its
            // admitted body and source edges before selecting its result type.
            let converted = match self.reached_closure(&selected.node_id)? {
                ContractRootViewV1::Lowered(converted) => converted,
                ContractRootViewV1::Absent | ContractRootViewV1::NonLowered(_) => {
                    return Ok(ContractResultContextV1::IncompleteContext);
                }
            };
            if !direct_narrow_of(converted.root, expression) {
                return Ok(ContractResultContextV1::IncompleteContext);
            }
            &converted.root.semantic_type
        } else {
            expression_type
        };

        let Some(selected_node) = self.node(selected_type) else {
            return Ok(ContractResultContextV1::MissingBound {
                type_key: selected_type.clone(),
            });
        };
        if conversion.is_none()
            && matches!(
                node_kind(selected_node),
                Some(CheckedNodeKind::ScalarType(_))
            )
        {
            return Ok(ContractResultContextV1::Unnarrowed);
        }
        if node_kind(selected_node) != Some(CheckedNodeKind::BoundedDomain(expected))
            || scalar_type != Some(&selected_node.semantic_type)
        {
            return Ok(ContractResultContextV1::MissingBound {
                type_key: selected_type.clone(),
            });
        }
        let Some(sources) = self.sources(selected_type) else {
            return Err(ContractRootViewError::Inconsistent);
        };
        let bound = ContractResultBoundV1 {
            node: selected_node,
            sources,
        };
        match conversion {
            None => Ok(ContractResultContextV1::DirectBounded { bound }),
            Some(selected) => {
                let Some(conversion_node) = self.node(&selected.node_id) else {
                    return Err(ContractRootViewError::Inconsistent);
                };
                let Some(conversion_sources) = self.sources(&selected.node_id) else {
                    return Err(ContractRootViewError::Inconsistent);
                };
                Ok(ContractResultContextV1::Narrowed {
                    conversion: conversion_node,
                    conversion_sources,
                    bound,
                })
            }
        }
    }

    fn node(&self, key: &CheckedNodeId) -> Option<&CheckedSemanticNodeV2> {
        self.wire
            .semantic_graph
            .nodes
            .iter()
            .find(|node| &node.node_id == key)
    }

    fn sources(&self, key: &CheckedNodeId) -> Option<&[CheckedSourceMapEntry]> {
        self.wire
            .source_map
            .nodes
            .iter()
            .find(|entry| &entry.node_id == key)
            .map(|entry| entry.occurrences.as_slice())
    }

    fn has_lowered(&self, key: &CheckedNodeId) -> bool {
        self.wire.feature_accounting.iter().any(|record| {
            record.node_id() == key && record.disposition() == ContractDispositionV1::Lowered
        })
    }
}

fn node_kind(node: &CheckedSemanticNodeV2) -> Option<CheckedNodeKind> {
    CheckedNodeKind::decode(
        CheckedNodeTag::from_wire(&node.node_tag)?,
        &node.semantic_form,
    )
}

fn direct_narrow_of(node: &CheckedSemanticNodeV2, expression: &CheckedNodeId) -> bool {
    if node_kind(node) != Some(CheckedNodeKind::Expression(ExpressionForm::Conversion)) {
        return false;
    }
    let body = &node.body;
    if super::super::super::common::body_term(body) != Some(BodyTerm::Application)
        || super::super::super::common::application_operator(body)
            != Some(super::super::ApplicationOperator::Convert)
    {
        return false;
    }
    let Some(operation) = body
        .get("operation")
        .and_then(|value| super::super::operations::OperationWire::deserialize(value).ok())
    else {
        return false;
    };
    if operation.identity.as_ref() != "quire.op.numeric.narrow" {
        return false;
    }
    let Some(arguments) = body.get("arguments").and_then(serde_json::Value::as_array) else {
        return false;
    };
    let [argument] = arguments.as_slice() else {
        return false;
    };
    if super::super::super::common::body_term(argument) != Some(BodyTerm::Reference) {
        return false;
    }
    argument
        .get("target")
        .and_then(|value| CheckedNodeId::deserialize(value).ok())
        .is_some_and(|target| target == *expression)
}
