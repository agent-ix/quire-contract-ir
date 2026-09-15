---
id: TASK-025
title: "Admit exact output-mapping requests"
type: Task
status: in_progress
track: H
priority: P0
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-032
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-043
    type: verifies
---
# TASK-025: Admit exact output-mapping requests

## Scope

Implement cycle-free target family/profile/source selection, aggregate limits,
strict `BoundPackage` obligation resolution, immutable request admission, and
typed refusal before target dispatch.

## Subtasks

- [ ] Define bounded domain-separated identity and exact FS06 profile values.
- [ ] Derive ordered immutable obligation views from executable bound clauses.
- [ ] Reject missing, duplicate, informational, stale, foreign, cross-family,
  unknown, zero-capacity, exceeded, overflowed, and cancelled requests.
- [ ] Prove request equality and mutation sensitivity without ambient inputs.

## Deliverables

- `quire_contract_model::output_mapping` request/profile/limit API
- TC-043 request admission and boundary cases

## Notes

- This task unblocks TASK-026.
- The model crate must remain free of QSL, QObs, Protocol, TL, and target runtimes.
