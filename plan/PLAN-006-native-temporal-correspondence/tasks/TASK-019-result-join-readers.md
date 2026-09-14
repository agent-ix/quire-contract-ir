---
id: TASK-019
title: "FR-026 formula result join and strict readers"
type: Task
status: done
track: A
priority: P0
relationships:
  - target: ix://agent-ix/quire-contract-ir/TASK-018
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-026
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-039
    type: verifies
---
# TASK-019: FR-026 formula result join and strict readers

## Scope

Compare only constructor-private formula-wide native and TL result views, preserve every typed non-value, and rederive projection/join decisions from complete owner authority.

## Subtasks

- [x] **Normalize structural outcomes.** Compare value/non-value, four progress/closure axes, completeness, activation, settlement, support, and bindings without Boolean coercion.
- [x] **Validate direct lineage.** Require paired owner corrections and the exact strict-read prior join without equating owner identity domains.
- [x] **Strict-read bridge decisions.** Reject hostile/noncanonical bytes and any mismatch from complete re-derivation.

## Deliverables

- `src/temporal/join.rs`
- `src/temporal/decision.rs`
- `src/temporal/reader.rs`
- TC-039 agreement, non-value, conflict, correction, and readback cases

## Notes

- Quire Protocol predicate results remain leaf-valuation inputs only.
- Unblocks TASK-020.
