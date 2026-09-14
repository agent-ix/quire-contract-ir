//! Exact owner selection and projection orchestration.

use quire_observation::authority::{
    availability, capture, clock, closure, completeness, position, progress, OpenClosed,
};
use quire_spec_language::protocol_artifact::temporal_subject::ValidatedTemporalSubject;

use crate::{
    bridge::{BridgeDigest, BridgeError, BridgeErrorCode, BridgeLimits, ContractSelection},
    predicate::{PredicateValuationDecision, ValidatedPredicateProjection},
};

use super::{
    correspondence,
    decision::{
        TemporalCause, TemporalCauseCode as Code, TemporalCauseDimension as Dim, TemporalDecision,
        TemporalProjectionDecision, TemporalProjectionKind,
    },
    formula, request, valuation, TemporalProjection,
};

/// Explicit valuations for one position. False cells remain present here.
#[derive(Clone, Copy, Debug)]
pub struct PositionValuations<'a> {
    pub position: u64,
    pub observation_identity: &'a str,
    pub valuations: &'a [PredicateValuationDecision],
}

/// Constructor-private observation authority needed for one projection.
#[derive(Clone, Copy, Debug)]
pub struct ObservationViews<'a> {
    pub position_ledger: &'a position::View,
    pub clock: &'a clock::View,
    pub capture: &'a capture::View,
    pub trigger_scope_closure: &'a closure::View,
    pub decision_scope_progress: &'a progress::View,
    pub decision_scope_closure: &'a closure::View,
    pub surrounding_execution_progress: &'a progress::View,
    pub surrounding_execution_closure: &'a closure::View,
    pub completeness: &'a completeness::View,
    pub availability: &'a availability::View,
    pub activation_guard: Option<&'a PredicateValuationDecision>,
    pub positions: &'a [PositionValuations<'a>],
    pub anchor: u64,
}

/// One independently selected owner-contract axis of the temporal bridge.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TargetContract {
    Native,
    PredicateProjection,
    PropositionMap,
    FormulaV1,
    FormulaV2,
    PastOperators,
    PositionLedger,
    Clock,
    Capture,
    Progress,
    Closure,
    Completeness,
    Availability,
    Trace,
    History,
    HistoryRequirement,
    Request,
    EvaluatorReport,
    NativeRequest,
    NativeResult,
    ProtocolResult,
    ProtocolMapping,
    TlMapping,
}

