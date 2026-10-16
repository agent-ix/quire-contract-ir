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
- A source-author binding that explicitly relates one selected, validated Quire
  corpus and obligation declaration scope to that `SourceIdentity`. This binding
  is authored provenance, not proof that two producers read identical bytes.
- Owner-issued `SourceInventoryOutcome` values, one for each of four owner
  positions (the census does not accept raw `SourceInventoryEnvelope` values).
  They carry the complete typed inventories from their owners: Quire-authored acceptance
  criteria; Contract IR V2 semantic nodes
  and source occurrences; QSL checked `RequirementRecord` values and native
  pending obligations; and `quire-code-rs` selected production module and
  function facts.
- Quire's criterion identity is the obligation source declaration, scoped
  document, and authored criterion ID returned from its declared acceptance
  criteria. The selected corpus and its traceability model define the complete
  scope, including criteria with no authored correspondence or test binding.
  This is a different identity from the formal `ClauseRef` of a Contract IR
  clause. An explicit authored relation may connect them where needed; neither
  the criterion ID nor a QSL requirement record may be converted into a
  `ClauseRef` by spelling or inference.
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
`SourceInventoryOutcome`, `SourceInventoryOutcomeError`,
`SourceInventoryEnumerator`, `SourceInventoryDisposition`,
`SourceInventoryCompleteness`, `SourceInventoryIncompleteReason`, `SourceInventoryOwner`,
`SourceInventoryScope`, `SourceCorrespondenceError`, and
`build_source_obligation_census`. These types shall be listed in FR-019's
public-items inventory when implemented. The current compiled surface has two parts, and
FR-019 lists it exactly. The provisional IR-784 envelope port exports
`SourceObligationInput`, `SourceInventoryEnvelope`, `SourceInventoryRow`,
typed owner identities and stop causes, and authored assertion/provenance
types; none of these carries a completeness state. The IR-794 neutral outcome
boundary exports `SourceInventoryEnumerator`, `SourceInventoryOutcome`,
`SourceInventoryOutcomeError` and `SourceInventoryCompleteness`; this is the
only completeness carrier. `SourceInventoryEnvelope` is not a census input.
Neither part exports a census or census builder.

The corrected neutral port shall use a typed
`SourceAuthoredCriterionId` carrying Quire's obligation declaration, scoped
document and criterion ID, plus a typed `SourceCriterionSourceBinding` carrying
the selected Quire scope and the source-contract author's selected
`SourceIdentity`. A `SourceInventoryIdentity::Criterion` row shall have owner
`Quire`; QSL rows shall contain only native checked requirement records and
pending obligations. The current port's `Criterion(ClauseRef)` and its QSL
owner assignment are provisional defects to correct under IR-795, not evidence
that a formal clause is an authored acceptance criterion. Any explicit
criterion-to-formal-clause relation shall use separate typed endpoints and
must not alter criterion inventory membership. FR-019 shall add the two new
public items when implemented, while retaining its inventory of the current
compiled surface until then.

The immutable census shall expose borrowed accessors for the selected source,
all inventory rows, accepted correspondence rows, and rejected assertions.
Each inventory row shall retain its typed owner identity, source provenance,
and disposition. Inventory disposition shall distinguish at least
`Mapped`, `ZeroMappings`, and `ConflictingIdentity`.
`ZeroMappings` shall remain a census row, not a dropped item or a fabricated
relation. Each accepted row shall retain the exact criterion,
occurrence, and production-symbol keys plus assertion provenance. Missing
mapping reasons shall be typed and distinguish at least no authored mapping
from an explicitly authored absence of a production function. A typed
top-level error shall distinguish missing owner inventory, incomplete owner
inventory, inventory source mismatch, and inventory scope mismatch. Its owner context shall identify Quire, IR V2,
QSL, or `quire-code-rs`; incomplete errors shall retain the typed owner cause.
`SourceCorrespondenceRejection` shall distinguish `ForeignSource`,
`DanglingCriterion`, `DanglingOccurrence`, `DanglingProductionSymbol`,
`ConflictingRelation`, and `ConflictingProvenance` reasons.

## Behavior

### Neutral API authority and source provenance

The public types and constructor shall be defined in the cycle-free
`quire-contract-model` crate, under the `source_obligation` module. They shall
use neutral model-owned identity and provenance types.

