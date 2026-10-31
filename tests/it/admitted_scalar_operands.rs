// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-458: a public consumer reads ordered operands from an admitted V1 package.

use crate::admitted_result_context::{
    admit, by_operation, fixture, package, package_with_profile, profile, sources_for,
};
use crate::support::checked_package::{family_key, node_id, refresh_identity, settle};
use ix_trace_rs::trace;
use quire_contract_model::{
    BoundedDomainForm, CheckedOccurrenceRole, CheckedScalarOperandError, ContractOccurrenceKeyV1,
    ContractRootViewError, ContractScalarOperandChildV1, ContractScalarOperandErrorV1,
    ContractScalarOperandViewV1, ContractScalarOwnRangeV1,
};
use serde_json::{json, Value};

fn revised(mut wire: Value, change: impl FnOnce(&mut Value)) -> Value {
    change(&mut wire);
    settle(&mut wire);
    wire
}

fn add_body_mut(wire: &mut Value) -> &mut Value {
    &mut wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("fixture graph")
        .iter_mut()
        .find(|node| node["body"]["operation"]["identity"] == "quire.op.integer.add")
        .expect("add node")["body"]
}

#[trace("TC-458", "FR-054-AC-1", "FR-054-AC-5")]
#[test]
fn tc_458_admitted_add_retains_graph_child_then_inline_one() {
    let checked = admit(&fixture());
    let add = by_operation(&checked, "quire.op.integer.add");
    let admitted = package(&checked, std::slice::from_ref(&add.node_id));
    let before = admitted.wire().clone();
    let reached = admitted
        .reached_closure(
            &quire_contract_model::ContractSourceKeyV1::from_structural(&add.node_id)
                .expect("structural source key"),
        )
        .expect("reached before");
    let result = admitted
        .result_context(&add.node_id, &add, None, BoundedDomainForm::IntegerRange)
        .expect("result before");

    let ContractScalarOperandViewV1::Lowered(view) = admitted
        .scalar_operands(&add.node_id, &add)
        .expect("admitted scalar operand view")
    else {
        panic!("the requested add is lowered");
    };
    assert_eq!(view.application, add);
    assert_eq!(
        view.application_source,
        &sources_for(&admitted, &add.node_id)[0]
    );
    assert_eq!(view.provenance, &admitted.wire().package);
    assert_eq!(view.arguments.len(), 2);
    assert_eq!(
        (view.arguments[0].ordinal, view.arguments[1].ordinal),
        (0, 1)
    );
    let ContractScalarOperandChildV1::GraphReference { node_id, sources } =
        &view.arguments[0].child
    else {
        panic!("first argument must be a represented graph child");
    };
    assert_eq!(*sources, sources_for(&admitted, node_id));
    assert!(matches!(
        view.arguments[0].own_range,
        ContractScalarOwnRangeV1::Finite { lower: 0, upper: 9 }
    ));
    let ContractScalarOperandChildV1::InlineInteger { value, occurrence } =
        &view.arguments[1].child
    else {
        panic!("second argument must be inline, not a fabricated graph node");
    };
    assert_eq!(value.to_string(), "1");
    assert_eq!(*occurrence, add);
    assert!(matches!(
        view.arguments[1].own_range,
        ContractScalarOwnRangeV1::Finite { lower: 1, upper: 1 }
    ));
    assert_eq!(admitted.wire(), &before);
    assert_eq!(
        admitted
            .reached_closure(
                &quire_contract_model::ContractSourceKeyV1::from_structural(&add.node_id)
                    .expect("structural source key")
            )
            .expect("reached after"),
        reached
    );
    assert_eq!(
        admitted
            .result_context(&add.node_id, &add, None, BoundedDomainForm::IntegerRange)
            .expect("result after"),
        result
    );
}

