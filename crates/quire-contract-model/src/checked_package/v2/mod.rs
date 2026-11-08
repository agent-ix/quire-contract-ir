//! Strict reader for QSpec I04 `quire.checked-package/v2`, the sole admitted
//! `CheckedPackage` contract.
//!
//! Implements the I04 contract described by AD-006. The wire shape this
//! module admits is stated here, in the types and checks below, rather than
//! by pointing at a revision of another repository. Model, source, and
//! definition selections retain the owning names and exact structure.

mod abstraction;
mod budgeted_read;
pub(in crate::checked_package) use budgeted_read::BudgetStop;
pub use budgeted_read::{
    CheckedPackageBudgetIncomplete, CheckedPackageBudgetedDispatchResult,
    CheckedPackagePendingOutcome, CheckedPackageReadExhausted, CheckedPackageV2BudgetedReadResult,
    PackageReadDeniedCharge, PackageReadResource,
};
mod callable;
mod checked_json;
pub(super) mod document_de;
pub(in crate::checked_package) use budgeted_read::denial as decode_denial;
pub use checked_json::{CheckedJson, CheckedJsonError};
pub(in crate::checked_package) use document_de::{exact_vec, DecodeContext};
mod model_operations;
pub use model_operations::{
    CheckedModelOperation, CheckedModelOperations, CheckedOperationParameter,
    CheckedOperationReadError, CheckedOperationReadErrorCode,
};
mod checked_term_data;
mod claim_terminal;
mod closure_index;
mod comparison_operands;
mod composite_operands;
mod contract_package;
mod dependency_references;
mod dispatch_projection;
pub(in crate::checked_package) mod encode;
mod equality_projection;
mod execution_point;
mod execution_point_accessor;
mod extension;
mod extension_body;
mod flat_wire;
mod frame;
mod graph_storage;
mod guarantee_composition;
mod guarantee_terms;
mod hybrid;
mod identity;
mod intake;
mod lock_tables;
mod lossless;
mod lower;
mod model_enum_witness;
mod model_fields;
mod model_members;
mod model_population_catalog;
mod model_root;
mod monitor_plan;
mod native_join_budget;
mod native_join_storage;
mod natural;
mod node_index;
mod node_lookup;
mod retained_index;
pub(in crate::checked_package) use node_index::StructuralOrder;
mod operation_catalog;
mod operation_catalog_index;
pub(in crate::checked_package) mod read_charge;
pub use operation_catalog::{initialize_operation_catalog, CatalogReadError};
mod operation_effects;
mod operation_witness;
mod operations;
mod owner;
mod parameter_declaration;
mod protocol_body;
mod protocol_locations;
mod raw_body;
mod reaches_population;
mod refinement;
mod relationship_end;
mod runtime_type_projection;
mod rust_spelling;
mod scalar_operands;
mod schedulability;
mod selected_document;
mod selected_facts;
mod selection_equality;
mod source_map;
mod stage_storage;
mod state;
mod step_relation;
mod structural;
mod temporal;
mod temporal_binding;
mod temporal_clause_names;
mod term_json;
// The node views are consumed as each validator moves off `serde_json::Value`.
#[allow(dead_code)]
mod term_view;
#[allow(unused_imports)]
pub(in crate::checked_package) use term_view::{
    NodeIdRead, TermItems, TermMembers, TermNode, TermSubterm, TermSubterms, TermView,
};
mod timed_model;
mod typed_terms;
mod vocabulary;

pub use callable::{
    CheckedCallable, CheckedCallableIdentity, CheckedCallableRead, CheckedCallableReadError,
    CheckedCallableReadErrorCode, CheckedCallableTable,
};
pub use checked_term_data::*;
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
    ContractSemanticGraphV1, ContractSourceHandoffError, ContractSourceKeyV1, ContractSourceMapV1,
    ContractSourceV1, ContractUnsupportedSourceKindError,
};
pub use dispatch_projection::{CheckedDispatchReadError, CheckedDispatchView};
pub use equality_projection::{
    AdmittedEqualityReadError, AdmittedEqualityReadErrorCode, EqualityDimensionRole,
    EqualityFactWitness, EqualityOperandTypeRole, EqualityUnitRole, EqualityWitnessKind,
    SelectedEqualityApplicationRow, SelectedEqualityFactRow,
};
pub use execution_point_accessor::{CheckedExecutionPointError, CheckedExecutionPointRefusal};
pub use guarantee_terms::{
    CheckedComponentClause, CheckedCompositionComponent, CheckedCompositionPremise,
    CheckedEnvironmentEntry, CheckedEnvironmentSupplies, CheckedFairnessSupply,
    CheckedGuaranteeMeasure, CheckedGuaranteeProgress,
};
pub use identity::*;
pub use lower::*;
pub use model_enum_witness::{
    SelectedModelEnumDeclarationRow, SelectedModelEnumError, SelectedModelEnumErrorCode,
    SelectedModelEnumMemberRow, SelectedModelEnumShapeCaseRow, SelectedModelEnumShapeRow,
    SelectedModelEnums,
};
pub use model_fields::{
    CheckedCollectionKind, CheckedMemberType, CheckedModelField, CheckedModelFieldsError,
    CheckedModelObjectFields, CheckedModelRecordField, CheckedModelRecordFields,
    CheckedModelRecordFieldsError, CheckedModelVariantBody, CheckedModelVariantCase,
    CheckedVariantPayloadType,
};
pub use model_population_catalog::{
    CheckedModelPopulation, CheckedModelPopulationCatalog, CheckedModelPopulationCatalogError,
    CheckedModelPopulationCatalogLimitKind, CheckedModelPopulationCatalogLimits,
    CheckedModelPopulationCatalogLocus, CheckedModelPopulationMember,
    CheckedModelPopulationMemberUnresolved, CheckedSelectedModelPopulations,
};
pub use monitor_plan::{
    CheckedMonitorBufferRef, CheckedMonitorClock, CheckedMonitorClockRead,
    CheckedMonitorClockReadError, CheckedMonitorClockReadErrorCode, CheckedMonitorEntryRef,
    CheckedMonitorPlanRef, SelectedMonitorClosureRows, SelectedMonitorTypeClosure,
    SelectedMonitorTypeClosureError, SelectedMonitorTypeClosureErrorCode,
};
pub use native_join_budget::{
    NativeJoinBudget, NativeJoinCharge, NativeJoinChargeKind, NativeJoinDeniedCharge,
    NativeJoinLocus, NativeJoinLocusSource, PackageReadStage,
};
pub use protocol_locations::CheckedProtocolLocationReadError;
pub use selected_facts::{
    SelectedFactEquality, SelectedFactEqualityId, SelectedFactField, SelectedFactFieldId,
    SelectedFactIds, SelectedFactRows, SelectedFacts, SelectedFactsError, SelectedFactsErrorCode,
};
pub use timed_model::{
    CheckedModelTimeRead, CheckedModelTimeReadError, CheckedModelTimeReadErrorCode,
};
mod exact_projection;
pub use exact_projection::{
    ExactTypeProjection, ExactTypeProjectionError, ExactTypeProjectionErrorCode,
    SelectedEnumDeclarationRow, SelectedEnumMemberRow, SelectedExactDeclarationForm,
    SelectedExactDeclarationRow, SelectedExactFieldRow, SelectedExactIdentities,
    SelectedExactIdentity, SelectedRecordFieldView, SelectedRecordView,
    SelectedSourceCollectionRow, SelectedSourceEnumRow, SelectedSourceRecordMemberRow,
    SelectedSourceRecordRow, SelectedSourceTupleRow, SelectedSourceUnionRow, SelectedSourceUnitRow,
    SelectedTupleDeclarationRow, SelectedTuplePositionRow, SelectedUnionMemberRow,
};
pub use operation_effects::{
    CheckedOperationEffect, CheckedOperationEffectOrigin, CheckedOperationEffectProvenance,
    CheckedOperationEffectRef, CheckedOperationEffectRow,
};
pub use operation_witness::{
    SelectedOperationIdentity, SelectedOperationMemberError, SelectedOperationMemberErrorCode,
    SelectedOperationMemberRow, SelectedOperationMembers,
};
pub use parameter_declaration::{CheckedParameterDeclaration, CheckedParameterDeclarationError};
pub use protocol_body::{CheckedProtocolActivation, CheckedProtocolActivationRow};
pub use reaches_population::CheckedReachesPopulationError;
pub use refinement::*;
pub use relationship_end::{CheckedRelationshipEnd, CheckedRelationshipEndError};
pub use runtime_type_projection::{
    SelectedTypeDeclarations, SelectedTypeDeclarationsError, SelectedTypeDimension,
    SelectedTypeModelUnit, SelectedTypeObject, SelectedTypeObjectField,
    SelectedTypeObjectOperation, SelectedTypeOperationParameter, SelectedTypeRecord,
    SelectedTypeTuple, SelectedTypeUnit,
};
pub use scalar_operands::{
    CheckedScalarOperand, CheckedScalarOperandChild, CheckedScalarOperandError,
    CheckedScalarOperandRange,
};
pub use schedulability::{
    CheckedSchedulabilityClaimRef, CheckedSchedulabilityClaimRow, CheckedSchedulabilityKind,
    CheckedSchedulabilityTaskRef, CheckedSchedulingPolicy, CheckedTaskArrival,
    CheckedTaskCriticality,
};
pub use step_relation::{
    CheckedStepExecution, CheckedStepRelationBody, CheckedStepRelationCapability,
    CheckedStepRelationObligationIdentity,
};
pub use temporal_binding::{ClockConfiguration, ClockExactNumber, TemporalBinding};
pub use temporal_clause_names::TemporalClauseName;
pub use typed_terms::*;
pub use vocabulary::*;

use crate::identity::ExecutionPoint;
use closure_index::ClosureIndex;
use dependency_references::{admit_dependencies, check_derived_closure, SuppliedDependencies};
use graph_storage::GraphIndex;
use ix_trace_rs::string_edge;
use operations::{validate_operations, OperationNodeOrder};
use owner::{validate_owner_joins, validate_owner_schema};
pub(in crate::checked_package) use retained_index::{
    source_index as retained_source_positions, PositionIndex,
};
use source_map::validate_source_map_entries;
use structural::{validate_derived_type_shapes, validate_selected_type_bounds};
use temporal::{
    validate_hyperproperties, validate_temporal, validate_timed_bounds_reduced, TemporalContext,
};

use super::common::{
    count, exceeds, is_nonempty, node_pointer, validate_definition_ref, validate_locked_source,
    validate_term, Ids, ReferenceMember, ReferenceSite, ReferenceVisitor, Step, TermGrammar, Trail,
    ValidationFailure,
};
use super::evidence::CheckedPackageEvidence;
use super::shared::{
    CheckedArtifactRef, CheckedCapability, CheckedNodeId, CheckedNodePathSegment,
    CheckedNodeTableDebug, CheckedOccurrence, CheckedOccurrenceRole, CheckedPackageIncomplete,
    CheckedPackageLimit, CheckedPackageReadLimits, CheckedPackageRefusal,
    CheckedPackageRefusalCause, CheckedPackageRefusalCode, CheckedSelection, CheckedSourceMapEntry,
    CheckedSourceRef, CheckedSourceRegion, JsonPointer,
};
use super::terms::TermSource;
use lock_tables::{
    checked_unit_refs, no_unit_refs, resolve_model_durations, selected_model_roots,
    validate_dependency_selections, validate_refinement_internals, validate_required_features,
};
use model_fields::{build_field_tables, DeclarationIndex, RetainedModels};
use model_members::{
    admit_selection, Budget, DomainModel, ModelOwners, SelectionFailure, UnitRefs,
};
use quire_canonical::FixedShape;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

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
/// `DeclarationTagRules` forbids it; see `validate_declaration`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(deny_unknown_fields)]
pub struct CheckedDeclaration {
    /// ASCII identifier segments.
    #[serde(deserialize_with = "document_de::exact_vec")]
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
    #[serde(deserialize_with = "document_de::exact_vec")]
    pub dependencies: Vec<CheckedNodeId>,
    /// Exact source occurrence keys.
    #[serde(deserialize_with = "document_de::exact_vec")]
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
    /// Rewrite each typed node identity slot once in deterministic node order.
    ///
    /// `None` keeps the original identity. A replacement is not visited again;
    /// the package reader remains responsible for validating a rewritten graph.
    pub fn map_node_ids(&mut self, mut map: impl FnMut(&CheckedNodeId) -> Option<CheckedNodeId>) {
        fn visit(
            id: &mut CheckedNodeId,
            map: &mut impl FnMut(&CheckedNodeId) -> Option<CheckedNodeId>,
        ) {
            if let Some(replacement) = map(id) {
                *id = replacement;
            }
        }

        visit(&mut self.node_id, &mut map);
        visit(&mut self.semantic_type, &mut map);
        for dependency in &mut self.dependencies {
            visit(dependency, &mut map);
        }
        if let Some(owner_model) = &mut self.owner_model {
            visit(owner_model, &mut map);
        }
        self.body.map_node_ids(map);
    }

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
    #[serde(deserialize_with = "document_de::exact_vec")]
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
    #[serde(deserialize_with = "document_de::exact_vec")]
    pub internal: Vec<Box<str>>,
}

/// The exact immutable V2 package lock.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(deny_unknown_fields)]
pub struct CheckedPackageLockV2 {
    /// Locked raw source documents.
    #[serde(deserialize_with = "document_de::exact_vec")]
    pub sources: Vec<CheckedSourceRef>,
    /// Selected edition.
    pub edition: CheckedSelection,
    /// Selected profiles.
    #[serde(deserialize_with = "document_de::exact_vec")]
    pub profile_selections: Vec<CheckedSelection>,
    /// Selected definitions.
    #[serde(deserialize_with = "document_de::exact_vec")]
    pub definition_selections: Vec<CheckedArtifactRef>,
    /// Required features.
    #[serde(deserialize_with = "document_de::exact_vec")]
    pub required_features: Vec<Box<str>>,
    /// Dependency selections: one per library identity, ascending.
    #[serde(deserialize_with = "document_de::exact_vec")]
    pub dependency_selections: Vec<CheckedDependencySelection>,
    /// Nonempty refinement internal lists, omitted when no list is present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(deserialize_with = "document_de::exact_optional_vec")]
    pub refinement_internals: Option<Vec<CheckedRefinementInternal>>,
}

/// A checked V2 graph.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, bound(deserialize = "Body: Deserialize<'de>"))]
pub struct CheckedSemanticGraphV2<Body = CheckedTerm> {
    /// Must be `quire.checked-semantic-graph/v2`.
    pub graph_version: Box<str>,
    /// Complete closed graph.
    #[serde(deserialize_with = "document_de::exact_vec")]
    pub nodes: Vec<CheckedSemanticNodeV2<Body>>,
    /// One operand row per node, in graph order: slot `k` of node `i` names
    /// `nodes[operand_rows[i][k]]`. Required on read.
    #[serde(deserialize_with = "document_de::exact_rows")]
    pub operand_rows: Vec<Vec<u64>>,
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
    #[serde(deserialize_with = "document_de::exact_vec")]
    pub details: Vec<CheckedJson>,
    /// Source loci under locked sources.
    #[serde(deserialize_with = "document_de::exact_vec")]
    pub loci: Vec<CheckedSourceRegion>,
}

