---
id: SR-1881
title: "spec-review/base review of IR-648 scalar operands"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@8b9752fbfe9cfb2bde786f96bc846786f0c7b8e0; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md"
review_set: subset
---

## Summary

Ticket: IR-648. Reviewed PR #303 at 8b9752fbfe9cfb2bde786f96bc846786f0c7b8e0. Examined FR-038-AC-159 through FR-038-AC-164 and TC-048.

## Verdict

**FAIL** — The findings below require a spec correction.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-038-AC-159 promises exact i128 ranges for every admitted integer application, but FR-038 permits unbounded Integer and admitted integer_range bounds beyond i128. Neither can produce that output. Limit the eligible domain or define a typed refusal for unrepresentable and unbounded ranges; add both cases to TC-048. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3245 |

## Dispositions

| FND | Outcome | SHA / reason |
| --- | --- | --- |
| FND-001 | fixed | 99799615409fce555e6ef0b988d5b7c385ef43b9 — AC-159 now limits successful results to finite i128-representable ranges; AC-162 and TC-048 specify typed refusals for excluded admitted operands. |

## Disposition Verdict

**PASS at 99799615409fce555e6ef0b988d5b7c385ef43b9** — FND-001 is fixed; no new defect found in the changed obligations.
