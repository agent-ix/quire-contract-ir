//! Real TL trace/history and assessment-request construction.

use std::collections::BTreeMap;

use quire_mltl::request::{
    self, ObservationInputs, RequestInput, TemporalInput, ValidatedTemporalRequest,
};
use quire_observation::authority::{self, clock::RangeRef, BoundaryRef, OpenClosed};
use quire_spec_language::protocol_artifact::{
    native_temporal::{self as native_owner, request as native_request, EvidenceRef},
    temporal_subject::ValidatedTemporalSubject,
    v2::wire::ClockConfiguration,
    wire::{Activation, TemporalOperation},
    ProtocolNumber,
};
use quire_spec_language::temporal as native_model;
use sha2::{Digest as _, Sha256};
use tl_mltl::wire::{OwnerLimits, OwnerReadError, OwnerReadErrorCode, ValidatedTrace};
use tl_mltl::{
    ClockBinding, ClockSample, ExactNumber, PositionHistoryDocument, PositionObservation,
    TraceDocument, TraceSchemaVersion,
};

use crate::{
    bridge::{BridgeDigest, BridgeLimits},
    predicate::ValidatedPredicateProjection,
};

use super::{
    admission::ObservationViews,
    decision::{TemporalCause, TemporalCauseCode as Code, TemporalCauseDimension as Dim},
    formula::{cause, BuiltFormula, Lane},
    valuation::ValuationRow,
    HISTORY_ID_PROFILE, TRACE_ID_PROFILE,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ValidatedTemporalInput {
    Future(ValidatedTrace),
    Past(tl_mltl::past::history::ValidatedPositionHistory),
}

impl ValidatedTemporalInput {
    pub(crate) fn identity(&self) -> &str {
        match self {
            Self::Future(value) => &value.document().trace_id,
            Self::Past(value) => value.document().history_id(),
        }
    }
    pub(crate) fn bytes(&self) -> &[u8] {
        match self {
            Self::Future(value) => value.bytes(),
            Self::Past(value) => value.bytes(),
        }
    }
}

pub(crate) struct BuiltRequest {
    pub(crate) native_request: native_request::ValidatedRequest,
    pub(crate) native_request_bytes: Vec<u8>,
    pub(crate) input: ValidatedTemporalInput,
    pub(crate) history_requirement: Option<tl_mltl::past::requirement::ValidatedHistoryRequirement>,
    pub(crate) request: ValidatedTemporalRequest,
    pub(crate) request_bytes: Vec<u8>,
}

pub(crate) fn build(
    subject: &ValidatedTemporalSubject,
    formula: &BuiltFormula,
    predicates: &ValidatedPredicateProjection,
    observations: ObservationViews<'_>,
    rows: &[ValuationRow],
    correspondence_identity: BridgeDigest,
    limits: BridgeLimits,
) -> Result<BuiltRequest, TemporalCause> {
    validate_observation_bindings(subject, observations, rows)?;
    let native_limits = native_owner_limits(limits);
    let native_input = native_input(
        subject,
        predicates,
        observations,
        rows,
        correspondence_identity,
    )?;
    let native_document = native_request::produce(subject, native_input, native_limits)
        .into_result()
        .map_err(|error| native_failure(error, subject.document().identity()))?;
    let native_request_bytes = native_document.bytes().to_vec();
    let native_request = native_request::read(&native_request_bytes, subject, native_limits)
        .into_result()
        .map_err(|error| native_failure(error, subject.document().identity()))?;
    let owner_limits = owner_limits(limits);
    let (input, history_requirement) = match formula.lane {
        Lane::Future => (future_input(rows, observations, owner_limits)?, None),
        Lane::Past => past_input(subject, formula, rows, observations, owner_limits)?,
    };
    let temporal_input = match &input {
        ValidatedTemporalInput::Future(value) => TemporalInput::Future(value),
        ValidatedTemporalInput::Past(value) => TemporalInput::Past(value),
    };
    let request_input = RequestInput {
        formula: &formula.document,
        proposition_map: &predicates.proposition_map,
        input: temporal_input,
        clock: observations.clock,
        subject_identity: subject.document().identity(),
        correspondence_identity: &correspondence_identity.to_string(),
        anchor: observations.anchor,
        observations: ObservationInputs {
            decision_scope_progress: observations.decision_scope_progress,
            decision_scope_closure: observations.decision_scope_closure,
            surrounding_execution_progress: observations.surrounding_execution_progress,
            surrounding_execution_closure: observations.surrounding_execution_closure,
            completeness: observations.completeness,
            availability: observations.availability,
        },
    };
    let document = request::derive(request_input, owner_limits).map_err(|error| {
        owner_failure(
            error,
            Dim::Request,
            Code::TemporalRequestRejected,
            subject.document().identity(),
        )
    })?;
    let request_bytes = document.bytes().to_vec();
    let request = request::read(&request_bytes, request_input, owner_limits).map_err(|error| {
        owner_failure(
            error,
            Dim::Request,
            Code::TemporalRequestRejected,
            subject.document().identity(),
        )
    })?;
    Ok(BuiltRequest {
        native_request,
        native_request_bytes,
        input,
        history_requirement,
        request,
        request_bytes,
    })
}

fn native_input(
    subject: &ValidatedTemporalSubject,
    predicates: &ValidatedPredicateProjection,
    observations: ObservationViews<'_>,
    rows: &[ValuationRow],
    correspondence_identity: BridgeDigest,
) -> Result<native_request::Input, TemporalCause> {
    let leaf_population = u64::try_from(subject.predicate_leaves().unwrap_or_default().len())
        .map_err(|_| resource("native.positions.population"))?;
    let position_population =
        u64::try_from(rows.len()).map_err(|_| resource("native.completeness.population"))?;
    let clock = clock_binding(subject, observations.clock)?;
    let positions = rows
        .iter()
        .map(|row| {
            let coordinate = i64::try_from(row.position)
                .map_err(|_| clock_error("native.position.coordinate"))?;
            let order =
                matches!(clock, ClockBinding::EventPosition).then(|| native_model::OrderKey {
                    authority: observations.clock.payload().clock_identity().to_owned(),
                    key: coordinate,
                });
            let digest = observation_record_digest(&row.observation_identity)?;
            Ok(native_request::ObservedPosition {
                observation: evidence(
                    authority::observation::CONTRACT,
                    authority::observation::SCHEMA_SHA256,
                    &row.observation_identity,
                    digest,
                    observations.position_ledger.authority(),
                    "observation",
                    leaf_population,
                ),
                position: native_model::Position {
                    coordinate,
                    order,
                    valuations: native_position_values(subject, predicates, row)?,
                },
            })
        })
        .collect::<Result<Vec<_>, TemporalCause>>()?;
    let captures = native_captures(subject, observations)?;
    let trigger_identity = observations.capture.payload().trigger_identity().to_owned();
    let guard = match subject.activation() {
        Some(Activation::Each {
            guard: quire_spec_language::protocol_artifact::wire::Nullable(Some(handle)),
            ..
        }) => {
            let valuation = observations.activation_guard.ok_or_else(|| {
                cause(
                    Dim::Activation,
                    Code::TemporalActivationIncomplete,
                    format!("{}:{}", handle.declaration, handle.index),
                    None,
                )
            })?;
            let anchor_observation = rows
                .iter()
                .find(|row| row.position == observations.anchor)
                .ok_or_else(|| {
                    cause(
                        Dim::Activation,
                        Code::TemporalActivationMismatch,
                        observations.anchor.to_string(),
                        None,
                    )
                })?;
            let projection_ref = predicates.decision().projection_ref().ok_or_else(|| {
                cause(
                    Dim::PredicateProjection,
                    Code::TemporalPredicateProjectionUnavailable,
                    "projection-ref",
                    None,
                )
            })?;
            super::valuation::validate_cell(
                valuation,
                projection_ref,
                observations.availability.identity().as_str(),
                &anchor_observation.observation_identity,
            )?;
            let definition = predicates
                .definition(valuation.predicate_ref())
                .ok_or_else(|| {
                    cause(
                        Dim::Activation,
                        Code::TemporalActivationMismatch,
                        valuation.predicate_ref().to_string(),
                        None,
                    )
                })?;
            if (definition.leaf_declaration(), definition.leaf_index())
                != (handle.declaration, handle.index)
            {
                return Err(cause(
                    Dim::Activation,
                    Code::TemporalActivationMismatch,
                    valuation.predicate_ref().to_string(),
                    None,
                ));
            }
            Some(valuation.value().ok_or_else(|| {
                cause(
                    Dim::Activation,
                    Code::TemporalActivationIncomplete,
                    valuation.predicate_ref().to_string(),
                    None,
                )
            })?)
        }
        Some(Activation::Each {
            guard: quire_spec_language::protocol_artifact::wire::Nullable(None),
            ..
        })
        | Some(Activation::Origin { .. }) => {
            if observations.activation_guard.is_some() {
                return Err(cause(
                    Dim::Activation,
                    Code::TemporalActivationMismatch,
                    "unexpected-guard",
                    None,
                ));
            }
            None
        }
        None => {
            return Err(cause(
                Dim::Activation,
                Code::TemporalActivationIncomplete,
                subject.document().identity(),
                None,
            ))
        }
    };
    let trigger = native_model::Trigger {
        identity: trigger_identity.clone(),
        receipt: observations.capture.identity().as_str().to_owned(),
        anchor: observations.anchor.to_string(),
        payload: BridgeDigest::raw(observations.capture.bytes()).to_string(),
        guard,
        captures,
    };
    let completeness_facts = observations
        .completeness
        .payload()
        .facts()
        .map(|fact| {
            let digest = BridgeDigest::domain(
                "quire.contract.native-temporal-completeness-fact-ref/v1",
                format!(
                    "{}\0{}",
                    fact.member_identity,
                    fact.observation_identity.unwrap_or_default()
                )
                .as_bytes(),
            );
            evidence(
                authority::completeness::CONTRACT,
                authority::completeness::SCHEMA_SHA256,
                fact.member_identity,
                digest.to_string(),
                observations.completeness.authority(),
                "completeness-fact",
                leaf_population,
            )
        })
        .collect();
    Ok(native_request::Input {
        instance: trigger_identity,
        correspondence: EvidenceRef::new(
            super::PROFILE,
            BridgeDigest::raw(include_bytes!(
                "../../spec/contract/FR-026-native-temporal-tl-correspondence.md"
            ))
            .to_string(),
            correspondence_identity.to_string(),
            correspondence_identity.to_string(),
            "agent-ix/quire-contract-ir",
            "980fcc0cccf18c95495fce279440a73c49d4f5c5",
            BridgeDigest::raw(include_bytes!(
                "../../spec/contract/FR-026-native-temporal-tl-correspondence.md"
            ))
            .to_string(),
            "correspondence",
            1,
        ),
        positions,
        anchor: observations.anchor.to_string(),
        triggers: vec![trigger],
        trigger_evidence: native_model::Evidence::Admitted,
        trigger_scope: closure_state(observations.trigger_scope_closure.payload().state()),
        decision_progress: native_request::ProgressInput {
            reference: observation_evidence(
                observations.decision_scope_progress,
                authority::progress::CONTRACT,
                authority::progress::SCHEMA_SHA256,
                "decision-progress",
            ),
            watermark: watermark(observations.decision_scope_progress.payload().boundary())?,
        },
        decision_closure: native_request::ClosureInput {
            reference: observation_evidence(
                observations.decision_scope_closure,
                authority::closure::CONTRACT,
                authority::closure::SCHEMA_SHA256,
                "decision-closure",
            ),
            state: closure_state(observations.decision_scope_closure.payload().state()),
        },
        surrounding_progress: native_request::ProgressInput {
            reference: observation_evidence(
                observations.surrounding_execution_progress,
                authority::progress::CONTRACT,
                authority::progress::SCHEMA_SHA256,
                "surrounding-progress",
            ),
            watermark: watermark(
                observations
                    .surrounding_execution_progress
                    .payload()
                    .boundary(),
            )?,
        },
        surrounding_closure: native_request::ClosureInput {
            reference: observation_evidence(
                observations.surrounding_execution_closure,
                authority::closure::CONTRACT,
                authority::closure::SCHEMA_SHA256,
                "surrounding-closure",
            ),
            state: closure_state(observations.surrounding_execution_closure.payload().state()),
        },
        execution: native_model::Execution::Completed,
        completeness: native_request::CompletenessInput {
            reference: observation_evidence_with_population(
                observations.completeness,
                authority::completeness::CONTRACT,
                authority::completeness::SCHEMA_SHA256,
                "completeness",
                position_population,
            ),
            state: native_model::Completeness::Complete,
            facts: completeness_facts,
        },
        authoritative_origin: subject.history_boundary() == Some("execution-origin"),
        evicted: Vec::new(),
    })
}

fn observation_record_digest(identity: &str) -> Result<String, TemporalCause> {
    let digest = identity.strip_prefix("sha256-jcs:").ok_or_else(|| {
        cause(
            Dim::Observation,
            Code::TemporalObservationMismatch,
            identity,
            None,
        )
    })?;
    BridgeDigest::parse(digest)
        .map(|value| value.to_string())
        .map_err(|_| {
            cause(
                Dim::Observation,
                Code::TemporalObservationMismatch,
                identity,
                None,
            )
        })
}

fn native_position_values(
    subject: &ValidatedTemporalSubject,
    predicates: &ValidatedPredicateProjection,
    row: &ValuationRow,
) -> Result<BTreeMap<u32, bool>, TemporalCause> {
    let by_leaf = predicates
        .correspondences()
        .iter()
        .map(|correspondence| {
            predicates
                .definition(correspondence.predicate_ref())
                .map(|definition| {
                    (
                        (definition.leaf_declaration(), definition.leaf_index()),
                        correspondence.proposition_id(),
                    )
                })
                .ok_or_else(|| {
                    cause(
                        Dim::PredicateProjection,
                        Code::TemporalPredicateProjectionMismatch,
                        correspondence.predicate_ref().to_string(),
                        None,
                    )
                })
        })
        .collect::<Result<BTreeMap<_, _>, TemporalCause>>()?;
    subject
        .temporal_nodes()
        .filter_map(|(node, temporal)| match &temporal.operation {
            TemporalOperation::Holds { value } => Some((node, value)),
            _ => None,
        })
        .map(|(node, value)| {
            let proposition = by_leaf
                .get(&(value.declaration, value.index))
                .ok_or_else(|| {
                    cause(
                        Dim::PredicateProjection,
                        Code::TemporalPredicateProjectionIncomplete,
                        format!("{}:{}", value.declaration, value.index),
                        None,
                    )
                })?;
            Ok((
                node,
                row.true_propositions
                    .binary_search(&tl_syntax::PropositionId(*proposition))
                    .is_ok(),
            ))
        })
        .collect()
}

fn future_input(
    rows: &[ValuationRow],
    observations: ObservationViews<'_>,
    limits: OwnerLimits,
) -> Result<ValidatedTemporalInput, TemporalCause> {
    let closed = observations.surrounding_execution_closure.payload().state() == OpenClosed::Closed;
    let seed = valuation_preimage(rows, closed);
    let trace = TraceDocument {
        schema_version: TraceSchemaVersion::V1,
        trace_id: BridgeDigest::domain(TRACE_ID_PROFILE, &seed).to_string(),
        closed,
        instants: rows
            .iter()
            .map(|row| row.true_propositions.clone())
            .collect(),
    };
    let document = tl_mltl::wire::trace::derive(&trace, limits)
        .map_err(|error| owner_failure(error, Dim::Trace, Code::TemporalTraceRejected, "trace"))?;
    let validated = tl_mltl::wire::trace::read(document.bytes(), &trace, limits)
        .map_err(|error| owner_failure(error, Dim::Trace, Code::TemporalTraceRejected, "trace"))?;
    Ok(ValidatedTemporalInput::Future(validated))
}

fn past_input(
    subject: &ValidatedTemporalSubject,
    formula: &BuiltFormula,
    rows: &[ValuationRow],
    observations: ObservationViews<'_>,
    limits: OwnerLimits,
) -> Result<
    (
        ValidatedTemporalInput,
        Option<tl_mltl::past::requirement::ValidatedHistoryRequirement>,
    ),
    TemporalCause,
> {
    if rows.is_empty() {
        return Err(cause(
            Dim::Observation,
            Code::TemporalObservationIncomplete,
            "history",
            None,
        ));
    }
    let clock = clock_binding(subject, observations.clock)?;
    let history_rows =
        rows.iter()
            .map(|row| {
                let sample = match &clock {
                    ClockBinding::FixedSample {
                        epoch,
                        period,
                        unit,
                    } => Some(ClockSample {
                        instant: tl_mltl::fixed_sample_instant(*epoch, *period, row.position)
                            .map_err(|_| {
                                cause(
                                    Dim::Clock,
                                    Code::TemporalClockMismatch,
                                    row.position.to_string(),
                                    None,
                                )
                            })?,
                        unit: unit.clone(),
                    }),
                    ClockBinding::EventPosition => None,
                    ClockBinding::Unsupported { .. } => {
                        return Err(cause(
                            Dim::Clock,
                            Code::TemporalProfileUnsupported,
                            "clock",
                            None,
                        ))
                    }
                };
                Ok(PositionObservation::new(
                    row.position,
                    row.true_propositions.clone(),
                    sample,
                ))
            })
            .collect::<Result<Vec<_>, TemporalCause>>()?;
    let seed = valuation_preimage(rows, true);
    let history_id = BridgeDigest::domain(HISTORY_ID_PROFILE, &seed).to_string();
    let through = rows.last().map_or(0, |row| row.position);
    let history =
        PositionHistoryDocument::new(history_id, 1, 0, through, Some(clock), history_rows)
            .map_err(|_| cause(Dim::Trace, Code::TemporalTraceRejected, "history", None))?;
    let bytes = tl_mltl::past::history::derive(&history, limits)
        .map_err(|error| owner_failure(error, Dim::Trace, Code::TemporalTraceRejected, "history"))?
        .bytes()
        .to_vec();
    let history = tl_mltl::past::history::read(&bytes, &history, limits).map_err(|error| {
        owner_failure(error, Dim::Trace, Code::TemporalTraceRejected, "history")
    })?;
    let formula_view = formula.document.validate().map_err(|_| {
        cause(
            Dim::Formula,
            Code::TemporalFormulaRejected,
            &formula.identity,
            None,
        )
    })?;
    let report = tl_mltl::analyze_required_history(formula_view, formula.identity.clone())
        .map_err(|_| {
            cause(
                Dim::TemporalProfile,
                Code::TemporalProfileUnsupported,
                &formula.identity,
                None,
            )
        })?;
    let requirement_bytes = tl_mltl::past::requirement::derive(&report, limits)
        .map_err(|error| {
            owner_failure(
                error,
                Dim::Request,
                Code::TemporalRequestRejected,
                "history-requirement",
            )
        })?
        .bytes()
        .to_vec();
    let requirement = tl_mltl::past::requirement::read(&requirement_bytes, &report, limits)
        .map_err(|error| {
            owner_failure(
                error,
                Dim::Request,
                Code::TemporalRequestRejected,
                "history-requirement",
            )
        })?;
    Ok((ValidatedTemporalInput::Past(history), Some(requirement)))
}

trait ObservationArtifact {
    fn artifact_identity(&self) -> &str;
    fn artifact_authority(&self) -> &authority::AuthoritySelection;
    fn artifact_bytes(&self) -> &[u8];
}

macro_rules! observation_artifact {
    ($($view:path),+ $(,)?) => {
        $(
            impl ObservationArtifact for $view {
                fn artifact_identity(&self) -> &str { self.identity().as_str() }
                fn artifact_authority(&self) -> &authority::AuthoritySelection { self.authority() }
                fn artifact_bytes(&self) -> &[u8] { self.bytes() }
            }
        )+
    };
}

observation_artifact!(
    authority::progress::View,
    authority::closure::View,
    authority::completeness::View,
);

fn observation_evidence<T: ObservationArtifact>(
    value: &T,
    contract: &str,
    schema_digest: &str,
    scope: &str,
) -> EvidenceRef {
    observation_evidence_with_population(value, contract, schema_digest, scope, 1)
}

fn observation_evidence_with_population<T: ObservationArtifact>(
    value: &T,
    contract: &str,
    schema_digest: &str,
    scope: &str,
    population: u64,
) -> EvidenceRef {
    evidence(
        contract,
        schema_digest,
        value.artifact_identity(),
        BridgeDigest::raw(value.artifact_bytes()).to_string(),
        value.artifact_authority(),
        scope,
        population,
    )
}

fn evidence(
    contract: &str,
    schema_digest: &str,
    identity: &str,
    digest: String,
    authority: &authority::AuthoritySelection,
    scope: &str,
    population: u64,
) -> EvidenceRef {
    EvidenceRef::new(
        contract,
        schema_digest,
        identity,
        digest,
        authority.definition_identity.as_str(),
        authority.definition_revision.as_str(),
        hex_bytes(authority.definition_digest.as_bytes()),
        scope,
        population,
    )
}

fn hex_bytes(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut result = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        result.push(char::from(HEX[usize::from(byte >> 4)]));
        result.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    result
}

fn native_captures(
    subject: &ValidatedTemporalSubject,
    observations: ObservationViews<'_>,
) -> Result<Vec<native_model::CaptureInput>, TemporalCause> {
    let by_identity = observations
        .capture
        .payload()
        .bindings()
        .map(|binding| (binding.capture_identity, binding.canonical_value))
        .collect::<BTreeMap<_, _>>();
    subject
        .captures()
        .unwrap_or_default()
        .iter()
        .map(|capture| {
            let identity = format!("{}:{}", capture.declaration, capture.index);
            by_identity
                .get(identity.as_str())
                .map(|value| native_model::CaptureInput::Value {
                    anchor: observations.anchor.to_string(),
                    value: (*value).to_owned(),
                })
                .ok_or_else(|| {
                    cause(
                        Dim::Capture,
                        Code::TemporalCaptureIncomplete,
                        identity,
                        None,
                    )
                })
        })
        .collect()
}

fn watermark(boundary: BoundaryRef<'_>) -> Result<i64, TemporalCause> {
    let value = match boundary {
        BoundaryRef::EventPosition { watermark, .. }
        | BoundaryRef::FixedSample { watermark, .. } => watermark,
        BoundaryRef::TimestampedEvent {
            watermark_nanos, ..
        } => watermark_nanos,
    };
    value.parse::<i64>().map_err(|_| clock_error(value))
}

const fn closure_state(state: OpenClosed) -> native_model::Closure {
    match state {
        OpenClosed::Open => native_model::Closure::Open,
        OpenClosed::Closed => native_model::Closure::Closed,
    }
}

fn native_failure(error: native_owner::Error, rejected: impl Into<String>) -> TemporalCause {
    let code = if matches!(
        error.code(),
        native_owner::ErrorCode::ResourceIncomplete | native_owner::ErrorCode::Allocation
    ) {
        Code::TemporalProjectionResourceExhausted
    } else {
        Code::TemporalRequestRejected
    };
    cause(
        Dim::Request,
        code,
        format!("{}.{}", rejected.into(), error.path()),
        Some(error.code().as_str().to_owned()),
    )
}

fn resource(path: &str) -> TemporalCause {
    cause(
        Dim::Request,
        Code::TemporalProjectionResourceExhausted,
        path,
        None,
    )
}

fn owner_failure(
    error: OwnerReadError,
    dimension: Dim,
    semantic_code: Code,
    rejected: impl Into<String>,
) -> TemporalCause {
    let rejected = rejected.into();
    if matches!(
        error.code(),
        OwnerReadErrorCode::ResourceIncomplete | OwnerReadErrorCode::Encoding
    ) {
        cause(
            dimension,
            Code::TemporalProjectionResourceExhausted,
            format!("{rejected}.{}", error.field()),
            Some(error.code().as_str().to_owned()),
        )
    } else {
        cause(
            dimension,
            semantic_code,
            rejected,
            Some(error.code().as_str().to_owned()),
        )
    }
}

fn validate_observation_bindings(
    native: &ValidatedTemporalSubject,
    observations: ObservationViews<'_>,
    rows: &[ValuationRow],
) -> Result<(), TemporalCause> {
    let subject = observations.position_ledger.subject();
    let expected = (
        subject.scope_identity.as_str(),
        subject.population_identity.as_str(),
    );
    for candidate in [
        observations.clock.subject(),
        observations.capture.subject(),
        observations.trigger_scope_closure.subject(),
        observations.decision_scope_progress.subject(),
        observations.decision_scope_closure.subject(),
        observations.completeness.subject(),
        observations.availability.subject(),
    ] {
        if (
            candidate.scope_identity.as_str(),
            candidate.population_identity.as_str(),
        ) != expected
        {
            return Err(cause(
                Dim::Observation,
                Code::TemporalObservationMismatch,
                candidate.scope_identity.as_str(),
                None,
            ));
        }
    }
    let surrounding_progress = observations.surrounding_execution_progress.subject();
    let surrounding_closure = observations.surrounding_execution_closure.subject();
    if surrounding_progress != surrounding_closure {
        return Err(cause(
            Dim::Observation,
            Code::TemporalObservationMismatch,
            surrounding_progress.scope_identity.as_str(),
            None,
        ));
    }
    if observations.position_ledger.payload().clock_identity()
        != observations.clock.payload().clock_identity()
        || observations.position_ledger.payload().clock_revision()
            != observations.clock.payload().clock_revision()
    {
        return Err(cause(
            Dim::Clock,
            Code::TemporalClockMismatch,
            observations.clock.identity().as_str(),
            None,
        ));
    }
    let _ = clock_binding(native, observations.clock)?;
    validate_activation(native, observations.capture)?;
    let anchor_matches = match observations.capture.payload().anchor() {
        quire_observation::authority::observation::AnchorRef::EventPosition { position } => {
            position.parse() == Ok(observations.anchor)
        }
        quire_observation::authority::observation::AnchorRef::FixedSample { index, .. } => {
            index.parse() == Ok(observations.anchor)
        }
        quire_observation::authority::observation::AnchorRef::Timestamp { .. } => false,
    };
    if !anchor_matches
        || rows
            .last()
            .is_none_or(|row| observations.anchor > row.position)
    {
        return Err(cause(
            Dim::Activation,
            Code::TemporalActivationMismatch,
            observations.capture.identity().as_str(),
            None,
        ));
    }
    if observations.completeness.payload().state()
        != quire_observation::authority::completeness::State::Complete
    {
        return Err(cause(
            Dim::Observation,
            Code::TemporalObservationIncomplete,
            observations.completeness.identity().as_str(),
            None,
        ));
    }
    Ok(())
}

fn clock_binding(
    subject: &ValidatedTemporalSubject,
    clock: &quire_observation::authority::clock::View,
) -> Result<ClockBinding, TemporalCause> {
    let (_, configured) = subject.clock().ok_or_else(|| {
        cause(
            Dim::Clock,
            Code::TemporalClockIncomplete,
            subject.document().identity(),
            None,
        )
    })?;
    match (configured, clock.payload().selection()) {
        (
            ClockConfiguration::EventPosition { sequence_authority },
            RangeRef::EventPosition { start: "0", .. },
        ) if sequence_authority == clock.payload().clock_identity() => {
            Ok(ClockBinding::EventPosition)
        }
        (
            ClockConfiguration::FixedSample {
                epoch,
                period,
                unit,
            },
            RangeRef::FixedSample {
                start: "0",
                epoch_nanos,
                period_nanos,
                ..
            },
        ) => {
            let expected_epoch =
                exact_nanos(epoch.checked().map_err(|_| clock_error("epoch"))?, unit)?;
            let expected_period =
                exact_nanos(period.checked().map_err(|_| clock_error("period"))?, unit)?;
            let observed_epoch = epoch_nanos
                .parse::<i64>()
                .map_err(|_| clock_error(epoch_nanos))?;
            let observed_period = period_nanos
                .parse::<i64>()
                .map_err(|_| clock_error(period_nanos))?;
            if expected_epoch != observed_epoch || expected_period != observed_period {
                return Err(clock_error("fixed-sample"));
            }
            Ok(ClockBinding::FixedSample {
                epoch: ExactNumber::new(observed_epoch, 1).map_err(|_| clock_error(epoch_nanos))?,
                period: ExactNumber::new(observed_period, 1)
                    .map_err(|_| clock_error(period_nanos))?,
                unit: "nanoseconds".to_owned(),
            })
        }
        (ClockConfiguration::TimestampedEvent { .. }, RangeRef::TimestampedEvent { .. }) => {
            Err(cause(
                Dim::TemporalProfile,
                Code::TemporalProfileUnsupported,
                "timestamped-event",
                Some("timestamped-event".to_owned()),
            ))
        }
        _ => Err(clock_error("clock-range")),
    }
}

fn exact_nanos(value: ProtocolNumber, unit: &str) -> Result<i64, TemporalCause> {
    let scale = match unit {
        "nanosecond" | "nanoseconds" => 1_i128,
        "microsecond" | "microseconds" => 1_000,
        "millisecond" | "milliseconds" => 1_000_000,
        "second" | "seconds" => 1_000_000_000,
        "minute" | "minutes" => 60_000_000_000,
        _ => return Err(clock_error(unit)),
    };
    let (numerator, denominator) = match value {
        ProtocolNumber::Integer(value) => (i128::from(value.value()), 1_i128),
        ProtocolNumber::Rational(value) => (
            i128::from(value.numerator()),
            i128::from(value.denominator()),
        ),
    };
    let scaled = numerator
        .checked_mul(scale)
        .ok_or_else(|| clock_error(unit))?;
    if scaled % denominator != 0 {
        return Err(clock_error(unit));
    }
    i64::try_from(scaled / denominator).map_err(|_| clock_error(unit))
}

fn validate_activation(
    subject: &ValidatedTemporalSubject,
    capture: &quire_observation::authority::capture::View,
) -> Result<(), TemporalCause> {
    let activation = subject.activation().ok_or_else(|| {
        cause(
            Dim::Activation,
            Code::TemporalActivationIncomplete,
            subject.document().identity(),
            None,
        )
    })?;
    let trigger = match activation {
        Activation::Origin { anchor } => anchor,
        Activation::Each { trigger, .. } => trigger,
    };
    let expected_trigger = format!("{}:{}", trigger.declaration, trigger.index);
    if capture.payload().trigger_identity() != expected_trigger {
        return Err(cause(
            Dim::Activation,
            Code::TemporalActivationMismatch,
            capture.payload().trigger_identity(),
            None,
        ));
    }
    let expected_captures = subject.captures().unwrap_or_default();
    let actual = capture.payload().bindings().collect::<Vec<_>>();
    if expected_captures.iter().any(|handle| {
        let identity = format!("{}:{}", handle.declaration, handle.index);
        actual
            .binary_search_by(|binding| binding.capture_identity.cmp(identity.as_str()))
            .is_err()
    }) {
        return Err(cause(
            Dim::Capture,
            Code::TemporalCaptureMismatch,
            capture.identity().as_str(),
            None,
        ));
    }
    Ok(())
}

fn valuation_preimage(rows: &[ValuationRow], closed: bool) -> Vec<u8> {
    let mut digest = Sha256::new();
    digest.update(if closed {
        b"closed".as_slice()
    } else {
        b"open".as_slice()
    });
    for row in rows {
        digest.update(row.position.to_be_bytes());
        digest.update(row.observation_identity.as_bytes());
        for proposition in &row.true_propositions {
            digest.update(proposition.0.to_be_bytes());
        }
    }
    digest.finalize().to_vec()
}

fn clock_error(value: &str) -> TemporalCause {
    cause(Dim::Clock, Code::TemporalClockMismatch, value, None)
}

pub(crate) fn owner_limits(limits: BridgeLimits) -> OwnerLimits {
    let limits = limits.effective();
    OwnerLimits {
        max_input_bytes: limits.document_bytes,
        max_output_bytes: limits.document_bytes,
        max_depth: limits.json_depth,
        max_string_bytes: limits.string_bytes,
        max_formula_nodes: limits.formula_nodes,
        max_formula_depth: limits.formula_depth,
        max_positions: limits.positions,
        max_propositions: limits.valuations,
        max_support: limits.facts,
        max_history_span: u64::try_from(limits.positions).unwrap_or(u64::MAX),
        max_evaluation_steps: u64::try_from(limits.visited_work).unwrap_or(u64::MAX),
        max_recursion_depth: u32::try_from(limits.formula_depth).unwrap_or(u32::MAX),
        max_visited_fields: limits.visited_work,
    }
}

pub(crate) fn native_owner_limits(limits: BridgeLimits) -> native_owner::Limits {
    let limits = limits.effective();
    native_owner::Limits {
        input_bytes: limits.document_bytes,
        output_bytes: limits.document_bytes,
        json_depth: limits.json_depth,
        string_bytes: limits.string_bytes,
        formula_nodes: limits.formula_nodes,
        formula_depth: limits.formula_depth,
        positions: limits.positions,
        valuations: limits.valuations,
        captures: limits.facts,
        support: limits.facts,
        history_span: limits.positions,
        evaluation_steps: limits.visited_work,
        lineage: limits.positions,
        visited: limits.visited_work,
    }
}
