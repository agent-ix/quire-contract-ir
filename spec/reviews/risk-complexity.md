---
id: SR-039
title: "Risk and complexity review of bounded Kani K0"
type: SpecReview
analysis: risk-complexity
scope: "FR-029, FR-030, FR-031, AD-002, AP-002, MP-002, TC-042"
review_set: all
---
# SR-039: Risk and complexity review of bounded Kani K0

## Summary

The highest technical risks are profile/tool drift, accidental invalid-input assumptions, and failed native replay. Their mitigations are immutable provenance, pre-harness validation, exhaustive typed-outcome corpus coverage, and replay agreement.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- |
| FND-391 | low | No issue found: the high-risk bounded-Kani integration is sliced by shared enablement and separate semantic modules, while AP-002 and MP-002 require explicit parity/replay evidence before a candidate decision. | AD-002, AP-002, MP-002 |
