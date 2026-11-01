---
id: SR-2026103036
title: "IR-1212 spec-review/integrity review"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-354-project-admitted-runtime-context.md; PR #750"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-354
    type: reviews
---

## Summary

Ticket: IR-1212. Reviewed frozen PR #750 head 37202b99cb8c33a328e68a3a105868236475ab34 against current origin/main 154227dcbcda09f3abf635375777893132fc7686. One new FR and eight criteria. No applicable AssuranceProfile is present.

## Verdict

**FAIL.** Producer identity inputs are incomplete for the positive runtime projections.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-354 accepts only FR-352 selected identity rows, which carry record/object/union/selected-model-unit declarations and record fields. They carry no tuple identity, source-unit identity, or enum declaration/member keys. AC-1, AC-5 and AC-6 require successful witnessed projections of those forms, so the positive oracles lack a producer input. Extend the QSL-stage witness contract for these identities or explicitly gate the affected positive criteria on that producer prerequisite. | spec/checked_package/functional/FR-354-project-admitted-runtime-context.md:33-37 |

## Coverage

- FR-354, role: examined. Excerpt: - An owner-admitted `CheckedPackageV2`, requested admitted node or model   identity, and finite projection work limit. - Where an Exact identity is required, the selected-document identity rows of   [FR-352](./FR-352-project-admitted-exact-value-types.md), produced by the   same QSL selection stage as the admitted package. A bare name or Exact key   does not establish selection.
- FR-354-AC-1, role: examined. Excerpt: A public IR-model read of an admitted package and matching selected identity rows supplies complete record, tuple and object declarations, effective fields, ancestry and units from which QSL constructs the expected `TypeEnvironment`; no source or JSON is reparsed.
- FR-354-AC-2, role: examined. Excerpt: A missing, foreign, duplicated or conflicting selected declaration, field, parent or unit identity returns a named typed outcome before any partial type-environment view; matching display names do not make a join.
- FR-354-AC-3, role: examined. Excerpt: The callable projection returns each selected function or operation's ordered parameters, admitted body handle, context and frame, including an intentionally empty frame; a dangling body, wrong owner or absent required frame returns a typed outcome.
- FR-354-AC-4, role: examined. Excerpt: The dispatch projection preserves every admitted candidate in producer order with its effective precondition term reference; a missing or ambiguous candidate/precondition join refuses without choosing a candidate or evaluating a predicate.
- FR-354-AC-5, role: examined. Excerpt: A selected root and converted unit yield exact dimension, target, scale, offset and witnessed identity sufficient for `UnitTable`; a dangling target, foreign identity or unresolved conversion returns a typed outcome without a partial resolver.
- FR-354-AC-6, role: examined. Excerpt: An ordered and an unordered selected enum each yield exact case order, member identity and position sufficient for `EnumMemberIndex`; duplicate, missing or foreign member witnesses refuse before any partial index.
- FR-354-AC-7, role: examined. Excerpt: Repeated bounded reads of an admitted package and its clone agree; an insufficient limit reports the charged work and locus without a partial view, source recheck, wire reparse or stack growth with term depth.
- FR-354-AC-8, role: examined. Excerpt: Public API and dependency inspection finds the runtime projections in IR model, no QSL `Scope`, `CheckedGraph`, evaluator, supplied-state admission or mirror expression type in IR, and no claimed success for a producer decision absent from the package.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | PR #750 head 1f6d10c2 adds QSL-stage acquisition rows for selected tuple declarations, source-owned units, enum declarations and members, binds each row to selected document/original identity/closed form/admitted structural node, and requires typed refusal until producer rows exist. Revised AC-1, AC-5 and AC-6 condition positive projections on those rows and test missing-row refusal. |
