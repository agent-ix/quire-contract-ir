//! Closed outcomes at the bounded-Kani boundary.

use serde::{Deserialize, Serialize};

/// Every terminal bounded-Kani outcome.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KaniOutcomeKind {
    /// The selected finite property was proved under the retained profile and assumptions.
    Proved,
    /// A concrete finite counterexample was found.
    Counterexample,
    /// The selected profile or construct refuses the request.
    Refused,
    /// The offered finite input is malformed or contradictory.
    InvalidInput,
    /// A required population or observation is incomplete.
    IncompleteInput,
    /// A required tool or dependency is unavailable.
    Unavailable,
    /// The selected execution timed out.
    TimedOut,
    /// A selected resource budget was exhausted.
    ResourceExhausted,
    /// The caller cancelled execution.
    Cancelled,
    /// The backend returned no qualified interpretation.
    Inconclusive,
}

/// Typed output which cannot manufacture a Boolean for a non-success state.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KaniOutcome {
    /// Closed outcome kind.
    pub kind: KaniOutcomeKind,
    /// Stable machine-readable cause code.
    pub code: String,
    /// Exact source or input identity that first caused the result.
    pub source_id: String,
    /// Profile and bound context selected for the result.
    pub context: String,
}

impl KaniOutcome {
    /// Constructs a proof result.
    pub fn proved(source_id: impl Into<String>, context: impl Into<String>) -> Self {
        Self::new(KaniOutcomeKind::Proved, "kani_proved", source_id, context)
    }

    /// Constructs a concrete counterexample result.
    pub fn counterexample(source_id: impl Into<String>, context: impl Into<String>) -> Self {
        Self::new(
            KaniOutcomeKind::Counterexample,
            "kani_counterexample",
            source_id,
            context,
        )
    }

    /// Maps a SUCCESS-check count to a proof result. A `Proved` run backed
    /// by zero SUCCESS checks proved nothing — no check in the obligation
    /// actually ran — so it settles the existing `Inconclusive` kind under
    /// the typed cause `kani_vacuous_proof` rather than manufacturing a new
    /// terminal kind or reporting `Proved`.
    ///
    /// This function maps a check count to an outcome; it does not itself
    /// observe or run anything. No caller in this repository's `src/` or
    /// `crates/` routes a Kani run through it yet.
    pub fn proved_from_checks(
        success_checks: usize,
        source_id: impl Into<String>,
        context: impl Into<String>,
    ) -> Self {
        if success_checks == 0 {
            return Self::non_success(
                KaniOutcomeKind::Inconclusive,
                "kani_vacuous_proof",
                source_id,
                context,
            );
        }
        Self::proved(source_id, context)
    }

    /// Constructs a typed non-success result.
    pub fn non_success(
        kind: KaniOutcomeKind,
        code: impl Into<String>,
        source_id: impl Into<String>,
        context: impl Into<String>,
    ) -> Self {
        match kind {
            KaniOutcomeKind::Proved | KaniOutcomeKind::Counterexample => Self::new(
                KaniOutcomeKind::Refused,
                "kani_outcome_kind_invalid",
                source_id,
                context,
            ),
            _ => Self::new(kind, code, source_id, context),
        }
    }

    fn new(
        kind: KaniOutcomeKind,
        code: impl Into<String>,
        source_id: impl Into<String>,
        context: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            code: code.into(),
            source_id: source_id.into(),
            context: context.into(),
        }
    }

    /// The sole Boolean claim represented by this outcome.
    #[must_use]
    pub const fn boolean_claim(&self) -> Option<bool> {
        match self.kind {
            KaniOutcomeKind::Proved => Some(true),
            KaniOutcomeKind::Counterexample => Some(false),
            KaniOutcomeKind::Refused
            | KaniOutcomeKind::InvalidInput
            | KaniOutcomeKind::IncompleteInput
            | KaniOutcomeKind::Unavailable
            | KaniOutcomeKind::TimedOut
            | KaniOutcomeKind::ResourceExhausted
            | KaniOutcomeKind::Cancelled
            | KaniOutcomeKind::Inconclusive => None,
        }
    }
}
