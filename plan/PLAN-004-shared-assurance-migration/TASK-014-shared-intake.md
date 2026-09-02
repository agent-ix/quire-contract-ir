---
id: TASK-014
title: "Implement the shared intake path, the legacy compatibility view, and the state fixtures"
type: Task
status: done
track: common
priority: P0
relationships:
  - target: ix://agent-ix/quire-contract-ir/TASK-013
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/PLAN-004
    type: part_of
---

# TASK-014: Implement the shared intake path, the legacy compatibility view, and the state fixtures

Deliver the native conformance result to Quoin through the `contract-conformance`
adapter and the `quoin change-assurance` chain, read the immutable PGM-01
history through Engineering Assurance's mapping, and give every required result
state a demonstrated case. Pair every negative result with a positive control so
no refusal can be a step that never worked.

## Acceptance

Every state has a case; no state collapses into another; the evidence tree is
byte-identical before and after every read; and mutation probes turn each
load-bearing check red.

## Completion

`make assurance` runs 4/4 pin classifications, 18/18 compatibility cases with 43
evidence files read and 0 bytes moved, 6/6 mutation probes detected, 10/10 chain
scenarios, 3/3 adapter probes, and 4/4 controls. TC-029 through TC-034 are
executable in `tests/test_shared_assurance.py`.
