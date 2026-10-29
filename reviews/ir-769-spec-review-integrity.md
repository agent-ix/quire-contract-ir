---
id: SR-8471
title: IR-769 integrity review of FR-049 and TC-447
type: SpecReview
analysis: integrity
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
| FND-001 | high | AC-4 requires refusal for a mismatched application occurrence, but its stated IR check authenticates only membership among the expression key’s source-map entries. RR-16 supplies two admitted application occurrences of one expression key. Supplying the other occurrence remains a valid map entry, so IR cannot identify it as the wrong QSL item without the per-item binding witness AC-5 says is absent. Limit refusal to absent or key-mismatched application occurrences, and verify valid occurrence pairing at the QSL driver. | spec/checked_package/functional/FR-049-admitted-scalar-result-context.md:207 |

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | Fixed in the reviewed PR. |
