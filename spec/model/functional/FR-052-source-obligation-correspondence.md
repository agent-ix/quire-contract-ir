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

The neutral census builder accepts four owner-issued outcomes and an explicit
source-author binding. Synthetic outcomes test its construction and refusal
rules. The IR V2 owner portion has native completeness and full-output tests;
the other owner portions and matched-source four-owner comparison remain Planned/UNRUN.
IR-794 defines the narrower owner-issued enumeration outcome that the neutral
model retains. It does not make a caller-supplied
row vector, count, or absent stop reason into evidence of exhaustion.

## Inputs

- One selected source identity and its provenance.
- A source-author binding that explicitly relates one selected, validated Quire
  corpus and obligation declaration scope to that `SourceIdentity`. This binding
  is authored provenance, not proof that two producers read identical bytes.
- Owner-issued `SourceInventoryOutcome` values, one for each of four owner
  positions.
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

The public `quire_contract_model::source_obligation` module exports
`SourceObligationInput`, `SourceObligationCensus`, `SourceObligationRow`,
`SourceInventoryRow`, `SourceCensusInventoryRow`, `SourceCorrespondenceAssertion`,
`SourceCorrespondenceRejection`, `SourceCorrespondenceRejectionReason`,
`SourceMissingMappingReason`,
`SourceInventoryOutcome`, `SourceInventoryOutcomeError`,
`SourceInventoryEnumerator`, `SourceInventoryDisposition`,
`SourceInventoryCompleteness`, `SourceInventoryIncompleteReason`, `SourceInventoryOwner`,
`SourceInventoryScope`, `SourceCorrespondenceError`,
`SourceIrV2InventoryError`, `build_source_obligation_census`, and
`source_inventory_from_ir_v2`. These public items shall be listed in FR-019's
public-items inventory. FR-019 lists the compiled public surface exactly. The owner-issued outcome
boundary supplies `SourceInventoryOutcome` and `SourceInventoryEnumerator`;
`SourceObligationInput` consumes four optional outcomes, and the census
builder evaluates their typed terminal states and authored assertions.
The census builder supersedes the provisional input port. FR-052-AC-17
supersedes retired FR-052-AC-8. `SourceObligationInput::new` now accepts the
four optional outcomes in Quire, IR V2, QSL and `quire-code-rs` order. TC-452
records the synthetic census checks separately from pending owner-native work.
The neutral port uses a typed
`SourceAuthoredCriterionId` carrying Quire's obligation declaration, scoped
document and criterion ID, plus a typed `SourceCriterionSourceBinding` carrying
the selected Quire scope and the source-contract author's selected
`SourceIdentity`. A `SourceInventoryIdentity::Criterion` row has owner
`Quire`; QSL rows contain only native checked requirement records and
pending obligations. A formal `ClauseRef` is distinct from an authored
acceptance criterion. Any explicit
criterion-to-formal-clause relation shall use separate typed endpoints and
must not alter criterion inventory membership. FR-019 lists both public
items in its compiled-surface inventory.

The immutable census shall expose borrowed accessors for the selected source,
all inventory rows, accepted correspondence rows, and rejected assertions.
Each inventory row shall retain its typed owner identity, source provenance,
and disposition. Inventory disposition shall distinguish at least
`Mapped`, `ZeroMappings`, and `ConflictingIdentity`.
`ZeroMappings` shall remain a census row, not a dropped item or a fabricated
relation. Each accepted row shall retain the exact criterion,
occurrence, and production-symbol keys plus assertion provenance. Missing
mapping reasons shall be typed and distinguish at least no accepted authored
mapping from an explicitly authored absence of a production function. A typed
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
A well-formed outcome supplied in another owner's position shall be refused with
`InventoryScopeMismatch`.

The Quire selection shall identify the exact corpus and validated traceability
model used to derive the criterion population. The source-contract author
shall bind that selection to the same selected `SourceIdentity` used by the
other outcomes and assertions. A missing, conflicting, or foreign binding
shall refuse the census with a distinct typed cause
(`MissingCriterionSourceBinding`, `ConflictingCriterionSourceBinding`, or
`ForeignCriterionSourceBinding`); matching labels alone
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
translate an absent stop cause into `Complete`.

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

