//! Strict reader for QSpec I04 `quire.checked-package/v2`, the sole admitted
//! `CheckedPackage` contract.
//!
//! Implements the I04 contract described by AD-006. The wire shape this
//! module admits is stated here, in the types and checks below, rather than
//! by pointing at a revision of another repository. Model, source, and
//! definition selections retain the owning names and exact structure.

mod abstraction;
mod claim_terminal;
mod closure_index;
mod comparison_operands;
mod composite_operands;
mod contract_package;
mod dependency_references;
pub(in crate::checked_package) mod encode;
mod execution_point;
mod execution_point_accessor;
mod extension;
mod flat_wire;
mod frame;
mod hybrid;
mod identity;
mod intake;
mod lossless;
mod lower;
mod model_fields;
mod model_members;
mod model_root;
mod natural;
mod operation_catalog;
mod operations;
mod owner;
mod parameter_declaration;
mod protocol_body;
mod reaches_population;
mod refinement;
mod relationship_end;
mod rust_spelling;
mod scalar_operands;
mod state;
mod step_relation;
mod structural;
mod temporal;
mod timed_model;
mod typed_terms;
mod vocabulary;

pub use comparison_operands::{
    CheckedClauseComparison, CheckedComparisonClauseKind, CheckedComparisonError,
    CheckedComparisonObservation, CheckedComparisonOperand, CheckedComparisonOperandIdentity,
    CheckedComparisonOperandKind, CheckedComparisonOperator, CheckedComparisonPath,
    CheckedComparisonRead, CheckedComparisonRefusal,
};
pub use composite_operands::{
    CheckedAuthoredCompositeDomain, CheckedCanonicalIntegerBound, CheckedCompositeChildEdge,
    CheckedCompositeDomainKey, CheckedCompositeDomainPosition, CheckedCompositeOperand,
    CheckedCompositeOperandDomain, CheckedCompositeOperandError, CheckedCompositeOperands,
    CheckedCompositeOperator, CheckedCompositeShapeEntry, CheckedUnsupportedCompositeOperand,
};
pub use contract_package::{
    ContractBodyIncompleteV1, ContractBodyRefusalV1, ContractDependencies, ContractDependencyInput,
    ContractDispositionV1, ContractFeatureRecordV1, ContractGuardContextErrorV1,
    ContractGuardContextFailureCauseV1, ContractGuardContextFailureV1, ContractGuardContextViewV1,
    ContractGuardReferenceV1, ContractLoweredGuardContextV1, ContractLoweredResultV1,
    ContractLoweredScalarOperandsV1, ContractNodeSourcesV1, ContractOccurrenceKeyV1,
    ContractPackageReadResult, ContractPackageV1, ContractPackageWireV1, ContractPathGuardV1,
    ContractProvenanceV1, ContractReachedClosureV1, ContractReachedNodeV1,
    ContractRefusalSourcesV1, ContractRequestSourcesV1, ContractResultBoundV1,
    ContractResultContextV1, ContractResultViewV1, ContractRootViewError, ContractRootViewV1,
    ContractScalarOperandChildV1, ContractScalarOperandErrorV1, ContractScalarOperandV1,
    ContractScalarOperandViewV1, ContractScalarOwnRangeV1, ContractSelectedReferenceV1,
    ContractSemanticGraphV1, ContractSourceHandoffError, ContractSourceMapV1, ContractSourceV1,
    ContractUnsupportedSourceKindError,
};
pub use execution_point_accessor::{CheckedExecutionPointError, CheckedExecutionPointRefusal};
pub use identity::*;
pub use lower::*;
pub use model_fields::{
    CheckedCollectionKind, CheckedMemberType, CheckedModelField, CheckedModelFieldsError,
    CheckedModelObjectFields,
};
pub use parameter_declaration::{CheckedParameterDeclaration, CheckedParameterDeclarationError};
pub use protocol_body::{CheckedProtocolActivation, CheckedProtocolActivationRow};
pub use reaches_population::CheckedReachesPopulationError;
pub use refinement::*;
pub use relationship_end::{CheckedRelationshipEnd, CheckedRelationshipEndError};
pub use scalar_operands::{
    CheckedScalarOperand, CheckedScalarOperandChild, CheckedScalarOperandError,
    CheckedScalarOperandRange,
};
pub use step_relation::{
    CheckedStepExecution, CheckedStepRelationBody, CheckedStepRelationCapability,
    CheckedStepRelationObligationIdentity,
};
pub use typed_terms::*;
pub use vocabulary::*;

use crate::identity::ExecutionPoint;
use closure_index::ClosureIndex;
use dependency_references::{admit_dependencies, check_derived_closure, SuppliedDependencies};
use ix_trace_rs::string_edge;
use operations::validate_operations;
use owner::{validate_owner_joins, validate_owner_schema};
use structural::{validate_derived_type_shapes, validate_structural_nodes};
use temporal::{validate_temporal, validate_timed_bounds_reduced};

use super::common::{
    count, exceeds, is_nonempty, node_pointer, read_typed_prefix, validate_definition_ref,
    validate_locked_source, validate_source_map_entries, validate_term, ReferenceMember,
    ReferenceSite, ReferenceVisitor, Step, TermGrammar, Trail, ValidationFailure,
};
use super::evidence::CheckedPackageEvidence;
use super::shared::{
    CheckedArtifactRef, CheckedCapability, CheckedNodeId, CheckedNodePathSegment,
    CheckedOccurrence, CheckedOccurrenceRole, CheckedPackageIncomplete, CheckedPackageLimit,
    CheckedPackageReadLimits, CheckedPackageRefusal, CheckedPackageRefusalCause,
    CheckedPackageRefusalCode, CheckedSelection, CheckedSourceMapEntry, CheckedSourceRef,
    CheckedSourceRegion, JsonPointer,
};
use model_fields::{build_field_tables, DeclarationIndex, RetainedModels};
use model_members::{
    admit_selection, Budget, DomainModel, ModelOwners, SelectionFailure, UnitRefs,
    DURATION_RESOLVED_PREFIX,
};
use quire_canonical::FixedShape;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

// Explicit null is a value; an absent member must remain a schema refusal.
fn required_nullable<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

/// The I04 transport version admitted by the V2 reader.
pub const CHECKED_PACKAGE_V2: &str = "quire.checked-package/v2";
/// The V2 semantic package identity domain.
pub const PACKAGE_DOMAIN_V2: &str = "quire.package.semantic/v2";
const GRAPH_V2: &str = "quire.checked-semantic-graph/v2";

#[derive(Clone, Copy, Eq, PartialEq)]
enum PackageWireTag {
    ContractVersion,
    GraphVersion,
}

impl PackageWireTag {
    #[ix_trace_rs::string_edge]
    fn from_wire(text: &str) -> Option<Self> {
        match text {
            CHECKED_PACKAGE_V2 => Some(Self::ContractVersion),
            GRAPH_V2 => Some(Self::GraphVersion),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum KnownOperation {
    StateClause,
    StateTransition,
    ProtocolControl,
    NumericNarrow,
    ModelNavigate,
    ModelReaches,
    FunctionCall,
}

impl KnownOperation {
    #[ix_trace_rs::string_edge]
    fn from_wire(text: &str) -> Option<Self> {
        match text {
            "quire.op.state.clause" => Some(Self::StateClause),
            "quire.op.state.transition" => Some(Self::StateTransition),
            "quire.op.protocol.control" => Some(Self::ProtocolControl),
            "quire.op.numeric.narrow" => Some(Self::NumericNarrow),
            "quire.op.model.navigate" => Some(Self::ModelNavigate),
            "quire.op.model.reaches" => Some(Self::ModelReaches),
            "quire.op.function.call" => Some(Self::FunctionCall),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum TermBindingName {
    Parameters,
    Body,
    Optional,
    Clause,
    Other,
}

impl TermBindingName {
    #[ix_trace_rs::string_edge]
    fn from_wire(text: &str) -> Self {
        match text {
            "parameters" => Self::Parameters,
            "body" => Self::Body,
            "optional" => Self::Optional,
            "clause" => Self::Clause,
            _ => Self::Other,
        }
    }
}
/// A node's declared qualified name (FR-208). Present exactly where the
/// schema's `DeclarationOccurrenceRule` requires it and forbidden where
/// `DeclarationTagRules` forbids it; see [`validate_declaration`].
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(deny_unknown_fields)]
pub struct CheckedDeclaration {
    /// ASCII identifier segments.
    pub qualified_name: Vec<Box<str>>,
}

/// The closed owner of a structural checked node.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum CheckedNodeOwner {
    /// The selected domain package as a whole, named by its identity.
    ModelRoot {
        /// Selected domain package identity.
        identity: Box<str>,
    },
    /// A declared node owned by one selected raw source unit.
    Source {
        /// Source authority.
        authority: Box<str>,
        /// Source identity.
        identity: Box<str>,
    },
    /// An undeclared node owned by one selected domain model declaration.
    Model {
        /// Selected domain package identity.
        identity: Box<str>,
        /// Declaring model node identity.
        node: Box<str>,
    },
}

/// A checked V2 semantic graph node.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedSemanticNodeV2<Body = CheckedTerm> {
    /// Stable checked node identity.
    pub node_id: CheckedNodeId,
    /// Must be `quire.checked-semantic-graph/v2`.
    pub schema_version: Box<str>,
    /// Closed semantic family (see [`CheckedNodeTag`]).
    pub node_tag: Box<str>,
    /// Closed semantic form for the family.
    pub semantic_form: Box<str>,
    /// Checked semantic type identity.
    pub semantic_type: CheckedNodeId,
    /// Identity-based dependencies.
    pub dependencies: Vec<CheckedNodeId>,
    /// Exact source occurrence keys.
    pub occurrences: Vec<CheckedOccurrence>,
    /// Explicit recursion group when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recursion_group: Option<Box<str>>,
    /// Declared qualified name; present exactly per `DeclarationOccurrenceRule`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub declaration: Option<CheckedDeclaration>,
    /// Structural owner, present exactly for declared or model-owned nodes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<CheckedNodeOwner>,
    /// Exact local model declaration owning a protocol clause.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_model: Option<CheckedNodeId>,
    /// Typed public semantic term.
    pub body: Body,
}

type RawSemanticNodeV2 = CheckedSemanticNodeV2<Value>;

impl RawSemanticNodeV2 {
    fn into_typed(mut self, body: CheckedTerm) -> CheckedSemanticNodeV2 {
        quire_canonical::drop_value(std::mem::take(&mut self.body));
        CheckedSemanticNodeV2 {
            node_id: self.node_id,
            schema_version: self.schema_version,
            node_tag: self.node_tag,
            semantic_form: self.semantic_form,
            semantic_type: self.semantic_type,
            dependencies: self.dependencies,
            occurrences: self.occurrences,
            recursion_group: self.recursion_group,
            declaration: self.declaration,
            owner: self.owner,
            owner_model: self.owner_model,
            body,
        }
    }
}

impl CheckedSemanticNodeV2 {
    /// Replace an exact node identity throughout this admitted, cloned node.
    /// This does not derive a replacement key or validate the resulting graph.
    pub fn replace_node_id(&mut self, old: &CheckedNodeId, new: &CheckedNodeId) {
        fn replace(id: &mut CheckedNodeId, old: &CheckedNodeId, new: &CheckedNodeId) {
            if id == old {
                *id = new.clone();
            }
        }
        replace(&mut self.node_id, old, new);
        replace(&mut self.semantic_type, old, new);
        for id in &mut self.dependencies {
            replace(id, old, new);
        }
        if let Some(owner_model) = &mut self.owner_model {
            replace(owner_model, old, new);
        }
        self.body.replace_node_id(old, new);
    }

    fn to_raw(&self) -> RawSemanticNodeV2 {
        self.try_to_raw().expect("typed term serializes to JSON")
    }
}

impl<Body: Serialize> CheckedSemanticNodeV2<Body> {
    fn try_to_raw(&self) -> Result<RawSemanticNodeV2, serde_json::Error> {
        Ok(RawSemanticNodeV2 {
            node_id: self.node_id.clone(),
            schema_version: self.schema_version.clone(),
            node_tag: self.node_tag.clone(),
            semantic_form: self.semantic_form.clone(),
            semantic_type: self.semantic_type.clone(),
            dependencies: self.dependencies.clone(),
            occurrences: self.occurrences.clone(),
            recursion_group: self.recursion_group.clone(),
            declaration: self.declaration.clone(),
            owner: self.owner.clone(),
            owner_model: self.owner_model.clone(),
            body: serde_json::to_value(&self.body)?,
        })
    }
}

/// Identity projection of a V2 node, excluding source occurrences.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedNodeProjectionV2<Body = CheckedTerm> {
    /// Stable node identity.
    pub node_id: CheckedNodeId,
    /// Graph schema version.
    pub schema_version: Box<str>,
    /// Closed node family tag.
    pub node_tag: Box<str>,
    /// Closed form within its node family.
    pub semantic_form: Box<str>,
    /// Checked semantic type identity.
    pub semantic_type: CheckedNodeId,
    /// Identity-based dependencies.
    pub dependencies: Vec<CheckedNodeId>,
    /// Explicit recursion group when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recursion_group: Option<Box<str>>,
    /// Declared qualified name when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub declaration: Option<CheckedDeclaration>,
    /// The corresponding node's structural owner.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<CheckedNodeOwner>,
    /// Exact local model declaration owning a protocol clause.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_model: Option<CheckedNodeId>,
    /// Typed semantic term.
    pub body: Body,
}

impl<Body: Clone> From<&CheckedSemanticNodeV2<Body>> for CheckedNodeProjectionV2<Body> {
    fn from(node: &CheckedSemanticNodeV2<Body>) -> Self {
        Self {
            node_id: node.node_id.clone(),
            schema_version: node.schema_version.clone(),
            node_tag: node.node_tag.clone(),
            semantic_form: node.semantic_form.clone(),
            semantic_type: node.semantic_type.clone(),
            dependencies: node.dependencies.clone(),
            recursion_group: node.recursion_group.clone(),
            declaration: node.declaration.clone(),
            owner: node.owner.clone(),
            owner_model: node.owner_model.clone(),
            body: node.body.clone(),
        }
    }
}

/// One selected library dependency, identified by the imported library name.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(deny_unknown_fields)]
pub struct CheckedDependencySelection {
    /// Library identity; the sort and uniqueness key of the array.
    pub identity: Box<str>,
}

/// One refinement and its nonempty canonical list of internal step classes.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(deny_unknown_fields)]
pub struct CheckedRefinementInternal {
    /// Qualified refinement declaration name.
    pub refinement: Box<str>,
    /// Step class spellings excluded from external obligations.
    pub internal: Vec<Box<str>>,
}

/// The exact immutable V2 package lock.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(deny_unknown_fields)]
pub struct CheckedPackageLockV2 {
    /// Locked raw source documents.
    pub sources: Vec<CheckedSourceRef>,
    /// Selected edition.
    pub edition: CheckedSelection,
    /// Selected profiles.
    pub profile_selections: Vec<CheckedSelection>,
    /// Selected definitions.
    pub definition_selections: Vec<CheckedArtifactRef>,
    /// Required features.
    pub required_features: Vec<Box<str>>,
    /// Dependency selections: one per library identity, ascending.
    pub dependency_selections: Vec<CheckedDependencySelection>,
    /// Nonempty refinement internal lists, omitted when no list is present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refinement_internals: Option<Vec<CheckedRefinementInternal>>,
}

/// A checked V2 graph.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedSemanticGraphV2<Body = CheckedTerm> {
    /// Must be `quire.checked-semantic-graph/v2`.
    pub graph_version: Box<str>,
    /// Complete closed graph.
    pub nodes: Vec<CheckedSemanticNodeV2<Body>>,
}

type RawSemanticGraphV2 = CheckedSemanticGraphV2<Value>;

/// Pipeline stage of a typed V2 diagnostic.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(rename_all = "snake_case")]
pub enum CheckedDiagnosticStage {
    /// `source_recognition`.
    SourceRecognition,
    /// `resolution`.
    Resolution,
    /// `type_checking`.
    TypeChecking,
    /// `package_read`.
    PackageRead,
    /// `lowering`.
    Lowering,
}

/// Closed V2 diagnostic code.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(rename_all = "snake_case")]
pub enum CheckedDiagnosticCode {
    /// `invalid_syntax`.
    InvalidSyntax,
    /// `unsupported_construct`.
    UnsupportedConstruct,
    /// `unknown_language`.
    UnknownLanguage,
    /// `unknown_edition`.
    UnknownEdition,
    /// `unknown_profile`.
    UnknownProfile,
    /// `unknown_wire`.
    UnknownWire,
    /// `unknown_required_feature`.
    UnknownRequiredFeature,
    /// `missing_import`.
    MissingImport,
    /// `missing_declaration`.
    MissingDeclaration,
    /// `ambiguous_declaration`.
    AmbiguousDeclaration,
    /// `invalid_package`.
    InvalidPackage,
    /// `invalid_model_binding`.
    InvalidModelBinding,
    /// `ill_typed`.
    IllTyped,
    /// `undefined_expression`.
    UndefinedExpression,
    /// `wrong_snapshot`.
    WrongSnapshot,
    /// `resource_exhausted`.
    ResourceExhausted,
    /// `unimplemented_capability`.
    UnimplementedCapability,
}

/// Closed V2 diagnostic cause tag.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(rename_all = "kebab-case")]
pub enum CheckedDiagnosticCause {
    /// `malformed-json`.
    MalformedJson,
    /// `missing-member`.
    MissingMember,
    /// `duplicate-member`.
    DuplicateMember,
    /// `unknown-member`.
    UnknownMember,
    /// `wrong-value-kind`.
    WrongValueKind,
    /// `invalid-value`.
    InvalidValue,
    /// `unsupported-wire`.
    UnsupportedWire,
    /// `unsupported-feature`.
    UnsupportedFeature,
    /// `revision-mismatch`.
    RevisionMismatch,
    /// `definition-cycle`.
    DefinitionCycle,
    /// `conflicting-definition`.
    ConflictingDefinition,
    /// `incompatible-definition`.
    IncompatibleDefinition,
    /// `feature-set-mismatch`.
    FeatureSetMismatch,
    /// `insufficient-next-charge`.
    InsufficientNextCharge,
}

/// One typed V2 diagnostic; never classified from message text.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedDiagnosticV2 {
    /// Pipeline stage.
    pub stage: CheckedDiagnosticStage,
    /// Closed diagnostic code.
    pub code: CheckedDiagnosticCode,
    /// Closed cause tag.
    pub cause_tag: CheckedDiagnosticCause,
    /// Typed semantic-term details.
    pub details: Vec<Value>,
    /// Source loci under locked sources.
    pub loci: Vec<CheckedSourceRegion>,
}

