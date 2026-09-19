---
id: FR-031
title: "Dispatch bounded Kani modules with replayable provenance"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-029
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-030
    type: depends_on
  - target: ix://agent-ix/quire-specification/AD-016
    type: references
---
# FR-031: Dispatch bounded Kani modules with replayable provenance

## Description

The bounded-Kani boundary shall dispatch semantic families modularly, preserve artifact identity through oracle/strategy/harness generation, and replay each concrete counterexample through the QSL complete-V1 executor entry `value::expression::CheckedPackage::call`.

## Inputs

A validated bounded execution input, selected profile/capability matrix, and selected native runtime and model identities.

## Outputs

A complete typed result with provenance, or a complete counterexample packet that the QSL complete-V1 executor entry `value::expression::CheckedPackage::call` can consume without Kani.

## Behavior

The dispatch index is the only cross-module vocabulary and routing authority. It selects independently versioned modules for definedness/checked arithmetic, finite object/reference/graph semantics, and bounded collection/query semantics. Each module declares constructs it owns, exact input/output ABI revision, definedness dependencies, resource charges, and supported/refused/inconclusive cases. Modules cannot invent source meaning, reinterpret another module's values, or silently substitute structural equality for identity, a collection set for an ordered duplicate-preserving sequence, or bounded graph search for unbounded reachability.

Oracle, strategy, lowering, and harness generators are explicit interfaces with content identities. Their generated artifacts bind the checked-clause identity, profile selection, complete input identity, module identities, Kani executable digest/options, declared assumptions, and proof dependencies. Any change in a bound, assumption, selected module, tool/options digest, source/model/snapshot identity, or generator bytes changes the artifact identity.

A `counterexample` serializes exact ABI/profile identities, concrete finite population/snapshots/invocation, selected bounds, strategy seed where used, evaluated witness, and provenance; the packet is Contract IR's. Deserialization validates the same input ABI before replay. The codegen replay adapter reconstructs the packet's input and invokes the QSL complete-V1 executor entry `value::expression::CheckedPackage::call`, which must reproduce the counterexample's outcome and relevant witness; identity mismatch, invalid reconstruction, unavailable executor, or disagreement is a typed non-success result, not a repaired replay. A `proved` result does not serialize a counterexample.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-031-AC-1 | The shared dispatch index routes definedness/arithmetic, object/reference/graph, and collection/query work through distinct declared modules and rejects cross-family approximation. | Test (TC-042) |
| FR-031-AC-2 | Every generated lowering, oracle, strategy, harness, proof, and result has exact provenance binding Kani version/digest/options, assumptions, bounds, inputs, modules, and dependencies. | Test (TC-042) |
| FR-031-AC-3 | Every serialized counterexample either reproduces through the QSL complete-V1 executor entry `value::expression::CheckedPackage::call` with the same outcome/witness or returns a typed non-success disagreement; proof, refusal, and inconclusive results never masquerade as replayed counterexamples. | Test (TC-054) |
| FR-031-AC-4 | An evaluated witness is parsed from a backend transcript and carries the concrete values that transcript recorded, untyped; it is typed only against a schema the generator declared, and a disagreement between the two — in arity, in byte width, or with the backend's own decoded value — is a typed refusal naming its cause, never an inferred value. A transcript that witnesses reachability or a bound rather than falsity is refused as a counterexample witness. | Test (TC-221) |

## Dependencies

FR-029 selects module and artifact ABI versions. FR-030 defines validated inputs and typed non-Boolean outcomes.

## Status

AC-1 and AC-2 are implemented through Contract IR PRs #88 through #92 and
qualified against the cycle-free integrated codegen corpus from codegen PR #47
at `73c82ad`. The dispatch index routes the three families through distinct
modules, refusing an unowned construct and a duplicate module owner; it does
not itself prove that a module cannot approximate another family's structural
equality, collection-set, or bounded graph search semantics.
`GeneratorProvenance` and `ArtifactIdentity` (`src/kani/provenance.rs`) exist
and are identity-sensitive — the one test tagged AC-2 mutates a single
assumption and confirms the resulting digest changes — but no generator emits
one: `ArithmeticLowering` and the object/collection lowerings carry no
provenance field. That evidence is not a general proof-engine or release
claim.

AC-3, the crossing behaviour, has its own row as TC-054, discharged by the QSL
crossing test at `agent-ix/quire-spec-language#243` (the layer-6 `replay`
facade, AD-016 arrows 6 and 7), which has not started. The codegen producer
still sets the packet's witness field to the artifact's own content digest
before Kani runs, and the test
`tc_042_counterexample_replays_through_native_runtime_execute` passes
`reconstruct` as `|_|`, discarding the packet and returning a constant input
whose truth the test asserts two lines earlier. Native `runtime::execute` is
therefore reached with an input no counterexample determined. Lowering,
emission, compilation, `cargo kani` execution and transcript classification are
built and gated; reconstruction and executor agreement are not. Per AD-016 the
counterexample packet is `src/kani/replay.rs`'s and the executor call is the
codegen replay adapter's, pending WP9 (issue #140); today `src/kani/replay.rs`
also carries `replay_with_native_runtime`, which invokes the executor itself.

AC-4, the witness vocabulary, is implemented by IR #139 and verified by
TC-221.