impl TargetContract {
    /// Closed declaration order used by exact-selection mutation tests.
    pub const ALL: [Self; 23] = [
        Self::Native,
        Self::PredicateProjection,
        Self::PropositionMap,
        Self::FormulaV1,
        Self::FormulaV2,
        Self::PastOperators,
        Self::PositionLedger,
        Self::Clock,
        Self::Capture,
        Self::Progress,
        Self::Closure,
        Self::Completeness,
        Self::Availability,
        Self::Trace,
        Self::History,
        Self::HistoryRequirement,
        Self::Request,
        Self::EvaluatorReport,
        Self::NativeRequest,
        Self::NativeResult,
        Self::ProtocolResult,
        Self::ProtocolMapping,
        Self::TlMapping,
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::Native => "native",
            Self::PredicateProjection => "predicate-projection",
            Self::PropositionMap => "proposition-map",
            Self::FormulaV1 => "formula-v1",
            Self::FormulaV2 => "formula-v2",
            Self::PastOperators => "past-operators",
            Self::PositionLedger => "position-ledger",
            Self::Clock => "clock",
            Self::Capture => "capture",
            Self::Progress => "progress",
            Self::Closure => "closure",
            Self::Completeness => "completeness",
            Self::Availability => "availability",
            Self::Trace => "trace",
            Self::History => "history",
            Self::HistoryRequirement => "history-requirement",
            Self::Request => "request",
            Self::EvaluatorReport => "evaluator-report",
            Self::NativeRequest => "native-request",
            Self::NativeResult => "native-result",
            Self::ProtocolResult => "protocol-result",
            Self::ProtocolMapping => "protocol-mapping",
            Self::TlMapping => "tl-mapping",
        }
    }

    const fn cause_codes(self) -> (Dim, Code, Code, Code) {
        match self {
            Self::Native => (
                Dim::NativeContract,
                Code::TemporalNativeContractUnsupported,
                Code::TemporalNativeContractUnavailable,
                Code::TemporalNativeContractConflict,
            ),
            Self::PredicateProjection => (
                Dim::PredicateProjection,
                Code::TemporalPredicateProjectionContractUnsupported,
                Code::TemporalPredicateProjectionContractUnavailable,
                Code::TemporalPredicateProjectionContractConflict,
            ),
            Self::PropositionMap | Self::FormulaV1 | Self::FormulaV2 => (
                Dim::FormulaContract,
                Code::TemporalFormulaContractUnsupported,
                Code::TemporalFormulaContractUnavailable,
                Code::TemporalFormulaContractConflict,
            ),
            Self::PastOperators => (
                Dim::SemanticContract,
                Code::TemporalSemanticContractUnsupported,
                Code::TemporalSemanticContractUnavailable,
                Code::TemporalSemanticContractConflict,
            ),
            Self::PositionLedger => (
                Dim::ObservationContract,
                Code::TemporalObservationContractUnsupported,
                Code::TemporalObservationContractUnavailable,
                Code::TemporalObservationContractConflict,
            ),
            Self::Clock => (
                Dim::ClockContract,
                Code::TemporalClockContractUnsupported,
                Code::TemporalClockContractUnavailable,
                Code::TemporalClockContractConflict,
            ),
            Self::Capture => (
                Dim::CaptureContract,
                Code::TemporalCaptureContractUnsupported,
                Code::TemporalCaptureContractUnavailable,
                Code::TemporalCaptureContractConflict,
            ),
            Self::Progress | Self::Closure => (
                Dim::ProgressContract,
                Code::TemporalProgressContractUnsupported,
                Code::TemporalProgressContractUnavailable,
                Code::TemporalProgressContractConflict,
            ),
            Self::Completeness => (
                Dim::CompletenessContract,
                Code::TemporalCompletenessContractUnsupported,
                Code::TemporalCompletenessContractUnavailable,
                Code::TemporalCompletenessContractConflict,
            ),
            Self::Availability => (
                Dim::AvailabilityContract,
                Code::TemporalAvailabilityContractUnsupported,
                Code::TemporalAvailabilityContractUnavailable,
                Code::TemporalAvailabilityContractConflict,
            ),
            Self::Trace | Self::History => (
                Dim::TraceContract,
                Code::TemporalTraceContractUnsupported,
                Code::TemporalTraceContractUnavailable,
                Code::TemporalTraceContractConflict,
            ),
            Self::HistoryRequirement | Self::Request | Self::NativeRequest => (
                Dim::RequestContract,
                Code::TemporalRequestContractUnsupported,
                Code::TemporalRequestContractUnavailable,
                Code::TemporalRequestContractConflict,
            ),
            Self::EvaluatorReport => (
                Dim::EvaluatorContract,
                Code::TemporalEvaluatorContractUnsupported,
                Code::TemporalEvaluatorContractUnavailable,
                Code::TemporalEvaluatorContractConflict,
            ),
            Self::NativeResult | Self::ProtocolResult | Self::ProtocolMapping => (
                Dim::NativeResultContract,
                Code::TemporalNativeResultContractUnsupported,
                Code::TemporalNativeResultContractUnavailable,
                Code::TemporalNativeResultContractConflict,
            ),
            Self::TlMapping => (
                Dim::TlResultContract,
                Code::TemporalTlResultContractUnsupported,
                Code::TemporalTlResultContractUnavailable,
                Code::TemporalTlResultContractConflict,
            ),
        }
    }
}

