//! Closed outcomes at the bounded-Kani boundary.

use quire_contract_model::Std001Code;
use serde::{Deserialize, Serialize};
use thiserror::Error;

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

/// A request to build an outcome that breaks a constructor rule.
///
/// Its [`code`](Self::code) is `kani_outcome_invalid` (STD-001), and it names the
/// kind and cause code that were requested. No outcome is built and no other kind
/// or cause is substituted.
#[derive(Clone, Debug, Eq, PartialEq, Error)]
#[error("kani_outcome_invalid: a {requested_kind:?} outcome with cause {requested_code} is not a non-success outcome")]
pub struct KaniOutcomeError {
    requested_kind: KaniOutcomeKind,
    requested_code: Std001Code,
}

impl KaniOutcomeError {
    /// The code of this error: `kani_outcome_invalid`.
    #[must_use]
    pub const fn code(&self) -> Std001Code {
        Std001Code::KANI_OUTCOME_INVALID
    }

    /// The outcome kind that was requested and refused.
    #[must_use]
    pub const fn requested_kind(&self) -> &KaniOutcomeKind {
        &self.requested_kind
    }

    /// The cause code that was requested with it.
    #[must_use]
    pub const fn requested_code(&self) -> Std001Code {
        self.requested_code
    }
}

/// The outcome kinds that carry no Boolean claim: every [`KaniOutcomeKind`]
/// except `Proved` and `Counterexample`.
///
/// It is the one place that rule is written. `TryFrom<KaniOutcomeKind>` is the
/// validation [`KaniOutcome::non_success`] applies to a request, and the raise
/// sites inside the `kani` module name a variant here, so none of them can
/// express a `proved` or `counterexample` outcome.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum NonSuccessKind {
    Refused,
    InvalidInput,
    IncompleteInput,
    Unavailable,
    TimedOut,
    ResourceExhausted,
    Cancelled,
    Inconclusive,
}

impl NonSuccessKind {
    /// The outcome kind this variant stands for.
    const fn kind(self) -> KaniOutcomeKind {
        match self {
            Self::Refused => KaniOutcomeKind::Refused,
            Self::InvalidInput => KaniOutcomeKind::InvalidInput,
            Self::IncompleteInput => KaniOutcomeKind::IncompleteInput,
            Self::Unavailable => KaniOutcomeKind::Unavailable,
            Self::TimedOut => KaniOutcomeKind::TimedOut,
            Self::ResourceExhausted => KaniOutcomeKind::ResourceExhausted,
            Self::Cancelled => KaniOutcomeKind::Cancelled,
            Self::Inconclusive => KaniOutcomeKind::Inconclusive,
        }
    }
}

impl TryFrom<KaniOutcomeKind> for NonSuccessKind {
    /// The refused kind, handed back.
    type Error = KaniOutcomeKind;

    fn try_from(kind: KaniOutcomeKind) -> Result<Self, Self::Error> {
        match kind {
            KaniOutcomeKind::Proved | KaniOutcomeKind::Counterexample => Err(kind),
            KaniOutcomeKind::Refused => Ok(Self::Refused),
            KaniOutcomeKind::InvalidInput => Ok(Self::InvalidInput),
            KaniOutcomeKind::IncompleteInput => Ok(Self::IncompleteInput),
            KaniOutcomeKind::Unavailable => Ok(Self::Unavailable),
            KaniOutcomeKind::TimedOut => Ok(Self::TimedOut),
            KaniOutcomeKind::ResourceExhausted => Ok(Self::ResourceExhausted),
            KaniOutcomeKind::Cancelled => Ok(Self::Cancelled),
            KaniOutcomeKind::Inconclusive => Ok(Self::Inconclusive),
        }
    }
}

/// Typed output which cannot manufacture a Boolean for a non-success state.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KaniOutcome {
    /// Closed outcome kind.
    pub kind: KaniOutcomeKind,
    /// Stable machine-readable cause code, in the STD-001 form (FR-044). The form
    /// is checked, not the registration or the issuing registry.
    pub code: Std001Code,
    /// Exact source or input identity that first caused the result.
    pub source_id: String,
    /// Profile and bound context selected for the result.
    pub context: String,
}

