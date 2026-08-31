---
id: FR-015
title: "Track partial-operation definedness"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-014
    type: depends_on
---
# FR-015: Track partial-operation definedness

## Description

Validation shall compute and check definedness obligations for option access,
collection indexing, division, remainder, bounded arithmetic, and guarded
subexpressions.

## Inputs

Typed expressions, numeric overflow policy, collection bounds, and guard facts
available at each evaluation point.

## Outputs

An executable expression with explicit definedness obligations or structured
potential-undefined diagnostics.

## Behavior

Short-circuit guards contribute facts only to conditionally evaluated branches.
Unchecked option access, possible zero divisors, possible out-of-range indexes,
and arithmetic outside the declared policy fail validation.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-015-AC-1 | Every partial operation is accepted only when its obligation is statically established by declarations or dominating guards. | Test (TC-016) |
| FR-015-AC-2 | Replacing a short-circuit guard with a total Boolean operator exposes the previously guarded undefined operation. | Test (TC-016) |

## Dependencies

FR-014 defines evaluation semantics.
