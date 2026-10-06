---
id: SR-2062
title: "Integrity review of IR-658 projection-owner refusal"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@6b234ec0a471280d88e2d5cac8b5e5bb0b1aab61; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-228-checked-package-v2-wire-owner-and-lock-joins.md"
review_set: subset
---

## Summary

Ticket: IR-658. Checked the amended owner rule for a single interpretation, consistent vocabulary, and alignment between requirement and test case. The well-shaped projection mismatch is distinct from malformed owner shape and from a node owner that fails its lock or model join.

## Verdict

**PASS** — the changed behavior has one observable result, and the TC identifies the source and model cases that exercise it.

## Scope examined

- FR-038-AC-154: malformed-wire and well-shaped mismatch branches.
- FR-038-AC-153 and FR-038-AC-155: presence and join boundaries.
- TC-228: procedure and expected results for both branches.
- QSpec FR-322-AC-51 and its two selected adverse mutations: external consistency check.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
