---
id: SR-065
title: "Evidence review of complete-V1 Contract IR backend delivery"
type: SpecReview
analysis: evidence
scope: "FR-035 through FR-037, TC-044 through TC-046, TM-002"
review_set: all
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-035
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-036
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-037
    type: reviews
---
# SR-065: Evidence review of complete-V1 Contract IR backend delivery

## Summary

The reviewed criteria use concrete Test controls and identify their QSpec
portable controls. The installed `quoin advise --json` path is unavailable
because it rejects the installed Quire CLI JSON version contract, so no
catalog-derived recommendation is claimed.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-605 | low | Evidence-advisor execution is inconclusive: Quoin cannot determine a compatible Quire CLI JSON contract. The authored Test methods are explicit human judgement and must be revisited when the compatible advisor path is installed. | FR-035 through FR-037; TC-044 through TC-046 | correct-requirement-no-evidence |
