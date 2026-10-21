// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-457: a public consumer reads ordered operands from an admitted V1 package.

use crate::admitted_result_context::{admit, by_operation, fixture, package, sources_for};
use ix_trace_rs::trace;
use quire_contract_model::{
    ContractScalarOperandChildV1, ContractScalarOperandViewV1, ContractScalarOwnRangeV1,
};

#[trace("TC-457", "FR-054-AC-1", "FR-054-AC-5")]
#[test]
fn tc_457_admitted_add_retains_graph_child_then_inline_one() {
    let checked = admit(&fixture());
    let add = by_operation(&checked, "quire.op.integer.add");
    let admitted = package(&checked, std::slice::from_ref(&add.node_id));
    let before = admitted.wire().clone();
    let identity = admitted.identity().clone();

    let ContractScalarOperandViewV1::Lowered(view) = admitted
        .scalar_operands(&add.node_id, &add)
        .expect("admitted scalar operand view")
    else {
        panic!("the requested add is lowered");
    };
    assert_eq!(view.application, add);
    assert_eq!(view.identity, admitted.identity());
    assert_eq!(view.provenance, &admitted.wire().package);
    assert_eq!(view.arguments.len(), 2);
    assert_eq!(
        (view.arguments[0].ordinal, view.arguments[1].ordinal),
        (0, 1)
    );
    let ContractScalarOperandChildV1::GraphReference { node, sources } = &view.arguments[0].child
    else {
        panic!("first argument must be a represented graph child");
    };
    assert_eq!(node.semantic_form.as_ref(), "parameter");
    assert_eq!(*sources, sources_for(&admitted, &node.node_id));
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
    assert_eq!(admitted.identity(), &identity);
}
