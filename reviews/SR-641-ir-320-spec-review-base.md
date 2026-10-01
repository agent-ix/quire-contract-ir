---
id: SR-641
title: "spec review (base, with rename and object review) of PR 236 (IR-320 subsystem restructure)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@92945cd2e400cdd7940e68db7aab21c3291b7395; spec/** (all 79 renames, 8 matrices, spec/spec.md, spec/tests.md), reviews/SR-034..SR-068, SR-541..SR-582 (30 moved), reviews/SR-633-ir-317-code-review.md"
review_set: subset
---
# SR-641: spec review (base) of PR 236

## Summary

Ticket: IR-320. This review covers the base checklist and a manual review of the renames and the diff. The spec-object-review lens is folded in here, because SpecReview has no object analysis value. Each check was run against origin/main efa5632 and against head 92945cd.

- Renames: commit 1 holds 79 renames at 100% similarity. That is 49 spec files plus 30 `spec/reviews/**` files moved to `reviews/SR-<id>-<slug>.md`. Each new file name matches that file's own frontmatter `id`. The commit also adds one new file, `reviews/SR-633-ir-317-code-review.md` (FND-003). Commit 2 adds the per-subsystem matrices, `spec/tests.md`, the registry and relative link fixes. Every link edit resolves.
- Ids: the frontmatter `id:` set under `spec/` is identical before and after. The only differences are the 30 SR ids that moved to `reviews/` and the new TM-003 through TM-009. No TM-003..TM-009 id was used before anywhere in the repo. Every `ix://` relationship target is id-based, so the move cannot break one. Every StR/FR/NFR/STD/TC/AD/ADR/AC id that disappeared from the `spec/` tree went with the moved review files: ADR-0055, FR-024, FR-201-AC-5, NFR-005, TC-036, TC-037, TC-046, TM-001 and SR-411 are mentioned only in their prose.
- `quire validate` on `spec/**` and `reviews/**` exits 0. `make spec` gives the same result before and after: 181 rows, 162 backed, "17 unbacked row(s) and 0 contradicted", and the same 17 ids. Warnings differ only by path. A relative-link check over every `.md` file finds 67 relative links at head, all resolving, and 0 broken on main.
- The `spec/spec.md` References list was cut to five entries. Every dropped entry was a link to a requirement or TC file plus that file's own title, so no information is lost. The only loss is navigation: `spec/spec.md` no longer links to any requirement file. The reworded TM-002 sentence is accurate, because TM-002 now covers checked_package only.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | TC-051's Test Procedure still names the deleted path `spec/contract/STD-003-output-mapping-refusal-registry.md`. The link fixes rewrote `](...)` links and two Rust comments, but missed this backtick path | spec/output_mapping/matrix/TC-051-output-mapping-refusal-registry.md:21 |
| FND-002 | low | 6 of the 30 moved reviews carry SR ids that other files already in `reviews/` also declare: SR-044, SR-045 and SR-046 (`2026-09-15-kani-{objects,collections,integration}-gap.md`), SR-061 and SR-062 (`2026-09-14-native-predicate-{code-review,gap-analysis}.md`) and SR-547 (`2026-09-15-output-mapping-foundation-gaps.md`). The collisions already existed on main, across the two trees. Now the file name asserts an id that is not unique, and `quire validate` does not catch it (ADR-0056 Tool support) | reviews/SR-044-native-predicate-bridge-base.md:2 |
| FND-003 | low | Commit 1 is described as "git mv only", but it also adds the 90-line `reviews/SR-633-ir-317-code-review.md`. Commit 2's message says commit 2 adds SR-633. The file is an IR-317 audit, outside the scope of this restructure, and ADR-0056 says a restructure PR is "git mv plus link fixes" | reviews/SR-633-ir-317-code-review.md:1 |

## Finding Detail

- FND-001: change the path to `spec/output_mapping/functional/STD-003-output-mapping-refusal-registry.md`. No test reads the file, so this is a prose fix only.
- FND-002: this does not block the PR, because the collisions existed before it. Open a follow-up ticket to renumber the later-dated colliding reviews to free SR ids (ADR-0056 Identifiers rule 5). The pre-existing duplicates SR-546, SR-591..593 and SR-624..628, among files already in `reviews/`, can go in the same ticket.
- FND-003: the SR-633 content matches the IR-317 Linear `reviewer` marker (`id=SR-633`, reviewed=8371caa) and validates. Accept as-is, or correct the commit 2 message in the squash text so the history is accurate.

## Verdict

The rename mechanics are sound. The id set is identical, the matrix rows are preserved (SR-642), links resolve and the gate is unchanged. Only low findings remain. On ADR-0056 rule 8 ("`spec/reviews/` does not exist"), moving the 30 reviews is justified, and the files are byte-identical.

## Dispositions

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 82d57ba |
| FND-002 | deferred | IR-485 filed; the collisions already existed on main and do not block #236 |
| FND-003 | accepted-no-change | The PR body says SR-633 is in commit 1 on purpose and that the commit 2 message is wrong. History is not rewritten, and the squash message can carry the correct text |
