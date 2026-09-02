---
id: SR-029
title: "Gap analysis — shared assurance migration"
type: SpecReview
analysis: gap-analysis
scope: "PLAN-004-shared-assurance-migration; FR-022; revised FR-009; PGM-01-R09; spec/test-matrix.md TC-029..TC-034; tests/test_shared_assurance.py; issue #39 acceptance criteria"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/PLAN-004
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-022
    type: references
  - target: ix://agent-ix/quire-contract-ir/issues/39
    type: references
---

# SR-029: Gap analysis — shared assurance migration

## Summary

Checked PLAN-004 for completeness, every Test Matrix row for a real tracking tag
in a real test, and every changed script for a requirement that owns it. All
three PLAN-004 tasks are done, TC-029 through TC-034 bind to executable tests
that quire's coverage census sees (25 Python candidates, 25 tagged, 25 bound, 0
unbacked rows), and the six issue #39 acceptance criteria each resolve to a
gate. Four gaps are recorded, none of which blocks the migration.

## Verdict

**CONDITIONAL** — no `high` gap. Four gaps: one deferred to a filed upstream
issue, one deferred to a filed follow-up, two accepted as boundaries of what
this migration claims.

## Issue #39 acceptance criteria

| Criterion | Where it is discharged | Evidence |
| --- | --- | --- |
| IR conformance emits a documented structured domain result suitable for shared intake | `quire.contract.conformance-jsonl/v1`, unchanged, consumed by the `contract-conformance` adapter | TC-030; adapter probe `accepts-the-real-run` exit 0 over 99 rows |
| Quire provides static obligation/symbol/relation facts and Quoin retains/audits dynamic results | `quire coverage --scope . --json` retained by digest through `quoin change-assurance intake`; audits supplied to the receipt | TC-031; scenario `retain-producer-output`, `retained_bytes_identical: true`; scenario `audited-clean-versus-unaudited` |
| Existing PGM-01 records remain readable through the compatibility mapping | All ten read through `map_pgm01_bytes`, every one `lossy`, every source digest preserved | TC-032; census 19/19, 43 evidence files read, 0 bytes moved |
| No generic command executor or evidence store remains in this repository | `scripts/verify_evidence.py` and its three test modules deleted; `TC-024` asserts no script references either frozen schema | TC-024, seen red by adding a reference |
| Make targets are thin native orchestration | Every target calls one toolchain; no target computes a verdict; producers run only in `assurance-inputs` | TC-031 asserts the chain invokes no producer |
| Required non-success and tamper fixtures pass against the pinned shared versions | Twelve states demonstrated against quire-cli 0.31.0, quoin 0.23.1, ix-flow 0.0.4, engineering-assurance 0.2.0 | TC-029, TC-033 |

## State coverage

| State | Demonstrated by | Outcome observed |
| --- | --- | --- |
| pass | chain `retain-producer-output`; census real records | intake 0, retained bytes identical |
| fail | chain `attested-failure` | proof `invalid`, reason `result_failed` |
| unavailable | chain `attested-unavailable`; census `derived-unavailable` | reason `result_unavailable`; mapped state `unavailable` |
| not-computed | chain `attested-not_computed`, `audited-clean-versus-unaudited`; census `derived-not-computed` | reason `result_not_computed`, `audit_not_evaluated`; mapping refuses a status v1 never had |
| malformed | census `malformed-v1` | `unreadable`, names the wrong field |
| partial | chain `unattested-proof-and-absent-decision` | receipt `incomplete`, `attestation_missing` + `decision_missing` |
| stale | chain `stale-candidate-binding` | not discharged |
| suspect | census `pgm-01-568bd05`, named by retained correction COR-001 | `support_status: suspect` |
| vacuous | adapter probe `refuses-a-vacuous-run` | exit 1 — an empty run is not a clean run |
| tampered | census `derived-tampered`; chain `retained-bytes-changed-after-sealing`, `refuse-an-edited-receipt` | `incompatible`; intake refused; receipt refused (exit 2) |
| unsupported | census `unsupported-schema`, `correction-record-is-not-an-evidence-record`, `adversarial-correction-is-refused-the-same-way` | `incompatible`, unknown schema major |
| inconclusive | census, all ten real records | preserved as `inconclusive`, not rounded |

