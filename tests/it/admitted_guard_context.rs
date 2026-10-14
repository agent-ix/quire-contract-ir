// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-450: admitted path guards retain their checked occurrences and polarity.

use crate::support::checked_package::{
    canonical, evidence_for, family_key, fixture_source, mint_ungrouped_structural_keys, node_id,
    refresh_identity, settle, sha256_hex, v2_all_families,
};
use ix_trace_rs::trace;
use quire_contract_model::{
    BoundedDomainForm, CheckedNodeId, CheckedNodeTag, CheckedOccurrenceRole,
    CheckedPackageReadLimits, CheckedPackageV2, CheckedPackageV2ReadResult,
    CompleteLoweringProfileV2, ContractDependencies, ContractDispositionV1,
    ContractGuardContextErrorV1, ContractGuardContextFailureCauseV1, ContractGuardContextViewV1,
    ContractOccurrenceKeyV1, ContractPackageReadResult, ContractPackageV1, ContractPackageWireV1,
    ContractPathGuardV1, ContractResultContextV1, ContractResultViewV1, LoweringProfileRef,
    CONTRACT_GUARD_CONTEXT_MAX_GUARDS,
};
use serde_json::{json, Value};

fn typed_key(digest: &str) -> CheckedNodeId {
    CheckedNodeId {
        domain: "quire.checked-semantic-node/v1".into(),
        digest: digest.into(),
    }
}

fn operation(identity: &str) -> Value {
    json!({
        "identity": identity,
        "laws": [],
        "mode": null,
        "member": null,
        "leaves": [],
    })
}

fn reference(digest: &str) -> Value {
    json!({"term": "reference", "target": node_id(digest)})
}

fn application_node(
    digest: &str,
    form: &str,
    operator: &str,
    operation_value: Value,
    result_type: &str,
    arguments: Vec<Value>,
    dependencies: &[&str],
    occurrences: Value,
) -> Value {
    json!({
        "node_id": node_id(digest),
        "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": "expression",
        "semantic_form": form,
        "semantic_type": node_id(result_type),
        "dependencies": dependencies.iter().map(|key| node_id(key)).collect::<Vec<_>>(),
        "occurrences": occurrences,
        "body": {
            "term": "application",
            "operator": operator,
            "operation": operation_value,
            "result_type": node_id(result_type),
            "arguments": arguments,
        },
    })
}

