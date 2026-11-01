---
id: SR-202611010108
title: "IR-1249 PR 789 failure-domain review"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-contract-ir; PR #789; spec/checked_package/functional/FR-038-consume-checked-package-v2.md"
review_set: subset
---

## Summary

Checked identity confusion and refusal precedence at the selected relationship role, model-population node, and producer-supplied destination label. No applicable AssuranceProfile was found under `spec/`.

## Verdict

**PASS** — the spec distinguishes the role-end type from a schema-valid `destination_type` label, gives an exact wrong-type refusal, and preserves the prior derived-shape and graph-term refusal boundaries.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-038 population comparison paragraph, role: examined. It names the selected relationship and role-end lookup, exact type comparison, and refusal locus.
- FR-038-AC-269, role: examined. It retains authentic selector identity and source-region requirements.
- FR-038-AC-274, role: examined. It distinguishes the producer label from the population element type and covers a different model type and subtype.
- FR-038-AC-279, role: examined. It retains negative/non-integer max, second member, wrong target, invalid body term, and noncanonical wire failure paths.
- QSpec FR-322-AC-73, AC-74, and AC-270, role: context_only. They establish the trust boundary between authored relationship type and producer-supplied destination label.

## Evidence

The Order/Shipment example specifically fails if the reader treats the producer label as the authoritative population object type. The refusal remains `ill_typed`/`operator-ineligible` at `member.population` when the population object type differs from the named role end.
