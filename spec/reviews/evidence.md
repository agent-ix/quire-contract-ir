---
id: SR-038
title: "Evidence review of bounded Kani K0"
type: SpecReview
analysis: evidence
scope: "FR-029, FR-030, FR-031, AP-002, MP-002, TC-042"
review_set: all
---
# SR-038: Evidence review of bounded Kani K0

## Summary

TC-042 and MP-002 specify exhaustive per-case profile-matrix parity and native replay evidence. The installed `quoin advise --json` could not interrogate the available `quire` binary because it did not expose the JSON version contract required by that advisor, so no advisor recommendation is claimed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- |
| FND-381 | low | Evidence-advisor execution is inconclusive because `quoin advise` could not determine a compatible Quire CLI JSON version; the authored Test method remains an explicit human judgement and must be revisited when the compatible advisor path is available. | FR-029, FR-030, FR-031, TC-042 |
