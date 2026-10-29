---
id: SR-12115
title: "IR-839 code review (rust lane)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; tests/it/conformance_mismatch_independence.rs, tests/it/main.rs, crates/quire-contract-model/src/conformance.rs (read as context), tests/it/conformance.rs (read as context)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-018
    type: references
---

## Summary

Ticket: IR-839, PR quire-contract-ir#436,, one commit over origin/main. The rust-review lane is folded into this file. The diff adds `tests/it/conformance_mismatch_independence.rs` (299 lines, two tests traced `TC-018` and `FR-018-AC-2`) and registers the module in `tests/it/main.rs`. No production code changes. The old combined test `tc_018_all_mismatch_kinds_and_exit_classes_are_stable` in `tests/it/conformance.rs` is untouched.

## Reviewed scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-018-AC-2 | examined | Mutation fixtures independently alter schema validity, diagnostic code/path/order/span/obligation, canonical byte, digest, dependency, and coverage row/reason; each produces the exact mismatch result without message parsing. |
| FR-020 (mismatch kinds) | context_only | Mismatch kinds are `validity`, `diagnostics`, `canonical_bytes`, `canonical_digest`, `dependencies`, and `coverage`. A fixture with several drifts retains all applicable kinds once in this fixed registry order |
| crates/quire-contract-model/src/conformance.rs:2095-2115 | context_only | `fn mismatch_kinds(actual: &Value, expected: &Value) -> Vec<&'static str>` pushes the six kinds in registry order |
| tests/it/conformance_mismatch_independence.rs:30-91 | examined | `const CASES: &[Case]`: ten one-mutation cases over six kinds |
| tests/it/conformance_mismatch_independence.rs:107-150 | examined | `struct Fresh` copies corpus and schemas to `temp_dir/quire-contract-ir-fr018ac2-<pid>-<label>`, removed on drop |
| tests/it/conformance_mismatch_independence.rs:193-209 | examined | `fn assert_only`: exit 1, empty stderr, mutated row `mismatch` with kinds exactly `[kind]`, every other row `match` with no kinds |
| tests/it/conformance_mismatch_independence.rs:214-248 | examined | `tc_018_each_mismatch_kind_is_reported_independently_for_one_mutation` |
| tests/it/conformance_mismatch_independence.rs:253-299 | examined | `tc_018_diagnostic_order_alone_is_reported_as_a_diagnostics_mismatch` |
| tests/it/main.rs:55 | examined | `mod conformance_mismatch_independence;` |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | A runner that compares only the first diagnostic survives both new tests and the old combined test (mutant `first_diag_only`: 3 of 3 pass). Every one-mutation case edits `diagnostics[0]`, and the order test's swap also changes element 0, so no case drifts a diagnostic past index 0 or changes the diagnostic count. Reproduced by hand: under the mutant, the order setup (2 actual diagnostics against 1 expected) reports only `coverage`, so a missing or extra diagnostic is invisible. FR-018-AC-2 asks for the exact mismatch result over ordered diagnostics. One more case on the re-baselined two-diagnostic fixture, in a fresh copy, would kill it: mutate `diagnostics[1]` only (for example its code), or drop the second diagnostic. | tests/it/conformance_mismatch_independence.rs:288-298, crates/quire-contract-model/src/conformance.rs:2100 |
| FND-002 | low | The table test runs the full corpus 11 times in sequence inside one `#[test]`: a baseline and ten cases. It took 102 s wall in a debug build here; one corpus run is about 7.4 s, and copying the 39 MB corpus takes about 20 ms. Shrinking the copies does not help, because a partial corpus fails `invalid_corpus` ("observed coverage union differs from inventory", reproduced). Splitting the table into one `#[test]` per case would let libtest run the cases in parallel at the same CPU cost. This is CI time only; the repo has no clock assertion. | tests/it/conformance_mismatch_independence.rs:214-232 |
| FND-003 | low | `Fresh`, `copy_tree`, `read_json`, `write_json` and `runner` re-implement helpers `tests/it/conformance.rs` already has (`Scratch`, `copy_tree`, `read_json`, `write_json`, `run_with`). Two copies of the same scratch-corpus code will drift. They could live in one shared test-helper module. | tests/it/conformance_mismatch_independence.rs:93-171, tests/it/conformance.rs:52-110 |
| FND-004 | low | The closing guard compares the table's kinds to a hand-typed list of six. It catches a case table that loses a kind. It cannot catch the runner gaining a seventh kind, and it does not check the criterion's sub-dimensions (code/path/span/obligation, row/reason), only the six kind names. Its comment says "Every kind the criterion names". The criterion names mutation dimensions; FR-020 names the six kinds. | tests/it/conformance_mismatch_independence.rs:234-247 |

