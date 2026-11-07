---
id: FR-350
title: "Establish native temporal and TL result correspondence"
type: FR
relationships:
  - target: ix://agent-ix/quire-specification/FR-095
    type: implements
  - target: ix://agent-ix/quire-specification/FR-092
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/FR-045
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/FR-050
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/FR-515
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
---
# FR-350: Establish native temporal and TL result correspondence

## Description

When QSL FR-045 returns a `Supported` classification for a temporal declaration
in a strict `quire.checked-package/v2` package, the temporal bridge in
`agent-ix/quire-integration` (IR-1005) SHALL construct and owner-read the TL
formula and valuation request, then join the native and TL results through one
bridge-owned correspondence record. The bridge SHALL take the target and
outstanding premises from the QSL classification and preserve the meaning
QSpec FR-095 defines. This artifact records the cross-repository bridge
contract; it assigns no QSL-typed API to either Contract IR package.

## Inputs

- An admitted `quire_contract_model::CheckedPackageV2` and the borrowed
  `qsl_package::checked_views::TemporalSubjectView` returned by
  `temporal_subject(&package, &declaration)` (QSL FR-515).
- The `qsl_package::checked_views::Classification` returned by
  `mapping_support(&subject, surrounding)` (QSL FR-527), with its complete
  support disposition, retained subject and sealed authenticated selection.
- The native `qsl_eval::temporal::evidence::{EvidenceRequest, TraceBinding}`
  input supplies positions with observation references, numeric stamps and
  explicit Boolean valuation cells (including instance-bound valuations),
  together with clock and observation binding, captures, opaque anchor, closure
  and history boundary. Projection also takes the exact
  `qsl_package::checked_views::ClauseOccurrence`, the selected valuation
  instance identified by the request's trigger identity, and an independently
  selected admitted observation identity with its numeric owner anchor. The
  native `quire_spec_language::temporal::{Report, evaluate_v2}` surface supplies
  the result for the join. The checked-view classifier replaces the legacy
  classification input; it does not replace the native evaluation contract.
- Exact selected public contracts for TL formula, trace or position history,
  request and mapped result.
- Caller-configured byte, node, position, valuation, support, cause and work
  limits.

## Outputs

A projection decision with owner-admitted TL formula and valuation request,
plus a bridge-owned correspondence record, or a typed unsupported, incomplete
or refused decision with no partial projection. A join decision compares the
QSL `evaluate_v2` report with the TL mapped result while retaining the full
result axes QSpec FR-095 names.

## Behavior

### Boundary and interface

The `agent-ix/quire-integration` bridge SHALL implement the public profile
`quire.native-temporal-correspondence/v1`. It consumes
`qsl_package::checked_views::{temporal_subject, mapping_support, Classification,
TemporalSubjectView}` and the native evidence/report surface above. Its public
operations SHALL be equivalent to:

```text
temporal::project_tl(
  admitted_package: &quire_contract_model::CheckedPackageV2,
  subject: &qsl_package::checked_views::TemporalSubjectView,
  &qsl_package::checked_views::Classification,
  surrounding: qsl_package::checked_views::Closure,
  request: &qsl_eval::temporal::evidence::EvidenceRequest,
  occurrence: &qsl_package::checked_views::ClauseOccurrence,
  valuation_instance: Option<&qsl_eval::temporal::evidence::SemanticTriggerIdentity>,
  selected_observation: (admitted_observation_identity, owner_numeric_anchor),
  TlSelection,
  TlLimits
) -> TlProjectionDecision

temporal::read_tl_projection(bytes, ExpectedTlProjection, TlLimits)
  -> Result<ValidatedTlProjection, TlDecision>

temporal::join_tl(
  &ValidatedTlProjection,
  Option<&quire_spec_language::temporal::Report>,
  Option<&TlMappedResult>,
  Option<&ValidatedTlJoin>,
  TlLimits
) -> TlJoinDecision

temporal::read_tl_join(bytes, ExpectedTlJoin, TlLimits)
  -> Result<ValidatedTlJoin, TlDecision>
```

No operation SHALL accept a parser, evaluator, callback, trust flag or
caller-authored mirror of an owner record. The bridge SHALL invoke neither the
native nor TL evaluator. QSL FR-045 owns classification and the retained native
subject; `agent-ix/quire-integration` owns TL construction, correspondence
validation and the cross-owner join. Neither `quire-contract-model` nor the
`quire-contract-ir` root SHALL depend on a QSL crate to implement this profile
(FR-028-AC-3).

