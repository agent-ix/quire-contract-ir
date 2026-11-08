---
id: FR-350
title: "Establish native temporal and TL result correspondence"
type: FR
relationships:
  - target: ix://agent-ix/quire-specification/FR-095
    type: implements
  - target: ix://agent-ix/quire-specification/FR-092
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-091
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/FR-045
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/FR-050
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/FR-052
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/FR-515
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/FR-516
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/FR-527
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/FR-526
    type: depends_on
  - target: ix://agent-ix/quire-observation/FR-004
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-011
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-012
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-013
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-014
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-018
    type: depends_on
  - target: ix://agent-ix/quire-mltl/FR-001
    type: depends_on
  - target: ix://agent-ix/quire-mltl/FR-003
    type: depends_on
  - target: ix://agent-ix/quire-mltl/FR-004
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-358
    type: references
  - target: ix://agent-ix/quire-contract-ir/AD-009
    type: references
---
# FR-350: Establish native temporal and TL result correspondence

## Description

When QSL FR-527 returns a `Supported` classification for a temporal
declaration occurrence in an admitted `quire.checked-package/v2` package, the
temporal bridge in `agent-ix/quire-integration` SHALL construct and owner-read
the TL formula, proposition map, trace or position history and request from
the native owner inputs, record one bridge-owned correspondence record, and
later join the native QSL FR-526 evaluator-issued result with the TL mapped result under that
record. The bridge SHALL take the target and outstanding premises from the QSL
classification and preserve the meaning QSpec FR-095 defines. This artifact is
the cross-repository bridge contract. It assigns no QSL-typed API to either
Contract IR package; [AD-009](../../assurance/AD-009-native-temporal-bridge.md)
states the component architecture. The integration contract IT-022 in
`agent-ix/quire-integration` specifies the integration scenarios that exercise
this contract and does not restate it.

## Inputs

Projection consumes exactly these owner values. Each is the value its owner
issued; the bridge accepts no copy, mirror, label or caller-authored substitute
for any of them.

| Input | Owner API | Use |
| --- | --- | --- |
| admitted package | `quire_contract_model::CheckedPackageV2` | package name, lock, source map, graph, `selected_model_packages()` and predicate terms |
| checked subject | `qsl_package::checked_views::TemporalSubjectView`, borrowed from that package by `temporal_subject` (QSL FR-515) | declaration, profile, occurrences, per-occurrence name and clock, captures, predicate leaves, formula graph, activation |
| classification | `qsl_package::checked_views::Classification` from `mapping_support` (QSL FR-527) | support disposition, target, outstanding premises, retained subject and sealed selection |
| surrounding closure | `qsl_package::checked_views::Closure` | selects the closed-trace or online-prefix future target |
| native request | `qsl_eval::temporal::evidence::EvidenceRequest` (QSL FR-516) | each member as stated in the disposition table below |
| exact occurrence | `qsl_package::checked_views::ClauseOccurrence` | node, role and ordinal of the assessed occurrence |
| admitted observation revision | `quire_integration::temporal::AdmittedObservationRevision` (IT-021) over a Quire Observation bundle | observation records keyed by observation identity, with clock and owner anchor |
| clock view | `quire_observation::authority::clock::View` | clock identity, clock revision and selected range |
| observation views | `quire_mltl::request::ObservationInputs` | decision and surrounding progress/closure, completeness and availability views |
| bridge definition | `ValidatedTlBridgeDefinition` from `read_tl_bridge_definition` (IT-022) | authority `agent-ix/quire-integration`, identity `native-temporal-correspondence/v1`, profile and version |
| owner limits | `tl_mltl::wire::OwnerLimits` | finite ceilings passed to the TL owner operations |
| bridge limits | `TlLimits` | the seven bridge counters below |

The join additionally borrows the original native whole call outcome
`Option<&Result<qsl_eval::temporal::evidence::EvaluatedEvidence, quire_eval::CallFailure>>`
(QSL FR-516 and FR-526), the TL `quire_mltl::contract_ir::ValidatedMappedResult`
with its source `ValidatedTemporalResult`, an optional previously admitted join
and the caller's `qsl_package::NativeJoinBudget` (FR-358) for owner package
projection. `None` is genuine absence; an original typed `Err` is not absence
or a native `NonValue`. Only the evaluator issues the opaque carrier. Its
`result()` borrows the complete original output immutably; a copied or modified
public `EvidenceResult` cannot substitute for it. `predecessor()` borrows this
result's identity for the next request, including for `Original`; the current
previous link is in `result().relation`.

Every `EvidenceRequest` member has exactly one disposition. **Checked** members
are compared in the binding step named; **recorded** members are written into
the correspondence record; **excluded** members carry no fact the TL side can
represent and are not read for TL construction. Exclusion does not remove
them from the original request binding checked at the join; their evaluation
effects remain native result outcomes, which the join compares.

| Member | Disposition |
| --- | --- |
| `binding`, `occurrence`, `control` | checked (steps 2, 3) |
| `cover` | checked (step 3): SHALL be `ClockCover::Exact` |
| `correspondence` | checked (step 4) and recorded (`native.correspondence`) |
| `trigger`, `trigger_evidence`, `trigger_scope`, `anchor`, `deliveries`, `origins`, `ledger`, `evicted` | checked (step 5); `trigger`, `trigger_scope` and `anchor` recorded |
| `positions` (each `observation` reference and `position` stamp) | checked (steps 6, 7) and recorded (`positions`) |
| `positions` `order_key` | recorded (`positions[].orderKey`) |
| `positions[].valuations`, `positions[].instance_valuations` | checked (step 9); selected Boolean cells recorded (`valuations`); all provided entries charged, including unselected instance maps |
| `positions[].aggregate_samples`, `positions[].instance_aggregate_samples` | excluded from TL construction and the correspondence record: not Boolean valuation inputs, not read or charged as entries by projection; retained in the original request identity binding at join. This exclusion does not admit aggregate atoms: the existing non-`holds` formula refusal still applies. |
| `decision_progress`, `decision_closure`, `surrounding_progress`, `surrounding_closure`, `completeness` | checked (step 8) and recorded (`native`) |
| `authoritative_origin` | checked (history premise) and recorded |
| `execution` | recorded (`native.execution`) and compared by the join |
| `relation` | compared by the join (Corrections) |
| `progress_ledger`, `limits` | excluded: native evaluator state and ceilings, whose effects appear only as native result outcomes |

