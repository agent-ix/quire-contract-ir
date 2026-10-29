---
id: SR-10810
title: IR-756 evidence insertion code and Rust review
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR 383; evidence.rs, model export, public reader tests, and migrated insertion callers"
review_set: subset
---

## Summary

Ticket IR-756. Reviewed the exact PR diff for first-value retention, typed errors, Rust idioms, caller result handling, and the public-reader tests.

## Verdict

**PASS (scoped)** — No actionable code or Rust defect found. Full gates remain with the author.

## Examined units

- `FR-047-AC-1` (`examined`): Domain vacant/equal insertion and independent exact keys.
- `FR-047-AC-2` (`examined`): Byte-distinct conflict and atomicity in both orders.
- `FR-047-AC-3` (`examined`): Equal admitted dependencies, retained first Arc, and distinct ceilings.
- `FR-047-AC-4` (`examined`): Full-content conflict, including equal package IDs with different retained metadata.
- `FR-047-AC-5` (`examined`): Exact feature declarations and reader support.
- `FR-047-AC-6` (`examined`): Public typed result surface and post-conflict reader equivalence.
- `FR-019` (`context_only`): Model public export inventory.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Review evidence

The two insertion methods use one ordered-map entry lookup and compare raw bytes or admitted package content before mutation. Error payloads carry the occupied exact key. `CheckedPackageV2::eq` compares wire and decoded kinds; the code does not accept package-id equality alone. Public test controls compare complete evidence and both reader entrypoints. Changed callers consume the new `Result`; no production caller was left ignoring a conflict. No Cargo or full gate was run by this reviewer.
