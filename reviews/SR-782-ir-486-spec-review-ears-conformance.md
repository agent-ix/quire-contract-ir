---
id: SR-782
title: "EARS and AC-shape conformance review of PR 255 (IR-486)"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-contract-ir@154fc38ef518b036ef68ca1bb25fd6911846750d; spec/checked_package/functional/FR-038-consume-checked-package-v2.md AC-43 amended, AC-69..71 and new prose; make spec grammar before/after"
review_set: subset
---
# SR-782: EARS and AC-shape conformance review of PR 255 (IR-486)

## Summary

Ticket: IR-486. In this repo, criteria are direct assertions of an observable outcome, not
`shall` statements. AC-69..71 and the amended AC-43 use no `shall` or obligation form. Each names
its inputs (concrete QSL-syntax types and leaf lists) and its outcome (admission, or code, cause and
pointer). `make spec` grammar: 299/300 docs grammar-clean, with 1 finding before and after (the
FR-014 `ac:vague-response` baseline). No new grammar finding. The criteria are compound, which
matches the FR-038 house style (AC-44 is the precedent).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | AC-70 opens with "Over `Node`," and lists five recursion-leaf cases under it. The fifth, "one at a reentry of a recursive record that reaches no `text` type", cannot be over `Node`, which reaches text. Fix: give that case its own subject (for example the integer `List` of AC-69) in a separate clause. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:990 |

## Verdict

Conforms, with one low scoping defect in AC-70.

## Dispositions

Round 1, reviewed at agent-ix/quire-contract-ir@2f54f96507fd81b6d1b428cbf1113ad4cd50f035. The PR was rebased onto origin/main 35c098f51a0e0e126a3d05ef9a3647aa3527df39, after #254 merged, and is one commit on it. Its diff against that base is the delta I reviewed; the rebase is excluded. GitHub reports MERGEABLE. `make spec` at round 1: validate passes, 1 grammar finding (baseline), 23 strict unbacked rows (baseline), 163/212 rows backed, FR-038 42/70 (main: 163/209, 42/67). The new ids are AC-70 to AC-72, and the only FR-038-AC-69 row is #254's. No AC id is duplicated. The diff and the PR body contain no SHA.

Notes on the round-1 checks: AC-71 gives the integer `List` case its own subject ("a recursion leaf at a reentry of an integer `List`, which reaches no `text` type").

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 2f54f96507fd81b6d1b428cbf1113ad4cd50f035 |

Round 2, reviewed at agent-ix/quire-contract-ir@7775e142e702e2a3e21a8426eca9356638838889: the delta from 2f54f96507fd81b6d1b428cbf1113ad4cd50f035, on base origin/main 35c098f51a0e0e126a3d05ef9a3647aa3527df39, which is unchanged. GitHub reports MERGEABLE. `make spec` at round 2: validate passes, 1 grammar finding (baseline), 23 strict unbacked rows (baseline), 163/212 rows backed, FR-038 42/70. The delta adds no SHA. The leader relayed a planner decision, recorded on IR-486, that the recursion leaf is now required wherever text is reachable. I have not confirmed it independently.

Round 2: nothing to disposition (FND-001 was fixed in round 1). One new low wording finding, below.

Round 3, reviewed at agent-ix/quire-contract-ir@4a0bcc7c336d47f945b83daab5932b784707d092: the delta from 7775e142e702e2a3e21a8426eca9356638838889, on base origin/main 35c098f51a0e0e126a3d05ef9a3647aa3527df39, which is unchanged. GitHub reports MERGEABLE. `make spec` at round 3: validate passes, 1 grammar finding (baseline), 23 strict unbacked rows (baseline), 163/212 rows backed, FR-038 42/70. The delta adds no SHA and touches only AC-70 (the typo), AC-72, one sentence of the iterative-walk paragraph and TC-048's ring step. I found no regression.

Notes on the round-3 checks: AC-70's doubled "and" is gone. FR-038 still contains the substring "and and", but only inside #254's "operand and an" (AC-69), which is not a doubled word.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-002 | fixed | 4a0bcc7c336d47f945b83daab5932b784707d092 |

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | low | AC-70 has a doubled word: "admit with `leaves` empty; and and the `Node` comparison with a second text leaf". Delete one "and". | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1162 |
