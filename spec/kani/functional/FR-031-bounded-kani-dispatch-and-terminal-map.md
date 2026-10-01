---
id: FR-031
title: "Dispatch bounded Kani modules and expose typed outcomes"
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
# FR-031: Dispatch bounded Kani modules and expose typed outcomes

## Description

The bounded-Kani boundary shall dispatch semantic families modularly and
expose its typed `KaniOutcome`. The map from a `KaniOutcome` to a QSL
`qsl_replay::TerminalValue` is not a Contract IR requirement: it is owned by
`agent-ix/quire-contract-codegen`, tracked there under Linear IR-358, because
the root crate depends on no QSL crate (FR-028) and the map needs both sides.
The family lowerings behind the dispatch index are codegen's backend
adapter's, not Contract IR's. Replay of a
counterexample is not a Contract IR operation: the counterexample envelope,
the witness, the replay source, the FR-331 terminal record and the obligation
identity are QSL's `qsl-replay` types, and replay runs through the QSL layer-6
facade `qsl_replay::replay` (QSL ADR-011 E9, ADR-013 TK-01), which the codegen
replay adapter calls.

## Inputs

A validated bounded execution input (FR-030) and the selected `kani-bounded/1`
profile and capability matrix (FR-029).

## Outputs

A dispatch route and a typed `KaniOutcome`. Contract IR exposes only its own
Kani outcome types; it produces no `qsl_replay::TerminalValue`.

## Behavior

The dispatch index is the only cross-module vocabulary and routing authority. Contract IR keeps the profile, the finite input ABI, the dispatch index and its module descriptors, and the typed outcome; the checked-arithmetic, collection and object lowerings each family module performs belong to the backend adapter in codegen, and the root crate carries no family lowering. The index selects independently versioned modules for definedness/checked arithmetic, finite object/reference/graph semantics, and bounded collection/query semantics. Each module declares constructs it owns, exact input/output ABI revision, definedness dependencies, resource charges, and supported/refused/unsupported cases (FR-029). Modules cannot invent source meaning, reinterpret another module's values, or silently substitute structural equality for identity, a collection set for an ordered duplicate-preserving sequence, or bounded graph search for unbounded reachability.

The map from a `KaniOutcome` to a QSL `TerminalValue`, following QSL ADR-013
O-16's proof column and QSpec FR-331, is owned by
`agent-ix/quire-contract-codegen` and is tracked there under Linear IR-358.
Contract IR states no row of that map. It exposes the outcome kinds and
causes the map reads, as FR-030 defines them: `Proved` with its SUCCESS check
count, `Inconclusive` with cause `kani_vacuous_proof`, `Counterexample`, the
three refusal kinds, the three limit kinds, and `Unavailable` with cause
`kani_solver_absent` or `kani_backend_absent`.

A construct the profile has no qualified interpretation for never becomes
an outcome: FR-029 settles its item `unsupported` at negotiation, with a
warning naming the item's capability kind from the closed `quire.capability-kind/v1` vocabulary (QSpec FR-290), and no `KaniOutcome` exists for it.

Contract IR defines no counterexample packet, witness, replay source, replay
agreement or terminal-record type of its own, parses no Kani transcript text,
and invokes no QSL executor. The Kani transcript parser is the codegen
backend adapter's.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-031-AC-1 | The shared dispatch index routes definedness/arithmetic, object/reference/graph, and collection/query work through distinct declared modules and rejects cross-family approximation. | Test (TC-042) |

## Dependencies

[FR-029](./FR-029-versioned-bounded-kani-profile.md) selects module and
artifact ABI versions. [FR-030](./FR-030-bounded-kani-domain-and-outcomes.md)
defines validated inputs and typed non-Boolean outcomes. QSL `qsl-replay`
owns `TerminalValue`, `TerminalRecord`, `Witness`, `ReplaySource`,
`WitnessEnvelope`, `ObligationIdentity` and `replay` (QSL FR-069, FR-070,
FR-098). [FR-039](./FR-039-root-crate-public-interface.md) states
the root crate's public surface after those types leave it.

## Status

AC-1 is implemented. The dispatch index routes the three families through
distinct modules, refusing an unowned construct and a duplicate module owner;
it does not itself prove that a module cannot approximate another family's
structural equality, collection-set, or bounded graph search semantics.

FR-031-AC-5 is retired and its ID is not reused (ADR-0056). It required only
the `KaniOutcome` to `TerminalValue` map, which moved: the map from a `KaniOutcome` to a QSL `TerminalValue` moved
to `agent-ix/quire-contract-codegen`, tracked there under Linear IR-358.
Today `KaniOutcomeKind::provider_result`
maps to this crate's own `KaniProviderResult`; that type and
`KaniProviderRecord` are not part of the root crate's interface (FR-039).
The family lowerings `src/kani/arithmetic.rs`, `collections.rs` and
`objects.rs` are in this crate today and are codegen's by this requirement.
