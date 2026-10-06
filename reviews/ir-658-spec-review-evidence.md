---
id: SR-2064
title: "Evidence review of IR-658 projection-owner refusal"
type: SpecReview
analysis: evidence
scope: "agent-ix/quire-contract-ir@6b234ec0a471280d88e2d5cac8b5e5bb0b1aab61; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-228-checked-package-v2-wire-owner-and-lock-joins.md"
review_set: subset
---

## Summary

Ticket: IR-658. The installed `quoin advise --json` recommends Test methods for FR-038-AC-154 and reports no mismatch with its authored `Test (TC-228)` method. TC-228 names the two QSpec adverse mutations and the precise code, cause, pointer, and recomputed package-id assertions. The external conformance row remains planned until the reader change.

## Verdict

**PASS** — the verification method and proposed cases fit the amended criterion. This review does not claim that TC-228 already executes the new cases.

## Scope examined

- FR-038-AC-154: authored verification cell and criterion text.
- TC-228: mutation procedure, expected results, and status.
- Installed method catalog advice for FR-038-AC-154: no mismatch.
- QSpec FR-322-AC-51 and both selected adverse mutations: expected evidence shape.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
