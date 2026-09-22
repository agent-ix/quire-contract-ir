use crate::support::{result_fixture, temporal_fixture};
use ix_trace_rs::trace;
use quire_contract_ir::{
    bridge::{BridgeDigest, BridgeLimits, ContractSelection},
    predicate,
    temporal::{
        self, ExpectedTemporalJoin, ExpectedTemporalProjection, ObservationViews,
        PositionValuations, TargetContract, TemporalCauseCode as Code,
        TemporalCauseDimension as Dim, TemporalDecision, TemporalProjectionKind,
    },
};
use quire_observation::authority::OpenClosed;
use quire_protocol::result::{
    contract_ir::{self, MappingSelection},
    Limits as ResultLimits, Truth,
};
use quire_spec_language::protocol_artifact::native_temporal::result::{
    self as native_result, Relation as NativeRelation,
};
use quire_spec_language::{
    checking::composed::{proofs, TypeDisposition, TypeLimits},
    linking::composed::definition_source::RegisteredDefinition,
    protocol_artifact::{self as qsl_artifact, native as qsl_native, v2, wire as qsl_wire},
};
use tl_mltl::{mapping, wire, wire::OwnerLimits};

use crate::support::native_protocol::{
    Inputs as NativeInputs, TemporalDefinitionExpectation, Unit,
};

