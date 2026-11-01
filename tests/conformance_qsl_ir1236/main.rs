// SPDX-License-Identifier: AGPL-3.0-or-later
//! Explicit live-producer lane for the complete admitted operation census.
//! IR1236_QSL_PACKAGE and IR1236_QSL_MODEL name exact freshly emitted QSL bytes
//! and their selected Semantic IR model. The ordinary integration target has
//! no external producer inputs; this target must be invoked explicitly.

use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageV2, CheckedPackageV2ReadResult,
};
const IDENTITY: &str = "acme/orders";
const OBJECT: &str = "object_type";

#[trace("TC-464", "FR-355-AC-1", "FR-355-AC-5")]
#[test]
fn qsl_emitted_package_keeps_the_unreferenced_operation_in_the_census() {
    let package =
        std::fs::read(std::env::var_os("IR1236_QSL_PACKAGE").expect("live QSL output path"))
            .expect("read exact producer bytes");
    let model = std::fs::read(std::env::var_os("IR1236_QSL_MODEL").expect("selected model path"))
        .expect("read exact selected document");
    let mut evidence = CheckedPackageEvidence::new();
    evidence.support_feature("quire.value.complete/v1");
    evidence
        .insert_domain_package_document(IDENTITY, model)
        .unwrap();
    let admitted =
        match CheckedPackageV2::read(&package, CheckedPackageReadLimits::bounded(), &evidence) {
            CheckedPackageV2ReadResult::Admitted(package) => package,
            other => panic!("strict reader must admit the live producer output: {other:?}"),
        };
    let context = admitted
        .graph()
        .nodes
        .iter()
        .find(|node| node.semantic_form.as_ref() == OBJECT)
        .expect("QSL selected Node's model declaration");
    let census = admitted.model_operations(&context.node_id, 10_000).unwrap();
    assert_eq!(
        census
            .operations()
            .iter()
            .map(|op| op.name())
            .collect::<Vec<_>>(),
        ["alpha", "zeta"]
    );
    assert_eq!(
        admitted
            .node_kinds()
            .iter()
            .filter(|kind| matches!(
                kind,
                quire_contract_model::CheckedNodeKind::State(
                    quire_contract_model::StateForm::OperationAnchor
                )
            ))
            .count(),
        1,
        "the source anchors alpha only; zeta remains in the model census"
    );
}
