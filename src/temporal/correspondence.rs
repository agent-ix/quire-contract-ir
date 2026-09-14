//! Immutable correspondence identity and constructor-private projection.

use quire_spec_language::protocol_artifact::temporal_subject::ValidatedTemporalSubject;
use serde::Serialize;
use serde_json::json;

use crate::{
    bridge::{canonical, BridgeDigest, BridgeError, BridgeErrorCode, BridgeLimits},
    predicate::ValidatedPredicateProjection,
};

use super::{
    admission::{ObservationViews, TargetSelection},
    decision::{TemporalProjectionDecision, TemporalProjectionKind},
    formula::{BuiltFormula, FormulaOccurrence},
    request::{BuiltRequest, ValidatedTemporalInput},
    valuation::ValuationRow,
    CORRESPONDENCE_PROFILE,
};

/// Complete admitted projection.
#[derive(Clone, Debug)]
pub struct TemporalProjection {
    validated: ValidatedTemporalProjection,
}

impl TemporalProjection {
    #[must_use]
    pub const fn validated(&self) -> &ValidatedTemporalProjection {
        &self.validated
    }
    #[must_use]
    pub const fn decision(&self) -> &TemporalProjectionDecision {
        &self.validated.decision
    }
}

/// Projection constructible only through owner-backed construction or exact readback.
#[derive(Clone, Debug)]
pub struct ValidatedTemporalProjection {
    pub(crate) decision: TemporalProjectionDecision,
    pub(crate) target: TargetSelection,
    pub(crate) subject_digest: BridgeDigest,
    pub(crate) formula: tl_syntax::FormulaDocument,
    pub(crate) formula_bytes: Vec<u8>,
    pub(crate) native_request:
        quire_spec_language::protocol_artifact::native_temporal::request::ValidatedRequest,
    pub(crate) native_request_bytes: Vec<u8>,
    pub(crate) input: ValidatedTemporalInput,
    pub(crate) history_requirement: Option<tl_mltl::past::requirement::ValidatedHistoryRequirement>,
    pub(crate) request: tl_mltl::wire::request::ValidatedTemporalRequest,
    pub(crate) request_bytes: Vec<u8>,
    pub(crate) occurrences: Vec<FormulaOccurrence>,
    pub(crate) valuations: Vec<TemporalValuationRow>,
}

impl PartialEq for ValidatedTemporalProjection {
    fn eq(&self, other: &Self) -> bool {
        self.decision == other.decision
            && self.target == other.target
            && self.subject_digest == other.subject_digest
            && self.formula == other.formula
            && self.formula_bytes == other.formula_bytes
            && self.native_request_bytes == other.native_request_bytes
            && self.input == other.input
            && self.history_requirement == other.history_requirement
            && self.request == other.request
            && self.request_bytes == other.request_bytes
            && self.occurrences == other.occurrences
            && self.valuations == other.valuations
    }
}

impl Eq for ValidatedTemporalProjection {}

/// Public immutable valuation manifest; false propositions are proven by the
/// complete population even though the TL trace/history stores only true IDs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemporalValuationRow {
    position: u64,
    observation_identity: String,
    true_propositions: Vec<u32>,
}

impl TemporalValuationRow {
    #[must_use]
    pub const fn position(&self) -> u64 {
        self.position
    }
    #[must_use]
    pub fn observation_identity(&self) -> &str {
        &self.observation_identity
    }
    #[must_use]
    pub fn true_propositions(&self) -> &[u32] {
        &self.true_propositions
    }
}

impl ValidatedTemporalProjection {
    #[must_use]
    pub const fn decision(&self) -> &TemporalProjectionDecision {
        &self.decision
    }
    #[must_use]
    pub const fn target(&self) -> &TargetSelection {
        &self.target
    }
    #[must_use]
    pub const fn subject_digest(&self) -> BridgeDigest {
        self.subject_digest
    }
    #[must_use]
    pub const fn formula(&self) -> &tl_syntax::FormulaDocument {
        &self.formula
    }
    #[must_use]
    pub fn formula_bytes(&self) -> &[u8] {
        &self.formula_bytes
    }
    #[must_use]
    pub fn input_bytes(&self) -> &[u8] {
        self.input.bytes()
    }
    #[must_use]
    pub const fn native_request(
        &self,
    ) -> &quire_spec_language::protocol_artifact::native_temporal::request::ValidatedRequest {
        &self.native_request
    }
    #[must_use]
    pub fn native_request_bytes(&self) -> &[u8] {
        &self.native_request_bytes
    }
    #[must_use]
    pub const fn request(&self) -> &tl_mltl::wire::request::ValidatedTemporalRequest {
        &self.request
    }
    #[must_use]
    pub const fn tl_request(&self) -> &tl_mltl::wire::request::ValidatedTemporalRequest {
        &self.request
    }
    #[must_use]
    pub fn request_bytes(&self) -> &[u8] {
        &self.request_bytes
    }
    #[must_use]
    pub fn history_requirement(
        &self,
    ) -> Option<&tl_mltl::past::requirement::ValidatedHistoryRequirement> {
        self.history_requirement.as_ref()
    }
    #[must_use]
    pub fn valuations(&self) -> &[TemporalValuationRow] {
        &self.valuations
    }
    #[must_use]
    pub fn occurrence_count(&self) -> usize {
        self.occurrences.len()
    }
}

