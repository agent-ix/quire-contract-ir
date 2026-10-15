---
id: SR-11195
title: "Scope-boundary review of IR-800 native scalar reader requirements"
type: SpecReview
analysis: scope-boundary
scope: "agent-ix/quire-contract-ir; FR-038-AC-242..245 and TC-048 native String/UUID/Timestamp section"
review_set: subset
---

## Summary

Ticket IR-800. IR owns strict V2 vocabulary, operation-family checks, literal admission and typed refusals; QSpec supplies the wire and catalog contract; QSL owns source production and end-to-end emission. The amendment reads existing selected `typeRef` and adds no FCD source-document member or unmeasured FCD API requirement.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | (placeholder) No issue found. | - |

## External contract

QSpec FR-322-AC-31/67 at 9b6a84d is the normative external dependency. Its six native operation identities, scalar forms and canonical payload rules are checked against the proposed IR criteria. The branch adds no new `relationships:` edge, so the dependency-analysis lens is not applicable.
