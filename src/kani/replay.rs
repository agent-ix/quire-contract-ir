//! Concrete counterexample serialization and strict native-replay agreement.

use std::collections::BTreeMap;

use quire_spec_language::{
    package::NativePackage,
    runtime::{self, ExecutionLimits, ExecutionReport, ExecutionSelection, RuntimeInput},
};
use serde::{Deserialize, Serialize};

use super::{FiniteInput, KaniOutcome, KaniOutcomeKind, Witness, WitnessValue};

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
    ///
    /// AD-016 "Replay source" specifies keying this by parameter
    /// `WireNodeId` (QSL ADR-013 C-11). `WireNodeId` does not exist in this
    /// codebase or in the pinned `quire-spec-language` dependency at this
    /// revision, so this keys by the declared parameter identifier instead.
    /// This is a deviation, not the intended design, recorded here so it is
    /// not later mistaken for intent.
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

/// The part of a counterexample packet that does not vary by arm: the exact
/// profile revision and finite ABI input that produced it. Every arm result
/// below holds one of these instead of a whole [`CounterexamplePacket`], so
/// an arm result cannot hold a [`ReplaySource`] and which arm it is is never
/// recorded a second time (AD-016 "Replay source": "Each arm's result type
/// holds that arm's payload and never a `ReplaySource`.").
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PacketIdentity {
    /// Exact profile revision.
    pub profile_revision: String,
    /// Exact finite ABI input that produced the counterexample.
    pub input: FiniteInput,
}

impl CounterexamplePacket {
    fn identity(&self) -> PacketIdentity {
        PacketIdentity {
            profile_revision: self.profile_revision.clone(),
            input: self.input.clone(),
        }
    }
}

/// Settled by replaying a packet's `Witness` arm and agreeing with it:
/// `reproduced-with-evaluated-witness`. This is the only arm from which a
/// backend-evidence verdict can ever be built (AD-016 "Replay ownership").
///
/// `witness` is a plain [`Witness`], not a [`ReplaySource`]: there is no
/// field this type could hold that names an `Input` arm, so which arm this
/// result settled follows from the type of `witness` alone, not from a
/// constructor's discipline. The fields are private and the only
/// constructor, [`WitnessReplayAgreement::new`], is private to this module,
/// so a caller outside `replay.rs` cannot build one at all.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WitnessReplayAgreement {
    identity: PacketIdentity,
    witness: Witness,
    native: KaniOutcome,
}

impl WitnessReplayAgreement {
    /// Builds the settled `Witness`-arm agreement. Only this module's
    /// replay functions call this, with the `Witness` taken directly out of
    /// the packet's `ReplaySource::Witness` arm by the match that selects
    /// this constructor over [`InputReplayAgreement::new`].
    fn new(identity: PacketIdentity, witness: Witness, native: KaniOutcome) -> Self {
        Self {
            identity,
            witness,
            native,
        }
    }

    /// The exact profile revision and finite ABI input that were replayed.
    pub fn identity(&self) -> &PacketIdentity {
        &self.identity
    }

    /// The evaluated witness this agreement reproduced.
    pub fn witness(&self) -> &Witness {
        &self.witness
    }

    /// The independently produced native result that agreed.
    pub fn native(&self) -> &KaniOutcome {
        &self.native
    }
}

/// Settled by replaying a packet's `Input` arm and agreeing with it:
/// `reproduced-without-witness`. An `Input` replay reproduces an answer
/// supplied with its own input, so agreement here can never show that a
/// backend counterexample reproduces natively.
///
/// `input` is the canonical assignment map, not a [`ReplaySource`] and not a
/// [`Witness`]: there is no field on this type that could ever hold a
/// [`Witness`], so this type cannot be turned into a backend-evidence
/// verdict by construction, not by a match arm that happens never to reach
/// it. See [`WitnessReplayAgreement`] for why the private fields and
/// constructor are the guard on who can build one.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InputReplayAgreement {
    identity: PacketIdentity,
    input: BTreeMap<String, WitnessValue>,
    native: KaniOutcome,
}

impl InputReplayAgreement {
    /// Builds the settled `Input`-arm agreement. See
    /// [`WitnessReplayAgreement::new`].
    fn new(
        identity: PacketIdentity,
        input: BTreeMap<String, WitnessValue>,
        native: KaniOutcome,
    ) -> Self {
        Self {
            identity,
            input,
            native,
        }
    }

    /// The exact profile revision and finite ABI input that were replayed.
    pub fn identity(&self) -> &PacketIdentity {
        &self.identity
    }

    /// The canonical assignments this agreement reproduced.
    pub fn input(&self) -> &BTreeMap<String, WitnessValue> {
        &self.input
    }

