//! Closed FR-025 decisions and typed causes.

use serde::{Deserialize, Serialize};

use crate::bridge::{canonical, BridgeDigest, BridgeError, BridgeLimits, ContractSelection};

use super::{
    PredicateCorrespondence, PredicateRef, PROFILE, PROJECTION_DECISION_PROFILE,
    VALUATION_DECISION_PROFILE,
};

/// Closed cause dimensions in normative output order.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PredicateCauseDimension {
    /// Native checked-leaf owner contract.
    NativeContract,
    /// TL target owner contracts.
    TargetContract,
    /// Authored source identity or span.
    Source,
    /// Checked leaf and parent binding.
    CheckedLeaf,
    /// Native semantic model identity.
    Model,
    /// Predicate construct or identity.
    Predicate,
    /// Selected predicate/fact population.
    Population,
    /// Generated signal catalog.
    Catalog,
    /// Generated proposition map or cross-document join.
    PropositionMap,
    /// Admitted predicate correspondence.
    Correspondence,
    /// Observation availability contract.
    AvailabilityContract,
    /// Protocol result contract.
    SourceResultContract,
    /// Protocol-to-Contract-IR mapping contract.
    SourceResultMapping,
    /// Result producer availability/profile.
    Producer,
    /// Exact observation assertion and revision.
    Observation,
    /// Native predicate execution disposition.
    PredicateExecution,
    /// Scope/anchor binding.
    Anchor,
    /// Immutable capture binding.
    Capture,
    /// Completeness assertion and state.
    Completeness,
    /// Exact decision-premise set.
    DecidingFacts,
    /// Result truth/value/profile binding.
    Result,
    /// Direct correction predecessor.
    Supersession,
}

impl PredicateCauseDimension {
    /// Every closed cause dimension in normative precedence order.
    pub const ALL: [Self; 22] = [
        Self::NativeContract,
        Self::TargetContract,
        Self::Source,
        Self::CheckedLeaf,
        Self::Model,
        Self::Predicate,
        Self::Population,
        Self::Catalog,
        Self::PropositionMap,
        Self::Correspondence,
        Self::AvailabilityContract,
        Self::SourceResultContract,
        Self::SourceResultMapping,
        Self::Producer,
        Self::Observation,
        Self::PredicateExecution,
        Self::Anchor,
        Self::Capture,
        Self::Completeness,
        Self::DecidingFacts,
        Self::Result,
        Self::Supersession,
    ];
}

/// Closed STD-001 predicate cause catalog.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum PredicateCauseCode {
    #[serde(rename = "predicate_native_contract_unavailable")]
    NativeContractUnavailable,
    #[serde(rename = "predicate_target_contract_unavailable")]
    TargetContractUnavailable,
    #[serde(rename = "predicate_native_profile_unsupported")]
    NativeProfileUnsupported,
    #[serde(rename = "predicate_target_profile_unsupported")]
    TargetProfileUnsupported,
    #[serde(rename = "predicate_producer_profile_unsupported")]
    ProducerProfileUnsupported,
    #[serde(rename = "predicate_evaluation_profile_unsupported")]
    EvaluationProfileUnsupported,
    #[serde(rename = "predicate_construct_unsupported")]
    ConstructUnsupported,
    #[serde(rename = "predicate_source_mismatch")]
    SourceMismatch,
    #[serde(rename = "predicate_checked_leaf_mismatch")]
    CheckedLeafMismatch,
    #[serde(rename = "predicate_model_mismatch")]
    ModelMismatch,
    #[serde(rename = "predicate_population_invalid")]
    PopulationInvalid,
    #[serde(rename = "predicate_native_contract_conflict")]
    NativeContractConflict,
    #[serde(rename = "predicate_target_contract_conflict")]
    TargetContractConflict,
    #[serde(rename = "predicate_identity_conflict")]
    IdentityConflict,
    #[serde(rename = "predicate_catalog_rejected")]
    CatalogRejected,
    #[serde(rename = "predicate_map_rejected")]
    MapRejected,
    #[serde(rename = "predicate_catalog_map_mismatch")]
    CatalogMapMismatch,
    #[serde(rename = "predicate_availability_contract_unsupported")]
    AvailabilityContractUnsupported,
    #[serde(rename = "predicate_availability_contract_unavailable")]
    AvailabilityContractUnavailable,
    #[serde(rename = "predicate_source_result_contract_unsupported")]
    SourceResultContractUnsupported,
    #[serde(rename = "predicate_source_result_contract_unavailable")]
    SourceResultContractUnavailable,
    #[serde(rename = "predicate_source_result_mapping_unsupported")]
    SourceResultMappingUnsupported,
    #[serde(rename = "predicate_source_result_mapping_unavailable")]
    SourceResultMappingUnavailable,
    #[serde(rename = "predicate_producer_unavailable")]
    ProducerUnavailable,
    #[serde(rename = "predicate_result_not_yet_observed")]
    ResultNotYetObserved,
    #[serde(rename = "predicate_execution_unsupported")]
    ExecutionUnsupported,
    #[serde(rename = "predicate_execution_refused")]
    ExecutionRefused,
    #[serde(rename = "predicate_execution_failed")]
    ExecutionFailed,
    #[serde(rename = "predicate_execution_incomplete")]
    ExecutionIncomplete,
    #[serde(rename = "predicate_observation_mismatch")]
    ObservationMismatch,
    #[serde(rename = "predicate_anchor_mismatch")]
    AnchorMismatch,
    #[serde(rename = "predicate_capture_mismatch")]
    CaptureMismatch,
    #[serde(rename = "predicate_valuation_population_mismatch")]
    ValuationPopulationMismatch,
    #[serde(rename = "predicate_completeness_incomplete")]
    CompletenessIncomplete,
    #[serde(rename = "predicate_completeness_conflict")]
    CompletenessConflict,
    #[serde(rename = "predicate_result_type_mismatch")]
    ResultTypeMismatch,
    #[serde(rename = "predicate_result_stale")]
    ResultStale,
    #[serde(rename = "predicate_supersession_invalid")]
    SupersessionInvalid,
}