fn guard_source() -> Value {
    let boolean = family_key("aaaa");
    let integer = family_key("a3a3");
    let mut source = v2_all_families();
    let nodes = source["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("fixture graph");
    let reference_guard = nodes
        .iter_mut()
        .find(|node| node["node_id"]["digest"] == family_key("eeee"))
        .expect("shared Boolean expression");
    reference_guard["body"]["target"] = node_id(&family_key("a4a4"));
    let mut reference_dependencies = [family_key("aaaa"), family_key("a4a4")];
    reference_dependencies.sort();
    reference_guard["dependencies"] = json!(reference_dependencies
        .iter()
        .map(|key| node_id(key))
        .collect::<Vec<_>>());
    reference_guard["occurrences"] = json!([
        {"role": "expression", "ordinal": 0},
        {"role": "expression", "ordinal": 1},
        {"role": "generated", "ordinal": 0},
    ]);
    let boolean_type = nodes
        .iter_mut()
        .find(|node| node["node_id"]["digest"] == boolean)
        .expect("Boolean type");
    boolean_type["occurrences"] = json!([{"role": "expression", "ordinal": 0}]);
    let unrelated_value = nodes
        .iter_mut()
        .find(|node| node["node_id"]["digest"] == family_key("a2a2"))
        .expect("unrelated temporal node");
    unrelated_value["occurrences"] = json!([{"role": "expression", "ordinal": 0}]);

    let nested_guard = sha256_hex(b"ir779 provisional nested guard");
    let application = sha256_hex(b"ir779 provisional application");
    let integer_guard = sha256_hex(b"ir779 provisional integer guard");
    let narrow = sha256_hex(b"ir779 provisional direct narrow");
    nodes.extend([
        application_node(
            &nested_guard,
            "unary",
            "unary",
            operation("quire.op.boolean.not"),
            &boolean,
            vec![json!({"term": "literal", "type": node_id(&boolean),
                "value_kind": "boolean", "value": true})],
            &[],
            json!([{"role": "expression", "ordinal": 0}]),
        ),
        application_node(
            &application,
            "binary",
            "binary",
            operation("quire.op.boolean.and"),
            &boolean,
            vec![
                reference(&family_key("a4a4")),
                reference(&family_key("a4a4")),
            ],
            &[family_key("a4a4").as_str()],
            json!([
                {"role": "expression", "ordinal": 0},
                {"role": "expression", "ordinal": 1},
            ]),
        ),
        application_node(
            &integer_guard,
            "binary",
            "binary",
            operation("quire.op.integer.add"),
            &integer,
            vec![
                json!({"term": "literal", "type": node_id(&integer),
                    "value_kind": "integer", "value": "1"}),
                json!({"term": "literal", "type": node_id(&integer),
                    "value_kind": "integer", "value": "2"}),
            ],
            &[],
            json!([{"role": "expression", "ordinal": 0}]),
        ),
        application_node(
            &narrow,
            "conversion",
            "convert",
            json!({
                "identity": "quire.op.numeric.narrow",
                "laws": [],
                "mode": null,
                "member": {"kind": "type_argument", "type_argument": node_id(&family_key("cccc"))},
                "leaves": [],
            }),
            &family_key("cccc"),
            vec![reference(&integer_guard)],
            &[integer_guard.as_str()],
            json!([{"role": "expression", "ordinal": 0}]),
        ),
    ]);
    mint_ungrouped_structural_keys(&mut source);
    settle(&mut source);
    let source_ref = fixture_source();
    let source_nodes = source["semantic_graph"]["nodes"].as_array().expect("nodes");
    let source_map = source_nodes
        .iter()
        .enumerate()
        .flat_map(|(position, node)| {
            let start = u64::try_from(position).expect("small fixture position");
            node["occurrences"]
                .as_array()
                .expect("occurrences")
                .iter()
                .map({
                    let source_ref = source_ref.clone();
                    move |occurrence| {
                        json!({
                            "node_id": node["node_id"],
                            "role": occurrence["role"],
                            "ordinal": occurrence["ordinal"],
                            "regions": [{"source": source_ref,
                                "start": start, "end": start + 1}],
                        })
                    }
                })
        })
        .collect::<Vec<_>>();
    source["source_map"] = json!(source_map);
    refresh_identity(&mut source);
    source
}

fn admit_source(value: &Value) -> CheckedPackageV2 {
    match CheckedPackageV2::read(
        &canonical(value),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(value),
    ) {
        CheckedPackageV2ReadResult::Admitted(package) => *package,
        other => panic!("FR-322 guard fixture must admit: {other:?}"),
    }
}

fn profile() -> CompleteLoweringProfileV2 {
    CompleteLoweringProfileV2 {
        reference: LoweringProfileRef {
            authority: "quire".into(),
            identity: "guard-context".into(),
        },
        supported_tags: CheckedNodeTag::ALL.iter().copied().collect(),
        require_bounds: false,
        work_limit: u64::MAX,
    }
}

fn package(source: &CheckedPackageV2, roots: &[CheckedNodeId]) -> ContractPackageV1 {
    let lowered = source.lower(roots, &profile());
    match ContractPackageWireV1::read(
        lowered.package.canonical_bytes().expect("FR-330 bytes"),
        CheckedPackageReadLimits::bounded(),
        &ContractDependencies::new(),
    ) {
        ContractPackageReadResult::Admitted(package) => *package,
        other => panic!("FR-330 guard package must admit: {other:?}"),
    }
}

fn node_with_operation<'a>(
    package: &'a CheckedPackageV2,
    identity: &str,
) -> &'a quire_contract_model::CheckedSemanticNodeV2 {
    package
        .graph()
        .nodes
        .iter()
        .find(|node| {
            node.body
                .pointer("/operation/identity")
                .and_then(Value::as_str)
                == Some(identity)
        })
        .expect("fixture operation")
}

fn reference_guard_node(
    package: &CheckedPackageV2,
) -> &quire_contract_model::CheckedSemanticNodeV2 {
    package
        .graph()
        .nodes
        .iter()
        .find(|node| {
            node.node_tag.as_ref() == "expression"
                && node.semantic_form.as_ref() == "reference"
                && node.body.pointer("/target/digest").and_then(Value::as_str)
                    == Some(family_key("a4a4").as_str())
        })
        .expect("shared reference guard")
}