    /// The independently produced native result that agreed.
    pub fn native(&self) -> &KaniOutcome {
        &self.native
    }
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
/// See [`WitnessReplayAgreement`] for why the private fields and constructor
/// are the guard, and why `witness` (not a [`ReplaySource`]) is what makes
/// the arm follow from the type.
#[derive(Debug)]
pub struct WitnessNativeReplayAgreement<'package, 'model> {
    identity: PacketIdentity,
    witness: Witness,
    native: ExecutionReport<'package, 'model>,
}

impl<'package, 'model> WitnessNativeReplayAgreement<'package, 'model> {
    /// Builds the settled `Witness`-arm agreement. See
    /// [`WitnessReplayAgreement::new`].
    fn new(
        identity: PacketIdentity,
        witness: Witness,
        native: ExecutionReport<'package, 'model>,
    ) -> Self {
        Self {
            identity,
            witness,
            native,
        }
    }

    /// The exact profile revision and finite ABI input that were replayed.
    pub fn identity(&self) -> &PacketIdentity {
        &self.identity
    }

    /// The evaluated witness this agreement reproduced.
    pub fn witness(&self) -> &Witness {
        &self.witness
    }

    /// The retained native execution report, including its original request.
    pub fn native(&self) -> &ExecutionReport<'package, 'model> {
        &self.native
    }
}

/// Agreement produced by replaying through QSL's independently implemented
/// native reference runtime, settled by a packet's `Input` arm.
/// See [`InputReplayAgreement`] for why the private fields and constructor
/// are the guard, and why `input` (not a [`ReplaySource`]) is what makes the
/// arm follow from the type.
#[derive(Debug)]
pub struct InputNativeReplayAgreement<'package, 'model> {
    identity: PacketIdentity,
    input: BTreeMap<String, WitnessValue>,
    native: ExecutionReport<'package, 'model>,
}

impl<'package, 'model> InputNativeReplayAgreement<'package, 'model> {
    /// Builds the settled `Input`-arm agreement. See
    /// [`WitnessReplayAgreement::new`].
    fn new(
        identity: PacketIdentity,
        input: BTreeMap<String, WitnessValue>,
        native: ExecutionReport<'package, 'model>,
    ) -> Self {
        Self {
            identity,
            input,
            native,
        }
    }

    /// The exact profile revision and finite ABI input that were replayed.
    pub fn identity(&self) -> &PacketIdentity {
        &self.identity
    }

    /// The canonical assignments this agreement reproduced.
    pub fn input(&self) -> &BTreeMap<String, WitnessValue> {
        &self.input
    }

    /// The retained native execution report, including its original request.
    pub fn native(&self) -> &ExecutionReport<'package, 'model> {
        &self.native
    }
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
        // `harness_symbol` and `check_text` are re-derived from
        // `witness.transcript` on every call (see the `witness.rs` module
        // doc) rather than trusted stored fields, so a `Witness` — including
        // one built through `Deserialize`, which now routes through
        // `Witness::parse` the same as direct construction — cannot disagree
        // with its own transcript here.
        //
        // `Witness::parse` is the only admission path and already refuses a
        // transcript whose selected block names a `cover` check
        // (`kani_witness_cover_refused`), so `witness.check()` can never
        // return `WitnessCheck::Cover` here; that disjunct would be dead
        // code and is not checked. It does not, however, refuse an
        // admitted block whose harness-name backticks or check-text quotes
        // are empty (`` `` `` or `: ""`), so an empty `harness_symbol` or
        // `check_text` is still reachable post-admission and both are still
        // refused here.
        let harness_symbol = witness.harness_symbol().map_err(|_| invalid())?;
        let check_text = witness.check_text().map_err(|_| invalid())?;
        if harness_symbol.trim().is_empty() || check_text.trim().is_empty() {
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
    let identity = packet.identity();
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
    // The arm is decided by matching `packet.source`, taking its payload by
    // value into the arm-specific result constructor: neither result type
    // can hold the other arm's payload, so there is no separate flag left
    // to record which arm was taken.
    // The arm is decided by matching `packet.source`, taking its payload by
    // value into the arm-specific result constructor: neither result type
    // can hold the other arm's payload, so there is no separate flag left
    // to record which arm was taken.
    Ok(match packet.source {
        ReplaySource::Witness(witness) => {
            ReplayAgreement::Witness(WitnessReplayAgreement::new(identity, witness, native))
        }
        ReplaySource::Input(assignments) => {
            ReplayAgreement::Input(InputReplayAgreement::new(identity, assignments, native))
        }
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
    let identity = packet.identity();
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
        ReplaySource::Witness(witness) => NativeReplayAgreement::Witness(
            WitnessNativeReplayAgreement::new(identity, witness, native),
        ),
        ReplaySource::Input(assignments) => NativeReplayAgreement::Input(
            InputNativeReplayAgreement::new(identity, assignments, native),
        ),
    })
}
