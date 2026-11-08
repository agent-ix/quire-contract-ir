---
id: SR-202611070951309515
title: spec-review/failure-domain review of FR-350 successor
type: SpecReview
analysis: failure-domain
scope: agent-ix/quire-contract-ir; IR-1400; spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md,
  spec/temporal/matrix/TC-461-native-temporal-tl-correspondence.md, spec/assurance/AD-009-native-temporal-bridge.md
review_set: subset
---

## Summary

Checked subject, occurrence, source map and input identity precede classification. Original call failure, absence, incomplete and NonValue remain distinct. Opaque carrier authority is independent of input identity. Native issued lineage and genuine ledger authority replace retired native bytes; TL complete-byte predecessor validation remains mandatory. Seven limits and construction acyclicity remain explicit.

## Verdict

**PASS**. Frozen PR #965 at d52e8d704ec84be855aad81dd1cd08f97b0bf890, base 43ef7640.

## Assurance Context

No AssuranceProfile exists under this repository spec tree; direct subset review applies. No enforced review run was required.

## Evidence and limits

Published QSL checkout 935d93e, FR-516 and FR-526 including AC-30, and qsl-eval public request/carrier APIs were inspected. Integration PR89 projection/native.rs and join.rs, IT-022 and TC-115 were read as context, not reviewed as changed Rust. The supplied focused receipt contains 79 exact runs; all 79 logs exist. Scoped Quire validation is clean; no Cargo, global matrix, make ci or Kani run was performed. This is a spec review, not an independent runtime qualification claim.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Examined scope

```yaml
scope:
- id: FR-350:unit-3-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: 'When QSL FR-527 returns a `Supported` classification for a temporal

    declaration occurrence in an admitted `quire.checked-package/v2` package, the

    temporal bridge in `agent-ix/quire-integration` SHALL construct and owner-read

    the TL formula, proposition map, trace or position history and request from

    the native owner inputs, record one bridge-owned correspondence record, and

    later join the native QSL FR-526 evaluator-issued result with the TL mapped result
    under that

    record. The bridge SHALL take the target and outstanding premises from the QSL

    classification and preserve the meaning QSpec FR-0'
- id: FR-350:unit-5-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: 'Projection consumes exactly these owner values. Each is the value its
    owner

    issued; the bridge accepts no copy, mirror, label or caller-authored substitute

    for any of them.'
- id: FR-350:unit-6-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| admitted package | `quire_contract_model::CheckedPackageV2` | package
    name, lock, source map, graph, `selected_model_packages()` and predicate terms
    |'
- id: FR-350:unit-6-2
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| checked subject | `qsl_package::checked_views::TemporalSubjectView`,
    borrowed from that package by `temporal_subject` (QSL FR-515) | declaration, profile,
    occurrences, per-occurrence name and clock, captures, predicate leaves, formula
    graph, activation |'
- id: FR-350:unit-6-3
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| classification | `qsl_package::checked_views::Classification` from `mapping_support`
    (QSL FR-527) | support disposition, target, outstanding premises, retained subject
    and sealed selection |'
- id: FR-350:unit-6-4
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| surrounding closure | `qsl_package::checked_views::Closure` | selects
    the closed-trace or online-prefix future target |'
- id: FR-350:unit-6-5
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| native request | `qsl_eval::temporal::evidence::EvidenceRequest` (QSL
    FR-516) | each member as stated in the disposition table below |'
- id: FR-350:unit-6-6
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| exact occurrence | `qsl_package::checked_views::ClauseOccurrence` |
    node, role and ordinal of the assessed occurrence |'
- id: FR-350:unit-6-7
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| admitted observation revision | `quire_integration::temporal::AdmittedObservationRevision`
    (IT-021) over a Quire Observation bundle | observation records keyed by observation
    identity, with clock and owner anchor |'
- id: FR-350:unit-6-8
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| clock view | `quire_observation::authority::clock::View` | clock identity,
    clock revision and selected range |'
- id: FR-350:unit-6-9
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| observation views | `quire_mltl::request::ObservationInputs` | decision
    and surrounding progress/closure, completeness and availability views |'
- id: FR-350:unit-6-10
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| bridge definition | `ValidatedTlBridgeDefinition` from `read_tl_bridge_definition`
    (IT-022) | authority `agent-ix/quire-integration`, identity `native-temporal-correspondence/v1`,
    profile and version |'
- id: FR-350:unit-6-11
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| owner limits | `tl_mltl::wire::OwnerLimits` | finite ceilings passed
    to the TL owner operations |'
- id: FR-350:unit-6-12
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| bridge limits | `TlLimits` | the seven bridge counters below |'
- id: FR-350:unit-7-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: 'The join additionally borrows the original native whole call outcome

    `Option<&Result<qsl_eval::temporal::evidence::EvaluatedEvidence, quire_eval::CallFailure>>`

    (QSL FR-516 and FR-526), the TL `quire_mltl::contract_ir::ValidatedMappedResult`

    with its source `ValidatedTemporalResult`, an optional previously admitted join

    and the caller''s `qsl_package::NativeJoinBudget` (FR-358) for owner package

    projection. `None` is genuine absence; an original typed `Err` is not absence

    or a native `NonValue`. Only the evaluator issues the opaque carrier. Its

    `result()` borrows the complete original output im'
- id: FR-350:unit-8-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: 'Every `EvidenceRequest` member has exactly one disposition. **Checked**
    members

    are compared in the binding step named; **recorded** members are written into

    the correspondence record; **excluded** members carry no fact the TL side can

    represent, are not read, and reach the join only through the native result''s

    outcome, which the join compares.'
- id: FR-350:unit-9-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `binding`, `occurrence`, `control` | checked (steps 2, 3) |'
- id: FR-350:unit-9-2
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `cover` | checked (step 3): SHALL be `ClockCover::Exact` |'
- id: FR-350:unit-9-3
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `correspondence` | checked (step 4) and recorded (`native.correspondence`)
    |'
- id: FR-350:unit-9-4
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `trigger`, `trigger_evidence`, `trigger_scope`, `anchor`, `deliveries`,
    `origins`, `ledger`, `evicted` | checked (step 5); `trigger`, `trigger_scope`
    and `anchor` recorded |'
- id: FR-350:unit-9-5
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `positions` (each `observation` reference and `position` stamp) | checked
    (steps 6, 7) and recorded (`positions`) |'
- id: FR-350:unit-9-6
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `positions` `order_key` | recorded (`positions[].orderKey`) |'
- id: FR-350:unit-9-7
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `positions[].valuations`, `positions[].instance_valuations` | checked
    (step 9); selected Boolean cells recorded (`valuations`); all provided entries
    charged, including unselected instance maps |'
- id: FR-350:unit-9-8
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `decision_progress`, `decision_closure`, `surrounding_progress`, `surrounding_closure`,
    `completeness` | checked (step 8) and recorded (`native`) |'
- id: FR-350:unit-9-9
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `authoritative_origin` | checked (history premise) and recorded |'
- id: FR-350:unit-9-10
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `execution` | recorded (`native.execution`) and compared by the join
    |'
- id: FR-350:unit-9-11
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `relation` | compared by the join (Corrections) |'
- id: FR-350:unit-9-12
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `progress_ledger`, `limits` | excluded: native evaluator state and ceilings,
    whose effects appear only as native result outcomes |'
- id: FR-350:unit-11-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '`project_tl` returns either a `ValidatedTlProjection` that holds the exact

    owner-admitted formula, proposition map, trace or history, and request values,

    the native input facts it checked, its correspondence record bytes and its

    usage counters, or one typed `TlProjectionError`. `join_tl` returns either a

    `ValidatedTlJoin` that holds its decision, ordered causes, optional Boolean,

    the original native call outcome and TL result values, the projection, the direct
    predecessor

    inputs, its join record bytes and usage, or one typed `TlJoinError`. No

    partial projection or join is observable on any err'
- id: FR-350:unit-14-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: 'The `agent-ix/quire-integration` bridge SHALL implement the public profile

    `quire.native-temporal-correspondence/v1`. Its public operations SHALL be

    equivalent to:'
- id: FR-350:unit-15-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: "```text\ntemporal::read_tl_bridge_definition(bytes, TlLimits)\n  -> Result<ValidatedTlBridgeDefinition,\
    \ TlBridgeDefinitionError>"