## Outputs

`project_tl` returns either a `ValidatedTlProjection` that holds the exact
owner-admitted formula, proposition map, trace or history, and request values,
the native input facts it checked, its correspondence record bytes and its
usage counters, or one typed `TlProjectionError`. `join_tl` returns either a
`ValidatedTlJoin` that holds its decision, ordered causes, optional Boolean,
the original native call outcome and TL result values, the projection, the direct predecessor
inputs, its join record bytes and usage, or one typed `TlJoinError`. No
partial projection or join is observable on any error.

## Behavior

### Boundary and interface

The `agent-ix/quire-integration` bridge SHALL implement the public profile
`quire.native-temporal-correspondence/v1`. Its public operations SHALL be
equivalent to:

```text
temporal::read_tl_bridge_definition(bytes, TlLimits)
  -> Result<ValidatedTlBridgeDefinition, TlBridgeDefinitionError>

temporal::project_tl(
  package: &CheckedPackageV2,
  subject: &TemporalSubjectView,
  classification: &Classification,
  surrounding: Closure,
  native: NativeProjectionInput { request, occurrence },
  selection: TlProjectionSelection {
    bridge_definition, observation, clock, observations, owner_limits },
  limits: TlLimits,
) -> Result<ValidatedTlProjection, TlProjectionError>

temporal::read_tl_projection(
  bytes,
  ExpectedTlProjection { package, subject, classification, surrounding, native, selection },
  TlLimits,
) -> Result<ValidatedTlProjection, TlProjectionError>

temporal::join_tl(
  projection: &ValidatedTlProjection,
  native: Option<&Result<qsl_eval::temporal::evidence::EvaluatedEvidence, quire_eval::CallFailure>>,
  mapped: Option<&quire_mltl::contract_ir::ValidatedMappedResult>,
  previous: Option<&ValidatedTlJoin>,
  limits: TlLimits,
  native_budget: &mut NativeJoinBudget,
) -> Result<ValidatedTlJoin, TlJoinError>

temporal::read_tl_join(
  bytes,
  ExpectedTlJoin { projection, native, mapped, previous },
  TlLimits,
  native_budget: &mut NativeJoinBudget,
) -> Result<ValidatedTlJoin, TlJoinError>
```

`ValidatedTlBridgeDefinition`, `ValidatedTlProjection` and `ValidatedTlJoin`
SHALL be constructor-private: only the operations above construct them. No
operation SHALL accept a parser, evaluator, callback, trust flag, alias map or
caller-authored mirror of an owner record. The bridge SHALL invoke neither the
native evaluator (`evaluate_temporal_evidence`) nor the TL evaluator (`quire_mltl::report::evaluate`
or `quire_mltl::report::read`, which re-evaluates). It admits TL results only
as `ValidatedMappedResult` values, which only the quire-mltl readers construct;
this is the TL owner's accepted result input. Neither `quire-contract-model`
nor the `quire-contract-ir` root SHALL depend on a QSL crate to implement this
profile (FR-028-AC-3).

### Error vocabulary

`TlProjectionError::code()` SHALL return exactly one of these codes, with the
stated payload:

| Code | Payload |
| --- | --- |
| `tl-projection-binding` | the mapping-binding refusal (admission or reclassification) |
| `tl-projection-unsupported` | the complete classification |
| `tl-projection-incomplete` | one missing premise: `history`, `anchor`, `instance`, `activation`, `capture`, `complete-input`, `observation`, `position` or `valuation{position, operand}` |
| `tl-projection-native-premise` | one native premise: `occurrence`, `control`, `subject-name`, `clock-cover`, `correspondence-reference`, `anchor`, `activation`, `capture`, `observation-revision`, `observation`, `position`, `clock`, `clock-representation`, `decision-closure`, `surrounding-closure`, `progress`, `completeness`, `source` or `interval` |
| `tl-projection-valuation` | the original request position index and unknown native atom index |
| `tl-projection-native-clock` | the native `BindingMismatch` from `TraceBinding::authenticate` |
| `tl-projection-formula` | the formula constructor refusal, including a non-`holds` atom |
| `tl-projection-owner` | the typed TL owner producer or reader error |
| `tl-projection-encoding` | the canonical writer refusal of the correspondence record |
| `tl-projection-correspondence-mismatch` | none: supplied record bytes differ from the reconstructed record |
| `tl-projection-limit-reached` | `{limit, value, setting}`, and the owner error when an owner refused at a lowered byte ceiling |

The binding steps below assign every failure exactly one of these codes and
payloads. Steps run in order and the first failure is returned.

### Admission and support

1. **Admission.** The bridge SHALL require `subject.belongs_to(package)` and
   `classification.belongs_to(package)`, then re-run
   `mapping_support(subject, surrounding)` and require the complete result to
   equal the supplied classification (support and premises, retained
   declaration/profile/activation, sealed package/clause/definition
   selection). A separately admitted package with equal bytes and name is a
   different admission; a clone retaining the admission is the same one. Any
   failure returns `tl-projection-binding`. Then only `Supported` permits
   projection, with its target taken from the classification; `Unsupported`
   returns `tl-projection-unsupported`.

