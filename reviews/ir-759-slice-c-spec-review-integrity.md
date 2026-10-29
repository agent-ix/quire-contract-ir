---
id: SR-11811
title: "IR-759 Slice C integrity review of FR-052 census construction rules"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir (PR 417, diff vs origin/main ); spec/model/functional/FR-052-source-obligation-correspondence.md, spec/model/matrix/TC-452-source-obligation-correspondence.md, spec/model/matrix/tests.md"
review_set: subset
date: '2026-10-16'
---

## Summary

Ticket: IR-759. Integrity analysis (internal consistency, single source of truth per rule, completeness of interfaces) of the PR 417 diff in FR-052 and TC-452.

## Verdict

Three low findings; no contradiction between two rules in the census pipeline itself. Exact-duplicate collapse precedes the per-ID group check, so collapsed duplicates never self-conflict; step 2 rejections never reach step 3, so each rejected assertion has exactly one reason as stated. Dispositions are a total function over row kinds (ConflictingIdentity, then Mapped, then ExplicitNoProductionFunction, then NoAuthoredMapping). Inventory exact-duplicate collapse and the SemanticNode conflict group commute. The precedence contradiction with AC-18 is recorded in SR-11810 FND-001 and not repeated here.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | AC-8 cites 'tests/it/source_obligation.rs (lines 368-369)'. The lines are correct but a line reference in an AC drifts with any edit to that file. Name the test function or the two asserted strings instead. | spec/model/functional/FR-052-source-obligation-correspondence.md:459 |
| FND-002 | low | The Behavior paragraph scopes ConflictingProvenance to 'repeated assertion identity', while pipeline step 4 also applies it to different assertion IDs stating one relation with differing provenance (and AC-19 tests that). Not contradictory, but the reason now has two definitions in two places; extend the Behavior sentence to name the cross-ID case so one rule is authoritative. | spec/model/functional/FR-052-source-obligation-correspondence.md:324-327, 394-398 |
| FND-003 | low | Every census accessor's return type is stated except `source()` and `criterion_source_binding()`. Because a census exists only with a binding, the latter could be `&SourceCriterionSourceBinding` or `Option<&...>` (the input accessor is Option). State it, since AC-5/AC-17 consumers compile against it. | spec/model/functional/FR-052-source-obligation-correspondence.md:361-364 |

## Examined units

```yaml
scope:
- id: FR-052#assertion-pipeline
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: 1. Exact duplicate assertions, equal in every field, collapse to one. 2. Assertions sharing one `SourceAssertionId` form a group. ... `ConflictingRelation` ... Otherwise ... `ConflictingProvenance`. 3. ... `ForeignSource` when its source or its provenance source differs from the selected source, then `DanglingCriterion`, `DanglingOccurrence` and `DanglingProductionSymbol`. 4. Remaining assertions with one criterion, occurrence and production function form one relation. When a relation holds more than one distinct provenance, every member is rejected with `ConflictingProvenance`. Otherwise the relation yields exactly one `SourceObligationRow`, whatever the number of assertion IDs that state it.
- id: FR-052#dispositions
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: A `SemanticNode` row in a conflicting key group is `ConflictingIdentity`. Otherwise a row is `Mapped` when an accepted row names it ... A row that is not `Mapped` is `ZeroMappings(ExplicitNoProductionFunction)` when it is an occurrence, or the node of an occurrence, named by an accepted row with no production function. Every other row, including an empty module and every QSL `RequirementRecord` and `PendingObligation` row, is `ZeroMappings(NoAuthoredMapping)`; no assertion endpoint can name a QSL row. That QSL rows are always `ZeroMappings(NoAuthoredMapping)` is an IR reading pending owner confirmation (see Open questions).
- id: FR-052#ordering
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: Inventory rows shall be ordered by owner in the order Quire, IR V2, QSL, `quire-code-rs`, then by `SourceInventoryIdentity` ascending. Accepted rows shall be ordered ascending by criterion, occurrence, production function and provenance. Rejections shall be ordered ascending by assertion ID, source, criterion, occurrence, production function, provenance and reason.
- id: FR-052#census-accessors
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: '`SourceObligationCensus` shall expose `source()`, `criterion_source_binding()`, `inventory_rows()` returning `&[SourceCensusInventoryRow]`, `correspondence_rows()` returning `&[SourceObligationRow]`, and `rejected_assertions()` returning `&[SourceCorrespondenceRejection]`. `SourceCensusInventoryRow` shall expose `row()`, the retained `SourceInventoryRow`, and `disposition()`.'
- id: FR-052#error-variants
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: It shall also expose `MissingCriterionSourceBinding`, `ForeignCriterionSourceBinding` retaining the expected and observed `SourceIdentity`, and `ConflictingCriterionSourceBinding` retaining the binding's obligation scope and the first conflicting criterion in ascending criterion order. It has no item-count variant.
- id: FR-052#behavior-conflicting-provenance
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: Repeated assertion identity with conflicting relation facts or provenance shall remain inspectable with `ConflictingRelation` or `ConflictingProvenance`, respectively; no conflicting assertion shall be accepted.
- id: FR-052-AC-8
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: 'A default-feature consumer can construct and borrow provisional owner rows, ... and the public module exposes no census or complete-census builder. FR-052-AC-17 supersedes this criterion when the census builder lands: that code change removes `SourceInventoryEnvelope`, the envelope-taking `SourceObligationInput` constructor, the tests tagged FR-052-AC-8, and the no-census assertions in `tests/it/source_obligation.rs` (lines 368-369); the FR-052-AC-4 inspection of `SourceObligationInput` fields remains.'
```

## Dispositions

Round 1, against fix.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | - |
| FND-002 | fixed | - |
| FND-003 | fixed | - |
