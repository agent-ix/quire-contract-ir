---
id: SR-2026110602354631
title: "FR-358 amendment integrity review"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; PR #934; FR-358-AC-2/AC-3; spec/checked_package/functional/FR-358-compose-native-identity-join-budget.md"
review_set: subset
---

## Summary

Ticket: IR-1367. Reviewed the amended FR-358 ownership and budget rules for completeness, consistency, atomicity, failure cases, and trace to AC2/AC3. No applicable AssuranceProfile was found under spec/.

## Verdict

**PASS** — The requested and first-locus footprints, form unit and bytes, and envelope unit define one diagnostic floor. Both affordable and unaffordable paths preserve the original unpaid charge; the scope boundary with FR-359 remains explicit.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-358-AC-2, role: examined. Exact and one-below limits, typed exhausted kind and charge, conditional first locus, compact refusal and first unpaid charge.
- FR-358-AC-3, role: examined. Logical storage precharge, diagnostic floor, zero-storage refusal and prior nonempty map copy.
- FR-358 Outputs and changed Behavior, role: examined. Owned diagnostic when affordable; compact refusal otherwise; FR-359 boundary.
- FR-358-AC-4, role: context_only. Refusal counter and first unpaid charge semantics.

## Evidence

The six native acquisition families share AC2/AC3; the amendment states exact, one-below and zero-storage boundaries and does not introduce a new dependency edge or domain object.
