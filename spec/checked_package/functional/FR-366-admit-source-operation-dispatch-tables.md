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

The reader validates a `checked_term_data.dispatch_tables` row against
selected model owners. It SHALL also admit a row whose receiver `context` and
`operation.declaring` are source-declared object type nodes, whose candidate
`selection_key` names a source-declared type node, and whose candidates'
`operation_body` and `effective_precondition` name source clause function nodes
(QSL ADR-013 QC-30 items 2d, 2f and 4; the row shape is QSpec FR-322).

## Inputs

A dispatch-table row `{application, occurrence, operation, context,
candidates}` whose `context` is a source-declared type, and the source clause
function nodes.

## Outputs

The admitted row through the existing dispatch read, or one typed located
refusal.

## Behavior

The strict reader SHALL join `context` and `operation.declaring` through their
`SourceOwner`, not through a selected model owner. It SHALL require the call's
member declaration to equal `context` as it does for a model row, every
candidate's `selection_key.original` to name a source-declared type node, each
`operation_body` to name a clause function node with `declaration`
`[<type>, <operation>, "body"]` and each `effective_precondition`, when
present, to name one with `[<type>, <operation>, "effective_precondition"]`,
every such node carrying the same `SourceOwner`, and candidate order and
witnesses to follow the model row's checks. The refusal code, cause and locus
of each failure SHALL be those of the matching model-row check.

The candidates of a source context are the context type or one of its
subtypes, since a redefining subtype is a valid candidate. The package carries
no `extends` information, so the strict reader does not check that family. The
family check runs in the source-object read of
[FR-367](./FR-367-join-source-object-rows-and-field-reads.md), which holds the
supertype rows: it SHALL require each candidate's `selection_key.original` to
be the context or a transitive subtype of it, and refuse otherwise with the
FR-352 outcome for a stale or mismatched stage row. A row mixing a
source-declared `context` and a model-owned `operation.declaring`, or the
reverse, SHALL refuse as a model row whose context and declaring family differ
does. The reader SHALL admit the three clause-function `declaration` forms of one
package, with the authored-precondition form
`[<type>, <operation>, "precondition", <label>]`, as sibling clause functions under the existing
one-holder-per-qualified-name rule, with no change to the qualified-name
identifier rule.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-366-AC-1 | A package with a source type `Order`, a redefining subtype `Sub` and a dispatched call on `Order` whose candidates are `Order` and `Sub` admits, and the dispatch read returns the context, candidate order and clause identities in the model row's order. | Test |
| FR-366-AC-2 | The three clause-function declaration forms for one operation, with an authored-precondition label, admit together and each qualified-name segment is an identifier. | Test |
| FR-366-AC-3 | A candidate `selection_key` naming a non-source node, an `operation_body` or `effective_precondition` naming a node of another declaration form or another operation, and a clause node with a different `SourceOwner` each refuse with the matching model-row check's code, cause and locus. | Test |
| FR-366-AC-4 | A row mixing a source context and a model-owned declaring family, and the reverse, refuses. | Test |
| FR-366-AC-5 | A model-owned dispatch table reads unchanged when a source table is in the same package. | Test |
| FR-366-AC-6 | In the FR-367 read with its supertype rows, a candidate that is neither the context nor a transitive subtype of it refuses, and `Sub` for context `Order` admits. | Test |

## Dependencies

- [FR-367](./FR-367-join-source-object-rows-and-field-reads.md) supplies the supertype rows for the family check.
- QSpec FR-322 owns the dispatch row.
