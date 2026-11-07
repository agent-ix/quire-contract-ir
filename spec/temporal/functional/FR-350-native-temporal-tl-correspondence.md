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
states the component architecture and build order. The integration contract
IT-022 in `agent-ix/quire-integration` specifies the integration scenarios that
exercise this contract and does not restate it.

## Inputs

Projection consumes exactly these owner values. Each is the value its owner
issued; the bridge accepts no copy, mirror, label or caller-authored substitute
for any of them.

| Input | Owner API | Use |
| --- | --- | --- |
| admitted package | `quire_contract_model::CheckedPackageV2` | package name, source map, graph, `selected_model_packages()` and model/type/predicate terms |
| checked subject | `qsl_package::checked_views::TemporalSubjectView`, borrowed from that package by `temporal_subject` (QSL FR-515) | declaration, profile, occurrences, per-occurrence name and clock, captures, predicate leaves, formula graph, activation |
| classification | `qsl_package::checked_views::Classification` from `mapping_support` (QSL FR-527) | support disposition, target, outstanding premises, retained subject and sealed selection |
| surrounding closure | `qsl_package::checked_views::Closure` | selects the closed-trace or online-prefix future target |
| native request | `qsl_eval::temporal::evidence::EvidenceRequest` (QSL FR-516) | `binding` (`TraceBinding`), `occurrence`, `control`, `trigger`, `trigger_evidence`, `trigger_scope`, `anchor`, ordered `positions` (observation reference and stamp), `deliveries`, `origins`, `ledger`, `evicted`, `decision_progress`, `decision_closure`, `surrounding_progress`, `surrounding_closure`, `execution`, `completeness`, `authoritative_origin`, `relation` |
| native valuations | `quire_spec_language::temporal::ValuationEvidence` (QSL FR-043) | the ordered six-bound Boolean relation; it is the sole valuation source |
| exact occurrence | `qsl_package::checked_views::ClauseOccurrence` | node, role and ordinal of the assessed occurrence |
| assessed instance | `quire_spec_language::temporal::ValuationInstance` | the instance's anchor and immutable capture-environment identity |
| native observation | `quire_spec_language::temporal::ObservationRef` | the observation and revision references the native report echoes |
| admitted observation revision | `quire_integration::temporal::AdmittedObservationRevision` (IT-021) over a Quire Observation bundle | observation records keyed by observation identity, with clock and owner anchor |
| clock view | `quire_observation::authority::clock::View` | clock identity, revision and selected range |
| observation views | `quire_mltl::request::ObservationInputs` | decision and surrounding progress/closure, completeness and availability views |
| bridge definition | `ValidatedTlBridgeDefinition` from `read_tl_bridge_definition` (IT-022) | authority, identity, profile and version of `quire.native-temporal-correspondence/v1` |
| owner limits | `tl_mltl::wire::OwnerLimits`, `quire_spec_language::temporal::ReportPredecessorLimits` | finite ceilings passed unchanged to the TL and native owner operations |
| bridge limits | `TlLimits` | the seven bridge counters below |

The join additionally consumes the native `quire_spec_language::temporal::Report`
(QSL FR-052), the TL `quire_mltl::contract_ir::ValidatedMappedResult` (quire-mltl
FR-003) with its source `ValidatedTemporalResult`, an optional previously
admitted join and the caller's `qsl_package::NativeJoinBudget` (FR-358).

The per-position `valuations`, `instance_valuations`, `aggregate_samples` and
`instance_aggregate_samples` maps of `EvidenceRequest.positions` are not a
valuation source on this route: the native owner replaces them with the
`ValuationEvidence` relation before evaluation, and the bridge SHALL NOT read
them. No input of this requirement is the QSL legacy caller-constructed
`quire_spec_language::temporal::Trace`.

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
as `ValidatedMappedResult` values, which only the quire-mltl readers construct,
and re-reads them with the non-evaluating `quire_mltl::contract_ir::read`.
Neither `quire-contract-model` nor the `quire-contract-ir` root SHALL depend on
a QSL crate to implement this profile (FR-028-AC-3).

### Admission and support

