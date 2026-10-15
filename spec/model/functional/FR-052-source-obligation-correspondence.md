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

The complete census and its owner-output oracle remain Planned/UNRUN. IR-784
provides only the provisional typed input port in this repository. That port
retains reported rows and authored assertions; it does not accept a
completeness attestation or publish a census.

IR-794 defines the narrower owner-issued enumeration outcome that the neutral
model may retain before the census exists. It does not make a caller-supplied
row vector, count, or absent stop reason into evidence of exhaustion.

## Inputs

- One selected source identity and its provenance.
- Complete typed inventories from their owners: Contract IR V2 semantic nodes
  and source occurrences; QSL checked `RequirementRecord` values and native
  pending obligations; and `quire-code-rs` selected production module and
  function facts.
- QSL checked requirement identities are projections of the keys of
  `CheckedGraph::requirements()`: the existing wire node ID bytes and the
  occurrence origin's role and ordinal. They are not `RequirementRef` values.
  Native pending-obligation identities project the exact `Obligation.site`
  declaration, unit, expression and byte span together with `Obligation.kind`,
  including the `PredicateTotal` target. Two identical native site/kind tuples
  denote one inventory identity; a different field denotes another identity.
- Explicit many-to-many correspondence assertions authored for the selected
  source by its source-contract author. These assertions are the authority for
  criterion-to-occurrence and occurrence-to-production-symbol links. The model
  does not derive them from QSL claim records, source mention edges, generated
  harnesses, test tags, or proof results.

## Outputs

On completion, the public `quire_contract_model::source_obligation` module shall export
`SourceObligationInput`, `SourceObligationCensus`, `SourceObligationRow`,
`SourceInventoryRow`, `SourceCorrespondenceAssertion`,
`SourceCorrespondenceRejection`, `SourceMissingMappingReason`,
`SourceInventoryEnvelope`, `SourceInventoryDisposition`, `SourceInventoryCompleteness`,
`SourceInventoryIncompleteReason`, `SourceInventoryOwner`,
`SourceInventoryScope`, `SourceCorrespondenceError`, and
`build_source_obligation_census`. These types shall be listed in FR-019's
public-items inventory when implemented. The provisional IR-784 port exports
only `SourceObligationInput`, `SourceInventoryEnvelope`, `SourceInventoryRow`,
typed owner identities and stop causes, and authored assertion/provenance
types. FR-019 lists that current public surface exactly.

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
top-level error shall distinguish missing owner inventory, incomplete owner
inventory, inventory source mismatch, inventory scope mismatch, and an
inventory item-count mismatch. Its owner context shall identify IR V2, QSL,
or `quire-code-rs`; incomplete errors shall retain the typed owner cause.
`SourceCorrespondenceRejection` shall distinguish `ForeignSource`,
`DanglingCriterion`, `DanglingOccurrence`, `DanglingProductionSymbol`,
`ConflictingRelation`, and `ConflictingProvenance` reasons.

## Behavior

### Neutral API authority and source provenance

The public types and constructor shall be defined in the cycle-free
`quire-contract-model` crate, under the `source_obligation` module. They shall
use neutral model-owned identity and provenance types.

The `SourceObligationInput` constructor shall take the selected source, three
independently optional owner inventory envelopes, and the author-authored
assertions as typed arguments. An envelope shall identify its owner, selected
source, covered scope, item count, typed completeness state, and inventory
items. The three required scopes shall be IR V2's complete semantic graph and
source-occurrence set, QSL's complete checked `RequirementRecord` set and
native pending-obligation set, and `quire-code-rs`'s complete selected
production-module and function facts. Each envelope's owner, source, and scope
shall match its required input position and the top-level selected source.

