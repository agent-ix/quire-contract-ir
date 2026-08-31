---
id: FR-014
title: "Type and validate executable contract expressions"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-013
    type: depends_on
---
# FR-014: Type and validate executable contract expressions

## Description

The model shall represent literals, references, field and option access,
indexing, arithmetic, comparisons, implication, bounded quantification, and
distinct short-circuit and total Boolean operators.

## Inputs

An expression tree, declaration environment, expected result type, and source
span for every node.

## Outputs

A typed expression tree or an ordered set of source-located diagnostics.

## Behavior

The checker applies explicit operand and result rules. Short-circuit operators
retain conditional right-hand evaluation. Total operators require both operands
to be defined. Bounded quantifiers name a finite domain and a scoped variable.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-014-AC-1 | Ill-typed operands, result mismatches, invalid scopes, and non-Boolean clause roots fail at the narrowest source span. | Test (TC-016) |
| FR-014-AC-2 | Canonical representation distinguishes short-circuit conjunction/disjunction from total conjunction/disjunction. | Test (TC-016) |

## Dependencies

FR-013 defines the closed type system.
