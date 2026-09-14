//! Closed projection/join decisions and the FR-026 diagnostic vocabulary.

use serde::{Deserialize, Serialize};

use crate::bridge::{canonical, BridgeDigest, BridgeError, BridgeLimits, ContractSelection};

use super::{JOIN_DECISION_PROFILE, PROFILE, PROJECTION_DECISION_PROFILE};

/// Semantic location of a temporal refusal or conflict.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TemporalCauseDimension {
    NativeContract,
    NativeSubject,
    PredicateProjection,
    TemporalProfile,
    Operator,
    Interval,
    ClockContract,
    Clock,
    ObservationContract,
    Observation,
    CaptureContract,
    Activation,
    Capture,
    FormulaContract,
    SemanticContract,
    EvaluatorContract,
    TraceContract,
    RequestContract,
    Formula,
    Trace,
    Request,
    Identity,
    AvailabilityContract,
    Availability,
    NativeResultContract,
    TlResultContract,
    NativeResult,
    TlResult,
    ResultBinding,
    ProgressContract,
    Progress,
    Closure,
    Truth,
    Settlement,
    Support,
    CompletenessContract,
    Completeness,
    Supersession,
}

/// Closed STD-001 FR-026 cause catalog.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporalCauseCode {
    InvalidNativeTemporalBridge,
    TemporalNativeContractUnsupported,
    TemporalNativeContractUnavailable,
    TemporalNativeContractConflict,
    TemporalSubjectMismatch,
    TemporalPredicateProjectionIncomplete,
    TemporalPredicateProjectionContractUnsupported,
    TemporalPredicateProjectionContractUnavailable,
    TemporalPredicateProjectionContractConflict,
    TemporalPredicateProjectionUnavailable,
    TemporalPredicateProjectionUnsupported,
    TemporalPredicateProjectionFailed,
    TemporalPredicateProjectionRefused,
    TemporalPredicateProjectionConflict,
    TemporalPredicateProjectionMismatch,
    TemporalProfileUnsupported,
    TemporalOperatorUnsupported,
    TemporalIntervalInvalid,
    TemporalClockIncomplete,
    TemporalClockContractUnsupported,
    TemporalClockContractUnavailable,
    TemporalClockContractConflict,
    TemporalClockUnavailable,
    TemporalClockMismatch,
    TemporalObservationIncomplete,
    TemporalObservationContractUnsupported,
    TemporalObservationContractUnavailable,
    TemporalObservationContractConflict,
    TemporalObservationMismatch,
    TemporalCaptureContractUnsupported,
    TemporalCaptureContractUnavailable,
    TemporalCaptureContractConflict,
    TemporalActivationIncomplete,
    TemporalActivationInactive,
    TemporalActivationMismatch,
    TemporalCaptureIncomplete,
    TemporalCaptureMismatch,
    TemporalFormulaContractUnsupported,
    TemporalFormulaContractUnavailable,
    TemporalFormulaContractConflict,
    TemporalSemanticContractUnsupported,
    TemporalSemanticContractUnavailable,
    TemporalSemanticContractConflict,
    TemporalEvaluatorContractUnsupported,
    TemporalEvaluatorContractUnavailable,
    TemporalEvaluatorContractConflict,
    TemporalTraceContractUnsupported,
    TemporalTraceContractUnavailable,
    TemporalTraceContractConflict,
    TemporalRequestContractUnsupported,
    TemporalRequestContractUnavailable,
    TemporalRequestContractConflict,
    TemporalFormulaRejected,
    TemporalTraceRejected,
    TemporalRequestRejected,
    TemporalIdentityConflict,
    TemporalAvailabilityContractUnsupported,
    TemporalAvailabilityContractUnavailable,
    TemporalAvailabilityContractConflict,
    TemporalAvailabilityAssertionMismatch,
    TemporalAvailabilityAssertionConflict,
    TemporalNativeResultContractUnsupported,
    TemporalNativeResultContractUnavailable,
    TemporalNativeResultContractConflict,
    TemporalTlResultContractUnsupported,
    TemporalTlResultContractUnavailable,
    TemporalTlResultContractConflict,
    TemporalNativeResultUnavailable,
    TemporalTlResultUnavailable,
    TemporalNativeResultIncomplete,
    TemporalTlResultIncomplete,
    TemporalNativeResultUnsupported,
    TemporalTlResultUnsupported,
    TemporalNativeResultFailed,
    TemporalTlResultFailed,
    TemporalNativeResultRefused,
    TemporalTlResultRefused,
    TemporalNativeResultContradicted,
    TemporalTlResultContradicted,
    TemporalResultBindingMismatch,
    TemporalResultIdentityConflict,
    TemporalProgressContractUnsupported,
    TemporalProgressContractUnavailable,
    TemporalProgressContractConflict,
    TemporalProgressMismatch,
    TemporalClosureMismatch,
    TemporalResultClosureDisagreement,
    TemporalTruthMismatch,
    TemporalSettlementMismatch,
    TemporalSupportMismatch,
    TemporalCompletenessContractUnsupported,
    TemporalCompletenessContractUnavailable,
    TemporalCompletenessContractConflict,
    TemporalCompletenessMismatch,
    TemporalSupersessionInvalid,
    TemporalProjectionResourceExhausted,
    TemporalResultJoinResourceExhausted,
}

