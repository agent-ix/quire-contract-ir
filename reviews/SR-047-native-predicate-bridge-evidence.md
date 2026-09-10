---
id: SR-047
title: "Evidence review of native predicate to TL projection"
type: SpecReview
analysis: evidence
scope: "FR-025 acceptance criteria; TC-038; SUITE-005 through SUITE-008; Quire/Quoin local results at cf4beaf15e35dfe276749637dbfb16c230070514"
review_set: all
relationships:
  - target: ix://agent-ix/quire-contract-ir/issues/63
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/FR-025
    type: reviews
---
# SR-047: Evidence review of native predicate to TL projection

## Summary

The plan separates properties, independent canonical goldens, real-reader
integration and deterministic resource failures while retaining planned status.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-6309 | medium | **FIXED:** One Property label could hide self-reference, mocked readers or unforced allocation errors. Four suites now separate properties, Snapshot goldens, real readers and failpoints. | SUR-001 SUITE-005..008 | wrong-requirement |
| FND-6310 | medium | **FIXED:** Matrix range syntax exposed only AC-1 and AC-8 to forward coverage. TC-038 now enumerates every criterion identity. | contract-test-matrix TC-038 | correct-requirement-no-evidence |
| FND-6311 | medium | **FIXED:** Extending STD-001 left its registry row falsely implemented. The row distinguishes implemented legacy codes from planned issue #63 codes. | contract-test-matrix STD-001 | correct-requirement-no-evidence |

## Evidence Boundary

The three canonical preimages independently reproduce their stated byte counts
and SHA-256 digests. Quoin classifies all eight FR-025 criteria with no new
method mismatch or inconclusive result. Coverage is intentionally 0/8 for
FR-025 and TC-038 is unbacked; no implementation or qualification claim is
made. **PASS for the specification evidence plan.**
