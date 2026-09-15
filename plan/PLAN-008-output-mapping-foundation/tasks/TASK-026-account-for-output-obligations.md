---
id: TASK-026
title: "Account for every mapped obligation"
type: Task
status: done
track: H
priority: P0
relationships:
  - target: ix://agent-ix/quire-contract-ir/TASK-025
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-033
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-043
    type: verifies
---
# TASK-026: Account for every mapped obligation

## Scope

Implement the bounded one-obligation mapper seam, source-fact states, closed
dispositions, typed conditions/causes/adequacy references, work charging, record
invariants, and derived record identity.

## Subtasks

- [x] Invoke only an exact-profile mapper once per obligation in admitted order.
- [x] Validate every disposition/source-state/output/condition/cause permutation.
- [x] Preserve exact dependencies and separate observation/protocol adequacy references.
- [x] Reject malformed, cross-wired, duplicate, unrequested, or over-budget candidates atomically.

## Deliverables

- Target-neutral mapper/candidate/budget API
- Immutable per-obligation mapping records and `sha256-jcs` identities
- TC-043 disposition, identity, and failure cases

## Notes

- This task unblocks TASK-027 and supplies the seam consumed by #55/#56/#57.
- Completed locally on 2026-09-15: TC-043 covers the mapper seam, candidate
  invariants, identity mutation matrix and whole-operation failure behavior;
  warning-denied workspace/all-target Clippy and the full locked workspace
  regression pass with `target-codex-backends`.
