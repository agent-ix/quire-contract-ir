---
id: SR-11193
title: "Failure-domain review of IR-800 native scalar reader requirements"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-contract-ir; FR-038-AC-242..245 and TC-048 native String/UUID/Timestamp section"
review_set: subset
---

## Summary

Ticket IR-800. Checked family confusion, literal type-before-value precedence, noncanonical spellings, identity rekeying, selected-source ownership and bounded-work precedence. No additional failure-domain defect beyond SR-11191's missing adverse permutations.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | (placeholder) No additional issue found. | - |

## Boundary checks

The String form remains distinct from Text and cannot inherit a text profile by spelling. The exact Unicode sequence, UUID grouping/case and signed-128 Timestamp endpoints match the authoritative QSpec. TC-048 requires rekeying mutated packages so an earlier stale-key refusal cannot masquerade as the named operation-step refusal, while preserving an exact work-limit precedence control.