Each owner adapter shall consume that owner's selected native object or
owner-issued terminal report, rather than a caller-authored inventory row
vector, count, or completion flag. IR V2's adapter belongs in
`quire-contract-model`, which already owns the admitted V2 graph. The Quire,
QSL, and `quire-code-rs` adapters belong with their respective owners; their
typed outcomes cross the neutral model boundary without adding those owners
as dependencies of `quire-contract-model`. FR-052-AC-16 tests Quire's owner
portion, and FR-052-AC-25 through AC-27 test the other three portions
separately. Passing one owner portion does not discharge the four-owner
requirements in FR-052-AC-7 or AC-9 through AC-11.

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

The completed input retains the selected source, optional owner-issued
outcomes, and authored assertions. It cannot claim owner-native exhaustion
without each owner's full-output oracle.
Source-contract authoring is the assertion producer; the input constructor
preserves assertions, and the census builder validates them without inference or repair. Each assertion and
inventory item shall carry selected-source provenance sufficient to establish
which source selection it belongs to. The census shall retain the supplied
provenance, including provenance on rejected rows.

If the Quire binding or any owner outcome is absent, incomplete, or has a
mismatched scope or source, `build_source_obligation_census` shall return a
typed error naming the failing binding or owner and shall return no census. An item-count,
owner/scope-pair, row-owner, row-source or stop-owner inconsistency is refused
earlier by `SourceInventoryOutcomeError` and never reaches the census. For
scope, the census sees only a well-formed outcome supplied in the wrong owner
position, which returns `InventoryScopeMismatch`. The census shall
report the first failing owner in fixed order: Quire, IR V2, QSL, then
`quire-code-rs`. The binding belongs to the Quire position: a missing binding
and then a foreign binding are checked before the Quire outcome, and a
conflicting binding is checked after the Quire outcome passes and before IR V2.
The full order is therefore: missing or foreign binding, Quire outcome,
conflicting binding, IR V2, QSL, `quire-code-rs`. Within one owner position the checks run in this order: missing outcome,
scope mismatch, source mismatch, then incomplete outcome. The
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
typed incomplete cause where applicable. It shall also expose
`MissingCriterionSourceBinding`, `ForeignCriterionSourceBinding` retaining the
expected and observed `SourceIdentity`, and `ConflictingCriterionSourceBinding`
retaining the binding's obligation scope and the first conflicting criterion in
ascending criterion order. It has no item-count variant.

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
duplicates. The only inventory fact beyond the key is the `semantic_type:
CheckedNodeId` of a `SemanticNode`, copied from the admitted
`CheckedSemanticNodeV2` alongside its `node_id` key. Rows with one node key and
differing semantic type keys conflict, so each conflicting entry shall remain
inspectable with `ConflictingIdentity`; none shall be selected by first-wins
order. The package-level `CheckedSemanticId` is not a per-node fact. Collapsing
exact duplicates happens in the census and does not alter the
outcome's retained rows or observed count, so a census may hold fewer inventory
rows than the outcome's observed count. Repeated assertion identity with
conflicting relation facts or provenance shall remain inspectable with
`ConflictingRelation` or `ConflictingProvenance`, respectively. Assertions with
different identities that state one relation with differing provenance shall
each remain inspectable with `ConflictingProvenance`. No conflicting
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

### Census construction rules

`SourceObligationInput::new` shall take, in order, the selected
`SourceIdentity`, an `Option<SourceCriterionSourceBinding>`, one
`Option<SourceInventoryOutcome>` for each of the Quire, IR V2, QSL and
`quire-code-rs` positions, and a `Vec<SourceCorrespondenceAssertion>`.
`build_source_obligation_census` shall take that input by value and return
`Result<SourceObligationCensus, SourceCorrespondenceError>`.

