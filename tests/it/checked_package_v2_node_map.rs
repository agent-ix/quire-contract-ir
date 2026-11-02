// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Public draft node-id rewriting follows FR-048 without admitting a package.

use crate::support::checked_package::{
    canonical, evidence_for, v2_all_families, v2_reference_chain,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    encode_checked_package_v2, CheckedAbstractionFrame, CheckedAbstractionObject,
    CheckedAbstractionPopulation, CheckedArtifactRef, CheckedDimensionOperand,
    CheckedFairnessTarget, CheckedNodeId, CheckedNodePathSegment, CheckedOperationMember,
    CheckedPackageEvidence, CheckedPackageReadLimits, CheckedPackageRefusalCode,
    CheckedPackageSourceCorrespondenceRef, CheckedPackageV2, CheckedPackageV2ReadResult,
    CheckedRational, CheckedSemanticNodeV2, CheckedTerm, DimensionTerm, LiteralKind, NominalBody,
    RefinementAbstractClass, RefinementAssume, RefinementClassCatalog, RefinementConcreteClass,
    RefinementEnsure, RefinementFairnessKind, RefinementField, RefinementGranularity,
    RefinementHistory, RefinementHistoryUpdate, RefinementKey, RefinementObject,
    RefinementPackageNodeKey, RefinementPopulation, RefinementStep, RefinementStepArgument,
    RefinementStepTarget,
};
use serde_json::Value;
use std::sync::Arc;

fn id(name: &str) -> CheckedNodeId {
    CheckedNodeId {
        package: "node-map".into(),
        path: vec![CheckedNodePathSegment::Name(name.into())],
        ordinal: 0,
    }
}

fn wire_node_ids<'a>(value: &'a Value, found: &mut Vec<&'a Value>) {
    match value {
        Value::Object(object)
            if object.len() == 3
                && object.contains_key("package")
                && object.contains_key("path")
                && object.contains_key("ordinal") =>
        {
            found.push(value);
        }
        Value::Object(object) => {
            for value in object.values() {
                wire_node_ids(value, found);
            }
        }
        Value::Array(values) => {
            for value in values {
                wire_node_ids(value, found);
            }
        }
        _ => {}
    }
}

fn assert_wire_ids(value: &Value, expected: &CheckedNodeId, count: usize) {
    let mut found = Vec::new();
    wire_node_ids(value, &mut found);
    assert_eq!(
        found.len(),
        count,
        "every typed identity slot is serialized"
    );
    let expected = serde_json::to_value(expected).expect("node ID wire");
    assert!(found.into_iter().all(|id| id == &expected));
}

