---
id: SR-031
title: "Closing gap analysis — shared assurance migration"
type: SpecReview
analysis: gap-analysis
scope: "PLAN-004 exit criteria; issue #39 acceptance criteria; migration-contract PR checklist; SR-029 findings FND-001..FND-004"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/PLAN-004
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-022
    type: references
  - target: ix://agent-ix/quire-contract-ir/issues/39
    type: references
---

# SR-031: Closing gap analysis — shared assurance migration

## Summary

Closing gap analysis over PR #45 at `c0eacbe`. Every line of the accepted
migration PR review checklist has an answer, all six issue #39 acceptance
criteria are discharged, PLAN-004's exit criteria are met, and the four SR-029
gaps are dispositioned — two deferred to filed issues, two accepted as stated
boundaries of what this migration claims.

## Verdict

**CONDITIONAL** — no `high` gap. Two deferred with linked issues, two accepted.

## The migration contract's PR review checklist

| Line | Answer |
| --- | --- |
| Script inventory in the PR, decision per family, reason for anything unclassified | Yes — issue #39 comment 5503859401 and the PR body. Nothing unclassified. |
| No repository-local generic evidence schema remains, none introduced | The verifier is gone and no new schema was added. Two schemas are frozen rather than deleted, with the reason stated and enforced: every immutable manifest names one by path and digest, so deleting it would break a reference inside bytes that must stay readable. `TC-024` asserts no script references either, checked red. |
| No verdict read from stdout or stderr | Yes. The adapter binds on the runner's declared protocol; every attestation states its own `result`; a test scans for the pattern and states its own limits. |
| Domain runners, oracles, corpora, result formats unchanged | Yes — `git diff origin/main -- src/ corpus/ schemas/` is empty. |
| Every existing evidence directory byte-identical | Yes — `git diff origin/main -- evidence/` is empty, and the census reports 43 files read, 0 bytes moved, on every run. |
| Compatibility view exercised over real legacy records, `lossy` and `unreadable` shown | Yes — all ten real records `lossy`; `malformed-v1`, `unreadable-truncated` and `derived-not-computed` `unreadable`; `derived-tampered`, `unsupported-schema` and three correction records `incompatible`. |
| Pass, fail, unavailable, not-computed, malformed, stale, tampered each demonstrated | Yes, plus partial, suspect, vacuous, unsupported and inconclusive — twelve, tabulated in the PR body. |
| Both paths pass at the same candidate revision; deletion commit separate and last | Both were **run** at `74165ae`. The old path exits 1 and has been failing on `main` since PR #41; the shared path exits 0 and reads all ten records the old one could not use. The deletion is `604bda3`, separate, and the last functional commit. |
| Makefile is native orchestration; no target computes a verdict | Yes. Producers run only in `assurance-inputs`; every other target calls one tool and reports what it returned. |
| `check_compatibility_matrix.py` passes in the migrating repository's environment | Partially, and the gap is FND-001 below. The version half passes: 4/4 components classify `compatible` against the accepted matrix, via Engineering Assurance's own classifier. The acceptance half cannot pass against the released artifact, because `v0.2.0` neither records acceptance nor ships the predicate that reads it. |
| No workflow changed from manual dispatch | Yes — `.github/workflows/ci.yml` is untouched and no run was dispatched. |

## PLAN-004 exit criteria

| Criterion | Met |
| --- | --- |
| All three tasks done | Yes — TASK-013, TASK-014, TASK-015 |
| Every FR-022 and revised FR-009 criterion resolves to a passing tagged test | Yes — TC-029..TC-034 and TC-032; quire binds 25/25 Python candidates with 0 unbacked rows |
| `make ci` and `make spec` pass at the exact merge head | Yes, at `c0eacbe` |
| Twelve required states each demonstrated | Yes |
| Mutation probes turn every load-bearing check red | 6/6 census, 7/7 governance, plus two hand-run probes for assertions with no probe mode |
| No verdict read from a console stream | Yes |
| Every byte under `evidence/` unchanged | Yes |
| No hosted workflow dispatched | Yes |

## Findings

| ID      | Severity | Summary                                                                | Refs                          | Escape Cause        |
| ------- | -------- | ---------------------------------------------------------------------- | ----------------------------- | ------------------- |
| FND-001 | medium   | The accepted engineering-assurance pin cannot satisfy the acceptance half of its own documented gate | engineering-assurance#20      | wrong-requirement   |
| FND-002 | medium   | The conformance run transcribes 99 entries and binds 0 obligations      | quire-contract-ir#44          | missing-requirement |
| FND-003 | low      | The receipt cannot reach `valid` while the human decision is absent     | assurance/change-assurance.json | correct-requirement-no-evidence |
| FND-004 | low      | `make ci` needs network on its first run to build `.venv-assurance`     | Makefile                      | correct-requirement-no-evidence |

## Dispositions

- **FND-001 — DEFERRED**, `agent-ix/engineering-assurance#20`. Not a blocker,
  and the reasoning is worth stating rather than assuming. The gate's acceptance
  condition answers "may an enforcing migration begin". A human answered it once,
  upstream, on record and attributed. What is missing is a *release* carrying
  that answer, and a repository-local gate that re-decided the question from a
  package which does not carry it would be wrong in one direction or the other:
  reading `pending` as a block would stop eight migrations over tag hygiene, and
  reading a branch head as the pin would be exactly the substitution
  `FR-012-AC-1` forbids. So this repository does neither. It gates on what is
  local and checkable — versions, consumed-artifact digests, registry — and
  reports the acceptance state it finds without interpreting it. The fix is a
  `v0.2.1` tag, which is a release action and was not taken from here.
- **FND-002 — DEFERRED**, `agent-ix/quire-contract-ir#44`. Transcription working
  and binding nothing are two facts and quoin reports them separately; only the
  first is true here today. Declaring the suite registry and the obligation
  mapping for 99 corpus fixtures is work on this repository's traceability
  model, and folding it in would mean this PR also rewrote how the contract
  requirements are traced.
- **FND-003 — ACCEPTED.** The receipt is `incomplete` for two true reasons —
  `decision_missing` and `unresolved_unknown` — and both are the honest state of
  the candidate. Only `@kreneskyp` may create the decision event. Full discharge
  is demonstrated at proof granularity instead: with a retained audit,
  `PROOF-conformance` reads `valid` with no reasons.
- **FND-004 — ACCEPTED.** Hosted CI is manual-dispatch only and unchanged, so
  this affects a developer's first local run. Vendoring Engineering Assurance to
  avoid it would replace a pin with a copy — the failure this campaign removes.

## Underspecified code

None. Every script added is owned by an FR-022 acceptance criterion and cited by
a test quire's census binds. No production symbol in this change lacks an owning
requirement.

## What this analysis does not claim

That the shared path is correct in general, or that this repository is qualified
for anything. It claims that at `c0eacbe` the declared gates ran, produced the
outcomes recorded here, and turn red when the checks they depend on are removed.
Qualification is use-specific and lives outside this campaign.
