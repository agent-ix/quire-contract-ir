---
id: SR-030
title: "Closing code review — shared assurance migration"
type: SpecReview
analysis: code-review
scope: "PR #45 at c0eacbe; SR-028 findings FND-001..FND-008; exact-head gates"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/PLAN-004
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-022
    type: references
  - target: ix://agent-ix/quire-contract-ir/issues/39
    type: references
---

# SR-030: Closing code review — shared assurance migration

## Summary

Closing review of PR #45 at its exact final head `c0eacbe`. Every SR-028 finding
is dispositioned, the gates were re-run at that head rather than carried over
from an earlier one, and no reviewer feedback exists to fix: at the time of
writing the PR carries 0 reviews and 0 comments, and issue #39 carries the one
inventory comment this work posted.

## Verdict

**CONDITIONAL** — no `high` finding remains open. Two `medium` findings are
accepted with stated rationale; both concern the shape of a guard, not a defect
in what it guards.

## Exact-head gates

Run at `c0eacbe` with a clean working tree.

| Gate | Result |
| --- | --- |
| `make ci` | exit 0 |
| `make spec` | exit 0 |
| Rust tests | 40 passed, 0 failed, 0 ignored |
| Python tests | 17 passed |
| `validate_governance.py --mutation-probes` | 7/7 detected |
| compatibility census | 19/19 cases, 43 evidence files read, 0 bytes moved |
| compatibility mutation probes | 6/6 detected |
| change-assurance chain | 10/10 scenarios |
| `contract-conformance` adapter probes | 3/3 |
| controls | 4/4 |
| shared pins | 4/4 compatible, 0 artifact mismatches, 0 mirror references |
| `git diff origin/main -- evidence/ schemas/ corpus/ src/` | empty |
| hosted CI | not dispatched; last workflow run on this repository is from 2026-08-30 on `main` |

## Findings

| ID      | Severity | Summary                                                        | Refs                                   | Escape Cause                    |
| ------- | -------- | -------------------------------------------------------------- | -------------------------------------- | ------------------------------- |
| FND-001 | low      | All SR-028 findings dispositioned; no new defect at final head  | reviews/SR-028-shared-assurance-migration-code-review.md | correct-requirement-no-evidence |

## Disposition of every SR-028 finding

| SR-028 ID | Severity | Disposition | Where |
| --- | --- | --- | --- |
| FND-001 discarded intake failure | high | **FIXED** | `c0eacbe`, `scripts/assurance_chain.py` — every intake meant to succeed checks its status and raises with the tool's own message |
| FND-002 `TypeError` on the intake failure path | high | **FIXED** | `c0eacbe` — the retention scenario requires a mapping before indexing |
| FND-003 mirror check exempted its own rule text | medium | **FIXED** | `c0eacbe` — reads `assurance/pins.json` as JSON and inspects only installable fields; seen red with `--registry=https://npm.ix/` |
| FND-004 fabricated audit `report_digest` | medium | **FIXED** | `c0eacbe` — SHA-256 of the canonically serialized report |
| FND-005 no-op `{detail[:0]}` interpolations | medium | **FIXED** | `c0eacbe` — removed |
| FND-006 configuration-digest aliasing | medium | **ACCEPTED** | Re-deriving independently would create two computations that could disagree; quoin refuses an attestation whose `configuration_digest` does not match its record's proof obligation, so a divergence fails immediately rather than passing quietly. Confined to one method with the coupling stated in a comment. |
| FND-007 lexical no-verdict-from-stdout guard | medium | **ACCEPTED** | A lexical scan cannot prove the negative and does not claim to. The substantive guarantee is structural: the adapter binds on the runner's declared protocol on every row and refuses a foreign one, and every attestation states its own `result`. The guard's narrowness is deliberate — the broad version flagged `SEMVER.search(stdout)`, and a rule that noisy gets reworded rather than obeyed. |
| FND-008 `__pycache__` panic in the TC-024 census | low | **FIXED** | `c0eacbe` — census filters to `.py`, `.sh`, `.txt` |

## Reviewer feedback

None received. PR #45 has no reviews and no comments at `c0eacbe`. Branch
protection reports `MERGEABLE` with `mergeStateStatus: BLOCKED`, which is the
CODEOWNER requirement: the same account authored the change and cannot approve
it, so no approval will arrive. Any reviewer artifact that does arrive later is
to be committed rather than discarded, per the campaign review procedure.

## Freshness

Every count in the PR body was taken from the `c0eacbe` run above, not from an
earlier one. The candidate revision named in the dual-run table is `74165ae`,
the commit immediately before the deletion, which is the revision at which both
paths were actually run.
