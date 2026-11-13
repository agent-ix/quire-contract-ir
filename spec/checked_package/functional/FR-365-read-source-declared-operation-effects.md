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
admits one provenance origin, `model_document_operation_frame`, and joins each
row to a selected model. The reader and encoder SHALL also admit a row whose
`declaring` node is a source-declared object type, with `provenance.origin`
`source_declaration_operation_frame` (QSL ADR-013 QC-30 items 2e and 4). IR
admits this origin ahead of QSpec's closed schema; the model-row vectors are
unchanged.

## Inputs

An `operation_effects` row `{document, declaring, operation, effect,
provenance}` with `declaring` naming a source-declared type. The encoder
accepts the same row through `CheckedPackageSidecarsRef` with a typed origin
field.

## Outputs

An admitted row exposed through the existing borrowed typed read in carrier
order, with a closed two-variant origin, or one typed located refusal.

## Behavior

A source type has no node per operation and the package binds only its fields,
so the reader checks only what the package supports. For a source row the
reader SHALL:

1. require `document` to be a string equal to the `identity` of the declaring
   node's `SourceOwner`; the field keeps its string type and the sort key
   `(document, declaring, operation)` in UTF-8 byte order is unchanged;
2. require `provenance.member` to equal `<type>::<operation>`, `<type>` being
   the final segment of the declaring node's `declaration` and `<operation>`
   the row's `operation`;
3. require the origin to match the declaring node's owner kind, in both
   directions;
4. where a `state`/`operation_anchor` in the package has the declaring node as
   its `context` and the row's `operation` as its `operation`, check `effect`
   against that anchor's frame with the matching model-row check.

The reader SHALL NOT require a row for each own operation of a source type,
distinguish an own from an inherited operation, or check `effect` of an
operation that no anchor names: the package and the source-object row carry no
operation list, so coverage is a producer fact. Every row-level refusal SHALL
use the code, cause and locus of the matching model-row check of FR-038. The
model-owned row rules, and the model-row coverage rule, are unchanged.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-365-AC-1 | A package with a source type declaring a modifying operation and a query, with an anchor for the modifying operation, and a carrier with one `source_declaration_operation_frame` row per operation, admits; the typed read returns both rows in carrier order with origin, `document`, `declaring`, `operation`, four effect collections and `provenance.member`. The encoder writes the same bytes from the typed rows. | Test |
| FR-365-AC-2 | A `document` that differs from the declaring node's `SourceOwner` `identity` refuses with the matching model-row check's code, cause and locus. | Test |
| FR-365-AC-3 | A wrong `provenance.member` and an origin mismatched to the owner kind, in either direction, each refuse as the matching model-row check does. | Test |
| FR-365-AC-4 | A row whose `effect` differs from the frame of the anchor that names its operation in any of the four members refuses. | Test |
| FR-365-AC-5 | A source row and a model row with equal `document` strings and distinct `declaring` nodes coexist and sort by the unchanged key; a repeated key refuses as the model carrier does. | Test |
| FR-365-AC-6 | Model-owned rows with `model_document_operation_frame` read unchanged when a source row is in the same carrier, and a carrier with a source type and no source row admits. | Test |

## Dependencies

- [FR-038](./FR-038-consume-checked-package-v2.md) owns the carrier and, with FR-019-AC-21, the public inventory this extends; [FR-354](./FR-354-project-admitted-runtime-context.md) consumes it.
- [FR-362](./FR-362-read-source-declared-object-types.md) defines the source-declared type.