/// Exact immutable owner contracts selected before construction.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct TargetSelection {
    native: ContractSelection,
    predicate_projection: ContractSelection,
    proposition_map: ContractSelection,
    formula_v1: ContractSelection,
    formula_v2: ContractSelection,
    past_operators: ContractSelection,
    position_ledger: ContractSelection,
    clock: ContractSelection,
    capture: ContractSelection,
    progress: ContractSelection,
    closure: ContractSelection,
    completeness: ContractSelection,
    availability: ContractSelection,
    trace: ContractSelection,
    history: ContractSelection,
    history_requirement: ContractSelection,
    request: ContractSelection,
    evaluator_report: ContractSelection,
    native_request: ContractSelection,
    native_result: ContractSelection,
    protocol_result: ContractSelection,
    protocol_mapping: ContractSelection,
    tl_mapping: ContractSelection,
}

impl TargetSelection {
    #[must_use]
    pub fn current() -> Self {
        const QSL: &str = "f1700a9264d6d3bcdd07e0f77b70f3dae9ed4c07";
        const TLS: &str = "842d82553f045eb69a7f38745756d968254fc25e";
        const TLM: &str = "22862189ac4eb515ab84928faec25b2eac47d835";
        const QOBS: &str = "9ac80e93f4b68a2c7d5a337f9a448ad10de798fc";
        const QPROTOCOL: &str = "34d1752e6c5f789a52ccf115b0694eedd96cdd46";
        const QCI_PREDICATE: &str = "202210cf6339208740299ae4050d6f16908d557e";
        let select = |contract: &str, version: &str, repo: &str, revision: &str, bytes: &[u8]| {
            ContractSelection::new(contract, version, repo, revision, BridgeDigest::raw(bytes))
        };
        Self {
            native: select(
                "quire.checked-temporal-subject/v1",
                "0.2.0",
                "agent-ix/quire-spec-language",
                QSL,
                quire_spec_language::protocol_artifact::temporal_subject::SCHEMA_BYTES,
            ),
            predicate_projection: ContractSelection::new(
                crate::predicate::PROFILE,
                "0.1.0",
                "agent-ix/quire-contract-ir",
                QCI_PREDICATE,
                BridgeDigest::raw(include_bytes!(
                    "../../spec/contract/FR-025-native-predicate-tl-projection.md"
                )),
            ),
            proposition_map: select(
                "tl-syntax.proposition-map/v1",
                "0.1.0",
                "agent-ix/tl-syntax",
                TLS,
                tl_syntax::PROPOSITION_MAP_V1_SCHEMA_BYTES,
            ),
            formula_v1: select(
                "tl-syntax.formula/v1",
                "0.1.0",
                "agent-ix/tl-syntax",
                TLS,
                tl_syntax::FORMULA_V1_SCHEMA_BYTES,
            ),
            formula_v2: select(
                "tl-syntax.formula/v2",
                "0.1.0",
                "agent-ix/tl-syntax",
                TLS,
                tl_syntax::FORMULA_V2_SCHEMA_BYTES,
            ),
            past_operators: select(
                tl_syntax::PAST_OPERATORS_V1,
                "0.1.0",
                "agent-ix/tl-syntax",
                TLS,
                tl_syntax::PAST_OPERATORS_V1.as_bytes(),
            ),
            position_ledger: select(
                position::CONTRACT,
                "0.1.0",
                "agent-ix/quire-observation",
                QOBS,
                position::SCHEMA_BYTES,
            ),
            clock: select(
                clock::CONTRACT,
                "0.1.0",
                "agent-ix/quire-observation",
                QOBS,
                clock::SCHEMA_BYTES,
            ),
            capture: select(
                capture::CONTRACT,
                "0.1.0",
                "agent-ix/quire-observation",
                QOBS,
                capture::SCHEMA_BYTES,
            ),
            progress: select(
                progress::CONTRACT,
                "0.1.0",
                "agent-ix/quire-observation",
                QOBS,
                progress::SCHEMA_BYTES,
            ),
            closure: select(
                closure::CONTRACT,
                "0.1.0",
                "agent-ix/quire-observation",
                QOBS,
                closure::SCHEMA_BYTES,
            ),
            completeness: select(
                completeness::CONTRACT,
                "0.1.0",
                "agent-ix/quire-observation",
                QOBS,
                completeness::SCHEMA_BYTES,
            ),
            availability: select(
                availability::CONTRACT,
                "0.1.0",
                "agent-ix/quire-observation",
                QOBS,
                availability::SCHEMA_BYTES,
            ),
            trace: select(
                tl_mltl::wire::trace::CONTRACT,
                "0.1.0",
                "agent-ix/tl-mltl",
                TLM,
                tl_mltl::wire::trace::SCHEMA_BYTES,
            ),
            history: select(
                tl_mltl::POSITION_HISTORY_V1,
                "0.1.0",
                "agent-ix/tl-mltl",
                TLM,
                tl_mltl::past::history::SCHEMA_BYTES,
            ),
            history_requirement: select(
                tl_mltl::HISTORY_REQUIREMENT_V1,
                "0.1.0",
                "agent-ix/tl-mltl",
                TLM,
                tl_mltl::past::requirement::SCHEMA_BYTES,
            ),
            request: select(
                tl_mltl::wire::request::CONTRACT,
                "0.1.0",
                "agent-ix/tl-mltl",
                TLM,
                tl_mltl::wire::request::SCHEMA_BYTES,
            ),
            evaluator_report: select(
                tl_mltl::wire::report::CONTRACT,
                "0.1.0",
                "agent-ix/tl-mltl",
                TLM,
                tl_mltl::wire::report::SCHEMA_BYTES,
            ),
            native_result: select(
                quire_spec_language::protocol_artifact::native_temporal::result::CONTRACT,
                "0.2.0",
                "agent-ix/quire-spec-language",
                QSL,
                quire_spec_language::protocol_artifact::native_temporal::result::SCHEMA_BYTES,
            ),
            native_request: select(
                quire_spec_language::protocol_artifact::native_temporal::request::CONTRACT,
                "0.2.0",
                "agent-ix/quire-spec-language",
                QSL,
                quire_spec_language::protocol_artifact::native_temporal::request::SCHEMA_BYTES,
            ),
            protocol_result: select(
                quire_protocol::result::CONTRACT,
                "0.1.0",
                "agent-ix/quire-protocol",
                QPROTOCOL,
                quire_protocol::result::SCHEMA_BYTES,
            ),
            protocol_mapping: select(
                quire_protocol::result::contract_ir::CONTRACT,
                "0.1.0",
                "agent-ix/quire-protocol",
                QPROTOCOL,
                quire_protocol::result::contract_ir::SCHEMA_BYTES,
            ),
            tl_mapping: select(
                tl_mltl::mapping::contract_ir::CONTRACT,
                "0.1.0",
                "agent-ix/tl-mltl",
                TLM,
                tl_mltl::mapping::contract_ir::SCHEMA_BYTES,
            ),
        }
    }
    #[must_use]
    pub const fn native(&self) -> &ContractSelection {
        &self.native
    }
    #[must_use]
    pub const fn predicate_projection(&self) -> &ContractSelection {
        &self.predicate_projection
    }
    #[must_use]
    pub const fn proposition_map(&self) -> &ContractSelection {
        &self.proposition_map
    }
    #[must_use]
    pub const fn formula_v1(&self) -> &ContractSelection {
        &self.formula_v1
    }
    #[must_use]
    pub const fn formula_v2(&self) -> &ContractSelection {
        &self.formula_v2
    }
    #[must_use]
    pub const fn past_operators(&self) -> &ContractSelection {
        &self.past_operators
    }
    #[must_use]
    pub const fn position_ledger(&self) -> &ContractSelection {
        &self.position_ledger
    }
    #[must_use]
    pub const fn clock(&self) -> &ContractSelection {
        &self.clock
    }
    #[must_use]
    pub const fn capture(&self) -> &ContractSelection {
        &self.capture
    }
    #[must_use]
    pub const fn progress(&self) -> &ContractSelection {
        &self.progress
    }
    #[must_use]
    pub const fn closure(&self) -> &ContractSelection {
        &self.closure
    }
    #[must_use]
    pub const fn completeness(&self) -> &ContractSelection {
        &self.completeness
    }
    #[must_use]
    pub const fn availability(&self) -> &ContractSelection {
        &self.availability
    }
    #[must_use]
    pub const fn trace(&self) -> &ContractSelection {
        &self.trace
    }
    #[must_use]
    pub const fn history(&self) -> &ContractSelection {
        &self.history
    }
    #[must_use]
    pub const fn history_requirement(&self) -> &ContractSelection {
        &self.history_requirement
    }
    #[must_use]
    pub const fn request(&self) -> &ContractSelection {
        &self.request
    }
    #[must_use]
    pub const fn evaluator_report(&self) -> &ContractSelection {
        &self.evaluator_report
    }
    #[must_use]
    pub const fn native_request(&self) -> &ContractSelection {
        &self.native_request
    }
    #[must_use]
    pub const fn native_result(&self) -> &ContractSelection {
        &self.native_result
    }
    #[must_use]
    pub const fn protocol_result(&self) -> &ContractSelection {
        &self.protocol_result
    }
    #[must_use]
    pub const fn protocol_mapping(&self) -> &ContractSelection {
        &self.protocol_mapping
    }
    #[must_use]
    pub const fn tl_mapping(&self) -> &ContractSelection {
        &self.tl_mapping
    }

