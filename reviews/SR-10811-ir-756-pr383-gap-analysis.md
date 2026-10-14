---
id: SR-10811
title: IR-756 evidence insertion gap analysis
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR 383 at 6ecd464; FR-047-AC-1..6, TC-059, evidence.rs, and public integration tests"
review_set: subset
---

## Summary

Ticket IR-756. Scoped requirement-to-test and code-to-requirement analysis of the changed evidence insertion surface.

## Verdict

**PASS (scoped)** — All six changed-scope criteria have substantive trace-tagged public tests, and the new source behavior is owned by FR-047. This is not a repository-wide gap verdict.

## Examined units

- `FR-047-AC-1` (`examined`): Two traced domain controls.
- `FR-047-AC-2` (`examined`): Raw-byte and malformed-first controls.
- `FR-047-AC-3` (`examined`): Distinct Arc/ceiling and selected dependency controls.
- `FR-047-AC-4` (`examined`): Different IDs and equal-ID metadata controls.
- `FR-047-AC-5` (`examined`): Exact feature variants.
- `FR-047-AC-6` (`examined`): Typed errors, reader equality, no latched failure.
- `TC-059` (`examined`): Six-control public procedure.
- `FR-019` (`context_only`): Export inventory.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Plan completion: not assessed.

The six `Trace:` lines in `tests/it/checked_package_evidence_insertion.rs` cover FR-047-AC-1 through AC-6. The reviewer inspected assertions and the production equality/insertion branches. A repository-wide `quire matrix` invocation did not finish within this bounded review; no computed-matrix or full-gate result is claimed here.