The support domain SHALL be exactly the QSL FR-045 and QSpec FR-095 table. It
includes each bounded past operator of QSpec FR-092, individually, under both
false-extension profiles when no future operator is reachable. A mixed
past-and-future graph, a timestamped-event (finite-window) profile and an
infinite-trace profile are unsupported. Decision-scope closure,
surrounding-execution closure, assessment execution, input completeness and
availability SHALL remain independent; surrounding closure selects between the
closed-trace and online-prefix targets and SHALL NOT substitute for
decision-scope closure. Every outstanding premise the classification names is
discharged by exactly one step below.

### Native input binding

2. **Occurrence.** `subject.arena_of(occurrence)` failing, a present
   `request.occurrence` unequal to `{occurrence.role, occurrence.ordinal}`, or
   an absent one with `subject.occurrences().len() != 1` returns
   native-premise `occurrence`. A present `request.control` not among
   `subject.requirements()` returns native-premise `control`. An absent
   `subject.name_of(occurrence)` returns native-premise `subject-name`; that
   name is the native subject name.
3. **Binding and cover.** `request.binding.authenticate` against the subject's
   declaration, profile and `clock_of(occurrence)` failing returns
   `tl-projection-native-clock`; its `AuthenticatedBinding` clock is retained.
   `request.cover` other than `ClockCover::Exact` returns native-premise
   `clock-cover`.
4. **Correspondence reference.** `request.correspondence.identity` SHALL equal
   the bridge definition identity `native-temporal-correspondence/v1` and its
   `authority_identity` the bridge authority `agent-ix/quire-integration`;
   otherwise native-premise `correspondence-reference`. These values are
   constant across corrections, so the native owner's `correspondence-changed`
   refusal never fires for a correction made through this bridge.
5. **Anchor, instance and captures.** An absent `request.anchor` returns
   incomplete `anchor`. The request anchor and selected trigger identify the
   genuine ledger instance or delivery/origin; no separate caller-authored
   instance or capture identity is accepted.
   `request.trigger_evidence` other than `Admitted`, or an absent
   `request.trigger`, returns incomplete `instance`. A contradiction reported
   by the native owner's `ActivationInput::contradiction` for the
   occurrence's obligation (built from `trigger_scope`, `trigger_evidence`,
   completeness, `deliveries`, `origins`, `request.anchor`, `evicted`,
   `ledger` and `execution`) returns native-premise `activation`.
   - When the `ledger` retains the instance: a retained fault of kind
     `Missing` returns incomplete `activation` and of kind `Refused`
     native-premise `activation`; a capture count unequal to the subject's
     capture names returns incomplete `capture`; a capture whose name differs
     from the declared name at its position or whose anchor differs from
     `request.anchor` returns native-premise `capture`. The retained payload
     and capture values are used.
   - Otherwise the first origin (`on origin`) or the first guarded delivery
     (`on each`, guard `Some(true)` when declared) whose identity equals
     `request.trigger` supplies them; none returns incomplete `instance`; a
     delivery anchor unequal to `request.anchor` returns native-premise
     `anchor`; a capture count mismatch, or a `Missing`, `Null` or `Stale`
     capture, returns incomplete `capture`; a `WrongType` capture or a value
     at another anchor returns native-premise `capture`.
   - An eviction of a capture of the selected instance returns incomplete
     `capture`; an eviction of a valuation of the selected instance, or of an
     unattributed valuation, returns incomplete `complete-input`.
6. **Observation.** Two bundle observation records with one observation
   identity return native-premise `observation`. For each
   request position in order: a position observation `authority_identity` or
   `authority_revision` unequal to the admitted bundle identity or decimal
   revision returns native-premise `observation-revision`; no bundle record with
   `positions[i].observation.identity` returns incomplete `observation`; a
   record whose clock identity or clock revision differs from the clock view's,
   or a stamp above the completeness `through`, returns native-premise
   `position`; an owner anchor not of the authenticated clock's form returns
   native-premise `clock`. The forms are: event position, where the clock's
   `sequence_authority` equals the clock view's clock identity and the owner
   anchor's decimal `position` equals the stamp; and fixed sample, where the
   owner anchor's decimal `index` equals the stamp and, when the clock unit is
   `nanoseconds`, the clock's integer epoch and period equal the owner
   anchor's decimal `epoch_nanos` and `period_nanos`. An epoch or period that
   is a rational with denominator other than 1 returns native-premise
   `clock-representation`. A non-nanosecond fixed-sample unit is never
   converted: for a future target it returns native-premise
   `clock-representation`; for a past target the epoch and period are passed
   unchanged and the TL request owner's typed refusal (quire-mltl FR-001-AC-9)
   is returned as `tl-projection-owner`.
   `request.anchor` equal to no position's observation identity returns
   incomplete `anchor`; equal to more than one returns native-premise
   `anchor`. The numeric TL anchor is that position's stamp minus the range
   start. The bridge SHALL NOT parse an opaque identity, infer an alias or
   substitute a numeric position for it.
7. **Positions.** A timestamped clock range returns native-premise `clock`; a
   range bound that is not a decimal `u64` returns native-premise
   `clock-representation`. A stamp outside `[start, end)` or a repeated stamp
   returns native-premise `position`; `end - start` unequal to the number of
   positions, or a gap in ascending stamp order, returns incomplete
   `position`. TL position order is ascending stamp order. For an
   origin-complete-history target, `start != 0` or `authoritative_origin !=
   Some(true)` returns incomplete `history`; a history cutoff is never an
   execution origin.