impl PredicateCauseCode {
    /// Every closed STD-001 predicate cause code.
    pub const ALL: [Self; 38] = [
        Self::NativeContractUnavailable,
        Self::TargetContractUnavailable,
        Self::NativeProfileUnsupported,
        Self::TargetProfileUnsupported,
        Self::ProducerProfileUnsupported,
        Self::EvaluationProfileUnsupported,
        Self::ConstructUnsupported,
        Self::SourceMismatch,
        Self::CheckedLeafMismatch,
        Self::ModelMismatch,
        Self::PopulationInvalid,
        Self::NativeContractConflict,
        Self::TargetContractConflict,
        Self::IdentityConflict,
        Self::CatalogRejected,
        Self::MapRejected,
        Self::CatalogMapMismatch,
        Self::AvailabilityContractUnsupported,
        Self::AvailabilityContractUnavailable,
        Self::SourceResultContractUnsupported,
        Self::SourceResultContractUnavailable,
        Self::SourceResultMappingUnsupported,
        Self::SourceResultMappingUnavailable,
        Self::ProducerUnavailable,
        Self::ResultNotYetObserved,
        Self::ExecutionUnsupported,
        Self::ExecutionRefused,
        Self::ExecutionFailed,
        Self::ExecutionIncomplete,
        Self::ObservationMismatch,
        Self::AnchorMismatch,
        Self::CaptureMismatch,
        Self::ValuationPopulationMismatch,
        Self::CompletenessIncomplete,
        Self::CompletenessConflict,
        Self::ResultTypeMismatch,
        Self::ResultStale,
        Self::SupersessionInvalid,
    ];

