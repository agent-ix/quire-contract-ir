//! Concrete counterexample serialization and strict native-replay agreement.

use quire_spec_language::{
    package::NativePackage,
    runtime::{self, ExecutionLimits, ExecutionReport, ExecutionSelection, RuntimeInput},
};
use serde::{Deserialize, Serialize};

use super::{FiniteInput, KaniOutcome, KaniOutcomeKind, Witness, WitnessCheck};

/// Retained concrete counterexample, never a proof or generic non-success result.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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
}

/// Agreement produced by replaying through QSL's independently implemented
/// native reference runtime.
#[derive(Debug)]
pub struct NativeReplayAgreement<'package, 'model> {
    /// The exact portable counterexample packet that was replayed.
    pub packet: CounterexamplePacket,
    /// The retained native execution report, including its original request.
    pub native: ExecutionReport<'package, 'model>,
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
        let structurally_invalid = witness.check == WitnessCheck::Cover
            || witness.harness_symbol.trim().is_empty()
            || witness.check_text.trim().is_empty()
            || witness.concrete_values.is_empty()
            || witness.concrete_values.iter().any(Vec::is_empty);
        if structurally_invalid {
            return Err(KaniOutcome::non_success(
                KaniOutcomeKind::InvalidInput,
                "kani_replay_witness_invalid",
                packet.input.source_id.clone(),
                packet.profile_revision.clone(),
            ));
        }
    }
    packet.input.clone().validate().map(|_| ())
}

/// Validates and replays a retained counterexample without repairing disagreement.
pub fn replay_counterexample(
    packet: CounterexamplePacket,
    execute_native: impl FnOnce(&FiniteInput) -> KaniOutcome,
) -> Result<ReplayAgreement, KaniOutcome> {
    validate_packet(&packet)?;
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
    Ok(ReplayAgreement { packet, native })
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
    Ok(NativeReplayAgreement { packet, native })
}
