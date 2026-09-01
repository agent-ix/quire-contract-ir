---
id: SR-027
title: "Gap analysis — post-merge governance reconciliation hardening"
type: SpecReview
analysis: gap-analysis
scope: "PLAN-003-shared-assurance-governance; FR-021; spec/test-matrix.md TC-023..TC-028; tests/governance_reconciliation.rs; issue #42 residuals R-1..R-4"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/PLAN-003
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-021
    type: references
  - target: ix://agent-ix/quire-contract-ir/issues/42
    type: references
---

# SR-027: Gap analysis — post-merge governance reconciliation hardening

## Summary

Verification gate over PLAN-003 and FR-021 after closing the four residuals in
quire-contract-ir #42. All PLAN-003 tasks are `done`, the engine reports no
unbacked matrix row, no status lie, and no untracked evidence symbol, and every
residual resolves to an owning acceptance criterion and a tagged test.

## Verdict

**CONDITIONAL** — no unbacked rows and no incomplete task; one medium finding
on plan-state hygiene and one low finding on pre-existing tag drift.

## Findings

| ID      | Severity | Summary                                                              | Refs                                                   |
| ------- | -------- | -------------------------------------------------------------------- | ------------------------------------------------------ |
| FND-001 | medium   | PLAN-003 was `status: active` with all three tasks `done`, and the #42 hardening has no owning Task | plan/PLAN-003-shared-assurance-governance/plan.md:5     |
| FND-002 | low      | Twelve tracking tags in the test tree resolve to no matrix row        | tests/foundation.rs:14                                  |

## Finding detail

### FND-001 — plan state trails the work

At analysis time `plan/PLAN-003-shared-assurance-governance/plan.md` carried
`status: active` while TASK-010, TASK-011, and TASK-012 were all
`status: done`. The #42 hardening — which changes the same requirement (FR-021)
and the same tests (TC-025, TC-026, TC-028) — landed under the issue alone, with
no Task in the bundle recording it.

Failure scenario: a later reader resolving "what is still open in PLAN-003" gets
`active` and no open task, so the plan cannot answer either question. The #42
change is invisible in the plan bundle even though it altered PLAN-003's
delivered tests.

Disposition: the stale status was corrected to `done` as part of the #42 closure
gate, since every task in the bundle is complete and the governance gate it
tracked (PR #41) is merged. The second half stands: post-merge hardening of a
closed plan is tracked by issue, not by a Task, and this analysis does not
retro-fit one.

### FND-002 — pre-existing unmatched tracking tags

`quire coverage --json` reports twelve `unmatched_tags` — tags cited in test
annotations that bind to no matrix row: `NFR-003-AC-1` (tests/conformance.rs),
`FR-009` and `NFR-004` (tests/evidence_corrections.rs), `STD-001`
(tests/expression.rs ×3, tests/identity.rs ×2), and `NFR-004` / `StR-003`
(tests/foundation.rs ×4).

Failure scenario: a reader treats one of these citations as coverage of the
named requirement, when the engine binds nothing for it.

All twelve predate this change; none is in `tests/governance_reconciliation.rs`.
Recorded so they are not attributed to the #42 hardening.

## Coverage

Plan completion — `plan/PLAN-003-shared-assurance-governance/`:

| Task | Status |
| --- | --- |
| TASK-010 specification review | done |
| TASK-011 documents and tests | done |
| TASK-012 closure gate | done |

Matrix verification — `quire coverage --scope . --json`:

| Measure | Value |
| --- | --- |
| Unbacked rows | 0 |
| Status lies | 0 |
| Untracked evidence symbols | 0 |
| Unmatched tracking tags | 12 (all pre-existing, see FND-002) |
| `coverage.backed` | 99 of 108 matrix rows; 50 of 50 evidence symbols examined and matched |
| `authoring.tag_rate` | 50 of 50 |
| Matrix status census | every ✅ row and PGM acceptance citation resolves to a completed executable test |

`make spec` (`quire validate`, `quire coverage --strict`,
`scripts/validate_matrix_status.py`) passes.

Residual traceability — each #42 residual to its owning criterion and test:

| Residual | Owning criterion | Backing test | State |
| --- | --- | --- | --- |
| R-1 dotted dependency keys | FR-021-AC-2 | TC-025 (`tc_025_preserves_domain_ownership_nonexecution_and_runtime_independence`) | closed, mutation-probed |
| R-2 TC-026 scope | FR-021-AC-3 | TC-026 (`tc_026_binds_the_retained_campaign_disposition_bytes`) | closed; AC-3 and the matrix row restated |
| R-3 marker enforcement | FR-021-AC-3 | TC-026 | closed; markers checked against retained bytes |
| R-4 quoted obsolete policy | FR-021-AC-5 | TC-028 (`tc_028_removes_conflicting_campaign_prescriptions`) | closed; rule normative in CONTRIBUTING.md, regression-probed |

Underspecified code — none. Every function added by this change
(`without_quotations`, `campaign_prescription_violations`, `retained_body`,
`assert_markers`) is test-tree guard logic reached only from TC-026 and TC-028,
each of which carries its `/// Tracing:` tag and an owning FR-021 criterion. No
production module was touched.

Semantic review — performed inline over FR-021's five criteria rather than
fanned out, because the change is confined to one requirement and three tests.
Each criterion's test exercises the real guard rather than a double: TC-025
scans the repository's actual Cargo manifests and then re-runs the guard against
mutations, TC-026 hashes and reads bytes retained on disk, and TC-028 walks the
real campaign-document census before probing synthetic paths. The one prior
intent mismatch — AC-3 claiming live GitHub state that an offline test cannot
observe — is what R-2 corrected; AC-3, the matrix row, the receipt, and the test
name now all describe retained-byte integrity.