    /// Returns the exact STD-001 label.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NativeContractUnavailable => "predicate_native_contract_unavailable",
            Self::TargetContractUnavailable => "predicate_target_contract_unavailable",
            Self::NativeProfileUnsupported => "predicate_native_profile_unsupported",
            Self::TargetProfileUnsupported => "predicate_target_profile_unsupported",
            Self::ProducerProfileUnsupported => "predicate_producer_profile_unsupported",
            Self::EvaluationProfileUnsupported => "predicate_evaluation_profile_unsupported",
            Self::ConstructUnsupported => "predicate_construct_unsupported",
            Self::SourceMismatch => "predicate_source_mismatch",
            Self::CheckedLeafMismatch => "predicate_checked_leaf_mismatch",
            Self::ModelMismatch => "predicate_model_mismatch",
            Self::PopulationInvalid => "predicate_population_invalid",
            Self::NativeContractConflict => "predicate_native_contract_conflict",
            Self::TargetContractConflict => "predicate_target_contract_conflict",
            Self::IdentityConflict => "predicate_identity_conflict",
            Self::CatalogRejected => "predicate_catalog_rejected",
            Self::MapRejected => "predicate_map_rejected",
            Self::CatalogMapMismatch => "predicate_catalog_map_mismatch",
            Self::AvailabilityContractUnsupported => "predicate_availability_contract_unsupported",
            Self::AvailabilityContractUnavailable => "predicate_availability_contract_unavailable",
            Self::SourceResultContractUnsupported => "predicate_source_result_contract_unsupported",
            Self::SourceResultContractUnavailable => "predicate_source_result_contract_unavailable",
            Self::SourceResultMappingUnsupported => "predicate_source_result_mapping_unsupported",
            Self::SourceResultMappingUnavailable => "predicate_source_result_mapping_unavailable",
            Self::ProducerUnavailable => "predicate_producer_unavailable",
            Self::ResultNotYetObserved => "predicate_result_not_yet_observed",
            Self::ExecutionUnsupported => "predicate_execution_unsupported",
            Self::ExecutionRefused => "predicate_execution_refused",
            Self::ExecutionFailed => "predicate_execution_failed",
            Self::ExecutionIncomplete => "predicate_execution_incomplete",
            Self::ObservationMismatch => "predicate_observation_mismatch",
            Self::AnchorMismatch => "predicate_anchor_mismatch",
            Self::CaptureMismatch => "predicate_capture_mismatch",
            Self::ValuationPopulationMismatch => "predicate_valuation_population_mismatch",
            Self::CompletenessIncomplete => "predicate_completeness_incomplete",
            Self::CompletenessConflict => "predicate_completeness_conflict",
            Self::ResultTypeMismatch => "predicate_result_type_mismatch",
            Self::ResultStale => "predicate_result_stale",
            Self::SupersessionInvalid => "predicate_supersession_invalid",
        }
    }
}

/// One stable ordered semantic cause.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PredicateCause {
    dimension: PredicateCauseDimension,
    code: PredicateCauseCode,
    rejected_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    raw_discriminator: Option<String>,
}

impl PredicateCause {
    /// Constructs a cause whose code/dimension allocation is checked.
    pub fn new(
        dimension: PredicateCauseDimension,
        code: PredicateCauseCode,
        rejected_ref: impl Into<String>,
        raw_discriminator: Option<String>,
    ) -> Option<Self> {
        if expected_dimension(code) == dimension {
            Some(Self {
                dimension,
                code,
                rejected_ref: rejected_ref.into(),
                raw_discriminator,
            })
        } else {
            None
        }
    }

    pub(crate) fn assigned(
        code: PredicateCauseCode,
        rejected_ref: impl Into<String>,
        raw_discriminator: Option<String>,
    ) -> Self {
        Self {
            dimension: expected_dimension(code),
            code,
            rejected_ref: rejected_ref.into(),
            raw_discriminator,
        }
    }

    /// Returns the cause dimension.
    #[must_use]
    pub const fn dimension(&self) -> PredicateCauseDimension {
        self.dimension
    }

    /// Returns the stable cause code.
    #[must_use]
    pub const fn code(&self) -> PredicateCauseCode {
        self.code
    }

    /// Returns the rejected exact identity, or empty when none exists.
    #[must_use]
    pub fn rejected_ref(&self) -> &str {
        &self.rejected_ref
    }

    /// Returns an unknown well-formed discriminator retained by the decision.
    #[must_use]
    pub fn raw_discriminator(&self) -> Option<&str> {
        self.raw_discriminator.as_deref()
    }
}

/// Projection disposition.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PredicateProjectionKind {
    /// Complete artifacts and correspondence were admitted.
    Admitted,
    /// An accepted dependency was unreachable.
    Unavailable,
    /// A well-formed selection or construct is unsupported.
    Unsupported,
    /// A stale, malformed, or rejected semantic selection was refused.
    Refused,
    /// Unequal content claimed one identity.
    Conflict,
}

/// Valuation disposition.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PredicateValuationKind {
    /// Exact owner mapping carried one final Boolean.
    Valued,
    /// More observation or bounded execution is required.
    Incomplete,
    /// An accepted owner or producer is unavailable.
    Unavailable,
    /// The selected capability is unsupported.
    Unsupported,
    /// The source evaluator failed.
    Failed,
    /// The result or binding was refused.
    Refused,
    /// A deciding premise is contradicted.
    Conflict,
}

