---
id: SR-202610310115
title: "IR-1215 traceability gap review"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; IR-1215; PR #752; borrowed nominal runtime reads and tests"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-354
    type: reviews
---

## Summary

Reviewed PR #752 initially at 998f2e7fc9efd7cf2ad0b268e22fde3315984dde, against merged FR-354 base 6d3229ea8e56513c6b6e1833cf974f094a557bfe. Disposition inspected the same PR through 8512c21c9de9b7987423ede2abcce60d6ffcca16. Ticket: IR-1215.

## Verdict

CONDITIONAL. All findings below are fixed by source inspection at the disposition head; compilation and execution remain pending the host-wide Cargo lock. No test-pass or unconditional mergeability claim is made.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | malformed_nominal_bodies_never_acquire_a_runtime_view carries an enum-index AC-6 trace tag but only mutates a unit target and exercises package admission. Remove the unrelated AC-6 tag from this unit-only control or add an independent malformed enum/member control; the acceptance criterion itself need not change. | tests/it/nominal_runtime.rs:315-330 |

## Coverage

Plan completion: not assessed.

One Rust lane is folded into code-review; no separate Rust artifact. No spec diff, so spec-review methods are inapplicable. No applicable AssuranceProfile was found. No dependency, workflow, unsafe-code, vendoring or compatibility change is present. Thin borrowed views and QSL-stage row carriers do not implement a second UnitGraph, EnumMemberIndex, checker or interpreter.

Reviewed public module exposure in identity.rs/lib.rs, the new nominal_runtime.rs, dedicated nominal_runtime tests and tests/it/main.rs registration. Unit target closure requires witnessed rows; dimension closure returns canonical nodes/terms using an iterative bounded traversal. Enum members join admitted literal type/case and return authored declaration order. Opaque Exact payload provenance remains a QSL-stage fact under FR-352, rather than authority created by a caller-built test row.

Reconciliation: quoin matrix 0.29.1 plus quire matrix, explicit repository scope. The computed repository matrix has 872 criteria: 712 tagged, 138 untagged, 1 tagged only by ignored tests and 21 method-without-symbol. All 872 have no run evidence in the quoin store. These are baseline context, not a claim of repository-wide assurance. In this PR slice AC-5, AC-6 and AC-7 are tagged by ordinary new tests; AC-8 is Inspection/method-without-symbol. AC-1 through AC-4 belong to other IR-1212..1214 lanes and remain outside this diff. Quire coverage reports no stale trace IDs in the new test file; unrelated repository diagnostics are retained in the captured output and are not treated as PR regressions.

No Cargo, compiler, Clippy, focused test or Kani process was run by this reviewer, per the shared thermal lock instruction. Source review and static matrix data cannot establish runtime pass. Parent owns final validation; the source disposition below is conditional on those checks passing.

- FR-354-AC-5, role: examined. Excerpt: With QSL-stage unit rows for source-owned declarations, a selected root and converted unit yield exact dimension, target, scale, offset and witnessed identity sufficient for `UnitTable`; absent rows, a dangling target, foreign identity or unresolved conversion return a typed outcome without a partial resolver.
- FR-354-AC-6, role: examined. Excerpt: With QSL-stage enum declaration and member rows, an ordered and an unordered selected enum each yield exact case order, member identity and position sufficient for `EnumMemberIndex`; absent, duplicate, mismatched-case or foreign member witnesses refuse before any partial index.
- FR-354-AC-7, role: examined. Excerpt: Repeated bounded reads of an admitted package and its clone agree; an insufficient limit reports the charged work and locus without a partial view, source recheck, wire reparse or stack growth with term depth.
- FR-354-AC-8, role: examined. Excerpt: Public API and dependency inspection finds the runtime projections in IR model, no QSL `Scope`, `CheckedGraph`, evaluator, supplied-state admission or mirror expression type in IR, and no claimed success for a producer decision absent from the package.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | fab521d0 removes AC-6 from the unit-only control and renames it dangling_unit_targets_cannot_enter_an_admitted_runtime_view. Other enum controls keep their AC-6 tags; no criterion is changed. |
