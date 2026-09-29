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
re_exports:
  - quire_contract_model::* # FR-019 governs that surface
invariants:
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
listed below, beside the model re-export
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
decision values; bounded-Kani lowerings and typed `KaniOutcome`s, each with
its QSL `TerminalValue`.

## Behavior

### Public items

| Module | Public items | Owning requirement |
| --- | --- | --- |
| `bridge` | `BridgeError`, `BridgeErrorCode`, `BridgeDigest`, `BridgeDigestParseError`, `BridgeLimits`, `ContractSelection` | FR-025, FR-026, STD-001 |
| `predicate` | `project`, `TargetSelection`, `value`, `read_projection`, `read_valuation`, `ExpectedProjection`, `ExpectedValuation`, `PredicateDefinition`, `PredicateRef`, `PredicateCorrespondence`, `PredicateProjection`, `ValidatedPredicateProjection`, the decision and cause types, and the `PROFILE` family of profile constants | FR-025 |
| `temporal` | `project`, `join`, `read_projection`, `read_join`, the admission, correspondence, join and decision types, and the `PROFILE` family of profile constants | FR-026 |
| `ecosystem_model` | `read`, `export`, `ValidatedEcosystemModel`, `ModelDocument`, `ModelCounts`, `ImprovementProposal`, `ModelDecision`, `ModelCauseCode`, `ModelAdjacency`, `ModelAdjacentEdge`, the `manifest` module, and the profile and schema constants | FR-027 |
| `kani` | `PROFILE`; `KaniProfile`, `ProfileSelection`, `ProfileError`, `CapabilityDisposition`, `CapabilityEntry` | FR-029 |
| `kani` | `FiniteInput`, `FiniteObject`, `FiniteReference`, `PopulationCompleteness`, `ResourceBounds`, `ValidatedFiniteInput`, `KaniOutcome`, `KaniOutcomeKind` | FR-030 |
| `kani` | `DispatchIndex`, `DispatchError`, `ModuleDescriptor`, `SemanticFamily`, `GeneratorProvenance`, `ArtifactIdentity`, `ProvenanceError`, and the one total map from a `KaniOutcome` to its `qsl_replay::TerminalValue` | FR-031 |
| `kani` | `lower_checked_arithmetic`, `ArithmeticLowering`, `CheckedArithmeticRequest`, `lower_query`, `CollectionLowering`, `CollectionQuery`, `QueryKind`, `lower_reaches`, `GraphLowering`, `GraphRequest` | FR-031; placement is AD-001 OQ-2 |

The root crate takes `qsl-replay` from the same QSL repository and revision
it already pins for its owner views, so the ruling adds no repository edge.
It has no dependency on `quire_spec_language::runtime`.

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
| FR-039-AC-1 | The root crate's public items outside the `quire_contract_model` re-export are exactly those the Public items table lists, checked by a public-signature inventory of `src/` that fails on an added or missing item. | Test (TC-055) |
| FR-039-AC-2 | No public item names a `quire_spec_language::runtime` type and no source file under `src/` names `quire_spec_language::runtime`. | Test (TC-055) |
| FR-039-AC-3 | Code naming any item the "Items QSL owns" section lists through `quire_contract_ir` fails to compile, and the terminal value the `kani` outcome map returns is `qsl_replay::TerminalValue`. | Test (TC-055) |
| FR-039-AC-4 | `BridgeErrorCode::all()` lists every variant exactly once, each `as_str()` spelling is registered in STD-001, and every public `bridge`, `predicate`, `temporal`, `ecosystem_model` and `kani` function over untrusted input returns a typed error with no public panic path under the negative corpora of TC-038 through TC-042. | Test (TC-055) |

## Dependencies

[FR-019](./FR-019-rust-library-interface.md) governs the re-exported model
surface. [FR-028](../contract/FR-028-separate-cycle-free-contract-model.md)
fixes the model/bridge package split. [AD-001](../assurance/AD-001-contract-ir-architecture.md)
records the module boundaries, the QSL replay-type ownership and the open
questions on the model re-export (OQ-1) and the Kani lowering placement
(OQ-2).
