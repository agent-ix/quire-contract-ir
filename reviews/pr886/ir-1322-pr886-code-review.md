---
id: SR-20261104020019
title: "IR-1322 PR 886 code and Rust review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR 886; protocol reader Rust and by-reference tests"
review_set: subset
---

## Summary

Independent diff review of the IR-1322 protocol reader changes and QSpec conformance tests. Ticket: IR-1322.

## Verdict

CONDITIONAL. The temporal-selection prepass changes the required first-refusal order for a package with an earlier state defect.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | A package combining QSpec V16's absent temporal profile with V14's state-clause defect now refuses missing-selection at the compensation before the state step can refuse ill_typed/operator-ineligible. QSpec FR-464 Reader order places state and other body steps before the protocol step. The new scan runs before structural and state validation, so it changes first-refusal precedence beyond V16 and CP28. | crates/quire-contract-model/src/checked_package/v2/mod.rs:2938; crates/quire-contract-model/src/checked_package/v2/protocol_body.rs:5376 |

## Examined Scope

- role: context_only; id: FR-038-AC-436; excerpt: A state-clause body reference to a protocol member node other than a `protocol_clause` or `protocol_reference` carrier refuses `ill_typed`/`operator-ineligible` at the containing state-clause node with no admitted package, even when the referenced member is Boolean typed. The protocol step applies this placement rule after earlier graph and state defects. TC-048 reads the same merged manifest and base fixture by reference, applies V14 unchanged, and checks that exact code and cause at `/semantic_graph/nodes/54`, with that node as structural loc
- role: examined; id: FR-038-AC-455; excerpt: PLANNED/UNRUN (IR-1322). In QSpec protocol mutation V2, a protocol node whose body applies `quire.op.state.clause` refuses `ill_typed`/`operator-ineligible` at the containing `/semantic_graph/nodes/64`, with the native node identity retained and no admitted package.
- role: examined; id: FR-038-AC-456; excerpt: PLANNED/UNRUN (IR-1322). QSpec V8, V9 and V10 refuse at their containing `/semantic_graph/nodes/64`, and V11 at `/semantic_graph/nodes/82`, while retaining their field-level `document_pointer`: `/semantic_graph/nodes/64/body/operation/member`, `/semantic_graph/nodes/64/body/operation/laws`, `/semantic_graph/nodes/64/body/operation/laws/0` and `/semantic_graph/nodes/82/body/operation/laws/0/definition`, respectively. Their code/cause pairs remain `invalid_package` with, respectively, `operation-member-mismatch`, `operation-law-missing`, `operati
- role: examined; id: FR-038-AC-457; excerpt: PLANNED/UNRUN (IR-1322). QSpec V16 and CP28 each remove the selected `temporal_profile` while a compensation reads a clock role; the first refusal is `missing_declaration`/`missing-selection` at `/semantic_graph/nodes/81`, before an `invalid_package` operation-law refusal. Both cases retain the same code, cause and primary pointer without a partial package.
- role: examined; id: FR-038-AC-458; excerpt: PLANNED/UNRUN (IR-1322). QSpec C11's clause `relationships` target changed to a role refuses `invalid_model_binding`/`wrong-export` at `/semantic_graph/nodes/82/body/arguments/4/value/members/0/target`, retaining required kind `relationship` and actual kind `role` with their published keys; it does not report the containing `protocol_clause` as the target form.
- role: examined; id: FR-038-AC-459; excerpt: PLANNED/UNRUN (IR-1322). TC-048 reads the current QSpec protocol mutation manifest and base package directly from the selected checkout, applies V2, V8, V9, V10, V11, RP6, V16, CP28 and C11 without a local fixture copy, and checks each published reader stage, code, cause, primary pointer, applicable `document_pointer` and retained target kind. The test fails for an unexpectedly admitted row or an unmatched refusal. QSpec's schema-stage V3, V4, V5, V7 and L19 and the already matching C12 are not reader-fix cases for this criterion.

## Coverage

Reviewed every changed Rust and test file in PR 886. Rust review covered protocol wire refusals, typed causes and pointers, node ordering, work-meter effects, and the QSpec by-reference fixture boundary. No CI, Cargo manifest, dependency, or public API file changed.

## Verification and Limits

The source call order is direct: validate_temporal_selection runs before structural::validate_structural_nodes_with_keys and state::validate_state. QSpec FR-464 Reader order puts the protocol step after both; QSpec V14 independently requires the state refusal and V16 independently requires the missing-selection refusal. Computed Quire matrix tags AC-455 through AC-459 to the changed by-reference test. The coder's focused tests passed; this reviewer did not rerun the aggregate gate before disposition.
