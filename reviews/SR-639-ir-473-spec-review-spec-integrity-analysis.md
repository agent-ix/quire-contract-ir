---
id: SR-639
title: "spec review of PR 234 (FR-014 range-set paragraph, FR-014-AC-7, matrix, STD-001)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@d8b1f16c424733c9e4d807e11ecc02ff52f6c6e0; spec/contract/FR-014-expression-semantics.md, spec/contract-test-matrix.md, spec/contract/STD-001-diagnostic-registry.md"
review_set: base
---
# SR-639: spec review of PR 234

## Summary

Ticket: IR-473. Integrity review of the new FR-014 Behavior paragraph (lines 102-107), the new FR-014-AC-7 (line 130), the FR-014 Test Matrix row (now AC-1 through AC-7, TC-016) and the STD-001 `potentially_undefined` row (line 75), each checked against the code at head. The matrix row and the STD-001 note are accurate. FR-014-AC-7 is concrete (120 leaves, more than 64 intervals, a named code and obligation kind) and is backed by TC-016 tests. `make spec` shows 17 unbacked rows, the same as origin/main.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-014 says "checking never allocates in proportion to a product of interval counts", but `integer_ranges` and `rational_range_results` allocate `Vec::with_capacity(left.len() * right.len())` and compute every pair, up to 64 x 64, before merging: the claim contradicts the code | spec/contract/FR-014-expression-semantics.md:106-107 |
| FND-002 | low | The same paragraph says "a non-zero or bound guard can split" a range, but bound guards only trim and only `!= 0` splits. It also says "every operator result is merged", but negation results are not re-canonicalized, which is harmless because negation never adds intervals | spec/contract/FR-014-expression-semantics.md:102-104 |

## Finding Detail

- FND-001: suggested text: "each operator combines at most 64 x 64 interval pairs, so checking memory is bounded independent of expression shape." If SR-637 FND-001 (widen instead of refuse) is adopted, rewrite the refusal sentence and FR-014-AC-7 to match.
- FND-002: suggested text: "which a non-zero guard can split" and "every binary operator result is merged".

## Verdict

Approve with one medium wording fix. On reusing `potentially_undefined` / `checked_range` rather than adding a new code: STD-001 defines the code as "a partial-operation obligation is not statically discharged". For a Reject-policy operator, declining to compute a too-large set does leave the `checked_range` obligation undischarged, so reuse is defensible there. For a Saturate-policy operator no such obligation exists (SR-637 FND-001). Widening, as SR-637 recommends, would avoid both the misreport and the STD-001, schema, corpus and trace churn of a new `range_set_too_large` code. No relationships edge changes, so dependency and object sub-analyses do not apply; FR-014's statement is unchanged, so EARS analysis does not apply.

## Dispositions

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 4b79f5f |
| FND-002 | fixed | 4b79f5f |
