---
id: SR-12030
title: "IR-836 NFR-003 tests code review (rust lane)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; tests/it/nfr003_invalid_input.rs, tests/it/main.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/NFR-003
    type: references
  - target: ix://agent-ix/quire-contract-ir/STD-001
    type: references
---

## Summary

Ticket: IR-836, PR quire-contract-ir#428, merged against origin/main after #426. The diff adds `tests/it/nfr003_invalid_input.rs` (three tests tagged TC-019 and NFR-003-AC-1/2), registers it in `tests/it/main.rs`, and edits status and type cells in `spec/core/matrix/tests.md` and `spec/tests.md`. The rust-review lane is folded into this file. The spec rows, the class definition and the coverage accounting are judged in the gap-analysis artifact (SR-12031).

Assertion strength was measured with 15 hand-written mutants. They ran in a throwaway worktree with its own target dir, which was deleted afterwards. The review worktree was not touched.

| Mutant | Result | Caught by |
| --- | --- | --- |
| M1 invalid package/expression row carries a canonical entry | killed | per-row AC-2 assertion only (line 449) |
| M2 stale coverage trace becomes a Deep artifact row, diagnostic kept | killed | per-row AC-2 assertion only (line 458) |
| M3 duplicate artifact trace marks its requirement Deep | killed | per-row AC-2 assertion only (line 461) |
| M4 coverage `orphaned_requirement_reference` relabelled `stale_requirement_revision` (package path keeps the code) | killed | `run_corpus` inventory (`fixtures.covers`) |
| M5 `empty_enum` emitted as `duplicate_field` | killed | `run_corpus` inventory |
| M6a empty input panics in `from_json_str` | killed | hostile-bytes loop (a real panic, caught by the test harness) |
| M6b non-UTF-8 input `unwrap`s in `from_json_bytes` | killed | hostile-bytes loop |
| M6c wire-depth pre-scan disabled | killed | `run_corpus` inventory only |
| M6c-iso pre-scan disabled, corpus part of the AC-1 test skipped | AC-1 test survives | nothing in the direct or hostile part |
| M7 `expression-empty-enum` fixture deleted from corpus | killed | STD-001 guard (fixture file check) and `run_corpus` |
| M9 admitted packages report no canonical entry | killed | positive control only (line 430) |
| M9 plus controls removed | survives | nothing: the control is load-bearing |
| M10 `invalid_package_namespace` path changed | survives these tests | TC-018 (`tc_018_*`, 6 tests fail on Match) |
| M11 `empty_enum` row dropped from `CLASSES` | killed | STD-001 guard and `reached` count |
| M12 `package-schema-1-0` dropped from its class | survives | nothing |

## Reviewed scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| NFR-003-AC-1 | examined | Every declared invalid-input class returns a stable diagnostic code and no public panic. |
| NFR-003-AC-2 | examined | No malformed, unsupported, stale, undefined, or orphaned input contributes successful canonical identity or covered status. |
| NFR-003 | context_only | Untrusted input shall never trigger permissive schema interpretation, silent repair, false covered status, or a public panic. Every rejected semantic condition shall retain a stable code and the narrowest available source span. |
| STD-001 | context_only | This registry owns the stable machine-readable diagnostic codes of FR-011 through FR-019, the bounded-Kani cause and outcome error codes FR-030 raises and the code-form code FR-044 raises. |
| tests/it/nfr003_invalid_input.rs | examined | `CLASSES` table, `run`, `std001_declared_codes`, `declared_classes_match_std001`, `tc_019_every_declared_invalid_input_class_returns_a_stable_code_and_no_panic`, `tc_019_no_invalid_input_contributes_canonical_identity_or_covered_status` |
| tests/it/main.rs | examined | `mod nfr003_invalid_input;` |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | NFR-003-AC-2 says "no ... input", but the test checks only the 60 fixtures named in `CLASSES`. It does not check every rejected fixture in the corpus. Eight committed fixtures with non-empty diagnostics are never checked for AC-2: `expression-declared-collection-over`, `expression-embedded-type-depth-over`, `expression-invalid-numeric-range`, `expression-invalid-wire-number-bound`, `expression-invalid-wire-number-literal`, `expression-invalid-wire-string-grammar`, `expression-semantic-collection-over` and `package-wire-depth-maximum`. Mutant M12, which drops `package-schema-1-0` from its class, survives all three tests. A regression that gives one of these fixtures `valid:true` with canonical bytes is caught only by TC-018's frozen Match gate, which this PR deliberately stopped reading. Fix: run the AC-2 loop over every row whose `actual.diagnostics` is non-empty, and keep `CLASSES` for AC-1's class-to-code mapping. | tests/it/nfr003_invalid_input.rs:441-467 |
| FND-002 | low | The 100000-`[` hostile input never reaches a recursive decoder. `json_nesting_exceeds` refuses it first, and the loop asserts only the code (`InvalidWireFormat`), not the path. With the pre-scan disabled and the corpus part skipped (M6c-iso), the direct and hostile assertions still pass. So the hostile loop shows that a refusal comes back, but it does not pin which guard produced it. STD-001 locates this refusal at `document.nesting`. Asserting that path for the deep input (and for `package-wire-depth-over` in the direct-decode list) would pin the pre-scan. | tests/it/nfr003_invalid_input.rs:403-417, crates/quire-contract-model/src/identity.rs:1343 |
| FND-003 | low | The AC-2 decode loop picks its wire-level fixtures with a hand-written `strip_prefix` match of 14 names inside a loop over `CLASSES`. Nothing checks that every name matched a table entry, so a fixture renamed in either place silently leaves the decode check. `assert_eq!(table.len(), 44)` repeats a count that the set equality with `DiagnosticCode::ALL` on the line before already fixes. Fix: drive the decode check from the operation (`package` rows whose input is `document_json` or that fail before semantic interpretation), or assert the filtered count, and drop the literal 44. | tests/it/nfr003_invalid_input.rs:315, tests/it/nfr003_invalid_input.rs:471-491 |