- Before consuming any mapping fact, the bridge SHALL require
  `subject.belongs_to(package)` and `classification.belongs_to(package)`. A
  separately admitted package with equal bytes and name is a different
  admission and SHALL NOT authorize the subject or classification; a clone
  retaining the original admission SHALL. It SHALL then re-run
  `mapping_support(subject, surrounding)` and require the complete result to
  equal the supplied classification: support and premises, retained
  declaration/profile/activation, and sealed package/clause/definition
  selection. Any difference SHALL return `tl-projection-binding` before a TL
  artifact exists.
- Only `Supported` permits projection, and its target SHALL be taken exactly
  from the classification. `Unsupported` SHALL return
  `tl-projection-unsupported` carrying the complete classification, and no
  formula, request or correspondence record.
- The support domain SHALL be exactly the QSL FR-045 and QSpec FR-095 table. It
  includes each bounded past operator of QSpec FR-092, individually, under
  both false-extension profiles when no future operator is reachable. A mixed
  past-and-future graph is unsupported. For an origin-complete-history target,
  a clock range whose start is not 0 or a request whose `authoritative_origin`
  is not `Some(true)` SHALL return `tl-projection-incomplete` with missing
  premise `history`; a history cutoff is never treated as an execution origin.
- The bridge SHALL discharge every outstanding premise the classification
  names, returning that premise's typed cause on failure: total Boolean
  predicate projection; source and clause identity; model, type and predicate
  binding; anchor and immutable captures; clock and observation binding;
  interval; closure and history; and result dimensions absent from the
  selected TL wire.
- Decision-scope closure, surrounding-execution closure, assessment execution,
  input completeness and availability SHALL remain independent. Surrounding
  closure selects between the closed-trace and online-prefix targets and SHALL
  NOT substitute for decision-scope closure.

### Native input binding

In this order, before any TL artifact is constructed:

1. **Occurrence.** `subject.arena_of(occurrence)` SHALL succeed. When
   `request.occurrence` is present it SHALL equal `{occurrence.role,
   occurrence.ordinal}`; when absent, `subject.occurrences()` SHALL have
   exactly one member. A present `request.control` SHALL be one of
   `subject.requirements()`. The native subject name is
   `subject.name_of(occurrence)`; its absence refuses.
2. **Binding.** `request.binding.authenticate` against the subject's
   declaration, profile and `clock_of(occurrence)` SHALL succeed; its
   `AuthenticatedBinding` clock is retained.
3. **Anchor and instance.** `request.anchor` SHALL be present and equal
   `instance.anchor`; `instance.capture_instance` SHALL be nonempty.
   `request.trigger_evidence` SHALL be `Admitted` and `request.trigger`
   present. The native owner's `ActivationInput::contradiction` SHALL report
   no contradiction for the occurrence's obligation. A retained ledger
   instance SHALL supply the payload and capture environment, with each
   capture's name in declaration order and its anchor equal to
   `instance.anchor`, and no retained fault; otherwise the single matching
   origin (for `on origin`) or guarded delivery (for `on each`) SHALL supply
   them. A missing, null or stale capture, or an eviction of a selected capture
   or valuation, is `incomplete`; a wrong-type capture, a capture at another
   anchor, or a retained `Refused` fault is refused.
4. **Observation.** `observation.reference` and `observation.revision` SHALL
   equal the admitted bundle's identity and revision. Every
   `request.positions[i].observation.identity` SHALL name exactly one
   observation record in that bundle, with the selected clock identity and
   revision and an owner anchor equal to `positions[i].position` in the
   authenticated clock form (event position, or fixed-sample index with equal
   epoch and period). The opaque `request.anchor` SHALL equal the observation
   identity of exactly one position; the numeric TL anchor is that position's
   stamp minus the clock range start. The bridge SHALL NOT parse an opaque
   identity, infer an alias or substitute a numeric position for it.
5. **Positions.** Stamps SHALL be distinct, lie in the clock range
   `[start, end)`, not exceed the completeness `through` value, and cover the
   range contiguously; `end - start` SHALL equal the number of positions.
   The TL position order is ascending stamp order; the record retains the
   native request index of each TL position.
