---
id: FR-367
title: "Join source-object identity rows and resolve fields on source object nodes"
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

# FR-367: Join source-object identity rows and resolve fields on source object nodes

## Description

A source-declared object type node has the body `aggregate` of one record-style
field binding per field of its effective view. Today a field member whose
declaring node has the empty body `aggregate{[]}` is refused as
`ill_typed`/`operator-ineligible`, and no reader joins a source-object identity
row. The reader SHALL resolve a `field` member on a source-declared node by the
binding of its name, and SHALL join the source-object identity rows of the
reader view to their nodes (QSL ADR-013 QC-30 items 2a and 4, FR-569
"Source-object identity rows").

## Inputs

A `field` member whose declaring node is a source-declared type; a producer
input row `SelectedSourceObjectRow` carrying `structural_node`,
`source_authority`, `source_identity`, `abstract`, `supertypes`,
`identity_fields`, `members` and `populations`, supplied alongside the package
as FR-352 does for other stage rows.

## Outputs

The field's type from the binding, and the joined row, or one typed located
refusal.

## Behavior

A `field` member on a source-declared node SHALL resolve to the binding of its
name in the node body, including an inherited field the effective view lists,
and a name with no binding SHALL refuse as it refuses for a model-owned node.
A source type with no field has `aggregate{[]}` and every field read on it
SHALL refuse.

A row SHALL be classified in this order and the first failure SHALL refuse with
no partial view: a row whose `source_authority` is the authority of no
`SourceOwner` in the package is a foreign witness; a row whose `structural_node`
is not a source-declared object type, or whose `source_identity` disagrees with
that node's `declaration`, is an unknown witness; a second row for one
(`source_authority`, `source_identity`) key is a conflicting witness; a node
with no row is a missing witness. A row's `members` SHALL equal the node body's
bindings in order, its `supertypes` SHALL name the node's direct supertype
nodes, and its `identity_fields` and `populations` SHALL be readable only for a
joined row.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-367-AC-1 | `self.total` on a source type with a field `total` resolves to the type of the binding, and an inherited field resolves through the effective view. | Test |
| FR-367-AC-2 | A field name with no binding, and any field read on an `aggregate{[]}` source type, refuse `ill_typed`/`operator-ineligible` at the member. | Test |
| FR-367-AC-3 | A row whose key is foreign, whose node is not a source object type, or whose `source_identity` disagrees with the declaration, and a duplicate-key row, each refuse in the stated order. | Test |
| FR-367-AC-4 | A row whose `members` differ from the body bindings, or whose `supertypes` are not the node's direct supertypes, refuses. | Test |
| FR-367-AC-5 | A complete row set exposes abstractness, supertypes, identity fields, members and populations for each source type through a typed read. | Test |
| FR-367-AC-6 | A field read on a model-owned node reads unchanged. | Test |

## Dependencies

- [FR-352](./FR-352-project-admitted-exact-value-types.md) owns the stage-row convention.
- QSL FR-390 and FR-531 own the row content and file.