/// Canonical admitted or rejected projection decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PredicateProjectionDecision {
    pub(crate) kind: PredicateProjectionKind,
    pub(crate) bridge_profile: Option<String>,
    pub(crate) native_contract: Option<ContractSelection>,
    pub(crate) signal_contract: Option<ContractSelection>,
    pub(crate) map_contract: Option<ContractSelection>,
    pub(crate) correspondences: Vec<PredicateCorrespondence>,
    pub(crate) signal_catalog_ref: Option<BridgeDigest>,
    pub(crate) proposition_map_ref: Option<BridgeDigest>,
    pub(crate) projection_ref: Option<BridgeDigest>,
    pub(crate) causes: Vec<PredicateCause>,
    pub(crate) bytes: Vec<u8>,
}

impl PredicateProjectionDecision {
    /// Returns the closed decision kind.
    #[must_use]
    pub const fn kind(&self) -> PredicateProjectionKind {
        self.kind
    }

    /// Returns the deterministic ordered cause set.
    #[must_use]
    pub fn causes(&self) -> &[PredicateCause] {
        &self.causes
    }

    /// Returns canonical decision bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the admitted projection identity.
    #[must_use]
    pub const fn projection_ref(&self) -> Option<BridgeDigest> {
        self.projection_ref
    }

    /// Returns admitted correspondences, empty for every rejection.
    #[must_use]
    pub fn correspondences(&self) -> &[PredicateCorrespondence] {
        &self.correspondences
    }

    /// Returns the admitted bridge profile, absent for a rejected selection.
    #[must_use]
    pub fn bridge_profile(&self) -> Option<&str> {
        self.bridge_profile.as_deref()
    }

    /// Returns the admitted native owner contract selection.
    #[must_use]
    pub const fn native_contract(&self) -> Option<&ContractSelection> {
        self.native_contract.as_ref()
    }

    /// Returns the admitted signal-catalog owner contract selection.
    #[must_use]
    pub const fn signal_contract(&self) -> Option<&ContractSelection> {
        self.signal_contract.as_ref()
    }

    /// Returns the admitted proposition-map owner contract selection.
    #[must_use]
    pub const fn map_contract(&self) -> Option<&ContractSelection> {
        self.map_contract.as_ref()
    }

    /// Returns the exact emitted signal-catalog artifact identity.
    #[must_use]
    pub const fn signal_catalog_ref(&self) -> Option<BridgeDigest> {
        self.signal_catalog_ref
    }

    /// Returns the exact emitted proposition-map artifact identity.
    #[must_use]
    pub const fn proposition_map_ref(&self) -> Option<BridgeDigest> {
        self.proposition_map_ref
    }

    pub(crate) fn wire(&self) -> ProjectionDecisionWire {
        ProjectionDecisionWire {
            format: PROJECTION_DECISION_PROFILE.to_owned(),
            kind: self.kind,
            bridge_profile: self.bridge_profile.clone(),
            native_contract: self.native_contract.clone(),
            target_contracts: self
                .signal_contract
                .clone()
                .zip(self.map_contract.clone())
                .map(|(signal_catalog, proposition_map)| TargetContractsWire {
                    proposition_map,
                    signal_catalog,
                }),
            correspondences: admitted_option(self.kind, self.correspondences.clone()),
            signal_catalog_ref: self.signal_catalog_ref,
            proposition_map_ref: self.proposition_map_ref,
            projection_ref: self.projection_ref,
            causes: self.causes.clone(),
        }
    }

    pub(crate) fn encode(&mut self, limits: BridgeLimits) -> Result<(), BridgeError> {
        self.bytes = canonical::encode(&self.wire(), limits)?;
        Ok(())
    }
}

/// Failure returned by projection or strict projection reading.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PredicateDecision {
    /// A complete typed semantic rejection.
    Semantic(Box<PredicateProjectionDecision>),
    /// Malformed or resource-incomplete work with no semantic decision.
    Operation(BridgeError),
}

impl PredicateDecision {
    /// Returns the semantic decision, when structural admission completed.
    #[must_use]
    pub fn semantic(&self) -> Option<&PredicateProjectionDecision> {
        match self {
            Self::Semantic(value) => Some(value.as_ref()),
            Self::Operation(_) => None,
        }
    }

    /// Returns the operation error, when no decision could be formed.
    #[must_use]
    pub const fn operation(&self) -> Option<&BridgeError> {
        match self {
            Self::Semantic(_) => None,
            Self::Operation(value) => Some(value),
        }
    }
}

/// One exact completeness gap outside the selected decision-premise set.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompletenessGap {
    /// Opaque owner fact identity.
    pub fact_ref: String,
    /// Exact owner completeness-state label applying to the gap.
    pub state: String,
}

