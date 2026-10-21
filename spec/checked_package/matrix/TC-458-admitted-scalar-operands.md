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
`ContractPackageV1` values. Build the positive through the existing QSL
increment source and ordinary FR-035 lowering, then read that same six-field
package. Other cases use the repository's admitted package fixtures and
reader, without copied schemas or a raw-JSON consumer extractor.

The QSL producer-origin positive is **Planned** pending
[QSL-940](https://linear.app/agent-ix/issue/QSL-940): the current increment emission
represents the authored integer literal as a graph reference in the scalar
application, while QSL FR-357/FR-358 and ADR-013 require an inline literal in
that operand identity. The reader-admitted fixture exercises the specified
graph-reference plus inline-integer shape now. Do not count a QSL producer
trace for this case until that emission is corrected and the six-field
package is read through the public typed query.

## Test Procedure

Read a lowered scalar add with a graph-reference first argument and inline
integer second argument. Supply its exact application occurrence from the
admitted source map. Assert ordered positions, graph child key and sources,
inline signed value and occurrence-qualified position. Assert that the inline
argument has no graph-node or separate source-map identity. Build an admitted
application with the same represented child in two positions and assert two
entries with distinct positions.

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

The positive returns the authored order, authentic occurrence and child
sources, exact inline value and no invented node. Repeated child references
remain repeated positions. Missing own range facts do not erase child
identity or invent an interval. Invalid requests return typed non-success
without a selected prefix. The admitted package and its other views remain
unchanged.
