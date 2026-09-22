//! Structural join of independent formula-wide QSL and TL owner results.

use quire_mltl::{
    contract_ir::{MappedOutcome, NonValueKind, ValidatedMappedResult as TlView},
    report::{ResultRelationKind as TlRelation, SettlementBasis as TlSettlement},
    request::AxisReference,
};
use quire_spec_language::{
    protocol_artifact::native_temporal::{
        result::{
            ActivationState as NativeActivation, NonValueKind as NativeNonValue,
            RelationKind as NativeRelation, Settlement as NativeSettlement, Truth as NativeTruth,
            ValidatedResult as NativeView,
        },
        EvidenceRef,
    },
    temporal::{Closure as NativeClosure, Completeness as NativeCompleteness},
};

use crate::bridge::{BridgeDigest, BridgeError, BridgeErrorCode, BridgeLimits};

use super::{
    decision::{
        JoinComparison, JoinDecisionInput, TemporalCause, TemporalCauseCode as Code,
        TemporalCauseDimension as Dim, TemporalJoinDecision, TemporalJoinKind,
        TemporalJoinRelation,
    },
    formula::cause,
    ValidatedTemporalProjection,
};

/// A join admitted only by [`super::read_join`] against both owner results.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedTemporalJoin {
    pub(crate) decision: TemporalJoinDecision,
}

impl ValidatedTemporalJoin {
    /// Exact canonical decision that was independently re-derived.
    #[must_use]
    pub const fn decision(&self) -> &TemporalJoinDecision {
        &self.decision
    }
}

/// Joins only exact constructor-private formula-result views; it evaluates neither model.
#[must_use]
pub fn join(
    projection: &ValidatedTemporalProjection,
    native: Option<&NativeView>,
    tl: Option<&TlView>,
    prior: Option<&ValidatedTemporalJoin>,
    limits: BridgeLimits,
) -> TemporalJoinDecision {
    match join_inner(projection, native, tl, prior, limits) {
        Ok(value) => value,
        Err(error) => TemporalJoinDecision::operation(
            correspondence(projection),
            native.map(|value| value.document().identity().to_owned()),
            tl.map(|value| value.identity().to_owned()),
            error,
        ),
    }
}

