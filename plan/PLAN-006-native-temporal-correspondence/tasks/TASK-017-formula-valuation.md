---
id: TASK-017
title: "FR-026 formula and valuation construction"
type: Task
status: done
track: A
priority: P0
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-026
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-039
    type: verifies
---
# TASK-017: FR-026 formula and valuation construction

## Scope

Build the occurrence-preserving future/past TL formula and admit the complete rectangular FR-025 valuation population from constructor-private owner views.

## Subtasks

- [x] **Traverse checked native graphs.** Preserve left-to-right postorder occurrences, intervals, Boolean structure, and closed semantic profiles.
- [x] **Bind predicate leaves.** Require exact projected correspondences and explicit Boolean cells for every formula leaf and position.
- [x] **Enforce boundaries.** Reject mixed/unrepresented profiles, replayed cells, invalid intervals, and resource excess without partial output.

## Deliverables

- `src/temporal/formula.rs`
- `src/temporal/valuation.rs`
- TC-039 formula, interval, profile, and valuation cases

## Notes

- The activation guard remains a distinct checked predicate; it is not added to formula-row width.
- Unblocks TASK-018.
