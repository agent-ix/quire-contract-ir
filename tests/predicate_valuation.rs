#[path = "support/result_fixture.rs"]
mod result_fixture;
#[path = "support/v2_handoff.rs"]
mod v2_handoff;

use ix_trace_rs::trace;
use quire_contract_ir::bridge::{BridgeDigest, BridgeLimits};
use quire_contract_ir::predicate::{
    project, read_valuation, value, ExpectedValuation, PredicateCauseCode, PredicateValuationKind,
    TargetSelection,
};
use quire_observation::authority::availability::State as AvailabilityState;
use quire_protocol::closure::AssessmentExecution;
use quire_protocol::result::contract_ir::{self, MappingSelection};
use quire_protocol::result::{Limits as ResultLimits, SettlementBasis, Truth};

#[trace("TC-038", "FR-025-AC-2", "FR-025-AC-3", "FR-025-AC-8")]
#[test]
fn tc_038_real_owner_mapping_is_the_only_boolean_valuation_source() {
    let projection_owner = result_fixture::fixture();
    let projection = project(
        std::slice::from_ref(projection_owner.predicate()),
        TargetSelection::current(),
        BridgeLimits::default(),
    )
    .expect("predicate projection");
    let predicate_ref = projection.decision().correspondences()[0].predicate_ref();

    let result_owner = result_fixture::fixture();
    let result = result_fixture::validated_result(&result_owner);
    let mapped = contract_ir::map(
        &result,
        MappingSelection::current(),
        ResultLimits::owner_max(),
    )
    .expect("constructor-private result mapping");
    let availability =
        result_fixture::availability(result.result_id(), AvailabilityState::Available);

    let decision = value(
        projection.validated(),
        predicate_ref,
        &availability,
        Some(&mapped),
        BridgeLimits::default(),
    );
    assert_eq!(decision.kind(), PredicateValuationKind::Valued);
    assert_eq!(decision.value(), Some(true));
    assert!(decision.causes().is_empty());
    assert!(decision.operation().is_none());
    assert_eq!(
        decision.availability_contract().revision(),
        "9ac80e93f4b68a2c7d5a337f9a448ad10de798fc"
    );
    assert_eq!(
        decision.availability_contract().schema_digest(),
        BridgeDigest::raw(quire_observation::authority::availability::SCHEMA_BYTES)
    );
    assert_eq!(
        decision
            .result_contract()
            .expect("result selection")
            .revision(),
        "34d1752e6c5f789a52ccf115b0694eedd96cdd46"
    );
    assert_eq!(
        decision
            .result_contract()
            .expect("result selection")
            .schema_digest(),
        BridgeDigest::raw(quire_protocol::result::SCHEMA_BYTES)
    );
    assert_eq!(
        decision
            .mapping_contract()
            .expect("mapping selection")
            .revision(),
        "34d1752e6c5f789a52ccf115b0694eedd96cdd46"
    );
    assert_eq!(
        decision
            .mapping_contract()
            .expect("mapping selection")
            .schema_digest(),
        BridgeDigest::raw(quire_protocol::result::contract_ir::SCHEMA_BYTES)
    );

    let expected = ExpectedValuation::new(
        projection.validated(),
        predicate_ref,
        &availability,
        Some(&mapped),
    );
    let reread = read_valuation(decision.bytes(), expected, BridgeLimits::default())
        .expect("strict valuation re-reader");
    assert_eq!(reread, decision);

    let false_owner = result_fixture::fixture();
    let false_result = result_fixture::validated_result_with(
        &false_owner,
        AssessmentExecution::Completed,
        Truth::Violated,
        SettlementBasis::ClosedScope,
    );
    let false_mapping = contract_ir::map(
        &false_result,
        MappingSelection::current(),
        ResultLimits::owner_max(),
    )
    .expect("false result mapping");
    let false_availability =
        result_fixture::availability(false_result.result_id(), AvailabilityState::Available);
    let false_decision = value(
        projection.validated(),
        predicate_ref,
        &false_availability,
        Some(&false_mapping),
        BridgeLimits::default(),
    );
    assert_eq!(false_decision.kind(), PredicateValuationKind::Valued);
    assert_eq!(false_decision.value(), Some(false));
    assert_eq!(false_decision.truth(), Some("violated"));
}