/// Canonical predicate valuation decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PredicateValuationDecision {
    pub(crate) kind: PredicateValuationKind,
    pub(crate) predicate_ref: PredicateRef,
    pub(crate) projection_ref: BridgeDigest,
    pub(crate) proposition_id: u32,
    pub(crate) signal_id: u32,
    pub(crate) availability_contract: ContractSelection,
    pub(crate) availability_identity: String,
    pub(crate) availability_revision: u64,
    pub(crate) availability_predecessor: Option<String>,
    pub(crate) availability_subject: String,
    pub(crate) population_identity: String,
    pub(crate) required_result_identity: String,
    pub(crate) result_availability: String,
    pub(crate) result_contract: Option<ContractSelection>,
    pub(crate) mapping_contract: Option<ContractSelection>,
    pub(crate) result_identity: Option<String>,
    pub(crate) result_digest: Option<String>,
    pub(crate) mapping_identity: Option<String>,
    pub(crate) native_subject_identity: Option<String>,
    pub(crate) native_correspondence_identity: Option<String>,
    pub(crate) execution: Option<String>,
    pub(crate) truth: Option<String>,
    pub(crate) settlement: Option<String>,
    pub(crate) completeness: Option<String>,
    pub(crate) decision_premises: Vec<String>,
    pub(crate) completeness_gaps: Vec<CompletenessGap>,
    pub(crate) predecessor_result_identity: Option<String>,
    pub(crate) predecessor_result_digest: Option<String>,
    pub(crate) value: Option<bool>,
    pub(crate) causes: Vec<PredicateCause>,
    pub(crate) bytes: Vec<u8>,
    pub(crate) operation_error: Option<BridgeError>,
}

impl PredicateValuationDecision {
    /// Returns the closed valuation kind.
    #[must_use]
    pub const fn kind(&self) -> PredicateValuationKind {
        self.kind
    }

    /// Returns the Boolean only for [`PredicateValuationKind::Valued`].
    #[must_use]
    pub const fn value(&self) -> Option<bool> {
        self.value
    }

    /// Returns the selected predicate identity.
    #[must_use]
    pub const fn predicate_ref(&self) -> PredicateRef {
        self.predicate_ref
    }

    /// Returns the exact admitted projection identity.
    #[must_use]
    pub const fn projection_ref(&self) -> BridgeDigest {
        self.projection_ref
    }

    /// Returns the assigned TL proposition identity.
    #[must_use]
    pub const fn proposition_id(&self) -> u32 {
        self.proposition_id
    }

    /// Returns the assigned TL signal identity.
    #[must_use]
    pub const fn signal_id(&self) -> u32 {
        self.signal_id
    }

    /// Returns the exact result-availability owner selection.
    #[must_use]
    pub const fn availability_contract(&self) -> &ContractSelection {
        &self.availability_contract
    }

    /// Returns the admitted availability assertion identity.
    #[must_use]
    pub fn availability_identity(&self) -> &str {
        &self.availability_identity
    }

    /// Returns the positive availability assertion revision.
    #[must_use]
    pub const fn availability_revision(&self) -> u64 {
        self.availability_revision
    }

    /// Returns the exact direct predecessor of a corrected availability assertion.
    #[must_use]
    pub fn availability_predecessor(&self) -> Option<&str> {
        self.availability_predecessor.as_deref()
    }

    /// Returns the exact observation scope identity.
    #[must_use]
    pub fn availability_subject(&self) -> &str {
        &self.availability_subject
    }

    /// Returns the exact observation population identity.
    #[must_use]
    pub fn population_identity(&self) -> &str {
        &self.population_identity
    }

    /// Returns the required source-result identity.
    #[must_use]
    pub fn required_result_identity(&self) -> &str {
        &self.required_result_identity
    }

    /// Returns the owner result-availability state label.
    #[must_use]
    pub fn result_availability(&self) -> &str {
        &self.result_availability
    }

    /// Returns the admitted protocol-result owner selection when a result was available.
    #[must_use]
    pub const fn result_contract(&self) -> Option<&ContractSelection> {
        self.result_contract.as_ref()
    }

    /// Returns the admitted Contract-IR mapping owner selection when available.
    #[must_use]
    pub const fn mapping_contract(&self) -> Option<&ContractSelection> {
        self.mapping_contract.as_ref()
    }

    /// Returns the source result identity when authenticated by availability.
    #[must_use]
    pub fn result_identity(&self) -> Option<&str> {
        self.result_identity.as_deref()
    }

