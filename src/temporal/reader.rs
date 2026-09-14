//! Strict projection decision readback by complete independent re-derivation.

use quire_spec_language::protocol_artifact::native_temporal::result::ValidatedResult as NativeTemporalResult;
use quire_spec_language::protocol_artifact::temporal_subject::ValidatedTemporalSubject;
use tl_mltl::mapping::contract_ir::ValidatedMappedResult as TlMappedResult;

use crate::{
    bridge::{canonical, BridgeError, BridgeErrorCode, BridgeLimits},
    predicate::ValidatedPredicateProjection,
};

use super::{
    admission::{ObservationViews, TargetSelection},
    decision::{ProjectionWire, TemporalDecision, TemporalJoinDecision},
    join, project, ValidatedTemporalJoin, ValidatedTemporalProjection,
};

/// Complete independent authority needed to read one projection.
#[derive(Clone, Copy)]
pub struct ExpectedTemporalProjection<'a> {
    pub subject: &'a ValidatedTemporalSubject,
    pub predicates: &'a ValidatedPredicateProjection,
    pub observations: ObservationViews<'a>,
    pub target: &'a TargetSelection,
}

/// Complete independent authority needed to read one result join.
#[derive(Clone, Copy)]
pub struct ExpectedTemporalJoin<'a> {
    pub projection: &'a ValidatedTemporalProjection,
    pub native: Option<&'a NativeTemporalResult>,
    pub tl: Option<&'a TlMappedResult>,
    pub prior: Option<&'a ValidatedTemporalJoin>,
}

/// Reads canonical join bytes and re-derives the decision from both owner views.
pub fn read_join(
    bytes: &[u8],
    expected: ExpectedTemporalJoin<'_>,
    limits: BridgeLimits,
) -> Result<ValidatedTemporalJoin, TemporalDecision> {
    canonical::decode::<TemporalJoinDecision>(bytes, limits).map_err(|error| {
        TemporalDecision::Operation(BridgeError::new(
            BridgeErrorCode::InvalidNativeTemporalBridge,
            error.message(),
            error.path(),
        ))
    })?;
    let decision = join(
        expected.projection,
        expected.native,
        expected.tl,
        expected.prior,
        limits,
    );
    if decision.operation_error().is_some() || decision.bytes() != bytes {
        return Err(TemporalDecision::Operation(BridgeError::new(
            BridgeErrorCode::InvalidNativeTemporalBridge,
            "join differs from complete owner re-derivation",
            "join",
        )));
    }
    Ok(ValidatedTemporalJoin { decision })
}

/// Reads canonical bridge bytes and re-derives every embedded owner artifact.
pub fn read_projection<'a>(
    bytes: &[u8],
    expected: ExpectedTemporalProjection<'a>,
    limits: BridgeLimits,
) -> Result<ValidatedTemporalProjection, TemporalDecision> {
    canonical::decode::<ProjectionWire>(bytes, limits).map_err(|error| {
        TemporalDecision::Operation(BridgeError::new(
            BridgeErrorCode::InvalidNativeTemporalBridge,
            error.message(),
            error.path(),
        ))
    })?;
    let projection = project(
        expected.subject,
        expected.predicates,
        expected.observations,
        expected.target.clone(),
        limits,
    )?;
    if projection.decision().bytes() != bytes {
        return Err(TemporalDecision::Operation(BridgeError::new(
            BridgeErrorCode::InvalidNativeTemporalBridge,
            "projection differs from complete owner re-derivation",
            "projection",
        )));
    }
    Ok(projection.validated().clone())
}
