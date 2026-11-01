---
id: SR-2026103033
title: "IR-1212 code-review review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-354-project-admitted-runtime-context.md; PR #750"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-354
    type: reviews
---

## Summary

Ticket: IR-1212. Reviewed frozen PR #750 head 37202b99cb8c33a328e68a3a105868236475ab34 against current origin/main 154227dcbcda09f3abf635375777893132fc7686. One new FR and eight criteria. No applicable AssuranceProfile is present.

## Verdict

**PASS.** No findings in this method.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

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