### Admission and support

- Before consuming mapping facts, the bridge SHALL require
  `subject.belongs_to(admitted_package)`. A separately admitted package with
  equal contents and the same name is a different instance and SHALL NOT
  authorize the supplied subject. The bridge SHALL re-run `mapping_support`
  on that subject with the supplied surrounding-execution closure and compare
  the complete classification: support (including premises), retained
  declaration/profile/activation and sealed package/clause/definition selection.
  A changed disposition, retained field, definition authority or identity, or
  closure-dependent target SHALL refuse before any TL artifact is constructed.
  Matching package names alone SHALL NOT authorize the classification.
- After that check, only `Supported` permits projection; its TL target SHALL
  be taken exactly from the checked classification. `Unsupported` SHALL return
  every dimension the classification names and no formula, request or
  correspondence record. Successful input binding discharges none of the
  outstanding projection premises below.
- The bridge SHALL authenticate `request.binding` with
  `TraceBinding::authenticate` against the clause selection supplied by the
  selected `TemporalSubjectView` (its declaration, profile and clock). It SHALL
  select the effective occurrence by the request's `{role, ordinal}` key when
  `request.occurrence` is present, or by the sole member of
  `subject.occurrences()` when it is absent. An absent key with zero or multiple
  occurrences, or a present key without exactly one match, SHALL refuse. The
  supplied `ClauseOccurrence` SHALL equal the effective occurrence in node,
  role and ordinal. The selected valuation instance SHALL agree with the
  request's trigger identity and instance-bound valuation cells. It SHALL
  locate the request's opaque anchor only by exact equality with an
  independently selected admitted observation identity in `request.positions`,
  then use that observation's numeric owner anchor for projection. It SHALL NOT parse the
  opaque identity, infer an alias or substitute a numeric position for the
  observation selection. A missing, duplicate or mismatched selection SHALL
  refuse before constructing a TL artifact.
- The support domain SHALL be exactly QSL FR-045 and QSpec FR-095's table. It
  SHALL include each bounded past operator defined by QSpec FR-092, individually
  under both false-extension profiles when no future operator is reachable.
  It SHALL refuse a mixed past-and-future graph as unsupported. A history
  cutoff that does not cover the required past positions SHALL return
  `incomplete`, cause `missing-history`; it SHALL NOT be treated as an
  execution origin.
- The bridge SHALL discharge every outstanding premise named by the
  classification. A failed premise SHALL return its typed cause without a
  partial artifact: total Boolean predicate projection; preserved source and
  clause identity; model, type and predicate bindings; anchor and immutable
  captures; clock and observation binding; interval; closure and history; and
  result dimensions absent from the selected TL wire.
- Decision-scope closure, surrounding-execution closure, assessment execution,
  input completeness and availability SHALL remain independent assertions.
  Surrounding-execution closure selects between the closed-trace and
  online-prefix targets; it SHALL NOT substitute for decision-scope closure.

### Formula and valuation construction

- Construction SHALL traverse the admitted checked-package/v2 temporal operation graph in
  left-to-right postorder, preserve each occurrence without deduplication,
  assign contiguous operand-before-consumer ids, and preserve each inclusive
  `[a,b]` interval where `0 <= a <= b <= u32::MAX`.
- Each `holds` leaf handle SHALL map to one distinct TL proposition id. Only a
  total Boolean leaf may be projected. Numeric, nullable, unavailable or
  pending values SHALL NOT become propositions.
- For every required position and proposition, the bridge SHALL require one
  explicit Boolean valuation cell bound to the same position, anchor, capture
  and observation revision. False is an explicit cell. A missing cell is
  `incomplete`; a duplicate, swapped, replayed or foreign cell refuses before
  either evaluator input exists.
- Future-only mappings SHALL use the formula and false-extension rules of QSpec
  FR-091. Past-only mappings SHALL use the formula edition and complete-history
  requirements selected by QSpec FR-092 and the TL owners' public contracts.
  A fixed-sample mapping SHALL retain its authenticated epoch, period and unit;
  an absent required sample is `incomplete`, and a changed period or event-count
  reinterpretation is refused even where formula bytes are equal.