/// Typed V2 package diagnostics.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedDiagnosticsV2 {
    /// Diagnostic catalog definition.
    pub catalog: CheckedArtifactRef,
    /// Typed diagnostic records.
    #[serde(deserialize_with = "document_de::exact_vec")]
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
        struct CheckedPackageEnvelope<C, N, L, G, S, R, T, B, M, H, E, A, D> {
            $($name: $parameter,)+
            #[serde(default, skip_serializing_if = "Option::is_none")]
            refinement_step_rows: Option<R>,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            checked_term_data: Option<T>,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            temporal_bindings: Option<B>,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            temporal_clause_names: Option<M>,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            schedulability_claims: Option<H>,
            #[serde(default, deserialize_with = "operation_effects::present_rows", bound(deserialize = "E: Deserialize<'de>"), skip_serializing_if = "Option::is_none")]
            operation_effects: Option<E>,
        }

        impl<Body: quire_canonical::Encode> quire_canonical::Encode for CheckedPackageEnvelope<
            &str,
            &str,
            &CheckedPackageLockV2,
            &CheckedSemanticGraphV2<Body>,
            &[CheckedSourceMapEntry],
            &[refinement::CheckedRefinementStepRowRegion],
            &checked_term_data::CheckedTermDataV2,
            &[temporal_binding::TemporalBinding],
            &[TemporalClauseName],
            &[CheckedSchedulabilityClaimRow],
            &[CheckedOperationEffectRow],
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
                if let Some(data) = self.checked_term_data {
                    encode::encoded(writer, "checked_term_data", data)?;
                }
                if let Some(rows) = &self.temporal_bindings {
                    encode::elements(writer, "temporal_bindings", rows)?;
                }
                if let Some(rows) = &self.temporal_clause_names {
                    encode::elements(writer, "temporal_clause_names", rows)?;
                }
                if let Some(rows) = self.schedulability_claims {
                    encode::elements(writer, "schedulability_claims", rows)?;
                }
                if let Some(rows) = &self.operation_effects {
                    encode::elements(writer, "operation_effects", rows)?;
                }
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
    checked_term_data::CheckedTermDataV2,
    Vec<temporal_binding::TemporalBinding>,
    Vec<TemporalClauseName>,
    Vec<CheckedSchedulabilityClaimRow>,
    Vec<CheckedOperationEffectRow>,
    Vec<CheckedCapability>,
    CheckedDiagnosticsV2,
>;

type RawCheckedPackageWireV2 = CheckedPackageWireV2<Value>;

/// The wire as the charged Document-based decode produces it: node bodies and
/// diagnostic details are handles into the one retained package document, and
/// every container holds exactly its final length. The validators still read
/// [`RawCheckedPackageWireV2`] until each moves to the node API.
#[allow(dead_code)]
type DecodedWireV2 = CheckedPackageWireV2<CheckedJson>;

/// Source correspondence supplied to the canonical V2 writer.
#[derive(Clone, Copy, Debug)]
pub struct CheckedPackageSourceCorrespondenceRef<'a> {
    /// Node occurrence regions.
    pub source_map: &'a [CheckedSourceMapEntry],
    /// Canonical refinement step row regions, absent when no steps exist.
    pub refinement_step_rows: Option<&'a [CheckedRefinementStepRowRegion]>,
}

/// Optional typed sidecars supplied to the complete V2 writer.
#[derive(Clone, Copy, Debug)]
pub struct CheckedPackageSidecarsRef<'a> {
    /// Admitted application and dispatch decision rows, when present.
    pub checked_term_data: Option<&'a CheckedTermDataV2>,
    /// Exact temporal declaration clock rows, when present.
    pub temporal_bindings: Option<&'a [TemporalBinding]>,
    /// Exact temporal-clause occurrence names, when present.
    pub temporal_clause_names: Option<&'a [TemporalClauseName]>,
    /// Untrusted identity rows, preserved exactly as supplied.
    pub schedulability_claims: Option<&'a [CheckedSchedulabilityClaimRow]>,
    /// Selected model operation effect rows, absent when unavailable.
    pub operation_effects: Option<&'a [CheckedOperationEffectRow]>,
}

/// Canonically encodes a complete checked-package V2 envelope under the
/// caller's byte limits. The supplied name is the package's identity. An
/// absent checked-term, temporal, schedulability or operation-effect sidecar is omitted.
pub fn encode_checked_package_v2<Body: quire_canonical::Encode>(
    name: &str,
    lock: &CheckedPackageLockV2,
    semantic_graph: &CheckedSemanticGraphV2<Body>,
    sources: CheckedPackageSourceCorrespondenceRef<'_>,
    sidecars: CheckedPackageSidecarsRef<'_>,
    capability_report: &[CheckedCapability],
    diagnostics: &CheckedDiagnosticsV2,
    limits: quire_canonical::Limits,
) -> Result<Vec<u8>, quire_canonical::Error> {
    if semantic_graph.nodes.iter().any(|node| {
        [&node.node_id, &node.semantic_type]
            .into_iter()
            .chain(node.dependencies.iter())
            .any(|id| {
                id.path
                    .iter()
                    .any(|segment| matches!(segment, CheckedNodePathSegment::Node(_)))
            })
    }) {
        return Err(quire_canonical::Error::Internal {
            invariant: "checked-package V2 paths require graph-local operand indices",
        });
    }
    let envelope = CheckedPackageEnvelope {
        contract_version: CHECKED_PACKAGE_V2,
        name,
        lock,
        semantic_graph,
        source_map: sources.source_map,
        refinement_step_rows: sources.refinement_step_rows,
        checked_term_data: sidecars.checked_term_data,
        temporal_bindings: sidecars.temporal_bindings,
        temporal_clause_names: sidecars.temporal_clause_names,
        schedulability_claims: sidecars.schedulability_claims,
        operation_effects: sidecars.operation_effects,
        capability_report,
        diagnostics,
    };
    let bytes = quire_canonical::to_vec(&envelope, limits)?;
    if node_index::contains_inline_node_path(&bytes) {
        return Err(quire_canonical::Error::Internal {
            invariant: "checked-package V2 paths cannot embed node IDs",
        });
    }
    Ok(bytes)
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
    /// The reader's operation catalog is unreadable, so no package can be
    /// checked: a skew between the reader and its catalog dependency, not a
    /// verdict on the input.
    CatalogUnavailable(CatalogReadError),
}

/// The graph in its wire form and the positions of the nodes whose own ID
/// holds an operand slot, by those bytes.
struct SlotView {
    graph: CheckedSemanticGraphV2,
    /// The positions of the slot-holding nodes, sorted by node ID bytes and
    /// then position.
    positions: Vec<usize>,
}

/// Immutable, admitted `quire.checked-package/v2` data.
///
/// The closed body grammar and flat ownership of complete selected documents
/// keep native stack use independent of package size during clone, comparison,
/// `Debug` rendering, lowering and drop.
#[derive(Clone)]
pub struct CheckedPackageV2 {
    /// Private admission capability; clones preserve it, independent reads do not.
    admission_binding: Option<AdmissionBinding>,
    wire: Arc<CheckedPackageWireV2>,
    kinds: Arc<Vec<CheckedNodeKind>>,
    schedulability_claim_index: Vec<(usize, usize)>,
    monitor_plan_index: Vec<monitor_plan::Plan>,
    timed_model_roots: Vec<usize>,
    /// Admission-derived lookup; excluded from content equality.
    node_index: Arc<PositionIndex>,
    /// Graph-local wire IDs rebuilt as shared, document-independent keys.
    structural_keys: Vec<std::sync::Arc<CheckedNodeId>>,
    /// The graph in its wire form, node-held operands as operand slots,
    /// built on first use from the resolved graph in `wire`.
    slot_graph: Arc<std::sync::OnceLock<SlotView>>,
    /// Admission-derived position of each exact source occurrence.
    pub(in crate::checked_package) source_index: Arc<PositionIndex>,
    /// Body terms and resolved lowering edges, retained from graph admission.
    lowering_facts: Vec<lower::LoweringNodeFacts>,
    /// The byte limit this package was read under: the ceiling of every
    /// encode its lowering makes.
    bytes: u64,
    /// What the reader derived from the selected domain package documents and
    /// keeps: the declaration index, the field tables, the operation parameter
    /// types and the relationship records.
    models: Arc<RetainedModels>,
    /// Complete admitted selected JSON values are content, not derived caches.
    selected_documents: Arc<Vec<Arc<quire_canonical::Document>>>,
    /// Direct admitted edges; their recursively derived closure is not copied.
    direct_dependencies: Vec<std::sync::Arc<CheckedPackageV2>>,
    /// Exact identities reachable through the direct edges, shared with children.
    closure_index: ClosureIndex,
    /// The operation catalog this package was admitted against.
    catalog: operation_catalog::CatalogRef,
}

/// The original immutable buffers read by native identity validation. No
/// dependency packages are retained, preserving iterative dependency drop.
struct AdmissionBuffers {
    wire: Arc<CheckedPackageWireV2>,
    kinds: Arc<Vec<CheckedNodeKind>>,
    models: Arc<RetainedModels>,
    node_index: Arc<PositionIndex>,
    source_index: Arc<PositionIndex>,
    selected_documents: Arc<Vec<Arc<quire_canonical::Document>>>,
}

/// Opaque capability issued by one admitted V2 read.
///
/// Cloning preserves its admission identity. Equal package bytes read again
/// issue a distinct capability. Only [`CheckedPackageV2::admission_binding`]
/// can issue one, and [`CheckedPackageV2::retains`] checks membership.
#[derive(Clone)]
pub struct AdmissionBinding(Arc<AdmissionBuffers>);

impl std::fmt::Debug for AdmissionBinding {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AdmissionBinding")
            .finish_non_exhaustive()
    }
}

impl PartialEq for AdmissionBinding {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for AdmissionBinding {}

impl AdmissionBinding {
    fn retains(&self, package: &CheckedPackageV2) -> bool {
        let original = &self.0;
        Arc::ptr_eq(&original.wire, &package.wire)
            && Arc::ptr_eq(&original.kinds, &package.kinds)
            && Arc::ptr_eq(&original.models, &package.models)
            && Arc::ptr_eq(&original.node_index, &package.node_index)
            && Arc::ptr_eq(&original.source_index, &package.source_index)
            && Arc::ptr_eq(&original.selected_documents, &package.selected_documents)
    }
}

impl CheckedPackageV2 {
    /// Issues this admitted package's cloneable admission capability.
    ///
    /// Every package returned by [`Self::read`] has a binding. The private
    /// construction path for unit fixtures is outside that reader contract.
    pub fn admission_binding(&self) -> AdmissionBinding {
        self.admission_binding
            .as_ref()
            .expect("admitted package has an admission binding")
            .clone()
    }

    /// Returns whether this package retains the original admission represented
    /// by `binding`. Package clones retain membership; independent reads do not.
    pub fn retains(&self, binding: &AdmissionBinding) -> bool {
        self.admission_binding
            .as_ref()
            .is_some_and(|own| own == binding && binding.retains(self))
    }

    /// Finish internal construction by retaining the original immutable fact
    /// buffers. The public reader calls this only after strict admission;
    /// private unit fixture builders use the same construction invariant.
    /// Retains the original immutable fact buffers, their shared block charged to the
    /// read's budget first.
    fn retain_admission(
        mut self,
        budget: &mut NativeJoinBudget,
    ) -> Result<Self, ValidationFailure> {
        let site = read_charge::Site::new(PackageReadStage::Retained, 0, "admission_binding");
        let buffers = AdmissionBuffers {
            wire: Arc::clone(&self.wire),
            kinds: Arc::clone(&self.kinds),
            models: Arc::clone(&self.models),
            node_index: Arc::clone(&self.node_index),
            source_index: Arc::clone(&self.source_index),
            selected_documents: Arc::clone(&self.selected_documents),
        };
        self.admission_binding = Some(AdmissionBinding(read_charge::new_arc(
            budget, site, buffers,
        )?));
        Ok(self)
    }
}

impl std::fmt::Debug for CheckedPackageV2 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let nodes = CheckedNodeTableDebug::new(self.structural_keys.iter().map(AsRef::as_ref));
        formatter
            .debug_struct("CheckedPackageV2")
            .field("name", &self.wire.name)
            .field("nodes", &nodes)
            .field("kinds", &self.kinds)
            .field("source_count", &self.wire.source_map.len())
            .field("diagnostic_count", &self.wire.diagnostics.entries.len())
            .field("bytes", &self.bytes)
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

/// Equality includes each selected document's complete admitted JSON value,
/// using the numeric values consumed by model intake rather than authored
/// number spelling, object member order or wire whitespace.
/// The read limit retained for lowering is not content, so two admissions of
/// the same bytes under different limits are equal; nor are the retained field tables,
/// declaration index, operation parameter types and relationship records, which
/// are derived from the same admitted documents.
impl PartialEq for CheckedPackageV2 {
    fn eq(&self, other: &Self) -> bool {
        self.wire == other.wire
            && self.kinds == other.kinds
            && selection_equality::equal_selections(
                &self.selected_documents,
                &other.selected_documents,
            )
    }
}

impl Eq for CheckedPackageV2 {}

/// Cumulative validation work, charged either to the caller's budget (the
/// strict read) or against one limit of its own (lowering and unit tests).
#[derive(Debug)]
pub(super) struct WorkMeter<'b> {
    source: WorkSource<'b>,
    /// Where the graph stage's heap requests are charged.
    site: Option<graph_storage::ReadSite>,
    /// The node bodies as handles into the retained document, with the
    /// operand row of each node, once the graph stage has them.
    bodies: Option<graph_storage::BodyTable<'b>>,
}

#[derive(Debug)]
enum WorkSource<'b> {
    Own { consumed: u64, limit: u64 },
    Budget(&'b mut NativeJoinBudget),
}

impl read_charge::ReadStorage for WorkMeter<'_> {
    /// Heap requests charge the caller's budget; a meter with a limit of its
    /// own (lowering, unit tests) holds no read-storage account.
    fn reserve(&mut self, bytes: u128, site: read_charge::Site) -> Result<(), read_charge::Denied> {
        match &mut self.source {
            WorkSource::Own { .. } => Ok(()),
            WorkSource::Budget(budget) => budget.reserve(bytes, site),
        }
    }

    fn denial(&self) -> ValidationFailure {
        match &self.source {
            WorkSource::Own { .. } => ValidationFailure::Budget(BudgetStop {
                resource: PackageReadResource::ReadStorage,
                amount: 0,
                path: None,
            }),
            WorkSource::Budget(budget) => budgeted_read::denial(budget),
        }
    }
}

impl<'b> WorkMeter<'b> {
    pub(super) const fn new(limit: u64) -> Self {
        Self {
            source: WorkSource::Own { consumed: 0, limit },
            site: None,
            bodies: None,
        }
    }

    /// The caller's budget when this meter charges one (the strict read), so
    /// the read's heap requests are charged to it as well; `None` for a meter
    /// with a work limit of its own.
    pub(super) fn budget_mut(&mut self) -> Option<&mut NativeJoinBudget> {
        match &mut self.source {
            WorkSource::Own { .. } => None,
            WorkSource::Budget(budget) => Some(&mut **budget),
        }
    }

