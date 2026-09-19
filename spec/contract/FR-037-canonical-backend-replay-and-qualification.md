---
id: FR-037
title: "Replay backend counterexamples through the complete-V1 executor"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/AD-003
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-036
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-197
    type: references
  - target: ix://agent-ix/quire-specification/FR-336
    type: references
  - target: ix://agent-ix/quire-specification/AD-016
    type: references
---
# FR-037: Replay backend counterexamples through the complete-V1 executor

## Description

When a backend reports a counterexample, the Contract IR replay boundary shall
canonicalize the typed input and execution identity, validate it against the
original domain, and replay it through the QSL complete-V1 executor entry
`value::expression::CheckedPackage::call` without a backend shortcut.

## Inputs

Backend counterexample, immutable backend-result digest, package and claim
digests, provider run identity, typed values, pre-state, selected transition or
trace occurrences, reported backend verdict, model domain, and executor
selection.

## Outputs

A canonical replay envelope and a backend/native verdict pair, or a typed
decode, domain, unavailable, minimization, or parity-failure outcome.

## Behavior

The replay boundary shall preserve exact value kinds, object and trace
identities, state anchors, bounds, source maps, profile versions, and run
lineage. The replay boundary shall recompute the immutable backend-result digest
over package, claim, profile, provider run, typed values including IEEE width
and bits, pre-state, selected occurrences, and reported backend verdict before
replay. When that digest differs
from the received result identity, the replay boundary shall return a typed
integrity failure without native execution. The replay boundary shall serialize exact numbers without narrowing.
The replay boundary shall preserve IEEE width and bits.
The replay boundary shall preserve collection occurrence identity in canonical ordering.
When the replay boundary produces a smaller counterexample candidate, the replay boundary shall create a new revision linked to its parent.
The replay boundary shall retain that revision only when domain validity and both failure verdicts persist.
When decode, domain, or verdicts differ, the replay boundary shall return a typed parity failure rather than a clause success or failure.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-037-AC-1 | Every backend-supported typed counterexample round-trips canonically without loss of identity, type, bound, or source correspondence. | Test (TC-046) |
| FR-037-AC-2 | A bounded Kani counterexample replays through the same native package and domain with the same verdict. | Test (TC-046) |
| FR-037-AC-3 | Decode, domain, executor-availability, or verdict disagreement remains an explicit non-success outcome. | Test (TC-046) |
| FR-037-AC-4 | Minimization creates linked revisions and retains only valid candidates with preserved backend and native failure. | Test (TC-046) |
| FR-037-AC-5 | Mutation or substitution of any received counterexample value, state, occurrence, verdict, or identity member fails immutable backend-result verification before replay. | Test (TC-046) |

## Dependencies

[FR-036](./FR-036-exact-backend-negotiation-and-emission.md) owns backend
inputs. QSpec FR-197 and I19 own the normative replay and corpus contracts.
