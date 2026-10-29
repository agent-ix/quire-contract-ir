---
id: SR-17014
title: "IR-347 PR 557 spec integrity analysis"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir#557 vs origin/main; spec/assurance/AD-006-codegen-consumption-seam.md, spec/kani/functional/FR-031-bounded-kani-dispatch-and-terminal-map.md, spec/kani/matrix/TC-055-root-crate-public-interface.md, spec/kani/matrix/tests.md"
review_set: subset
---
# SR-17014: IR-347 PR 557 spec integrity analysis

## Summary

Ticket: IR-347. Method: `spec-review` with the `spec-integrity-analysis` sub-analysis. It checks
cross-reference and status consistency for the four changed spec files. Read for context: FR-039,
TC-041 (`spec/model/matrix/tests.md`), FR-044, STD-001, AD-005, FR-029 and FR-030. Codegen
origin/main was read to check claims about codegen: `src/kani/generate/lower/` and
AD-004 at lines 241 and 315. `make spec` validation reports no structural error in the touched
files.

Units examined:

- AD-006 "What crosses the seam" row "Kani family lowerings", the identity-and-codes bullets, the
  G-1 and G-2 goals and the status list.
- FR-031 `## Status`.
- The TC-055 description, procedure and status.
- The tests.md FR-031, FR-037, FR-039, TC-042 and TC-055 rows.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-039-AC-4 and TC-055's procedure require running "the TC-041 and TC-042 negative corpora under `catch_unwind`". TC-041 is the Cargo dependency-graph and model-boundary case (`spec/model/matrix/tests.md:32`, `:43`; `tests/it/cycle_free_model.rs`) and has no corpus of inputs to any `kani` function. The reference is stale. The coder ran TC-042-style cases and left the TC-041 citation in both artifacts while marking them ✅. Correct the citation to TC-042, which removes no coverage because TC-041 exercises no `kani` function, or name the corpus AC-4 actually means. Do not narrow "every public `kani` function over untrusted input" | spec/kani/functional/FR-039-root-crate-public-interface.md:116, spec/kani/matrix/TC-055-root-crate-public-interface.md:34 |
| FND-002 | medium | New prose says all three family lowerings "now live in codegen". Codegen does not have them all. On codegen main, `definedness_arithmetic.rs` was retired by CG FR-015-AC-109 (CG AD-004:241,:315), and there are no `lower_checked_arithmetic`, `lower_query` or `lower_reaches` functions; the collection and graph paths are `prepare_bounded_collection_query` and `prepare_finite_graph_reaches`, pending FR-015-AC-110 and AC-111. AD-006:58 names `definedness_arithmetic.rs` as a codegen owner of a lowering path, and that file does not exist. Rewrite the prose to say: IR no longer has them; codegen retired arithmetic (FR-015-AC-109) and holds the collection and graph lowerers. Do not restore arithmetic | spec/assurance/AD-006-codegen-consumption-seam.md:58, spec/assurance/AD-006-codegen-consumption-seam.md:87, spec/kani/functional/FR-031-bounded-kani-dispatch-and-terminal-map.md:107, spec/kani/matrix/tests.md:14 |
| FND-003 | medium | The PR leaves prose that the deletion makes false. FR-044:151-156 ("The family lowerings still in this repository (`src/kani/arithmetic.rs`…) … until IR-347 removes them") and FR-044:184-185 ("the family lowerings build their unregistered codes … (IR-347 removes them)"). AD-005:218-220 (the lowerings "stay in this repository and are IR-347 scope"). FR-029:51 ("none of the three family lowerings converts a profile code"). AD-006:57 (lists the retired CG `definedness_arithmetic.rs`). `spec/core/matrix/tests.md:12` (the StR-001 row still lists "the root and model crate interfaces with no root re-export of the model" as planned, while FR-039 and TC-055 are now ✅) | spec/core/functional/FR-044-typed-std001-code.md:151, spec/core/functional/FR-044-typed-std001-code.md:184, spec/assurance/AD-005-qsl-consumption-seam.md:218, spec/kani/functional/FR-029-versioned-bounded-kani-profile.md:51, spec/core/matrix/tests.md:12 |

## Verdict

**PASS WITH FINDINGS.** The ownership direction is right: the lowerings are absent from IR, and
FR-039 "Items codegen owns" matches the code. The status edits overstate in two directions:

- They cite a TC-041 corpus that does not exist (FND-001).
- They say all three lowerings moved to codegen, when codegen retired arithmetic (FND-002).

Several unchanged files still describe the lowerings as present in IR (FND-003). All three
findings are prose corrections inside this PR. None requires a code change or an AC narrowing.

## Dispositions

Round 1. Checked against fix. A grep of `spec/` finds no remaining "still in this repository", "now live in codegen", "IR-347 removes them" or `definedness_arithmetic` claim.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | deferred | Operator decision e29 is pending. FR-039-AC-4 and TC-055's procedure still cite TC-041's negative corpus, unchanged and not narrowed. TC-041 still has no Kani negative corpus, so the stale reference remains. The fix round does not hide this. TC-055 Status, the kani matrix FR-039 and TC-055 rows, the core matrix StR-001 row and AD-005 D-3 all now say AC-4 is partial and the reference "awaits owner resolution". The citation stays, and real TC-042 negatives are bound under `catch_unwind`. The coder cannot resolve this; it needs an owner ruling on the TC-041 citation. It blocks closing FR-039-AC-4, TC-055 and IR-347, and it is not waived |
| FND-002 | fixed | AD-006 (lines 38, 58, 87, 117, 159, 189), FR-031 (lines 38, 58, 107), the kani tests.md rows FR-031 and TC-042, and spec.md now say: IR exports no family lowering; codegen retains the collection and graph lowerers; arithmetic was retired under CG FR-015-AC-109. AD-006:58 no longer names `definedness_arithmetic.rs`. Nothing proposes restoring arithmetic |
| FND-003 | fixed | Updated: FR-044:151-155 and:182; AD-005:155,:173 and:219-220; FR-029:50-51; AD-006:57 (the `definedness_arithmetic.rs` consumer is dropped); core matrix StR-001 (the root inventory is backed by TC-055 and TC-058, and AC-4 is partial) |