    /// Returns one exact owner-contract selection.
    #[must_use]
    pub const fn selection(&self, axis: TargetContract) -> &ContractSelection {
        match axis {
            TargetContract::Native => &self.native,
            TargetContract::PredicateProjection => &self.predicate_projection,
            TargetContract::PropositionMap => &self.proposition_map,
            TargetContract::FormulaV1 => &self.formula_v1,
            TargetContract::FormulaV2 => &self.formula_v2,
            TargetContract::PastOperators => &self.past_operators,
            TargetContract::PositionLedger => &self.position_ledger,
            TargetContract::Clock => &self.clock,
            TargetContract::Capture => &self.capture,
            TargetContract::Progress => &self.progress,
            TargetContract::Closure => &self.closure,
            TargetContract::Completeness => &self.completeness,
            TargetContract::Availability => &self.availability,
            TargetContract::Trace => &self.trace,
            TargetContract::History => &self.history,
            TargetContract::HistoryRequirement => &self.history_requirement,
            TargetContract::Request => &self.request,
            TargetContract::EvaluatorReport => &self.evaluator_report,
            TargetContract::NativeRequest => &self.native_request,
            TargetContract::NativeResult => &self.native_result,
            TargetContract::ProtocolResult => &self.protocol_result,
            TargetContract::ProtocolMapping => &self.protocol_mapping,
            TargetContract::TlMapping => &self.tl_mapping,
        }
    }