The completed `SourceObligationInput` constructor shall take the selected
source, an explicit Quire-selection-to-source binding, four
independently optional owner `SourceInventoryOutcome` values (one per owner
position: Quire, IR V2, QSL, `quire-code-rs`), and the author-authored
assertions as typed arguments. An outcome identifies its owner, selected
source, covered scope, observed row count, typed completeness state, and
retained inventory rows. The four required scopes shall be Quire's complete selected authored
acceptance-criterion population, IR V2's complete semantic graph and
source-occurrence set, QSL's complete checked `RequirementRecord` set and
native pending-obligation set, and `quire-code-rs`'s complete selected
production-module and function facts. Each outcome's owner and scope
shall match its required input position, and its source shall match the
top-level selected source.

The Quire selection shall identify the exact corpus and validated traceability
model used to derive the criterion population. The source-contract author
shall bind that selection to the same selected `SourceIdentity` used by the
other outcomes and assertions. A missing, conflicting, or foreign binding
shall refuse the census with a distinct typed cause; matching labels alone
shall not authenticate equal content across Quire, IR V2, QSL, and production
sources. In particular, QSL's current `CheckedUnit` checks source labels for
its graph and type report, not one shared digest-bound parse. No complete
matched-source census may be claimed from that interim match alone.

`SourceInventoryCompleteness` shall be either `Complete` or `Incomplete` with
a `SourceInventoryIncompleteReason`. The owner-native inventory enumerator or
its typed adapter shall issue `Complete` only after normal exhaustion of the
entire named scope; an early stop, failed parse, failed owner request, or
unavailable selected module shall issue `Incomplete` with the original typed
cause. QSL type-work exhaustion shall retain its native dimension, limit,
prior, requested charge and site in a dedicated typed cause. QSL
`TypeDisposition::Unfinished` shall be a different typed cause keyed by its
native declaration handle. This state is evidence from enumeration, not a
caller-authored boolean or eligibility policy. The outcome's observed row count shall equal the number of
retained rows; a mismatch is refused with
`SourceInventoryOutcomeError::ObservedCountMismatch` at outcome construction,
so the census never receives one. The outcome retains every emitted row without
deduplication, and its observed count counts duplicates. `Complete` with zero
rows is a valid complete inventory when the owner enumerated the named scope
and found no entries. A complete outcome may also contain an empty component
inventory, such as a fully parsed production module with no functions. These
cases are distinct from an absent outcome, an incomplete outcome, and an
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

Quire owns issuance of the authored-criterion outcome for its selected corpus
and validated traceability model. Its producer shall retain each declared
criterion, including one with zero correspondence assertions or test binders,
and issue `Complete` only after discovery, loading, table scanning, and row
enumeration reach their normal ends without an incomplete cause. Unreadable or
skipped selected documents, omitted row IDs or statements, invalid selection,
failed discovery, and an interrupted scan shall retain distinct typed owner
causes and any already emitted rows. A valid selected scope with zero declared
criteria may complete with zero rows. Quire's current
`obligation::derive(&Spec, root, &TraceabilityModel)` returns obligation rows
and `SkippedRow` records but no owner-issued terminal state or scan diagnostics;
`Spec::from_path` can yield an empty corpus with a diagnostic, and directory
discovery can discard walk errors. Therefore neither that return value nor a
zero-row or zero-skipped-row result may be promoted to `Complete` by Contract
IR. The Quire owner-issued API and its lossless typed stop projection are
pending PLAT-1165. Until it exists, the Quire outcome stays absent or
incomplete and the full census remains Planned/UNRUN.

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
`build_source_obligation_census` permission to claim all four owners complete;
the matched-source, four-owner census and correspondence checks remain under
the later FR-052 acceptance criteria.

A synthetic `SourceInventoryEnumerator` can verify the neutral model boundary
without claiming to be an owner-native enumerator. In this boundary test,
`Ok(Some(row))` emits one row, `Ok(None)` is the only normal terminal event,
and `Err(cause)` is a typed incomplete terminal event. The model derives its
retained row count from emitted rows and checks the enumerator's observed
count against that value; neither count establishes owner-native exhaustion.
This test may prove terminal-state handling and structural rejection for zero
and nonzero synthetic streams. It does not discharge the owner-native
enumeration, full-output comparison, or matched-source census obligations in
FR-052-AC-9 through AC-11.

The provisional envelope port has no completeness state, caller supplied
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

