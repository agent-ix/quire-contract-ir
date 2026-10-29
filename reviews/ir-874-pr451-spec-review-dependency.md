---
id: SR-12450
title: "IR-874 PR 451 spec-review-dependency review"
type: SpecReview
analysis: dependency
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md"
review_set: subset
---

## Summary

Checked new QSpec FR-464/465/466 and TC-946/947/948 relationships against the referenced documents. Dependency edges are coherent; the missing published machine vectors are recorded in evidence review. Ticket: IR-874. Frozen head:.

## Examined units

- **FR-038** (`examined`): When a V2 package contains a `protocol` node other than the `protocol_reference` carrier, the reader SHALL apply the exact body, key, declaration and dependency rules of QSpec FR-464, FR-465 and FR-466.
- **TC-048** (`examined`): Use QSpec FR-464, FR-465 and FR-466 and their TC-946, TC-947 and TC-948 vectors as the independent expected outcomes.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

Checked new QSpec FR-464/465/466 and TC-946/947/948 relationships against the referenced documents. Dependency edges are coherent; the missing published machine vectors are recorded in evidence review.
