//! The admitted V2 body vocabulary is available as Rust variants.

use crate::support::checked_package::{canonical, evidence_for, v2_all_families};
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
    for node in &admitted.graph().nodes {
        let term = admitted
            .term(&node.node_id)
            .expect("every admitted node has a typed body");
        assert_eq!(
            serde_json::to_value(term).expect("typed body serializes"),
            node.body,
            "typed body preserves the admitted wire value for {}",
            node.node_id.digest
        );
        if matches!(term, CheckedTerm::Aggregate { .. }) {
            saw_aggregate = true;
        }
    }
    assert!(saw_aggregate, "positive fixture includes an aggregate body");
    let expected_id: CheckedSemanticId =
        serde_json::from_value(fixture["package_id"].clone()).expect("fixture identity");
    assert_eq!(admitted.package_id(), &expected_id);
    assert_eq!(
        quire_canonical::to_vec(admitted.identity_preimage(), Limits::new(u64::MAX))
            .expect("admitted JCS preimage"),
        canonical(&fixture["identity_preimage"]),
    );
}