/// Typed V2 package diagnostics.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedDiagnosticsV2 {
    /// Diagnostic catalog definition.
    pub catalog: CheckedArtifactRef,
    /// Typed diagnostic records.
    pub entries: Vec<CheckedDiagnosticV2>,
}

macro_rules! encode_envelope_field {
    ($writer:expr, $name:expr, $value:expr, fixed) => {
        encode::member($writer, $name, $value)?
    };
    ($writer:expr, $name:expr, $value:expr, nested) => {
        encode::encoded($writer, $name, $value)?
    };
    ($writer:expr, $name:expr, $value:expr, elements) => {
        encode::elements($writer, $name, $value)?
    };
    ($writer:expr, $name:expr, $value:expr, optional) => {
        if let Some(value) = $value {
            encode::member($writer, $name, value)?;
        }
    };
    ($writer:expr, $name:expr, $value:expr, optional_elements) => {
        if let Some(value) = $value {
            encode::elements($writer, $name, value)?;
        }
    };
}

// One field list generates the top-level shape and its borrowed canonical
// event stream. The nested Value-bearing members use their iterative encoders.
macro_rules! checked_package_envelope {
    ($($name:ident: $parameter:ident => $encoding:ident),+ $(,)?) => {
        #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
        #[serde(deny_unknown_fields)]
        struct CheckedPackageEnvelope<C, N, L, G, S, R, A, D> {
            $($name: $parameter,)+
            #[serde(default, skip_serializing_if = "Option::is_none")]
            refinement_step_rows: Option<R>,
        }

        impl<Body: quire_canonical::Encode> quire_canonical::Encode for CheckedPackageEnvelope<
            &str,
            &str,
            &CheckedPackageLockV2,
            &CheckedSemanticGraphV2<Body>,
            &[CheckedSourceMapEntry],
            &[refinement::CheckedRefinementStepRowRegion],
            &[CheckedCapability],
            &CheckedDiagnosticsV2,
        > {
            fn encode_into<T: quire_canonical::Sink + ?Sized>(
                &self,
                writer: &mut quire_canonical::Writer<'_, T>,
            ) -> Result<(), quire_canonical::Error> {
                writer.begin_object()?;
                $(encode_envelope_field!(writer, stringify!($name), self.$name, $encoding);)+
                encode_envelope_field!(writer, "refinement_step_rows", self.refinement_step_rows, optional_elements);
                writer.end_object()
            }
        }
    };
}

checked_package_envelope! {
    contract_version: C => fixed,
    name: N => fixed,
    lock: L => fixed,
    semantic_graph: G => nested,
    source_map: S => elements,
    capability_report: A => fixed,
    diagnostics: D => nested,
}

type CheckedPackageWireV2<Body = CheckedTerm> = CheckedPackageEnvelope<
    Box<str>,
    Box<str>,
    CheckedPackageLockV2,
    CheckedSemanticGraphV2<Body>,
    Vec<CheckedSourceMapEntry>,
    Vec<refinement::CheckedRefinementStepRowRegion>,
    Vec<CheckedCapability>,
    CheckedDiagnosticsV2,
>;

type RawCheckedPackageWireV2 = CheckedPackageWireV2<Value>;

/// Source correspondence supplied to the canonical V2 writer.
#[derive(Clone, Copy, Debug)]
pub struct CheckedPackageSourceCorrespondenceRef<'a> {
    /// Node occurrence regions.
    pub source_map: &'a [CheckedSourceMapEntry],
    /// Canonical refinement step row regions, absent when no steps exist.
    pub refinement_step_rows: Option<&'a [CheckedRefinementStepRowRegion]>,
}

/// Canonically encodes a complete checked-package V2 envelope under the
/// caller's byte limits. The supplied name is the package's identity.
pub fn encode_checked_package_v2<Body: quire_canonical::Encode>(
    name: &str,
    lock: &CheckedPackageLockV2,
    semantic_graph: &CheckedSemanticGraphV2<Body>,
    sources: CheckedPackageSourceCorrespondenceRef<'_>,
    capability_report: &[CheckedCapability],
    diagnostics: &CheckedDiagnosticsV2,
    limits: quire_canonical::Limits,
) -> Result<Vec<u8>, quire_canonical::Error> {
    let envelope = CheckedPackageEnvelope {
        contract_version: CHECKED_PACKAGE_V2,
        name,
        lock,
        semantic_graph,
        source_map: sources.source_map,
        refinement_step_rows: sources.refinement_step_rows,
        capability_report,
        diagnostics,
    };
    quire_canonical::to_vec(&envelope, limits)
}

/// The closed result of reading untrusted bytes as V2.
// The required inline typed refusal makes this public result larger than its
// incomplete case; changing the variant to a box would change its API.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckedPackageV2ReadResult {
    /// A complete V2 package is safe for lowering.
    Admitted(Box<CheckedPackageV2>),
    /// The input is invalid for V2.
    Refused(CheckedPackageRefusal),
    /// The input exceeded a caller limit.
    Incomplete(CheckedPackageIncomplete),
}

/// Immutable, admitted `quire.checked-package/v2` data.
///
/// Every value an admitted package holds nests to a depth the closed body
/// grammar fixes, whatever the package's node count, so its clone,
/// comparison, `Debug` rendering, lowering and drop need no stack that follows
/// the package's size.
#[derive(Clone)]
pub struct CheckedPackageV2 {
    wire: CheckedPackageWireV2,
    kinds: Vec<CheckedNodeKind>,
    terms: Vec<CheckedTerm>,
    /// Admission-derived lookup; excluded from content equality.
    node_index: BTreeMap<CheckedNodeId, usize>,
    /// Admission-derived position of each exact source occurrence.
    pub(in crate::checked_package) source_index:
        BTreeMap<(CheckedNodeId, CheckedOccurrenceRole, u64), usize>,
    /// Body terms and resolved lowering edges, retained from graph admission.
    lowering_facts: Vec<lower::LoweringNodeFacts>,
    /// The byte limit this package was read under: the ceiling of every
    /// encode its lowering makes.
    bytes: u64,
    /// What the reader derived from the selected domain package documents and
    /// keeps: the declaration index, the field tables, the operation parameter
    /// types and the relationship records.
    models: RetainedModels,
    /// Direct admitted edges; their recursively derived closure is not copied.
    direct_dependencies: Vec<std::sync::Arc<CheckedPackageV2>>,
    /// Exact identities reachable through the direct edges, shared with children.
    closure_index: ClosureIndex,
}

impl std::fmt::Debug for CheckedPackageV2 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CheckedPackageV2")
            .field("wire", &self.wire)
            .field("kinds", &self.kinds)
            .field("node_index", &self.node_index)
            .field("source_index", &self.source_index)
            .field("bytes", &self.bytes)
            .field("models", &self.models)
            .field("direct_dependency_count", &self.direct_dependencies.len())
            .finish()
    }
}

impl Drop for CheckedPackageV2 {
    fn drop(&mut self) {
        let mut pending = std::mem::take(&mut self.direct_dependencies);
        while let Some(dependency) = pending.pop() {
            if let Ok(mut unique) = std::sync::Arc::try_unwrap(dependency) {
                pending.extend(std::mem::take(&mut unique.direct_dependencies));
            }
        }
    }
}

/// Equality is the equality of the admitted content. The read limit a package
/// retains for its lowering is not content, so two admissions of the same bytes
/// under different limits are equal; nor are the retained field tables,
/// declaration index, operation parameter types and relationship records, which
/// are derived from the same admitted documents.
impl PartialEq for CheckedPackageV2 {
    fn eq(&self, other: &Self) -> bool {
        self.wire == other.wire && self.kinds == other.kinds
    }
}

impl Eq for CheckedPackageV2 {}

fn retained_node_index(wire: &CheckedPackageWireV2) -> BTreeMap<CheckedNodeId, usize> {
    wire.semantic_graph
        .nodes
        .iter()
        .enumerate()
        .map(|(position, node)| (node.node_id.clone(), position))
        .collect()
}

fn retained_source_index(
    wire: &CheckedPackageWireV2,
) -> BTreeMap<(CheckedNodeId, CheckedOccurrenceRole, u64), usize> {
    super::selected_population::source_index(&wire.source_map)
}

/// Cumulative validation work against one caller limit.
#[derive(Clone, Copy, Debug)]
pub(super) struct WorkMeter {
    consumed: u64,
    limit: u64,
}

impl WorkMeter {
    pub(super) const fn new(limit: u64) -> Self {
        Self { consumed: 0, limit }
    }

    #[cfg(test)]
    pub(super) const fn consumed(&self) -> u64 {
        self.consumed
    }

    /// Charges `work` for the value at `at`, which an exhausted budget
    /// reports as the value whose charge failed.
    pub(super) fn charge(
        &mut self,
        work: u64,
        at: impl FnOnce() -> JsonPointer,
    ) -> Result<(), ValidationFailure> {
        self.consumed = self.consumed.saturating_add(work);
        if exceeds(self.consumed, self.limit) {
            Err(ValidationFailure::incomplete(
                CheckedPackageLimit::Work,
                self.limit,
                self.consumed,
                Some(at()),
            ))
        } else {
            Ok(())
        }
    }
}

fn refuse(code: CheckedPackageRefusalCode, path: JsonPointer) -> ValidationFailure {
    ValidationFailure::refused(code, path)
}

/// A refusal located at a specific graph node, carrying the cause this stage
/// determined (if any) and the node key of the offending entry or node.
fn refuse_at(
    code: CheckedPackageRefusalCode,
    path: JsonPointer,
    cause: Option<CheckedPackageRefusalCause>,
    locus: CheckedNodeId,
) -> ValidationFailure {
    ValidationFailure::refused_at(code, path, cause, locus)
}

/// The pointer of a fixed member path from the document root.
fn member_pointer(keys: &[&str]) -> JsonPointer {
    keys.iter()
        .fold(JsonPointer::root(), |pointer, key| pointer.key(key))
}

/// Members every `dependency_selections` entry requires.
const DEPENDENCY_SELECTION_MEMBERS: [&str; 1] = ["identity"];

/// An entry of `dependency_selections` that both lacks a required member and
/// carries a member outside the closed shape (a `Selection` or a
/// `DefinitionRef` where a `DependencySelection` belongs) is the wrong shape
/// as a whole: `malformed_wire` at the entry, not `unknown_member` at
/// whichever extra member the decoder met first. An entry carrying every
/// required member keeps the decoder's `unknown_member` at the extra member.
fn classify_dependency_entry_shape_bytes(
    failure: ValidationFailure,
    bytes: &[u8],
) -> ValidationFailure {
    classify_dependency_entry_shape_with(failure, |path| intake::source_value(bytes, path))
}

fn classify_dependency_entry_shape_with(
    failure: ValidationFailure,
    source: impl Fn(&JsonPointer) -> Option<Value>,
) -> ValidationFailure {
    let ValidationFailure::Refused(mut refusal) = failure else {
        return failure;
    };
    if refusal.code != CheckedPackageRefusalCode::UnknownMember {
        return ValidationFailure::Refused(refusal);
    }
    let entry = refusal.primary_path().and_then(|path| {
        let text = path.as_str();
        ["/lock/dependency_selections/"]
            .iter()
            .find_map(|prefix| text.strip_prefix(prefix))
            .and_then(|rest| rest.split_once('/'))
            .filter(|(index, member)| {
                !index.is_empty()
                    && index.bytes().all(|byte| byte.is_ascii_digit())
                    && !member.is_empty()
                    && !member.contains('/')
            })
            .and_then(|_| text.rsplit_once('/'))
            .map(|(entry, _)| entry.to_owned())
    });
    if let Some(entry) = entry {
        let lacks_member = JsonPointer::parse(&entry)
            .and_then(|pointer| source(&pointer))
            .and_then(|value| value.as_object().cloned())
            .is_some_and(|object| {
                DEPENDENCY_SELECTION_MEMBERS
                    .iter()
                    .any(|member| !object.contains_key(*member))
            });
        if lacks_member {
            refusal.code = CheckedPackageRefusalCode::MalformedWire;
            refusal.set_primary_path(JsonPointer::parse(&entry));
        }
    }
    ValidationFailure::Refused(refusal)
}

enum OwnerShapeMember {
    Owner,
    OwnerModel,
}

impl OwnerShapeMember {
    #[ix_trace_rs::string_edge]
    fn from_wire(text: &str) -> Option<Self> {
        match text {
            "owner" => Some(Self::Owner),
            "owner_model" => Some(Self::OwnerModel),
            _ => None,
        }
    }

    fn as_wire(&self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::OwnerModel => "owner_model",
        }
    }
}

