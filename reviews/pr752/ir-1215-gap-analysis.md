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

PASS for the reviewed PR slice at 76f0d596e82e5353378cd5b22b90a4f27377528e. All findings are fixed and dispositioned, and final-source workspace Clippy, focused nominal runtime tests (8/8), and formatting checks passed. The PR is mergeable under the assigned focused-check speed rule; this is not repository-wide assurance.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | malformed_nominal_bodies_never_acquire_a_runtime_view carries an enum-index AC-6 trace tag but only mutates a unit target and exercises package admission. Remove the unrelated AC-6 tag from this unit-only control or add an independent malformed enum/member control; the acceptance criterion itself need not change. | tests/it/nominal_runtime.rs:315-330 |

## Coverage

Plan completion: not assessed.

One Rust lane is folded into code-review; no separate Rust artifact. No spec diff, so spec-review methods are inapplicable. No applicable AssuranceProfile was found. No dependency, workflow, unsafe-code, vendoring or compatibility change is present. Thin borrowed views and QSL-stage row carriers do not implement a second UnitGraph, EnumMemberIndex, checker or interpreter.

Reviewed public module exposure in identity.rs/lib.rs, the new nominal_runtime.rs, dedicated nominal_runtime tests and tests/it/main.rs registration. Unit target closure requires witnessed rows; dimension closure returns canonical nodes/terms using an iterative bounded traversal. Enum members join admitted literal type/case and return authored declaration order. Opaque Exact payload provenance remains a QSL-stage fact under FR-352, rather than authority created by a caller-built test row.

Reconciliation: quoin matrix 0.29.1 plus quire matrix, explicit repository scope. The computed repository matrix has 872 criteria: 712 tagged, 138 untagged, 1 tagged only by ignored tests and 21 method-without-symbol. All 872 have no run evidence in the quoin store. These are baseline context, not a claim of repository-wide assurance. In this PR slice AC-5, AC-6 and AC-7 are tagged by ordinary new tests; AC-8 is Inspection/method-without-symbol. AC-1 through AC-4 belong to other IR-1212..1214 lanes and remain outside this diff. Quire coverage reports no stale trace IDs in the new test file; unrelated repository diagnostics are retained in the captured output and are not treated as PR regressions.

No Cargo, compiler, Clippy, focused test or Kani process was run by this reviewer, per the shared thermal lock instruction. The initial source disposition was conditional on parent-owned validation. The final-source validation receipts inspected below satisfy that condition.

- FR-354-AC-5, role: examined. Excerpt: With QSL-stage unit rows for source-owned declarations, a selected root and converted unit yield exact dimension, target, scale, offset and witnessed identity sufficient for `UnitTable`; absent rows, a dangling target, foreign identity or unresolved conversion return a typed outcome without a partial resolver.
- FR-354-AC-6, role: examined. Excerpt: With QSL-stage enum declaration and member rows, an ordered and an unordered selected enum each yield exact case order, member identity and position sufficient for `EnumMemberIndex`; absent, duplicate, mismatched-case or foreign member witnesses refuse before any partial index.
- FR-354-AC-7, role: examined. Excerpt: Repeated bounded reads of an admitted package and its clone agree; an insufficient limit reports the charged work and locus without a partial view, source recheck, wire reparse or stack growth with term depth.
- FR-354-AC-8, role: examined. Excerpt: Public API and dependency inspection finds the runtime projections in IR model, no QSL `Scope`, `CheckedGraph`, evaluator, supplied-state admission or mirror expression type in IR, and no claimed success for a producer decision absent from the package.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | fab521d0 removes AC-6 from the unit-only control and renames it dangling_unit_targets_cannot_enter_an_admitted_runtime_view. Other enum controls keep their AC-6 tags; no criterion is changed. |

## Final validation disposition

Final reviewed source: 76f0d596e82e5353378cd5b22b90a4f27377528e. The only source change since the prior c8383879 disposition replaces a one-element cloned-row slice in a negative test with `std::slice::from_ref(&converted_row)`. Both forms pass the same one-element row contents to the same missing-target-witness assertion. Production code, selected identity semantics, trace tags and acceptance criteria are unchanged; no new finding.

The parent reports all following commands completed with exit 0 on that source head, and the reviewer inspected their retained outputs:

- Workspace all-targets Clippy: `/home/peter/dev/qualification-logs/titan-ir1215/clippy-final.log` contains the successful finished dev-profile result.
- Workspace focused nominal runtime tests: `/home/peter/dev/qualification-logs/titan-ir1215/focused-tests.log` records eight tests passed, zero failed, zero ignored, including malformed-row budget/locus, unit target closure, derived dimension, enum order and foreign-member controls.
- Cargo formatting check: `/home/peter/dev/qualification-logs/titan-ir1215/fmt-check.log` is empty on success; exit 0 is retained by the parent execution result. No new Cargo invocation was made by this reviewer.

All prior findings remain fixed. Earlier pending-validation statements in this artifact describe their historical source-disposition checkpoints and are superseded by this final disposition. The same review is now mergeable under the task's focused-check speed rule. No Kani proof was required or run, and no aggregate/repository-wide test pass is claimed.