#[trace("TC-458", "FR-054-AC-2")]
#[test]
fn tc_458_repeated_graph_child_keeps_both_authored_positions() {
    let wire = revised(fixture(), |wire| {
        let arguments = add_body_mut(wire)["arguments"]
            .as_array_mut()
            .expect("arguments");
        arguments[1] = arguments[0].clone();
    });
    let checked = admit(&wire);
    let add = by_operation(&checked, "quire.op.integer.add");
    let admitted = package(&checked, std::slice::from_ref(&add.node_id));
    let ContractScalarOperandViewV1::Lowered(view) = admitted
        .scalar_operands(&add.node_id, &add)
        .expect("repeated child view")
    else {
        panic!("the add must be lowered");
    };
    assert_eq!(view.arguments.len(), 2);
    assert_eq!(
        (view.arguments[0].ordinal, view.arguments[1].ordinal),
        (0, 1)
    );
    let (
        ContractScalarOperandChildV1::GraphReference {
            node_id: first,
            sources: first_sources,
        },
        ContractScalarOperandChildV1::GraphReference {
            node_id: second,
            sources: second_sources,
        },
    ) = (&view.arguments[0].child, &view.arguments[1].child)
    else {
        panic!("both positions must be graph references");
    };
    assert_eq!(first, second);
    assert_eq!(first_sources, second_sources);
}