/// Owner variants and the local-only clause model edge are closed wire shapes
/// whose unknown members are `malformed_wire` at the owner member.
fn classify_owner_shape(failure: ValidationFailure) -> ValidationFailure {
    let ValidationFailure::Refused(mut refusal) = failure else {
        return failure;
    };
    if refusal.code == CheckedPackageRefusalCode::UnknownMember {
        let owner_path = refusal.primary_path().and_then(|path| {
            let index = path.as_str().split('/').nth(3)?.parse::<usize>().ok()?;
            let prefix = node_pointer(index);
            let member_path = path
                .as_str()
                .strip_prefix(prefix.as_str())?
                .strip_prefix('/')?;
            let member = member_path.split('/').next()?;
            OwnerShapeMember::from_wire(member).map(|member| prefix.key(member.as_wire()))
        });
        if let Some(owner_path) = owner_path {
            refusal.code = CheckedPackageRefusalCode::MalformedWire;
            refusal.set_primary_path(Some(owner_path));
        }
    }
    ValidationFailure::Refused(refusal)
}

impl CheckedPackageV2 {
    /// The graph node with `id`, from the retained index.
    pub(in crate::checked_package) fn node(
        &self,
        id: &CheckedNodeId,
    ) -> Option<&CheckedSemanticNodeV2> {
        let position = *self.node_index.get(id)?;
        self.wire
            .semantic_graph
            .nodes
            .get(position)
            .inspect(|_| {
                #[cfg(test)]
                model_fields::probe::node_visited();
            })
            .filter(|candidate| &candidate.node_id == id)
    }

    /// Reads one canonical V2 value without exposing a partial package.
    pub fn read(
        bytes: &[u8],
        limits: CheckedPackageReadLimits,
        evidence: &CheckedPackageEvidence,
    ) -> CheckedPackageV2ReadResult {
        match Self::admit_bytes(bytes, limits, evidence) {
            Ok(package) => CheckedPackageV2ReadResult::Admitted(Box::new(package)),
            Err(stop) => stop.into_result(
                CheckedPackageV2ReadResult::Refused,
                CheckedPackageV2ReadResult::Incomplete,
            ),
        }
    }