A binding is foreign when its `source()` differs from the selected source. A
binding conflicts when a `Criterion` row of the Quire outcome has an
`obligation_declaration()` different from the binding's `obligation_scope()`.
The census does not compare the binding's corpus, traceability model or author
with any other input. This definition of a conflicting binding is an IR
reading pending owner confirmation (see Open questions).

`SourceObligationCensus` shall expose `source()` returning `&SourceIdentity`,
`criterion_source_binding()` returning `&SourceCriterionSourceBinding` (a
census exists only with a binding), `inventory_rows()` returning `&[SourceCensusInventoryRow]`,
`correspondence_rows()` returning `&[SourceObligationRow]`, and
`rejected_assertions()` returning `&[SourceCorrespondenceRejection]`.
`SourceCensusInventoryRow` shall expose `row()`, the retained
`SourceInventoryRow`, and `disposition()`. `SourceInventoryDisposition` shall
have the variants `Mapped`, `ZeroMappings(SourceMissingMappingReason)` and
`ConflictingIdentity`. `SourceMissingMappingReason` shall have the variants
`NoAuthoredMapping` and `ExplicitNoProductionFunction`.
`SourceObligationRow` shall expose `criterion()`, `occurrence()`,
`production_function()` as an `Option`, and `provenance()`.
`SourceCorrespondenceRejection` shall expose `assertion()`, the original
assertion, and `reason()`, a `SourceCorrespondenceRejectionReason`.

An assertion endpoint resolves only by exact typed identity against the
selected inventories. The criterion endpoint resolves against `Criterion`
rows. The occurrence endpoint resolves against a `SemanticOccurrence` row
whose node has a `SemanticNode` row and no `ConflictingIdentity` row. A present
production-function endpoint resolves against a `ProductionFunction` row whose
module has a `ProductionModule` row. An absent production function is an
explicit authored absence and needs no function row. An occurrence whose node
row is absent or conflicting, and a function whose module row is absent, do not
resolve. An assertion naming one is rejected with `DanglingOccurrence` or
`DanglingProductionSymbol`; the orphan row itself stays in the census. A
complete owner outcome contains no such orphan, and the census does not treat
one as a valid endpoint.

The census shall process assertions in these steps:

1. Exact duplicate assertions, equal in every field, collapse to one.
2. Assertions sharing one `SourceAssertionId` form a group. When the group
   holds more than one distinct tuple of source, criterion, occurrence and
   production function, every member is rejected with `ConflictingRelation`.
   Otherwise, when it holds more than one distinct provenance, every member is
   rejected with `ConflictingProvenance`.
3. Each remaining assertion receives the first matching reason in this order:
   `ForeignSource` when its source or its provenance source differs from the
   selected source, then `DanglingCriterion`, `DanglingOccurrence` and
   `DanglingProductionSymbol`.
4. Remaining assertions with one criterion, occurrence and production function
   form one relation. When a relation holds more than one distinct provenance,
   every member is rejected with `ConflictingProvenance`. Otherwise the relation
   yields exactly one `SourceObligationRow`, whatever the number of assertion
   IDs that state it.

Each rejected assertion carries exactly one reason.

The census shall assign each inventory row one disposition. A `SemanticNode`
row in a conflicting key group is `ConflictingIdentity`. Otherwise a row is
`Mapped` when an accepted row names it, under these rules. A criterion or
production function is named by an accepted row that contains it. A semantic
occurrence is named by an accepted row that contains it with a production
function. A semantic node is named when one of its occurrences is `Mapped`, and
a production module when one of its functions is `Mapped`. A row that is not
`Mapped` is `ZeroMappings(ExplicitNoProductionFunction)` when it is an
occurrence, or the node of an occurrence, named by an accepted row with no
production function. Every other row, including an empty module and every QSL
`RequirementRecord` and `PendingObligation` row, is
`ZeroMappings(NoAuthoredMapping)`; no assertion endpoint can name a QSL row.
That QSL rows are always `ZeroMappings(NoAuthoredMapping)` is an IR reading
pending owner confirmation (see Open questions).