#[trace("TC-458", "FR-054-AC-3")]
#[test]
fn tc_458_absent_own_range_does_not_erase_positions_or_invent_a_fallback() {
    let unbounded = revised(fixture(), |wire| {
        let target = add_body_mut(wire)["arguments"][0]["target"].clone();
        let parameter = wire["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("fixture graph")
            .iter_mut()
            .find(|node| node["node_id"] == target)
            .expect("referenced parameter");
        parameter["semantic_type"] = node_id(&family_key("a3a3"));
    });
    let checked = admit(&unbounded);
    let add = by_operation(&checked, "quire.op.integer.add");
    let admitted = package(&checked, std::slice::from_ref(&add.node_id));
    let ContractScalarOperandViewV1::Lowered(view) = admitted
        .scalar_operands(&add.node_id, &add)
        .expect("unbounded child remains readable")
    else {
        panic!("the add must be lowered");
    };
    assert_eq!(view.arguments.len(), 2);
    assert_eq!(
        view.arguments[0].own_range,
        ContractScalarOwnRangeV1::Unbounded
    );
    assert!(matches!(
        view.arguments[0].child,
        ContractScalarOperandChildV1::GraphReference { .. }
    ));
    assert_eq!(
        view.arguments[1].own_range,
        ContractScalarOwnRangeV1::Finite { lower: 1, upper: 1 }
    );

    let huge = "170141183460469231731687303715884105728";
    let outside = revised(fixture(), |wire| {
        add_body_mut(wire)["arguments"][1]["value"] = Value::String(huge.into());
    });
    let checked = admit(&outside);
    let add = by_operation(&checked, "quire.op.integer.add");
    let admitted = package(&checked, std::slice::from_ref(&add.node_id));
    let ContractScalarOperandViewV1::Lowered(view) = admitted
        .scalar_operands(&add.node_id, &add)
        .expect("outside-native literal remains readable")
    else {
        panic!("the add must be lowered");
    };
    assert_eq!(view.arguments.len(), 2);
    assert_eq!(
        view.arguments[1].own_range,
        ContractScalarOwnRangeV1::OutsideNative
    );
    let ContractScalarOperandChildV1::InlineInteger { value, .. } = &view.arguments[1].child else {
        panic!("large integer remains an inline literal");
    };
    assert_eq!(value.to_string(), huge);
}

#[trace("TC-458", "FR-054-AC-4", "FR-054-AC-5")]
#[test]
fn tc_458_invalid_occurrence_refuses_without_mutating_admitted_views() {
    let checked = admit(&fixture());
    let add = by_operation(&checked, "quire.op.integer.add");
    let narrow = by_operation(&checked, "quire.op.numeric.narrow");
    let admitted = package(&checked, std::slice::from_ref(&add.node_id));
    let before = admitted.wire().clone();
    let before_reached = admitted
        .reached_closure(
            &quire_contract_model::ContractSourceKeyV1::from_structural(&add.node_id)
                .expect("structural source key"),
        )
        .expect("reached");
    for invalid in [
        ContractOccurrenceKeyV1 {
            node_id: add.node_id.clone(),
            role: CheckedOccurrenceRole::Type,
            ordinal: add.ordinal,
        },
        ContractOccurrenceKeyV1 {
            node_id: add.node_id.clone(),
            role: CheckedOccurrenceRole::Expression,
            ordinal: add.ordinal + 1,
        },
        narrow.clone(),
    ] {
        assert_eq!(
            admitted.scalar_operands(&add.node_id, &invalid),
            Err(ContractScalarOperandErrorV1::InvalidOccurrence)
        );
    }
    let absent = serde_json::from_value(node_id(&family_key("a1a1"))).expect("typed key");
    assert!(matches!(
        admitted.scalar_operands(&absent, &add),
        Ok(ContractScalarOperandViewV1::Absent)
    ));
    let with_narrow = package(&checked, &[add.node_id.clone(), narrow.node_id.clone()]);
    assert_eq!(
        with_narrow.scalar_operands(&narrow.node_id, &narrow),
        Err(ContractScalarOperandErrorV1::Projection(
            CheckedScalarOperandError::IneligibleOperator
        ))
    );
    let parameter = checked
        .graph()
        .nodes
        .iter()
        .find(|node| {
            node.node_tag.as_ref() == "value" && node.semantic_form.as_ref() == "parameter"
        })
        .expect("fixture parameter");
    let parameter_key = ContractOccurrenceKeyV1 {
        node_id: parameter.node_id.clone(),
        role: CheckedOccurrenceRole::Expression,
        ordinal: 0,
    };
    let with_parameter = package(&checked, std::slice::from_ref(&parameter.node_id));
    assert_eq!(
        with_parameter.scalar_operands(&parameter.node_id, &parameter_key),
        Err(ContractScalarOperandErrorV1::Projection(
            CheckedScalarOperandError::NotApplication
        ))
    );
    let mut unsupported_profile = profile();
    unsupported_profile.supported_tags.clear();
    let unsupported = package_with_profile(
        &checked,
        std::slice::from_ref(&add.node_id),
        &unsupported_profile,
    );
    assert!(matches!(
        unsupported.scalar_operands(&add.node_id, &add),
        Ok(ContractScalarOperandViewV1::NonLowered(_))
    ));
    assert_eq!(admitted.wire(), &before);
    assert_eq!(
        admitted
            .reached_closure(
                &quire_contract_model::ContractSourceKeyV1::from_structural(&add.node_id)
                    .expect("structural source key")
            )
            .expect("reached"),
        before_reached
    );
}

#[trace("TC-458", "FR-054-AC-4")]
#[test]
fn tc_458_two_valid_represented_occurrences_remain_distinct() {
    let mut wire = revised(fixture(), |wire| {
        let add = wire["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("fixture graph")
            .iter_mut()
            .find(|node| node["body"]["operation"]["identity"] == "quire.op.integer.add")
            .expect("add node");
        add["occurrences"]
            .as_array_mut()
            .expect("occurrences")
            .push(json!({"role": "expression", "ordinal": 1}));
    });
    let add_id = wire["semantic_graph"]["nodes"]
        .as_array()
        .expect("fixture graph")
        .iter()
        .find(|node| node["body"]["operation"]["identity"] == "quire.op.integer.add")
        .expect("add node")["node_id"]
        .clone();
    let mut second_source = wire["source_map"]
        .as_array()
        .expect("source map")
        .iter()
        .find(|entry| entry["node_id"] == add_id)
        .expect("add source")
        .clone();
    second_source["ordinal"] = json!(1);
    wire["source_map"]
        .as_array_mut()
        .expect("source map")
        .push(second_source);
    refresh_identity(&mut wire);
    let checked = admit(&wire);
    let first = by_operation(&checked, "quire.op.integer.add");
    let admitted = package(&checked, std::slice::from_ref(&first.node_id));
    let second = ContractOccurrenceKeyV1 {
        ordinal: 1,
        ..first.clone()
    };
    let ContractScalarOperandViewV1::Lowered(first_view) = admitted
        .scalar_operands(&first.node_id, &first)
        .expect("first represented occurrence")
    else {
        panic!("first occurrence must be lowered");
    };
    let ContractScalarOperandViewV1::Lowered(second_view) = admitted
        .scalar_operands(&second.node_id, &second)
        .expect("second represented occurrence")
    else {
        panic!("second occurrence must be lowered");
    };
    assert_eq!(first_view.application, first);
    assert_eq!(second_view.application, second);
    assert_ne!(
        first_view.application_source,
        second_view.application_source
    );
    assert_eq!(first_view.arguments.len(), second_view.arguments.len());
}

#[trace("TC-458", "FR-054-AC-4")]
#[test]
fn tc_458_inconsistent_child_source_refuses_without_a_partial_vector() {
    let checked = admit(&fixture());
    let add = by_operation(&checked, "quire.op.integer.add");
    let mut admitted = package(&checked, std::slice::from_ref(&add.node_id));
    let child = {
        let ContractScalarOperandViewV1::Lowered(view) = admitted
            .scalar_operands(&add.node_id, &add)
            .expect("pre-injection view")
        else {
            panic!("add lowered");
        };
        let ContractScalarOperandChildV1::GraphReference { node_id, .. } = &view.arguments[0].child
        else {
            panic!("first argument is graph reference");
        };
        (*node_id).clone()
    };
    assert!(admitted.fault_inject_missing_source_for_test(&child));
    assert_eq!(
        admitted.scalar_operands(&add.node_id, &add),
        Err(ContractScalarOperandErrorV1::View(
            ContractRootViewError::Inconsistent
        ))
    );
}

#[trace("TC-458", "FR-054-AC-1", "FR-054-AC-5", "FR-038-AC-162")]
#[test]
fn admitted_graph_literal_keeps_its_singleton_range_in_both_package_views() {
    let wire = revised(fixture(), |wire| {
        let target = add_body_mut(wire)["arguments"][0]["target"].clone();
        let integer = node_id(&family_key("a3a3"));
        let literal = wire["semantic_graph"]["nodes"]
            .as_array_mut()
            .expect("fixture graph")
            .iter_mut()
            .find(|node| node["node_id"] == target)
            .expect("selected graph child");
        literal["semantic_form"] = json!("literal");
        literal["semantic_type"] = integer.clone();
        literal["body"] =
            json!({"term":"literal", "type":integer, "value_kind":"integer", "value":"13"});
    });
    let checked = admit(&wire);
    let add = by_operation(&checked, "quire.op.integer.add");
    let child = match checked.term(&add.node_id).expect("admitted application") {
        quire_contract_model::CheckedTerm::Application { arguments, .. } => match &arguments[0] {
            quire_contract_model::CheckedTerm::Reference { target } => target.clone(),
            _ => panic!("first operand must be a graph reference"),
        },
        _ => panic!("selected node must be an application"),
    };
    let source_operands = checked
        .scalar_application_operands(
            &add.node_id,
            &quire_contract_model::CheckedOccurrence {
                role: CheckedOccurrenceRole::Expression,
                ordinal: add.ordinal,
            },
        )
        .expect("source singleton operands");
    assert_eq!(
        source_operands[0].range,
        quire_contract_model::CheckedScalarOperandRange {
            lower: 13,
            upper: 13
        }
    );
    assert_eq!(
        source_operands[0].child,
        quire_contract_model::CheckedScalarOperandChild::GraphChild(child.clone())
    );
    let admitted = package(&checked, std::slice::from_ref(&add.node_id));
    let before = admitted.wire().clone();
    let ContractScalarOperandViewV1::Lowered(view) = admitted
        .scalar_operands(&add.node_id, &add)
        .expect("lowered singleton operands")
    else {
        panic!("selected add must lower");
    };
    assert_eq!(view.arguments.len(), 2);
    assert_eq!(view.arguments[0].ordinal, 0);
    assert_eq!(
        view.arguments[0].own_range,
        ContractScalarOwnRangeV1::Finite {
            lower: 13,
            upper: 13
        }
    );
    let ContractScalarOperandChildV1::GraphReference { node_id, sources } =
        &view.arguments[0].child
    else {
        panic!("literal remains a graph child");
    };
    assert_eq!(**node_id, child);
    assert_eq!(*sources, sources_for(&admitted, &child));
    assert_eq!(view.arguments[1].ordinal, 1);
    assert_eq!(
        view.arguments[1].own_range,
        ContractScalarOwnRangeV1::Finite { lower: 1, upper: 1 }
    );
    assert_eq!(admitted.wire(), &before);
}