fn join_inner(
    projection: &ValidatedTemporalProjection,
    native: Option<&NativeView>,
    tl: Option<&TlView>,
    prior: Option<&ValidatedTemporalJoin>,
    limits: BridgeLimits,
) -> Result<TemporalJoinDecision, BridgeError> {
    let work = native
        .map_or(0, |value| value.document().bytes().len())
        .saturating_add(tl.map_or(0, |value| value.bytes().len()))
        .saturating_add(prior.map_or(0, |value| value.decision.bytes().len()));
    if work > limits.effective().visited_work {
        return Err(BridgeError::new(
            BridgeErrorCode::TemporalResultJoinResourceExhausted,
            "formula result inputs exceed visited-work ceiling",
            "results",
        ));
    }
    let (Some(native), Some(tl)) = (native, tl) else {
        let mut causes = Vec::new();
        if native.is_none() {
            causes.push(cause(
                Dim::NativeResult,
                Code::TemporalNativeResultUnavailable,
                "native",
                None,
            ));
        }
        if tl.is_none() {
            causes.push(cause(
                Dim::TlResult,
                Code::TemporalTlResultUnavailable,
                "tl",
                None,
            ));
        }
        return Ok(make_decision(
            projection,
            native,
            tl,
            prior,
            JoinDisposition {
                relation: None,
                kind: TemporalJoinKind::Unavailable,
                comparison: JoinComparison::NotCompared,
                value: None,
                causes,
            },
            limits,
        ));
    };

    let mut causes = Vec::new();
    validate_binding(projection, native, tl, &mut causes);
    compare_axis(
        native.decision_progress().reference(),
        tl.source().decision_scope_progress(),
        Dim::Progress,
        Code::TemporalProgressMismatch,
        &mut causes,
    );
    compare_axis(
        native.decision_closure().reference(),
        tl.source().decision_scope_closure(),
        Dim::Closure,
        Code::TemporalResultClosureDisagreement,
        &mut causes,
    );
    compare_axis(
        native.surrounding_progress().reference(),
        tl.source().surrounding_execution_progress(),
        Dim::Progress,
        Code::TemporalProgressMismatch,
        &mut causes,
    );
    compare_axis(
        native.surrounding_closure().reference(),
        tl.source().surrounding_execution_closure(),
        Dim::Closure,
        Code::TemporalResultClosureDisagreement,
        &mut causes,
    );
    compare_axis_states(native, tl, &mut causes);
    compare_completeness(native, tl, &mut causes);
    compare_settlement(native, tl, &mut causes);
    if native.activation() != NativeActivation::Active {
        causes.push(cause(
            Dim::Activation,
            Code::TemporalActivationInactive,
            native.instance(),
            None,
        ));
    }
    let relation = compare_lineage(native, tl, prior, correspondence(projection), &mut causes);
    let native_state = native_state(native);
    let tl_state = tl_state(tl);
    if matches!(native_state, NormalizedState::Value(_))
        && matches!(tl_state, NormalizedState::Value(_))
    {
        compare_support(projection, native, tl, &mut causes);
    }
    if native_state != tl_state {
        causes.push(cause(
            Dim::Truth,
            Code::TemporalTruthMismatch,
            native.document().identity(),
            None,
        ));
    }
    if !causes.is_empty() {
        return Ok(make_decision(
            projection,
            Some(native),
            Some(tl),
            prior,
            JoinDisposition {
                relation,
                kind: TemporalJoinKind::Conflict,
                comparison: JoinComparison::Mismatch,
                value: None,
                causes,
            },
            limits,
        ));
    }

    let (kind, comparison, value, state_cause) = match native_state {
        NormalizedState::Value(value) => (
            TemporalJoinKind::Agreement,
            JoinComparison::EqualFinal,
            Some(value),
            None,
        ),
        NormalizedState::Pending => (
            TemporalJoinKind::Agreement,
            JoinComparison::EqualPending,
            None,
            None,
        ),
        NormalizedState::Unavailable => (
            TemporalJoinKind::Unavailable,
            JoinComparison::NotCompared,
            None,
            Some(Code::TemporalNativeResultUnavailable),
        ),
        NormalizedState::Incomplete => (
            TemporalJoinKind::Incomplete,
            JoinComparison::NotCompared,
            None,
            Some(Code::TemporalNativeResultIncomplete),
        ),
        NormalizedState::Unsupported => (
            TemporalJoinKind::Unsupported,
            JoinComparison::NotCompared,
            None,
            Some(Code::TemporalNativeResultUnsupported),
        ),
        NormalizedState::Failed => (
            TemporalJoinKind::Failed,
            JoinComparison::NotCompared,
            None,
            Some(Code::TemporalNativeResultFailed),
        ),
        NormalizedState::Refused => (
            TemporalJoinKind::Refused,
            JoinComparison::NotCompared,
            None,
            Some(Code::TemporalNativeResultRefused),
        ),
        NormalizedState::Contradicted => (
            TemporalJoinKind::Conflict,
            JoinComparison::NotCompared,
            None,
            Some(Code::TemporalNativeResultContradicted),
        ),
    };
    let causes = state_cause
        .into_iter()
        .map(|code| cause(Dim::Truth, code, native.document().identity(), None))
        .collect();
    Ok(make_decision(
        projection,
        Some(native),
        Some(tl),
        prior,
        JoinDisposition {
            relation,
            kind,
            comparison,
            value,
            causes,
        },
        limits,
    ))
}

fn validate_binding(
    projection: &ValidatedTemporalProjection,
    native: &NativeView,
    tl: &TlView,
    causes: &mut Vec<TemporalCause>,
) {
    let correspondence = correspondence(projection).to_string();
    let tl_request = tl.source().request();
    if native.request_identity()
        != projection
            .decision()
            .native_request_identity()
            .unwrap_or_default()
        || native.subject_identity() != projection.decision().subject_identity().unwrap_or_default()
        || native.correspondence().identity() != correspondence
        || tl_request.identity() != projection.decision().request_identity().unwrap_or_default()
        || tl_request.subject_identity()
            != projection.decision().subject_identity().unwrap_or_default()
        || tl_request.correspondence_identity() != correspondence
        || tl_request.formula().identity()
            != projection.decision().formula_identity().unwrap_or_default()
        || tl_request.input_artifact().identity()
            != projection.decision().input_identity().unwrap_or_default()
    {
        causes.push(cause(
            Dim::ResultBinding,
            Code::TemporalResultBindingMismatch,
            native.document().identity(),
            None,
        ));
    }
}

fn compare_axis(
    native: &EvidenceRef,
    tl: &AxisReference,
    dimension: Dim,
    code: Code,
    causes: &mut Vec<TemporalCause>,
) {
    if native.identity() != tl.artifact().identity()
        || !digest_equal(native.digest(), tl.artifact().digest())
        || native.authority_identity() != tl.authority_identity()
        || native.authority_revision() != tl.authority_revision()
        || !digest_equal(native.authority_digest(), tl.authority_digest())
    {
        causes.push(cause(dimension, code, native.identity(), None));
    }
}

