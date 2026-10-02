---
id: SR-726
title: "gap analysis of PR 248 against IR-485 (duplicate SR ids)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@bf72c142b6221889d73d729b5ac4a5bc8ca40ac2; reviews/**/*.md; Makefile spec target; IR-485 needs (rule decision, renumber or rename, validator or lint); compared against origin/main 4233b56"
review_set: subset
---
# SR-726: gap analysis of PR 248 against IR-485

## Summary

Ticket: IR-485. The PR is records-only, so the gap analysis checks the ticket's needs
against the head rather than requirement-to-test coverage. Spec coverage is unchanged:
`make spec` gives byte-identical output at origin/main 4233b56 and head bf72c14 (161/184
rows backed, the same 23 unbacked rows, 1 warning, exit 2 at both). No requirement, matrix
row or test is touched.

The ticket asks for three things (untrusted ticket text, re-measured here):

1. A rule. The PR applies the rule that merged in quire-contract-codegen#225 (IR-487):
   renumber only files with no posted Linear marker, keep dated files that other
   artifacts cite, and rename SR-prefixed files so the prefix equals the id. Done, for the
   seven groups this rule covers.
2. Renumber or rename. Done for SR-044, 045, 046, 061, 062, 546 and 547 (to SR-970..976).
   Not done for SR-591, 592, 593, 624, 625, 626, 627 and 628 (21 files). Every file in
   those groups has a posted reviewer marker, so renumbering would orphan the marker's `id=`.
3. A validator or lint. Not done. quire 0.33.0 `validate` passes over `reviews/**/*.md` at
   head with 8 duplicated groups present.

PR hygiene: the title carries no ticket id, the body opens "Part of IR-485" and has no
Closes line, and it lists the blocked groups with their PRs. The PARTIAL outcome is stated
plainly.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | IR-485 stays incomplete after this PR: SR-591..593 and SR-624..628 (21 files) still share ids. All of them carry posted markers, so the next step needs an owner or planner decision, not a coder. | reviews/ir-89-code-review.md:2 |
| FND-002 | low | Nothing detects a duplicate SpecReview id. `quire validate` passes with 8 duplicated groups present, so the class can recur as soon as two reviewers allocate in parallel. | Makefile:62 |

## Verdict

The PR delivers the marker-free half of IR-485 correctly and claims nothing more. It is
mergeable as a "Part of" PR, and IR-485 stays open.

Recommendation for the marker-bearing groups (owner or planner decision, not a blocker):
accept them and key them as (repo, Linear ticket, id) rather than renumbering and
reposting. Within each group every file's markers sit on a different Linear ticket, with a
distinct `pr=` and `reviewed=`. Each file name also carries its ticket (`ir-314-…`,
`SR-624-ir-279-…`, `2026-09-30-ir-283-…`), so the triple resolves to exactly one file. The
reviewer skill already tells miners to key on (repo, file path), not on the id alone. Its
markers carry no path, but the ticket gives the same disambiguation. Renumbering would
mean editing or reposting more than 60 posted marker and dispositions comments, which
rewrites the provenance the markers exist to preserve. Record the accepted groups once,
for example in a closing note on IR-485. Then close FND-002 in a follow-up: add a
duplicate-id check that fails on new collisions and allow-lists these 8 groups.
Separately, the reviewer skill's step 3 says a parallel duplicate is something "`quire
validate` catches". That claim is false for quire 0.33.0 and belongs in that skill's own
ticket.

## Dispositions

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | deferred | Second PR under IR-485. A planner decision recorded on IR-485 (comment 3e27b527, relayed by luna:IR:plan) assigns this PR the 7 marker-free groups. It sends the 8 marker-bearing groups (SR-591, 592, 593, 624-628; 21 files) to a second PR, to be renumbered with a former-id line added to each renamed file. The PR body opens "Part of IR-485" and has no Closes line, so IR-485 stays open. Re-measured on a trial merge with origin/main 70314b7: exactly those 8 groups remain duplicated. |
| FND-002 | accepted-no-change | The same IR-485 planner decision says "No new tracking machinery", which rules out a duplicate-id validator or lint. The gap is real: quire 0.33.0 `validate` still passes over `reviews/**/*.md` with the 8 duplicated groups present. Uniqueness depends on the second PR and on reviewers allocating ids carefully. |
