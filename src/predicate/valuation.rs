//! Runtime valuation from constructor-private observation and protocol owner views.

use std::collections::BTreeSet;

use quire_observation::authority::availability::{
    State as AvailabilityState, View as ValidatedAvailability,
};
use quire_observation::authority::completeness::State as CompletenessState;
use quire_protocol::closure::AssessmentExecution;
use quire_protocol::result::contract_ir::MappedResultView;
use quire_protocol::result::{SettlementBasis, Truth};

use crate::bridge::{BridgeDigest, BridgeError, BridgeErrorCode, BridgeLimits, ContractSelection};

use super::decision::{
    sort_causes, CompletenessGap, PredicateCause, PredicateCauseCode, PredicateValuationDecision,
    PredicateValuationKind,
};
use super::{PredicateRef, ValidatedPredicateProjection};

/// Values one admitted predicate from exact observation and result-owner views.
#[must_use]
pub fn value(
    projection: &ValidatedPredicateProjection,
    predicate_ref: PredicateRef,
    availability: &ValidatedAvailability,
    mapped: Option<&MappedResultView<'_>>,
    limits: BridgeLimits,
) -> PredicateValuationDecision {
    let limits = limits.effective();
    let projection_ref = projection.decision().projection_ref().unwrap_or_else(|| {
        BridgeDigest::domain(super::PROJECTION_REF_PROFILE, projection.decision().bytes())
    });
    let correspondence = projection.correspondence(predicate_ref);
    let definition = projection.definition(predicate_ref);
    let required_allocation = availability
        .bytes()
        .len()
        .saturating_add(mapped.map_or(0, |view| view.bytes().len()))
        .saturating_add(512);
    let input_work = availability
        .bytes()
        .len()
        .saturating_add(mapped.map_or(0, |view| view.bytes().len()));
    if input_work > limits.visited_work {
        return operation_decision(
            projection_ref,
            predicate_ref,
            correspondence,
            availability,
            BridgeError::new(
                BridgeErrorCode::PredicateValuationResourceExhausted,
                "combined owner input exceeds the visited-work ceiling",
                "valuation",
            ),
        );
    }
    if required_allocation > limits.allocation_bytes {
        return operation_decision(
            projection_ref,
            predicate_ref,
            correspondence,
            availability,
            BridgeError::new(
                BridgeErrorCode::PredicateValuationResourceExhausted,
                "valuation reservation exceeds caller allocation ceiling",
                "valuation",
            ),
        );
    }
    for bytes in [
        Some(availability.bytes()),
        mapped.map(MappedResultView::bytes),
    ]
    .into_iter()
    .flatten()
    {
        if let Err(error) = crate::bridge::canonical::preflight(bytes, limits) {
            return operation_decision(
                projection_ref,
                predicate_ref,
                correspondence,
                availability,
                BridgeError::new(
                    BridgeErrorCode::PredicateValuationResourceExhausted,
                    error.message(),
                    error.path(),
                ),
            );
        }
    }
    let mut causes = Vec::new();
    if correspondence.is_none() || definition.is_none() {
        causes.push(PredicateCause::assigned(
            PredicateCauseCode::ValuationPopulationMismatch,
            predicate_ref.to_string(),
            None,
        ));
    }

    let payload = availability.payload();
    let required_result_identity = mapped
        .map(MappedResultView::source_result_id)
        .or_else(|| payload.required_results().first().map(String::as_str))
        .unwrap_or_default()
        .to_owned();
    if payload.required_results().is_empty()
        || payload.required_results().len() > limits.facts
        || (!required_result_identity.is_empty()
            && payload
                .required_results()
                .binary_search(&required_result_identity)
                .is_err())
    {
        causes.push(PredicateCause::assigned(
            PredicateCauseCode::ValuationPopulationMismatch,
            availability.subject().population_identity.as_str(),
            None,
        ));
    }

    match payload.state() {
        AvailabilityState::Available => {
            if mapped.is_none() {
                causes.push(PredicateCause::assigned(
                    PredicateCauseCode::SourceResultMappingUnavailable,
                    &required_result_identity,
                    None,
                ));
            }
        }
        AvailabilityState::NotYetObserved => causes.push(PredicateCause::assigned(
            PredicateCauseCode::ResultNotYetObserved,
            availability.identity().as_str(),
            None,
        )),
        AvailabilityState::ProducerUnavailable | AvailabilityState::ContractUnavailable => {
            causes.push(PredicateCause::assigned(
                PredicateCauseCode::ProducerUnavailable,
                availability.identity().as_str(),
                Some(availability_state_label(payload.state()).to_owned()),
            ));
        }
    }

    let mut result_contract = None;
    let mut mapping_contract = None;
    let mut result_identity = None;
    let mut result_digest = None;
    let mut mapping_identity = None;
    let mut native_subject_identity = None;
    let mut native_correspondence_identity = None;
    let mut execution = None;
    let mut truth = None;
    let mut settlement = None;
    let mut completeness = None;
    let mut decision_premises = Vec::new();
    let mut completeness_gaps = Vec::new();
    let mut predecessor_result_identity = None;
    let mut predecessor_result_digest = None;
    let mut candidate_value = None;

    if payload.state() == AvailabilityState::Available {
        if let Some(mapped) = mapped {
            result_contract = Some(result_contract_selection());
            mapping_contract = Some(mapping_contract_selection());
            result_identity = Some(mapped.source_result_id().to_owned());
            result_digest = Some(mapped.source_result_digest().to_owned());
            mapping_identity = Some(mapped.identity().to_owned());
            native_subject_identity = Some(mapped.native_subject_identity().to_owned());
            native_correspondence_identity =
                Some(mapped.native_correspondence_identity().to_owned());
            execution = Some(mapped.execution().label().to_owned());
            truth = Some(truth_label(mapped.truth()).to_owned());
            settlement = Some(settlement_label(mapped.settlement().basis).to_owned());
            completeness = Some(completeness_label(mapped.completeness().state()).to_owned());

            if payload
                .available_results()
                .binary_search_by(|candidate| candidate.as_str().cmp(mapped.source_result_id()))
                .is_err()
            {
                causes.push(PredicateCause::assigned(
                    PredicateCauseCode::ObservationMismatch,
                    mapped.source_result_id(),
                    None,
                ));
            }
            if availability.subject().population_identity.as_str()
                != mapped.completeness().population_identity()
            {
                causes.push(PredicateCause::assigned(
                    PredicateCauseCode::ValuationPopulationMismatch,
                    mapped.completeness().population_identity(),
                    None,
                ));
            }
            let decision_progress = mapped.decision_scope_progress();
            let decision_closure = mapped.decision_scope_closure();
            let scopes = [
                decision_progress.scope_identity(),
                decision_closure.scope_identity(),
            ];
            if scopes
                .iter()
                .any(|scope| *scope != availability.subject().scope_identity.as_str())
            {
                causes.push(PredicateCause::assigned(
                    PredicateCauseCode::AnchorMismatch,
                    availability.subject().scope_identity.as_str(),
                    None,
                ));
            }
            if let Some(definition) = definition {
                if mapped.native_correspondence_identity() != definition.owner_document_identity() {
                    causes.push(PredicateCause::assigned(
                        PredicateCauseCode::CheckedLeafMismatch,
                        mapped.native_correspondence_identity(),
                        None,
                    ));
                }
            }

            decision_premises = mapped
                .decision_support()
                .iter()
                .map(|support| support.observation_identity.clone())
                .collect();
            decision_premises.sort();
            decision_premises.dedup();
            if decision_premises.len() > limits.facts {
                causes.push(PredicateCause::assigned(
                    PredicateCauseCode::ValuationPopulationMismatch,
                    mapped.completeness().population_identity(),
                    None,
                ));
            }
            let deciding: BTreeSet<&str> = decision_premises.iter().map(String::as_str).collect();
            let completeness_view = mapped.completeness();
            let mapped_value = mapped.value();
            let incomplete_code = match completeness_view.state() {
                CompletenessState::Complete => None,
                CompletenessState::Incomplete => Some(PredicateCauseCode::CompletenessIncomplete),
                CompletenessState::Contradicted => Some(PredicateCauseCode::CompletenessConflict),
            };
            if let Some(code) = incomplete_code {
                for fact in completeness_view.fact_identities() {
                    if !deciding.contains(fact.as_str()) {
                        completeness_gaps.push(CompletenessGap {
                            fact_ref: fact.clone(),
                            state: completeness_label(completeness_view.state()).to_owned(),
                        });
                    }
                }
                if mapped_value.is_none() {
                    causes.push(PredicateCause::assigned(
                        code,
                        completeness_view.document_identity(),
                        None,
                    ));
                }
            }

            add_execution_cause(mapped.execution(), mapped.source_result_id(), &mut causes);
            if mapped.execution() == AssessmentExecution::Completed {
                match mapped.truth() {
                    Truth::Pending => causes.push(PredicateCause::assigned(
                        PredicateCauseCode::CompletenessIncomplete,
                        mapped.source_result_id(),
                        None,
                    )),
                    Truth::Unavailable => causes.push(PredicateCause::assigned(
                        PredicateCauseCode::SourceResultContractUnavailable,
                        mapped.source_result_id(),
                        None,
                    )),
                    Truth::Satisfied | Truth::Violated => {}
                }
            }
            candidate_value = mapped_value;
            if candidate_value.is_none() && mapped.non_value().is_none() {
                causes.push(PredicateCause::assigned(
                    PredicateCauseCode::ResultTypeMismatch,
                    mapped.source_result_id(),
                    None,
                ));
            }
            predecessor_result_identity = mapped
                .correction()
                .map(|correction| correction.predecessor_result_id().to_owned());
            predecessor_result_digest = mapped
                .correction()
                .map(|correction| correction.predecessor_digest().to_owned());
        }
    }

    sort_causes(&mut causes);
    causes.dedup();
    let cause_limit_exceeded = causes.len() > limits.causes;
    if cause_limit_exceeded {
        causes.clear();
    }
    let kind = valuation_kind(&causes);
    let value = (kind == PredicateValuationKind::Valued)
        .then_some(candidate_value)
        .flatten();
    let correspondence = correspondence.cloned();
    let mut decision = PredicateValuationDecision {
        kind,
        predicate_ref,
        projection_ref,
        proposition_id: correspondence
            .as_ref()
            .map_or(u32::MAX, super::PredicateCorrespondence::proposition_id),
        signal_id: correspondence
            .as_ref()
            .map_or(u32::MAX, super::PredicateCorrespondence::signal_id),
        availability_contract: availability_contract_selection(),
        availability_identity: availability.identity().as_str().to_owned(),
        availability_revision: availability.revision(),
        availability_predecessor: availability
            .predecessor()
            .map(|identity| identity.as_str().to_owned()),
        availability_subject: availability.subject().scope_identity.as_str().to_owned(),
        population_identity: availability
            .subject()
            .population_identity
            .as_str()
            .to_owned(),
        required_result_identity,
        result_availability: availability_state_label(payload.state()).to_owned(),
        result_contract,
        mapping_contract,
        result_identity,
        result_digest,
        mapping_identity,
        native_subject_identity,
        native_correspondence_identity,
        execution,
        truth,
        settlement,
        completeness,
        decision_premises,
        completeness_gaps,
        predecessor_result_identity,
        predecessor_result_digest,
        value,
        causes,
        bytes: Vec::new(),
        operation_error: None,
    };
    if cause_limit_exceeded {
        decision.fail_operation(BridgeError::new(
            BridgeErrorCode::PredicateValuationResourceExhausted,
            "valuation cause ceiling exceeded",
            "valuation.causes",
        ));
        return decision;
    }
    if let Err(error) = decision.encode(limits) {
        decision.fail_operation(error);
    } else if input_work.saturating_add(decision.bytes.len()) > limits.visited_work {
        decision.fail_operation(BridgeError::new(
            BridgeErrorCode::PredicateValuationResourceExhausted,
            "valuation decision exceeds the remaining visited-work ceiling",
            "valuation_decision",
        ));
    }
    decision
}