fn compare_axis_states(native: &NativeView, tl: &TlView, causes: &mut Vec<TemporalCause>) {
    let pairs = [
        (
            native.decision_closure().state(),
            tl.source().decision_scope_closure().state(),
        ),
        (
            native.surrounding_closure().state(),
            tl.source().surrounding_execution_closure().state(),
        ),
    ];
    if pairs.into_iter().any(|(native, tl)| {
        !matches!(
            (native, tl),
            (
                NativeClosure::Open,
                quire_observation::authority::OpenClosed::Open
            ) | (
                NativeClosure::Closed,
                quire_observation::authority::OpenClosed::Closed
            )
        )
    }) {
        causes.push(cause(
            Dim::Closure,
            Code::TemporalResultClosureDisagreement,
            native.document().identity(),
            None,
        ));
    }
}

fn compare_completeness(native: &NativeView, tl: &TlView, causes: &mut Vec<TemporalCause>) {
    let native = native.completeness();
    let tl = tl.source().completeness();
    let state_equal = matches!(
        (native.state(), tl.state()),
        (
            NativeCompleteness::Complete,
            quire_observation::authority::completeness::State::Complete
        ) | (
            NativeCompleteness::Incomplete,
            quire_observation::authority::completeness::State::Incomplete
        )
    );
    let facts_equal = native.facts().len() == tl.facts().len()
        && native
            .facts()
            .zip(tl.facts())
            .all(|(native, tl)| native.identity() == tl.member_identity());
    if native.reference().identity() != tl.artifact().identity()
        || !digest_equal(native.reference().digest(), tl.artifact().digest())
        || !state_equal
        || !facts_equal
    {
        causes.push(cause(
            Dim::Completeness,
            Code::TemporalCompletenessMismatch,
            native.reference().identity(),
            None,
        ));
    }
}

fn compare_settlement(native: &NativeView, tl: &TlView, causes: &mut Vec<TemporalCause>) {
    let equal = matches!(
        (native.settlement(), tl.source().settlement()),
        (
            Some(NativeSettlement::ClosedScope),
            TlSettlement::ClosedScope
        ) | (
            Some(NativeSettlement::DecisiveWitness),
            TlSettlement::DecisiveWitness
        ) | (
            Some(NativeSettlement::DecisiveCounterexample),
            TlSettlement::DecisiveCounterexample
        ) | (Some(NativeSettlement::Unsettled), TlSettlement::Unsettled)
            | (
                Some(NativeSettlement::Unavailable),
                TlSettlement::Unavailable
            )
    );
    if !equal {
        causes.push(cause(
            Dim::Settlement,
            Code::TemporalSettlementMismatch,
            native.document().identity(),
            None,
        ));
    }
}

fn compare_support(
    projection: &ValidatedTemporalProjection,
    native: &NativeView,
    tl: &TlView,
    causes: &mut Vec<TemporalCause>,
) {
    let admitted = projection
        .valuations()
        .iter()
        .map(super::TemporalValuationRow::observation_identity)
        .collect::<std::collections::BTreeSet<_>>();
    let position_to_observation = projection
        .native_request()
        .positions()
        .map(|position| (position.identity(), position.observation().identity()))
        .collect::<std::collections::BTreeMap<_, _>>();
    let native_support = native
        .support()
        .filter_map(|identity| position_to_observation.get(identity).copied())
        .collect::<std::collections::BTreeSet<_>>();
    let tl_support = tl
        .source()
        .decision_support()
        .iter()
        .map(String::as_str)
        .collect::<std::collections::BTreeSet<_>>();
    let expected_tl_support = tl
        .source()
        .completeness()
        .facts()
        .iter()
        .filter(|fact| {
            fact.status() == quire_observation::authority::completeness::FactStatus::Available
        })
        .filter_map(|fact| fact.observation_identity())
        .collect::<std::collections::BTreeSet<_>>();
    if native_support.len() != native.support().len()
        || tl_support != expected_tl_support
        || native_support
            .iter()
            .any(|identity| !admitted.contains(identity) || !tl_support.contains(identity))
    {
        causes.push(cause(
            Dim::Support,
            Code::TemporalSupportMismatch,
            native.document().identity(),
            None,
        ));
    }
}

fn compare_lineage(
    native: &NativeView,
    tl: &TlView,
    prior: Option<&ValidatedTemporalJoin>,
    current_correspondence: BridgeDigest,
    causes: &mut Vec<TemporalCause>,
) -> Option<TemporalJoinRelation> {
    let relation = match (native.relation(), tl.source().relation_kind()) {
        (NativeRelation::Original, TlRelation::Original) => TemporalJoinRelation::Original,
        (NativeRelation::Superseding, TlRelation::Superseding) => TemporalJoinRelation::Superseding,
        (NativeRelation::Invalidating, TlRelation::Invalidating) => {
            TemporalJoinRelation::Invalidating
        }
        _ => {
            causes.push(lineage_cause(native));
            return None;
        }
    };
    let valid = match relation {
        TemporalJoinRelation::Original => prior.is_none() && native.predecessor().is_none(),
        TemporalJoinRelation::Superseding | TemporalJoinRelation::Invalidating => prior
            .is_some_and(|prior| {
                let prior = prior.decision();
                prior.correspondence_identity() == current_correspondence
                    && native.predecessor().is_some_and(|(identity, digest)| {
                        Some(identity) == prior.native_result_identity()
                            && BridgeDigest::parse(digest).ok() == prior.native_result_digest()
                    })
                    && tl.source().direct_predecessor_identity()
                        == prior.tl_source_result_identity()
            }),
    };
    if !valid {
        causes.push(lineage_cause(native));
    }
    Some(relation)
}