8. **Progress, closure and completeness.** Closures compare by state only:
   native `ProgressClosure::Closed`/`Open` against the QObs closure view's
   `payload().state()` `Closed`/`Open`. Progress compares the native `u64`
   watermark with the decimal `watermark` text of the QObs progress view's
   event-position or fixed-sample boundary, parsed as `u64`. An absent
   `decision_closure` returns incomplete `complete-input`; a differing one
   native-premise `decision-closure`. A `surrounding_closure` unequal to the
   supplied surrounding closure (`Complete` as `Closed`) or to the QObs view
   returns native-premise `surrounding-closure`. An absent or differing
   progress value, an unparsable watermark or a timestamped boundary returns
   native-premise `progress`. An absent or `Incomplete` `request.completeness`,
   or a QObs completeness state `Incomplete`, returns incomplete
   `complete-input`; a QObs state `Contradicted` returns native-premise
   `completeness`.
9. **Valuations.** The bridge SHALL use `native_atom_operands(subject)` and
   the checked subject's instance-bound atom selection to map native atom
   indices to checked predicate identities. It SHALL charge every supplied
   shared and instance-map entry at every request position, including foreign
   trigger maps not selected for evaluation. An unknown atom key in either
   map returns `tl-projection-valuation` with the original request position
   and atom index, in request-position and ordered-map traversal order (shared
   map before instance maps). For each TL position and mapped predicate leaf,
   a shared atom reads `positions[i].valuations[atom]`; an instance-bound atom
   reads only `positions[i].instance_valuations[request.trigger][atom]`.
   A foreign trigger's value SHALL NOT satisfy the selected cell. `True` and
   `False` supply explicit Booleans; `Missing`, `Undefined` or absence returns
   incomplete `valuation{position, operand}`. Maps admit one entry per position,
   atom and instance; duplicate-preserving or disagreeing rows have no request
   form. An unresolved atom-to-predicate or instance-bound selection returns
   native-premise `source`.

### Formula and valuation construction

- Construction SHALL traverse the admitted checked-package/v2 temporal
  operation graph in left-to-right postorder, preserve each occurrence without
  deduplication, and assign contiguous operand-before-consumer ids. Each
  integer interval `[a,b]` with `0 <= a <= b <= u32::MAX` is preserved; a timed
  interval, an integer interval without an upper bound, or a bound above
  `u32::MAX` returns native-premise `interval`.
- Each `holds` leaf SHALL map to one distinct TL proposition id. A formula atom
  that is not a `holds` leaf, including an aggregate atom, returns
  `tl-projection-formula`. Numeric, nullable, unavailable or pending values
  SHALL NOT become propositions.
- Future-only mappings SHALL use the formula and false-extension rules of QSpec
  FR-091; past-only mappings SHALL use the formula edition and origin-complete
  history of QSpec FR-092 and the TL owners' public contracts. A fixed-sample
  mapping SHALL retain its authenticated epoch, period and unit; an absent
  sample is incomplete `position`, and a changed period or event-count
  reinterpretation is refused (native-premise `clock`) even where formula
  bytes are equal.
- Every generated formula, proposition map, trace or position history and
  request SHALL pass its owner's public strict reader before use, under the
  supplied `OwnerLimits` with each byte ceiling lowered to `max_bytes`. An
  owner refusal returns `tl-projection-owner`. The bridge SHALL retain each
  exact admitted owner value and compute no content hash or derived identity.
- The source-map entry is the unique `package.source_map()` entry whose node,
  role and ordinal equal the occurrence; zero or several return native-premise
  `source`.

### Construction order

Projection SHALL construct its values in this order; each step consumes only
values from earlier steps or the inputs, so no value depends on a later one:

1. Admission, support and native input binding (steps 1 to 9 above).
2. The source-map entry.
3. TL formula and proposition map, then the trace (future) or position
   history (past) from the step-9 cell values; each owner-read.
4. `quire_mltl::request::derive` with `RequestInput { formula,
   proposition_map, input, clock, subject_identity, correspondence_identity,
   anchor, observations }`, where `subject_identity` is the native subject
   name and `correspondence_identity` is the constant profile name
   `quire.native-temporal-correspondence/v1`; then
   `quire_mltl::request::read` of the derived bytes against the same input.
5. The correspondence record, which contains the exact request bytes from
   step 4.

The request SHALL NOT contain the correspondence record, its bytes or any value
derived from them. The native `request.correspondence` names the bridge
definition, not the record. A wrong subject or profile name, or a request
unequal to the value retained in the record, SHALL refuse before a projection
is returned.

### Correspondence record

The correspondence record SHALL be the RFC 8785 canonical JSON serialization of
one object with exactly the members below, produced by the bridge's canonical
writer under `max_bytes`. Encodings:

- **hex**: the lowercase hexadecimal string of the exact owner bytes.
- **dec**: the base-10 string, without leading zeros (`"0"` for zero), of an
  integer; a negative integer carries a leading `-`.
- **str**: the exact owner string. **bool**: a JSON Boolean.
- **IR**: the `quire-contract-model` serde JSON value of that type, in which
  every JSON number is replaced by its dec string. An IR value containing a
  non-integer number returns `tl-projection-encoding`.
- JSON `null` encodes `None` where the table says so. No JSON number appears in
  the record. Array order is the stated order and is significant.

