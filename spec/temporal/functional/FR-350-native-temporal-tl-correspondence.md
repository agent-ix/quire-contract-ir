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
  - target: ix://agent-ix/quire-spec-language/FR-043
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
later join the native QSL FR-052 report with the TL mapped result under that
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
| native valuations | `quire_spec_language::temporal::ValuationEvidence` (QSL FR-043) | the ordered six-bound Boolean relation; it is the sole valuation source |
| exact occurrence | `qsl_package::checked_views::ClauseOccurrence` | node, role and ordinal of the assessed occurrence |
| assessed instance | `quire_spec_language::temporal::ValuationInstance` | the instance's anchor and immutable capture-environment identity |
| native observation | `quire_spec_language::temporal::ObservationRef` | the observation and revision references the native report echoes |
| admitted observation revision | `quire_integration::temporal::AdmittedObservationRevision` (IT-021) over a Quire Observation bundle | observation records keyed by observation identity, with clock and owner anchor |
| clock view | `quire_observation::authority::clock::View` | clock identity, clock revision and selected range |
| observation views | `quire_mltl::request::ObservationInputs` | decision and surrounding progress/closure, completeness and availability views |
| bridge definition | `ValidatedTlBridgeDefinition` from `read_tl_bridge_definition` (IT-022) | authority `agent-ix/quire-integration`, identity `native-temporal-correspondence/v1`, profile and version |
| owner limits | `tl_mltl::wire::OwnerLimits`, `quire_spec_language::temporal::ReportPredecessorLimits` | finite ceilings passed to the TL and native owner operations |
| bridge limits | `TlLimits` | the seven bridge counters below |

The join additionally consumes the native `quire_spec_language::temporal::Report`
(QSL FR-052), the TL `quire_mltl::contract_ir::ValidatedMappedResult` (quire-mltl
FR-003) with its source `ValidatedTemporalResult`, an optional previously
admitted join and the caller's `qsl_package::NativeJoinBudget` (FR-358).

Every `EvidenceRequest` member has exactly one disposition. **Checked** members
are compared in the binding step named; **recorded** members are written into
the correspondence record; **excluded** members carry no fact the TL side can
represent, are not read, and reach the join only through the native report's
outcome, which the join compares.

| Member | Disposition |
| --- | --- |
| `binding`, `occurrence`, `control` | checked (steps 2, 3) |
| `cover` | checked (step 3): SHALL be `ClockCover::Exact` |
| `correspondence` | checked (step 4) and recorded (`native.correspondence`) |
| `trigger`, `trigger_evidence`, `trigger_scope`, `anchor`, `deliveries`, `origins`, `ledger`, `evicted` | checked (step 5); `trigger`, `trigger_scope` and `anchor` recorded |
| `positions` (each `observation` reference, `position` stamp and `order_key`) | checked (steps 6, 7) and recorded (`positions`) |
| `positions` valuation and aggregate maps | excluded: the native owner replaces them with `ValuationEvidence` before evaluation |
| `decision_progress`, `decision_closure`, `surrounding_progress`, `surrounding_closure`, `completeness` | checked (step 8) and recorded (`native`) |
| `authoritative_origin` | checked (history premise) and recorded |
| `execution` | recorded (`native.execution`) and compared by the join |
| `relation` | compared by the join (Corrections) |
| `native_valuations` | excluded: the native owner sets it from `ValuationEvidence` |
| `progress_ledger`, `limits` | excluded: native evaluator state and ceilings, whose effects appear only as native report outcomes |

## Outputs

`project_tl` returns either a `ValidatedTlProjection` that holds the exact
owner-admitted formula, proposition map, trace or history, and request values,
the native input facts it checked, its correspondence record bytes and its
usage counters, or one typed `TlProjectionError`. `join_tl` returns either a
`ValidatedTlJoin` that holds its decision, ordered causes, optional Boolean,
the native and TL result values, the projection, the direct predecessor
results, its join record bytes and usage, or one typed `TlJoinError`. No
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
  native: NativeProjectionInput { request, valuations, occurrence, instance, observation },
  selection: TlProjectionSelection {
    bridge_definition, observation, clock, observations, owner_limits, native_report_limits },
  limits: TlLimits,
) -> Result<ValidatedTlProjection, TlProjectionError>

temporal::read_tl_projection(
  bytes,
  ExpectedTlProjection { package, subject, classification, surrounding, native, selection },
  TlLimits,
) -> Result<ValidatedTlProjection, TlProjectionError>