Every negative chain result is paired with a positive control, so no refusal
above can be a step that never worked: 4/4 controls hold.

## Findings

| ID      | Severity | Summary                                                                  | Refs                                              | Escape Cause                    |
| ------- | -------- | ------------------------------------------------------------------------ | ------------------------------------------------- | ------------------------------- |
| FND-001 | medium   | The accepted engineering-assurance pin does not contain the acceptance record its own documented gate requires | assurance/pins.json; engineering-assurance#20     | wrong-requirement               |
| FND-002 | medium   | `quoin evidence record` binds 0 obligations because this repository declares no suite registry | Makefile assurance-record                          | missing-requirement             |
| FND-003 | low      | The verification receipt cannot reach `valid` while the human decision is absent, so the fully-discharged end state is shown per-proof rather than per-receipt | scripts/assurance_chain.py `audited-clean-versus-unaudited` | correct-requirement-no-evidence |
| FND-004 | low      | `make ci` now requires network on first run to build `.venv-assurance` | Makefile assurance-env                            | correct-requirement-no-evidence |

## Dispositions

- **FND-001 — DEFERRED**, `agent-ix/engineering-assurance#20`. `v0.2.0` packages
  `"state": "pending_human_acceptance"` and has no `human_acceptance_recorded`
  predicate at all; both landed on `main` afterwards with no `v0.2.1`, so the
  documented acceptance condition is satisfiable only from a branch head — the
  one thing `FR-012-AC-1` forbids. This does not block the migration: the two
  artifacts this repository consumes are byte-identical at the tag and at `main`,
  so the read-only mapping used here *is* the released pin, and
  `check_shared_pins.py` reports the acceptance state it finds while gating only
  on the local toolchain, so neither an absent field nor a branch head is read as
  an approval in either direction. The fix is a tag, which is a release action
  and is not taken from here.
- **FND-002 — DEFERRED**, `agent-ix/quire-contract-ir#44`. Transcription works
  and binds nothing, and quoin reports those as two separate facts rather than
  one. Declaring `spec/evidence/suites.md` and the obligation model that binds
  conformance fixtures to `FR-011`..`FR-018` is real work on the traceability
  model, not on the migration boundary, and folding it in here would mean this
  PR also rewrites how the contract requirements are traced.
- **FND-003 — ACCEPTED.** The receipt is `incomplete` for two honest reasons:
  two open unknowns, and no ix-flow decision event. Only `@kreneskyp` can create
  the second, and a receipt that read `valid` without one would be the single
  worst thing this migration could produce. That the machinery *can* fully
  discharge a proof is shown at proof granularity instead — with an audit
  retained, `PROOF-conformance` reads `valid` with no reasons — which
  demonstrates the same capability without inventing a decision.
- **FND-004 — ACCEPTED.** Hosted CI is manual-dispatch only and is not being
  changed, so this affects a developer's first local run. The alternative —
  vendoring Engineering Assurance into the tree — would replace a pin with a
  copy, which is the failure mode the campaign exists to remove. `make clean`
  drops the environment; `make assurance-env` rebuilds it.

## PLAN-004 completeness

| Task | Status | Backed by |
| --- | --- | --- |
| TASK-013 inventory, pins, specification | done | issue #39 comment 5503859401; `assurance/pins.json`; `schemas/README.md`; FR-022; TC-029..TC-034 in the matrix |
| TASK-014 shared intake, compatibility view, state fixtures | done | `make assurance` exit 0 with the counts above; 15 new Python tests |
| TASK-015 dual run, then deletion | done | dual-run table in `plan.md`; deletion commit `604bda3`, separate and last |

## Underspecified code

None. Every script added by this change is owned by an FR-022 acceptance
criterion and cited by a test that quire's census binds:
`check_shared_pins.py` → FR-022-AC-1, `assurance_chain.py` → FR-022-AC-2/AC-3/
AC-5/AC-6, `pgm01_compatibility_view.py` → FR-009-AC-5 and FR-022-AC-4/AC-5/
AC-6.
