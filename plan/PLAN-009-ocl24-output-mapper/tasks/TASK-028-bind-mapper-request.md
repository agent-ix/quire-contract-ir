---
id: TASK-028
title: "Bind mapper dispatch to the exact admitted request"
type: Task
status: in_progress
track: O
priority: P0
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-035
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-044
    type: verifies
---
# TASK-028: Bind mapper dispatch to the exact admitted request

## Scope

Retain the canonical mapping-request identity, require every `OutputMapper` to
expose its immutable request binding, and reject binding drift before any target
mapper invocation or partial result.

## Subtasks

- [ ] Add red tests for equal and independently mutated request identities.
- [ ] Add a domain-separated `MappingRequestId` to admitted requests.
- [ ] Extend the mapper seam with a mandatory immutable request binding.
- [ ] Check binding/profile equality before the first and every later dispatch.
- [ ] Prove mismatch and mid-run drift expose no candidate or package.

## Deliverables

- Cycle-free request identity and mapper-binding API
- Focused FR-035/TC-044 tests, including compatibility updates to TC-043 mappers

## Notes

- This is the only target-neutral change in #55 and unblocks TASK-029.
- The canonical identity material already exists under
  `quire.output.mapping-request-identity/v1-draft.1`; do not add ambient fields
  or change FR-299's closed package identity preimage.
