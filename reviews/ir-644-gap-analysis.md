---
id: SR-1741
title: "Gap analysis of IR-644 wrapped optional leaf evidence"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@66c362435cd627f5d72718f8f34b065ee97713dd; crates/quire-contract-model/src/checked_package/v2/operations.rs, tests/it/checked_package_v2_recursive_leaves.rs, spec/checked_package/functional/FR-038-consume-checked-package-v2.md"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-contract-ir/FR-038", type: references }
---

## Summary

Planless traceability check of the PR #301 code and tests against FR-038-AC-151/152. The tests run and carry the intended tags, but the computed matrix mints neither criterion from the merged spec.

## Verdict

**FAIL** — AC-151 and AC-152 are absent from the computed Test Matrix, so their tagged tests cannot count as criterion evidence; the spec still calls them unimplemented.

## Scope

- **FR-038-AC-151 (examined):** PLANNED for IR-644 code; no backing test yet. Over the QSL-shaped recursive `List` record with an integer field and `next` encoded as `binding(next, aggregate([binding(optional, reference Option<List>)]))`, `structural.eq` over two `List` values admits with `leaves` empty, and the leaf derivation terminates through the option's `inner` edge at the record reentry. Replacing that integer field with a text field whose profile is selected admits exactly its text leaf followed by `["field:next", "inner", "recursion:0"]`; omission of that recursion leaf refuses `invalid_package`/`operation-law-missing` at `operation.leaves`. Adding a healthy text sibling after `next` retains its own leaf after the recursion leaf in declaration order, proving that the wrapper consumes exactly one field edge. A direct field `reference` to `Option<List>` also admits with the same `field:next`, `inner` path.
- **FR-038-AC-152 (examined):** PLANNED for IR-644 code; no backing test yet. With the rest of the `List` comparison well formed and its identity members recomputed after each mutation, replace only the value of `next` by an `aggregate` with no member, two `optional` members, a member named otherwise, a non-binding member, or an `optional` binding whose value is not a reference to an option type. Each mutation is admitted by the flat body grammar; after identity re-derivation, each reaches the operation check and refuses `ill_typed`/`operator-ineligible` at `operation.leaves`, rather than silently omitting `next` or deriving a leaf from a different member.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | A blank line splits AC-151/152 from the FR-038 criteria table, so Quire mints neither criterion and cannot bind the new test tags | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3126; tests/it/checked_package_v2_recursive_leaves.rs:307,356 |
| FND-002 | medium | FR-038 and TC-048 still claim AC-151/152 are planned and have no executable test, contradicting the added passing tests | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3127; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:575 |

## Coverage

- Reconciliation: `quoin matrix --repo /home/peter/dev/worktrees/review-ir644 --json` (quoin 0.28.1) found 299 criteria, 254 tagged, 43 untagged and 2 method-without-symbol; FR-038's last minted criterion is AC-150. `quire matrix --scope /home/peter/dev/worktrees/review-ir644 --format json` independently has neither AC-151 nor AC-152. The 43 unrelated untagged criteria predate this PR and are not attributed to its diff.
- The two new tests carry `#[trace("TC-048", "FR-038-AC-151")]` and `#[trace("TC-048", "FR-038-AC-152")]` and pass in the focused integration run. Their rows follow a blank line after AC-150 in the merged spec at 4d5754b6af95b07ee1977ab1bd58216136aa86d0.
- Source behavior inventory for this diff: one record-field wrapper branch, one direct-reference branch, one malformed-shape refusal path; all map to the two intended ACs. No source or test stubs found in the diff.
- Plan completion: not assessed
- Semantic review: skipped; the targeted code review in SR-1740 compared the two criteria with source and test behavior.

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed ce210cf63520bf520fac1bf3522bceae063cd10f | The adjacent AC-151/152 rows are minted by Quire and each is tagged by the intended TC-048 test. |
| FND-002 | fixed ce210cf63520bf520fac1bf3522bceae063cd10f | FR-038, TC-048 and the test matrix now state that both ACs are implemented and tested. |

### Round 1 verification

At `ce210cf63520bf520fac1bf3522bceae063cd10f`, `quire matrix` reports FR-038-AC-151 and FR-038-AC-152 as `tagged`, with binders `tc_048_wrapped_optional_record_fields_keep_their_leaf_order` and `tc_048_malformed_optional_record_fields_refuse_at_operation_leaves`. `quire coverage` reports no focused untracked symbols or diagnostics. The focused recursive-leaf integration filter passed 7 tests. The fix changes specification and review files only; the clean code review in SR-1740 remains valid. No new finding arose from the delta.
