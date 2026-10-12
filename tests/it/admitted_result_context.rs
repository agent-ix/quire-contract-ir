// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-447: admitted, occurrence-scoped scalar result context.

use crate::support::checked_package::{
    bounds_body, canonical, evidence_for, family_key, mint_ungrouped_structural_keys, node_id,
    refresh_identity, settle, sha256_hex, v2_all_families,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    BoundedDomainForm, CheckedNodeId, CheckedNodeTag, CheckedOccurrenceRole,
    CheckedPackageReadLimits, CheckedPackageV2, CheckedPackageV2ReadResult,
    CompleteLoweringProfileV2, ContractDependencies, ContractFeatureRecordV1,
    ContractOccurrenceKeyV1, ContractPackageReadResult, ContractPackageV1, ContractPackageWireV1,
    ContractResultContextV1, ContractResultViewV1, LoweringProfileRef,
};
use serde_json::{json, Value};

fn provisional(letter: char) -> String {
    sha256_hex(format!("ir769/provisional/{letter}").as_bytes())
}

fn node(digest: &str, tag: &str, form: &str, ty: &str, body: Value) -> Value {
    json!({
        "node_id": node_id(digest),
        "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": tag,
        "semantic_form": form,
        "semantic_type": node_id(ty),
        "dependencies": [],
        "occurrences": [{"role": "expression", "ordinal": 0}],
        "body": body,
    })
}

fn operation(identity: &str, member: Value) -> Value {
    json!({"identity": identity, "laws": [], "mode": null, "member": member, "leaves": []})
}

fn application(operator: &str, operation: Value, result: &str, arguments: Vec<Value>) -> Value {
    json!({
        "term": "application", "operator": operator, "operation": operation,
        "result_type": node_id(result), "arguments": arguments,
    })
}

fn expression_ref(digest: &str) -> Value {
    json!({"term": "reference", "target": node_id(digest)})
}

