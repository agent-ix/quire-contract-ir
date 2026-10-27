// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-447: admitted, occurrence-scoped scalar result context.

use crate::support::checked_package::{
    bounds_body, canonical, evidence_for, family_key, mint_ungrouped_structural_keys, node_id,
    refresh_identity, refresh_identity_projection, settle, sha256_hex, v2_all_families,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    BoundedDomainForm, CheckedNodeId, CheckedNodeTag, CheckedOccurrenceRole,
    CheckedPackageReadLimits, CheckedPackageRefusalCause, CheckedPackageRefusalCode,
    CheckedPackageV2, CheckedPackageV2ReadResult, CompleteLoweringProfileV2, ContractDependencies,
    ContractFeatureRecordV1, ContractOccurrenceKeyV1, ContractPackageReadResult, ContractPackageV1,
    ContractPackageWireV1, ContractResultContextV1, ContractResultViewV1, ContractRootViewError,
    LoweringProfileRef,
};
use serde_json::{json, Value};

fn body_at(term: &quire_contract_model::CheckedTerm, path: &str) -> Option<Value> {
    serde_json::to_value(term).ok()?.pointer(path).cloned()
}

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
pub(crate) fn fixture() -> Value {
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
                json!({"kind": "type_argument", "declaration": node_id(&result_bound)}),
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

#[trace("TC-048", "FR-038-AC-247", "FR-038-AC-250")]
#[test]
fn tc_048_numeric_narrow_result_equals_type_argument() {
    let mut wire = fixture();
    let at = wire["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["body"]["operation"]["identity"] == "quire.op.numeric.narrow")
        .expect("narrow");
    admit(&wire);
    let integer = family_key("a3a3");
    wire["semantic_graph"]["nodes"][at]["body"]["result_type"] = node_id(&integer);
    wire["semantic_graph"]["nodes"][at]["semantic_type"] = node_id(&integer);
    settle(&mut wire);
    let CheckedPackageV2ReadResult::Refused(refusal) = CheckedPackageV2::read(
        &canonical(&wire),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&wire),
    ) else {
        panic!("numeric.narrow mismatch must refuse");
    };
    assert_eq!(
        refusal.code,
        CheckedPackageRefusalCode::IllTyped,
        "{refusal:?}"
    );
    assert_eq!(
        refusal.cause,
        Some(CheckedPackageRefusalCause::OperatorIneligible)
    );
    assert_eq!(
        refusal.path.as_ref().map(ToString::to_string),
        Some(format!("/semantic_graph/nodes/{at}/body/result_type"))
    );
    assert_eq!(
        serde_json::to_value(refusal.locus).expect("locus"),
        wire["semantic_graph"]["nodes"][at]["node_id"]
    );
}

pub(crate) fn admit(value: &Value) -> CheckedPackageV2 {
    match CheckedPackageV2::read(
        &canonical(value),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(value),
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("FR-049 fixture must admit: {other:?}"),
    }
}

pub(crate) fn profile() -> CompleteLoweringProfileV2 {
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

pub(crate) fn package(source: &CheckedPackageV2, roots: &[CheckedNodeId]) -> ContractPackageV1 {
    package_with_profile(source, roots, &profile())
}

pub(crate) fn package_with_profile(
    source: &CheckedPackageV2,
    roots: &[CheckedNodeId],
    selected_profile: &CompleteLoweringProfileV2,
) -> ContractPackageV1 {
    let lower = source.lower(roots, selected_profile);
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

pub(crate) fn sources_for<'a>(
    package: &'a ContractPackageV1,
    key: &CheckedNodeId,
) -> &'a [quire_contract_model::CheckedSourceMapEntry] {
    package
        .source_occurrences(key)
        .expect("admitted source-map node")
}

pub(crate) fn by_operation(package: &CheckedPackageV2, identity: &str) -> ContractOccurrenceKeyV1 {
    let node = package
        .graph()
        .nodes
        .iter()
        .find(|node| {
            body_at(&node.body, "/operation/identity")
                .as_ref()
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
    let first_source = contextual
        .wire()
        .source_map
        .nodes
        .iter()
        .find(|entry| entry.node_id == add.node_id)
        .expect("selected application source");
    assert_eq!(
        contextual.source_occurrences(&add.node_id),
        Some(first_source.occurrences.as_slice())
    );
    let missing: CheckedNodeId =
        serde_json::from_value(node_id(&provisional('9'))).expect("missing node id");
    assert!(contextual.source_occurrences(&missing).is_none());
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
        bound,
        conversion,
        conversion_sources,
    } = view.context
    else {
        panic!("the selected direct narrow must supply a bound");
    };
    assert_eq!(conversion.node_id, narrow.node_id);
    assert_eq!(view.provenance, &contextual.wire().package);
    assert_eq!(view.provenance.lowering_profile, profile().reference);
    assert_eq!(view.identity, contextual.identity());
    assert_eq!(view.application, add);
    assert_eq!(view.conversion, Some(narrow.clone()));
    assert_eq!(
        conversion_sources,
        sources_for(&contextual, &narrow.node_id)
    );
    assert_eq!(bound.node.semantic_form.as_ref(), "integer_range");
    assert_eq!(bound.node.body, bounds_body("0", "10"));
    assert_eq!(bound.sources, sources_for(&contextual, &bound.node.node_id));
    assert!(bound
        .sources
        .iter()
        .all(|source| source.node_id == bound.node.node_id));
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
    let expected = contextual
        .result_context(
            &add.node_id,
            &add,
            Some(&narrow),
            BoundedDomainForm::IntegerRange,
        )
        .expect("admitted result context");
    for requests in [2, 4, 8] {
        for _ in 0..requests {
            assert_eq!(
                contextual.result_context(
                    &add.node_id,
                    &add,
                    Some(&narrow),
                    BoundedDomainForm::IntegerRange,
                ),
                Ok(expected.clone())
            );
        }
    }
    assert_eq!(contextual.wire(), &before);
    assert_eq!(contextual.identity(), &before_identity);
}

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
                && body_at(&node.body, "/members/0/value/value")
                    .as_ref()
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

#[trace("TC-447", "FR-049-AC-3")]
#[test]
fn tc_447_represented_wrong_conversion_shapes_refuse_without_a_bound() {
    let mut wire = fixture();
    let nodes = wire["semantic_graph"]["nodes"].as_array().expect("nodes");
    let add_key = nodes
        .iter()
        .find(|node| node["body"]["operation"]["identity"] == "quire.op.integer.add")
        .expect("add")["node_id"]["digest"]
        .as_str()
        .expect("add key")
        .to_owned();
    let parameter_key = nodes
        .iter()
        .find(|node| {
            node["semantic_form"] == "parameter"
                && node["body"]["members"][0]["value"]["value"] == "a"
        })
        .expect("authored integer parameter")["node_id"]["digest"]
        .as_str()
        .expect("parameter key")
        .to_owned();
    let result_bound = nodes
        .iter()
        .find(|node| node["body"]["operation"]["identity"] == "quire.op.numeric.narrow")
        .expect("narrow")["semantic_type"]["digest"]
        .as_str()
        .expect("result key")
        .to_owned();
    let integer = family_key("a3a3");
    let rational = provisional('d');
    let rational_type = node(
        &rational,
        "scalar_type",
        "rational",
        &rational,
        json!({"term": "aggregate", "members": []}),
    );
    let other_add_key = provisional('a');
    let other_add = node(
        &other_add_key,
        "expression",
        "binary",
        &integer,
        application(
            "binary",
            operation("quire.op.integer.add", Value::Null),
            &integer,
            vec![
                expression_ref(&parameter_key),
                json!({"term": "literal", "type": node_id(&integer),
                    "value_kind": "integer", "value": "2"}),
            ],
        ),
    );
    let other_narrow = node(
        &provisional('b'),
        "expression",
        "conversion",
        &result_bound,
        application(
            "convert",
            operation(
                "quire.op.numeric.narrow",
                json!({"kind": "type_argument", "declaration": node_id(&result_bound)}),
            ),
            &result_bound,
            vec![expression_ref(&other_add_key)],
        ),
    );
    let wrong_operation = node(
        &provisional('c'),
        "expression",
        "conversion",
        &rational,
        application(
            "convert",
            operation(
                "quire.op.numeric.convert",
                json!({"kind": "type_argument", "declaration": node_id(&rational)}),
            ),
            &rational,
            vec![expression_ref(&add_key)],
        ),
    );
    wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .extend([rational_type, other_add, other_narrow, wrong_operation]);
    mint_ungrouped_structural_keys(&mut wire);
    settle(&mut wire);

    let input = admit(&wire);
    let add = input
        .graph()
        .nodes
        .iter()
        .find(|node| {
            body_at(&node.body, "/operation/identity") == Some(json!("quire.op.integer.add"))
                && body_at(&node.body, "/arguments/1/value") == Some(json!("1"))
        })
        .expect("original add");
    let other_add = input
        .graph()
        .nodes
        .iter()
        .find(|node| {
            body_at(&node.body, "/operation/identity") == Some(json!("quire.op.integer.add"))
                && body_at(&node.body, "/arguments/1/value") == Some(json!("2"))
        })
        .expect("other add");
    let other_narrow = input
        .graph()
        .nodes
        .iter()
        .find(|node| {
            body_at(&node.body, "/operation/identity") == Some(json!("quire.op.numeric.narrow"))
                && body_at(&node.body, "/arguments/0/target")
                    == Some(serde_json::to_value(&other_add.node_id).expect("typed key"))
        })
        .expect("direct narrow of another expression");
    let wrong_operation = input
        .graph()
        .nodes
        .iter()
        .find(|node| {
            body_at(&node.body, "/operation/identity") == Some(json!("quire.op.numeric.convert"))
        })
        .expect("represented wrong operation");
    let occurrence = |node: &quire_contract_model::CheckedSemanticNodeV2| ContractOccurrenceKeyV1 {
        node_id: node.node_id.clone(),
        role: CheckedOccurrenceRole::Expression,
        ordinal: 0,
    };
    let requested = occurrence(add);
    let emitted = package(
        &input,
        &[
            add.node_id.clone(),
            other_add.node_id.clone(),
            other_narrow.node_id.clone(),
            wrong_operation.node_id.clone(),
        ],
    );
    for (selected, reason) in [
        (occurrence(other_add), "wrong form: binary, not conversion"),
        (
            occurrence(wrong_operation),
            "wrong operation: numeric.convert",
        ),
        (
            occurrence(other_narrow),
            "wrong direct argument: another expression",
        ),
    ] {
        assert!(
            emitted.wire().feature_accounting.iter().any(|record| {
                record.node_id() == &selected.node_id
                    && matches!(record, ContractFeatureRecordV1::Lowered { .. })
            }),
            "{reason} must be represented and lowered"
        );
        let ContractResultViewV1::Lowered(view) = emitted
            .result_context(
                &add.node_id,
                &requested,
                Some(&selected),
                BoundedDomainForm::IntegerRange,
            )
            .expect("admitted query")
        else {
            panic!("original add must be lowered");
        };
        assert_eq!(
            view.context,
            ContractResultContextV1::IncompleteContext,
            "{reason}"
        );
    }
}

#[trace("TC-447", "FR-049-AC-4")]
#[test]
fn tc_447_exhausted_lowering_and_post_admission_edge_return_no_partial_context() {
    let input = admit(&fixture());
    let add = by_operation(&input, "quire.op.integer.add");
    let narrow = by_operation(&input, "quire.op.numeric.narrow");

    let mut exhausted_profile = profile();
    exhausted_profile.work_limit = 0;
    let exhausted = package_with_profile(
        &input,
        std::slice::from_ref(&add.node_id),
        &exhausted_profile,
    );
    assert!(matches!(
        exhausted.wire().feature_accounting.as_slice(),
        [ContractFeatureRecordV1::Failed { node_id, .. }] if node_id == &add.node_id
    ));
    assert!(matches!(
        exhausted.result_context(
            &add.node_id,
            &add,
            None,
            BoundedDomainForm::IntegerRange,
        ),
        Ok(ContractResultViewV1::NonLowered(ContractFeatureRecordV1::Failed {
            node_id, ..
        })) if node_id == &add.node_id
    ));

    let mut admitted = package(&input, &[add.node_id.clone(), narrow.node_id.clone()]);
    assert!(matches!(
        admitted.result_context(
            &add.node_id,
            &add,
            Some(&narrow),
            BoundedDomainForm::IntegerRange,
        ),
        Ok(ContractResultViewV1::Lowered(_))
    ));
    assert!(admitted.fault_inject_missing_source_for_test(&add.node_id));
    assert_eq!(
        admitted.result_context(
            &add.node_id,
            &add,
            Some(&narrow),
            BoundedDomainForm::IntegerRange,
        ),
        Err(ContractRootViewError::Inconsistent),
        "a post-admission lost conversion source edge must not return a partial result"
    );
}

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
                json!({"kind": "type_argument", "declaration": node_id(&result_bound)}),
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
                && body_at(&node.body, "/arguments/0/target") == Some(node_id(&parameter_key))
        })
        .expect("own-bounded expression");
    let narrow = input
        .graph()
        .nodes
        .iter()
        .find(|node| {
            body_at(&node.body, "/operation/identity") == Some(json!("quire.op.numeric.narrow"))
                && body_at(&node.body, "/arguments/0/target")
                    == Some(serde_json::to_value(&expression.node_id).expect("typed key"))
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
                json!({"kind": "type_argument", "declaration": node_id(&other_bound)}),
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
            body_at(&node.body, "/operation/identity")
                .as_ref()
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
        assert_eq!(view.provenance, &combined.wire().package);
        assert_eq!(view.provenance.lowering_profile, profile().reference);
        assert_eq!(view.identity, combined.identity());
        assert_eq!(view.application, *application);
        assert_eq!(view.conversion.as_ref(), Some(conversion));
        assert_eq!(view.reached.root.node_id, add.node_id);
        let ContractResultContextV1::Narrowed {
            conversion: converted,
            conversion_sources,
            bound,
        } = view.context
        else {
            panic!("each selected direct narrow must return a bound");
        };
        assert_eq!(converted.node_id, conversion.node_id);
        assert_eq!(
            conversion_sources,
            sources_for(&combined, &conversion.node_id)
        );
        assert_eq!(bound.node.node_id, converted.semantic_type);
        assert_eq!(bound.sources, sources_for(&combined, &bound.node.node_id));
        assert!(bound
            .sources
            .iter()
            .all(|source| source.node_id == bound.node.node_id));
        (bound.node.node_id.clone(), bound.node.body.clone())
    };
    let first_result = result(&add, &first);
    let second_result = result(&application_one, &second);
    assert_ne!(first_result.0, second_result.0);
    assert_ne!(first_result.1, second_result.1);
    assert!([first_result.1.clone(), second_result.1.clone()].contains(&bounds_body("0", "10")));
    assert!([first_result.1.clone(), second_result.1.clone()].contains(&bounds_body("0", "1000")));
    assert_eq!(result(&add, &second), second_result);
    assert_eq!(result(&application_one, &first), first_result);
}

