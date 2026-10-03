---
id: SR-1012
title: "spec review of PR 265 (inexact-integer and inexact-number causes for a selected model document)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@8edd1e5e1f660fe5e493c909a9a2113d09b33a6e; git diff origin/main...HEAD (base ebea678): FR-038 (step prose, the new canonical-encoding paragraph, FR-038-AC-96 to AC-98, Dependencies), TC-048, spec/checked_package/matrix/tests.md, spec/tests.md; cross-checked against quire-specification FR-271 and FR-272 at origin/main (merged b1da9c8 is an ancestor)"
review_set: subset
---
# SR-1012: spec review of PR 265

## Summary

Ticket: IR-542. This review checks the new criteria's form, their numbering,
and how they cite the QSpec rule. It also checks every example in them against
binary64 arithmetic.

- FR-038-AC-96, AC-97 and AC-98 come straight after AC-95, which was the last
  criterion on main. They are written as direct assertions, with no "shall".
  Their id cells hold only the id. The FR-038 matrix row and TC-048 mark them
  planned (🚧, IR-542). FR-038 has no mutation rows, so none were needed.
- The cause names match FR-272's `noncanonical_wire` row exactly:
  `inexact-integer` and `inexact-number`, carried with `document_pointer`. The
  rule that a number matching both reports `inexact-integer` is consistent
  with FR-272. FR-272 defines `inexact-number` as "any other number", so
  `inexact-integer` takes precedence by definition. AC-98 matches FR-272's
  statement that the reader's byte-stream refusal carries neither a cause nor
  a `document_pointer`. QSL #617 is not cited, and no QSpec text is copied.
- Every listed example was checked with correctly rounded parsing and
  shortest round-trip text, and each one classifies as the criteria say.
  - `inexact-number`: 0.1000000000000000000001, 9007199254740993.5,
    -0.1000000000000000000001, 4.9e-324 (its nearest double prints as
    `5e-324`), 1e-400 (its nearest double is 0) and 9007199254740992.5 (its
    nearest double is 9007199254740992, and the text is not a whole number).
  - Admitted: 0.1, 0.5, 1.5, -0.25, 5e-324 and 2.5e-10.
  - `inexact-integer`: the five AC-93 numbers and 9007199254740993.0.
  - AC-96 uses FR-272's own test ("exact decimal value equals that of its
    double's shortest round-trip text"), not exact binary representability.
    That is correct: 0.1, 5e-324 and 2.5e-10 are not exactly representable,
    and they are admitted.
- `quire validate` passes, with 1 grammar finding (FR-014 line 137, the
  baseline). Strict coverage reports 23 unbacked rows and 0 contradicted.
  Both were run in a detached worktree at the PR head.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The prose and the matrix describe the refused set as "a number that a double cannot hold exactly", in the FR-038 step text, the new paragraph, the TC-048 description, the matrix row and the commit title. FR-272 and the PR's own criteria contradict that: 0.1, 5e-324 and 2.5e-10 cannot be held exactly yet are admitted (AC-96), and 1e20 is held exactly yet refused `inexact-integer` (AC-93, AC-97). Read literally, the step text would refuse 0.1. Use FR-271's wording ("a number whose exact value its RFC 8785 encoding loses") or simply cite FR-272 | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:229 |
| FND-002 | low | AC-97 says "`9007199254740992`, … are unchanged and admitted, as `9007199254740992.5` … refuses `inexact-number`". "as" reads as causal or comparative and joins two unrelated assertions. Use "while", or split them | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1600 |
| FND-003 | low | FR-038 now cites quire-specification FR-271 and FR-272 as the source of the causes, but its `relationships` has no `references` edge to either. The other QSpec FRs it builds on (FR-322, FR-201, FR-195, FR-340) all have one, so this dependency cannot be traced mechanically | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:5 |

## Verdict

Changes requested for FND-001. The criteria are correct and their examples
are sound. The prose that frames them describes the rule wrongly in five
places, and one of those places is the normative step text. FND-002 and
FND-003 are one-line fixes.

## Dispositions

Round 1, reviewed at agent-ix/quire-contract-ir@2f80c1ecb15272f49513f6db26a4632d9629b194, the delta 8edd1e5..2f80c1e only. `make spec` was run in a detached throwaway worktree: validate passes, 1 grammar finding (FR-014, the baseline), strict coverage 23 unbacked and 0 contradicted. `git diff --check` is clean.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 2f80c1e: "a number whose exact value its RFC 8785 encoding loses", in FR-038 l.233, the paragraph, TC-048 and the matrix row; the PR title is patched to match; no "cannot hold exactly" is left in spec/ |
| FND-002 | fixed | 2f80c1e: AC-97 now reads "…unchanged and admitted, while `9007199254740992.5` … refuses `inexact-number`" |
| FND-003 | fixed | 2f80c1e: `references` edges to ix://agent-ix/quire-specification/FR-271 and FR-272 added to the frontmatter |