`SourceInventoryCompleteness` shall be either `Complete` or `Incomplete` with
a `SourceInventoryIncompleteReason`. The owner-native inventory enumerator or
its typed adapter shall issue `Complete` only after normal exhaustion of the
entire named scope; an early stop, failed parse, failed owner request, or
unavailable selected module shall issue `Incomplete` with the original typed
cause. QSL type-work exhaustion shall retain its native dimension, limit,
prior, requested charge and site in a dedicated typed cause. QSL
`TypeDisposition::Unfinished` shall be a different typed cause keyed by its
native declaration handle. This state is evidence from enumeration, not a
caller-authored boolean or eligibility policy. The envelope item count shall equal the number of
retained items; a mismatch shall refuse construction. `Complete` with zero
items is a valid complete inventory when the owner enumerated the named scope
and found no entries. A complete envelope may also contain an empty component
inventory, such as a fully parsed production module with no functions. These
cases are distinct from an absent envelope, an incomplete envelope, and an
inventoried entity whose correspondence count is zero.

For the IR-794 boundary, an owner-issued enumeration outcome shall carry the
selected `SourceIdentity`, its typed `SourceInventoryOwner` and matching
`SourceInventoryScope`, all rows emitted for that selection, an observed row
count, and exactly one terminal state: `Complete` or `Incomplete` with the
original typed stop cause. The outcome shall be issued at the terminal branch
of the owner-native enumerator or its typed adapter, not by a constructor that
accepts an arbitrary row slice and a caller-chosen completion flag. The adapter
shall enumerate the complete named scope from its selected owner object; it
shall retain every emitted row and advance to the owner's normal end before
issuing `Complete`. Any early return, cancellation, capacity refusal, parser
failure, missing selected module, or unfinished type admission shall issue
`Incomplete` and retain the original typed cause, including rows emitted before
the stop. In particular, QSL type-work exhaustion and
`TypeDisposition::Unfinished` remain separate causes. An adapter must not
translate `stop_reason: None` on the provisional envelope into `Complete`.

The neutral stop projection is closed over owner paths that actually exist.
QSL's current `qsl_inventory` projects `TypeReport::exhaustion()` to
`SourceInventoryIncompleteReason::TypeWorkExhausted` with dimension, limit,
prior, requested charge and site; it projects `TypeDisposition::Unfinished`
to `TypeAdmissionUnfinished` with the declaration handle. An IR V2 reader
limit shall retain the existing model-owned `CheckedPackageIncomplete`
payload (`CheckedPackageLimit`, limit, consumed and optional `JsonPointer`)
in a distinct `IrV2ReadLimit` cause, never a message string. Where an owner
actually accepts a `quire_exact::Cancel` and reports its `CancelCause`, its
adapter shall project `Requested` and `Deadline` exhaustively to a neutral
`SourceInventoryCancelCause` carried by a distinct
`Cancelled { owner, cause }` incomplete variant. This projection adds no
`quire-exact`, QSL or `quire-code-rs` dependency to the model. The current
QSL inventory and `quire-code-rs` selection API do not emit such a
cancellation outcome, so neither adapter may synthesize one. A future
owner-specific capacity or stop type shall get a distinct typed neutral
projection and a field-preserving adapter test before it can be reported as
owner-issued `Incomplete`; it shall not be collapsed into `EarlyStop`,
`RequestFailed { cause: Box<str> }` or a display string. Existing textual
parse/request/module causes remain provisional owner reports, not proof of
lossless projection for a new owner error type.

The model shall preserve the owner-issued terminal state, validate the owner,
scope, source and observed row count against its retained rows, and reject a
mismatch. The model cannot independently attest that an external owner
enumerated every row: owner issuance is an obligation of the native enumerator
or typed adapter, verified against that owner's full selected output. A public
model type or count is not an authenticity credential. IR-794 does not grant
`build_source_obligation_census` permission to claim all three owners complete;
the matched-source, three-owner census and correspondence checks remain under
the later FR-052 acceptance criteria.

The provisional port has no `SourceInventoryCompleteness`, caller supplied
item count, or complete-census constructor. Its envelope carries owner,
source, named scope, reported rows, and an optional typed `stop_reason`.
`stop_reason: None` means only that no stop cause was reported; it is never
proof of normal exhaustion or complete owner output. The provisional input
retains the selected source, optional owner envelopes, and authored assertions
without accepting or rejecting any correspondence.
Source-contract authoring is the assertion producer; the constructor validates
and preserves assertions but does not infer or repair them. Each assertion and
inventory item shall carry selected-source provenance sufficient to establish
which source selection it belongs to. The census shall retain the supplied
provenance, including provenance on rejected rows.

