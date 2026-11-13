---
id: FR-364
title: "Admit a source population reference node"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-362
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/ADR-013
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-569
    type: references
  - target: ix://agent-ix/quire-specification/FR-322
    type: references
---

# FR-364: Admit a source population reference node

## Description

QSpec FR-322 admits a `relation`/`population` population reference only with a
selected-document owner (FR-322-AC-82/83). The reader SHALL also admit a
`relation`/`population` node that carries a `declaration`
`{qualified_name: [<population name>]}` and a `SourceOwner`, for a
source-declared population (QSL ADR-013 QC-30 items 2c and 4).

## Inputs

A `relation`/`population` node with a `declaration` and a `SourceOwner`, and
its use in `all_instances` and `lookup`.

## Outputs

The admitted node as a self-typed population reference, or one typed located
refusal.

## Behavior

The reader SHALL admit a population reference node with a `declaration` and a
`SourceOwner` as a declared node keyed by its structural id, without a selected
document. The reader SHALL admit a source population reference in each
position where it admits a selected-document population reference, and SHALL
refuse it in each position where it refuses one, with the refusal of the
model-owned check. The reader SHALL refuse a population reference node that
carries a `declaration` and a `ModelOwner` of no selected declaration, and one
that carries a `SourceOwner` and no `declaration`, with the code, cause and
locus FR-038 gives today for a population node with that owner and declaration
defect. A node has one owner, so a node with both owner kinds is not
representable. A source population that no checked node names has no node and
the reader requires none.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-364-AC-1 | A source population reference node with a `declaration` and a `SourceOwner` admits, is self-typed, and is a distinct node per population name. | Test |
| FR-364-AC-2 | `all_instances` and `lookup` over the source population admit, with the same dependencies and result typing as over a declared model population. | Test |
| FR-364-AC-3 | The forbidden positions of TC-229 (collection operand, reaches link, forbidden declaring member positions) refuse for a source population with the code, cause and locus they give for a model population. | Test |
| FR-364-AC-4 | A population node with a `declaration` and a `ModelOwner` of no selected declaration, and one with a `SourceOwner` and no `declaration`, each refuse with the FR-038 code, cause and locus for that defect. | Test |
| FR-364-AC-5 | Two source populations of distinct names give distinct keys; one name repeated refuses with the FR-038 duplicate-declaration code, cause and locus. | Test |

## Dependencies

- [FR-362](./FR-362-read-source-declared-object-types.md) defines the source-declared type.
- [TC-229](../matrix/TC-229-declared-population-reference-reader.md) verifies the model-owned counterpart.
