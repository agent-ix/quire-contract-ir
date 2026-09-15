---
id: SR-062
title: "Failure-domain review of complete-V1 Contract IR backend delivery"
type: SpecReview
analysis: failure-domain
scope: "AD-003, FR-035 through FR-037, TC-044 through TC-046"
review_set: all
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-035
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-036
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-037
    type: reviews
---
# SR-062: Failure-domain review of complete-V1 Contract IR backend delivery

## Summary

The boundary now names stale identity, unknown version, malformed input,
missing bounds, unavailable runtime, resource exhaustion, decode failure, and
verdict disagreement as independently typed outcomes. No callback, foreign
runtime, or target parser enters the semantic path.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-602 | low | Closed during review: FR-035 now requires a per-item `failed` result without a substitute node when a declared resource limit is exceeded, preventing limit exhaustion from becoming partial meaning. | FR-035 Behavior; TC-044 | missing-requirement |
