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
#![cfg_attr(
    not(feature = "fault-injection"),
    doc = "```compile_fail,E0432\nuse quire_contract_model::MappingAllocationPoint;\n```"
)]
#![cfg_attr(
    not(feature = "fault-injection"),
    doc = "```compile_fail,E0599\nlet _ = quire_contract_model::MappingExecutionControl::fail_allocation_at;\n```"
)]
//! The artifact references have closed member sets (FR-019-AC-6, FR-038
//! "Artifact references"): a definition reference is `{authority, identity}`, a
//! source reference `{authority, identity, digest_domain, digest}` and a locator
//! `{authority, identity, domain}`. Each builds with exactly its members,
//!
//! ```
//! use quire_contract_model::{CheckedArtifactLocator, CheckedArtifactRef, CheckedSourceRef};
//! let _ = CheckedArtifactRef { authority: "a".into(), identity: "i".into() };
//! let _ = CheckedSourceRef {
//!     authority: "a".into(),
//!     identity: "i".into(),
//!     digest_domain: "d".into(),
//!     digest: "x".into(),
//! };
//! let _ = CheckedArtifactLocator {
//!     authority: "a".into(),
//!     identity: "i".into(),
//!     domain: "d".into(),
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
//!     digest_domain: "d".into(),
//!     digest: "x".into(),
//!     revision: "r".into(),
//! };
//! ```
//!
//! ```compile_fail,E0560
//! let _ = quire_contract_model::CheckedArtifactLocator {
//!     authority: "a".into(),
//!     identity: "i".into(),
//!     domain: "d".into(),
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
mod wire;

pub use binding::{
    BoundClause, BoundPackage, BOUND_IDENTITY_PROFILE, EXECUTABLE_PROJECTION_FORMAT,
    EXECUTABLE_PROJECTION_SCHEMA,
};
pub use canonical::{
    CanonicalBody, CanonicalBytes, CanonicalDigest, CanonicalKind, CanonicalOutput,
    CanonicalProfile, CANONICAL_PROFILE,
};
pub use checked_package::{
    read_checked_package, BoundedDomainForm, CheckedArtifactLocator, CheckedArtifactRef,
    CheckedAuthoredCompositeDomain, CheckedCanonicalIntegerBound, CheckedCapability,
    CheckedCapabilityDisposition, CheckedCollectionKind, CheckedCompositeChildEdge,
    CheckedCompositeDomainKey, CheckedCompositeDomainPosition, CheckedCompositeOperand,
    CheckedCompositeOperandDomain, CheckedCompositeOperandError, CheckedCompositeOperands,
    CheckedCompositeShapeEntry, CheckedDeclaration, CheckedDependencySelection,
    CheckedDiagnosticCause, CheckedDiagnosticCode, CheckedDiagnosticStage, CheckedDiagnosticV2,
    CheckedDiagnosticsV2, CheckedDomainPackageRef, CheckedMemberType, CheckedModelField,
    CheckedModelFieldsError, CheckedModelObjectFields, CheckedNodeId, CheckedNodeKind,
    CheckedNodeOwner, CheckedNodeProjectionV2, CheckedNodeTag, CheckedOccurrence,
    CheckedOccurrenceRole, CheckedPackageDispatchResult, CheckedPackageEvidence,
    CheckedPackageIdentityPreimageV2, CheckedPackageIncomplete, CheckedPackageLimit,
    CheckedPackageLockV2, CheckedPackageReadLimits, CheckedPackageRefusal,
    CheckedPackageRefusalCause, CheckedPackageRefusalCode, CheckedPackageV2,
    CheckedPackageV2ReadResult, CheckedRational, CheckedScalarOperand, CheckedScalarOperandChild,
    CheckedScalarOperandError, CheckedScalarOperandRange, CheckedSelection, CheckedSelectionRole,
    CheckedSemanticGraphV2, CheckedSemanticId, CheckedSemanticNodeV2, CheckedSourceMapEntry,
    CheckedSourceRef, CheckedSourceRegion, CheckedUnsupportedCompositeOperand, ClaimForm,
    CompleteContractNodeV2, CompleteContractPackageV2, CompleteLoweringProfileV2,
    CompleteLoweringRecordV2, CompleteLoweringResultV2, CompositeTypeForm,
    ContractPackageDependencyV2, CorrespondenceForm, DimensionPreimage, DimensionTerm,
    EnumDeclarationPreimage, EnumMemberPreimage, ExpressionForm, FunctionForm, JsonPointer,
    ModelForm, NominalIdentityPreimage, NominalOwner, ProtocolForm, RelationForm, ScalarTypeForm,
    StateForm, TemporalForm, UnitPreimage, ValueForm, CHECKED_PACKAGE_V2,
    CONTRACT_IR_SEMANTIC_DOMAIN, CONTRACT_PACKAGE_VERSION, DOMAIN_PACKAGE_DIGEST,
    PACKAGE_DOMAIN_V2,
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
    DependencySource, Diagnostic, DiagnosticCode, ExecutionPoint, PackageId, ReferenceBody,
    Requirement, RequirementId, RequirementRef, RequirementRevision, SchemaVersion,
    SemanticIdentity, Severity, SourceDocumentId, SourceIdentity, SourceLocation, SourceRevision,
    SourceSpan, StateObservation,
};
pub use limits::{
    MAX_SEMANTIC_COLLECTION_ITEMS, MAX_SEMANTIC_DEPTH, MAX_SEMANTIC_NODES, MAX_WIRE_JSON_DEPTH,
};
pub use output_mapping::{
    assemble_output_package, map_admitted_request, map_admitted_request_controlled,
    AdmittedMappingObligation, AdmittedMappingRequest, CompletedMappings, GeneratedOutputPackage,
    GeneratedOutputPackageId, MappingCancellation, MappingCancellationToken, MappingCandidate,
    MappingCause, MappingCondition, MappingDependencyKind, MappingDependencyRef,
    MappingDisposition, MappingExecutionControl, MappingLimits, MappingRecordId,
    MappingRecordSource, MappingRequestError, MappingRequestErrorCode, MappingRuleDigest,
    MappingSourcePackageRef, MappingWorkBudget, ModelSourceSelection, NativeSourceSelection,
    ObservationAdequacyRef, ObservationAdequacyState, OutputByteRegion, OutputCapability,
    OutputGeneratorIdentity, OutputMapper, OutputMappingProfile, OutputMappingRecord,
    OutputTargetFamily, ProtocolAdequacyRef, ProtocolAdequacyState, RequestedMappingObligation,
    SemanticSourceSelection, SourceBytesDigest, SourceFactState, StructuralObservationOutcome,
    StructuralObservationRef, StructuralObserverIdentity, TargetBytesDigest,
    GENERATED_OUTPUT_PACKAGE_IDENTITY_VERSION, OUTPUT_MAPPING_RECORD_IDENTITY_VERSION,
    OUTPUT_MAPPING_REQUEST_IDENTITY_VERSION, OUTPUT_MAPPING_REVISION,
};

#[cfg(feature = "fault-injection")]
pub use output_mapping::MappingAllocationPoint;
