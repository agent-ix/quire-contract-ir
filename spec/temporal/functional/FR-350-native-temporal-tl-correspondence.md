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
in a strict `quire.compiled-protocol/2` package, the Contract IR temporal
subsystem SHALL construct and owner-read the TL formula and valuation request,
then join the native and TL results through one bridge-owned correspondence
record. The subsystem SHALL take the target and outstanding premises from the
QSL classification and preserve the meaning QSpec FR-095 defines.

## Inputs

- The strict `/2` package and declaration admitted under QSL FR-050.
- The authenticated `Classification` returned by QSL
  `temporal::mapping_support_v2`, including its retained selection.
- The QSL FR-043 `temporal::Trace` used by native `evaluate_v2`, with explicit
  Boolean valuation cells, clock and observation binding, captures, anchor,
  closure and history boundary.
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

The public bridge profile SHALL be
`quire.native-temporal-correspondence/v1`. Its public operations SHALL
be equivalent to:

```text
temporal::project_tl(
  &quire_spec_language::protocol_artifact::v2::AdmittedPackage,
  declaration,
  &quire_spec_language::temporal::Classification,
  &quire_spec_language::temporal::Trace,
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
caller-authored mirror of an owner record. Contract IR SHALL invoke neither the
native nor TL evaluator. QSL FR-045 owns classification and the retained native
subject; this subsystem owns TL construction, correspondence validation and the
cross-owner join.

### Admission and support

- The bridge SHALL accept only a `Supported` classification whose authenticated
  selection names the same package digest, declaration, definition identity and
  revision as the admitted `/2` package. It SHALL take the TL target exactly
  from that classification. An `Unsupported` classification SHALL return
  `unsupported` with every dimension the classification names and no formula,
  request or correspondence record.
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

- Construction SHALL traverse the admitted `/2` temporal operation graph in
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
- Every generated formula, trace or position history, request and mapped-result
  artifact SHALL pass its owner's public reader before it is used. Each
  artifact identity SHALL be computed from its own canonical bytes and SHALL
  NOT substitute for another artifact's identity.
- Construction SHALL be acyclic. The bridge SHALL first fix the native subject
  identity from the admitted declaration and the correspondence *profile*
  identity `quire.native-temporal-correspondence/v1`. It SHALL supply those
  two exact strings as `RequestInput.subject_identity` and
  `RequestInput.correspondence_identity` to quire-mltl FR-004, after owner-reading
  the formula, proposition map, trace or history, clock and observation facts.
  It SHALL then owner-read the derived request, take the SHA-256 of that
  request's canonical bytes for `target.request`, construct the correspondence
  record containing that digest, and only then derive the bridge-owned record
  reference. The request SHALL NOT contain the record reference or record
  digest, and the record reference SHALL NOT replace the request's opaque
  correspondence profile identity. A request carrying a different profile
  identity or a record whose `target.request` disagrees with the owner-read
  request SHALL be refused before projection is admitted.
- The correspondence record SHALL preserve the exact package digest,
  declaration and clause identity, native source digest and span, selected
  language/profile definition, model/type/predicate bindings, anchor, captures,
  clock and observation binding, every interval, closure/history premises, TL
  formula/profile/proposition identities and bridge definition. Its
  bridge-owned derived reference SHALL change with every preserved-key change
  and SHALL NOT reuse a native or TL identity. Source and bridge-definition
  digests SHALL follow QSpec FR-201's `raw-artifact-digest` domain.

### Join and strict reading

- The bridge SHALL verify the QSL native report and TL mapped result against
  the projection, then compare their normalized structures. It SHALL compare
  owner data, never owner label strings.
- The join SHALL preserve assessment-execution state; decision-scope and
  surrounding-execution progress/closure; truth; settlement basis; exact
  decision-support identities; completeness; and supersession. Only an equal
  final pair may yield `agreement` with a Boolean. Every non-final decision
  SHALL carry no Boolean and a nonempty ordered cause set. A correction SHALL
  retain immutable owner bytes and the exact direct predecessor.
- Projection and join readers SHALL reject unknown, duplicate, missing,
  out-of-order, noncanonical or trailing fields; identity mismatches; and
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
  subject, request, profile and correspondence checks. A refused owner read or
  failed correspondence check SHALL become a typed refused state for the first
  row; it SHALL NOT be used as a value. Each row predicate SHALL then be
  evaluated against the same pair of resulting states, independently of earlier
  matches. The first matching row selects the decision, while every matching
  non-final row contributes its named cause once in table order. If no row
  matches, the join SHALL refuse the pair as `result-mismatch` without a
  Boolean. A later equal-final row SHALL NOT override an earlier failure row.

| Validated mapped state | Join decision | Cause |
| --- | --- | --- |
| either result refused, or a stale or wrong subject | `refused` / `not-compared` | `result-refused` |
| either result failed | `failed` / `not-compared` | `result-failed` |
| a required result or producer unavailable | `unavailable` / `not-compared` | `result-unavailable` |
| either result unsupported | `unsupported` / `not-compared` | `result-unsupported` |
| either result incomplete | `incomplete` / `not-compared` | `result-incomplete` |
| both results values or both pending, with unequal values or premises, contradicted deciding support, or an identity collision | `conflict` / `mismatch` | `result-mismatch` |
| both values equal and every applicable premise and support equal | `agreement` / `equal-final`, that Boolean | none |
| both pending under an open decision scope, equal | `agreement` / `equal-pending`, no Boolean | `result-pending` |

- A non-final decision SHALL carry no Boolean and a nonempty ordered cause
  set. Assessment execution, both progress and closure axes, settlement
  basis, decision support, completeness and supersession SHALL be preserved,
  and omitting or mutating any of them SHALL NOT yield `agreement`.
- A corrected pair SHALL require the strict-read prior join whose native and
  TL source-result identities equal the two direct predecessors. A mixed
  original and corrected pair, a missing prior join or a predecessor mismatch
  SHALL be `conflict`. A correction never rewrites a prior join.

### Wire

Every record below SHALL be a JSON object written in RFC 8785 canonical form
by `quire-canonical` (ADR-013 §2): members in RFC 8785 order, integers as
JSON numbers, digests as 64 lowercase hex characters, owner bytes as
unpadded base64url strings, absent values as `null`. A reader SHALL reject
any byte sequence that is not the canonical form of an admitted record.

The **correspondence record** of profile
`quire.native-temporal-correspondence/v1` holds exactly these members:

| Member | Value |
| --- | --- |
| `anchor` | `{binder, kind}`: `kind` `"origin"` for a whole-execution origin activation or `"trigger"` for a trigger activation (the two activation records FR-044 emits), and `binder` the trigger binder's name, `null` for `origin` |
| `bindings` | one `{leaf, predicate, predicate_node, proposition}` per `holds` leaf in ascending leaf handle: the leaf handle, the leaf's checked predicate name, the predicate's checked node id (which fixes its parameter and result types, the type binding), and the TL proposition id |
| `bridge` | `{definition, profile}`: the `raw-artifact-digest` of the bridge definition, and a `profile` object `{profile, selection}` serialized in RFC 8785 form as part of the record, with `selection` the `TlSelection` and its inner `profile` `"quire.native-temporal-correspondence/v1"` |
| `captures` | one `{index, value}` per capture in declaration order, `value` the capture's canonical value |
| `clause` | `{declaration, identity}`: the declaration index and the clause's declared name |
| `clock` | `{name, parameters}`: the emitted clock binding name and the authenticated parameter map |
| `closure` | `{surrounding}`: `"open"` or `"closed"` |
| `definition` | `{identity, revision}` of the selected profile definition |
| `history` | `"execution-origin"` or `"history-cutoff"` for a past target, `null` for a future target |
| `intervals` | every `[a, b]` in postorder |
| `models` | one `{alias, package}` per model binding of the unit in ascending alias: the model alias and its domain package digest |
| `observation` | `{origin, trace}`: the trace's admitted execution-origin identity (`null` when the declaration does not activate on a whole-execution origin) and the observation identity, the SHA-256 of the RFC 8785 encoding of the trace's clock binding, positions and valuation maps |
| `package` | the `/2` package digest |
| `source` | `{digest, span}`: the native source's `raw-artifact-digest` and the clause's `[start, end]` byte span |
| `target` | `{formula, formula_edition, profile, request, trace, trace_contract, valuation}`: the digests of the generated formula, request, trace or history and valuation cells (each the SHA-256 of that artifact's own canonical bytes), the formula edition, the TL target profile and the trace or history contract |

### Canonical record vector for FR-350-AC-7

The next code block contains one complete RFC 8785 record preimage as a single
UTF-8 line. The line has **1429 bytes excluding the Markdown line ending**.
Its 64-zero digests are fixture identity values for the byte-level record and
reference check; owner-admission controls in AC-1 use actual owner-read artifact
bytes and their recomputed digests. The admitted execution-origin identity is `origin-1`, matching the
whole-execution `anchor.kind`, and the capture value is exactly 54 ASCII `v`
characters. Independently compute SHA-256 of the exact line to check its
correspondence reference; no recorded digest is an oracle.

```json
{"anchor":{"binder":null,"kind":"origin"},"bindings":[{"leaf":0,"predicate":"p","predicate_node":0,"proposition":0}],"bridge":{"definition":"0000000000000000000000000000000000000000000000000000000000000000","profile":{"profile":"quire.native-temporal-correspondence/v1","selection":{"profile":"mltl.closed-trace/v1"}}},"captures":[{"index":0,"value":"vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv"}],"clause":{"declaration":0,"identity":"c"},"clock":{"name":"event-position","parameters":{}},"closure":{"surrounding":"closed"},"definition":{"identity":"d","revision":"r"},"history":null,"intervals":[[0,1]],"models":[{"alias":"m","package":"0000000000000000000000000000000000000000000000000000000000000000"}],"observation":{"origin":"origin-1","trace":"0000000000000000000000000000000000000000000000000000000000000000"},"package":"0000000000000000000000000000000000000000000000000000000000000000","source":{"digest":"0000000000000000000000000000000000000000000000000000000000000000","span":[0,1]},"target":{"formula":"0000000000000000000000000000000000000000000000000000000000000000","formula_edition":"formula-v1","profile":"mltl.closed-trace/v1","request":"0000000000000000000000000000000000000000000000000000000000000000","trace":"0000000000000000000000000000000000000000000000000000000000000000","trace_contract":"mltl.closed-trace/v1","valuation":"0000000000000000000000000000000000000000000000000000000000000000"}}
```

Starting from that exact record, change **only** the named member below,
re-encode in RFC 8785 form, and independently compute SHA-256 of the resulting
canonical bytes. Each mutation remains 1429 bytes and has a digest distinct
from the original and the other mutations:

| Mutation |
| --- |
| `intervals` from `[[0,1]]` to `[[0,2]]` |
| `models[0].package` from 64 `0`s to 64 `a`s |
| `bridge.definition` from 64 `0`s to 64 `b`s |

The **correspondence reference** SHALL be `{domain, digest}` with `domain`
`"quire.native-temporal-correspondence/v1"` and `digest` the SHA-256 of the
record's canonical bytes. The **projection** (`quire.native-temporal-projection/v1`)
holds `correspondence` (the record), `formula` and `request` (the owner bytes),
`reference` and `version` `"quire.native-temporal-projection/v1"`. The **join** (`quire.native-temporal-join/v1`)
holds `causes` (in table order), `decision`, `native` and `tl` (the digests of
the two results' bytes), `outcome`, `predecessor` (the prior join's
reference, or `null`), `reference` (the correspondence reference), `value`
(`true`, `false` or `null`) and `version` `"quire.native-temporal-join/v1"`.

### Strict reading and limits

`read_tl_projection` and `read_tl_join` SHALL reject unknown, duplicate,
missing or out-of-order members, noncanonical or trailing bytes, identity or
digest mismatches and cross-wired positions, scopes or clocks.

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
| FR-350-AC-1 | Given a strict `/2` package and QSL `Supported` classification, the projection takes its target from the classification, emits the required formula and valuation request for each supported row, and each generated artifact is admitted by its selected owner reader. Each bounded past operator defined by QSpec FR-092 is projected alone under both false-extension profiles with origin-complete history; a history cutoff yields `incomplete`/`missing-history`. | Test (TC-461) |
| FR-350-AC-2 | Timestamped-event, mixed past-and-future, infinite-trace unbounded and other unsupported classifications return `unsupported` with every named dimension and no formula, request or correspondence record; a finite-window-to-closed-trace equivalence claim is refused. | Test (TC-461) |
| FR-350-AC-3 | Mutating any preserved source/span, package/declaration/profile, model/type/predicate binding, anchor, capture, clock, observation, interval, closure/history, TL selection or bridge-definition axis changes the bridge-owned correspondence reference and identifies that axis; a non-Boolean leaf never becomes a proposition. | Test (TC-461) |
| FR-350-AC-4 | Fixed-sample projection retains epoch, period and unit, refuses a changed period or event-count reinterpretation with equal formula bytes, and reports a missing required sample as `incomplete`. | Test (TC-461) |
| FR-350-AC-5 | Every join decision row is reached; only an equal-final pair carries a Boolean; every non-final pair preserves its causes and all result axes, and a correction is accepted only with its exact direct predecessor. | Test (TC-461) |
| FR-350-AC-6 | Missing, duplicate, swapped, replayed or foreign valuation cells; owner-reader rejection; malformed or noncanonical projection/join data; and each one-over resource limit return one deterministic non-admitted/incomplete result, with no partial artifact and no evaluator invocation. Exact limits admit. | Test (TC-461) |
| FR-350-AC-7 | The canonical correspondence record matches the complete field inventory and 1429-byte vector specified in the Wire section; an independently computed SHA-256 over those exact bytes equals its correspondence reference. Each specified single-member interval, model-binding and bridge-definition mutation remains 1429 bytes, has its own independently derived digest distinct from the original and other mutations, and its correspondence reference equals that derived digest; reordered or whitespace-altered bytes are refused. No fixed expected digest is used. | Test (TC-461) |
| FR-350-AC-8 | Each `TlLimits` setting uses its stated default, charges before work, admits exactly at the setting and returns `incomplete` one past it with `limit-reached{limit, value, setting}` and no partial artifact. | Test (TC-461) |

## Dependencies

- QSL FR-045 supplies `mapping_support_v2` classification and its retained
  authenticated selection.
- QSL FR-050 supplies the strict `/2` package and temporal binding.
- QSL FR-043 supplies `evaluate_v2`, trace valuation cells and the native
  result.
- QSpec FR-095 owns the correspondence meaning; FR-092 owns the complete
  bounded-past operator semantics; FR-091 owns future false-extension rules;
  FR-201 owns the raw-artifact digest domain.
- Quire Observation FR-004 supplies selected observation views.
- tl-syntax FR-011/FR-012/FR-013/FR-014, tl-mltl FR-018 and quire-mltl
  FR-001/FR-003 supply the public formula, history/trace, request and mapped
  result contracts and readers.