impl TemporalCauseCode {
    /// Returns the exact STD-001 spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidNativeTemporalBridge => "invalid_native_temporal_bridge",
            Self::TemporalNativeContractUnsupported => "temporal_native_contract_unsupported",
            Self::TemporalNativeContractUnavailable => "temporal_native_contract_unavailable",
            Self::TemporalNativeContractConflict => "temporal_native_contract_conflict",
            Self::TemporalSubjectMismatch => "temporal_subject_mismatch",
            Self::TemporalPredicateProjectionIncomplete => {
                "temporal_predicate_projection_incomplete"
            }
            Self::TemporalPredicateProjectionContractUnsupported => {
                "temporal_predicate_projection_contract_unsupported"
            }
            Self::TemporalPredicateProjectionContractUnavailable => {
                "temporal_predicate_projection_contract_unavailable"
            }
            Self::TemporalPredicateProjectionContractConflict => {
                "temporal_predicate_projection_contract_conflict"
            }
            Self::TemporalPredicateProjectionUnavailable => {
                "temporal_predicate_projection_unavailable"
            }
            Self::TemporalPredicateProjectionUnsupported => {
                "temporal_predicate_projection_unsupported"
            }
            Self::TemporalPredicateProjectionFailed => "temporal_predicate_projection_failed",
            Self::TemporalPredicateProjectionRefused => "temporal_predicate_projection_refused",
            Self::TemporalPredicateProjectionConflict => "temporal_predicate_projection_conflict",
            Self::TemporalPredicateProjectionMismatch => "temporal_predicate_projection_mismatch",
            Self::TemporalProfileUnsupported => "temporal_profile_unsupported",
            Self::TemporalOperatorUnsupported => "temporal_operator_unsupported",
            Self::TemporalIntervalInvalid => "temporal_interval_invalid",
            Self::TemporalClockIncomplete => "temporal_clock_incomplete",
            Self::TemporalClockContractUnsupported => "temporal_clock_contract_unsupported",
            Self::TemporalClockContractUnavailable => "temporal_clock_contract_unavailable",
            Self::TemporalClockContractConflict => "temporal_clock_contract_conflict",
            Self::TemporalClockUnavailable => "temporal_clock_unavailable",
            Self::TemporalClockMismatch => "temporal_clock_mismatch",
            Self::TemporalObservationIncomplete => "temporal_observation_incomplete",
            Self::TemporalObservationContractUnsupported => {
                "temporal_observation_contract_unsupported"
            }
            Self::TemporalObservationContractUnavailable => {
                "temporal_observation_contract_unavailable"
            }
            Self::TemporalObservationContractConflict => "temporal_observation_contract_conflict",
            Self::TemporalObservationMismatch => "temporal_observation_mismatch",
            Self::TemporalCaptureContractUnsupported => "temporal_capture_contract_unsupported",
            Self::TemporalCaptureContractUnavailable => "temporal_capture_contract_unavailable",
            Self::TemporalCaptureContractConflict => "temporal_capture_contract_conflict",
            Self::TemporalActivationIncomplete => "temporal_activation_incomplete",
            Self::TemporalActivationInactive => "temporal_activation_inactive",
            Self::TemporalActivationMismatch => "temporal_activation_mismatch",
            Self::TemporalCaptureIncomplete => "temporal_capture_incomplete",
            Self::TemporalCaptureMismatch => "temporal_capture_mismatch",
            Self::TemporalFormulaContractUnsupported => "temporal_formula_contract_unsupported",
            Self::TemporalFormulaContractUnavailable => "temporal_formula_contract_unavailable",
            Self::TemporalFormulaContractConflict => "temporal_formula_contract_conflict",
            Self::TemporalSemanticContractUnsupported => "temporal_semantic_contract_unsupported",
            Self::TemporalSemanticContractUnavailable => "temporal_semantic_contract_unavailable",
            Self::TemporalSemanticContractConflict => "temporal_semantic_contract_conflict",
            Self::TemporalEvaluatorContractUnsupported => "temporal_evaluator_contract_unsupported",
            Self::TemporalEvaluatorContractUnavailable => "temporal_evaluator_contract_unavailable",
            Self::TemporalEvaluatorContractConflict => "temporal_evaluator_contract_conflict",
            Self::TemporalTraceContractUnsupported => "temporal_trace_contract_unsupported",
            Self::TemporalTraceContractUnavailable => "temporal_trace_contract_unavailable",
            Self::TemporalTraceContractConflict => "temporal_trace_contract_conflict",
            Self::TemporalRequestContractUnsupported => "temporal_request_contract_unsupported",
            Self::TemporalRequestContractUnavailable => "temporal_request_contract_unavailable",
            Self::TemporalRequestContractConflict => "temporal_request_contract_conflict",
            Self::TemporalFormulaRejected => "temporal_formula_rejected",
            Self::TemporalTraceRejected => "temporal_trace_rejected",
            Self::TemporalRequestRejected => "temporal_request_rejected",
            Self::TemporalIdentityConflict => "temporal_identity_conflict",
            Self::TemporalAvailabilityContractUnsupported => {
                "temporal_availability_contract_unsupported"
            }
            Self::TemporalAvailabilityContractUnavailable => {
                "temporal_availability_contract_unavailable"
            }
            Self::TemporalAvailabilityContractConflict => "temporal_availability_contract_conflict",
            Self::TemporalAvailabilityAssertionMismatch => {
                "temporal_availability_assertion_mismatch"
            }
            Self::TemporalAvailabilityAssertionConflict => {
                "temporal_availability_assertion_conflict"
            }
            Self::TemporalNativeResultContractUnsupported => {
                "temporal_native_result_contract_unsupported"
            }
            Self::TemporalNativeResultContractUnavailable => {
                "temporal_native_result_contract_unavailable"
            }
            Self::TemporalNativeResultContractConflict => {
                "temporal_native_result_contract_conflict"
            }
            Self::TemporalTlResultContractUnsupported => "temporal_tl_result_contract_unsupported",
            Self::TemporalTlResultContractUnavailable => "temporal_tl_result_contract_unavailable",
            Self::TemporalTlResultContractConflict => "temporal_tl_result_contract_conflict",
            Self::TemporalNativeResultUnavailable => "temporal_native_result_unavailable",
            Self::TemporalTlResultUnavailable => "temporal_tl_result_unavailable",
            Self::TemporalNativeResultIncomplete => "temporal_native_result_incomplete",
            Self::TemporalTlResultIncomplete => "temporal_tl_result_incomplete",
            Self::TemporalNativeResultUnsupported => "temporal_native_result_unsupported",
            Self::TemporalTlResultUnsupported => "temporal_tl_result_unsupported",
            Self::TemporalNativeResultFailed => "temporal_native_result_failed",
            Self::TemporalTlResultFailed => "temporal_tl_result_failed",
            Self::TemporalNativeResultRefused => "temporal_native_result_refused",
            Self::TemporalTlResultRefused => "temporal_tl_result_refused",
            Self::TemporalNativeResultContradicted => "temporal_native_result_contradicted",
            Self::TemporalTlResultContradicted => "temporal_tl_result_contradicted",
            Self::TemporalResultBindingMismatch => "temporal_result_binding_mismatch",
            Self::TemporalResultIdentityConflict => "temporal_result_identity_conflict",
            Self::TemporalProgressContractUnsupported => "temporal_progress_contract_unsupported",
            Self::TemporalProgressContractUnavailable => "temporal_progress_contract_unavailable",
            Self::TemporalProgressContractConflict => "temporal_progress_contract_conflict",
            Self::TemporalProgressMismatch => "temporal_progress_mismatch",
            Self::TemporalClosureMismatch => "temporal_closure_mismatch",
            Self::TemporalResultClosureDisagreement => "temporal_result_closure_disagreement",
            Self::TemporalTruthMismatch => "temporal_truth_mismatch",
            Self::TemporalSettlementMismatch => "temporal_settlement_mismatch",
            Self::TemporalSupportMismatch => "temporal_support_mismatch",
            Self::TemporalCompletenessContractUnsupported => {
                "temporal_completeness_contract_unsupported"
            }
            Self::TemporalCompletenessContractUnavailable => {
                "temporal_completeness_contract_unavailable"
            }
            Self::TemporalCompletenessContractConflict => "temporal_completeness_contract_conflict",
            Self::TemporalCompletenessMismatch => "temporal_completeness_mismatch",
            Self::TemporalSupersessionInvalid => "temporal_supersession_invalid",
            Self::TemporalProjectionResourceExhausted => "temporal_projection_resource_exhausted",
            Self::TemporalResultJoinResourceExhausted => "temporal_result_join_resource_exhausted",
        }
    }
}

