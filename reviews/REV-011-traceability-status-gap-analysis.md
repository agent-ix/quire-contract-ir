---
id: REV-011
title: "Criterion binding and matrix-status gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "issues #33 and #34 remediation candidate"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/NFR-004
    type: reviews
---

# REV-011: Criterion binding and matrix-status gap analysis

## Summary

Every StR-001, StR-002, and StR-003 validation criterion is now bound to an existing test that
exercises its stated semantics. NFR-004-AC-5 owns completed-row/test-status consistency and states
the responsibility split explicitly. The status validator accepts `--root`; its test builds a real
temporary specification/test tree, observes failure with no symbol, then success after adding one.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-1101 | high | **FIXED** — Quire reports 2/2 backed for each of StR-001, StR-002, and StR-003. | issue #33; TC-015..TC-018; TC-020 | correct-requirement-no-evidence |
| FND-1102 | medium | **FIXED** — matrix-status code/tests bind NFR-004-AC-5; the plan/review criterion remains on its own TC-021 test. | issue #34; NFR-004 | wrong-requirement |
| FND-1103 | high | **FIXED** — the focused suite invokes `main` against real files and observes both exit directions. | `tests/test_matrix_status.py` | correct-requirement-no-evidence |
| FND-1104 | medium | **CONTAINED** — the census deliberately recognizes declared test symbols, including ignored tests, because the repository's full Rust command uses `--include-ignored`. It does not claim that source presence proves execution. | Makefile `test`; NFR-004-AC-5 | wrong-requirement |
| FND-1105 | medium | **OPEN PROCESS GATE** — independent exact-head review remains required before landing. | issues #33 and #34 | correct-requirement-no-evidence |

## Verification

- `python3 -m unittest discover -s tests -p 'test_matrix_status.py'`: 3/3 pass.
- `python3 scripts/validate_matrix_status.py`: pass on the candidate tree.
- `quire coverage --scope . --strict`: exit 0; no unbacked row or contradicted status; all seven
  stakeholder criteria and NFR-004-AC-5 are backed.

Full clean-head verification remains required. No hosted workflow was dispatched.
