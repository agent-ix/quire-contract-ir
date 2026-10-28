// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Public projection of an admitted state-clause execution point (FR-058).

use super::{
    CheckedNodeId, CheckedNodeKind, CheckedOccurrence, CheckedOccurrenceRole, CheckedPackageV2,
    CheckedTerm, ExecutionPoint, StateForm,
};
use thiserror::Error;

/// The first reason a selected clause claim has no admitted execution point.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckedExecutionPointRefusal {
    /// The supplied clause identity is absent from the package.
    UnknownClause,
    /// The selected node is not a state clause.
    NotStateClause,
    /// The supplied occurrence is not an authentic claim of the selected clause.
    MissingClaimOccurrence,
    /// The admitted clause has no execution point.
    MissingExecutionPoint,
}

impl CheckedExecutionPointRefusal {
    /// The stable code spelling for this refusal.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnknownClause => "unknown_clause",
            Self::NotStateClause => "not_state_clause",
            Self::MissingClaimOccurrence => "missing_claim_occurrence",
            Self::MissingExecutionPoint => "missing_execution_point",
        }
    }

    /// All closed refusal codes in precedence order.
    pub const fn all() -> &'static [Self] {
        &[
            Self::UnknownClause,
            Self::NotStateClause,
            Self::MissingClaimOccurrence,
            Self::MissingExecutionPoint,
        ]
    }

    /// Parse one stable refusal code.
    #[ix_trace_rs::string_edge]
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "unknown_clause" => Some(Self::UnknownClause),
            "not_state_clause" => Some(Self::NotStateClause),
            "missing_claim_occurrence" => Some(Self::MissingClaimOccurrence),
            "missing_execution_point" => Some(Self::MissingExecutionPoint),
            _ => None,
        }
    }
}

/// A typed refusal with the caller's exact clause and owner-bound occurrence.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[error("{code:?} for clause {clause:?} and claim {claim_owner:?}/{claim:?}")]
pub struct CheckedExecutionPointError {
    /// The first applicable closed refusal code.
    pub code: CheckedExecutionPointRefusal,
    /// The supplied clause identity.
    pub clause: CheckedNodeId,
    /// The supplied claim owner's node identity.
    pub claim_owner: CheckedNodeId,
    /// The supplied occurrence, even when its role is not `claim`.
    pub claim: CheckedOccurrence,
}

impl CheckedPackageV2 {
    /// Return the exact point retained on a selected admitted state clause.
    ///
    /// The supplied owner and occurrence must be an authentic claim of this
    /// node. A point from another clause, its anchor, or source text is never used.
    pub fn clause_execution_point(
        &self,
        clause: &CheckedNodeId,
        claim: (&CheckedNodeId, &CheckedOccurrence),
    ) -> Result<ExecutionPoint, CheckedExecutionPointError> {
        use CheckedExecutionPointRefusal as Refusal;

        let error = |code| CheckedExecutionPointError {
            code,
            clause: clause.clone(),
            claim_owner: claim.0.clone(),
            claim: claim.1.clone(),
        };
        let node = self
            .node(clause)
            .ok_or_else(|| error(Refusal::UnknownClause))?;
        if self.node_kind(clause) != Some(CheckedNodeKind::State(StateForm::StateClause)) {
            return Err(error(Refusal::NotStateClause));
        }
        if claim.0 != clause
            || claim.1.role != CheckedOccurrenceRole::Claim
            || !node.occurrences.contains(claim.1)
        {
            return Err(error(Refusal::MissingClaimOccurrence));
        }
        let point = match self.term(clause) {
            Some(CheckedTerm::Application { operation, .. }) => operation.execution_point(),
            _ => None,
        };
        point
            .cloned()
            .ok_or_else(|| error(Refusal::MissingExecutionPoint))
    }
}
