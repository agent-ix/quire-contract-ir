---
id: FR-365
title: "Read operation effects for a source-declared operation"
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

# FR-365: Read operation effects for a source-declared operation

## Description

The `operation_effects` carrier of [FR-038](./FR-038-consume-checked-package-v2.md)
requires, for each row, a selected `document`, a `declaring` node that joins a
selected model and the single provenance origin
`model_document_operation_frame`. The reader SHALL also admit a row whose
`declaring` node is a source-declared object type, with `provenance.origin`
`source_declaration_operation_frame` (QSL ADR-013 QC-30 items 2e and 4).

## Inputs

An `operation_effects` row `{document, declaring, operation, effect,
provenance}` with `declaring` naming a source-declared type, a
`provenance.origin` of `source_declaration_operation_frame`, and the source
operation's frame.

## Outputs

An admitted row exposed through the existing borrowed typed read in carrier
order with its origin, or one typed located refusal.

## Behavior

For a source row the reader SHALL:

1. require `document` to be a string equal to the `identity` of the declaring
   node's `SourceOwner`; the field keeps its string type and the row sort key
   `(document, declaring, operation)` in UTF-8 byte order is unchanged;
2. require `provenance.member` to equal `<type>::<operation>` for the declaring
   type's source name and the row's `operation`;
3. require `operation` to be an operation the source type itself declares, and
   exactly one row per own operation of every source-declared type, including
   a query (four empty arrays); an inherited-only operation has no row under
   the subtype;
4. check `effect` against the source operation's frame, not a selected model
   row.

A `source_declaration_operation_frame` origin on a row whose declaring node is
model-owned, and `model_document_operation_frame` on a source row, SHALL
refuse. The model-owned row rules are unchanged. The typed read SHALL expose
the origin as a closed two-variant value.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-365-AC-1 | A package with a source object type declaring a modifying operation and a query, and a carrier with one `source_declaration_operation_frame` row per operation, admits; the typed read returns both rows in carrier order with origin, `document`, `declaring`, `operation`, four effect collections and `provenance.member` `<type>::<operation>`. | Test |
| FR-365-AC-2 | A `document` that differs from the declaring node's `SourceOwner` `identity` refuses. | Test |
| FR-365-AC-3 | A wrong `provenance.member`, an origin mismatched to the owner kind in either direction, a row naming an operation the type does not declare, an inherited-only operation, and a missing row for an own operation each refuse. | Test |
| FR-365-AC-4 | An `effect` that differs from the source frame in any of the four members refuses. | Test |
| FR-365-AC-5 | A source row and a model row with equal `document` strings and distinct `declaring` nodes coexist and sort by the unchanged key. | Test |
| FR-365-AC-6 | Model-owned rows with `model_document_operation_frame` read unchanged when a source row is present in the same carrier. | Test |

## Dependencies

- [FR-038](./FR-038-consume-checked-package-v2.md) owns the carrier; [FR-354](./FR-354-project-admitted-runtime-context.md) consumes it.
- [FR-362](./FR-362-read-source-declared-object-types.md) defines the source-declared type.