temporal::join_tl(
  projection: &ValidatedTlProjection,
  native: Option<&quire_spec_language::temporal::Report>,
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
native evaluator (`evaluate_v2`) nor the TL evaluator (`quire_mltl::report::evaluate`
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
| `tl-projection-valuation` | the `ValuationCause` and admitted row index |
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
   incomplete `anchor`. A `request.anchor` unequal to `instance.anchor`, or an
   empty `instance.capture_instance`, returns native-premise `anchor`.
   `request.trigger_evidence` other than `Admitted`, or an absent
   `request.trigger`, returns incomplete `instance`. A contradiction reported
   by the native owner's `ActivationInput::contradiction` for the
   occurrence's obligation (built from `trigger_scope`, `trigger_evidence`,
   completeness, `deliveries`, `origins`, `instance.anchor`, `evicted`,
   `ledger` and `execution`) returns native-premise `activation`.
   - When the `ledger` retains the instance: a retained fault of kind
     `Missing` returns incomplete `activation` and of kind `Refused`
     native-premise `activation`; a capture count unequal to the subject's
     capture names returns incomplete `capture`; a capture whose name differs
     from the declared name at its position or whose anchor differs from
     `instance.anchor` returns native-premise `capture`. The retained payload
     and capture values are used.
   - Otherwise the first origin (`on origin`) or the first guarded delivery
     (`on each`, guard `Some(true)` when declared) whose identity equals
     `request.trigger` supplies them; none returns incomplete `instance`; a
     delivery anchor unequal to `instance.anchor` returns native-premise
     `anchor`; a capture count mismatch, or a `Missing`, `Null` or `Stale`
     capture, returns incomplete `capture`; a `WrongType` capture or a value
     at another anchor returns native-premise `capture`.
   - An eviction of a capture of the selected instance returns incomplete
     `capture`; an eviction of a valuation of the selected instance, or of an
     unattributed valuation, returns incomplete `complete-input`.
6. **Observation.** `observation.reference` and `observation.revision`
   unequal to the admitted bundle's identity and decimal revision return
   native-premise `observation-revision`. Two bundle observation records with
   one observation identity return native-premise `observation`. For each
   request position in order: no bundle record with
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
9. **Valuations.** For each `ValuationEvidence` row in admission order the
   bridge SHALL apply the native owner's check order and return
   `tl-projection-valuation` with the first failing cause and that row's
   index: `OccurrenceNotDeclared` (subject unequal to the occurrence),
   `OperandNotInFormula` (operand not in `subject.predicate_leaves()`),
   `PositionNotAdmitted` (position not below `request.positions.len()`),
   `AnchorMismatch` (anchor unequal to `instance.anchor`),
   `CaptureInstanceMismatch` and `RevisionMismatch` (revision unequal to
   `observation.revision`). Then for each TL position and predicate leaf the
   cell value is the value all rows for that `(position, operand)` agree on,
   as the native owner's `value_of` defines it: rows that agree, including
   repeated rows, give that value and every row is kept in the record; no row,
   or rows that disagree, return incomplete `valuation{position, operand}`.
   A missing value is never false; false is an explicit row.

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
| `anchor` | str | `ValuationInstance::anchor` |
| `captureInstance` | str | `ValuationInstance::capture_instance` |
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
| `valuations` | array of object | per `ValuationEvidence` row in admission order, all rows kept: `node`: IR `CheckedNodeId`; `role`: IR; `ordinal`: dec; `position`: dec; `anchor`: str; `captureInstance`: str; `revision`: str; `operand`: IR `CheckedNodeId`; `value`: bool |
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

`join_tl` SHALL write the native report through `Report::to_owner_bytes_bounded`
under `native_report_limits` and strict-read it with `Report::from_owner_bytes`
against `CheckedSubject::new(package, occurrence, valuations, request)`, the
projection's native observation and `native_budget`. It SHALL re-read the
mapped result with `quire_mltl::contract_ir::read(mapped.bytes(),
mapped.source(), &MappingSelection::for_result(mapped.source()), owner_limits)`,
where `owner_limits` is the projection's `owner_limits` with byte ceilings
lowered to `max_bytes`; the selection is computed by the owner, so the bridge
still computes no hash. A refused read becomes the side's state and is never
used as a value.

Each side SHALL have exactly one state, the first column (left to right) whose
condition holds:

| Side | `refused` | `lineage` | `failed` | `unavailable` | `unsupported` | `incomplete` | `contradicted` | `final(b)` | `pending` |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| native | strict read refused other than `InvalidPredecessor`, or outcome `Refused` | strict read or `validate_predecessor` refused `InvalidPredecessor` | never | report absent, or outcome `Inactive`, `Unactivated` or `ActivationUnknown` | never | outcome `Missing`, `Indeterminate`, `Limit`, `Meter`, `CoveredPositions`, `WindowLimit` or `EvictedValuation` | never | outcome `Evaluated`, truth `True`/`False`, settlement `ClosedScope`, `DecisiveWitness` or `DecisiveCounterexample` | every other admitted report (outcome `Evaluated`) |
| TL | mapping re-read refused, execution `Refused`, or source request, `subject_identity` or `correspondence_identity` unequal to the projection's request | never | execution `Failed` | result absent, outcome `NonValue{Unavailable}`, or availability not `Available` | execution `Unsupported` | execution `ResourceIncomplete`, or completeness `Incomplete` | completeness `Contradicted` | outcome `Value{b}` and `is_final()` | every other result (execution `Completed`) |

Each row is total: every native report outcome and every TL execution,
completeness, availability and outcome combination falls in some column.

**Premise equality** `P` holds when the TL source request equals the
projection's request; native `request.execution` `Completed`/`Failed` equals TL
execution `Completed`/`Failed`; the TL decision and surrounding progress and
closure, completeness and availability references equal the projection
request's; the native settlement equals the TL settlement basis (`None` or
`Unavailable` matches `Unavailable`; otherwise like-named variants); and the
set of observation identities `request.positions[s.row.position].observation.identity`
over native support entries `s` equals the set of TL `decision_support()`
identities. **Lineage conflict** `L` holds as defined under Corrections.