    fn admit_bytes(
        bytes: &[u8],
        limits: CheckedPackageReadLimits,
        evidence: &CheckedPackageEvidence,
    ) -> Result<Self, ValidationFailure> {
        read_typed_prefix(bytes, limits)?;
        match intake::source_value(bytes, &member_pointer(&["contract_version"])) {
            Some(Value::String(version))
                if PackageWireTag::from_wire(&version) == Some(PackageWireTag::ContractVersion) => {
            }
            Some(Value::String(version)) => {
                return Err(ValidationFailure::unknown_contract_version(&version))
            }
            Some(_) => {
                return Err(refuse(
                    CheckedPackageRefusalCode::MalformedWire,
                    member_pointer(&["contract_version"]),
                ))
            }
            None => {
                return Err(refuse(
                    CheckedPackageRefusalCode::MalformedWire,
                    JsonPointer::root(),
                ))
            }
        }
        let wire = intake::decode_borrowed_wire(bytes)
            .map_err(|failure| classify_dependency_entry_shape_bytes(failure, bytes))
            .map_err(classify_owner_shape)?;
        intake::check_lossless_source(bytes, &wire)?;
        validate_owner_schema(&wire)?;
        let ValidatedPackage {
            kinds,
            terms,
            models,
            lowering_facts,
        } = validate(&wire, limits, evidence)?;
        let direct_dependencies = wire
            .lock
            .dependency_selections
            .iter()
            .enumerate()
            .map(|(index, selection)| {
                evidence
                    .retained_dependency(&selection.identity)
                    .ok_or_else(|| {
                        ValidationFailure::refused_because(
                            CheckedPackageRefusalCode::MissingImport,
                            member_pointer(&["lock", "dependency_selections"]).index(index),
                            CheckedPackageRefusalCause::MissingSelection,
                        )
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let closure_index =
            ClosureIndex::from_direct(&wire.lock.dependency_selections, &direct_dependencies);
        let CheckedPackageEnvelope {
            contract_version,
            name,
            lock,
            semantic_graph,
            source_map,
            refinement_step_rows,
            capability_report,
            diagnostics,
        } = wire;
        if semantic_graph.nodes.len() != terms.len() {
            return Err(refuse(
                CheckedPackageRefusalCode::MalformedWire,
                member_pointer(&["semantic_graph", "nodes"]),
            ));
        }
        let semantic_graph = CheckedSemanticGraphV2 {
            graph_version: semantic_graph.graph_version,
            nodes: semantic_graph
                .nodes
                .into_iter()
                .zip(terms.iter().cloned())
                .map(|(node, term)| node.into_typed(term))
                .collect(),
        };
        let wire = CheckedPackageEnvelope {
            contract_version,
            name,
            lock,
            semantic_graph,
            source_map,
            refinement_step_rows,
            capability_report,
            diagnostics,
        };
        let node_index = retained_node_index(&wire);
        let source_index = retained_source_index(&wire);
        Ok(Self {
            wire,
            kinds,
            terms,
            node_index,
            source_index,
            lowering_facts,
            bytes: limits.bytes,
            models,
            direct_dependencies,
            closure_index,
        })
    }

    /// The package's exact name identity.
    pub fn name(&self) -> &str {
        &self.wire.name
    }

    /// The exact immutable lock.
    pub fn lock(&self) -> &CheckedPackageLockV2 {
        &self.wire.lock
    }

    /// The complete closed checked graph.
    pub fn graph(&self) -> &CheckedSemanticGraphV2 {
        &self.wire.semantic_graph
    }

    /// Each graph node's decoded family and form, in graph order.
    pub fn node_kinds(&self) -> &[CheckedNodeKind] {
        &self.kinds
    }

    /// The retained admitted family and form of one graph node.
    pub(crate) fn node_kind(&self, id: &CheckedNodeId) -> Option<CheckedNodeKind> {
        self.kinds.get(*self.node_index.get(id)?).copied()
    }

    /// The typed body of an admitted node.
    pub fn term(&self, id: &CheckedNodeId) -> Option<&CheckedTerm> {
        self.terms.get(*self.node_index.get(id)?)
    }

    /// The exact source map.
    pub fn source_map(&self) -> &[CheckedSourceMapEntry] {
        &self.wire.source_map
    }

    /// Source regions of canonical refinement step rows, when steps exist.
    pub fn refinement_step_rows(&self) -> Option<&[CheckedRefinementStepRowRegion]> {
        self.wire.refinement_step_rows.as_deref()
    }

    /// The reported capabilities.
    pub fn capability_report(&self) -> &[CheckedCapability] {
        &self.wire.capability_report
    }

    /// The typed diagnostics.
    pub fn diagnostics(&self) -> &CheckedDiagnosticsV2 {
        &self.wire.diagnostics
    }
}

/// Checks the contract version and nonempty package name.
fn check_package_header(wire: &RawCheckedPackageWireV2) -> Result<(), ValidationFailure> {
    if PackageWireTag::from_wire(&wire.contract_version) != Some(PackageWireTag::ContractVersion) {
        return Err(ValidationFailure::unknown_contract_version(
            &wire.contract_version,
        ));
    }
    if !is_nonempty(&wire.name) {
        return Err(refuse(
            CheckedPackageRefusalCode::MalformedWire,
            member_pointer(&["name"]),
        ));
    }
    Ok(())
}

struct ValidatedPackage {
    kinds: Vec<CheckedNodeKind>,
    terms: Vec<CheckedTerm>,
    models: RetainedModels,
    lowering_facts: Vec<lower::LoweringNodeFacts>,
}

struct ValidatedGraph {
    kinds: Vec<CheckedNodeKind>,
    terms: Vec<CheckedTerm>,
    declarations: DeclarationIndex,
    lowering_facts: Vec<lower::LoweringNodeFacts>,
}

fn validate(
    wire: &RawCheckedPackageWireV2,
    limits: CheckedPackageReadLimits,
    evidence: &CheckedPackageEvidence,
) -> Result<ValidatedPackage, ValidationFailure> {
    check_package_header(wire)?;
    // The body grammar is strict wire validation: a body outside it refuses
    // here, before any identity is recomputed (FR-038-AC-116).
    flat_wire::check(wire)?;
    let mut meter = WorkMeter::new(limits.work);
    let LockAdmission {
        models,
        dependencies,
    } = validate_lock(wire, evidence, &mut meter, limits.bytes)?;
    let ValidatedGraph {
        kinds,
        terms,
        declarations,
        lowering_facts,
    } = validate_graph(wire, limits, &mut meter, &models, &dependencies, evidence)?;
    validate_source_map_entries(
        wire.semantic_graph
            .nodes
            .iter()
            .map(|node| (&node.node_id, node.occurrences.as_slice())),
        &wire.source_map,
        &wire.lock.sources,
        limits,
    )?;
    identity::validate_equivalence_occurrences(wire, &kinds, &mut meter)?;
    extension::validate_source_regions(wire, &kinds)?;
    validate_capabilities(wire, evidence)?;
    validate_diagnostics(wire, limits, &mut meter)?;
    Ok(ValidatedPackage {
        kinds,
        terms,
        models: RetainedModels::new(declarations, models),
        lowering_facts,
    })
}

/// What the lock stage admits: each selected domain package's declarations,
/// in lock order, and each selected dependency's supplied package.
struct LockAdmission<'a> {
    models: Vec<DomainModel>,
    dependencies: SuppliedDependencies<'a>,
}

/// Candidate unit references for model selection admission. The input byte
/// limit bounds this one graph walk and its index before graph work is charged.
/// The graph stage subsequently validates each node, name and key before the
/// package admits.
#[string_edge]
fn checked_unit_refs(
    nodes: &[RawSemanticNodeV2],
    meter: &mut WorkMeter,
) -> Result<UnitRefs, ValidationFailure> {
    let mut refs: BTreeMap<Box<str>, CheckedNodeId> = BTreeMap::new();
    for node in nodes {
        if CheckedNodeTag::from_wire(&node.node_tag)
            .and_then(|tag| CheckedNodeKind::decode(tag, &node.semantic_form))
            != Some(CheckedNodeKind::ScalarType(ScalarTypeForm::Unit))
        {
            continue;
        }
        let name = match (&node.owner, &node.declaration) {
            (Some(CheckedNodeOwner::Source { .. }), Some(declaration)) => {
                let declared = declaration
                    .qualified_name
                    .iter()
                    .map(AsRef::as_ref)
                    .collect::<Vec<&str>>()
                    .join("/");
                format!("ix://quire/unit/{declared}")
            }
            (
                Some(CheckedNodeOwner::Model {
                    identity,
                    node: owner_node,
                }),
                None,
            ) => {
                let prefix = format!("ix://{identity}/unit/");
                let Some(unit_name) = owner_node.strip_prefix(&prefix) else {
                    continue;
                };
                format!("ix://quire/unit/{identity}/{unit_name}")
            }
            _ => continue,
        };
        refs.entry(name.into())
            .or_insert_with(|| node.node_id.clone());
    }
    // A Duration member can use its owner's s only when that unit has the
    // exact one-second Time type. Keep the candidate's presence in `refs`
    // even when invalid: an invalid selected s must not fall back to source s.
    let candidates: Vec<_> = refs
        .keys()
        .filter(|name| name.ends_with("/s"))
        .cloned()
        .collect();
    for name in candidates {
        if let Some(unit_id) = identity::duration_unit_key(nodes, &name, meter)? {
            refs.insert(format!("{DURATION_RESOLVED_PREFIX}{name}").into(), unit_id);
        }
    }
    Ok(std::sync::Arc::new(refs))
}

/// Checks the lock and returns what it admits.
fn validate_lock<'a>(
    wire: &'a RawCheckedPackageWireV2,
    evidence: &'a CheckedPackageEvidence,
    meter: &mut WorkMeter,
    bytes: u64,
) -> Result<LockAdmission<'a>, ValidationFailure> {
    let lock = &wire.lock;
    if lock.sources.is_empty() {
        return Err(refuse(
            CheckedPackageRefusalCode::MalformedWire,
            member_pointer(&["lock", "sources"]),
        ));
    }
    for (index, source) in lock.sources.iter().enumerate() {
        validate_locked_source(source, &|| {
            member_pointer(&["lock", "sources"]).index(index)
        })?;
    }
    validate_definition_ref(&lock.edition.definition, &|| {
        member_pointer(&["lock", "edition", "definition"])
    })?;
    for (index, selection) in lock.profile_selections.iter().enumerate() {
        validate_definition_ref(&selection.definition, &|| {
            member_pointer(&["lock", "profile_selections"])
                .index(index)
                .key("definition")
        })?;
    }
    validate_dependency_selections(&lock.dependency_selections)?;
    validate_refinement_internals(lock.refinement_internals.as_deref())?;
    for (index, definition) in lock.definition_selections.iter().enumerate() {
        validate_definition_ref(definition, &|| {
            member_pointer(&["lock", "definition_selections"]).index(index)
        })?;
    }
    let roots = selected_model_roots(&wire.semantic_graph.nodes)?;
    let unit_refs = if roots.is_empty() {
        std::sync::Arc::new(BTreeMap::new())
    } else {
        checked_unit_refs(&wire.semantic_graph.nodes, meter)?
    };
    let mut models =
        validate_domain_packages(&roots, &wire.name, evidence, meter, bytes, &unit_refs)?;
    if !models.is_empty() {
        let mut resolved = (*unit_refs).clone();
        for model in &models {
            let name = format!("ix://quire/unit/{}/s", model.identity);
            if resolved.contains_key(name.as_str()) {
                if let Some(unit_id) =
                    identity::model_duration_unit_key(&wire.semantic_graph.nodes, model, meter)?
                {
                    resolved.insert(format!("{DURATION_RESOLVED_PREFIX}{name}").into(), unit_id);
                }
            }
        }
        let resolved = std::sync::Arc::new(resolved);
        for model in &mut models {
            model.unit_refs = resolved.clone();
        }
    }
    // Implements: FR-322. Every selected dependency's admitted package is supplied and
    // binds to its entry, before any `dependency_reference` is read.
    let dependencies = admit_dependencies(&lock.dependency_selections, evidence, meter)?;
    check_derived_closure(
        &lock.dependency_selections,
        &dependencies.packages(),
        JsonPointer::root(),
        meter,
    )?;
    let mut features = BTreeSet::new();
    if let Some(index) = lock
        .required_features
        .iter()
        .position(|feature| !(is_nonempty(feature) && features.insert(feature)))
    {
        return Err(refuse(
            CheckedPackageRefusalCode::MalformedWire,
            member_pointer(&["lock", "required_features"]).index(index),
        ));
    }
    validate_definition_ref(&wire.diagnostics.catalog, &|| {
        member_pointer(&["diagnostics", "catalog"])
    })?;
    // IR-628: the last act of the lock stage. Every row's step 1 has admitted,
    // so a step 1 refusal at any row outranks a table charge at an earlier one;
    // the tables precede the graph stage, so an exhausted limit outranks a
    // graph refusal.
    build_field_tables(&mut models, meter)?;
    Ok(LockAdmission {
        models,
        dependencies,
    })
}

/// Checks `lock.dependency_selections` (FR-322): nonempty identity,
/// uniqueness, then strictly ascending
/// UTF-8 byte order of `identity`. Each condition sweeps the whole array
/// before the next, so the refusal is decided by defect class and not by
/// array position. A repeated identity refuses `invalid_package` /
/// `conflicting-definition` at the repeating entry, the later of the two
/// entries FR-322 names as loci; an entry not strictly after its
/// predecessor refuses `invalid_package` / `invalid-value` at that entry.
/// Only the lock carries selections.
fn validate_dependency_selections(
    selections: &[CheckedDependencySelection],
) -> Result<(), ValidationFailure> {
    let at = |index: usize| member_pointer(&["lock", "dependency_selections"]).index(index);
    let malformed = selections.iter().enumerate().find_map(|(index, entry)| {
        if !is_nonempty(&entry.identity) {
            Some(at(index).key("identity"))
        } else {
            None
        }
    });
    if let Some(path) = malformed {
        return Err(refuse(CheckedPackageRefusalCode::MalformedWire, path));
    }
    let mut identities = BTreeMap::new();
    for (index, entry) in selections.iter().enumerate() {
        if let Some(&first) = identities.get(entry.identity.as_ref()) {
            return Err(ValidationFailure::refused_because_at_two_paths(
                CheckedPackageRefusalCode::InvalidPackage,
                at(first),
                at(index),
                CheckedPackageRefusalCause::ConflictingDefinition,
            ));
        }
        identities.insert(entry.identity.as_ref(), index);
    }
    if let Some(index) = selections
        .windows(2)
        .position(|pair| pair[0].identity >= pair[1].identity)
    {
        return Err(ValidationFailure::refused_because(
            CheckedPackageRefusalCode::InvalidPackage,
            at(index + 1),
            CheckedPackageRefusalCause::InvalidValue,
        ));
    }
    Ok(())
}

/// Enforce FR-322's canonical refinement internal lock rows and spellings.
fn validate_refinement_internals(
    rows: Option<&[CheckedRefinementInternal]>,
) -> Result<(), ValidationFailure> {
    let Some(rows) = rows else { return Ok(()) };
    let at = |index: usize| member_pointer(&["lock", "refinement_internals"]).index(index);
    if rows.is_empty() {
        return Err(refuse(
            CheckedPackageRefusalCode::MalformedWire,
            member_pointer(&["lock", "refinement_internals"]),
        ));
    }
    let mut seen_rows = BTreeMap::new();
    for (index, row) in rows.iter().enumerate() {
        if !is_nonempty(&row.refinement) || row.internal.is_empty() {
            return Err(refuse(CheckedPackageRefusalCode::MalformedWire, at(index)));
        }
        if let Some(first) = seen_rows.insert(row.refinement.as_ref(), index) {
            return Err(ValidationFailure::refused_because_at_two_paths(
                CheckedPackageRefusalCode::InvalidPackage,
                at(first).key("refinement"),
                at(index).key("refinement"),
                CheckedPackageRefusalCause::ConflictingDefinition,
            ));
        }
        let mut seen_classes = BTreeMap::new();
        for (class_index, class) in row.internal.iter().enumerate() {
            let class_at = || at(index).key("internal").index(class_index);
            if !is_nonempty(class) {
                return Err(refuse(CheckedPackageRefusalCode::MalformedWire, class_at()));
            }
            if let Some(first) = seen_classes.insert(class.as_ref(), class_index) {
                return Err(ValidationFailure::refused_because_at_two_paths(
                    CheckedPackageRefusalCode::InvalidPackage,
                    at(index).key("internal").index(first),
                    class_at(),
                    CheckedPackageRefusalCause::ConflictingDefinition,
                ));
            }
            if class_index > 0 && row.internal[class_index - 1] >= *class {
                return Err(ValidationFailure::refused_because(
                    CheckedPackageRefusalCode::InvalidPackage,
                    class_at(),
                    CheckedPackageRefusalCause::InvalidValue,
                ));
            }
        }
        if index > 0 && rows[index - 1].refinement >= row.refinement {
            return Err(ValidationFailure::refused_because(
                CheckedPackageRefusalCode::InvalidPackage,
                at(index).key("refinement"),
                CheckedPackageRefusalCause::InvalidValue,
            ));
        }
    }
    Ok(())
}

/// The selected domain packages: one per `model_root` node owner, in
/// ascending identity order. Each is the node position of its root. Two roots
/// naming one identity refuse as an invalid graph at the later root's owner.
fn selected_model_roots(
    nodes: &[RawSemanticNodeV2],
) -> Result<Vec<(Box<str>, usize)>, ValidationFailure> {
    let mut roots: BTreeMap<Box<str>, usize> = BTreeMap::new();
    for (position, node) in nodes.iter().enumerate() {
        if let Some(CheckedNodeOwner::ModelRoot { identity }) = &node.owner {
            if roots.insert(identity.clone(), position).is_some() {
                return Err(refuse(
                    CheckedPackageRefusalCode::InvalidSemanticGraph,
                    node_pointer(position).key("owner"),
                ));
            }
        }
    }
    Ok(roots.into_iter().collect())
}

/// Admits the document supplied under each selected domain package identity
/// (FR-322 "Model-owned members" step 1) and returns its declarations, in
/// ascending identity order. Each document is charged to `meter` at its
/// root's owner (`/semantic_graph/nodes/<i>/owner`), and an exhausted `work`
/// limit is `incomplete` there.
fn validate_domain_packages(
    roots: &[(Box<str>, usize)],
    checked_package: &str,
    evidence: &CheckedPackageEvidence,
    meter: &mut WorkMeter,
    bytes: u64,
    unit_refs: &UnitRefs,
) -> Result<Vec<DomainModel>, ValidationFailure> {
    let row = |root: usize| node_pointer(root).key("owner");
    let at = |root: usize, member: &str| row(root).key(member);
    roots
        .iter()
        .map(|(identity, root)| {
            let root = *root;
            let mut budget = Budget::new(meter, root, bytes);
            admit_selection(identity, checked_package, evidence, &mut budget, unit_refs).map_err(
                |failure| {
                    match failure {
                        SelectionFailure::Limit(failure) => failure,
                        SelectionFailure::InexactNumber {
                            document_pointer,
                            cause,
                        } => ValidationFailure::refused_number_in_document(
                            at(root, "identity"),
                            document_pointer,
                            cause,
                        ),
                        SelectionFailure::Refused(refused) => {
                            let model_members::SelectionRefusal {
                                refusal,
                                member,
                                declaration_identity,
                                declaration_origin,
                                declaration_path,
                                document_pointer,
                            } = *refused;
                            // Intake metadata is private. Its owning lifetime ends at
                            // this projection; none is attached to the public refusal.
                            drop((declaration_identity, declaration_origin, declaration_path));
                            let path = member.map_or_else(|| row(root), |member| at(root, member));
                            if let Some(document_pointer) = document_pointer {
                                ValidationFailure::refused_because_in_document(
                                    refusal.code,
                                    path,
                                    refusal.cause,
                                    document_pointer,
                                )
                            } else {
                                ValidationFailure::refused_because(
                                    refusal.code,
                                    path,
                                    refusal.cause,
                                )
                            }
                        }
                    }
                },
            )
        })
        .collect()
}

/// Checks a structural node identity and all nested operand identities.
fn validate_node_id(
    id: &CheckedNodeId,
    at: impl FnOnce() -> JsonPointer,
) -> Result<(), ValidationFailure> {
    const MAX_ORDINAL: u64 = 9_007_199_254_740_991;
    let mut pending = vec![(id, at())];
    while let Some((candidate, pointer)) = pending.pop() {
        if !is_nonempty(&candidate.package) {
            return Err(refuse(
                CheckedPackageRefusalCode::MalformedWire,
                pointer.key("package"),
            ));
        }
        if candidate.path.is_empty() {
            return Err(refuse(
                CheckedPackageRefusalCode::MalformedWire,
                pointer.key("path"),
            ));
        }
        if candidate.ordinal > MAX_ORDINAL {
            return Err(refuse(
                CheckedPackageRefusalCode::MalformedWire,
                pointer.key("ordinal"),
            ));
        }
        for (index, segment) in candidate.path.iter().enumerate().rev() {
            if let CheckedNodePathSegment::Node(child) = segment {
                pending.push((child.as_ref(), pointer.clone().key("path").index(index)));
            }
        }
    }
    Ok(())
}

/// Enforces the schema's `DeclarationTagRules` and `DeclarationOccurrenceRule`
/// (FR-208): `declaration` is forbidden on `expression`, `relation`, `state`,
/// `temporal` and `correspondence` nodes and on `value`/`enum_value` nodes;
/// for every other family it is present exactly when the node carries a
/// `declaration`-role occurrence.
fn validate_declaration(
    kind: CheckedNodeKind,
    occurrences: &[CheckedOccurrence],
    declaration: Option<&CheckedDeclaration>,
    position: usize,
) -> Result<(), ValidationFailure> {
    let forced_absent = declaration_forbidden(kind);
    let required = !forced_absent
        && occurrences
            .iter()
            .any(|occurrence| occurrence.role == CheckedOccurrenceRole::Declaration);
    match (required, declaration) {
        (true, None) => Err(refuse(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            node_pointer(position),
        )),
        (false, Some(_)) => Err(refuse(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            node_pointer(position).key("declaration"),
        )),
        (true, Some(declaration)) => {
            identity::validate_qualified_name(&declaration.qualified_name, &|| {
                node_pointer(position)
                    .key("declaration")
                    .key("qualified_name")
            })
        }
        (false, None) => Ok(()),
    }
}

/// `DeclarationTagRules`: whether a node of this kind may never carry a
/// `declaration`. The rule is stated per family, except `value`/`enum_value`,
/// but every form is listed so a new form is a compile error here until its
/// rule is decided.
fn declaration_forbidden(kind: CheckedNodeKind) -> bool {
    use CheckedNodeKind as K;
    match kind {
        K::Extension(_) => false,
        K::TimedModel(form) => form != TimedModelForm::Model,
        K::Hybrid(HybridForm::Model | HybridForm::Mode) => false,
        K::Hybrid(HybridForm::Transition) => true,
        K::ScalarType(
            ScalarTypeForm::Boolean
            | ScalarTypeForm::Integer
            | ScalarTypeForm::Rational
            | ScalarTypeForm::Decimal
            | ScalarTypeForm::Float32
            | ScalarTypeForm::Float64
            | ScalarTypeForm::Text
            | ScalarTypeForm::String
            | ScalarTypeForm::Uuid
            | ScalarTypeForm::Timestamp
            | ScalarTypeForm::Dimension
            | ScalarTypeForm::DimensionEquivalence
            | ScalarTypeForm::Unit
            | ScalarTypeForm::Enum,
        ) => false,
        // QSL FR-094: a compound unit is anonymous and never declares.
        K::ScalarType(ScalarTypeForm::CompoundUnit) => true,
        K::CompositeType(
            CompositeTypeForm::Option
            | CompositeTypeForm::Sequence
            | CompositeTypeForm::Set
            | CompositeTypeForm::Bag
            | CompositeTypeForm::OrderedSet
            | CompositeTypeForm::Record
            | CompositeTypeForm::Tuple
            | CompositeTypeForm::Union
            | CompositeTypeForm::Alias
            | CompositeTypeForm::Reference,
        ) => false,
        K::BoundedDomain(
            BoundedDomainForm::IntegerRange
            | BoundedDomainForm::RationalRange
            | BoundedDomainForm::DecimalRange
            | BoundedDomainForm::FloatRounding
            | BoundedDomainForm::TextBounds
            | BoundedDomainForm::CollectionBounds
            | BoundedDomainForm::ModelPopulation,
        ) => false,
        // QSL FR-092: a parameter names its binder's value; it declares no
        // name of its own. QSpec FR-440: a union value carries no
        // `declaration` (its union type node is the source-declared one).
        K::Value(ValueForm::EnumValue | ValueForm::Parameter | ValueForm::UnionValue) => true,
        K::Value(
            ValueForm::Literal
            | ValueForm::CollectionValue
            | ValueForm::RecordValue
            | ValueForm::TupleValue
            | ValueForm::OptionValue,
        ) => false,
        K::Expression(
            ExpressionForm::Reference
            | ExpressionForm::Call
            | ExpressionForm::Unary
            | ExpressionForm::Binary
            | ExpressionForm::Conditional
            | ExpressionForm::Case
            | ExpressionForm::Let
            | ExpressionForm::Quantify
            | ExpressionForm::Collection
            | ExpressionForm::Conversion
            | ExpressionForm::Query
            | ExpressionForm::PreRead
            | ExpressionForm::PresenceRead
            | ExpressionForm::ValueRead
            | ExpressionForm::Deref
            | ExpressionForm::Reachability,
        ) => true,
        K::Function(
            FunctionForm::PureFunction | FunctionForm::Predicate | FunctionForm::RecursiveFunction,
        ) => false,
        K::Model(ModelForm::ModelRoot) => true,
        K::Model(
            ModelForm::ModelImport
            | ModelForm::ObjectType
            | ModelForm::ValueType
            | ModelForm::VariantType
            | ModelForm::TupleValueType
            | ModelForm::RecordValueType
            | ModelForm::EventType
            | ModelForm::StateMachine
            | ModelForm::Process
            | ModelForm::PersistenceInterface
            | ModelForm::Namespace
            | ModelForm::SystemsInterface
            | ModelForm::SystemsPart
            | ModelForm::SystemsPort
            | ModelForm::SystemsConnection
            | ModelForm::SystemsAllocation,
        ) => false,
        K::Relation(
            RelationForm::Relationship
            | RelationForm::Population
            | RelationForm::Membership
            | RelationForm::CausalRelation
            | RelationForm::Refinement,
        ) => true,
        K::State(
            StateForm::StateClause
            | StateForm::Frame
            | StateForm::Transition
            | StateForm::OperationAnchor
            | StateForm::Snapshot,
        ) => true,
        K::Temporal(
            TemporalForm::TemporalClause
            | TemporalForm::Formula
            | TemporalForm::Fairness
            | TemporalForm::Clock
            | TemporalForm::Window
            | TemporalForm::Activation
            | TemporalForm::Deadline
            | TemporalForm::Terminal,
        ) => true,
        K::Protocol(ProtocolForm::ProtocolReference) => true,
        K::Protocol(
            ProtocolForm::ProtocolClause
            | ProtocolForm::Role
            | ProtocolForm::Channel
            | ProtocolForm::Queue
            | ProtocolForm::Control
            | ProtocolForm::Obligation
            | ProtocolForm::Compensation,
        ) => false,
        K::Claim(ClaimForm::StateGraph) => true,
        K::Claim(ClaimForm::StepRelation) => false,
        K::Claim(
            ClaimForm::VerificationClaim
            | ClaimForm::AnalysisClaim
            | ClaimForm::Hyperproperty
            | ClaimForm::SynthesisRequest,
        ) => false,
        K::Correspondence(
            CorrespondenceForm::SourceLocus
            | CorrespondenceForm::ModelCorrespondence
            | CorrespondenceForm::BindingRole
            | CorrespondenceForm::ProfileCorrespondence
            | CorrespondenceForm::AbstractionRelation,
        ) => true,
    }
}

/// Retain an anchor's bare point through its dedicated body decoder.
fn decode_admitted_body(
    kind: CheckedNodeKind,
    body: &Value,
) -> Result<CheckedTerm, typed_terms::TermConversionCause> {
    if source_nominal_body(kind, body) {
        identity::NominalBody::from_value(body, JsonPointer::root())
            .map(|body| CheckedTerm::NominalBody(Box::new(body)))
            .map_err(|_| typed_terms::TermConversionCause::MalformedMember {
                member: "body",
                category: serde_json::error::Category::Data,
            })
    } else if kind == CheckedNodeKind::State(StateForm::Frame) {
        frame::read_frame_body(body, &Trail::Base(&[]))
            .map(|(frame, _)| frame.into_checked_term())
            .map_err(|_| typed_terms::TermConversionCause::MalformedMember {
                member: "body",
                category: serde_json::error::Category::Data,
            })
    } else if kind == CheckedNodeKind::State(StateForm::OperationAnchor) {
        execution_point::decode_anchor_body(body)
    } else {
        CheckedTerm::decode_admitted(body)
    }
}

fn source_nominal_body(kind: CheckedNodeKind, body: &Value) -> bool {
    matches!(
        (kind, body.get("term").and_then(Value::as_str)),
        (
            CheckedNodeKind::ScalarType(ScalarTypeForm::Enum),
            Some("enum")
        ) | (
            CheckedNodeKind::ScalarType(ScalarTypeForm::Dimension),
            Some("dimension")
        ) | (
            CheckedNodeKind::ScalarType(ScalarTypeForm::Unit),
            Some("unit")
        ) | (
            CheckedNodeKind::ScalarType(ScalarTypeForm::DimensionEquivalence),
            Some("dimension_equivalence")
        )
    )
}

/// Minimal structural admission for one node body. Most nodes use the closed
/// `SemanticTerm` grammar. Extension nodes use [`extension::read_body`],
/// `state`/`frame` nodes use [`frame::read_frame_body`], and
/// `correspondence`/`abstraction_relation` nodes are scanned for applications
/// by [`abstraction::scan_body`] and read at the abstraction step. The graph
/// loop collects extension clause targets directly. Frame and abstraction bodies report no
/// reference targets to the caller: its entries are joined to the node's
/// `dependencies` by the frame and abstraction steps
/// ([`frame::validate_frame_semantics`],
/// [`abstraction::validate_abstraction`]), never through the generic
/// reference edges `validate_graph` resolves for every other body.
/// `at` is the body's own position, `/semantic_graph/nodes/{n}/body`.
fn validate_body(
    kind: CheckedNodeKind,
    body: &Value,
    at: &Trail<'_>,
    visit: &mut ReferenceVisitor<'_>,
) -> Result<u64, ValidationFailure> {
    #[cfg(test)]
    lower::BODY_WALKS.with(|walks| walks.set(walks.get() + 1));
    if source_nominal_body(kind, body) {
        let nominal = identity::NominalBody::from_value(body, at.pointer())?;
        let site = ReferenceSite {
            member: ReferenceMember::Target,
            is_body_root: false,
        };
        match &nominal {
            identity::NominalBody::Enum { .. } => {}
            identity::NominalBody::Dimension { terms } => {
                for (index, term) in terms.iter().enumerate() {
                    visit(
                        &term.dimension_node_id,
                        site,
                        &at.key("terms").index(index).key("dimension_node_id"),
                    );
                }
            }
            identity::NominalBody::Unit {
                dimension_node_id,
                target_unit_node_id,
                ..
            } => {
                visit(dimension_node_id, site, &at.key("dimension_node_id"));
                if let Some(target) = target_unit_node_id {
                    visit(target, site, &at.key("target_unit_node_id"));
                }
            }
            identity::NominalBody::DimensionEquivalence {
                dimension_node_ids, ..
            } => {
                for (index, id) in dimension_node_ids.iter().enumerate() {
                    visit(id, site, &at.key("dimension_node_ids").index(index));
                }
            }
        }
        return Ok(1 + match nominal {
            identity::NominalBody::Dimension { terms } => terms.len() as u64,
            identity::NominalBody::DimensionEquivalence { .. } => 2,
            identity::NominalBody::Unit { .. } => 2,
            identity::NominalBody::Enum { members, .. } => members.len() as u64,
        });
    }
    if kind == CheckedNodeKind::Protocol(ProtocolForm::ProtocolReference)
        && !body.as_object().is_some_and(|object| {
            super::common::exact_members(object, &["term", "target"])
                && super::common::body_term(body) == Some(BodyTerm::Reference)
        })
    {
        return Err(refuse(
            CheckedPackageRefusalCode::MalformedWire,
            at.pointer(),
        ));
    }
    match body_grammar(kind) {
        BodyGrammar::Term if kind == CheckedNodeKind::State(StateForm::OperationAnchor) => {
            execution_point::validate_anchor_body(body, at, visit)
        }
        BodyGrammar::Extension => {
            extension::read_body(body, at.pointer()).map(|body| extension::body_work(&body))
        }
        BodyGrammar::Frame => frame::read_frame_body(body, at).map(|(_, work)| work),
        BodyGrammar::AbstractionRelation => abstraction::scan_body(body, at, visit),
        BodyGrammar::Refinement => refinement::validate_body(body, at, visit),
        // `body` is the node's own top-level term, never a nested one, so
        // this is the one call in the module that reports `is_body_root: true`.
        BodyGrammar::Term | BodyGrammar::TimedModel | BodyGrammar::Hybrid => {
            validate_term(body, TermGrammar::V2, true, at, visit)
        }
    }
}

/// The grammar a node's `body` is validated against (QSpec FR-322
/// `BodyBindingRules`): extension nodes use FR-469's declaration body,
/// `state`/`frame` nodes use the frame reference triple,
/// `correspondence`/`abstraction_relation` nodes use FR-346's flat shape,
/// and every other node's body is a `SemanticTerm`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BodyGrammar {
    /// The closed extension declaration body.
    Extension,
    /// The closed `SemanticTerm` grammar.
    Term,
    /// The frame body of FR-040.
    Frame,
    /// The abstraction relation body of FR-346.
    AbstractionRelation,
    /// The closed refinement body of FR-375.
    Refinement,
    /// The closed nested hybrid model, mode or transition shape.
    Hybrid,
    /// Closed timed-model bodies.
    TimedModel,
}