#[trace(
    "TC-039",
    "FR-026-AC-1",
    "FR-026-AC-2",
    "FR-026-AC-3",
    "FR-026-AC-4",
    "FR-026-AC-5",
    "FR-026-AC-6",
    "FR-026-AC-8"
)]
#[test]
fn tc_039_future_projection_uses_exact_owner_views_and_rereads() {
    let fixture = result_fixture::fixture();
    let predicates = predicate::project(
        fixture.predicates(),
        predicate::TargetSelection::current(),
        BridgeLimits::default(),
    )
    .expect("predicate projection");
    let decision =
        temporal_fixture::event_position(&fixture, "decision-temporal", OpenClosed::Closed);
    let surrounding =
        temporal_fixture::event_position(&fixture, "surrounding-temporal", OpenClosed::Closed);
    let result = result_fixture::validated_result_for_temporal(
        &fixture,
        &decision.progress,
        &decision.closure,
        &surrounding.progress,
        &surrounding.closure,
        &decision.completeness,
        &decision.observation_identity,
        Truth::Violated,
    );
    let mapped = contract_ir::map(
        &result,
        MappingSelection::current(),
        ResultLimits::owner_max(),
    )
    .expect("native result map");
    let availability = decision.availability(result.result_id());
    let predicate_ref = predicates.decision().correspondences()[0].predicate_ref();
    let valuation = predicate::value(
        predicates.validated(),
        predicate_ref,
        &availability,
        Some(&mapped),
        BridgeLimits::default(),
    );
    assert_eq!(valuation.value(), Some(false));
    let cells = [valuation];
    let position_values = [PositionValuations {
        position: 0,
        observation_identity: &decision.observation_identity,
        valuations: &cells,
    }];
    let observations = ObservationViews {
        position_ledger: &decision.position,
        clock: &decision.clock,
        capture: &decision.capture,
        trigger_scope_closure: &decision.closure,
        decision_scope_progress: &decision.progress,
        decision_scope_closure: &decision.closure,
        surrounding_execution_progress: &surrounding.progress,
        surrounding_execution_closure: &surrounding.closure,
        completeness: &decision.completeness,
        availability: &availability,
        activation_guard: None,
        positions: &position_values,
        anchor: 0,
    };
    let target = temporal::TargetSelection::current();
    let projection = temporal::project(
        fixture.temporal(),
        predicates.validated(),
        observations,
        target.clone(),
        BridgeLimits::default(),
    )
    .expect("temporal projection");
    assert_eq!(
        projection.decision().kind(),
        TemporalProjectionKind::Admitted
    );
    assert_eq!(projection.validated().occurrence_count(), 2);
    assert!(projection.validated().valuations()[0]
        .true_propositions()
        .is_empty());

    assert_projection_refusal(
        fixture.temporal(),
        predicates.validated(),
        ObservationViews {
            positions: &[],
            ..observations
        },
        target.clone(),
        TemporalProjectionKind::Refused,
        Code::TemporalObservationMismatch,
    );
    let duplicate_cells = [cells[0].clone(), cells[0].clone()];
    let duplicate_positions = [PositionValuations {
        position: 0,
        observation_identity: &decision.observation_identity,
        valuations: &duplicate_cells,
    }];
    assert_projection_refusal(
        fixture.temporal(),
        predicates.validated(),
        ObservationViews {
            positions: &duplicate_positions,
            ..observations
        },
        target.clone(),
        TemporalProjectionKind::Refused,
        Code::TemporalObservationMismatch,
    );
    let foreign_positions = [PositionValuations {
        position: 0,
        observation_identity: "observation:foreign",
        valuations: &cells,
    }];
    assert_projection_refusal(
        fixture.temporal(),
        predicates.validated(),
        ObservationViews {
            positions: &foreign_positions,
            ..observations
        },
        target.clone(),
        TemporalProjectionKind::Refused,
        Code::TemporalObservationMismatch,
    );
    let unavailable_cell = predicate::value(
        predicates.validated(),
        predicate_ref,
        &availability,
        None,
        BridgeLimits::default(),
    );
    let unavailable_cells = [unavailable_cell];
    let unavailable_positions = [PositionValuations {
        position: 0,
        observation_identity: &decision.observation_identity,
        valuations: &unavailable_cells,
    }];
    assert_projection_refusal(
        fixture.temporal(),
        predicates.validated(),
        ObservationViews {
            positions: &unavailable_positions,
            ..observations
        },
        target.clone(),
        TemporalProjectionKind::Unavailable,
        Code::TemporalPredicateProjectionUnavailable,
    );
    let replay_decision =
        temporal_fixture::event_position(&fixture, "decision-replay", OpenClosed::Closed);
    let replay_surrounding =
        temporal_fixture::event_position(&fixture, "surrounding-replay", OpenClosed::Closed);
    let replay_result = result_fixture::validated_result_for_temporal(
        &fixture,
        &replay_decision.progress,
        &replay_decision.closure,
        &replay_surrounding.progress,
        &replay_surrounding.closure,
        &replay_decision.completeness,
        &replay_decision.observation_identity,
        Truth::Violated,
    );
    let replay_map = contract_ir::map(
        &replay_result,
        MappingSelection::current(),
        ResultLimits::owner_max(),
    )
    .expect("replayed result map");
    let replay_availability = replay_decision.availability(replay_result.result_id());
    let replay_cell = predicate::value(
        predicates.validated(),
        predicate_ref,
        &replay_availability,
        Some(&replay_map),
        BridgeLimits::default(),
    );
    assert_eq!(
        replay_cell.kind(),
        predicate::PredicateValuationKind::Valued
    );
    let replay_cells = [replay_cell];
    let replay_positions = [PositionValuations {
        position: 0,
        observation_identity: &decision.observation_identity,
        valuations: &replay_cells,
    }];
    assert_projection_refusal(
        fixture.temporal(),
        predicates.validated(),
        ObservationViews {
            positions: &replay_positions,
            ..observations
        },
        target.clone(),
        TemporalProjectionKind::Refused,
        Code::TemporalPredicateProjectionMismatch,
    );
    let exhausted_projection = temporal::project(
        fixture.temporal(),
        predicates.validated(),
        observations,
        target.clone(),
        BridgeLimits {
            valuations: 0,
            ..BridgeLimits::default()
        },
    )
    .expect_err("valuation ceiling refuses without a partial projection");
    let TemporalDecision::Operation(error) = exhausted_projection else {
        panic!("valuation ceiling must be an operation error");
    };
    assert_eq!(
        error.code(),
        quire_contract_ir::bridge::BridgeErrorCode::TemporalProjectionResourceExhausted
    );

    let reread = temporal::read_projection(
        projection.decision().bytes(),
        ExpectedTemporalProjection {
            subject: fixture.temporal(),
            predicates: predicates.validated(),
            observations,
            target: &target,
        },
        BridgeLimits::default(),
    )
    .expect("temporal projection strict reread");
    assert_eq!(reread, projection.validated().clone());
    let mut trailing_projection = projection.decision().bytes().to_vec();
    trailing_projection.push(b'\n');
    let malformed = temporal::read_projection(
        &trailing_projection,
        ExpectedTemporalProjection {
            subject: fixture.temporal(),
            predicates: predicates.validated(),
            observations,
            target: &target,
        },
        BridgeLimits::default(),
    )
    .expect_err("projection reader rejects trailing data");
    let TemporalDecision::Operation(error) = malformed else {
        panic!("malformed projection must be an operation error");
    };
    assert_eq!(
        error.code(),
        quire_contract_ir::bridge::BridgeErrorCode::InvalidNativeTemporalBridge
    );

    let native_document = native_result::evaluate(
        projection.validated().native_request(),
        NativeRelation::Original,
        projection.validated().native_request().limits(),
    )
    .into_result()
    .expect("native temporal evaluation result");
    let native = native_result::read(
        native_document.bytes(),
        projection.validated().native_request(),
        NativeRelation::Original,
        projection.validated().native_request().limits(),
    )
    .into_result()
    .expect("native temporal result reader");

    let result_document = wire::report::evaluate(
        projection.validated().request(),
        wire::report::ResultRelationInput::Original,
        OwnerLimits::owner_max(),
    )
    .expect("TL evaluation result");
    let tl_result = wire::report::read(
        result_document.bytes(),
        projection.validated().request(),
        wire::report::ResultRelationInput::Original,
        OwnerLimits::owner_max(),
    )
    .expect("TL result reader");
    let tl_selection = mapping::contract_ir::MappingSelection::for_result(&tl_result);
    let tl_document =
        mapping::contract_ir::map(&tl_result, &tl_selection, OwnerLimits::owner_max())
            .expect("TL result mapping");
    let tl_mapped = mapping::contract_ir::read(
        tl_document.bytes(),
        &tl_result,
        &tl_selection,
        OwnerLimits::owner_max(),
    )
    .expect("TL mapping reader");
    let joined = temporal::join(
        projection.validated(),
        Some(&native),
        Some(&tl_mapped),
        None,
        BridgeLimits::default(),
    );
    assert_eq!(
        joined.kind(),
        temporal::TemporalJoinKind::Agreement,
        "{:?}",
        joined.causes()
    );
    assert_eq!(joined.comparison(), temporal::JoinComparison::EqualFinal);
    assert_eq!(joined.value(), Some(false));
    assert!(joined.operation_error().is_none());
    let exhausted = temporal::join(
        projection.validated(),
        Some(&native),
        Some(&tl_mapped),
        None,
        BridgeLimits {
            visited_work: 0,
            ..BridgeLimits::default()
        },
    );
    assert!(exhausted.operation_error().is_some());
    assert!(exhausted.bytes().is_empty());
    let validated_join = temporal::read_join(
        joined.bytes(),
        ExpectedTemporalJoin {
            projection: projection.validated(),
            native: Some(&native),
            tl: Some(&tl_mapped),
            prior: None,
        },
        BridgeLimits::default(),
    )
    .expect("temporal join strict reread");
    assert_eq!(validated_join.decision(), &joined);
    let unavailable = temporal::join(
        projection.validated(),
        None,
        Some(&tl_mapped),
        None,
        BridgeLimits::default(),
    );
    assert_eq!(unavailable.kind(), temporal::TemporalJoinKind::Unavailable);
    assert_eq!(
        unavailable.comparison(),
        temporal::JoinComparison::NotCompared
    );
    assert_eq!(unavailable.value(), None);
    let mut trailing = joined.bytes().to_vec();
    trailing.push(b'\n');
    assert!(temporal::read_join(
        &trailing,
        ExpectedTemporalJoin {
            projection: projection.validated(),
            native: Some(&native),
            tl: Some(&tl_mapped),
            prior: None,
        },
        BridgeLimits::default(),
    )
    .is_err());

    let correction_limits = BridgeLimits {
        visited_work: 1_999_999,
        ..BridgeLimits::default()
    };
    let corrected_projection = temporal::project(
        fixture.temporal(),
        predicates.validated(),
        observations,
        target,
        correction_limits,
    )
    .expect("corrected temporal projection");
    assert_eq!(
        corrected_projection.decision().correspondence_identity(),
        projection.decision().correspondence_identity()
    );
    assert_ne!(
        corrected_projection
            .validated()
            .native_request()
            .document()
            .identity(),
        projection
            .validated()
            .native_request()
            .document()
            .identity()
    );
    let corrected_native_document = native_result::evaluate(
        corrected_projection.validated().native_request(),
        NativeRelation::Superseding(&native),
        corrected_projection.validated().native_request().limits(),
    )
    .into_result()
    .expect("corrected native result");
    let corrected_native = native_result::read(
        corrected_native_document.bytes(),
        corrected_projection.validated().native_request(),
        NativeRelation::Superseding(&native),
        corrected_projection.validated().native_request().limits(),
    )
    .into_result()
    .expect("corrected native result reader");
    let corrected_tl_document = wire::report::evaluate(
        corrected_projection.validated().tl_request(),
        wire::report::ResultRelationInput::Superseding(&tl_result),
        OwnerLimits::owner_max(),
    )
    .expect("corrected TL result");
    let corrected_tl = wire::report::read(
        corrected_tl_document.bytes(),
        corrected_projection.validated().tl_request(),
        wire::report::ResultRelationInput::Superseding(&tl_result),
        OwnerLimits::owner_max(),
    )
    .expect("corrected TL result reader");
    let corrected_selection = mapping::contract_ir::MappingSelection::for_result(&corrected_tl);
    let corrected_map_document = mapping::contract_ir::map(
        &corrected_tl,
        &corrected_selection,
        OwnerLimits::owner_max(),
    )
    .expect("corrected TL mapping");
    let corrected_map = mapping::contract_ir::read(
        corrected_map_document.bytes(),
        &corrected_tl,
        &corrected_selection,
        OwnerLimits::owner_max(),
    )
    .expect("corrected TL mapping reader");
    let missing_prior = temporal::join(
        corrected_projection.validated(),
        Some(&corrected_native),
        Some(&corrected_map),
        None,
        correction_limits,
    );
    assert_eq!(missing_prior.kind(), temporal::TemporalJoinKind::Conflict);
    assert!(missing_prior
        .causes()
        .iter()
        .any(|cause| cause.code() == Code::TemporalSupersessionInvalid));
    let mixed_lineage = temporal::join(
        corrected_projection.validated(),
        Some(&corrected_native),
        Some(&tl_mapped),
        Some(&validated_join),
        correction_limits,
    );
    assert_eq!(mixed_lineage.kind(), temporal::TemporalJoinKind::Conflict);
    assert_eq!(mixed_lineage.value(), None);
    let corrected_join = temporal::join(
        corrected_projection.validated(),
        Some(&corrected_native),
        Some(&corrected_map),
        Some(&validated_join),
        correction_limits,
    );
    assert_eq!(corrected_join.kind(), temporal::TemporalJoinKind::Agreement);
    assert_eq!(
        corrected_join.relation(),
        Some(temporal::TemporalJoinRelation::Superseding)
    );
    temporal::read_join(
        corrected_join.bytes(),
        ExpectedTemporalJoin {
            projection: corrected_projection.validated(),
            native: Some(&corrected_native),
            tl: Some(&corrected_map),
            prior: Some(&validated_join),
        },
        correction_limits,
    )
    .expect("corrected temporal join strict reread");
}

