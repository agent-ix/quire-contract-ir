---
id: SR-549
title: "Integrity review of replay-result integrity remediation"
type: SpecReview
analysis: integrity
scope: "FR-037, TC-046"
review_set: all
---
# Integrity review of replay-result integrity remediation

## Summary

The specified digest covers package and claim identity, profile, provider run,
typed values including IEEE representation, pre-state, selected occurrences,
and reported backend verdict.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-5491 | low | The canonical immutable-result digest closes the identity gap that allowed a same-domain but substituted failing payload to pass result verification. | FR-037 AC-4, AC-5; TC-046 | missing-requirement |