    /// A meter that charges the caller's budget work counter.
    pub(super) fn over(budget: &'b mut NativeJoinBudget) -> Self {
        Self {
            source: WorkSource::Budget(budget),
            site: None,
            bodies: None,
        }
    }

    pub(super) fn consumed(&self) -> u64 {
        match &self.source {
            WorkSource::Own { consumed, .. } => *consumed,
            WorkSource::Budget(budget) => budget.consumed().work,
        }
    }

    /// Charges `work` for the value at `at`, which an exhausted budget
    /// reports as the value whose charge failed.
    pub(super) fn charge(
        &mut self,
        work: u64,
        at: impl FnOnce() -> JsonPointer,
    ) -> Result<(), ValidationFailure> {
        match &mut self.source {
            WorkSource::Own { consumed, limit } => {
                *consumed = consumed.saturating_add(work);
                if exceeds(*consumed, *limit) {
                    #[cfg(test)]
                    work_probe::charge_denied();
                    Err(ValidationFailure::incomplete(
                        CheckedPackageLimit::Work,
                        *limit,
                        *consumed,
                        Some(at()),
                    ))
                } else {
                    Ok(())
                }
            }
            WorkSource::Budget(budget) => {
                if budget.charge(NativeJoinChargeKind::Work, u128::from(work)) {
                    Ok(())
                } else {
                    #[cfg(test)]
                    work_probe::charge_denied();
                    Err(ValidationFailure::Budget(BudgetStop {
                        resource: PackageReadResource::Work,
                        amount: u128::from(work),
                        path: Some(at()),
                    }))
                }
            }
        }
    }
}

#[cfg(test)]
mod work_probe {
    use std::cell::Cell;

    thread_local! {
        static DENIED: Cell<bool> = const { Cell::new(false) };
        static VISITS_AFTER_DENIAL: Cell<u64> = const { Cell::new(0) };
        static CONTINUE_AFTER_DENIAL: Cell<bool> = const { Cell::new(false) };
    }

    pub(super) fn reset() {
        DENIED.with(|denied| denied.set(false));
        VISITS_AFTER_DENIAL.with(|visits| visits.set(0));
        CONTINUE_AFTER_DENIAL.with(|mutant| mutant.set(false));
    }

    pub(super) fn charge_denied() {
        DENIED.with(|denied| denied.set(true));
    }

    pub(super) fn recursion_edge_visited() {
        if DENIED.with(Cell::get) {
            VISITS_AFTER_DENIAL.with(|visits| visits.set(visits.get().saturating_add(1)));
        }
    }

    pub(super) fn visits_after_denial() -> u64 {
        VISITS_AFTER_DENIAL.with(Cell::get)
    }

    pub(super) fn continue_after_denial(enabled: bool) {
        CONTINUE_AFTER_DENIAL.with(|mutant| mutant.set(enabled));
    }

    pub(super) fn mutant_enabled() -> bool {
        CONTINUE_AFTER_DENIAL.with(Cell::get)
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
        self.node_with_position(id).map(|(_, node)| node)
    }

    /// Return the retained graph position with the node from the same lookup.
    pub(in crate::checked_package) fn node_with_position(
        &self,
        id: &CheckedNodeId,
    ) -> Option<(usize, &CheckedSemanticNodeV2)> {
        let position = self.position_of(id)?;
        self.wire
            .semantic_graph
            .nodes
            .get(position)
            .inspect(|_| {
                #[cfg(test)]
                model_fields::probe::node_visited();
            })
            .filter(|candidate| node_index::holds_slot(id) || &candidate.node_id == id)
            .map(|node| (position, node))
    }

    fn admit_bytes(
        catalog: operation_catalog::CatalogRef,
        bytes: &[u8],
        limits: CheckedPackageReadLimits,
        evidence: &CheckedPackageEvidence,
        budget: &mut NativeJoinBudget,
    ) -> Result<Self, ValidationFailure> {
        // The document the shared reader built is the grammar and canonical
        // proof of the bytes; the typed decode below still reads the bytes.
        let document = budgeted_read::read_envelope(bytes, limits, budget)?;
        let document = budgeted_read::share_document(document, budget)?;
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
        if intake::source_value(bytes, &member_pointer(&["name"])).is_none() {
            return Err(refuse(
                CheckedPackageRefusalCode::MalformedWire,
                member_pointer(&["name"]),
            ));
        }
        let (mut wire, bodies) = intake::decode_borrowed_wire(bytes, &document, budget)
            .map_err(|failure| classify_dependency_entry_shape_bytes(failure, bytes))
            .map_err(classify_owner_shape)?;
        intake::check_lossless_source(bytes, &wire)?;
        // Every later stage compares operands by graph index, so each node-held
        // slot is read through its row once here.
        node_index::resolve_slots(&mut wire, budget)?;
        validate_owner_schema(&wire, &bodies, budget)?;
        let ValidatedPackage {
            kinds,
            schedulability_claim_index,
            monitor_plan_index,
            terms,
            models,
            selected_documents,
            lowering_facts,
            structural_keys,
        } = validate(
            &catalog, &wire, &bodies, &document, limits, evidence, budget,
        )?;
        let site = |form| read_charge::Site::new(PackageReadStage::Retained, 0, form);
        let selections = &wire.lock.dependency_selections;
        let mut direct_dependencies: Vec<Arc<CheckedPackageV2>> =
            read_charge::with_capacity(budget, site("direct_dependencies"), selections.len())?;
        for (index, selection) in selections.iter().enumerate() {
            direct_dependencies.push(
                evidence
                    .retained_dependency(&selection.identity)
                    .ok_or_else(|| {
                        ValidationFailure::refused_because(
                            CheckedPackageRefusalCode::MissingImport,
                            member_pointer(&["lock", "dependency_selections"]).index(index),
                            CheckedPackageRefusalCause::MissingSelection,
                        )
                    })?,
            );
        }
        let closure_index = ClosureIndex::from_direct(&direct_dependencies, budget)?;
        let CheckedPackageEnvelope {
            contract_version,
            name,
            lock,
            semantic_graph,
            source_map,
            refinement_step_rows,
            checked_term_data,
            temporal_bindings,
            temporal_clause_names,
            schedulability_claims,
            operation_effects,
            capability_report,
            diagnostics,
        } = wire;
        if semantic_graph.nodes.len() != terms.len() {
            return Err(refuse(
                CheckedPackageRefusalCode::MalformedWire,
                member_pointer(&["semantic_graph", "nodes"]),
            ));
        }
        let mut nodes: Vec<CheckedSemanticNodeV2> =
            read_charge::with_capacity(budget, site("semantic_graph"), terms.len())?;
        nodes.extend(
            semantic_graph
                .nodes
                .into_iter()
                .zip(terms)
                .map(|(node, term)| node.into_typed(term)),
        );
        let semantic_graph = CheckedSemanticGraphV2 {
            graph_version: semantic_graph.graph_version,
            nodes,
            operand_rows: semantic_graph.operand_rows,
        };
        let wire = CheckedPackageEnvelope {
            contract_version,
            name,
            lock,
            semantic_graph,
            source_map,
            refinement_step_rows,
            checked_term_data,
            temporal_bindings,
            temporal_clause_names,
            schedulability_claims,
            operation_effects,
            capability_report,
            diagnostics,
        };
        let node_index = retained_index::node_index(&wire, budget)?;
        let source_index = super::selected_population::source_index(&wire.source_map, budget)?;
        let is_timed_root =
            |kind: &CheckedNodeKind| *kind == CheckedNodeKind::TimedModel(TimedModelForm::Model);
        let mut timed_model_roots: Vec<usize> = read_charge::with_capacity(
            budget,
            site("timed_model_roots"),
            kinds.iter().filter(|kind| is_timed_root(kind)).count(),
        )?;
        timed_model_roots.extend(
            kinds
                .iter()
                .enumerate()
                .filter_map(|(position, kind)| is_timed_root(kind).then_some(position)),
        );
        let package = site("package");
        let wire = read_charge::new_arc(budget, package, wire)?;
        let kinds = read_charge::new_arc(budget, package, kinds)?;
        let models = read_charge::new_arc(budget, package, models)?;
        let node_index = read_charge::new_arc(budget, package, node_index)?;
        let source_index = read_charge::new_arc(budget, package, source_index)?;
        let selected_documents = read_charge::new_arc(budget, package, selected_documents)?;
        let slot_graph = read_charge::new_arc(budget, package, std::sync::OnceLock::new())?;
        let package = Self {
            admission_binding: None,
            wire,
            kinds,
            schedulability_claim_index,
            monitor_plan_index,
            timed_model_roots,
            node_index,
            structural_keys,
            slot_graph,
            source_index,
            lowering_facts,
            bytes: limits.bytes,
            models,
            selected_documents,
            direct_dependencies,
            closure_index,
            catalog,
        };
        let package = package.retain_admission(budget)?;
        read_charge::charge(budget, site("result"), std::mem::size_of::<Self>() as u128)?;
        Ok(package)
    }

    /// The package's exact name identity.
    pub fn name(&self) -> &str {
        &self.wire.name
    }

    /// The exact immutable lock.
    pub fn lock(&self) -> &CheckedPackageLockV2 {
        &self.wire.lock
    }

    /// Selected domain package identities in ascending identity order.
    ///
    /// These are the admitted model-root selections. The checked package does
    /// not carry source model aliases; an empty iterator means no model was
    /// selected. A refused or incomplete read yields no package to query.
    pub fn selected_model_packages(&self) -> impl ExactSizeIterator<Item = &str> {
        self.models
            .selected_models()
            .iter()
            .map(|model| model.identity.as_ref())
    }

    /// The admitted direct dependency packages in lock selection order.
    /// Dependencies selected by a child are available from that child's accessor.
    pub fn direct_dependencies(&self) -> impl ExactSizeIterator<Item = &CheckedPackageV2> {
        self.direct_dependencies.iter().map(std::sync::Arc::as_ref)
    }

    /// The complete closed checked graph as the wire holds it: each
    /// node-held operand is an operand slot read through the node's row of
    /// `operand_rows`, each document-held one an index segment.
    pub fn graph(&self) -> &CheckedSemanticGraphV2 {
        &self.slot_view().graph
    }

    fn slot_view(&self) -> &SlotView {
        self.slot_graph.get_or_init(|| {
            let graph = node_index::slot_graph(&self.wire.semantic_graph);
            let nodes = &graph.nodes;
            let mut positions: Vec<usize> = (0..nodes.len())
                .filter(|&position| node_index::holds_slot(&nodes[position].node_id))
                .collect();
            positions.sort_unstable_by(|left, right| {
                nodes[*left]
                    .node_id
                    .cmp(&nodes[*right].node_id)
                    .then(left.cmp(right))
            });
            SlotView { graph, positions }
        })
    }

    /// The graph position of `id` by the retained node index.
    fn node_position(&self, id: &CheckedNodeId) -> Option<usize> {
        self.node_index.node(&self.wire.semantic_graph.nodes, id)
    }

    /// The graph's node identities in ascending order.
    pub(in crate::checked_package) fn node_ids_ascending(
        &self,
    ) -> impl Iterator<Item = &CheckedNodeId> {
        self.node_index.node_ids(&self.wire.semantic_graph.nodes)
    }

    /// The source-map position of one occurrence by the retained source index.
    pub(in crate::checked_package) fn source_position(
        &self,
        id: &CheckedNodeId,
        role: &CheckedOccurrenceRole,
        ordinal: u64,
    ) -> Option<usize> {
        self.source_index
            .source(&self.wire.source_map, id, role, ordinal)
    }

    /// The typed body of the node at `position`.
    pub(in crate::checked_package) fn term_at(&self, position: usize) -> Option<&CheckedTerm> {
        self.wire
            .semantic_graph
            .nodes
            .get(position)
            .map(|node| &node.body)
    }

    /// The graph position of `id`: a resolved index-form ID by the retained
    /// index, or a node ID as `graph()` shows it, operand slots included, when
    /// exactly one node has those bytes. Equal bytes name different nodes
    /// when their rows differ, so such an ID names none.
    fn position_of(&self, id: &CheckedNodeId) -> Option<usize> {
        if node_index::holds_slot(id) {
            let view = self.slot_view();
            let nodes = &view.graph.nodes;
            let first = view
                .positions
                .partition_point(|&position| nodes[position].node_id < *id);
            let found = view
                .positions
                .get(first)
                .filter(|&&position| nodes[position].node_id == *id)?;
            // Equal bytes in two nodes name none: the rows tell them apart.
            let unique = view
                .positions
                .get(first + 1)
                .is_none_or(|&next| nodes[next].node_id != *id);
            return unique.then_some(*found);
        }
        self.node_position(id)
    }

    /// The graph with every node-held operand resolved to the index segment of
    /// the node it names, the form the reader checks and the typed views
    /// borrow from. Its `operand_rows` are the wire's.
    pub fn resolved_graph(&self) -> &CheckedSemanticGraphV2 {
        &self.wire.semantic_graph
    }

    /// The graph node's document-independent structural key. Operand keys
    /// share storage, regardless of the wire indices chosen by this document.
    pub fn structural_key_at(&self, index: usize) -> Option<&CheckedNodeId> {
        self.structural_keys.get(index).map(std::sync::Arc::as_ref)
    }

    /// The ID of the graph node at `position` as a document-held ID, the form
    /// a request, report or sidecar row names a node by: each operand is the
    /// index segment of the node it names rather than a slot of the node's row.
    pub fn document_held_id(&self, position: usize) -> Option<&CheckedNodeId> {
        self.wire
            .semantic_graph
            .nodes
            .get(position)
            .map(|node| &node.node_id)
    }

    /// The structural key of a node-held ID written in the node at graph
    /// `holder`, as `graph()` shows it: each operand slot is read through the
    /// holder's operand row, each index segment against the graph. `None` when
    /// a slot or index names no node.
    pub fn resolve_node_held(&self, holder: usize, id: &CheckedNodeId) -> Option<CheckedNodeId> {
        let row = self.wire.semantic_graph.operand_rows.get(holder)?;
        let mut path = Vec::with_capacity(id.path.len());
        for segment in &id.path {
            path.push(match segment {
                CheckedNodePathSegment::Operand(slot) => {
                    let entry = row.get(usize::try_from(*slot).ok()?)?;
                    CheckedNodePathSegment::Node(Arc::clone(
                        self.structural_keys.get(usize::try_from(*entry).ok()?)?,
                    ))
                }
                CheckedNodePathSegment::Index(index) => CheckedNodePathSegment::Node(Arc::clone(
                    self.structural_keys.get(usize::try_from(*index).ok()?)?,
                )),
                other => other.clone(),
            });
        }
        Some(CheckedNodeId {
            package: id.package.clone(),
            path,
            ordinal: id.ordinal,
        })
    }

    /// Resolve an ID written by this package against this package's graph.
    /// The ID's `package` names its owner and does not select the graph.
    pub fn structural_key(&self, id: &CheckedNodeId) -> Option<&CheckedNodeId> {
        self.structural_key_at(self.position_of(id)?)
    }

    /// Each graph node's decoded family and form, in graph order.
    pub fn node_kinds(&self) -> &[CheckedNodeKind] {
        &self.kinds
    }

