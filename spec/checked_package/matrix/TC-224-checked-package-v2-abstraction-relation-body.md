---
id: TC-224
title: "CheckedPackage V2 abstraction relation bodies admit or refuse exactly"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-345
    type: verifies
  - target: ix://agent-ix/quire-specification/FR-451
    type: references
---
# TC-224: CheckedPackage V2 abstraction relation bodies admit or refuse exactly

## Description

Verify FR-345-AC-1 through FR-345-AC-9: the V2 reader admits QSpec FR-451's
`correspondence`/`abstraction_relation` body and refuses every other shape
with the code, cause and locus FR-451 fixes, in its reader order.

## Test Procedure

Over a lock selecting a domain package document built here (`ConfigVersion`
with the field `version`, the operation `attemptUpdate(next: Integer)`, and a
subtype that only inherits the operation), build checked packages holding one
abstraction relation node and read each: the minimal body of FR-345-AC-1, an
all-empty body, a tuple-field `rust_field`, and a body beside a
`state`/`frame` node and its operation anchor for the same pair. Then apply
each single mutation the FR-345 criteria name and read each mutated package:
each missing or extra member, an empty `RustPath`, each wrong member type, the
`term`, an occurrence role, another form carrying the body, each identity
member, each array's order, each target defect, each member defect, a
duplicate key within one node and across two nodes, and pairs of defects
across the state step and the abstraction step, across the checks of one node
and across two nodes. Compute each expected `node_id` from the body in the
test through `quire-canonical`, not through the reader.

## Expected Results

The unmutated packages admit. Each mutation returns exactly the code, cause
and locus its criterion names and no package; a package with defects in two
steps or two checks reports the earlier one.

## Status

Planned (IR-509 code change follows this specification).
