//! Bounded read of producer-captured model dispatch candidates (FR-354 AC-4).

use super::checked_term_data::{CheckedDispatchCandidate, CheckedDispatchCandidateTable};
use super::CheckedPackageV2;
use crate::checked_package::shared::{CheckedNodeId, CheckedOccurrence};

/// A complete admitted dispatch table, borrowing candidates in producer order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CheckedDispatchView<'a> {
    /// Exact application and occurrence selected by the producer.
    pub table: &'a CheckedDispatchCandidateTable,
}

impl CheckedDispatchView<'_> {
    /// Candidate rows in the producer's order, including an authoritative empty set.
    pub fn candidates(&self) -> &[CheckedDispatchCandidate] {
        &self.table.candidates
    }
}

/// Why a bounded dispatch read returned no complete table.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum CheckedDispatchReadError {
    /// The producer retained no checked-term sidecar.
    #[error("checked-term dispatch data is unavailable")]
    Unavailable,
    /// No admitted table names the requested application and occurrence.
    #[error("no dispatch table names the requested occurrence")]
    Missing,
    /// The first uncharged row or candidate exceeded the caller's work limit.
    #[error("dispatch read exhausted its work limit")]
    WorkExhausted {
        /// Caller-supplied work limit.
        limit: u64,
        /// Work attempted, including the rejected charge.
        consumed: u64,
        /// Sidecar row at which the charge failed.
        row: usize,
        /// Candidate position when the rejected charge was for a candidate.
        candidate: Option<usize>,
    },
}

impl CheckedPackageV2 {
    /// Returns one complete producer-captured dispatch table under a finite work limit.
    ///
    /// The read does not choose a candidate, evaluate a precondition, or derive
    /// a table from the graph. `Missing` differs from a captured empty set.
    pub fn dispatch_candidates(
        &self,
        application: &CheckedNodeId,
        occurrence: &CheckedOccurrence,
        work_limit: u64,
    ) -> Result<CheckedDispatchView<'_>, CheckedDispatchReadError> {
        let Some(data) = self.wire.checked_term_data.as_ref() else {
            return Err(CheckedDispatchReadError::Unavailable);
        };
        let mut consumed = 0_u64;
        for (row_index, table) in data.dispatch_tables.iter().enumerate() {
            consumed = consumed.saturating_add(1);
            if consumed > work_limit {
                return Err(CheckedDispatchReadError::WorkExhausted {
                    limit: work_limit,
                    consumed,
                    row: row_index,
                    candidate: None,
                });
            }
            if &table.application != application || &table.occurrence != occurrence {
                continue;
            }
            for candidate_index in 0..table.candidates.len() {
                consumed = consumed.saturating_add(1);
                if consumed > work_limit {
                    return Err(CheckedDispatchReadError::WorkExhausted {
                        limit: work_limit,
                        consumed,
                        row: row_index,
                        candidate: Some(candidate_index),
                    });
                }
            }
            return Ok(CheckedDispatchView { table });
        }
        Err(CheckedDispatchReadError::Missing)
    }
}