/// Builds test-authored integer intervals matching FR-049's QSL declaration.
/// This is an IR reader fixture; the named producer lane separately checks
/// the source-to-wire correspondence with QSL's current compiler.
fn fixture() -> Value {
    let mut value = v2_all_families();
    let integer = family_key("a3a3");
    let text = family_key("a1a1");
    let input_bound = provisional('1');
    let result_bound = provisional('2');
    let parameter = provisional('3');
    let add = provisional('4');
    let narrow = provisional('5');

    let input = node(
        &input_bound,
        "bounded_domain",
        "integer_range",
        &integer,
        bounds_body("0", "9"),
    );
    let result = node(
        &result_bound,
        "bounded_domain",
        "integer_range",
        &integer,
        bounds_body("0", "10"),
    );
    let parameter_node = node(
        &parameter,
        "value",
        "parameter",
        &input_bound,
        json!({"term": "aggregate", "members": [
            {"term": "binding", "name": "name", "value": {
                "term": "literal", "type": node_id(&text),
                "value_kind": "text", "value": "a"}},
            {"term": "binding", "name": "level", "value": {
                "term": "literal", "type": node_id(&integer),
                "value_kind": "integer", "value": "0"}},
        ]}),
    );
    let add_node = node(
        &add,
        "expression",
        "binary",
        &integer,
        application(
            "binary",
            operation("quire.op.integer.add", Value::Null),
            &integer,
            vec![
                expression_ref(&parameter),
                json!({"term": "literal", "type": node_id(&integer),
                    "value_kind": "integer", "value": "1"}),
            ],
        ),
    );
    let narrow_node = node(
        &narrow,
        "expression",
        "conversion",
        &result_bound,
        application(
            "convert",
            operation(
                "quire.op.numeric.narrow",
                json!({"kind": "type_argument", "type_argument": node_id(&result_bound)}),
            ),
            &result_bound,
            vec![expression_ref(&add)],
        ),
    );
    value["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("fixture graph")
        .extend([input, result, parameter_node, add_node, narrow_node]);
    mint_ungrouped_structural_keys(&mut value);
    settle(&mut value);
    value
}

fn admit(value: &Value) -> CheckedPackageV2 {
    match CheckedPackageV2::read(
        &canonical(value),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(value),
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("FR-049 fixture must admit: {other:?}"),
    }
}

fn profile() -> CompleteLoweringProfileV2 {
    CompleteLoweringProfileV2 {
        reference: LoweringProfileRef {
            authority: "quire".into(),
            identity: "result-context".into(),
        },
        supported_tags: CheckedNodeTag::ALL.iter().copied().collect(),
        require_bounds: false,
        work_limit: u64::MAX,
    }
}

fn package(source: &CheckedPackageV2, roots: &[CheckedNodeId]) -> ContractPackageV1 {
    let lower = source.lower(roots, &profile());
    let bytes = lower
        .package
        .canonical_bytes()
        .expect("lowered package bytes");
    match ContractPackageWireV1::read(
        bytes,
        CheckedPackageReadLimits::bounded(),
        &ContractDependencies::new(),
    ) {
        ContractPackageReadResult::Admitted(package) => *package,
        other => panic!("FR-330 result must admit: {other:?}"),
    }
}

fn by_operation(package: &CheckedPackageV2, identity: &str) -> ContractOccurrenceKeyV1 {
    let node = package
        .graph()
        .nodes
        .iter()
        .find(|node| {
            node.body
                .pointer("/operation/identity")
                .and_then(Value::as_str)
                == Some(identity)
        })
        .expect("named operation");
    ContractOccurrenceKeyV1 {
        node_id: node.node_id.clone(),
        role: CheckedOccurrenceRole::Expression,
        ordinal: 0,
    }
}

/// Trace: FR-049-AC-1, FR-049-AC-2, FR-049-AC-4, FR-049-AC-5, FR-049-AC-6
#[trace(
    "TC-447",
    "FR-049-AC-1",
    "FR-049-AC-2",
    "FR-049-AC-4",
    "FR-049-AC-5",
    "FR-049-AC-6"
)]
#[test]
fn tc_447_context_root_selects_authored_result_interval_without_rewriting_forward_closure() {
    let input = admit(&fixture());
    let add = by_operation(&input, "quire.op.integer.add");
    let narrow = by_operation(&input, "quire.op.numeric.narrow");
    let plain = package(&input, std::slice::from_ref(&add.node_id));
    let contextual = package(&input, &[add.node_id.clone(), narrow.node_id.clone()]);
    assert_eq!(plain.wire().feature_accounting.len(), 1);
    assert_eq!(contextual.wire().feature_accounting.len(), 2);
    assert!(matches!(
        contextual.wire().feature_accounting.as_slice(),
        [
            ContractFeatureRecordV1::Lowered { .. },
            ContractFeatureRecordV1::Lowered { .. }
        ]
    ));
    assert_ne!(plain.identity(), contextual.identity());
    let before = contextual.wire().clone();
    let before_identity = contextual.identity().clone();
    let ContractResultViewV1::Lowered(plain_view) = plain
        .result_context(
            &add.node_id,
            &add,
            Some(&narrow),
            BoundedDomainForm::IntegerRange,
        )
        .expect("plain view")
    else {
        panic!("add lowered in plain package");
    };
    assert_eq!(
        plain_view.context,
        ContractResultContextV1::IncompleteContext
    );
    let ContractResultViewV1::Lowered(view) = contextual
        .result_context(
            &add.node_id,
            &add,
            Some(&narrow),
            BoundedDomainForm::IntegerRange,
        )
        .expect("contextual view")
    else {
        panic!("add lowered in contextual package");
    };
    let ContractResultContextV1::Narrowed {
        bound, conversion, ..
    } = view.context
    else {
        panic!("the selected direct narrow must supply a bound");
    };
    assert_eq!(conversion.node_id, narrow.node_id);
    assert_eq!(bound.node.semantic_form.as_ref(), "integer_range");
    assert_eq!(bound.node.body, bounds_body("0", "10"));
    assert!(bound
        .sources
        .iter()
        .any(|source| source.node_id == bound.node.node_id));
    assert_eq!(view.reached.root.node_id, add.node_id);
    assert!(!view
        .reached
        .nodes
        .iter()
        .any(|entry| entry.node.node_id == narrow.node_id));
    assert!(view
        .reached
        .nodes
        .iter()
        .any(|entry| entry.node.body == bounds_body("0", "9")));
    assert_eq!(contextual.wire(), &before);
    assert_eq!(contextual.identity(), &before_identity);
}

