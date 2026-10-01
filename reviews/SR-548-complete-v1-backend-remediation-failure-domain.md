---
id: SR-548
title: "Failure-domain review of replay-result integrity remediation"
type: SpecReview
analysis: failure-domain
scope: "FR-037, TC-046"
review_set: all
---
# Failure-domain review of replay-result integrity remediation

## Summary

The replay trust boundary is now explicit: a transport-valid result is not
replayable until its immutable result digest verifies.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-5481 | low | Substitution of a domain-valid counterexample, state, occurrence, or verdict is now a typed integrity refusal before native execution rather than a potentially accepted replay. | FR-037 AC-4, AC-5; TC-046 | missing-requirement |