| Member | Encoding | Value |
| --- | --- | --- |
| `profile` | str | `quire.native-temporal-correspondence/v1` |
| `bridgeDefinition` | hex | `ValidatedTlBridgeDefinition` record bytes |
| `package` | str | `CheckedPackageV2::name()` |
| `lock` | IR | `CheckedPackageV2::lock()` |
| `source` | IR | the source-map entry |
| `declaration` | object | `node`: IR `CheckedNodeId`; `role`: IR; `ordinal`: dec; `name`: str native subject name |
| `subjectProfile` | IR | `TemporalSubjectView::profile()` |
| `models` | array of str | `CheckedPackageV2::selected_model_packages()` in the owner's ascending identity order; no alias member |
| `bindings` | array of object | per TL proposition in id order: `predicate`: IR `CheckedNodeId`; `term`: IR `CheckedPackageV2::term(predicate)`; `proposition`: dec |
| `intervals` | array of object | per `TemporalSubjectView::formula_graph()` node in owner order: `node`: IR; `operation`: IR; `lower`: dec or null; `upper`: dec or null |
| `anchor` | str | `EvidenceRequest::anchor` |
| `captures` | array of hex | the step-5 capture values in declaration capture order |
| `trigger` | hex | `EvidenceRequest::trigger` bytes |
| `triggerPayload` | hex | the step-5 instance payload |
| `clock` | IR | the `AuthenticatedBinding` `ClockConfiguration` |
| `observation` | hex | admitted observation bundle bytes |
| `clockView` | hex | clock view bytes |
| `decisionProgress`, `decisionClosure`, `surroundingProgress`, `surroundingClosure`, `completenessView`, `availabilityView` | hex | the six `ObservationInputs` view bytes |
| `surrounding` | str | `complete` or `open` |
| `authoritativeOrigin` | bool or null | `EvidenceRequest::authoritative_origin` |
| `positions` | array of object | per TL position in TL order: `index`: dec native request index; `stamp`: dec; `observation`: an `EvidenceRef` object; `orderKey`: null or object `authority`: str, `key`: dec |
| `formula`, `propositionMap`, `request` | hex | the owner-admitted formula, proposition map and request bytes |
| `input` | object | `kind`: `future` or `past`; `bytes`: hex trace or history bytes |
| `valuations` | array of object | per selected Boolean cell in ascending `(request position index, CheckedNodeId)` order: `position`: dec; `operand`: IR `CheckedNodeId`; `value`: bool |
| `native` | object | `correspondence`: an `EvidenceRef` object; `triggerScope`: `open` or `closed`; `decisionProgress`, `surroundingProgress`: dec or null; `decisionClosed`, `surroundingClosed`: bool or null; `execution`: `completed` or `failed`; `completeThrough`: dec or null; `completenessFacts`: array of str; `completenessReference`: an `EvidenceRef` object |

An `EvidenceRef` object has members `identity`, `authorityIdentity`,
`authorityRevision`, `scope`, `population` and `contract` (str) and
`schemaDigest`, `digest` and `authorityDigest` (hex), echoing those opaque
owner values. The record carries complete values and public names, never a
seal, digest or reference computed by the bridge. `read_tl_projection` SHALL
refuse a supplied byte value longer than `max_bytes`, reconstruct the expected
projection from the independently supplied inputs through the same owner
constructors and readers, and require byte equality with the supplied bytes;
any unknown, duplicate, missing, reordered, non-canonical or trailing content,
or any changed value, returns `tl-projection-correspondence-mismatch`.

### Join decision

`join_tl` SHALL borrow the original evaluator-issued carrier and compare its
immutable result subject's package identity, checked package version, clause,
occurrence role and ordinal, and source-map entry with the projection. A mismatch
is native `refused`. It SHALL project the owner `EvidencePackage` under
`NativeJoinBudget`, select its subject and use `matches_result_identity` to bind
the original request to the carrier's result identity; a mismatch is native
`refused`. This function authenticates inputs only: the private carrier, not
identity equality, supplies authority for actual truth, support, settlement and
ledger. No native report serializer, strict result reader, predecessor-byte
copy or result-byte storage is used. It SHALL re-read the
mapped result with `quire_mltl::contract_ir::read(mapped.bytes(),
mapped.source(), &MappingSelection::for_result(mapped.source()), owner_limits)`,
where `owner_limits` is the projection's `owner_limits` with byte ceilings
lowered to `max_bytes`; the selection is computed by the owner, so the bridge
still computes no hash. A refused read becomes the side's state and is never
used as a value.

Each side SHALL have exactly one state. Native subject, request and lineage
checks precede result classification. Within an admitted result, `incomplete`
or `indeterminate` precedes `NonValue`, which precedes activation and truth.
Original admission failures retain their refused/incomplete distinction; an
original fault is failed. Other call families are whole-operation errors.
The TL side uses the first column (left to right) whose condition holds:

| Side | `refused` | `lineage` | `failed` | `unavailable` | `unsupported` | `incomplete` | `contradicted` | `final(b)` | `pending` |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| native | checked subject or request mismatch; original admission `Refused`; result `NonValue::Refused` | predecessor identity or lineage mismatch as below | original `CallFailure::Fault` | absent outcome; result `NonValue::{Inactive,Unactivated,ActivationUnknown}`; activation `Inactive`, `Unknown` or `Unavailable` | never | original admission `Incomplete`; result `incomplete` or `indeterminate`; result `NonValue::Missing` | never | active result with truth `True`/`False`, settlement `ClosedScope`, `DecisiveWitness` or `DecisiveCounterexample` | every other active result |
| TL | mapping re-read refused, execution `Refused`, or source request, `subject_identity` or `correspondence_identity` unequal to the projection's request | never | execution `Failed` | result absent, outcome `NonValue{Unavailable}`, or availability not `Available` | execution `Unsupported` | execution `ResourceIncomplete`, or completeness `Incomplete` | completeness `Contradicted` | outcome `Value{b}` and `is_final()` | every other result (execution `Completed`) |

Each row is total: every native result outcome and every TL execution,
completeness, availability and outcome combination falls in some column.

**Premise equality** `P` holds when the TL source request equals the
projection's request; native `request.execution` `Completed`/`Failed` equals TL
execution `Completed`/`Failed`; the TL decision and surrounding progress and
closure, completeness and availability references equal the projection
request's; the native settlement equals the TL settlement basis (`None` or
`Unavailable` matches `Unavailable`; otherwise like-named variants); and the
set of observation identities `s.observation.identity`
over native support entries `s` equals the set of TL `decision_support()`
identities. **Lineage conflict** `L` holds as defined under Corrections.

