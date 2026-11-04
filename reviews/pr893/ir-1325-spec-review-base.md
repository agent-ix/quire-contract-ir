---
id: SR-2026110413592004
title: "IR-1325 PR 893 spec-review-base"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR 893; spec/checked_package/functional/FR-038-consume-checked-package-v2.md"
review_set: base
---

## Summary

Independent scoped spec-review-base of the FR-038-AC-460 status change at PR #893 head b8ce731f4f9c02371d9c6c51b19dbb79931828b1. Ticket: IR-1325.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Examined Scope

- role: examined; id: FR-038-AC-460; excerpt: Implemented. A public read-only `CheckedPackageV2::direct_dependencies()` iteration borrows each admitted direct dependency package in `lock.dependency_selections` order. A package selecting `geometry` then `units` yields the two corresponding complete admitted values, including distinct non-name content, and no clone or identity-only surrogate; a root selecting one child whose own lock selects a grandchild yields only that child, whose accessor yields the grandchild. A package with no direct selections yields no item. The returned borrows cannot outlive their owning package.

## Analysis

The changed status is testable and accurately describes the merged implementation. Selection order, complete value retention, empty selections, direct-only iteration, and borrowing are specific outcomes. The merged test changes `units` source-map content beyond its name and compares complete admitted values; a second tagged test checks the transitive boundary. The criterion neither claims a transitive closure from the root accessor nor introduces a digest requirement. The unchanged neighboring prose says dependency conflicts use structural equality, with no computed hash.

## Verification and Limits

Released Quire 0.38.3 validated the scoped FR-038 document with exit 0. It reported one pre-existing non-canonical-shape advisory at line 5693, outside AC-460. Properties reported 169/448 extractable and zero candidates. Quire trace found two structural verification claims for AC-460. No applicable AssuranceProfile was found under spec/; no full gate was run.

## Verdict

PASS. The status change reflects evidence already merged on main and adds no new requirement or ceremony.
