---
id: SR-1129
title: "code review of PR 274: QSpec conformance harness for FR-038-AC-112 and AC-113, prose cleanup, recursion tests (IR-552)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@cb0a8bf46b879a39de3c27682c208cb879299a7d; git diff origin/main...cb0a8bf (base 90ad41d, 10 files: tests/conformance_qspec/main.rs, tests/it/checked_package_v2_{reader,recursive_leaves,temporal,union}.rs, crates/quire-contract-model/src/checked_package/{common.rs,v2/operations.rs,v2/temporal.rs}, spec/checked_package/matrix/tests.md, spec/tests.md)"
review_set: subset
---
# SR-1129: code review of PR 274: QSpec conformance harness for FR-038-AC-112 and AC-113, prose cleanup, recursion tests (IR-552)

## Summary

Ticket: IR-552. This file holds the code-review method with its rust-review lane. The PR body and the coder's claims were treated as data and measured.

What I measured, in a detached throwaway worktree at cb0a8bf with its own target directory, against a throwaway worktree of quire-specification origin/main at 396493c (both since removed):

- `QUIRE_SPECIFICATION_DIR=<qspec origin/main> make conformance-qspec` FAILS. Four of five tests pass. `tc_048_qspec_adverse_mutations_refuse_as_recorded` fails with one problem: `negative-temporal-interval-bound: read as refused:invalid_package/invalid-value, not the recorded refused:invalid_package`. That mutation was added to `adverse.json` by QSpec #181 (f39c93f). The stale local checkout at b1da9c8 does not hold it, which is why the run passed for the author. See FND-001.
- The five `body_grammar_mutations` produce exactly the listed refusals on origin/main (no problem is reported for any of them, and the run collects every problem). Each `flattened` control is not refused `malformed_wire`.
- AC-113 passes: the version-free selections derive the recorded `0606043a...abf2` through `CheckedPackageIdentityPreimageV2` and `quire_canonical::sha256`, the same call the reader makes in `validate` (mod.rs:772). The harness does not hash raw JSON.
- No QSpec file is copied into the repo: no `adverse*.json`, `dependency-selection-vectors*` or `positive-*` file exists in the tree, and no file was added. `.github` is untouched.
- The touched it modules pass (recursive_leaves, union, temporal: 49 tests). `make fmt-check` and `make lint` (workspace and model-only clippy) pass. `make spec` stops at the known baseline: validate passes, grammar 1 (FR-014), 23 unbacked under strict, 0 contradicted.
- Recursion leaves checked against merged QSpec FR-322 "Structural leaf walk" (lines 632-692), which gives both examples verbatim: `record Cell { item: (Text, Option<Cell>) }` lists `["field:item", "position:1", "inner", "recursion:0"]`, and `Option<Chain>` lists `["inner", "member:Link", "position:1", "recursion:1"]`. The new tests assert exactly these, and refuse the wrong depth.
- Prose. operations.rs: the `member:<Name>` and recursion-leaf comments now match FR-322 "Structural leaf walk", and both V2 schemas' `LeafSegment` pattern admits `recursion:<d>`, so dropping "a stated deviation from QSpec's schema" is right. temporal.rs `misplaced_in_details`: FR-322 "Body grammar", FR-370 lines 180-181 and FR-440 "Case placement" all admit a `details` reference to a union or union value node, so the citation is right. common.rs: the FR-322/FR-440 citation for nested `case` is right; see FND-006 for the rest of that sentence. The FR-370 "orphan" claim holds: the temporal test edits are comment text only, inside `tc_048_an_interval_member_admits_and_refuses_on_each_interval_operator`, which already carries `FR-038-AC-97`; see FND-005 for the new wording.