| Row | Predicate | Decision / comparison | Cause |
| --- | --- | --- | --- |
| R1 | either side `refused` | `refused` / `not-compared` | `result-refused` |
| R2 | native `request.execution` is `Failed`, or either side `failed` | `failed` / `not-compared` | `result-failed` |
| R3 | either side `unavailable` | `unavailable` / `not-compared` | `result-unavailable` |
| R4 (defensive) | TL side `unsupported` | `unsupported` / `not-compared` | `result-unsupported` |
| R5 | either side `incomplete` | `incomplete` / `not-compared` | `result-incomplete` |
| R6 | `L`; or native side `lineage`; or TL side `contradicted`; or both sides in `{final, pending}` and neither R7 nor R8 holds | `conflict` / `mismatch` | `result-mismatch` |
| R7 | native `final(b)`, TL `final(b)` with the same `b`, `P`, and not `L` | `agreement` / `equal-final`, `b` | none |
| R8 | both sides `pending`, native truth `Pending` and settlement `Unsettled`, TL truth `Pending` and basis `Unsettled`, native `decision_closure` `Open` and TL decision-scope closure `Open`, `P`, and not `L` | `agreement` / `equal-pending`, no Boolean | `result-pending` |

R7 and R8 do not refer to R6, so R6 is well defined. The decision SHALL be the
first of R1 to R6 that holds; when none holds, R7 if it holds, else R8.
**Totality**: if a side is `refused`, `failed`, `unavailable`, `unsupported`
or `incomplete`, or the native request execution is `Failed`, one of R1 to R5
holds. If a side is `lineage` or `contradicted`, R6 holds. Otherwise both sides
are in `{final, pending}`: R7 requires both `final` and R8 both `pending`, so
at most one holds, and R6 holds exactly when neither does (including an
equal-final or equal-pending pair with `P` false, and a final/pending pair).
Every pair therefore has exactly one decision.

R4 is a defensive total-table row. A TL `Unsupported` execution cannot arise
from a bridge-built `Supported` request: quire-mltl returns `Unsupported` only
for an unsupported profile or past node, and any other request is refused at
R1. R4 is therefore excluded from FR-350-AC-5's reachable-row coverage.

The cause list SHALL hold the cause of every R1 to R6 row whose predicate
holds, once each, in row order, followed by `result-pending` when R8 is the
decision; an R7 decision has an empty cause list. Only R7 carries a Boolean.

### Corrections

A native issued result whose `relation` is not `Original`, or a TL source
result whose relation is not `Original`, is a correction. For the native side,
`Original` requires no previous join. A correction requires a previous join
whose native side was admitted, its original issued carrier's `result_identity`
equal to the current relation's predecessor, and its lineage plus one (checked
arithmetic) equal to the current lineage. Failure gives native state `lineage`
and the typed predecessor mismatch. The carrier preserves the actual output;
QSL admission already requires changed inputs, the authentic evaluator-written
ledger's latest predecessor and unchanged obligation/correspondence. Input
identity alone never authenticates corrected truth or ledger.

`L` SHALL hold when exactly one side is a correction; both are originals and
`previous` is supplied; both are corrections and `previous` is absent or its
native side was not admitted; or
`mapped.source().validate_supplied_predecessor(previous.mapped().map(|m|
m.source().bytes()))` refuses (quire-mltl FR-001). TL complete predecessor-byte
validation remains required. Native lineage is bounded by QSL FR-523's
`lineage` counter, with no native predecessor-byte work or storage budget.
Earlier carriers and ledgers remain unchanged. `L` SHALL be evaluated only
when a native successful carrier and a TL result are supplied; an absent side
or original native error does not add `result-mismatch` on account of `L`.

### Join record

The join record SHALL be the RFC 8785 canonical JSON serialization of one
object with exactly these members, under the same encodings:

| Member | Encoding | Value |
| --- | --- | --- |
| `version` | str | `quire.native-temporal-join/v1` |
| `correspondence` | hex | the projection's correspondence record bytes |
| `native` | hex, object or null | issued result identity bytes; original typed failure object below; or genuine absence |
| `nativeAdmitted` | bool | whether carrier subject, request and lineage checks admitted |
| `mapped` | hex or null | `ValidatedMappedResult::bytes()` |
| `tlResult` | hex or null | the mapped result's source `ValidatedTemporalResult::bytes()` |
| `previous` | object or null | the previous join's `native` (hex, object or null) and `tlResult` (hex or null) members; earlier lineage is not retained |
| `decision` | str | `refused`, `failed`, `unavailable`, `unsupported`, `incomplete`, `conflict` or `agreement` |
| `comparison` | str | `not-compared`, `mismatch`, `equal-final` or `equal-pending` |
| `causes` | array of str | the ordered cause list |
| `value` | bool or null | the R7 Boolean |

