---
id: SR-20261104020017
title: "IR-1322 PR 885 scope and boundary review"
type: SpecReview
analysis: scope-boundary
scope: "agent-ix/quire-contract-ir; PR 885; FR-038 protocol reader scope correction"
review_set: subset
---

## Summary

Independent scope and boundary review of the PR #885 FR-038 correction against QSpec main b840f098.

## Verdict

PASS. The changed criteria assign only the nine reader-stage refusals to IR-1322. The new behavior paragraph states the published QSpec schema-stage verdict for V3, V4, V5, V7, and L19, while AC-459 excludes them from this reader-fix slice pending owner allocation. C12 is recorded as already matching at node 78. No QSL or QSpec implementation obligation is silently taken into this PR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Examined Scope

- role: examined; id: FR-038-AC-455; excerpt: PLANNED/UNRUN (IR-1322). In QSpec protocol mutation V2, a protocol node whose body applies `quire.op.state.clause` refuses `ill_typed`/`operator-ineligible` at the containing `/semantic_graph/nodes/64`, with the native node identity retained and no admitted package.
- role: examined; id: FR-038-AC-456; excerpt: PLANNED/UNRUN (IR-1322). QSpec V8, V9 and V10 refuse at their containing `/semantic_graph/nodes/64`, and V11 at `/semantic_graph/nodes/82`, while retaining their field-level `document_pointer`: `/semantic_graph/nodes/64/body/operation/member`, `/semantic_graph/nodes/64/body/operation/laws`, `/semantic_graph/nodes/64/body/operation/laws/0` and `/semantic_graph/nodes/82/body/operation/laws/0/definition`, respectively. Their code/cause pairs remain `invalid_package` with, respectively, `operation-member-mismatch`, `operation-law-missing`, `operati
- role: examined; id: FR-038-AC-457; excerpt: PLANNED/UNRUN (IR-1322). QSpec V16 and CP28 each remove the selected `temporal_profile` while a compensation reads a clock role; the first refusal is `missing_declaration`/`missing-selection` at `/semantic_graph/nodes/81`, before an `invalid_package` operation-law refusal. Both cases retain the same code, cause and primary pointer without a partial package.
- role: examined; id: FR-038-AC-458; excerpt: PLANNED/UNRUN (IR-1322). QSpec C11's clause `relationships` target changed to a role refuses `invalid_model_binding`/`wrong-export` at `/semantic_graph/nodes/82/body/arguments/4/value/members/0/target`, retaining required kind `relationship` and actual kind `role` with their published keys; it does not report the containing `protocol_clause` as the target form.
- role: examined; id: FR-038-AC-459; excerpt: PLANNED/UNRUN (IR-1322). TC-048 reads the current QSpec protocol mutation manifest and base package directly from the selected checkout, applies V2, V8, V9, V10, V11, RP6, V16, CP28 and C11 without a local fixture copy, and checks each published reader stage, code, cause, primary pointer, applicable `document_pointer` and retained target kind. The test fails for an unexpectedly admitted row or an unmatched refusal. QSpec's schema-stage V3, V4, V5, V7 and L19 and the already matching C12 are not reader-fix cases for this criterion.
- role: examined; id: FR-038; excerpt: QSpec V3, V4, V5, V7 and L19 fail strict V2 schema validation as malformed_wire at the containing node pointer before a protocol or operation step.

## Verification and Limits

No active AssuranceProfile or review_policy declaration was found under this repository's spec/ or plan/ trees. Compared the frozen PR diff with the published QSpec #617 manifest and FR-464/TC-946 contract. This review verifies requirements wording; it does not claim the current IR reader passes the nine rows or that ownership of the five schema-stage residuals has been resolved.