6. **Progress, closure and completeness.** `request.decision_closure` and
   `request.surrounding_closure` SHALL equal the corresponding QObs closure
   views and the supplied surrounding closure; `decision_progress` and
   `surrounding_progress` SHALL equal the QObs progress watermarks;
   `request.completeness` SHALL be `Complete { through }` and the QObs
   completeness view `Complete`. Absent or incomplete completeness is
   `incomplete`; a contradicted completeness view is refused.
7. **Valuations.** Each `ValuationEvidence` row SHALL have `subject` equal to
   the occurrence, `position` below `request.positions.len()`, `anchor` equal
   to `instance.anchor`, `capture_instance` equal to
   `instance.capture_instance`, `revision` equal to `observation.revision` and
   `operand` among `subject.predicate_leaves()`; the first failing row refuses
   with its `ValuationCause` and row index. Two rows for one
   `(position, operand)` refuse as duplicate, even when their values agree.
   Every TL position and predicate leaf SHALL then have exactly one row; a
   missing row is `incomplete` and never false. False is an explicit row.

### Formula and valuation construction

- Construction SHALL traverse the admitted checked-package/v2 temporal
  operation graph in left-to-right postorder, preserve each occurrence without
  deduplication, assign contiguous operand-before-consumer ids, and preserve
  each inclusive `[a,b]` interval where `0 <= a <= b <= u32::MAX`. A timed
  interval is refused with premise `interval`.
- Each `holds` leaf SHALL map to one distinct TL proposition id. Only a total
  Boolean leaf may be projected; numeric, nullable, unavailable or pending
  values SHALL NOT become propositions. A formula atom that is not a `holds`
  leaf, including an aggregate atom, fails the total-Boolean premise with
  `tl-projection-formula` and no artifact. Because `ValuationEvidence` rows
  carry only Booleans, an absent row is the only representation of a missing
  or undefined valuation and is `incomplete`.
- Future-only mappings SHALL use the formula and false-extension rules of QSpec
  FR-091; past-only mappings SHALL use the formula edition and origin-complete
  history of QSpec FR-092 and the TL owners' public contracts. A fixed-sample
  mapping SHALL retain its authenticated epoch, period and unit and match them
  to the owner anchor only by exact equality; an absent sample is
  `incomplete`, and a changed period or event-count reinterpretation is
  refused even where formula bytes are equal. Where the selected TL or
  observation owner cannot represent an authored clock unit, the bridge SHALL
  return that owner's typed refusal (or premise `clock-representation`) and
  SHALL NOT convert units.
- Every generated formula, proposition map, trace or position history and
  request SHALL pass its owner's public strict reader before use, under the
  supplied `OwnerLimits` with each byte ceiling lowered to `max_bytes`. The
  bridge SHALL retain each exact admitted owner value and compute no content
  hash or derived identity of its own.

### Construction order

Projection SHALL construct its values in this order; each step consumes only
values from earlier steps or the inputs, so no value depends on a later one:

1. Admission, support and native input binding (above).
2. Source-map entry: the unique `package.source_map()` entry whose node, role
   and ordinal equal the occurrence; none refuses with premise `source`.
3. TL formula and proposition map, then the trace (future) or position
   history (past) from the checked valuations; each owner-read.
4. `quire_mltl::request::derive` with `RequestInput { formula,
   proposition_map, input, clock, subject_identity, correspondence_identity,
   anchor, observations }`, where `subject_identity` is the native subject
   name from step 1 and `correspondence_identity` is the constant profile name
   `quire.native-temporal-correspondence/v1`; then
   `quire_mltl::request::read` of the derived bytes against the same input.
5. The correspondence record, which contains the exact request bytes from
   step 4.

The request SHALL NOT contain the correspondence record, its bytes or any value
derived from them. A wrong subject or profile name, or a request unequal to the
value retained in the record, SHALL refuse before a projection is returned.

### Correspondence record

