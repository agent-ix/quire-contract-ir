---
id: FR-019
title: "Expose a stable Rust semantic-model interface"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-001
    type: traces_to
---
# FR-019: Expose a stable Rust semantic-model interface

## Contract

```yaml
name: ContractIrRustApi
ownership: quire-contract-ir
crate: quire-contract-model
inputs:
  - unvalidated package values
  - validation options
  - explicit schema and canonicalization profiles
outputs:
  - immutable validated packages
  - ordered structured diagnostics
  - canonical bytes and digests
  - dependency and coverage classifications
  - fixed public conformance registries
invariants:
  - wire values remain distinct from validated semantic values
  - untrusted input has no public panic path
  - no downstream engine type appears in the public contract
```

## Description

The `quire-contract-model` crate shall expose construction, validation, dependency derivation,
canonicalization, digest, and coverage-classification operations
without exposing mutable internal caches or downstream engine types.

## Inputs

Owned or borrowed contract data, validation options, and explicit supported
schema/canonicalization profiles.

## Outputs

Immutable validated packages, ordered diagnostics, canonical bytes, digests,
dependency sets, and coverage classifications.

## Behavior

The stable surface is the public API of `quire_contract_model` in a
default-feature build: exactly the items the Public items table below lists.
The crate root re-exports each of them by name from its module and has no
glob (`pub use …::*`) re-export, so a `pub` item a module adds does not join
the surface unless this table lists it. The `quire-contract-ir` root crate
re-exports none of these items (FR-039); a consumer imports them from
`quire_contract_model`. The model crate's `fault-injection` feature is
test-only and outside that surface: it exposes `MappingAllocationPoint` and
`MappingExecutionControl::fail_allocation_at`, which inject one deterministic
allocation failure so the integration tests can qualify all-or-nothing output
mapping, and only this package's own dev-dependency enables it. The
cancellation surface those controls share is stable: `MappingCancellationToken`,
`MappingExecutionControl::{active, cancelled, with_token}`, `admit_controlled`
and `map_admitted_request_controlled` let a caller cancel a running mapping
through a token it owns, checked between stages.
Serde deserialization trait implementations are not part of that surface:
untrusted package JSON enters through `ContractPackage::from_json_str` or
`from_json_bytes`, while validated values remain serializable.
Unvalidated JSON enters only through wire/request decoders. Validated identity,
package, declaration, expression, canonical, and coverage types keep
fields private and expose checked constructors plus immutable accessors. There
is no `From`/unchecked constructor from untrusted wire values to validated
types. `ValidationOptions::strict()` is the sole option set and cannot
disable limits, diagnostics, version preflight, or definedness.

Package parsing accepts UTF-8 `&str` and byte slices. Invalid UTF-8, unknown
object members, malformed wire shape, and wire nesting above the fixed limit are
`invalid_wire_format`. Parsing is separate from semantic validation and version
preflight precedes semantic conversion. Expression conformance requests decode
to public wire types, then explicitly validate declarations, expression nodes,
expected type, execution point, and clause-root policy. Fallible operations
return typed results. Public diagnostics carry code, severity, message, source
span, semantic path, related identities, and obligation kind. Callers never
need to parse display/debug/panic text.

Canonical APIs require the explicit closed `CanonicalProfile`, which registers
only `quire.contract.canonical-json/v1`. The V2 checked-package wire types
`CheckedPackageLockV2`, `CheckedPackageIdentityPreimageV2`,
`CheckedSemanticGraphV2`, `CheckedDiagnosticsV2`, `CheckedSourceMapEntry`,
`CheckedCapability` and `CheckedSemanticId` implement `quire_canonical::Encode`
(the fixed-depth ones through `FixedShape`), and those implementations are part
of the stable public surface; FR-038 owns which type takes which path and the
bytes they produce. Coverage accepts immutable traces and returns a complete report
plus ordered diagnostics. No mutable cache, global registry, filesystem path,
process handle, host-width integer, downstream engine type, or schema-library
type appears in the semantic API.

