---
id: SR-552
title: "Risk-complexity review of replay-result integrity remediation"
type: SpecReview
analysis: risk-complexity
scope: "FR-037, TC-046"
review_set: all
---
# Risk-complexity review of replay-result integrity remediation

## Summary

The digest has a deliberately broad input surface, but it reduces a
high-consequence ambiguity at the provider-to-native replay boundary.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-5521 | medium | Canonical encoding of typed values, especially IEEE representations and pre-state, remains a high-risk implementation detail that #100 and runtime #16 must make deterministic. | FR-037 AC-4; PLAN-009 |
