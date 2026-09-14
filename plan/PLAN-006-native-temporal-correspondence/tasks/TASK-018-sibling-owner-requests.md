---
id: TASK-018
title: "FR-026 sibling native and TL owner requests"
type: Task
status: done
track: A
priority: P0
relationships:
  - target: ix://agent-ix/quire-contract-ir/TASK-017
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-026
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-039
    type: verifies
---
# TASK-018: FR-026 sibling native and TL owner requests

## Scope

Construct and strict-read independent QSL-native and tl-mltl request families from the same admitted subject, observations, valuations, and correspondence.

## Subtasks

- [x] **Bind all authority axes.** Preserve clock, capture, activation, progress, closure, completeness, availability, support, and anchor inputs.
- [x] **Construct future and past carriers.** Emit owner-read trace or position-history/history-requirement documents for the selected profile.
- [x] **Keep evaluation outside Contract IR.** Expose validated requests without parser, evaluator, callback, plugin, or mirrored wire entry points.

## Deliverables

- `src/temporal/request.rs`
- `src/temporal/correspondence.rs`
- TC-039 real-owner request and identity cases

## Notes

- QSL `f1700a92` supplies the exact activation-guard checked-predicate owner API.
- Unblocks TASK-019.
