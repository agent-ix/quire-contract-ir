---
id: SR-13251
title: "IR-893 PR 481 gap analysis: FR-035-AC-2, AC-12 to AC-14 backing"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir vs merge- and origin/main; spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md, spec/checked_package/matrix/TC-047-complete-v1-contract-package-output.md, spec/checked_package/matrix/tests.md, tests/it/complete_v1_contract_package.rs"
review_set: subset
---

## Summary

Ticket: IR-893. PR agent-ix/quire-contract-ir#481. Plan completion: not assessed (planless).

The reviewer ran `quire coverage --scope . --format tsv` (quire 0.36.2) at the merge and at the:

- Merge base: 543/612 backed.
- Head: 546/612 backed.

The only difference is that the three unbacked rows FR-035-AC-12, AC-13 and AC-14 are gone. No row was lost, and neither side has a contradicted row. FR-035 is 13/14 at the head, with AC-9 unbacked. This matches the author's claim. `quire validate` is grammar-clean (1131/1131). `make spec` still exits non-zero on its strict step, because of 40 unbacked rows that existed before this PR.

Bindings checked:

- `reader_refuses_each_missing_required_top_level_member` → FR-035-AC-12. It covers all six members, including the `contract_version` branch.
- `reader_refuses_each_leading_json_whitespace_byte` → FR-035-AC-13. It covers space, tab, LF and CR, each with path and cause absent.
- `reader_refuses_first_accounting_record_for_later_requested_items` → FR-035-AC-14. It covers `requested_items[1..=3]` with a recomputed identity, and asserts `/feature_accounting` and `invalid_value`. The test is weak against a mutant of the order check (SR-13250 FND-001).
- `reader_refuses_stale_own_identity_at_digest` and `reader_refuses_stale_selected_dependency_at_selection` → FR-035-AC-2.

All five tags are correct.

The PR conflicts with origin/main. PR #480 (IR-889, merged) changed the same TC-047 and tests.md lines. It amended AC-11, added AC-15 and AC-16, and recorded that the AC-11-tagged test's assertions are stale.

## Examined criteria

- FR-035-AC-12 (examined): "Given an otherwise valid canonical `ContractPackageWireV1` that omits any of its six required top-level members ... returns `Refused` with code `invalid_package`, root path `\"\"`, and cause `missing_member`; it returns no admitted package."
- FR-035-AC-13 (examined): "Given otherwise valid `ContractPackageWireV1` bytes with leading whitespace before the canonical JSON value, `ContractPackageWireV1::read` returns `Refused` with code `noncanonical_wire`, absent path, and absent cause; it returns no admitted package."
- FR-035-AC-14 (examined): "Given an otherwise valid canonical `ContractPackageWireV1` except that its first `feature_accounting` record names a requested node other than the first `source_map.requested_items` entry, `ContractPackageWireV1::read` returns `Refused` with code `invalid_package`, path `/feature_accounting`, and cause `invalid_value`; it returns no admitted package."
- FR-035-AC-2 (examined): "A source, type, anchor, identity, bound, dependency, or contract-version mutation that changes or invalidates meaning refuses before a backend artifact; a stale selected dependency does not resolve through an identically keyed local node."
- TC-047 (examined): the test procedure and Status sections.
- tests.md FR-035 and TC-047 rows (examined).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The PR does not merge: GitHub reports CONFLICTING, and `git merge-tree` conflicts in TC-047 and tests.md against origin/main (#480). The PR's status text is also stale against main. TC-047 Status line 127 and both tests.md rows say AC-10 and AC-11 are backed, and that "AC-11 work-limit scope remains open under IR-889". Main now says that the AC-11-tagged test's assertions are stale, do not back amended AC-11, and that AC-11, AC-15 and AC-16 are PLANNED/UNRUN. Main's FR row range is AC-1 through AC-16, and TC-047 traces AC-15 and AC-16. Resolved with this PR's side, the merge would re-assert a backing that main withdrew. Fix: rebase onto origin/main. Keep #480's AC-10, AC-11, AC-15 and AC-16 sentences and its row range. Replace only the IR-893 sentences ("AC-12 through AC-14 and the AC-2 TC-047 retags... PLANNED/UNRUN"). Re-run `quire coverage` to confirm +3 rows against the new main, and update the coverage figures in the PR body | spec/checked_package/matrix/TC-047-complete-v1-contract-package-output.md:127-135; spec/checked_package/matrix/tests.md:12,30 |

## Verdict

**FAIL until rebased.** The coverage delta is exactly +3 backed rows, with none lost and none contradicted. All five bindings are correct. FND-001 blocks the merge, because the branch conflicts with main and its status prose contradicts #480. AC-14's backing is mutation-weak; see SR-13250 FND-001.

## Dispositions

Disposition pass 1 reviewed the PR after origin/main was merged in.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | GitHub now reports MERGEABLE, and the only block is the required review. The TC-047 Status and tests.md rows keep #480's AC-10, AC-11, AC-15 and AC-16 text and its AC-1 through AC-16 range. Only the IR-893 sentences changed. `quire coverage` gives 543/614 on origin/main and 546/614 at the head. Only FR-035-AC-12 to AC-14 moved, none were lost, and none are contradicted |
