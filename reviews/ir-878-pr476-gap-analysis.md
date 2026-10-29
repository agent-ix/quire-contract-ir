---
id: SR-13091
title: "gap analysis of PR 476 (IR-878 canonicalization and coverage criteria tests)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; spec/model/functional/FR-016-canonicalization-digests.md, spec/model/functional/FR-017-version-orphan-coverage.md, spec/core/non-functional/NFR-003-diagnostic-integrity.md, spec/core/matrix/tests.md, spec/model/matrix/tests.md, tests/it/canonicalization.rs, crates/quire-contract-model/src/coverage.rs, crates/quire-contract-model/src/canonical.rs"
review_set: subset
---
# SR-13091: gap analysis of PR 476

## Summary

Ticket: IR-878. Plan completion: not assessed. This is a PR-scoped audit of five new tags:
FR-016-AC-2, FR-017-AC-3, FR-017-AC-4, FR-017-AC-6 and NFR-003-AC-3. It checks whether each
tag's test asserts the whole criterion, using mutants run in a throwaway worktree.

### Strict coverage

`quire coverage --scope. --strict` (quire 0.36.2, engine 0.50.2) was run on main
and on the PR head.

- Backed rows go from 536/607 to 541/607, with 0 contradicted statuses.
- The newly backed rows are exactly FR-016-AC-2, FR-017-AC-3, FR-017-AC-4, FR-017-AC-6 and
  NFR-003-AC-3. No row lost backing.
- The criteria tagged count goes from 493 to 498.

The footer's "40 unbacked row(s)" is unchanged, and the set of 40 rows is identical on both
sides. The footer and the headline count different things:

- The footer lists `unbacked_rows`. These are traceability references
  (`verification`, `nfr-verification`, `functional-coverage`, `traces-to`) whose target
  has no backing symbol.
- The headline counts each minted criterion that carries its own trace tag.

Each of the five criteria verifies through TC-017, and TC-017 was already backed on main.
None of the five was ever in the footer, so the footer stays at 40. Nothing regressed.

### Mutants

Each mutant was applied to the model, the tests were run, and the original source was
restored.

| Mutant | Result |
| --- | --- |
| M1: artifact rows reversed | killed by the new FR-017-AC-3/AC-4 test and the TC-017 coverage test |
| M1b: requirement rows reversed | killed |
| M3a: first trace wins | killed by the new AC-4 test |
| M3b: last trace wins (deep-before-shallow lost) | killed only by the new AC-4 test |
| M4a: one duplicate diagnostic per ID | killed only by the new AC-6 test |
| M4b: diagnostic on the first occurrence too | killed |
| M5: duplicate diagnostics appended after reference errors | killed only by the new AC-6 test |
| M6a: reference errors carry the source span | killed |
| M6b: stale-revision diagnostic drops its span | killed by the new NFR-003-AC-3 loop |
| M6c: duplicate carries the target span | killed |
| M7a: standalone clause digest ignores the clause body | killed by the new FR-016-AC-2 clause inequality |
| M7c: clause digest depends on every clause in the package | killed only by the new unrelated-clause equality (line 158) |
| M8: package requirements left unsorted | killed |
| M2: rows sorted case-insensitively | survived the full `cargo test --workspace` |
| M5b: diagnostics sorted by span instead of authored order | survived the full `cargo test --workspace` |

One more mutant was weaker than intended. M7b (a clause digest that also depends on its
sibling clauses) survives the PR's tests, because every fixture requirement has one
clause. It is killed elsewhere by the FR-016-AC-6 written-out bytes, and FR-016-AC-2's
"unrelated clause" is asserted on another requirement's clause, so it is not raised as a
finding.

No production code changed, so there is no reverse gap and no stub. The semantic review
was carried out as the mutant pass above.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-017-AC-6 requires that coverage diagnostics "retain authored trace order". In both tagged fixtures, authored order matches ascending span order: every later trace sits at a higher offset in one source document. An implementation that sorts diagnostics by span (mutant M5b) passes the whole workspace suite. The authored-order clause is therefore not backed independently of source position. Fix: author at least one reference-error or duplicate trace whose span sorts before an earlier-authored trace, for example a lower offset or another document, and keep the exact expected order | tests/it/canonicalization.rs:922-984 |
| FND-002 | medium | FR-017-AC-3 requires requirement and artifact rows "in Unicode scalar value order". The new test's IDs are REQ_a/REQ_z and art_a/art_z, all lowercase and differing in one letter. Scalar order, case-insensitive order and length-then-lexicographic order cannot be told apart. A case-folded sort (mutant M2) passes the whole workspace suite. The ID grammar allows uppercase ASCII. Fix: include IDs whose scalar order differs from folded order, for example `REQ_B` before `REQ_a` and `art_Z` before `art_a` | tests/it/canonicalization.rs:845-920 |

## Verdict

CONDITIONAL. FR-016-AC-2, FR-017-AC-4 and NFR-003-AC-3 are fully asserted by their tagged
tests:

- FR-016-AC-2: the clause, requirement and package digests change, the unrelated
  requirement and its clause stay stable, and repeated and reversed bytes are identical
  against an exact golden.
- FR-017-AC-4: both trace orders are checked.
- NFR-003-AC-3: all four span kinds are checked, with source and target spans that
  differ.

FR-017-AC-6's counting clause is well backed: three occurrences of duplicate_a and two of
duplicate_b, interleaved with cross-package and missing errors. Its authored-order clause
is not distinguishable from span order (FND-001). FR-017-AC-3's scalar-order clause is not
distinguishable from case-folded order (FND-002). Both gaps are fixed by small changes to
the fixtures.

## Coverage

- Plan completion: not assessed
- Criteria examined: FR-016-AC-2, FR-017-AC-3, FR-017-AC-4, FR-017-AC-6, NFR-003-AC-3
  (all tagged).
- Strict coverage: 541/607 backed (main 536/607), 0 contradicted, footer 40 to 40 with an
  identical set.
- Semantic review: carried out as a mutant pass (15 mutants, 2 survived).

## Dispositions

- FND-001: fixed in round 1. The earlier-authored cross-package trace has a later source span than the following duplicate and missing-reference traces. The span-sorted diagnostic mutant fails the authored-order assertion.
- FND-002: fixed in round 1. `REQ_B` and `art_Z` precede `REQ_a` and `art_a` in scalar order, while case folding reverses both pairs. The case-folded row mutant fails the requirement assertion; an artifact-only case-folded mutant fails the artifact assertion.
