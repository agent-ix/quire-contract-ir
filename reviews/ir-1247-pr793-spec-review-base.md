---
id: SR-202611010110
title: "IR-1247 PR 793 base spec review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR #793; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/model/functional/FR-019-rust-library-interface.md"
review_set: subset
---

## Summary

Reviewed FR-038 AC-387 through AC-389 and FR-019 AC-17 against QSpec FR-322-AC-299, its dimension-equivalence body rule, and XP-01 through XP-15. No applicable AssuranceProfile was found under `spec/`.

## Verdict

**PASS** — the criteria distinguish local NodeRef and direct-dependency PackageNodeRef operands, test exact typed refusals, and defer the public inventory until code lands.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-038 dimension operand paragraph, role: examined. The carrier is closed, the dependency boundary is direct, and the three refusal locations are named.
- FR-038-AC-387, role: examined. XP-01/10 admissions and writer preservation make the two typed forms observable.
- FR-038-AC-388, role: examined. XP-11/12/15 and XP-13 distinguish missing selection from missing name at exact member paths.
- FR-038-AC-389, role: examined. XP-14, wrong-kind operands, and pair consistency are asserted with typed refusals.
- FR-019 planned API paragraph, inventory row, and AC-17, role: examined. Default-feature external construction and distinct closed wire forms can be compile-tested.
- QSpec FR-322-AC-299 and dimension-equivalence body rule, role: context_only. They define the authoritative operand forms and first-refusal order.

## Evidence

Both changed files pass scoped Quire validation. The sole FR-038 grammar warning is on an unchanged line. New criteria have Test verification cells and are planned pending the IR-1247 code PR; no full code gate applies to this spec-only PR.