| Row | Predicate | Decision / comparison | Cause |
| --- | --- | --- | --- |
| R1 | either side `refused` | `refused` / `not-compared` | `result-refused` |
| R2 | native `request.execution` is `Failed`, or TL side `failed` | `failed` / `not-compared` | `result-failed` |
| R3 | either side `unavailable` | `unavailable` / `not-compared` | `result-unavailable` |
| R4 | TL side `unsupported` | `unsupported` / `not-compared` | `result-unsupported` |
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

The cause list SHALL hold the cause of every R1 to R6 row whose predicate
holds, once each, in row order, followed by `result-pending` when R8 is the
decision; an R7 decision has an empty cause list. Only R7 carries a Boolean.

### Corrections

A native report with a predecessor, or a TL source result whose relation is
not `Original`, is a correction. `L` SHALL hold when exactly one side is a
correction; both are originals and `previous` is supplied; both are
corrections and `previous` is absent or its native side was not admitted; or
`mapped.source().validate_supplied_predecessor(previous.mapped().map(|m|
m.source().bytes()))` returns `missing-predecessor`, `unexpected-predecessor`
or `predecessor-mismatch` (quire-mltl FR-001). For a native correction with a
previous admitted native report, the strict read uses
`CheckedSubject::with_predecessor(previous.native(), native_report_limits)`
and is followed by `Report::validate_predecessor`; `InvalidPredecessor` from
either gives the native side state `lineage`. Equal result identities never
establish a predecessor. A correction never rewrites the previous join.

### Join record

The join record SHALL be the RFC 8785 canonical JSON serialization of one
object with exactly these members, under the same encodings:

| Member | Encoding | Value |
| --- | --- | --- |
| `version` | str | `quire.native-temporal-join/v1` |
| `correspondence` | hex | the projection's correspondence record bytes |
| `native` | hex or null | `Report::to_owner_bytes_bounded` bytes |
| `nativeAdmitted` | bool | whether the native strict read admitted |
| `mapped` | hex or null | `ValidatedMappedResult::bytes()` |
| `tlResult` | hex or null | the mapped result's source `ValidatedTemporalResult::bytes()` |
| `previous` | object or null | the previous join's `native` and `tlResult` members (hex or null); earlier lineage is not retained |
| `decision` | str | `refused`, `failed`, `unavailable`, `unsupported`, `incomplete`, `conflict` or `agreement` |
| `comparison` | str | `not-compared`, `mismatch`, `equal-final` or `equal-pending` |
| `causes` | array of str | the ordered cause list |
| `value` | bool or null | the R7 Boolean |

`read_tl_join` SHALL rebuild the expected join from the expected projection,
results and previous join and require byte equality.