    /// The retained admitted family and form of one graph node by its ID.
    /// Returns `None` for an ID absent from this package.
    pub fn node_kind(&self, id: &CheckedNodeId) -> Option<CheckedNodeKind> {
        self.kinds.get(self.position_of(id)?).copied()
    }

    /// The typed body of an admitted node.
    pub fn term(&self, id: &CheckedNodeId) -> Option<&CheckedTerm> {
        self.term_at(self.position_of(id)?)
    }

    /// The exact source map.
    pub fn source_map(&self) -> &[CheckedSourceMapEntry] {
        &self.wire.source_map
    }

    /// Source regions of canonical refinement step rows, when steps exist.
    pub fn refinement_step_rows(&self) -> Option<&[CheckedRefinementStepRowRegion]> {
        self.wire.refinement_step_rows.as_deref()
    }

    /// Admitted checked-term sidecar, absent when the producer supplied none.
    pub fn checked_term_data(&self) -> Option<&CheckedTermDataV2> {
        self.wire.checked_term_data.as_ref()
    }

    /// Admitted temporal clock rows, keyed by exact clause or compensation ID.
    pub fn temporal_bindings(&self) -> Option<&[TemporalBinding]> {
        self.wire.temporal_bindings.as_deref()
    }

    /// Borrow exact admitted temporal-clause occurrence names.
    pub fn temporal_clause_names(&self) -> Option<&[TemporalClauseName]> {
        self.wire.temporal_clause_names.as_deref()
    }

    /// Admitted selected operation effects, absent when the producer supplied no carrier.
    pub fn operation_effects(
        &self,
    ) -> Option<impl ExactSizeIterator<Item = CheckedOperationEffectRef<'_>>> {
        self.wire
            .operation_effects
            .as_ref()
            .map(|rows| rows.iter().map(CheckedOperationEffectRef::admitted))
    }

    /// Admitted producer operation-effect rows for borrowed typed re-encoding.
    ///
    /// Absence means the optional carrier was omitted; a present slice preserves
    /// the selected row order and all admitted effect data.
    pub fn operation_effect_rows(&self) -> Option<&[CheckedOperationEffectRow]> {
        self.wire.operation_effects.as_deref()
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
    schedulability_claim_index: Vec<(usize, usize)>,
    monitor_plan_index: Vec<monitor_plan::Plan>,
    terms: Vec<CheckedTerm>,
    models: RetainedModels,
    selected_documents: Vec<Arc<quire_canonical::Document>>,
    lowering_facts: Vec<lower::LoweringNodeFacts>,
    structural_keys: Vec<std::sync::Arc<CheckedNodeId>>,
}

struct ValidatedGraph {
    kinds: Vec<CheckedNodeKind>,
    schedulability_claim_index: Vec<(usize, usize)>,
    monitor_plan_index: Vec<monitor_plan::Plan>,
    terms: Vec<CheckedTerm>,
    declarations: DeclarationIndex,
    lowering_facts: Vec<lower::LoweringNodeFacts>,
}

/// The graph's node index, still a `BTreeMap` until the sorted `GraphIndex`
/// replaces it, answers the extension joins' lookup.
impl node_lookup::NodeLookup for BTreeMap<&CheckedNodeId, usize> {
    fn position(&self, id: &CheckedNodeId) -> Option<&usize> {
        self.get(id)
    }
}

fn validate(
    catalog: &operation_catalog::OperationCatalog,
    wire: &RawCheckedPackageWireV2,
    bodies: &[CheckedJson],
    document: &quire_canonical::Document,
    limits: CheckedPackageReadLimits,
    evidence: &CheckedPackageEvidence,
    budget: &mut NativeJoinBudget,
) -> Result<ValidatedPackage, ValidationFailure> {
    check_package_header(wire)?;
    // The body grammar is strict wire validation: a body outside it refuses
    // here, before any identity is recomputed (FR-038-AC-116).
    let flat_kinds = flat_wire::check(wire, bodies, budget)?;
    let structural_keys = node_index::check(document, wire, evidence, budget)?;
    flat_wire::check_interval_bounds(wire, bodies, &flat_kinds, budget)?;
    let mut meter = WorkMeter::over(budget);
    meter.attach_bodies(bodies, &wire.semantic_graph.operand_rows);
    let LockAdmission {
        models,
        documents,
        dependencies,
    } = validate_lock(wire, evidence, &mut meter, limits.bytes)?;
    let ValidatedGraph {
        kinds,
        schedulability_claim_index,
        monitor_plan_index,
        terms,
        declarations,
        lowering_facts,
    } = validate_graph(
        catalog,
        wire,
        bodies,
        limits,
        &mut meter,
        &models,
        &dependencies,
        evidence,
        &structural_keys,
    )?;
    validate_source_map_entries(
        wire.semantic_graph
            .nodes
            .iter()
            .map(|node| (&node.node_id, node.occurrences.as_slice())),
        &wire.source_map,
        &wire.lock.sources,
        limits,
        &mut meter,
    )?;
    identity::validate_equivalence_occurrences(wire, &kinds, &mut meter)?;
    extension::validate_source_regions(wire, &kinds, &mut meter)?;
    validate_capabilities(wire, evidence, &mut meter)?;

    validate_diagnostics(wire, limits, &mut meter)?;
    let models = RetainedModels::build(declarations, models, &mut Budget::node(&mut meter, 0))?;
    operation_effects::validate(wire.operation_effects.as_deref(), &models, &mut meter)?;
    if let Some(data) = wire.checked_term_data.as_ref() {
        dispatch_projection::validate_checked_term_data(
            data, wire, &kinds, &terms, &models, &mut meter,
        )?;
    } else if kinds.contains(&CheckedNodeKind::Claim(ClaimForm::ProbabilisticClaim)) {
        return Err(ValidationFailure::refused_because(
            CheckedPackageRefusalCode::InvalidPackage,
            JsonPointer::root().key("checked_term_data"),
            CheckedPackageRefusalCause::InvalidValue,
        ));
    }
    Ok(ValidatedPackage {
        kinds,
        schedulability_claim_index,
        monitor_plan_index,
        terms,
        models,
        selected_documents: documents,
        lowering_facts,
        structural_keys,
    })
}

/// What the lock stage admits: each selected domain package's declarations,
/// in lock order, and each selected dependency's supplied package.
struct LockAdmission<'a> {
    models: Vec<DomainModel>,
    documents: Vec<Arc<quire_canonical::Document>>,
    dependencies: SuppliedDependencies<'a>,
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
    validate_dependency_selections(&lock.dependency_selections, meter)?;
    validate_refinement_internals(lock.refinement_internals.as_deref(), meter)?;
    for (index, definition) in lock.definition_selections.iter().enumerate() {
        validate_definition_ref(definition, &|| {
            member_pointer(&["lock", "definition_selections"]).index(index)
        })?;
    }
    let roots = selected_model_roots(&wire.semantic_graph.nodes, meter)?;
    let unit_refs = if roots.is_empty() {
        no_unit_refs(meter)?
    } else {
        let duration_terms = identity::duration_terms(&wire.semantic_graph.nodes, meter)?;
        checked_unit_refs(&wire.semantic_graph.nodes, &duration_terms, meter)?
    };
    let (mut models, documents) =
        validate_domain_packages(&roots, &wire.name, evidence, meter, bytes, &unit_refs)?;
    resolve_model_durations(&unit_refs, &mut models, &wire.semantic_graph.nodes, meter)?;
    // Implements: FR-322. Every selected dependency's admitted package is supplied and
    // binds to its entry, before any `dependency_reference` is read.
    let dependencies = admit_dependencies(&lock.dependency_selections, evidence, meter)?;
    check_derived_closure(
        &lock.dependency_selections,
        &dependencies.packages(),
        JsonPointer::root(),
        meter,
    )?;
    validate_required_features(&lock.required_features, meter)?;
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
        documents,
        dependencies,
    })
}

