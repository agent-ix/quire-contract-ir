---
id: TASK-029
title: "Admit the typed OCL 2.4 correspondence catalog"
type: Task
status: not_started
track: O
priority: P0
relationships:
  - target: ix://agent-ix/quire-contract-ir/TASK-028
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-342
    type: references
  - target: ix://agent-ix/quire-contract-ir/TC-220
    type: verifies
---
# TASK-029: Admit the typed OCL 2.4 correspondence catalog

## Scope

Implement target-specific OCL names and request-complete correspondence entries
with deterministic ordering, exact dependency provenance, fixed resource bounds
and constructor refusal for ambiguity or arbitrary target snippets.

## Subtasks

- [ ] Add red tests for identifier, keyword, qualified-name and catalog limits.
- [ ] Define bounded context/constraint/operation/value/field/function,
  definedness and relationship correspondence values.
- [ ] Require exactly one catalog entry per requested obligation and the exact
  TASK-028 request binding.
- [ ] Canonicalize order and reject duplicate, ambiguous, stale, foreign, extra,
  missing, cross-request and over-limit entries.
- [ ] Expose only read-only typed dependencies needed by TASK-030.

## Deliverables

- `quire_contract_ir::ocl24` name, dependency and catalog API
- Focused FR-342/TC-220 constructor and boundary tests

## Notes

- No raw OCL header/expression/comment/file string is an input.
- This task depends on TASK-028 and unblocks TASK-030.