fn operation_decision(
    projection_ref: BridgeDigest,
    predicate_ref: PredicateRef,
    correspondence: Option<&super::PredicateCorrespondence>,
    availability: &ValidatedAvailability,
    error: BridgeError,
) -> PredicateValuationDecision {
    PredicateValuationDecision {
        kind: PredicateValuationKind::Incomplete,
        predicate_ref,
        projection_ref,
        proposition_id: correspondence.map_or(u32::MAX, |value| value.proposition_id()),
        signal_id: correspondence.map_or(u32::MAX, |value| value.signal_id()),
        availability_contract: availability_contract_selection(),
        availability_identity: availability.identity().as_str().to_owned(),
        availability_revision: availability.revision(),
        availability_predecessor: availability
            .predecessor()
            .map(|identity| identity.as_str().to_owned()),
        availability_subject: availability.subject().scope_identity.as_str().to_owned(),
        population_identity: availability
            .subject()
            .population_identity
            .as_str()
            .to_owned(),
        required_result_identity: String::new(),
        result_availability: availability_state_label(availability.payload().state()).to_owned(),
        result_contract: None,
        mapping_contract: None,
        result_identity: None,
        result_digest: None,
        mapping_identity: None,
        native_subject_identity: None,
        native_correspondence_identity: None,
        execution: None,
        truth: None,
        settlement: None,
        completeness: None,
        decision_premises: Vec::new(),
        completeness_gaps: Vec::new(),
        predecessor_result_identity: None,
        predecessor_result_digest: None,
        value: None,
        causes: Vec::new(),
        bytes: Vec::new(),
        operation_error: Some(error),
    }
}