Fail-closed cases of AC-112, each checked for whether it would still pass with its guard removed: unset, empty and non-checkout variable (genuine); missing file and non-JSON file (genuine); no `flattened` member (genuine, asserts the `flattened` message); unlisted mutation not as recorded (genuine, asserts the id); listed id absent, stale and neither (genuine, each asserts its own message). The absent-list and empty-list cases are vacuous; see FND-002.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The harness compares the whole outcome string, so a recorded outcome that names a code but no cause fails whenever the reader also gives a cause. AC-112 says each mutation refuses with 'exactly the code its outcome names (refused:<code>, and the cause after a / where it gives one)': the cause is compared only where the outcome gives one. On quire-specification origin/main (396493c) `negative-temporal-interval-bound` records `refused:invalid_package`, the reader gives `refused:invalid_package/invalid-value`, and `make conformance-qspec` fails. The author's pass came from a checkout older than QSpec #181. Fix: compare the code, and the cause only when the recorded outcome has a `/<cause>`; apply the same rule to the listed-refusal and stale-entry comparisons; add a fail-closed case for a cause-less outcome. Do not list this id as an expected failure: the reader refuses it with the recorded code. | tests/conformance_qspec/main.rs:289-307 |
| FND-002 | medium | The 'list absent' and 'list empty' fail-closed assertions are vacuous. Both pass `body_grammar_mutations[0]` with an empty expected-failure list. The reader gives that entry `refused:invalid_package/stale-node-key`, not its recorded `refused:malformed_wire`, so `adverse_problems` returns a problem whether or not the absent/empty-list guard exists. Deleting the `mutations` non-empty check leaves both assertions green. Assert the specific message (`has no non-empty structural_mutations list`), or pair the list under test with entries that pass on their own. | tests/conformance_qspec/main.rs:474-482 |
| FND-003 | low | `assert!(listed.ticket.starts_with("IR-"))` iterates a `const` whose every entry is the literal `"IR-495"`, so it cannot fail. Its comment says 'A listed id is a mutation of the file', which this loop does not check (the absent-id check is in `adverse_problems`). Drop the loop or replace it with a check that can fail. | tests/conformance_qspec/main.rs:436-439 |
| FND-004 | low | The `moved` case's comment says it shows 'the harness derives through the reader's own derivation'. A harness that hashed the raw JSON would also fail it, since any changed digest moves any hash. The assertion is fine as a regression check. The claim is not what it proves; the typed-preimage call itself is what satisfies AC-113's 'cannot pass' clause. Reword the comment. | tests/conformance_qspec/main.rs:599-603 |
| FND-005 | low | The new citation 'merged QSpec FR-370 temporal step' for an interval bound outside the pattern contradicts merged QSpec. FR-370 lines 121-125 say such a bound 'refuses invalid_package/invalid-value at that bound's pointer during strict wire validation, before any step of this requirement runs'. The comment's own 'in the early stage' agrees with QSpec, and the citation contradicts it. Cite 'merged QSpec FR-370 (strict wire validation, before the temporal step)'. | tests/it/checked_package_v2_temporal.rs:615-618 |
| FND-006 | low | The edited comment still says the nested `case` refusal is 'here where nested applications are refused'. The reader does not refuse other nested applications today: the harness's own expected-failure list shows `application-in-application-arguments` reaching `stale-node-key`, and IR-495 owns that change. Merged FR-322 'Body grammar' refuses every other nested application `malformed_wire`. The comment should say that this is where nested applications will be refused, and that today only `case` is refused here, a deviation that IR-495 closes. | crates/quire-contract-model/src/checked_package/common.rs:741-745 |
| FND-007 | low | The doc comment of `tc_048_a_union_cycle_under_an_option_enters_at_recursion_one` lists three refused cases, 'the recursion leaf missing, one reading recursion:0 and the text leaf alone'. The first and third are the same list, and the test asserts two cases. Drop 'and the text leaf alone'. | tests/it/checked_package_v2_union.rs:805-809 |

## Verdict

The harness design is sound. It reads QSpec's files from the checkout and copies nothing. It applies each mutation to a fresh copy with no identity refreshed. Its code and cause words are exhaustive matches with no wildcard, so an added refusal variant cannot read as a stale word. The expected-failure semantics (absent, stale, neither, unlisted) are implemented as AC-112 states, and each has a genuine fail-closed test. AC-113 derives through the reader's own types and `quire-canonical`. The recursion tests match merged QSpec FR-322 exactly, and the prose citations are accurate apart from FND-005 and FND-006.

FND-001 blocks merge. The AC-112 gate is red on current QSpec, and the matrix row says it is verified. FND-002 is a medium oracle gap. The rest are low.

Out of diff scope, for information only: the Makefile help line for `conformance-qspec` still reads 'QSpec's positive fixtures admit'.

## Dispositions

Round 1 was reviewed at 05465f3f419fe4d49aa6c8a598bbac730cd40fc8, one fix commit on cb0a8bf. I measured it in throwaway worktrees, since removed. `make conformance-qspec` against quire-specification origin/main (396493c) passes all 6 tests, including the new `tc_048_a_recorded_outcome_without_a_cause_is_met_by_its_code`. I deleted each guard in turn in a throwaway copy, never on the PR branch, and every mutant fails a test:

- The cause compared always: the new unit test and the adverse result test fail.
- The non-empty list guard deleted, both halves: `tc_048_qspec_adverse_run_fails_closed` fails.
- Only the empty-list half deleted: the same test fails.
- The selections non-empty filter deleted: `tc_048_qspec_selection_run_fails_closed` fails.
- The own-package_id check deleted: the same test fails.

`make fmt-check` and `make lint` pass. `make spec` is at the baseline: grammar 1, 23 unbacked, 0 contradicted.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed 05465f3 | `refuses_as` compares the cause only when the recorded outcome has one. The unlisted and stale-entry checks both use it, and a unit test pins it. The run passes on QSpec main, and a mutant with the whole-string compare fails. |
| FND-002 | fixed 05465f3 | The body-list cases assert exactly one problem, the list's own. The structural-list cases assert that list's specific message. Deleting the guard fails the test. |
| FND-003 | fixed 05465f3 | The tautological ticket loop and its comment are removed. |
| FND-004 | fixed 05465f3 | The comment no longer claims the case proves the reader's derivation, and the assertion now names its problem. |
| FND-005 | fixed 05465f3 | Now cited as 'merged QSpec FR-370: strict wire validation, before the temporal step'. |
| FND-006 | fixed 05465f3 | The comment now says merged FR-322 refuses every other nested application `malformed_wire`, that this reader refuses only `case` here today, and that IR-495 owns the rest. |
| FND-007 | fixed 05465f3 | The doc now lists the two cases: the recursion leaf missing (the text leaf alone) and `recursion:0`. |
