---
id: REV-013
title: "Local release-gate composition gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "issue #35 local composite-gate remediation candidate"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/NFR-002
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/NFR-004
    type: reviews
---

# REV-013: Local release-gate composition gap analysis

## Summary

The local `ci` composite now includes `spec` and `msrv`; `release-check` delegates to the same
complete local gate. The shared `assurance` target remains the Quoin-backed replacement for the
deleted repository-local verifier. No hosted workflow was changed or dispatched.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-1301 | high | **FIXED** — `ci` now runs Quire validation, strict coverage, and the matrix-status census through `spec`. | issue #35; NFR-004-AC-5 | correct-requirement-no-evidence |
| FND-1302 | high | **FIXED** — `ci` now runs exact Rust 1.75 over all locked targets through `msrv`. | issue #35; NFR-002-AC-2 | correct-requirement-no-evidence |
| FND-1303 | medium | **FIXED** — `release-check` and `ci` no longer describe different sets of mandatory local release gates. | Makefile | correct-requirement-no-evidence |
| FND-1304 | medium | **RETAINED** — generic evidence verification stays deleted; the existing shared-assurance scenarios exercise Quoin seal, intake, receipt, audit, and re-verification instead. | migration contract; FR-022; TC-029..TC-034 | wrong-requirement |
| FND-1305 | medium | **DEFERRED BY POLICY** — the manual-only hosted workflow is unchanged and was not dispatched. | issue #13; issue #35 | correct-requirement-no-evidence |
| FND-1306 | medium | **OPEN PROCESS GATE** — independent exact-head review remains required before landing. | issue #35 | correct-requirement-no-evidence |

## Verification

Full exact-head local verification is required after this review artifact and the composition change
are committed. Hosted CI is excluded.