/// One ordered typed cause.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemporalCause {
    dimension: TemporalCauseDimension,
    code: TemporalCauseCode,
    rejected_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    raw_discriminator: Option<String>,
}

impl TemporalCause {
    #[must_use]
    pub fn new(
        dimension: TemporalCauseDimension,
        code: TemporalCauseCode,
        rejected_ref: impl Into<String>,
        raw_discriminator: Option<String>,
    ) -> Self {
        Self {
            dimension,
            code,
            rejected_ref: rejected_ref.into(),
            raw_discriminator,
        }
    }
    #[must_use]
    pub const fn dimension(&self) -> TemporalCauseDimension {
        self.dimension
    }
    #[must_use]
    pub const fn code(&self) -> TemporalCauseCode {
        self.code
    }
    #[must_use]
    pub fn rejected_ref(&self) -> &str {
        &self.rejected_ref
    }
    #[must_use]
    pub fn raw_discriminator(&self) -> Option<&str> {
        self.raw_discriminator.as_deref()
    }
}

/// Closed projection outcome.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TemporalProjectionKind {
    Admitted,
    Unsupported,
    Incomplete,
    Unavailable,
    Refused,
    Failed,
    Conflict,
}

/// Canonical projection decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemporalProjectionDecision {
    pub(crate) kind: TemporalProjectionKind,
    pub(crate) native_contract: Option<ContractSelection>,
    pub(crate) predicate_projection_ref: Option<BridgeDigest>,
    pub(crate) subject_identity: Option<String>,
    pub(crate) semantic_profile: Option<String>,
    pub(crate) formula_identity: Option<String>,
    pub(crate) input_identity: Option<String>,
    pub(crate) native_request_identity: Option<String>,
    pub(crate) request_identity: Option<String>,
    pub(crate) correspondence_identity: Option<BridgeDigest>,
    pub(crate) causes: Vec<TemporalCause>,
    pub(crate) bytes: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProjectionWire {
    format: String,
    kind: TemporalProjectionKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    bridge_profile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    native_contract: Option<ContractSelection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    predicate_projection_ref: Option<BridgeDigest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    subject_identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    semantic_profile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    formula_identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    input_identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    native_request_identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    correspondence_identity: Option<BridgeDigest>,
    causes: Vec<TemporalCause>,
}

