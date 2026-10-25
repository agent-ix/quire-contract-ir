---
id: SR-565874
title: "IR-874 PR 565 protocol order and activation spec review"
type: SpecReview
analysis: base
scope: "spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md"
review_set: base
---

## Summary

Reviewed PR #565 at `cb98dbfbdff22822ecff187b3b436cfff2f98a4a` against QSpec FR-322, FR-375, FR-464 and TC-310, TC-946 through TC-948, then checked the FND-001 correction at `9aa73406f26328094fa36b21c2570bc22b8540d8`. The body-step order, activation relation, class-catalog comparison and refusal precedence match the source contracts. No applicable AssuranceProfile was found under this repository's `spec/` tree.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-324 and TC-048 step 7 say a repeated or duplicate requirement yields one admitted activation row. QSpec FR-464 canonical sets refuse a repeated `requirements` wire entry as `invalid_semantic_graph`; TC-946 A4 instead repeats a source `requires temporal` declaration, which the producer normalizes to one wire entry. State that distinction explicitly so the reader test cannot accidentally admit noncanonical duplicate wire entries. | `spec/checked_package/functional/FR-038-consume-checked-package-v2.md:5218`; `spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:1676`; QSpec FR-464 "Canonical order" and TC-946 A4 |

## Verdict

PASS after the FND-001 correction. The existing AC-282 canonical-set refusal remains in force.

## Dispositions

| FND | Outcome | Evidence |
| --- | --- | --- |
| FND-001 | fixed | At `9aa73406`, FR-038-AC-324 and TC-048 step 7 distinguish two source declarations normalized to one canonical wire entry from duplicate wire entries that refuse `invalid_semantic_graph` with no activation relation. The prose states the same rule. This agrees with QSpec FR-464 canonical order and TC-946 A4. |