Dispositions derive only from accepted rows. `NoAuthoredMapping` means that no
accepted authored mapping names the row. A row whose only authored assertions
were rejected therefore reads `ZeroMappings(NoAuthoredMapping)`, as does an
orphan or a row under a conflicting node. The census reports those assertions,
and their reasons, only through `rejected_assertions()`.

Inventory rows shall be ordered by owner in the order Quire, IR V2, QSL,
`quire-code-rs`, then by `SourceInventoryIdentity` ascending. Accepted rows
shall be ordered ascending by criterion, occurrence, production function and
provenance. Rejections shall be ordered ascending by assertion ID, source,
criterion, occurrence, production function, provenance and reason.

The source-obligation input schema shall contain no method, profile, result,
artifact, attempt, or run field. Census membership is derived only from the
selected source inventories and explicit assertions. The model shall not
qualify proof evidence, calculate coverage projections, or alter FR-045's
candidate denominator.

### Open questions

- Conflicting binding. The census construction rules read a binding's
  `obligation_scope()` as one obligation declaration that every Quire
  `Criterion` row must carry. FR-052-AC-7 names the bound corpus's "declared
  obligation sources" in the plural. Whether one binding may cover several
  obligation declarations, and so what counts as a conflicting binding,
  awaits owner confirmation.
- QSL rows. The correspondence assertion has criterion, occurrence and
  production-function endpoints only, so the census reads every QSL
  `RequirementRecord` and `PendingObligation` row as
  `ZeroMappings(NoAuthoredMapping)`. Whether QSL rows are meant to be census
  members only, with no correspondence endpoint, awaits owner confirmation.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-052-AC-1 | A default-feature consumer imports and calls the input, constructor, census and row types from `quire_contract_model::source_obligation`; FR-019's public-items inventory lists every exported item. | Test |
