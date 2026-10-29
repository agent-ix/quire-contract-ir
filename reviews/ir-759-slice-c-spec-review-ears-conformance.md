---
id: SR-11813
title: "IR-759 Slice C EARS conformance review of FR-052 census construction rules and AC-17..19"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir (PR 417, diff vs origin/main); spec/model/functional/FR-052-source-obligation-correspondence.md, spec/model/matrix/TC-452-source-obligation-correspondence.md, spec/model/matrix/tests.md"
review_set: subset
date: '2026-10-16'
---

## Summary

Ticket: IR-759. EARS conformance of the new Behavior statements and the singular-ness of FR-052-AC-17..19. The FR-052 requirement statement itself is unchanged.

## Verdict

One low finding. The new Behavior statements are ubiquitous EARS ('X shall ...'); the numbered pipeline uses indicative 'is rejected' under a 'shall process ... in these steps' lead-in, which is acceptable. FR-052's acceptance criteria are written as Given/observation tests, not EARS, throughout; the new ACs follow that local convention.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | AC-17, AC-18 and AC-19 are compound: AC-19 bundles five independent rejection cases, AC-18 three precedence cases, AC-17 rows, dispositions, ordering and permutation. A single failing case fails the whole AC and hides which rule broke. Consistent with FR-052's existing AC style, so low; splitting would sharpen the matrix. | spec/model/functional/FR-052-source-obligation-correspondence.md:468-470 |

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
- id: FR-052#census-construction-rules
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: '### Census construction rules: `SourceObligationInput::new` shall take, in order, the selected `SourceIdentity`, an `Option<SourceCriterionSourceBinding>`, one `Option<SourceInventoryOutcome>` for each of the Quire, IR V2, QSL and `quire-code-rs` positions, and a `Vec<SourceCorrespondenceAssertion>`. ... A binding conflicts when a `Criterion` row of the Quire outcome has an `obligation_declaration()` different from the binding''s `obligation_scope()`. ... This definition of a conflicting binding is an IR reading pending owner confirmation (see Open questions).'
- id: FR-052#assertion-pipeline
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: 1. Exact duplicate assertions, equal in every field, collapse to one. 2. Assertions sharing one `SourceAssertionId` form a group. ... `ConflictingRelation` ... Otherwise ... `ConflictingProvenance`. 3. ... `ForeignSource` when its source or its provenance source differs from the selected source, then `DanglingCriterion`, `DanglingOccurrence` and `DanglingProductionSymbol`. 4. Remaining assertions with one criterion, occurrence and production function form one relation. When a relation holds more than one distinct provenance, every member is rejected with `ConflictingProvenance`. Otherwise the relation yields exactly one `SourceObligationRow`, whatever the number of assertion IDs that state it.
```

## Dispositions

Round 1, against fix.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | - |
