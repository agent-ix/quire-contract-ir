---
id: SR-993
title: "record-integrity review of PR 262 (keeper choice and former-id bridges)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@b86735023d5fc3e5704bc5c090e90e20da3bb6e6; the 15 renumbered reviews/ files and their Former id lines; Linear reviewer and reviewer-dispositions markers carrying id=SR-591..593 and SR-624..628; PR 262 title and body"
review_set: subset
---
# SR-993: record-integrity review of PR 262

## Summary

Ticket: IR-485. This review checks three things. First, the file each Linear
marker names. Second, the keeper rule. Third, the bridge line in each
renumbered file.

Marker ownership (`<!-- reviewer ... id=SR-NNN method=... pr=... -->` on
Linear, repo=agent-ix/quire-contract-ir):

- SR-591: IR-89 code-review #205 (keeps); IR-313 spec-integrity-analysis #204 to SR-977; IR-314 base #203 to SR-978.
- SR-592: IR-89 gap-analysis #205 (keeps); IR-313 base #204 to SR-979; IR-314 integrity #203 to SR-980.
- SR-593: IR-89 spec-integrity-analysis #205 (keeps); IR-314 ears-conformance #203 to SR-981.
- SR-624: IR-279 code-review #186 (keeps); IR-307 code-review #230 to SR-982; IR-411 code-review #228 to SR-983; IR-370 code-review #229 to SR-984.
- SR-625: IR-279 gap-analysis #186 (keeps); IR-307 gap-analysis #230 to SR-985; IR-370 gap-analysis #229 to SR-986; IR-448 code-review #232 to SR-987.
- SR-626: IR-279 integrity #186 (keeps); IR-370 integrity #229 to SR-988; IR-448 gap-analysis #232 to SR-989.
- SR-627: IR-283 code-review #231 (keeps); IR-448 spec-review/base #232 to SR-990.
- SR-628: IR-283 gap-analysis #231 (keeps); IR-448 spec-review/integrity #232 to SR-991.

Each marker's method and PR match the file it is assigned to. Exactly one file
keeps each id. All 15 `Former id: SR-NNN (cited by the marker on IR-XXX).` lines
name the correct old id and the correct ticket. All 15 renumbered tickets do
carry a marker with the old id, so the planner's condition for the line holds in
every case.

Other markers with the same ids belong to other repos: filament-ide-rs on
AGE-2092 and AGE-2104, quire-contract-runtime on IR-322 and IR-323, and
quire-contract-codegen on IR-92, IR-93 and IR-433. Their `repo=` field puts them
in a separate namespace, and this PR does not affect them.

Keeper rule: an SR-prefixed file name keeps its id; otherwise the oldest
ticket keeps it. By ticket creation the rule is accurate. IR-89 (2026-09-17) is
older than IR-313 and IR-314 (2026-09-29). IR-283 (09-25) is older than IR-448
(09-30). IR-279 (09-24) is older than IR-307, IR-370, IR-411 and IR-448. By
commit order, the renumbered IR-313/314 files and the IR-448 files reached main
before their keepers did. Under the planner ruling, any single keeper per group
is valid because ticket comments are history and the former-id line is the
bridge. The rule is sound. A grep for an old id also finds the bridge lines,
so a reader who starts from a stale marker reaches the right file.

PR body: "Closes IR-485" is justified. #248 cleared the 7 marker-free groups,
and this PR clears the last 8. The ticket's "make the validator or a lint catch
it" was set aside by the planner's "no new tracking machinery" (SR-726 FND-002,
accepted-no-change). The title carries no bare ticket id. The body carries no
short SHA.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The PR body sentence "The ticket treated as read less is the later, narrower one in each group." is garbled. It will land in the squash text unless edited, for example to "The later, narrower ticket in each group is renumbered." | PR 262 body, Rule section bullet 3 |

## Verdict

Approve. The record bridges are complete and correct, and the keeper choice is
justified. FND-001 is a wording nit in the PR body and does not block the merge.

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | PR body edit, head unchanged b86735023d5fc3e5704bc5c090e90e20da3bb6e6; the body now reads "The later, narrower ticket in each group is renumbered." and states the 23-file count (15 renumbered, 8 keep) |