fn reverse_chain_id(id: &CheckedNodeId, length: usize) -> Option<CheckedNodeId> {
    let [CheckedNodePathSegment::Name(label)] = id.path.as_slice() else {
        return None;
    };
    let index = label
        .strip_prefix("expression/reference/")?
        .parse::<usize>()
        .ok()?;
    (index < length).then(|| CheckedNodeId {
        package: id.package.clone(),
        path: vec![CheckedNodePathSegment::Name(
            format!("expression/reference/{}", length - 1 - index).into(),
        )],
        ordinal: id.ordinal,
    })
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

#[trace("FR-048-AC-6", "FR-048-AC-7")]
#[test]
fn admitted_operation_member_maps_whole_nested_id_and_typed_mirror() {
    let package = crate::checked_package_v2_catalogue_differential::admitted_catalogue_control(
        "quire.op.collection.index",
    );
    let mut term = package
        .graph()
        .nodes
        .iter()
        .find_map(|node| match &node.body {
            CheckedTerm::Application { operation, .. }
                if operation.identity() == Some("quire.op.collection.index") =>
            {
                Some(node.body.clone())
            }
            _ => None,
        })
        .expect("admitted index operation");
    let CheckedTerm::Application {
        operation,
        result_type,
        ..
    } = &term
    else {
        unreachable!()
    };
    let original_result_type = result_type.clone();
    let Some(CheckedOperationMember::Element { declaration }) =
        operation.read().expect("admitted view").member
    else {
        panic!("index operation has an element member");
    };
    let original = declaration.clone();
    let nested = CheckedNodeId {
        package: original.package.clone(),
        path: vec![
            CheckedNodePathSegment::Name("nested-member".into()),
            CheckedNodePathSegment::Node(Arc::new(original.clone())),
        ],
        ordinal: 1,
    };
    let final_id = id("final-member");
    let mut first_visits = Vec::new();
    let mut original_visits = 0;
    term.map_node_ids(|seen| {
        first_visits.push(seen.clone());
        if seen == &original {
            original_visits += 1;
            (original_visits == 1).then(|| nested.clone())
        } else if seen == &nested {
            Some(final_id.clone())
        } else {
            None
        }
    });
    assert!(
        !first_visits.contains(&nested),
        "replacement is not revisited"
    );
    let CheckedTerm::Application {
        operation,
        result_type,
        ..
    } = &term
    else {
        unreachable!()
    };
    assert!(
        matches!(operation.read().expect("first mapped view").member,
        Some(CheckedOperationMember::Element { declaration }) if declaration == &nested)
    );
    assert_eq!(
        result_type, &original_result_type,
        "result slot remained independent"
    );
    assert_eq!(
        serde_json::to_value(operation).unwrap()["member"]["declaration"],
        serde_json::to_value(&nested).unwrap()
    );
    let mut nested_visits = 0;
    term.map_node_ids(|seen| {
        if seen == &nested {
            nested_visits += 1;
            Some(final_id.clone())
        } else {
            None
        }
    });
    assert_eq!(nested_visits, 1, "whole member ID is one logical slot");
    let CheckedTerm::Application { operation, .. } = &term else {
        unreachable!()
    };
    assert!(matches!(operation.read().expect("mapped view").member,
        Some(CheckedOperationMember::Element { declaration }) if declaration == &final_id));
    let raw = serde_json::to_value(operation).expect("mapped raw operation");
    assert_eq!(
        raw["member"]["declaration"],
        serde_json::to_value(&final_id).unwrap()
    );
}

fn admitted_with_evidence(mut wire: Value, evidence: CheckedPackageEvidence) -> CheckedPackageV2 {
    crate::support::checked_package::index_fixture_node_ids(&mut wire);
    match CheckedPackageV2::read(
        &canonical(&wire),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("member control must admit: {other:?}"),
    }
}

fn member_slots(member: &CheckedOperationMember) -> Vec<(&'static str, CheckedNodeId)> {
    match member {
        CheckedOperationMember::Field { declaration, .. }
        | CheckedOperationMember::Position { declaration, .. }
        | CheckedOperationMember::Element { declaration }
        | CheckedOperationMember::Operation { declaration, .. }
        | CheckedOperationMember::TypeArgument { declaration } => {
            vec![("/declaration", declaration.clone())]
        }
        CheckedOperationMember::RelationshipEnd {
            declaration,
            destination_type,
            population,
            ..
        } => {
            let mut slots = vec![
                ("/declaration", declaration.clone()),
                ("/destination_type", destination_type.clone()),
            ];
            if let Some(population) = population {
                slots.push(("/population", population.clone()));
            }
            slots
        }
        CheckedOperationMember::Fairness { target, .. } => match target {
            CheckedFairnessTarget::Operation { declaration, .. }
            | CheckedFairnessTarget::HybridTransition { declaration, .. } => {
                vec![("/target/declaration", declaration.clone())]
            }
            CheckedFairnessTarget::ProtocolRoot { node }
            | CheckedFairnessTarget::AttemptNode { node }
            | CheckedFairnessTarget::CompensationTemplate { node } => {
                vec![("/target/node", node.clone())]
            }
            CheckedFairnessTarget::ParallelBranch { parallel, .. } => {
                vec![("/target/parallel", parallel.clone())]
            }
        },
        CheckedOperationMember::ProfileOperator { .. }
        | CheckedOperationMember::StateClause { .. }
        | CheckedOperationMember::TemporalInterval { .. } => Vec::new(),
    }
}

fn assert_admitted_member_census_case(
    package: CheckedPackageV2,
    identity: &str,
    kind: &str,
    target_kind: Option<&str>,
    expected_slots: usize,
) {
    let mut term = package
        .graph()
        .nodes
        .iter()
        .find_map(|node| match &node.body {
            CheckedTerm::Application { operation, .. }
                if operation.identity() == Some(identity) =>
            {
                Some(node.body.clone())
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("admitted {identity} application"));
    let CheckedTerm::Application { operation, .. } = &term else {
        unreachable!()
    };
    let read = operation.read().expect("reader-admitted operation");
    let member = read.member.expect("reader-admitted member");
    let original = member_slots(member);
    assert_eq!(original.len(), expected_slots, "{identity}");
    let raw = serde_json::to_value(operation).expect("raw operation");
    assert_eq!(raw["member"]["kind"], kind, "{identity}");
    if let Some(target_kind) = target_kind {
        assert_eq!(raw["member"]["target"]["kind"], target_kind, "{identity}");
    }
    let mut expected_member = raw["member"].clone();
    let nested: Vec<_> = original
        .iter()
        .enumerate()
        .map(|(index, (_, source))| CheckedNodeId {
            package: source.package.clone(),
            path: vec![
                CheckedNodePathSegment::Name(format!("nested-{index}").into()),
                CheckedNodePathSegment::Node(Arc::new(source.clone())),
            ],
            ordinal: 0,
        })
        .collect();
    for ((path, _), mapped) in original.iter().zip(&nested) {
        *expected_member.pointer_mut(path).expect("member slot") =
            serde_json::to_value(mapped).expect("nested ID wire");
    }
    let mut calls = 0;
    term.map_node_ids(|seen| {
        if calls < original.len() {
            assert_eq!(seen, &original[calls].1, "member before result: {identity}");
            let mapped = nested[calls].clone();
            calls += 1;
            Some(mapped)
        } else {
            assert!(!nested.contains(seen), "replacement revisited: {identity}");
            None
        }
    });
    assert_eq!(calls, expected_slots, "{identity}");
    let CheckedTerm::Application { operation, .. } = &term else {
        unreachable!()
    };
    assert_eq!(
        serde_json::to_value(operation).unwrap()["member"],
        expected_member,
        "{identity}"
    );
    assert_eq!(
        member_slots(operation.read().unwrap().member.unwrap()),
        original
            .iter()
            .map(|(path, _)| *path)
            .zip(nested.iter().cloned())
            .collect::<Vec<_>>(),
        "typed nested mirror: {identity}"
    );
    let final_ids: Vec<_> = (0..nested.len())
        .map(|index| id(&format!("final-{index}")))
        .collect();
    let mut second_calls = 0;
    term.map_node_ids(|seen| {
        if second_calls < nested.len() {
            assert_eq!(seen, &nested[second_calls], "whole nested slot: {identity}");
            let mapped = final_ids[second_calls].clone();
            second_calls += 1;
            Some(mapped)
        } else {
            None
        }
    });
    assert_eq!(second_calls, expected_slots, "{identity}");
    let CheckedTerm::Application { operation, .. } = &term else {
        unreachable!()
    };
    for ((path, _), final_id) in original.iter().zip(&final_ids) {
        assert_eq!(
            serde_json::to_value(operation).unwrap()["member"]
                .pointer(path)
                .unwrap(),
            &serde_json::to_value(final_id).unwrap(),
            "{identity} {path}"
        );
    }
    assert_eq!(
        member_slots(operation.read().unwrap().member.unwrap()),
        original
            .iter()
            .map(|(path, _)| *path)
            .zip(final_ids)
            .collect::<Vec<_>>(),
        "typed final mirror: {identity}"
    );
}

#[trace("FR-048-AC-7")]
#[test]
fn external_admitted_operation_member_census_maps_structural_ids() {
    let (field_wire, field_evidence) =
        crate::checked_package_v2_model_fields::record_project_control();
    let (relationship_wire, relationship_evidence) =
        crate::checked_package_v2_model_members::model_member_control("quire.op.model.navigate");
    let (operation_wire, operation_evidence) =
        crate::checked_package_v2_model_members::model_member_control(
            "quire.op.model.dispatch_call",
        );
    for (package, identity, kind, target_kind, slots) in [
        (
            admitted_with_evidence(field_wire, field_evidence),
            "quire.op.record.project",
            "field",
            None,
            1,
        ),
        (
            crate::checked_package_v2_catalogue_differential::admitted_catalogue_control(
                "quire.op.tuple.project",
            ),
            "quire.op.tuple.project",
            "position",
            None,
            1,
        ),
        (
            crate::checked_package_v2_catalogue_differential::admitted_catalogue_control(
                "quire.op.collection.index",
            ),
            "quire.op.collection.index",
            "element",
            None,
            1,
        ),
        (
            admitted_with_evidence(relationship_wire, relationship_evidence),
            "quire.op.model.navigate",
            "relationship_end",
            None,
            2,
        ),
        (
            crate::checked_package_v2_model_members::admitted_reaches_population_for_node_map(),
            "quire.op.model.reaches",
            "relationship_end",
            None,
            3,
        ),
        (
            admitted_with_evidence(operation_wire, operation_evidence),
            "quire.op.model.dispatch_call",
            "operation",
            None,
            1,
        ),
        (
            crate::checked_package_v2_catalogue_differential::admitted_catalogue_control(
                "quire.op.numeric.convert",
            ),
            "quire.op.numeric.convert",
            "type_argument",
            None,
            1,
        ),
        (
            crate::checked_package_v2_temporal::admitted_fairness_source_for_contract(),
            "quire.op.temporal.fair",
            "fairness",
            Some("operation"),
            1,
        ),
        (
            crate::checked_package_v2_protocol_body::admitted_protocol_fairness_for_node_map(
                "protocol_root",
            ),
            "quire.op.temporal.fair",
            "fairness",
            Some("protocol_root"),
            1,
        ),
        (
            crate::checked_package_v2_protocol_body::admitted_protocol_fairness_for_node_map(
                "attempt_node",
            ),
            "quire.op.temporal.fair",
            "fairness",
            Some("attempt_node"),
            1,
        ),
        (
            crate::checked_package_v2_protocol_body::admitted_protocol_fairness_for_node_map(
                "parallel_branch",
            ),
            "quire.op.temporal.fair",
            "fairness",
            Some("parallel_branch"),
            1,
        ),
        (
            crate::checked_package_v2_protocol_body::admitted_protocol_fairness_for_node_map(
                "compensation_template",
            ),
            "quire.op.temporal.fair",
            "fairness",
            Some("compensation_template"),
            1,
        ),
    ] {
        assert_admitted_member_census_case(package, identity, kind, target_kind, slots);
    }
}

#[trace("FR-048-AC-7")]
#[test]
fn frame_maps_only_typed_ids() {
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

#[trace("FR-048-AC-7")]
#[test]
fn remaining_closed_forms_map_every_typed_identity_slot() {
    let original = id("original");
    let replacement = id("replacement");
    let ratio = CheckedRational {
        numerator: "1".into(),
        denominator: "1".into(),
    };
    let mut forms = [
        (
            CheckedTerm::NominalBody(Box::new(
                NominalBody::try_unit(
                    id("dimension"),
                    Some(id("target")),
                    ratio.clone(),
                    ratio.clone(),
                )
                .expect("valid unit"),
            )),
            2,
        ),
        (
            CheckedTerm::NominalBody(Box::new(
                NominalBody::try_dimension_equivalence(
                    [
                        CheckedDimensionOperand::Local(id("dimension-a")),
                        CheckedDimensionOperand::Local(id("dimension-b")),
                    ],
                    ratio,
                )
                .expect("valid equivalence"),
            )),
            2,
        ),
        (
            CheckedTerm::AbstractionRelation {
                objects: vec![CheckedAbstractionObject {
                    type_id: original.clone(),
                    type_path: vec!["original".into()],
                    fields: Vec::new(),
                }],
                populations: vec![CheckedAbstractionPopulation {
                    population: original.clone(),
                    collection: vec!["original".into()],
                }],
                frames: vec![CheckedAbstractionFrame {
                    context: original.clone(),
                    operation: "original".into(),
                    function: vec!["original".into()],
                    receiver: "original".into(),
                    parameters: Vec::new(),
                }],
            },
            3,
        ),
        (
            CheckedTerm::Refinement {
                abstract_key: Box::new(RefinementKey::Local(original.clone())),
                profile: CheckedArtifactRef {
                    authority: "source".into(),
                    identity: "original".into(),
                },
                populations: vec![RefinementPopulation {
                    r#abstract: RefinementKey::Local(original.clone()),
                    concrete: original.clone(),
                }],
                objects: vec![RefinementObject {
                    abstract_type: RefinementKey::Local(original.clone()),
                    concrete_type: original.clone(),
                    fields: vec![RefinementField {
                        name: "original".into(),
                        expression: CheckedTerm::reference(original.clone()),
                    }],
                }],
                steps: vec![RefinementStep {
                    source_class: "original".into(),
                    source_node: Some(original.clone()),
                    target: RefinementStepTarget::Operation {
                        operation: RefinementKey::Local(original.clone()),
                        receiver: Box::new(CheckedTerm::reference(original.clone())),
                        arguments: vec![RefinementStepArgument::Term(Box::new(
                            CheckedTerm::reference(original.clone()),
                        ))],
                    },
                }],
                histories: vec![RefinementHistory {
                    concrete_type: original.clone(),
                    name: "original".into(),
                    type_id: original.clone(),
                    initial: CheckedTerm::reference(original.clone()),
                    updates: vec![RefinementHistoryUpdate {
                        operation: original.clone(),
                        expression: CheckedTerm::reference(original.clone()),
                    }],
                }],
                assume: vec![RefinementAssume {
                    operation: original.clone(),
                    kind: RefinementFairnessKind::Weak,
                    granularity: RefinementGranularity::Whole,
                }],
                ensure: vec![RefinementEnsure {
                    operation: RefinementKey::Local(original.clone()),
                    kind: RefinementFairnessKind::Strong,
                    granularity: RefinementGranularity::Each,
                }],
                protocol: Some(Box::new(RefinementKey::External(
                    RefinementPackageNodeKey {
                        package: "source".into(),
                        node: original.clone(),
                    },
                ))),
                abstract_internal: Some(vec!["original".into()]),
                class_catalog: Some(RefinementClassCatalog {
                    concrete: vec![RefinementConcreteClass::Declared {
                        class: "original".into(),
                        node: original.clone(),
                        binder_types: vec![original.clone()],
                    }],
                    r#abstract: vec![RefinementAbstractClass::Declared {
                        class: "original".into(),
                        node: RefinementKey::Local(original.clone()),
                        binder_types: vec![RefinementKey::Local(original.clone())],
                    }],
                }),
            },
            22,
        ),
    ];
    for (index, (term, expected_count)) in forms.iter_mut().enumerate() {
        let mut visits = 0;
        term.map_node_ids(|_| {
            visits += 1;
            Some(replacement.clone())
        });
        assert_eq!(visits, *expected_count);
        let wire = serde_json::to_value(&term).expect("closed term wire");
        assert_wire_ids(&wire, &replacement, *expected_count);
        if index >= 2 {
            assert!(
                wire.to_string().contains("original"),
                "source labels survive"
            );
        }
    }
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
fn hundred_thousand_node_position_remap_preserves_admitted_references() {
    const LENGTH: usize = 100_000;
    let base_fixture = v2_all_families();
    let evidence = evidence_for(&base_fixture);
    let base_count = base_fixture["semantic_graph"]["nodes"]
        .as_array()
        .expect("base nodes")
        .len();
    let (bytes, _) = v2_reference_chain(LENGTH);
    let limits = CheckedPackageReadLimits {
        bytes: 1 << 36,
        nodes: 200_000,
        edges: 1_600_000,
        occurrences: 1_600_000,
        diagnostics: 1 << 20,
        work: 100_000_000,
    };
    let original = match CheckedPackageV2::read(&bytes, limits, &evidence) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("valid chain refused: {other:?}"),
    };
    assert_eq!(original.graph().nodes.len(), base_count + LENGTH);
    let mut graph = original.graph().clone();
    let mut base_visits = 0;
    for node in &mut graph.nodes[..base_count] {
        node.map_node_ids(|_| {
            base_visits += 1;
            None
        });
    }
    let mut visits = base_visits;
    let mut chain_id_visits = 0;
    for node in &mut graph.nodes[base_count..] {
        node.map_node_ids(|id| {
            visits += 1;
            let replacement = reverse_chain_id(id, LENGTH);
            chain_id_visits += usize::from(replacement.is_some());
            replacement
        });
    }
    assert_eq!(visits, base_visits + LENGTH * 4);
    assert_eq!(chain_id_visits, LENGTH + (LENGTH - 1) * 2);
    let mut source_map = original.source_map().to_vec();
    for row in &mut source_map {
        if let Some(replacement) = reverse_chain_id(&row.node_id, LENGTH) {
            row.node_id = replacement;
        }
    }
    let encoded = encode_checked_package_v2(
        original.name(),
        original.lock(),
        &graph,
        CheckedPackageSourceCorrespondenceRef {
            source_map: &source_map,
            refinement_step_rows: original.refinement_step_rows(),
        },
        quire_contract_model::CheckedPackageSidecarsRef {
            checked_term_data: original.checked_term_data(),
            temporal_bindings: original.temporal_bindings(),
            schedulability_claims: None,
        },
        original.capability_report(),
        original.diagnostics(),
        quire_canonical::Limits::new(u64::MAX),
    )
    .expect("rewritten draft encodes");
    let rewritten = match CheckedPackageV2::read(&encoded, limits, &evidence) {
        CheckedPackageV2ReadResult::Admitted(package) => package,
        other => panic!("valid remap refused: {other:?}"),
    };
    for position in 1..LENGTH {
        let previous = &rewritten.graph().nodes[base_count + position - 1].node_id;
        let node = &rewritten.graph().nodes[base_count + position];
        assert_eq!(node.dependencies.len(), 1);
        assert_eq!(node.dependencies.first(), Some(previous));
        assert!(matches!(&node.body, CheckedTerm::Reference { target } if target == previous));
    }
    for position in [0, LENGTH / 2, LENGTH - 1] {
        let old = &original.graph().nodes[base_count + position].node_id;
        let expected = reverse_chain_id(old, LENGTH).expect("chain node");
        assert_eq!(
            rewritten.graph().nodes[base_count + position].node_id,
            expected
        );
    }
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
        quire_contract_model::CheckedPackageSidecarsRef {
            checked_term_data: admitted.checked_term_data(),
            temporal_bindings: admitted.temporal_bindings(),
            schedulability_claims: None,
        },
        admitted.capability_report(),
        admitted.diagnostics(),
        quire_canonical::Limits::new(u64::MAX),
    )
    .expect("draft package encodes");
    let CheckedPackageV2ReadResult::Refused(refusal) = CheckedPackageV2::read(
        &bytes,
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&fixture),
    ) else {
        panic!("a duplicate mapped ID must refuse admission");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::InvalidSemanticGraph
    );
    assert_eq!(
        refusal.loci[0].path.as_ref().map(ToString::to_string),
        Some("/semantic_graph/nodes/1/node_id".to_owned())
    );
}