/// Admits the document supplied under each selected domain package identity
/// (FR-322 "Model-owned members" step 1) and returns its declarations, in
/// ascending identity order. Each document is charged to `meter` at its
/// root's owner (`/semantic_graph/nodes/<i>/owner`), and an exhausted `work`
/// limit is `incomplete` there.
fn validate_domain_packages(
    roots: &[(&str, usize)],
    checked_package: &str,
    evidence: &CheckedPackageEvidence,
    meter: &mut WorkMeter,
    bytes: u64,
    unit_refs: &UnitRefs,
) -> Result<(Vec<DomainModel>, Vec<Arc<quire_canonical::Document>>), ValidationFailure> {
    let row = |root: usize| node_pointer(root).key("owner");
    let at = |root: usize, member: &str| row(root).key(member);
    roots
        .iter()
        .enumerate()
        .map(|(position, (identity, root))| {
            let root = *root;
            let mut budget =
                Budget::new(meter, root, bytes).at(PackageReadStage::ModelDocument, position);
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

/// [`validate_node_id_charged`] under a budget that charges nothing, for the
/// graph validators whose stage has not yet threaded the read's budget here.
fn validate_node_id(
    id: &CheckedNodeId,
    at: impl FnOnce() -> JsonPointer,
) -> Result<(), ValidationFailure> {
    validate_node_id_charged(
        id,
        at,
        &mut NativeJoinBudget::unlimited(),
        read_charge::Site::new(PackageReadStage::Graph, 0, "node_id"),
    )
}

/// Whether a structural node identity and all nested operand identities are
/// within the wire bounds. The worklist of nested operands is the only
/// allocation, charged at `site`; a plain identity allocates nothing.
fn node_id_is_valid(
    id: &CheckedNodeId,
    budget: &mut NativeJoinBudget,
    site: read_charge::Site,
) -> Result<bool, ValidationFailure> {
    const MAX_ORDINAL: u64 = 9_007_199_254_740_991;
    let mut pending: Vec<&CheckedNodeId> = Vec::new();
    let mut candidate = id;
    loop {
        if !is_nonempty(&candidate.package)
            || candidate.path.is_empty()
            || candidate.ordinal > MAX_ORDINAL
        {
            return Ok(false);
        }
        for segment in &candidate.path {
            if let CheckedNodePathSegment::Node(child) = segment {
                read_charge::push(budget, site, &mut pending, child.as_ref())?;
            }
        }
        match pending.pop() {
            Some(next) => candidate = next,
            None => return Ok(true),
        }
    }
}

/// Checks a structural node identity and all nested operand identities. The
/// pointer of a refusal is built only for a defective identity.
fn validate_node_id_charged(
    id: &CheckedNodeId,
    at: impl FnOnce() -> JsonPointer,
    budget: &mut NativeJoinBudget,
    site: read_charge::Site,
) -> Result<(), ValidationFailure> {
    if node_id_is_valid(id, budget, site)? {
        return Ok(());
    }
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

/// A declared group member keeps its own independently checked path. An
/// anonymous derived member uses that path only when the label has exactly one
/// declared member. With zero or several declarations its path is authored;
/// the ordinary ID shape, uniqueness and reference rules still apply.
fn validate_recursion_group_paths(nodes: &[RawSemanticNodeV2]) -> Result<(), ValidationFailure> {
    let mut sole_declaration = BTreeMap::<&str, Option<&CheckedNodeId>>::new();
    for node in nodes {
        if let (Some(label), Some(_)) = (node.recursion_group.as_deref(), &node.declaration) {
            sole_declaration
                .entry(label)
                .and_modify(|anchor| *anchor = None)
                .or_insert(Some(&node.node_id));
        }
    }
    for (position, node) in nodes.iter().enumerate() {
        let Some(label) = node.recursion_group.as_deref() else {
            continue;
        };
        if node.declaration.is_some() {
            continue;
        }
        let Some(anchor) = sole_declaration.get(label).copied().flatten() else {
            continue;
        };
        if node.node_id.path != anchor.path {
            return Err(refuse(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                node_pointer(position).key("node_id").key("path"),
            ));
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
        K::ScalarType(
            ScalarTypeForm::CompoundUnit
            | ScalarTypeForm::CarriedQuantity
            | ScalarTypeForm::BoundedQuantity,
        ) => true,
        K::CompositeType(
            CompositeTypeForm::Option
            | CompositeTypeForm::Sequence
            | CompositeTypeForm::Set
            | CompositeTypeForm::Bag
            | CompositeTypeForm::OrderedSet
            | CompositeTypeForm::Map
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
            | TemporalForm::Terminal
            | TemporalForm::MonitorPlan,
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
        K::Claim(ClaimForm::Schedulability) => true,
        K::Claim(
            ClaimForm::VerificationClaim
            | ClaimForm::AnalysisClaim
            | ClaimForm::Hyperproperty
            | ClaimForm::SynthesisRequest
            | ClaimForm::ProbabilisticClaim,
        ) => false,
        K::Correspondence(
            CorrespondenceForm::SourceLocus
            | CorrespondenceForm::ModelCorrespondence
            | CorrespondenceForm::BindingRole
            | CorrespondenceForm::ProfileCorrespondence
            | CorrespondenceForm::AbstractionRelation
            | CorrespondenceForm::ImplementationGuarantee
            | CorrespondenceForm::ComponentComposition,
        ) => true,
    }
}

/// Decode a validated body directly from the retained package document.
fn decode_admitted_body(
    kind: CheckedNodeKind,
    body: &CheckedJson,
    row: &[u64],
    position: usize,
    meter: &mut WorkMeter,
) -> Result<CheckedTerm, ValidationFailure> {
    let at = || node_pointer(position).key("body");
    let node = TermNode::of(body);
    if source_nominal_body(kind, node.get("term").and_then(TermNode::as_str)) {
        let nominal = identity::NominalBody::from_document(node, row, &at, meter)?;
        meter.reserve(std::mem::size_of::<identity::NominalBody>())?;
        return Ok(CheckedTerm::NominalBody(Box::new(nominal)));
    }
    let site = read_charge::Site::new(PackageReadStage::Graph, position, kind.tag().as_wire());
    // Every graph read owns its caller's budget; authored V1 bodies use
    // contract_package's separate decoder and never enter this function.
    let budget = meter
        .budget_mut()
        .expect("the strict graph decoder has a caller budget");
    let result = if kind == CheckedNodeKind::State(StateForm::OperationAnchor) {
        node.decode_anchor_in(body.document(), budget, site, row)?
    } else {
        node.decode_in(body.document(), budget, site, row)?
    };
    result.map_err(|_| refuse(CheckedPackageRefusalCode::MalformedWire, at()))
}

#[ix_trace_rs::string_edge]
fn source_nominal_body(kind: CheckedNodeKind, term: Option<&str>) -> bool {
    matches!(
        (kind, term),
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
fn validate_body<'a, V: TermSource<'a>>(
    kind: CheckedNodeKind,
    body: &'a Value,
    node: V,
    at: &Trail<'a>,
    visit: &mut ReferenceVisitor<'_>,
    ids: Ids<'_, '_>,
    nominal: Option<&identity::NominalBody>,
) -> Result<u64, ValidationFailure> {
    #[cfg(test)]
    lower::BODY_WALKS.with(|walks| walks.set(walks.get() + 1));
    if kind == CheckedNodeKind::CompositeType(CompositeTypeForm::Map) {
        let members = body.get("members").and_then(Value::as_array);
        let valid = body.get("term").and_then(Value::as_str) == Some("aggregate")
            && members.is_some_and(|members| {
                matches!(members.as_slice(), [key, value]
                    if key.get("term").and_then(Value::as_str) == Some("binding")
                        && key.get("name").and_then(Value::as_str) == Some("key")
                        && key.get("value").and_then(|value| value.get("term"))
                            .and_then(Value::as_str) == Some("reference")
                        && value.get("term").and_then(Value::as_str) == Some("binding")
                        && value.get("name").and_then(Value::as_str) == Some("value")
                        && value.get("value").and_then(|value| value.get("term"))
                            .and_then(Value::as_str) == Some("reference"))
            });
        if !valid {
            return Err(refuse(
                CheckedPackageRefusalCode::MalformedWire,
                at.pointer(),
            ));
        }
    }
    if let Some(nominal) = nominal {
        let site = ReferenceSite {
            member: ReferenceMember::Target,
            is_body_root: false,
        };
        match nominal.kind() {
            identity::NominalBodyKind::Enum { .. } => {}
            identity::NominalBodyKind::Dimension { terms, .. } => {
                for (index, term) in terms.iter().enumerate() {
                    visit(
                        &term.dimension_node_id,
                        site,
                        &at.key("terms").index(index).key("dimension_node_id"),
                    );
                }
            }
            identity::NominalBodyKind::Unit {
                dimension_node_id,
                target_unit_node_id,
                ..
            } => {
                visit(dimension_node_id, site, &at.key("dimension_node_id"));
                if let Some(target) = target_unit_node_id {
                    visit(target, site, &at.key("target_unit_node_id"));
                }
            }
            identity::NominalBodyKind::DimensionEquivalence {
                dimension_node_ids, ..
            } => {
                for (index, operand) in dimension_node_ids.iter().enumerate() {
                    if let identity::CheckedDimensionOperand::Local(id) = operand {
                        visit(id, site, &at.key("dimension_node_ids").index(index));
                    }
                }
            }
        }
        return Ok(1 + match nominal.kind() {
            identity::NominalBodyKind::Dimension { terms, .. } => terms.len() as u64,
            identity::NominalBodyKind::DimensionEquivalence { .. } => 2,
            identity::NominalBodyKind::Unit { .. } => 2,
            identity::NominalBodyKind::Enum { members, .. } => members.len() as u64,
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
        BodyGrammar::Literal if super::common::body_term(body) != Some(BodyTerm::Literal) => Err(
            refuse(CheckedPackageRefusalCode::MalformedWire, at.pointer()),
        ),
        BodyGrammar::Term if kind == CheckedNodeKind::State(StateForm::OperationAnchor) => {
            execution_point::validate_anchor_body(node, at, visit, ids)
        }
        BodyGrammar::Extension => {
            extension::read_body(node, at.pointer(), ids).map(|body| extension::body_work(&body))
        }
        BodyGrammar::Frame => frame::read_frame_body_legacy(body, at).map(|(_, work)| work),
        BodyGrammar::AbstractionRelation | BodyGrammar::FlatCorrespondence => {
            abstraction::scan_body(node, at, visit, ids)
        }
        BodyGrammar::Refinement => refinement::validate_body(node, at, visit, ids),
        // `body` is the node's own top-level term, never a nested one, so
        // this is the one call in the module that reports `is_body_root: true`.
        BodyGrammar::Literal
        | BodyGrammar::Term
        | BodyGrammar::TimedModel
        | BodyGrammar::Hybrid => validate_term(node, TermGrammar::V2, true, at, visit, ids),
    }
}

/// [`validate_body`] for a body held as a `Value` and no budget: the V1
/// reader's, and the re-walk that finds a reference's pointer.
fn validate_value_body(
    kind: CheckedNodeKind,
    body: &Value,
    at: &Trail<'_>,
    visit: &mut ReferenceVisitor<'_>,
) -> Result<u64, ValidationFailure> {
    let nominal = if source_nominal_body(kind, body.get("term").and_then(Value::as_str)) {
        Some(identity::NominalBody::from_value(body, at.pointer())?)
    } else {
        None
    };
    super::common::with_unmetered_ids(|ids| {
        validate_body(kind, body, body, at, visit, ids, nominal.as_ref())
    })
}

/// The grammar a node's `body` is validated against (QSpec FR-322
/// `BodyBindingRules`): extension nodes use FR-469's declaration body,
/// `state`/`frame` nodes use the frame reference triple,
/// `correspondence`/`abstraction_relation` nodes use FR-346's flat shape,
/// `value`/`literal` nodes use a single literal, and every other node's body
/// is a `SemanticTerm`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BodyGrammar {
    /// A `value`/`literal` body contains exactly one literal.
    Literal,
    /// The closed extension declaration body.
    Extension,
    /// The closed `SemanticTerm` grammar.
    Term,
    /// The frame body of FR-040.
    Frame,
    /// The abstraction relation body of FR-346.
    AbstractionRelation,
    /// The implementation guarantee and component composition bodies of
    /// FR-038: flat and closed like the abstraction relation body, read by
    /// their own step.
    FlatCorrespondence,
    /// The closed refinement body of FR-375.
    Refinement,
    /// The closed nested hybrid model, mode or transition shape.
    Hybrid,
    /// Closed timed-model bodies.
    TimedModel,
}

/// Every form is listed explicitly,
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
            | ScalarTypeForm::CompoundUnit
            | ScalarTypeForm::CarriedQuantity
            | ScalarTypeForm::BoundedQuantity,
        ) => BodyGrammar::Term,
        K::CompositeType(
            CompositeTypeForm::Option
            | CompositeTypeForm::Sequence
            | CompositeTypeForm::Set
            | CompositeTypeForm::Bag
            | CompositeTypeForm::OrderedSet
            | CompositeTypeForm::Map
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
        K::Value(ValueForm::Literal) => BodyGrammar::Literal,
        K::Value(
            ValueForm::EnumValue
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
            | TemporalForm::Terminal
            | TemporalForm::MonitorPlan,
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
            | ClaimForm::StepRelation
            | ClaimForm::Schedulability
            | ClaimForm::ProbabilisticClaim,
        ) => BodyGrammar::Term,
        K::Correspondence(CorrespondenceForm::AbstractionRelation) => {
            BodyGrammar::AbstractionRelation
        }
        K::Correspondence(
            CorrespondenceForm::ImplementationGuarantee | CorrespondenceForm::ComponentComposition,
        ) => BodyGrammar::FlatCorrespondence,
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
    index: &GraphIndex<'_>,
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
    let _ = validate_value_body(
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

/// Compare a selected import's complete content after resolving each graph-
/// local operand in its own document. Both packages may choose different node
/// order and thus different wire index spellings for the same structural key.
fn imported_node_matches(
    importer: &RawSemanticNodeV2,
    selected: &RawSemanticNodeV2,
    importer_keys: &[std::sync::Arc<CheckedNodeId>],
    selected_keys: &[std::sync::Arc<CheckedNodeId>],
) -> bool {
    let same_id = |left: &CheckedNodeId, right: &CheckedNodeId| match (
        node_index::resolve_id(left, importer_keys),
        node_index::resolve_id(right, selected_keys),
    ) {
        (Some(left), Some(right)) => left == right,
        _ => false,
    };
    same_id(&importer.node_id, &selected.node_id)
        && importer.schema_version == selected.schema_version
        && importer.node_tag == selected.node_tag
        && importer.semantic_form == selected.semantic_form
        && same_id(&importer.semantic_type, &selected.semantic_type)
        && importer.dependencies.len() == selected.dependencies.len()
        && importer
            .dependencies
            .iter()
            .zip(&selected.dependencies)
            .all(|(left, right)| same_id(left, right))
        && importer.occurrences == selected.occurrences
        && importer.recursion_group == selected.recursion_group
        && importer.declaration == selected.declaration
        && importer.owner == selected.owner
        && match (&importer.owner_model, &selected.owner_model) {
            (Some(left), Some(right)) => same_id(left, right),
            (None, None) => true,
            _ => false,
        }
        && node_index::equivalent_values(
            &importer.body,
            &selected.body,
            importer_keys,
            selected_keys,
        )
}

fn structural_key<'a>(
    id: &'a CheckedNodeId,
    index: &GraphIndex<'_>,
    structural_keys: &'a [std::sync::Arc<CheckedNodeId>],
) -> &'a CheckedNodeId {
    index
        .get(id)
        .and_then(|&position| structural_keys.get(position))
        .map_or(id, std::sync::Arc::as_ref)
}

// `bodies` is the node-view table: `TermNode::of(&bodies[position])` is the
// body of graph node `position`, read from the retained document with the
// node's operand slots resolved through `wire.semantic_graph.operand_rows`
// (`TermNode::node_id_in`, `TermNode::decode_in`). The `Value` in
// `wire.semantic_graph.nodes[position].body` is the uncharged bridge for a
// validator that has not moved to the views.
#[allow(unused_variables)]
#[allow(clippy::too_many_arguments)] // the catalog and the node bodies are two inputs of one graph check
fn validate_graph(
    catalog: &operation_catalog::OperationCatalog,
    wire: &RawCheckedPackageWireV2,
    bodies: &[CheckedJson],
    limits: CheckedPackageReadLimits,
    meter: &mut WorkMeter,
    models: &[DomainModel],
    dependencies: &SuppliedDependencies<'_>,
    evidence: &CheckedPackageEvidence,
    structural_keys: &[std::sync::Arc<CheckedNodeId>],
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
    meter.at_node(0, graph_storage::node_form(&graph.nodes[0].node_tag));
    let index = GraphIndex::build(graph.nodes.iter().map(|node| &node.node_id), meter)?
        .with_ranks(structural_keys, meter)?;
    let repeated_id = index.first_repeat();
    let mut kinds = meter.vec(graph.nodes.len())?;
    let mut references: Vec<Vec<BodyReference>> = meter.vec(graph.nodes.len())?;
    let mut body_terms = meter.vec(graph.nodes.len())?;
    let mut terms = meter.vec(graph.nodes.len())?;
    // Each frame node's typed body, read once here and joined by the frame step.
    let frame_nodes = graph
        .nodes
        .iter()
        .filter(|node| {
            CheckedNodeTag::from_wire(&node.node_tag)
                .and_then(|tag| CheckedNodeKind::decode(tag, &node.semantic_form))
                == Some(CheckedNodeKind::State(StateForm::Frame))
        })
        .count();
    let mut frames = meter.vec(frame_nodes)?;
    let mut edges = 0_u64;
    let mut import_index = None;
    for (position, node) in graph.nodes.iter().enumerate() {
        let at = |member: &str| node_pointer(position).key(member);
        meter.at_node(position, graph_storage::node_form(&node.node_tag));
        validate_node_id(&node.node_id, || at("node_id"))?;
        let expected_package = match node.owner.as_ref() {
            Some(
                CheckedNodeOwner::ModelRoot { identity } | CheckedNodeOwner::Model { identity, .. },
            ) => identity.as_ref(),
            _ => wire.name.as_ref(),
        };
        if node.node_id.package.as_ref() != expected_package {
            let imported = if node.owner.is_none()
                && node.declaration.is_none()
                && dependencies
                    .selected_package(&node.node_id.package)
                    .is_some()
            {
                if import_index.is_none() {
                    import_index =
                        Some(dependencies.structural_import_index(structural_keys, meter)?);
                }
                import_index
                    .as_ref()
                    .and_then(
                        |lookup: &dependency_references::StructuralImportIndex<'_>| {
                            lookup.lookup(&node.node_id.package, position)
                        },
                    )
                    .is_some_and(|(selected_position, selected_package)| {
                        imported_node_matches(
                            node,
                            &selected_package.resolved_graph().nodes[selected_position].to_raw(),
                            structural_keys,
                            &selected_package.structural_keys,
                        )
                    })
            } else {
                false
            };
            if !imported {
                return Err(refuse(
                    CheckedPackageRefusalCode::InvalidSemanticGraph,
                    at("node_id").key("package"),
                ));
            }
        }
        // Charge the graph-node visit before checking identity uniqueness so
        // a denied visit cannot proceed to the duplicate-id refusal.
        meter.charge(1, || at("node_id"))?;
        if repeated_id == Some(position) {
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
        if kind == CheckedNodeKind::Temporal(TemporalForm::MonitorPlan) {
            if node.owner.is_some() {
                return Err(refuse(
                    CheckedPackageRefusalCode::InvalidSemanticGraph,
                    at("owner"),
                ));
            }
            if let Some(i) = node
                .occurrences
                .iter()
                .position(|o| o.role != CheckedOccurrenceRole::Generated)
            {
                return Err(refuse(
                    CheckedPackageRefusalCode::InvalidSourceMap,
                    at("occurrences").index(i),
                ));
            }
        }
        kinds.push(kind);
        validate_declaration(kind, &node.occurrences, node.declaration.as_ref(), position)?;
        validate_node_id(&node.semantic_type, || at("semantic_type"))?;
        if kind == CheckedNodeKind::Value(ValueForm::Literal) && !node.dependencies.is_empty() {
            return Err(refuse(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                at("dependencies"),
            ));
        }
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
        let view = TermNode::of(&bodies[position]);
        let nominal = if source_nominal_body(kind, view.get("term").and_then(TermNode::as_str)) {
            Some(
                identity::NominalBody::from_document(
                    view,
                    &wire.semantic_graph.operand_rows[position],
                    &|| at("body"),
                    meter,
                )
                .map_err(|failure| with_node_locus(failure, &node.node_id))?,
            )
        } else {
            None
        };
        let work = match body_grammar(kind) {
            BodyGrammar::Extension => {
                let body = extension::read_body(
                    TermNode::of(&bodies[position]),
                    at("body"),
                    read_charge::ids(
                        &mut *meter,
                        PackageReadStage::Graph,
                        position,
                        CheckedNodeTag::from_wire(&node.node_tag)
                            .map_or("node", CheckedNodeTag::as_wire),
                        &wire.semantic_graph.operand_rows[position],
                    ),
                )?;
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
                let (body, work) = frame::read_frame_body(
                    TermNode::of(&bodies[position]),
                    position,
                    &wire.semantic_graph.operand_rows[position],
                    meter,
                )?;
                frame_term = Some(body.checked_term(meter)?);
                frames.push((&node.node_id, position, body));
                work
            }
            BodyGrammar::Refinement => refinement::validate_body(
                TermNode::of(&bodies[position]),
                &Trail::Base(&body_steps),
                &mut |target, site, _target_at| targets.push((target.clone(), site)),
                read_charge::ids(
                    &mut *meter,
                    PackageReadStage::Graph,
                    position,
                    CheckedNodeTag::from_wire(&node.node_tag)
                        .map_or("node", CheckedNodeTag::as_wire),
                    &wire.semantic_graph.operand_rows[position],
                ),
            )
            .map_err(|failure| with_node_locus(failure, &node.node_id))?,
            BodyGrammar::Literal
            | BodyGrammar::Term
            | BodyGrammar::TimedModel
            | BodyGrammar::Hybrid
            | BodyGrammar::AbstractionRelation
            | BodyGrammar::FlatCorrespondence => validate_body(
                kind,
                &node.body,
                TermNode::of(&bodies[position]),
                &Trail::Base(&body_steps),
                &mut |target, site, _target_at| {
                    targets.push((target.clone(), site));
                },
                read_charge::ids(
                    &mut *meter,
                    PackageReadStage::Graph,
                    position,
                    CheckedNodeTag::from_wire(&node.node_tag)
                        .map_or("node", CheckedNodeTag::as_wire),
                    &wire.semantic_graph.operand_rows[position],
                ),
                nominal.as_ref(),
            )
            .map_err(|failure| with_node_locus(failure, &node.node_id))?,
        };
        meter.charge(work, || at("body"))?;
        body_terms.push(work);
        references.push(targets);
        if node.occurrences.is_empty() {
            return Err(refuse(
                CheckedPackageRefusalCode::InvalidSourceMap,
                at("occurrences"),
            ));
        }
        if let Some(repeat) = graph_storage::first_repeat(node.occurrences.iter(), meter)? {
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
            None => {
                if let Some(nominal) = nominal {
                    meter.reserve(std::mem::size_of::<identity::NominalBody>())?;
                    Ok(CheckedTerm::NominalBody(Box::new(nominal)))
                } else {
                    decode_admitted_body(
                        kind,
                        &bodies[position],
                        &wire.semantic_graph.operand_rows[position],
                        position,
                        meter,
                    )
                }
            }
        };
        terms.push(match typed_term {
            Ok(term) => Some(term),
            Err(ValidationFailure::Refused(_))
                if matches!(
                    body_grammar(kind),
                    BodyGrammar::AbstractionRelation | BodyGrammar::FlatCorrespondence
                ) =>
            {
                None
            }
            Err(failure) => return Err(failure),
        });
    }
    model_root::validate_shape(&graph.nodes, &kinds, &terms, meter)?;
    let missing_protocol_temporal_selection =
        protocol_body::has_missing_temporal_selection(&kinds, &terms, &index, &wire.lock);
    let binding_result = temporal_binding::validate(
        wire.temporal_bindings.as_deref(),
        &graph.nodes,
        &kinds,
        &index,
        structural_keys,
        &wire.lock,
        meter,
    );
    if let Err(failure) = binding_result {
        // An absent temporal selection makes this clock-kind reference
        // undecidable. Keep other binding refusals in their original order;
        // the protocol selection refusal is reported after placement.
        let deferred_clock_kind = missing_protocol_temporal_selection
            && matches!(&failure, ValidationFailure::Refused(refusal)
                if refusal.code == CheckedPackageRefusalCode::InvalidPackage
                    && refusal.cause == Some(CheckedPackageRefusalCause::TimedNodeReference)
                    && refusal.locus.as_ref()
                        .and_then(|id| index.get(id))
                        .and_then(|&position| kinds.get(position))
                        == Some(&CheckedNodeKind::Protocol(ProtocolForm::Compensation))
                    && refusal.primary_path().is_some_and(|path|
                        path.as_str().starts_with("/temporal_bindings/")
                            && path.as_str().ends_with("/clock/kind")));
        if !deferred_clock_kind {
            return Err(failure);
        }
    }
    temporal_clause_names::validate(
        wire.temporal_clause_names.as_deref(),
        &graph.nodes,
        &kinds,
        &index,
        structural_keys,
        meter,
    )?;
    // Graph-shape body and dependency rules precede declaration, frame, state
    // and operation checks. All of them
    // run before the dependency and body-reference edges below are resolved,
    // so an entry, anchor or clause reference naming no node is the step's
    // own `missing_declaration` refusal rather than the generic
    // unresolved-reference `invalid_semantic_graph`.
    structural::validate_structural_nodes_with_keys(
        &graph.nodes,
        &kinds,
        &terms,
        &index,
        structural_keys,
        Some(models),
        meter,
    )?;
    validate_owner_joins(wire, &kinds, &terms, models, meter)?;
    schedulability::validate_graph(&graph.nodes, &kinds, &terms, &index, meter)?;
    validate_derived_type_shapes(
        catalog,
        &graph.nodes,
        &kinds,
        &terms,
        &index,
        structural_keys,
        models,
        meter,
    )?;
    validate_selected_type_bounds(models, &terms, structural_keys, meter)?;
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
        meter.at_node(position, graph_storage::node_form(&node.node_tag));
        let targets = references[position]
            .iter()
            .filter(|(_, site)| site.member == ReferenceMember::Target);
        let mut expected = meter.vec(targets.clone().count())?;
        expected.extend(targets.map(|(target, _)| index.resolved(target, structural_keys)));
        expected.sort_unstable();
        expected.dedup();
        if !node
            .dependencies
            .iter()
            .map(|id| index.resolved(id, structural_keys))
            .eq(expected)
        {
            return Err(ValidationFailure::refused_at(
                CheckedPackageRefusalCode::InvalidSemanticGraph,
                node_pointer(position).key("dependencies"),
                None,
                node.node_id.clone(),
            ));
        }
    }
    monitor_plan::validate_dependencies(
        &graph.nodes,
        &kinds,
        &references,
        &index,
        structural_keys,
        meter,
    )?;
    refinement::validate_dependencies(
        &graph.nodes,
        &kinds,
        &terms,
        &index,
        structural_keys,
        meter,
    )?;
    let imported_nominals = validate_nominal_nodes(
        &graph.nodes,
        &kinds,
        &index,
        &wire.lock,
        models,
        dependencies,
        &terms,
        meter,
    )?;
    validate_declaration_names(&graph.nodes, &kinds, &index)?;
    validate_recursion_group_paths(&graph.nodes)?;
    // FR-038 "The timed interval form" stage 2: the last graph check of the
    // nodes before the frame step.
    validate_timed_bounds_reduced(&graph.nodes, &index, meter)?;
    validate_protocol_references(&graph.nodes, &kinds, &index)?;
    // FR-322 step 2: each selected declaration's model declaration node key,
    // one validation visit apiece, charged at the selection it belongs to.
    let owners = ModelOwners::build(
        models,
        &graph.nodes,
        &mut Budget::node(meter, 0),
        |budget, root| budget.charge_at(1, "model", || node_pointer(root).key("owner")),
    )?;
    extension::validate(wire, &kinds, &terms, &index, evidence, &owners, meter)?;
    let declarations = owners.index_charged(&mut Budget::node(meter, 0))?;
    frame::validate_frame_semantics(frames, &graph.nodes, &kinds, &index, &owners, meter)?;
    state::validate_state(
        &graph.nodes,
        &kinds,
        &terms,
        &index,
        structural_keys,
        &owners,
        meter,
    )?;
    claim_terminal::validate_state_graph(&graph.nodes, &kinds, &terms, &index, &owners, meter)?;
    validate_temporal(
        &terms,
        &graph.nodes,
        &kinds,
        &index,
        &owners,
        TemporalContext {
            catalog,
            lock: &wire.lock,
            diagnostics: &wire.diagnostics.entries,
            bindings: wire.temporal_bindings.as_deref(),
        },
        meter,
    )?;
    if kinds.contains(&CheckedNodeKind::Temporal(TemporalForm::MonitorPlan)) {
        validate_source_map_entries(
            graph
                .nodes
                .iter()
                .map(|node| (&node.node_id, node.occurrences.as_slice())),
            &wire.source_map,
            &wire.lock.sources,
            limits,
            meter,
        )?;
    }
    let early_operation_views =
        if kinds.contains(&CheckedNodeKind::Temporal(TemporalForm::MonitorPlan)) {
            Some(validate_operations(
                &graph.nodes,
                &kinds,
                OperationNodeOrder {
                    catalog,
                    wire_index: &index,
                    structural_keys,
                },
                &terms,
                &wire.lock,
                &owners,
                dependencies,
                meter,
            )?)
        } else {
            None
        };
    let monitor_plan_index = monitor_plan::validate(
        &graph.nodes,
        &kinds,
        &terms,
        &index,
        structural_keys,
        models,
        meter,
    )?;
    abstraction::validate_abstraction(&graph.nodes, &kinds, &index, &owners, meter)?;
    guarantee_composition::validate(
        &graph.nodes,
        &kinds,
        &terms,
        &index,
        &owners,
        dependencies,
        meter,
    )?;
    claim_terminal::validate_terminal(&graph.nodes, &kinds, &terms, &index, &owners, meter)?;
    step_relation::validate_step_relations(&graph.nodes, &kinds, &terms, &index, &owners, meter)?;
    validate_hyperproperties(&graph.nodes, &kinds, &terms, &index, &owners, meter)?;
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
            meter,
        );
        if let Err(failure) = source_check {
            if let ValidationFailure::Refused(refusal) = &failure {
                if refusal.code == CheckedPackageRefusalCode::InvalidSourceMap {
                    if let Some(path) = refusal.primary_path() {
                        let source_root = JsonPointer::root().key("source_map");
                        if path == &source_root {
                            refinement::validate_occurrences(
                                refinement::RefinementGraph::new(
                                    &graph.nodes,
                                    &kinds,
                                    &terms,
                                    &index,
                                    structural_keys,
                                    dependencies,
                                    &wire.name,
                                ),
                                &wire.source_map,
                                wire.refinement_step_rows.as_deref(),
                                &owners,
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
            refinement::RefinementGraph::new(
                &graph.nodes,
                &kinds,
                &terms,
                &index,
                structural_keys,
                dependencies,
                &wire.name,
            ),
            &wire.source_map,
            wire.refinement_step_rows.as_deref(),
            &owners,
            meter,
        )?;
    }
    refinement::validate_refinements(
        refinement::RefinementGraph::new(
            &graph.nodes,
            &kinds,
            &terms,
            &index,
            structural_keys,
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
        models,
        meter,
    )?;
    let schedulability_claim_index = schedulability::validate_carrier(
        &wire.name,
        wire.schedulability_claims.as_deref(),
        &graph.nodes,
        &kinds,
        &index,
        structural_keys,
        meter,
    )?;
    protocol_body::validate_protocol_nodes(
        &graph.nodes,
        &kinds,
        &terms,
        &index,
        protocol_body::ProtocolPass {
            catalog,
            lock: &wire.lock,
            bytes: limits.bytes,
        },
        &owners,
        meter,
    )?;
    schedulability::validate_bodies(
        &graph.nodes,
        &kinds,
        &terms,
        &index,
        structural_keys,
        models,
        meter,
    )?;
    refinement::validate_catalog_graph(
        refinement::RefinementGraph::new(
            &graph.nodes,
            &kinds,
            &terms,
            &index,
            structural_keys,
            dependencies,
            &wire.name,
        ),
        &owners,
        meter,
    )?;
    let admitted_views = match early_operation_views {
        Some(views) => views,
        None => validate_operations(
            &graph.nodes,
            &kinds,
            OperationNodeOrder {
                catalog,
                wire_index: &index,
                structural_keys,
            },
            &terms,
            &wire.lock,
            &owners,
            dependencies,
            meter,
        )?,
    };
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
                let endpoint_type = kinds[position]
                    == CheckedNodeKind::ScalarType(ScalarTypeForm::BoundedQuantity)
                    && kinds.get(target)
                        == Some(&CheckedNodeKind::ScalarType(ScalarTypeForm::Rational));
                lowering_successors.push((
                    target,
                    if endpoint_type {
                        lower::EdgeRole::Annotation
                    } else {
                        lower::EdgeRole::Typing
                    },
                ));
                successors.push((target, EdgeSite::Dependency(dependency_index)));
            } else if !(kinds[position]
                == CheckedNodeKind::Correspondence(CorrespondenceForm::ComponentComposition)
                && dependencies.holds_node(dependency))
                && (!matches!(
                    kinds[position],
                    CheckedNodeKind::ScalarType(
                        ScalarTypeForm::Dimension
                            | ScalarTypeForm::Unit
                            | ScalarTypeForm::DimensionEquivalence
                    )
                ) || imported_nominals.binary_search(dependency).is_err())
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
                && imported_nominals.binary_search(target).is_ok()
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
    let mut retained_terms = meter.vec(terms.len())?;
    for (position, term) in terms.into_iter().enumerate() {
        retained_terms.push(match term {
            Some(term) => term,
            None => decode_admitted_body(
                kinds[position],
                &bodies[position],
                &wire.semantic_graph.operand_rows[position],
                position,
                meter,
            )?,
        });
    }
    let terms = retained_terms;
    Ok(ValidatedGraph {
        kinds,
        schedulability_claim_index,
        monitor_plan_index,
        terms,
        declarations,
        lowering_facts,
    })
}

/// Joins already keyed protocol-reference carriers at the declaration step.
fn validate_protocol_references(
    nodes: &[RawSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &GraphIndex<'_>,
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
                let charged = meter.charge(1, || edge_pointer(vertex, site));
                #[cfg(test)]
                if work_probe::mutant_enabled() {
                    // Test-only mutation: continue the actual Tarjan walk
                    // after a denied edge charge.
                    let _ = charged;
                } else {
                    charged?;
                }
                #[cfg(not(test))]
                charged?;
                #[cfg(test)]
                work_probe::recursion_edge_visited();
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
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    use stage_storage::{Path, Refusal, Site};
    let code = CheckedPackageRefusalCode::UnknownRequiredCapability;
    let report = Path::root().key("capability_report");
    let site = |index: usize| {
        Site::new(
            PackageReadStage::Capabilities,
            index,
            "capability_report",
            report,
        )
    };
    let entries = &wire.capability_report;
    // The reported capabilities, ascending by feature then position.
    let mut reported: Vec<(&str, usize, CheckedCapabilityDisposition)> =
        stage_storage::vec(meter, &site(0), entries.len())?;
    reported.extend(
        entries.iter().enumerate().map(|(index, capability)| {
            (capability.feature.as_ref(), index, capability.disposition)
        }),
    );
    reported.sort_unstable_by(|left, right| (left.0, left.1).cmp(&(right.0, right.1)));
    // The first entry whose feature is empty or repeats an earlier one.
    let first_empty = entries
        .iter()
        .position(|capability| !is_nonempty(&capability.feature));
    let first_repeat = reported
        .windows(2)
        .filter(|pair| pair[0].0 == pair[1].0)
        .map(|pair| pair[1].1)
        .min();
    if let Some(index) = first_empty.into_iter().chain(first_repeat).min() {
        return Err(stage_storage::refuse(
            meter,
            &site(index),
            Refusal::new(code, report.index(index).key("feature")),
        ));
    }
    for (index, feature) in wire.lock.required_features.iter().enumerate() {
        let found = reported
            .binary_search_by(|entry| entry.0.cmp(feature.as_ref()))
            .ok()
            .map(|at| reported[at]);
        let path = match found {
            // Unreported: the report lacks the capability.
            None => report,
            Some((_, reported_at, disposition))
                if disposition != CheckedCapabilityDisposition::Available =>
            {
                report.index(reported_at).key("disposition")
            }
            // Reported available, but this reader does not support it.
            Some(_) if !evidence.supports(feature) => Path::root()
                .key("lock")
                .key("required_features")
                .index(index),
            Some(_) => continue,
        };
        return Err(stage_storage::refuse(
            meter,
            &site(index),
            Refusal::new(code, path),
        ));
    }
    Ok(())
}

fn validate_diagnostics(
    wire: &RawCheckedPackageWireV2,
    limits: CheckedPackageReadLimits,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    use stage_storage::{Path, Refusal, Site};
    let entries = Path::root().key("diagnostics").key("entries");
    let site = |index: usize, at: Path<'static>| {
        Site::new(
            PackageReadStage::Diagnostics,
            index,
            "diagnostics.entries",
            at,
        )
    };
    let diagnostics = count(wire.diagnostics.entries.len());
    if exceeds(diagnostics, limits.diagnostics) {
        // The first entry past the ceiling is the one whose charge failed.
        let first = usize::try_from(limits.diagnostics).unwrap_or(usize::MAX);
        return Err(stage_storage::incomplete(
            meter,
            &site(first, entries),
            CheckedPackageLimit::Diagnostics,
            limits.diagnostics,
            diagnostics,
            entries.index(first),
        ));
    }
    // The graph's node ids, ascending, when any detail names one.
    let mut nodes: Vec<&CheckedNodeId> = Vec::new();
    if wire
        .diagnostics
        .entries
        .iter()
        .any(|entry| !entry.details.is_empty())
    {
        nodes = stage_storage::vec(meter, &site(0, entries), wire.semantic_graph.nodes.len())?;
        nodes.extend(wire.semantic_graph.nodes.iter().map(|node| &node.node_id));
        nodes.sort_unstable();
    }
    for (entry_index, entry) in wire.diagnostics.entries.iter().enumerate() {
        let entry_at = entries.index(entry_index);
        for (detail_index, detail) in entry.details.iter().enumerate() {
            let detail_steps = [
                Step::Key("diagnostics"),
                Step::Key("entries"),
                Step::Index(entry_index),
                Step::Key("details"),
                Step::Index(detail_index),
            ];
            let detail_at = Trail::Base(&detail_steps);
            let detail_site = site(entry_index, entry_at.key("details").index(detail_index));
            let checked = term_json::validate_term_json(
                detail.node(),
                false,
                &detail_at,
                meter,
                &detail_site,
                &mut |target, _site| nodes.binary_search(&target).is_err(),
            )?;
            stage_storage::work(meter, &detail_site, checked.work)?;
            if let Some(steps) = checked.remembered {
                let position = Trail::Base(&steps);
                return Err(stage_storage::refuse(
                    meter,
                    &Site::new(
                        PackageReadStage::Diagnostics,
                        entry_index,
                        "diagnostics.entries",
                        position,
                    ),
                    Refusal::new(CheckedPackageRefusalCode::InvalidSemanticGraph, position),
                ));
            }
        }
        for (locus_index, region) in entry.loci.iter().enumerate() {
            let locus = entry_at.key("loci").index(locus_index);
            if !wire.lock.sources.contains(&region.source) {
                return Err(stage_storage::refuse(
                    meter,
                    &site(entry_index, locus),
                    Refusal::new(
                        CheckedPackageRefusalCode::InvalidSourceMap,
                        locus.key("source"),
                    ),
                ));
            }
            if region.start >= region.end {
                return Err(stage_storage::refuse(
                    meter,
                    &site(entry_index, locus),
                    Refusal::new(CheckedPackageRefusalCode::InvalidSourceMap, locus),
                ));
            }
        }
    }
    Ok(())
}

/// Helpers shared by the in-crate unit tests of the typed accessors.
#[cfg(test)]
mod test_support {
    use super::{CheckedNodeId, CheckedPackageWireV2, CheckedTerm};
    use serde_json::{json, Value};

    /// Bytes of a package a test authored in resolved index form, written as
    /// the V2 wire: each node-held index segment becomes an operand slot,
    /// numbered by first occurrence in the node's canonical order, beside
    /// the node's operand row. A package that already has rows is left alone.
    pub(super) fn canonical(package: &Value) -> Vec<u8> {
        fn walk(value: &mut Value, row: &mut Vec<u64>) {
            match value {
                Value::Array(items) => items.iter_mut().for_each(|item| walk(item, row)),
                Value::Object(object) => {
                    if object.len() == 2 && object.contains_key("package") {
                        return;
                    }
                    if object.len() == 3 && object.contains_key("path") {
                        if let Some(Value::Array(path)) = object.get_mut("path") {
                            for segment in path {
                                let index = segment
                                    .as_object()
                                    .filter(|member| member.len() == 1)
                                    .and_then(|member| member.get("node"))
                                    .and_then(Value::as_u64);
                                if let Some(index) = index {
                                    let slot = row
                                        .iter()
                                        .position(|entry| *entry == index)
                                        .unwrap_or_else(|| {
                                            row.push(index);
                                            row.len() - 1
                                        });
                                    *segment = json!({ "operand": slot });
                                }
                            }
                        }
                    }
                    object.values_mut().for_each(|member| walk(member, row));
                }
                _ => {}
            }
        }
        let mut wire = package.clone();
        if wire["semantic_graph"]["nodes"].is_array()
            && wire["semantic_graph"].get("operand_rows").is_none()
        {
            let rows = wire["semantic_graph"]["nodes"]
                .as_array_mut()
                .expect("graph nodes")
                .iter_mut()
                .map(|node| {
                    let mut row = Vec::new();
                    walk(node, &mut row);
                    json!(row)
                })
                .collect::<Vec<_>>();
            wire["semantic_graph"]["operand_rows"] = Value::Array(rows);
        }
        serde_json::to_vec(&wire).expect("canonical test JSON")
    }

    /// The JSON of a distinct structural node identity.
    pub(super) fn id(digit: char) -> Value {
        json!({"package": "test", "path": [digit.to_string()], "ordinal": 0})
    }

    /// A typed structural node identity.
    pub(super) fn node_id(digit: char) -> CheckedNodeId {
        serde_json::from_value(id(digit)).expect("node id")
    }

    /// Construct a private post-admission test wire through the same typed
    /// body decoder used after reader validation. Serde's untagged term
    /// deserializer cannot decode nested structural node IDs from a buffered
    /// JSON value.
    pub(super) fn private_wire(mut value: Value) -> CheckedPackageWireV2 {
        // Test graphs name operands by graph index, so their rows are empty.
        let count = value["semantic_graph"]["nodes"]
            .as_array()
            .expect("test nodes")
            .len();
        value["semantic_graph"]["operand_rows"] = json!(vec![Vec::<u64>::new(); count]);
        let nodes = value["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("test nodes");
        let bodies = nodes
            .iter_mut()
            .map(|node| {
                let body = std::mem::take(&mut node["body"]);
                node["body"] = json!({"term": "aggregate", "members": []});
                body
            })
            .collect::<Vec<_>>();
        let mut wire: CheckedPackageWireV2 = serde_json::from_value(value).expect("test wire");
        for (node, body) in wire.semantic_graph.nodes.iter_mut().zip(bodies) {
            node.body = CheckedTerm::decode_admitted(&body).expect("test body");
        }
        wire
    }
}

#[cfg(test)]
mod tests {
    use super::{
        body_grammar, node_pointer, reference_requires_cycle_edge, validate_recursion,
        validate_recursion_group_paths, validate_value_body, BodyGrammar,
        CheckedDependencySelection, CheckedNodeKind, CheckedPackageRefusalCode,
        CheckedSemanticNodeV2, CorrespondenceForm, EdgeSite, ExpressionForm, ReferenceMember,
        ReferenceSite, StateForm, Step, Trail, ValidationFailure, WorkMeter,
        DEPENDENCY_SELECTION_MEMBERS,
    };
    use ix_trace_rs::trace;
    use serde_json::json;

    fn refinement_internals(
        rows: Option<&[super::CheckedRefinementInternal]>,
    ) -> Result<(), ValidationFailure> {
        super::validate_refinement_internals(rows, &mut super::WorkMeter::new(u64::MAX))
    }

    fn dependency_selections(
        rows: &[super::CheckedDependencySelection],
    ) -> Result<(), ValidationFailure> {
        super::validate_dependency_selections(rows, &mut super::WorkMeter::new(u64::MAX))
    }

    /// Trace: FR-322-AC-77, TC-233.
    #[test]
    fn refinement_internal_lock_rows_are_nonempty_ordered_and_unique() {
        use super::CheckedRefinementInternal;
        let row = |refinement: &str, internal: &[&str]| CheckedRefinementInternal {
            refinement: refinement.into(),
            internal: internal.iter().map(|name| (*name).into()).collect(),
        };
        assert!(refinement_internals(None).is_ok());
        assert!(refinement_internals(Some(&[
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
            let Err(ValidationFailure::Refused(refusal)) = refinement_internals(Some(&rows)) else {
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
        let repeated_row = refinement_internals(Some(&[row("R", &["a"]), row("R", &["b"])]))
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
        let repeated_spelling = refinement_internals(Some(&[row("R", &["a", "a"])]))
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
        let repeated_dependency = dependency_selections(&[
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
        let one_location =
            refinement_internals(Some(&[row("R", &[])])).expect_err("empty internal list");
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

    #[trace("FR-038-AC-364", "TC-048")]
    #[test]
    fn group_path_join_uses_only_a_sole_declaration() {
        let member = |digit: char, path: &str, declared: bool| {
            let mut node = recursion_test_node(digit, Some("group"));
            node.node_id.path = vec![super::CheckedNodePathSegment::Name(path.into())];
            if declared {
                node.declaration = Some(super::CheckedDeclaration {
                    qualified_name: vec![path.into()],
                });
            }
            node
        };

        let declared = member('a', "A", true);
        let anonymous = member('b', "A", false);
        assert!(validate_recursion_group_paths(&[declared.clone(), anonymous.clone()]).is_ok());
        assert_recursion_refusal(
            validate_recursion_group_paths(&[declared.clone(), member('b', "other", false)]),
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            node_pointer(1).key("node_id").key("path").to_string(),
        );
        let mut owned_anonymous = member('b', "other", false);
        owned_anonymous.node_tag = "scalar_type".into();
        owned_anonymous.semantic_form = "dimension_equivalence".into();
        owned_anonymous.owner = Some(super::CheckedNodeOwner::Source {
            authority: "source".into(),
            identity: "equivalence".into(),
        });
        assert_recursion_refusal(
            validate_recursion_group_paths(&[declared.clone(), owned_anonymous]),
            CheckedPackageRefusalCode::InvalidSemanticGraph,
            node_pointer(1).key("node_id").key("path").to_string(),
        );

        // With no declared member the reader has no path anchor to invent.
        assert!(validate_recursion_group_paths(&[
            member('a', "first", false),
            member('b', "second", false),
        ])
        .is_ok());
        // Several declared nodes each retain their own path, and an anonymous
        // member has no unique declared path to inherit.
        assert!(validate_recursion_group_paths(&[
            declared,
            member('c', "C", true),
            member('b', "chosen", false),
        ])
        .is_ok());
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

    #[trace("FR-038-AC-149", "TC-226")]
    #[test]
    fn tc_226_large_group_stops_before_visiting_a_denied_edge() {
        const MEMBERS: usize = 100_000;
        let mut nodes = vec![recursion_test_node('a', Some("shared")); MEMBERS];
        for (position, node) in nodes.iter_mut().enumerate() {
            node.node_id.path = vec![super::CheckedNodePathSegment::Name(
                format!("member-{position}").into(),
            )];
        }
        let adjacency = (0..MEMBERS)
            .map(|position| vec![((position + 1) % MEMBERS, EdgeSite::Reference(0))])
            .collect::<Vec<_>>();
        let at = |vertex: usize, _: EdgeSite| node_pointer(vertex).key("body");

        super::work_probe::reset();
        let mut exact = WorkMeter::new(u64::MAX);
        validate_recursion(&nodes, &adjacency, &mut exact, &at).expect("large cycle admits");
        let minimum = exact.consumed();
        assert_eq!(minimum, MEMBERS as u64);
        assert_eq!(super::work_probe::visits_after_denial(), 0);

        super::work_probe::reset();
        let mut short = WorkMeter::new(minimum - 1);
        let Err(ValidationFailure::Incomplete(incomplete)) =
            validate_recursion(&nodes, &adjacency, &mut short, &at)
        else {
            panic!("one unit less must stop at the first unpaid edge");
        };
        assert_eq!(incomplete.limit_kind, super::CheckedPackageLimit::Work);
        assert_eq!(incomplete.limit, minimum - 1);
        assert_eq!(incomplete.consumed, minimum);
        assert_eq!(
            incomplete.path,
            Some(at(MEMBERS - 1, EdgeSite::Reference(0)))
        );
        assert_eq!(super::work_probe::visits_after_denial(), 0);

        // Apply the continue-after-denial mutation to the actual Tarjan walk,
        // then restore normal propagation. The observer must detect the
        // unpaid edge visit that the production branch prevents.
        super::work_probe::reset();
        super::work_probe::continue_after_denial(true);
        let mut mutant = WorkMeter::new(minimum - 1);
        validate_recursion(&nodes, &adjacency, &mut mutant, &at)
            .expect("test mutation deliberately ignores the denied charge");
        assert_eq!(super::work_probe::visits_after_denial(), 1);
        super::work_probe::reset();
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
            "semantic_graph": {"graph_version": "quire.checked-semantic-graph/v2", "nodes": [], "operand_rows": []},
            "source_map": [], "capability_report": [],
            "diagnostics": {"catalog": {"authority": "test", "identity": "diagnostics"}, "entries": []}
        }))
        .expect("private owned wire")
    }

    /// Trace: FR-038-AC-117.
    /// Provenance: IR-1196, QSL FR-264-AC-1.
    #[test]
    fn deep_node_and_package_debug_write_each_node_once_on_a_small_stack() {
        std::thread::Builder::new()
            .stack_size(512 * 1024)
            .spawn(|| {
                const CHAIN_LEN: usize = 100_000;
                let mut key = std::sync::Arc::new(super::test_support::node_id('a'));
                let mut keys = vec![std::sync::Arc::clone(&key)];
                for ordinal in 1..=CHAIN_LEN {
                    key = std::sync::Arc::new(super::CheckedNodeId {
                        package: "test".into(),
                        path: vec![super::CheckedNodePathSegment::Node(key)],
                        ordinal: u64::try_from(ordinal).expect("depth fits u64"),
                    });
                    keys.push(std::sync::Arc::clone(&key));
                }

                let node_debug = format!("{key:?}");
                assert_eq!(node_debug.matches("package: ").count(), CHAIN_LEN + 1);
                assert_eq!(node_debug.matches("Node(#").count(), CHAIN_LEN);
                assert!(node_debug.len() < CHAIN_LEN * 100);
                drop(node_debug);
                drop(key);

                let wire = tc_451_private_owned_wire();
                let package = super::CheckedPackageV2 {
                    schedulability_claim_index: Vec::new(),
                    monitor_plan_index: Vec::new(),
                    timed_model_roots: Vec::new(),
                    node_index: std::sync::Arc::new(Default::default()),
                    structural_keys: keys,
                    slot_graph: Default::default(),
                    source_index: std::sync::Arc::new(Default::default()),
                    lowering_facts: Vec::new(),
                    wire: std::sync::Arc::new(wire),
                    kinds: std::sync::Arc::new(Vec::new()),
                    bytes: 0,
                    models: std::sync::Arc::new(Default::default()),
                    selected_documents: std::sync::Arc::new(Vec::new()),
                    direct_dependencies: Vec::new(),
                    closure_index: Default::default(),
                    catalog: crate::checked_package::v2::operation_catalog::owned_operation_catalog(
                    ),
                    admission_binding: Default::default(),
                }
                .retain_original_admission();
                let package_debug = format!("{package:?}");
                assert_eq!(package_debug.matches("package: ").count(), CHAIN_LEN + 1);
                assert_eq!(package_debug.matches("Node(#").count(), CHAIN_LEN);
                assert!(package_debug.contains("roots: [#0, #1, #2"));
                assert!(package_debug.len() < CHAIN_LEN * 110);
            })
            .expect("small-stack thread spawns")
            .join()
            .expect("small-stack Debug completes");
    }

    #[trace("TC-451", "FR-051-AC-1", "FR-051-AC-4", "FR-051-AC-5")]
    #[test]
    fn tc_451_borrowed_encoder_equals_private_owned_wire_bytes() {
        let node = super::test_support::id('b');
        let occurrence = json!({"role": "expression", "ordinal": 0});
        let mut populated = json!({
            "equality_decisions": [{
                "application": node, "occurrence": occurrence, "term_path": [],
                "operator": "equal", "schedule": "text",
                "left": {"operand": 0, "source_type": node, "conversion_target": null},
                "right": {"operand": 1, "source_type": node, "conversion_target": null},
                "units": [], "enum_members": []
            }],
            "protocol_locations": [{
                "application": node, "occurrence": occurrence, "term_ordinal": 0,
                "location": {"origin": {"kind": "type_declaration", "name": "Record"}, "path": [1]}
            }],
            "dispatch_tables": [{
                "application": node, "occurrence": occurrence,
                "operation": {"declaring": node, "operation": "run"},
                "context": node,
                "candidates": [
                    {
                        "selected_operation": {"declaring": node, "operation": "run"},
                        "selection_key": {"owner_effective_type": null,
                            "original": {"package": "test", "node": "Zeta"}},
                        "operation_body": node, "effective_precondition": null
                    },
                    {
                        "selected_operation": {"declaring": node, "operation": "run"},
                        "selection_key": {"owner_effective_type": null,
                            "original": {"package": "test", "node": "Alpha"}},
                        "operation_body": node, "effective_precondition": node
                    }
                ]
            }],
            "probabilistic_claims": []
        });
        let mut next_equality = populated["equality_decisions"][0].clone();
        next_equality["occurrence"]["ordinal"] = json!(1);
        populated["equality_decisions"]
            .as_array_mut()
            .expect("equality array")
            .insert(0, next_equality);
        let mut next_location = populated["protocol_locations"][0].clone();
        next_location["term_ordinal"] = json!(1);
        populated["protocol_locations"]
            .as_array_mut()
            .expect("location array")
            .insert(0, next_location);

        let empty = json!({
            "equality_decisions": [], "protocol_locations": [], "dispatch_tables": [], "probabilistic_claims": []
        });
        let mut wire = tc_451_private_owned_wire();
        for sidecar in [None, Some(empty), Some(populated)] {
            wire.checked_term_data =
                sidecar.map(|value| serde_json::from_value(value).expect("closed sidecar"));
            let bytes = super::encode_checked_package_v2(
                &wire.name,
                &wire.lock,
                &wire.semantic_graph,
                super::CheckedPackageSourceCorrespondenceRef {
                    source_map: &wire.source_map,
                    refinement_step_rows: wire.refinement_step_rows.as_deref(),
                },
                super::CheckedPackageSidecarsRef {
                    temporal_clause_names: wire.temporal_clause_names.as_deref(),
                    operation_effects: wire.operation_effects.as_deref(),
                    checked_term_data: wire.checked_term_data.as_ref(),
                    temporal_bindings: wire.temporal_bindings.as_deref(),
                    schedulability_claims: wire.schedulability_claims.as_deref(),
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
    }

    #[trace("TC-451", "FR-051-AC-7", "FR-038-AC-400")]
    #[test]
    fn tc_451_schedulability_rows_match_private_wire_without_normalization() {
        let mut wire = tc_451_private_owned_wire();
        let declaration: super::CheckedNodeId =
            serde_json::from_value(super::test_support::id('b')).expect("producer identity");
        wire.operation_effects = Some(vec![super::CheckedOperationEffectRow {
            document: "test/effects".into(),
            declaring: declaration.clone(),
            operation: "run".into(),
            effect: super::CheckedOperationEffect {
                modifies: Vec::new(),
                creates: Vec::new(),
                deletes: Vec::new(),
                deletes_scoped: Vec::new(),
            },
            provenance: super::CheckedOperationEffectProvenance {
                origin: super::CheckedOperationEffectOrigin::ModelDocumentOperationFrame,
                member: "run".into(),
            },
        }]);
        let rows = vec![
            super::CheckedSchedulabilityClaimRow {
                declaration: declaration.clone(),
                occurrence: 9,
            },
            super::CheckedSchedulabilityClaimRow {
                declaration,
                occurrence: 2,
            },
        ];
        for rows in [None, Some(Vec::new()), Some(rows)] {
            wire.schedulability_claims = rows;
            let bytes = super::encode_checked_package_v2(
                &wire.name,
                &wire.lock,
                &wire.semantic_graph,
                super::CheckedPackageSourceCorrespondenceRef {
                    source_map: &wire.source_map,
                    refinement_step_rows: wire.refinement_step_rows.as_deref(),
                },
                super::CheckedPackageSidecarsRef {
                    temporal_clause_names: wire.temporal_clause_names.as_deref(),
                    operation_effects: wire.operation_effects.as_deref(),
                    checked_term_data: wire.checked_term_data.as_ref(),
                    temporal_bindings: wire.temporal_bindings.as_deref(),
                    schedulability_claims: wire.schedulability_claims.as_deref(),
                },
                &wire.capability_report,
                &wire.diagnostics,
                quire_canonical::Limits::new(u64::MAX),
            )
            .expect("producer rows encode unchanged");
            let expected =
                serde_json::to_vec(&serde_json::to_value(&wire).expect("private envelope"))
                    .expect("canonical value");
            assert_eq!(bytes, expected);
            if wire
                .schedulability_claims
                .as_ref()
                .is_none_or(|rows| !rows.is_empty())
            {
                let document =
                    std::sync::Arc::new(quire_canonical::read(&bytes, u64::MAX).expect("reads"));
                let (decoded, _) = super::intake::decode_borrowed_wire(
                    &bytes,
                    &document,
                    &mut super::NativeJoinBudget::unlimited(),
                )
                .expect("closed intake");
                assert_eq!(decoded.schedulability_claims, wire.schedulability_claims);
                assert_eq!(decoded.operation_effects, wire.operation_effects);
            }
        }
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
            "body": {"term":"aggregate","members":[]}
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
                    value: {
                        let mut deep = nested(DEEP);
                        let held = super::CheckedJson::from_value(&deep).expect("a deep value");
                        quire_canonical::drop_value(std::mem::take(&mut deep));
                        held
                    },
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
                        super::CheckedPackageSidecarsRef {
                            temporal_clause_names: wire.temporal_clause_names.as_deref(),
                            operation_effects: wire.operation_effects.as_deref(),
                            checked_term_data: wire.checked_term_data.as_ref(),
                            temporal_bindings: wire.temporal_bindings.as_deref(),
                            schedulability_claims: wire.schedulability_claims.as_deref(),
                        },
                        &wire.capability_report,
                        &wire.diagnostics,
                        limits,
                    )?;
                    Ok((expected, actual))
                })();

                quire_canonical::drop_value(std::mem::take(&mut private_wire_value));
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
            validate_value_body(
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

#[cfg(test)]
mod admitted_document_bridge_tests {
    use super::*;
    use ix_trace_rs::trace;
    use serde_json::json;

    #[trace("TC-472", "FR-038-AC-504")]
    #[test]
    fn admitted_document_bridge_prepays_retention_and_preserves_operand_rows() {
        let cases = [
            (
                CheckedNodeKind::Expression(ExpressionForm::Reference),
                json!({"term":"reference","target":{"package":"fixture","path":[{"operand":0}],"ordinal":0}}),
            ),
            (
                CheckedNodeKind::ScalarType(ScalarTypeForm::Dimension),
                json!({"term":"dimension","base":true,"terms":[]}),
            ),
            (
                CheckedNodeKind::ScalarType(ScalarTypeForm::Unit),
                json!({"term":"unit",
                "dimension_node_id":{"package":"fixture","path":[{"operand":0}],"ordinal":0},
                "target_unit_node_id":null,"scale":{"numerator":"1","denominator":"1"},
                "offset":{"numerator":"0","denominator":"1"}}),
            ),
            (
                CheckedNodeKind::Correspondence(CorrespondenceForm::ImplementationGuarantee),
                json!({"term":"implementation_guarantee",
                "clause":{"package":"fixture","path":[{"operand":0}],"ordinal":0},
                "relation":{"package":"fixture","path":[{"operand":0}],"ordinal":0},
                "initial":["begin"],"progress":[],"environment":[]}),
            ),
            (
                CheckedNodeKind::Correspondence(CorrespondenceForm::ComponentComposition),
                json!({"term":"component_composition",
                "rule":"interface","components":[],"guarantees":[],"relies":[],"liveness_dependencies":[],"supplies":[],"environment":[]}),
            ),
        ];
        for (kind, wire) in cases {
            let body = CheckedJson::from_value(&wire).expect("fixture document");
            let run = |limit| {
                let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
                let mut outcome = None;
                let measured = allocation_counter::measure(|| {
                    let mut meter = WorkMeter::over(&mut budget);
                    meter.at_node(7, "body");
                    outcome = Some(decode_admitted_body(kind, &body, &[19], 7, &mut meter));
                });
                (
                    outcome.expect("ran"),
                    measured.bytes_total,
                    budget.read_storage_consumed(),
                )
            };
            let (term, allocated, charged) = run(u64::MAX);
            let term = term.expect("closed body");
            assert!(charged > 0);
            assert_eq!(allocated, charged, "{kind:?}");
            match kind {
                CheckedNodeKind::Expression(ExpressionForm::Reference) => {
                    let CheckedTerm::Reference { target } = &term else {
                        panic!("retained reference");
                    };
                    assert_eq!(target.path, [CheckedNodePathSegment::Index(19)]);
                }
                CheckedNodeKind::ScalarType(ScalarTypeForm::Dimension) => {
                    let CheckedTerm::NominalBody(body) = &term else {
                        panic!("retained dimension");
                    };
                    let identity::NominalBodyKind::Dimension { base, terms } = body.kind() else {
                        panic!("dimension body");
                    };
                    assert!(*base);
                    assert!(terms.is_empty());
                }
                CheckedNodeKind::ScalarType(ScalarTypeForm::Unit) => {
                    let CheckedTerm::NominalBody(body) = &term else {
                        panic!("retained unit");
                    };
                    let identity::NominalBodyKind::Unit {
                        dimension_node_id,
                        target_unit_node_id,
                        ..
                    } = body.kind()
                    else {
                        panic!("unit body");
                    };
                    assert_eq!(dimension_node_id.path, [CheckedNodePathSegment::Index(19)]);
                    assert!(target_unit_node_id.is_none());
                }
                CheckedNodeKind::Correspondence(CorrespondenceForm::ImplementationGuarantee) => {
                    let CheckedTerm::ImplementationGuarantee {
                        clause,
                        relation,
                        initial,
                        progress,
                        environment,
                    } = &term
                    else {
                        panic!("typed guarantee");
                    };
                    assert_eq!(clause.path, [CheckedNodePathSegment::Index(19)]);
                    assert_eq!(relation.path, [CheckedNodePathSegment::Index(19)]);
                    assert_eq!(
                        initial.iter().map(AsRef::as_ref).collect::<Vec<&str>>(),
                        ["begin"]
                    );
                    assert!(progress.is_empty() && environment.is_empty());
                }
                CheckedNodeKind::Correspondence(CorrespondenceForm::ComponentComposition) => {
                    let CheckedTerm::ComponentComposition {
                        rule,
                        components,
                        guarantees,
                        relies,
                        liveness_dependencies,
                        supplies,
                        environment,
                    } = &term
                    else {
                        panic!("typed composition");
                    };
                    assert_eq!(rule.as_ref(), "interface");
                    assert!(
                        components.is_empty()
                            && guarantees.is_empty()
                            && relies.is_empty()
                            && liveness_dependencies.is_empty()
                            && supplies.is_empty()
                            && environment.is_empty()
                    );
                }
                _ => panic!("unexpected test kind"),
            }
            assert_eq!(run(charged).0.expect("exact allowance"), term);
            assert!(
                matches!(run(charged - 1).0, Err(ValidationFailure::Budget(_))),
                "one short: {kind:?}"
            );
        }
    }

    #[trace("TC-472", "FR-038-AC-512")]
    #[test]
    fn malformed_nominal_bridge_preserves_the_body_member_refusal() {
        let body = CheckedJson::from_value(&json!({"term":"unit",
            "dimension_node_id":{"package":"fixture","path":["Time"],"ordinal":0},
            "target_unit_node_id":null,"scale":{"numerator":1,"denominator":"1"},
            "offset":{"numerator":"0","denominator":"1"}}))
        .expect("document");
        let failure = decode_admitted_body(
            CheckedNodeKind::ScalarType(ScalarTypeForm::Unit),
            &body,
            &[],
            3,
            &mut WorkMeter::new(u64::MAX),
        )
        .expect_err("malformed scalar");
        assert!(matches!(failure,ValidationFailure::Refused(refusal)
            if refusal.code==CheckedPackageRefusalCode::MalformedWire && refusal.primary_path().is_some_and(|path|path.as_str()=="/semantic_graph/nodes/3/body/scale/numerator")));
    }
    #[trace("TC-472", "FR-038-AC-512")]
    #[test]
    fn lock_duration_partial_defers_shape_refusal_but_propagates_reservation_denial() {
        let malformed = serde_json::json!({"term":"unit","dimension_node_id":{},"target_unit_node_id":null,
            "scale":{"numerator":"1","denominator":"1"},"offset":{"numerator":"0","denominator":"1"}});
        let id = test_support::node_id('1');
        let nodes = [CheckedSemanticNodeV2 {
            node_id: id.clone(),
            schema_version: "test".into(),
            node_tag: "scalar_type".into(),
            semantic_form: "unit".into(),
            semantic_type: id,
            dependencies: vec![],
            occurrences: vec![],
            recursion_group: None,
            declaration: None,
            owner: None,
            owner_model: None,
            body: (),
        }];
        let bodies =
            graph_storage::OwnedBodies::of_values(std::slice::from_ref(&malformed), vec![vec![]]);
        let run = |limit| {
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut meter = WorkMeter::over(&mut budget);
            bodies.attach(&mut meter);
            identity::duration_terms(&nodes, &mut meter)
        };
        assert!(run(u64::MAX).expect("shape is deferred")[0].is_none());
        assert!(matches!(run(0), Err(ValidationFailure::Budget(_))));
        let handle = CheckedJson::from_value(&malformed).expect("document");
        assert!(
            matches!(decode_admitted_body(CheckedNodeKind::ScalarType(ScalarTypeForm::Unit),&handle,&[],0,&mut WorkMeter::new(u64::MAX)),
            Err(ValidationFailure::Refused(refusal)) if refusal.code==CheckedPackageRefusalCode::MalformedWire)
        );
    }
}

#[cfg(test)]
mod retained_fixture;
#[cfg(test)]
use retained_fixture::{retained_node_index, retained_source_index};