#[trace("TC-039", "FR-026-AC-1", "FR-026-AC-3", "FR-026-AC-6", "FR-026-AC-7")]
#[test]
fn tc_039_fixed_sample_requests_are_owner_read_and_evaluated_independently() {
    let fixture = result_fixture::fixture_for_temporal(1);
    let predicates = predicate::project(
        fixture.predicates(),
        predicate::TargetSelection::current(),
        BridgeLimits::default(),
    )
    .expect("predicate projection");
    let decision = temporal_fixture::fixed_sample(&fixture, "decision-fixed", OpenClosed::Closed);
    let surrounding =
        temporal_fixture::fixed_sample(&fixture, "surrounding-fixed", OpenClosed::Closed);
    let leaf = result_fixture::validated_result_for_temporal(
        &fixture,
        &decision.progress,
        &decision.closure,
        &surrounding.progress,
        &surrounding.closure,
        &decision.completeness,
        &decision.observation_identity,
        Truth::Violated,
    );
    let leaf_map = contract_ir::map(
        &leaf,
        MappingSelection::current(),
        ResultLimits::owner_max(),
    )
    .expect("leaf result map");
    let availability = decision.availability(leaf.result_id());
    let valuation = predicate::value(
        predicates.validated(),
        predicates.decision().correspondences()[0].predicate_ref(),
        &availability,
        Some(&leaf_map),
        BridgeLimits::default(),
    );
    let cells = [valuation];
    let position_values = [PositionValuations {
        position: 0,
        observation_identity: &decision.observation_identity,
        valuations: &cells,
    }];
    let projection = temporal::project(
        fixture.temporal(),
        predicates.validated(),
        ObservationViews {
            position_ledger: &decision.position,
            clock: &decision.clock,
            capture: &decision.capture,
            trigger_scope_closure: &decision.closure,
            decision_scope_progress: &decision.progress,
            decision_scope_closure: &decision.closure,
            surrounding_execution_progress: &surrounding.progress,
            surrounding_execution_closure: &surrounding.closure,
            completeness: &decision.completeness,
            availability: &availability,
            activation_guard: None,
            positions: &position_values,
            anchor: 0,
        },
        temporal::TargetSelection::current(),
        BridgeLimits::default(),
    )
    .expect("fixed-sample temporal projection");
    let native_document = native_result::evaluate(
        projection.validated().native_request(),
        NativeRelation::Original,
        projection.validated().native_request().limits(),
    )
    .into_result()
    .expect("fixed native result");
    let native = native_result::read(
        native_document.bytes(),
        projection.validated().native_request(),
        NativeRelation::Original,
        projection.validated().native_request().limits(),
    )
    .into_result()
    .expect("fixed native reader");
    let tl_document = wire::report::evaluate(
        projection.validated().tl_request(),
        wire::report::ResultRelationInput::Original,
        OwnerLimits::owner_max(),
    )
    .expect("fixed TL result");
    let tl = wire::report::read(
        tl_document.bytes(),
        projection.validated().tl_request(),
        wire::report::ResultRelationInput::Original,
        OwnerLimits::owner_max(),
    )
    .expect("fixed TL reader");
    let selection = mapping::contract_ir::MappingSelection::for_result(&tl);
    let mapped_document =
        mapping::contract_ir::map(&tl, &selection, OwnerLimits::owner_max()).expect("fixed map");
    let mapped = mapping::contract_ir::read(
        mapped_document.bytes(),
        &tl,
        &selection,
        OwnerLimits::owner_max(),
    )
    .expect("fixed map reader");
    let joined = temporal::join(
        projection.validated(),
        Some(&native),
        Some(&mapped),
        None,
        BridgeLimits::default(),
    );
    assert_eq!(
        joined.kind(),
        temporal::TemporalJoinKind::Agreement,
        "{:?}",
        joined.causes()
    );
    assert_eq!(joined.value(), Some(false));
}

#[trace("TC-039", "FR-026-AC-1", "FR-026-AC-3", "FR-026-AC-8")]
#[test]
fn tc_039_timestamped_event_profile_is_explicitly_unsupported() {
    let fixture = result_fixture::fixture_for_temporal(2);
    let predicates = predicate::project(
        fixture.predicates(),
        predicate::TargetSelection::current(),
        BridgeLimits::default(),
    )
    .expect("predicate projection");
    let decision =
        temporal_fixture::timestamped_event(&fixture, "decision-timestamp", OpenClosed::Closed);
    let surrounding =
        temporal_fixture::timestamped_event(&fixture, "surrounding-timestamp", OpenClosed::Closed);
    let leaf = result_fixture::validated_result_for_temporal(
        &fixture,
        &decision.progress,
        &decision.closure,
        &surrounding.progress,
        &surrounding.closure,
        &decision.completeness,
        &decision.observation_identity,
        Truth::Violated,
    );
    let leaf_map = contract_ir::map(
        &leaf,
        MappingSelection::current(),
        ResultLimits::owner_max(),
    )
    .expect("leaf result map");
    let availability = decision.availability(leaf.result_id());
    let valuation = predicate::value(
        predicates.validated(),
        predicates.decision().correspondences()[0].predicate_ref(),
        &availability,
        Some(&leaf_map),
        BridgeLimits::default(),
    );
    let cells = [valuation];
    let positions = [PositionValuations {
        position: 0,
        observation_identity: &decision.observation_identity,
        valuations: &cells,
    }];
    let outcome = temporal::project(
        fixture.temporal(),
        predicates.validated(),
        ObservationViews {
            position_ledger: &decision.position,
            clock: &decision.clock,
            capture: &decision.capture,
            trigger_scope_closure: &decision.closure,
            decision_scope_progress: &decision.progress,
            decision_scope_closure: &decision.closure,
            surrounding_execution_progress: &surrounding.progress,
            surrounding_execution_closure: &surrounding.closure,
            completeness: &decision.completeness,
            availability: &availability,
            activation_guard: None,
            positions: &positions,
            anchor: 0,
        },
        temporal::TargetSelection::current(),
        BridgeLimits::default(),
    );
    let Err(TemporalDecision::Semantic(refusal)) = outcome else {
        panic!("timestamped-event projection must be a semantic refusal");
    };
    assert_eq!(refusal.kind(), TemporalProjectionKind::Unsupported);
    assert!(refusal.causes().iter().any(|cause| {
        cause.dimension() == Dim::TemporalProfile
            && cause.code() == Code::TemporalProfileUnsupported
    }));
}