#[trace("TC-038", "FR-025-AC-3", "FR-025-AC-4")]
#[test]
fn tc_038_owner_non_values_remain_distinct_and_never_coerce_to_boolean() {
    let projection_owner = result_fixture::fixture();
    let projection = project(
        std::slice::from_ref(projection_owner.predicate()),
        TargetSelection::current(),
        BridgeLimits::default(),
    )
    .expect("predicate projection");
    let predicate_ref = projection.decision().correspondences()[0].predicate_ref();

    for (availability_state, expected_kind, expected_code) in [
        (
            AvailabilityState::NotYetObserved,
            PredicateValuationKind::Incomplete,
            PredicateCauseCode::ResultNotYetObserved,
        ),
        (
            AvailabilityState::ProducerUnavailable,
            PredicateValuationKind::Unavailable,
            PredicateCauseCode::ProducerUnavailable,
        ),
        (
            AvailabilityState::ContractUnavailable,
            PredicateValuationKind::Unavailable,
            PredicateCauseCode::ProducerUnavailable,
        ),
    ] {
        let availability = result_fixture::availability("result:required", availability_state);
        let decision = value(
            projection.validated(),
            predicate_ref,
            &availability,
            None,
            BridgeLimits::default(),
        );
        assert_eq!(decision.kind(), expected_kind);
        assert_eq!(decision.value(), None);
        assert_eq!(decision.causes()[0].code(), expected_code);
    }
}

#[trace("TC-038", "FR-025-AC-3", "FR-025-AC-4")]
#[test]
fn tc_038_mapped_execution_non_values_preserve_closed_precedence() {
    let projection_owner = result_fixture::fixture();
    let projection = project(
        std::slice::from_ref(projection_owner.predicate()),
        TargetSelection::current(),
        BridgeLimits::default(),
    )
    .expect("predicate projection");
    let predicate_ref = projection.decision().correspondences()[0].predicate_ref();

    for (execution, expected_kind, expected_code) in [
        (
            AssessmentExecution::Unsupported,
            PredicateValuationKind::Unsupported,
            PredicateCauseCode::ExecutionUnsupported,
        ),
        (
            AssessmentExecution::Refused,
            PredicateValuationKind::Refused,
            PredicateCauseCode::ExecutionRefused,
        ),
        (
            AssessmentExecution::ResourceIncomplete,
            PredicateValuationKind::Incomplete,
            PredicateCauseCode::ExecutionIncomplete,
        ),
        (
            AssessmentExecution::Failed,
            PredicateValuationKind::Failed,
            PredicateCauseCode::ExecutionFailed,
        ),
    ] {
        let result_owner = result_fixture::fixture();
        let result = result_fixture::validated_result_with(
            &result_owner,
            execution,
            Truth::Unavailable,
            SettlementBasis::Unavailable,
        );
        let mapped = contract_ir::map(
            &result,
            MappingSelection::current(),
            ResultLimits::owner_max(),
        )
        .expect("constructor-private non-value mapping");
        assert_eq!(mapped.value(), None);
        assert!(mapped.non_value().is_some());
        let availability =
            result_fixture::availability(result.result_id(), AvailabilityState::Available);
        let decision = value(
            projection.validated(),
            predicate_ref,
            &availability,
            Some(&mapped),
            BridgeLimits::default(),
        );
        assert_eq!(decision.kind(), expected_kind);
        assert_eq!(decision.value(), None);
        assert!(decision
            .causes()
            .iter()
            .any(|cause| cause.code() == expected_code));
    }

    let conflict_owner = result_fixture::fixture();
    let conflict_result = result_fixture::contradicted_non_value(&conflict_owner);
    let conflict_mapping = contract_ir::map(
        &conflict_result,
        MappingSelection::current(),
        ResultLimits::owner_max(),
    )
    .expect("contradicted non-value mapping");
    let conflict_availability =
        result_fixture::availability(conflict_result.result_id(), AvailabilityState::Available);
    let conflict = value(
        projection.validated(),
        predicate_ref,
        &conflict_availability,
        Some(&conflict_mapping),
        BridgeLimits::default(),
    );
    assert_eq!(conflict.kind(), PredicateValuationKind::Conflict);
    assert_eq!(conflict.value(), None);
    assert!(conflict
        .causes()
        .iter()
        .any(|cause| cause.code() == PredicateCauseCode::CompletenessConflict));
}

