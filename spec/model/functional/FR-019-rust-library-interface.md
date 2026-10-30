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
  - canonical V1 and V2 bytes and structural V2 identities
  - dependency and coverage classifications
  - fixed public conformance registries
invariants:
  - wire values remain distinct from validated semantic values
  - untrusted input has no public panic path
  - no downstream engine type appears in the public contract
```

## Description

The `quire-contract-model` crate shall expose construction, validation, dependency derivation,
V1 and V2 canonicalization, structural V2 identity reads, and coverage-classification operations
without exposing mutable internal caches or downstream engine types.

## Inputs

Owned or borrowed contract data, validation options, and explicit supported
schema/canonicalization profiles.

## Outputs

Immutable validated packages, ordered diagnostics, canonical V1 and V2 bytes, structural V2 identities,
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
`from_json_bytes`, while validated values remain serializable. The one exception is `Std001Code`
(FR-044), which deserializes through its validating constructor and so cannot
yield an invalid code. The serialized form of
`IntegerType` and `RationalType` (and of the integer and rational literals) is
stable public surface: their `minimum`, `maximum`, `numerator_minimum`,
`numerator_maximum`, `maximum_denominator`, `value`, `numerator` and
`denominator` members serialize as decimal strings (FR-013), the spelling FR-016
canonicalizes, and a JSON number in one of them is `invalid_wire_format`.
`IntegerType::new` takes `i128` minimum and maximum and its `minimum()` and
`maximum()` return `i128`. `RationalType::new` takes `i128` numerator bounds
and maximum denominator; its three numeric accessors return `i128`.
`ExpressionKind::IntegerLiteral.value` and
`ExpressionKind::RationalLiteral.{numerator, denominator}` hold `i128`.
These are source API changes: callers holding `i64` or `u64` arguments must
convert deliberately, and callers expecting those narrower accessor results
must widen their storage or explicitly check a narrowing conversion. The wire
remains decimal-string based. The change adds no alternate constructor.
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
`CheckedPackageLockV2`, `CheckedSemanticGraphV2`, `CheckedDiagnosticsV2`,
`CheckedSourceMapEntry`, `CheckedCapability` and structural `CheckedNodeId`
implement `quire_canonical::Encode`
(the fixed-depth ones through `FixedShape`), and those implementations are part
of the stable public surface; FR-038 owns which type takes which path and the
bytes they produce. Coverage accepts immutable traces and returns a complete report
plus ordered diagnostics. No mutable cache, global registry, filesystem path,
process handle, host-width integer, downstream engine type, or schema-library
type appears in the semantic API. Here the semantic API means the public
construction, wire decoding, semantic validation, dependency derivation,
canonicalization, and coverage-classification entrypoints and their semantic
input and result types. The separately exported conformance runner and fixture
interfaces of FR-018 and FR-020 are outside that subset; `run_corpus`
deliberately accepts filesystem paths.

The model crate root additionally exports `expected_inventory` as stable
API; it returns the sorted published construct and boundary inventory as owned
`String` values, is pure, allocates its own output and takes no host handle.
Its name, signature and output spelling are stable.

The separate, default-off `test-support` feature exposes a fixture input for
`CheckedApplicationOperation` to external test consumers. Its fallible
`from_test_fixture` constructor accepts an optional operation identity, an
optional mode with the existing closed `OperationModeKind` and a value, ordered
`CheckedArtifactRef` law definitions with the fixed `integer_division` role,
and an optional member declaration key. It constructs the operation's typed
read views from those inputs without accepting an arbitrary JSON value. A
missing identity or mode, an empty law list, or missing member declaration remains a deliberate
fixture input; a present identity or mode value and each present definition
authority and identity must be nonempty. The constructor refuses malformed
fixture shape with a typed error and yields no partial operation. Because
`CheckedNodeId` has public fields, a present member declaration must carry
a non-empty package and path and an ordinal from 0 through 9007199254740991. The feature-gated `is_test_fixture` query remains true when any optional input is
absent, and false for a value obtained by direct serde deserialization or by
the production checked-package reader. Thus a consumer can distinguish a
validated fixture's intentional absence from a non-admitted operation whose
read views are empty. This feature adds no package constructor, admission
marker, or mutation path on `ContractPackageV1` or `CheckedPackageV2`;
serialization of a fixture operation is still ordinary untrusted input to the
production reader, with all normal admission checks. The fixture exists only
to exercise typed graph-reader decisions on inputs strict package admission
refuses, including CG scalar derivation's identity, mode, law, and member
precedence controls. It is outside the default-feature stable API and cannot
alter the production reader's typed non-admitted refusal.

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

The former pre-read candidate-identity exports of
[FR-046](./FR-046-derive-candidate-application-node-identity.md) and
[FR-048](./FR-048-derive-structural-and-group-candidates.md) are withdrawn
under FR-038-AC-367. The model crate exposes the structural `CheckedNodeId`
and the public V2 encoder and reader. A producer-supplied node ID or group
label is an identity label, not proof of admission. Only the strict reader
returns an admitted `CheckedPackageV2` after checking graph order, body,
owner, references and source correspondence. The former candidate helpers,
context and result types, along with retired V2 identity-preimage types and
`PACKAGE_DOMAIN_V2`, are absent from the required Public items inventory.
Their current code exports remain an open FR-019-AC-5 discrepancy until the
IR-1141 code removal lands. The structural path segment and exact node
projection are public because a consumer must construct a producer-assigned
structural ID and may inspect the admitted graph; the refinement-internal row
is public because it is a member of the public V2 graph. The V2 wire's
`SourceOwner` is represented by the already public `CheckedNodeOwner`; the
former separate `SourceOwner` and `SourceOwnerKind` Rust types described a
retired nominal identity preimage and add no distinct consumer operation.
`CONTRACT_IR_SEMANTIC_DOMAIN` likewise belonged to the retired identity
construction path. This table names the intended public surface after
IR-1141; until its code lands, FR-019-AC-5 remains unbacked for these rows.

FR-051 specifies `CheckedPackageSourceCorrespondenceRef` and
`encode_checked_package_v2`, which are implemented and exported by the
default-feature model crate. FR-052 and FR-053 specify additional public
model APIs; their owning requirements retain those API obligations, including
the planned IR-1141 structural selection controls of FR-053.
FR-038-AC-324 specifies the planned `CheckedProtocolActivation` and
`CheckedProtocolActivationRow` types and the associated
`CheckedPackageV2::protocol_activation_relation` method. They SHALL enter
the implemented Public items inventory when the reader exports them;
until then FR-038 retains the full API and behavior obligation.

### Public items

These are the required items for the `quire_contract_model` crate root in a
default-feature build, grouped by the module that defines them. Planned
exceptions are marked in their row. An item's own
public fields, variants, associated constants and inherent methods are part
of the item and are not listed separately unless an independent public
operation has its own owning requirement, as listed below.

| Module | Public items | Owning requirement |
| --- | --- | --- |
| `identity` | types `AnchorName`, `Clause`, `ClauseId`, `ClauseKind`, `ClauseRef`, `ContractPackage`, `DefinednessObligationKind`, `DependencyIdentity`, `DependencyKind`, `DependencyName`, `Diagnostic`, `DiagnosticCode`, `ExecutionPoint`, `ExecutionPointName`, `PackageId`, `ReferenceBody`, `Requirement`, `RequirementId`, `RequirementRef`, `RequirementRevision`, `SchemaVersion`, `SemanticIdentity`, `Severity`, `SourceDocumentId`, `SourceIdentity`, `SourceLocation`, `SourceRevision`, `SourceSpan`, `StateObservation`; trait `DependencySource` | FR-011, FR-012, STD-001 |
| `expression` | types `BooleanOperator`, `CollectionType`, `ComparisonOperator`, `DeclarationEnvironment`, `DischargedObligation`, `EnumDeclaration`, `EnumVariantDeclaration`, `Expression`, `ExpressionKind`, `FunctionParameter`, `IntegerDomain`, `IntegerType`, `NumericOperator`, `OverflowPolicy`, `PureFunctionDeclaration`, `QuantifierDomain`, `QuantifierKind`, `RationalType`, `RecordDeclaration`, `RecordFieldDeclaration`, `RecordLiteralField`, `SymbolName`, `TypeDeclaration`, `TypedExpression`, `TypedNode`, `ValueDeclaration`, `ValueDeclarationKind`, `ValueType`; constants `MAX_EXPRESSION_DEPTH`, `MAX_EXPRESSION_NODES`, `MAX_TEXT_LENGTH` | FR-013 through FR-015 |
| `canonical` | types `CanonicalBytes`, `CanonicalKind`, `CanonicalOutput`, `CanonicalProfile`; trait `CanonicalBody`; constant `CANONICAL_PROFILE` | FR-016, FR-017 |
| `coverage` | function `classify_coverage`; types `ArtifactCoverageRow`, `ArtifactId`, `ArtifactTrace`, `CoverageClass`, `CoverageReport`, `CoverageResult`, `OrphanReason`, `RequirementCoverageRow`, `TraceDepth` | FR-017 |
| `binding` | types `BoundClause`, `BoundPackage`; constants `EXECUTABLE_PROJECTION_FORMAT`, `EXECUTABLE_PROJECTION_SCHEMA` | FR-023 |
| `conformance` | functions `expected_inventory`, `run_corpus`; types `ConformanceOperation`, `FixtureResult`, `FixtureStatus`, `RunnerError`, `RunnerErrorCode`, `ToolIdentity`, `ValidationOptions`; constants `CONFORMANCE_BOUNDARIES`, `CONFORMANCE_PROTOCOL`, `CONFORMANCE_SCHEMA_ID`, `MAX_CONFORMANCE_FILE_BYTES`, `MAX_CONFORMANCE_FIXTURES`, `MAX_CONFORMANCE_TOTAL_BYTES`, `PACKAGE_SCHEMA_ID`, `PUBLIC_CONSTRUCT_TAGS` | FR-018 through FR-020 |
| `code` | types `Std001Code`, `Std001CodeError`; macro `std001_code` | FR-044, STD-001 |
| `limits` | constants `MAX_SEMANTIC_COLLECTION_ITEMS`, `MAX_SEMANTIC_DEPTH`, `MAX_SEMANTIC_NODES`, `MAX_WIRE_JSON_DEPTH` | FR-019 |
| `output_mapping` | functions `assemble_output_package`, `map_admitted_request`, `map_admitted_request_controlled`; types `AdmittedMappingObligation`, `AdmittedMappingRequest`, `CompletedMappings`, `GeneratedOutputPackage`, `MappingCancellation`, `MappingCancellationToken`, `MappingCandidate`, `MappingCause`, `MappingCondition`, `MappingDependencyKind`, `MappingDependencyRef`, `MappingDisposition`, `MappingExecutionControl`, `MappingLimits`, `MappingRecordSource`, `MappingRequestError`, `MappingRequestErrorCode`, `MappingSourcePackageRef`, `MappingWorkBudget`, `ModelSourceSelection`, `NativeSourceSelection`, `ObservationAdequacyRef`, `ObservationAdequacyState`, `OutputByteRegion`, `OutputCapability`, `OutputGeneratorIdentity`, `OutputMappingProfile`, `OutputMappingRecord`, `OutputTargetFamily`, `ProtocolAdequacyRef`, `ProtocolAdequacyState`, `RequestedMappingObligation`, `SemanticSourceSelection`, `SourceFactState`, `StructuralObservationOutcome`, `StructuralObservationRef`, `StructuralObserverIdentity`; trait `OutputMapper`; constants `OUTPUT_MAPPING_REVISION` | FR-032 through FR-034, STD-003 |
| `checked_package` | function `read_checked_package`; types `CheckedArtifactLocator`, `CheckedArtifactRef`, `CheckedCapability`, `CheckedNodeId`, `CheckedNodePathSegment`, `CheckedOccurrence`, `CheckedOccurrenceRole`, `CheckedPackageDispatchResult`, `CheckedPackageEvidence`, `CheckedPackageEvidenceInsertionError`, `CheckedPackageIncomplete`, `CheckedPackageLimit`, `CheckedPackageReadLimits`, `CheckedPackageRefusal`, `CheckedPackageRefusalCause`, `CheckedPackageRefusalCode`, `CheckedSelection`, `CheckedSourceMapEntry`, `CheckedSourceRef`, `CheckedSourceRegion`, `CheckedTerminalDuplicate`, `JsonPointer` | FR-038, FR-047, FR-062 |
| `checked_package` (V2 refusal locations) | type `CheckedPackageRefusalLocation`, the public record type of the refusal's loci array | FR-038-AC-368 |
| `checked_package` (V2 reader) | types `CheckedCollectionKind`, `CheckedDeclaration`, `CheckedDependencySelection`, `CheckedDiagnosticCause`, `CheckedDiagnosticCode`, `CheckedDiagnosticStage`, `CheckedDiagnosticV2`, `CheckedDiagnosticsV2`, `CheckedMemberType`, `CheckedModelField`, `CheckedModelFieldsError`, `CheckedModelObjectFields`, `CheckedNodeOwner`, `CheckedNodeProjectionV2`, `CheckedPackageLockV2`, `CheckedPackageV2`, `CheckedPackageV2ReadResult`, `CheckedProtocolActivation`, `CheckedProtocolActivationRow`, `CheckedProtocolDecisionRead`, `CheckedProtocolRefusalDetail`, `CheckedSemanticGraphV2`, `CheckedSemanticNodeV2`; constant `CHECKED_PACKAGE_V2` | FR-038, FR-040, FR-065 |
| `checked_package` (V2 nominal body fields) | type `DimensionTerm` | FR-038-AC-5 |
| `checked_package` (V2 nominal body construction and enum view) | opaque type `NominalBody` with four fallible body-shape constructors and a read-only borrowed enum-details accessor | FR-038-AC-374, FR-038-AC-375 |
| `checked_package` (V2 typed terms) | types `ApplicationOperator`, `CheckedAbstractionField`, `CheckedAbstractionFrame`, `CheckedAbstractionObject`, `CheckedAbstractionParameter`, `CheckedAbstractionPopulation`, `CheckedFieldScope`, `CheckedFrameModification`, `CheckedScopedDeletion`, `CheckedTerm`, `LiteralKind` | FR-349 |
| `checked_package` (V2 execution point and admitted operation views) | types `CheckedApplicationOperation`, `CheckedExecutionPointError`, `CheckedExecutionPointRefusal`, `CheckedOperationAnchorBody`, `CheckedOperationModeView`, `CheckedOperationViewError`, `OperationModeKind`, `StateClauseKind`; its NotAdmitted refusal distinguishes absent admission from admitted absence; the point accessor returns the canonical point type from the identity module; operation views are owned by FR-038 and FR-040 | FR-011, FR-038, FR-040, FR-058, FR-066 |
| `checked_package` (V2 refinement catalog and source rows) | types `CheckedRefinementInternal`, `CheckedRefinementStepRowRegion`, `RefinementAbstractClass`, `RefinementClassCatalog`, `RefinementConcreteClass`, `RefinementDrain`, `RefinementMemoryClass`, `RefinementMemoryModel`, `RefinementTsoModel` | FR-061 |
| `checked_package` (V2 extension evidence) | types `CheckedExtensionClauseSchema`, `CheckedExtensionDefinition`, `CheckedExtensionGrammarSchema`, `CheckedExtensionNodeSchema`, `CheckedExtensionProduction`, `CheckedExtensionRefusalDetail`, `CheckedExtensionTypedNodeSchema` | FR-348 |
| `checked_package` (V2 envelope encoder) | types `CheckedPackageSourceCorrespondenceRef`; function `encode_checked_package_v2` | FR-051, FR-061 |
| `checked_package` (V2 scalar operands) | types `CheckedScalarOperand`, `CheckedScalarOperandChild`, `CheckedScalarOperandError`, `CheckedScalarOperandRange` | FR-038 |
| `checked_package` (V2 parameter declaration) | types `CheckedParameterDeclaration`, `CheckedParameterDeclarationError` | FR-038 |
| `checked_package` (V2 step relation claim) | types `CheckedStepExecution`, `CheckedStepRelationBody`, `CheckedStepRelationCapability`, `CheckedStepRelationObligationIdentity` | FR-351 |
| `checked_package` (V2 relationship end) | types `CheckedRelationshipEnd`, `CheckedRelationshipEndError` | FR-038 |
| `checked_package` (V2 reaches population) | type `CheckedReachesPopulationError` | FR-038 |
| `checked_package` (V2 clause comparisons) | types `CheckedClauseComparison`, `CheckedComparisonClauseKind`, `CheckedComparisonError`, `CheckedComparisonObservation`, `CheckedComparisonOperand`, `CheckedComparisonOperandIdentity`, `CheckedComparisonOperandKind`, `CheckedComparisonOperator`, `CheckedComparisonPath`, `CheckedComparisonRead`, `CheckedComparisonRefusal` | FR-038 |
| `checked_package` (V2 composite operands) | types `CheckedAuthoredCompositeDomain`, `CheckedCanonicalIntegerBound`, `CheckedCompositeChildEdge`, `CheckedCompositeDomainKey`, `CheckedCompositeDomainPosition`, `CheckedCompositeOperand`, `CheckedCompositeOperandDomain`, `CheckedCompositeOperandError`, `CheckedCompositeOperands`, `CheckedCompositeOperator`, `CheckedCompositeShapeEntry`, `CheckedUnsupportedCompositeOperand` | FR-038 |
| `checked_package` (V2 rational value) | type `CheckedRational` | FR-038, FR-063 |
| `checked_package` (V2 vocabulary) | types `BoundedDomainForm`, `CheckedCapabilityDisposition`, `CheckedNodeKind`, `CheckedNodeTag`, `CheckedSelectionRole`, `ClaimForm`, `CompositeTypeForm`, `CorrespondenceForm`, `ExpressionForm`, `FunctionForm`, `HybridForm`, `ModelForm`, `ProtocolForm`, `RelationForm`, `ScalarTypeForm`, `StateForm`, `TemporalForm`, `TimedModelForm`, `ValueForm` | FR-038, FR-040, FR-065 |
| `checked_package` (V2 lowering) | types `CompleteContractNodeV2`, `CompleteContractPackageV2`, `CompleteLoweringProfileV2`, `CompleteLoweringRecordV2`, `CompleteLoweringResultV2`, `ContractPackageDependencyV2`; constant `CONTRACT_PACKAGE_VERSION` | FR-035, FR-038 |
| `checked_package` (V2 refinement body) | types `RefinementAssume`, `RefinementEnsure`, `RefinementFairnessKind`, `RefinementField`, `RefinementGranularity`, `RefinementHistory`, `RefinementHistoryUpdate`, `RefinementKey`, `RefinementObject`, `RefinementOpen`, `RefinementPackageNodeKey`, `RefinementPopulation`, `RefinementStep`, `RefinementStepArgument`, `RefinementStepTarget` | FR-038 |
| `crate_root` | module `source_obligation` | FR-052 |
| `crate_root` | module `selected_population` | FR-053 |
| `selected_population` | function `select_population`; types `CheckedAdmittedUnboundedPopulation`, `CheckedPopulationMaximum`, `CheckedSelectedPopulation`, `CheckedSelectedPopulationOutcome`, `CheckedSelectedPopulationSelector`, `CheckedSelectedPopulationSource` | FR-053 |
| `source_obligation` | types `SourceAssertionId`, `SourceAssertionProvenance`, `SourceAuthoredCriterionId`, `SourceCensusInventoryRow`, `SourceContractAuthorId`, `SourceCorrespondenceAssertion`, `SourceCorrespondenceError`, `SourceCorrespondenceRejection`, `SourceCorrespondenceRejectionReason`, `SourceCriterionSourceBinding`, `SourceDeclarationHandle`, `SourceExpressionHandle`, `SourceInventoryCancelCause`, `SourceInventoryCompleteness`, `SourceInventoryDisposition`, `SourceInventoryIdentity`, `SourceInventoryIncompleteReason`, `SourceInventoryOutcome`, `SourceInventoryOutcomeError`, `SourceInventoryOwner`, `SourceInventoryRow`, `SourceInventoryScope`, `SourceIrV2InventoryError`, `SourceMissingMappingReason`, `SourceObligationCensus`, `SourceObligationInput`, `SourceObligationRow`, `SourceObligationSite`, `SourcePendingObligationId`, `SourcePendingObligationKind`, `SourceProductionFunctionId`, `SourceProductionModuleId`, `SourceRequirementOccurrenceId`, `SourceSemanticOccurrenceId`, `SourceTypeExhaustion`, `SourceTypeWorkDimension`, `SourceUnitHandle`; trait `SourceInventoryEnumerator`; functions `build_source_obligation_census`, `source_inventory_from_ir_v2` | FR-052-AC-1 through AC-6, FR-052-AC-12 through AC-15, FR-052-AC-17 through AC-25 (IR V2 adapter backs AC-25; the other owner adapters and composite AC-7/9/10/11 remain Planned) |
| `checked_package` (ContractPackage v1) | types `ContractBodyIncompleteV1`, `ContractBodyRefusalV1`, `ContractDependencies`, `ContractDependencyInput`, `ContractDispositionV1`, `ContractFeatureRecordV1`, `ContractGuardContextErrorV1`, `ContractGuardContextFailureCauseV1`, `ContractGuardContextFailureV1`, `ContractGuardContextViewV1`, `ContractGuardReferenceV1`, `ContractLoweredGuardContextV1`, `ContractLoweredResultV1`, `ContractLoweredScalarOperandsV1`, `ContractNodeSourcesV1`, `ContractOccurrenceKeyV1`, `ContractPackageReadResult`, `ContractPackageV1`, `ContractPackageWireV1`, `ContractPathGuardV1`, `ContractProvenanceV1`, `ContractReachedClosureV1`, `ContractReachedNodeV1`, `ContractRefusalSourcesV1`, `ContractRequestSourcesV1`, `ContractResultBoundV1`, `ContractResultContextV1`, `ContractResultViewV1`, `ContractRootViewError`, `ContractRootViewV1`, `ContractScalarOperandChildV1`, `ContractScalarOperandErrorV1`, `ContractScalarOperandV1`, `ContractScalarOperandViewV1`, `ContractScalarOwnRangeV1`, `ContractSelectedReferenceV1`, `ContractSemanticGraphV1`, `ContractSourceHandoffError`, `ContractSourceMapV1`, `ContractSourceV1`, `ContractUnsupportedSourceKindError`, `LoweringProfileRef` | FR-035, FR-049, FR-050, FR-054 (consumes QSL FR-062 path conditions) |

With the `fault-injection` feature the crate root also exports
`MappingAllocationPoint`; that item is outside the stable surface.
With the separate `test-support` feature the crate root also exports
`CheckedApplicationOperationFixture` and
`CheckedApplicationOperationFixtureError`; these items and
`CheckedApplicationOperation::{from_test_fixture,is_test_fixture}` are outside
the default-feature stable surface. Enabling both features adds the union of
their independent exports.

The public `CheckedPackageV2::lower_independent_records` method is defined on
the `CheckedPackageV2` type listed above; it is not a separate crate-root export.
The public `CheckedPackageV2::clause_execution_point` method is likewise on
that listed type; its typed error and refusal types are in the V2 execution
point row.
The public `ContractPackageV1::node` and `ContractPackageV1::term` keyed
lookups are methods on the listed `ContractPackageV1` type, not separate
crate-root exports. They expose the represented node and admitted typed body
of the lowered package, including after strict reading.
The planned FR-035 authenticated V2-source handoff on `ContractPackageWireV1`
is a method on that listed type. Its admitted result is `ContractPackageV1`;
it does not add a second package or wire type (IR-1087).

The table records the required target public surface. Planned entries are
excluded from the current compiled inventory until their owning code lands.
The current candidate-helper and identity-preimage exports are a
PLANNED/UNRUN IR-1141 removal; TC-058 retains the exact compiled-inventory
check for the final code revision. `SourceAuthoredCriterionId`
identifies a Quire-authored acceptance criterion, and
`SourceCriterionSourceBinding` records the source author's selected Quire scope
and IR source. The neutral input retains the binding; synthetic census tests verify typed
missing, foreign, and conflicting binding refusals under FR-052-AC-15.

### Structural V2 identities

`CheckedNodeId` is the public closed structural value `{package, path, ordinal}`.
The package name and declared contents are read directly from the envelope. No
identity preimage, projection, semantic digest, derived node-key helper or
old-wire admission path is exported.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-019-AC-1 | A TC-018 integration test imports `PUBLIC_CONSTRUCT_TAGS`, `CONFORMANCE_BOUNDARIES`, `DiagnosticCode::ALL`, `ConformanceOperation::ALL`, and `expected_inventory` from `quire_contract_model` and uses them to construct the published inventory. | Test (TC-018) |
| FR-019-AC-3 | `expected_inventory` equals the five prefixed registries and nothing else, strictly ascending with no duplicate. | Test (TC-018) |
| FR-019-AC-4 | A default-feature build of `quire-contract-model` exports neither `MappingAllocationPoint` nor `MappingExecutionControl::fail_allocation_at`: code naming either fails to compile without the `fault-injection` feature. | Test (TC-018) |
| FR-019-AC-5 | The `quire_contract_model` crate root has no glob re-export, and its default-feature public items are exactly those the Public items table lists as implemented at the tested revision. Entries explicitly marked planned are excluded until their owning code lands; exporting a planned item fails the inventory until its row is marked implemented. The public-item inventory fails on an added or missing implemented item; no implemented item in the table is reachable through a `quire_contract_ir` path. The retired `PACKAGE_DOMAIN_V2` export is absent. | Test (TC-058) |
| FR-019-AC-6 | The table's `CheckedArtifactRef`, `CheckedSourceRef` and `CheckedArtifactLocator` each have exactly the members `authority` and `identity`, and no `CheckedRevision` exists (FR-038 "Artifact references"). A struct literal with those two members builds for each type, while one naming a `domain`, `digest_domain` or `digest` member fails to compile. | Test (TC-018) |
| FR-019-AC-2 | The complete negative corpus executes package/expression decode, validation, canonicalization, and coverage through `catch_unwind`; exact-at-limit and one-past-limit type depth, semantic node, and semantic collection cases return the specified result with no public panic, partial result, or message parsing. | Test (TC-018) |
| FR-019-AC-7 | Public compile-time/API fixtures construct and read `IntegerType` bounds, `RationalType` numerator and denominator bounds, and integer/rational literal payloads at `i64::MAX + 1`, `u64::MAX` and the permitted i128 endpoints through signed 128-bit signatures without lossy casts; exact and one-past numeric wire boundary requests through `catch_unwind` return the specified result with no public panic or partial validated value. | Test (TC-018) |
| FR-019-AC-8 | Compile-time fixtures and public-source inspection show package and expression wire/request values are distinct from validated semantic values, validated fields are private, and no unchecked constructor or `From` conversion crosses that boundary. | Inspection (TC-018) |
| FR-019-AC-9 | An otherwise valid package or expression request with one unknown object member is refused as `invalid_wire_format`, consistently with its published schema, before it yields a validated value. | Test (TC-018) |
| FR-019-AC-10 | Public package and expression decoding, semantic validation, canonicalization, and coverage classification signatures return typed fallible results; none silently converts an invalid request into a validated value. | Inspection (TC-018) |
| FR-019-AC-11 | Each public canonicalization entrypoint requires an explicit `CanonicalProfile`, and the closed profile exposes only `quire.contract.canonical-json/v1`; omitting the profile does not compile. | Inspection (TC-018) |
| FR-019-AC-12 | A default-feature external API test imports `MAX_SEMANTIC_NODES: u32 = 25000`, `MAX_SEMANTIC_DEPTH: u32 = 256`, `MAX_SEMANTIC_COLLECTION_ITEMS: u32 = 10000`, and `MAX_WIRE_JSON_DEPTH: u32 = 576` from `quire_contract_model` and checks their exact values. | Test (TC-018) |
| FR-019-AC-13 | Public-source signature inspection finds no host handle, filesystem path, mutable cache, downstream engine type, or schema-library type in the default-feature semantic API subset defined in Behavior; it excludes the conformance runner and fixture APIs and runs on stable Rust without nightly rustdoc JSON. | Inspection (TC-018) |
| FR-019-AC-14 | With `test-support`, an external consumer constructs typed fixture operations with absent identity, mode, law definitions or member declaration, and with the identity, exact/non-exact rounding modes, foreign and conflicting division definitions, and wrong-kind/wrong-value/matching mode inputs used by the four CG FR-014-AC-18/48/50 scalar derivation controls. Typed `identity`, `mode`, `law_definitions`, and `member_declaration` reads return the authored values in order, including intentional absence; `is_test_fixture` is true in every case. No direct JSON view injection is required. | Test (TC-058) |
| FR-019-AC-15 | The test fixture constructor rejects an empty present identity, an empty present mode value, a law definition with empty authority or identity, and a present member key with an empty package or path or out-of-range ordinal by typed error without yielding an operation. A fixture with intentionally absent optional fields remains constructible and distinguishable through `is_test_fixture` from direct-serde non-admitted operations. A fixture operation cannot be supplied as an already admitted `ContractPackageV1` or `CheckedPackageV2`, and serializing it into a package wire does not bypass strict production admission or its typed refusal. | Test (TC-058) |
| FR-019-AC-16 | `test-support` is disabled by default; a default-feature external compile probe cannot import its fixture types or call its constructor or provenance query. A feature-enabled external probe can use exactly the documented fixture exports and methods; the existing default-feature inventory, the independent `fault-injection` delta, and the absence of a model API through `quire_contract_ir` remain enforced. | Test (TC-058) |

## Dependencies

FR-011 through FR-018 define the operations and results.