#[trace("TC-039", "FR-026-AC-3", "FR-026-AC-5", "FR-026-AC-7")]
#[test]
fn tc_039_open_future_pending_remains_a_non_boolean_agreement() {
    let fixture = result_fixture::fixture();
    let predicates = predicate::project(
        fixture.predicates(),
        predicate::TargetSelection::current(),
        BridgeLimits::default(),
    )
    .expect("predicate projection");
    let decision = temporal_fixture::event_position(&fixture, "decision-open", OpenClosed::Open);
    let surrounding =
        temporal_fixture::event_position(&fixture, "surrounding-open", OpenClosed::Open);
    let leaf = result_fixture::validated_result_for_temporal(
        &fixture,
        &decision.progress,
        &decision.closure,
        &surrounding.progress,
        &surrounding.closure,
        &decision.completeness,
        &decision.observation_identity,
        Truth::Satisfied,
    );
    let leaf_map = contract_ir::map(
        &leaf,
        MappingSelection::current(),
        ResultLimits::owner_max(),
    )
    .expect("leaf result map");
    let availability = decision.availability(leaf.result_id());
    let valuation = predicate::value(
        predicates.validated(),
        predicates.decision().correspondences()[0].predicate_ref(),
        &availability,
        Some(&leaf_map),
        BridgeLimits::default(),
    );
    let cells = [valuation];
    let position_values = [PositionValuations {
        position: 0,
        observation_identity: &decision.observation_identity,
        valuations: &cells,
    }];
    let projection = temporal::project(
        fixture.temporal(),
        predicates.validated(),
        ObservationViews {
            position_ledger: &decision.position,
            clock: &decision.clock,
            capture: &decision.capture,
            trigger_scope_closure: &decision.closure,
            decision_scope_progress: &decision.progress,
            decision_scope_closure: &decision.closure,
            surrounding_execution_progress: &surrounding.progress,
            surrounding_execution_closure: &surrounding.closure,
            completeness: &decision.completeness,
            availability: &availability,
            activation_guard: None,
            positions: &position_values,
            anchor: 0,
        },
        temporal::TargetSelection::current(),
        BridgeLimits::default(),
    )
    .expect("open temporal projection");
    assert_eq!(
        projection.validated().formula().semantic_profile(),
        tl_syntax::SemanticProfile::OnlinePrefixV1
    );
    let native_document = native_result::evaluate(
        projection.validated().native_request(),
        NativeRelation::Original,
        projection.validated().native_request().limits(),
    )
    .into_result()
    .expect("open native result");
    let native = native_result::read(
        native_document.bytes(),
        projection.validated().native_request(),
        NativeRelation::Original,
        projection.validated().native_request().limits(),
    )
    .into_result()
    .expect("open native reader");
    let tl_document = wire::report::evaluate(
        projection.validated().tl_request(),
        wire::report::ResultRelationInput::Original,
        OwnerLimits::owner_max(),
    )
    .expect("open TL result");
    let tl = wire::report::read(
        tl_document.bytes(),
        projection.validated().tl_request(),
        wire::report::ResultRelationInput::Original,
        OwnerLimits::owner_max(),
    )
    .expect("open TL reader");
    let selection = mapping::contract_ir::MappingSelection::for_result(&tl);
    let mapped_document =
        mapping::contract_ir::map(&tl, &selection, OwnerLimits::owner_max()).expect("open map");
    let mapped = mapping::contract_ir::read(
        mapped_document.bytes(),
        &tl,
        &selection,
        OwnerLimits::owner_max(),
    )
    .expect("open map reader");
    let joined = temporal::join(
        projection.validated(),
        Some(&native),
        Some(&mapped),
        None,
        BridgeLimits::default(),
    );
    assert_eq!(
        joined.kind(),
        temporal::TemporalJoinKind::Agreement,
        "{:?}",
        joined.causes()
    );
    assert_eq!(joined.comparison(), temporal::JoinComparison::EqualPending);
    assert_eq!(joined.value(), None);
}

#[trace("TC-039", "FR-026-AC-1", "FR-026-AC-3", "FR-026-AC-4", "FR-026-AC-7")]
#[test]
fn tc_039_past_origin_false_extension_builds_formula_v2_and_history() {
    with_authored_temporal("once[1,1] holds(view.ready)", |fixture| {
        let predicates = predicate::project(
            fixture.predicates(),
            predicate::TargetSelection::current(),
            BridgeLimits::default(),
        )
        .expect("predicate projection");
        let decision =
            temporal_fixture::event_position(&fixture, "decision-past", OpenClosed::Closed);
        let surrounding =
            temporal_fixture::event_position(&fixture, "surrounding-past", OpenClosed::Closed);
        let leaf = result_fixture::validated_result_for_temporal(
            &fixture,
            &decision.progress,
            &decision.closure,
            &surrounding.progress,
            &surrounding.closure,
            &decision.completeness,
            &decision.observation_identity,
            Truth::Satisfied,
        );
        let leaf_map = contract_ir::map(
            &leaf,
            MappingSelection::current(),
            ResultLimits::owner_max(),
        )
        .expect("leaf result map");
        let availability = decision.availability(leaf.result_id());
        let valuation = predicate::value(
            predicates.validated(),
            predicates.decision().correspondences()[0].predicate_ref(),
            &availability,
            Some(&leaf_map),
            BridgeLimits::default(),
        );
        let cells = [valuation];
        let position_values = [PositionValuations {
            position: 0,
            observation_identity: &decision.observation_identity,
            valuations: &cells,
        }];
        let projection = temporal::project(
            fixture.temporal(),
            predicates.validated(),
            ObservationViews {
                position_ledger: &decision.position,
                clock: &decision.clock,
                capture: &decision.capture,
                trigger_scope_closure: &decision.closure,
                decision_scope_progress: &decision.progress,
                decision_scope_closure: &decision.closure,
                surrounding_execution_progress: &surrounding.progress,
                surrounding_execution_closure: &surrounding.closure,
                completeness: &decision.completeness,
                availability: &availability,
                activation_guard: None,
                positions: &position_values,
                anchor: 0,
            },
            temporal::TargetSelection::current(),
            BridgeLimits::default(),
        )
        .expect("past temporal projection");
        assert_eq!(
            projection.validated().formula().schema_version(),
            tl_syntax::FormulaSchemaVersion::V2
        );
        assert_eq!(
            projection.validated().formula().semantic_profile(),
            tl_syntax::SemanticProfile::OriginCompleteHistoryV1
        );
        assert!(projection.validated().history_requirement().is_some());
        let native_document = native_result::evaluate(
            projection.validated().native_request(),
            NativeRelation::Original,
            projection.validated().native_request().limits(),
        )
        .into_result()
        .expect("past native result");
        let native = native_result::read(
            native_document.bytes(),
            projection.validated().native_request(),
            NativeRelation::Original,
            projection.validated().native_request().limits(),
        )
        .into_result()
        .expect("past native reader");
        let tl_document = wire::report::evaluate(
            projection.validated().tl_request(),
            wire::report::ResultRelationInput::Original,
            OwnerLimits::owner_max(),
        )
        .expect("past TL result");
        let tl = wire::report::read(
            tl_document.bytes(),
            projection.validated().tl_request(),
            wire::report::ResultRelationInput::Original,
            OwnerLimits::owner_max(),
        )
        .expect("past TL reader");
        let selection = mapping::contract_ir::MappingSelection::for_result(&tl);
        let mapped_document =
            mapping::contract_ir::map(&tl, &selection, OwnerLimits::owner_max()).expect("past map");
        let mapped = mapping::contract_ir::read(
            mapped_document.bytes(),
            &tl,
            &selection,
            OwnerLimits::owner_max(),
        )
        .expect("past map reader");
        let joined = temporal::join(
            projection.validated(),
            Some(&native),
            Some(&mapped),
            None,
            BridgeLimits::default(),
        );
        assert_eq!(
            joined.kind(),
            temporal::TemporalJoinKind::Agreement,
            "{:?}",
            joined.causes()
        );
        assert_eq!(joined.value(), Some(false));
    });
}

