---
id: SR-10808
title: "IR-754/755 PR 382 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR 382 FR-028-AC-1 through FR-028-AC-5, tests/it/cycle_free_model.rs, Makefile"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-028
    type: reviews
---

## Summary

Ticket: IR-754 and IR-755. The new tests carry TC-041 and per-criterion trace tags. A green binding overstates coverage for AC-1 and AC-5 because the tagged controls cannot fail for specific forbidden changes.

## Verdict

FAIL. AC-1 and AC-5 have hollow coverage despite their trace tags. This is a diff-scoped manual intent-to-test check; the computed matrix command did not return usable JSON in this reviewer invocation. No repository-wide gap verdict or full gate is claimed.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | AC-1's tagged test omits nonhost production target edges and cannot recognize a path-sourced owner crate whose package name misses its fixed name list. Either dependency can violate owner-free reachability while the test passes. | tests/it/cycle_free_model.rs:20-30, tests/it/cycle_free_model.rs:63-90 | correct-requirement-no-evidence |
| FND-002 | high | AC-5's tagged source scan is blind to public callback aliases; a type alias to `dyn Fn` used by a public admission method preserves a forbidden validator surface and passes its control. | tests/it/cycle_free_model.rs:489-539 | correct-requirement-no-evidence |

## Coverage

- Examined FR-028-AC-1: production graph trace binding exists, but FND-001 limits its intent coverage.
- Examined FR-028-AC-2: direct model/root corpus byte comparison and changed-schema refusal bind the named result categories; no additional gap found.
- Examined FR-028-AC-3 as context: root direct dependency control retained.
- Examined FR-028-AC-4: TC-041 tag is on the ignored build test and explicit Make target; author-reported run is unverified by this review.
- Examined FR-028-AC-5: source-boundary trace binding exists, but FND-002 limits its intent coverage.
- Reverse-gap scan of changed production code: none; the diff adds test code and a Make target only.
- Plan completion: not assessed.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | still-open | Its nonhost target-edge case remains valid. The path-sourced `quire-exact` example in the original row is rejected: `quire-exact` is a separate repository, not a QSL-owned package. The next review copy should carry only the target-edge case. |
| FND-001 | fixed | removes host filtering and adds a target-gated owner edge adverse control. The original path-owner clause remains rejected, as recorded above. |
| FND-002 | fixed | scans public callback aliases and adds two alias adverse controls, restoring AC-5's intended trace binding for the reviewed case. |

## Disposition verdict

Round 1: the two valid diff-scoped evidence gaps are fixed. The erroneous path-owner clause stays withdrawn. No repository-wide matrix or gate verdict is inferred.
