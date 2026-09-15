---
id: TASK-027
title: "Assemble the output package atomically"
type: Task
status: done
track: H
priority: P0
relationships:
  - target: ix://agent-ix/quire-contract-ir/TASK-025
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/TASK-026
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-034
    type: references
  - target: ix://agent-ix/quire-contract-ir/issues/95
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-043
    type: verifies
---
# TASK-027: Assemble the output package atomically

## Scope

Implement deterministic fragment assembly, checked absolute regions, raw and
JCS identity domains, immutable packages, downstream observer references,
fault/cancellation atomicity, and the unchanged-head promotion gate for #95.

## Subtasks

- [x] Assemble fragments and shift/validate regions with checked arithmetic.
- [x] Verify complete record/order/profile/generator/digest/limit identity before exposure.
- [x] Prove deterministic replay, mutation sensitivity, and ambient/observer independence.
- [x] Inject cancellation, allocation, mapper, record, region, overflow, and resource failures.
- [x] Complete TC-043 traces, matrix/plan status, Rust review, gap analysis, and local gates.

## Deliverables

- Immutable `GeneratedOutputPackage` and downstream `StructuralObservationRef`
- Complete TC-043 and cycle-free dependency evidence
- PR-ready #95 tracking and downstream handoff

## Notes

- One PR closes #95; target-specific mappers remain outside this task.
- Implementation is Green locally on 2026-09-15: 15 integration cases and one
  checked-overflow unit case cover atomic admission, mapping, assembly, package
  identity, cancellation/fault injection, and downstream observer evidence.
  PR-time Rust review SR-546 and gap analysis SR-547 pass after all findings
  were repaired, and the unchanged-head promotion gates pass.
