---
id: SR-035
title: "Failure-domain review of bounded Kani K0"
type: SpecReview
analysis: failure-domain
scope: "FR-029, FR-030, FR-031, AD-002, AP-002, MP-002, TC-042"
review_set: all
---
# SR-035: Failure-domain review of bounded Kani K0

## Summary

The K0 boundary explicitly preserves invalid, incomplete, unavailable, exhausted, cancelled, timed-out, refused, and inconclusive states; it also names identity and graph termination boundaries. No unaddressed failure-domain gap was found in the reviewed design.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- |
| FND-351 | low | No issue found: pre-harness population validation, typed non-success outcomes, exact identities, and bounded graph/module ownership cover the reviewed extension, identity, purity, and topology failure domains. | FR-029, FR-030, FR-031 |
