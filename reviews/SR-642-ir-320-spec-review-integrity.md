---
id: SR-642
title: "integrity review of PR 236 (IR-320 matrix split, TestMatrixIndex, registry)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@92945cd2e400cdd7940e68db7aab21c3291b7395; spec/*/matrix/tests.md (TM-002..TM-008), spec/tests.md (TM-009), spec/spec.md Subsystem Registry; compared against origin/main spec/contract-test-matrix.md"
review_set: subset
---
# SR-642: integrity review of PR 236

## Summary

Ticket: IR-320. The old TM-002 had 67 data rows across its StR, FR, NFR, registry, Test Case Summary and Coverage Design tables. Each of those 67 rows appears exactly once across the seven new matrices, byte-identical: a sorted multiset diff is empty and no row is duplicated. Each row sits in the matrix of the subsystem whose directory holds its requirement or TC file. The StR rows and the NFR rows sit in core. TM-002 keeps its id for checked_package (ADR-0056 Matrices rule 5). The old matrix had no prose or notes outside its tables, so none was lost. In every TM-009 row, Requirements equals the set of requirement ids in that subsystem's directory and no other (ADR-0056). Each matrix carries the required FR Coverage and Test Case Summary tables. Each TC file sits beside the matrix that declares it. The registry names every model module, `quire-contract-ir::kani`, the binary and the root crate residue, each once.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | TM-009's Core Status mentions only TC-019 as planned. Core's own matrix also marks STD-001 as 🚧 (`kani_outcome_invalid` is planned, TC-223), and StR-001 and StR-003 as 🚧 (FR-019-AC-5 and others are planned) | spec/tests.md:12 |

## Verdict

Integrity holds: the rows, ids, counts and index agree. FND-001 is a summary-wording gap and needs no structural change.

## Dispositions

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 82d57ba |
