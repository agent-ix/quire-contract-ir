---
id: SR-1882
title: "spec-review/integrity review of IR-648 scalar operands"
type: SpecReview
analysis: integrity
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
| FND-001 | high | The new unconditional exact-i128 result conflicts with FR-038’s existing unlimited integer-range admission and explicit out-of-i128 treatment. Resolve the domain and result/refusal contract before implementation. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3245 |

## Dispositions

| FND | Outcome | SHA / reason |
| --- | --- | --- |
| FND-001 | fixed | 99799615409fce555e6ef0b988d5b7c385ef43b9 — The revised result contract explicitly separates package admission from accessor eligibility and preserves exact values through typed refusals. |

## Disposition Verdict

**PASS at 99799615409fce555e6ef0b988d5b7c385ef43b9** — FND-001 is fixed; no new defect found in the changed obligations.