fn occurrence(
    node: &quire_contract_model::CheckedSemanticNodeV2,
    ordinal: u64,
) -> ContractOccurrenceKeyV1 {
    ContractOccurrenceKeyV1 {
        node_id: node.node_id.clone(),
        role: CheckedOccurrenceRole::Expression,
        ordinal,
    }
}

fn lowered<'a>(
    package: &'a ContractPackageV1,
    expression: &CheckedNodeId,
    application: &ContractOccurrenceKeyV1,
    guards: &[ContractPathGuardV1],
) -> Box<quire_contract_model::ContractLoweredGuardContextV1<'a>> {
    match package
        .guard_context(expression, application, guards)
        .expect("admitted guard query")
    {
        ContractGuardContextViewV1::Lowered(context) => context,
        other => panic!("application must be lowered: {other:?}"),
    }
}

fn path_guard(occurrence: ContractOccurrenceKeyV1, holds: bool) -> ContractPathGuardV1 {
    ContractPathGuardV1 { occurrence, holds }
}

/// Trace: FR-050-AC-1, FR-050-AC-2, FR-050-AC-5
#[trace("TC-450", "FR-050-AC-1", "FR-050-AC-2", "FR-050-AC-5")]
#[test]
fn tc_450_guard_context_preserves_order_polarity_identity_and_root_accounting() {
    let source = admit_source(&guard_source());
    let app = node_with_operation(&source, "quire.op.boolean.and");
    let integer_expression = node_with_operation(&source, "quire.op.integer.add");
    let direct_narrow = node_with_operation(&source, "quire.op.numeric.narrow");
    let shared_guard = reference_guard_node(&source);
    let nested_guard = node_with_operation(&source, "quire.op.boolean.not");
    let app_id = app.node_id.clone();
    let app_first = occurrence(app, 0);
    let app_else = occurrence(app, 1);
    let guard_outer = occurrence(shared_guard, 0);
    let guard_inner = occurrence(nested_guard, 0);
    let guard_repeat = occurrence(shared_guard, 1);
    let integer_occurrence = occurrence(integer_expression, 0);
    let narrow_occurrence = occurrence(direct_narrow, 0);
    let application_only = package(&source, std::slice::from_ref(&app_id));
    let absent_application = typed_key(&sha256_hex(b"ir779 absent application"));
    assert!(matches!(
        application_only.guard_context(&absent_application, &app_first, &[]),
        Ok(ContractGuardContextViewV1::Absent)
    ));
    let mut unsupported_application_profile = profile();
    unsupported_application_profile
        .supported_tags
        .remove(&CheckedNodeTag::Expression);
    let unsupported_application_lowering = source.lower(
        std::slice::from_ref(&app_id),
        &unsupported_application_profile,
    );
    let unsupported_application = match ContractPackageWireV1::read(
        unsupported_application_lowering
            .package
            .canonical_bytes()
            .expect("bytes"),
        CheckedPackageReadLimits::bounded(),
        &ContractDependencies::new(),
    ) {
        ContractPackageReadResult::Admitted(package) => *package,
        other => panic!("non-lowered application record admits: {other:?}"),
    };
    let ContractGuardContextViewV1::NonLowered(application_record) = unsupported_application
        .guard_context(&app_id, &app_first, &[])
        .expect("non-lowered application disposition")
    else {
        panic!("unsupported application retains its FR-035 disposition");
    };
    assert_eq!(
        application_record.disposition(),
        ContractDispositionV1::Unsupported
    );
    let plain_result_roots = package(
        &source,
        &[
            app_id.clone(),
            integer_expression.node_id.clone(),
            direct_narrow.node_id.clone(),
        ],
    );
    let contextual = package(
        &source,
        &[
            app_id.clone(),
            integer_expression.node_id.clone(),
            direct_narrow.node_id.clone(),
            shared_guard.node_id.clone(),
            nested_guard.node_id.clone(),
        ],
    );
    assert!(contextual.wire().feature_accounting.iter().any(|record| {
        record.node_id() == &guard_outer.node_id
            && record.disposition() == ContractDispositionV1::Lowered
    }));
    assert!(contextual.wire().feature_accounting.iter().any(|record| {
        record.node_id() == &guard_inner.node_id
            && record.disposition() == ContractDispositionV1::Lowered
    }));
    assert_ne!(plain_result_roots.identity(), contextual.identity());
    assert_eq!(
        serde_json::to_value(contextual.wire())
            .expect("wire fields")
            .as_object()
            .expect("closed wire object")
            .len(),
        6
    );
    assert!(matches!(
        application_only.guard_context(
            &app_id,
            &app_first,
            &[path_guard(guard_outer.clone(), true)]
        ),
        Err(ContractGuardContextErrorV1::Guard(failure))
            if failure.position == 0
                && failure.cause == ContractGuardContextFailureCauseV1::AbsentRoot
                && failure.occurrence == guard_outer
    ));

    let before_wire = contextual.wire().clone();
    let before_closure = contextual
        .reached_closure(&app_id)
        .expect("application closure")
        .clone();
    let then_guards = [
        path_guard(guard_outer.clone(), true),
        path_guard(guard_inner.clone(), true),
        path_guard(guard_repeat.clone(), false),
    ];
    let then_view = lowered(&contextual, &app_id, &app_first, &then_guards);
    assert_eq!(then_view.application, app_first);
    assert_eq!(then_view.provenance, &contextual.wire().package);
    assert_eq!(then_view.identity, contextual.identity());
    assert_eq!(then_view.guards.len(), 3);
    assert_eq!(
        then_view
            .guards
            .iter()
            .map(|guard| (guard.occurrence.clone(), guard.holds))
            .collect::<Vec<_>>(),
        vec![
            (guard_outer.clone(), true),
            (guard_inner.clone(), true),
            (guard_repeat.clone(), false),
        ]
    );
    for guard in &then_view.guards {
        assert_eq!(guard.expression.node_id, guard.occurrence.node_id);
        assert!(guard.source.node_id == guard.occurrence.node_id);
        assert_eq!(guard.source.role, CheckedOccurrenceRole::Expression);
        assert_eq!(guard.source.ordinal, guard.occurrence.ordinal);
        assert!(guard.selected_references.is_empty());
    }

    let else_guards = [path_guard(guard_inner.clone(), false)];
    let else_view = lowered(&contextual, &app_id, &app_else, &else_guards);
    assert_eq!(else_view.application.ordinal, 1);
    assert_eq!(else_view.guards.len(), 1);
    assert!(!else_view.guards[0].holds);
    let unguarded = lowered(&contextual, &app_id, &app_first, &[]);
    assert!(unguarded.guards.is_empty());
    let excessive_guards = std::iter::repeat_with(|| path_guard(guard_outer.clone(), true))
        .take(CONTRACT_GUARD_CONTEXT_MAX_GUARDS + 1)
        .collect::<Vec<_>>();
    assert!(matches!(
        contextual.guard_context(&app_id, &app_first, &excessive_guards),
        Err(ContractGuardContextErrorV1::GuardLimitExceeded {
            requested,
            limit: CONTRACT_GUARD_CONTEXT_MAX_GUARDS,
        }) if requested == CONTRACT_GUARD_CONTEXT_MAX_GUARDS + 1
    ));
    let result_without_guards = match plain_result_roots
        .result_context(
            &integer_expression.node_id,
            &integer_occurrence,
            Some(&narrow_occurrence),
            BoundedDomainForm::IntegerRange,
        )
        .expect("FR-049 query without guard roots")
    {
        ContractResultViewV1::Lowered(result) => result.context.clone(),
        other => panic!("integer result root is lowered: {other:?}"),
    };
    let result_with_guards = match contextual
        .result_context(
            &integer_expression.node_id,
            &integer_occurrence,
            Some(&narrow_occurrence),
            BoundedDomainForm::IntegerRange,
        )
        .expect("FR-049 query with guard roots")
    {
        ContractResultViewV1::Lowered(result) => result.context.clone(),
        other => panic!("integer result root is lowered: {other:?}"),
    };
    assert_eq!(result_with_guards, result_without_guards);
    assert!(matches!(
        result_with_guards,
        ContractResultContextV1::Narrowed { .. }
    ));
    assert_eq!(contextual.wire(), &before_wire);
    assert_eq!(
        contextual.reached_closure(&app_id).expect("same closure"),
        before_closure
    );
}