The correspondence record SHALL be the RFC 8785 canonical JSON serialization of
one object with exactly the members below, produced by the bridge's canonical
writer under `max_bytes`. Encoding rules: a **hex** value is the lowercase
hexadecimal string of the exact owner bytes; a **dec** value is a base-10
string without leading zeros (`"0"` for zero) of an unsigned integer; an
**IR** value is the `quire-contract-model` serde JSON value of that type; a
**str** value is the exact owner string; JSON `null` encodes `None`. Array order
is the stated order and is significant.

| Member | Encoding | Value |
| --- | --- | --- |
| `profile` | str | `quire.native-temporal-correspondence/v1` |
| `bridgeDefinition` | hex | `ValidatedTlBridgeDefinition` record bytes |
| `package` | str | `CheckedPackageV2::name()` |
| `lock` | IR | `CheckedPackageV2::lock()` |
| `source` | IR | the step-2 `CheckedSourceMapEntry` |
| `declaration` | object | `node` (IR `CheckedNodeId`), `role` (IR), `ordinal` (dec), `name` (str native subject name) |
| `subjectProfile` | IR | `TemporalSubjectView::profile()` |
| `models` | array of str | `CheckedPackageV2::selected_model_packages()` in owner iteration order; no alias member |
| `bindings` | array of object | per TL proposition in id order: `predicate` (IR `CheckedNodeId`), `term` (IR `CheckedPackageV2::term(predicate)`), `proposition` (dec) |
| `intervals` | array of object | per `TemporalSubjectView::formula_graph()` node in owner order: `node` (IR), `operation` (IR), `lower` (dec or null), `upper` (dec or null) |
| `anchor` | str | `ValuationInstance::anchor` |
| `captureInstance` | str | `ValuationInstance::capture_instance` |
| `captures` | array of hex | retained capture values in declaration capture order |
| `trigger` | hex or null | `EvidenceRequest::trigger` bytes |
| `triggerPayload` | hex | the selected instance payload |
| `clock` | IR | the `AuthenticatedBinding` `ClockConfiguration` |
| `observation` | hex | admitted observation bundle bytes |
| `clockView` | hex | clock view bytes |
| `decisionProgress`, `decisionClosure`, `surroundingProgress`, `surroundingClosure`, `completenessView`, `availabilityView` | hex | the six `ObservationInputs` view bytes |
| `surrounding` | str | `complete` or `open` |
| `authoritativeOrigin` | bool or null | `EvidenceRequest::authoritative_origin` |
| `sourcePositions` | array of dec | native request index of each TL position, in TL order |
| `formula`, `propositionMap`, `request` | hex | the owner-admitted formula, proposition map and request bytes |
| `input` | object | `kind` (`future` or `past`) and `bytes` (hex trace or history bytes) |
| `valuations` | array of object | per `ValuationEvidence` row in admission order: `node`, `role` (IR), `ordinal`, `position` (dec), `anchor`, `captureInstance`, `revision` (str), `operand` (IR), `value` (bool) |
| `native` | object | `decisionProgress`, `surroundingProgress` (dec or null), `decisionClosed`, `surroundingClosed` (bool or null), `execution` (`completed` or `failed`), `completeThrough` (dec or null), `completenessFacts` (array of str), `completenessReference` (object of the `EvidenceRef` members `identity`, `authorityIdentity`, `authorityRevision`, `scope`, `population`, `contract` as str and `schemaDigest`, `digest`, `authorityDigest` as hex, echoed opaque owner values) |

The record carries complete values and public names, never a seal, digest or
reference computed by the bridge. `read_tl_projection` SHALL reconstruct the
expected projection from the independently supplied inputs through the same
owner constructors and readers, then require byte equality with the supplied
bytes; any unknown, duplicate, missing, reordered, non-canonical or trailing
content, or any changed value, SHALL return the typed `Correspondence` error,
whose code is `tl-projection-native-premise`. A supplied byte value longer than
`max_bytes` SHALL refuse before reconstruction.

### Join decision

`join_tl` SHALL write the native report through `Report::to_owner_bytes_bounded`
under `native_report_limits`, strict-read it with `Report::from_owner_bytes`
against `CheckedSubject::new(package, occurrence, valuations, request)` and the
projection's native observation, and re-read the mapped result with
`quire_mltl::contract_ir::read` against its own source result. A refused read
becomes the side's `refused` state and is never used as a value.

