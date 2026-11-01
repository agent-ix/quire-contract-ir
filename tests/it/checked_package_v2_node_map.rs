// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Public draft node-id rewriting follows FR-048 without admitting a package.

use crate::support::checked_package::{canonical, evidence_for, v2_all_families};
use ix_trace_rs::trace;
use quire_contract_model::{
    encode_checked_package_v2, CheckedArtifactRef, CheckedNodeId, CheckedNodePathSegment,
    CheckedPackageReadLimits, CheckedPackageSourceCorrespondenceRef, CheckedPackageV2,
    CheckedPackageV2ReadResult, CheckedSemanticNodeV2, CheckedTerm, DimensionTerm, LiteralKind,
    NominalBody, RefinementKey,
};

fn id(name: &str) -> CheckedNodeId {
    CheckedNodeId {
        package: "node-map".into(),
        path: vec![CheckedNodePathSegment::Name(name.into())],
        ordinal: 0,
    }
}

#[trace("FR-048-AC-6", "FR-048-AC-7", "FR-048-AC-9")]
#[test]
fn public_node_and_term_mapping_visits_original_slots_once_in_order() {
    let a = id("a");
    let b = id("b");
    let c = id("c");
    let literal_data = serde_json::to_value(&a).expect("node id JSON");
    let mut node = CheckedSemanticNodeV2 {
        node_id: a.clone(),
        schema_version: "quire.checked-semantic-graph/v2".into(),
        node_tag: "expression".into(),
        semantic_form: "test".into(),
        semantic_type: b.clone(),
        dependencies: vec![a.clone(), b.clone()],
        occurrences: Vec::new(),
        recursion_group: Some("a".into()),
        declaration: None,
        owner: None,
        owner_model: Some(a.clone()),
        body: CheckedTerm::aggregate(vec![
            CheckedTerm::reference(a.clone()),
            CheckedTerm::binding("member", CheckedTerm::reference(b.clone())),
            CheckedTerm::Literal {
                type_id: a.clone(),
                value_kind: LiteralKind::String,
                value: literal_data.clone(),
            },
        ]),
    };
    let mut observed = Vec::new();
    node.map_node_ids(|original| {
        observed.push(original.clone());
        if original == &a {
            Some(b.clone())
        } else if original == &b {
            Some(c.clone())
        } else {
            None
        }
    });
    assert_eq!(
        observed,
        vec![
            a.clone(),
            b.clone(),
            a.clone(),
            b.clone(),
            a.clone(),
            a.clone(),
            b.clone(),
            a.clone()
        ]
    );
    assert_eq!(node.node_id, b);
    assert_eq!(node.semantic_type, c);
    assert_eq!(node.dependencies[0], node.node_id);
    assert_eq!(node.owner_model.as_ref(), Some(&node.node_id));
    let CheckedTerm::Aggregate { members } = &node.body else {
        panic!("aggregate body changed form");
    };
    assert!(matches!(&members[0], CheckedTerm::Reference { target } if target == &node.node_id));
    assert!(matches!(&members[1], CheckedTerm::Binding { value, .. }
        if matches!(value.as_ref(), CheckedTerm::Reference { target } if target == &node.semantic_type)));
    assert!(
        matches!(&members[2], CheckedTerm::Literal { type_id, value, .. }
        if type_id == &node.node_id && value == &literal_data)
    );
    assert_eq!(node.recursion_group.as_deref(), Some("a"));

    let mut independent = CheckedTerm::aggregate(vec![
        CheckedTerm::reference(a.clone()),
        CheckedTerm::reference(a.clone()),
    ]);
    let mut next = 1;
    independent.map_node_ids(|original| {
        assert_eq!(original, &a);
        let replacement = CheckedNodeId {
            package: a.package.clone(),
            path: a.path.clone(),
            ordinal: next,
        };
        next += 1;
        Some(replacement)
    });
    assert_eq!(next, 3);
    let CheckedTerm::Aggregate { members } = independent else {
        panic!("aggregate changed form");
    };
    assert!(matches!(&members[0], CheckedTerm::Reference { target } if target.ordinal == 1));
    assert!(matches!(&members[1], CheckedTerm::Reference { target } if target.ordinal == 2));
}

