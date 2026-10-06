---
id: SR-2060
title: "Code review of IR-658 projection-owner specification diff"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@6b234ec0a471280d88e2d5cac8b5e5bb0b1aab61; spec/checked_package/functional/FR-038-consume-checked-package-v2.md; spec/checked_package/matrix/TC-228-checked-package-v2-wire-owner-and-lock-joins.md"
review_set: subset
---

## Summary

Ticket: IR-658. Examined the two changed spec files for duplication, fidelity to the selected QSpec contract, and consistency with the current reader and test. The mismatch refusal matches QSpec FR-322-AC-51 and both selected adverse mutations; the TC correctly records the reader update as pending. The QSpec fixture is referenced from its checkout and is not copied into IR.

## Verdict

**PASS** — no findings in the changed lines. The existing reader still returns `stale_dependency` at the first differing owner member; that is the explicitly pending code change, not a defect in this spec-only PR.

## Scope examined

- FR-038-AC-154: the projection mismatch code, cause, pointer, and recomputed package identity case.
- TC-228: the two projection mutation steps, expected refusal, and planned status.
- FR-038-AC-153 and FR-038-AC-155: owner-bearing node and join context.
- QSpec FR-322-AC-51 and the two `projection_owner_mutations` in merged QSpec `adverse.json`: external authority, read in place.
- Existing IR `owner.rs` and `checked_package_v2_owners.rs`: current behavior and pending implementation boundary.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