    /// Replaces one untrusted owner selection for explicit admission by [`project`].
    #[must_use]
    pub fn with_selection(mut self, axis: TargetContract, selection: ContractSelection) -> Self {
        match axis {
            TargetContract::Native => self.native = selection,
            TargetContract::PredicateProjection => self.predicate_projection = selection,
            TargetContract::PropositionMap => self.proposition_map = selection,
            TargetContract::FormulaV1 => self.formula_v1 = selection,
            TargetContract::FormulaV2 => self.formula_v2 = selection,
            TargetContract::PastOperators => self.past_operators = selection,
            TargetContract::PositionLedger => self.position_ledger = selection,
            TargetContract::Clock => self.clock = selection,
            TargetContract::Capture => self.capture = selection,
            TargetContract::Progress => self.progress = selection,
            TargetContract::Closure => self.closure = selection,
            TargetContract::Completeness => self.completeness = selection,
            TargetContract::Availability => self.availability = selection,
            TargetContract::Trace => self.trace = selection,
            TargetContract::History => self.history = selection,
            TargetContract::HistoryRequirement => self.history_requirement = selection,
            TargetContract::Request => self.request = selection,
            TargetContract::EvaluatorReport => self.evaluator_report = selection,
            TargetContract::NativeRequest => self.native_request = selection,
            TargetContract::NativeResult => self.native_result = selection,
            TargetContract::ProtocolResult => self.protocol_result = selection,
            TargetContract::ProtocolMapping => self.protocol_mapping = selection,
            TargetContract::TlMapping => self.tl_mapping = selection,
        }
        self
    }
}

