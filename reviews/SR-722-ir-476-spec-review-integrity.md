---
id: SR-722
title: "spec-review integrity (trace honesty) of PR 247 (IR-476)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@f512aac4330ad47cf474d23af9f2407a32c45f19; tests/it/executable_binding.rs, crates/quire-contract-model/src/checked_package/v2/operations.rs, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, PR body"
review_set: subset
---
# SR-722: spec-review integrity (trace honesty) of PR 247

## Summary

Ticket: IR-476. Sub-analysis: integrity, limited to trace and coverage honesty. No `spec/**`
file changes. TC-048 ("CheckedPackage V2 strict reader re-derives package and nominal
identities") owns these four tests plausibly at the TC level. FR-038-AC-35 covers
`stale-node-key` for a `dependency_reference` callee, and TC-048's procedure (line 164)
re-derives an application key. No AC names application-key re-derivation in general, and the
PR invents none, which is correct. The TC-035 to FR-023-AC-n pairings in executable_binding.rs
are unchanged and agree with the FR-023 matrix rows.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The PR body misstates the measurement. It says "The bare doc lines are not read as tags (binding came from the tc_035_ test names)". The module declares `rust-doc-comment-id` (`^\s*///\s*<id>` with a trailing delimiter), a legacy form that matches `/// TC-035` and `/// FR-023-AC-1`. The real justification for the rewrite is that the form is legacy with `rewrite_to: rust-trace-attribute`, not that it was unread. The body also cites short SHAs (4233b56, f512aac), which the brief excludes | PR #247 body |

## Verdict

Trace pairings are honest. The only integrity problem in the code is the inert TC-048
annotation (SR-720 FND-001). Correct the body before merge.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | low | The PR body's Gate section describes the superseded run: "The run shared its log file with an earlier orphaned run, so the log footer was appended by hand." The current gate evidence is a clean, single re-run at f6cc2e9a7f9a387a073211e653fddfac5f4dd54c. Its footer was written by the run (`head=<full sha> exit=2`), with only `spec` failing. The body should describe that run, not the interleaved one | PR #247 body |
| FND-003 | low | The body's strict-count sentence ("the AC-43 and AC-44 rows were backed on the AC side by this PR, and the strict count line did not move") does not say why. The strict `unbacked_rows` list (23, identical at base and head) never contained AC-43/44. The coverage denominator went from 23 to 21 unbacked of 184. Both counts were 23 at base. The body should say these are two different lists, so a reader does not take the unchanged 23 as unbacked AC rows | PR #247 body |

## Dispositions

Round 1, reviewed agent-ix/quire-contract-ir@f6cc2e9a7f9a387a073211e653fddfac5f4dd54c. The
body now gives the right reason for the executable_binding.rs rewrite ("already read as
tags by the legacy `rust-doc-comment-id` form ... canonical-form hygiene with unchanged
bindings"), and the head short SHA is gone. One short SHA remains: "Main (4233b56)". No
spec file changes in the delta. The 24 new `#[trace]` bindings carry only AC ids that already
existed and were already cited, so nothing was invented.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | still-open | The misstatement is corrected in the PR body, but the body still cites the short SHA "4233b56" in "Main (4233b56)". Replace it with the full 4233b569a4723f1cf60c0103a3dc1f3e5fd9e798 or drop it. |

Round 2, reviewed agent-ix/quire-contract-ir@28855114846601a48962fbe4f52eb690dae7d34e. The delta
from f6cc2e9a7f9a387a073211e653fddfac5f4dd54c only adds the three SR files under `reviews/`
(byte-identical to the round-1 reviewer files, and `quire validate` on `reviews/**` passes).
The merge base is still origin/main 4233b569a4723f1cf60c0103a3dc1f3e5fd9e798. All three
fixes are edits to the PR body, which has no commit, so the sha recorded is the head at
which the body was verified.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 28855114846601a48962fbe4f52eb690dae7d34e (PR body edit: "Main:" with no SHA; no short hex SHA remains in the body) |
| FND-002 | fixed | 28855114846601a48962fbe4f52eb690dae7d34e (PR body edit: Gate section describes only the clean re-run at the full f6cc2e9 SHA, exit=2, only spec failing) |
| FND-003 | fixed | 28855114846601a48962fbe4f52eb690dae7d34e (PR body edit: the body names the coverage list, 23 -> 21 of 184, and the separate strict list, unchanged) |
