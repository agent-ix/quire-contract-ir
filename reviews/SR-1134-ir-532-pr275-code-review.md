---
id: SR-1134
title: "code review of PR 275 (IR-532/IR-476 bind 23 operation-law unit tests to FR-038 ACs)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@f257d1fb2612a59e410249f8c78897165eee2501; crates/quire-contract-model/src/checked_package/v2/operations.rs"
review_set: subset
---
# SR-1134: code review of PR 275

## Summary

Ticket: IR-532 (and its parent IR-476). This is a code review with the rust-review lane
folded in. It covers `git diff origin/main...HEAD` at
f257d1fb2612a59e410249f8c78897165eee2501, base main 74087d296f9bf42ee7400f479e735e3fde6272f2
(the merge base is main itself, so there is no drift).

What was checked, and what holds:

- **The diff is attributes and doc lines only.** It has 69 additions and 0 deletions over 23
  hunks. Each hunk adds exactly `///`, `/// Tracing: TC-048, FR-038-AC-NN` and
  `#[trace("TC-048", "FR-038-AC-NN")]` above an existing `#[test]`. No test body, helper
  or production line changed.
- **Convention.** The added block matches the file's own convention: the doc line plus the
  `#[trace]` attribute above `#[test]`, as on `operand_classification_is_exactly_the_catalog_mapping`
  (AC-87) and the 43 tests that were already traced. The `trace` macro is already in scope
  in the tests module.
- **Counts.** At head, `operations.rs` has 66 `#[test]` and 66 `#[trace(` (43 at base).
  `tests/it/executable_binding.rs` has 13 `#[test]` and 13 `#[trace(`, and no bare
  `/// TC-NNN` line. No `.rs` file in the repo carries a bare `/// TC-035` line. These
  counts come from reading the files and matching attributes, not from the PR text.
- **Each binding, read against the merged FR-038 AC text.** All 23 bindings are correct
  (listed in the posted comment's `bindings:`). The least certain one is
  `operation_defect_refuses_mode_type_mismatch_on_leaf` to AC-44, and it is correct. The
  test asserts `operation-mode-type-mismatch` at `operation.leaves/0/mode/value` for a leaf
  mode `{text_profile, nfd}` over a field whose `text_bounds` type binds `nfc`. AC-44's
  clause is "a catalogued value other than the pinned one `operation-mode-type-mismatch`
  at `mode/value`", for a leaf. AC-83 is about the top-level `operation.mode` with
  `rounding`. `nfd` is catalogued: the code returns type-mismatch for it, not mode-mismatch,
  and the AC-44 case at operations.rs:5017 asserts that. The doc prose above this test is
  wrong, though (FND-001).
- **Rust lane.** No unsafe, panic surface, integer conversion, async or lock change.
  `cargo test --locked -p quire-contract-model --lib checked_package::v2::operations`
  passes 66/66. `cargo fmt --all -- --check` is clean. `make lint` (clippy `-D warnings`)
  passes.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The doc comment of `operation_defect_refuses_mode_type_mismatch_on_leaf`, which now carries `Tracing: TC-048, FR-038-AC-44`, says the record's `name` field type "pins `rounding` to `\"nearest-even\"`" and that `check_leaf_count` refuses it. The fixture actually binds `text_profile` to `nfc`, and the leaf mode is `{text_profile, nfd}`. That prose describes AC-83's subject (a `rounding` pin), so the new AC-44 trace reads as a wrong binding even though it is right. The prose predates the PR, but the PR attaches a trace claim to it. Fix: say the field's `text_bounds` type binds `text_profile` `nfc` and the leaf's catalogued `nfd` disagrees | crates/quire-contract-model/src/checked_package/v2/operations.rs:4098-4108 |

## Verdict

The change is what it says: 23 trace attributes and doc lines, with no test body changed,
in the file's own convention. Every binding backs the AC it names. The one finding is low
(stale prose above a correct binding). On its own, this method would not block merge. See
SR-1135 for the matrix finding that does.

## Dispositions

Round 1 at ec5ac123f3e93d9cf75c09eb9665e37681aff977 (one fix commit on top of f257d1f; main
has not moved from 74087d2, and GitHub reports MERGEABLE). The full `origin/main...HEAD` diff
of `operations.rs` still changes only `///` and `#[trace(` lines, so no test body changed.
66 tests and 66 trace attributes. The 66 `operations.rs` tests pass, and fmt-check is clean.

FND-001. The leaf test's doc now says the `name` field's text type pins the `nfc`
`text_profile` and that the leaf's `nfd` is a catalogued value other than the pinned one.
This matches the fixture and AC-44's clause, and the `rounding` wording is gone.

No new findings.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | ec5ac123f3e93d9cf75c09eb9665e37681aff977 |
