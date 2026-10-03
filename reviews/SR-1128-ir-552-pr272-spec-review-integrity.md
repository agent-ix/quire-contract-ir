---
id: SR-1128
title: "integrity and matrix review of PR 272 (FR-038-AC-112 and AC-113 trace)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@29cdb1495b4a623e25df532705553da1e03d6112; spec/checked_package/matrix/tests.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/tests.md; make spec at the reviewed sha"
review_set: subset
---
# SR-1128: integrity and matrix review of PR 272

## Summary

Ticket: IR-552. Checked the trace and matrix rows for FR-038-AC-112 and
FR-038-AC-113.

Measured:

- `spec/checked_package/matrix/tests.md` gives the FR-038 row's AC range as
  "FR-038-AC-109 through FR-038-AC-113". Its status names AC-112 and AC-113 as
  planned (IR-552) with no test yet. The TC-048 row names both as planned, run
  in `tests/conformance_qspec/main.rs` by `make conformance-qspec`. That file
  and the Makefile target exist at the reviewed sha.
- The `spec/tests.md` Checked package row adds "AC-112 and AC-113 planned,
  IR-552". The file records no AC counts that would need changing.
- `make spec`: validate passes; 362/363 docs are grammar-clean, with 1
  finding (FR-014 `ac:vague-response`, the baseline); strict coverage reports
  23 unbacked rows (the baseline, not increased).
- The diff touches four spec files only, and copies no QSpec fixture.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | TC-048's Description still ends its scope at "FR-038-AC-109 through FR-038-AC-111" and does not name AC-112 or AC-113. The Test Procedure adds them, but it lists only "unset, empty or names no checkout" as failures. It omits a missing or non-JSON file, an absent or empty list, a recorded-outcome mismatch and the stale expected-failure entry that AC-112 requires. Expected Results gives no outcome for either AC | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:43-50, 284-287 |

## Verdict

Mergeable on this method alone. The matrix rows are consistent and the
baselines hold. FND-001 is a low trace gap to fix with the AC-112 and AC-113
amendments that SR-1127 asks for.

## Dispositions

Round 1, reviewed at agent-ix/quire-contract-ir@cbcdd758c34ce335b39f4684cf20dff9ca32cd38. `make spec`: validate passes, 1 grammar finding (FR-014 baseline), strict 23 unbacked (baseline).

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | cbcdd75 |