- Every generated formula, trace or position history, request and mapped-result artifact SHALL pass its owner's public reader before use. The bridge SHALL retain each exact admitted typed owner value, its public name and version where one exists, and the selected definition references. It SHALL compute no content hash or derived identity.
- Construction SHALL be acyclic. The bridge SHALL first take the native subject name from the admitted declaration and the correspondence profile name `quire.native-temporal-correspondence/v1`. It SHALL supply those names to quire-mltl `RequestInput`, owner-read the generated request, and construct the correspondence record with that exact admitted request value. The request SHALL NOT contain the record or a record-derived reference. A wrong subject/profile name or a request unequal to the value retained in the record SHALL refuse before projection admission.
- The correspondence record SHALL retain the package name, declaration and clause names, selected source `{authority, identity}` and span, selected model package identities from `CheckedPackageV2::selected_model_packages` in its admitted identity order, model/type/predicate bindings, anchor, captures, clock and observation binding, every interval, closure/history premises, the exact owner-admitted TL formula, request, trace/history and valuation values, their public versions and the bridge definition `{authority, identity}`. A change to any preserved axis SHALL change the corresponding exposed typed member. The checked package carries no source model aliases; the bridge SHALL NOT derive or accept a caller-authored alias map. No record reference, digest or hash is produced.

### Join and strict reading

- The bridge SHALL verify the QSL native report and TL mapped result against
  the projection, then compare their normalized structures. It SHALL compare
  owner data, never owner label strings.
- The join SHALL preserve assessment-execution state; decision-scope and
  surrounding-execution progress/closure; truth; settlement basis; exact
  decision-support identities; completeness; and supersession. Only an equal
  final pair may yield `agreement` with a Boolean. Every non-final decision
  SHALL carry no Boolean and a nonempty ordered cause set. A correction SHALL
  retain immutable owner bytes and the exact direct predecessor value.
- Projection and join readers SHALL reject unknown, duplicate, missing,
  out-of-order, noncanonical or trailing fields; name/version or typed-content mismatches; and
  cross-wired axes. A resource limit reached exactly SHALL admit; one unit over
  SHALL return `incomplete`, name the limit and setting, and expose no partial
  artifact. Work SHALL be charged before the work it pays for.

### Join decision table

- `temporal::join_tl` SHALL verify the native report (FR-043 `evaluate_v2`, carrying
  its authenticated binding) and the TL mapped result against the projection,
  and then compare normalized structures, never owner label strings.
- The rows of this table SHALL be evaluated in order and the first row the
  pair matches SHALL be the decision; the cause set SHALL list the cause of
  every row the pair matches, in table order. A final `agreement` row has no
  cause, and its decision carries an empty cause set:
  The bridge SHALL first attempt each owner's reader and the projection's
  subject, request, profile and complete correspondence checks. A refused owner read or
  failed correspondence check SHALL become a typed refused state for the first
  row; it SHALL NOT be used as a value. Each row predicate SHALL then be
  evaluated against the same pair of resulting states, independently of earlier
  matches. The first matching row selects the decision, while every matching
  non-final row contributes its named cause once in table order. If no row
  matches, the join SHALL refuse the pair as `result-mismatch` without a
  Boolean. A later equal-final row SHALL NOT override an earlier failure row.

| Validated mapped state | Join decision | Cause |
| --- | --- | --- |
| either result refused, or a wrong subject | `refused` / `not-compared` | `result-refused` |
| either result failed | `failed` / `not-compared` | `result-failed` |
| a required result or producer unavailable | `unavailable` / `not-compared` | `result-unavailable` |
| either result unsupported | `unsupported` / `not-compared` | `result-unsupported` |
| either result incomplete | `incomplete` / `not-compared` | `result-incomplete` |
| both results values or both pending, with unequal values or premises, contradicted deciding support, or conflicting complete values | `conflict` / `mismatch` | `result-mismatch` |
| both values equal and every applicable premise and support equal | `agreement` / `equal-final`, that Boolean | none |
| both pending under an open decision scope, equal | `agreement` / `equal-pending`, no Boolean | `result-pending` |

- A non-final decision SHALL carry no Boolean and a nonempty ordered cause
  set. Assessment execution, both progress and closure axes, settlement
  basis, decision support, completeness and supersession SHALL be preserved,
  and omitting or mutating any of them SHALL NOT yield `agreement`.