fn add_execution_cause(
    execution: AssessmentExecution,
    result_identity: &str,
    causes: &mut Vec<PredicateCause>,
) {
    let code = match execution {
        AssessmentExecution::Completed => return,
        AssessmentExecution::Unsupported => PredicateCauseCode::ExecutionUnsupported,
        AssessmentExecution::Refused => PredicateCauseCode::ExecutionRefused,
        AssessmentExecution::ResourceIncomplete => PredicateCauseCode::ExecutionIncomplete,
        AssessmentExecution::Failed => PredicateCauseCode::ExecutionFailed,
    };
    causes.push(PredicateCause::assigned(code, result_identity, None));
}

fn valuation_kind(causes: &[PredicateCause]) -> PredicateValuationKind {
    let has = |wanted| causes.iter().any(|cause| cause.code() == wanted);
    if has(PredicateCauseCode::CompletenessConflict) {
        PredicateValuationKind::Conflict
    } else if causes.iter().any(|cause| {
        matches!(
            cause.code(),
            PredicateCauseCode::SourceMismatch
                | PredicateCauseCode::CheckedLeafMismatch
                | PredicateCauseCode::ModelMismatch
                | PredicateCauseCode::ExecutionRefused
                | PredicateCauseCode::ObservationMismatch
                | PredicateCauseCode::AnchorMismatch
                | PredicateCauseCode::CaptureMismatch
                | PredicateCauseCode::ValuationPopulationMismatch
                | PredicateCauseCode::ResultTypeMismatch
                | PredicateCauseCode::ResultStale
                | PredicateCauseCode::SupersessionInvalid
        )
    }) {
        PredicateValuationKind::Refused
    } else if causes.iter().any(|cause| {
        matches!(
            cause.code(),
            PredicateCauseCode::AvailabilityContractUnsupported
                | PredicateCauseCode::SourceResultContractUnsupported
                | PredicateCauseCode::SourceResultMappingUnsupported
                | PredicateCauseCode::ProducerProfileUnsupported
                | PredicateCauseCode::EvaluationProfileUnsupported
                | PredicateCauseCode::ExecutionUnsupported
        )
    }) {
        PredicateValuationKind::Unsupported
    } else if has(PredicateCauseCode::ExecutionFailed) {
        PredicateValuationKind::Failed
    } else if causes.iter().any(|cause| {
        matches!(
            cause.code(),
            PredicateCauseCode::AvailabilityContractUnavailable
                | PredicateCauseCode::SourceResultContractUnavailable
                | PredicateCauseCode::SourceResultMappingUnavailable
                | PredicateCauseCode::ProducerUnavailable
        )
    }) {
        PredicateValuationKind::Unavailable
    } else if causes.iter().any(|cause| {
        matches!(
            cause.code(),
            PredicateCauseCode::ResultNotYetObserved
                | PredicateCauseCode::ExecutionIncomplete
                | PredicateCauseCode::CompletenessIncomplete
        )
    }) {
        PredicateValuationKind::Incomplete
    } else {
        PredicateValuationKind::Valued
    }
}