- id: FR-350:unit-16-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: "temporal::project_tl(\n  package: &CheckedPackageV2,\n  subject: &TemporalSubjectView,\n\
    \  classification: &Classification,\n  surrounding: Closure,\n  native: NativeProjectionInput\
    \ { request, occurrence },\n  selection: TlProjectionSelection {\n    bridge_definition,\
    \ observation, clock, observations, owner_limits },\n  limits: TlLimits,\n) ->\
    \ Result<ValidatedTlProjection, TlProjectionError>"
- id: FR-350:unit-17-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: "temporal::read_tl_projection(\n  bytes,\n  ExpectedTlProjection { package,\
    \ subject, classification, surrounding, native, selection },\n  TlLimits,\n) ->\
    \ Result<ValidatedTlProjection, TlProjectionError>"
- id: FR-350:unit-18-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: "temporal::join_tl(\n  projection: &ValidatedTlProjection,\n  native: Option<&Result<qsl_eval::temporal::evidence::EvaluatedEvidence,\
    \ quire_eval::CallFailure>>,\n  mapped: Option<&quire_mltl::contract_ir::ValidatedMappedResult>,\n\
    \  previous: Option<&ValidatedTlJoin>,\n  limits: TlLimits,\n  native_budget:\
    \ &mut NativeJoinBudget,\n) -> Result<ValidatedTlJoin, TlJoinError>"
- id: FR-350:unit-19-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: "temporal::read_tl_join(\n  bytes,\n  ExpectedTlJoin { projection, native,\
    \ mapped, previous },\n  TlLimits,\n  native_budget: &mut NativeJoinBudget,\n\
    ) -> Result<ValidatedTlJoin, TlJoinError>\n```"
- id: FR-350:unit-20-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '`ValidatedTlBridgeDefinition`, `ValidatedTlProjection` and `ValidatedTlJoin`

    SHALL be constructor-private: only the operations above construct them. No

    operation SHALL accept a parser, evaluator, callback, trust flag, alias map or

    caller-authored mirror of an owner record. The bridge SHALL invoke neither the

    native evaluator (`evaluate_temporal_evidence`) nor the TL evaluator (`quire_mltl::report::evaluate`

    or `quire_mltl::report::read`, which re-evaluates). It admits TL results only

    as `ValidatedMappedResult` values, which only the quire-mltl readers construct;

    this is the TL owner''s accepted'
- id: FR-350:unit-22-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '`TlProjectionError::code()` SHALL return exactly one of these codes, with
    the

    stated payload:'
- id: FR-350:unit-23-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `tl-projection-binding` | the mapping-binding refusal (admission or
    reclassification) |'
- id: FR-350:unit-23-2
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `tl-projection-unsupported` | the complete classification |'
- id: FR-350:unit-23-3
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `tl-projection-incomplete` | one missing premise: `history`, `anchor`,
    `instance`, `activation`, `capture`, `complete-input`, `observation`, `position`
    or `valuation{position, operand}` |'
- id: FR-350:unit-23-4
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `tl-projection-native-premise` | one native premise: `occurrence`, `control`,
    `subject-name`, `clock-cover`, `correspondence-reference`, `anchor`, `activation`,
    `capture`, `observation-revision`, `observation`, `position`, `clock`, `clock-representation`,
    `decision-closure`, `surrounding-closure`, `progress`, `completeness`, `source`
    or `interval` |'
- id: FR-350:unit-23-5
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `tl-projection-valuation` | the original request position index and
    unknown native atom index |'
- id: FR-350:unit-23-6
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `tl-projection-native-clock` | the native `BindingMismatch` from `TraceBinding::authenticate`
    |'
- id: FR-350:unit-23-7
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `tl-projection-formula` | the formula constructor refusal, including
    a non-`holds` atom |'
- id: FR-350:unit-23-8
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `tl-projection-owner` | the typed TL owner producer or reader error
    |'
- id: FR-350:unit-23-9
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `tl-projection-encoding` | the canonical writer refusal of the correspondence
    record |'
- id: FR-350:unit-23-10
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `tl-projection-correspondence-mismatch` | none: supplied record bytes
    differ from the reconstructed record |'
- id: FR-350:unit-23-11
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `tl-projection-limit-reached` | `{limit, value, setting}`, and the owner
    error when an owner refused at a lowered byte ceiling |'
- id: FR-350:unit-24-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: 'The binding steps below assign every failure exactly one of these codes
    and

    payloads. Steps run in order and the first failure is returned.'
- id: FR-350:unit-26-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: "1. **Admission.** The bridge SHALL require `subject.belongs_to(package)`\
    \ and\n   `classification.belongs_to(package)`, then re-run\n   `mapping_support(subject,\
    \ surrounding)` and require the complete result to\n   equal the supplied classification\
    \ (support and premises, retained\n   declaration/profile/activation, sealed package/clause/definition\n\
    \   selection). A separately admitted package with equal bytes and name is a\n\
    \   different admission; a clone retaining the admission is the same one. Any\n\
    \   failure returns `tl-projection-binding`. Then only `Supported` permits\n \
    \  projection, with its targ"