impl TemporalProjectionDecision {
    #[must_use]
    pub const fn kind(&self) -> TemporalProjectionKind {
        self.kind
    }
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    #[must_use]
    pub fn causes(&self) -> &[TemporalCause] {
        &self.causes
    }
    #[must_use]
    pub fn subject_identity(&self) -> Option<&str> {
        self.subject_identity.as_deref()
    }
    #[must_use]
    pub fn semantic_profile(&self) -> Option<&str> {
        self.semantic_profile.as_deref()
    }
    #[must_use]
    pub fn formula_identity(&self) -> Option<&str> {
        self.formula_identity.as_deref()
    }
    #[must_use]
    pub fn input_identity(&self) -> Option<&str> {
        self.input_identity.as_deref()
    }
    #[must_use]
    pub fn request_identity(&self) -> Option<&str> {
        self.request_identity.as_deref()
    }
    #[must_use]
    pub fn native_request_identity(&self) -> Option<&str> {
        self.native_request_identity.as_deref()
    }
    #[must_use]
    pub const fn correspondence_identity(&self) -> Option<BridgeDigest> {
        self.correspondence_identity
    }
    #[must_use]
    pub const fn predicate_projection_ref(&self) -> Option<BridgeDigest> {
        self.predicate_projection_ref
    }
    pub(crate) fn wire(&self) -> ProjectionWire {
        ProjectionWire {
            format: PROJECTION_DECISION_PROFILE.to_owned(),
            kind: self.kind,
            bridge_profile: (self.kind == TemporalProjectionKind::Admitted)
                .then(|| PROFILE.to_owned()),
            native_contract: self.native_contract.clone(),
            predicate_projection_ref: self.predicate_projection_ref,
            subject_identity: self.subject_identity.clone(),
            semantic_profile: self.semantic_profile.clone(),
            formula_identity: self.formula_identity.clone(),
            input_identity: self.input_identity.clone(),
            native_request_identity: self.native_request_identity.clone(),
            request_identity: self.request_identity.clone(),
            correspondence_identity: self.correspondence_identity,
            causes: self.causes.clone(),
        }
    }
    pub(crate) fn encode(&mut self, limits: BridgeLimits) -> Result<(), BridgeError> {
        self.bytes = canonical::encode(&self.wire(), limits)?;
        Ok(())
    }
}