/// Trace: FR-049-AC-2, FR-049-AC-3, FR-049-AC-4
#[trace("TC-447", "FR-049-AC-2", "FR-049-AC-3", "FR-049-AC-4")]
#[test]
fn tc_447_none_and_invalid_occurrences_have_distinct_typed_outcomes() {
    let input = admit(&fixture());
    let add = by_operation(&input, "quire.op.integer.add");
    let narrow = by_operation(&input, "quire.op.numeric.narrow");
    let contextual = package(&input, &[add.node_id.clone(), narrow.node_id.clone()]);
    let query = |application: &ContractOccurrenceKeyV1,
                 conversion: Option<&ContractOccurrenceKeyV1>,
                 expected| {
        let ContractResultViewV1::Lowered(view) = contextual
            .result_context(&add.node_id, application, conversion, expected)
            .expect("admitted query")
        else {
            panic!("add lowered");
        };
        view.context
    };
    assert_eq!(
        query(&add, None, BoundedDomainForm::IntegerRange),
        ContractResultContextV1::Unnarrowed,
        "a represented narrow belonging to another item cannot alter QSL None"
    );
    assert!(matches!(
        query(&add, Some(&narrow), BoundedDomainForm::RationalRange),
        ContractResultContextV1::MissingBound { type_key } if
            type_key == contextual.wire().semantic_graph.nodes.iter()
                .find(|node| node.node_id == narrow.node_id).expect("narrow").semantic_type
    ));
    for invalid_application in [
        ContractOccurrenceKeyV1 {
            role: CheckedOccurrenceRole::Type,
            ..add.clone()
        },
        ContractOccurrenceKeyV1 {
            ordinal: 1,
            ..add.clone()
        },
        ContractOccurrenceKeyV1 {
            node_id: narrow.node_id.clone(),
            ..add.clone()
        },
    ] {
        assert_eq!(
            query(
                &invalid_application,
                Some(&narrow),
                BoundedDomainForm::IntegerRange
            ),
            ContractResultContextV1::IncompleteContext
        );
    }
    for invalid_conversion in [
        ContractOccurrenceKeyV1 {
            role: CheckedOccurrenceRole::Generated,
            ..narrow.clone()
        },
        ContractOccurrenceKeyV1 {
            ordinal: 1,
            ..narrow.clone()
        },
        ContractOccurrenceKeyV1 {
            node_id: add.node_id.clone(),
            ..narrow.clone()
        },
    ] {
        assert_eq!(
            query(
                &add,
                Some(&invalid_conversion),
                BoundedDomainForm::IntegerRange
            ),
            ContractResultContextV1::IncompleteContext
        );
    }

    let parameter = input
        .graph()
        .nodes
        .iter()
        .find(|node| {
            node.node_tag.as_ref() == "value"
                && node.semantic_form.as_ref() == "parameter"
                && node
                    .body
                    .pointer("/members/0/value/value")
                    .and_then(Value::as_str)
                    == Some("a")
        })
        .expect("authored parameter");
    let parameter_occurrence = ContractOccurrenceKeyV1 {
        node_id: parameter.node_id.clone(),
        role: CheckedOccurrenceRole::Expression,
        ordinal: 0,
    };
    let own_package = package(&input, std::slice::from_ref(&parameter.node_id));
    let ContractResultViewV1::Lowered(own_view) = own_package
        .result_context(
            &parameter.node_id,
            &parameter_occurrence,
            None,
            BoundedDomainForm::IntegerRange,
        )
        .expect("own result")
    else {
        panic!("parameter lowered");
    };
    let ContractResultContextV1::DirectBounded { bound } = own_view.context else {
        panic!("the author's Int[0, 9] is the direct result");
    };
    assert_eq!(bound.node.body, bounds_body("0", "9"));

    let not_requested = ContractOccurrenceKeyV1 {
        node_id: parameter.node_id.clone(),
        role: CheckedOccurrenceRole::Expression,
        ordinal: 0,
    };
    assert_eq!(
        contextual.result_context(
            &parameter.node_id,
            &not_requested,
            None,
            BoundedDomainForm::IntegerRange,
        ),
        Ok(ContractResultViewV1::Absent),
        "a key in the add closure is not a requested root"
    );
    let absent = CheckedNodeId {
        domain: "quire.checked-semantic-node/v1".into(),
        digest: provisional('f').into(),
    };
    let with_refusal = package(&input, &[add.node_id.clone(), absent.clone()]);
    assert!(matches!(
        with_refusal.result_context(&absent, &add, None, BoundedDomainForm::IntegerRange),
        Ok(ContractResultViewV1::NonLowered(ContractFeatureRecordV1::InvalidInput { node_id }))
            if node_id == &absent
    ));
}