#[trace("TC-038", "FR-025-AC-2", "FR-025-AC-4")]
#[test]
fn tc_038_cross_wired_availability_refuses_the_mapped_result() {
    let projection_owner = result_fixture::fixture();
    let projection = project(
        std::slice::from_ref(projection_owner.predicate()),
        TargetSelection::current(),
        BridgeLimits::default(),
    )
    .expect("predicate projection");
    let predicate_ref = projection.decision().correspondences()[0].predicate_ref();
    let result_owner = result_fixture::fixture();
    let result = result_fixture::validated_result(&result_owner);
    let mapped = contract_ir::map(
        &result,
        MappingSelection::current(),
        ResultLimits::owner_max(),
    )
    .expect("result mapping");
    let availability = result_fixture::availability("result:other", AvailabilityState::Available);
    let decision = value(
        projection.validated(),
        predicate_ref,
        &availability,
        Some(&mapped),
        BridgeLimits::default(),
    );
    assert_eq!(decision.kind(), PredicateValuationKind::Refused);
    assert_eq!(decision.value(), None);
    assert!(decision.causes().iter().any(|cause| {
        matches!(
            cause.code(),
            PredicateCauseCode::ObservationMismatch
                | PredicateCauseCode::ValuationPopulationMismatch
        )
    }));
}

#[trace("TC-038", "FR-025-AC-6")]
#[test]
fn tc_038_valuation_allocation_failpoint_exposes_no_boolean_or_partial_bytes() {
    let projection_owner = result_fixture::fixture();
    let projection = project(
        std::slice::from_ref(projection_owner.predicate()),
        TargetSelection::current(),
        BridgeLimits::default(),
    )
    .expect("predicate projection");
    let predicate_ref = projection.decision().correspondences()[0].predicate_ref();
    let result_owner = result_fixture::fixture();
    let result = result_fixture::validated_result(&result_owner);
    let mapped = contract_ir::map(
        &result,
        MappingSelection::current(),
        ResultLimits::owner_max(),
    )
    .expect("result mapping");
    let availability =
        result_fixture::availability(result.result_id(), AvailabilityState::Available);
    let decision = value(
        projection.validated(),
        predicate_ref,
        &availability,
        Some(&mapped),
        BridgeLimits {
            allocation_bytes: 0,
            ..BridgeLimits::default()
        },
    );
    assert_eq!(decision.value(), None);
    assert!(decision.bytes().is_empty());
    assert!(decision.causes().is_empty());
    assert_eq!(
        decision
            .operation()
            .expect("resource failure")
            .code()
            .as_str(),
        "predicate_valuation_resource_exhausted"
    );
}

#[trace("TC-038", "FR-025-AC-5")]
#[test]
fn tc_038_incomplete_or_contradicted_facts_outside_deciding_set_preserve_value_as_gaps() {
    let projection_owner = result_fixture::fixture();
    let projection = project(
        std::slice::from_ref(projection_owner.predicate()),
        TargetSelection::current(),
        BridgeLimits::default(),
    )
    .expect("predicate projection");
    let predicate_ref = projection.decision().correspondences()[0].predicate_ref();

    for (contradicted, expected_state) in [(false, "incomplete"), (true, "contradicted")] {
        let result_owner = result_fixture::fixture();
        let result = result_fixture::validated_result_with_outside_gap(&result_owner, contradicted);
        let mapped = contract_ir::map(
            &result,
            MappingSelection::current(),
            ResultLimits::owner_max(),
        )
        .expect("gap mapping");
        assert_eq!(mapped.value(), Some(true));
        let availability =
            result_fixture::availability(result.result_id(), AvailabilityState::Available);
        let decision = value(
            projection.validated(),
            predicate_ref,
            &availability,
            Some(&mapped),
            BridgeLimits::default(),
        );
        assert_eq!(
            decision.kind(),
            PredicateValuationKind::Valued,
            "unexpected causes: {:?}",
            decision.causes()
        );
        assert_eq!(decision.value(), Some(true));
        assert!(decision.causes().is_empty());
        assert_eq!(decision.completeness_gaps().len(), 1);
        assert_eq!(decision.completeness_gaps()[0].state, expected_state);
        assert!(!decision
            .decision_premises()
            .contains(&decision.completeness_gaps()[0].fact_ref));
    }
}

