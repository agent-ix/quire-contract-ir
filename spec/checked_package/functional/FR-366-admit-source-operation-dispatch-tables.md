---
id: FR-366
title: "Admit dispatch tables over source-declared operations"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-362
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-367
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/ADR-013
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-569
    type: references
  - target: ix://agent-ix/quire-specification/FR-322
    type: references
---

# FR-366: Admit dispatch tables over source-declared operations

## Description

The reader validates a `checked_term_data.dispatch_tables` row against selected
model owners. It SHALL also admit a row whose receiver context and candidate
declaring types are source-declared object type nodes, and whose candidate
operation-body and effective-precondition identities are source clause
function nodes (QSL ADR-013 QC-30 items 2d, 2f and 4).

## Inputs

A dispatch-table row, FR-494's shape, whose receiver context and each
candidate's declaring type are source-declared `model`/`object_type` nodes; the
clause function nodes of the source operation.

## Outputs

The admitted row, exposed through the existing dispatch read, or one typed
located refusal.

## Behavior

For a source context and declaring family the reader SHALL join each node
through its `SourceOwner` and its source-object row (FR-367), not through a
selected model owner. It SHALL require every candidate's declaring type to be
the context or a source supertype of it, each operation-body and
effective-precondition identity to name a clause function node whose
`declaration` is `[<type>, <operation>, "body"]` or
`[<type>, <operation>, "effective_precondition"]` and whose owner is the same
`SourceOwner`, and candidate order and witnesses to follow FR-494. A mixed row
whose context is source-declared and a candidate declaring type is model-owned,
or the reverse, SHALL refuse. Clause functions of one package with the three
`declaration` forms of QC-30 item 2d, `[<type>, <operation>, "precondition",
<label>]` included, SHALL admit as sibling clause functions under the existing
one-holder-per-qualified-name rule with no change to the qualified-name
identifier rule.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-366-AC-1 | A package with a source type, a redefining subtype operation and a dispatched call admits, and the dispatch read returns the context, candidate order, declaring types and clause identities in FR-494 order. | Test |
| FR-366-AC-2 | The three clause-function declaration forms for one operation, with an authored precondition label, admit together and each qualified-name segment is an identifier. | Test |
| FR-366-AC-3 | A candidate declaring type that is not the context or a supertype, a clause identity naming a node of another declaration form or another operation, and a clause node with a different `SourceOwner` each refuse. | Test |
| FR-366-AC-4 | A row mixing a source context and a model-owned candidate refuses, and the reverse refuses. | Test |
| FR-366-AC-5 | A model-owned dispatch table reads unchanged when a source table is in the same package. | Test |

## Dependencies

- [FR-367](./FR-367-join-source-object-rows-and-field-reads.md) supplies the source-object join.
- QSpec FR-494 owns the dispatch row.