- id: FR-350:unit-27-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: 'The support domain SHALL be exactly the QSL FR-045 and QSpec FR-095 table.
    It

    includes each bounded past operator of QSpec FR-092, individually, under both

    false-extension profiles when no future operator is reachable. A mixed

    past-and-future graph, a timestamped-event (finite-window) profile and an

    infinite-trace profile are unsupported. Decision-scope closure,

    surrounding-execution closure, assessment execution, input completeness and

    availability SHALL remain independent; surrounding closure selects between the

    closed-trace and online-prefix targets and SHALL NOT substitute for

    decision-sco'
- id: FR-350:unit-29-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: "2. **Occurrence.** `subject.arena_of(occurrence)` failing, a present\n\
    \   `request.occurrence` unequal to `{occurrence.role, occurrence.ordinal}`, or\n\
    \   an absent one with `subject.occurrences().len() != 1` returns\n   native-premise\
    \ `occurrence`. A present `request.control` not among\n   `subject.requirements()`\
    \ returns native-premise `control`. An absent\n   `subject.name_of(occurrence)`\
    \ returns native-premise `subject-name`; that\n   name is the native subject name.\n\
    3. **Binding and cover.** `request.binding.authenticate` against the subject's\n\
    \   declaration, profile and `clock_of(occurrence)` fa"
- id: FR-350:unit-31-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: "- Construction SHALL traverse the admitted checked-package/v2 temporal\n\
    \  operation graph in left-to-right postorder, preserve each occurrence without\n\
    \  deduplication, and assign contiguous operand-before-consumer ids. Each\n  integer\
    \ interval `[a,b]` with `0 <= a <= b <= u32::MAX` is preserved; a timed\n  interval,\
    \ an integer interval without an upper bound, or a bound above\n  `u32::MAX` returns\
    \ native-premise `interval`.\n- Each `holds` leaf SHALL map to one distinct TL\
    \ proposition id. A formula atom\n  that is not a `holds` leaf, including an aggregate\
    \ atom, returns\n  `tl-projection-formula`. Nu"
- id: FR-350:unit-33-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: 'Projection SHALL construct its values in this order; each step consumes
    only

    values from earlier steps or the inputs, so no value depends on a later one:'
- id: FR-350:unit-34-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: "1. Admission, support and native input binding (steps 1 to 9 above).\n\
    2. The source-map entry.\n3. TL formula and proposition map, then the trace (future)\
    \ or position\n   history (past) from the step-9 cell values; each owner-read.\n\
    4. `quire_mltl::request::derive` with `RequestInput { formula,\n   proposition_map,\
    \ input, clock, subject_identity, correspondence_identity,\n   anchor, observations\
    \ }`, where `subject_identity` is the native subject\n   name and `correspondence_identity`\
    \ is the constant profile name\n   `quire.native-temporal-correspondence/v1`;\
    \ then\n   `quire_mltl::request::read` of the "
- id: FR-350:unit-35-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: 'The request SHALL NOT contain the correspondence record, its bytes or
    any value

    derived from them. The native `request.correspondence` names the bridge

    definition, not the record. A wrong subject or profile name, or a request

    unequal to the value retained in the record, SHALL refuse before a projection

    is returned.'
- id: FR-350:unit-37-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: 'The correspondence record SHALL be the RFC 8785 canonical JSON serialization
    of

    one object with exactly the members below, produced by the bridge''s canonical

    writer under `max_bytes`. Encodings:'
- id: FR-350:unit-38-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: "- **hex**: the lowercase hexadecimal string of the exact owner bytes.\n\
    - **dec**: the base-10 string, without leading zeros (`\"0\"` for zero), of an\n\
    \  integer; a negative integer carries a leading `-`.\n- **str**: the exact owner\
    \ string. **bool**: a JSON Boolean.\n- **IR**: the `quire-contract-model` serde\
    \ JSON value of that type, in which\n  every JSON number is replaced by its dec\
    \ string. An IR value containing a\n  non-integer number returns `tl-projection-encoding`.\n\
    - JSON `null` encodes `None` where the table says so. No JSON number appears in\n\
    \  the record. Array order is the stated order and i"
- id: FR-350:unit-39-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `profile` | str | `quire.native-temporal-correspondence/v1` |'
- id: FR-350:unit-39-2
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `bridgeDefinition` | hex | `ValidatedTlBridgeDefinition` record bytes
    |'
- id: FR-350:unit-39-3
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `package` | str | `CheckedPackageV2::name()` |'
- id: FR-350:unit-39-4
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `lock` | IR | `CheckedPackageV2::lock()` |'
- id: FR-350:unit-39-5
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `source` | IR | the source-map entry |'
- id: FR-350:unit-39-6
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `declaration` | object | `node`: IR `CheckedNodeId`; `role`: IR; `ordinal`:
    dec; `name`: str native subject name |'
- id: FR-350:unit-39-7
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `subjectProfile` | IR | `TemporalSubjectView::profile()` |'
- id: FR-350:unit-39-8
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `models` | array of str | `CheckedPackageV2::selected_model_packages()`
    in the owner''s ascending identity order; no alias member |'
- id: FR-350:unit-39-9
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `bindings` | array of object | per TL proposition in id order: `predicate`:
    IR `CheckedNodeId`; `term`: IR `CheckedPackageV2::term(predicate)`; `proposition`:
    dec |'
- id: FR-350:unit-39-10
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `intervals` | array of object | per `TemporalSubjectView::formula_graph()`
    node in owner order: `node`: IR; `operation`: IR; `lower`: dec or null; `upper`:
    dec or null |'
- id: FR-350:unit-39-11
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `anchor` | str | `EvidenceRequest::anchor` |'
- id: FR-350:unit-39-12
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `captures` | array of hex | the step-5 capture values in declaration
    capture order |'
- id: FR-350:unit-39-13
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `trigger` | hex | `EvidenceRequest::trigger` bytes |'
- id: FR-350:unit-39-14
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `triggerPayload` | hex | the step-5 instance payload |'
- id: FR-350:unit-39-15
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `clock` | IR | the `AuthenticatedBinding` `ClockConfiguration` |'
- id: FR-350:unit-39-16
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `observation` | hex | admitted observation bundle bytes |'
- id: FR-350:unit-39-17
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `clockView` | hex | clock view bytes |'
- id: FR-350:unit-39-18
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `decisionProgress`, `decisionClosure`, `surroundingProgress`, `surroundingClosure`,
    `completenessView`, `availabilityView` | hex | the six `ObservationInputs` view
    bytes |'
- id: FR-350:unit-39-19
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `surrounding` | str | `complete` or `open` |'
- id: FR-350:unit-39-20
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `authoritativeOrigin` | bool or null | `EvidenceRequest::authoritative_origin`
    |'
- id: FR-350:unit-39-21
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `positions` | array of object | per TL position in TL order: `index`:
    dec native request index; `stamp`: dec; `observation`: an `EvidenceRef` object;
    `orderKey`: null or object `authority`: str, `key`: dec |'
