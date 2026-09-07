---
id: TASK-016
title: "Declare the conformance suite and emit its trace targets"
type: Task
status: in_progress
track: A
priority: P0
relationships:
  - target: ix://agent-ix/quire-contract-ir/PLAN-005
    type: part_of
  - target: ix://agent-ix/quire-contract-ir/FR-018
    type: references
---

# TASK-016: Declare the conformance suite and emit its trace targets

## Scope

Complete the IR-owned half of issue #44: suite identity, manifest-declared trace targets, structured
result propagation, schema validation, mutation controls, regeneration, and gap analysis.

## Current State

Implementation is in progress. Final binding remains dependent on a released Quoin
`contract-conformance` adapter that preserves producer `trace_ids`; quoin 0.23.1 currently drops
them.