/// Trace: FR-050-AC-3, FR-050-AC-4
#[trace("TC-450", "FR-050-AC-3", "FR-050-AC-4")]
#[test]
fn tc_450_guard_context_refuses_first_bad_occurrence_without_a_prefix() {
    let source_value = guard_source();
    let source = admit_source(&source_value);
    let app = node_with_operation(&source, "quire.op.boolean.and");
    let reference_guard = reference_guard_node(&source);
    let nested_guard = node_with_operation(&source, "quire.op.boolean.not");
    let integer_guard = node_with_operation(&source, "quire.op.integer.add");
    let non_expression = source
        .graph()
        .nodes
        .iter()
        .find(|node| node.node_id.digest.as_ref() == family_key("aaaa"))
        .expect("Boolean scalar node");
    let app_id = app.node_id.clone();
    let app_occurrence = occurrence(app, 0);
    let good = occurrence(reference_guard, 0);
    let context = package(
        &source,
        &[
            app_id.clone(),
            reference_guard.node_id.clone(),
            nested_guard.node_id.clone(),
            integer_guard.node_id.clone(),
            non_expression.node_id.clone(),
        ],
    );

    let absent = ContractOccurrenceKeyV1 {
        node_id: typed_key(&sha256_hex(b"ir779 absent guard")),
        role: CheckedOccurrenceRole::Expression,
        ordinal: 0,
    };
    let wrong_role = ContractOccurrenceKeyV1 {
        role: CheckedOccurrenceRole::Generated,
        ..good.clone()
    };
    let stale_ordinal = ContractOccurrenceKeyV1 {
        ordinal: 99,
        ..good.clone()
    };
    let non_boolean = occurrence(integer_guard, 0);
    let wrong_form = occurrence(non_expression, 0);
    for (occurrence, cause) in [
        (absent, ContractGuardContextFailureCauseV1::AbsentRoot),
        (
            wrong_role,
            ContractGuardContextFailureCauseV1::WrongOccurrenceRole,
        ),
        (
            stale_ordinal,
            ContractGuardContextFailureCauseV1::StaleOccurrenceOrdinal,
        ),
        (
            non_boolean,
            ContractGuardContextFailureCauseV1::NonBooleanExpression,
        ),
        (
            wrong_form,
            ContractGuardContextFailureCauseV1::WrongNodeForm,
        ),
    ] {
        let result = context.guard_context(
            &app_id,
            &app_occurrence,
            &[
                path_guard(good.clone(), true),
                path_guard(occurrence.clone(), false),
            ],
        );
        assert!(matches!(
            result,
            Err(ContractGuardContextErrorV1::Guard(failure))
                if failure.position == 1
                    && failure.occurrence == occurrence
                    && failure.cause == cause
        ));
    }

    let mut missing_source = context.clone();
    assert!(missing_source.fault_inject_missing_source_for_test(&good.node_id));
    assert!(matches!(
        missing_source.guard_context(
            &app_id,
            &app_occurrence,
            &[path_guard(good.clone(), true)]
        ),
        Err(ContractGuardContextErrorV1::Guard(failure))
            if failure.position == 0
                && failure.cause == ContractGuardContextFailureCauseV1::MissingSourceMapEntry
                && failure.source.is_none()
    ));

    let mut inconsistent_body = context.clone();
    assert!(inconsistent_body.fault_inject_invalid_body_for_test(&nested_guard.node_id));
    let invalid_body_result = inconsistent_body.guard_context(
        &app_id,
        &app_occurrence,
        &[
            path_guard(good.clone(), true),
            path_guard(occurrence(nested_guard, 0), false),
        ],
    );
    assert!(
        matches!(
            &invalid_body_result,
            Err(ContractGuardContextErrorV1::Guard(failure))
                if failure.position == 1
                    && matches!(failure.cause,
                        ContractGuardContextFailureCauseV1::InvalidBody(_))
        ),
        "{invalid_body_result:?}"
    );

    let mut unsupported_profile = profile();
    unsupported_profile
        .supported_tags
        .remove(&CheckedNodeTag::Temporal);
    let unsupported_key = family_key("a2a2");
    let unsupported_lowering = source.lower(
        &[app_id.clone(), typed_key(&unsupported_key)],
        &unsupported_profile,
    );
    let unsupported_package = match ContractPackageWireV1::read(
        unsupported_lowering
            .package
            .canonical_bytes()
            .expect("bytes"),
        CheckedPackageReadLimits::bounded(),
        &ContractDependencies::new(),
    ) {
        ContractPackageReadResult::Admitted(package) => *package,
        other => panic!("unsupported root package admits: {other:?}"),
    };
    let unsupported_guard = ContractOccurrenceKeyV1 {
        node_id: typed_key(&unsupported_key),
        role: CheckedOccurrenceRole::Expression,
        ordinal: 0,
    };
    let unsupported_result = unsupported_package.guard_context(
        &app_id,
        &app_occurrence,
        &[path_guard(unsupported_guard.clone(), true)],
    );
    assert!(
        matches!(
            &unsupported_result,
            Err(ContractGuardContextErrorV1::Guard(failure))
                if failure.position == 0
                    && failure.occurrence == unsupported_guard
                    && failure.cause == ContractGuardContextFailureCauseV1::NonLoweredRoot {
                        disposition: ContractDispositionV1::Unsupported
                    }
        ),
        "{unsupported_result:?}"
    );
}