/// Exactly one kind has each of the three flat grammars. Every form is listed,
/// so a new form is a compile error until its grammar is decided.
fn body_grammar(kind: CheckedNodeKind) -> BodyGrammar {
    use CheckedNodeKind as K;
    match kind {
        K::Extension(_) => BodyGrammar::Extension,
        K::Hybrid(_) => BodyGrammar::Hybrid,
        K::TimedModel(_) => BodyGrammar::TimedModel,
        K::ScalarType(
            ScalarTypeForm::Boolean
            | ScalarTypeForm::Integer
            | ScalarTypeForm::Rational
            | ScalarTypeForm::Decimal
            | ScalarTypeForm::Float32
            | ScalarTypeForm::Float64
            | ScalarTypeForm::Text
            | ScalarTypeForm::String
            | ScalarTypeForm::Uuid
            | ScalarTypeForm::Timestamp
            | ScalarTypeForm::Dimension
            | ScalarTypeForm::DimensionEquivalence
            | ScalarTypeForm::Unit
            | ScalarTypeForm::Enum
            | ScalarTypeForm::CompoundUnit,
        ) => BodyGrammar::Term,
        K::CompositeType(
            CompositeTypeForm::Option
            | CompositeTypeForm::Sequence
            | CompositeTypeForm::Set
            | CompositeTypeForm::Bag
            | CompositeTypeForm::OrderedSet
            | CompositeTypeForm::Record
            | CompositeTypeForm::Tuple
            | CompositeTypeForm::Union
            | CompositeTypeForm::Alias
            | CompositeTypeForm::Reference,
        ) => BodyGrammar::Term,
        K::BoundedDomain(
            BoundedDomainForm::IntegerRange
            | BoundedDomainForm::RationalRange
            | BoundedDomainForm::DecimalRange
            | BoundedDomainForm::FloatRounding
            | BoundedDomainForm::TextBounds
            | BoundedDomainForm::CollectionBounds
            | BoundedDomainForm::ModelPopulation,
        ) => BodyGrammar::Term,
        K::Value(
            ValueForm::Literal
            | ValueForm::EnumValue
            | ValueForm::CollectionValue
            | ValueForm::RecordValue
            | ValueForm::TupleValue
            | ValueForm::UnionValue
            | ValueForm::OptionValue
            | ValueForm::Parameter,
        ) => BodyGrammar::Term,
        K::Expression(
            ExpressionForm::Reference
            | ExpressionForm::Call
            | ExpressionForm::Unary
            | ExpressionForm::Binary
            | ExpressionForm::Conditional
            | ExpressionForm::Case
            | ExpressionForm::Let
            | ExpressionForm::Quantify
            | ExpressionForm::Collection
            | ExpressionForm::Conversion
            | ExpressionForm::Query
            | ExpressionForm::PreRead
            | ExpressionForm::PresenceRead
            | ExpressionForm::ValueRead
            | ExpressionForm::Deref
            | ExpressionForm::Reachability,
        ) => BodyGrammar::Term,
        K::Function(
            FunctionForm::PureFunction | FunctionForm::Predicate | FunctionForm::RecursiveFunction,
        ) => BodyGrammar::Term,
        K::Model(
            ModelForm::ModelRoot
            | ModelForm::ModelImport
            | ModelForm::ObjectType
            | ModelForm::ValueType
            | ModelForm::VariantType
            | ModelForm::TupleValueType
            | ModelForm::RecordValueType
            | ModelForm::EventType
            | ModelForm::StateMachine
            | ModelForm::Process
            | ModelForm::PersistenceInterface
            | ModelForm::Namespace
            | ModelForm::SystemsInterface
            | ModelForm::SystemsPart
            | ModelForm::SystemsPort
            | ModelForm::SystemsConnection
            | ModelForm::SystemsAllocation,
        ) => BodyGrammar::Term,
        K::Relation(
            RelationForm::Relationship
            | RelationForm::Population
            | RelationForm::Membership
            | RelationForm::CausalRelation,
        ) => BodyGrammar::Term,
        K::Relation(RelationForm::Refinement) => BodyGrammar::Refinement,
        K::State(StateForm::Frame) => BodyGrammar::Frame,
        K::State(
            StateForm::StateClause
            | StateForm::Transition
            | StateForm::OperationAnchor
            | StateForm::Snapshot,
        ) => BodyGrammar::Term,
        K::Temporal(
            TemporalForm::TemporalClause
            | TemporalForm::Formula
            | TemporalForm::Fairness
            | TemporalForm::Clock
            | TemporalForm::Window
            | TemporalForm::Activation
            | TemporalForm::Deadline
            | TemporalForm::Terminal,
        ) => BodyGrammar::Term,
        K::Protocol(
            ProtocolForm::ProtocolClause
            | ProtocolForm::ProtocolReference
            | ProtocolForm::Role
            | ProtocolForm::Channel
            | ProtocolForm::Queue
            | ProtocolForm::Control
            | ProtocolForm::Obligation
            | ProtocolForm::Compensation,
        ) => BodyGrammar::Term,
        K::Claim(
            ClaimForm::VerificationClaim
            | ClaimForm::AnalysisClaim
            | ClaimForm::Hyperproperty
            | ClaimForm::SynthesisRequest
            | ClaimForm::StateGraph
            | ClaimForm::StepRelation,
        ) => BodyGrammar::Term,
        K::Correspondence(CorrespondenceForm::AbstractionRelation) => {
            BodyGrammar::AbstractionRelation
        }
        K::Correspondence(
            CorrespondenceForm::SourceLocus
            | CorrespondenceForm::ModelCorrespondence
            | CorrespondenceForm::BindingRole
            | CorrespondenceForm::ProfileCorrespondence,
        ) => BodyGrammar::Term,
    }
}