Each supplied side SHALL be classified into exactly one state:

| Side | `refused` | `unavailable` | `unsupported` | `incomplete` | `contradicted` | `final(b)` | `pending` |
| --- | --- | --- | --- | --- | --- | --- | --- |
| native | strict read refused (except `InvalidPredecessor`), or outcome `Refused` | report absent, or outcome `Inactive`, `Unactivated` or `ActivationUnknown` | never | outcome `Missing`, `Indeterminate`, `Limit`, `Meter`, `CoveredPositions`, `WindowLimit` or `EvictedValuation` | never | outcome `Evaluated`, truth `True`/`False` and settlement `ClosedScope`, `DecisiveWitness` or `DecisiveCounterexample` | any other `Evaluated` report |
| TL | mapping re-read refused, execution `Refused`, or source request, `subject_identity` or `correspondence_identity` unequal to the projection's request | result absent, outcome `NonValue{Unavailable}`, or availability not `Available` | execution `Unsupported` | execution `ResourceIncomplete`, or completeness `Incomplete` | completeness `Contradicted` | outcome `Value{b}` with `is_final()` | any other completed result |

States are tested in column order and the first applicable column is the side's
state. **Premise equality** `P` holds when the TL source request equals the
projection's request; native `request.execution` `Completed`/`Failed` equals TL
execution `Completed`/`Failed`; the TL decision and surrounding progress and
closure, completeness and availability references equal the projection
request's; the native settlement equals the TL settlement basis
(`None` or `Unavailable` matches `Unavailable`); and the set of observation
identities `request.positions[s.row.position].observation.identity` over
native support `s` equals the TL `decision_support()` set. **Lineage conflict**
`L` holds as defined under Corrections.

The rows SHALL be evaluated independently against the same pair:

| Row | Predicate | Decision / comparison | Cause |
| --- | --- | --- | --- |
| R1 | either side `refused` | `refused` / `not-compared` | `result-refused` |
| R2 | native `request.execution` is `Failed`, or TL execution `Failed` | `failed` / `not-compared` | `result-failed` |
| R3 | either side `unavailable` | `unavailable` / `not-compared` | `result-unavailable` |
| R4 | TL side `unsupported` | `unsupported` / `not-compared` | `result-unsupported` |
| R5 | either side `incomplete` | `incomplete` / `not-compared` | `result-incomplete` |
| R6 | `L`; or TL side `contradicted`; or both sides in `{final, pending}` and neither R7 nor R8's state condition holds | `conflict` / `mismatch` | `result-mismatch` |
| R7 | both `final`, equal Booleans, `P` | `agreement` / `equal-final`, that Boolean | none |
| R8 | both `pending`, native truth `Pending` with settlement `Unsettled`, TL truth `Pending` with basis `Unsettled`, native `decision_closure` and TL decision-scope closure both `Open`, and `P` | `agreement` / `equal-pending`, no Boolean | `result-pending` |

The decision SHALL be the first of R1 to R6 whose predicate holds; when none
holds, R7 if it holds, else R8. These rows are exhaustive: when R1 to R5 do not
hold both sides are in `{final, pending}`, and exactly one of R6, R7 and R8
holds. The cause list SHALL hold the cause of every R1 to R6 row whose
predicate holds, once each, in row order, followed by `result-pending` when R8
is the decision; an R7 decision has an empty cause list. Only R7 carries a
Boolean. Supplying the two results in the opposite argument roles is not
possible; reordering the construction of the pair SHALL NOT change decision or
causes.

### Corrections

A native report with a predecessor or a TL source result whose relation is not
`Original` is a correction. `L` SHALL hold when: exactly one side is a
correction; both are originals and `previous` is supplied; both are corrections
and `previous` is absent or its native side was not admitted; the native strict
read with `CheckedSubject::with_predecessor(previous.native(),
native_report_limits)` or the subsequent `Report::validate_predecessor` returns
`InvalidPredecessor`; or
`mapped.source().validate_supplied_predecessor(previous.mapped().map(|m| m.source().bytes()))`
returns `missing-predecessor`, `unexpected-predecessor` or
`predecessor-mismatch`. Equal result identities never establish a
predecessor. A correction never rewrites the previous join.

