---
id: SR-2026102730
title: "IR-1003 PR 670 traceability gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR 670; FR-061-AC-3/7 test bindings in checked_package_v2_refinement.rs"
review_set: subset
---

## Summary

Inspected the PR 670 trace tags, IR TC-062 and the changed reader tests. FR-061-AC-3/7 are tagged. The population test has one foreign concrete row and therefore cannot distinguish correct population-first precedence from a reader that checks a later array first. The repository-wide matrix command did not complete during this diff-scoped review; this is a manual acceptance-criterion-to-test check.

## Verdict

**CONDITIONAL** with FND-001. The test does exercise a population owner refusal, but its precedence oracle is incomplete.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The new population-root swap test contains no foreign object or step key. It passes even if row checking runs objects or steps before populations, so it does not prove TC-062's population-first refusal order. Add a later competing foreign concrete key and assert the population locus. | tests/it/checked_package_v2_refinement.rs:1898 |

## Coverage

| Unit | Role | Result |
| --- | --- | --- |
| FR-061-AC-3 | examined | Changed tests cover population, object, operation anchor, history, assume, catalog and row-empty swap cases; population precedence lacks a competing defect. |
| FR-061-AC-7 | examined | Distinct row-empty node identities and public readback are asserted. |
| TC-062 | examined | Its population-before-object/step control is not implemented by the tagged population test. |
| FR-375-AC-33 | context_only | Defines concrete-key row order and refusal precedence. |
| FR-375-AC-34 | context_only | No reflexive wire mechanism is in this PR; upstream ruling remains pending. |

Plan completion: not assessed.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | At PR head 3525ecf2, the population swap test adds a foreign object key and a later step key to the same refinement while asserting `invalid_model_binding`/`malformed-declaration` at `populations/0/concrete`. This now discriminates population-first order. Focused execution and hosted checks were pending at disposition time. |
