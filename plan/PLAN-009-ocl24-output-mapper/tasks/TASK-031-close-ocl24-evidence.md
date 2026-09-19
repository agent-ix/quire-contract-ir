---
id: TASK-031
title: "Close integrated OCL mapper evidence"
type: Task
status: not_started
track: O
priority: P0
relationships:
  - target: ix://agent-ix/quire-contract-ir/TASK-030
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-341
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-342
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-343
    type: references
  - target: ix://agent-ix/quire-contract-ir/issues/55
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-220
    type: verifies
---
# TASK-031: Close integrated OCL mapper evidence

## Scope

Drive the real mapper through common record/package assembly, complete the
portable ConfigVersion/ParentPrecedes and Sequence corpus, promote the matrix
only from real trace evidence, and pass every local PR gate.

## Subtasks

- [ ] Complete TC-220 end-to-end record/package, determinism, mutation, resource,
  refusal and ambient-independence cases.
- [ ] Verify all 14 acceptance criteria have real Rust test tags and no scoped
  status lie or reverse-trace gap.
- [ ] Update API documentation, matrix, plan tasks and log from actual results.
- [ ] Run locked tests, rustfmt, warning-denied Clippy, cargo-deny, unsafe audit,
  Quire coverage and matrix-status checks on one unchanged head.
- [ ] Run PR-time self `/rust-review` and `/gap-analysis`, fix every finding,
  admin-merge, close #55 and reconcile #52/#58 plus QSpec #1/#9.

## Deliverables

- Integrated TC-220 evidence and promoted TM-002 rows
- Completed PLAN-009 and exact local verification record
- One reviewed, merged PR closing Contract IR #55

## Notes

- This task depends on TASK-030 and is the promotion gate for #55.
- Generated target text remains output-only even when every local gate passes.
