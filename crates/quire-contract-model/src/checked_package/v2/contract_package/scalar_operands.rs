// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-054: position-preserving scalar operands from one admitted V1 package.

use super::{
    ContractFeatureRecordV1, ContractOccurrenceKeyV1, ContractPackageV1, ContractProvenanceV1,
    ContractRootViewError, ContractRootViewV1,
};
use crate::checked_package::shared::{
    CheckedNodeId, CheckedOccurrenceRole, CheckedSemanticId, CheckedSourceMapEntry,
};
use crate::checked_package::v2::scalar_operands::{scalar_arguments, ScalarArgumentChild};
use crate::checked_package::v2::{
    CheckedScalarOperandError, CheckedScalarOperandRange, CheckedSemanticNodeV2,
};
use num_bigint::BigInt;
use thiserror::Error;

/// A represented graph child or an occurrence-qualified inline integer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContractScalarOperandChildV1<'a> {
    /// One referenced node with its exact admitted source-map entries.
    GraphReference {
        /// The represented graph node.
        node: &'a CheckedSemanticNodeV2,
        /// That node's admitted sources.
        sources: &'a [CheckedSourceMapEntry],
    },
    /// One literal inside the selected application occurrence, without a node.
    InlineInteger {
        /// Exact signed integer value.
        value: BigInt,
        /// Admitted application occurrence that owns this inline position.
        occurrence: ContractOccurrenceKeyV1,
    },
}

/// What the operand's own integer type or literal says about its native range.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractScalarOwnRangeV1 {
    /// Exact inclusive range representable as native `i128` endpoints.
    Finite {
        /// Inclusive lower endpoint.
        lower: i128,
        /// Inclusive upper endpoint.
        upper: i128,
    },
    /// The admitted child is an unbounded integer.
    Unbounded,
    /// The child has no readable integer range of the expected shape.
    Unavailable,
    /// A finite endpoint or literal exceeds native `i128`.
    OutsideNative,
}

impl From<CheckedScalarOperandRange> for ContractScalarOwnRangeV1 {
    fn from(range: CheckedScalarOperandRange) -> Self {
        Self::Finite {
            lower: range.lower,
            upper: range.upper,
        }
    }
}

/// One argument at its authored zero-based position.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractScalarOperandV1<'a> {
    /// Position in the admitted application's `arguments` array.
    pub ordinal: u64,
    /// Authentic graph child or inline integer.
    pub child: ContractScalarOperandChildV1<'a>,
    /// Its own finite range or typed absence, independent of generation choice.
    pub own_range: ContractScalarOwnRangeV1,
}

/// Ordered operands of one lowered scalar application.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractLoweredScalarOperandsV1<'a> {
    /// Exact source package and selected lowering profile.
    pub provenance: &'a ContractProvenanceV1,
    /// Canonical identity of the admitted six-field package.
    pub identity: &'a CheckedSemanticId,
    /// The caller's authenticated application occurrence.
    pub application: ContractOccurrenceKeyV1,
    /// Exact admitted source entry matching the supplied occurrence.
    pub application_source: &'a CheckedSourceMapEntry,
    /// One entry per authored argument position, in order.
    pub arguments: Vec<ContractScalarOperandV1<'a>>,
}

/// Exact request disposition for an admitted scalar operand query.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContractScalarOperandViewV1<'a> {
    /// A lowered scalar application with its ordered arguments.
    Lowered(Box<ContractLoweredScalarOperandsV1<'a>>),
    /// Exact non-lowered accounting record for this request.
    NonLowered(&'a ContractFeatureRecordV1),
    /// No accounting record names this key.
    Absent,
}

/// A typed refusal before any operand list is selected.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum ContractScalarOperandErrorV1 {
    /// The reached admitted view failed defensively or exhausted its work limit.
    #[error("admitted view failed: {0:?}")]
    View(ContractRootViewError),
    /// The supplied expression occurrence is not an exact admitted source key.
    #[error("application occurrence does not match the admitted source map")]
    InvalidOccurrence,
    /// The represented application is not an eligible scalar integer operation.
    #[error("scalar argument projection failed: {0}")]
    Projection(CheckedScalarOperandError),
}

impl From<ContractRootViewError> for ContractScalarOperandErrorV1 {
    fn from(error: ContractRootViewError) -> Self {
        Self::View(error)
    }
}

impl ContractPackageV1 {
    /// Project typed, ordered scalar arguments from one admitted lowered root.
    /// The original checked graph and any consumer-selected interval are not inputs.
    pub fn scalar_operands(
        &self,
        application: &CheckedNodeId,
        occurrence: &ContractOccurrenceKeyV1,
    ) -> Result<ContractScalarOperandViewV1<'_>, ContractScalarOperandErrorV1> {
        let reached = match self.reached_closure(application)? {
            ContractRootViewV1::Absent => return Ok(ContractScalarOperandViewV1::Absent),
            ContractRootViewV1::NonLowered(record) => {
                return Ok(ContractScalarOperandViewV1::NonLowered(record));
            }
            ContractRootViewV1::Lowered(reached) => reached,
        };
        let sources = self
            .sources(application)
            .ok_or(ContractScalarOperandErrorV1::View(
                ContractRootViewError::Inconsistent,
            ))?;
        if occurrence.node_id != *application
            || occurrence.role != CheckedOccurrenceRole::Expression
        {
            return Err(ContractScalarOperandErrorV1::InvalidOccurrence);
        }
        let application_source = sources
            .iter()
            .find(|entry| occurrence.matches(entry))
            .ok_or(ContractScalarOperandErrorV1::InvalidOccurrence)?;

        let arguments = scalar_arguments(&self.wire.semantic_graph.nodes, reached.root)
            .map_err(ContractScalarOperandErrorV1::Projection)?
            .map(|argument| {
                let argument = argument.map_err(ContractScalarOperandErrorV1::Projection)?;
                let own_range = match argument.own_range {
                    Ok(range) => range.into(),
                    Err(CheckedScalarOperandError::UnboundedRange) => {
                        ContractScalarOwnRangeV1::Unbounded
                    }
                    Err(CheckedScalarOperandError::MissingRange) => {
                        ContractScalarOwnRangeV1::Unavailable
                    }
                    Err(CheckedScalarOperandError::RangeOutOfI128) => {
                        ContractScalarOwnRangeV1::OutsideNative
                    }
                    Err(other) => return Err(ContractScalarOperandErrorV1::Projection(other)),
                };
                let child = match argument.child {
                    ScalarArgumentChild::Graph(node) => {
                        let sources = self.sources(&node.node_id).ok_or(
                            ContractScalarOperandErrorV1::View(ContractRootViewError::Inconsistent),
                        )?;
                        ContractScalarOperandChildV1::GraphReference { node, sources }
                    }
                    ScalarArgumentChild::InlineInteger(value) => {
                        ContractScalarOperandChildV1::InlineInteger {
                            value,
                            occurrence: occurrence.clone(),
                        }
                    }
                };
                Ok(ContractScalarOperandV1 {
                    ordinal: argument.ordinal,
                    child,
                    own_range,
                })
            })
            .collect::<Result<Vec<_>, ContractScalarOperandErrorV1>>()?;
        Ok(ContractScalarOperandViewV1::Lowered(Box::new(
            ContractLoweredScalarOperandsV1 {
                provenance: &self.wire.package,
                identity: &self.wire.identity,
                application: occurrence.clone(),
                application_source,
                arguments,
            },
        )))
    }
}