impl KaniOutcome {
    /// Constructs a proof result.
    pub fn proved(source_id: impl Into<String>, context: impl Into<String>) -> Self {
        Self::new(
            KaniOutcomeKind::Proved,
            Std001Code::KANI_PROVED,
            source_id,
            context,
        )
    }

    /// Constructs a concrete counterexample result.
    pub fn counterexample(source_id: impl Into<String>, context: impl Into<String>) -> Self {
        Self::new(
            KaniOutcomeKind::Counterexample,
            Std001Code::KANI_COUNTEREXAMPLE,
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
    /// observe or run anything. This is the one shared implementation of
    /// that rule: `quire-contract-codegen`'s `classify_run`
    /// (`src/kani_execution.rs`) routes its own SUCCESS-check classification
    /// through this function instead of re-deriving the zero-check rule from
    /// its transcript parse (agent-ix/quire-contract-codegen#99). No caller
    /// inside this repository's own `src/` or `crates/` needs to route a
    /// Kani run through it — the caller is external — so within this
    /// repository it is exercised only by `tests/kani_shared.rs`.
    pub fn proved_from_checks(
        success_checks: usize,
        source_id: impl Into<String>,
        context: impl Into<String>,
    ) -> Self {
        if success_checks == 0 {
            return Self::raise(
                NonSuccessKind::Inconclusive,
                Std001Code::KANI_VACUOUS_PROOF,
                source_id,
                context,
            );
        }
        Self::proved(source_id, context)
    }

    /// Constructs a typed non-success result carrying `code`.
    ///
    /// The code is a [`Std001Code`], so a string cannot be passed: a code known at
    /// compile time is a registered constant or `std001_code!`, and one known
    /// at run time is checked by [`Std001Code::new`] first.
    ///
    /// ```compile_fail
    /// use quire_contract_ir::kani::{KaniOutcome, KaniOutcomeKind};
    /// let _ = KaniOutcome::non_success(KaniOutcomeKind::Refused, "kani_x", "source", "context");
    /// ```
    ///
    /// ```compile_fail
    /// use quire_contract_ir::kani::{KaniOutcome, KaniOutcomeKind};
    /// let code = String::from("kani_x");
    /// let _ = KaniOutcome::non_success(KaniOutcomeKind::Refused, code, "source", "context");
    /// ```
    ///
    /// Each probe is paired with the passing call, which names the same
    /// arguments with a `Std001Code`:
    ///
    /// ```
    /// use quire_contract_ir::kani::{KaniOutcome, KaniOutcomeKind};
    /// use quire_contract_model::std001_code;
    /// let outcome = KaniOutcome::non_success(
    ///     KaniOutcomeKind::Refused,
    ///     std001_code!("kani_x"),
    ///     "source",
    ///     "context",
    /// );
    /// assert!(outcome.is_ok());
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`KaniOutcomeError`] when `kind` is `Proved` or `Counterexample`,
    /// which carry a Boolean claim and are built by [`KaniOutcome::proved`] and
    /// [`KaniOutcome::counterexample`] only.
    pub fn non_success(
        kind: KaniOutcomeKind,
        code: Std001Code,
        source_id: impl Into<String>,
        context: impl Into<String>,
    ) -> Result<Self, KaniOutcomeError> {
        match NonSuccessKind::try_from(kind) {
            Ok(kind) => Ok(Self::raise(kind, code, source_id, context)),
            Err(requested_kind) => Err(KaniOutcomeError {
                requested_kind,
                requested_code: code,
            }),
        }
    }

    /// The raise sites of the `kani` module's lowerings and input checks, and the
    /// build step of [`KaniOutcome::non_success`]. It takes a [`NonSuccessKind`],
    /// so a `proved` or `counterexample` request cannot be written at a raise
    /// site and the call has no error path.
    pub(super) fn raise(
        kind: NonSuccessKind,
        code: Std001Code,
        source_id: impl Into<String>,
        context: impl Into<String>,
    ) -> Self {
        Self::new(kind.kind(), code, source_id, context)
    }

    /// Builds the outcome after the caller has chosen a kind and code that agree.
    fn new(
        kind: KaniOutcomeKind,
        code: Std001Code,
        source_id: impl Into<String>,
        context: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            code,
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
