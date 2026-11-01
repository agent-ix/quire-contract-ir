---
id: SR-202610310114
title: "IR-1215 code and Rust review"
type: SpecReview
analysis: code-review
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
| FND-001 | medium | The declaration helper traverses every caller-supplied qualified-name segment in try_fold before charging work, and empty segments add no cost. A malformed declaration containing arbitrarily many empty segments can consume unbounded pre-refusal work under a tiny projection limit. Charge each segment, including empty ones, before visiting it and add a discriminating malformed-row budget test. | crates/quire-contract-model/src/checked_package/v2/nominal_runtime.rs:205-211 |
| FND-002 | medium | Budget::error clones the caller-supplied node ID after a failed partial identity charge. A zero-limit request with an arbitrarily wide package/path still copies that complete root locus outside the budget. Borrow the caller or admitted canonical locus, or return an already-bounded locator, instead of allocating the rejected identity on exhaustion. | crates/quire-contract-model/src/checked_package/v2/nominal_runtime.rs:88-94 |

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
| FND-001 | fixed | 7d0af169 replaces the pre-charge fold with per-component/per-segment charges, including empty segments. The new 1000-empty-segment malformed-row control expects LimitReached at attempted work 501 with limit 500; old code would instead finish its fold and return ForeignWitness. |
| FND-002 | fixed | 7d0af169 changes NominalReadError to borrow its node locus and removes the clone from Budget::error. The new exhaustion controls check pointer equality, including a zero-limit refusal. |

## Compiler-fix disposition

Source follow-up at c8383879223d4b14317c0ec72c73bcf44a6993ee inspected the same PR's two-line `mutable_key_type` fix. The accepted `UnitId::Declared(key)` branch now indexes uniqueness by its borrowed canonical `NodeKey`, instead of retaining the wider `UnitId` enum as the ordered key. Equality and ordering of admitted declared-unit identities are exactly equality and ordering of that contained key. The returned view still borrows the original declared identity, duplicate keys still refuse, and the compound-unit arm remains `UnsupportedForm`. No new semantic type, fallback, witness rule or trace binding is introduced. No new finding.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Earlier bounded per-segment charging remains unchanged by c8383879. |
| FND-002 | fixed | Earlier borrowed refusal loci remain unchanged by c8383879. |

The parent reports the first workspace Clippy invocation reached the code and raised `mutable_key_type`; this reviewer did not execute Cargo or independently claim a clean Clippy result. The rerun and focused tests remain pending, so the verdict remains CONDITIONAL on actual validation passing.