/// Constructs every selected TL owner artifact without evaluating either model.
pub fn project(
    subject: &ValidatedTemporalSubject,
    predicates: &ValidatedPredicateProjection,
    observations: ObservationViews<'_>,
    target: TargetSelection,
    limits: BridgeLimits,
) -> Result<TemporalProjection, TemporalDecision> {
    let limits = limits.effective();
    validate_target(&target, limits)?;
    if subject
        .document()
        .bytes()
        .len()
        .saturating_add(predicates.decision().bytes().len())
        > limits.visited_work
    {
        return Err(TemporalDecision::Operation(BridgeError::new(
            BridgeErrorCode::TemporalProjectionResourceExhausted,
            "temporal owner inputs exceed the visited-work ceiling",
            "projection",
        )));
    }
    let closed = observations.surrounding_execution_closure.payload().state() == OpenClosed::Closed;
    let formula = formula::build(subject, predicates, closed, limits)
        .map_err(|cause| projection_failure(cause, limits))?;
    let rows = valuation::admit(
        subject,
        predicates,
        observations.position_ledger,
        observations.availability,
        observations.positions,
        limits,
    )
    .map_err(|cause| projection_failure(cause, limits))?;
    let preliminary =
        correspondence::identity(subject, predicates, observations, &formula, &target, limits)
            .map_err(TemporalDecision::Operation)?;
    let request = request::build(
        subject,
        &formula,
        predicates,
        observations,
        &rows,
        preliminary,
        limits,
    )
    .map_err(|cause| projection_failure(cause, limits))?;
    correspondence::finish(correspondence::ProjectionParts {
        subject,
        predicates,
        formula,
        rows,
        request,
        target,
        correspondence_identity: preliminary,
        limits,
    })
    .map_err(TemporalDecision::Operation)
}

fn validate_target(target: &TargetSelection, limits: BridgeLimits) -> Result<(), TemporalDecision> {
    let current = TargetSelection::current();
    for axis in TargetContract::ALL {
        let selected = target.selection(axis);
        let expected = current.selection(axis);
        if selected == expected {
            continue;
        }
        let (dimension, unsupported, unavailable, conflict) = axis.cause_codes();
        let code = if !selected.structurally_valid()
            || selected.schema_digest() != expected.schema_digest()
        {
            conflict
        } else if selected.contract() != expected.contract() {
            unsupported
        } else {
            unavailable
        };
        return Err(semantic(
            kind_for(code),
            vec![TemporalCause::new(
                dimension,
                code,
                axis.label(),
                Some(selected.contract().to_owned()),
            )],
            limits,
        ));
    }
    Ok(())
}

fn projection_failure(cause: TemporalCause, limits: BridgeLimits) -> TemporalDecision {
    if cause.code() == Code::TemporalProjectionResourceExhausted {
        TemporalDecision::Operation(BridgeError::new(
            BridgeErrorCode::TemporalProjectionResourceExhausted,
            "temporal projection resource ceiling exhausted",
            cause.rejected_ref(),
        ))
    } else {
        semantic(kind_for(cause.code()), vec![cause], limits)
    }
}