/// Trace: FR-050-AC-3, FR-050-AC-4
#[trace("TC-450", "FR-050-AC-3", "FR-050-AC-4")]
#[test]
fn tc_450_unrepresented_guard_key_cannot_gain_authority_from_its_body_position() {
    let source = admit_source(&guard_source());
    let app = node_with_operation(&source, "quire.op.boolean.and");
    let represented_parameter = source
        .graph()
        .nodes
        .iter()
        .find(|node| node.node_id.digest.as_ref() == family_key("a4a4"))
        .expect("parameter in the application's ordinary forward closure");
    let app_id = app.node_id.clone();
    let app_occurrence = occurrence(app, 0);
    let guard_occurrence = occurrence(represented_parameter, 0);
    let application_only = package(&source, std::slice::from_ref(&app_id));
    assert!(application_only
        .wire()
        .semantic_graph
        .nodes
        .iter()
        .any(|node| node.node_id == guard_occurrence.node_id));
    assert!(matches!(
        application_only.guard_context(
            &app_id,
            &app_occurrence,
            &[path_guard(guard_occurrence.clone(), true)]
        ),
        Err(ContractGuardContextErrorV1::Guard(failure))
            if failure.cause == ContractGuardContextFailureCauseV1::AbsentRoot
                && failure.occurrence == guard_occurrence
    ));

    let contextual = package(&source, &[app_id.clone(), guard_occurrence.node_id.clone()]);
    let plausible_but_unrepresented = ContractOccurrenceKeyV1 {
        node_id: typed_key(&sha256_hex(b"ir779 plausible caller key")),
        role: CheckedOccurrenceRole::Expression,
        ordinal: 0,
    };
    assert!(matches!(
        contextual.guard_context(
            &app_id,
            &app_occurrence,
            &[path_guard(plausible_but_unrepresented.clone(), true)]
        ),
        Err(ContractGuardContextErrorV1::Guard(failure))
            if failure.cause == ContractGuardContextFailureCauseV1::AbsentRoot
                && failure.occurrence == plausible_but_unrepresented
    ));
}

