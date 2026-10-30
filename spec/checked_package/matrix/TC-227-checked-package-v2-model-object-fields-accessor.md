---
id: TC-227
title: "CheckedPackage V2 returns a model object type's fields and derived member types as values"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: verifies
---
# TC-227: CheckedPackage V2 returns a model object type's fields and derived member types as values

## Description

Verify FR-038-AC-136 through FR-038-AC-144 and AC-370 through AC-373: the reader computes, at
admission and under the `work` limit, each selected object type's effective field
table and retains it; the admitted `CheckedPackageV2` exports
`model_object_fields`, which returns a model declaration object type's fields in
the retained declaration order, each with its authentic declaring object node id
and the member type FR-322 step 4 derives (`Boolean`,
`Integer`, `IntRange` with `i128` bounds, `Reference`, `Option`, `Collection`) or
`None` where the tables give none, by the functions admission uses, independent
of every node body.

## Test Procedure

Build domain package documents in this repository and a lock selecting each:
one with two `Int[0, 1000]` fields and a read of one; one with an inheritance
chain, an inherited field, a redefinition with a narrower range and a
most-derived redefiner; one declaring every kind of AC-138 (the `i128` extremes,
2^53 + 1 and one past each `i128` extreme); one with an ambiguous
name, in a package with no read of it and a sibling with a read; one with a 200-type chain and 100
redefinitions; chains of 4, 8, 1000 and 1500 types with one field each.
Include an object declaring `zeta` then `alpha`, a Base/Mid/Leaf chain, a
redefiner and a diamond; include their actual model declaration nodes in the
graph. Build the checked packages in the test, selecting every read's type node by its valid structural id and checking its declared type independently of the reader. Admit each, call `model_object_fields` on the model declaration
node, and compare the result with the declared values and graph declaration ids.
Look up `alpha` from the unsorted returned slice. Separately remove Base's
declaration node while Leaf still inherits `Base.b`; this package admits
under QSpec FR-322-AC-30, but `model_object_fields(Leaf)` must return the
named typed `MissingDeclaringNode` for `Base.b`, with no partial result. Add
Base’s authentic graph node and require success with its ID. Two admitted
object declaration nodes matching the same selected owner identity but with
different structural ordinals must return `AmbiguousDeclaringNode` for the
first exposed field they own, choosing neither. Call the accessor with an unknown id
and with each admitted non-model-object node of AC-139. Mutate only an unread selected model declaration's closed body while retaining its valid structural id and require admission when its declaration and graph joins remain valid; separately append a declaration with an unselected model owner and assert its owner-join refusal. In a crate-internal test,
replace a read node's body bounds after admission and compare the result. For
each returned type, admit a read naming its selected structural type node and a read naming the distinct structural type node for its perturbation (AC-141). Compile the API fixtures of AC-142 (exhaustive matches, a
struct literal that must fail to compile). For AC-144, read the table-building
counter (13 for 4 types, 43 for 8), find the smallest whole-read admitting
`work` limit and check its one-below selected-owner charge pointer; separately
stop one charge before the last selected table entry and check the same
model-selection pointer. A duplicate-id chain separately finds the graph-stage refusal threshold and checks that one below it stops at the unpaid structural node-id charge. Read the 1000 and 1500 chains under `bounded()`, run the
two precedence packages, read a field and an operation of the 4-chain for the
read charges, compare the smallest admitting work budgets for full-reader
frame-field, abstraction-field and relationship-edge packages whose selected
tables differ by one entry (accounting for an extra graph edge in the frame
case), and admit the cyclic documents (a self-cycle, a two-cycle, a
self-cycle with a redefinition, a type extending a cycle), comparing each table
and charge with the values recorded from the pre-change `resolve` and with
items 4 to 9.

## Expected Results

The fields, declaration order, declaring node ids, types and `None` entries equal
the selected declarations and admitted graph; `zeta` precedes `alpha`, and
Base/Mid/Leaf identities remain distinct. The absent Base and ambiguous owner
controls return the distinct typed accessor errors with no partial field
set; no accessor result invents a node id. Each accessor error is
the variant AC-139 names; a valid retained model declaration body change admits and updates the selected document value, while the crate-internal post-admission
body replacement changes nothing; a read naming the selected structural type node admits and its perturbation refuses `ill_typed`; the API
fixtures compile or fail as stated; the chains admit under a sufficient limit
and are `incomplete` at the selected-owner charge one below the whole-read
minimum, or at the model selection one below the last table charge. The duplicate-id chain is `incomplete` at its structural node-id charge one below the first budget that reaches its duplicate-id refusal. The stated precedence holds, and
the call does no resolution.

## Status

IR-1141 revised structural-id and valid-body-mutation controls are PLANNED/UNRUN.
IR-1164/IR-1165 declaration-order, genuine declaring-node identity and
typed missing/ambiguous declaring-node controls (AC-370 through AC-373 and the revised
AC-136/AC-142 portions) are PLANNED/UNRUN. The independent table and accessor controls remain implemented (IR-628) in `tests/it/checked_package_v2_model_fields.rs` (AC-136 to
AC-139, AC-141 to AC-144 over packages the reader admits, using the earlier content-derived type-node fixture, which must be updated for structural ids) and the unit tests
of `crates/quire-contract-model/src/checked_package/v2/model_fields.rs` (the
tables, charges, cycles, read charges, node-body independence) and
`model_members.rs` (the `resolve` answers recorded before the change, run
against the field tables). Frame-entry, abstraction-field-entry and
relationship-edge charges are also measured end to end in
`checked_package_v2_frame_entries.rs`, `checked_package_v2_abstraction_relation.rs`
and `checked_package_v2_model_members.rs`: each paired admitted package has a
one-entry table difference and refuses under one less than its admitting work
limit. The initial implementation did not run the mutation rows as mutations.
The branch-dominance, former name-order, ambiguity, f64-bound, equality, and stack
regressions were added in the review fix.
