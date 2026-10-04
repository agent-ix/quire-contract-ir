---
id: TC-225
title: "CheckedPackage V2 abstraction relation bodies admit or refuse exactly"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-346
    type: verifies
  - target: ix://agent-ix/quire-specification/FR-451
    type: references
---
# TC-225: CheckedPackage V2 abstraction relation bodies admit or refuse exactly

## Description

Verify FR-346-AC-1 through FR-346-AC-10: the V2 reader admits QSpec FR-451's
`correspondence`/`abstraction_relation` body and refuses every other shape
with the code, cause and locus FR-346 names, which are FR-451's where FR-451
states them and FR-346's marked IR readings where it does not.

## Test Procedure

Over a lock selecting a domain package document built here (`ConfigVersion`
with the field `version`, the operations `attemptUpdate(next: Integer)` and
`rebase(from: Integer, to: Integer)`, a subtype `ConfigVersionDraft` that only
inherits `attemptUpdate`, and a type inheriting one field name and one
operation name from two supertypes), build checked packages holding one
abstraction relation node and read each: the minimal body of FR-346-AC-1, an
all-empty body, a tuple-field `rust_field`, and a body beside a
`state`/`frame` node and its operation anchor for the same pair. Then apply
each single mutation the FR-346 criteria name and read each mutated package:
each missing or extra member, an empty `RustPath`, each wrong member type, the
`term`, an application body root and an application member value, an
occurrence role, another form carrying the body, each identity member, a stale
`node_id`, each array's order, each target defect, each member defect (the
keyword and non-identifier Rust strings, each frame operation outcome, the
parameter list, the equal `rust_parameter` pair over `rebase` and the
receiver clash over `attemptUpdate`), a duplicate key within one node and
across two nodes, and pairs of defects across the frame, state, abstraction
and operation steps, across the checks of one node and across two nodes.
Compute each expected `node_id` from the body in the test through
`quire-canonical`, not through the reader.

## Expected Results

The unmutated packages admit. Each mutation returns exactly the code, cause,
path and locus its criterion names and no package; a package with defects in
two steps or two checks reports the earlier one.

## Status

Planned (IR-509 code change follows this specification).