fn availability_state_label(state: AvailabilityState) -> &'static str {
    match state {
        AvailabilityState::Available => "available",
        AvailabilityState::NotYetObserved => "not-yet-observed",
        AvailabilityState::ProducerUnavailable => "producer-unavailable",
        AvailabilityState::ContractUnavailable => "contract-unavailable",
    }
}

fn truth_label(truth: Truth) -> &'static str {
    match truth {
        Truth::Satisfied => "satisfied",
        Truth::Violated => "violated",
        Truth::Pending => "pending",
        Truth::Unavailable => "unavailable",
    }
}

fn settlement_label(settlement: SettlementBasis) -> &'static str {
    match settlement {
        SettlementBasis::ClosedScope => "closed-scope",
        SettlementBasis::DecisiveWitness => "decisive-witness",
        SettlementBasis::DecisiveCounterexample => "decisive-counterexample",
        SettlementBasis::Unsettled => "unsettled",
        SettlementBasis::Unavailable => "unavailable",
    }
}

fn completeness_label(state: CompletenessState) -> &'static str {
    match state {
        CompletenessState::Complete => "complete",
        CompletenessState::Incomplete => "incomplete",
        CompletenessState::Contradicted => "contradicted",
    }
}

fn availability_contract_selection() -> ContractSelection {
    ContractSelection::new(
        quire_observation::authority::availability::CONTRACT,
        "0.1.0",
        "agent-ix/quire-observation",
        "9ac80e93f4b68a2c7d5a337f9a448ad10de798fc",
        BridgeDigest::raw(quire_observation::authority::availability::SCHEMA_BYTES),
    )
}

fn result_contract_selection() -> ContractSelection {
    ContractSelection::new(
        quire_protocol::result::CONTRACT,
        "0.1.0",
        "agent-ix/quire-protocol",
        "36af8d7bb4753ea89f020fe1e5080cef21879b65",
        BridgeDigest::raw(quire_protocol::result::SCHEMA_BYTES),
    )
}

fn mapping_contract_selection() -> ContractSelection {
    ContractSelection::new(
        quire_protocol::result::contract_ir::CONTRACT,
        "0.1.0",
        "agent-ix/quire-protocol",
        "36af8d7bb4753ea89f020fe1e5080cef21879b65",
        BridgeDigest::raw(quire_protocol::result::contract_ir::SCHEMA_BYTES),
    )
}
