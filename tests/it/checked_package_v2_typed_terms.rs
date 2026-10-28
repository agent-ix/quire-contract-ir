//! The admitted V2 body vocabulary is available as Rust variants.

use crate::support::checked_package::{
    canonical, evidence_for, positive_operation_identities, v2_all_families,
};
use ix_trace_rs::trace;
use quire_canonical::Limits;
use quire_contract_model::{
    CheckedApplicationOperation, CheckedPackageReadLimits, CheckedPackageV2,
    CheckedPackageV2ReadResult, CheckedSemanticId, CheckedTerm, StateClauseKind,
};
use serde_json::json;

#[trace(
    "TC-048",
    "FR-038-AC-331",
    "FR-038-AC-332",
    "FR-038-AC-333",
    "FR-038-AC-334",
    "FR-040-AC-37"
)]
#[test]
fn public_deserialization_does_not_admit_operation_views() {
    let forged = json!({
        "identity": "not-in-the-operation-catalog",
        "laws": [
            {"role": "ieee_profile", "definition": {"authority": "agent-ix", "identity": "unselected"}},
            {"role": "ieee_profile", "definition": false}
        ],
        "mode": {"kind": "rounding", "value": "arbitrary"},
        "member": {"kind": "field", "declaration": {
            "domain": "quire.checked-semantic-node/v1", "digest": "a".repeat(64)
        }, "name": "forged"},
        "leaves": []
    });
    let operation: CheckedApplicationOperation =
        serde_json::from_value(forged.clone()).expect("public decode retains the wire record");
    assert_eq!(operation.identity(), Some("not-in-the-operation-catalog"));
    assert_eq!(operation.state_clause_kind(), None);
    assert_eq!(operation.projection_field(), None);
    assert_eq!(operation.member_declaration(), None);
    assert_eq!(operation.mode(), None);
    assert!(operation.law_definitions().is_empty());
    assert_eq!(
        serde_json::to_value(&operation).expect("wire round trip"),
        forged
    );

    let clause = json!({
        "identity": "quire.op.state.clause", "laws": [], "mode": null,
        "member": {"kind": "state_clause", "clause": "invariant"}, "leaves": []
    });
    let decoded: CheckedApplicationOperation =
        serde_json::from_value(clause.clone()).expect("public clause decode");
    assert_eq!(decoded.state_clause_kind(), None);
    assert_eq!(
        serde_json::to_value(&decoded).expect("clause wire round trip"),
        clause
    );
    assert_eq!(
        CheckedApplicationOperation::state_clause(StateClauseKind::Invariant, None)
            .state_clause_kind(),
        Some(StateClauseKind::Invariant),
    );
}

#[trace("FR-349-AC-1", "FR-349-AC-3")]
#[test]
fn admitted_v2_bodies_are_closed_typed_terms() {
    let fixture = v2_all_families();
    let admitted = match CheckedPackageV2::read(
        &canonical(&fixture),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&fixture),
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("positive V2 fixture refused: {other:?}"),
    };

    let mut saw_aggregate = false;
    let mut saw_reference = false;
    let mut saw_application_argument = false;
    let mut saw_nested_binding = false;
    for node in &admitted.graph().nodes {
        let body: &CheckedTerm = &node.body;
        let term = admitted
            .term(&node.node_id)
            .expect("every admitted node has a typed body");
        assert_eq!(
            term, body,
            "typed body preserves the admitted wire value for {}",
            node.node_id.digest
        );
        match term {
            CheckedTerm::Aggregate { members } => {
                saw_aggregate = true;
                saw_nested_binding |= members.iter().any(|member| {
                    matches!(member, CheckedTerm::Binding { value, .. }
                        if matches!(value.as_ref(), CheckedTerm::Literal { .. }))
                });
            }
            CheckedTerm::Reference { .. } => saw_reference = true,
            CheckedTerm::Application { arguments, .. } => {
                saw_application_argument |= arguments
                    .iter()
                    .any(|argument| matches!(argument, CheckedTerm::Reference { .. }));
            }
            _ => {}
        }
    }
    assert!(saw_aggregate, "positive fixture includes an aggregate body");
    assert!(saw_reference, "positive fixture includes a reference root");
    assert!(
        saw_application_argument,
        "application has a typed reference argument"
    );
    assert!(
        saw_nested_binding,
        "aggregate has a typed binding and literal"
    );
    let literal_fixture = positive_operation_identities();
    let literal_package = match CheckedPackageV2::read(
        &canonical(&literal_fixture),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&literal_fixture),
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("literal V2 fixture refused: {other:?}"),
    };
    assert!(literal_package.graph().nodes.iter().any(|node| {
        matches!(
            literal_package.term(&node.node_id),
            Some(CheckedTerm::Literal { .. })
        )
    }));
    let expected_id: CheckedSemanticId =
        serde_json::from_value(fixture["package_id"].clone()).expect("fixture identity");
    assert_eq!(admitted.package_id(), &expected_id);
    assert_eq!(
        quire_canonical::to_vec(admitted.identity_preimage(), Limits::new(u64::MAX))
            .expect("admitted JCS preimage"),
        canonical(&fixture["identity_preimage"]),
    );
}
