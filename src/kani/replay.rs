//! Concrete counterexample serialization and strict native-replay agreement.

use quire_spec_language::{
    package::NativePackage,
    runtime::{self, ExecutionLimits, ExecutionReport, ExecutionSelection, RuntimeInput},
};
use serde::{Deserialize, Serialize};

use super::{FiniteInput, KaniOutcome, KaniOutcomeKind, Witness, WitnessCheck};

/// Retained concrete counterexample, never a proof or generic non-success result.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CounterexamplePacket {
    /// Exact profile revision.
    pub profile_revision: String,
    /// Exact finite ABI input that produced the witness.
    pub input: FiniteInput,
    /// The evaluated Kani concrete-playback witness that backs this
    /// counterexample, when a Kani backend produced one. `None` is a real,
    /// honestly modeled state: a corpus counterexample that never ran Kani
    /// retains no backend transcript, and must not fabricate one.
    pub witness: Option<Witness>,
}

/// Result of replaying an exact packet through an independently supplied native executor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayAgreement {
    pub packet: CounterexamplePacket,
    pub native: KaniOutcome,
    /// Whether this agreement was reproduced together with an evaluated
    /// witness (`packet.witness` was `Some` and passed structural
    /// validation — its harness symbol, check kind, check text, and concrete
    /// values were all re-derived from `transcript` and found non-`Cover`,
    /// non-empty, and self-consistent), as opposed to reproduced with no
    /// witness at all. `ReplayAgreement`'s "same outcome" is always
    /// established; this records which half of "same outcome, same
    /// witness" was also established.
    pub witness_backed: bool,
}

/// Agreement produced by replaying through QSL's independently implemented
/// native reference runtime.
#[derive(Debug)]
pub struct NativeReplayAgreement<'package, 'model> {
    /// The exact portable counterexample packet that was replayed.
    pub packet: CounterexamplePacket,
    /// The retained native execution report, including its original request.
    pub native: ExecutionReport<'package, 'model>,
    /// See [`ReplayAgreement::witness_backed`].
    pub witness_backed: bool,
}

fn validate_packet(packet: &CounterexamplePacket) -> Result<(), KaniOutcome> {
    if packet.profile_revision.trim().is_empty()
        || packet.profile_revision != packet.input.profile.revision
    {
        return Err(KaniOutcome::non_success(
            KaniOutcomeKind::InvalidInput,
            "kani_replay_packet_invalid",
            packet.input.source_id.clone(),
            packet.profile_revision.clone(),
        ));
    }
    if let Some(witness) = &packet.witness {
        let invalid = || {
            KaniOutcome::non_success(
                KaniOutcomeKind::InvalidInput,
                "kani_replay_witness_invalid",
                packet.input.source_id.clone(),
                packet.profile_revision.clone(),
            )
        };
        // F1/F2: `harness_symbol`, `check`, and `check_text` are re-derived
        // from `witness.transcript` on every call (see the `witness.rs`
        // module doc) rather than trusted stored fields, so a `Witness`
        // built directly by `Deserialize` — with a `transcript` that is a
        // verbatim cover playback block, or no playback block at all —
        // cannot disagree with its own transcript here. There is no `check`
        // field left to bypass this re-derivation with.
        let check = witness.check().map_err(|_| invalid())?;
        let harness_symbol = witness.harness_symbol().map_err(|_| invalid())?;
        let check_text = witness.check_text().map_err(|_| invalid())?;
        if check == WitnessCheck::Cover
            || harness_symbol.trim().is_empty()
            || check_text.trim().is_empty()
        {
            return Err(invalid());
        }
        // Re-parses `transcript` rather than trusting a separately retained
        // copy of the concrete bytes: `transcript` is the single source of
        // truth (see `witness.rs` module doc), and this is the only place
        // the shipped replay path re-derives from it. A zero-binding
        // witness (F9) legitimately parses to an empty, valid result.
        //
        // F5: also cross-checks each concrete value against its own `//`
        // decoded-value comment wherever its byte width unambiguously
        // implies a value kind, with no schema in hand — otherwise a
        // transcript whose bytes contradict their own comment would replay
        // as `witness_backed = true` despite never having reproduced what
        // Kani actually recorded.
        witness.validate_concrete_entries().map_err(|_| invalid())?;
    }
    packet.input.clone().validate().map(|_| ())
}

/// Validates and replays a retained counterexample without repairing disagreement.
pub fn replay_counterexample(
    packet: CounterexamplePacket,
    execute_native: impl FnOnce(&FiniteInput) -> KaniOutcome,
) -> Result<ReplayAgreement, KaniOutcome> {
    validate_packet(&packet)?;
    let witness_backed = packet.witness.is_some();
    let validated = packet.input.clone().validate()?;
    let native = execute_native(validated.input());
    if native.kind != KaniOutcomeKind::Counterexample {
        return Err(KaniOutcome::non_success(
            KaniOutcomeKind::Inconclusive,
            "kani_native_replay_disagreement",
            packet.input.source_id,
            packet.profile_revision,
        ));
    }
    Ok(ReplayAgreement {
        packet,
        native,
        witness_backed,
    })
}

/// Reconstructs and executes a counterexample with the native QSL runtime.
///
/// The adapter is responsible for reconstructing the typed QSL request from
/// the packet's retained ABI input. This boundary always invokes
/// [`runtime::execute`] itself and accepts an agreement only for a completed
/// native `false`; validation failures, incomplete native execution, and a
/// native proof are all surfaced as a non-Boolean result.
pub fn replay_with_native_runtime<'package, 'model>(
    packet: CounterexamplePacket,
    package: &'package NativePackage<'model>,
    reconstruct: impl FnOnce(&CounterexamplePacket) -> (RuntimeInput, ExecutionSelection),
    limits: ExecutionLimits,
) -> Result<NativeReplayAgreement<'package, 'model>, KaniOutcome> {
    validate_packet(&packet)?;
    let witness_backed = packet.witness.is_some();
    let (input, selection) = reconstruct(&packet);
    let native = runtime::execute(package, input, selection, limits, || false);
    if native.truth() != Some(false) {
        return Err(KaniOutcome::non_success(
            KaniOutcomeKind::Inconclusive,
            "kani_native_replay_disagreement",
            packet.input.source_id,
            packet.profile_revision,
        ));
    }
    Ok(NativeReplayAgreement {
        packet,
        native,
        witness_backed,
    })
}
