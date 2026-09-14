---
id: FR-025
title: "Project checked native predicates into TL Boolean propositions"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-012
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-014
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-015
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-016
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/FR-051
    type: depends_on
  - target: ix://agent-ix/quire-observation/FR-004
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-042
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-007
    type: depends_on
  - target: ix://agent-ix/tl-syntax/IF-005
    type: implements
  - target: ix://agent-ix/tl-syntax/VO-007
    type: implements
---
# FR-025: Project checked native predicates into TL Boolean propositions

## Description

When a finite population of checked native Boolean leaves is selected for
temporal lowering, the Contract IR predicate bridge SHALL derive one
deterministic Boolean signal/proposition correspondence and SHALL admit a
valuation only from exact validated owner views, without parsing or evaluating
either language or coercing a non-value.

The bridge contract is
`quire.contract.native-predicate-tl-projection/v1`. Native Quire remains the
sole editable source language. TL catalogs/maps and valuation sets are derived
artifacts, never an authored predicate surface.

## Predicate subsystem and public API

The implementation SHALL be organized as
`predicate::{admission,definition,artifacts,valuation,decision,reader}` over the
shared `contract`, `canonical`, `limits`, `identity` and `diagnostic`
foundations. The public API SHALL expose:

```text
predicate::project(&[ValidatedCheckedPredicate], TargetSelection, Limits)
  -> Result<PredicateProjection, PredicateDecision>
predicate::read_projection(bytes, ExpectedProjection, Limits)
  -> Result<ValidatedPredicateProjection, PredicateDecision>
predicate::value(&ValidatedPredicateProjection, PredicateRef,
  &ValidatedAvailability, Option<&ProtocolMappedResultView>, Limits)
  -> PredicateValuationDecision
```

`ValidatedCheckedPredicate` comes only from the QSL FR-051 strict reader;
`ValidatedAvailability` comes only from the observation FR-004 reader; and
`ProtocolMappedResultView` comes only from the Quire Protocol FR-042 selected
mapping. The bridge accepts no raw owner struct, callback, trait-object
validator, trust/total flag, AST, source string or locally mirrored owner enum.

## Static correspondence

For each checked leaf, `PredicateRef` is lowercase SHA-256 over
`quire-contract-ir`, a zero byte, profile
`quire.contract.native-predicate-ref/v1`, a zero byte, and a canonical tuple of:

- exact QSL handoff contract/schema selection, document identity/digest and
  leaf/parent subject identities;
- native definition, package, source, clause and expression identities/spans;
- binding-requirements, model, declaration and typed-expression identities;
- exact Boolean type and native evaluation/definedness profiles; and
- the bridge profile.

Every applicable mutation changes the reference. Display text does not.

The selected population contains 1 through 10,000 distinct references. The
bridge sorts them by reference bytes, assigns contiguous zero-based
`PropositionId` and `SignalId` values, and names both
`quire-predicate/<PredicateRef>`. Every signal has Boolean domain. It emits one
`tl-syntax.signal-catalog/v1` and one `tl-syntax.proposition-map/v1` document,
then admits both through their public strict readers and verifies equal ordered
proposition populations, IDs, names and same-ordinal signal bindings.

The projection identity covers bridge/native/target contract selections,
ordered correspondences and exact catalog/map identities. Permuting input
preserves it; adding or removing a member changes it. No usable partial artifact
is exposed on failure.

## Runtime valuation

Before any result-dependent decision, the bridge admits the selected
`quire.observation.result-availability/v1` bytes through the owner reader and
matches authority, subject, revision, observation population and required
source result identity. Availability values are exactly `available`,
`not-yet-observed`, `producer-unavailable`, and `contract-unavailable`.

When the result is `available`, the caller SHALL supply the exact canonical
protocol-result bytes and selected mapping. Quire Protocol first strict-reads
the result and derives/strict-reads
`quire.protocol.contract-ir-result-map/v1`. Contract IR consumes only the
constructor-private mapped view. It does not read native truth labels, derive a
second source-result projection, reconstruct observation facts, or normalize
display strings.

The mapped view must bind the selected predicate/native subject, source result,
observation/capture/model/population, the four independent progress/closure
owner assertions, completeness assertion/state/facts, settlement, decision
support and correction predecessor. All mappings and owner artifacts must select
exact immutable contracts/schema digests/revisions before content admission.