#[trace("TC-039", "FR-026-AC-1", "FR-026-AC-5", "FR-026-AC-7")]
#[test]
fn tc_039_each_activation_uses_its_own_checked_guard_valuation() {
    with_authored_temporal_activation(
        "on each (started: M::Plain) when (started.ready)",
        "always[0,0] holds(view.ready)",
        |fixture| {
            assert_authored_guard_refusal(
                &fixture,
                AuthoredProjectionMode::MissingGuard,
                Code::TemporalActivationIncomplete,
            );
            assert_authored_guard_refusal(
                &fixture,
                AuthoredProjectionMode::FormulaLeafAsGuard,
                Code::TemporalActivationMismatch,
            );
            let projection = project_authored_temporal(&fixture, BridgeLimits::default())
                .expect("each-activated temporal projection");
            let trigger = projection
                .validated()
                .native_request()
                .triggers()
                .next()
                .expect("one admitted trigger");
            assert_eq!(trigger.guard(), Some(true));
            let false_guard = project_authored_temporal_with_guard(
                &fixture,
                BridgeLimits::default(),
                AuthoredProjectionMode::FalseGuard,
            )
            .expect("false activation guard projection");
            assert_eq!(
                false_guard
                    .validated()
                    .native_request()
                    .triggers()
                    .next()
                    .expect("one false-guard trigger")
                    .guard(),
                Some(false)
            );
            assert_ne!(
                projection.decision().correspondence_identity(),
                false_guard.decision().correspondence_identity(),
                "activation-guard decisions must participate in correspondence identity"
            );

            let native_document = native_result::evaluate(
                projection.validated().native_request(),
                NativeRelation::Original,
                projection.validated().native_request().limits(),
            )
            .into_result()
            .expect("each native result");
            let native = native_result::read(
                native_document.bytes(),
                projection.validated().native_request(),
                NativeRelation::Original,
                projection.validated().native_request().limits(),
            )
            .into_result()
            .expect("each native result reader");
            let tl_document = wire::report::evaluate(
                projection.validated().tl_request(),
                wire::report::ResultRelationInput::Original,
                OwnerLimits::owner_max(),
            )
            .expect("each TL result");
            let tl = wire::report::read(
                tl_document.bytes(),
                projection.validated().tl_request(),
                wire::report::ResultRelationInput::Original,
                OwnerLimits::owner_max(),
            )
            .expect("each TL reader");
            let selection = mapping::contract_ir::MappingSelection::for_result(&tl);
            let mapped_document =
                mapping::contract_ir::map(&tl, &selection, OwnerLimits::owner_max())
                    .expect("each TL map");
            let mapped = mapping::contract_ir::read(
                mapped_document.bytes(),
                &tl,
                &selection,
                OwnerLimits::owner_max(),
            )
            .expect("each TL map reader");
            let joined = temporal::join(
                projection.validated(),
                Some(&native),
                Some(&mapped),
                None,
                BridgeLimits::default(),
            );
            assert_eq!(
                joined.kind(),
                temporal::TemporalJoinKind::Agreement,
                "{:?}",
                joined.causes()
            );
            assert_eq!(joined.value(), Some(true));
        },
    );
}

#[trace("TC-039", "FR-026-AC-1", "FR-026-AC-4", "FR-026-AC-8")]
#[test]
fn tc_039_zero_width_and_u32_max_intervals_are_preserved_exactly() {
    with_projected_authored_temporal("always[0,0] holds(view.ready)", |projection| {
        let root = projection.formula().root();
        let root_index = usize::try_from(root.0).expect("validated node id");
        let interval = match &projection.formula().nodes()[root_index].kind {
            tl_syntax::NodeKind::Globally { interval, .. } => interval,
            other => panic!("expected authored unary temporal root, got {other:?}"),
        };
        assert_eq!((interval.start(), interval.end()), (0, 0));
    });

    with_authored_temporal(
        "eventually[4294967295,4294967295] holds(view.ready)",
        |fixture| {
            let error = project_authored_temporal(&fixture, BridgeLimits::default())
                .expect_err("maximum interval requires an unavailable bounded horizon");
            let TemporalDecision::Operation(error) = error else {
                panic!("maximum interval must fail as a bounded operation");
            };
            assert_eq!(
                error.code(),
                quire_contract_ir::bridge::BridgeErrorCode::TemporalProjectionResourceExhausted
            );
            assert!(error.path().contains("history_span"));
        },
    );

    for (formula, since) in [
        ("holds(view.ready) since[0,0] holds(view.ready)", true),
        ("holds(view.ready) triggered[0,0] holds(view.ready)", false),
    ] {
        with_projected_authored_temporal(formula, |projection| {
            let root = projection.formula().root();
            let root_index = usize::try_from(root.0).expect("validated node id");
            let (interval, left, right) = match &projection.formula().nodes()[root_index].kind {
                tl_syntax::NodeKind::Since {
                    interval,
                    left,
                    right,
                } if since => (interval, left, right),
                tl_syntax::NodeKind::Triggered {
                    interval,
                    left,
                    right,
                } if !since => (interval, left, right),
                other => panic!("wrong bounded-past binary mapping: {other:?}"),
            };
            assert_eq!((interval.start(), interval.end()), (0, 0));
            assert_ne!(
                left, right,
                "repeated native occurrences must not deduplicate"
            );
        });
    }
}

#[trace("TC-039", "FR-026-AC-1", "FR-026-AC-4", "FR-026-AC-7")]
#[test]
fn tc_039_every_supported_operator_agrees_between_independent_owners() {
    for (formula, expected) in [
        ("not holds(view.ready)", false),
        ("true and holds(view.ready)", true),
        ("false or holds(view.ready)", true),
        ("false implies holds(view.ready)", true),
        ("eventually[0,0] holds(view.ready)", true),
        ("always[0,0] holds(view.ready)", true),
        ("false until[1,1] holds(view.ready)", false),
        ("true release[1,1] holds(view.ready)", false),
        ("once[0,0] holds(view.ready)", true),
        ("historically[0,0] holds(view.ready)", true),
        ("false since[1,1] holds(view.ready)", false),
        ("true triggered[1,1] holds(view.ready)", false),
    ] {
        with_authored_temporal(formula, |fixture| {
            assert!(
                !fixture.predicates().is_empty(),
                "operator fixture must expose a checked leaf: {formula}"
            );
            let projection = project_authored_temporal(&fixture, BridgeLimits::default())
                .expect("supported operator projection");
            let joined = evaluate_owner_agreement(projection.validated());
            assert_eq!(
                joined.kind(),
                temporal::TemporalJoinKind::Agreement,
                "{formula}: {:?}",
                joined.causes()
            );
            assert_eq!(joined.value(), Some(expected), "{formula}");
        });
    }
}

