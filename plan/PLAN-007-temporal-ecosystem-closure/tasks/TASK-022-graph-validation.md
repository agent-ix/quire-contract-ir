---
id: TASK-022
title: "FR-027 typed ecosystem graph validation"
type: Task
status: done
track: G
priority: P0
relationships:
  - target: ix://agent-ix/quire-contract-ir/TASK-021
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-027
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-040
    type: verifies
---
# TASK-022: FR-027 typed ecosystem graph validation

## Scope

Validate complete node/edge populations, relation-specific endpoint kinds, unique executable ownership and acyclic runtime/ownership topology.

## Subtasks

- [x] Reject missing, duplicate, dangling, ill-typed and self-forbidden nodes/edges with closed precedence.
- [x] Require exactly one executable owner for every executable contract and valid owning repositories for runtime components.
- [x] Derive deterministic adjacency and topological order under bounded work without recursive panic paths.

## Deliverables

- `src/ecosystem_model/graph.rs`
- TC-040 relation, ownership, cycle and ordering differentials

## Notes

- Normative and verification edges are descriptive and cannot become runtime authority.
- Unblocks TASK-023.
