---
id: SR-20261104020021
title: "IR-1322 PR 887 code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR 887; spec/checked_package/functional/FR-038-consume-checked-package-v2.md"
review_set: subset
---

## Summary

Reviewed the five status-cell changes in FR-038 against merged reader PR #886 and its by-reference tests. No applicable AssuranceProfile was found under spec/ or plan/.

## Verdict

PASS. The status changes introduce no code or copied artifact. The merged test reads the selected QSpec manifest and package by reference, asserts code, cause, primary pointer and document pointer for all nine reader rows, and checks C11's target identity. Paired V14+V16 and V11+V16 tests enforce reader precedence.

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

PR #887 changes only the first cell of AC-455 through AC-459. Reader PR #886 merged. Its hosted Rust, supply-chain and CLA checks passed, and the final local run passed 1,214 integration and 366 model tests; strict Quire coverage improved from 110 to 105 unbacked rows with zero contradictions. This Markdown-only review ran no Cargo gate. QSL owns schema-stage V3, V4, V5, V7 and L19.
