---
id: SR-992
title: "integrity review of PR 262 (IR-485 marker-bearing duplicate SR ids)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@b86735023d5fc3e5704bc5c090e90e20da3bb6e6; reviews/**/*.md frontmatter ids and SR- filename prefixes; the 15 renumbered reviews/ files (SR-977..SR-991) and the 8 keepers of SR-591, 592, 593, 624, 625, 626, 627, 628; citations of those ids across the whole repo; open PR branches 243, 250, 253, 259; compared against origin/main 9631d7e86976855f17b3e590a29604a122b60443"
review_set: subset
---
# SR-992: integrity review of PR 262

## Summary

Ticket: IR-485. The diff touches 15 files under `reviews/`, all records. No file is
renamed. Each file changes its frontmatter `id:`, its H1, and gains one
`Former id:` line. Five files also repoint citations to renumbered siblings.

Measured at head:

- Group sizes on origin/main: SR-591 3, SR-592 3, SR-593 2, SR-624 4, SR-625 4,
  SR-626 3, SR-627 2, SR-628 2. That is 23 files, not the 21 the ticket stated.
  15 are renumbered and 8 keep their id.
- Frontmatter SR ids across the whole repo: no duplicate. The only duplicated
  frontmatter id anywhere is PLAN-006 (plan/), which is pre-existing on main and
  outside IR-485.
- Every `SR-NNN-` file name prefix equals that file's id. Highest id is SR-991.
- SR-977..SR-991 collide with nothing. No frontmatter on any origin branch uses
  them except this PR's. No Linear comment in any repo mentions SR-977..SR-999.
  The open PRs add SR-790/791 (#250) and SR-795/796 (#253), which are free at
  head. #243 and #259 add no review files.
- Citation repoints: ir-307-gap-analysis (SR-624 to SR-982), ir-370-gap-analysis
  (SR-624 to SR-984, twice), ir-370-spec-review-integrity (SR-624 to SR-984,
  SR-625 to SR-986), ir-448-gap-analysis (SR-625 to SR-987, twice; SR-628
  to SR-991) and ir-448-spec-review-base (SR-628 to SR-991, twice). Each was
  checked against the cited finding's text, and each points at the right
  sibling. Examples: the IR-307 code review's FND-001 is the non-reference
  `same_type` gap. The IR-370 code review's FND-001 is the direction defect.
  The IR-448 integrity review holds both the ADR-0056 TC-058 finding and the
  cross-repo links.
- Remaining citations of the 8 kept ids all mean the keeper. In ir-89-gap-analysis
  and ir-89-spec-review-spec-integrity-analysis, SR-591 FND-001 is the IR-89 code
  review's `record_value_type` finding. The IR-279 trio cite each other's
  FND-001/002/004/006/007/011, and those findings exist in the keepers. In
  2026-09-30-ir-283-gap-analysis and SR-629, SR-627 FND-001 is the IR-283 code
  review's value-path masking finding. SR-641, SR-725 and SR-726 name the groups
  as history only. No sibling repo (codegen, runtime, quire-specification,
  quire-spec-language) cites these IR ids.
- `make spec` results are identical to origin/main: validate exits 0; grammar
  baseline 1 (FR-014 line 137, `ac:vague-response`); strict reports 23
  unbacked and 0 contradicted (`make spec` exits 1 on main as well);
  coverage 174/220. `quire validate 'reviews/**/*.md'` exits 0.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

Clean. After the PR every SpecReview id in reviews/ is unique. This holds with
the open PRs' additions too. No citation in the repo now points at the wrong
file.
