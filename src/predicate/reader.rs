//! Strict re-deriving readers for bridge-owned decisions.

use quire_observation::authority::availability::View as ValidatedAvailability;
use quire_protocol::result::contract_ir::MappedResultView;
use quire_spec_language::protocol_artifact::checked_predicate::ValidatedCheckedPredicate;

use crate::bridge::{canonical, BridgeError, BridgeErrorCode, BridgeLimits};

use super::decision::{ProjectionDecisionWire, ValuationDecisionWire};
use super::{
    project, value, PredicateDecision, PredicateProjectionKind, PredicateRef,
    PredicateValuationDecision, TargetSelection, ValidatedPredicateProjection,
};

/// Independent source and artifact selections used to re-derive a projection.
#[derive(Clone, Copy, Debug)]
pub struct ExpectedProjection<'a> {
    predicates: &'a [ValidatedCheckedPredicate],
    target: &'a TargetSelection,
    signal_catalog_bytes: &'a [u8],
    proposition_map_bytes: &'a [u8],
}

impl<'a> ExpectedProjection<'a> {
    /// Constructs an exact projection-reader expectation.
    #[must_use]
    pub const fn new(
        predicates: &'a [ValidatedCheckedPredicate],
        target: &'a TargetSelection,
        signal_catalog_bytes: &'a [u8],
        proposition_map_bytes: &'a [u8],
    ) -> Self {
        Self {
            predicates,
            target,
            signal_catalog_bytes,
            proposition_map_bytes,
        }
    }
}

/// Strict-reads and independently re-derives one admitted projection.
pub fn read_projection(
    bytes: &[u8],
    expected: ExpectedProjection<'_>,
    limits: BridgeLimits,
) -> Result<ValidatedPredicateProjection, PredicateDecision> {
    let wire: ProjectionDecisionWire =
        canonical::decode(bytes, limits).map_err(PredicateDecision::Operation)?;
    if !wire.validate() || wire.kind() != PredicateProjectionKind::Admitted {
        return Err(PredicateDecision::Operation(invalid(
            "projection decision is not one valid admitted shape",
        )));
    }
    let candidate = project(expected.predicates, expected.target.clone(), limits)?;
    if candidate.decision().bytes() != bytes
        || candidate.signal_catalog_bytes() != expected.signal_catalog_bytes
        || candidate.proposition_map_bytes() != expected.proposition_map_bytes
        || candidate.decision().wire() != wire
    {
        return Err(PredicateDecision::Operation(invalid(
            "projection differs from independent owner inputs or artifacts",
        )));
    }
    Ok(candidate.validated().clone())
}

/// Independent owner views used to re-derive one valuation decision.
#[derive(Clone, Copy, Debug)]
pub struct ExpectedValuation<'a, 'result> {
    projection: &'a ValidatedPredicateProjection,
    predicate_ref: PredicateRef,
    availability: &'a ValidatedAvailability,
    mapped: Option<&'a MappedResultView<'result>>,
}

impl<'a, 'result> ExpectedValuation<'a, 'result> {
    /// Constructs an exact valuation-reader expectation.
    #[must_use]
    pub const fn new(
        projection: &'a ValidatedPredicateProjection,
        predicate_ref: PredicateRef,
        availability: &'a ValidatedAvailability,
        mapped: Option<&'a MappedResultView<'result>>,
    ) -> Self {
        Self {
            projection,
            predicate_ref,
            availability,
            mapped,
        }
    }
}

/// Strict-reads and independently re-derives one valuation decision.
pub fn read_valuation(
    bytes: &[u8],
    expected: ExpectedValuation<'_, '_>,
    limits: BridgeLimits,
) -> Result<PredicateValuationDecision, BridgeError> {
    let wire: ValuationDecisionWire = canonical::decode(bytes, limits).map_err(valuation_error)?;
    if !wire.validate() {
        return Err(valuation_invalid(
            "valuation decision does not have one valid closed shape",
        ));
    }
    let candidate = value(
        expected.projection,
        expected.predicate_ref,
        expected.availability,
        expected.mapped,
        limits,
    );
    if let Some(error) = candidate.operation() {
        return Err(error.clone());
    }
    if candidate.bytes() != bytes || candidate.wire() != wire {
        return Err(valuation_invalid(
            "valuation differs from independent owner inputs",
        ));
    }
    Ok(candidate)
}

fn invalid(message: &'static str) -> BridgeError {
    BridgeError::new(
        BridgeErrorCode::InvalidNativePredicateProjection,
        message,
        "projection_decision",
    )
}

fn valuation_invalid(message: &'static str) -> BridgeError {
    BridgeError::new(
        BridgeErrorCode::InvalidNativePredicateProjection,
        message,
        "valuation_decision",
    )
}

fn valuation_error(error: BridgeError) -> BridgeError {
    if error.code() == BridgeErrorCode::PredicateProjectionResourceExhausted {
        BridgeError::new(
            BridgeErrorCode::PredicateValuationResourceExhausted,
            error.message(),
            error.path(),
        )
    } else {
        error
    }
}
