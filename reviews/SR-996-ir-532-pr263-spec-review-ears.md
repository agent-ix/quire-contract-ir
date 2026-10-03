---
id: SR-996
title: "AC-shape review of PR 263 (FR-038-AC-81 to AC-88)"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir@376269e0d67ee1451ee9629dbe897c06a8a730de; spec/checked_package/functional/FR-038-consume-checked-package-v2.md FR-038-AC-81 to FR-038-AC-88 and the two new prose sections"
review_set: subset
---
# SR-996: AC-shape review of PR 263

## Summary

Ticket: IR-532. FR-038's ACs are direct assertions, not EARS `shall` sentences.
This repo's convention for FR-038 is that each AC names a concrete instance, the
refusal code, the cause and the pointer. The new ACs follow it. `quire validate`
reports the grammar baseline unchanged: 326/327 docs clean, with the one finding
in FR-014 line 137 (`ac:vague-response`) that is already on main. No new AC trips
a grammar rule.

Each of AC-81 to AC-88 names an instance, a code/cause and a pointer, or an
admission, and none uses a vague outcome verb. AC-81 carries an admission control.
AC-86 and AC-88 are white-box (see SR-997).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | AC-84's first instance, `quire.op.quantity.convert` "with no member", does not say that its `rounding` mode is supplied. Without one the reader refuses `operation-mode-mismatch` first, and the new section disclaims any order, so the instance as written does not decide one outcome. The test and TC-048 set the mode | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1406 |
| FND-002 | low | AC-85 bundles five behaviours into one row: arity, literal operand family, clause family, literal mode pin and `same_type` over parameters, literals and applications. A failure does not localise, and the mode-pin clause repeats AC-83. Splitting it into arity/family (one AC) and `same_type` (one AC) would match the test split | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1407 |

## Verdict

The shape is acceptable apart from two low findings. No `shall`-form changes are
needed.

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 9d4379b |
| FND-002 | fixed | 9d4379b |
