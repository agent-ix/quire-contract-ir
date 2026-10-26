---
id: SR-567891
title: "IR-874 PR 576 code-review review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/v2/protocol_body.rs, crates/quire-contract-model/src/checked_package/shared.rs, tests/it/checked_package_v2_protocol_body.rs; PR #576 frozen head 6ca1fa11517ad1e24aa4fd6011dd3835ae5b7fe2 against decf3725"
review_set: subset
---

## Summary

Reviewed the frozen commit-dominance diff against QSpec FR-464/466 and IR FR-038, including containment, first-refusal order, bounded traversal, typed detail and public-reader tests.

## Verdict

**FAIL** — Commit dominance runs after membership and all templates’ other local checks, reversing QSpec first-refusal order.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Commit dominance runs after membership and all templates’ other local checks, reversing QSpec first-refusal order. | crates/quire-contract-model/src/checked_package/v2/protocol_body.rs:3124-3132 |

## Scope

- **FR-038-AC-284 (examined)**: The reader runs a protocol step after state-graph, temporal, abstraction, refinement, terminal, step-relation, hybrid and timed-model and before operations. That step reports the first defect by QSpec FR-464 order: placement of applications and member references, operator and law, declarations, member nodes in role/channel/queue/obligation/control/compensation order, membership, clauses, then refinement class-catalog graph joins; nodes within a stage are visited by ascending node-id digest. A package with both a refinement-step and protocol defect reports the refinement defect; one with both a
- **FR-038-AC-289 (context_only)**: A `compensate` body admits the thirteen FR-466 bindings in order, including distinct registration and activation captures, a trigger, interval, at-least-one attempt, retry relation, optional commit and recovery predicate. `commit never` is an absent optional Group, not a fabricated commit. The reader refuses a forward target that is not an `effect`, a commit target that is not a `commit`, an inverted interval or zero attempts, a commit that dominates its forward effect, and a read outside any of the five compensation stages with the corresponding FR-466 code, cause and locus; a commit in one c
- **FR-466-AC-11 (examined)**: A `compensate` template is admitted with `commit` present and absent (`never`), each of the thirteen arguments in order, and with a commit that follows or is unordered against the forward effect; `attempts.maximum: 0` and an inverted `within` refuse `ill_typed`/`type-mismatch` (member, expected, actual), and a commit that dominates the forward effect refuses `unsupported_construct`/`declaration-form` retaining the profile and reason `commit-precedes-effect`, while a commit in one case of a `choice` before the effect is admitted; a `forward` that is not an `effect` control and a `commit` that i

## Coverage

The diff changes Rust model code and public-reader tests only; no CI, dependency, spec or plan file changed. FR-038-AC-284/289 remain Planned.
