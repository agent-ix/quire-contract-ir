---
id: SR-023
title: "Gap analysis — PR #41 shared assurance governance reconciliation"
type: SpecReview
analysis: gap-analysis
scope: "PLAN-003, TM-001, FR-008, FR-009, FR-021, PGM-01-R07/R08/R09/R11, TC-023 through TC-028"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/PLAN-003
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/TM-001
    type: references
---
# SR-023: Gap analysis — PR #41 shared assurance governance reconciliation

## Summary

Post-implementation gap analysis of PLAN-003 (shared assurance governance) at
`14960ae03e3886d72983485ee0422542087b49a7`, covering plan completion, Test
Matrix backing, and code with no owning requirement. `quire validate` reports
81/81 docs grammar-clean. `quire coverage --strict` reports 97/108 rows backed
with two unbacked rows and zero contradicted statuses. The matrix is honest:
TC-026 and FR-021 are marked planned rather than covered, and
`scripts/validate_matrix_status.py` passes on that basis.

## Verdict

**FAIL** — TASK-012 is the open task, two matrix rows are unbacked under
`--strict`, and `make spec` exits 1. Every one of these is the PR's own declared
merge condition; none is a hidden gap.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-229 | high | PLAN-003 is not complete: TASK-012 (code review, gap analysis, issue feedback, merge) is the open task, and its exit criteria require the issue dispositions this PR defers until after review. | plan/PLAN-003-shared-assurance-governance/plan.md:24 |
| FND-230 | high | `quire coverage --strict` reports two unbacked rows — `TC-026` in `spec/test-matrix.md:63` and `FR-021-AC-3` in `spec/functional/FR-021-shared-assurance-ownership.md:54` — so `make spec` exits 1 and `make release-check` cannot pass on this branch. | spec/test-matrix.md:63 |
| FND-231 | medium | `spec/test-matrix.md` marks FR-008 and FR-009 `✅ covered` on the strength of TC-023, TC-024 and TC-025, all three of which resolve to substring assertions over documents in the same commit. The rows are honestly backed by tagged tests; the oracle behind them is document consistency, not behaviour. | spec/test-matrix.md:22 |
| FND-232 | medium | FR-021-AC-3 depends entirely on GitHub state that no gate can observe from the tree. Until `tc_026_*` exists and inspects a retained receipt, the requirement's only evidence is the PR narrative. | spec/functional/FR-021-shared-assurance-ownership.md:54 |
| FND-233 | medium | `validate_matrix_status.py` inspects `spec/test-matrix.md` and `spec/contract-test-matrix.md` only. `spec/program/PGM-01-governance.md` carries its own Acceptance Criteria table in which PGM-01-R11-AC-1 cites TC-026, and that table is outside the gate. | scripts/validate_matrix_status.py:12 |
| FND-234 | medium | `docs/shared-assurance-governance.md` is a plan Output and a test oracle but is outside `make spec`'s validation globs, so it can drift from PGM-01 without any gate objecting; the only coupling is the three substring assertions in `tc_024`, `tc_027` and `tc_028`. | Makefile:69 |
| FND-235 | low | `quire coverage --strict` reports `status-column-matches-nothing` for the Functional Requirement Coverage table in `spec/test-matrix.md`: the declared status column `Status` does not match the authored `Coverage Status`, so status classification is skipped and complete-but-unbacked rows are not checked there. Pre-existing, and also present on `contract-test-matrix.md`. | spec/test-matrix.md:16 |
| FND-236 | low | StR-001, StR-002 and StR-003 remain at 0/2 backed and NFR-001/NFR-002 at 50%. Pre-existing on `main`; not introduced by this PR. | spec/stakeholder/StR-001-authoritative-contract.md:1 |
| FND-237 | low | The plan id `PLAN-003` is also taken by `plan/PLAN-003-pgm02/` on the unpushed `issue-20-shared-evidence` branch. Since #20 is superseded here the collision should resolve by abandoning that branch, but it is worth recording so the id is not reused a third time. | plan/PLAN-003-shared-assurance-governance/plan.md:2 |

## Coverage

| Measure | Value |
|---|---|
| Plan tasks done | 2 of 3 (TASK-010, TASK-011) |
| New matrix rows added | 6 (TC-023 through TC-028) |
| New rows with a backing tagged test | 5 of 6 (TC-026 planned) |
| `quire coverage` rows backed | 97 of 108 |
| Unbacked rows / contradicted statuses | 2 / 0 |
| `spec/test-matrix.md` | 18/19 (94%) |
| `spec/functional/FR-021-shared-assurance-ownership.md` | 4/5 (80%) |
| `validate_matrix_status.py` | pass |
| `make ci` | exit 0 |
| `make spec` | exit 1 |

No underspecified code was found: the diff adds no source module, and every new
test symbol carries a resolving `Tracing:` tag. The optional standalone semantic
expansion was skipped because the user did not opt in; SR-022 still performed
the code-review skill's mandatory spec-code faithfulness check.