#[trace("TC-038", "FR-025-AC-2", "FR-025-AC-7")]
#[test]
fn tc_038_direct_correction_is_bound_and_predecessor_bytes_remain_immutable() {
    let projection_owner = result_fixture::fixture();
    let projection = project(
        std::slice::from_ref(projection_owner.predicate()),
        TargetSelection::current(),
        BridgeLimits::default(),
    )
    .expect("predicate projection");
    let predicate_ref = projection.decision().correspondences()[0].predicate_ref();

    let result_owner = result_fixture::fixture();
    let predecessor = result_fixture::validated_result(&result_owner);
    let predecessor_bytes = predecessor.bytes().to_vec();
    let predecessor_identity = predecessor.result_id().to_owned();
    let predecessor_digest = predecessor.digest().to_owned();
    let corrected = result_fixture::corrected_result(&result_owner, &predecessor);
    let mapped = contract_ir::map(
        &corrected,
        MappingSelection::current(),
        ResultLimits::owner_max(),
    )
    .expect("corrected mapping");
    let availability =
        result_fixture::availability(corrected.result_id(), AvailabilityState::Available);
    let decision = value(
        projection.validated(),
        predicate_ref,
        &availability,
        Some(&mapped),
        BridgeLimits::default(),
    );
    assert_eq!(decision.kind(), PredicateValuationKind::Unsupported);
    assert_eq!(decision.value(), None);
    assert_eq!(
        decision.predecessor_result_identity(),
        Some(predecessor_identity.as_str())
    );
    assert_eq!(
        decision.predecessor_result_digest(),
        Some(predecessor_digest.as_str())
    );
    assert_eq!(predecessor.result_id(), predecessor_identity);
    assert_eq!(predecessor.bytes(), predecessor_bytes);
}

#[trace("TC-038", "FR-025-AC-6", "FR-025-AC-8")]
#[test]
fn tc_038_valuation_reader_rejects_hostile_bytes_and_wrong_identity() {
    let projection_owner = result_fixture::fixture();
    let projection = project(
        std::slice::from_ref(projection_owner.predicate()),
        TargetSelection::current(),
        BridgeLimits::default(),
    )
    .expect("predicate projection");
    let predicate_ref = projection.decision().correspondences()[0].predicate_ref();
    let result_owner = result_fixture::fixture();
    let result = result_fixture::validated_result(&result_owner);
    let mapped = contract_ir::map(
        &result,
        MappingSelection::current(),
        ResultLimits::owner_max(),
    )
    .expect("result mapping");
    let availability =
        result_fixture::availability(result.result_id(), AvailabilityState::Available);
    let decision = value(
        projection.validated(),
        predicate_ref,
        &availability,
        Some(&mapped),
        BridgeLimits::default(),
    );
    let expected = |selected| {
        ExpectedValuation::new(
            projection.validated(),
            selected,
            &availability,
            Some(&mapped),
        )
    };

    let mut trailing = decision.bytes().to_vec();
    trailing.push(b'\n');
    assert!(read_valuation(&trailing, expected(predicate_ref), BridgeLimits::default()).is_err());

    let mut json: serde_json::Value =
        serde_json::from_slice(decision.bytes()).expect("valuation JSON");
    json.as_object_mut()
        .expect("valuation object")
        .insert("unknown".into(), true.into());
    assert!(read_valuation(
        &serde_json::to_vec(&json).expect("hostile JSON"),
        expected(predicate_ref),
        BridgeLimits::default()
    )
    .is_err());

    let wrong = quire_contract_ir::predicate::PredicateRef::parse(&"0".repeat(64))
        .expect("synthetic predicate digest");
    assert!(read_valuation(decision.bytes(), expected(wrong), BridgeLimits::default()).is_err());
}