- id: FR-350:unit-39-22
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `formula`, `propositionMap`, `request` | hex | the owner-admitted formula,
    proposition map and request bytes |'
- id: FR-350:unit-39-23
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `input` | object | `kind`: `future` or `past`; `bytes`: hex trace or
    history bytes |'
- id: FR-350:unit-39-24
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `valuations` | array of object | per selected Boolean cell in ascending
    `(request position index, CheckedNodeId)` order: `position`: dec; `operand`: IR
    `CheckedNodeId`; `value`: bool |'
- id: FR-350:unit-39-25
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `native` | object | `correspondence`: an `EvidenceRef` object; `triggerScope`:
    `open` or `closed`; `decisionProgress`, `surroundingProgress`: dec or null; `decisionClosed`,
    `surroundingClosed`: bool or null; `execution`: `completed` or `failed`; `completeThrough`:
    dec or null; `completenessFacts`: array of str; `completenessReference`: an `EvidenceRef`
    object |'
- id: FR-350:unit-40-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: 'An `EvidenceRef` object has members `identity`, `authorityIdentity`,

    `authorityRevision`, `scope`, `population` and `contract` (str) and

    `schemaDigest`, `digest` and `authorityDigest` (hex), echoing those opaque

    owner values. The record carries complete values and public names, never a

    seal, digest or reference computed by the bridge. `read_tl_projection` SHALL

    refuse a supplied byte value longer than `max_bytes`, reconstruct the expected

    projection from the independently supplied inputs through the same owner

    constructors and readers, and require byte equality with the supplied bytes;

    any unk'
- id: FR-350:unit-42-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '`join_tl` SHALL borrow the original evaluator-issued carrier and compare
    its

    immutable result subject''s package identity, checked package version, clause,

    occurrence role and ordinal, and source-map entry with the projection. A mismatch

    is native `refused`. It SHALL project the owner `EvidencePackage` under

    `NativeJoinBudget`, select its subject and use `matches_result_identity` to bind

    the original request to the carrier''s result identity; a mismatch is native

    `refused`. This function authenticates inputs only: the private carrier, not

    identity equality, supplies authority for actual truth, s'
- id: FR-350:unit-43-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: 'Each side SHALL have exactly one state. Native subject, request and lineage

    checks precede result classification. Within an admitted result, `incomplete`

    or `indeterminate` precedes `NonValue`, which precedes activation and truth.

    Original admission failures retain their refused/incomplete distinction; an

    original fault is failed. Other call families are whole-operation errors.

    The TL side uses the first column (left to right) whose condition holds:'
- id: FR-350:unit-44-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| native | checked subject or request mismatch; original admission `Refused`;
    result `NonValue::Refused` | predecessor identity or lineage mismatch as below
    | original `CallFailure::Fault` | absent outcome; result `NonValue::{Inactive,Unactivated,ActivationUnknown}`;
    activation `Inactive`, `Unknown` or `Unavailable` | never | original admission
    `Incomplete`; result `incomplete` or `indeterminate`; result `NonValue::Missing`
    | never | active result with truth `True`/`False`, settlement `ClosedScope`, `DecisiveWitness`
    or `DecisiveCounterexample` | every other active result |'
- id: FR-350:unit-44-2
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| TL | mapping re-read refused, execution `Refused`, or source request,
    `subject_identity` or `correspondence_identity` unequal to the projection''s request
    | never | execution `Failed` | result absent, outcome `NonValue{Unavailable}`,
    or availability not `Available` | execution `Unsupported` | execution `ResourceIncomplete`,
    or completeness `Incomplete` | completeness `Contradicted` | outcome `Value{b}`
    and `is_final()` | every other result (execution `Completed`) |'
- id: FR-350:unit-45-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: 'Each row is total: every native result outcome and every TL execution,

    completeness, availability and outcome combination falls in some column.'
- id: FR-350:unit-46-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '**Premise equality** `P` holds when the TL source request equals the

    projection''s request; native `request.execution` `Completed`/`Failed` equals
    TL

    execution `Completed`/`Failed`; the TL decision and surrounding progress and

    closure, completeness and availability references equal the projection

    request''s; the native settlement equals the TL settlement basis (`None` or

    `Unavailable` matches `Unavailable`; otherwise like-named variants); and the

    set of observation identities `s.observation.identity`

    over native support entries `s` equals the set of TL `decision_support()`

    identities. **Lineage '
- id: FR-350:unit-47-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| R1 | either side `refused` | `refused` / `not-compared` | `result-refused`
    |'
- id: FR-350:unit-47-2
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| R2 | native `request.execution` is `Failed`, or either side `failed`
    | `failed` / `not-compared` | `result-failed` |'
- id: FR-350:unit-47-3
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| R3 | either side `unavailable` | `unavailable` / `not-compared` | `result-unavailable`
    |'
- id: FR-350:unit-47-4
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| R4 (defensive) | TL side `unsupported` | `unsupported` / `not-compared`
    | `result-unsupported` |'
- id: FR-350:unit-47-5
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| R5 | either side `incomplete` | `incomplete` / `not-compared` | `result-incomplete`
    |'
- id: FR-350:unit-47-6
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| R6 | `L`; or native side `lineage`; or TL side `contradicted`; or both
    sides in `{final, pending}` and neither R7 nor R8 holds | `conflict` / `mismatch`
    | `result-mismatch` |'
- id: FR-350:unit-47-7
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| R7 | native `final(b)`, TL `final(b)` with the same `b`, `P`, and not
    `L` | `agreement` / `equal-final`, `b` | none |'
- id: FR-350:unit-47-8
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| R8 | both sides `pending`, native truth `Pending` and settlement `Unsettled`,
    TL truth `Pending` and basis `Unsettled`, native `decision_closure` `Open` and
    TL decision-scope closure `Open`, `P`, and not `L` | `agreement` / `equal-pending`,
    no Boolean | `result-pending` |'
