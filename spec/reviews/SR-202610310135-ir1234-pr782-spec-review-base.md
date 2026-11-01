---
id: SR-202610310135
title: "IR-1234 PR 782 quantified timed binder base spec review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR #782; spec/checked_package/functional/FR-038-consume-checked-package-v2.md"
review_set: subset
---

## Summary

Reviewed FR-038's changed timed layout and binder join with AC-385 and AC-386 against QSpec's published CL-57 through CL-62 cases and existing unquantified controls. No applicable AssuranceProfile was found under `spec/`.

## Verdict

**PASS** — the added rule covers the positive guard and invariant carriers, the non-slot, repeated and reversed-order binder refusals, and the unchanged unquantified form without an internal contradiction.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-038 timed-clause layout and binder join, role: examined. Optional binder positions, typed slot membership/order, first bad target refusal, and old layout are stated.
- FR-038-AC-385, role: examined. CL-57 and CL-59 admission, ordered binder retention, and unquantified positive controls are testable.
- FR-038-AC-386, role: examined. CL-58, CL-60 and CL-62 each name exact typed code, cause and vector target path; the containing timed node locus is stated in the behavior paragraph.
- QSpec FR-322 binder rule, role: context_only. The authoritative rule and vectors supply the wire forms; this PR specifies IR reader behavior rather than copying the source fixture.
- QSpec CL-55, CL-56 and CL-61, role: context_only. The current-main reader measurement passes CL-55/56 before failing CL-57; CL-55/61 are schema-only controls outside this binder join.

## Evidence

The changed FR-038 document passes scoped Quire validation. AC-385 and AC-386 are extractable assertions; their matrix rows are untagged pending the code PR. QSpec's vector metadata currently points CL-57 through CL-60 to an unrelated rule ID; the IR text cites the authoritative vector path and case IDs without importing that metadata. QSpec's later renumbering has a distinct binder requirement. The repository diff adds no hash, digest or pin record.
