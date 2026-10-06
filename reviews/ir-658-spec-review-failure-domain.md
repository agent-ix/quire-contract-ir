---
id: SR-2063
title: "Failure-domain review of IR-658 projection-owner refusal"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-contract-ir@6b234ec0a471280d88e2d5cac8b5e5bb0b1aab61; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-228-checked-package-v2-wire-owner-and-lock-joins.md"
review_set: subset
---

## Summary

Ticket: IR-658. Tested the changed identity boundary against a tampered projection with a recomputed package id, for both SourceOwner and ModelOwner. The specified refusal targets the projection owner itself before the owner join and derived-key checks, preventing either later stage from hiding the mismatch.

## Verdict

**PASS** — the changed rule addresses the applicable identity confusion and first-fault cases. No callback, side-effect, or graph traversal behavior is changed by this diff.

## Scope examined

- FR-038-AC-154: owner mismatch and refusal precedence.
- TC-228: both mutation cases and exact result assertions.
- FR-038-AC-155: adjacent owner-join behavior.
- QSpec FR-322-AC-51 and adverse `projection_owner_mutations`: source/model tampering cases.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
