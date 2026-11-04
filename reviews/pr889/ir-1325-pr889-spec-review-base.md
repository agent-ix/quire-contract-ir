---
id: SR-20261104133702
title: "IR-1325 PR 889 spec-review-base review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR 889; FR-038-AC-460 and direct-dependency read statement"
review_set: subset
---

## Summary

Independent spec-review/base review of frozen PR #889 at 231f46a8fce312adbea95da850d4f81a5ac8dbb7. Ticket: IR-1325.

## Verdict

PASS. The criterion states direct-only scope, order, empty selection, distinct admitted values and borrow lifetime. Each case can be checked with admitted dependency packages.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Examined Scope

- role: examined; id: FR-038-AC-460; excerpt: PLANNED/UNRUN (IR-1325). A public read-only `CheckedPackageV2::direct_dependencies()` iteration borrows each admitted direct dependency package in `lock.dependency_selections` order. A package selecting `geometry` then `units` yields the two corresponding complete admitted values, including distinct non-name content, and no clone or identity-only surrogate; a root selecting one child whose own lock selects a grandchild yields only that child, whose accessor yields the grandchild. A package with no direct selections yields no item. The returned borrows cannot outlive their owning package.
- role: examined; id: FR-038; excerpt: After admission, `CheckedPackageV2::direct_dependencies()` SHALL expose the retained admitted dependency package values as immutable borrows in the same order as `lock.dependency_selections`. The accessor SHALL yield only direct dependencies of that package: a child's selected dependencies remain available through that child's accessor and SHALL NOT be inserted into its parent's iteration. An empty direct selection yields an empty iteration. The accessor SHALL borrow the packages already retained by the reader; it SHALL NOT copy their complete values, rebuild a dependency closure, or return only the selection identities.

## Verification and Limits

Compared the criterion with the reader’s retained direct dependency vector, admission order and QSL FR-531’s dependency-closure use. The criterion remains PLANNED/UNRUN.
