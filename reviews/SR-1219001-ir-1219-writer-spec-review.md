---
id: SR-1219001
title: "IR-1219 checked-term-data writer contract review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR #757; FR-051, TC-451, FR-019"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-051
    type: reviews
---
# IR-1219 checked-term-data writer contract review

## Summary

Reviewed PR #757's writer input and encoding criteria against QSpec FR-322's optional `checked_term_data` object and the planned separate IR equality, protocol-location, and dispatch readers. The review covers the changed spec text only.

## Verdict

**PASS after disposition.** The writer contract now makes canonical byte construction independently testable without claiming unfinished reader admission.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The initial AC-5 and TC-451 required every positive sidecar vector to be admitted by the strict reader in the writer PR, although equality and protocol reads are separate work and the dispatch operation identity still needs a QSpec correction. | FR-051-AC-5; TC-451 procedure |

## Dispositions

- FND-001: **fixed** at `afa30b19`. AC-5 and TC-451 now test canonical encoding, ordering, presence, and byte limits; each read criterion owns its own admission outcomes.

## Coverage

- FR-051 inputs and behavior preserve `None` as omitted and `Some(empty)` as a present object with three arrays.
- FR-051-AC-1, AC-4, and AC-5 cover the public signature, canonical bytes, producer order, empty candidate rows, and exact byte limit.
- TC-451 and FR-019 agree with the revised public writer surface.
- Targeted Quire validation and property analysis were reported by the author; this review independently inspected the final PR diff and its corrected acceptance boundary.
