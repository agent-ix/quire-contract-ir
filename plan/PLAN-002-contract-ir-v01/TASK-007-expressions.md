---
id: TASK-007
title: "Implement types, expressions, and definedness"
type: Task
status: done
track: A
priority: P0
relationships:
  - target: ix://agent-ix/quire-contract-ir/issues/8
    type: references
  - target: ix://agent-ix/quire-contract-ir/TASK-006
    type: depends_on
---
# TASK-007: Implement types, expressions, and definedness

Implement FR-013 through FR-015 after TASK-006 is Done. Preserve short-circuit
semantics and source-located failure evidence.

## Plan Delta

- Define a closed, declaration-backed value type model for scalars, enums,
  records, options, bounded collections, inputs, states, and pure functions.
- Validate declaration uniqueness, numeric/collection bounds, named-type
  existence, and cycles before checking expressions.
- Type literals, observations, access, indexing, calls, arithmetic,
  comparisons, Boolean operators, implication, and finite quantifiers.
- Infer true/false guard facts across conditionally evaluated branches and
  reject every undischarged option, index, divisor, or checked-overflow
  obligation at its narrowest span.
- Retain ordered diagnostics, structural expression equality, mechanically
  derived dependencies, and language-neutral serialized vocabulary.

SR-011 records 33 fixed specification findings and a clean narrow confirmation.
REV-004 records 23 fixed implementation findings, the rejected malformed and
failed reviewer attempts, a final clean static confirmation, and the separate
passing local gate result. The project item may move through review to Done.
