---
id: TC-473
title: "Reached map rows on the shared native budget"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-357
    type: verifies
  - target: ix://agent-ix/quire-contract-ir/FR-358
    type: verifies
---
# TC-473: Reached map rows on the shared native budget

## Description

Verify that reached map rows expose and retain the admitted key and value
structural nodes and complete producer Exact map value, and charge selection,
row materialization and nested projection to one native budget.

## Test Procedure

Strictly admit a fixture with a selected monitor root whose reached types
include a map with an admitted Text key and record value. Supply its complete producer
collection row with distinct structural, base, key and value node handles and
its Exact map type. Put an unrelated unreached map row before that row in the
original collection channel. Join the map through the public source-collection
row acquisition and Exact projection path, then exercise the selected monitor
closure with the corresponding real rows. Compare every retained node handle,
full Exact map value, key-then-value edge order, reached record members and
selected map-row census with independently stated expectations. Exercise the
existing sequence, set, bag and ordered-set controls with no key node.

Repeat with the reached key absent, foreign or replaced by its value node,
and with a key supplied on a non-map collection. Assert the existing closure
`CollectionKey` code and the collection node, with no partial closure. Supply
a scalar Exact value through the collection row channel and assert the
existing `MismatchedIdentityWitness` refusal instead of a successful map.
Do not treat a general scalar Exact projection as a collection-row request.

Run the valid ordered selection, row acquisition and projection on one shared
native budget and record its successful work and logical storage. Reproduce
that value at the measured exact combined allowances. Repeat with each
allowance one unit lower, retaining a genuine paid prefix before the failed
map charge. Independently identify the expected original row index and key or
value structural node; do not derive the expected locus from the refusal
being asserted. Observe that row selection uses its original `ReachedRow`
index and that a preceding public caller charge retains its supplied native
locus without changing the following native read's allowance. With all fixture
inputs constructed before measurement, count allocations during a zero-storage denial.

## Expected Results

The valid row and closure retain the exact complete map value and its separate
key/value structural nodes in admitted order. Only the reached map row enters
the result, while existing non-map collection values are unchanged. Invalid
key joins and the non-collection/non-map row retain their specified typed
refusals without a partial result.

Exact shared allowances succeed. One-under allowances retain the first unpaid
kind, full amount, original bound and independently expected row/key/value
locus, preserve the consumed prefix and clear the operation's success delta.
No fresh budget or charge replay hides the denial. Zero-storage denial
allocates nothing, including owned diagnostic material, and returns the
existing compact typed refusal. The later reached row retains its original
channel index rather than a compacted selected-row index.
