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

impl KaniOutcomeKind {
    /// The QSpec FR-331 terminal result a Kani run ending in this kind
    /// records: QSL ADR-013's O-16 proof column, implemented here as its one
    /// total map (C-09). It reads the kind alone; [`KaniOutcome::provider_record`]
    /// pairs the result with the outcome's typed cause.
    ///
    /// O-16 records `proved` only for a proof with at least one SUCCESS check.
    /// That is a precondition on how the outcome was built, not something
    /// this map can see: [`KaniOutcome::proved_from_checks`] turns a
    /// zero-check proof into `Inconclusive` before it gets here, while
    /// [`KaniOutcome::proved`] records whatever its caller asserts.
    pub const fn provider_result(&self) -> KaniProviderResult {
        match self {
            Self::Proved => KaniProviderResult::Proved,
            Self::Counterexample => KaniProviderResult::Refuted,
            Self::Refused | Self::InvalidInput | Self::IncompleteInput => {
                KaniProviderResult::Declined
            }
            Self::Unavailable => KaniProviderResult::Unsupported,
            Self::TimedOut | Self::ResourceExhausted | Self::Cancelled => {
                KaniProviderResult::Incomplete
            }
            Self::Inconclusive => KaniProviderResult::Inconclusive,
        }
    }
}

/// The QSpec FR-331 `results` values a Kani run can record. FR-331's other
/// two values, `tested` and `failed`, are not produced by any Kani outcome
/// kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KaniProviderResult {
    /// `proved`: the obligation was proved.
    Proved,
    /// `refuted`: a concrete counterexample was found.
    Refuted,
    /// `declined`: the run refused the request; the item keeps its
    /// `supported` disposition.
    Declined,
    /// `unsupported`: the solver or backend was absent after negotiation.
    Unsupported,
    /// `incomplete`: a timeout, a cancellation or an exhausted resource.
    Incomplete,
    /// `inconclusive`: no qualified interpretation, including a vacuous proof.
    Inconclusive,
}

/// One FR-331 terminal record for a Kani run: the result together with the
/// outcome's typed cause, so outcomes that share a result stay
/// distinguishable (the three refusal kinds inside `declined`, the three
/// limit kinds inside `incomplete`).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KaniProviderRecord {
    /// The terminal result.
    pub result: KaniProviderResult,
    /// The outcome's stable machine-readable cause code, unchanged.
    pub cause: Std001Code,
}

/// A request to build an outcome that breaks a constructor rule.
///
/// Its [`code`](Self::code) is `kani_outcome_invalid` (STD-001). No outcome is
/// built and no other kind or cause is substituted.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
#[error("kani_outcome_invalid: a `proved` or `counterexample` outcome is not built as a non-success outcome")]
pub struct KaniOutcomeError(());

impl KaniOutcomeError {
    /// The code of this error: `kani_outcome_invalid`.
    #[must_use]
    pub const fn code(&self) -> Std001Code {
        Std001Code::KANI_OUTCOME_INVALID
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
    /// This outcome's FR-331 terminal record: [`KaniOutcomeKind::provider_result`]
    /// of its kind, with its cause code carried unchanged.
    pub fn provider_record(&self) -> KaniProviderRecord {
        KaniProviderRecord {
            result: self.kind.provider_result(),
            cause: self.code,
        }
    }

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
            return Self::new(
                KaniOutcomeKind::Inconclusive,
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
        match kind {
            KaniOutcomeKind::Proved | KaniOutcomeKind::Counterexample => Err(KaniOutcomeError(())),
            KaniOutcomeKind::Refused
            | KaniOutcomeKind::InvalidInput
            | KaniOutcomeKind::IncompleteInput
            | KaniOutcomeKind::Unavailable
            | KaniOutcomeKind::TimedOut
            | KaniOutcomeKind::ResourceExhausted
            | KaniOutcomeKind::Cancelled
            | KaniOutcomeKind::Inconclusive => Ok(Self::new(kind, code, source_id, context)),
        }
    }

    /// Builds an outcome without checking `kind`: for the `kani` module's own
    /// raise sites, whose kind is a literal non-success kind.
    pub(super) fn new(
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
