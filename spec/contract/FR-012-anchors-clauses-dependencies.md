---
id: FR-012
title: "Anchor clauses and derive referenced-value dependencies"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-011
    type: depends_on
---
# FR-012: Anchor clauses and derive referenced-value dependencies

## Description

The model shall represent source spans, named execution points, executable and
non-executable clause kinds, and mechanically derived referenced-value
dependencies.

## Inputs

Source ranges, anchor names, clause kinds, expressions, and declarations.

## Outputs

Validated anchored clauses with deterministic dependency sets and structured
reference diagnostics.

## Behavior

Executable preconditions, postconditions, invariants, assertions, and cases
require a named initialization, handler, or pre/post anchor. Informational
clauses remain unanchored and non-executable. Dependency derivation walks the
complete expression tree and deduplicates identities canonically.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-012-AC-1 | A floating executable clause is rejected at its source span with a stable diagnostic code. | Test (TC-015) |
| FR-012-AC-2 | Every referenced input, state value, field, enum variant, and called pure function appears exactly once in the clause dependency set. | Test (TC-015) |

## Dependencies

FR-011 supplies package and requirement identity.
