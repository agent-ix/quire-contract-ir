---
id: SR-11814
title: "IR-759 Slice C evidence review of FR-052-AC-17..19 and TC-452"
type: SpecReview
analysis: evidence
scope: "agent-ix/quire-contract-ir@794d6e2 (PR 417, diff vs origin/main 6b80801); spec/model/functional/FR-052-source-obligation-correspondence.md, spec/model/matrix/TC-452-source-obligation-correspondence.md, spec/model/matrix/tests.md"
review_set: subset
date: '2026-10-16'
---

## Summary

Ticket: IR-759. Evidence analysis of FR-052-AC-17..19 (Verification: Test) and the TC-452 synthetic-census procedure at 794d6e2: whether the declared evidence can fail.

## Verdict

Test is the right method for all three; each has a concrete failing input. Two findings concern vacuous satisfaction. TC-452 and the tests.md row list AC-17..19 and quire matrix shows them untagged, as expected for Planned rows.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-17 runs on 'the AC-2 fixture shape', which has no QSL rows, so 'every QSL row is ZeroMappings(NoAuthoredMapping)' passes vacuously with an empty QSL outcome. TC-452 implies QSL rows exist but does not require them. Require at least one RequirementRecord and one PendingObligation row in the AC-17 fixture. | spec/model/functional/FR-052-source-obligation-correspondence.md:468; spec/model/matrix/TC-452-source-obligation-correspondence.md:154-158 |
| FND-002 | low | 'A fault in each of two owner positions' does not say which pairs; one pair satisfies it, yet the rule is an order over five positions (with the binding split per SR-11810 FND-001). Name the pairs (at least each adjacent pair) so a test that checks only Quire-versus-IR V2 cannot back the whole order. | spec/model/functional/FR-052-source-obligation-correspondence.md:469; spec/model/matrix/TC-452-source-obligation-correspondence.md:159-160 |
| FND-003 | low | TC-452 says the synthetic census 'matches the census construction rules field by field'. A test that recomputes the rules in a helper and compares is tautological; say the expected census is hand-authored per fixture. Also 'the occurrence under a conflicting node is DanglingOccurrence' misnames the subject: the assertion naming it is rejected DanglingOccurrence. | spec/model/matrix/TC-452-source-obligation-correspondence.md:215-223 |

## Examined units

```yaml
scope:
- id: FR-052-AC-17
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: Given four synthetic `Complete` outcomes and a valid binding for the AC-2 fixture shape, `build_source_obligation_census` returns a census whose inventory rows, dispositions, missing-mapping reasons, accepted rows and their provenance, and orderings follow the census construction rules; an occurrence asserted only without a production function is `ZeroMappings(ExplicitNoProductionFunction)`, and every QSL row is `ZeroMappings(NoAuthoredMapping)`. Permuting assertion order and outcome row order yields an equal census. This grants no owner-native completeness or FR-052-AC-2 full-owner credit.
- id: FR-052-AC-18
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: Given synthetic outcomes, a fault in each of two owner positions yields only the error for the earlier position in the order Quire binding, Quire, IR V2, QSL, `quire-code-rs`; within one position an outcome that is both in the wrong position and incomplete yields `InventoryScopeMismatch`, and one that has the wrong source and is incomplete yields `InventorySourceMismatch`. This grants no FR-052-AC-6 or AC-7 owner-native credit.
- id: FR-052-AC-19
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: Given synthetic `Complete` outcomes, an assertion that is both foreign and dangling is rejected only with `ForeignSource`; one repeated assertion ID with differing relation and differing provenance is rejected only with `ConflictingRelation`; two assertion IDs stating one relation with equal provenance yield one accepted row; two IDs stating one relation with differing provenance are both rejected with `ConflictingProvenance`; and an occurrence under a `ConflictingIdentity` node is rejected with `DanglingOccurrence`. This grants no FR-052-AC-3 full-owner credit.
- id: TC-452#synthetic-procedure
  path: spec/model/matrix/TC-452-source-obligation-correspondence.md
  role: examined
  excerpt: Separately, build the same fixture from four synthetic `Complete` outcomes and a valid binding. ... Supply a missing binding, a binding naming another source, and a binding whose obligation scope differs from a Quire criterion row's obligation declaration. ... These checks use synthetic outcomes only.
- id: TC-452#synthetic-expected
  path: spec/model/matrix/TC-452-source-obligation-correspondence.md
  role: examined
  excerpt: The synthetic census matches the census construction rules field by field and is equal under permutation. Only the earliest failing position's error is returned ... and the occurrence under a conflicting node is `DanglingOccurrence`. None of these observations grants full-owner AC-1 through AC-3, AC-5 through AC-7, or AC-9 through AC-11 credit.
- id: tests.md#TC-452
  path: spec/model/matrix/tests.md
  role: examined
  excerpt: '| TC-452 | ... | FR-052-AC-1, ..., FR-052-AC-17, FR-052-AC-18, FR-052-AC-19 | Planned/UNRUN: ... AC-17..19 synthetic-census checks await the census builder and grant no full-owner credit; AC-8 is covered by provisional-port tests and is superseded by AC-17 when the census builder lands, which removes the envelope port and those tests; ...'
```

## Dispositions

Round 1, against fix commit 8119d05.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | - |
| FND-002 | fixed | - |
| FND-003 | fixed | - |