For an original admission failure, `native` is an object with exactly
`failure: "admission"`, `kind: "refused"` or `"incomplete"`, `code`, `cause`,
`prefix`, `fields`, `member`, `index` (decimal string or null), and `path`.
`fields` retains the owner's name/value object; each path step is
`{"field": name}`, `{"option": true}` or `{"index": decimal-string}`.
For an original fault, it has exactly `failure: "fault"`, `stage`, `invariant`
and `named` (the original owner's serde representation of named fields). No Debug or Display text
is parsed. Admission failures outside member loci or carrying a kernel cause,
faults with a kernel cause, and other call families return the original typed
whole-operation `tl-join-native-report` refusal rather than erasing detail.

`read_tl_join` SHALL rebuild the expected join from the expected projection,
results and previous join and require byte equality.

`TlJoinError::code()` SHALL return exactly one of `tl-join-expected-mismatch`
(supplied join bytes differ from the rebuilt join), `tl-join-limit-reached`
(a bridge limit, or an exhausted native package projection budget
retaining the native owner's resource and attempted charge),
`tl-join-encoding` (canonical writer refusal) and `tl-join-native-report`
(another typed native package or evidence binding refusal). These are whole-operation refusals,
not decision rows, and leave no join.

### Limits

`TlLimits` SHALL hold seven caller settings, each defaulting to the value below
and exposed by `TlLimits::default()`. Each operation starts fresh counters and
exposes them as its usage. Each unit SHALL be charged before the work it pays
for; a counter equal to its setting admits, and the charge that would make it
`setting + 1` SHALL stop the operation with `limit-reached{limit, value:
setting + 1, setting}` and no partial value, leaving the rejected unit
unvisited. When one action would cross several limits, the first in table
order is the sole returned limit. A caller may set any positive value to
exercise a boundary.

The expected count `n` of each limit is defined by the operation's inputs, not
by the bridge's own accounting:

| Limit | Setting | Default | Expected count `n` of a successful operation |
| --- | --- | --- | --- |
| `max_bytes` | `temporal.correspondence.max_bytes` | 16,777,216 | the length of the largest single byte value the operation checks. Projection checks the bridge definition record, bundle, clock view, six observation views, formula, proposition map, trace or history, request and correspondence record; join checks each current/previous native identity or diagnostic byte size, the canonical current native outcome encoding, mapped bytes, source result bytes and join record; each strict reader also checks the supplied bytes. Usage reports that largest length. |
| `max_nodes` | `temporal.correspondence.max_nodes` | 65,536 | the node count of the owner-read TL formula |
| `max_positions` | `temporal.correspondence.max_positions` | 1,048,576 | `request.positions.len()` |
| `max_valuations` | `temporal.correspondence.max_valuations` | 16,777,216 | the total entries in all request positions' shared valuation maps and every instance valuation map, including unused foreign-trigger maps |
| `max_support` | `temporal.correspondence.max_support` | 1,048,576 | when native carrier binding and TL mapped re-read admit, `native.result().support.len()` plus `decision_support().len()`; otherwise 0 |
| `max_causes` | `temporal.correspondence.max_causes` | 64 | the length of the join's cause list |
| `max_work` | `temporal.correspondence.max_work` | 4,294,967,296 | projection: the sum of the source-map entry count, bundle record count, `request.positions.len()`, all provided valuation-map entries, TL positions times native atom operands, TL formula node count, proposition count, `formula_graph()` length, selected model count, ledger instance and result counts, deliveries plus their captures, origins plus their captures, eviction count, and retained capture count; join: the `max_support` count plus current and previous native diagnostic visits (admission: field pairs plus path steps plus one; fault: named pairs plus one) |

For a native admission diagnostic, its checked byte size is the sum of member
name, field names and values, and field-name path steps; indices and option
steps add no string bytes. For a fault it is the sum of stage, invariant, and
named field names and values. Each current or previous issued identity is also
checked at its own byte length. The current native outcome's canonical encoding
and final join record are separately checked; these bridge-owned encodings do
not restore a native report serializer or predecessor-byte budget.

Every listed item is visited exactly once per operation, so each count is
computable from the inputs. A strict reader's counts equal those of the
operation it reconstructs. Owner-internal work is bounded by the owners'
`OwnerLimits` and `NativeJoinBudget` and is not
counted here. An owner refusal at a byte ceiling lowered to `max_bytes` SHALL
be reported as `limit-reached{limit: max_bytes, value: setting + 1, setting}`
retaining the owner error; `value` then states that at least one byte exceeds
the setting. This mapping is defensive and unreachable when the bridge's own
input check precedes owner calls; it is not a tested AC clause.

For `max_bytes`, the measured `n` of a correspondence or request record is its
length computed with the setting itself embedded, because the quire-mltl
request wire embeds the lowered `max_input_bytes` and `max_output_bytes`. The
boundary is therefore the fixed point: the smallest setting `s` with
`record_len(s) <= s` admits and `s - 1` refuses with `limit-reached{limit: max_bytes,
value: record_len(s - 1), setting: s - 1}`; `value` is the record length measured
with the setting `s - 1` embedded, which equals `s` only when `s - 1` and `s` have
the same number of digits.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-350-AC-1 | Given an admitted package and a QSL `Supported` classification, projection takes its target from the classification and returns owner-admitted formula, proposition map, trace or history and request values for every supported row: future-only under complete and open surrounding closure (closed-trace and online-prefix targets), and each bounded past operator of QSpec FR-092 alone under both false-extension profiles with origin-complete history. A non-zero range start or an `authoritative_origin` other than `Some(true)` for a past target returns incomplete `history`. | Test |
| FR-350-AC-2 | Timestamped-event (finite-window), mixed past-and-future and infinite-trace classifications return `tl-projection-unsupported` carrying the complete classification and no formula, request or record. A changed support, retained field, definition selection or surrounding closure, a foreign subject, or a classification from another admission of equal bytes returns `tl-projection-binding`; a clone retaining the admission is accepted. | Test |
| FR-350-AC-3 | Mutating each correspondence-record member's source input independently (source entry, declaration, lock, subject profile, model selection, binding, interval, anchor, capture, trigger, payload, clock, each observation view, surrounding, origin, each position's observation reference and order key, owner document, selected valuation cell, and each `native` member including the correspondence reference and trigger scope) changes exactly the corresponding record member, and `read_tl_projection` of the original bytes against the mutated inputs returns `tl-projection-correspondence-mismatch`; a non-`holds` atom returns `tl-projection-formula`. | Test |
| FR-350-AC-4 | Fixed-sample projection with a nanosecond unit retains epoch, period and unit and matches them to the owner anchor by exact equality; a changed period or event-count reinterpretation with equal formula bytes returns native-premise `clock`; a missing sample returns incomplete `position`; a non-nanosecond unit is not converted and returns native-premise `clock-representation` for a future target and `tl-projection-owner` with the TL request owner's refusal for a past target. | Test |
| FR-350-AC-5 | For each of R1 to R8 a genuine owner-produced pair reaches that row with the stated decision, comparison and cause list, excluding the defensive row R4, including an equal-final pair with `P` false and a final/pending pair (R6), a TL `Failed` result (R2) and a native predecessor mismatch (R6); for each pair of rows R1 to R6 a pair satisfying both returns the first decision and both causes once in row order; only R7 carries a Boolean; every join retains the complete native and TL result values. Each lineage condition of `L` and a different prior issued carrier or TL equal-identity different-content predecessor yields R6, while a correction whose native issued identity and lineage and TL complete direct predecessor match the previous join's admitted results reaches R7. | Test |
| FR-350-AC-6 | Each binding failure returns its assigned code and premise; an unknown native atom key returns `tl-projection-valuation` with original request position and atom index; explicit false projects as false; missing, undefined or absent selected values return incomplete `valuation{position, operand}`, and a foreign trigger's map cannot supply them. Each owner-reader refusal retains the typed owner error; alternate projection/join bytes return their mismatch code, with no partial value. Joins retain the original immutable native carrier or typed call failure and unchanged TL owner usage. No bridge operation calls `evaluate_temporal_evidence`, `quire_mltl::report::evaluate` or `quire_mltl::report::read`. | Test, Inspection |
| FR-350-AC-7 | Withdrawn: the earlier fixed byte vector, SHA-256 known answers and derived correspondence reference were removed under the no-hash rule. This id is retained and never reused. | Withdrawn |
| FR-350-AC-8 | For each of the seven limits, `TlLimits::default()` returns the stated default; with the other six settings above their counts, an operation whose expected count `n`, computed by the test from its inputs by the Limits table, admits at setting `n` and, at setting `n - 1`, returns `limit-reached{limit, value: n, setting: n - 1}` with the rejected unit unvisited and no partial value; the operation's reported usage equals `n`; an action crossing two limits at once returns only the first in table order; for `max_bytes` the measured `n` is the record length with the setting embedded, so the boundary is the smallest setting `s` with `record_len(s) <= s`, which admits, while `s - 1` refuses with `value` equal to `record_len(s - 1)`. Projection exercises `max_bytes`, `max_nodes`, `max_positions`, `max_valuations` and `max_work`; join exercises `max_bytes`, `max_support`, `max_causes` and `max_work`; each strict reader exercises `max_bytes`. | Test |
| FR-350-AC-9 | Given the authenticated binding, exact occurrence and original request maps, projection selects shared or selected-trigger Boolean values through the native atom-to-predicate mapping and maps the opaque request anchor by exact equality with one admitted position observation identity, including when it resembles another numeric position. An absent request occurrence selects the sole occurrence. Wrong binding, node/role/ordinal, ambiguous occurrence, foreign control, mismatched delivered anchor/capture, absent observation or duplicate observation each returns the assigned refusal before any TL artifact. Changing a selected request valuation changes the projected cell; a shared value cannot replace an instance-bound value, and a foreign trigger cannot supply it. | Test |
| FR-350-AC-10 | The projection and join retain the exact supplied package admission; the `models` member equals `selected_model_packages()` in its ascending identity order, and two packages selecting the same model set in different source orders yield an identical `models` member. A correction join retains the original prior native carrier and TL direct predecessor; `previous` records the native issued identity or typed failure and complete TL result bytes. A different prior issued carrier or different complete TL predecessor of equal result identity refuses strict reading. | Test |
| FR-350-AC-11 | The test constructs the expected correspondence record and join record independently from the owner inputs by the member tables and encodings above (no JSON number appears) and requires byte equality with the bridge's output for a future, a past and a fixed-sample projection and for an R7 and a correction join; the request bytes inside the record equal the owner-read request, whose `subject_identity` is the native subject name and `correspondence_identity` the profile name, and the request contains no record bytes. A wrong subject or profile name refuses before a projection is returned. | Test |

## Dependencies

- `agent-ix/quire-integration` owns the QSL-typed bridge, the bridge
  definition, strict projection and join readers, the correspondence and join
  records, and the executable TC-461 evidence. FR-028-AC-3 and TC-041 keep both
  Contract IR packages free of QSL dependencies.
- QSL FR-045 defines the support table; FR-515 supplies the checked subject;
  FR-527 supplies `mapping_support`; FR-516 supplies `EvidenceRequest`;
  FR-050 supplies the admitted package and temporal binding; FR-526 supplies
  the immutable `EvaluatedEvidence` carrier, checked result subject, support
  and evaluator-written ledger lineage authority. FR-516 owns request maps
  and typed call outcomes; its successor removes the retired native report API.
- QSpec FR-095 owns the correspondence meaning; FR-092 the bounded-past
  semantics; FR-091 the future false-extension rules; FR-322 the structural
  package names and source references.
- Quire Observation FR-004 supplies observation, clock, progress, closure,
  completeness and availability views.
- tl-syntax FR-011 to FR-014 and tl-mltl FR-018 supply the formula, proposition
  map, trace and history contracts and readers. quire-mltl FR-001 supplies
  `RequestInput`, request derivation and reading, result admission, the
  public direct-predecessor bytes and `validate_supplied_predecessor`;
  quire-mltl FR-003 supplies the mapped result, `MappingSelection::for_result`
  and its non-evaluating reader.
