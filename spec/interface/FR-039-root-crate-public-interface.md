---
id: FR-039
title: "Expose the quire-contract-ir root crate's bridge, correspondence and bounded-Kani interface"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-019
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-028
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/AD-001
    type: references
  - target: ix://agent-ix/quire-spec-language/ADR-011
    type: references
  - target: ix://agent-ix/quire-contract-codegen/ADR-002
    type: references
---
# FR-039: Expose the quire-contract-ir root crate's bridge, correspondence and bounded-Kani interface

## Contract

```yaml
name: ContractIrRootApi
version: quire-contract-ir-v0.1
ownership: quire-contract-ir
modules:
  - bridge          # shared owner-bridge errors, digests, limits, selections
  - predicate       # FR-025 native predicate to TL projection
  - temporal        # FR-026 native temporal to TL correspondence
  - ecosystem_model # FR-027 bounded non-authoritative ecosystem model
  - kani            # FR-029..FR-031 kani-bounded/1 profile boundary
re_exports: [] # model items are imported from quire_contract_model (FR-019)
invariants:
  - no public item is a quire_contract_model item under a second path
  - no public item is a QSL runtime type, and nothing calls quire_spec_language::runtime
  - replay, witness, counterexample-envelope, terminal-record and obligation-identity types are qsl_replay's
  - every conversion from untrusted input is fallible and returns a typed error
compatibility:
  supported-rust-minimum: Rust 1.98.1
  licensing: AGPL-3.0-or-later
  publication: disabled pending a later human release decision
```

## Description

The `quire-contract-ir` root crate shall expose exactly the public items
listed below and shall re-export no `quire_contract_model` item: the model's
public items have one import path, `quire_contract_model`, which
[FR-019](./FR-019-rust-library-interface.md) governs. It shall expose no
counterexample packet, witness, replay source, replay agreement, terminal
record or obligation identity of its own: those are QSL `qsl-replay` types,
and where the root crate names one it names QSL's.

## Inputs

Owner-checked views from QSL, Quire Observation, Quire Protocol, tl-syntax and
tl-mltl (for `predicate` and `temporal`); an exact externally selected
campaign manifest (for `ecosystem_model`); a checked clause, a
`kani-bounded/1` profile selection and a finite input (for `kani`); and
caller-selected `BridgeLimits`.

## Outputs

Validated projections, joins and model documents; typed `BridgeError` and
decision values; the `kani-bounded/1` profile selection, validated finite
inputs, dispatch routes, provenance and typed `KaniOutcome`s; and, for each
outcome, either its QSL `TerminalValue` or, for a non-vacuous `Inconclusive`
while the pinned `qsl_replay::TerminalValue` has no inconclusive arm, a typed
absence (FR-031).

## Behavior

### Public items

| Module | Public items | Owning requirement |
| --- | --- | --- |
| `bridge` | `BridgeError`, `BridgeErrorCode`, `BridgeDigest`, `BridgeDigestParseError`, `BridgeLimits`, `ContractSelection` | FR-025, FR-026, STD-001 |
| `predicate` | functions `project`, `value`, `read_projection`, `read_valuation`; types `TargetSelection`, `ExpectedProjection`, `ExpectedValuation`, `PredicateDefinition`, `PredicateRef`, `PredicateCorrespondence`, `PredicateProjection`, `ValidatedPredicateProjection`, `CompletenessGap`, `PredicateCause`, `PredicateCauseCode`, `PredicateCauseDimension`, `PredicateDecision`, `PredicateProjectionDecision`, `PredicateProjectionKind`, `PredicateValuationDecision`, `PredicateValuationKind`; constants `PROFILE`, `PREDICATE_REF_PROFILE`, `PROJECTION_DECISION_PROFILE`, `VALUATION_DECISION_PROFILE`, `PROJECTION_REF_PROFILE`, `SIGNAL_ARTIFACT_PROFILE`, `MAP_ARTIFACT_PROFILE` | FR-025 |
| `temporal` | functions `project`, `join`, `read_projection`, `read_join`; types `TargetSelection`, `TargetContract`, `ObservationViews`, `PositionValuations`, `TemporalProjection`, `TemporalValuationRow`, `ValidatedTemporalProjection`, `ValidatedTemporalJoin`, `ExpectedTemporalProjection`, `ExpectedTemporalJoin`, `JoinComparison`, `TemporalCause`, `TemporalCauseCode`, `TemporalCauseDimension`, `TemporalDecision`, `TemporalJoinDecision`, `TemporalJoinKind`, `TemporalJoinRelation`, `TemporalProjectionDecision`, `TemporalProjectionKind`; constants `PROFILE`, `PROJECTION_DECISION_PROFILE`, `JOIN_DECISION_PROFILE`, `CORRESPONDENCE_PROFILE`, `TRACE_ID_PROFILE`, `HISTORY_ID_PROFILE` | FR-026 |
| `ecosystem_model` | functions `read`, `export`; types `ValidatedEcosystemModel`, `ModelDocument`, `ModelCounts`, `ImprovementProposal`, `ModelDecision`, `ModelCauseCode`, `ModelAdjacency`, `ModelAdjacentEdge`, `CheckedManifestSet`, `EcosystemLimits`, `ExpectedCampaign`, `ExpectedRepository`, `ManifestEdge`, `ManifestEdgeKind`, `ManifestGap`, `ManifestNode`; constants `MANIFEST_PROFILE`, `MODEL_PROFILE`, `MANIFEST_ID_PROFILE`, `PROPOSAL_ID_PROFILE`, `MANIFEST_SCHEMA_BYTES`, `MANIFEST_SCHEMA_SHA256`, `MODEL_SCHEMA_BYTES`, `MODEL_SCHEMA_SHA256` | FR-027 |
| `ecosystem_model::manifest` | function `read`; constants `REPOSITORY_IDENTITIES`, `OWNER_MAX`; and the same eight types the `ecosystem_model` row re-exports from it (`CheckedManifestSet`, `EcosystemLimits`, `ExpectedCampaign`, `ExpectedRepository`, `ManifestEdge`, `ManifestEdgeKind`, `ManifestGap`, `ManifestNode`) | FR-027 |
| `kani` | constant `PROFILE`; types `KaniProfile`, `ProfileSelection`, `ProfileError`, `CapabilityDisposition`, `CapabilityEntry` | FR-029 |
| `kani` | types `FiniteInput`, `FiniteObject`, `FiniteReference`, `PopulationCompleteness`, `ResourceBounds`, `ValidatedFiniteInput`, `KaniOutcome`, `KaniOutcomeKind` | FR-030 |
| `kani` | types `DispatchIndex`, `DispatchError`, `ModuleDescriptor`, `SemanticFamily`, `GeneratorProvenance`, `ArtifactIdentity`, `ProvenanceError`; and one public function from a `KaniOutcome` to its `qsl_replay::TerminalValue` or a typed absence, as FR-031's map gives it | FR-031 |

