---
id: SR-1000
title: "integrity review of PR 264 (identity digests and v1 integers through quire-canonical)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@184e6668bb8722f03c68a6b9924586593233118d; git diff origin/main...HEAD: FR-011, FR-012, FR-013, FR-016, FR-019, FR-020, FR-032, FR-033, FR-034, STD-003, FR-038, TC-048, AD-004, AD-005, the five matrices and spec/tests.md; cross-checked against merged FR-038-AC-26, AC-30, AC-65 to AC-80 and open PR 263 (FR-038-AC-81 to AC-88)"
review_set: subset
---
# SR-1000: integrity review of PR 264

## Summary

Ticket: IR-274. Checks completeness, consistency and atomicity across the
amended requirements, and that no requirement is removed silently.

- `quire validate` passes. Grammar findings are 1 before and 1 after (the
  FR-014 line 137 baseline). Acceptance criteria go from 198 to 214. Strict
  coverage has 23 unbacked rows before and after. Coverage rows backed go from
  174/220 to 174/236. These match the PR body.
- Named amendments: FR-011-AC-3, FR-012-AC-6, FR-016-AC-1 and FR-038-AC-27 are
  amended in place and keep their trace tags. FR-038's "leaves every other
  canonical path as it is", "retired by IR-274" and the crate-private `Value`
  walker paragraph are replaced by named text. Nothing is removed silently. The
  v1 depth limits and the v2 16,384 deviation are unchanged.
- IR-274's ticket acceptance is covered. Items 1 and 2 by FR-016-AC-6 and
  FR-038-AC-91. Item 3 by FR-016-AC-6 (source) plus AD-005's VER-52 note on
  `Cargo.lock`. Item 4 by FR-033-AC-6 and FR-034-AC-6. Item 5 by the
  no-copied-vector clauses. The ticket comment asking the v1 loaders to refuse
  large bare numbers is covered by type: strings for the eight members, and
  revisions refused at construction.
- Textual conflicts are expected rebase work, not defects. Against #263 the
  conflicts are in the FR-038, TC-048 and checked_package tests.md files.
  Against #250, #253 and #259 they are on the one-line FR-038 row of
  checked_package tests.md.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-020-AC-3 and its prose contradict themselves. The fixture schema rejects a JSON number in the eight members, and the runner validates every fixture input against that schema before decoding it (`conformance.rs:570-574`, `invalid_corpus`). So no corpus fixture with a JSON number in one of them can "decode to `invalid_wire_format`" with an expectation carrying that code. The `invalid_wire_format` clause holds only for a library-level decode, not for the corpus | spec/conformance/functional/FR-020-json-conformance-interface.md:99-110, 127 |
| FND-002 | low | The FR-020 matrix row says "the schemas and corpus are amended with the spec". The corpus is not amended in this PR: 59 inputs and 47 canonical files still carry numbers. That gap is what makes the schema change fail (SR-999 FND-001) | spec/conformance/matrix/tests.md:13 |
| FND-003 | low | FR-016-AC-7's second clause restates the 2^53 revision and byte-offset refusal that FR-011-AC-3 and FR-012-AC-6 already own (same codes, same boundary). One behaviour now has three acceptance owners, so an amendment must touch all three | spec/model/functional/FR-016-canonicalization-digests.md:142 |

## Verdict

Changes needed for FND-001. Scope FR-020-AC-3's `invalid_wire_format` clause to
the library decoders (FR-013-AC-5 already covers them), and say a corpus input
with a number is `invalid_corpus`, or drop the clause. FND-002 follows from
moving the schema edits (SR-999 FND-001). FND-003: keep the refusal in
FR-011/FR-012 and have FR-016-AC-7 cite them.

## Dispositions

Reviewed at agent-ix/quire-contract-ir@553736cfce62fe6949915559c471d4a7f5349d21 (main still 7d7716d; not rebased onto #263, whose FR-038, TC-048 and checked_package tests.md hunks conflict textually, expected rebase work). Proof re-run: no schemas/, corpus or code file in `git diff origin/main --stat`; `make corpus` exit 0; workspace tests all pass (204 + 101, 0 failed); validate passes, grammar 1 (FR-014 baseline), 215 ACs, strict 23 unbacked, coverage rows 237.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 553736c |
| FND-002 | fixed | 553736c |
| FND-003 | fixed | 553736c |