fn kind_for(code: Code) -> TemporalProjectionKind {
    match code {
        Code::TemporalProfileUnsupported
        | Code::TemporalOperatorUnsupported
        | Code::TemporalNativeContractUnsupported
        | Code::TemporalPredicateProjectionUnsupported
        | Code::TemporalPredicateProjectionContractUnsupported
        | Code::TemporalClockContractUnsupported
        | Code::TemporalObservationContractUnsupported
        | Code::TemporalCaptureContractUnsupported
        | Code::TemporalFormulaContractUnsupported
        | Code::TemporalSemanticContractUnsupported
        | Code::TemporalEvaluatorContractUnsupported
        | Code::TemporalTraceContractUnsupported
        | Code::TemporalRequestContractUnsupported
        | Code::TemporalAvailabilityContractUnsupported
        | Code::TemporalNativeResultContractUnsupported
        | Code::TemporalTlResultContractUnsupported
        | Code::TemporalProgressContractUnsupported
        | Code::TemporalCompletenessContractUnsupported => TemporalProjectionKind::Unsupported,
        Code::TemporalObservationIncomplete
        | Code::TemporalPredicateProjectionIncomplete
        | Code::TemporalClockIncomplete => TemporalProjectionKind::Incomplete,
        Code::TemporalNativeContractUnavailable
        | Code::TemporalPredicateProjectionUnavailable
        | Code::TemporalPredicateProjectionContractUnavailable
        | Code::TemporalClockContractUnavailable
        | Code::TemporalClockUnavailable
        | Code::TemporalObservationContractUnavailable
        | Code::TemporalCaptureContractUnavailable
        | Code::TemporalFormulaContractUnavailable
        | Code::TemporalSemanticContractUnavailable
        | Code::TemporalEvaluatorContractUnavailable
        | Code::TemporalTraceContractUnavailable
        | Code::TemporalRequestContractUnavailable
        | Code::TemporalAvailabilityContractUnavailable
        | Code::TemporalNativeResultContractUnavailable
        | Code::TemporalTlResultContractUnavailable
        | Code::TemporalProgressContractUnavailable
        | Code::TemporalCompletenessContractUnavailable => TemporalProjectionKind::Unavailable,
        Code::TemporalPredicateProjectionFailed => TemporalProjectionKind::Failed,
        Code::TemporalNativeContractConflict
        | Code::TemporalPredicateProjectionConflict
        | Code::TemporalPredicateProjectionContractConflict
        | Code::TemporalClockContractConflict
        | Code::TemporalObservationContractConflict
        | Code::TemporalCaptureContractConflict
        | Code::TemporalFormulaContractConflict
        | Code::TemporalSemanticContractConflict
        | Code::TemporalEvaluatorContractConflict
        | Code::TemporalTraceContractConflict
        | Code::TemporalRequestContractConflict
        | Code::TemporalIdentityConflict
        | Code::TemporalAvailabilityContractConflict
        | Code::TemporalNativeResultContractConflict
        | Code::TemporalTlResultContractConflict
        | Code::TemporalProgressContractConflict
        | Code::TemporalCompletenessContractConflict => TemporalProjectionKind::Conflict,
        _ => TemporalProjectionKind::Refused,
    }
}

fn semantic(
    kind: TemporalProjectionKind,
    mut causes: Vec<TemporalCause>,
    limits: BridgeLimits,
) -> TemporalDecision {
    causes.sort();
    causes.dedup();
    let mut decision = TemporalProjectionDecision {
        kind,
        native_contract: None,
        predicate_projection_ref: None,
        subject_identity: None,
        semantic_profile: None,
        formula_identity: None,
        input_identity: None,
        native_request_identity: None,
        request_identity: None,
        correspondence_identity: None,
        causes,
        bytes: Vec::new(),
    };
    match decision.encode(limits) {
        Ok(()) => TemporalDecision::Semantic(Box::new(decision)),
        Err(error) => TemporalDecision::Operation(BridgeError::new(
            BridgeErrorCode::TemporalProjectionResourceExhausted,
            error.message(),
            error.path(),
        )),
    }
}
