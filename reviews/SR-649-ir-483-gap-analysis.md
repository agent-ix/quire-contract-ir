---
id: SR-649
title: "gap analysis of PR 238 (IR-483 operation leaves)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@b696d0e8363f01fa63de82f4cd6cd521c8d97943; crates/quire-contract-model/src/checked_package/v2/operations.rs, spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/tests.md"
review_set: base
---
# SR-649: gap analysis of PR 238

## Summary

Ticket: IR-483. Plan completion: not assessed. Each clause of FR-038-AC-43 is backed by a tagged test (`Tracing: TC-048, FR-038-AC-43`):

- `structural.eq`/`ne` over an all-integer record admit `[]`: `tc_048_leaf_source_admits_empty_leaves_over_a_type_without_text`
- `collection.contains` over a set of integers admits: `tc_048_inner_leaf_source_admits_empty_leaves_over_a_type_without_text`
- a nested text field and a set of text refuse: `tc_048_leaf_source_refuses_empty_leaves_over_a_type_with_text`
- one leaf over two text fields refuses: `tc_048_leaf_source_refuses_too_few_leaves`

The tests assert the full code, cause, pointer and locus. The matrix rows for FR-038 and TC-048 list AC-43. `make spec` reports the same 17 unbacked rows, none in this diff. FR-038 is at 39/40 extractable.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Four production branches of the leaf walk have no test, and their mutants survive all 32 `tc_048_` tests: alias/bounded-domain forwarding, the tuple arm, the option/collection arm inside `text_leaf_count`, and the whole `result_inner` source. FR-038 prose claims each branch. The `inner:0` set-of-text test strips the set before the walk, so the collection arm is never reached | crates/quire-contract-model/src/checked_package/v2/operations.rs:1687, 1761, 1771, 1802 |

## Verdict

AC-43 is honestly backed for what it states. Its scope is narrower than the code and the FR-038 prose, and that untested `result_inner` branch is where SR-648 FND-001 lives. Mutation results:

- `text` counted 0: KILLED
- count off by one: KILLED
- alias hop dropped: SURVIVED
- tuple arm dropped: SURVIVED
- wrapper walk dropped: SURVIVED
- `result_inner` dropped: SURVIVED

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 7e10a1e: `tc_048_leaf_walk_reaches_text_through_every_type_form` and `tc_048_result_inner_counts_leaves_only_for_set_like_results` added. Mutants are all killed: bounded-domain-only, option-only, set-only, text-0, result-inner-to-sequence, memo-off, cycle-off and `>=` to `>` |
