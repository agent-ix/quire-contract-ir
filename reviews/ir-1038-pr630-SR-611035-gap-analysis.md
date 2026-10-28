---
id: SR-611035
title: "Gap analysis — IR-1038 QSpec execution point vectors"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; FR-066-AC-1, FR-066-AC-2; tests/conformance_qspec/execution_point_identity.rs; tests/it/checked_package_v2_frame_entries.rs"
review_set: subset
---

## Summary

Reviewed the FR-066-AC-1/2 vector residual against the new trace-tagged conformance test, the existing local execution point tests, and the production candidate derivation. The vector oracle is read from QSpec and the test fails for a changed digest, row set, point kind, or IR candidate.

## Verdict

**PASS** — no gap in the PR's published-vector slice. The broader FR-066 acceptance is supported by existing local tests and remains subject to the final focused conformance run.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-066-AC-1: new `published_execution_point_application_vectors_match_ir_candidates` carries the AC trace and checks one row per published initialization, handler, pre, and post application vector, then recomputes each canonical SHA-256 and compares IR's production candidate.
- FR-066-AC-2: the same test checks independent canonical derivation against each published row; existing TC-056 tests exercise absent/present clause and anchor key differences, package IDs, round trip, and refusals.
- Computed `quoin matrix --repo . --json`: both AC-1 and AC-2 are statically tagged by the new test. The matrix reports no run evidence at this PR head; the coder's focused conformance run remains pending.
- Reverse gap: the added test exercises the existing FR-066 application identity path; it introduces no production behavior or duplicate upstream vector.
- Plan completion: not assessed.
- Optional semantic review: not requested; this pass inspected the concrete changed AC bindings and their production path.
