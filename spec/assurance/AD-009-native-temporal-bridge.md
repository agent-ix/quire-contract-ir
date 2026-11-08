---
id: AD-009
title: "Native temporal to TL correspondence bridge architecture"
type: ArchitectureDescription
status: accepted
owner: kreneskyp
system: native temporal to TL correspondence bridge (quire.native-temporal-correspondence/v1)
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-350
    type: realizes
  - target: ix://agent-ix/quire-specification/FR-095
    type: references
  - target: ix://agent-ix/quire-contract-ir/AD-001
    type: references
  - target: ix://agent-ix/quire-contract-ir/AD-007
    type: references
---
# Native temporal to TL correspondence bridge architecture

## System Boundary

The bridge takes one admitted checked-package temporal occurrence that QSL
classifies as supported, builds the equivalent TL formula, proposition map,
trace or position history and request from the native owner inputs, records
their correspondence, and later joins the native QSL evaluator-issued result
with the TL mapped result under that record. It lives in `agent-ix/quire-integration`, the one
repository allowed to depend on QSL, Quire Observation and the TL crates
together. Contract IR holds the normative contract (FR-350) and the model types
the bridge reads; neither Contract IR package depends on QSL or TL.

The bridge evaluates nothing. Native opaque carriers are issued by QSL's evaluator
and TL results by quire-mltl's evaluator, both outside the bridge; the bridge
binds the native carrier to the checked subject and request and admits TL
results through their owner's non-evaluating reader. It computes no hash
or derived identity: it retains owner values and compares them.

## Components

| Component | Repository | Owns | Public seam the bridge uses |
| --- | --- | --- | --- |
| Checked package model | `quire-contract-ir` (`quire-contract-model`) | `CheckedPackageV2`, source map, graph, `selected_model_packages()`, `CheckedNodeId`, `ClockConfiguration`, `NativeJoinBudget` (FR-358) | read-only accessors and serde forms |
| Checked views and classification | `quire-spec-language` (`qsl-package`) | `temporal_subject`, `TemporalSubjectView`, `mapping_support`, `Classification`, `ClauseOccurrence`, `Closure`, `belongs_to` | FR-515, FR-527 |
| Native evidence request | `quire-spec-language` (`qsl-eval`) | `EvidenceRequest`, `TraceBinding::authenticate`, `ActivationInput::contradiction`, ledger | FR-516 |
| Native result authority | `quire-spec-language` (`qsl-eval::temporal::evidence`) | `EvidencePackage`, opaque `EvaluatedEvidence::{result, predecessor}`, `matches_result_identity`, evaluator-written ledger | FR-516, FR-526 |
| Observation authority | `quire-observation` | observation bundle, clock, progress, closure, completeness and availability views | FR-004 |
| Observation revision admission | `quire-integration` | `AdmittedObservationRevision` | IT-021 |
| TL syntax | `tl-syntax` | formula document, proposition map, strict readers | FR-011 to FR-014 |
| TL trace and history | `tl-mltl` | `ValidatedTrace`, `ValidatedPositionHistory`, `OwnerLimits` | FR-018 |
| TL request, result and mapping | `quire-mltl` | `RequestInput`, `request::{derive, read}`, `report::{evaluate, read}`, `ValidatedTemporalResult::{direct_predecessor_bytes, validate_supplied_predecessor}`, `contract_ir::{map, read}`, `ValidatedMappedResult` | FR-001, FR-003, FR-004 |
| Bridge definition | `quire-integration` | `definitions/native-temporal-correspondence-v1.json`, `read_tl_bridge_definition` | IT-022 |
| Bridge projection | `quire-integration` (`temporal::projection`) | admission and support checks, native input binding, TL construction through the owners, correspondence record, `project_tl`, `read_tl_projection` | FR-350 |
| Bridge join | `quire-integration` (`temporal::join`) | native carrier binding and TL strict re-read, side classification, decision rows, corrections, join record, `join_tl`, `read_tl_join` | FR-350 |

## Views

