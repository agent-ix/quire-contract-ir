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
  - target: ix://agent-ix/quire-contract-ir/FR-352
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

A source-declared object type node has the body `aggregate` of one
record-style field binding per field of its field view. Today a `field`
member whose declaring node has the body `aggregate{[]}` is refused as
`ill_typed`/`operator-ineligible`, and no reader joins a source-object
identity row. The reader SHALL resolve a `field` member on a source-declared
node by the binding of its name, and SHALL join the source-object identity
rows to their nodes (QSL ADR-013 QC-30 items 2a and 4; the row is QSL
FR-569's "Source-object identity rows").

## Inputs

A `field` member whose declaring node is a source-declared type; and a
producer input row `SelectedSourceObjectRow` with `structural_node`,
`source_authority`, `source_identity`, `abstract`, `supertypes`,
`identity_fields`, `members` and `populations`, supplied beside the package as
FR-352 does for its stage rows. The IR public inventory (FR-019) gains the row
type and its typed read.

## Outputs

The field's type from the binding, and the joined rows through a typed read, or
one typed located refusal.

## Behavior

The reader SHALL resolve a `field` member on a source-declared node to the
binding of its name in the node body, including an inherited field the
field view lists, and SHALL refuse a name with no binding, and any field
read on an `aggregate{[]}` source type, as it refuses an absent field on a
model-owned node.

The reader SHALL classify every supplied row before it reports. The reader reports
the first reason in the reason order of QSL FR-390-AC-67, at the `Package`
locus with no partial view and no meter charge. Foreign comes first: a row
whose `source_authority` is the authority of no `SourceOwner` in the package.
Unknown comes next: a row whose `structural_node` is not a source-declared
object type, or whose `source_identity` disagrees with that node's
`declaration`, or whose `members` or `supertypes` fail the checks below.
Conflicting follows: two rows for one (`source_authority`, `source_identity`)
key, even with equal contents, as FR-352 returns `ConflictingIdentityWitness`.
Missing comes last, after all rows are classified: a source-declared type with
no row. Each outcome is the FR-352 outcome of the same name.

The row `members` check compares, in order, each member's name, field type
node and optional presence with the node body's bindings: the binding name,
the bound type node reference and its `optional` group. The `supertypes`
check requires each entry to name an admitted source-declared type node of the
package that is not the row's own node, and the supertype relation of all rows
to be acyclic. `identity_fields` and `populations` are producer facts read only
from a joined row; the package has no `extends`, identity or population
information against which to check them.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-367-AC-1 | `self.total` on a source type with a field `total` resolves to the type of the binding, and an inherited field resolves through the field view. | Test |
| FR-367-AC-2 | A field name with no binding, and any field read on an `aggregate{[]}` source type, refuse `ill_typed`/`operator-ineligible` at the member. | Test |
| FR-367-AC-3 | A supply holding a foreign row and an unknown row reports foreign whatever their order; one holding an unknown row and a duplicate-key pair reports unknown; one holding a duplicate-key pair and an omitted type reports conflicting; one omitting a type reports missing. | Test |
| FR-367-AC-4 | A row whose `members` differ from the body bindings in name, type node, optional presence or order refuses unknown. A row whose `supertypes` name a non-source node, the row's own node, or form a cycle refuses unknown. | Test |
| FR-367-AC-5 | A complete row set exposes abstractness, supertypes, identity fields, members and populations for each source type through a typed read. | Test |
| FR-367-AC-6 | A field read on a model-owned node reads unchanged in a package that also holds a source type. | Test |

## Dependencies

- [FR-352](./FR-352-project-admitted-exact-value-types.md) owns the stage-row convention and its closed outcomes.
- QSL FR-569 owns the row content, FR-531 the file that carries it, and FR-390-AC-67 the reason order.
