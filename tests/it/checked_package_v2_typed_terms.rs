//! The admitted V2 body vocabulary is available as Rust variants.

use crate::support::checked_package::{
    canonical, evidence_for, positive_operation_identities, v2_all_families,
};
use ix_trace_rs::trace;
use quire_canonical::Limits;
use quire_contract_model::{
    CheckedPackageReadLimits, CheckedPackageV2, CheckedPackageV2ReadResult, CheckedSemanticId,
    CheckedTerm,
};

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