#[derive(Serialize)]
struct ValuationCellRef<'a> {
    position: u64,
    observation_identity: &'a str,
    predicate_ref: crate::predicate::PredicateRef,
    decision_digest: BridgeDigest,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum ActivationGuardRef {
    Absent,
    Present { decision_digest: BridgeDigest },
}

pub(crate) fn identity(
    subject: &ValidatedTemporalSubject,
    predicates: &ValidatedPredicateProjection,
    observations: ObservationViews<'_>,
    formula: &BuiltFormula,
    target: &TargetSelection,
    limits: BridgeLimits,
) -> Result<BridgeDigest, BridgeError> {
    let mut cells = observations
        .positions
        .iter()
        .flat_map(|row| {
            row.valuations.iter().map(|valuation| ValuationCellRef {
                position: row.position,
                observation_identity: row.observation_identity,
                predicate_ref: valuation.predicate_ref(),
                decision_digest: BridgeDigest::raw(valuation.bytes()),
            })
        })
        .collect::<Vec<_>>();
    cells.sort_by_key(|cell| (cell.position, cell.predicate_ref));
    let activation_guard =
        observations
            .activation_guard
            .map_or(ActivationGuardRef::Absent, |guard| {
                ActivationGuardRef::Present {
                    decision_digest: BridgeDigest::raw(guard.bytes()),
                }
            });
    let tuple = json!({
        "bridge_profile": super::PROFILE,
        "formula_identity": formula.identity,
        "formula_schema": formula.document.schema_version().as_str(),
        "lane": formula.lane.label(),
        "target": target,
        "observation": {
            "availability": BridgeDigest::raw(observations.availability.bytes()),
            "capture": BridgeDigest::raw(observations.capture.bytes()),
            "clock": BridgeDigest::raw(observations.clock.bytes()),
            "completeness": BridgeDigest::raw(observations.completeness.bytes()),
            "trigger_scope_closure": BridgeDigest::raw(observations.trigger_scope_closure.bytes()),
            "decision_closure": BridgeDigest::raw(observations.decision_scope_closure.bytes()),
            "decision_progress": BridgeDigest::raw(observations.decision_scope_progress.bytes()),
            "position_ledger": BridgeDigest::raw(observations.position_ledger.bytes()),
            "surrounding_closure": BridgeDigest::raw(observations.surrounding_execution_closure.bytes()),
            "surrounding_progress": BridgeDigest::raw(observations.surrounding_execution_progress.bytes()),
        },
        "predicate_projection": predicates.decision().projection_ref(),
        "subject_digest": BridgeDigest::raw(subject.document().bytes()),
        "subject_identity": subject.document().identity(),
        "activation_guard": activation_guard,
        "valuation_cells": cells,
    });
    let bytes = canonical::encode(&tuple, limits).map_err(|error| {
        BridgeError::new(
            BridgeErrorCode::TemporalProjectionResourceExhausted,
            error.message(),
            error.path(),
        )
    })?;
    Ok(BridgeDigest::domain(CORRESPONDENCE_PROFILE, &bytes))
}

pub(crate) struct ProjectionParts<'a> {
    pub(crate) subject: &'a ValidatedTemporalSubject,
    pub(crate) predicates: &'a ValidatedPredicateProjection,
    pub(crate) formula: BuiltFormula,
    pub(crate) rows: Vec<ValuationRow>,
    pub(crate) request: BuiltRequest,
    pub(crate) target: TargetSelection,
    pub(crate) correspondence_identity: BridgeDigest,
    pub(crate) limits: BridgeLimits,
}

pub(crate) fn finish(parts: ProjectionParts<'_>) -> Result<TemporalProjection, BridgeError> {
    let ProjectionParts {
        subject,
        predicates,
        formula,
        rows,
        request,
        target,
        correspondence_identity,
        limits,
    } = parts;
    let input_identity = request.input.identity().to_owned();
    let request_identity = request.request.identity().to_owned();
    let native_request_identity = request.native_request.document().identity().to_owned();
    let mut decision = TemporalProjectionDecision {
        kind: TemporalProjectionKind::Admitted,
        native_contract: Some(target.native().clone()),
        predicate_projection_ref: predicates.decision().projection_ref(),
        subject_identity: Some(subject.document().identity().to_owned()),
        semantic_profile: Some(formula.document.semantic_profile().as_str().to_owned()),
        formula_identity: Some(formula.identity.clone()),
        input_identity: Some(input_identity),
        native_request_identity: Some(native_request_identity),
        request_identity: Some(request_identity),
        correspondence_identity: Some(correspondence_identity),
        causes: Vec::new(),
        bytes: Vec::new(),
    };
    decision.encode(limits).map_err(|error| {
        BridgeError::new(
            BridgeErrorCode::TemporalProjectionResourceExhausted,
            error.message(),
            error.path(),
        )
    })?;
    let valuations = rows
        .into_iter()
        .map(|row| TemporalValuationRow {
            position: row.position,
            observation_identity: row.observation_identity,
            true_propositions: row
                .true_propositions
                .into_iter()
                .map(|value| value.0)
                .collect(),
        })
        .collect();
    Ok(TemporalProjection {
        validated: ValidatedTemporalProjection {
            decision,
            target,
            subject_digest: BridgeDigest::raw(subject.document().bytes()),
            formula: formula.document,
            formula_bytes: formula.bytes,
            native_request: request.native_request,
            native_request_bytes: request.native_request_bytes,
            input: request.input,
            history_requirement: request.history_requirement,
            request: request.request,
            request_bytes: request.request_bytes,
            occurrences: formula.occurrences,
            valuations,
        },
    })
}
