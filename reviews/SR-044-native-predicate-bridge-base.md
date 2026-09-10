---
id: SR-044
title: "Base review of native predicate to TL projection"
type: SpecReview
analysis: base
scope: "issue #63; FR-025; STD-001 issue-63 codes; TC-038; SUITE-005 through SUITE-008 at cf4beaf15e35dfe276749637dbfb16c230070514"
review_set: all
relationships:
  - target: ix://agent-ix/quire-contract-ir/issues/63
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-025
    type: reviews
---
# SR-044: Base review of native predicate to TL projection

## Summary

FR-025 defines a pure, versioned bridge from checked native Quire Boolean
predicates to internal tl-syntax signal/catalog artifacts. Native Quire remains
the sole editable source language. The reviewed requirement does not claim an
implemented producer, formula-complete temporal lowering, evaluator, evidence
result, or release.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-6301 | high | **FIXED:** A caller-selected predicate set could not prove complete coverage of an absent temporal formula. FR-025 now makes this bridge formula-independent; issue #64 must call real `bind_formula` and refuse each missing occurrence. | FR-025 Behavior; FR-025-AC-1/8 | wrong-requirement |
| FND-6302 | high | **FIXED:** The draft conflated proposition-map entries with proposition-to-signal bindings. The two target documents, sibling identities, public readers and cross-document join are now distinct and exact. | FR-025 Public v1 records; Behavior | wrong-requirement |
| FND-6303 | high | **FIXED:** The initial 100,000-item bound exceeded FR-019's 10,000-item public collection limit. Predicate, fact, correspondence and gap populations now use the 10,000/10,001 boundary. | FR-025 Bounds; FR-025-AC-6 | wrong-requirement |

## Reviewed Revision and Gate

The reviewed specification revision is
`cf4beaf15e35dfe276749637dbfb16c230070514`. Strict Quire validation reports
47/47 documents grammar-clean. TC-038 and SUITE-005 through SUITE-008 are
deliberately planned and unbacked; this review is not implementation evidence.

## Verdict

**PASS for specification.** Implementation remains blocked on the exact native
source/result/mapping/availability contracts and accepted tl-syntax target
selections named by FR-025.
