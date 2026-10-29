---
id: SR-8469
title: IR-769 base review of FR-049 and TC-447
type: SpecReview
analysis: base
scope: agent-ix/quire-contract-ir; spec/checked_package/functional/FR-049-admitted-scalar-result-context.md,
  spec/checked_package/matrix/TC-447-admitted-scalar-result-context.md
review_set: subset
---

## Summary

Ticket IR-769. Examined FR-049 and TC-447 at the reviewed head against QSL FR-062/FR-075 and the FR-035/FR-330 package boundary. Examined units: FR-049, FR-049-AC-1, FR-049-AC-2, FR-049-AC-3, FR-049-AC-4, FR-049-AC-5, FR-049-AC-6, TC-447.

## Verdict

**FAIL** — The stated refusal exceeds what the admitted six-field package can verify.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | AC-3 requires IncompleteContext for a wrong conversion ordinal without limiting this to an absent or structurally invalid occurrence. A different admitted expression-role occurrence of the same represented direct narrow can have another ordinal yet pass every stated IR source-map, lowered-root, operation and direct-reference check. AC-5 and TC-447 expressly say IR cannot detect a swapped valid context. Qualify this refusal to invalid or absent keys and require the QSL driver pairing test for a valid-key swap. | spec/checked_package/functional/FR-049-admitted-scalar-result-context.md:206 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Fixed in the reviewed PR. |