    /// Returns the source result digest when authenticated by availability.
    #[must_use]
    pub fn result_digest(&self) -> Option<&str> {
        self.result_digest.as_deref()
    }

    /// Returns the exact owner mapping identity when available.
    #[must_use]
    pub fn mapping_identity(&self) -> Option<&str> {
        self.mapping_identity.as_deref()
    }

    /// Returns the native temporal-subject identity carried by the owner mapping.
    #[must_use]
    pub fn native_subject_identity(&self) -> Option<&str> {
        self.native_subject_identity.as_deref()
    }

    /// Returns the source-bound checked-predicate identity carried by the mapping.
    #[must_use]
    pub fn native_correspondence_identity(&self) -> Option<&str> {
        self.native_correspondence_identity.as_deref()
    }

    /// Returns the exact owner execution state label when a result was available.
    #[must_use]
    pub fn execution(&self) -> Option<&str> {
        self.execution.as_deref()
    }

    /// Returns the exact native truth label without Boolean coercion.
    #[must_use]
    pub fn truth(&self) -> Option<&str> {
        self.truth.as_deref()
    }

    /// Returns the exact owner settlement-basis label.
    #[must_use]
    pub fn settlement(&self) -> Option<&str> {
        self.settlement.as_deref()
    }

    /// Returns the exact owner completeness-state label.
    #[must_use]
    pub fn completeness(&self) -> Option<&str> {
        self.completeness.as_deref()
    }

    /// Returns the canonical exact decision-premise population.
    #[must_use]
    pub fn decision_premises(&self) -> &[String] {
        &self.decision_premises
    }

    /// Returns the mapped direct predecessor result identity for a correction.
    #[must_use]
    pub fn predecessor_result_identity(&self) -> Option<&str> {
        self.predecessor_result_identity.as_deref()
    }

    /// Returns the exact digest of the direct predecessor result.
    #[must_use]
    pub fn predecessor_result_digest(&self) -> Option<&str> {
        self.predecessor_result_digest.as_deref()
    }

    /// Returns stable ordered causes; empty only for `valued`.
    #[must_use]
    pub fn causes(&self) -> &[PredicateCause] {
        &self.causes
    }

    /// Returns completeness gaps outside the exact deciding set.
    #[must_use]
    pub fn completeness_gaps(&self) -> &[CompletenessGap] {
        &self.completeness_gaps
    }

    /// Returns canonical decision bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns an operational resource error when no canonical decision could be emitted.
    #[must_use]
    pub const fn operation(&self) -> Option<&BridgeError> {
        self.operation_error.as_ref()
    }

    pub(crate) fn wire(&self) -> ValuationDecisionWire {
        ValuationDecisionWire {
            format: VALUATION_DECISION_PROFILE.to_owned(),
            kind: self.kind,
            predicate_ref: self.predicate_ref,
            projection_ref: self.projection_ref,
            proposition_id: self.proposition_id,
            signal_id: self.signal_id,
            availability_contract: self.availability_contract.clone(),
            availability_identity: self.availability_identity.clone(),
            availability_revision: self.availability_revision,
            availability_predecessor: self.availability_predecessor.clone(),
            availability_subject: self.availability_subject.clone(),
            population_identity: self.population_identity.clone(),
            required_result_identity: self.required_result_identity.clone(),
            result_availability: self.result_availability.clone(),
            result_contract: self.result_contract.clone(),
            mapping_contract: self.mapping_contract.clone(),
            result_identity: self.result_identity.clone(),
            result_digest: self.result_digest.clone(),
            mapping_identity: self.mapping_identity.clone(),
            native_subject_identity: self.native_subject_identity.clone(),
            native_correspondence_identity: self.native_correspondence_identity.clone(),
            execution: self.execution.clone(),
            truth: self.truth.clone(),
            settlement: self.settlement.clone(),
            completeness: self.completeness.clone(),
            decision_premises: self.decision_premises.clone(),
            completeness_gaps: self.completeness_gaps.clone(),
            predecessor_result_identity: self.predecessor_result_identity.clone(),
            predecessor_result_digest: self.predecessor_result_digest.clone(),
            value: self.value,
            causes: self.causes.clone(),
        }
    }

    pub(crate) fn encode(&mut self, limits: BridgeLimits) -> Result<(), BridgeError> {
        self.bytes = canonical::encode(&self.wire(), limits).map_err(|error| {
            BridgeError::new(
                crate::bridge::BridgeErrorCode::PredicateValuationResourceExhausted,
                error.message(),
                error.path(),
            )
        })?;
        Ok(())
    }