If the Quire binding or any owner outcome is absent, incomplete, or has a
mismatched scope or source, `build_source_obligation_census` shall return a
typed error naming that owner and shall return no census. An item-count,
owner/scope-pair, row-owner, row-source or stop-owner inconsistency is refused
earlier by `SourceInventoryOutcomeError` and never reaches the census. For
scope, the census sees only a well-formed outcome supplied in the wrong owner
position, which returns `InventoryScopeMismatch`. The census shall
report the first failing owner in fixed order: Quire, IR V2, QSL, then
`quire-code-rs`. The
constructor shall not treat a missing outcome or partial owner enumeration as
a zero-mapping entity or a complete empty inventory. A census is complete only
when the binding and all four required owner outcomes pass these checks.

`SourceInventoryOwner` shall distinguish `Quire`, `IrV2`, `Qsl`, and
`QuireCodeRs`. `SourceInventoryScope` shall distinguish
`QuireSelectedAuthoredCriteria`, `IrV2SemanticGraphAndOccurrences`,
`QslCheckedRequirementsAndPendingObligations`, and
`QuireCodeRsProductionModulesAndFunctions`. Each owner shall supply exactly
its corresponding scope. `SourceCorrespondenceError` shall expose distinct
variants `MissingOwnerInventory`, `IncompleteOwnerInventory`,
`InventorySourceMismatch`, and `InventoryScopeMismatch`, retaining the
owner and the relevant expected and observed source or scope, and the
typed incomplete cause where applicable. It has no item-count variant.

### Census and correspondence

The constructor shall preserve every Quire-issued authored criterion, semantic occurrence,
module, and function as an inventory row, including rows with no accepted
correspondence. It shall preserve an empty module and every zero-mapping source
entity. It shall accept explicit many-to-many assertions and shall emit one
accepted correspondence row per distinct valid relation without deriving a
Cartesian product of the inventories. Exact repeated assertions shall collapse
deterministically. Exact repeated inventory entries shall collapse to one
row. Duplicates and conflicts arise only among the rows of one outcome:
each owner position has one outcome and each identity belongs to one owner. The
inventory key of a row is, per `SourceInventoryIdentity` variant: the
criterion identity for `Criterion`; the `SourceRequirementOccurrenceId` for
`RequirementRecord`; the `CheckedNodeId` for `SemanticNode`; and the whole
identity for `SemanticOccurrence`, `PendingObligation`, `ProductionModule` and
`ProductionFunction`. Rows with equal keys and equal facts are exact
duplicates. The only inventory fact beyond the key is the `CheckedSemanticId` of
a `SemanticNode`; rows with one `CheckedNodeId` and differing
`CheckedSemanticId` values conflict, so each conflicting entry shall remain
inspectable with `ConflictingIdentity`; none shall be selected by first-wins
order. Collapsing exact duplicates happens in the census and does not alter the
outcome's retained rows or observed count, so a census may hold fewer inventory
rows than the outcome's observed count. Repeated assertion identity with
conflicting relation facts or provenance shall remain inspectable with
`ConflictingRelation` or `ConflictingProvenance`, respectively; no conflicting
assertion shall be accepted. Input ordering shall not change the disposition
or ordered census.

A foreign-source inventory row is refused at the outcome boundary with
`SourceInventoryOutcomeError::RowSourceMismatch` and never reaches the census;
an outcome whose own source differs from the selected source returns
`InventorySourceMismatch`. The census therefore has no foreign-inventory
disposition. An assertion with a foreign source or an
endpoint absent from the selected inventories shall be retained as a rejected
assertion with a corresponding typed reason (`ForeignSource`,
`DanglingCriterion`, `DanglingOccurrence`, or `DanglingProductionSymbol`) and
shall not become an accepted relation. The census shall distinguish those
rejected relations from valid entities that have no authored relation. No
malformed or incomplete input shall produce a census that appears complete.
An assertion-only criterion key, a QSL `RequirementRecord`, or a formal
`ClauseRef` outside an explicit authored relation shall never satisfy the
criterion endpoint lookup.