/// Failure returned by projection or projection reading.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TemporalDecision {
    Semantic(Box<TemporalProjectionDecision>),
    Operation(BridgeError),
}

/// Structural comparison attached to a temporal join.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum JoinComparison {
    EqualFinal,
    EqualPending,
    NotCompared,
    Mismatch,
}

/// Closed result-join outcome.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TemporalJoinKind {
    Agreement,
    Unavailable,
    Unsupported,
    Incomplete,
    Failed,
    Refused,
    Conflict,
}

/// Lineage relation shared by both independently validated formula results.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TemporalJoinRelation {
    Original,
    Superseding,
    Invalidating,
}

/// Canonical result-join decision.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemporalJoinDecision {
    format: String,
    kind: TemporalJoinKind,
    comparison: JoinComparison,
    correspondence_identity: BridgeDigest,
    #[serde(skip_serializing_if = "Option::is_none")]
    native_result_identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    native_result_digest: Option<BridgeDigest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tl_mapping_identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tl_source_result_identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tl_source_result_digest: Option<BridgeDigest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    relation: Option<TemporalJoinRelation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    predecessor_join_digest: Option<BridgeDigest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<bool>,
    causes: Vec<TemporalCause>,
    #[serde(skip)]
    bytes: Vec<u8>,
    #[serde(skip)]
    operation_error: Option<BridgeError>,
}

