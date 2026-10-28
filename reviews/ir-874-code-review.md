---
id: SR-567887
title: "IR-874 PR 574 code-review review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/protocol_body.rs, tests/it/checked_package_v2_protocol_body.rs; PR #574 frozen head d922a9ca87a928ba534e2d70539ff089c5e05141 against 54565d18"
review_set: subset
---

## Summary

Reviewed the frozen compensation diff against QSpec FR-464/466 and IR FR-038, including malformed input, typed references, work charging, refusal order and public-reader tests.

## Verdict

**PASS** — no code or Rust-style findings in this local compensation slice. The remaining global protocol stages remain Planned.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Scope

- **FR-038-AC-282 (examined)**: Each of the seven protocol member forms admits only the FR-464, FR-465 or FR-466 fixed argument names, order and term shapes for its selected operator: thirteen clause bindings, the role, channel, queue, finish and terminal layouts, thirteen control layouts and thirteen compensation bindings. A missing, extra, reordered, wrongly shaped or misspelled binding refuses `malformed_wire`; an optional Group admits zero or one member only, an interval has non-negative integer bounds with lower no greater than upper, and a set or sorted-record list repeated or out of its specified ascending order refus
- **FR-038-AC-284 (context_only)**: The reader runs a protocol step after state-graph, temporal, abstraction, refinement, terminal, step-relation, hybrid and timed-model and before operations. That step reports the first defect by QSpec FR-464 order: placement of applications and member references, operator and law, declarations, member nodes in role/channel/queue/obligation/control/compensation order, membership, clauses, then refinement class-catalog graph joins; nodes within a stage are visited by ascending node-id digest. A package with both a refinement-step and protocol defect reports the refinement defect; one with both a
- **FR-038-AC-289 (examined)**: A `compensate` body admits the thirteen FR-466 bindings in order, including distinct registration and activation captures, a trigger, interval, at-least-one attempt, retry relation, optional commit and recovery predicate. `commit never` is an absent optional Group, not a fabricated commit. The reader refuses a forward target that is not an `effect`, a commit target that is not a `commit`, an inverted interval or zero attempts, a commit that dominates its forward effect, and a read outside any of the five compensation stages with the corresponding FR-466 code, cause and locus; a commit in one c
- **FR-466-AC-11 (examined)**: A `compensate` template is admitted with `commit` present and absent (`never`), each of the thirteen arguments in order, and with a commit that follows or is unordered against the forward effect; `attempts.maximum: 0` and an inverted `within` refuse `ill_typed`/`type-mismatch` (member, expected, actual), and a commit that dominates the forward effect refuses `unsupported_construct`/`declaration-form` retaining the profile and reason `commit-precedes-effect`, while a commit in one case of a `choice` before the effect is admitted; a `forward` that is not an `effect` control and a `commit` that i
- **FR-466-AC-12 (context_only)**: Compensation scope, by the five stages: the trigger guard reads the trigger binder, the protocol captures and the registration captures and refuses the forward binder and the protocol activation binder (`missing_declaration`/`missing-name`) and `over` (`ill_typed`/`operator-ineligible`); an activation capture reads the trigger binder, `over`, the captures and earlier activation captures; the retry relation reads `earlier`, `later` and the captures and refuses the trigger binder (`missing-name`) and `over` (`operator-ineligible`); the recovery predicate reads its binder and the captures and ref

## Coverage

The diff changes Rust model code and public-reader tests only; no CI, dependency, spec or plan file changed. The seven compensation integration tests passed on the frozen head. The captured-record test gap is recorded separately in SR-567888. Full protocol membership, causality, commit dominance and binder-slot stages remain outside this slice.
