//! Concrete counterexample serialization and strict native-replay agreement.

use std::collections::BTreeMap;

use quire_spec_language::{
    package::NativePackage,
    runtime::{self, ExecutionLimits, ExecutionReport, ExecutionSelection, RuntimeInput},
};
use serde::{Deserialize, Serialize};

use super::{FiniteInput, KaniOutcome, KaniOutcomeKind, Witness, WitnessCheck, WitnessValue};

/// Where a replayed counterexample's input comes from.
///
/// A packet holds exactly one arm, never both, and the arm is the fact: no
/// packet or replay result stores a second field restating which arm it is
/// (see the AD-016 "Replay source" section). `Witness` replay yields
/// `reproduced-with-evaluated-witness` on agreement; `Input` replay yields
/// `reproduced-without-witness`, and can never be evidence of a backend
/// counterexample reproducing natively, because it reproduces an answer
/// supplied with its own input.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplaySource {
    /// The evaluated Kani concrete-playback witness that backs this
    /// counterexample, admitted only through [`Witness::parse`].
    Witness(Witness),
    /// Canonical input assignments for a counterexample that did not come
    /// from a backend transcript (a corpus counterexample): one concrete
    /// value per declared parameter identifier, with no backend transcript
    /// stored beside it.
    Input(BTreeMap<String, WitnessValue>),
}

/// Retained concrete counterexample, never a proof or generic non-success result.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CounterexamplePacket {
    /// Exact profile revision.
    pub profile_revision: String,
    /// Exact finite ABI input that produced the counterexample.
    pub input: FiniteInput,
    /// Where this counterexample's input comes from: a backend witness, or
    /// stored canonical assignments with no witness at all. Exactly one arm.
    pub source: ReplaySource,
}

/// Settled by replaying a packet's `Witness` arm and agreeing with it:
/// `reproduced-with-evaluated-witness`. This is the only arm from which a
/// backend-evidence verdict can ever be built (AD-016 "Replay ownership").
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WitnessReplayAgreement {
    /// The exact portable counterexample packet that was replayed; its
    /// `source` is always [`ReplaySource::Witness`].
    pub packet: CounterexamplePacket,
    /// The independently produced native result that agreed.
    pub native: KaniOutcome,
}

/// Settled by replaying a packet's `Input` arm and agreeing with it:
/// `reproduced-without-witness`. An `Input` replay reproduces an answer
/// supplied with its own input, so agreement here can never show that a
/// backend counterexample reproduces natively: this type carries no
/// [`Witness`] anywhere and cannot be turned into a backend-evidence
/// verdict.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InputReplayAgreement {
    /// The exact portable counterexample packet that was replayed; its
    /// `source` is always [`ReplaySource::Input`].
    pub packet: CounterexamplePacket,
    /// The independently produced native result that agreed.
    pub native: KaniOutcome,
}

/// Result of replaying an exact packet through an independently supplied
/// native executor: a sum of the two distinct arm result types (AD-016
/// "Replay result"). The arm is the fact; neither variant carries a
/// second, separately stored flag that restates it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReplayAgreement {
    /// Reproduced together with an evaluated witness.
    Witness(WitnessReplayAgreement),
    /// Reproduced with no witness at all.
    Input(InputReplayAgreement),
}

/// Agreement produced by replaying through QSL's independently implemented
/// native reference runtime, settled by a packet's `Witness` arm.
/// See [`WitnessReplayAgreement`].
#[derive(Debug)]
pub struct WitnessNativeReplayAgreement<'package, 'model> {
    /// The exact portable counterexample packet that was replayed; its
    /// `source` is always [`ReplaySource::Witness`].
    pub packet: CounterexamplePacket,
    /// The retained native execution report, including its original request.
    pub native: ExecutionReport<'package, 'model>,
}

/// Agreement produced by replaying through QSL's independently implemented
/// native reference runtime, settled by a packet's `Input` arm.
/// See [`InputReplayAgreement`].
#[derive(Debug)]
pub struct InputNativeReplayAgreement<'package, 'model> {
    /// The exact portable counterexample packet that was replayed; its
    /// `source` is always [`ReplaySource::Input`].
    pub packet: CounterexamplePacket,
    /// The retained native execution report, including its original request.
    pub native: ExecutionReport<'package, 'model>,
}

/// Agreement produced by replaying through QSL's independently implemented
/// native reference runtime: a sum of the two distinct arm result types,
/// mirroring [`ReplayAgreement`].
#[derive(Debug)]
pub enum NativeReplayAgreement<'package, 'model> {
    /// Reproduced together with an evaluated witness.
    Witness(WitnessNativeReplayAgreement<'package, 'model>),
    /// Reproduced with no witness at all.
    Input(InputNativeReplayAgreement<'package, 'model>),
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
    if let ReplaySource::Witness(witness) = &packet.source {
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
        // built directly by `Deserialize` — with a transcript that is a
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
        // as agreement despite never having reproduced what Kani actually
        // recorded.
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
    // The arm is decided by matching `packet.source` (by reference, so
    // `packet` itself stays intact to move whole into the settled arm
    // result below); no separate flag records which arm was taken.
    Ok(match packet.source {
        ReplaySource::Witness(_) => {
            ReplayAgreement::Witness(WitnessReplayAgreement { packet, native })
        }
        ReplaySource::Input(_) => ReplayAgreement::Input(InputReplayAgreement { packet, native }),
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
    Ok(match packet.source {
        ReplaySource::Witness(_) => {
            NativeReplayAgreement::Witness(WitnessNativeReplayAgreement { packet, native })
        }
        ReplaySource::Input(_) => {
            NativeReplayAgreement::Input(InputNativeReplayAgreement { packet, native })
        }
    })
}
