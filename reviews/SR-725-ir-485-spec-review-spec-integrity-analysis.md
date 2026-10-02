---
id: SR-725
title: "integrity review of PR 248 (IR-485 marker-free duplicate SR ids)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@bf72c142b6221889d73d729b5ac4a5bc8ca40ac2; reviews/**/*.md frontmatter ids and SR- filename prefixes; reviews/SR-970..SR-975 (renamed from SR-044/045/046/061/062/547), reviews/2026-09-15-plan-009-complete-v1-backend-gap-analysis.md (SR-546 -> SR-976); citations in spec/, plan/, reviews/, README.md, CLAUDE.md, AGENTS.md, Makefile, scripts/, tests/; compared against origin/main 4233b56"
review_set: subset
---
# SR-725: integrity review of PR 248

## Summary

Ticket: IR-485. Reviewed head bf72c14 against origin/main 4233b56. The diff is seven
records-only files under `reviews/`: six renames (similarity 93-98%) and one in-place edit.
Every change is the frontmatter `id:` line plus, where the H1 carried the id, the H1. No
other byte changed.

Ids recomputed from every frontmatter in `reviews/` (181 files):

- origin/main: 159 distinct ids, 15 duplicated groups.
- head: 166 distinct ids, 8 duplicated groups: SR-591, SR-592, SR-593, SR-624, SR-625,
  SR-626, SR-627, SR-628 (21 files). These are exactly the groups the PR body lists as
  blocked. The seven groups SR-044, 045, 046, 061, 062, 546 and 547 no longer collide.
- Every `SR-NNN-` filename prefix equals its frontmatter id (94 SR-prefixed files, 0
  mismatches). No file lacks an id.

Provenance, pulled independently from Linear (2028 reviewer-marker comments, all repos):

- 167 marker comments carry `repo=agent-ix/quire-contract-ir`, 68 distinct ids, SR-584 to
  SR-702. None carries SR-044, 045, 046, 061, 062, 546 or 547. The hits for SR-044/045/046
  and SR-061/062 in other repos (ix-relay, quire-contract-codegen, tl-mltl) are other
  repos' ids and do not bear on IR.
- No marker in any repo carries SR-970..SR-979. No origin branch other than this PR's
  declares SR-97x in `reviews/`.
- Each blocked group's files each have a posted review marker (distinct `pr=` and
  `reviewed=`, each on a distinct Linear ticket), so the PR body's reason for leaving them
  is accurate.

Citations: PLAN-008 `log.md:10-12` and `TASK-027:48` cite SR-546/SR-547 as the Rust review
and gap analysis, which are `2026-09-15-output-mapping-foundation-{code-rust,gaps}.md`; both
keep their ids. `2026-09-14-native-predicate-gap-analysis.md:36,55` cites SR-061 (the
native-predicate code review, kept). `2026-09-15-output-mapping-foundation-gaps.md:32,52`
cites SR-546 (kept). No file cites SR-546 meaning the PLAN-009 gap analysis, and no file
cites any of the six old SR-prefixed file names, apart from SR-641 (FND-001).

`make spec` at both shas: byte-identical output, exit 2 at both (strict coverage
161/184, 23 unbacked rows, 1 grammar warning on FR-014 line 137). Validation passes.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | SR-641 FND-002's Refs cell cites `reviews/SR-044-native-predicate-bridge-base.md:2`, a path that no longer exists at head. The PR body says none of the old SR-prefixed file names is cited by path anywhere; it is cited once, here. | reviews/SR-641-ir-320-spec-review-base.md:25 |

## Verdict

Integrity holds. The renumbering removes exactly the seven marker-free collisions, keeps
every cited id on its cited file, and allocates ids nothing else uses. FND-001 is a
historical reference in a posted review artifact. Leaving SR-641 as written is right,
since it records what was true at its reviewed sha. Only the PR body's claim needs
correcting in the squash text.

## Dispositions

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | accepted-no-change | Round 1 at bf72c14 (no new commit). The false claim was in the PR body, not the repo. The body now reads "One old SR-prefixed file name is still cited by path: reviews/SR-641-ir-320-spec-review-base.md FND-002 cites reviews/SR-044-native-predicate-bridge-base.md as history. SR-641 is a posted review and is left as written." SR-641 stays unedited because it records what was true at its reviewed sha. Re-measured on a trial merge with origin/main 70314b7: that Refs cell is the only reference to any of the six old file names. |
