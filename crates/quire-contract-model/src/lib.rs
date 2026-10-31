//! Versioned, implementation-language-independent semantic contract model.
//!
//! This package is the cycle-free substrate shared by Quire owner crates and
//! the `quire-contract-ir` bounded-Kani crate. It intentionally has no
//! dependency on language, observation, protocol, temporal-logic, or bridge
//! packages.
//!
//! The fault-injection surface is test-only: a default build does not export
//! it (FR-019-AC-4).
//!
//! Untrusted JSON cannot deserialize directly into a validated package or
//! expression (FR-019-AC-8). Callers use the checked request decoders.
//!
//! ```compile_fail
//! let _: quire_contract_model::ContractPackage<quire_contract_model::ReferenceBody> =
//!     serde_json::from_str("{}").unwrap();
//! ```
//!
//! ```compile_fail
//! let _: quire_contract_model::Expression = serde_json::from_str("{}").unwrap();
//! ```
//!
//! Canonicalization requires an explicit profile (FR-019-AC-11).
//!
//! ```compile_fail
//! fn omit_profile<B: quire_contract_model::CanonicalBody>(
//!     package: &quire_contract_model::ContractPackage<B>,
//! ) {
//!     let _ = package.canonical_package();
//! }
//! ```
//!
#![cfg_attr(
    not(feature = "fault-injection"),
    doc = "```compile_fail,E0432\nuse quire_contract_model::MappingAllocationPoint;\n```"
)]
#![cfg_attr(
    not(feature = "fault-injection"),
    doc = "```compile_fail,E0599\nlet _ = quire_contract_model::MappingExecutionControl::fail_allocation_at;\n```"
)]
#![cfg_attr(
    not(feature = "test-support"),
    doc = "```compile_fail,E0432\nuse quire_contract_model::CheckedApplicationOperationFixture;\n```"
)]
#![cfg_attr(
    not(feature = "test-support"),
    doc = "```compile_fail,E0432\nuse quire_contract_model::CheckedApplicationOperationFixtureError;\n```"
)]
#![cfg_attr(
    not(feature = "test-support"),
    doc = "```compile_fail,E0599\nlet _ = quire_contract_model::CheckedApplicationOperation::from_test_fixture;\n```"
)]
#![cfg_attr(
    not(feature = "test-support"),
    doc = "```compile_fail,E0599\nlet _ = quire_contract_model::CheckedApplicationOperation::is_test_fixture;\n```"
)]
#![cfg_attr(
    feature = "test-support",
    doc = "```compile_fail,E0277\nuse quire_contract_model::{CheckedApplicationOperation, CheckedApplicationOperationFixture, CheckedPackageV2};\nlet operation = CheckedApplicationOperation::from_test_fixture(CheckedApplicationOperationFixture::default()).unwrap();\nlet _: CheckedPackageV2 = operation.into();\n```"
)]
#![cfg_attr(
    feature = "test-support",
    doc = "```compile_fail,E0277\nuse quire_contract_model::{CheckedApplicationOperation, CheckedApplicationOperationFixture, ContractPackageV1};\nlet operation = CheckedApplicationOperation::from_test_fixture(CheckedApplicationOperationFixture::default()).unwrap();\nlet _: ContractPackageV1 = operation.into();\n```"
)]
//! The artifact references have closed member sets (FR-019-AC-6, FR-038
//! "Artifact references"): a definition reference is `{authority, identity}`, a
//! source reference `{authority, identity}` and a locator
//! `{authority, identity}`. Each builds with exactly its members,
//!
//! ```
//! use quire_contract_model::{CheckedArtifactLocator, CheckedArtifactRef, CheckedSourceRef};
//! let _ = CheckedArtifactRef { authority: "a".into(), identity: "i".into() };
//! let _ = CheckedSourceRef {
//!     authority: "a".into(),
//!     identity: "i".into(),
//! };
//! let _ = CheckedArtifactLocator {
//!     authority: "a".into(),
//!     identity: "i".into(),
//! };
//! ```
//!
//! and fails to build with a further member, such as `revision`. Stable rustc
//! does not check the error code a `compile_fail` doctest names, and a type
//! change to an added `revision` field would fail these probes for another
//! reason, so the real oracles for a member added to a type are the positive
//! doctest above, whose literal must name every member, and
//! `tc_018_the_artifact_reference_member_sets_are_exact` in the root crate's
//! tests; these probes pin that an absent member is refused:
//!
//! ```compile_fail,E0560
//! let _ = quire_contract_model::CheckedArtifactRef {
//!     authority: "a".into(),
//!     identity: "i".into(),
//!     revision: "r".into(),
//! };
//! ```
//!
//! ```compile_fail,E0560
//! let _ = quire_contract_model::CheckedSourceRef {
//!     authority: "a".into(),
//!     identity: "i".into(),
//!     revision: "r".into(),
//! };
//! ```
//!
//! ```compile_fail,E0560
//! let _ = quire_contract_model::CheckedArtifactLocator {
//!     authority: "a".into(),
//!     identity: "i".into(),
//!     revision: "r".into(),
//! };
//! ```
//!
//! and no revision type exists:
//!
//! ```compile_fail,E0432
//! use quire_contract_model::CheckedRevision;
//! ```

