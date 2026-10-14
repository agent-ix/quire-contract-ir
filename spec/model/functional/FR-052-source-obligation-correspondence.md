---
id: FR-052
title: "Accept typed source-obligation correspondence in the neutral model"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-045
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-028
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/AD-008
    type: references
---
# FR-052: Accept typed source-obligation correspondence in the neutral model

## Description

When a selected source is prepared for proof-coverage accounting, the
`quire-contract-model` crate shall accept a typed correspondence input linking
author-authored requirement criteria and semantic obligation occurrences to
selected production symbols, and retain its complete source census for
downstream consumers before any generation or proof attempt.

## Inputs

- One selected source identity and its provenance.
- Complete typed inventories from their owners: Contract IR V2 semantic nodes
  and source occurrences; QSL checked `RequirementRecord` values and native
  pending obligations; and `quire-code-rs` selected production module and
  function facts.
- Explicit many-to-many correspondence assertions authored for the selected
  source by its source-contract author. These assertions are the authority for
  criterion-to-occurrence and occurrence-to-production-symbol links. The model
  does not derive them from QSL claim records, source mention edges, generated
  harnesses, test tags, or proof results.

## Outputs

The public `quire_contract_model::source_obligation` module shall export
`SourceObligationInput`, `SourceObligationCensus`, `SourceObligationRow`,
`SourceInventoryRow`, `SourceCorrespondenceAssertion`,
`SourceCorrespondenceRejection`, `SourceMissingMappingReason`,
`SourceInventoryDisposition`, `SourceCorrespondenceError`, and
`build_source_obligation_census`. These types shall be listed in FR-019's
public-items inventory and shall not be re-exported by `quire_contract_ir`.

The immutable census shall expose borrowed accessors for the selected source,
all inventory rows, accepted correspondence rows, and rejected assertions.
Each inventory row shall retain its typed owner identity, source provenance,
and disposition. Inventory disposition shall distinguish at least
`Mapped`, `ZeroMappings`, `ForeignInventory`, and `ConflictingIdentity`.
`ZeroMappings` shall remain a census row, not a dropped item or a fabricated
relation. Each accepted row shall retain the exact criterion,
occurrence, and production-symbol keys plus assertion provenance. Missing
mapping reasons shall be typed and distinguish at least no authored mapping
from an explicitly authored absence of a production function. A typed
top-level error shall distinguish invalid source selection, missing required
inventory, and incomplete required inventory.

## Behavior

### Neutral API authority and source provenance

The public types and constructor shall be defined in the cycle-free
`quire-contract-model` crate, under the `source_obligation` module. They shall
use only neutral model-owned identity and provenance types. The model manifest
and public API shall acquire no dependency on Contract Codegen, QSL,
`quire-code-rs`, a parser crate, or a proof runner. `quire-contract-ir` shall
not become the API owner or re-export these items. A downstream Codegen adapter
may borrow the model census and owner-native values; the dependency direction
shall remain downstream from the model.

The `SourceObligationInput` constructor shall take the selected source and
owner inventories plus the author-authored assertions as typed arguments.
Source-contract authoring is the assertion producer; the constructor validates
and preserves assertions but does not infer or repair them. Each assertion and
inventory item shall carry selected-source provenance sufficient to establish
which source selection it belongs to. The census shall retain the supplied
provenance, including provenance on rejected rows.

### Census and correspondence

The constructor shall preserve every supplied criterion, semantic occurrence,
module, and function as an inventory row, including rows with no accepted
correspondence. It shall preserve an empty module and every zero-mapping source
entity. It shall accept explicit many-to-many assertions and shall emit one
accepted correspondence row per distinct valid relation without deriving a
Cartesian product of the inventories. Exact repeated assertions shall collapse
deterministically. Exact repeated inventory entries shall collapse to one
row. When one typed identity is repeated with differing inventory facts, each
conflicting entry shall remain inspectable with `ConflictingIdentity`; none
shall be selected by first-wins order. Repeated assertion identity with
conflicting relation facts or provenance shall remain inspectable as a typed
rejection; no conflicting assertion shall be accepted. Input ordering shall
not change the disposition or ordered census.

An inventory item whose source identity or root is outside the selected source
shall remain visible with a typed foreign-inventory disposition and shall not
serve as an accepted endpoint. An assertion with a foreign source or an
endpoint absent from the selected inventories shall be retained as a rejected
assertion with a corresponding typed reason (`ForeignSource`,
`DanglingCriterion`, `DanglingOccurrence`, or `DanglingProductionSymbol`) and
shall not become an accepted relation. The census shall distinguish those
rejected relations from valid entities that have no authored relation. No
malformed or incomplete input shall produce a census that appears complete.

The constructor shall receive no method, profile, result, artifact, attempt,
or run input. Removing an obligation from its authoritative inventory may
change the census; removing a proof result or changing a method or profile
shall not change it. The model shall not qualify proof evidence, calculate
coverage projections, or alter FR-045's candidate denominator.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-052-AC-1 | A default-feature consumer imports the input, constructor, census and row types from `quire_contract_model::source_obligation`; the public-items inventory lists them, and neither the IR root nor a CG/QSL/parser dependency owns or re-exports them. | Test |
| FR-052-AC-2 | A typed fixture containing two criteria, three semantic occurrences, one empty module, two production functions and explicit many-to-many assertions preserves every inventory row and exactly the asserted valid relations with their selected-source provenance. Unmapped criteria, occurrences, modules and functions remain visible with typed missing reasons. | Test |
| FR-052-AC-3 | An assertion with a foreign source, each dangling endpoint kind, a foreign inventory entry, an exact duplicate, conflicting inventory facts, and conflicting assertion provenance each has its specified typed deterministic disposition; rejected entries remain inspectable and never enter accepted relations. | Test |
| FR-052-AC-4 | Reordering equivalent inputs yields the same ordered census and dispositions. Removing a source occurrence changes the census; changing or omitting proof results, methods, or profiles cannot change it because those inputs are absent from the constructor. | Test |
| FR-052-AC-5 | Inspection of the three owner APIs confirms the input inventory facts retain their native identities and are not copied into a second parser/schema; a downstream consumer can borrow the census without introducing a dependency from IR root or `quire-contract-model` to CG or QSL. | Inspection |

## Dependencies

[FR-045](../../core/functional/FR-045-derive-proof-coverage-baseline.md)
owns candidate eligibility, evidence joins, and projections;
[AD-008](../../assurance/AD-008-proof-coverage-accounting.md) owns their
accounting meaning. [FR-019](./FR-019-rust-library-interface.md) owns the
model crate public-items inventory, and [FR-028](./FR-028-separate-cycle-free-contract-model.md)
establishes the dependency boundary. The typed inputs retain owner-native
identities from the IR V2, QSL, and `quire-code-rs` APIs; this requirement
defines no replacement parser or owner schema.
