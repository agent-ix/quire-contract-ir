---
id: SR-13090
title: "code review of PR 476 (IR-878 canonicalization and coverage criteria tests)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; git diff origin/main...HEAD (main ): tests/it/canonicalization.rs, spec/core/matrix/tests.md, spec/model/matrix/tests.md"
review_set: subset
---
# SR-13090: code review of PR 476

## Summary

Ticket: IR-878. Non-author review of agent-ix/quire-contract-ir PR 476 at
, scoped to `git diff origin/main...HEAD`. The PR
changes three files: tests only in `tests/it/canonicalization.rs`, and status prose in two
matrix files. It changes no production code. The rust-review lane is folded into this file.

What was examined:

- `tc_017_canonical_bytes_digests_ordering_and_resource_failure_conform` gains the
  FR-016-AC-2 tag. It adds a changed-clause digest inequality, an unrelated-clause digest
  equality (the REQ_b clause), and an explicit byte equality for the reversed-insertion
  package.
- `tc_017_coverage_classes_orphans_diagnostics_and_sorting_conform` gains the NFR-003-AC-3
  tag and a loop. The loop checks the exact target-reference span of the cross-package,
  missing-requirement and stale-revision diagnostics (6, 8, 10). The existing assertion
  already checked the later duplicate's source span (16, not 14 and not 17).
- New `coverage_sorts_out_of_order_inputs_and_deep_wins_in_both_orders` is tagged
  FR-017-AC-3 and FR-017-AC-4.
- New `duplicate_diagnostics_count_later_occurrences_in_authored_order` is tagged
  FR-017-AC-6.
- Matrix status rows for StR-002, StR-003, NFR-003, FR-016, FR-017 and TC-017.

Rust-review lane: tags use `#[trace(...)]` only, with no `/// Trace:` lines. The tests use
`unwrap` only on fixture construction. Closures and fixtures are idiomatic. The tests
avoid tautologies: every expected value is written out by hand rather than computed by the
code under test. `cargo fmt --all -- --check` is clean. `cargo clippy --locked --workspace
--all-targets -- -D warnings` is clean. All nine canonicalization tests pass. TC-456,
`public_interface.rs` and `identity.rs` are untouched, and the NFR-002 matrix row is
unchanged, so the IR-881 credit hold stays in place.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The rewritten StR-002 status row lists FR-017-AC-3, AC-4 and AC-6 as newly backed. It omits FR-016-AC-2, which this PR also backs and which StR-002 traces (FR-016 through FR-018). Its "await" list names only FR-017-AC-2 and NFR-001, but FR-016-AC-1 and FR-016-AC-3 are also still untagged. A reader of the StR row gets an incomplete picture of what is still open. "now have" is also time-relative wording in a status cell | spec/core/matrix/tests.md:13 |

## Verdict

The test code is correct, idiomatic and passes fmt, clippy and the tests. The matrix prose
matches the tags, except that the StR-002 row leaves out FR-016 criteria (FND-001, low).
The criterion-strength gaps in the new tags are recorded in the gap analysis (SR-13091).

## Dispositions

- FND-001: fixed in round 1. The StR-002 status now includes FR-016-AC-2 as backed, names FR-016-AC-1 and AC-3 as awaiting direct evidence, and uses the existing 🚧 status vocabulary without time-relative wording.