#[trace("TC-039", "FR-026-AC-2", "FR-026-AC-8")]
#[test]
fn tc_039_valuation_set_order_does_not_change_correspondence_identity() {
    with_authored_temporal("holds(view.ready) and holds(not view.ready)", |fixture| {
        let canonical = project_authored_temporal(&fixture, BridgeLimits::default())
            .expect("canonical valuation order");
        let reversed = project_authored_temporal_with_guard(
            &fixture,
            BridgeLimits::default(),
            AuthoredProjectionMode::ReverseFormulaCells,
        )
        .expect("reversed valuation set order");
        assert_eq!(
            canonical.decision().bytes(),
            reversed.decision().bytes(),
            "set ordering must not alter a canonical correspondence"
        );
    });
}

#[trace("TC-039", "FR-026-AC-2", "FR-026-AC-8")]
#[test]
fn tc_039_every_owner_contract_axis_is_selected_independently() {
    let fixture = result_fixture::fixture();
    let predicates = predicate::project(
        fixture.predicates(),
        predicate::TargetSelection::current(),
        BridgeLimits::default(),
    )
    .expect("predicate projection");
    let decision =
        temporal_fixture::event_position(&fixture, "decision-selection", OpenClosed::Closed);
    let surrounding =
        temporal_fixture::event_position(&fixture, "surrounding-selection", OpenClosed::Closed);
    let result = result_fixture::validated_result_for_temporal(
        &fixture,
        &decision.progress,
        &decision.closure,
        &surrounding.progress,
        &surrounding.closure,
        &decision.completeness,
        &decision.observation_identity,
        Truth::Violated,
    );
    let mapped = contract_ir::map(
        &result,
        MappingSelection::current(),
        ResultLimits::owner_max(),
    )
    .expect("native result map");
    let availability = decision.availability(result.result_id());
    let predicate_ref = predicates.decision().correspondences()[0].predicate_ref();
    let valuation = predicate::value(
        predicates.validated(),
        predicate_ref,
        &availability,
        Some(&mapped),
        BridgeLimits::default(),
    );
    let cells = [valuation];
    let position_values = [PositionValuations {
        position: 0,
        observation_identity: &decision.observation_identity,
        valuations: &cells,
    }];
    let observations = ObservationViews {
        position_ledger: &decision.position,
        clock: &decision.clock,
        capture: &decision.capture,
        trigger_scope_closure: &decision.closure,
        decision_scope_progress: &decision.progress,
        decision_scope_closure: &decision.closure,
        surrounding_execution_progress: &surrounding.progress,
        surrounding_execution_closure: &surrounding.closure,
        completeness: &decision.completeness,
        availability: &availability,
        activation_guard: None,
        positions: &position_values,
        anchor: 0,
    };
    let expected = [
        (
            Dim::NativeContract,
            Code::TemporalNativeContractUnsupported,
            Code::TemporalNativeContractUnavailable,
            Code::TemporalNativeContractConflict,
        ),
        (
            Dim::PredicateProjection,
            Code::TemporalPredicateProjectionContractUnsupported,
            Code::TemporalPredicateProjectionContractUnavailable,
            Code::TemporalPredicateProjectionContractConflict,
        ),
        (
            Dim::FormulaContract,
            Code::TemporalFormulaContractUnsupported,
            Code::TemporalFormulaContractUnavailable,
            Code::TemporalFormulaContractConflict,
        ),
        (
            Dim::FormulaContract,
            Code::TemporalFormulaContractUnsupported,
            Code::TemporalFormulaContractUnavailable,
            Code::TemporalFormulaContractConflict,
        ),
        (
            Dim::FormulaContract,
            Code::TemporalFormulaContractUnsupported,
            Code::TemporalFormulaContractUnavailable,
            Code::TemporalFormulaContractConflict,
        ),
        (
            Dim::SemanticContract,
            Code::TemporalSemanticContractUnsupported,
            Code::TemporalSemanticContractUnavailable,
            Code::TemporalSemanticContractConflict,
        ),
        (
            Dim::ObservationContract,
            Code::TemporalObservationContractUnsupported,
            Code::TemporalObservationContractUnavailable,
            Code::TemporalObservationContractConflict,
        ),
        (
            Dim::ClockContract,
            Code::TemporalClockContractUnsupported,
            Code::TemporalClockContractUnavailable,
            Code::TemporalClockContractConflict,
        ),
        (
            Dim::CaptureContract,
            Code::TemporalCaptureContractUnsupported,
            Code::TemporalCaptureContractUnavailable,
            Code::TemporalCaptureContractConflict,
        ),
        (
            Dim::ProgressContract,
            Code::TemporalProgressContractUnsupported,
            Code::TemporalProgressContractUnavailable,
            Code::TemporalProgressContractConflict,
        ),
        (
            Dim::ProgressContract,
            Code::TemporalProgressContractUnsupported,
            Code::TemporalProgressContractUnavailable,
            Code::TemporalProgressContractConflict,
        ),
        (
            Dim::CompletenessContract,
            Code::TemporalCompletenessContractUnsupported,
            Code::TemporalCompletenessContractUnavailable,
            Code::TemporalCompletenessContractConflict,
        ),
        (
            Dim::AvailabilityContract,
            Code::TemporalAvailabilityContractUnsupported,
            Code::TemporalAvailabilityContractUnavailable,
            Code::TemporalAvailabilityContractConflict,
        ),
        (
            Dim::TraceContract,
            Code::TemporalTraceContractUnsupported,
            Code::TemporalTraceContractUnavailable,
            Code::TemporalTraceContractConflict,
        ),
        (
            Dim::TraceContract,
            Code::TemporalTraceContractUnsupported,
            Code::TemporalTraceContractUnavailable,
            Code::TemporalTraceContractConflict,
        ),
        (
            Dim::RequestContract,
            Code::TemporalRequestContractUnsupported,
            Code::TemporalRequestContractUnavailable,
            Code::TemporalRequestContractConflict,
        ),
        (
            Dim::RequestContract,
            Code::TemporalRequestContractUnsupported,
            Code::TemporalRequestContractUnavailable,
            Code::TemporalRequestContractConflict,
        ),
        (
            Dim::EvaluatorContract,
            Code::TemporalEvaluatorContractUnsupported,
            Code::TemporalEvaluatorContractUnavailable,
            Code::TemporalEvaluatorContractConflict,
        ),
        (
            Dim::RequestContract,
            Code::TemporalRequestContractUnsupported,
            Code::TemporalRequestContractUnavailable,
            Code::TemporalRequestContractConflict,
        ),
        (
            Dim::NativeResultContract,
            Code::TemporalNativeResultContractUnsupported,
            Code::TemporalNativeResultContractUnavailable,
            Code::TemporalNativeResultContractConflict,
        ),
        (
            Dim::NativeResultContract,
            Code::TemporalNativeResultContractUnsupported,
            Code::TemporalNativeResultContractUnavailable,
            Code::TemporalNativeResultContractConflict,
        ),
        (
            Dim::NativeResultContract,
            Code::TemporalNativeResultContractUnsupported,
            Code::TemporalNativeResultContractUnavailable,
            Code::TemporalNativeResultContractConflict,
        ),
        (
            Dim::TlResultContract,
            Code::TemporalTlResultContractUnsupported,
            Code::TemporalTlResultContractUnavailable,
            Code::TemporalTlResultContractConflict,
        ),
    ];
    for (axis, (dimension, unsupported, unavailable, conflict)) in
        TargetContract::ALL.into_iter().zip(expected)
    {
        let current = temporal::TargetSelection::current();
        let selected = current.selection(axis);
        let unsupported_selection = ContractSelection::new(
            format!("{}.unknown", selected.contract()),
            selected.package_version(),
            selected.repository(),
            selected.revision(),
            selected.schema_digest(),
        );
        assert_selection_refusal(
            fixture.temporal(),
            predicates.validated(),
            observations,
            current.clone().with_selection(axis, unsupported_selection),
            TemporalProjectionKind::Unsupported,
            dimension,
            unsupported,
        );
        let unavailable_selection = ContractSelection::new(
            selected.contract(),
            selected.package_version(),
            selected.repository(),
            "0000000000000000000000000000000000000000",
            selected.schema_digest(),
        );
        assert_selection_refusal(
            fixture.temporal(),
            predicates.validated(),
            observations,
            current.clone().with_selection(axis, unavailable_selection),
            TemporalProjectionKind::Unavailable,
            dimension,
            unavailable,
        );
        let conflict_selection = ContractSelection::new(
            selected.contract(),
            selected.package_version(),
            selected.repository(),
            selected.revision(),
            BridgeDigest::raw(b"wrong-schema"),
        );
        assert_selection_refusal(
            fixture.temporal(),
            predicates.validated(),
            observations,
            current.with_selection(axis, conflict_selection),
            TemporalProjectionKind::Conflict,
            dimension,
            conflict,
        );
    }
}

