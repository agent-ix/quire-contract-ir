//! The admitted V2 body vocabulary is available as Rust variants.

use crate::support::checked_package::{canonical, evidence_for, v2_all_families};
use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedPackageReadLimits, CheckedPackageV2, CheckedPackageV2ReadResult, CheckedTerm,
};
use serde_json::json;

#[trace("FR-349-AC-1", "FR-349-AC-2", "FR-349-AC-3")]
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
}

#[trace("FR-349-AC-1", "FR-349-AC-2", "FR-349-AC-3")]
#[test]
fn checked_term_variants_round_trip_the_closed_wire_arms() {
    let node = json!({
        "domain": "quire.checked-semantic-node/v1",
        "digest": "a".repeat(64)
    });
    let package = json!({
        "domain": "quire.package.semantic/v2",
        "algorithm": "sha256",
        "digest": "b".repeat(64)
    });
    let literal = json!({"term":"literal","type":node,"value_kind":"integer","value":"1"});
    let reference = json!({"term":"reference","target":node});
    let dependency = json!({"term":"dependency_reference","package":package,"node":node});
    let binding = json!({"term":"binding","name":"x","value":literal});
    let aggregate = json!({"term":"aggregate","members":[binding,reference,dependency]});
    let application = json!({
        "term":"application","operator":"binary",
        "operation":{"identity":"quire.op.test","laws":[],"mode":null,"member":null,"leaves":[]},
        "result_type":node,"arguments":[aggregate]
    });
    let frame = json!({
        "term":"frame","modifies":[],"creates":[],"deletes":[],"deletes_scoped":[]
    });
    let abstraction = json!({
        "term":"abstraction_relation","objects":[],"populations":[],"frames":[]
    });
    let extension = json!({
        "term":"extension_form",
        "definition":{"authority":"test","identity":"extension"},
        "node_kind":"retention","clauses":[]
    });

    for body in [
        literal,
        reference,
        dependency,
        binding,
        aggregate,
        application,
        frame,
        abstraction,
        extension,
    ] {
        let term: CheckedTerm = serde_json::from_value(body.clone()).expect("closed term");
        assert_eq!(serde_json::to_value(term).expect("term serializes"), body);
    }
    assert!(
        serde_json::from_value::<CheckedTerm>(json!({"term":"group_reference","ordinal":0}))
            .is_err(),
        "a preimage-only group reference cannot become a wire term"
    );
}
