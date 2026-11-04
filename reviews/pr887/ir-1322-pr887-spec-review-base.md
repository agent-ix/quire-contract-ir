---
id: SR-20261104020022
title: "IR-1322 PR 887 base spec review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR 887; spec/checked_package/functional/FR-038-consume-checked-package-v2.md"
review_set: subset
---

## Summary

Reviewed the FR-038 criterion statuses and their cited verification against the merged reader. No applicable AssuranceProfile was found under spec/ or plan/.

## Verdict

PASS. AC-455 through AC-459 have concrete by-reference test evidence on current QSpec. Their text retains the separate QSL schema-stage boundary; the PR makes no new reader or complete-package claim. IDs, links and verification methods are unchanged.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Examined Scope

- role: examined; id: FR-038-AC-455; excerpt: Implemented. In QSpec protocol mutation V2, a protocol node whose body applies `quire.op.state.clause` refuses `ill_typed`/`operator-ineligible` at the containing `/semantic_graph/nodes/64`, with the native node identity retained and no admitted package.
- role: examined; id: FR-038-AC-456; excerpt: Implemented. QSpec V8, V9 and V10 refuse at their containing `/semantic_graph/nodes/64`, and V11 at `/semantic_graph/nodes/82`, while retaining their field-level `document_pointer`: `/semantic_graph/nodes/64/body/operation/member`, `/semantic_graph/nodes/64/body/operation/laws`, `/semantic_graph/nodes/64/body/operation/laws/0` and `/semantic_graph/nodes/82/body/operation/laws/0/definition`, respectively. Their code/cause pairs remain `invalid_package` with, respectively, `operation-member-mismatch`, `operation-law-missing`, `operation-law-mismatch` and `operation-law-unselected`. RP6 refuses an integer `Poll` guard as `ill_typed`/`non-boolean-root` at `/semantic_graph/nodes/76`, retaining `/semantic_graph/nodes/76/body/arguments/4/value` as `document_pointer`. None returns an admitted package or a partial protocol view.
- role: examined; id: FR-038-AC-457; excerpt: Implemented. QSpec V16 and CP28 each remove the selected `temporal_profile` while a compensation reads a clock role; the first refusal is `missing_declaration`/`missing-selection` at `/semantic_graph/nodes/81`, before an `invalid_package` operation-law refusal. Both cases retain the same code, cause and primary pointer without a partial package.
- role: examined; id: FR-038-AC-458; excerpt: Implemented. QSpec C11's clause `relationships` target changed to a role refuses `invalid_model_binding`/`wrong-export` at `/semantic_graph/nodes/82/body/arguments/4/value/members/0/target`, retaining required kind `relationship` and actual kind `role` with their published keys; it does not report the containing `protocol_clause` as the target form.
- role: examined; id: FR-038-AC-459; excerpt: Implemented. TC-048 reads the current QSpec protocol mutation manifest and base package directly from the selected checkout, applies V2, V8, V9, V10, V11, RP6, V16, CP28 and C11 without a local fixture copy, and checks each published reader stage, code, cause, primary pointer, applicable `document_pointer` and retained target kind. The test fails for an unexpectedly admitted row or an unmatched refusal. QSpec's schema-stage V3, V4, V5, V7 and L19 and the already matching C12 are not reader-fix cases for this criterion.

## Verification and Limits

The PR changes only PLANNED/UNRUN to Implemented in five existing criteria. `current_qspec_protocol_reader_refusals_preserve_published_provenance` carries all five trace tags and asserts every selected row's refusal against the published manifest. Separate paired tests cover state-before-protocol and structural-ID protocol order. PR #886 qualified against the selected QSpec main checkout. The review ran scoped Quire validation; no Cargo gate was run for this Markdown-only change.
