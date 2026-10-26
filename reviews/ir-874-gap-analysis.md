---
id: SR-567888
title: "IR-874 PR 574 gap-analysis review"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/protocol_body.rs, tests/it/checked_package_v2_protocol_body.rs; PR #574 frozen head d922a9ca87a928ba534e2d70539ff089c5e05141 against 54565d18"
review_set: subset
---

## Summary

The computed Quire matrix tags the protocol criteria, but this PR's public-reader tests never construct a nonempty compensation capture list.

## Verdict

**CONDITIONAL** — one focused test gap remains in the changed behavior.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | All admitted compensation fixtures have empty registration and activation captures, leaving the new nested capture shape, binder-reference and per-record work-charge paths untested. | tests/it/checked_package_v2_protocol_body.rs:1196 |

## Scope

- **FR-038-AC-282 (examined)**: Each of the seven protocol member forms admits only the FR-464, FR-465 or FR-466 fixed argument names, order and term shapes for its selected operator: thirteen clause bindings, the role, channel, queue, finish and terminal layouts, thirteen control layouts and thirteen compensation bindings. A missing, extra, reordered, wrongly shaped or misspelled binding refuses `malformed_wire`; an optional Group admits zero or one member only, an interval has non-negative integer bounds with lower no greater than upper, and a set or sorted-record list repeated or out of its specified ascending order refus
- **FR-038-AC-284 (context_only)**: The reader runs a protocol step after state-graph, temporal, abstraction, refinement, terminal, step-relation, hybrid and timed-model and before operations. That step reports the first defect by QSpec FR-464 order: placement of applications and member references, operator and law, declarations, member nodes in role/channel/queue/obligation/control/compensation order, membership, clauses, then refinement class-catalog graph joins; nodes within a stage are visited by ascending node-id digest. A package with both a refinement-step and protocol defect reports the refinement defect; one with both a
- **FR-038-AC-289 (examined)**: A `compensate` body admits the thirteen FR-466 bindings in order, including distinct registration and activation captures, a trigger, interval, at-least-one attempt, retry relation, optional commit and recovery predicate. `commit never` is an absent optional Group, not a fabricated commit. The reader refuses a forward target that is not an `effect`, a commit target that is not a `commit`, an inverted interval or zero attempts, a commit that dominates its forward effect, and a read outside any of the five compensation stages with the corresponding FR-466 code, cause and locus; a commit in one c
- **FR-466-AC-11 (examined)**: A `compensate` template is admitted with `commit` present and absent (`never`), each of the thirteen arguments in order, and with a commit that follows or is unordered against the forward effect; `attempts.maximum: 0` and an inverted `within` refuse `ill_typed`/`type-mismatch` (member, expected, actual), and a commit that dominates the forward effect refuses `unsupported_construct`/`declaration-form` retaining the profile and reason `commit-precedes-effect`, while a commit in one case of a `choice` before the effect is admitted; a `forward` that is not an `effect` control and a `commit` that i
- **FR-466-AC-12 (context_only)**: Compensation scope, by the five stages: the trigger guard reads the trigger binder, the protocol captures and the registration captures and refuses the forward binder and the protocol activation binder (`missing_declaration`/`missing-name`) and `over` (`ill_typed`/`operator-ineligible`); an activation capture reads the trigger binder, `over`, the captures and earlier activation captures; the retry relation reads `earlier`, `later` and the captures and refuses the trigger binder (`missing-name`) and `over` (`operator-ineligible`); the recovery predicate reads its binder and the captures and ref

## Coverage

Plan completion: not assessed. `quire matrix --scope . --format json` completed; it is a tag index, not evidence that this nonempty capture path ran. Reverse-gap and stub inspection found no ownerless changed behavior or stub. The optional repository-wide semantic review was not requested; this scoped review compared the changed behavior with FR-464/466 and IR FR-038. The seven compensation public-reader tests passed. Full FR-038-AC-282/284/289 remain Planned.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The shared admitted fixture now contains distinct nonempty registration and activation capture records. The public-reader test admits that fixture and refuses wrong binder forms for both lists and a reordered activation record. These paths execute the per-record meter charge. |

## Disposition verdict

**PASS at 79688b6a** — FND-001 is fixed on the original PR; full FR-038-AC-282/284/289 remain Planned.
