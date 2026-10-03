---
id: SR-816
title: "integrity review of PR 258 canonical encoding (IR-533)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@f3606b05780f22897c15fccc0eb2756ab284f14e; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/tests.md, spec/model/functional/FR-019-rust-library-interface.md"
review_set: subset
---
# SR-816: integrity review of PR 258 canonical encoding

## Summary

Ticket: IR-533. Checked the IDs, traces and consistency of the six new ACs. Each is traced
from TC-048 and from the FR-038 and TC-048 rows of tests.md. The TC-048 procedure section
covers every clause. FR-038's Dependencies names quire-canonical as the encoder's owner.
The FR-019 paragraph defers to FR-038 for the assignment. `quire validate` passes.

These checks measured the open PRs against this head with
`git merge-tree <other> f3606b05780f22897c15fccc0eb2756ab284f14e`:

- #257 (IR-450, head 10a17ec6b705ec88b5442a3367b826043d76ed2b) gives a conflict in FR-038
  and in tests.md.
- #250 (IR-505) and #253 (IR-530) each give a conflict in tests.md.
- #243 (IR-274, held) gives no textual conflict in these files, but it overlaps in meaning
  (SR-815 FND-001).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC numbering collides. Open spec PR #257 (IR-450) also defines FR-038-AC-73, the `unit`/`compound_unit` position rule. #257 was opened first (2026-10-03T02:08:46Z, against 02:10:01Z for #258). Both PRs cannot merge with their numbering as written, and the textual conflict in FR-038's AC table hides that the two are different criteria with the same id. Recommendation: whichever merges second renumbers. Merge #257 first, and renumber #258's criteria to AC-74 to AC-79 in FR-038, TC-048 (procedure, description and trace list) and both tests.md rows. The leader decides the order. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1236-1241 |
| FND-002 | low | The FR-038 and TC-048 rows of tests.md are one physical line each. #250, #253 and #257 all edit the same lines, so each merge after the first needs a hand merge. IR-535's code PR will also touch these rows. Each PR's planned-AC prose is a separate clause, so they are compatible. Merge them by hand, keep every PR's clause, and do not take one side wholesale. | spec/checked_package/matrix/tests.md:15,23 |

## Verdict

The criteria and traces are consistent within the PR. Resolve FND-001 by merge order and
renumbering before merge. FND-002 is merge mechanics.

## Dispositions

Round 1 at 2f491e0a30bd3b29b9ae6f9abf81f28ba09bac81, rebased on main
7ed352beb994355413b9d7c572e45667aee229ba with #257 merged. FR-038 has 78 AC rows and no
duplicate id. AC-73 is #257's quantity rule, traced from TC-050. The new criteria are
AC-74 to AC-80 in FR-038, TC-048 (description, procedure and traces), the Dependencies
paragraph and both tests.md rows. The tests.md FR-038 row keeps main's status text
verbatim, #257's AC-73 clause included, and adds the IR-533 clause in front of it. The
TC-048 row does the same.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | fixed 2f491e0a30bd3b29b9ae6f9abf81f28ba09bac81 |
| FND-002 | fixed | fixed 2f491e0a30bd3b29b9ae6f9abf81f28ba09bac81 |
