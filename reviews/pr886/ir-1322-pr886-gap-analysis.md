---
id: SR-20261104020020
title: "IR-1322 PR 886 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR 886; FR-038 AC-455..459 test evidence"
review_set: subset
---

## Summary

Planless traceability and test-evidence audit of the nine protocol reader rows. Ticket: IR-1322.

## Verdict

CONDITIONAL. Every changed acceptance criterion is tagged, but the test does not exercise the precedence case introduced by the new prepass.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The by-reference test applies V16 and CP28 only as single mutations. It never combines absent temporal selection with QSpec V14's earlier state-clause defect, so the test passes even though the new prepass reverses FR-464 Reader order. Add a paired-defect control that checks the earlier state refusal. | tests/conformance_qspec/protocol_mutation_manifest.rs:147-159; crates/quire-contract-model/src/checked_package/v2/mod.rs:2938 |

## Examined Scope

- role: context_only; id: FR-038-AC-436; excerpt: A state-clause body reference to a protocol member node other than a `protocol_clause` or `protocol_reference` carrier refuses `ill_typed`/`operator-ineligible` at the containing state-clause node with no admitted package, even when the referenced member is Boolean typed. The protocol step applies this placement rule after earlier graph and state defects. TC-048 reads the same merged manifest and base fixture by reference, applies V14 unchanged, and checks that exact code and cause at `/semantic_graph/nodes/54`, with that node as structural loc
- role: examined; id: FR-038-AC-455; excerpt: PLANNED/UNRUN (IR-1322). In QSpec protocol mutation V2, a protocol node whose body applies `quire.op.state.clause` refuses `ill_typed`/`operator-ineligible` at the containing `/semantic_graph/nodes/64`, with the native node identity retained and no admitted package.
- role: examined; id: FR-038-AC-456; excerpt: PLANNED/UNRUN (IR-1322). QSpec V8, V9 and V10 refuse at their containing `/semantic_graph/nodes/64`, and V11 at `/semantic_graph/nodes/82`, while retaining their field-level `document_pointer`: `/semantic_graph/nodes/64/body/operation/member`, `/semantic_graph/nodes/64/body/operation/laws`, `/semantic_graph/nodes/64/body/operation/laws/0` and `/semantic_graph/nodes/82/body/operation/laws/0/definition`, respectively. Their code/cause pairs remain `invalid_package` with, respectively, `operation-member-mismatch`, `operation-law-missing`, `operati
- role: examined; id: FR-038-AC-457; excerpt: PLANNED/UNRUN (IR-1322). QSpec V16 and CP28 each remove the selected `temporal_profile` while a compensation reads a clock role; the first refusal is `missing_declaration`/`missing-selection` at `/semantic_graph/nodes/81`, before an `invalid_package` operation-law refusal. Both cases retain the same code, cause and primary pointer without a partial package.
- role: examined; id: FR-038-AC-458; excerpt: PLANNED/UNRUN (IR-1322). QSpec C11's clause `relationships` target changed to a role refuses `invalid_model_binding`/`wrong-export` at `/semantic_graph/nodes/82/body/arguments/4/value/members/0/target`, retaining required kind `relationship` and actual kind `role` with their published keys; it does not report the containing `protocol_clause` as the target form.
- role: examined; id: FR-038-AC-459; excerpt: PLANNED/UNRUN (IR-1322). TC-048 reads the current QSpec protocol mutation manifest and base package directly from the selected checkout, applies V2, V8, V9, V10, V11, RP6, V16, CP28 and C11 without a local fixture copy, and checks each published reader stage, code, cause, primary pointer, applicable `document_pointer` and retained target kind. The test fails for an unexpectedly admitted row or an unmatched refusal. QSpec's schema-stage V3, V4, V5, V7 and L19 and the already matching C12 are not reader-fix cases for this criterion.

## Coverage

Plan completion: not assessed. The computed Quire matrix tags FR-038-AC-455..459 to the by-reference conformance test; AC-456 also has a local protocol-body test. No stale trace id or copied upstream fixture appears in the diff. The new early-validation path needs an adverse ordering control.

## Verification and Limits

Read the frozen PR diff and QSpec FR-464 Reader order, V14 and V16 manifest rows. The aggregate gate belongs to the code lead after review disposition.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | medium | The new V14+V16 control covers the state-before-protocol boundary, but no paired control covers FR-464 step-2 ascending node-id order. Apply published V11 and V16 together and assert V11's `invalid_package`/`operation-law-unselected` at clause node 82, including its `/body/operation/laws/0/definition` document pointer, before V16's missing selection at compensation node 81. The clause structural identity sorts first. | tests/conformance_qspec/protocol_mutation_manifest.rs:219; QSpec FR-464 Reader order step 2; QSpec protocol-node-mutation-manifest.json V11 and V16 |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | The added by-reference V14+V16 test applies both published edits and asserts the earlier state code, cause, path and structural locus. |
| FND-002 | still-open | The fix round adds V14+V16 only; V11+V16 step-2 precedence remains untested. |

| FND-002 | fixed | The by-reference paired V11+V16 test asserts the published V11 refusal, including its document pointer and clause structural locus; the V14+V16 control remains. |
