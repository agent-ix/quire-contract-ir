//! Complete rectangular FR-025 valuation admission.

use std::collections::BTreeSet;

use quire_spec_language::protocol_artifact::temporal_subject::ValidatedTemporalSubject;
use tl_syntax::PropositionId;

use crate::{
    bridge::BridgeLimits,
    predicate::{PredicateValuationDecision, PredicateValuationKind, ValidatedPredicateProjection},
};

use super::{
    admission::PositionValuations,
    decision::{TemporalCause, TemporalCauseCode as Code, TemporalCauseDimension as Dim},
    formula::cause,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ValuationRow {
    pub(crate) position: u64,
    pub(crate) observation_identity: String,
    pub(crate) true_propositions: Vec<PropositionId>,
}

pub(crate) fn admit(
    subject: &ValidatedTemporalSubject,
    projection: &ValidatedPredicateProjection,
    ledger: &quire_observation::authority::position::View,
    availability: &quire_observation::authority::availability::View,
    rows: &[PositionValuations<'_>],
    limits: BridgeLimits,
) -> Result<Vec<ValuationRow>, TemporalCause> {
    let limits = limits.effective();
    let required_handles: BTreeSet<_> = subject
        .predicate_leaves()
        .unwrap_or_default()
        .iter()
        .map(|handle| (handle.declaration, handle.index))
        .collect();
    let required: BTreeSet<_> = projection
        .correspondences()
        .iter()
        .filter_map(|item| {
            let definition = projection.definition(item.predicate_ref())?;
            required_handles
                .contains(&(definition.leaf_declaration(), definition.leaf_index()))
                .then_some(item.proposition_id())
        })
        .collect();
    if required.len() != required_handles.len() {
        return Err(cause(
            Dim::PredicateProjection,
            Code::TemporalPredicateProjectionIncomplete,
            subject.document().identity(),
            None,
        ));
    }
    let width = required.len();
    let cells = rows
        .len()
        .checked_mul(width)
        .ok_or_else(|| resource("valuations"))?;
    if rows.len() > limits.positions || cells > limits.valuations || cells > limits.visited_work {
        return Err(resource("valuations"));
    }
    let ledger_rows: Vec<_> = ledger.payload().positions().collect();
    if ledger_rows.len() != rows.len() || rows.len() > limits.positions {
        return Err(cause(
            Dim::Observation,
            Code::TemporalObservationMismatch,
            ledger.identity().as_str(),
            None,
        ));
    }
    let projection_ref = projection.decision().projection_ref().ok_or_else(|| {
        cause(
            Dim::PredicateProjection,
            Code::TemporalPredicateProjectionUnavailable,
            "projection-ref",
            None,
        )
    })?;
    let mut result = Vec::new();
    result
        .try_reserve_exact(rows.len())
        .map_err(|_| resource("valuations.rows"))?;
    for (ordinal, (row, (position_text, observation_identity))) in
        rows.iter().zip(ledger_rows).enumerate()
    {
        let position = position_text.parse::<u64>().map_err(|_| {
            cause(
                Dim::Observation,
                Code::TemporalObservationMismatch,
                position_text,
                None,
            )
        })?;
        if position != row.position
            || position != u64::try_from(ordinal).map_err(|_| resource("position"))?
            || observation_identity != row.observation_identity
            || row.valuations.len() != width
        {
            return Err(cause(
                Dim::Observation,
                Code::TemporalObservationMismatch,
                row.observation_identity,
                None,
            ));
        }
        let mut seen = BTreeSet::new();
        let mut true_propositions = Vec::new();
        for valuation in row.valuations {
            validate_cell(
                valuation,
                projection_ref,
                availability.identity().as_str(),
                row.observation_identity,
            )?;
            if !required.contains(&valuation.proposition_id())
                || !seen.insert(valuation.proposition_id())
            {
                return Err(cause(
                    Dim::PredicateProjection,
                    Code::TemporalPredicateProjectionMismatch,
                    valuation.proposition_id().to_string(),
                    None,
                ));
            }
            if valuation.value() == Some(true) {
                true_propositions.push(PropositionId(valuation.proposition_id()));
            }
        }
        if seen != required {
            return Err(cause(
                Dim::PredicateProjection,
                Code::TemporalPredicateProjectionIncomplete,
                row.observation_identity,
                None,
            ));
        }
        true_propositions.sort();
        result.push(ValuationRow {
            position,
            observation_identity: row.observation_identity.to_owned(),
            true_propositions,
        });
    }
    Ok(result)
}

pub(crate) fn validate_cell(
    cell: &PredicateValuationDecision,
    projection_ref: crate::bridge::BridgeDigest,
    availability_identity: &str,
    observation_identity: &str,
) -> Result<(), TemporalCause> {
    if cell.operation().is_some() {
        return Err(resource("valuation.operation"));
    }
    let code = match cell.kind() {
        PredicateValuationKind::Valued => None,
        PredicateValuationKind::Incomplete => Some(Code::TemporalPredicateProjectionIncomplete),
        PredicateValuationKind::Unavailable => Some(Code::TemporalPredicateProjectionUnavailable),
        PredicateValuationKind::Unsupported => Some(Code::TemporalPredicateProjectionUnsupported),
        PredicateValuationKind::Failed => Some(Code::TemporalPredicateProjectionFailed),
        PredicateValuationKind::Refused => Some(Code::TemporalPredicateProjectionRefused),
        PredicateValuationKind::Conflict => Some(Code::TemporalPredicateProjectionConflict),
    };
    if let Some(code) = code {
        return Err(cause(
            Dim::PredicateProjection,
            code,
            cell.predicate_ref().to_string(),
            None,
        ));
    }
    if cell.projection_ref() != projection_ref
        || cell.availability_identity() != availability_identity
        || cell.value().is_none()
        || cell
            .decision_premises()
            .binary_search_by(|value| value.as_str().cmp(observation_identity))
            .is_err()
    {
        return Err(cause(
            Dim::PredicateProjection,
            Code::TemporalPredicateProjectionMismatch,
            cell.predicate_ref().to_string(),
            None,
        ));
    }
    Ok(())
}

fn resource(path: &str) -> TemporalCause {
    cause(
        Dim::PredicateProjection,
        Code::TemporalProjectionResourceExhausted,
        path,
        None,
    )
}
