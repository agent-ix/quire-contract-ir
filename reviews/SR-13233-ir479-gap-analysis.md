---
id: "SR-13233"
title: "IR PR 479 gap analysis: FR-035-AC-12 to AC-14 coverage delta"
type: SpecReview
analysis: gap-analysis
review_set: subset
scope: "agent-ix/quire-contract-ir@f5229f01c640aa772fe7ba1c109a3e94dfe75b26; spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md, spec/checked_package/matrix/TC-047-complete-v1-contract-package-output.md, spec/checked_package/matrix/tests.md, tests/it/complete_v1_contract_package.rs, crates/quire-contract-model/src/checked_package/v2/contract_package.rs (diff origin/main...HEAD)"
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-035
    type: reviews
---

# SR-13233: IR PR 479 gap analysis

## Summary

Ticket: IR-893. PR: agent-ix/quire-contract-ir#479 at f5229f0. Plan completion: not assessed.

Computed matrix delta, from `quire matrix --format tsv` (quire 0.36.2) at the merge-base and
at the PR head:

- Exactly three rows were added: FR-035-AC-12, AC-13 and AC-14, each `untagged`.
- Nothing else changed: no row changed, 0 contradicted, 0 newly tagged.
- Status totals at the PR head: tagged 500 (unchanged), untagged 65 to 68,
  method-without-symbol 11, tagged-by-ignored-test 1.
- FR-035-AC-2 stays `tagged` by its two TC-044 tests in `tests/it/complete_v1_checked_package.rs`.

Existing tests against the new criteria (all tagged `FR-035-AC-5` today, so none of them backs
the new criteria yet, as the PR states):

- AC-12: `reader_refuses_malformed_outer_shape_and_stale_own_identity` already asserts the exact
  code, the `""` path and `MissingMember` for a missing `source_map`.
- AC-13: `duplicate_member_and_noncanonical_bytes_refuse_before_identity` asserts only the
  `NoncanonicalWire` code. The tests-first PR must add the absent-path and absent-cause
  assertions.
- AC-14: `reader_requires_total_source_correspondence_and_request_order` asserts the exact
  fields. It uses `missing()`, which is requested item 3, without rehashing. AC-14 states
  "valid recomputed identity", which SR-13230 FND-002 covers.
- AC-2 stale dependency: `selected_dependencies_are_bound_by_library_identity_before_graph_checks`
  asserts the code, the cause and the expected and actual identities, but not the
  `/package/source/lock/dependency_selections/0` path that TC-047 now requires. A probe confirmed
  that path.

Every behaviour was reproduced with throwaway probe tests (see SR-13230). The exact CI command
`cargo test --locked --workspace --all-targets --jobs 2 -- --include-ignored` was run in the
reviewer worktree under flock: exit 0, 933 passed, 0 failed. 3 of those tests were the reviewer's
uncommitted probes, which only print, so 930 PR tests passed. The diff touches no file that a test
reads (only `tests/it/public_interface.rs` includes spec Markdown: FR-019 and STD-001).

## Verdict

CLEAN. The coverage delta is strictly three new planned rows, and the PR says they are planned.
The notes above are inputs for the tests-first PR, not defects in this one.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