/// Trace: FR-049-AC-2
#[trace("TC-447", "FR-049-AC-2")]
#[test]
fn tc_447_direct_bounded_expression_keeps_own_result_until_its_narrow_is_selected() {
    let mut wire = fixture();
    let parameter = wire["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| {
            node["node_tag"] == "value"
                && node["semantic_form"] == "parameter"
                && node["body"]["members"][0]["value"]["value"] == "a"
        })
        .expect("authored parameter");
    let parameter_key = parameter["node_id"]["digest"]
        .as_str()
        .expect("parameter key")
        .to_owned();
    let input_bound = parameter["semantic_type"]["digest"]
        .as_str()
        .expect("input bound key")
        .to_owned();
    let result_bound = wire["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["body"]["operation"]["identity"] == "quire.op.numeric.narrow")
        .expect("authored result narrow")["semantic_type"]["digest"]
        .as_str()
        .expect("result bound key")
        .to_owned();
    let expression_key = provisional('7');
    let expression = node(
        &expression_key,
        "expression",
        "binary",
        &input_bound,
        application(
            "binary",
            operation("quire.op.integer.add", Value::Null),
            &input_bound,
            vec![
                expression_ref(&parameter_key),
                json!({"term": "literal", "type": node_id(&family_key("a3a3")),
                    "value_kind": "integer", "value": "0"}),
            ],
        ),
    );
    let narrow = node(
        &provisional('8'),
        "expression",
        "conversion",
        &result_bound,
        application(
            "convert",
            operation(
                "quire.op.numeric.narrow",
                json!({"kind": "type_argument", "type_argument": node_id(&result_bound)}),
            ),
            &result_bound,
            vec![expression_ref(&expression_key)],
        ),
    );
    wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend([expression, narrow]);
    mint_ungrouped_structural_keys(&mut wire);
    settle(&mut wire);

    let input = admit(&wire);
    let expression = input
        .graph()
        .nodes
        .iter()
        .find(|node| {
            node.node_tag.as_ref() == "expression"
                && node.semantic_form.as_ref() == "binary"
                && node.semantic_type.digest.as_ref() == input_bound.as_str()
                && node.body.pointer("/arguments/0/target") == Some(&node_id(&parameter_key))
        })
        .expect("own-bounded expression");
    let narrow = input
        .graph()
        .nodes
        .iter()
        .find(|node| {
            node.body.pointer("/operation/identity") == Some(&json!("quire.op.numeric.narrow"))
                && node.body.pointer("/arguments/0/target")
                    == Some(&serde_json::to_value(&expression.node_id).expect("typed key"))
        })
        .expect("direct narrow over own-bounded expression");
    let application = ContractOccurrenceKeyV1 {
        node_id: expression.node_id.clone(),
        role: CheckedOccurrenceRole::Expression,
        ordinal: 0,
    };
    let conversion = ContractOccurrenceKeyV1 {
        node_id: narrow.node_id.clone(),
        role: CheckedOccurrenceRole::Expression,
        ordinal: 0,
    };
    let emitted = package(
        &input,
        &[expression.node_id.clone(), narrow.node_id.clone()],
    );
    let outcome = |selected: Option<&ContractOccurrenceKeyV1>, expected| {
        let ContractResultViewV1::Lowered(view) = emitted
            .result_context(&expression.node_id, &application, selected, expected)
            .expect("admitted result")
        else {
            panic!("own-bounded expression lowered");
        };
        view.context
    };
    let ContractResultContextV1::DirectBounded { bound } =
        outcome(None, BoundedDomainForm::IntegerRange)
    else {
        panic!("QSL None selects the expression's own bound");
    };
    assert_eq!(bound.node.node_id.digest.as_ref(), input_bound.as_str());
    assert_eq!(bound.node.body, bounds_body("0", "9"));
    assert!(matches!(
        outcome(None, BoundedDomainForm::RationalRange),
        ContractResultContextV1::MissingBound { type_key }
            if type_key == expression.semantic_type
    ));
    let ContractResultContextV1::Narrowed { bound, .. } =
        outcome(Some(&conversion), BoundedDomainForm::IntegerRange)
    else {
        panic!("QSL Some selects the direct narrow even over an own-bounded expression");
    };
    assert_eq!(bound.node.node_id.digest.as_ref(), result_bound.as_str());
    assert_eq!(bound.node.body, bounds_body("0", "10"));
}

