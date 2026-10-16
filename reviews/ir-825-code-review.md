---
id: SR-11845
title: "IR-825 PR 423 code review (Rust lane folded in)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@5e0eb9beb287d977dba918b81a12413374ec84ee; git diff origin/main...HEAD: Makefile, crates/quire-contract-model/tests/source_obligation_public.rs, tests/it/source_obligation.rs, tests/it/public_interface.rs, spec/model/functional/FR-052-source-obligation-correspondence.md, spec/model/functional/FR-019-rust-library-interface.md, spec/model/matrix/TC-452-source-obligation-correspondence.md, spec/model/matrix/tests.md"
review_set: subset
---

## Summary

Ticket: IR-825, PR quire-contract-ir#423, head 5e0eb9b. Code review with the
Rust lane (rust-review) folded in. The authority is FR-052 and TC-452. The PR
changes tests, trace tags, one `make test` line and status prose. It does not
change production code.

Checks run on a throwaway worktree at the head commit:

- `cargo fmt --check` passed.
- `cargo clippy --locked --workspace --all-targets -- -D warnings` passed.
- `cargo clippy -p quire-contract-model --all-targets -- -D warnings` passed.
- All 19 `source_obligation` tests pass. `source_obligation_public` passes,
  and so do the model and root doctests.
- `make test` (with `QUIRE_SPEC_LANGUAGE_DIR` set) fails in one test,
  `selected_population::duplicate_max_on_reader_admitted_population_is_not_population`.
  The QSL checkout has no `population-fixtures` feature. The PR does not touch
  that test, so this is an environment issue and not a finding here. Because
  make stops at that failure, the later `make test` lines were run by hand.
  All of them passed.

Default-feature probe for FR-052-AC-1 and AC-5: a
`#[cfg(feature = "fault-injection")] compile_error!` line was added to the
consumer test. The new make line
(`cargo test -p quire-contract-model --test source_obligation_public`) still
passed. A `--workspace` build of the same target failed with that error. So
the new make line really runs the consumer test with default features only,
and the `--workspace --all-targets` line does not.

Mutation testing: 16 source mutants were run against the branch's tests.
Fifteen were killed. These covered:

- accepting rejected assertions;
- accepting incomplete QSL;
- rewriting the incomplete cause;
- accepting a missing `quire-code-rs` outcome;
- changing the module and occurrence missing reasons;
- marking a criterion as mapped;
- dropping the inventory sort or the row dedup;
- disabling the dangling-occurrence or dangling-function check;
- dropping the same-id group dedup;
- disabling the late provenance check;
- dropping the empty-module row;
- ignoring a foreign provenance source.

One mutant survived: removing the early same-id `ConflictingProvenance`
check (FND-001).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The surviving "early provenance check" mutant is not equivalent: no test checks FR-052-AC-3's conflicting-provenance disposition for a same-ID group in which one member's provenance source is foreign. Take one assertion ID with one relation and two provenances, one from `source()` and one from a foreign source. At HEAD both members are rejected `ConflictingProvenance` and no row is accepted. With lines 1314-1315 removed, the census rejects only the foreign member, as `ForeignSource`, and accepts the other member as a correspondence row. This breaks AC-3: conflicting provenance would not get its named disposition, and a member of a conflicting group would enter accepted relations. The coder's claim that the later relation check still rejects holds only when every member passes the endpoint and source checks. A reviewer probe test kills the mutant; the branch's tests do not. Add that case to `absent_occurrence_and_same_id_provenance_conflict_never_enter_accepted_rows`. | crates/quire-contract-model/src/source_obligation.rs:1312-1318; tests/it/source_obligation.rs:1691-1733 |
| FND-002 | low | The PR adds FR-052-AC-3 to the `#[trace]` attribute of six tests, and FR-052-AC-1 to `tc_058`, but leaves each test's `/// Trace:` doc line unchanged. An example is `/// Trace: FR-052-AC-12, FR-052-AC-13` above `#[trace("FR-052-AC-12", "FR-052-AC-13", "FR-052-AC-3")]`. A reader who uses the doc line gets the wrong criterion set. The matrix reads only the attribute, so coverage is unaffected. | tests/it/source_obligation.rs:417,648,1478,1551,1599,1735; tests/it/public_interface.rs:186 |

## Verdict

CONDITIONAL. FND-001 should be fixed in this PR before merge. It is one more
case in an existing test. FND-002 is a doc-line sync.

The rest is sound:

- The consumer test builds with default features only and borrows the
  census, an inventory row, an accepted row and a rejection through typed
  references.
- The AC-2 test checks exact identities and accepted rows with provenance, a
  full-equality permutation, the occurrence-removal difference, and typed
  missing reasons in the sparse census.
- The AC-6 test covers missing and typed-incomplete outcomes at all four
  owner positions with equal causes, plus the complete empty module.
- Every AC-3 tag is on a test that asserts part of the clause. The AC-17
  test's tag covers exact-duplicate inventory rows, since `census_input`
  repeats the AC-1 row; the dedup mutant confirmed this.
- No `#[allow]`, `unsafe`, lint weakening or CI-lane change. The Makefile
  change only adds a lane.
