---
id: TC-050
title: "CheckedPackage V2 items lower independently with exact correspondence"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: verifies
  - target: ix://agent-ix/quire-specification/TC-217
    type: references
---
# TC-050: CheckedPackage V2 items lower independently with exact correspondence

## Description

Verify FR-038-AC-6 (QSpec FR-195-AC-1 through FR-195-AC-5) through the admitted
V2 package API.

## Test Procedure

Lower every node of the vendored all-families and nominal fixtures under a
profile supporting every tag. Lower a mixed request holding a supported node,
an absent key, a node reaching an unsupported tag, an unbounded integer type
under a bounds-required profile, the same type with a bounding domain, and a
request past the work limit. Repeat the supported request alone.

## Expected Results

Each supported node lowers with its exact source-map entries, semantic type,
reachable dependencies, bounding domains, reachable claims and a stable
`quire.contract-ir.semantic/v1` digest. Each other request returns exactly its
`invalid_input`, `unsupported`, `requires_bound` or `failed` record without a
node, and the supported sibling's record equals its stand-alone record.

## Literal type annotations are not value types (FR-038-AC-39)

Under a bounds-required profile, lower `x + 1` over a parameter `x` typed at an
`integer_range` domain over `integer`; every literal carries its FR-322 `type`,
so the parameter's name literal names the unbounded `text` type. Lower a
parameter typed at an unbounded `integer`, and one typed at `rational`.

The first request lowers, with the domain in `bounds` and the `text`
annotation in `dependencies`. The other two return `requires_bound` naming the
type the parameter is typed at, never an annotation.