The source-obligation input schema shall contain no method, profile, result,
artifact, attempt, or run field. Census membership is derived only from the
selected source inventories and explicit assertions. The model shall not
qualify proof evidence, calculate coverage projections, or alter FR-045's
candidate denominator.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-052-AC-1 | A default-feature consumer imports and calls the input, constructor, census and row types from `quire_contract_model::source_obligation`; FR-019's public-items inventory lists every exported item. | Test |
| FR-052-AC-2 | A typed fixture containing two Quire-authored acceptance criteria, including one with no mapping, three semantic occurrences, one empty module, two production functions and explicit many-to-many assertions preserves every inventory row and exactly the asserted valid relations with their selected-source provenance. Unmapped criteria, occurrences, modules and functions remain visible with typed missing reasons. Reordering equivalent inputs yields the same ordered census; removing one occurrence from a complete IR V2 outcome changes it. | Test |
| FR-052-AC-3 | A foreign source, each dangling endpoint, an exact duplicate, conflicting inventory facts (two `SemanticNode` rows with one `CheckedNodeId` and differing `CheckedSemanticId`), conflicting assertion relation facts, and conflicting assertion provenance each receive their named deterministic disposition; rejected entries remain inspectable and never enter accepted relations. A foreign-source inventory row is refused at the outcome boundary, not by the census (FR-052-AC-13). | Test |
| FR-052-AC-4 | Inspection of the public constructor and `SourceObligationInput` signature confirms that the input schema has no result, method, profile, artifact, attempt or run field. | Inspection |
| FR-052-AC-5 | An external default-feature consumer compiles a borrow of `SourceObligationCensus` and its rows from `quire_contract_model::source_obligation`. | Test |
| FR-052-AC-6 | Omitting each of the four owner outcomes in turn returns `MissingOwnerInventory` naming that owner and no census. Supplying, for each position, an outcome whose enumerator ended with a typed `Err` (issued `Incomplete`) returns `IncompleteOwnerInventory` with the supplied typed cause and no census. A fully parsed production module with no function facts is accepted as complete, retains the module row, and differs observably from both outcomes. No Quire `Complete` fixture counts as owner evidence until PLAT-1165 supplies it. | Test |
| FR-052-AC-7 | Complete Quire scope covers every authored acceptance criterion in the bound selected corpus and declared obligation sources, including unbound criteria; complete IR V2 scope covers every semantic node and source occurrence in the selected V2 graph; complete QSL scope covers every checked `RequirementRecord` and native pending obligation for the selected request, identified by its collision-free native occurrence key or site/kind tuple rather than an IR requirement reference or semantic digest; complete `quire-code-rs` scope covers every selected production module and every function fact, including empty modules. Normal exhaustion is required for `Complete`; Quire skipped/load/scan failures, QSL exhaustion and `Unfinished`, early stop, failed parse/request, unavailable selected module, or scope/source mismatch produce a typed incomplete or error result and no census; an observed-count mismatch is refused as `SourceInventoryOutcomeError::ObservedCountMismatch` and issues no outcome. | Test |
| FR-052-AC-8 | A default-feature consumer can construct and borrow provisional owner rows, QSL native occurrence and pending-obligation identity projections, distinct typed QSL exhaustion and unfinished stop causes, and authored assertions through `SourceObligationInput`. The envelope exposes no `Complete` claim or caller item count, and the public module exposes no census or complete-census builder. | Test |
| FR-052-AC-9 | For each of the four owners, the native enumerator or typed adapter issues a `Complete` outcome only after the selected named scope reaches its normal end; the outcome retains the selected source, typed owner and scope, every owner-emitted row and the observed count. For Quire this requires PLAT-1165's selected-scope producer, not an IR traversal of `obligation::derive` output. A complete zero-row scope and a fully parsed module with zero functions remain complete, while an absent owner report remains absent. Deleting one row after the owner's full output is observed makes the owner-output comparison fail even when the remaining row count is changed to match. | Test |
| FR-052-AC-10 | For each owner, stopping before normal exhaustion issues `Incomplete` with retained partial rows and the typed cause the owner actually reports. Quire discovery/load/scan/skipped-row failures retain their distinct owner-native causes once PLAT-1165 supplies them; current `obligation::derive` cannot issue `Complete`. QSL type-work exhaustion retains dimension, limit, prior, requested charge and site; QSL `TypeDisposition::Unfinished` retains its declaration handle and is distinct. An IR V2 reader limit retains the `CheckedPackageIncomplete` fields. For an owner path that reports `quire_exact::CancelCause`, requested and deadline cancellation remain distinct through `SourceInventoryCancelCause`; the current QSL inventory and `quire-code-rs` adapter do not claim cancellation support. `stop_reason: None` on a provisional envelope cannot be promoted to `Complete`, and a new owner stop type cannot be projected into a string or generic early stop. | Test |
| FR-052-AC-11 | The neutral model retains an owner-issued outcome only when its source, owner, scope and observed count match the retained rows; a mismatch is typed and refuses the outcome. Owner adapter tests show that no adapter completion branch accepts an arbitrary caller-supplied row vector/count/completion flag in place of enumerating the selected native owner output. The model does not claim independent authentication of external owner exhaustion or a completed four-owner census. | Test |
| FR-052-AC-12 | A synthetic `SourceInventoryEnumerator` producing zero or multiple `Ok(Some(row))` events yields a neutral `Complete` outcome only after its `Ok(None)` terminal event. Replacing that terminal event with `Err(typed_cause)` yields `Incomplete(typed_cause)` and retains exactly the preceding rows; the typed cause is equal to the emitted error. The test claims no owner-native exhaustion or full-output comparison. | Test |
| FR-052-AC-13 | For a synthetic enumerator, the neutral outcome keeps its retained rows and count private: an external default-feature consumer fails to compile when it directly reads either field or constructs an outcome by struct literal, while accessor calls compile. A zero- or nonzero-row stream reports the count derived from its retained rows. Mismatched observed count, owner/scope pair, row owner, row source, or stop-cause owner returns the corresponding typed structural error and no outcome. This structural test grants no FR-052-AC-9 through AC-11 owner-native credit. | Test |
| FR-052-AC-14 | A default-feature neutral consumer distinguishes a typed Quire criterion identity containing obligation declaration, scoped document and authored criterion ID from `ClauseRef` and QSL `RequirementRecord`. Two neutral fixture rows with distinct authored criterion identities, one without a correspondence assertion, remain distinct Quire-owned rows before any generation or result. A formal clause, QSL requirement record or assertion-only key cannot substitute for either fixture criterion. This neutral fixture does not claim Quire owner-issued completeness. | Test |
| FR-052-AC-15 | An explicit source-author binding connects the selected Quire corpus/obligation scope to the selected IR `SourceIdentity`. A missing, foreign or conflicting binding refuses a complete census with distinct typed causes; a matching label or revision alone is not treated as proof of equal source bytes across Quire, IR V2 and QSL. Foreign and dangling AC-to-occurrence/function assertions retain their separate typed dispositions. | Test |
| FR-052-AC-16 | With PLAT-1165's real Quire producer, normal exhaustion of the validated selected scope issues `Complete` with every authored criterion and its provenance; a valid empty scope completes with zero rows. Changing or deleting one authored criterion in the selected Quire corpus changes the owner-issued inventory before generation or proof results, including when that criterion has zero mappings. An unreadable selected document, skipped criterion row, failed discovery, missing declared table/ID column, and early stop each report typed `Incomplete` with emitted partial rows, and deletion of a row from full owner output fails comparison even when the shortened count matches. Until this producer exists, no Quire `Complete` or full FR-052 census is claimed. | Test |