`TlJoinError::code()` SHALL return exactly one of `tl-join-expected-mismatch`
(supplied join bytes differ from the rebuilt join), `tl-join-limit-reached`
(a bridge limit, or a native `PredecessorLimit` or `NativeBudget` denial
retaining the native owner's resource and attempted charge),
`tl-join-encoding` (canonical writer refusal) and `tl-join-native-report`
(another native bounded-writer refusal). These are whole-operation refusals,
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
| `max_bytes` | `temporal.correspondence.max_bytes` | 16,777,216 | the length of the largest single byte value the operation checks. Projection checks the bridge definition record, bundle, clock view, six observation views, formula, proposition map, trace or history, request and correspondence record; join checks the native report bytes, mapped bytes, source result bytes and join record; each strict reader also checks the supplied bytes. Usage reports that largest length. |
| `max_nodes` | `temporal.correspondence.max_nodes` | 65,536 | the node count of the owner-read TL formula |
| `max_positions` | `temporal.correspondence.max_positions` | 1,048,576 | `request.positions.len()` |
| `max_valuations` | `temporal.correspondence.max_valuations` | 16,777,216 | the number of `ValuationEvidence` rows |
| `max_support` | `temporal.correspondence.max_support` | 1,048,576 | when both strict reads admit, `native.support().len()` plus `decision_support().len()`; otherwise 0 |
| `max_causes` | `temporal.correspondence.max_causes` | 64 | the length of the join's cause list |
| `max_work` | `temporal.correspondence.max_work` | 4,294,967,296 | projection: the sum of the source-map entry count, bundle record count, `request.positions.len()`, row count, TL positions times predicate leaves, TL formula node count, proposition count, `formula_graph()` length, selected model count, ledger instance and result counts, deliveries plus their captures, origins plus their captures, eviction count, and retained capture count; join: the `max_support` count |

Every listed item is visited exactly once per operation, so each count is
computable from the inputs. A strict reader's counts equal those of the
operation it reconstructs. Owner-internal work is bounded by the owners'
`OwnerLimits`, `ReportPredecessorLimits` and `NativeJoinBudget` and is not
counted here. An owner refusal at a byte ceiling lowered to `max_bytes` SHALL
be reported as `limit-reached{limit: max_bytes, value: setting + 1, setting}`
retaining the owner error; `value` then states that at least one byte exceeds
the setting.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-350-AC-1 | Given an admitted package and a QSL `Supported` classification, projection takes its target from the classification and returns owner-admitted formula, proposition map, trace or history and request values for every supported row: future-only under complete and open surrounding closure (closed-trace and online-prefix targets), and each bounded past operator of QSpec FR-092 alone under both false-extension profiles with origin-complete history. A non-zero range start or an `authoritative_origin` other than `Some(true)` for a past target returns incomplete `history`. | Test (TC-461) |
| FR-350-AC-2 | Timestamped-event (finite-window), mixed past-and-future and infinite-trace classifications return `tl-projection-unsupported` carrying the complete classification and no formula, request or record. A changed support, retained field, definition selection or surrounding closure, a foreign subject, or a classification from another admission of equal bytes returns `tl-projection-binding`; a clone retaining the admission is accepted. | Test (TC-461) |
| FR-350-AC-3 | Mutating each correspondence-record member's source input independently (source entry, declaration, lock, subject profile, model selection, binding, interval, anchor, capture instance, capture, trigger, payload, clock, each observation view, surrounding, origin, each position's observation reference and order key, owner document, valuation row, and each `native` member including the correspondence reference and trigger scope) changes exactly the corresponding record member, and `read_tl_projection` of the original bytes against the mutated inputs returns `tl-projection-correspondence-mismatch`; a non-`holds` atom returns `tl-projection-formula`. | Test (TC-461) |
| FR-350-AC-4 | Fixed-sample projection with a nanosecond unit retains epoch, period and unit and matches them to the owner anchor by exact equality; a changed period or event-count reinterpretation with equal formula bytes returns native-premise `clock`; a missing sample returns incomplete `position`; a non-nanosecond unit is not converted and returns native-premise `clock-representation` for a future target and `tl-projection-owner` with the TL request owner's refusal for a past target. | Test (TC-461) |
| FR-350-AC-5 | For each of R1 to R8 a genuine owner-produced pair reaches that row with the stated decision, comparison and cause list, including an equal-final pair with `P` false and a final/pending pair (R6), a TL `Failed` result (R2) and a native `InvalidPredecessor` (R6); for each pair of rows R1 to R6 a pair satisfying both returns the first decision and both causes once in row order; only R7 carries a Boolean; every join retains the complete native and TL result values. Each lineage condition of `L` and an equal-identity different-content predecessor yields R6, while a correction whose native and TL direct predecessors equal the previous join's admitted results reaches R7. | Test (TC-461) |
| FR-350-AC-6 | Each binding step failure returns the code and premise FR-350 assigns it; a row failing several valuation checks returns the first cause in the owner's check order; repeated agreeing rows project their value and are all recorded; disagreeing or missing rows return incomplete `valuation{position, operand}`; each owner-reader refusal returns `tl-projection-owner`; non-canonical, reordered, unknown-member and trailing projection or join bytes return their mismatch code; no error leaves a partial value. Joined native and TL values equal the supplied values with unchanged owner usage, and no bridge operation has a call path to `evaluate_v2`, `quire_mltl::report::evaluate` or `quire_mltl::report::read`. | Test, Inspection (TC-461) |
| FR-350-AC-7 | Withdrawn: the earlier fixed byte vector, SHA-256 known answers and derived correspondence reference were removed under the no-hash rule. This id is retained and never reused. | Withdrawn |
| FR-350-AC-8 | For each of the seven limits, `TlLimits::default()` returns the stated default; with the other six settings above their counts, an operation whose expected count `n`, computed by the test from its inputs by the Limits table, admits at setting `n` and, at setting `n - 1`, returns `limit-reached{limit, value: n, setting: n - 1}` with the rejected unit unvisited and no partial value; the operation's reported usage equals `n`; an action crossing two limits at once returns only the first in table order; and an owner refusal at the lowered byte ceiling returns `max_bytes` limit-reached with the owner error. Projection exercises `max_bytes`, `max_nodes`, `max_positions`, `max_valuations` and `max_work`; join exercises `max_bytes`, `max_support`, `max_causes` and `max_work`; each strict reader exercises `max_bytes`. | Test (TC-461) |
| FR-350-AC-9 | Given an authenticated `TraceBinding`, the exact occurrence, the assessed instance and complete `ValuationEvidence`, projection uses those values and maps the opaque `request.anchor` by exact equality with one position's observation identity in the admitted bundle to that position's numeric anchor, including when the opaque identity resembles another numeric position. An absent `request.occurrence` selects the sole occurrence; a wrong binding (`tl-projection-native-clock`), wrong node/role/ordinal, ambiguous occurrence or foreign control (native-premise `occurrence`/`control`), mismatched instance anchor or capture (native-premise `anchor`/`capture`), absent observation identity (incomplete `observation`) and duplicate observation identity (native-premise `observation`) each refuse before any TL artifact. Request per-position valuation maps that disagree with `ValuationEvidence` do not change the projection. | Test (TC-461) |
| FR-350-AC-10 | The projection and join retain the exact supplied package admission; the `models` member equals `selected_model_packages()` in its ascending identity order, and two packages selecting the same model set in different source orders yield an identical `models` member. A correction join retains the complete native and TL direct predecessor bytes in `previous`, and `read_tl_join` with a different complete predecessor of equal result identity refuses. | Test (TC-461) |
| FR-350-AC-11 | The test constructs the expected correspondence record and join record independently from the owner inputs by the member tables and encodings above (no JSON number appears) and requires byte equality with the bridge's output for a future, a past and a fixed-sample projection and for an R7 and a correction join; the request bytes inside the record equal the owner-read request, whose `subject_identity` is the native subject name and `correspondence_identity` the profile name, and the request contains no record bytes. A wrong subject or profile name refuses before a projection is returned. | Test (TC-461) |

## Dependencies

- `agent-ix/quire-integration` owns the QSL-typed bridge, the bridge
  definition, strict projection and join readers, the correspondence and join
  records, and the executable TC-461 evidence. FR-028-AC-3 and TC-041 keep both
  Contract IR packages free of QSL dependencies.
- QSL FR-045 defines the support table; FR-515 supplies the checked subject;
  FR-527 supplies `mapping_support`; FR-516 supplies `EvidenceRequest`;
  FR-043 supplies `ValuationEvidence`; FR-050 supplies the admitted package and
  temporal binding; FR-052 supplies the native `Report`, its strict reader,
  bounded writer and full-predecessor validation.
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