The model crate root additionally exports `expected_inventory` as stable
API; it returns the sorted published construct and boundary inventory as owned
`String` values, is pure, allocates its own output and takes no host handle.
Its name, signature and output spelling are stable.

The model crate root exports `PUBLIC_CONSTRUCT_TAGS` and `CONFORMANCE_BOUNDARIES` as
sorted fixed-width `&'static [&'static str]` registries and retains
`DiagnosticCode::ALL` as its sorted fixed-width enum registry. Their ordering,
contents, names, and types are stable API and are inspected alongside the
other public signatures.

All recursive or collection-bearing untrusted inputs undergo fixed-limit
preflight before recursive conversion. The model crate root exports the
wire-independent `MAX_SEMANTIC_NODES: u32 = 25000`,
`MAX_SEMANTIC_DEPTH: u32 = 256`, and
`MAX_SEMANTIC_COLLECTION_ITEMS: u32 = 10000`, plus the parser guard
`MAX_WIRE_JSON_DEPTH: u32 = 576`. A complete operation input may
contain at most that many decoded semantic nodes; nested value-type or other
recursive structure may be at most that deep; and every declaration,
requirement, clause, field, variant, parameter, trace, item, or other semantic
collection may contain at most that many entries. Preflight is iterative and
occurs before recursive validation, canonicalization, or coverage.
The first node, depth, or collection path crossing a limit returns
`semantic_input_too_large` and no partial semantic result. Public decode,
validate, canonicalize, and classify calls return without panic for
the complete negative corpus.

### Public items

These are the items the `quire_contract_model` crate root exports in a
default-feature build, grouped by the module that defines them. An item's own
public fields, variants, associated constants and inherent methods are part
of the item and are not listed separately.

