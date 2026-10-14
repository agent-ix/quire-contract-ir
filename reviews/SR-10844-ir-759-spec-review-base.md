---
id: SR-10844
title: "Base spec review — IR-759 source-obligation correspondence"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; FR-052, TC-452, FR-019 addition, spec.md index"
review_set: subset
---

## Summary

The new identifiers and links validate, and TC-452 supplies planned procedures for the new behavior. AC-4 asks for a test to vary values that the same FR explicitly forbids as constructor inputs, and the error classification for incomplete inventories has no matching acceptance or procedure; no applicable AssuranceProfile was found.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-4's test requires varying proof/method/profile values that the constructor cannot accept | FR-052-AC-4 |
| FND-002 | medium | The typed missing/incomplete-inventory error has no acceptance criterion or test procedure | FR-052 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | medium | The `InvalidSourceSelection` error remains without an acceptance criterion or test case | FR-052, TC-452 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The AC now requires inspection of the constructor signature to confirm evidence and method/profile inputs are absent, instead of varying nonexistent arguments. |
| FND-002 | fixed | FR-052-AC-6 now exercises omitted and incomplete owner envelopes and their typed errors. |
| FND-003 | still-open | The error contract still lists `InvalidSourceSelection`, but the criteria and TC use only a valid selection and inventory mismatches; they never supply an invalid top-level selection. |
