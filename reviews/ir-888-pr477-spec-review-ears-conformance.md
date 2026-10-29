---
id: SR-13141
title: "IR-888 quire-contract-ir PR 477 EARS analysis: FR-035 reader-outcome statements"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir PR #477 vs origin/main; spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-035
    type: reviews
---

## Summary

Ticket: IR-888. This is an EARS check of the requirement statements the PR adds to FR-035: the new Behavior paragraph at lines 139-145, and FR-035-AC-10 and FR-035-AC-11.

## Reviewed units

- FR-035 Behavior lines 139-141: "When reading a canonical `ContractPackageWireV1`, Contract IR SHALL refuse an unrecognized string `contract_version` with `unknown_contract_version` located at `/contract_version`, retaining the supplied version in the diagnostic." (examined)
- FR-035 Behavior lines 142-145: "When a caller-selected read limit is exhausted, the reader SHALL return `Incomplete`...; neither outcome admits a `ContractPackageV1`." (examined)
- FR-035-AC-10 (examined)
- FR-035-AC-11 (examined)

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The second sentence joins two obligations with a semicolon, and the second one ("neither outcome admits a `ContractPackageV1`") has no SHALL. It also switches subject from "Contract IR" to an undefined "the reader". It should be split into its own event-driven SHALL statement with one named subject | spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md:142-145 |

## Verdict

Both Behavior sentences follow the event-driven pattern (When..., <subject> SHALL...), with a single trigger each. FR-035-AC-10 and FR-035-AC-11 use the Given/returns form with observable typed outcomes. Only the low-severity compound clause above needs attention. Testability defects in AC-11 are recorded in SR-13140.
