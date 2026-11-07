---
id: SR-2026110602354654
title: "ears-conformance review of IR-1387 PR #943"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir; IR-1387 PR #943; spec/checked_package/functional/FR-357-project-selected-generated-type-closure.md, spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md, spec/model/functional/FR-019-rust-library-interface.md, spec/checked_package/matrix/TC-467-native-selected-identity-join-budget.md"
review_set: subset
---

## Summary

Reviewed changed requirement statements under FR-357 and FR-358 for named actors, concrete responses and EARS grammar. No scoped defect found; AC table rows and TC prose are outside this lens.

## Verdict

**PASS** — No scoped findings.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-357-AC-15 (examined): Implemented (IR-1387 public budget API; TC-467). A driver can debit selection and adapter work and storage before allocation through one exported finite budget, then pass that same budget to a native read. The combined exact limit succeeds and one below refuses without resetting prior consumption or changing original limits. Driver charges retain their first unpaid kind, attempted amount and locus.
- FR-358-AC-9 (examined): Two external call sites use public `resume` and `accounted_charge` on one `NativeJoinBudget` before passing it to a native stage and projection read. Exact combined work and storage limits succeed; one below refuses with typed first unpaid charge, original bound and first locus. An external zero-storage denial occurs before allocation and does not consume storage. Resuming preserves all spent counters and configured limits; the budget offers no public counter reset or clone.
- FR-019 (examined): NativeJoinBudget exports resume and accounted_charge for external driver selection and adapter charges before allocation. These methods reuse its existing finite counters and first-unpaid metadata; they add no second budget type or public counter-reset surface.
- TC-467 (examined): From public consumer code, debit selection work at one call site and adapter work and storage at another through accounted_charge. Resume the same budget without restoring its counters, then pass it to the native stage and projection reads. Exercise the exact combined allowance, one-below work and zero-storage external charge.