An item's own public fields, variants and inherent methods are part of the
item and are not listed separately.

The root crate takes `qsl-replay` from the same QSL repository and revision
it already pins for its owner views, so taking `qsl-replay` adds no
repository edge.
It has no dependency on `quire_spec_language::runtime`.

### Items codegen owns

`lower_checked_arithmetic`, `lower_query`, `lower_reaches`,
`ArithmeticLowering`, `CheckedArithmeticRequest`, `CollectionLowering`,
`CollectionQuery`, `QueryKind`, `GraphLowering` and `GraphRequest` are not
part of this interface. The family lowerings for checked arithmetic,
collections and objects are codegen's backend adapter's
(quire-contract-codegen `spec/decisions/ADR-002-backend-adapter-boundary.md`
and `spec/functional/FR-007-bounded-kani-profile-corpus.md`), and the root
crate has no `arithmetic`, `collections` or `objects` module.

### Items QSL owns

`CounterexamplePacket`, `PacketIdentity`, `ReplaySource`, `ReplayAgreement`,
`WitnessReplayAgreement`, `InputReplayAgreement`, `NativeReplayAgreement`,
`WitnessNativeReplayAgreement`, `InputNativeReplayAgreement`,
`replay_counterexample`, `replay_with_native_runtime`, `Witness`,
`WitnessBinding`, `WitnessCheck`, `WitnessValue`, `WitnessValueType`,
`KaniProviderResult` and `KaniProviderRecord` are not part of this interface.
Their roles are QSL's `WitnessEnvelope`, `ReplaySource`, `Witness`,
`ReplayResult`, `TerminalValue`, `TerminalRecord`, `ObligationIdentity` and
`replay` (QSL FR-069 through FR-073, FR-098); the Kani transcript parser is
the codegen backend adapter's.

### Error surface

`BridgeError` carries a closed `BridgeErrorCode` whose `all()` registry and
`as_str()` spellings, registered in STD-001, are stable, and every public function over untrusted
input returns a typed error rather than panicking. `BridgeLimits` values are
caller-selected and clamped to the owner maxima.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-039-AC-1 | The root crate's public items are exactly those the Public items table lists, checked by a public-signature inventory of `src/` that fails on an added or missing item; `src/lib.rs` re-exports no `quire_contract_model` item, and code naming a model item through `quire_contract_ir` fails to compile. | Test (TC-055) |
| FR-039-AC-2 | No public item names a `quire_spec_language::runtime` type and no source file under `src/` names `quire_spec_language::runtime`. | Test (TC-055) |
| FR-039-AC-3 | Code naming any item the "Items QSL owns" or "Items codegen owns" section lists through `quire_contract_ir` fails to compile, and the value the `kani` outcome map returns is a `qsl_replay::TerminalValue` for every outcome FR-031's table maps, while for a non-vacuous `Inconclusive` it is the typed absence. | Test (TC-055) |
| FR-039-AC-4 | `BridgeErrorCode::all()` lists every variant exactly once, each `as_str()` spelling is registered in STD-001, and every public `bridge`, `predicate`, `temporal`, `ecosystem_model` and `kani` function over untrusted input returns a typed error with no public panic path under the negative corpora of TC-038 through TC-042. | Test (TC-055) |

## Dependencies

[FR-019](./FR-019-rust-library-interface.md) governs the model crate's
surface, which this crate does not re-export.
[FR-028](../contract/FR-028-separate-cycle-free-contract-model.md) fixes the
model/root package split. [AD-001](../assurance/AD-001-contract-ir-architecture.md)
records the module boundaries, the one-path rule for public items, the
QSL replay-type ownership and the placement of the Kani family lowerings in
codegen's backend adapter.