| Module | Public items | Owning requirement |
| --- | --- | --- |
| `identity` | types `AnchorName`, `Clause`, `ClauseId`, `ClauseKind`, `ClauseRef`, `ContractPackage`, `DefinednessObligationKind`, `DependencyIdentity`, `DependencyKind`, `DependencyName`, `Diagnostic`, `DiagnosticCode`, `ExecutionPoint`, `PackageId`, `ReferenceBody`, `Requirement`, `RequirementId`, `RequirementRef`, `RequirementRevision`, `SchemaVersion`, `SemanticIdentity`, `Severity`, `SourceDocumentId`, `SourceIdentity`, `SourceLocation`, `SourceRevision`, `SourceSpan`, `StateObservation`; trait `DependencySource` | FR-011, FR-012, STD-001 |
| `expression` | types `BooleanOperator`, `CollectionType`, `ComparisonOperator`, `DeclarationEnvironment`, `DischargedObligation`, `EnumDeclaration`, `EnumVariantDeclaration`, `Expression`, `ExpressionKind`, `FunctionParameter`, `IntegerDomain`, `IntegerType`, `NumericOperator`, `OverflowPolicy`, `PureFunctionDeclaration`, `QuantifierDomain`, `QuantifierKind`, `RationalType`, `RecordDeclaration`, `RecordFieldDeclaration`, `RecordLiteralField`, `SymbolName`, `TypeDeclaration`, `TypedExpression`, `TypedNode`, `ValueDeclaration`, `ValueDeclarationKind`, `ValueType`; constants `MAX_EXPRESSION_DEPTH`, `MAX_EXPRESSION_NODES`, `MAX_TEXT_LENGTH` | FR-013 through FR-015 |
| `canonical` | types `CanonicalBytes`, `CanonicalDigest`, `CanonicalKind`, `CanonicalOutput`, `CanonicalProfile`; trait `CanonicalBody`; constant `CANONICAL_PROFILE` | FR-016, FR-017 |
| `coverage` | function `classify_coverage`; types `ArtifactCoverageRow`, `ArtifactId`, `ArtifactTrace`, `CoverageClass`, `CoverageReport`, `CoverageResult`, `OrphanReason`, `RequirementCoverageRow`, `TraceDepth` | FR-017 |
| `binding` | types `BoundClause`, `BoundPackage`; constants `BOUND_IDENTITY_PROFILE`, `EXECUTABLE_PROJECTION_FORMAT`, `EXECUTABLE_PROJECTION_SCHEMA` | FR-023 |
| `conformance` | functions `expected_inventory`, `run_corpus`; types `ConformanceOperation`, `FixtureResult`, `FixtureStatus`, `RunnerError`, `RunnerErrorCode`, `ToolIdentity`, `ValidationOptions`; constants `CONFORMANCE_BOUNDARIES`, `CONFORMANCE_PROTOCOL`, `CONFORMANCE_SCHEMA_ID`, `MAX_CONFORMANCE_FILE_BYTES`, `MAX_CONFORMANCE_FIXTURES`, `MAX_CONFORMANCE_TOTAL_BYTES`, `PACKAGE_SCHEMA_ID`, `PUBLIC_CONSTRUCT_TAGS` | FR-018 through FR-020 |
| `limits` | constants `MAX_SEMANTIC_COLLECTION_ITEMS`, `MAX_SEMANTIC_DEPTH`, `MAX_SEMANTIC_NODES`, `MAX_WIRE_JSON_DEPTH` | FR-019 |
| `output_mapping` | functions `assemble_output_package`, `map_admitted_request`, `map_admitted_request_controlled`; types `AdmittedMappingObligation`, `AdmittedMappingRequest`, `CompletedMappings`, `GeneratedOutputPackage`, `GeneratedOutputPackageId`, `MappingCancellation`, `MappingCancellationToken`, `MappingCandidate`, `MappingCause`, `MappingCondition`, `MappingDependencyKind`, `MappingDependencyRef`, `MappingDisposition`, `MappingExecutionControl`, `MappingLimits`, `MappingRecordId`, `MappingRecordSource`, `MappingRequestError`, `MappingRequestErrorCode`, `MappingRuleDigest`, `MappingSourcePackageRef`, `MappingWorkBudget`, `ModelSourceSelection`, `NativeSourceSelection`, `ObservationAdequacyRef`, `ObservationAdequacyState`, `OutputByteRegion`, `OutputCapability`, `OutputGeneratorIdentity`, `OutputMappingProfile`, `OutputMappingRecord`, `OutputTargetFamily`, `ProtocolAdequacyRef`, `ProtocolAdequacyState`, `RequestedMappingObligation`, `SemanticSourceSelection`, `SourceBytesDigest`, `SourceFactState`, `StructuralObservationOutcome`, `StructuralObservationRef`, `StructuralObserverIdentity`, `TargetBytesDigest`; trait `OutputMapper`; constants `GENERATED_OUTPUT_PACKAGE_IDENTITY_VERSION`, `OUTPUT_MAPPING_RECORD_IDENTITY_VERSION`, `OUTPUT_MAPPING_REQUEST_IDENTITY_VERSION`, `OUTPUT_MAPPING_REVISION` | FR-032 through FR-034, STD-003 |
| `checked_package` | function `read_checked_package`; types `CheckedArtifactLocator`, `CheckedArtifactRef`, `CheckedCapability`, `CheckedNodeId`, `CheckedOccurrence`, `CheckedOccurrenceRole`, `CheckedPackageDispatchResult`, `CheckedPackageEvidence`, `CheckedPackageIncomplete`, `CheckedPackageLimit`, `CheckedPackageReadLimits`, `CheckedPackageRefusal`, `CheckedPackageRefusalCause`, `CheckedPackageRefusalCode`, `CheckedSelection`, `CheckedSemanticId`, `CheckedSourceMapEntry`, `CheckedSourceRef`, `CheckedSourceRegion`, `JsonPointer` | FR-038 |
| `checked_package` (V2 reader) | types `CheckedDeclaration`, `CheckedDependencySelection`, `CheckedDiagnosticCause`, `CheckedDiagnosticCode`, `CheckedDiagnosticStage`, `CheckedDiagnosticV2`, `CheckedDiagnosticsV2`, `CheckedDomainPackageRef`, `CheckedNodeProjectionV2`, `CheckedPackageIdentityPreimageV2`, `CheckedPackageLockV2`, `CheckedPackageV2`, `CheckedPackageV2ReadResult`, `CheckedSemanticGraphV2`, `CheckedSemanticNodeV2`; constants `CHECKED_PACKAGE_V2`, `DOMAIN_PACKAGE_DIGEST`, `PACKAGE_DOMAIN_V2` | FR-038, FR-040 |
| `checked_package` (V2 nominal identity) | types `CheckedRational`, `DimensionPreimage`, `DimensionTerm`, `EnumDeclarationPreimage`, `EnumMemberPreimage`, `NominalIdentityPreimage`, `NominalOwner`, `UnitPreimage` | FR-038 |
| `checked_package` (V2 vocabulary) | types `BoundedDomainForm`, `CheckedCapabilityDisposition`, `CheckedNodeKind`, `CheckedNodeTag`, `CheckedSelectionRole`, `ClaimForm`, `CompositeTypeForm`, `CorrespondenceForm`, `ExpressionForm`, `FunctionForm`, `ModelForm`, `ProtocolForm`, `RelationForm`, `ScalarTypeForm`, `StateForm`, `TemporalForm`, `ValueForm` | FR-038, FR-040 |
| `checked_package` (V2 lowering) | types `CompleteContractNodeV2`, `CompleteContractPackageV2`, `CompleteLoweringProfileV2`, `CompleteLoweringRecordV2`, `CompleteLoweringResultV2`, `ContractPackageDependencyV2`; constants `CONTRACT_IR_SEMANTIC_DOMAIN`, `CONTRACT_PACKAGE_VERSION` | FR-035, FR-038 |

