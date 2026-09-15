---
id: TASK-030
title: "Render and classify bounded OCL 2.4 obligations"
type: Task
status: not_started
track: O
priority: P0
relationships:
  - target: ix://agent-ix/quire-contract-ir/TASK-029
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-037
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-044
    type: verifies
---
# TASK-030: Render and classify bounded OCL 2.4 obligations

## Scope

Implement a bounded, iterative OCL expression renderer and whole-obligation
classifier returning common `MappingCandidate` values with exact target bytes,
regions, work, dependencies, conditions and causes.

## Subtasks

- [ ] Add red golden tests for every admitted scalar, anchor, state observation,
  Sequence operation and local-scope form.
- [ ] Render canonical constraint headers, parentheses, names, whitespace,
  line endings, collection syntax and `@pre` placement.
- [ ] Charge checked node/output work and refuse exact/just-over/overflow budgets.
- [ ] Emit native-domain and exact definedness/relationship conditions.
- [ ] Return whole-obligation unrepresented/refused candidates for every
  unsupported, missing, conflicting or non-ready case with no substitute bytes.

## Deliverables

- OCL mapper implementing the request-bound `OutputMapper` seam
- Focused FR-037/TC-044 golden, boundary and refusal tests

## Notes

- Avoid recursive rendering on attacker-controlled depth; the admitted request
  already carries an explicit nesting bound.
- No OCL parser/typechecker or target runtime participates in classification.
- This task depends on TASK-029 and unblocks TASK-031.
