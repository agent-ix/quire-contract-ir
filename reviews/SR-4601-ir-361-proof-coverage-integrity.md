---
id: SR-4601
title: "Integrity review of IR-361 proof coverage"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@0eca02b43bd14db1411abff447a45eacced9752f; AD-008, FR-045, StR-004, TC-446, matrix/index edits"
review_set: subset
---

## Summary

Examined the claim census, result join, and requirement/module/function/obligation projections in AD-008 and FR-045, including all six FR-045 criteria. One identity rule undermines a stable source-derived denominator.

## Verdict

FAIL. Define a source-semantic candidate identity independently of evidence properties, then bind method, proof subject, backend/profile revision and run to that candidate as evidence dimensions. Test that adding or changing a producer, profile or proof subject does not mint or remove source candidates.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The semantic claim key includes method, proof subject and backend/profile revision, although the candidate population is supposed to derive from requirement, obligation and production source inventories before results. A single source obligation with both production and shadow attempts, or a revised backend profile, becomes multiple or different claims. Thus evidence configuration can change the denominator or split the two strength views across different claims. Keep the semantic candidate key source-based; join evidence by exact candidate, selected source/profile, method, subject, domain and run as separate fields. | spec/assurance/AD-008-proof-coverage-accounting.md:47; spec/core/functional/FR-045-derive-proof-coverage-baseline.md:39 |

## Dispositions

Round 1 reviewed at f9bf82c7f62728cbcc60f994541735b4f366834a. Scoped fix review found no new defect.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | f9bf82c7f62728cbcc60f994541735b4f366834a: AD-008 now keys the source-semantic candidate by criterion, semantic obligation, selected source identity, and declared domain. Method, subject, profile, assumptions, run and effective bounds are joined evidence attempts; FR-045-AC-1 and TC-446 assert attempts leave candidate keys and denominators stable. |

After excerpt: “The accounting unit is a source-semantic candidate claim, keyed by its owning requirement criterion, semantic-obligation identity, source identity at the selected source state, and declared domain.”