pub(crate) struct JoinDecisionInput {
    pub(crate) kind: TemporalJoinKind,
    pub(crate) comparison: JoinComparison,
    pub(crate) correspondence_identity: BridgeDigest,
    pub(crate) native_result_identity: Option<String>,
    pub(crate) native_result_digest: Option<BridgeDigest>,
    pub(crate) tl_mapping_identity: Option<String>,
    pub(crate) tl_source_result_identity: Option<String>,
    pub(crate) tl_source_result_digest: Option<BridgeDigest>,
    pub(crate) relation: Option<TemporalJoinRelation>,
    pub(crate) predecessor_join_digest: Option<BridgeDigest>,
    pub(crate) value: Option<bool>,
    pub(crate) causes: Vec<TemporalCause>,
}

impl TemporalJoinDecision {
    pub(crate) fn new(mut input: JoinDecisionInput, limits: BridgeLimits) -> Self {
        input.causes.sort();
        input.causes.dedup();
        let mut result = Self {
            format: JOIN_DECISION_PROFILE.to_owned(),
            kind: input.kind,
            comparison: input.comparison,
            correspondence_identity: input.correspondence_identity,
            native_result_identity: input.native_result_identity,
            native_result_digest: input.native_result_digest,
            tl_mapping_identity: input.tl_mapping_identity,
            tl_source_result_identity: input.tl_source_result_identity,
            tl_source_result_digest: input.tl_source_result_digest,
            relation: input.relation,
            predecessor_join_digest: input.predecessor_join_digest,
            value: input.value,
            causes: input.causes,
            bytes: Vec::new(),
            operation_error: None,
        };
        match canonical::encode(&result, limits) {
            Ok(bytes) => result.bytes = bytes,
            Err(error) => result.operation_error = Some(error),
        }
        result
    }

    pub(crate) fn operation(
        correspondence_identity: BridgeDigest,
        native_result_identity: Option<String>,
        tl_mapping_identity: Option<String>,
        error: BridgeError,
    ) -> Self {
        Self {
            format: JOIN_DECISION_PROFILE.to_owned(),
            kind: TemporalJoinKind::Incomplete,
            comparison: JoinComparison::NotCompared,
            correspondence_identity,
            native_result_identity,
            native_result_digest: None,
            tl_mapping_identity,
            tl_source_result_identity: None,
            tl_source_result_digest: None,
            relation: None,
            predecessor_join_digest: None,
            value: None,
            causes: Vec::new(),
            bytes: Vec::new(),
            operation_error: Some(error),
        }
    }
    #[must_use]
    pub const fn kind(&self) -> TemporalJoinKind {
        self.kind
    }
    #[must_use]
    pub const fn comparison(&self) -> JoinComparison {
        self.comparison
    }
    #[must_use]
    pub const fn value(&self) -> Option<bool> {
        self.value
    }
    #[must_use]
    pub const fn correspondence_identity(&self) -> BridgeDigest {
        self.correspondence_identity
    }
    #[must_use]
    pub fn native_result_identity(&self) -> Option<&str> {
        self.native_result_identity.as_deref()
    }
    #[must_use]
    pub const fn native_result_digest(&self) -> Option<BridgeDigest> {
        self.native_result_digest
    }
    #[must_use]
    pub fn tl_mapping_identity(&self) -> Option<&str> {
        self.tl_mapping_identity.as_deref()
    }
    #[must_use]
    pub fn tl_source_result_identity(&self) -> Option<&str> {
        self.tl_source_result_identity.as_deref()
    }
    #[must_use]
    pub const fn tl_source_result_digest(&self) -> Option<BridgeDigest> {
        self.tl_source_result_digest
    }
    #[must_use]
    pub const fn relation(&self) -> Option<TemporalJoinRelation> {
        self.relation
    }
    #[must_use]
    pub const fn predecessor_join_digest(&self) -> Option<BridgeDigest> {
        self.predecessor_join_digest
    }
    #[must_use]
    pub fn causes(&self) -> &[TemporalCause] {
        &self.causes
    }
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    #[must_use]
    pub const fn operation_error(&self) -> Option<&BridgeError> {
        self.operation_error.as_ref()
    }
}