/// FR-322's declaration-name refusals, after the nominal checks and before
/// frame and operation checks. Declared paths equal their assigned node paths.
/// Non-protocol declarations share separate unit, dimension and other-type
/// namespaces. A model-owned unit also occupies its package-qualified unit
/// name, so a source unit with the same type reference is ambiguous. The
/// later structural node id is refused. Protocol duplicates run in their own
/// later stage. Names were charged by the preceding graph walk.
fn validate_declaration_names(
    nodes: &[RawSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &BTreeMap<&CheckedNodeId, usize>,
) -> Result<(), ValidationFailure> {
    let name_pointer = |position: usize| {
        node_pointer(position)
            .key("declaration")
            .key("qualified_name")
    };
    for (&node_id, &position) in index {
        let node = &nodes[position];
        let Some(declaration) = &node.declaration else {
            continue;
        };
        let path_matches = node.node_id.path.len() == declaration.qualified_name.len()
            && node
                .node_id
                .path
                .iter()
                .zip(&declaration.qualified_name)
                .all(|(part, name)| matches!(part, CheckedNodePathSegment::Name(value) if value == name));
        if !path_matches {
            return Err(refuse_at(
                CheckedPackageRefusalCode::InvalidPackage,
                node_pointer(position).key("node_id").key("path"),
                Some(CheckedPackageRefusalCause::DeclarationNominalMismatch),
                node_id.clone(),
            ));
        }
    }
    let mut holders = BTreeSet::new();
    for (&node_id, &position) in index {
        if matches!(kinds.get(position), Some(CheckedNodeKind::Protocol(_))) {
            continue;
        }
        let node = &nodes[position];
        let namespace = match (node.node_tag.as_ref(), node.semantic_form.as_ref()) {
            ("scalar_type", "dimension") => 0_u8,
            ("scalar_type", "unit") => 1,
            _ => 2,
        };
        let name = match (&node.declaration, &node.owner, namespace) {
            (Some(declaration), _, _) => declaration
                .qualified_name
                .iter()
                .map(AsRef::as_ref)
                .collect::<Vec<&str>>()
                .join("/"),
            (
                None,
                Some(CheckedNodeOwner::Model {
                    identity,
                    node: owner_node,
                }),
                1,
            ) => {
                let prefix = format!("ix://{identity}/unit/");
                let Some(unit_name) = owner_node.strip_prefix(&prefix) else {
                    continue;
                };
                format!("{identity}/{unit_name}")
            }
            _ => continue,
        };
        if !holders.insert((namespace, name)) {
            let path = if node.declaration.is_some() {
                name_pointer(position)
            } else {
                node_pointer(position)
            };
            return Err(refuse_at(
                CheckedPackageRefusalCode::AmbiguousDeclaration,
                path,
                Some(CheckedPackageRefusalCause::AmbiguousName),
                node_id.clone(),
            ));
        }
    }
    Ok(())
}

/// Which member of a node supplied one graph edge, so a charge on that edge
/// can name the value that carried it.
#[derive(Clone, Copy, Debug)]
enum EdgeSite {
    SemanticType,
    Dependency(usize),
    /// The body reference at this index of the node's recorded references.
    Reference(usize),
}

/// One body reference target and the site that carried it. Its pointer is
/// built only on refusal, by [`body_reference_pointer`].
type BodyReference = (CheckedNodeId, ReferenceSite);

/// Locates the two cause-bearing refusals of strict wire validation at the node
/// whose body it read: a bound outside the interval pattern
/// (`invalid_package`/`invalid-value`, `flat_wire::check_interval_bounds`'s) and a nested `case`
/// application (`ill_typed`/`operator-ineligible`, the flat-wire check's), each
/// with its path already on the bound or the nested `operator`. Every other
/// refusal of a walk is left as it is.
fn with_node_locus(failure: ValidationFailure, node_id: &CheckedNodeId) -> ValidationFailure {
    match failure {
        ValidationFailure::Refused(mut refusal)
            if matches!(
                (refusal.code, refusal.cause),
                (
                    CheckedPackageRefusalCode::InvalidPackage,
                    Some(CheckedPackageRefusalCause::InvalidValue)
                ) | (
                    CheckedPackageRefusalCode::IllTyped,
                    Some(CheckedPackageRefusalCause::OperatorIneligible)
                )
            ) && refusal.locus.is_none() =>
        {
            refusal.locus = Some(node_id.clone());
            ValidationFailure::Refused(refusal)
        }
        other => other,
    }
}

/// The pointer of the `reference`-th target a node's body walk reports,
/// found by re-running that same walk and materializing the trail only at
/// that target. The body was admitted by the first walk, so the walk is
/// deterministic and reaches it; the body pointer is the fallback.
fn body_reference_pointer(
    node: &RawSemanticNodeV2,
    kind: CheckedNodeKind,
    position: usize,
    reference: usize,
) -> JsonPointer {
    let body_steps = [
        Step::Key("semantic_graph"),
        Step::Key("nodes"),
        Step::Index(position),
        Step::Key("body"),
    ];
    let mut seen = 0_usize;
    let mut found = None;
    // The walk already succeeded once; a failure here only means no pointer.
    let _ = validate_body(
        kind,
        &node.body,
        &Trail::Base(&body_steps),
        &mut |_target, _site, target_at| {
            if seen == reference && found.is_none() {
                found = Some(target_at.pointer());
            }
            seen = seen.saturating_add(1);
        },
    );
    found.unwrap_or_else(|| node_pointer(position).key("body"))
}

/// Checks the graph's fixed version tag.
// Intake: checks the fixed graph version tag.
fn check_graph_version(graph: &RawSemanticGraphV2) -> Result<(), ValidationFailure> {
    if PackageWireTag::from_wire(&graph.graph_version) != Some(PackageWireTag::GraphVersion) {
        return Err(refuse(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            member_pointer(&["semantic_graph", "graph_version"]),
        ));
    }
    Ok(())
}

/// Checks a node's fixed schema version and decodes its tag and form once.
// Intake: checks the fixed schema version and decodes the node tag and form.
#[string_edge]
fn decode_node_kind<Body>(
    node: &CheckedSemanticNodeV2<Body>,
    at: impl Fn(&str) -> JsonPointer,
) -> Result<CheckedNodeKind, ValidationFailure> {
    if node.schema_version.as_ref() != GRAPH_V2 {
        return Err(refuse(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            at("schema_version"),
        ));
    }
    let Some(tag) = CheckedNodeTag::from_wire(&node.node_tag) else {
        return Err(refuse(
            CheckedPackageRefusalCode::UnsupportedNodeTag,
            at("node_tag"),
        ));
    };
    CheckedNodeKind::decode(tag, &node.semantic_form).ok_or_else(|| {
        refuse(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            at("semantic_form"),
        )
    })
}

fn validate_graph(
    wire: &RawCheckedPackageWireV2,
    limits: CheckedPackageReadLimits,
    meter: &mut WorkMeter,
    models: &[DomainModel],
    dependencies: &SuppliedDependencies<'_>,
    evidence: &CheckedPackageEvidence,
) -> Result<ValidatedGraph, ValidationFailure> {
    let graph = &wire.semantic_graph;
    check_graph_version(graph)?;
    if graph.nodes.is_empty() {
        return Err(refuse(
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            member_pointer(&["semantic_graph", "nodes"]),
        ));
    }
    let node_count = count(graph.nodes.len());
    if exceeds(node_count, limits.nodes) {
        // The first node past the ceiling is the one whose charge failed.
        return Err(ValidationFailure::incomplete(
            CheckedPackageLimit::Nodes,
            limits.nodes,
            node_count,
            Some(node_pointer(
                usize::try_from(limits.nodes).unwrap_or(usize::MAX),
            )),
        ));
    }
    let mut index = BTreeMap::new();
    let mut kinds = Vec::with_capacity(graph.nodes.len());
    let mut references: Vec<Vec<BodyReference>> = Vec::with_capacity(graph.nodes.len());
    let mut body_terms = Vec::with_capacity(graph.nodes.len());
    let mut terms = Vec::with_capacity(graph.nodes.len());
    // Each frame node's typed body, read once here and joined by the frame step.
    let mut frames = Vec::new();
    let mut edges = 0_u64;
    let mut group_ordinals: BTreeMap<&str, u64> = BTreeMap::new();
    for (position, node) in graph.nodes.iter().enumerate() {
        let at = |member: &str| node_pointer(position).key(member);
        validate_node_id(&node.node_id, || at("node_id"))?;
        let expected_package = match node.owner.as_ref() {
            Some(
                CheckedNodeOwner::ModelRoot { identity } | CheckedNodeOwner::Model { identity, .. },
            ) => identity.as_ref(),
            _ => wire.name.as_ref(),
        };
        if node.node_id.package.as_ref() != expected_package
            && (node.owner.is_some()
                || node.declaration.is_some()
                || !dependencies
                    .selected_node(&node.node_id)
                    .is_some_and(|selected| selected.to_raw() == *node))
        {
            return Err(refuse(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                at("node_id").key("package"),
            ));
        }
        if let Some(label) = node.recursion_group.as_deref() {
            let next = group_ordinals.entry(label).or_default();
            if node.node_id.ordinal != *next {
                return Err(refuse(
                    CheckedPackageRefusalCode::InvalidSemanticGraph,
                    at("node_id").key("ordinal"),
                ));
            }
            *next = next.saturating_add(1);
        }
        if index.insert(&node.node_id, position).is_some() {
            return Err(refuse(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                at("node_id"),
            ));
        }
        let kind = decode_node_kind(node, at)?;
        if kind == CheckedNodeKind::Relation(RelationForm::Refinement) {
            for (present, member) in [
                (node.recursion_group.is_some(), "recursion_group"),
                (node.declaration.is_some(), "declaration"),
                (node.owner.is_some(), "owner"),
            ] {
                if present {
                    return Err(refuse(CheckedPackageRefusalCode::MalformedWire, at(member)));
                }
            }
        }
        kinds.push(kind);
        validate_declaration(kind, &node.occurrences, node.declaration.as_ref(), position)?;
        validate_node_id(&node.semantic_type, || at("semantic_type"))?;
        for (dependency_index, dependency) in node.dependencies.iter().enumerate() {
            validate_node_id(dependency, || at("dependencies").index(dependency_index))?;
        }
        if node.recursion_group.as_deref().is_some_and(str::is_empty) {
            return Err(refuse(
                CheckedPackageRefusalCode::MalformedWire,
                at("recursion_group"),
            ));
        }
        let edges_before = edges;
        edges = edges.saturating_add(count(node.dependencies.len()));
        if exceeds(edges, limits.edges) {
            // `edges_before` was admitted, so the dependency that took the
            // counter one past the ceiling is the one whose charge failed.
            let first_over = limits.edges.saturating_sub(edges_before);
            return Err(ValidationFailure::incomplete(
                CheckedPackageLimit::Edges,
                limits.edges,
                edges,
                Some(at("dependencies").index(usize::try_from(first_over).unwrap_or(usize::MAX))),
            ));
        }
        let mut targets = Vec::new();
        let body_steps = [
            Step::Key("semantic_graph"),
            Step::Key("nodes"),
            Step::Index(position),
            Step::Key("body"),
        ];
        let mut frame_term = None;
        let work = match body_grammar(kind) {
            BodyGrammar::Extension => {
                let body = extension::read_body(&node.body, at("body"))?;
                for target in extension::targets(&body) {
                    targets.push((
                        target.clone(),
                        ReferenceSite {
                            member: ReferenceMember::Target,
                            is_body_root: false,
                        },
                    ));
                }
                extension::body_work(&body)
            }
            BodyGrammar::Frame => {
                let (body, work) = frame::read_frame_body(&node.body, &Trail::Base(&body_steps))?;
                frame_term = Some(body.clone().into_checked_term());
                frames.push((&node.node_id, position, body));
                work
            }
            BodyGrammar::Refinement => refinement::validate_body(
                &node.body,
                &Trail::Base(&body_steps),
                &mut |target, site, _target_at| targets.push((target.clone(), site)),
            )
            .map_err(|failure| with_node_locus(failure, &node.node_id))?,
            BodyGrammar::Term
            | BodyGrammar::TimedModel
            | BodyGrammar::Hybrid
            | BodyGrammar::AbstractionRelation => validate_body(
                kind,
                &node.body,
                &Trail::Base(&body_steps),
                &mut |target, site, _target_at| {
                    targets.push((target.clone(), site));
                },
            )
            .map_err(|failure| with_node_locus(failure, &node.node_id))?,
        };
        meter.charge(work, || at("body"))?;
        body_terms.push(work);
        references.push(targets);
        let mut occurrences = BTreeSet::new();
        if node.occurrences.is_empty() {
            return Err(refuse(
                CheckedPackageRefusalCode::InvalidSourceMap,
                at("occurrences"),
            ));
        }
        if let Some(repeat) = node
            .occurrences
            .iter()
            .position(|occurrence| !occurrences.insert(occurrence))
        {
            return Err(refuse(
                CheckedPackageRefusalCode::InvalidSourceMap,
                at("occurrences").index(repeat),
            ));
        }
        // FR-346 owns the abstraction body's first shape refusal later in
        // this graph stage. Its flat scan accepts shapes that the relation
        // step deliberately reports as InvalidSemanticGraph, so a failed
        // typed decode cannot preempt that step.
        let typed_term = match frame_term {
            Some(term) => Ok(term),
            None => decode_admitted_body(kind, &node.body),
        };
        terms.push(match typed_term {
            Ok(term) => Some(term),
            Err(_) if body_grammar(kind) == BodyGrammar::AbstractionRelation => None,
            Err(_) => {
                return Err(refuse(CheckedPackageRefusalCode::MalformedWire, at("body")));
            }
        });
    }
    model_root::validate_shape(&graph.nodes, &kinds, &terms)?;
    validate_owner_joins(wire, &kinds, &terms, models, meter)?;
    // Graph-shape body and dependency rules precede declaration, frame, state
    // and operation checks. All of them
    // run before the dependency and body-reference edges below are resolved,
    // so an entry, anchor or clause reference naming no node is the step's
    // own `missing_declaration` refusal rather than the generic
    // unresolved-reference `invalid_semantic_graph`.
    validate_structural_nodes(&graph.nodes, &kinds, &terms, &index)?;
    validate_derived_type_shapes(&graph.nodes, &kinds, &terms, &index)?;
    for &position in index.values() {
        if !matches!(
            kinds.get(position),
            Some(CheckedNodeKind::Claim(
                ClaimForm::StateGraph | ClaimForm::StepRelation
            )) | Some(CheckedNodeKind::Temporal(TemporalForm::Terminal))
        ) {
            continue;
        }
        let node = &graph.nodes[position];
        let expected = references[position]
            .iter()
            .filter(|(_, site)| site.member == ReferenceMember::Target)
            .map(|(target, _)| target.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        if node.dependencies != expected {
            return Err(ValidationFailure::refused_at(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                node_pointer(position).key("dependencies"),
                None,
                node.node_id.clone(),
            ));
        }
    }
    refinement::validate_dependencies(&graph.nodes, &kinds, &terms)?;
    let imported_nominals = validate_nominal_nodes(
        &graph.nodes,
        &kinds,
        &index,
        &wire.lock,
        models,
        dependencies,
        meter,
        limits.bytes,
    )?;
    validate_declaration_names(&graph.nodes, &kinds, &index)?;
    // FR-038 "The timed interval form" stage 2: the last graph check of the
    // nodes before the frame step.
    validate_timed_bounds_reduced(&graph.nodes, &index, meter)?;
    validate_protocol_references(&graph.nodes, &kinds, &index)?;
    // FR-322 step 2: each selected declaration's model declaration node key,
    // one validation visit apiece, charged at the selection it belongs to.
    let owners = ModelOwners::new(models, &graph.nodes, |root| {
        meter.charge(1, || node_pointer(root).key("owner"))
    })?;
    extension::validate(wire, &kinds, &terms, &index, evidence, &owners, meter)?;
    let declarations = owners.index();
    frame::validate_frame_semantics(frames, &graph.nodes, &kinds, &index, &owners, meter)?;
    state::validate_state(&graph.nodes, &kinds, &terms, &index, &owners, meter)?;
    claim_terminal::validate_state_graph(&graph.nodes, &kinds, &terms, &index, &owners)?;
    validate_temporal(
        &graph.nodes,
        &kinds,
        &index,
        &owners,
        &wire.lock,
        &wire.diagnostics.entries,
        meter,
    )?;
    abstraction::validate_abstraction(&graph.nodes, &kinds, &index, &owners, meter)?;
    claim_terminal::validate_terminal(&graph.nodes, &kinds, &terms, &index, &owners, meter)?;
    step_relation::validate_step_relations(&graph.nodes, &kinds, &terms, &index, &owners, meter)?;
    refinement::validate_step_rows(
        &graph.nodes,
        &kinds,
        &terms,
        wire.refinement_step_rows.as_deref(),
        &wire.lock.sources,
        meter,
    )?;
    if kinds.contains(&CheckedNodeKind::Relation(RelationForm::Refinement)) {
        // FR-375 orders all source-map defects before the refinement meaning
        // step; the ordinary package-wide check below remains the sole check
        // for graphs without a refinement.
        let source_check = validate_source_map_entries(
            graph
                .nodes
                .iter()
                .map(|node| (&node.node_id, node.occurrences.as_slice())),
            &wire.source_map,
            &wire.lock.sources,
            limits,
        );
        if let Err(failure) = source_check {
            if let ValidationFailure::Refused(refusal) = &failure {
                if refusal.code == CheckedPackageRefusalCode::InvalidSourceMap {
                    if let Some(path) = refusal.primary_path() {
                        let source_root = JsonPointer::root().key("source_map");
                        if path == &source_root {
                            refinement::validate_occurrences(
                                &graph.nodes,
                                &kinds,
                                &terms,
                                &wire.source_map,
                                wire.refinement_step_rows.as_deref(),
                                &owners,
                                dependencies,
                                meter,
                            )?;
                        } else if let Some(entry) = path
                            .as_str()
                            .strip_prefix(source_root.as_str())
                            .and_then(|suffix| suffix.strip_prefix('/'))
                            .and_then(|suffix| suffix.split('/').next())
                            .and_then(|index| index.parse::<usize>().ok())
                            .and_then(|index| wire.source_map.get(index))
                        {
                            if index.get(&entry.node_id).is_some_and(|&position| {
                                kinds[position]
                                    == CheckedNodeKind::Relation(RelationForm::Refinement)
                            }) {
                                return Err(ValidationFailure::refused_at(
                                    CheckedPackageRefusalCode::InvalidPackage,
                                    path.clone(),
                                    Some(CheckedPackageRefusalCause::InvalidValue),
                                    entry.node_id.clone(),
                                ));
                            }
                        }
                    }
                }
            }
            return Err(failure);
        }
        refinement::validate_occurrences(
            &graph.nodes,
            &kinds,
            &terms,
            &wire.source_map,
            wire.refinement_step_rows.as_deref(),
            &owners,
            dependencies,
            meter,
        )?;
    }
    refinement::validate_refinements(
        refinement::RefinementGraph::new(
            &graph.nodes,
            &kinds,
            &terms,
            &index,
            dependencies,
            &wire.name,
        ),
        &wire.lock,
        &owners,
        meter,
    )?;
    hybrid::validate_models(&graph.nodes, &kinds, &terms, &index, meter)?;
    hybrid::validate_claims(&graph.nodes, &kinds, &terms, &index, meter)?;
    timed_model::validate_models(
        &graph.nodes,
        &kinds,
        &terms,
        &index,
        &wire.source_map,
        &owners,
        meter,
    )?;
    protocol_body::validate_protocol_nodes(
        &graph.nodes,
        &kinds,
        &terms,
        &index,
        &wire.lock,
        &owners,
        meter,
        limits.bytes,
    )?;
    refinement::validate_catalog_graph(
        refinement::RefinementGraph::new(
            &graph.nodes,
            &kinds,
            &terms,
            &index,
            dependencies,
            &wire.name,
        ),
        &owners,
        meter,
    )?;
    let admitted_views = validate_operations(
        &graph.nodes,
        &kinds,
        &index,
        &terms,
        &wire.lock,
        &owners,
        dependencies,
        meter,
    )?;
    for (position, views) in admitted_views {
        let Some(Some(CheckedTerm::Application { operation, .. })) = terms.get_mut(position) else {
            return Err(refuse(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                node_pointer(position).key("body"),
            ));
        };
        operation.install_admitted_views(views);
    }
    let mut adjacency = Vec::with_capacity(graph.nodes.len());
    let mut lowering_facts = Vec::with_capacity(graph.nodes.len());
    let reference_pointer = |position: usize, reference: usize| match (
        graph.nodes.get(position),
        kinds.get(position),
    ) {
        (Some(node), Some(kind)) => body_reference_pointer(node, *kind, position, reference),
        _ => node_pointer(position).key("body"),
    };
    for (position, (node, targets)) in graph.nodes.iter().zip(&references).enumerate() {
        let resolve = |id: &CheckedNodeId, path: &dyn Fn() -> JsonPointer| {
            index
                .get(id)
                .copied()
                .ok_or_else(|| refuse(CheckedPackageRefusalCode::InvalidSemanticGraph, path()))
        };
        let at = |member: &str| node_pointer(position).key(member);
        let semantic_type = resolve(&node.semantic_type, &|| at("semantic_type"))?;
        let mut lowering_successors = vec![(semantic_type, lower::EdgeRole::Typing)];
        let mut successors = Vec::new();
        if semantic_type != position {
            successors.push((semantic_type, EdgeSite::SemanticType));
        }
        for (dependency_index, dependency) in node.dependencies.iter().enumerate() {
            if let Some(&target) = index.get(dependency) {
                lowering_successors.push((target, lower::EdgeRole::Typing));
                successors.push((target, EdgeSite::Dependency(dependency_index)));
            } else if !matches!(
                kinds[position],
                CheckedNodeKind::ScalarType(
                    ScalarTypeForm::Dimension
                        | ScalarTypeForm::Unit
                        | ScalarTypeForm::DimensionEquivalence
                )
            ) || !imported_nominals.contains(dependency)
            {
                return Err(refuse(
                    CheckedPackageRefusalCode::InvalidSemanticGraph,
                    at("dependencies").index(dependency_index),
                ));
            }
        }
        for (reference_index, (target, site)) in targets.iter().enumerate() {
            if !index.contains_key(target)
                && matches!(
                    kinds[position],
                    CheckedNodeKind::ScalarType(
                        ScalarTypeForm::Dimension
                            | ScalarTypeForm::Unit
                            | ScalarTypeForm::DimensionEquivalence
                    )
                )
                && imported_nominals.contains(target)
            {
                // A selected nominal in an admitted dependency is validated
                // by the nominal join above. It is outside this local graph's
                // recursion and lowering vertex set.
                continue;
            }
            let target = resolve(target, &|| reference_pointer(position, reference_index))?;
            let role = match site.member {
                ReferenceMember::Type => lower::EdgeRole::Annotation,
                ReferenceMember::Target
                | ReferenceMember::ResultType
                | ReferenceMember::FrameEntry => lower::EdgeRole::Typing,
            };
            // The extension reader uses its targets for graph admission, but
            // the lowering body grammar charges no extension-body reference
            // edges. Preserve that existing per-request closure and budget.
            if kinds.get(position).copied().map(body_grammar) != Some(BodyGrammar::Extension) {
                lowering_successors.push((target, role));
            }
            // Only a self-typed node's own top-level `literal.type` — the
            // literal that *is* the node body — may name itself from its body
            // without counting as a reference cycle requiring
            // `recursion_group`. This is conditional on an input reaching the
            // graph stage after earlier key authentication. The
            // carve-out is keyed on that member *and* on `is_body_root`, not
            // on node identity alone: a `literal.type` nested inside an
            // `aggregate` member, a `binding` value or an `application`
            // argument reports the same `ReferenceMember::Type` but with
            // `is_body_root: false`, and does not qualify — nor does a
            // `reference` body or an `application.result_type`
            // self-referencing a self-typed node. Both still resolve through
            // `recursion_group` or refuse, exactly like every other 1-node
            // cycle reached by this loop.
            if reference_requires_cycle_edge(position, target, semantic_type, *site) {
                successors.push((target, EdgeSite::Reference(reference_index)));
            }
        }
        adjacency.push(successors);
        let terms = body_terms
            .get(position)
            .copied()
            .ok_or_else(|| refuse(CheckedPackageRefusalCode::InvalidSemanticGraph, at("body")))?;
        lowering_facts.push(lower::LoweringNodeFacts {
            terms: Ok(terms),
            successors: lowering_successors,
            component: position,
        });
    }
    let edge_pointer = |vertex: usize, site: EdgeSite| match site {
        EdgeSite::SemanticType => node_pointer(vertex).key("semantic_type"),
        EdgeSite::Dependency(dependency) => {
            node_pointer(vertex).key("dependencies").index(dependency)
        }
        EdgeSite::Reference(reference) => reference_pointer(vertex, reference),
    };
    validate_recursion(&graph.nodes, &adjacency, meter, &edge_pointer)?;
    lower::assign_components(&mut lowering_facts);
    let terms = terms
        .into_iter()
        .enumerate()
        .map(|(position, term)| match term {
            Some(term) => Ok(term),
            None => {
                decode_admitted_body(kinds[position], &graph.nodes[position].body).map_err(|_| {
                    refuse(
                        CheckedPackageRefusalCode::MalformedWire,
                        node_pointer(position).key("body"),
                    )
                })
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ValidatedGraph {
        kinds,
        terms,
        declarations,
        lowering_facts,
    })
}

/// Joins already keyed protocol-reference carriers at the declaration step.
fn validate_protocol_references(
    nodes: &[RawSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &BTreeMap<&CheckedNodeId, usize>,
) -> Result<(), ValidationFailure> {
    for (&carrier_id, &position) in index {
        if kinds.get(position) != Some(&CheckedNodeKind::Protocol(ProtocolForm::ProtocolReference))
        {
            continue;
        }
        let Some(carrier) = nodes.get(position) else {
            continue;
        };
        let at = || node_pointer(position).key("body").key("target");
        let Some(target_id) = structural::reference_target(&carrier.body) else {
            return Err(refuse(CheckedPackageRefusalCode::MalformedWire, at()));
        };
        let Some(&target_position) = index.get(&target_id) else {
            return Err(ValidationFailure::refused_at(
                CheckedPackageRefusalCode::MissingDeclaration,
                at(),
                Some(CheckedPackageRefusalCause::MissingName),
                carrier_id.clone(),
            ));
        };
        if kinds
            .get(target_position)
            .is_none_or(|kind| kind.tag() != CheckedNodeTag::Protocol)
            || nodes
                .get(target_position)
                .is_none_or(|target| target.semantic_type != carrier.semantic_type)
        {
            return Err(ValidationFailure::refused_at(
                CheckedPackageRefusalCode::IllTyped,
                at(),
                Some(CheckedPackageRefusalCause::OperatorIneligible),
                carrier_id.clone(),
            ));
        }
    }
    Ok(())
}

/// Graph-stage reference decision after the target and semantic type resolve.
/// Public key authentication runs earlier; the exemption remains conditional
/// for callers that reach this stage with an already keyed graph.
fn reference_requires_cycle_edge(
    position: usize,
    target: usize,
    semantic_type: usize,
    site: ReferenceSite,
) -> bool {
    target != position
        || !site.is_body_root
        || site.member != ReferenceMember::Type
        || semantic_type != position
}

/// Every strongly connected component that forms a cycle must share one
/// explicit `recursion_group`. Iterative Tarjan; each edge costs one work,
/// charged at the member that carried the edge (`edge_pointer`).
fn validate_recursion(
    nodes: &[RawSemanticNodeV2],
    adjacency: &[Vec<(usize, EdgeSite)>],
    meter: &mut WorkMeter,
    edge_pointer: &dyn Fn(usize, EdgeSite) -> JsonPointer,
) -> Result<(), ValidationFailure> {
    const UNVISITED: usize = usize::MAX;
    let size = adjacency.len();
    let mut order = vec![UNVISITED; size];
    let mut low = vec![0_usize; size];
    let mut on_stack = vec![false; size];
    let mut stack = Vec::new();
    let mut calls: Vec<(usize, usize)> = Vec::new();
    let mut next = 0_usize;
    for root in 0..size {
        if order.get(root) != Some(&UNVISITED) {
            continue;
        }
        enter(
            root,
            &mut next,
            &mut order,
            &mut low,
            &mut on_stack,
            &mut stack,
        );
        calls.push((root, 0));
        while let Some((vertex, edge)) = calls.last().copied() {
            let successor = adjacency
                .get(vertex)
                .and_then(|edges| edges.get(edge))
                .copied();
            if let Some((successor, site)) = successor {
                meter.charge(1, || edge_pointer(vertex, site))?;
                if let Some(frame) = calls.last_mut() {
                    frame.1 = edge.saturating_add(1);
                }
                if order.get(successor) == Some(&UNVISITED) {
                    enter(
                        successor,
                        &mut next,
                        &mut order,
                        &mut low,
                        &mut on_stack,
                        &mut stack,
                    );
                    calls.push((successor, 0));
                } else if on_stack.get(successor) == Some(&true) {
                    let reached = order.get(successor).copied().unwrap_or(UNVISITED);
                    if let Some(current) = low.get_mut(vertex) {
                        *current = (*current).min(reached);
                    }
                }
                continue;
            }
            calls.pop();
            let vertex_low = low.get(vertex).copied().unwrap_or(0);
            if let Some((parent, _)) = calls.last() {
                if let Some(parent_low) = low.get_mut(*parent) {
                    *parent_low = (*parent_low).min(vertex_low);
                }
            }
            if order.get(vertex) != Some(&vertex_low) {
                continue;
            }
            let mut component = Vec::new();
            while let Some(member) = stack.pop() {
                if let Some(flag) = on_stack.get_mut(member) {
                    *flag = false;
                }
                component.push(member);
                if member == vertex {
                    break;
                }
            }
            let cyclic = component.len() > 1
                || adjacency
                    .get(vertex)
                    .is_some_and(|edges| edges.iter().any(|(target, _)| *target == vertex));
            if cyclic {
                let group_of = |member: usize| {
                    nodes
                        .get(member)
                        .and_then(|node| node.recursion_group.as_ref())
                };
                // Members in graph order: the lowest-positioned member sets
                // the group the others must share. The refusal names that
                // member when it has no group, else the first member whose
                // group differs — its `recursion_group` when it carries one,
                // else the node object that lacks the member.
                let first_member = component.iter().copied().min();
                let first = first_member.and_then(group_of);
                let offender = if first.is_none() {
                    first_member
                } else {
                    component
                        .iter()
                        .copied()
                        .filter(|member| group_of(*member) != first)
                        .min()
                };
                if let Some(member) = offender {
                    let path = if group_of(member).is_some() {
                        node_pointer(member).key("recursion_group")
                    } else {
                        node_pointer(member)
                    };
                    return Err(refuse(
                        CheckedPackageRefusalCode::InvalidSemanticGraph,
                        path,
                    ));
                }
            } else if nodes.get(vertex).is_some_and(|node| {
                node.recursion_group.is_some()
                    && CheckedNodeTag::from_wire(&node.node_tag)
                        .and_then(|tag| CheckedNodeKind::decode(tag, &node.semantic_form))
                        == Some(CheckedNodeKind::Protocol(ProtocolForm::ProtocolReference))
            }) {
                return Err(refuse(
                    CheckedPackageRefusalCode::MalformedWire,
                    node_pointer(vertex).key("recursion_group"),
                ));
            }
        }
    }
    Ok(())
}

fn enter(
    vertex: usize,
    next: &mut usize,
    order: &mut [usize],
    low: &mut [usize],
    on_stack: &mut [bool],
    stack: &mut Vec<usize>,
) {
    if let (Some(slot), Some(low_slot), Some(flag)) = (
        order.get_mut(vertex),
        low.get_mut(vertex),
        on_stack.get_mut(vertex),
    ) {
        *slot = *next;
        *low_slot = *next;
        *flag = true;
    }
    *next = next.saturating_add(1);
    stack.push(vertex);
}

fn validate_capabilities(
    wire: &RawCheckedPackageWireV2,
    evidence: &CheckedPackageEvidence,
) -> Result<(), ValidationFailure> {
    let report = || member_pointer(&["capability_report"]);
    let mut reported = BTreeMap::new();
    for (index, capability) in wire.capability_report.iter().enumerate() {
        if !is_nonempty(&capability.feature)
            || reported
                .insert(capability.feature.as_ref(), (capability.disposition, index))
                .is_some()
        {
            return Err(refuse(
                CheckedPackageRefusalCode::UnknownRequiredCapability,
                report().index(index).key("feature"),
            ));
        }
    }
    for (index, feature) in wire.lock.required_features.iter().enumerate() {
        let path = match reported.get(feature.as_ref()) {
            // Unreported: the report lacks the capability.
            None => report(),
            Some((disposition, reported_at))
                if *disposition != CheckedCapabilityDisposition::Available =>
            {
                report().index(*reported_at).key("disposition")
            }
            // Reported available, but this reader does not support it.
            Some(_) if !evidence.supports(feature) => {
                member_pointer(&["lock", "required_features"]).index(index)
            }
            Some(_) => continue,
        };
        return Err(refuse(
            CheckedPackageRefusalCode::UnknownRequiredCapability,
            path,
        ));
    }
    Ok(())
}

fn validate_diagnostics(
    wire: &RawCheckedPackageWireV2,
    limits: CheckedPackageReadLimits,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let entries = || member_pointer(&["diagnostics", "entries"]);
    let diagnostics = count(wire.diagnostics.entries.len());
    if exceeds(diagnostics, limits.diagnostics) {
        // The first entry past the ceiling is the one whose charge failed.
        return Err(ValidationFailure::incomplete(
            CheckedPackageLimit::Diagnostics,
            limits.diagnostics,
            diagnostics,
            Some(entries().index(usize::try_from(limits.diagnostics).unwrap_or(usize::MAX))),
        ));
    }
    let nodes = wire
        .semantic_graph
        .nodes
        .iter()
        .map(|node| &node.node_id)
        .collect::<BTreeSet<_>>();
    for (entry_index, entry) in wire.diagnostics.entries.iter().enumerate() {
        for (detail_index, detail) in entry.details.iter().enumerate() {
            let detail_steps = [
                Step::Key("diagnostics"),
                Step::Key("entries"),
                Step::Index(entry_index),
                Step::Key("details"),
                Step::Index(detail_index),
            ];
            let detail_at = Trail::Base(&detail_steps);
            let mut unresolved = None;
            let work = validate_term(
                detail,
                TermGrammar::V2,
                false,
                &detail_at,
                &mut |target, _site, target_at| {
                    if unresolved.is_none() && !nodes.contains(target) {
                        unresolved = Some(target_at.pointer());
                    }
                },
            )?;
            meter.charge(work, || detail_at.pointer())?;
            if let Some(path) = unresolved {
                return Err(refuse(
                    CheckedPackageRefusalCode::InvalidSemanticGraph,
                    path,
                ));
            }
        }
        for (locus_index, region) in entry.loci.iter().enumerate() {
            let locus = || entries().index(entry_index).key("loci").index(locus_index);
            if !wire.lock.sources.contains(&region.source) {
                return Err(refuse(
                    CheckedPackageRefusalCode::InvalidSourceMap,
                    locus().key("source"),
                ));
            }
            if region.start >= region.end {
                return Err(refuse(CheckedPackageRefusalCode::InvalidSourceMap, locus()));
            }
        }
    }
    Ok(())
}

/// Helpers shared by the in-crate unit tests of the typed accessors.
#[cfg(test)]
mod test_support {
    use super::CheckedNodeId;
    use serde_json::{json, Value};

    /// The JSON of a distinct structural node identity.
    pub(super) fn id(digit: char) -> Value {
        json!({"package": "test", "path": [digit.to_string()], "ordinal": 0})
    }

    /// A typed structural node identity.
    pub(super) fn node_id(digit: char) -> CheckedNodeId {
        serde_json::from_value(id(digit)).expect("node id")
    }
}

#[cfg(test)]
mod tests {
    use super::{
        body_grammar, node_pointer, reference_requires_cycle_edge, validate_body,
        validate_recursion, BodyGrammar, CheckedDependencySelection, CheckedNodeKind,
        CheckedPackageRefusalCode, CheckedSemanticNodeV2, CorrespondenceForm, EdgeSite,
        ExpressionForm, ReferenceMember, ReferenceSite, StateForm, Step, Trail, ValidationFailure,
        WorkMeter, DEPENDENCY_SELECTION_MEMBERS,
    };
    use ix_trace_rs::trace;
    use serde_json::json;

    /// Trace: FR-322-AC-77, TC-233.
    #[test]
    fn refinement_internal_lock_rows_are_nonempty_ordered_and_unique() {
        use super::CheckedRefinementInternal;
        let row = |refinement: &str, internal: &[&str]| CheckedRefinementInternal {
            refinement: refinement.into(),
            internal: internal.iter().map(|name| (*name).into()).collect(),
        };
        assert!(super::validate_refinement_internals(None).is_ok());
        assert!(super::validate_refinement_internals(Some(&[
            row("A", &["memory drain", "memory flush"]),
            row("B", &["protocol call"]),
        ]))
        .is_ok());
        let invalid = [
            (vec![], CheckedPackageRefusalCode::MalformedWire),
            (
                vec![row("A", &[])],
                CheckedPackageRefusalCode::MalformedWire,
            ),
            (
                vec![row("B", &["step"]), row("A", &["step"])],
                CheckedPackageRefusalCode::InvalidPackage,
            ),
            (
                vec![row("A", &["step"]), row("A", &["step"])],
                CheckedPackageRefusalCode::InvalidPackage,
            ),
            (
                vec![row("A", &["z", "a"])],
                CheckedPackageRefusalCode::InvalidPackage,
            ),
            (
                vec![row("A", &["a", "a"])],
                CheckedPackageRefusalCode::InvalidPackage,
            ),
        ];
        for (rows, expected) in invalid {
            let Err(ValidationFailure::Refused(refusal)) =
                super::validate_refinement_internals(Some(&rows))
            else {
                panic!("expected lock refusal");
            };
            assert_eq!(refusal.code, expected);
        }
    }

    #[trace("FR-038-AC-368", "FR-038-AC-369", "TC-048")]
    #[test]
    fn repeated_lock_values_report_both_concrete_locations_in_declaration_order() {
        use super::{CheckedDependencySelection, CheckedRefinementInternal};

        let paths = |failure: ValidationFailure| {
            let ValidationFailure::Refused(refusal) = failure else {
                panic!("expected a typed refusal");
            };
            assert!(refusal
                .loci
                .iter()
                .all(|location| location.source_region.is_none()));
            refusal
                .loci
                .iter()
                .map(|location| {
                    location
                        .path
                        .as_ref()
                        .expect("lock value pointer")
                        .to_string()
                })
                .collect::<Vec<_>>()
        };
        let row = |refinement: &str, internal: &[&str]| CheckedRefinementInternal {
            refinement: refinement.into(),
            internal: internal.iter().map(|name| (*name).into()).collect(),
        };
        let repeated_row =
            super::validate_refinement_internals(Some(&[row("R", &["a"]), row("R", &["b"])]))
                .expect_err("repeated refinement");
        assert_eq!(
            paths(repeated_row),
            [
                "/lock/refinement_internals/0/refinement",
                "/lock/refinement_internals/1/refinement"
            ]
            .map(str::to_owned)
            .to_vec()
        );
        let repeated_spelling =
            super::validate_refinement_internals(Some(&[row("R", &["a", "a"])]))
                .expect_err("repeated internal spelling");
        assert_eq!(
            paths(repeated_spelling),
            [
                "/lock/refinement_internals/0/internal/0",
                "/lock/refinement_internals/0/internal/1"
            ]
            .map(str::to_owned)
            .to_vec()
        );
        let repeated_dependency = super::validate_dependency_selections(&[
            CheckedDependencySelection {
                identity: "lib".into(),
            },
            CheckedDependencySelection {
                identity: "lib".into(),
            },
        ])
        .expect_err("repeated dependency identity");
        assert_eq!(
            paths(repeated_dependency),
            [
                "/lock/dependency_selections/0",
                "/lock/dependency_selections/1"
            ]
            .map(str::to_owned)
            .to_vec()
        );
        let one_location = super::validate_refinement_internals(Some(&[row("R", &[])]))
            .expect_err("empty internal list");
        assert_eq!(
            paths(one_location),
            ["/lock/refinement_internals/0"].map(str::to_owned).to_vec()
        );
    }

    fn recursion_test_node(digit: char, group: Option<&str>) -> super::RawSemanticNodeV2 {
        serde_json::from_value(json!({
            "node_id": super::test_support::id(digit),
            "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": "protocol",
            "semantic_form": "protocol_reference",
            "semantic_type": super::test_support::id('f'),
            "dependencies": [],
            "occurrences": [{"role": "expression", "ordinal": 0}],
            "recursion_group": group,
            "body": {"term": "reference", "target": super::test_support::id('e')},
        }))
        .expect("recursion-test node")
    }

    fn assert_recursion_refusal(
        result: Result<(), ValidationFailure>,
        code: CheckedPackageRefusalCode,
        path: String,
    ) {
        let Err(ValidationFailure::Refused(refusal)) = result else {
            panic!("expected recursion refusal, got {result:?}");
        };
        assert_eq!(refusal.code, code);
        assert_eq!(
            refusal.loci[0].path.as_ref().map(ToString::to_string),
            Some(path)
        );
    }

    #[trace("FR-038-AC-241", "TC-048", "TC-946")]
    #[test]
    fn tc_946_recursion_groups_must_cover_exactly_a_cycle_component() {
        let mismatch = [
            recursion_test_node('a', Some("first")),
            recursion_test_node('b', Some("second")),
        ];
        let cycle = vec![
            vec![(1, EdgeSite::Reference(0))],
            vec![(0, EdgeSite::Reference(0))],
        ];
        let mut meter = WorkMeter::new(u64::MAX);
        assert_recursion_refusal(
            validate_recursion(&mismatch, &cycle, &mut meter, &|_, _| {
                node_pointer(1).key("recursion_group")
            }),
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            node_pointer(1).key("recursion_group").to_string(),
        );

        let missing = [
            recursion_test_node('a', Some("shared")),
            recursion_test_node('b', None),
        ];
        let mut meter = WorkMeter::new(u64::MAX);
        assert_recursion_refusal(
            validate_recursion(&missing, &cycle, &mut meter, &|_, _| {
                node_pointer(1).key("recursion_group")
            }),
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            node_pointer(1).to_string(),
        );

        let orphan = [recursion_test_node('c', Some("orphan"))];
        let mut meter = WorkMeter::new(u64::MAX);
        assert_recursion_refusal(
            validate_recursion(&orphan, &[vec![]], &mut meter, &|_, _| {
                node_pointer(0).key("recursion_group")
            }),
            CheckedPackageRefusalCode::MalformedWire,
            node_pointer(0).key("recursion_group").to_string(),
        );
    }

    fn tc_451_private_owned_wire() -> super::CheckedPackageWireV2 {
        let edition =
            json!({"role": "edition", "definition": {"authority": "test", "identity": "edition"}});
        serde_json::from_value(json!({
            "contract_version": super::CHECKED_PACKAGE_V2,
            "name": "test",
            "lock": {
                "sources": [], "edition": edition, "profile_selections": [],
                "definition_selections": [],
                "required_features": [], "dependency_selections": []
            },
            "semantic_graph": {"graph_version": "quire.checked-semantic-graph/v2", "nodes": []},
            "source_map": [], "capability_report": [],
            "diagnostics": {"catalog": {"authority": "test", "identity": "diagnostics"}, "entries": []}
        }))
        .expect("private owned wire")
    }

    #[trace("TC-451", "FR-051-AC-1", "FR-051-AC-4")]
    #[test]
    fn tc_451_borrowed_encoder_equals_private_owned_wire_bytes() {
        let wire = tc_451_private_owned_wire();
        let bytes = super::encode_checked_package_v2(
            &wire.name,
            &wire.lock,
            &wire.semantic_graph,
            super::CheckedPackageSourceCorrespondenceRef {
                source_map: &wire.source_map,
                refinement_step_rows: wire.refinement_step_rows.as_deref(),
            },
            &wire.capability_report,
            &wire.diagnostics,
            quire_canonical::Limits::new(u64::MAX),
        )
        .expect("canonical envelope");
        let owned_bytes = serde_json::to_vec(&serde_json::to_value(&wire).expect("wire value"))
            .expect("private wire bytes");
        assert_eq!(bytes, owned_bytes);
    }

    #[trace("TC-451", "FR-051-AC-4")]
    #[test]
    fn tc_451_deep_borrowed_envelope_equals_private_owned_wire_on_a_small_stack() {
        fn nested(depth: usize) -> serde_json::Value {
            let mut value = serde_json::Value::Null;
            for _ in 0..depth {
                value = serde_json::Value::Array(vec![value]);
            }
            value
        }

        let mut wire = tc_451_private_owned_wire();
        let node_id = super::test_support::id('b');
        let node: CheckedSemanticNodeV2 = serde_json::from_value(json!({
            "node_id": node_id.clone(),
            "schema_version": "quire.checked-semantic-graph/v2",
            "node_tag": "test", "semantic_form": "test",
            "semantic_type": node_id, "dependencies": [], "occurrences": [],
            "body": {"term":"literal","type":node_id,"value_kind":"integer","value":null}
        }))
        .expect("shallow graph node");
        wire.semantic_graph.nodes.push(node);

        let (expected, actual) = std::thread::Builder::new()
            .stack_size(256 * 1024)
            .spawn(move || -> Result<_, quire_canonical::Error> {
                const DEEP: usize = 100_000;
                // The typed private wire supplies every envelope field. Build
                // its JSON value while bodies are shallow, then place deep
                // bodies without recursive serde serialization.
                let mut private_wire_value =
                    serde_json::to_value(&wire).expect("private owned wire value");
                let mut expected_body = serde_json::Map::new();
                expected_body.insert("term".into(), json!("literal"));
                expected_body.insert("type".into(), node_id.clone());
                expected_body.insert("value_kind".into(), json!("integer"));
                expected_body.insert("value".into(), nested(DEEP));
                private_wire_value["semantic_graph"]["nodes"][0]["body"] =
                    serde_json::Value::Object(expected_body);
                wire.semantic_graph.nodes[0].body = super::CheckedTerm::Literal {
                    type_id: super::test_support::node_id('b'),
                    value_kind: super::LiteralKind::Integer,
                    value: nested(DEEP),
                };

                let result = (|| {
                    let ceiling = 1_000_000;
                    let limits = quire_canonical::Limits::new(ceiling);
                    let expected = quire_canonical::to_vec(&private_wire_value, limits)?;
                    let actual = super::encode_checked_package_v2(
                        &wire.name,
                        &wire.lock,
                        &wire.semantic_graph,
                        super::CheckedPackageSourceCorrespondenceRef {
                            source_map: &wire.source_map,
                            refinement_step_rows: wire.refinement_step_rows.as_deref(),
                        },
                        &wire.capability_report,
                        &wire.diagnostics,
                        limits,
                    )?;
                    Ok((expected, actual))
                })();

                quire_canonical::drop_value(std::mem::take(&mut private_wire_value));
                if let super::CheckedTerm::Literal { value, .. } =
                    &mut wire.semantic_graph.nodes[0].body
                {
                    quire_canonical::drop_value(std::mem::take(value));
                }
                result
            })
            .expect("small-stack thread spawns")
            .join()
            .expect("small-stack thread completes")
            .expect("deep private and borrowed envelopes encode");
        assert_eq!(actual, expected);
    }

    /// The body-root literal exemption is a graph-stage rule after key
    /// authentication; nested literal, reference and result-type sites keep
    /// their cycle edges even on a self-typed node.
    #[trace("TC-048", "FR-038-AC-18")]
    #[test]
    fn tc_048_only_a_self_typed_body_root_literal_omits_its_cycle_edge() {
        let id = super::test_support::node_id('a');
        let typed = serde_json::to_value(&id).expect("node id");
        let bodies = [
            (
                json!({"term":"literal", "type":typed, "value_kind":"integer", "value":1}),
                false,
            ),
            (
                json!({"term":"aggregate", "members":[{"term":"literal", "type":typed, "value_kind":"integer", "value":1}]}),
                true,
            ),
            (
                json!({"term":"aggregate", "members":[{"term":"binding", "name":"x", "value":{"term":"literal", "type":typed, "value_kind":"integer", "value":1}}]}),
                true,
            ),
            (json!({"term":"reference", "target":typed}), true),
        ];
        for (body, requires_edge) in bodies {
            let mut sites = Vec::new();
            validate_body(
                CheckedNodeKind::Expression(ExpressionForm::Reference),
                &body,
                &Trail::Base(&[Step::Key("body")]),
                &mut |target, site, _| {
                    if target == &id {
                        sites.push(site);
                    }
                },
            )
            .expect("closed body");
            assert_eq!(sites.len(), 1);
            let has_edge = reference_requires_cycle_edge(0, 0, 0, sites[0]);
            assert_eq!(has_edge, requires_edge);
            let node: super::RawSemanticNodeV2 = serde_json::from_value(json!({
                "node_id":typed,
                "schema_version":"quire.checked-semantic-graph/v2",
                "node_tag":"expression",
                "semantic_form":"reference",
                "semantic_type":typed,
                "dependencies":[],
                "occurrences":[{"role":"generated", "ordinal":0}],
                "body":body,
            }))
            .expect("graph-stage node");
            let adjacency = if has_edge {
                vec![vec![(0, EdgeSite::Reference(0))]]
            } else {
                vec![vec![]]
            };
            let mut meter = WorkMeter::new(u64::MAX);
            let result = validate_recursion(&[node], &adjacency, &mut meter, &|_, _| {
                node_pointer(0).key("body")
            });
            if requires_edge {
                let Err(ValidationFailure::Refused(refusal)) = result else {
                    panic!("a nonexempt self-reference must refuse");
                };
                assert_eq!(
                    refusal.code,
                    CheckedPackageRefusalCode::InvalidSemanticGraph
                );
                assert_eq!(refusal.loci[0].path.clone(), Some(node_pointer(0)));
            } else {
                assert!(result.is_ok(), "the body-root literal is exempt");
            }
        }
        let root_literal = ReferenceSite {
            member: ReferenceMember::Type,
            is_body_root: true,
        };
        assert!(!reference_requires_cycle_edge(0, 0, 0, root_literal));
        assert!(reference_requires_cycle_edge(0, 1, 0, root_literal));
        assert!(reference_requires_cycle_edge(0, 0, 1, root_literal));
        for site in [
            ReferenceSite {
                is_body_root: false,
                ..root_literal
            },
            ReferenceSite {
                member: ReferenceMember::Target,
                is_body_root: true,
            },
            ReferenceSite {
                member: ReferenceMember::ResultType,
                is_body_root: true,
            },
        ] {
            assert!(reference_requires_cycle_edge(0, 0, 0, site));
        }
    }

    /// The members `classify_dependency_entry_shape` treats as required are
    /// exactly the members of `CheckedDependencySelection`.
    #[test]
    fn dependency_selection_required_members_match_the_type() {
        let entry = CheckedDependencySelection {
            identity: "a".into(),
        };
        let value = serde_json::to_value(entry).expect("serializes");
        let mut members: Vec<&str> = value
            .as_object()
            .expect("an object")
            .keys()
            .map(String::as_str)
            .collect();
        let mut required = DEPENDENCY_SELECTION_MEMBERS.to_vec();
        members.sort_unstable();
        required.sort_unstable();
        assert_eq!(members, required);
    }

    /// `BodyBindingRules`: exactly one kind, `state`/`frame`, has the frame
    /// reference-triple body; every other kind's body is a `SemanticTerm`.
    #[test]
    fn exactly_one_kind_has_a_frame_body() {
        let frames = CheckedNodeKind::all()
            .into_iter()
            .filter(|kind| body_grammar(*kind) == BodyGrammar::Frame)
            .collect::<Vec<_>>();
        assert_eq!(frames, [CheckedNodeKind::State(StateForm::Frame)]);
    }

    /// FR-346: exactly one kind, `correspondence`/`abstraction_relation`, has
    /// the abstraction relation body.
    #[trace("TC-225", "FR-346-AC-2")]
    #[test]
    fn tc_225_exactly_one_kind_has_an_abstraction_relation_body() {
        let relations = CheckedNodeKind::all()
            .into_iter()
            .filter(|kind| body_grammar(*kind) == BodyGrammar::AbstractionRelation)
            .collect::<Vec<_>>();
        assert_eq!(
            relations,
            [CheckedNodeKind::Correspondence(
                CorrespondenceForm::AbstractionRelation
            )]
        );
    }
}
