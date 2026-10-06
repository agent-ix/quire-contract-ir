---
id: SR-2191
title: "Gap analysis of IR-662 numeric width change"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@a352c4509f155c6de1b430501a3e9b211283a2d2; FR-013-AC-5, FR-013-AC-6, FR-014-AC-8, FR-015-AC-8, FR-016-AC-9, FR-019-AC-7; changed model source, wire, corpus, and tests"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-013
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-014
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-015
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-016
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: references
---

## Summary

Ticket: IR-662. Compared the changed implementation and tests with the six affected acceptance criteria using the computed Quoin Test Matrix and a source-to-test inspection. No new traceability, ownership, or hollow-test gap was found in this diff.

## Verdict

**PASS for the changed scope** — all six examined criteria have concrete trace-tagged tests that exercise the new width or arithmetic behavior. This is a diff-scoped review, not a claim that unrelated repository criteria have run evidence.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Plan completion: not assessed

Quoin matrix reported these changed-scope bindings as tagged: FR-013-AC-5 (six binders), FR-013-AC-6 (two), FR-014-AC-8 (one), FR-015-AC-8 (three), FR-016-AC-9 (one), and FR-019-AC-7 (two). The new tests assert actual decoded/serialized values, normalized rational acceptance, checked-range refusal, and canonical quoted decimal spelling. The source changes are owned by FR-013, FR-014, FR-015 and FR-016; the request boundary is owned by FR-019. The matrix has no run evidence for these criteria in this checkout; the focused tests recorded in SR-2190 passed locally. Optional semantic review was not invoked.