| FR-052-AC-2 | A typed fixture containing two Quire-authored acceptance criteria, including one with no mapping, three semantic occurrences, one empty module, two production functions and explicit many-to-many assertions preserves every inventory row and exactly the asserted valid relations with their selected-source provenance. Unmapped criteria, occurrences, modules and functions remain visible with typed missing reasons. Reordering equivalent inputs yields the same ordered census; removing one occurrence from a complete IR V2 outcome changes it. | Test |
| FR-052-AC-3 | A foreign source, each dangling endpoint, an exact duplicate, conflicting assertion relation facts, and conflicting assertion provenance each receive their named deterministic disposition; rejected entries remain inspectable and never enter accepted relations. A foreign-source inventory row is refused at the outcome boundary, not by the census (FR-052-AC-13). Conflicting inventory facts are FR-052-AC-24. | Test |
| FR-052-AC-4 | Inspection of the public constructor and `SourceObligationInput` signature confirms that the input schema has no result, method, profile, artifact, attempt or run field. | Inspection |
| FR-052-AC-5 | An external default-feature consumer compiles a borrow of `SourceObligationCensus` and its rows from `quire_contract_model::source_obligation`. | Test |
| FR-052-AC-6 | Omitting each of the four owner outcomes in turn returns `MissingOwnerInventory` naming that owner and no census. Supplying, for each position, an outcome whose enumerator ended with a typed `Err` (issued `Incomplete`) returns `IncompleteOwnerInventory` with the supplied typed cause and no census. A fully parsed production module with no function facts is accepted as complete, retains the module row, and differs observably from both outcomes. No Quire `Complete` fixture counts as owner evidence until PLAT-1165 supplies it. | Test |
| FR-052-AC-7 | Complete Quire scope covers every authored acceptance criterion in the bound selected corpus and declared obligation sources, including unbound criteria; complete IR V2 scope covers every semantic node and source occurrence in the selected V2 graph; complete QSL scope covers every checked `RequirementRecord` and native pending obligation for the selected request, identified by its collision-free native occurrence key or site/kind tuple rather than an IR requirement reference or semantic digest; complete `quire-code-rs` scope covers every selected production module and every function fact, including empty modules. Normal exhaustion is required for `Complete`; Quire skipped/load/scan failures, QSL exhaustion and `Unfinished`, early stop, failed parse/request, unavailable selected module, or scope/source mismatch produce a typed incomplete or error result and no census; an observed-count mismatch is refused as `SourceInventoryOutcomeError::ObservedCountMismatch` and issues no outcome. | Test |
| FR-052-AC-9 | For each of the four owners, the native enumerator or typed adapter issues a `Complete` outcome only after the selected named scope reaches its normal end; the outcome retains the selected source, typed owner and scope, every owner-emitted row and the observed count. For Quire this requires PLAT-1165's selected-scope producer, not an IR traversal of `obligation::derive` output. A complete zero-row scope and a fully parsed module with zero functions remain complete, while an absent owner report remains absent. Deleting one row after the owner's full output is observed makes the owner-output comparison fail even when the remaining row count is changed to match. | Test |
| FR-052-AC-10 | For each owner, stopping before normal exhaustion issues `Incomplete` with retained partial rows and the typed cause the owner actually reports. Quire discovery/load/scan/skipped-row failures retain their distinct owner-native causes once PLAT-1165 supplies them; current `obligation::derive` cannot issue `Complete`. QSL type-work exhaustion retains dimension, limit, prior, requested charge and site; QSL `TypeDisposition::Unfinished` retains its declaration handle and is distinct. An IR V2 reader limit retains the `CheckedPackageIncomplete` fields. For an owner path that reports `quire_exact::CancelCause`, requested and deadline cancellation remain distinct through `SourceInventoryCancelCause`; the current QSL inventory and `quire-code-rs` adapter do not claim cancellation support. An absent stop cause cannot be promoted to `Complete`, which follows only from the enumerator's `Ok(None)`, and a new owner stop type cannot be projected into a string or generic early stop. | Test |
| FR-052-AC-11 | The neutral model retains an owner-issued outcome only when its source, owner, scope and observed count match the retained rows; a mismatch is typed and refuses the outcome. Owner adapter tests show that no adapter completion branch accepts an arbitrary caller-supplied row vector/count/completion flag in place of enumerating the selected native owner output. The model does not claim independent authentication of external owner exhaustion or a completed four-owner census. | Test |
| FR-052-AC-12 | A synthetic `SourceInventoryEnumerator` producing zero or multiple `Ok(Some(row))` events yields a neutral `Complete` outcome only after its `Ok(None)` terminal event. Replacing that terminal event with `Err(typed_cause)` yields `Incomplete(typed_cause)` and retains exactly the preceding rows; the typed cause is equal to the emitted error. The test claims no owner-native exhaustion or full-output comparison. | Test |
| FR-052-AC-13 | For a synthetic enumerator, the neutral outcome keeps its retained rows and count private: an external default-feature consumer fails to compile when it directly reads either field or constructs an outcome by struct literal, while accessor calls compile. A zero- or nonzero-row stream reports the count derived from its retained rows. Mismatched observed count, owner/scope pair, row owner, row source, or stop-cause owner returns the corresponding typed structural error and no outcome. This structural test grants no FR-052-AC-9 through AC-11 owner-native credit. | Test |
| FR-052-AC-14 | A default-feature neutral consumer distinguishes a typed Quire criterion identity containing obligation declaration, scoped document and authored criterion ID from `ClauseRef` and QSL `RequirementRecord`. Two neutral fixture rows with distinct authored criterion identities, one without a correspondence assertion, remain distinct Quire-owned rows before any generation or result. A formal clause, QSL requirement record or assertion-only key cannot substitute for either fixture criterion. This neutral fixture does not claim Quire owner-issued completeness. | Test |
| FR-052-AC-15 | An explicit source-author binding connects the selected Quire corpus/obligation scope to the selected IR `SourceIdentity`. A missing, foreign or conflicting binding refuses a complete census with distinct typed causes; a matching label or revision alone is not treated as proof of equal source bytes across Quire, IR V2 and QSL. Foreign and dangling AC-to-occurrence/function assertions retain their separate typed dispositions. | Test |
| FR-052-AC-16 | With PLAT-1165's real Quire producer, normal exhaustion of the validated selected scope issues `Complete` with every authored criterion and its provenance; a valid empty scope completes with zero rows. Changing or deleting one authored criterion in the selected Quire corpus changes the owner-issued inventory before generation or proof results, including when that criterion has zero mappings. An unreadable selected document, skipped criterion row, failed discovery, missing declared table/ID column, and early stop each report typed `Incomplete` with emitted partial rows, and deletion of a row from full owner output fails comparison even when the shortened count matches. Until this producer exists, no Quire `Complete` or full FR-052 census is claimed. | Test |
| FR-052-AC-17 | Given four synthetic `Complete` outcomes and a valid binding, where the outcomes hold the AC-2 fixture shape plus a QSL outcome with at least one `RequirementRecord` row and one `PendingObligation` row, `build_source_obligation_census` returns a census equal to a hand-written expected census: every inventory row with its disposition and missing-mapping reason, every accepted row with its provenance, and their orderings. In that census an occurrence asserted only without a production function is `ZeroMappings(ExplicitNoProductionFunction)` and each QSL row is `ZeroMappings(NoAuthoredMapping)`; the QSL expectation is an IR reading pending owner confirmation (see Open questions). This grants no owner-native completeness and does not by itself back FR-052-AC-2. | Test |
| FR-052-AC-18 | Given synthetic outcomes, two faults in adjacent positions of the order (missing or foreign binding, Quire outcome, conflicting binding, IR V2, QSL, `quire-code-rs`) yield only the error for the earlier position. The tested pairs are a foreign binding with an incomplete Quire outcome (`ForeignCriterionSourceBinding`), an incomplete Quire outcome with a conflicting binding (`IncompleteOwnerInventory` for Quire), a conflicting binding with a missing IR V2 outcome (`ConflictingCriterionSourceBinding`), a missing IR V2 outcome with a missing QSL outcome (`MissingOwnerInventory` for IR V2), and an incomplete QSL outcome with a missing `quire-code-rs` outcome (`IncompleteOwnerInventory` for QSL). The conflicting-binding definition used is an IR reading pending owner confirmation. This does not by itself back FR-052-AC-6 and grants no AC-7 owner-native credit. | Test |
| FR-052-AC-19 | Given synthetic `Complete` outcomes, an assertion that is both foreign and dangling is rejected only with `ForeignSource`, and one repeated assertion ID whose members differ in both relation and provenance is rejected only with `ConflictingRelation`. This does not by itself back FR-052-AC-3. | Test |
| FR-052-AC-20 | Given the FR-052-AC-17 inputs, permuting the assertion order and the row order of each outcome yields a census equal to the unpermuted one. | Test |
| FR-052-AC-21 | Given synthetic outcomes, an outcome in one owner position that is both in the wrong position and incomplete yields `InventoryScopeMismatch`, and one that names the wrong source and is incomplete yields `InventorySourceMismatch`. | Test |
| FR-052-AC-22 | Given synthetic `Complete` outcomes, two assertion IDs stating one relation with equal provenance yield exactly one accepted row, and two assertion IDs stating one relation with differing provenance are both rejected with `ConflictingProvenance` and yield no accepted row. | Test |
| FR-052-AC-23 | Given synthetic `Complete` outcomes, an assertion whose occurrence lies under a `ConflictingIdentity` node, an assertion whose occurrence has no `SemanticNode` row, and an assertion whose production function has no `ProductionModule` row are rejected with `DanglingOccurrence`, `DanglingOccurrence` and `DanglingProductionSymbol` respectively; each orphan or conflicting row stays in the census. | Test |
| FR-052-AC-24 | Conflicting inventory facts (two `SemanticNode` rows with one `CheckedNodeId` node key and differing `semantic_type: CheckedNodeId` facts) receive their named deterministic disposition, `ConflictingIdentity`; each conflicting entry remains inspectable, and no occurrence under a conflicting node enters an accepted relation. | Test |
| FR-052-AC-25 | Given an owner-issued IR V2 reader result for a selected artifact, the model-owned adapter enumerates every node in an admitted package as its `node_id` plus native `semantic_type: CheckedNodeId` fact and every source occurrence under its node key, and issues `Complete` only after the graph reaches its normal end. Its selected source, IR V2 owner and scope, retained rows and observed count equal an independent traversal of that graph; deleting one emitted row fails that comparison even with an adjusted count. A reader `Incomplete` instead yields an incomplete outcome preserving the original `CheckedPackageIncomplete` limit kind, limit, consumed count and optional path; a reader refusal, including a node with no occurrence, never yields `Complete`. No adapter completion path accepts a caller-authored row vector, count or flag in place of the selected reader result. | Test |
| FR-052-AC-26 | Given a selected QSL checked request, the QSL-owned adapter enumerates every `CheckedGraph::requirements()` key and every native pending obligation, preserving each requirement's wire node bytes, occurrence role and ordinal and each obligation's declaration, unit, expression, byte span, kind and predicate target. Its `Complete` outcome matches an independent full-output traversal only after normal exhaustion, including a valid empty scope; an omitted row fails comparison even with an adjusted count. Native type-work exhaustion instead issues `Incomplete` with dimension, limit, prior, requested charge and site, and `TypeDisposition::Unfinished` issues a distinct `Incomplete` with the native declaration handle. No completion path accepts caller-authored rows, count or flag, or fabricates cancellation unsupported by the selected owner path. | Test |
| FR-052-AC-27 | Given a `quire-code-rs` selected production module, the owner adapter retains that module and every function fact from its complete native parse, including a parsed module with zero functions, and issues `Complete` only after the selected module's normal terminal state. Its source, owner, scope, rows and count equal independent full output; deleting one emitted row fails comparison even with an adjusted count. A missing selected module or failed parse instead issues `Incomplete` with the native module or parse cause and retained partial rows; no completion path accepts caller-authored rows, count or flag or fabricates cancellation unsupported by the selected owner path. | Test |

