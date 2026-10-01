---
id: SR-661
title: "gap analysis of PR 240 (IR-486 exact operation leaves)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@5dddf2eaea8b4115ca611c915318bcd48d394d4e; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/tests.md, crates/quire-contract-model/src/checked_package/v2/operations.rs"
review_set: base
---
# SR-661: gap analysis of PR 240

## Summary

Ticket: IR-486. Plan completion: not assessed (planless). This audit checks that FR-038-AC-44 is backed by the tests tagged to it, and that the tagged tests check what the AC states.

All ten new tests carry `Tracing: TC-048, FR-038-AC-44`: `tc_048_leaf_source_admits_exactly_the_derived_leaves`, `..._refuses_extra_and_unrelated_leaves`, `..._refuses_a_wrong_path_and_a_wrong_order`, `..._refuses_a_wrong_or_missing_leaf_law`, `..._refuses_an_unselected_leaf_law`, `tc_048_entry_without_a_leaf_source_refuses_a_supplied_leaf`, `tc_048_leaf_paths_are_derived_lazily_over_a_shared_field_chain` and `tc_048_leaf_path_under_nested_options_is_exact`. Every clause of AC-44 is exercised, each with its exact code, cause and pointer. `make spec`: FR-038 is 40/42 backed. The 17 unbacked rows are all outside FR-038 (FR-036, FR-037, FR-039, FR-019 and others), the same as on main, and none comes from this diff. The tests.md diff is exactly the two intended row edits: FR-038's AC range extended to AC-44, and TC-048 listing AC-44. No row or requirement was removed and nothing else changed, despite the `sed -i` edit. AC-43 is untouched, and its 12-level shared-field test still passes.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The admit cases of FR-038-AC-44 and `tc_048_leaf_source_admits_exactly_the_derived_leaves` use bare `text` scalars that pin no `text_profile`, with leaves that carry no `mode`. The reference reader refuses these packages: `collect_leaves` returns None when a text leaf pins no profile, which is `operator-ineligible`, and a leaf without a `text_profile` mode is `operation-mode-mismatch`. So the AC and its test assert, as conformance, an admission QSpec FR-322 forbids ("with the text profile law and the mode that leaf's type pins") | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:724; crates/quire-contract-model/src/checked_package/v2/operations.rs:3549-3598 |

## Verdict

Mergeable on the gap-analysis axis once FND-001 is fixed. The trace is complete and every binding is correct. The fix: build the admit fixtures over a `text_bounds` that pins `text_profile`, give each leaf `mode: {kind: text_profile, value: <pin>}`, and add "and the mode its type pins" to AC-44. Then the admitted packages are ones the reference admits too, and the reader's own remaining mode gaps stay as stated limits (SR-662) rather than being encoded into an AC.

## Dispositions

Reviewed at agent-ix/quire-contract-ir@b2ba6c53bfb89f7a7bff9be7890ad92662452764.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 489d6a8 |
