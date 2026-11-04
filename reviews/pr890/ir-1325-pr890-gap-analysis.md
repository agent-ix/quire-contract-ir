---
id: SR-2026110413592002
title: "IR-1325 PR 890 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR 890 at ee68bad80; FR-038-AC-460; tests/it/checked_package_v2_dependency_selections.rs; crates/quire-contract-model/src/checked_package/v2/mod.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: reviews
---

## Summary

Ticket: IR-1325. Scoped reconciliation of FR-038-AC-460, the computed Quire Test Matrix, tagged tests, and changed implementation. No aggregate gate was run.

## Verdict

PASS for the PR diff. Quire 0.38.3 reports FR-038-AC-460 tagged by the two new or extended TC-048 tests. Their assertions exercise the criterion's distinct-value, order, direct-only, and empty behaviors through the public strict reader and accessor. No PR-scope traceability or test-oracle gap was found.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-038-AC-460 (examined, `spec/checked_package/functional/FR-038-consume-checked-package-v2.md:5762`): public borrowed iteration over admitted direct values in lock order; distinct content, empty case, one-edge traversal, owner-bound lifetime.
- Computed matrix (examined, `quire matrix --scope . --format json`): status `tagged`; binders `ascending_dependency_names_admit_and_are_retained` at line 99 and `direct_dependency_accessor_stops_at_one_edge` at line 136.
- Test intent (examined, `tests/it/checked_package_v2_dependency_selections.rs:90` and `:136`): admission creates two values with different source map content, compares ordered complete values, and checks child/grandchild separation and empty iterators.
- Reverse gap (examined, `crates/quire-contract-model/src/checked_package/v2/mod.rs:1327`): the changed accessor is owned by FR-038-AC-460. The documentation correction in `evidence.rs` removes a stale digest claim and adds no behavior.

Plan completion and repository-wide gaps were not assessed. Optional semantic review was not run; the criterion's test intent was checked directly in this review.