Only the owner mapped view's `value:true` or `value:false` produces a `valued`
decision. Every owner `nonValue` preserves its typed execution/truth/settlement/
completeness causes as `incomplete`, `unavailable`, `unsupported`, `failed`,
`refused` or `conflict` by the closed bridge precedence. No non-valued decision
contains a Boolean.

A final value may coexist with incomplete facts outside its exact complete
decision-support set; those facts remain completeness gaps. A missing,
incomplete or contradicted deciding fact removes the Boolean and yields the
corresponding non-value. A correction requires an immutable exact direct
predecessor and greater owner revision; prior bytes never change.

## Decisions, strict reading and resources

`PredicateProjectionDecision` kinds are `admitted`, `unavailable`,
`unsupported`, `refused`, `conflict`. `PredicateValuationDecision` kinds are
`valued`, `incomplete`, `unavailable`, `unsupported`, `failed`, `refused`,
`conflict`. Every non-success has a nonempty stable ordered cause set; unknown
well-formed discriminators retain bounded raw text. Contract-admission failures
contain no fields supposedly authenticated by the unadmitted contract.

Canonical decision readers reject unknown/duplicate/missing/out-of-order
fields, trailing data, noncanonical bytes, invalid variants, mismatched
identities/contracts and forbidden fields. Limits independently bound bytes,
depth, strings, causes, predicates, facts and visited work. Work is charged
before traversal/allocation. Exact limits pass; one-over and deterministic
allocation failure return no partial projection, artifact, mapping or value.

## Forbidden responsibilities

This bridge SHALL NOT define QSL, observation, protocol-result or TL wire
vocabularies; parse native or TL source; evaluate predicates/formulas; infer
availability/completeness/progress/closure; execute plugins/network/processes;
or add another authored TL/FRETish surface.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-025-AC-1 | Every admitted checked predicate maps bijectively and deterministically to one Boolean signal and proposition accepted by both target readers; input permutation preserves IDs/names/bytes/identity while population change does not. | Test (TC-038) |
| FR-025-AC-2 | PredicateRef and every decision bind the complete applicable owner contract, source/subject, package, definition, span, binding, model, expression, observation, capture, population, result and mapping identities; each one-axis mutation prevents reuse. | Test (TC-038) |
| FR-025-AC-3 | Only exact `value:true` or `value:false` from the validated owner mapping produces a valuation; every non-value, non-Boolean, non-final, malformed or wrong-subject input exposes no Boolean and is never coerced. | Test (TC-038) |
| FR-025-AC-4 | Availability and mapped-result non-values retain distinct incomplete/unavailable/unsupported/failed/refused/conflict decisions under deterministic precedence, and contract-admission failure authenticates no downstream fields. | Test (TC-038) |
| FR-025-AC-5 | Missing/incomplete/contradicted facts outside the exact complete decision-premise set preserve a final value plus gaps; the same mutation inside that set removes the value and produces the applicable non-value. | Test (TC-038) |
| FR-025-AC-6 | Exact bounds pass; one-over predicates/facts/bytes/depth/strings/work, allocation failure, duplicate/conflicting identities, invalid correction, non-bijective artifacts and target-reader rejection expose no partial output. | Test (TC-038) |
| FR-025-AC-7 | Equal inputs are structurally equal; corrections require an immutable exact direct predecessor and greater owner revision, reject self/wrong-subject/same-revision edges, and leave all predecessor bytes unchanged. | Test (TC-038) |
| FR-025-AC-8 | Real owner readers admit every positive input/artifact and the Contract IR cross-document join rejects every population/ID/name/domain/binding disagreement without private wire imports, callbacks, parser/evaluator invocation or an authored alternate language. | Test (TC-038) |

## Dependencies

QSL FR-051 owns checked native leaves; Quire Observation FR-004 owns runtime
authority; Quire Protocol FR-042 owns canonical results and the selected result
mapping; tl-syntax owns signal/proposition artifacts. FR-012/014/015/016 supply
the Contract IR identity, type and canonical foundations.

## Status

Implemented for `quire-contract-ir#70` and `tl-syntax#52` against immutable QSL
`4f404454b3d5cfb78dfdc468c76de85c199191e5`, Quire Observation
`9ac80e93f4b68a2c7d5a337f9a448ad10de798fc`, Quire Protocol
`36af8d7bb4753ea89f020fe1e5080cef21879b65`, and tl-syntax
`842d82553f045eb69a7f38745756d968254fc25e` owner revisions. TC-038 traces all
eight acceptance criteria through real owner inputs, strict readers, negative
identity/join cases, deterministic goldens, corrections, and bounded failure
paths.