fn assert_selection_refusal(
    subject: &quire_spec_language::protocol_artifact::temporal_subject::ValidatedTemporalSubject,
    predicates: &predicate::ValidatedPredicateProjection,
    observations: ObservationViews<'_>,
    target: temporal::TargetSelection,
    kind: TemporalProjectionKind,
    dimension: Dim,
    code: Code,
) {
    let decision = match temporal::project(
        subject,
        predicates,
        observations,
        target,
        BridgeLimits::default(),
    ) {
        Err(TemporalDecision::Semantic(decision)) => decision,
        other => panic!("expected semantic target-selection decision, got {other:?}"),
    };
    assert_eq!(decision.kind(), kind);
    let [cause] = decision.causes() else {
        panic!("one exact target-selection cause: {:?}", decision.causes());
    };
    assert_eq!(cause.dimension(), dimension);
    assert_eq!(cause.code(), code);
}

fn assert_projection_refusal(
    subject: &quire_spec_language::protocol_artifact::temporal_subject::ValidatedTemporalSubject,
    predicates: &predicate::ValidatedPredicateProjection,
    observations: ObservationViews<'_>,
    target: temporal::TargetSelection,
    kind: TemporalProjectionKind,
    code: Code,
) {
    let decision = match temporal::project(
        subject,
        predicates,
        observations,
        target,
        BridgeLimits::default(),
    ) {
        Err(TemporalDecision::Semantic(decision)) => decision,
        other => panic!("expected semantic projection refusal, got {other:?}"),
    };
    assert_eq!(decision.kind(), kind);
    assert!(
        decision.causes().iter().any(|cause| cause.code() == code),
        "expected {code:?}, got {:?}",
        decision.causes()
    );
}

fn with_projected_authored_temporal(
    formula: &str,
    test: impl FnOnce(&temporal::ValidatedTemporalProjection),
) {
    with_authored_temporal(formula, |fixture| {
        let projection = project_authored_temporal(&fixture, BridgeLimits::default())
            .expect("authored temporal projection");
        test(projection.validated());
    });
}

fn project_authored_temporal(
    fixture: &result_fixture::Fixture,
    limits: BridgeLimits,
) -> Result<temporal::TemporalProjection, TemporalDecision> {
    project_authored_temporal_with_guard(fixture, limits, AuthoredProjectionMode::Normal)
}

fn evaluate_owner_agreement(
    projection: &temporal::ValidatedTemporalProjection,
) -> temporal::TemporalJoinDecision {
    let native_document = native_result::evaluate(
        projection.native_request(),
        NativeRelation::Original,
        projection.native_request().limits(),
    )
    .into_result()
    .expect("native operator result");
    let native = native_result::read(
        native_document.bytes(),
        projection.native_request(),
        NativeRelation::Original,
        projection.native_request().limits(),
    )
    .into_result()
    .expect("native operator result reader");
    let tl_document = wire::report::evaluate(
        projection.tl_request(),
        wire::report::ResultRelationInput::Original,
        OwnerLimits::owner_max(),
    )
    .expect("TL operator result");
    let tl = wire::report::read(
        tl_document.bytes(),
        projection.tl_request(),
        wire::report::ResultRelationInput::Original,
        OwnerLimits::owner_max(),
    )
    .expect("TL operator result reader");
    let selection = mapping::contract_ir::MappingSelection::for_result(&tl);
    let mapped_document = mapping::contract_ir::map(&tl, &selection, OwnerLimits::owner_max())
        .expect("TL operator mapping");
    let mapped = mapping::contract_ir::read(
        mapped_document.bytes(),
        &tl,
        &selection,
        OwnerLimits::owner_max(),
    )
    .expect("TL operator mapping reader");
    temporal::join(
        projection,
        Some(&native),
        Some(&mapped),
        None,
        BridgeLimits::default(),
    )
}

#[derive(Clone, Copy)]
enum AuthoredProjectionMode {
    Normal,
    MissingGuard,
    FormulaLeafAsGuard,
    FalseGuard,
    ReverseFormulaCells,
}

fn assert_authored_guard_refusal(
    fixture: &result_fixture::Fixture,
    mode: AuthoredProjectionMode,
    code: Code,
) {
    let decision = project_authored_temporal_with_guard(fixture, BridgeLimits::default(), mode)
        .expect_err("invalid activation guard must fail closed");
    let TemporalDecision::Semantic(decision) = decision else {
        panic!("invalid activation guard must produce a semantic refusal: {decision:?}")
    };
    assert_eq!(decision.kind(), TemporalProjectionKind::Refused);
    assert!(
        decision.causes().iter().any(|cause| cause.code() == code),
        "expected {code:?}, got {:?}",
        decision.causes()
    );
}