## Verdict

Changes requested: FND-001 (medium), which is a small test addition. FND-002..FND-004 are low and do not block merge.

Kinds against the specification:

- FR-020 names exactly six mismatch kinds, and the runner's `mismatch_kinds` pushes exactly those six in registry order. The table covers exactly these six, with no extra and none missing.
- FR-018-AC-2's dimensions map to cases as follows. Validity: `validity`. Diagnostic code, path, span and obligation: four `diagnostics` cases. Diagnostic order: the separate order test. Canonical byte: `canonical_bytes`. Digest: `canonical_digest`. Dependency: `dependencies`. Coverage row and reason: two `coverage` cases. Every dimension the criterion names has its own case.
- "Without message parsing" holds by construction. The fixture schema has no `message` member and no expectation carries one (grep over `expectations/` and `schemas/`), so a message cannot reach the comparison.

Mechanics (verified):

- Each case builds a fresh `Fresh` copy, applies one closure to one expectation, and asserts that the mutation changed the value (`assert_ne!(before, expectation)`). It then writes the file and runs `assert_only`.
- `assert_only` checks exit 1, empty stderr, the mutated row's `status == "mismatch"` with `mismatch_kinds == [kind]`, and that every other row is `match` with an empty kinds array. It also checks that the fixture's row was seen.
- The unmutated corpus control runs first and asserts exit 0 and empty stderr.
- The canonical-bytes case repoints `bytes_path` at `canonical/package-constructs-1.json`, which exists. The digest field is unchanged, so only `canonical_bytes` drifts. The digest case zeroes the digest and leaves the bytes alone.

Order test, judged fair and not circular:

- No shipped expectation has two or more diagnostics (checked with jq over all 106 expectations), so the test needs a constructed input.
- The second stale trace is fixture setup. It is not a mutation of the comparison under test. The test re-baselines the expectation from the observed result, then proves the baseline with a full-corpus exit-0 control run before it changes anything. The one mutation is then the swap.
- Taking the observed result as the expectation is fair, because the property under test is relational: swapping two distinct diagnostics must be reported as `diagnostics` and nothing else. It is not a claim that the diagnostics themselves are correct; that is FR-018-AC-1's job and the frozen corpus's.
- The guards are sound. `len >= 2` and `assert_ne!(d[0], d[1])` make the swap a real reordering.

Mutants, run in a throwaway worktree with its own target dir (since removed). Each was run against the two new tests and the old combined test:

| Mutant | Change in `mismatch_kinds` | New table test | New order test | Old combined test |
| --- | --- | --- | --- | --- |
| merge_diag_into_validity | diagnostics drift reported as `validity` | killed | killed | killed |
| coverage_as_dependencies | coverage drift reported as `dependencies` | killed | pass | killed |
| extra_kind | diagnostics drift also pushes `coverage` | killed | killed | killed |
| extra_kind_any | any drift also pushes `coverage` | killed | killed | killed |
| drop_dependencies | dependencies never compared | killed (exit 0) | pass | killed |
| swap_canonical | `canonical_bytes` and `canonical_digest` labels swapped | killed | pass | killed |
| digest_as_bytes | digest drift reported as `canonical_bytes` | killed | pass | killed |
| first_diag_only | only `diagnostics[0]` compared (plus presence) | **survived** | **survived** | survived |
| sorted_diags | diagnostics sorted before comparison | pass | killed (exit 0) | survived |

`sorted_diags` is killed only by the new order test, so that test closes a real hole: before this PR, order insensitivity could not fail anything. `first_diag_only` is FND-001.

Rust lane:

