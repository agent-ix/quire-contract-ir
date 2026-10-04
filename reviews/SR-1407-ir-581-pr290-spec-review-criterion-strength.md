---
id: SR-1407
title: "criterion strength review of PR 290 (FR-043-AC-1 to AC-4)"
type: SpecReview
analysis: criterion-strength
scope: "agent-ix/quire-contract-ir@0c24bb3bd1059fcf390657a17086e5b9c72a9e9e; spec/core/functional/FR-043-one-lock-entry-per-crate.md FR-043-AC-1 to FR-043-AC-4; spec/core/matrix/tests.md TC-441"
review_set: subset
---
# SR-1407: criterion strength review of PR 290

## Summary

Ticket: IR-581. Reviewer judgement (no Jev client exists), with a fixture: for each AC,
whether a wrong implementation could pass it.

- AC-2 is strong for its case and testable as claimed: a local-git workspace with two
  tags of one crate at 0.1.0 gives a two-entry lock that cargo-deny fails, naming the
  crate. Its case is narrow (below).
- AC-3 is strong: a fixture and the repository's own lock must both pass, so an
  over-strict check (or missing `skip` entries) fails it.
- AC-1 and AC-4 are weaker (below). All four are honestly marked PLANNED (IR-581), the
  matrix rows say planned, and strict coverage shows them unbacked.
- The ACs are tool-neutral as claimed, except AC-4's parenthetical, which is a harmless
  example.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-043-AC-2 tests only one first-party crate at one version from two revisions. A replacement tool (O-4 options b or c) that compares revisions alone passes AC-2 yet misses two versions of one first-party crate, or one git entry plus one registry entry of the same name, both of which the awk catches today | spec/core/functional/FR-043-one-lock-entry-per-crate.md:50 |
| FND-002 | medium | FR-043-AC-4 requires each exception to name a third-party crate with a reason, but not that it still matches a lock entry. A stale `skip` is a warning and exit 0 under cargo-deny (measured), so AC-4 passes while skip lists drift; add a clause that an exception matching no duplicate fails (cargo-deny: `-D unmatched-skip`, measured exit 2) | spec/core/functional/FR-043-one-lock-entry-per-crate.md:52 |
| FND-003 | low | FR-043-AC-1's "holds no file whose job is the check" is a judgement; its only concrete clause is the name `scripts/check_one_copy.awk`, so a renamed copy passes the concrete part | spec/core/functional/FR-043-one-lock-entry-per-crate.md:49 |

## Verdict

AC-2 and AC-3 can fail. AC-2's adverse coverage and AC-4's drift gap should be closed in
the fix round.

## New findings (disposition pass 1)

Reviewed at agent-ix/quire-contract-ir@adb9fc2eac014d05d569226dd24659d93f895017.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | medium | FR-043-AC-4 says an exception "names a crate whose lock source is not under the agent-ix organisation" instead of using the description's defined term "first-party crate", which includes a registry entry whose name an agent-ix entry also carries. For AC-2 case (c), a name with one git entry and one registry entry, it is ambiguous whether a `skip` on that name is allowed. The AC also holds trivially for a tool with no exceptions (option c) and should say so | spec/core/functional/FR-043-one-lock-entry-per-crate.md:63 |

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 674867b |
| FND-002 | fixed | 674867b |
| FND-003 | fixed | 674867b |

## Dispositions (round 2)

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-004 | fixed | 49245d4 |