If any owner envelope is absent, incomplete, has a mismatched scope or source,
or has an item-count mismatch, `build_source_obligation_census` shall return a
typed error naming that owner and shall return no census. It shall report the
first failing owner in fixed order: IR V2, QSL, then `quire-code-rs`. The
constructor shall not treat a missing envelope or partial owner enumeration as
a zero-mapping entity or a complete empty inventory. A census is complete only
when all three required owner envelopes pass these checks.

`SourceInventoryOwner` shall distinguish `IrV2`, `Qsl`, and `QuireCodeRs`.
`SourceInventoryScope` shall distinguish `IrV2SemanticGraphAndOccurrences`,
`QslCheckedRequirementsAndPendingObligations`, and
`QuireCodeRsProductionModulesAndFunctions`. Each owner shall supply exactly
its corresponding scope. `SourceCorrespondenceError` shall expose distinct
variants `MissingOwnerInventory`, `IncompleteOwnerInventory`,
`InventorySourceMismatch`,
`InventoryScopeMismatch`, and `InventoryItemCountMismatch`, retaining the
owner and the relevant expected and observed source, scope, or counts, and the
typed incomplete cause where applicable.

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
conflicting relation facts or provenance shall remain inspectable with
`ConflictingRelation` or `ConflictingProvenance`, respectively; no conflicting
assertion shall be accepted. Input ordering shall not change the disposition
or ordered census.

An inventory item whose source identity or root is outside the selected source
shall remain visible with a typed foreign-inventory disposition and shall not
serve as an accepted endpoint. An assertion with a foreign source or an
endpoint absent from the selected inventories shall be retained as a rejected
assertion with a corresponding typed reason (`ForeignSource`,
`DanglingCriterion`, `DanglingOccurrence`, or `DanglingProductionSymbol`) and
shall not become an accepted relation. The census shall distinguish those
rejected relations from valid entities that have no authored relation. No
malformed or incomplete input shall produce a census that appears complete.

