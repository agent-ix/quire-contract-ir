---
id: SR-202611081512300004
title: "Spec integrity — IR-1404 PR 999"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; spec/checked_package/matrix/tests.md; spec/checked_package/matrix/TC-472-strict-v2-read-exact-budget-charging.md"
review_set: subset
---

## Summary

Ticket: IR-1404. PR quire-contract-ir#999 at 53410c466a2db86d2e26a9cd995adfcd13e97f74, diff scope origin/main...HEAD only.

## Verdict

**FAIL** — The prior AC-512 and AC-513 conflict is fixed: AC-512 leaves the Planned range and AC-513 has its own row stating it is not complete. AC-513's row and the Planned row do not claim the AC-511 or AC-513 proofs are closed. The new Tested row overstates what the tests assert.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The matrix marks AC-503 and AC-507 Tested with 'exact' cases that the tests do not contain | spec/checked_package/matrix/tests.md:21-22 |
| FND-002 | medium | AC-512's row drops the corpus limit and now reads as whole-corpus Tested | spec/checked_package/matrix/tests.md:22 |

## Coverage

Examined units: tests-matrix-rows, FR-038-AC-503, FR-038-AC-507, FR-038-AC-512.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | low | The AC-507 row says each retained request is denied and admitted, but only the AC's seven forms are | spec/checked_package/matrix/tests.md:21 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Verified in fix commit b4c684ec47e638bcf13bfc87ee38c9214de56a3c |
| FND-002 | fixed | Verified in fix commit b4c684ec47e638bcf13bfc87ee38c9214de56a3c |
| FND-003 | fixed | Verified in fix commit bd9092b93c93357b7b85a5a2e7787bd6b76a2e6c |