/// Trace: FR-049-AC-3, FR-049-AC-5, FR-049-AC-6
#[trace("TC-447", "FR-049-AC-3", "FR-049-AC-5", "FR-049-AC-6")]
#[test]
fn tc_447_shared_expression_uses_each_supplied_occurrence_and_direct_narrow() {
    let mut wire = fixture();
    let add_key = wire["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .find(|node| node["body"]["operation"]["identity"] == "quire.op.integer.add")
        .expect("add")["node_id"]["digest"]
        .as_str()
        .expect("add digest")
        .to_owned();
    let other_bound = family_key("cccc"); // The fixture's authored Int[0, 1000].
    let other = node(
        &provisional('6'),
        "expression",
        "conversion",
        &other_bound,
        application(
            "convert",
            operation(
                "quire.op.numeric.narrow",
                json!({"kind": "type_argument", "type_argument": node_id(&other_bound)}),
            ),
            &other_bound,
            vec![expression_ref(&add_key)],
        ),
    );
    wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .push(other);
    settle(&mut wire);

    // One semantic add key has two QSL-like application occurrences. Their
    // distinct ordinals belong to the caller's checked per-item association.
    let add_id = node_id(&add_key);
    let nodes = wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes");
    let add_node = nodes
        .iter_mut()
        .find(|node| node["node_id"] == add_id)
        .expect("add after settling");
    add_node["occurrences"]
        .as_array_mut()
        .expect("occurrences")
        .push(json!({"role": "expression", "ordinal": 1}));
    let sources = wire["source_map"].as_array_mut().expect("source map");
    let add_position = sources
        .iter()
        .position(|entry| entry["node_id"] == add_id)
        .expect("add source entry");
    let mut second = sources[add_position].clone();
    second["ordinal"] = json!(1);
    sources.insert(add_position + 1, second);
    refresh_identity(&mut wire);

    let input = admit(&wire);
    let add = by_operation(&input, "quire.op.integer.add");
    let narrows = input
        .graph()
        .nodes
        .iter()
        .filter(|node| {
            node.body
                .pointer("/operation/identity")
                .and_then(Value::as_str)
                == Some("quire.op.numeric.narrow")
        })
        .collect::<Vec<_>>();
    let [first, second] = narrows.as_slice() else {
        panic!("two represented direct narrows");
    };
    let selected = |node: &quire_contract_model::CheckedSemanticNodeV2| ContractOccurrenceKeyV1 {
        node_id: node.node_id.clone(),
        role: CheckedOccurrenceRole::Expression,
        ordinal: 0,
    };
    let first = selected(first);
    let second = selected(second);
    let combined = package(
        &input,
        &[
            add.node_id.clone(),
            first.node_id.clone(),
            second.node_id.clone(),
        ],
    );
    let application_one = ContractOccurrenceKeyV1 {
        ordinal: 1,
        ..add.clone()
    };
    let result = |application: &ContractOccurrenceKeyV1, conversion: &ContractOccurrenceKeyV1| {
        let ContractResultViewV1::Lowered(view) = combined
            .result_context(
                &add.node_id,
                application,
                Some(conversion),
                BoundedDomainForm::IntegerRange,
            )
            .expect("admitted query")
        else {
            panic!("shared add lowered");
        };
        let ContractResultContextV1::Narrowed { bound, .. } = view.context else {
            panic!("each selected direct narrow must return a bound");
        };
        bound.node.body.clone()
    };
    let first_body = result(&add, &first);
    let second_body = result(&application_one, &second);
    assert_ne!(first_body, second_body);
    assert!([first_body.clone(), second_body.clone()].contains(&bounds_body("0", "10")));
    assert!([first_body.clone(), second_body.clone()].contains(&bounds_body("0", "1000")));
    assert_eq!(result(&add, &second), second_body);
    assert_eq!(result(&application_one, &first), first_body);
}
