---
id: SR-2026110414560077
title: "IR-1352 PR 919 FR-038 integrity review"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; PR 919; spec/checked_package/functional/FR-038-consume-checked-package-v2.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: reviews
---

## Summary

Ticket: IR-1352. Checked FR-038-AC-472's completeness, failure-domain identity, validation order, and consistency with AC-465/466 and QSpec FR-440. No applicable AssuranceProfile exists under `spec/`.

## Verdict

PASS. The new rule allocates only protocol-slot scope to IR, while QSpec retains the case wire and binder-level definition. Exact binder node identity prevents same-name sibling laundering; separate positive and refusal controls address leakage across arms, the scrutinee, and outer expressions. The barred-state and older let/query outcomes remain explicit. The test method and QSpec references are present.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-038 case-arm slot paragraph near line 1850, role: examined. Scope ownership, identity and precedence.
- FR-038-AC-472, role: examined. Admitted-value and refusal obligations.
- FR-038-AC-465/466, role: examined. Existing local-scope behavior and nonleak controls.
- QSpec FR-440-AC-3/4/5, role: context_only. Authoritative shape and binder-level rules.
