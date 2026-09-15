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
---
# Bounded Kani backend architecture

## System Boundary

This boundary consumes already checked native clauses and exact finite model inputs. It chooses a bounded Kani profile, validates its input ABI, dispatches to versioned lowering modules, generates identified oracle/strategy/harness artifacts, interprets Kani results as typed outcomes, and replays concrete counterexamples through native `runtime::execute`. It does not parse source, define Quire semantics, make a release decision, execute a foreign runtime, or turn any bounded result into an unqualified claim about an unbounded domain.

## Views

```text
checked clause + exact profile + finite ABI input
  -> profile/matrix selection -> input validation -> dispatch index
  -> family module -> oracle/strategy/harness artifacts -> Kani
  -> typed outcome + provenance
counterexample only -> replay packet -> native runtime::execute -> agreement or typed non-success
```

The dispatch index is shared; family modules are separate for checked arithmetic/definedness, objects/references/graphs, and collections/queries. Validation runs before any Kani `assume`; assumptions restrict only a previously validated finite model. The output envelope is shared and no non-success kind has a Boolean value.

## Decisions

- Version profile, ABI, support matrix, tool/options digest, and generators together.
- Make finite universes, snapshots, completeness, identities, and resource bounds explicit.
- Preserve invalid, incomplete, unavailable, and exhausted inputs as results rather than assumptions.
- Couple every counterexample to a native replay packet and provenance graph.
- Keep shared vocabulary/index in one delivery lane; keep semantic families in separate Rust/test modules.

## Risks

- Kani/tool drift invalidates a profile selection; exact digest and options bind it.
- A generator could claim support without native parity; replay agreement and corpus parity expose it.
- A cross-family shortcut could erase identity or duplicate/order semantics; module ownership and refusal tests forbid it.
- Resource pressure could look like success; the outcome envelope preserves exhaustion and timeout.
