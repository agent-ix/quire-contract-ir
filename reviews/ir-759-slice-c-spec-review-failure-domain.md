---
id: SR-11812
title: "IR-759 Slice C failure-domain review of FR-052 census construction rules"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-contract-ir@794d6e2 (PR 417, diff vs origin/main 6b80801); spec/model/functional/FR-052-source-obligation-correspondence.md, spec/model/matrix/TC-452-source-obligation-correspondence.md, spec/model/matrix/tests.md"
review_set: subset
date: '2026-10-16'
---

## Summary

Ticket: IR-759. Failure-domain analysis of the census construction rules at 794d6e2: identity confusion, unstated edge cases in endpoint resolution and disposition derivation, and what ZeroMappings retains.

## Verdict

Two low findings. Covered well: foreign-and-dangling, conflicting relation and provenance, occurrence under a conflicting node, explicit absence of a production function, empty module, ZeroMappings retaining the full SourceInventoryRow via row(), missing-versus-incomplete outcomes, wrong-position outcomes, and a conflicting binding reported with the first conflicting criterion in ascending order.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | A row whose only authored assertions were rejected (dangling, conflicting) and an occurrence under a ConflictingIdentity node both fall to 'every other row' and read ZeroMappings(NoAuthoredMapping), although an authored mapping exists. The rule is decisive, but the reason name says the opposite of the fact; a consumer reading dispositions alone cannot tell 'never authored' from 'authored and rejected'. State explicitly that rejected assertions do not affect dispositions and that the distinction lives only in rejected_assertions(). | spec/model/functional/FR-052-source-obligation-correspondence.md:336-340, 402-413 |
| FND-002 | low | An occurrence whose node has no SemanticNode row, or a function whose module has no ProductionModule row, resolves as a valid endpoint under these rules (only a ConflictingIdentity node is excluded). The design reference branch treated both as dangling. Either is a legitimate choice, but the spec should say which on purpose, since a complete IR V2 or quire-code-rs outcome should not contain orphans. | spec/model/functional/FR-052-source-obligation-correspondence.md:375-380 |

## Examined units

```yaml
scope:
- id: FR-052#endpoint-resolution
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: An assertion endpoint resolves only by exact typed identity. The criterion endpoint resolves against `Criterion` rows. The occurrence endpoint resolves against `SemanticOccurrence` rows whose node has no `ConflictingIdentity` row. A present production-function endpoint resolves against `ProductionFunction` rows. An absent production function is an explicit authored absence and needs no function row.
- id: FR-052#dispositions
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: A `SemanticNode` row in a conflicting key group is `ConflictingIdentity`. Otherwise a row is `Mapped` when an accepted row names it ... A row that is not `Mapped` is `ZeroMappings(ExplicitNoProductionFunction)` when it is an occurrence, or the node of an occurrence, named by an accepted row with no production function. Every other row, including an empty module and every QSL `RequirementRecord` and `PendingObligation` row, is `ZeroMappings(NoAuthoredMapping)`; no assertion endpoint can name a QSL row. That QSL rows are always `ZeroMappings(NoAuthoredMapping)` is an IR reading pending owner confirmation (see Open questions).
- id: FR-052#assertion-pipeline
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: 1. Exact duplicate assertions, equal in every field, collapse to one. 2. Assertions sharing one `SourceAssertionId` form a group. ... `ConflictingRelation` ... Otherwise ... `ConflictingProvenance`. 3. ... `ForeignSource` when its source or its provenance source differs from the selected source, then `DanglingCriterion`, `DanglingOccurrence` and `DanglingProductionSymbol`. 4. Remaining assertions with one criterion, occurrence and production function form one relation. When a relation holds more than one distinct provenance, every member is rejected with `ConflictingProvenance`. Otherwise the relation yields exactly one `SourceObligationRow`, whatever the number of assertion IDs that state it.
- id: FR-052#census-accessors
  path: spec/model/functional/FR-052-source-obligation-correspondence.md
  role: examined
  excerpt: '`SourceObligationCensus` shall expose `source()`, `criterion_source_binding()`, `inventory_rows()` returning `&[SourceCensusInventoryRow]`, `correspondence_rows()` returning `&[SourceObligationRow]`, and `rejected_assertions()` returning `&[SourceCorrespondenceRejection]`. `SourceCensusInventoryRow` shall expose `row()`, the retained `SourceInventoryRow`, and `disposition()`.'
```
