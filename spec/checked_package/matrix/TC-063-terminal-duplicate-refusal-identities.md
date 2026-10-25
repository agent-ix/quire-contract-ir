---
id: TC-063
title: "Terminal duplicate refusal carries both typed identities"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-062
    type: verifies
---

# TC-063: Terminal duplicate refusal carries both typed identities

## Description

Verify [FR-062](../functional/FR-062-carry-terminal-duplicate-identities.md)
through the public V2 reader and refusal type, using IR-owned vectors for
QSpec FR-476's two duplicate forms.

## Test Procedure

Read a valid `any` terminal on model `M`. Add a second valid terminal on
`M` with different mode or condition, rebuild both node and package
identities, and read the package. Repeat with the source order of the two
terminal declarations reversed while retaining their node identities. For
the occurrence case, add a second and then a third `claim` occurrence with
matching source-map entries to one otherwise valid terminal node. Read
each package and inspect its code, cause, path, locus and related payload.

As controls, read either terminal node alone; a nonterminal ambiguous-name
case; and packages where a strict-wire, graph-shape, key, source-map, body or
target defect coexists with a duplicate. Compile an external consumer that
matches both related-payload variants using only the public model types.

## Expected Results

Each single-terminal control admits. The two-node duplicate returns the
`Nodes` case with the same first and second node ids under both source
orders, the second as the primary locus. The same-node duplicate returns
the `Occurrences` case with that node and its first two checked occurrences;
adding a third leaves the pair unchanged. Earlier defects win with no
related payload. The external consumer distinguishes the cases without
body or prose parsing, and no terminal duplicate populates
`document_pointer` or another unrelated field.