#[trace("FR-048-AC-7")]
#[test]
fn frame_and_abstraction_map_only_typed_ids() {
    let original = id("original");
    let replacement = id("replacement");
    let mut frame = CheckedTerm::Frame {
        modifies: vec![
            quire_contract_model::CheckedFrameModification::Relationship {
                declaration: original.clone(),
            },
        ],
        creates: vec![original.clone()],
        deletes: vec![original.clone()],
        deletes_scoped: vec![quire_contract_model::CheckedScopedDeletion::Parameter {
            name: "original".into(),
        }],
    };
    let mut calls = 0;
    frame.map_node_ids(|id| {
        assert_eq!(id, &original);
        calls += 1;
        Some(replacement.clone())
    });
    assert_eq!(calls, 3);
    let CheckedTerm::Frame {
        modifies,
        creates,
        deletes,
        deletes_scoped,
    } = frame
    else {
        panic!("frame changed form");
    };
    assert!(
        matches!(&modifies[0], quire_contract_model::CheckedFrameModification::Relationship { declaration }
        if declaration == &replacement)
    );
    assert_eq!(creates, vec![replacement.clone()]);
    assert_eq!(deletes, vec![replacement]);
    assert_eq!(
        deletes_scoped,
        vec![quire_contract_model::CheckedScopedDeletion::Parameter {
            name: "original".into()
        }]
    );
}

#[trace("FR-048-AC-7")]
#[test]
fn nominal_and_refinement_keys_map_without_touching_source_labels() {
    let original = id("original");
    let replacement = id("replacement");
    let nominal = NominalBody::try_dimension(vec![DimensionTerm {
        dimension_node_id: original.clone(),
        exponent: "1".into(),
    }])
    .expect("valid dimension body");
    let mut term = CheckedTerm::NominalBody(Box::new(nominal));
    term.map_node_ids(|seen| {
        assert_eq!(seen, &original);
        Some(replacement.clone())
    });
    let encoded = serde_json::to_value(&term).expect("nominal wire");
    assert_eq!(
        encoded["terms"][0]["dimension_node_id"],
        serde_json::to_value(&replacement).expect("node ID wire")
    );
    assert_eq!(encoded["terms"][0]["exponent"], "1");

    let mut term = CheckedTerm::Refinement {
        abstract_key: Box::new(RefinementKey::Local(original.clone())),
        profile: CheckedArtifactRef {
            authority: "source".into(),
            identity: "original".into(),
        },
        populations: Vec::new(),
        objects: Vec::new(),
        steps: Vec::new(),
        histories: Vec::new(),
        assume: Vec::new(),
        ensure: Vec::new(),
        protocol: Some(Box::new(RefinementKey::Local(original.clone()))),
        abstract_internal: Some(vec!["original".into()]),
        class_catalog: None,
    };
    let mut calls = 0;
    term.map_node_ids(|seen| {
        assert_eq!(seen, &original);
        calls += 1;
        Some(replacement.clone())
    });
    assert_eq!(calls, 2);
    let encoded = serde_json::to_value(&term).expect("refinement wire");
    assert_eq!(
        encoded["abstract"],
        serde_json::to_value(&replacement).expect("node ID wire")
    );
    assert_eq!(
        encoded["protocol"],
        serde_json::to_value(&replacement).expect("node ID wire")
    );
    assert_eq!(encoded["profile"]["identity"], "original");
    assert_eq!(encoded["abstract_internal"][0], "original");
}

#[trace("FR-048-AC-8")]
#[test]
fn nested_term_mapping_uses_a_constant_call_stack() {
    let original = id("deep");
    let mut term = CheckedTerm::reference(original.clone());
    for _ in 0..100_000 {
        term = CheckedTerm::binding("layer", term);
    }
    let mut calls = 0;
    term.map_node_ids(|seen| {
        assert_eq!(seen, &original);
        calls += 1;
        None
    });
    assert_eq!(calls, 1);
    // Recursive `Drop` for the preexisting boxed term shape is independent of
    // the iterative mapper and would overflow the test thread's call stack.
    std::mem::forget(term);
}

#[trace("FR-048-AC-8")]
#[test]
fn mapped_duplicate_is_refused_by_the_package_reader() {
    let fixture = v2_all_families();
    let admitted = match CheckedPackageV2::read(
        &canonical(&fixture),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&fixture),
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("positive fixture refused: {other:?}"),
    };
    let mut graph = admitted.graph().clone();
    let duplicate = graph.nodes[1].node_id.clone();
    let original = graph.nodes[0].node_id.clone();
    let mut visits = 0;
    graph.nodes[0].map_node_ids(|id| {
        visits += 1;
        (id == &original).then(|| duplicate.clone())
    });
    assert!(visits > 1, "the node mapper visits typed body slots too");
    assert_eq!(graph.nodes[0].node_id, graph.nodes[1].node_id);
    let bytes = encode_checked_package_v2(
        admitted.name(),
        admitted.lock(),
        &graph,
        CheckedPackageSourceCorrespondenceRef {
            source_map: admitted.source_map(),
            refinement_step_rows: admitted.refinement_step_rows(),
        },
        admitted.capability_report(),
        admitted.diagnostics(),
        quire_canonical::Limits::new(u64::MAX),
    )
    .expect("draft package encodes");
    assert!(matches!(
        CheckedPackageV2::read(
            &bytes,
            CheckedPackageReadLimits::bounded(),
            &evidence_for(&fixture),
        ),
        CheckedPackageV2ReadResult::Refused(_)
    ));
}
