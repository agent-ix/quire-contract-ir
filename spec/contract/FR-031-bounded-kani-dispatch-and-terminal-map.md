---
id: FR-031
title: "Dispatch bounded Kani modules and map outcomes to QSL terminal records"
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
# FR-031: Dispatch bounded Kani modules and map outcomes to QSL terminal records

## Description

The bounded-Kani boundary shall dispatch semantic families modularly and map
each Kani outcome to one QSL `qsl_replay::TerminalValue`. The family lowerings
behind the dispatch index are codegen's backend adapter's, not Contract IR's.
Replay of a
counterexample is not a Contract IR operation: the counterexample envelope,
the witness, the replay source, the FR-331 terminal record and the obligation
identity are QSL's `qsl-replay` types, and replay runs through the QSL layer-6
facade `qsl_replay::replay` (QSL ADR-011 E9, ADR-013 TK-01), which the codegen
replay adapter calls.

## Inputs

A validated bounded execution input (FR-030) and the selected `kani-bounded/1`
profile and capability matrix (FR-029).

## Outputs

A dispatch route and typed `KaniOutcome`, and the one
`qsl_replay::TerminalValue` the outcome records.

## Behavior

The dispatch index is the only cross-module vocabulary and routing authority. Contract IR keeps the profile, the finite input ABI, the dispatch index and its module descriptors, and the typed outcome; the checked-arithmetic, collection and object lowerings each family module performs belong to the backend adapter in codegen, and the root crate carries no family lowering. The index selects independently versioned modules for definedness/checked arithmetic, finite object/reference/graph semantics, and bounded collection/query semantics. Each module declares constructs it owns, exact input/output ABI revision, definedness dependencies, resource charges, and supported/refused/unsupported cases (FR-029). Modules cannot invent source meaning, reinterpret another module's values, or silently substitute structural equality for identity, a collection set for an ordered duplicate-preserving sequence, or bounded graph search for unbounded reachability.

The bounded-Kani boundary shall map Kani outcomes to QSL `TerminalValue`s
as follows, following QSL ADR-013 O-16's proof column and QSpec FR-331:

| Outcome | `TerminalValue` |
| --- | --- |
| `Proved`, carrying its SUCCESS check count `n` (at least one, FR-030) | `Proved { success_checks: n }` |
| `Inconclusive` with cause `kani_vacuous_proof` (a proof with zero SUCCESS checks, FR-030) | `Proved { success_checks: 0 }` |
| `Counterexample` | `Refuted` |
| `Refused`, `InvalidInput`, `IncompleteInput` | `Declined` with `ProofRefusalCause::Refused`, `InvalidInput`, `IncompleteInput` |
| `TimedOut`, `ResourceExhausted`, `Cancelled` | `Incomplete` with `IncompleteCause::TimedOut`, `ResourceExhausted`, `Cancelled` |
| `Unavailable` with cause `kani_solver_absent` (FR-030) | `Unsupported(UnavailabilityCause::SolverAbsent)` |
| `Unavailable` with cause `kani_backend_absent` (FR-030) | `Unsupported(UnavailabilityCause::BackendAbsent)` |

QSL reads `Proved { success_checks: 0 }` as category `Inconclusive` with
`vacuous_proof_cause()` `KaniVacuousProof`, the vacuity record QSpec
FR-331-AC-8 requires. A zero-check run reaches that value only through
FR-030's `kani_vacuous_proof` classification and never through the `Proved`
kind. Because FR-030 admits no other
`Inconclusive` cause and no `Unavailable` cause beyond the two above, the map
is total over `KaniOutcome`. No outcome maps to `Tested` or `Failed`.

A construct the profile has no qualified interpretation for never reaches
this map: FR-029 settles its item `unsupported` at negotiation, with a
warning naming the item's capability kind from the closed `quire.capability-kind/v1` vocabulary (QSpec FR-290), and no `KaniOutcome` or `TerminalValue`
exists for it.

Contract IR defines no counterexample packet, witness, replay source, replay
agreement or terminal-record type of its own, parses no Kani transcript text,
and invokes no QSL executor. The Kani transcript parser is the codegen
backend adapter's.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-031-AC-1 | The shared dispatch index routes definedness/arithmetic, object/reference/graph, and collection/query work through distinct declared modules and rejects cross-family approximation. | Test (TC-042) |
| FR-031-AC-5 | Each outcome in the map's table maps to exactly its listed `TerminalValue`: a proof with three SUCCESS checks to `Proved { success_checks: 3 }`, a `kani_vacuous_proof` outcome to `Proved { success_checks: 0 }`, whose QSL category is `Inconclusive`, `Counterexample` to `Refuted`, the three refusal kinds to `Declined` with three distinct `ProofRefusalCause`s, the three limit kinds to `Incomplete` with three distinct `IncompleteCause`s, and `Unavailable` with `kani_solver_absent` and with `kani_backend_absent` to `Unsupported` with `SolverAbsent` and `BackendAbsent`; the test's `match` over `KaniOutcomeKind` has no wildcard; no outcome maps to `Tested` or `Failed`. | Test (TC-223) |

### Retired criteria

`FR-031-AC-3` required every serialized counterexample to reproduce through a
QSL executor entry with the same outcome and witness. Replay is not a Contract
IR operation: the envelope and replay source are QSL's, and the crossing runs
from the codegen replay adapter through `qsl_replay::replay` (QSL FR-098). Its
test belongs to those owners, so TC-054 is withdrawn from this repository.

`FR-031-AC-4` required an evaluated witness to be parsed from a backend
transcript and typed against a generator-declared schema. `Witness` is QSL's
`qsl_replay::Witness` (QSL FR-070) and the transcript parser is the codegen
backend adapter's, so Contract IR carries neither. TC-221 is withdrawn; its
tests remain in `tests/it/kani_replay.rs` until `src/kani/witness.rs` is
deleted with them.

## Dependencies

[FR-029](./FR-029-versioned-bounded-kani-profile.md) selects module and
artifact ABI versions. [FR-030](./FR-030-bounded-kani-domain-and-outcomes.md)
defines validated inputs and typed non-Boolean outcomes. QSL `qsl-replay`
owns `TerminalValue`, `TerminalRecord`, `Witness`, `ReplaySource`,
`WitnessEnvelope`, `ObligationIdentity` and `replay` (QSL FR-069, FR-070,
FR-098). [FR-039](../interface/FR-039-root-crate-public-interface.md) states
the root crate's public surface after those types leave it.

## Status

AC-1 is implemented. The dispatch index routes the three families through
distinct modules, refusing an unowned construct and a duplicate module owner;
it does not itself prove that a module cannot approximate another family's
structural equality, collection-set, or bounded graph search semantics.

AC-5 is planned against QSL's `TerminalValue`. Today
`KaniOutcomeKind::provider_result`
maps to this crate's own `KaniProviderResult` and maps every `Unavailable`
to one `Unsupported` value without reading its cause; that type and
`KaniProviderRecord` are not part of the root crate's interface (FR-039).
The test at `tests/it/kani_shared.rs:250` is tagged FR-031-AC-5 and TC-223
but verifies the retired `KaniProviderResult` map, so that tag is stale and
does not back AC-5.
The family lowerings `src/kani/arithmetic.rs`, `collections.rs` and
`objects.rs` are in this crate today and are codegen's by this requirement.
