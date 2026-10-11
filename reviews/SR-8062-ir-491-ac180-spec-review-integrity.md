---
id: SR-8062
title: IR-491 AC-180 correction integrity review
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@7d0adeb864110b696a568f8983f64e766ac0615c; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md; IR-491"
review_set: subset
---

## Summary

The exhaustive mapping and corrected test plan agree on the 15-plus-3 split, but the type-walk table's first-column heading still calls all eighteen forms admitted.

## Scope

- `FR-038` type-walk table (examined): the first column says `Admitted form` and includes `float32`, `float64` and `float_rounding`.
- `FR-038` reader-admission paragraph (examined): expressly excludes those three forms from admitted structural comparisons.
- `FR-038-AC-180` (examined): assigns the excluded forms to private type-walk controls and public refusals.
- `TC-048` AC180 paragraph (examined): separates the fifteen public positives from the three private controls and public refusals.
- `FR-038-AC-231` (context_only): supports the public IEEE refusal rule.

## Verdict

**FINDING** — Rename the table's `Admitted form` heading to `Type form` (or equivalent) so the table does not imply public admission of the three IEEE forms. The adjacent prose makes the intended behavior clear; this is low severity.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The exhaustive type-walk table labels three non-admitted IEEE forms as `Admitted form`; use `Type form` or equivalent. | FR-038:3147; FR-038-AC-180; TC-048 |

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 4a3b1a09a169fe6bfeb859f78c4a6e4011286e3d — `| Type form | Introduced descriptor and source node |` |