FR-052-AC-4 and FR-052-AC-8 have provisional-port evidence. FR-052-AC-12
and AC-13 are Implemented with traced tests for the neutral synthetic
boundary only. FR-052-AC-1
through AC-3 and AC-5 through AC-7 remain Planned/UNRUN until native owner
adapters compare each declared complete outcome with full selected owner
output and the completed census API exists. FR-052-AC-9 through AC-11 are
Planned/UNRUN until the owner-issued boundary and its full-output oracle land.
Passing AC-12 and AC-13 shall not change AC-9 through AC-11 status.
FR-052-AC-14 is Implemented for the neutral typed identity and synthetic
Quire-owned rows, with traced integration and compile-fail tests. The neutral
binding retention portion of FR-052-AC-15 is Implemented with traced tests;
its missing, foreign, and conflicting binding census refusals and assertion
dispositions remain Planned/UNRUN until the census builder exists.
AC-16 and the Quire portion of AC-2, AC-6, AC-7, and AC-9 through AC-11
remain Planned/UNRUN pending PLAT-1165. The matched-source FR-052 census and
IR-759 acceptance remain open.
The model API tests use synthetic inventories and do not establish owner-issued
completeness.

## Dependencies

[FR-045](../../core/functional/FR-045-derive-proof-coverage-baseline.md)
owns candidate eligibility, evidence joins, and projections;
[AD-008](../../assurance/AD-008-proof-coverage-accounting.md) owns their
accounting meaning. [FR-019](./FR-019-rust-library-interface.md) owns the
model crate public-items inventory, and [FR-028](./FR-028-separate-cycle-free-contract-model.md)
owns the model dependency boundary. The typed inputs retain owner-native
identities from Quire, IR V2, QSL, and `quire-code-rs` APIs; this requirement
defines no replacement parser or owner schema.

The selected source is an existing typed `SourceIdentity` from
[FR-011](../../core/functional/FR-011-package-identity.md). Its private fields
are constructed from a `SourceDocumentId` accepted by the validating
constructor and a `SourceRevision` in the positive, bounded range. Invalid
source identity values are unrepresentable at this interface. An owner
inventory that names another valid source uses the distinct
`InventorySourceMismatch` error.
