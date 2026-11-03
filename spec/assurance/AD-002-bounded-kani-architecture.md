---
id: AD-002
title: "Bounded Kani backend architecture"
type: ArchitectureDescription
status: accepted
owner: kreneskyp
system: quire-contract-ir bounded Kani backend profile family
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-029
    type: realizes
  - target: ix://agent-ix/quire-contract-ir/FR-030
    type: realizes
  - target: ix://agent-ix/quire-contract-ir/FR-031
    type: realizes
  - target: ix://agent-ix/quire-specification/AD-016
    type: references
  - target: ix://agent-ix/quire-contract-ir/AD-001
    type: references
---
# Bounded Kani backend architecture

## System Boundary

This boundary consumes already checked native clauses and exact finite model inputs. It chooses a bounded Kani profile, validates its input ABI, routes each construct through the dispatch index to a versioned family module, and interprets Kani results as typed outcomes (FR-030). The map from an outcome to a QSL `qsl_replay::TerminalValue` is owned by `agent-ix/quire-contract-codegen` (tracked there under Linear IR-358), and Contract IR exposes only its own Kani outcome types. A construct the profile has no qualified interpretation for settles `unsupported` at negotiation, with a warning naming the item's capability kind from the closed `quire.capability-kind/v1` vocabulary (QSpec FR-290), and produces no artifact or outcome (FR-029). Contract IR holds the profile, input ABI, dispatch index and outcome; the active collection and finite-reference-graph lowerings and harness emission are codegen's. Codegen retired its checked-arithmetic lowerer and corpus arm under FR-015-AC-109; collection and graph remain under FR-015-AC-110 and AC-111 (CG `bec3f380`, `spec/assurance/AD-004-cg-crate-layout.md:327`, `src/kani/generate/collection.rs:1-12`, `src/kani/generate/lower/finite_reference_graphs.rs:1-15`). The counterexample envelope, witness and replay source are QSL's `qsl-replay` types; the codegen backend adapter parses the Kani transcript, and the codegen replay adapter currently calls `qsl_replay::replay` with its request (CG `bec3f380`, `src/replay/function.rs:180-197,809-817`). QSL ADR-011 E9 assigns that call to the orchestrating driver and request construction to CG (QSL `adde683e`, `spec/decisions/ADR-011-stage-dag-and-dependency-architecture.md:264-266`); IR-492 tracks the current CG call as a transition residual. This boundary does not parse source or transcripts, define Quire semantics, execute a foreign runtime, or turn any bounded result into an unqualified claim about an unbounded domain.

## Views

```text
checked clause + exact profile + finite ABI input
  -> profile/matrix selection -> input validation -> dispatch index
  -> family module -> oracle/strategy/harness artifacts -> Kani
  -> typed outcome
counterexample (codegen) -> qsl_replay::WitnessEnvelope -> CG request -> QSL replay facade -> ReplayResult
current caller: CG replay adapter; E9 owner: orchestrating driver (IR-492 transition)
```

The dispatch index is shared; the active codegen lowerers are separate for finite reference graphs and collections. Checked arithmetic remains a profile family, but its former codegen lowerer and corpus arm are retired. Validation runs before any Kani `assume`; assumptions restrict only a previously validated finite model. The output envelope is shared and no non-success kind has a Boolean value.

## Decisions

- Version profile, ABI, support matrix, and generators together.
- Make finite universes, snapshots, completeness, identities, and resource bounds explicit.
- Preserve invalid, incomplete, unavailable, and exhausted inputs as results rather than assumptions.
- Map every outcome to its one QSL terminal value; QSL owns the counterexample and replay types, CG builds the request, and E9 assigns the facade call to the driver. Track CG's current direct call as IR-492.
- Keep the shared vocabulary and dispatch index in Contract IR; keep active collection and graph lowerings in codegen's separate modules. Do not count the retired arithmetic lowerer as a current backend path.

## Risks

- A generator could claim support without native parity; replay agreement through QSL's facade and corpus parity expose it. The current CG caller does not transfer E9 ownership from the driver.
- A cross-family shortcut could erase identity or duplicate/order semantics; module ownership and refusal tests forbid it.
- Resource pressure could look like success; the outcome envelope preserves exhaustion and timeout.
