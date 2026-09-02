---
id: TASK-015
title: "Prove both paths at one candidate revision, then delete the local verifier"
type: Task
status: done
track: common
priority: P0
relationships:
  - target: ix://agent-ix/quire-contract-ir/TASK-014
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/PLAN-004
    type: part_of
---

# TASK-015: Prove both paths at one candidate revision, then delete the local verifier

Run the old and the new path at the same candidate revision and record both
results, whatever they are. Then remove `scripts/verify_evidence.py`, its three
Python test modules, and its Make targets in a commit of their own, last. Freeze
the two generic PGM-01 schemas in place and make the freeze enforceable.

## Acceptance

The dual-run result is recorded before deletion; the deletion commit is separate
and last; every byte under `evidence/` is unchanged; and no script references a
frozen schema afterwards.

## Completion

The dual-run at `8e0953e` is in `plan.md`: the old verifier exits 1 for every
retained record because no subject tree matches `HEAD`, and the shared path
reads all ten. The deletion removes 488 lines of verifier and 552 lines of its
tests. TC-024 now locks the three schema files by digest and asserts no script
references either frozen schema.
