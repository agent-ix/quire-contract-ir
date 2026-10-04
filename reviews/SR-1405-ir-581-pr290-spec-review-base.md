---
id: SR-1405
title: "base spec review of PR 290 (AD-007 O-4 ruling, FR-043)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@0c24bb3bd1059fcf390657a17086e5b9c72a9e9e; spec/assurance/AD-005-qsl-consumption-seam.md, spec/assurance/AD-006-codegen-consumption-seam.md, spec/assurance/AD-007-cross-repo-dependency-graph.md, spec/core/functional/FR-043-one-lock-entry-per-crate.md, spec/core/matrix/tests.md, spec/tests.md, spec/spec.md"
review_set: subset
---
# SR-1405: base spec review of PR 290

## Summary

Ticket: IR-581. Scope is `git diff origin/main...HEAD` only. Measured independently with
cargo-deny 0.20.2 offline, a local-git fixture workspace, and each repository's own
`origin/main` lock.

Confirmed: the three `scripts/check_one_copy.awk` files are 9 lines and md5-identical on
IR, CG and RT `origin/main`; the script's agent-ix crate sets are 5 names (IR), 20 (CG), 1
(RT, `quire-exact`); a two-entry lock of one crate at one version from two git tags fails
`cargo deny check bans` (`found 2 duplicate entries for crate 'first'`) with the global
`multiple-versions = "deny"` and with a per-crate `deny-multiple-versions = true`; the glob
`first*` matches nothing and passes; global deny on IR's lock fails on exactly the 8 named
third-party crates and on CG's on 10. IR CI runs `cargo deny check`; CG and RT CI run
`cargo deny check licenses` only. FR-043 and TC-441 never appear in history before this
commit; FR-042 existed and was removed (#210), so skipping it is correct. `quire validate`
exits 0. Sorted `quire coverage --strict` main vs head differs only by the five new planned
rows (FR-043-AC-1..4, TC-441): 23 to 28 unbacked, 0 contradicted; honest. The added lines
carry no SHA, pin, line citation or quire-research content; O-1 to O-3 are untouched; no
"none named" or "no owner named" claim about the check survives.

The AD is honest that the tool is an open question: the owner table row, the O-4 heading
("Decided in part") and O-4a/O-4b ("not decided here") all say so. The defects are in the
case for option (a) and in FR-043's statement.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-043's statement requires one lock entry for every crate name, but AC-4 and the recommended design keep third-party duplicates (IR's lock holds 8 today, kept as `skip` entries); the SHALL contradicts its own exception rule and should say first-party (agent-ix-sourced) crates | spec/core/functional/FR-043-one-lock-entry-per-crate.md:17 |
| FND-002 | medium | O-4 calls the global form equivalent to and a superset of the script; it is not: cargo-deny judges the cfg-resolved graph, the awk the raw lock. A lock entry reachable only through a never-true cfg fails the awk and passes cargo-deny (fixture measured), and RT's lock holds two `syn` entries yet global deny passes with 0 errors, so RT needs 0 `skip` entries, not 1 | spec/assurance/AD-007-cross-repo-dependency-graph.md:305 |
| FND-003 | medium | The options table pre-answers O-4a ("Callers hold a copy: no") and omits option (a)'s main cost: the first-party guarantee rests on each repo's own policy, and FR-043-AC-4 (no `skip` names an agent-ix crate) needs a source-aware lock check that cargo-deny cannot express, so each of IR, CG and RT must carry its own guard (a per-repo copy again) or go unguarded | spec/assurance/AD-007-cross-repo-dependency-graph.md:287 |
| FND-004 | medium | O-4 leaves stale-`skip` behaviour unmeasured; measured: an unmatched `skip` gives `warning[unmatched-skip]` and `bans ok` (exit 0), and fails (exit 2) only with `-D unmatched-skip`. Three hand-kept skip lists therefore drift silently, and the AD should record this and the remedy | spec/assurance/AD-007-cross-repo-dependency-graph.md:307 |
| FND-005 | low | The CI claim is incomplete: RT's CI is manual-only (never dispatched; `make ci` is RT's gate) and RT's `make deny` already runs `licenses bans sources`, CG's runs a full `cargo deny check`; the awk ran in no repository's CI. Adopting (a) needs no CG or RT CI change for `make deny` to enforce it | spec/assurance/AD-007-cross-repo-dependency-graph.md:309 |
| FND-006 | low | FR-043 cites "FR-028's reason for the branch rule", but FR-028 states neither a branch rule nor the non-unifying-types reason (it says no split can arise because the root depends on no QSL crate); the rule is AD-007 G-3 | spec/core/functional/FR-043-one-lock-entry-per-crate.md:35 |
| FND-007 | low | O-4 attributes to the owner's first-hand ruling the clauses "creates no dependency cycle", "not vendored" and "QSL is offered the same check"; IR-581 records the owner's verbatim answer as the option label "Replace with one owned tool (Recommended)", and those clauses are the planner's Work text | spec/assurance/AD-007-cross-repo-dependency-graph.md:272 |
| FND-008 | low | The migration paragraph presumes option (a) while O-4a is open ("QSL decides whether to widen its entry"), and "the three copies are deleted in the same change that adds the check" cannot hold across three repositories' separate changes | spec/assurance/AD-007-cross-repo-dependency-graph.md:330 |

## Verdict

Mergeable after FND-001 is fixed; FND-002 to FND-004 should be fixed in the same round
because they are the measured basis the owner will answer O-4a from. Key judgement:
recording O-4a as an owner question is honest. Recommending (a) as "one owned tool" is
defensible (config of a shared tool is not a vendored script), but the AD states its cost
too narrowly: it replaces one copied script with three independently kept policies, and
the first-party guarantee in each repository depends on a guard that is itself per-repo.

## New findings (disposition pass 1)

Reviewed at agent-ix/quire-contract-ir@adb9fc2eac014d05d569226dd24659d93f895017.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-009 | medium | The (b)/(c) rows and the recommendation leave out how `make deny` gets the binary. Measured with a local-git probe: `cargo install --git ... --branch main` creates no caller lock entry and commits no pin (true), but if the tool is already installed, an offline run prints "Ignored package ... already installed" and keeps the old revision; only an online run replaces it. So either every `make deny` needs the network, or each machine runs whatever revision it last installed, with nothing recording which. That is the version-drift cost of (b)/(c), the counterpart of (a)'s skip lists, and it is unstated | spec/assurance/AD-007-cross-repo-dependency-graph.md:292 |
| FND-010 | low | "Option (c) ... is the only option that keeps one copy of the logic, matches the script exactly, and creates no cycle" is contradicted by the next sentence: "Option (b) has the same properties" | spec/assurance/AD-007-cross-repo-dependency-graph.md:333 |

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 674867b |
| FND-002 | fixed | 674867b |
| FND-003 | fixed | 674867b |
| FND-004 | fixed | 674867b |
| FND-005 | fixed | 674867b |
| FND-006 | fixed | 674867b |
| FND-007 | fixed | 674867b |
| FND-008 | fixed | 674867b |

## New findings (disposition pass 2)

Reviewed at agent-ix/quire-contract-ir@49245d4fa6a085a63efa89b0ecb39d8cffed8fe2.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-011 | low | The (b)/(c) remedy is over-costed. The AD says the fix for drift is a forced reinstall on every run ("a fetch and build each run"). Measured: a plain online `cargo install --git --branch main`, without `--force`, already replaces the binary when the branch has moved and only fetches when it has not; `--force` adds a rebuild every run and still installs the stale cached revision when offline. The remedy is an online install on every run (a fetch each run, a build only on change) | spec/assurance/AD-007-cross-repo-dependency-graph.md:340 |

## Dispositions (round 2)

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-009 | fixed | 49245d4 |
| FND-010 | fixed | 49245d4 |

## Dispositions (round 3)

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-011 | fixed | f74ec15 |
