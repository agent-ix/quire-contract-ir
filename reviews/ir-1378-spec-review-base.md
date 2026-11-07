---
id: SR-2026110602354626
title: 'spec-review-base review of FR-350 PR #938'
type: SpecReview
analysis: base
scope: 'agent-ix/quire-contract-ir; spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md;
  IR-1378; PR #938'
review_set: subset
---

## Summary

The native EvidenceRequest and TraceBinding names, authentication call, and retained Report/evaluate_v2 API match public QSL. Two contract gaps remain in singleton occurrence selection and acceptance coverage. No applicable AssuranceProfile was found under `spec/`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The request occurrence is optional in QSL and None selects the sole occurrence. FR-350 does not define the effective occurrence in that case, so an implementation can either refuse a valid request or treat None as an unconstrained match. Define the exact singleton fallback and require the supplied ClauseOccurrence to match its node, role and ordinal. | FR-350 (`spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md:148-153`) |
| FND-002 | medium | The new binding, exact occurrence, trigger-instance, and opaque-anchor observation selection have no acceptance test with a concrete admitted value and mismatched/refused cases. AC-1 only checks owner-read output, AC-3 checks typed member mutation, and AC-6 checks valuation cells. Add a value-asserting criterion for the selected numeric owner anchor and refusals on mismatched binding, occurrence, instance, or observation identity. | FR-350-AC-1 (`spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md:316`) |

## Verdict

FAIL: corrections required before merge.
