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
selected-document owner (the `DeclarationTagRules` of FR-322-AC-82/83). The
reader SHALL also admit a `relation`/`population` node that carries a
`declaration` `{qualified_name: [<population name>]}` and a `SourceOwner`, for a
source-declared population (QSL ADR-013 QC-30 items 2c and 4).

## Inputs

A `relation`/`population` node with a `declaration` and a `SourceOwner`, and its
use in `all_instances` and `lookup`.

## Outputs

The admitted node as a self-typed population reference, or one typed located
refusal.

## Behavior

The reader SHALL admit a population reference node with a `SourceOwner` as a
declared node, keyed by its structural id, without a selected document. A
population reference with a `SourceOwner` SHALL be admitted in the positions
where a selected-document population reference is admitted, and SHALL be
refused in each position where one is refused. A population reference with
neither a selected-document owner nor a `SourceOwner`, and one carrying both,
SHALL refuse as the ownerless case refuses today. A source population that no
checked node names has no node and the reader requires none.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-364-AC-1 | A source population reference node with a `declaration` and a `SourceOwner` admits, is self-typed, and is readable as a distinct node per population name. | Test |
| FR-364-AC-2 | `all_instances` and `lookup` over the source population admit, with the same dependencies and result typing as over a declared model population. | Test |
| FR-364-AC-3 | The forbidden positions of TC-229 (collection operand, reaches link, forbidden declaring member positions) refuse for a source population exactly as for a model population. | Test |
| FR-364-AC-4 | A population reference node with no owner, and one with both a selected-document owner and a `SourceOwner`, refuse. | Test |
| FR-364-AC-5 | Two source populations of distinct names give distinct keys; one name repeated refuses as a duplicate declaration. | Test |

## Dependencies

- [FR-362](./FR-362-read-source-declared-object-types.md) defines the source-declared type.
- [TC-229](../matrix/TC-229-declared-population-reference-reader.md) verifies the model-owned counterpart.