mod binding;
mod canonical;
mod checked_package;
mod code;
mod conformance;
mod coverage;
mod decimal;
mod expression;
mod identity;
mod limits;
mod output_mapping;
pub mod selected_population;
pub mod source_obligation;
mod wire;

pub use binding::{
    BoundClause, BoundPackage, EXECUTABLE_PROJECTION_FORMAT, EXECUTABLE_PROJECTION_SCHEMA,
};
pub use canonical::{
    CanonicalBody, CanonicalBytes, CanonicalKind, CanonicalOutput, CanonicalProfile,
    CANONICAL_PROFILE,
};
pub use checked_package::{
    encode_checked_package_v2, read_checked_package, BoundedDomainForm, CheckedArtifactLocator,
    CheckedArtifactRef, CheckedAuthoredCompositeDomain, CheckedCanonicalIntegerBound,
    CheckedCapability, CheckedCapabilityDisposition, CheckedClauseComparison,
    CheckedCollectionKind, CheckedComparisonClauseKind, CheckedComparisonError,
    CheckedComparisonObservation, CheckedComparisonOperand, CheckedComparisonOperandIdentity,
    CheckedComparisonOperandKind, CheckedComparisonOperator, CheckedComparisonPath,
    CheckedComparisonRead, CheckedComparisonRefusal, CheckedCompositeChildEdge,
    CheckedCompositeDomainKey, CheckedCompositeDomainPosition, CheckedCompositeOperand,
    CheckedCompositeOperandDomain, CheckedCompositeOperandError, CheckedCompositeOperands,
    CheckedCompositeOperator, CheckedCompositeShapeEntry, CheckedDeclaration,
    CheckedDependencySelection, CheckedDiagnosticCause, CheckedDiagnosticCode,
    CheckedDiagnosticStage, CheckedDiagnosticV2, CheckedDiagnosticsV2,
    CheckedExtensionClauseSchema, CheckedExtensionDefinition, CheckedExtensionGrammarSchema,
    CheckedExtensionNodeSchema, CheckedExtensionProduction, CheckedExtensionRefusalDetail,
    CheckedExtensionTypedNodeSchema, CheckedMemberType, CheckedModelField, CheckedModelFieldsError,
    CheckedModelObjectFields, CheckedNodeId, CheckedNodeKind, CheckedNodeOwner,
    CheckedNodePathSegment, CheckedNodeProjectionV2, CheckedNodeTag, CheckedOccurrence,
    CheckedOccurrenceRole, CheckedOperationModeView, CheckedOperationViewError,
    CheckedPackageDispatchResult, CheckedPackageEvidence, CheckedPackageEvidenceInsertionError,
    CheckedPackageIncomplete, CheckedPackageLimit, CheckedPackageLockV2, CheckedPackageReadLimits,
    CheckedPackageRefusal, CheckedPackageRefusalCause, CheckedPackageRefusalCode,
    CheckedPackageRefusalLocation, CheckedPackageSourceCorrespondenceRef, CheckedPackageV2,
    CheckedPackageV2ReadResult, CheckedParameterDeclaration, CheckedParameterDeclarationError,
    CheckedProtocolActivation, CheckedProtocolActivationRow, CheckedProtocolDecisionRead,
    CheckedProtocolRefusalDetail, CheckedRational, CheckedReachesPopulationError,
    CheckedRefinementInternal, CheckedRefinementStepRowRegion, CheckedRelationshipEnd,
    CheckedRelationshipEndError, CheckedScalarOperand, CheckedScalarOperandChild,
    CheckedScalarOperandError, CheckedScalarOperandRange, CheckedSelection, CheckedSelectionRole,
    CheckedSemanticGraphV2, CheckedSemanticNodeV2, CheckedSourceMapEntry, CheckedSourceRef,
    CheckedSourceRegion, CheckedStepExecution, CheckedStepRelationBody,
    CheckedStepRelationCapability, CheckedStepRelationObligationIdentity, CheckedTerminalDuplicate,
    CheckedUnsupportedCompositeOperand, ClaimForm, CompleteContractNodeV2,
    CompleteContractPackageV2, CompleteLoweringProfileV2, CompleteLoweringRecordV2,
    CompleteLoweringResultV2, CompositeTypeForm, ContractBodyIncompleteV1, ContractBodyRefusalV1,
    ContractDependencies, ContractDependencyInput, ContractDispositionV1, ContractFeatureRecordV1,
    ContractGuardContextErrorV1, ContractGuardContextFailureCauseV1, ContractGuardContextFailureV1,
    ContractGuardContextViewV1, ContractGuardReferenceV1, ContractLoweredGuardContextV1,
    ContractLoweredResultV1, ContractLoweredScalarOperandsV1, ContractNodeSourcesV1,
    ContractOccurrenceKeyV1, ContractPackageDependencyV2, ContractPackageReadResult,
    ContractPackageV1, ContractPackageWireV1, ContractPathGuardV1, ContractProvenanceV1,
    ContractReachedClosureV1, ContractReachedNodeV1, ContractRefusalSourcesV1,
    ContractRequestSourcesV1, ContractResultBoundV1, ContractResultContextV1, ContractResultViewV1,
    ContractRootViewError, ContractRootViewV1, ContractScalarOperandChildV1,
    ContractScalarOperandErrorV1, ContractScalarOperandV1, ContractScalarOperandViewV1,
    ContractScalarOwnRangeV1, ContractSelectedReferenceV1, ContractSemanticGraphV1,
    ContractSourceHandoffError, ContractSourceKeyV1, ContractSourceMapV1, ContractSourceV1,
    ContractUnsupportedSourceKindError, CorrespondenceForm, DimensionTerm, ExpressionForm,
    FunctionForm, HybridForm, JsonPointer, LoweringProfileRef, ModelForm, NominalBody,
    ProtocolForm, RelationForm, ScalarTypeForm, StateForm, TemporalForm, TimedModelForm, ValueForm,
    CHECKED_PACKAGE_V2, CONTRACT_PACKAGE_VERSION,
};