### Join record

The join record SHALL be the RFC 8785 canonical JSON serialization of one
object with exactly these members, under the same encoding rules:

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
results and previous join and require byte equality, returning
`tl-join-expected-mismatch` otherwise.

### Errors

`TlProjectionError::code()` SHALL return exactly one of the closed codes
`tl-projection-binding`, `tl-projection-unsupported`,
`tl-projection-incomplete`, `tl-projection-native-premise`,
`tl-projection-valuation`, `tl-projection-native-clock`,
`tl-projection-formula`, `tl-projection-owner` and
`tl-projection-limit-reached`, each carrying its typed owner error, premise,
missing premise or limit. `TlJoinError::code()` SHALL return exactly one of
`tl-join-expected-mismatch`, `tl-join-limit-reached`, `tl-join-encoding` and
`tl-join-native-report`. A native `PredecessorLimit` or `NativeBudget` denial
SHALL be `tl-join-limit-reached` retaining the native owner's resource and
attempted charge; another bounded-writer denial is `tl-join-native-report`.
These are whole-operation refusals, not decision rows, and leave no join.

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

| Limit | Setting | Default | One unit is | Charged by |
| --- | --- | --- | --- | --- |
| `max_bytes` | `temporal.correspondence.max_bytes` | 16,777,216 | one byte of a single byte value produced, admitted or retained (each owner view, generated owner document, native report bytes, mapped and source result bytes, and each record), checked per value; also the byte ceiling passed to owner producers and readers | all operations |
| `max_nodes` | `temporal.correspondence.max_nodes` | 65,536 | one TL formula node constructed | projection |
| `max_positions` | `temporal.correspondence.max_positions` | 1,048,576 | one `request.positions` entry, charged before it is visited | projection |
| `max_valuations` | `temporal.correspondence.max_valuations` | 16,777,216 | one `ValuationEvidence` row, charged before it is inspected | projection |
| `max_support` | `temporal.correspondence.max_support` | 1,048,576 | one native support entry or one TL decision-support identity compared | join |
| `max_causes` | `temporal.correspondence.max_causes` | 64 | one cause appended to a join's cause list | join |
| `max_work` | `temporal.correspondence.max_work` | 4,294,967,296 | one visit of a node, position, valuation row or support entry, plus each source-map entry, bundle record, ledger instance or result row, delivery, origin, capture, eviction, binding leaf, interval node and model identity visited | all operations |

