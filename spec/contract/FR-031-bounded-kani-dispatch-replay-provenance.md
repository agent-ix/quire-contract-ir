---
id: FR-031
title: "Dispatch bounded Kani modules with provenance and map outcomes to QSL terminal records"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-029
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-030
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/AD-001
    type: references
  - target: ix://agent-ix/quire-specification/AD-016
    type: references
  - target: ix://agent-ix/quire-specification/FR-331
    type: references
  - target: ix://agent-ix/quire-spec-language/ADR-011
    type: references
  - target: ix://agent-ix/quire-spec-language/ADR-013
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-069
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-070
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-098
    type: references
---
# FR-031: Dispatch bounded Kani modules with provenance and map outcomes to QSL terminal records

## Description

The bounded-Kani boundary shall dispatch semantic families modularly, preserve
artifact identity through oracle/strategy/harness generation, and map every
Kani outcome to exactly one QSL `qsl_replay::TerminalValue`. Replay of a
counterexample is not a Contract IR operation: the counterexample envelope,
the witness, the replay source, the FR-331 terminal record and the obligation
identity are QSL's `qsl-replay` types, and replay runs through the QSL layer-6
facade `qsl_replay::replay` (QSL ADR-011 E9, ADR-013 TK-01), which the codegen
replay adapter calls.

## Inputs

A validated bounded execution input (FR-030), the selected `kani-bounded/1`
profile and capability matrix (FR-029), and the checked-clause and model
identities the generated artifacts bind.

## Outputs

A typed `KaniOutcome` with its provenance, and, for that outcome, the one
`qsl_replay::TerminalValue` it records.

## Behavior

The dispatch index is the only cross-module vocabulary and routing authority. It selects independently versioned modules for definedness/checked arithmetic, finite object/reference/graph semantics, and bounded collection/query semantics. Each module declares constructs it owns, exact input/output ABI revision, definedness dependencies, resource charges, and supported/refused/inconclusive cases. Modules cannot invent source meaning, reinterpret another module's values, or silently substitute structural equality for identity, a collection set for an ordered duplicate-preserving sequence, or bounded graph search for unbounded reachability.

Oracle, strategy, lowering, and harness generators are explicit interfaces with content identities. Their generated artifacts bind the checked-clause identity, profile selection, complete input identity, module identities, Kani executable digest/options, declared assumptions, and proof dependencies. Any change in a bound, assumption, selected module, tool/options digest, source/model/snapshot identity, or generator bytes changes the artifact identity.

Each Kani outcome kind maps to exactly one QSL `TerminalValue` through one
exhaustive map with no fallback arm, following QSL ADR-013 O-16's proof
column: `Proved` maps to `Proved { success_checks }` carrying the run's
SUCCESS check count; `Counterexample` maps to `Refuted`; `Refused`,
`InvalidInput` and `IncompleteInput` map to `Declined` with
`ProofRefusalCause::Refused`, `InvalidInput` and `IncompleteInput`
respectively; `TimedOut`, `ResourceExhausted` and `Cancelled` map to
`Incomplete` with `IncompleteCause::TimedOut`, `ResourceExhausted` and
`Cancelled` respectively. A proof with zero SUCCESS checks maps to
`Proved { success_checks: 0 }`, which QSL reads as `inconclusive` with cause
`KaniVacuousProof`. No kind maps to `Tested` or `Failed`. The targets of
`Unavailable` and of a capability-declared `Inconclusive` are the open
question recorded in [AD-001](../assurance/AD-001-contract-ir-architecture.md)
"Open questions" (OQ-3); until it is decided, those two kinds have no
`TerminalValue`.

Contract IR defines no counterexample packet, witness, replay source, replay
agreement or terminal-record type of its own, parses no Kani transcript text,
and invokes no QSL executor. The Kani transcript parser is the codegen
backend adapter's.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-031-AC-1 | The shared dispatch index routes definedness/arithmetic, object/reference/graph, and collection/query work through distinct declared modules and rejects cross-family approximation. | Test (TC-042) |
| FR-031-AC-2 | Every generated lowering, oracle, strategy, harness, proof, and result has exact provenance binding Kani version/digest/options, assumptions, bounds, inputs, modules, and dependencies. | Test (TC-042) |
| FR-031-AC-5 | Each of the eight Kani outcome kinds with a decided target maps to exactly the one QSL `TerminalValue` the map above gives it (`Proved { success_checks }`, `Refuted`, `Declined` ×3 with distinct `ProofRefusalCause`s, `Incomplete` ×3 with distinct `IncompleteCause`s), through one exhaustive map; a zero-check proof maps to `Proved { success_checks: 0 }`; and no kind maps to `Tested` or `Failed`. | Test (TC-223) |

### Retired criteria

`FR-031-AC-3` required every serialized counterexample to reproduce through a
QSL executor entry with the same outcome and witness. Replay is not a Contract
IR operation: the envelope and replay source are QSL's, and the crossing runs
from the codegen replay adapter through `qsl_replay::replay` (QSL FR-098). Its
test belongs to those owners, so TC-054 is withdrawn from this repository.

`FR-031-AC-4` required an evaluated witness to be parsed from a backend
transcript and typed against a generator-declared schema. `Witness` is QSL's
`qsl_replay::Witness` (QSL FR-070) and the transcript parser is the codegen
backend adapter's, so Contract IR carries neither. TC-221 verifies the
behaviour while `src/kani/witness.rs` remains and is withdrawn with it.

## Dependencies

[FR-029](./FR-029-versioned-bounded-kani-profile.md) selects module and
artifact ABI versions. [FR-030](./FR-030-bounded-kani-domain-and-outcomes.md)
defines validated inputs and typed non-Boolean outcomes. QSL `qsl-replay`
owns `TerminalValue`, `TerminalRecord`, `Witness`, `ReplaySource`,
`WitnessEnvelope`, `ObligationIdentity` and `replay` (QSL FR-069, FR-070,
FR-098). [FR-039](../interface/FR-039-root-crate-public-interface.md) states
the root crate's public surface after those types leave it.

## Status

AC-1 and AC-2 are implemented and qualified against the integrated codegen
corpus. The dispatch index routes the three families through distinct
modules, refusing an unowned construct and a duplicate module owner; it does
not itself prove that a module cannot approximate another family's
structural equality, collection-set, or bounded graph search semantics.
`GeneratorProvenance` and `ArtifactIdentity` (`src/kani/provenance.rs`) are
identity-sensitive, and the test tagged AC-2 mutates a single assumption and
confirms the digest changes; no generator yet emits one, because
`ArithmeticLowering` and the object/collection lowerings carry no provenance
field. That evidence is not a general proof-engine or release claim.

AC-5 is planned against QSL's `TerminalValue`. Today
`KaniOutcomeKind::provider_result` maps to this crate's own
`KaniProviderResult`, which TC-223 verifies; that type and
`KaniProviderRecord` are removed when the map targets `TerminalValue`.
