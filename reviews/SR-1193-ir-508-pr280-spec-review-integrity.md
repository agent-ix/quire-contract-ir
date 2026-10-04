---
id: SR-1193
title: "integrity review of PR 280 (ID hygiene, trace and matrix consistency)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@f5bb9052b113d34353396cf8821c60c01c8140ed; git diff origin/main...HEAD (base 1117eba64f329fad9f7324b0ec9d2149be66f07f): FR-345, TC-224, spec/checked_package/matrix/tests.md, spec/core/matrix/tests.md, spec/spec.md, spec/tests.md; IDs checked against spec/, plan/, reviews/ and the full git history (git log --all)"
review_set: base
---
# SR-1193: integrity review of PR 280

## Summary

Ticket: IR-508. This review checks ID hygiene, frontmatter relationships, and
whether the matrix, tests and spec.md rows agree with each other.

- No other file at HEAD declares `id: FR-345` or `id: TC-224`. But
  `reviews/ir-314-spec-review-*.md` (eight files) cite FR-345 and TC-224 as
  a different artifact, and git history shows both IDs were issued on `main`
  before (FND-001).
- FR-346 and TC-225 appear in no path and in no `id:` line anywhere in
  `git log --all`. They are the next free IDs.
- The frontmatter follows FR-040's pattern: `traces_to` StR-001, `depends_on`
  FR-038 (plus FR-040, which this step follows), and `references` to the QSpec
  FRs. TC-224 `verifies` FR-345.
- The FR-345 matrix row (AC-1 through AC-9 to TC-224), the TC-224 summary row
  (it lists AC-1..9) and the cases row agree with each other. So do the
  spec/tests.md checked-package row, the spec/core/matrix/tests.md StR-001 row
  (FR-344, FR-345 / TC-223, TC-224) and the spec.md sentence.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-345 and TC-224 are reissued IDs. PR #203 (IR-314, merged 2026-09-29, fc99987) issued FR-345 "check artifact IDs and relocation maps" and TC-224 on `main`. #209 (a17a667) deleted them ("Delete FR-345 and TC-224 with their TM-001 rows and index entries"). ADR-0056 "Identifiers" rule 2: "An ID is never renumbered by a move and never reissued after deletion." The author says the sequence "tops out at FR-344 and TC-223", which counts only the files at HEAD. `reviews/ir-314-spec-review-*.md` still cite FR-345-AC-1..6 and TC-224 with the old meaning, so the reused IDs clash in the repo's own review records. Renumber to FR-346 and TC-225 (verified unused in all history), including the AC IDs and every matrix, tests and spec.md reference | spec/checked_package/functional/FR-345-admit-abstraction-relation-body.md:2; spec/checked_package/matrix/TC-224-checked-package-v2-abstraction-relation-body.md:2; spec/decisions/ADR-0056-spec-layout-convention.md:133-136; reviews/ir-314-spec-review-evidence.md:27-34 |

## Verdict

One high finding: the reissued IDs. Renaming them is mechanical, and the
traces and matrices are otherwise consistent.

## Dispositions

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | f4fb763 (git mv to FR-346 and TC-225, AC IDs FR-346-AC-1..10. `git log --all -S` finds `FR-346`, `TC-225`, `id: FR-346` and `id: TC-225` only in f4fb763. No FR-345 or TC-224 reference is left in spec/ or plan/. The eight reviews/ir-314-* files are history and are untouched. The matrix, tests.md, spec.md and core StR-001 rows agree on FR-346/TC-225 and AC-1..10) |