With the `fault-injection` feature the crate root also exports
`MappingAllocationPoint`; that item is outside the stable surface.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-019-AC-1 | Compile-time/API fixtures plus public-source signature inspection show wire/request values are distinct from private-field validated values, unknown members are rejected consistently with the published schema, every conversion is fallible, canonical profiles are explicit, the fixed conformance registries, three semantic-limit constants, and wire-depth constant are stable public exports, and forbidden host/downstream/schema-library vocabulary is absent without requiring nightly rustdoc JSON. | Inspection (TC-018) |
| FR-019-AC-3 | `expected_inventory` equals the five prefixed registries and nothing else, strictly ascending with no duplicate. | Test (TC-018) |
| FR-019-AC-4 | A default-feature build of `quire-contract-model` exports neither `MappingAllocationPoint` nor `MappingExecutionControl::fail_allocation_at`: code naming either fails to compile without the `fault-injection` feature. | Test (TC-018) |
| FR-019-AC-5 | The `quire_contract_model` crate root has no glob re-export, and its default-feature public items are exactly those the Public items table lists, checked by a public-item inventory that fails on an added or missing item; no item in the table is reachable through a `quire_contract_ir` path; the table's `CheckedArtifactRef` has exactly the members `authority` and `identity`, `CheckedSourceRef` exactly `authority`, `identity`, `digest_domain` and `digest`, `CheckedArtifactLocator` exactly `authority`, `identity` and `domain`, and no `CheckedRevision` exists (FR-038 "Artifact references"). | Test (TC-058) |
| FR-019-AC-2 | The complete negative corpus executes package/expression decode, validation, canonicalization, and coverage through `catch_unwind`; exact-at-limit and one-past-limit type depth, semantic node, and semantic collection cases return the specified result with no public panic, partial result, or message parsing. | Test (TC-018) |

## Dependencies

FR-011 through FR-018 define the operations and results.