The source-obligation input schema shall contain no method, profile, result,
artifact, attempt, or run field. Census membership is derived only from the
selected source inventories and explicit assertions. The model shall not
qualify proof evidence, calculate coverage projections, or alter FR-045's
candidate denominator.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-052-AC-1 | A default-feature consumer imports and calls the input, constructor, census and row types from `quire_contract_model::source_obligation`; FR-019's public-items inventory lists every exported item. | Test |
| FR-052-AC-2 | A typed fixture containing two criteria, three semantic occurrences, one empty module, two production functions and explicit many-to-many assertions preserves every inventory row and exactly the asserted valid relations with their selected-source provenance. Unmapped criteria, occurrences, modules and functions remain visible with typed missing reasons. Reordering equivalent inputs yields the same ordered census; removing one occurrence from a complete IR V2 inventory changes it. | Test |
| FR-052-AC-3 | A foreign source, each dangling endpoint, a foreign inventory entry, an exact duplicate, conflicting inventory facts, conflicting assertion relation facts, and conflicting assertion provenance each receive their named deterministic disposition; rejected entries remain inspectable and never enter accepted relations. | Test |
| FR-052-AC-4 | Inspection of the public constructor and `SourceObligationInput` signature confirms that the input schema has no result, method, profile, artifact, attempt or run field. | Inspection |
| FR-052-AC-5 | An external default-feature consumer compiles a borrow of `SourceObligationCensus` and its rows from `quire_contract_model::source_obligation`. | Test |
| FR-052-AC-6 | Omitting each of the three owner envelopes in turn returns `MissingOwnerInventory` naming that owner and no census. Marking each envelope incomplete returns `IncompleteOwnerInventory` with the supplied typed cause and no census. A fully parsed production module with no function facts is accepted as complete, retains the module row, and differs observably from both outcomes. | Test |
| FR-052-AC-7 | Complete IR V2 scope covers every semantic node and source occurrence in the selected V2 graph; complete QSL scope covers every checked `RequirementRecord` and native pending obligation for the selected request, identified by its collision-free native occurrence key or site/kind tuple rather than an IR requirement reference or semantic digest; complete `quire-code-rs` scope covers every selected production module and every function fact, including empty modules. Normal exhaustion is required for `Complete`; QSL exhaustion and `Unfinished` remain distinct typed causes, and an early stop, failed parse/request, unavailable selected module, scope/source mismatch, or item-count mismatch produces a typed error and no census. | Test |
| FR-052-AC-8 | A default-feature consumer can construct and borrow provisional owner rows, QSL native occurrence and pending-obligation identity projections, distinct typed QSL exhaustion and unfinished stop causes, and authored assertions through `SourceObligationInput`. The envelope exposes no `Complete` claim or caller item count, and the public module exposes no census or complete-census builder. | Test |
| FR-052-AC-9 | For each owner, the native enumerator or typed adapter issues a `Complete` outcome only after the selected named scope reaches its normal end; the outcome retains the selected source, typed owner and scope, every owner-emitted row and the observed count. A complete zero-row scope and a fully parsed module with zero functions remain complete, while an absent owner report remains absent. Deleting one row after the owner's full output is observed makes the owner-output comparison fail even when the remaining row count is changed to match. | Test |
| FR-052-AC-10 | For each owner, stopping before normal exhaustion issues `Incomplete` with retained partial rows and the typed cause the owner actually reports. QSL type-work exhaustion retains dimension, limit, prior, requested charge and site; QSL `TypeDisposition::Unfinished` retains its declaration handle and is distinct. An IR V2 reader limit retains the `CheckedPackageIncomplete` fields. For an owner path that reports `quire_exact::CancelCause`, requested and deadline cancellation remain distinct through `SourceInventoryCancelCause`; the current QSL inventory and `quire-code-rs` adapter do not claim cancellation support. `stop_reason: None` on a provisional envelope cannot be promoted to `Complete`, and a new owner stop type cannot be projected into a string or generic early stop. | Test |
| FR-052-AC-11 | The neutral model retains an owner-issued outcome only when its source, owner, scope and observed count match the retained rows; a mismatch is typed and refuses the outcome. Owner adapter tests show that no adapter completion branch accepts an arbitrary caller-supplied row vector/count/completion flag in place of enumerating the selected native owner output. The model does not claim independent authentication of external owner exhaustion or a completed three-owner census. | Test |

FR-052-AC-4 and FR-052-AC-8 have provisional-port evidence. FR-052-AC-1
through AC-3 and AC-5 through AC-7 remain Planned/UNRUN until native owner
adapters compare each declared complete envelope with full selected owner
output and the completed census API exists. FR-052-AC-9 through AC-11 are
Planned/UNRUN until the owner-issued boundary and its full-output oracle land.
The model API tests use synthetic inventories and do not establish owner-issued
completeness.

## Dependencies

[FR-045](../../core/functional/FR-045-derive-proof-coverage-baseline.md)
owns candidate eligibility, evidence joins, and projections;
[AD-008](../../assurance/AD-008-proof-coverage-accounting.md) owns their
accounting meaning. [FR-019](./FR-019-rust-library-interface.md) owns the
model crate public-items inventory, and [FR-028](./FR-028-separate-cycle-free-contract-model.md)
owns the model dependency boundary. The typed inputs retain owner-native
identities from the IR V2, QSL, and `quire-code-rs` APIs; this requirement
defines no replacement parser or owner schema.

The selected source is an existing typed `SourceIdentity` from
[FR-011](../../core/functional/FR-011-package-identity.md). Its private fields
are constructed from a `SourceDocumentId` accepted by the validating
constructor and a `SourceRevision` in the positive, bounded range. Invalid
source identity values are unrepresentable at this interface. An owner
inventory that names another valid source uses the distinct
`InventorySourceMismatch` error.
