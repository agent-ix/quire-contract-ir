---
id: AD-002
title: "Bounded Kani backend architecture"
type: ArchitectureDescription
status: proposed
owner: kreneskyp
system: quire-contract-ir bounded Kani backend profile family
relationships:
  - target: ix://agent-ix/quire-contract-ir/AP-002
    type: realizes
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

This boundary consumes already checked native clauses and exact finite model inputs. It chooses a bounded Kani profile, validates its input ABI, routes each construct through the dispatch index to a versioned family module, binds generated oracle/strategy/harness artifacts to their provenance, interprets Kani results as typed outcomes, and maps each outcome to one QSL `qsl_replay::TerminalValue` (FR-031). A construct the profile has no qualified interpretation for settles `unsupported` at negotiation, with a warning naming the construct kind, and produces no artifact or outcome (FR-029). Contract IR holds the profile, input ABI, dispatch index, outcome and provenance; the family lowerings for checked arithmetic, collections and objects, and harness emission, are the codegen backend adapter's (AD-001 "Kani boundary"). The counterexample envelope, witness and replay source are QSL's `qsl-replay` types; the codegen backend adapter parses the Kani transcript, and the codegen replay adapter replays a counterexample through `qsl_replay::replay` (QSL ADR-011 E9). This boundary does not parse source or transcripts, define Quire semantics, make a release decision, execute a foreign runtime, or turn any bounded result into an unqualified claim about an unbounded domain.

## Views

```text
checked clause + exact profile + finite ABI input
  -> profile/matrix selection -> input validation -> dispatch index
  -> family module -> oracle/strategy/harness artifacts -> Kani
  -> typed outcome + provenance -> qsl_replay::TerminalValue
counterexample (codegen) -> qsl_replay::WitnessEnvelope -> qsl_replay::replay -> ReplayResult
```

The dispatch index is shared; family modules are separate for checked arithmetic/definedness, objects/references/graphs, and collections/queries. Validation runs before any Kani `assume`; assumptions restrict only a previously validated finite model. The output envelope is shared and no non-success kind has a Boolean value.

## Decisions

- Version profile, ABI, support matrix, tool/options digest, and generators together.
- Make finite universes, snapshots, completeness, identities, and resource bounds explicit.
- Preserve invalid, incomplete, unavailable, and exhausted inputs as results rather than assumptions.
- Couple every outcome to its provenance graph, and every outcome to its one QSL terminal value; leave the counterexample envelope and replay to QSL and codegen.
- Keep the shared vocabulary and dispatch index in Contract IR; keep the semantic family lowerings in separate modules of the codegen backend adapter.

## Risks

- Kani/tool drift invalidates a profile selection; exact digest and options bind it.
- A generator could claim support without native parity; QSL replay agreement, run by codegen, and corpus parity expose it.
- A cross-family shortcut could erase identity or duplicate/order semantics; module ownership and refusal tests forbid it.
- Resource pressure could look like success; the outcome envelope preserves exhaustion and timeout.