```text
QSL source --QSL emitter--> checked-package/v2 bytes --IR reader--> CheckedPackageV2
CheckedPackageV2 --temporal_subject--> TemporalSubjectView --mapping_support--> Classification
caller: EvidenceRequest (shared and selected-trigger instance valuation maps) + ClauseOccurrence
caller: AdmittedObservationRevision + clock::View + ObservationInputs + bridge definition

project_tl
  1 admission/support    belongs_to x2, reclassify, Supported target
  2 native binding       occurrence, TraceBinding, anchor/instance/captures, observation,
                         positions, progress/closure/completeness, selected valuation cells
  3 source-map entry
  4 TL construction      tl-syntax formula + proposition map, tl-mltl trace/history
  5 TL request           quire-mltl derive + read (subject name, profile name)
  6 record               RFC 8785 correspondence record holding the request bytes
  -> ValidatedTlProjection

outside the bridge:  EvidencePackage::evaluate_temporal_evidence -> Result<EvaluatedEvidence, CallFailure>
                     quire-mltl report::evaluate/read + contract_ir::map/read -> ValidatedMappedResult

join_tl(projection, original call outcome, ValidatedMappedResult, previous join, NativeJoinBudget)
  native: borrow issued result -> checked subject + request identity + prior issued identity/lineage
  TL:     contract_ir::read [+ validate_supplied_predecessor]
  classify sides -> rows R1..R8 -> join record
  -> ValidatedTlJoin -> integration and assurance consumers
```

Construction is acyclic: the request carries the native subject name and the
constant profile name, and the record carries the request bytes, never the
reverse.

## Decisions

- The bridge belongs to `quire-integration`; Contract IR keeps the contract and
  model types and gains no QSL or TL dependency.
- The valuation source is the original request: shared maps for shared atoms,
  selected-trigger instance maps for instance-bound atoms, mapped through the
  native atom-to-predicate accessor. Missing and undefined cells are not false.
- TL results enter the join only as `ValidatedMappedResult` values, which only
  quire-mltl's readers construct; this is the TL owner's accepted result
  input. The bridge re-reads them with the non-evaluating mapping reader and
  never reads TL result bytes itself.
- Native output authority is the original immutable evaluator-issued carrier.
  Input identity matching only binds it to the request; it does not authenticate
  copied truth, support, settlement or ledgers. Native corrections retain the
  evaluator-written ledger and check prior issued identity and lineage. TL
  corrections still require complete bytes through `validate_supplied_predecessor`.
- Correspondence and join records are RFC 8785 canonical JSON with complete
  TL/observation owner bytes in hexadecimal, native issued identity or typed
  original call failure, and every integer as a decimal string; strict readers rebuild the expected record from
  independent inputs and compare bytes.
- Bridge limits are seven caller counters charged before work; owner work stays
  under the owners' limits, with byte ceilings lowered to `max_bytes`.

## Component Dependencies

The bridge projection depends on the checked package model, the QSL checked
views, native evidence request and result-authority components, the observation
authority and revision admission, the TL syntax, trace and history readers, the
quire-mltl request derivation and reader, and the bridge definition. The
bridge join depends on the projection, the QSL issued carrier, immutable result
access and non-evaluating input binding, the quire-mltl mapped-result reader,
`MappingSelection::for_result`, the public direct-predecessor bytes with
`validate_supplied_predecessor`, and the Contract IR `NativeJoinBudget`. The
quire-mltl request input uses the same tl-syntax, tl-mltl and Quire Observation
types as the bridge. No Contract IR component depends on the bridge.

## Risks

- Selecting a foreign trigger map could let TL and native evaluate different
  cells; FR-350 names the selected-trigger scope and TC-461 steps 4 and 5
  reject foreign-trigger substitution.
- A TL result constructed outside quire-mltl would bypass owner admission; the
  bridge accepts only the constructor-private owner value.
- Owner clock units the TL or observation owner cannot represent stop the
  projection with the owner's typed refusal rather than a converted value.
