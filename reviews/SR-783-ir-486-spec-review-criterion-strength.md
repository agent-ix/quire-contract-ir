---
id: SR-783
title: "criterion-strength review of PR 255 (IR-486)"
type: SpecReview
analysis: criterion-strength
scope: "agent-ix/quire-contract-ir@154fc38ef518b036ef68ca1bb25fd6911846750d; spec/checked_package/functional/FR-038-consume-checked-package-v2.md AC-43 amended, AC-69..71; spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md Recursive compared types; reader code (operations.rs LeafWalk) to judge which implementations each clause can fail; quire-spec-language origin/main FR-093-AC-11"
review_set: subset
---
# SR-783: criterion-strength review of PR 255 (IR-486)

## Summary

Ticket: IR-486. Each new criterion can fail against today's reader, which refuses every cycle
`operator-ineligible`. AC-69's admissions fail, and so do AC-71's 20000-record admission and its
work cases. AC-70's malformed-leaf clauses pin exact pointers, so an implementation that ignores
recursion leaves entirely fails them. AC-71's 256 KiB stack clause catches a recursive walk (see
SR-780). The amended AC-43 can fail in both directions: an option-only cycle must refuse, and a
record cycle must admit.

The gap is in discrimination. A predictable minimal implementation passes every AC-69 case and
is still wrong.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-69 cannot tell the specified walk from the minimal patch. That patch turns today's `on_stack` revisit into a count of 0 and keeps the global per-node memo. Every AC-69 case passes under it: Node, Option<Node>, A/B compared at A (B is used once), Two (Node's count really is context-free), and List. Yet it miscounts whenever a memoised node is reused under a different open set. Add a discriminating case: `X { t: Text; n?: Y }`, `Y { u: Text; x?: X }`, `Wrap { y: Y; x: X }`. Its text leaves are `[field:y, field:u]`, `[field:y, field:x, inner, field:t]`, `[field:x, field:t]` and `[field:x, field:n, inner, field:u]`. The minimal patch expects 3 and refuses `operation-law-mismatch` at the fourth. Also add the A/B pair compared at B after A in one package, so that a memo shared across operations is caught too. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:989 |
| FND-002 | low | QSL FR-093-AC-11's `Tree2 { label: Text[0, 8; binary-utf8]; kids: Sequence<Tree2>[0, 3]; }` (leaves `field:label`; `field:kids`, `inner`, `recursion:0`) is QSL's one corpus case whose reentry goes through a bounded collection (`collection_bounds` over `sequence`). No AC-69 case reenters through a collection or a bounded domain. Add Tree2 to AC-69. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:989 |
| FND-003 | low | The `n = 10` clause does not name its work limit. It returns `incomplete` for `work` under the default 1 000 000 (about 10^7 units by my estimate). Under a much larger limit, a correct reader instead finishes and refuses `operation-law-missing`. The 12-composite bisection names no shape. Fix: say "under the default read limits" for `n = 10`, and name the 12-composite type (for example a ring of 12 records each holding text). | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:991 |

## Verdict

Adequate for the main path. One medium discrimination gap and two low gaps.

## Dispositions

Round 1, reviewed at agent-ix/quire-contract-ir@2f54f96507fd81b6d1b428cbf1113ad4cd50f035. The PR was rebased onto origin/main 35c098f51a0e0e126a3d05ef9a3647aa3527df39, after #254 merged, and is one commit on it. Its diff against that base is the delta I reviewed; the rebase is excluded. GitHub reports MERGEABLE. `make spec` at round 1: validate passes, 1 grammar finding (baseline), 23 strict unbacked rows (baseline), 163/212 rows backed, FR-038 42/70 (main: 163/209, 42/67). The new ids are AC-70 to AC-72, and the only FR-038-AC-69 row is #254's. No AC id is duplicated. The diff and the PR body contain no SHA.

Notes on the round-1 checks: AC-70 adds `Wrap` over `X`/`Y`, with 4 text leaves and the law-missing and law-mismatch edges. It compares `A` and `B` in one package, so a memo shared across operations is caught. It adds `Tree2`, which reenters through a bounded `Sequence`, and the tuple `Pair`. AC-72 names "the default read limits" for the ten records and a ring of 12 text-holding records for the bisection. The minimal patch now fails AC-70.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 2f54f96507fd81b6d1b428cbf1113ad4cd50f035 |
| FND-002 | fixed | 2f54f96507fd81b6d1b428cbf1113ad4cd50f035 |
| FND-003 | fixed | 2f54f96507fd81b6d1b428cbf1113ad4cd50f035 |
