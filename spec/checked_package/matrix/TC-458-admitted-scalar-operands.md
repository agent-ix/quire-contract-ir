---
id: TC-458
title: "Admitted scalar operand projection preserves positions and absence"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-054
    type: verifies
---
# TC-458: Admitted scalar operand projection preserves positions and absence

## Description

Exercise the public typed operand query on reader-admitted
`ContractPackageV1` values. Compile the existing QSL increment source through
ordinary FR-035 lowering, then query that same six-field package: QSL represents
both the parameter and the literal as graph nodes. Build the graph-reference
followed by inline-integer case and the other cases from the repository's
admitted package fixtures and reader, without copied schemas or a raw-JSON
consumer extractor.

## Test Procedure

From the fresh QSL increment source, read the lowered scalar add's two graph
references through the public typed query. Supply its exact application
occurrence from the admitted source map. Assert positions 0 and 1 in source
order, each authentic child key and its source-map entries, and an unchanged
six-field package. Separately, read a reader-admitted scalar add with a graph
reference followed by an inline integer. Assert its graph child key and
sources, exact inline signed value and occurrence-qualified position; the
inline argument has no graph-node or separate source-map identity. Build an
admitted application with the same represented child in two positions and
assert two entries with distinct positions.

Read admitted applications with a finite child range, an unbounded integer
child, a child without a readable own range and an endpoint outside `i128`.
Assert the complete argument list and the corresponding typed range facts in
each case. Compare the consumer's preexisting inferred or result-bound
interval branch separately; the IR query supplies no interval for it.

Query a missing key, a non-lowered key, a represented non-application, a
wrong operation, and an occurrence with a wrong key, role or ordinal. Inject
one inconsistent child only through the repository's test-only admitted
fault seam. Require typed non-success with no list. Query another valid
represented occurrence of the same application and observe structural
admission without asserting which QSL requirement owns it.

Compile an external consumer against the public model API and query a
reader-admitted six-field package. Check that it needs no checked graph or
`serde_json::Value` field reads. Compare the complete wire, canonical
identity, feature accounting, result context and reached closure before and
after repeated queries. Preserve the existing checked-package scalar operand
accessor's finite-range and all-or-error behavior.

## Expected Results

The QSL producer case returns both represented child keys and sources in
authored order. The reader-admitted inline case returns the authentic
occurrence and child sources, exact inline value and no invented node.
Repeated child references remain repeated positions. Missing own range facts
do not erase child identity or invent an interval. Invalid requests return
typed non-success without a selected prefix. The admitted package and its
other views remain unchanged.