    pub(crate) fn fail_operation(&mut self, error: BridgeError) {
        self.kind = PredicateValuationKind::Incomplete;
        self.result_contract = None;
        self.mapping_contract = None;
        self.result_identity = None;
        self.result_digest = None;
        self.mapping_identity = None;
        self.native_subject_identity = None;
        self.native_correspondence_identity = None;
        self.execution = None;
        self.truth = None;
        self.settlement = None;
        self.completeness = None;
        self.predecessor_result_identity = None;
        self.predecessor_result_digest = None;
        self.bytes.clear();
        self.value = None;
        self.causes.clear();
        self.completeness_gaps.clear();
        self.decision_premises.clear();
        self.operation_error = Some(error);
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TargetContractsWire {
    proposition_map: ContractSelection,
    signal_catalog: ContractSelection,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProjectionDecisionWire {
    format: String,
    kind: PredicateProjectionKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    bridge_profile: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    native_contract: Option<ContractSelection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    target_contracts: Option<TargetContractsWire>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    correspondences: Option<Vec<PredicateCorrespondence>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    signal_catalog_ref: Option<BridgeDigest>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    proposition_map_ref: Option<BridgeDigest>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    projection_ref: Option<BridgeDigest>,
    causes: Vec<PredicateCause>,
}

impl ProjectionDecisionWire {
    pub(crate) const fn kind(&self) -> PredicateProjectionKind {
        self.kind
    }

    pub(crate) fn validate(&self) -> bool {
        let admitted = self.kind == PredicateProjectionKind::Admitted;
        self.format == PROJECTION_DECISION_PROFILE
            && if admitted {
                self.bridge_profile.as_deref() == Some(PROFILE)
                    && self.native_contract.is_some()
                    && self.target_contracts.is_some()
                    && self
                        .correspondences
                        .as_ref()
                        .is_some_and(|values| !values.is_empty())
                    && self.signal_catalog_ref.is_some()
                    && self.proposition_map_ref.is_some()
                    && self.projection_ref.is_some()
                    && self.causes.is_empty()
            } else {
                self.bridge_profile.is_none()
                    && self.native_contract.is_none()
                    && self.target_contracts.is_none()
                    && self.correspondences.is_none()
                    && self.signal_catalog_ref.is_none()
                    && self.proposition_map_ref.is_none()
                    && self.projection_ref.is_none()
                    && !self.causes.is_empty()
            }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ValuationDecisionWire {
    format: String,
    kind: PredicateValuationKind,
    predicate_ref: PredicateRef,
    projection_ref: BridgeDigest,
    proposition_id: u32,
    signal_id: u32,
    availability_contract: ContractSelection,
    availability_identity: String,
    availability_revision: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    availability_predecessor: Option<String>,
    availability_subject: String,
    population_identity: String,
    required_result_identity: String,
    result_availability: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    result_contract: Option<ContractSelection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    mapping_contract: Option<ContractSelection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    result_identity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    result_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    mapping_identity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    native_subject_identity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    native_correspondence_identity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    execution: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    truth: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    settlement: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    completeness: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    decision_premises: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    completeness_gaps: Vec<CompletenessGap>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    predecessor_result_identity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    predecessor_result_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    value: Option<bool>,
    causes: Vec<PredicateCause>,
}

impl ValuationDecisionWire {
    pub(crate) fn validate(&self) -> bool {
        self.format == VALUATION_DECISION_PROFILE
            && self.availability_revision > 0
            && match self.kind {
                PredicateValuationKind::Valued => self.value.is_some() && self.causes.is_empty(),
                _ => self.value.is_none() && !self.causes.is_empty(),
            }
    }
}

fn admitted_option<T>(kind: PredicateProjectionKind, value: T) -> Option<T> {
    (kind == PredicateProjectionKind::Admitted).then_some(value)
}

pub(crate) fn expected_dimension(code: PredicateCauseCode) -> PredicateCauseDimension {
    use PredicateCauseCode as C;
    use PredicateCauseDimension as D;
    match code {
        C::NativeContractUnavailable | C::NativeProfileUnsupported | C::NativeContractConflict => {
            D::NativeContract
        }
        C::TargetContractUnavailable | C::TargetProfileUnsupported | C::TargetContractConflict => {
            D::TargetContract
        }
        C::SourceMismatch => D::Source,
        C::CheckedLeafMismatch => D::CheckedLeaf,
        C::ModelMismatch => D::Model,
        C::ConstructUnsupported | C::IdentityConflict => D::Predicate,
        C::PopulationInvalid | C::ValuationPopulationMismatch => D::Population,
        C::CatalogRejected => D::Catalog,
        C::MapRejected | C::CatalogMapMismatch => D::PropositionMap,
        C::AvailabilityContractUnsupported | C::AvailabilityContractUnavailable => {
            D::AvailabilityContract
        }
        C::SourceResultContractUnsupported | C::SourceResultContractUnavailable => {
            D::SourceResultContract
        }
        C::SourceResultMappingUnsupported | C::SourceResultMappingUnavailable => {
            D::SourceResultMapping
        }
        C::ProducerProfileUnsupported | C::ProducerUnavailable => D::Producer,
        C::ResultNotYetObserved | C::ObservationMismatch => D::Observation,
        C::ExecutionUnsupported
        | C::ExecutionRefused
        | C::ExecutionFailed
        | C::ExecutionIncomplete => D::PredicateExecution,
        C::AnchorMismatch => D::Anchor,
        C::CaptureMismatch => D::Capture,
        C::CompletenessIncomplete | C::CompletenessConflict => D::DecidingFacts,
        C::EvaluationProfileUnsupported | C::ResultTypeMismatch | C::ResultStale => D::Result,
        C::SupersessionInvalid => D::Supersession,
    }
}

pub(crate) fn projection_kind(causes: &[PredicateCause]) -> PredicateProjectionKind {
    if causes.iter().any(|cause| {
        matches!(
            cause.code,
            PredicateCauseCode::NativeContractConflict
                | PredicateCauseCode::TargetContractConflict
                | PredicateCauseCode::IdentityConflict
        )
    }) {
        PredicateProjectionKind::Conflict
    } else if causes.iter().any(|cause| {
        matches!(
            cause.code,
            PredicateCauseCode::SourceMismatch
                | PredicateCauseCode::CheckedLeafMismatch
                | PredicateCauseCode::ModelMismatch
                | PredicateCauseCode::PopulationInvalid
                | PredicateCauseCode::CatalogRejected
                | PredicateCauseCode::MapRejected
                | PredicateCauseCode::CatalogMapMismatch
        )
    }) {
        PredicateProjectionKind::Refused
    } else if causes.iter().any(|cause| {
        matches!(
            cause.code,
            PredicateCauseCode::NativeProfileUnsupported
                | PredicateCauseCode::TargetProfileUnsupported
                | PredicateCauseCode::ConstructUnsupported
        )
    }) {
        PredicateProjectionKind::Unsupported
    } else if causes.is_empty() {
        PredicateProjectionKind::Admitted
    } else {
        PredicateProjectionKind::Unavailable
    }
}

pub(crate) fn sort_causes(causes: &mut [PredicateCause]) {
    causes.sort_by(|left, right| {
        dimension_rank(left.dimension)
            .cmp(&dimension_rank(right.dimension))
            .then_with(|| left.code.as_str().cmp(right.code.as_str()))
            .then_with(|| {
                left.rejected_ref
                    .as_bytes()
                    .cmp(right.rejected_ref.as_bytes())
            })
            .then_with(|| left.raw_discriminator.cmp(&right.raw_discriminator))
    });
}

fn dimension_rank(dimension: PredicateCauseDimension) -> u8 {
    match dimension {
        PredicateCauseDimension::NativeContract => 0,
        PredicateCauseDimension::TargetContract => 1,
        PredicateCauseDimension::Source => 2,
        PredicateCauseDimension::CheckedLeaf => 3,
        PredicateCauseDimension::Model => 4,
        PredicateCauseDimension::Predicate => 5,
        PredicateCauseDimension::Population => 6,
        PredicateCauseDimension::Catalog => 7,
        PredicateCauseDimension::PropositionMap => 8,
        PredicateCauseDimension::Correspondence => 9,
        PredicateCauseDimension::AvailabilityContract => 10,
        PredicateCauseDimension::SourceResultContract => 11,
        PredicateCauseDimension::SourceResultMapping => 12,
        PredicateCauseDimension::Producer => 13,
        PredicateCauseDimension::Observation => 14,
        PredicateCauseDimension::PredicateExecution => 15,
        PredicateCauseDimension::Anchor => 16,
        PredicateCauseDimension::Capture => 17,
        PredicateCauseDimension::Completeness => 18,
        PredicateCauseDimension::DecidingFacts => 19,
        PredicateCauseDimension::Result => 20,
        PredicateCauseDimension::Supersession => 21,
    }
}