fn lineage_cause(native: &NativeView) -> TemporalCause {
    cause(
        Dim::Supersession,
        Code::TemporalSupersessionInvalid,
        native.document().identity(),
        None,
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NormalizedState {
    Value(bool),
    Pending,
    Unavailable,
    Incomplete,
    Unsupported,
    Failed,
    Refused,
    Contradicted,
}

fn native_state(native: &NativeView) -> NormalizedState {
    if native.execution() == quire_spec_language::temporal::Execution::Failed {
        return NormalizedState::Failed;
    }
    match (native.truth(), native.non_value().map(|value| value.kind())) {
        (Some(NativeTruth::True), _) => NormalizedState::Value(true),
        (Some(NativeTruth::False), _) => NormalizedState::Value(false),
        (Some(NativeTruth::Pending), _) => NormalizedState::Pending,
        (_, Some(NativeNonValue::Missing)) => NormalizedState::Incomplete,
        (_, Some(NativeNonValue::Refused)) => NormalizedState::Refused,
        (_, Some(NativeNonValue::Inactive | NativeNonValue::Unactivated)) => {
            NormalizedState::Unavailable
        }
        (_, Some(NativeNonValue::ActivationUnknown)) | (None, None) => NormalizedState::Incomplete,
    }
}

fn tl_state(tl: &TlView) -> NormalizedState {
    match tl.outcome() {
        MappedOutcome::Value { value } => NormalizedState::Value(value),
        MappedOutcome::NonValue {
            reason: NonValueKind::Pending,
        } => NormalizedState::Pending,
        MappedOutcome::NonValue {
            reason: NonValueKind::Unavailable,
        } => NormalizedState::Unavailable,
        MappedOutcome::NonValue {
            reason: NonValueKind::Incomplete | NonValueKind::ResourceIncomplete,
        } => NormalizedState::Incomplete,
        MappedOutcome::NonValue {
            reason: NonValueKind::Unsupported,
        } => NormalizedState::Unsupported,
        MappedOutcome::NonValue {
            reason: NonValueKind::Failed,
        } => NormalizedState::Failed,
        MappedOutcome::NonValue {
            reason: NonValueKind::Refused,
        } => NormalizedState::Refused,
        MappedOutcome::NonValue {
            reason: NonValueKind::Contradicted,
        } => NormalizedState::Contradicted,
    }
}

struct JoinDisposition {
    relation: Option<TemporalJoinRelation>,
    kind: TemporalJoinKind,
    comparison: JoinComparison,
    value: Option<bool>,
    causes: Vec<TemporalCause>,
}

fn make_decision(
    projection: &ValidatedTemporalProjection,
    native: Option<&NativeView>,
    tl: Option<&TlView>,
    prior: Option<&ValidatedTemporalJoin>,
    disposition: JoinDisposition,
    limits: BridgeLimits,
) -> TemporalJoinDecision {
    TemporalJoinDecision::new(
        JoinDecisionInput {
            kind: disposition.kind,
            comparison: disposition.comparison,
            correspondence_identity: correspondence(projection),
            native_result_identity: native.map(|value| value.document().identity().to_owned()),
            native_result_digest: native.map(|value| BridgeDigest::raw(value.document().bytes())),
            tl_mapping_identity: tl.map(|value| value.identity().to_owned()),
            tl_source_result_identity: tl.map(|value| value.source_result_identity().to_owned()),
            tl_source_result_digest: tl.map(|value| BridgeDigest::raw(value.source().bytes())),
            relation: disposition.relation,
            predecessor_join_digest: prior.map(|value| BridgeDigest::raw(value.decision().bytes())),
            value: disposition.value,
            causes: disposition.causes,
        },
        limits,
    )
}

fn correspondence(projection: &ValidatedTemporalProjection) -> BridgeDigest {
    projection
        .decision()
        .correspondence_identity()
        .unwrap_or_else(|| BridgeDigest::raw(projection.decision().bytes()))
}

fn digest_equal(left: &str, right: &str) -> bool {
    left.strip_prefix("sha256:").unwrap_or(left) == right.strip_prefix("sha256:").unwrap_or(right)
}