- `cargo fmt --all --check` passes.
- `cargo clippy --workspace --all-targets -- -D warnings` is clean (run in the throwaway worktree, unmutated).
- The `Case.mutate` closures are non-capturing and coerce to `fn(&mut Value)`.
- `unwrap` appears only in test code.
- `Drop` removes the scratch dir on success and during a panic unwind. A killed process (SIGKILL, harness timeout) leaves its 39 MB copy behind. The old `Scratch` helper has the same limit.
- Names are `quire-contract-ir-fr018ac2-<pid>-<label>`. The labels are unique across both tests, and the prefix is distinct from `conformance.rs`'s `quire-contract-ir-tc018-`, so parallel test threads and parallel processes cannot collide.
- After all the runs, including the panicking mutant runs, no `quire-contract-ir-fr018ac2-*` dir remained in `/tmp`. Sixteen older `quire-contract-ir-tc018-*` dirs from 2026-10-06, left by the old helper in killed processes, are not this PR's.

Traceability:

- `quire trace --id FR-018-AC-2 --json` resolves three `rust-trace-attribute` claims: the old combined test plus the two new tests.
- `make spec` gives strict 43 unbacked / 0 contradicted and `Coverage: 508/565 rows backed`. Both are unchanged; AC-2 was already backed by the combined test.

## Dispositions

Round 1. The new commits are (a merge of origin/main ) and. The PR diff against origin/main is now `tests/it/conformance_mismatch_independence.rs`, `tests/it/main.rs`, the new `tests/it/support/corpus.rs`, `tests/it/support/mod.rs` (+1 line), `tests/it/conformance.rs` (imports only), and the two `reviews/` copies.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | adds `two_diagnostic_baseline`. It builds the two-diagnostic setup, takes the observed result as the expectation, and asserts an exit-0 control run. Three tests sit on it, each with its own fresh copy and one mutation: swap the two diagnostics; change only `diagnostics[1].code`; drop the second diagnostic. I re-ran the `first_diag_only` mutant in a throwaway worktree with its own target dir (since removed). It is now killed by `tc_018_a_drift_in_the_second_diagnostic_alone_is_a_diagnostics_mismatch` and `tc_018_a_missing_second_diagnostic_alone_is_a_diagnostics_mismatch`. `sorted_diags` is still killed by the order test. |
| FND-002 | fixed | The ten cases are now ten `#[test]` functions that call `run_case(label)`. A separate `tc_018_the_case_table_produces_every_fr020_mismatch_kind` holds the guard. The module's 14 tests run in 32 s wall, down from 102 s + 17 s. Scratch labels are `ac2-<label>`, unique across the 14 tests. None of `conformance.rs`'s 22 `Scratch::corpus` labels starts with `ac2-` or repeats. After the runs, no `quire-contract-ir-tc018-*ac2*` dir remained in `/tmp`. |
| FND-003 | fixed | The helpers moved verbatim to `tests/it/support/corpus.rs`, now `pub`. `support/mod.rs` gains `pub mod corpus;`. `git diff origin/main...HEAD -- tests/it/conformance.rs` shows only the `PathBuf` import narrowed to `Path`, the helper block removed, and one `use crate::support::corpus::{...}` added. No test, assertion or trace attribute changed. `tc_018_all_mismatch_kinds_and_exit_classes_are_stable` is untouched. All 26 `conformance*` tests pass. |
| FND-004 | accepted-no-change | Residual accepted, with the comment fixed. The guard comment now cites FR-020 and states that a hand-typed set cannot notice a seventh runner kind. The runner exports no kind registry. A seventh kind would need an FR-020 change, which is the point at which this list gets reviewed. The sub-dimension half is now guarded by construction: each sub-dimension has its own named test, and `run_case` unwraps the label lookup, so removing a case from the table makes its test panic. |

Also checked this round:

- `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --all --check` are clean.
- `quire trace --id FR-018-AC-2` resolves 15 `rust-trace-attribute` claims: the old combined test plus 14 new ones.
- `make spec` gives strict 43/0 and 508/572 rows backed, identical to origin/main. The denominator grew from 565 to 572 through the merge of main, not through this PR.
- The per-case tests no longer run their own unmutated-corpus control. `assert_only` still requires every other row to match, and `tc_018_the_corpus_runs_deterministically_and_every_fixture_matches` owns the baseline, so this is not a defect.
- `Scratch` now places the copy at `<root>/contract-v0.1`, so the rows carry the real corpus id.