FR-052-AC-8 is retired and its ID is not reused (ADR-0056).
FR-052-AC-17 succeeds it. FR-052-AC-4 and AC-12 through AC-15, and AC-17
through AC-23 are Implemented for the neutral synthetic boundary with traced
tests. Assertion deduplication, foreign provenance, rejection precedence,
and rejection ordering are exercised by AC-19 and AC-22 tests.

FR-052-AC-1, AC-2, AC-3, AC-5 and AC-6 are Implemented for the neutral
synthetic boundary with the traced TC-452 tests. The external model-crate test
runs with default features; the AC-2 fixture verifies both many-to-many and
unmapped-function assertion sets, input permutations and occurrence removal.
AC-3 tests assert every named rejection and absence from accepted rows. AC-6
compares missing and typed incomplete outcomes at every owner position with a
complete module that has no functions. These synthetic tests grant no
owner-issued completeness, owner full-output comparison, or complete four-owner
census.

FR-052-AC-24 holds the conflicting-inventory-facts clause split from AC-3; its
obligation is unchanged. The inventory fact is the admitted node's
`semantic_type: CheckedNodeId`. AC-24 is Implemented by the public type and
traced synthetic conflict test. Four-owner completeness remains Planned.

AC-7, AC-9, AC-10 and AC-11 name owner scopes, owner-native enumerators or
owner adapter tests, and they remain Planned/UNRUN pending owner-native
adapters. AC-25 is Implemented by the IR V2 reader-result adapter and traced
tests. AC-26 and AC-27 remain Planned until their owner adapters and traced
tests land. Quire's selected-scope producer awaits PLAT-1165. AC-11's first sentence is exercised at the
neutral boundary by the AC-13 tests, but its second sentence needs owner
adapter tests, so AC-11 stays Planned as a whole. AC-16 remains Planned/UNRUN
pending PLAT-1165. Synthetic completion does not establish the full
matched-source FR-052 census or close IR-759. Passing the AC-17 through AC-23
checks does not by itself change the status of any other criterion.

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
