---
id: SR-20261104020012
title: "IR-1322 PR 884 base specification review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR 884; FR-038-AC-455..459 and QSpec protocol manifest"
review_set: subset
---

## Summary

Independent base review of five planned reader criteria against the published QSpec protocol mutation manifest and the current QSpec requirements.

## Verdict

PASS. AC-455 through AC-459 name exactly V2, V5, V8, V9, V10, V11, RP6, V16, CP28, C11 and C12. Their stage, code, cause and primary pointer match the manifest. AC-456 preserves the five published field-level document pointers, AC-458 preserves C11's target form and keys, and AC-459 requires an admission/refusal assertion against the selected QSpec checkout. All new criteria are marked PLANNED/UNRUN until reader code and tests land. M1 and the four QSpec prose/schema reconciliation rows are outside this change.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Examined Scope

- role: examined; id: FR-038-AC-455; excerpt: V2 is an operator-ineligible reader refusal at node 64; V5 is a malformed-wire schema refusal at node 78.
- role: examined; id: FR-038-AC-456; excerpt: V8-V11 and RP6 retain node-level primary pointers and published field document pointers.
- role: examined; id: FR-038-AC-457; excerpt: V16 and CP28 refuse missing-selection at compensation node 81.
- role: examined; id: FR-038-AC-458; excerpt: C11 retains role target kind and keys; C12's unattached control wins at node 67.
- role: examined; id: FR-038-AC-459; excerpt: TC-048 reads the QSpec manifest and base package by reference, with no local fixture copy or frozen row count.
- role: context_only; id: FR-464-AC-24; excerpt: strict schema layout refusal uses the validator's first node-level error path.
- role: context_only; id: FR-464-AC-6; excerpt: relationship target form and containment/listing are checked as published.
- role: context_only; id: FR-464-AC-11; excerpt: absent temporal-profile selection for an await or compensate refuses missing-selection.

## Verification and Limits

Compared the PR diff with QSpec main ad180f16 manifest entries, FR-464 reader order and membership rules, FR-272 document-pointer meaning, and FR-466 guard rule. This review verifies the specification; it does not claim the IR reader already conforms. The current-main replay reported 11 relevant first mismatches and stops at each first mismatch, so later unprinted diagnostic fields are not inferred.