pub use checked_package::{CheckedExecutionPointError, CheckedExecutionPointRefusal};

pub use checked_package::{
    ApplicationOperator, CheckedAbstractionField, CheckedAbstractionFrame,
    CheckedAbstractionObject, CheckedAbstractionParameter, CheckedAbstractionPopulation,
    CheckedApplicationOperation, CheckedBindingName, CheckedBindingNameKind,
    CheckedFairnessGranularity, CheckedFairnessKind, CheckedFieldScope, CheckedFrameModification,
    CheckedIntervalEnd, CheckedLiteralRead, CheckedLiteralValue, CheckedOperationAnchorBody,
    CheckedOperationConstraint, CheckedOperationInterval, CheckedOperationLaw,
    CheckedOperationLeaf, CheckedOperationMember, CheckedOperationMode, CheckedOperationRational,
    CheckedOperationRead, CheckedScopedDeletion, CheckedTerm, LawRole, LiteralKind,
    OperationConstraintKind, OperationMemberKind, OperationModeKind, RefinementAbstractClass,
    RefinementAssume, RefinementClassCatalog, RefinementConcreteClass, RefinementDrain,
    RefinementEnsure, RefinementFairnessKind, RefinementField, RefinementGranularity,
    RefinementHistory, RefinementHistoryUpdate, RefinementKey, RefinementMemoryClass,
    RefinementMemoryModel, RefinementObject, RefinementOpen, RefinementPackageNodeKey,
    RefinementPopulation, RefinementStep, RefinementStepArgument, RefinementStepTarget,
    RefinementTsoModel, StateClauseKind,
};
#[cfg(feature = "test-support")]
pub use checked_package::{
    CheckedApplicationOperationFixture, CheckedApplicationOperationFixtureError,
};
pub use code::{Std001Code, Std001CodeError};
pub use conformance::{
    expected_inventory, run_corpus, ConformanceOperation, FixtureResult, FixtureStatus,
    RunnerError, RunnerErrorCode, ToolIdentity, ValidationOptions, CONFORMANCE_BOUNDARIES,
    CONFORMANCE_PROTOCOL, CONFORMANCE_SCHEMA_ID, MAX_CONFORMANCE_FILE_BYTES,
    MAX_CONFORMANCE_FIXTURES, MAX_CONFORMANCE_TOTAL_BYTES, PACKAGE_SCHEMA_ID,
    PUBLIC_CONSTRUCT_TAGS,
};
pub use coverage::{
    classify_coverage, ArtifactCoverageRow, ArtifactId, ArtifactTrace, CoverageClass,
    CoverageReport, CoverageResult, OrphanReason, RequirementCoverageRow, TraceDepth,
};
pub use expression::{
    BooleanOperator, CollectionType, ComparisonOperator, DeclarationEnvironment,
    DischargedObligation, EnumDeclaration, EnumVariantDeclaration, Expression, ExpressionKind,
    FunctionParameter, IntegerDomain, IntegerType, NumericOperator, OverflowPolicy,
    PureFunctionDeclaration, QuantifierDomain, QuantifierKind, RationalType, RecordDeclaration,
    RecordFieldDeclaration, RecordLiteralField, SymbolName, TypeDeclaration, TypedExpression,
    TypedNode, ValueDeclaration, ValueDeclarationKind, ValueType, MAX_EXPRESSION_DEPTH,
    MAX_EXPRESSION_NODES, MAX_TEXT_LENGTH,
};
pub use identity::{
    AnchorName, Clause, ClauseId, ClauseKind, ClauseRef, ContractPackage,
    DefinednessObligationKind, DependencyIdentity, DependencyKind, DependencyName,
    DependencySource, Diagnostic, DiagnosticCode, ExecutionPoint, ExecutionPointName, PackageId,
    ReferenceBody, Requirement, RequirementId, RequirementRef, RequirementRevision, SchemaVersion,
    SemanticIdentity, Severity, SourceDocumentId, SourceIdentity, SourceLocation, SourceRevision,
    SourceSpan, StateObservation,
};
pub use limits::{
    MAX_SEMANTIC_COLLECTION_ITEMS, MAX_SEMANTIC_DEPTH, MAX_SEMANTIC_NODES, MAX_WIRE_JSON_DEPTH,
};
pub use output_mapping::{
    assemble_output_package, map_admitted_request, map_admitted_request_controlled,
    AdmittedMappingObligation, AdmittedMappingRequest, CompletedMappings, GeneratedOutputPackage,
    MappingCancellation, MappingCancellationToken, MappingCandidate, MappingCause,
    MappingCondition, MappingDependencyKind, MappingDependencyRef, MappingDisposition,
    MappingExecutionControl, MappingLimits, MappingRecordSource, MappingRequestError,
    MappingRequestErrorCode, MappingSourcePackageRef, MappingWorkBudget, ModelSourceSelection,
    NativeSourceSelection, ObservationAdequacyRef, ObservationAdequacyState, OutputByteRegion,
    OutputCapability, OutputGeneratorIdentity, OutputMapper, OutputMappingProfile,
    OutputMappingRecord, OutputTargetFamily, ProtocolAdequacyRef, ProtocolAdequacyState,
    RequestedMappingObligation, SemanticSourceSelection, SourceFactState,
    StructuralObservationOutcome, StructuralObservationRef, StructuralObserverIdentity,
    OUTPUT_MAPPING_REVISION,
};

#[cfg(feature = "fault-injection")]
pub use output_mapping::MappingAllocationPoint;