fn project_authored_temporal_with_guard(
    fixture: &result_fixture::Fixture,
    limits: BridgeLimits,
    mode: AuthoredProjectionMode,
) -> Result<temporal::TemporalProjection, TemporalDecision> {
    let predicates = predicate::project(
        fixture.predicates(),
        predicate::TargetSelection::current(),
        limits,
    )
    .expect("predicate projection");
    let decision = temporal_fixture::event_position(fixture, "decision-bound", OpenClosed::Closed);
    let surrounding =
        temporal_fixture::event_position(fixture, "surrounding-bound", OpenClosed::Closed);
    let activation_handle = match fixture.temporal().activation() {
        Some(qsl_wire::Activation::Each {
            guard: qsl_wire::Nullable(Some(handle)),
            ..
        }) => Some((handle.declaration, handle.index)),
        _ => None,
    };
    let leaves = fixture
        .predicates()
        .iter()
        .map(|checked| {
            let truth = if matches!(mode, AuthoredProjectionMode::FalseGuard)
                && Some((checked.leaf().declaration, checked.leaf().index)) == activation_handle
            {
                Truth::Violated
            } else {
                Truth::Satisfied
            };
            result_fixture::validated_result_for_temporal_predicate(
                fixture,
                checked,
                &decision.progress,
                &decision.closure,
                &surrounding.progress,
                &surrounding.closure,
                &decision.completeness,
                &decision.observation_identity,
                truth,
            )
        })
        .collect::<Vec<_>>();
    let mut result_identities = leaves
        .iter()
        .map(|leaf| leaf.result_id())
        .collect::<Vec<_>>();
    result_identities.sort_unstable();
    assert!(
        result_identities.windows(2).all(|pair| pair[0] != pair[1]),
        "distinct checked leaves must produce distinct owner results"
    );
    let availability = decision.availability_for(&result_identities);
    let leaf_maps = leaves
        .iter()
        .map(|leaf| {
            contract_ir::map(leaf, MappingSelection::current(), ResultLimits::owner_max())
                .expect("leaf result map")
        })
        .collect::<Vec<_>>();
    let cells = predicates
        .decision()
        .correspondences()
        .iter()
        .map(|correspondence| {
            let owner_identity = predicates
                .validated()
                .definition(correspondence.predicate_ref())
                .expect("projected predicate definition")
                .owner_document_identity();
            let leaf_map = leaf_maps
                .iter()
                .find(|mapped| mapped.native_correspondence_identity() == owner_identity)
                .expect("result mapped for exact checked leaf");
            (
                correspondence.predicate_ref(),
                predicate::value(
                    predicates.validated(),
                    correspondence.predicate_ref(),
                    &availability,
                    Some(leaf_map),
                    limits,
                ),
            )
        })
        .collect::<Vec<_>>();
    assert!(
        cells
            .iter()
            .all(|(_, cell)| cell.kind() == predicate::PredicateValuationKind::Valued),
        "every authored temporal leaf must receive an exact valued owner result: {cells:#?}"
    );
    let formula_handles = fixture
        .temporal()
        .predicate_leaves()
        .unwrap_or_default()
        .iter()
        .map(|handle| (handle.declaration, handle.index))
        .collect::<std::collections::BTreeSet<_>>();
    let mut formula_cells = Vec::new();
    let mut activation_guard = None;
    for (predicate_ref, cell) in cells {
        let definition = predicates
            .validated()
            .definition(predicate_ref)
            .expect("projected predicate definition");
        let handle = (definition.leaf_declaration(), definition.leaf_index());
        if formula_handles.contains(&handle) {
            formula_cells.push(cell);
        } else if Some(handle) == activation_handle {
            assert!(
                activation_guard.replace(cell).is_none(),
                "activation guard must be unique"
            );
        } else {
            panic!("unexpected authored predicate handle {handle:?}");
        }
    }
    if matches!(mode, AuthoredProjectionMode::ReverseFormulaCells) {
        formula_cells.reverse();
    }
    let position_values = [PositionValuations {
        position: 0,
        observation_identity: &decision.observation_identity,
        valuations: &formula_cells,
    }];
    let selected_guard = match mode {
        AuthoredProjectionMode::Normal
        | AuthoredProjectionMode::FalseGuard
        | AuthoredProjectionMode::ReverseFormulaCells => activation_guard,
        AuthoredProjectionMode::MissingGuard => None,
        AuthoredProjectionMode::FormulaLeafAsGuard => formula_cells.first().cloned(),
    };
    temporal::project(
        fixture.temporal(),
        predicates.validated(),
        ObservationViews {
            position_ledger: &decision.position,
            clock: &decision.clock,
            capture: &decision.capture,
            trigger_scope_closure: &decision.closure,
            decision_scope_progress: &decision.progress,
            decision_scope_closure: &decision.closure,
            surrounding_execution_progress: &surrounding.progress,
            surrounding_execution_closure: &surrounding.closure,
            completeness: &decision.completeness,
            availability: &availability,
            activation_guard: selected_guard.as_ref(),
            positions: &position_values,
            anchor: 0,
        },
        temporal::TargetSelection::current(),
        limits,
    )
}

fn with_authored_temporal(formula: &str, test: impl FnOnce(result_fixture::Fixture)) {
    with_authored_temporal_activation("on origin", formula, test);
}

fn with_authored_temporal_activation(
    activation: &str,
    formula: &str,
    test: impl FnOnce(result_fixture::Fixture),
) {
    let temporal_source = format!(
        "temporal ByTime using T over (view: M::Plain) clock \"orders\" {activation} {{ {formula} }}"
    );
    let inputs = NativeInputs::new(&[
        Unit {
            name: "temporal-evaluation",
            body: &temporal_source,
            declarations: &["ByTime"],
        },
        Unit {
            name: "temporal-consumer",
            body: "protocol Flow using P over (view: M::Plain) on origin {
                role Service on M::Node;
                requires temporal ByTime;
                run sequence Main {
                    event Happened by Service as (happened: M::Plain) { happened.ready };
                }
                finish Closed as (closed: M::Plain) { closed.ready };
            }",
            declarations: &["Flow"],
        },
    ]);
    inputs.with_proofs(
        TypeLimits::default(),
        proofs::ProofLimits::default(),
        |proofs, selected| {
            assert!(proofs.declarations().iter().all(|entry| {
                proofs.types().disposition(entry.declaration()) == Some(TypeDisposition::Typed)
            }));
            let [declaration_id] = proofs.types().binding().namespace().lookup("ByTime") else {
                panic!("one authored ByTime declaration")
            };
            let source_span = proofs
                .types()
                .binding()
                .namespace()
                .syntax(*declaration_id)
                .expect("authored ByTime syntax")
                .span;
            let span = qsl_wire::Span {
                start: u32::try_from(source_span.start).expect("bounded source start"),
                end: u32::try_from(source_span.end).expect("bounded source end"),
            };
            let definition = RegisteredDefinition::EventPosition;
            let definition_artifact = selected
                .dependencies
                .iter()
                .find(|dependency| {
                    dependency.artifact.identity == definition.identity()
                        && dependency.bytes == definition.bytes()
                })
                .expect("selected event-position definition")
                .artifact;
            let revision = qsl_wire::Revision {
                namespace: selected.definition_revision_namespace.into(),
                value: definition.revision().into(),
            };
            let clock = v2::wire::ClockConfiguration::EventPosition {
                sequence_authority: "orders".into(),
            };
            let producer = [qsl_native::TemporalSelection {
                source: &inputs.source_references[0],
                span: &span,
                definition_identity: definition.identity(),
                definition_revision: &revision,
                definition_artifact,
                clock: &clock,
            }];
            let expected = [inputs.temporal_expectation(
                proofs,
                0,
                "ByTime",
                TemporalDefinitionExpectation {
                    identity: definition.identity().into(),
                    revision: revision.clone(),
                    artifact: definition_artifact.clone(),
                    clock: clock.clone(),
                },
            )];
            let emission =
                qsl_native::admit_v2(proofs, selected, &producer, qsl_artifact::Limits::default())
                    .into_result()
                    .expect("admit authored temporal package");
            let package = inputs
                .read_v2(proofs, &emission, &expected)
                .into_result()
                .expect("strict-read authored temporal package");
            let declaration = package
                .inherited()
                .declarations
                .iter()
                .position(|entry| entry.name == "ByTime")
                .and_then(|value| u32::try_from(value).ok())
                .expect("ByTime declaration index");
            test(result_fixture::fixture_from_package(&package, declaration));
        },
    );
}