- id: FR-350:unit-48-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: 'R7 and R8 do not refer to R6, so R6 is well defined. The decision SHALL
    be the

    first of R1 to R6 that holds; when none holds, R7 if it holds, else R8.

    **Totality**: if a side is `refused`, `failed`, `unavailable`, `unsupported`

    or `incomplete`, or the native request execution is `Failed`, one of R1 to R5

    holds. If a side is `lineage` or `contradicted`, R6 holds. Otherwise both sides

    are in `{final, pending}`: R7 requires both `final` and R8 both `pending`, so

    at most one holds, and R6 holds exactly when neither does (including an

    equal-final or equal-pending pair with `P` false, and a final/pe'
- id: FR-350:unit-49-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: 'R4 is a defensive total-table row. A TL `Unsupported` execution cannot
    arise

    from a bridge-built `Supported` request: quire-mltl returns `Unsupported` only

    for an unsupported profile or past node, and any other request is refused at

    R1. R4 is therefore excluded from FR-350-AC-5''s reachable-row coverage.'
- id: FR-350:unit-50-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: 'The cause list SHALL hold the cause of every R1 to R6 row whose predicate

    holds, once each, in row order, followed by `result-pending` when R8 is the

    decision; an R7 decision has an empty cause list. Only R7 carries a Boolean.'
- id: FR-350:unit-52-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: 'A native issued result whose `relation` is not `Original`, or a TL source

    result whose relation is not `Original`, is a correction. For the native side,

    `Original` requires no previous join. A correction requires a previous join

    whose native side was admitted, its original issued carrier''s `result_identity`

    equal to the current relation''s predecessor, and its lineage plus one (checked

    arithmetic) equal to the current lineage. Failure gives native state `lineage`

    and the typed predecessor mismatch. The carrier preserves the actual output;

    QSL admission already requires changed inputs, the authe'
- id: FR-350:unit-53-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '`L` SHALL hold when exactly one side is a correction; both are originals
    and

    `previous` is supplied; both are corrections and `previous` is absent or its

    native side was not admitted; or

    `mapped.source().validate_supplied_predecessor(previous.mapped().map(|m|

    m.source().bytes()))` refuses (quire-mltl FR-001). TL complete predecessor-byte

    validation remains required. Native lineage is bounded by QSL FR-523''s

    `lineage` counter, with no native predecessor-byte work or storage budget.

    Earlier carriers and ledgers remain unchanged. `L` SHALL be evaluated only

    when a native successful carrier and a '
- id: FR-350:unit-55-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: 'The join record SHALL be the RFC 8785 canonical JSON serialization of
    one

    object with exactly these members, under the same encodings:'
- id: FR-350:unit-56-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `version` | str | `quire.native-temporal-join/v1` |'
- id: FR-350:unit-56-2
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `correspondence` | hex | the projection''s correspondence record bytes
    |'
- id: FR-350:unit-56-3
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `native` | hex, object or null | issued result identity bytes; original
    typed failure object below; or genuine absence |'
- id: FR-350:unit-56-4
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `nativeAdmitted` | bool | whether carrier subject, request and lineage
    checks admitted |'
- id: FR-350:unit-56-5
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `mapped` | hex or null | `ValidatedMappedResult::bytes()` |'
- id: FR-350:unit-56-6
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `tlResult` | hex or null | the mapped result''s source `ValidatedTemporalResult::bytes()`
    |'
- id: FR-350:unit-56-7
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `previous` | object or null | the previous join''s `native` (hex, object
    or null) and `tlResult` (hex or null) members; earlier lineage is not retained
    |'
- id: FR-350:unit-56-8
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `decision` | str | `refused`, `failed`, `unavailable`, `unsupported`,
    `incomplete`, `conflict` or `agreement` |'
- id: FR-350:unit-56-9
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `comparison` | str | `not-compared`, `mismatch`, `equal-final` or `equal-pending`
    |'
- id: FR-350:unit-56-10
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `causes` | array of str | the ordered cause list |'
- id: FR-350:unit-56-11
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `value` | bool or null | the R7 Boolean |'
- id: FR-350:unit-57-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: 'For an original admission failure, `native` is an object with exactly

    `failure: "admission"`, `kind: "refused"` or `"incomplete"`, `code`, `cause`,

    `prefix`, `fields`, `member`, `index` (decimal string or null), and `path`.

    `fields` retains the owner''s name/value object; each path step is

    `{"field": name}`, `{"option": true}` or `{"index": decimal-string}`.

    For an original fault, it has exactly `failure: "fault"`, `stage`, `invariant`

    and `named` (the original owner''s serde representation of named fields). No Debug
    or Display text

    is parsed. Admission failures outside member loci or carrying a'
- id: FR-350:unit-58-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '`read_tl_join` SHALL rebuild the expected join from the expected projection,

    results and previous join and require byte equality.'
- id: FR-350:unit-59-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '`TlJoinError::code()` SHALL return exactly one of `tl-join-expected-mismatch`

    (supplied join bytes differ from the rebuilt join), `tl-join-limit-reached`

    (a bridge limit, or an exhausted native package projection budget

    retaining the native owner''s resource and attempted charge),

    `tl-join-encoding` (canonical writer refusal) and `tl-join-native-report`

    (another typed native package or evidence binding refusal). These are whole-operation
    refusals,

    not decision rows, and leave no join.'
- id: FR-350:unit-61-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '`TlLimits` SHALL hold seven caller settings, each defaulting to the value
    below

    and exposed by `TlLimits::default()`. Each operation starts fresh counters and

    exposes them as its usage. Each unit SHALL be charged before the work it pays

    for; a counter equal to its setting admits, and the charge that would make it

    `setting + 1` SHALL stop the operation with `limit-reached{limit, value:

    setting + 1, setting}` and no partial value, leaving the rejected unit

    unvisited. When one action would cross several limits, the first in table

    order is the sole returned limit. A caller may set any positive val'
- id: FR-350:unit-62-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: 'The expected count `n` of each limit is defined by the operation''s inputs,
    not

    by the bridge''s own accounting:'
- id: FR-350:unit-63-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `max_bytes` | `temporal.correspondence.max_bytes` | 16,777,216 | the
    length of the largest single byte value the operation checks. Projection checks
    the bridge definition record, bundle, clock view, six observation views, formula,
    proposition map, trace or history, request and correspondence record; join checks
    each current/previous native identity or diagnostic byte size, the canonical current
    native outcome encoding, mapped bytes, source result bytes and join record; each
    strict reader also checks the supplied bytes. Usage reports that largest length.
    |'
- id: FR-350:unit-63-2
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `max_nodes` | `temporal.correspondence.max_nodes` | 65,536 | the node
    count of the owner-read TL formula |'
- id: FR-350:unit-63-3
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `max_positions` | `temporal.correspondence.max_positions` | 1,048,576
    | `request.positions.len()` |'
- id: FR-350:unit-63-4
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `max_valuations` | `temporal.correspondence.max_valuations` | 16,777,216
    | the total entries in all request positions'' shared valuation maps and every
    instance valuation map, including unused foreign-trigger maps |'
- id: FR-350:unit-63-5
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `max_support` | `temporal.correspondence.max_support` | 1,048,576 |
    when native carrier binding and TL mapped re-read admit, `native.result().support.len()`
    plus `decision_support().len()`; otherwise 0 |'
- id: FR-350:unit-63-6
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `max_causes` | `temporal.correspondence.max_causes` | 64 | the length
    of the join''s cause list |'
- id: FR-350:unit-63-7
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| `max_work` | `temporal.correspondence.max_work` | 4,294,967,296 | projection:
    the sum of the source-map entry count, bundle record count, `request.positions.len()`,
    all provided valuation-map entries, TL positions times native atom operands, TL
    formula node count, proposition count, `formula_graph()` length, selected model
    count, ledger instance and result counts, deliveries plus their captures, origins
    plus their captures, eviction count, and retained capture count; join: the `max_support`
    count plus current and previous native diagnostic visits (admission: field pairs
    plus path steps plus '
- id: FR-350:unit-64-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: 'For a native admission diagnostic, its checked byte size is the sum of
    member

    name, field names and values, and field-name path steps; indices and option

    steps add no string bytes. For a fault it is the sum of stage, invariant, and

    named field names and values. Each current or previous issued identity is also

    checked at its own byte length. The current native outcome''s canonical encoding

    and final join record are separately checked; these bridge-owned encodings do

    not restore a native report serializer or predecessor-byte budget.'
- id: FR-350:unit-65-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: 'Every listed item is visited exactly once per operation, so each count
    is

    computable from the inputs. A strict reader''s counts equal those of the

    operation it reconstructs. Owner-internal work is bounded by the owners''

    `OwnerLimits` and `NativeJoinBudget` and is not

    counted here. An owner refusal at a byte ceiling lowered to `max_bytes` SHALL

    be reported as `limit-reached{limit: max_bytes, value: setting + 1, setting}`

    retaining the owner error; `value` then states that at least one byte exceeds

    the setting. This mapping is defensive and unreachable when the bridge''s own

    input check precedes o'
- id: FR-350:unit-66-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: 'For `max_bytes`, the measured `n` of a correspondence or request record
    is its

    length computed with the setting itself embedded, because the quire-mltl

    request wire embeds the lowered `max_input_bytes` and `max_output_bytes`. The

    boundary is therefore the fixed point: the smallest setting `s` with

    `record_len(s) <= s` admits and `s - 1` refuses with `limit-reached{limit: max_bytes,

    value: record_len(s - 1), setting: s - 1}`; `value` is the record length measured

    with the setting `s - 1` embedded, which equals `s` only when `s - 1` and `s`
    have

    the same number of digits.'
- id: FR-350-AC-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| FR-350-AC-1 | Given an admitted package and a QSL `Supported` classification,
    projection takes its target from the classification and returns owner-admitted
    formula, proposition map, trace or history and request values for every supported
    row: future-only under complete and open surrounding closure (closed-trace and
    online-prefix targets), and each bounded past operator of QSpec FR-092 alone under
    both false-extension profiles with origin-complete history. A non-zero range start
    or an `authoritative_origin` other than `Some(true)` for a past target returns
    incomplete `history`. | Test |'
- id: FR-350-AC-2
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| FR-350-AC-2 | Timestamped-event (finite-window), mixed past-and-future
    and infinite-trace classifications return `tl-projection-unsupported` carrying
    the complete classification and no formula, request or record. A changed support,
    retained field, definition selection or surrounding closure, a foreign subject,
    or a classification from another admission of equal bytes returns `tl-projection-binding`;
    a clone retaining the admission is accepted. | Test |'
- id: FR-350-AC-3
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| FR-350-AC-3 | Mutating each correspondence-record member''s source input
    independently (source entry, declaration, lock, subject profile, model selection,
    binding, interval, anchor, capture, trigger, payload, clock, each observation
    view, surrounding, origin, each position''s observation reference and order key,
    owner document, selected valuation cell, and each `native` member including the
    correspondence reference and trigger scope) changes exactly the corresponding
    record member, and `read_tl_projection` of the original bytes against the mutated
    inputs returns `tl-projection-correspondence-m'
- id: FR-350-AC-4
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| FR-350-AC-4 | Fixed-sample projection with a nanosecond unit retains
    epoch, period and unit and matches them to the owner anchor by exact equality;
    a changed period or event-count reinterpretation with equal formula bytes returns
    native-premise `clock`; a missing sample returns incomplete `position`; a non-nanosecond
    unit is not converted and returns native-premise `clock-representation` for a
    future target and `tl-projection-owner` with the TL request owner''s refusal for
    a past target. | Test |'
- id: FR-350-AC-5
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| FR-350-AC-5 | For each of R1 to R8 a genuine owner-produced pair reaches
    that row with the stated decision, comparison and cause list, excluding the defensive
    row R4, including an equal-final pair with `P` false and a final/pending pair
    (R6), a TL `Failed` result (R2) and a native predecessor mismatch (R6); for each
    pair of rows R1 to R6 a pair satisfying both returns the first decision and both
    causes once in row order; only R7 carries a Boolean; every join retains the complete
    native and TL result values. Each lineage condition of `L` and a different prior
    issued carrier or TL equal-identi'
- id: FR-350-AC-6
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| FR-350-AC-6 | Each binding failure returns its assigned code and premise;
    an unknown native atom key returns `tl-projection-valuation` with original request
    position and atom index; explicit false projects as false; missing, undefined
    or absent selected values return incomplete `valuation{position, operand}`, and
    a foreign trigger''s map cannot supply them. Each owner-reader refusal retains
    the typed owner error; alternate projection/join bytes return their mismatch code,
    with no partial value. Joins retain the original immutable native carrier or typed
    call failure and unchanged TL owner usa'
- id: FR-350-AC-7
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| FR-350-AC-7 | Withdrawn: the earlier fixed byte vector, SHA-256 known
    answers and derived correspondence reference were removed under the no-hash rule.
    This id is retained and never reused. | Withdrawn |'
- id: FR-350-AC-8
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| FR-350-AC-8 | For each of the seven limits, `TlLimits::default()` returns
    the stated default; with the other six settings above their counts, an operation
    whose expected count `n`, computed by the test from its inputs by the Limits table,
    admits at setting `n` and, at setting `n - 1`, returns `limit-reached{limit, value:
    n, setting: n - 1}` with the rejected unit unvisited and no partial value; the
    operation''s reported usage equals `n`; an action crossing two limits at once
    returns only the first in table order; for `max_bytes` the measured `n` is the
    record length with the setting embedded,'
- id: FR-350-AC-9
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| FR-350-AC-9 | Given the authenticated binding, exact occurrence and
    original request maps, projection selects shared or selected-trigger Boolean values
    through the native atom-to-predicate mapping and maps the opaque request anchor
    by exact equality with one admitted position observation identity, including when
    it resembles another numeric position. An absent request occurrence selects the
    sole occurrence. Wrong binding, node/role/ordinal, ambiguous occurrence, foreign
    control, mismatched delivered anchor/capture, absent observation or duplicate
    observation each returns the assigned refusal'
- id: FR-350-AC-10
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| FR-350-AC-10 | The projection and join retain the exact supplied package
    admission; the `models` member equals `selected_model_packages()` in its ascending
    identity order, and two packages selecting the same model set in different source
    orders yield an identical `models` member. A correction join retains the original
    prior native carrier and TL direct predecessor; `previous` records the native
    issued identity or typed failure and complete TL result bytes. A different prior
    issued carrier or different complete TL predecessor of equal result identity refuses
    strict reading. | Test |'
- id: FR-350-AC-11
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: '| FR-350-AC-11 | The test constructs the expected correspondence record
    and join record independently from the owner inputs by the member tables and encodings
    above (no JSON number appears) and requires byte equality with the bridge''s output
    for a future, a past and a fixed-sample projection and for an R7 and a correction
    join; the request bytes inside the record equal the owner-read request, whose
    `subject_identity` is the native subject name and `correspondence_identity` the
    profile name, and the request contains no record bytes. A wrong subject or profile
    name refuses before a projection is'
- id: FR-350:unit-70-1
  path: spec/temporal/functional/FR-350-native-temporal-tl-correspondence.md
  role: examined
  excerpt: "- `agent-ix/quire-integration` owns the QSL-typed bridge, the bridge\n\
    \  definition, strict projection and join readers, the correspondence and join\n\
    \  records, and the executable TC-461 evidence. FR-028-AC-3 and TC-041 keep both\n\
    \  Contract IR packages free of QSL dependencies.\n- QSL FR-045 defines the support\
    \ table; FR-515 supplies the checked subject;\n  FR-527 supplies `mapping_support`;\
    \ FR-516 supplies `EvidenceRequest`;\n  FR-050 supplies the admitted package and\
    \ temporal binding; FR-526 supplies\n  the immutable `EvaluatedEvidence` carrier,\
    \ checked result subject, support\n  and evaluator-writt"
- id: TC-461:unit-3-1
  path: spec/temporal/matrix/TC-461-native-temporal-tl-correspondence.md
  role: examined
  excerpt: 'Verify the `agent-ix/quire-integration` temporal bridge against the native

    owner inputs of FR-350: an admitted checked package, QSL''s checked subject and

    classification, the original native `EvidenceRequest` and exact occurrence,

    the admitted observation revision and QObs views, and the original whole

    `Result<EvaluatedEvidence, CallFailure>` call outcome; and against

    the TL owners'' public producers and readers, including the quire-mltl mapped

    result and its non-evaluating predecessor validation. The case executes in

    `agent-ix/quire-integration`, which owns the QSL-typed bridge; neither Contrac'
- id: TC-461:unit-5-1
  path: spec/temporal/matrix/TC-461-native-temporal-tl-correspondence.md
  role: examined
  excerpt: "1. **Admission (AC-2).** Emit one source and admit its bytes twice. Require\
    \ a\n   local subject and classification to project; substitute the other\n  \
    \ admission's subject, then its classification, and require\n   `tl-projection-binding`.\
    \ A clone retaining the admission projects. Alter\n   support, a retained field,\
    \ the definition selection and the surrounding\n   closure independently; each\
    \ returns `tl-projection-binding` before a TL\n   artifact.\n2. **Supported rows\
    \ (AC-1, AC-11).** Project a future-only event-position\n   formula under complete\
    \ and open surrounding closure and require the\n   close"
- id: TC-461:unit-7-1
  path: spec/temporal/matrix/TC-461-native-temporal-tl-correspondence.md
  role: examined
  excerpt: "- Each supported row projects with owner-admitted values; each cutoff\
    \ case is\n  `incomplete` with `history`; each unsupported row returns the complete\n\
    \  classification and no artifact.\n- Each binding, valuation, capture, observation\
    \ and fixed-sample defect\n  returns its typed error before any TL artifact, and\
    \ a missing value is never\n  false.\n- The bridge's correspondence and join records\
    \ equal the bytes the test\n  computes from FR-350's rules; each input mutation\
    \ changes exactly its member\n  and fails strict re-reading.\n- Every decision\
    \ row is reached with its decision, causes and Boolean; on"
- id: AD-009:unit-3-1
  path: spec/assurance/AD-009-native-temporal-bridge.md
  role: examined
  excerpt: 'The bridge takes one admitted checked-package temporal occurrence that
    QSL

    classifies as supported, builds the equivalent TL formula, proposition map,

    trace or position history and request from the native owner inputs, records

    their correspondence, and later joins the native QSL evaluator-issued result

    with the TL mapped result under that record. It lives in `agent-ix/quire-integration`,
    the one

    repository allowed to depend on QSL, Quire Observation and the TL crates

    together. Contract IR holds the normative contract (FR-350) and the model types

    the bridge reads; neither Contract IR package de'
- id: AD-009:unit-4-1
  path: spec/assurance/AD-009-native-temporal-bridge.md
  role: examined
  excerpt: 'The bridge evaluates nothing. Native opaque carriers are issued by QSL''s
    evaluator

    and TL results by quire-mltl''s evaluator, both outside the bridge; the bridge

    binds the native carrier to the checked subject and request and admits TL

    results through their owner''s non-evaluating reader. It computes no hash

    or derived identity: it retains owner values and compares them.'
- id: AD-009:unit-6-1
  path: spec/assurance/AD-009-native-temporal-bridge.md
  role: examined
  excerpt: '| Checked package model | `quire-contract-ir` (`quire-contract-model`)
    | `CheckedPackageV2`, source map, graph, `selected_model_packages()`, `CheckedNodeId`,
    `ClockConfiguration`, `NativeJoinBudget` (FR-358) | read-only accessors and serde
    forms |'
- id: AD-009:unit-6-2
  path: spec/assurance/AD-009-native-temporal-bridge.md
  role: examined
  excerpt: '| Checked views and classification | `quire-spec-language` (`qsl-package`)
    | `temporal_subject`, `TemporalSubjectView`, `mapping_support`, `Classification`,
    `ClauseOccurrence`, `Closure`, `belongs_to` | FR-515, FR-527 |'
- id: AD-009:unit-6-3
  path: spec/assurance/AD-009-native-temporal-bridge.md
  role: examined
  excerpt: '| Native evidence request | `quire-spec-language` (`qsl-eval`) | `EvidenceRequest`,
    `TraceBinding::authenticate`, `ActivationInput::contradiction`, ledger | FR-516
    |'
- id: AD-009:unit-6-4
  path: spec/assurance/AD-009-native-temporal-bridge.md
  role: examined
  excerpt: '| Native result authority | `quire-spec-language` (`qsl-eval::temporal::evidence`)
    | `EvidencePackage`, opaque `EvaluatedEvidence::{result, predecessor}`, `matches_result_identity`,
    evaluator-written ledger | FR-516, FR-526 |'
- id: AD-009:unit-6-5
  path: spec/assurance/AD-009-native-temporal-bridge.md
  role: examined
  excerpt: '| Observation authority | `quire-observation` | observation bundle, clock,
    progress, closure, completeness and availability views | FR-004 |'
- id: AD-009:unit-6-6
  path: spec/assurance/AD-009-native-temporal-bridge.md
  role: examined
  excerpt: '| Observation revision admission | `quire-integration` | `AdmittedObservationRevision`
    | IT-021 |'
- id: AD-009:unit-6-7
  path: spec/assurance/AD-009-native-temporal-bridge.md
  role: examined
  excerpt: '| TL syntax | `tl-syntax` | formula document, proposition map, strict
    readers | FR-011 to FR-014 |'
- id: AD-009:unit-6-8
  path: spec/assurance/AD-009-native-temporal-bridge.md
  role: examined
  excerpt: '| TL trace and history | `tl-mltl` | `ValidatedTrace`, `ValidatedPositionHistory`,
    `OwnerLimits` | FR-018 |'
- id: AD-009:unit-6-9
  path: spec/assurance/AD-009-native-temporal-bridge.md
  role: examined
  excerpt: '| TL request, result and mapping | `quire-mltl` | `RequestInput`, `request::{derive,
    read}`, `report::{evaluate, read}`, `ValidatedTemporalResult::{direct_predecessor_bytes,
    validate_supplied_predecessor}`, `contract_ir::{map, read}`, `ValidatedMappedResult`
    | FR-001, FR-003, FR-004 |'
- id: AD-009:unit-6-10
  path: spec/assurance/AD-009-native-temporal-bridge.md
  role: examined
  excerpt: '| Bridge definition | `quire-integration` | `definitions/native-temporal-correspondence-v1.json`,
    `read_tl_bridge_definition` | IT-022 |'
- id: AD-009:unit-6-11
  path: spec/assurance/AD-009-native-temporal-bridge.md
  role: examined
  excerpt: '| Bridge projection | `quire-integration` (`temporal::projection`) | admission
    and support checks, native input binding, TL construction through the owners,
    correspondence record, `project_tl`, `read_tl_projection` | FR-350 |'
- id: AD-009:unit-6-12
  path: spec/assurance/AD-009-native-temporal-bridge.md
  role: examined
  excerpt: '| Bridge join | `quire-integration` (`temporal::join`) | native carrier
    binding and TL strict re-read, side classification, decision rows, corrections,
    join record, `join_tl`, `read_tl_join` | FR-350 |'
- id: AD-009:unit-8-1
  path: spec/assurance/AD-009-native-temporal-bridge.md
  role: examined
  excerpt: '```text

    QSL source --QSL emitter--> checked-package/v2 bytes --IR reader--> CheckedPackageV2

    CheckedPackageV2 --temporal_subject--> TemporalSubjectView --mapping_support-->
    Classification

    caller: EvidenceRequest (shared and selected-trigger instance valuation maps)
    + ClauseOccurrence

    caller: AdmittedObservationRevision + clock::View + ObservationInputs + bridge
    definition'
- id: AD-009:unit-9-1
  path: spec/assurance/AD-009-native-temporal-bridge.md
  role: examined
  excerpt: "project_tl\n  1 admission/support    belongs_to x2, reclassify, Supported\
    \ target\n  2 native binding       occurrence, TraceBinding, anchor/instance/captures,\
    \ observation,\n                         positions, progress/closure/completeness,\
    \ selected valuation cells\n  3 source-map entry\n  4 TL construction      tl-syntax\
    \ formula + proposition map, tl-mltl trace/history\n  5 TL request           quire-mltl\
    \ derive + read (subject name, profile name)\n  6 record               RFC 8785\
    \ correspondence record holding the request bytes\n  -> ValidatedTlProjection"
- id: AD-009:unit-10-1
  path: spec/assurance/AD-009-native-temporal-bridge.md
  role: examined
  excerpt: "outside the bridge:  EvidencePackage::evaluate_temporal_evidence -> Result<EvaluatedEvidence,\
    \ CallFailure>\n                     quire-mltl report::evaluate/read + contract_ir::map/read\
    \ -> ValidatedMappedResult"
- id: AD-009:unit-11-1
  path: spec/assurance/AD-009-native-temporal-bridge.md
  role: examined
  excerpt: "join_tl(projection, original call outcome, ValidatedMappedResult, previous\
    \ join, NativeJoinBudget)\n  native: borrow issued result -> checked subject +\
    \ request identity + prior issued identity/lineage\n  TL:     contract_ir::read\
    \ [+ validate_supplied_predecessor]\n  classify sides -> rows R1..R8 -> join record\n\
    \  -> ValidatedTlJoin -> integration and assurance consumers\n```"
- id: AD-009:unit-12-1
  path: spec/assurance/AD-009-native-temporal-bridge.md
  role: examined
  excerpt: 'Construction is acyclic: the request carries the native subject name and
    the

    constant profile name, and the record carries the request bytes, never the

    reverse.'
- id: AD-009:unit-14-1
  path: spec/assurance/AD-009-native-temporal-bridge.md
  role: examined
  excerpt: "- The bridge belongs to `quire-integration`; Contract IR keeps the contract\
    \ and\n  model types and gains no QSL or TL dependency.\n- The valuation source\
    \ is the original request: shared maps for shared atoms,\n  selected-trigger instance\
    \ maps for instance-bound atoms, mapped through the\n  native atom-to-predicate\
    \ accessor. Missing and undefined cells are not false.\n- TL results enter the\
    \ join only as `ValidatedMappedResult` values, which only\n  quire-mltl's readers\
    \ construct; this is the TL owner's accepted result\n  input. The bridge re-reads\
    \ them with the non-evaluating mapping reader and\n  neve"
- id: AD-009:unit-16-1
  path: spec/assurance/AD-009-native-temporal-bridge.md
  role: examined
  excerpt: 'The bridge projection depends on the checked package model, the QSL checked

    views, native evidence request and result-authority components, the observation

    authority and revision admission, the TL syntax, trace and history readers, the

    quire-mltl request derivation and reader, and the bridge definition. The

    bridge join depends on the projection, the QSL issued carrier, immutable result

    access and non-evaluating input binding, the quire-mltl mapped-result reader,

    `MappingSelection::for_result`, the public direct-predecessor bytes with

    `validate_supplied_predecessor`, and the Contract IR `Native'
- id: AD-009:unit-18-1
  path: spec/assurance/AD-009-native-temporal-bridge.md
  role: examined
  excerpt: "- Selecting a foreign trigger map could let TL and native evaluate different\n\
    \  cells; FR-350 names the selected-trigger scope and TC-461 steps 4 and 5\n \
    \ reject foreign-trigger substitution.\n- A TL result constructed outside quire-mltl\
    \ would bypass owner admission; the\n  bridge accepts only the constructor-private\
    \ owner value.\n- Owner clock units the TL or observation owner cannot represent\
    \ stop the\n  projection with the owner's typed refusal rather than a converted\
    \ value.\n"
```
