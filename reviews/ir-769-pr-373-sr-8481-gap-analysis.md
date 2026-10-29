---
id: SR-8481
title: "gap-analysis review of IR-769 PR #373"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; FR-049-AC-1..6; TC-447; tests/it/admitted_result_context.rs; tests/conformance_qsl_ir769/main.rs"
review_set: subset
---

## Summary

All six FR-049 criteria are tagged in the computed matrix, and four focused tests pass. Three tagged obligations lack direct assertions.

## Verdict

**CONDITIONAL** — 3 finding(s) need disposition.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-049-AC-3 is tagged but wrong-shape conversion refusal is untested | tests/it/admitted_result_context.rs:276 |
| FND-002 | medium | FR-049-AC-4 post-admission error paths have no query assertion | tests/it/admitted_result_context.rs:276 |
| FND-003 | medium | FR-049-AC-5 provenance and bound source correspondence are not asserted | tests/it/admitted_result_context.rs:201 |

## Coverage

Plan completion: not assessed. Computed matrix: FR-049-AC-1 through AC-6 are tagged. The optional semantic review was not selected; this scoped PR check still compared the changed tests to the six criteria. QSL-712 per-item pairing remains downstream.

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | Fixed in the reviewed PR. |
| FND-002 | fixed | Fixed in the reviewed PR. |
| FND-003 | fixed | Fixed in the reviewed PR. |