/// Golden bytes and work for the admitted fixture with a declared protocol body.
#[trace("TC-447", "FR-049-AC-5")]
#[test]
fn ir987_scalar_and_result_results_bytes_and_work_match_admitted_fixture() {
    let wire = fixture();
    let bytes = canonical(&wire);
    let checked = admit(&wire);
    let add = by_operation(&checked, "quire.op.integer.add");
    let narrow = by_operation(&checked, "quire.op.numeric.narrow");
    let occurrence = quire_contract_model::CheckedOccurrence {
        role: CheckedOccurrenceRole::Expression,
        ordinal: 0,
    };
    let scalar = checked.scalar_application_operands(&add.node_id, &occurrence);
    let scalar = scalar.expect("scalar operands");
    let [parameter, literal] = scalar.as_slice() else {
        panic!("two scalar operands")
    };
    assert_eq!(parameter.ordinal, 0);
    assert_eq!(
        parameter.range,
        quire_contract_model::CheckedScalarOperandRange { lower: 0, upper: 9 }
    );
    let parameter_key: CheckedNodeId = serde_json::from_value(
        wire["semantic_graph"]["nodes"]
            .as_array()
            .expect("authored nodes")
            .iter()
            .find(|node| {
                node["node_id"] == serde_json::to_value(&add.node_id).expect("application id")
            })
            .expect("authored application")["body"]["arguments"][0]["target"]
            .clone(),
    )
    .expect("parameter key");
    assert_eq!(
        parameter.child,
        quire_contract_model::CheckedScalarOperandChild::GraphChild(parameter_key)
    );
    assert_eq!(literal.ordinal, 1);
    assert_eq!(
        literal.range,
        quire_contract_model::CheckedScalarOperandRange { lower: 1, upper: 1 }
    );
    assert_eq!(
        literal.child,
        quire_contract_model::CheckedScalarOperandChild::InlineLiteral {
            application: add.node_id.clone(),
            occurrence: occurrence.clone(),
            ordinal: 1
        }
    );
    let missing: CheckedNodeId =
        serde_json::from_value(node_id(&"f".repeat(64))).expect("missing id");
    let missing_scalar = checked.scalar_application_operands(&missing, &occurrence);
    assert_eq!(
        missing_scalar,
        Err(quire_contract_model::CheckedScalarOperandError::UnknownNode)
    );
    let mut prior_wire = wire.clone();
    let root_at = prior_wire["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["semantic_form"] == "model_root")
        .expect("selected model root");
    let root_id = prior_wire["semantic_graph"]["nodes"][root_at]["node_id"].clone();
    prior_wire["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .remove(root_at);
    let sources = prior_wire["source_map"].as_array_mut().expect("source map");
    assert_eq!(sources[root_at]["node_id"], root_id);
    sources.remove(root_at);
    for entry in sources.iter_mut().skip(root_at) {
        for region in entry["regions"].as_array_mut().expect("regions") {
            for boundary in ["start", "end"] {
                let offset = region[boundary].as_u64().expect("source offset");
                region[boundary] = json!(offset - 1);
            }
        }
    }
    refresh_identity_projection(&mut prior_wire);
    assert_eq!(
        canonical(&prior_wire),
        include_bytes!("../fixtures/ir987/scalar.json")
    );
    assert_eq!(
        *checked.package_id(),
        serde_json::from_value(wire["package_id"].clone()).expect("package identity")
    );
    let authored_sources: Vec<quire_contract_model::CheckedSourceMapEntry> =
        serde_json::from_value(wire["source_map"].clone()).expect("authored sources");
    assert_eq!(checked.source_map(), authored_sources);
    let evidence = evidence_for(&wire);
    let mut low = 0;
    let mut high = CheckedPackageReadLimits::bounded().work;
    while low + 1 < high {
        let middle = low + (high - low) / 2;
        let mut limits = CheckedPackageReadLimits::bounded();
        limits.work = middle;
        match CheckedPackageV2::read(&bytes, limits, &evidence) {
            CheckedPackageV2ReadResult::Admitted(_) => high = middle,
            CheckedPackageV2ReadResult::Incomplete(_) => low = middle,
            other => panic!("baseline work: {other:?}"),
        }
    }
    let mut limits = CheckedPackageReadLimits::bounded();
    limits.work = high - 1;
    let CheckedPackageV2ReadResult::Incomplete(incomplete) =
        CheckedPackageV2::read(&bytes, limits, &evidence)
    else {
        panic!("work boundary");
    };
    // Protocol membership charges five, causal graph/progress seven, and the
    // binder-scope pass charges 36 on the pre-root fixture: 340 + 5 + 7 + 36.
    // The selected root adds four validation visits to the admitted wire.
    assert_eq!((high, incomplete.consumed), (388 + 4, 392));

    let contract = package(&checked, &[add.node_id.clone(), narrow.node_id.clone()]);
    let result = contract.result_context(
        &add.node_id,
        &add,
        Some(&narrow),
        BoundedDomainForm::IntegerRange,
    );
    let ContractResultViewV1::Lowered(view) = result.expect("result context") else {
        panic!("lowered")
    };
    assert_eq!(view.application, add);
    assert_eq!(view.conversion, Some(narrow.clone()));
    assert_eq!(view.identity, contract.identity());
    assert_eq!(view.provenance, &contract.wire().package);
    let ContractResultContextV1::Narrowed {
        conversion,
        conversion_sources,
        bound,
    } = view.context
    else {
        panic!("selected direct narrow")
    };
    assert_eq!(conversion.node_id, narrow.node_id);
    assert_eq!(conversion_sources, sources_for(&contract, &narrow.node_id));
    assert_eq!(bound.node.body, bounds_body("0", "10"));
    assert_eq!(bound.sources, sources_for(&contract, &bound.node.node_id));
    let missing_conversion = ContractOccurrenceKeyV1 {
        ordinal: 99,
        ..narrow.clone()
    };
    let missing_result = contract.result_context(
        &add.node_id,
        &add,
        Some(&missing_conversion),
        BoundedDomainForm::IntegerRange,
    );
    let ContractResultViewV1::Lowered(missing_view) = missing_result.expect("missing result")
    else {
        panic!("lowered")
    };
    assert_eq!(
        missing_view.context,
        ContractResultContextV1::IncompleteContext
    );
    let contract_bytes = canonical(&serde_json::to_value(contract.wire()).expect("wire"));
    // Compare the complete frozen context after restoring only the source
    // identity and source regions shifted by the added selected root.
    let mut prior_contract = serde_json::to_value(contract.wire()).expect("contract wire");
    prior_contract["package"]["source"]["package_id"] = prior_wire["package_id"].clone();
    let prior_sources = prior_wire["source_map"]
        .as_array()
        .expect("prior source map");
    for group in ["nodes", "requested_items"] {
        for entry in prior_contract["source_map"][group]
            .as_array_mut()
            .expect("contract source entries")
        {
            for occurrence in entry["occurrences"]
                .as_array_mut()
                .expect("contract occurrences")
            {
                let prior = prior_sources
                    .iter()
                    .find(|source| {
                        source["node_id"] == occurrence["node_id"]
                            && source["role"] == occurrence["role"]
                            && source["ordinal"] == occurrence["ordinal"]
                    })
                    .expect("prior source occurrence");
                occurrence["regions"] = prior["regions"].clone();
            }
        }
    }
    let prior_contract_identity = json!({
        "contract_version": prior_contract["contract_version"],
        "package": prior_contract["package"],
        "semantic_graph": prior_contract["semantic_graph"],
        "feature_accounting": prior_contract["feature_accounting"],
    });
    prior_contract["identity"]["digest"] = json!(sha256_hex(&canonical(&prior_contract_identity)));
    assert_eq!(
        canonical(&prior_contract),
        include_bytes!("../fixtures/ir987/context.json")
    );
    assert_eq!(contract.identity(), &contract.wire().identity);
    let mut low = 0;
    let mut high = CheckedPackageReadLimits::bounded().work;
    while low + 1 < high {
        let middle = low + (high - low) / 2;
        let mut limits = CheckedPackageReadLimits::bounded();
        limits.work = middle;
        match ContractPackageWireV1::read(&contract_bytes, limits, &ContractDependencies::new()) {
            ContractPackageReadResult::Admitted(_) => high = middle,
            ContractPackageReadResult::Incomplete(_) => low = middle,
            other => panic!("baseline contract work: {other:?}"),
        }
    }
    let mut limits = CheckedPackageReadLimits::bounded();
    limits.work = high - 1;
    let ContractPackageReadResult::Incomplete(incomplete) =
        ContractPackageWireV1::read(&contract_bytes, limits, &ContractDependencies::new())
    else {
        panic!("contract work boundary");
    };
    assert_eq!((high, incomplete.consumed), (74, 74));
}