- A corrected pair SHALL require the strict-read prior join whose native and
  TL source-result values equal the two direct predecessors. A mixed
  original and corrected pair, a missing prior join or a predecessor mismatch
  SHALL be `conflict`. A correction never rewrites a prior join.

### Correspondence value

The bridge-owned correspondence record SHALL retain exactly the following typed axes. An encoder SHALL write admitted records in RFC 8785 canonical JSON form; a reader SHALL reject duplicate, missing, unknown, out-of-order or trailing members and noncanonical bytes. The record carries complete values and public names/versions, never a derived seal.

| Axis | Retained value |
| --- | --- |
| `package` | the admitted checked-package/v2 `name` and selected clause structural identity |
| `source` | the selected `{authority, identity}` reference and exact clause span |
| `declaration` | declaration index and declared clause name |
| `models` | each owner-admitted selected model package identity in ascending identity order, as returned by `CheckedPackageV2::selected_model_packages`; no alias member |
| `bindings` | each leaf handle, predicate name, structural predicate node id and TL proposition id |
| `anchor`, `captures`, `clock`, `observation` | the exact admitted activation, capture values, clock parameters and observation value |
| `intervals`, `closure`, `history` | ordered interval pairs and separate surrounding/history facts |
| `definition`, `bridge` | selected profile and bridge definition `{authority, identity}` pairs, profile names and versions |
| `target` | the complete owner-admitted formula, request, trace or position history, valuation and mapped-result values with their public names/versions |

The projection SHALL hold this record and the complete owner-admitted formula and request values. The join SHALL hold its decision, ordered causes, native and TL result values, complete correspondence record, optional exact direct predecessor value, optional Boolean and public join version. A previous join is supplied and checked as an admitted value; no reference or digest substitutes for it. Structural equality compares all members and array order.

### Strict reading and limits

`read_tl_projection` and `read_tl_join` SHALL reject unknown, duplicate,
missing or out-of-order members, noncanonical or trailing bytes, name/version or typed-content mismatches and cross-wired positions, scopes or clocks.

`TlLimits` SHALL hold seven caller-configured limits, each a request setting
with the published default below (QSpec FR-461). Each is charged before the
work it pays for. A limit reached exactly admits; one past it SHALL stop with
`incomplete`, cause `limit-reached{limit, value, setting}`, and no partial
artifact:

Each limit SHALL be checked independently against its own counter, using the
effective caller setting or the published default. `value` is the first
observed count exceeding `setting`; the rejected unit SHALL not be processed.
When one action would cross several limits, the first check in the table below
SHALL select the sole returned limit cause. The counters SHALL reset for each
projection, join or strict-read operation. `max_bytes` counts the canonical
bytes of each record under construction or read, not a sum over distinct
records; `max_nodes`, `max_positions`, `max_valuations` and `max_support` count
distinct visited items of their named kind; `max_causes` counts causes emitted
by one join; `max_work` counts one unit for each node, position, cell or support
entry actually visited. A caller may set a smaller positive limit to exercise
an exact and one-over boundary without constructing a default-sized artifact.

| Limit | Setting | Default | Counts |
| --- | --- | --- | --- |
| `max_bytes` | `temporal.correspondence.max_bytes` | 16,777,216 | bytes read or written per record |
| `max_nodes` | `temporal.correspondence.max_nodes` | 65,536 | formula nodes |
| `max_positions` | `temporal.correspondence.max_positions` | 1,048,576 | trace positions |
| `max_valuations` | `temporal.correspondence.max_valuations` | 16,777,216 | valuation cells |
| `max_support` | `temporal.correspondence.max_support` | 1,048,576 | decision-support entries compared |
| `max_causes` | `temporal.correspondence.max_causes` | 64 | causes on one decision |
| `max_work` | `temporal.correspondence.max_work` | 4,294,967,296 | total work units, one per node, position, cell or support entry visited |


## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-350-AC-1 | Given an admitted checked-package/v2 and QSL `Supported` classification, the projection takes its target from the classification, emits the required formula and valuation request for each supported row, and each generated artifact is admitted by its selected owner reader. Each bounded past operator defined by QSpec FR-092 is projected alone under both false-extension profiles with origin-complete history; a history cutoff yields `incomplete`/`missing-history`. | Test (TC-461) |
| FR-350-AC-2 | Timestamped-event, mixed past-and-future, infinite-trace unbounded and other unsupported classifications return `unsupported` with every named dimension and no formula, request or correspondence record; a finite-window-to-closed-trace equivalence claim is refused. | Test (TC-461) |
| FR-350-AC-3 | Mutating any preserved source/span, package/declaration/profile, model/type/predicate binding, anchor, capture, clock, observation, interval, closure/history, TL selection or bridge-definition axis changes the corresponding exposed typed member and identifies that axis; a non-Boolean leaf never becomes a proposition. | Test (TC-461) |
| FR-350-AC-4 | Fixed-sample projection retains epoch, period and unit, refuses a changed period or event-count reinterpretation with equal formula bytes, and reports a missing required sample as `incomplete`. | Test (TC-461) |
| FR-350-AC-5 | Every join decision row is reached; only an equal-final pair carries a Boolean; every non-final pair preserves its causes and all result axes, and a correction is accepted only with its exact direct predecessor value. | Test (TC-461) |
| FR-350-AC-6 | Missing, duplicate, swapped, replayed or foreign valuation cells; owner-reader rejection; malformed or noncanonical projection/join data; and each one-over resource limit return one deterministic non-admitted/incomplete result, with no partial artifact and no evaluator invocation. Exact limits admit. | Test (TC-461) |
| FR-350-AC-7 | Withdrawn by the IR-1141 no-hash decision under QSpec #517/#518: the fixed 1429-byte vector, SHA-256 known answers and derived correspondence reference are removed. This id is retained and never reused. | Withdrawn |
| FR-350-AC-8 | Each `TlLimits` setting uses its stated default, charges before work, admits exactly at the setting and returns `incomplete` one past it with `limit-reached{limit, value, setting}` and no partial artifact. | Test (TC-461) |
| FR-350-AC-9 | Given an admitted request with an authenticated `TraceBinding`, an exact clause occurrence, a selected trigger instance and explicit instance-bound Boolean cells, TC-461 asserts that projection uses those values and maps an opaque anchor by exact equality with an independently selected admitted observation identity to that observation's numeric owner anchor, even when the opaque identity resembles a different numeric position. `request.occurrence = None` selects the sole occurrence; a wrong binding, wrong node/role/ordinal, absent or ambiguous occurrence, mismatched trigger instance, or absent, duplicate or mismatched observation identity refuses with no partial artifact or evaluator invocation. | Test (TC-461) |
| FR-350-AC-10 | Given selected model roots whose graph order differs from ascending package identity order, TC-461 records exactly the package identities returned by the admitted checked package's `selected_model_packages` read in that order, including the empty sequence for a package with no selected model. A renamed, missing, duplicated or reordered model entry fails strict correspondence reading; the bridge does not infer a source alias or accept a caller-authored alias map. | Test (TC-461; IR owner read TC-233) |

## Dependencies

- `agent-ix/quire-integration` (IR-1005) owns the QSL-typed `project_tl` bridge,
  strict projection and join readers, correspondence value, and executable
  TC-461 evidence. This Contract IR requirement remains the cross-repository
  contract for FR-350-AC-1 through FR-350-AC-6 and FR-350-AC-8 through
  FR-350-AC-9; AC-7 remains
  withdrawn. FR-028-AC-3 and TC-041 keep both Contract IR packages free of
  QSL dependencies.
- QSL FR-045 defines the support table; FR-515 supplies the borrowed checked
  temporal subject and exact instance membership; FR-527 supplies
  `checked_views::mapping_support` and its retained authenticated selection.
- QSL FR-050 supplies the admitted checked-package/v2 and temporal binding.
- QSL FR-043 supplies `evaluate_v2`, trace valuation cells and the native
  result.
- QSpec FR-095 owns the correspondence meaning; FR-092 owns the complete
  bounded-past operator semantics; FR-091 owns future false-extension rules;
  FR-322 owns structural package names and source references.
- Quire Observation FR-004 supplies selected observation views.
- tl-syntax FR-011/FR-012/FR-013/FR-014, tl-mltl FR-018 and quire-mltl
  FR-001/FR-003 supply the public formula, history/trace, request and mapped
  result contracts and readers.