Owner-internal work is bounded by the owners' own `OwnerLimits`,
`ReportPredecessorLimits` and `NativeJoinBudget` and is not counted here. An
owner refusal at a byte ceiling lowered to `max_bytes` SHALL be reported as
`max_bytes` `limit-reached` retaining the owner error.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-350-AC-1 | Given an admitted package and a QSL `Supported` classification, projection takes its target from the classification and returns owner-admitted formula, proposition map, trace or history and request values for every supported row: future-only under complete and open surrounding closure (closed-trace and online-prefix targets), and each bounded past operator of QSpec FR-092 alone under both false-extension profiles with origin-complete history. A non-zero range start or an `authoritative_origin` other than `Some(true)` for a past target returns `tl-projection-incomplete` with missing premise `history`. | Test (TC-461) |
| FR-350-AC-2 | Timestamped-event, mixed past-and-future, infinite-trace unbounded and other unsupported classifications return `tl-projection-unsupported` carrying the complete classification and no formula, request or record; a finite-window-to-closed-trace equivalence claim is refused. A changed support, retained field, definition selection or surrounding closure, a foreign subject, or a classification from another admission of equal bytes returns `tl-projection-binding`; a clone retaining the admission is accepted. | Test (TC-461) |
| FR-350-AC-3 | Mutating each correspondence-record member's source input independently (source entry, declaration, lock, subject profile, model selection, binding, interval, anchor, capture instance, capture, trigger, payload, clock, each observation view, surrounding, origin, position, owner document, valuation row and each `native` member) changes exactly the corresponding record member, and `read_tl_projection` of the original bytes against the mutated inputs refuses; a non-Boolean leaf never becomes a proposition. | Test (TC-461) |
| FR-350-AC-4 | Fixed-sample projection retains epoch, period and unit, matches them to the owner anchor by exact equality, refuses a changed period or event-count reinterpretation with equal formula bytes, reports a missing required sample as `incomplete`, and returns the owner's typed refusal, without unit conversion, for a unit the selected owner cannot represent. | Test (TC-461) |
| FR-350-AC-5 | For each of R1 to R8 a genuine owner-produced pair reaches that row with the stated decision, comparison and cause list; for each pair of non-final rows a pair satisfying both returns the first row's decision and both causes once in row order; only R7 carries a Boolean; every non-R7 decision preserves the complete native and TL result values in the join. Each lineage condition of `L` (one-sided correction, original with previous, correction without previous, native full-predecessor mismatch, TL `missing-predecessor`/`unexpected-predecessor`/`predecessor-mismatch`, and an equal-identity different-content predecessor) yields R6, while a correction whose native and TL direct predecessors equal the previous join's admitted results reaches R7. | Test (TC-461) |
| FR-350-AC-6 | Each valuation row cause (foreign occurrence, unadmitted position, anchor, capture instance, revision, operand), a duplicate row, a missing row, each owner-reader refusal, and non-canonical, reordered, unknown-member and trailing projection or join bytes return their typed error with no partial value. Joined native and TL values equal the supplied values with unchanged owner usage, and no bridge operation has a call path to `evaluate_v2`, `quire_mltl::report::evaluate` or `quire_mltl::report::read`. | Test, Inspection (TC-461) |
| FR-350-AC-7 | Withdrawn: the earlier fixed byte vector, SHA-256 known answers and derived correspondence reference were removed under the no-hash rule. This id is retained and never reused. | Withdrawn |
| FR-350-AC-8 | For each of the seven limits, `TlLimits::default()` returns the stated default; with the other six settings above their observed counts, an operation whose measured usage of that limit is `n` admits at setting `n` and, at setting `n - 1`, returns `limit-reached{limit, value: n, setting: n - 1}` with the rejected unit unvisited and no partial value; and an action crossing two limits at once returns only the first in table order. Projection exercises `max_bytes`, `max_nodes`, `max_positions`, `max_valuations` and `max_work`; join exercises `max_bytes`, `max_support`, `max_causes` and `max_work`; each strict reader exercises `max_bytes`. | Test (TC-461) |
| FR-350-AC-9 | Given an authenticated `TraceBinding`, the exact occurrence, the assessed instance and complete `ValuationEvidence`, projection uses those values and maps the opaque `request.anchor` by exact equality with one position's observation identity in the admitted bundle to that position's numeric anchor, including when the opaque identity resembles another numeric position. An absent `request.occurrence` selects the sole occurrence; a wrong binding, wrong node/role/ordinal, absent or ambiguous occurrence, foreign control, mismatched instance or capture, and an absent, duplicate or foreign observation identity each refuse with their typed error before any TL artifact. Request per-position valuation maps that disagree with `ValuationEvidence` do not change the projection. | Test (TC-461) |
| FR-350-AC-10 | The projection and join retain the exact supplied package admission and `selected_model_packages()` order; a package with the same model identities in a different selection order yields a different `models` member. A correction join retains the complete native and TL direct predecessor bytes in `previous`, and `read_tl_join` with a different complete predecessor of equal result identity refuses. | Test (TC-461) |
| FR-350-AC-11 | The test constructs the expected correspondence record and join record independently from the owner inputs by the member tables and encoding rules above, and requires byte equality with the bridge's output for a future, a past and a fixed-sample projection and for an R7 and a correction join; the request bytes inside the record equal the owner-read request, whose `subject_identity` is the native subject name and `correspondence_identity` the profile name, and the request contains no record bytes. A wrong subject or profile name refuses before a projection is returned. | Test (TC-461) |

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
  quire-mltl FR-003 supplies the mapped result and its non-evaluating reader.