## Verdict

CHANGES REQUESTED (one medium, two low). Mergeability also depends on SR-12031 FND-001 (high).

Clean units:

- **AC-1 per-class code check.** Each of the 44 classes is asserted on each of its fixtures. The test checks that the code is present (`codes.contains`), that every diagnostic is registered and has severity `error`, and that the code round-trips through the typed `DiagnosticCode`. `reached.len() == ALL.len()` closes the loop. Mutants M4, M5, M7 and M11 are killed.
- **STD-001 guard.** `std001_declared_codes` parses the four semantic tables between `## Identity and Reference Codes` and `## Typed Code Form` in the spec text. It is compared both with the hand-written table and with `DiagnosticCode::ALL`. So adding a code to the spec without the enum, adding one to the enum without a table row, or dropping a row (M11) all fail. A renamed heading fails loudly through `unwrap`. The table is a copy, but it is guarded on both sides, so it cannot drift silently.
- **AC-2 assertions can fail.** M1, M2 and M3 are killed only by the per-row assertions. `run_corpus`'s inventory check does not see them, because each mutant keeps the codes. The coverage assertion is closed: every artifact must be `orphaned` and every requirement `uncovered`, rather than "not deep". The positive controls are load-bearing (M9).
- **Removing the frozen Match assertion from `run()`.** This decouples the tests and does not weaken anything. TC-018 still asserts `FixtureStatus::Match` over the whole corpus (`tests/it/conformance.rs:526-528`, `tests/it/cycle_free_model.rs:332`), and M10 fails six `tc_018_*` tests. The NFR-003 tests now fail only on NFR-003 properties.
- **Panic detection without `catch_unwind`.** In `run_corpus`, each fixture runs on its own thread (`conformance.rs:598-616`), and a panic becomes a `RunnerError` (`fixture execution thread terminated unexpectedly`), which `expect` turns into a test failure. A panic in a direct `from_json_*` call fails the test through the harness. M6a and M6b are killed. A stack overflow would abort the test binary, which is still a failure.
- **Fixtures.** Every fixture is the repo's own committed `corpus/contract-v0.1`. The PR adds or copies no bytes, and its only non-test change is spec text.
- **Rust lane.** `use ix_trace_rs::trace;` plus a `/// Trace:` line and a matching `#[trace(...)]` on each test follows the repo convention. `unwrap`/`expect` appear only in test code. There is no unsafe code, integer conversion or async surface. `cargo test --test it nfr003` on head gives 3 passed.
- **Note, outside the diff.** `tests/it/conformance.rs:517` still carries a legacy `/// Tracing: TC-018, FR-019-AC-2, NFR-003-AC-1.` line, which quire does not read. It is harmless, but it is the same kind of over-claim that IR-210 removed.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1, after merging origin/main. AC-2 now loops over `rejected_rows(&rows)`, which is every corpus row whose `actual.diagnostics` is non-empty, not over `CLASSES`. A floor of `>= 60` guards against an empty run. The AC-1 test also requires each class's fixture list to equal exactly the set of rejected corpus rows that emit its code. The eight previously unchecked fixtures are now listed. Re-run in a throwaway worktree with its own target: dropping `package-schema-1-0` from its class now fails at line 391 ("fixtures of class unsupported_schema_version"), and adding an unlisted copy of `expression-empty-enum` to the corpus fails at line 392. The per-row mutants (invalid row given a canonical entry, stale trace made Deep, duplicate trace marking its requirement Deep) and the positive control are still killed. |
| FND-002 | fixed | Round 1. Each hostile input now carries an expected path and asserts `errors[0].path`: the 100000-`[` input must be refused at `document.nesting`, the other five at `document`. The test also asserts that the runner row `package-wire-depth-over` reports `document.nesting`. A mutant that renames the pre-scan's path to `document` fails both tests (`run_corpus`'s inventory check fires first, ahead of the new path assertions). The empty-input panic and non-UTF-8 `unwrap` mutants are still killed. |
| FND-003 | fixed | Round 1. The hand-written `strip_prefix` list is gone. Decode checks are derived from the runner rows. Every rejected `package` row is decoded: an `Err` must equal the runner's codes exactly, and an `Ok` may carry only the five codes in `AFTER_DECODE` (reference resolution and the canonical limit). A new post-decode code therefore fails loudly instead of being skipped. `decode_rejected >= 14` is a lower-bound backstop, not an exact count, so it is not brittle. The literal 44 is now `DiagnosticCode::ALL.len()`. |