/// Trace: FR-050-AC-3
#[trace("TC-450", "FR-050-AC-3")]
#[test]
fn tc_450_reader_refuses_a_lowered_guard_root_without_its_node() {
    let source = admit_source(&guard_source());
    let app = node_with_operation(&source, "quire.op.boolean.and");
    let guard = reference_guard_node(&source);
    let app_id = app.node_id.clone();
    let guard_id = guard.node_id.clone();
    let admitted = package(&source, &[app_id.clone(), guard_id.clone()]);
    let mut malformed = serde_json::to_value(admitted.wire()).expect("wire JSON");
    malformed["semantic_graph"]["nodes"]
        .as_array_mut()
        .expect("nodes")
        .retain(|node| node["node_id"] != serde_json::to_value(&guard_id).expect("guard key"));
    malformed["source_map"]["nodes"]
        .as_array_mut()
        .expect("source nodes")
        .retain(|node| node["node_id"] != serde_json::to_value(&guard_id).expect("guard key"));
    let requested = malformed["source_map"]["requested_items"]
        .as_array_mut()
        .expect("requested items");
    if let Some(entry) = requested.iter_mut().find(|entry| {
        entry["requested_item"] == serde_json::to_value(&guard_id).expect("guard key")
    }) {
        entry["occurrences"] = json!([]);
    }
    let projection = json!({
        "contract_version": malformed["contract_version"],
        "package": malformed["package"],
        "semantic_graph": malformed["semantic_graph"],
        "feature_accounting": malformed["feature_accounting"],
    });
    malformed["identity"]["digest"] = json!(sha256_hex(&canonical(&projection)));
    let outcome = ContractPackageWireV1::read(
        &canonical(&malformed),
        CheckedPackageReadLimits::bounded(),
        &ContractDependencies::new(),
    );
    assert!(matches!(&outcome, ContractPackageReadResult::Refused(_)));
    assert!(!matches!(&outcome, ContractPackageReadResult::Admitted(_)));
}
