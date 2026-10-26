---
id: SR-590009
title: "Gap analysis — IR-1028 indexed application group key lookup"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, crates/quire-contract-model/src/checked_package/v2/application_identity.rs, crates/quire-contract-model/src/checked_package/v2/operations.rs"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-contract-ir/FR-038", type: references }
---

## Summary

Ticket: IR-1028. Audited the changed key lookup against FR-038-AC-88, its computed trace binding, and the affected production and test paths. This is a PR-scoped, planless review.

## Verdict

**PASS** — The changed behavior has an owning criterion and a real traced test; no stub, untraced behavior, or coverage inflation was found in the changed scope.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- Reconciliation: quire matrix (static trace axis; no run evidence read). FR-038-AC-88 status: tagged; binders include the new 100,000-member test and four pre-existing tests.
- Plan completion: not assessed
- Changed behavior inventory: indexed group lookup, graph-order ordinal assignment, application-key re-derivation. Untraced behaviors: 0; source stubs: 0; test stubs: 0 in the changed scope.
- The author reports the new test passed (1/1, 38.34s), related unit tests passed, and public reader test passed. The reviewer ran no build under the host cap; final aggregate gate belongs to the dispatching lead.
- Semantic review: acceptance-criterion-to-code/test agreement examined for FR-038-AC-88; no end-to-end QSL-authored protocol-package Verified claim was assessed.
